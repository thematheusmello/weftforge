"""Evaluate real JSON receipts against pre-frozen original authored expectations."""
from copy import deepcopy
from decimal import getcontext
from pathlib import Path
import hashlib
import json
import math
import struct
import sys

from oracle import ABS_BUDGET, D, Geometry, ONE, Path as SewingPath, ZERO, dec, interpolate

ROOT = Path(__file__).resolve().parent
getcontext().prec = 60


def load(path):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError(f"duplicate JSON field {key}")
            result[key] = value
        return result
    return json.loads(Path(path).read_text(), object_pairs_hook=pairs,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))


def same(expected, actual, path="source"):
    if isinstance(expected, dict):
        assert isinstance(actual, dict) and expected.keys() == actual.keys(), f"{path}: field set changed"
        for key in expected:
            same(expected[key], actual[key], f"{path}.{key}")
    elif isinstance(expected, list):
        assert isinstance(actual, list) and len(expected) == len(actual), f"{path}: list/order changed"
        for index, (a, b) in enumerate(zip(expected, actual)):
            same(a, b, f"{path}[{index}]")
    elif isinstance(expected, float):
        assert isinstance(actual, (int, float)) and not isinstance(actual, bool), f"{path}: numeric type changed"
        assert struct.pack(">d", expected) == struct.pack(">d", float(actual)), f"{path}: f64 bits changed"
    else:
        assert type(expected) is type(actual) and expected == actual, f"{path}: value changed"


def distance(a, b):
    return sum(((x - y) ** 2 for x, y in zip(a, b)), ZERO).sqrt()


def finite(value):
    assert isinstance(value, (float, int)) and not isinstance(value, bool), "expected a numeric field"
    assert math.isfinite(value), "nonfinite derived value"
    return dec(value)


def owner_for(path, interval):
    t0, t1 = finite(interval["t0"]), finite(interval["t1"])
    assert t0 != t1, "zero derived portion"
    middle = (t0 + t1) / 2
    choices = []
    for index, authored in enumerate(path.intervals):
        a, b = dec(authored["t0"]), dec(authored["t1"])
        if authored["curve"] == interval["curve"] and min(a, b) < middle < max(a, b):
            if (t1 - t0) * (b - a) > ZERO:
                assert min(a, b) - D("1e-12") <= min(t0, t1) <= max(t0, t1) <= max(a, b) + D("1e-12"), "portion crosses a selected span"
                choices.append(index)
    assert len(choices) == 1, "portion has no unique authored owner/direction"
    return choices[0]


def check_group(geometry, authored, actual, frozen):
    assert actual["id"] == authored["id"], "group identity changed"
    a, b = SewingPath(geometry, authored["a"]), SewingPath(geometry, authored["b"])
    for expected, observed in zip((a.total, b.total), actual["lengths"]):
        assert abs(expected - finite(observed)) <= ABS_BUDGET, "authored total length outside 1e-8 m"
    assert len(actual["lengths"]) == 2, "expected two side lengths"
    required = [D(item["u"][0]) for item in frozen["independent_details"]] + [ONE]
    cursor = ZERO
    endpoints = {"a": set(), "b": set()}
    max_error = ZERO
    for portion in actual["portions"]:
        ai, bi = owner_for(a, portion["a"]), owner_for(b, portion["b"])
        us = [a.fraction(ai, finite(portion["a"][f"t{k}"])) for k in (0, 1)]
        vs = [b.fraction(bi, finite(portion["b"][f"t{k}"])) for k in (0, 1)]
        assert us[1] > us[0] and vs[1] > vs[0], "portion order is reversed"
        assert abs(us[0] - cursor) * a.total <= ABS_BUDGET, "portion gap, duplicate or order change"
        assert us[0] >= -ABS_BUDGET / a.total and us[1] <= ONE + ABS_BUDGET / a.total, "portion outside path"
        for u, v in zip(us, vs):
            error = abs(v - interpolate(authored["correspondence"], u)) * b.total
            max_error = max(max_error, error)
            assert error <= ABS_BUDGET, "portion violates authored arc-length correspondence"
        for knot in required:
            assert not (us[0] + ABS_BUDGET / a.total < knot < us[1] - ABS_BUDGET / a.total), "portion crosses a required A/B/map cell knot"
        for label, path, index, fractions in (("a", a, ai, us), ("b", b, bi, vs)):
            for k, fraction in enumerate(fractions):
                for end in (0, 1):
                    if abs(fraction - path.knots[index + end]) * path.total < D("1e-12"):
                        expected_t = path.intervals[index][f"t{end}"]
                        same(expected_t, portion[label][f"t{k}"], "exact stored selected endpoint")
                        endpoints[label].add((index, end))
        cursor = us[1]
    assert abs(cursor - ONE) * a.total <= ABS_BUDGET, "missing final path coverage"
    for label, path in (("a", a), ("b", b)):
        assert endpoints[label] == {(i, k) for i in range(len(path.intervals)) for k in (0, 1)}, "lost one-sided disconnected endpoint"
    return {"group": authored["id"], "portions": len(actual["portions"]),
            "maximum_portion_pair_error_m": str(max_error)}


def weighted_options(geometry, mesh, weighted):
    ids = weighted["vertices"]
    assert len(ids) == 2 and all(isinstance(v, int) and 0 <= v < len(mesh["positions"]) for v in ids), "invalid stitch vertex"
    weight = finite(weighted["weight"])
    assert ZERO <= weight <= ONE, "stitch weight outside [0,1]"
    records = [[r for r in mesh["boundary"] if r["vertex"] == vertex] for vertex in ids]
    choices = []
    for left in records[0]:
        for right in records[1]:
            if (left["curve"], left["piece"]) != (right["curve"], right["piece"]):
                continue
            t = (ONE - weight) * finite(left["t"]) + weight * finite(right["t"])
            rest = tuple((ONE - weight) * finite(x) + weight * finite(y)
                         for x, y in zip(mesh["positions"][ids[0]], mesh["positions"][ids[1]]))
            choices.append({"curve": left["curve"], "t": t, "rest": rest})
    assert choices, "weighted stitch invents a connector across spans/curves"
    return choices


def owning_options(path, options):
    result = []
    for option in options:
        for index, interval in enumerate(path.intervals):
            if option["curve"] != interval["curve"]:
                continue
            t = option["t"]
            low, high = sorted((dec(interval["t0"]), dec(interval["t1"])))
            if low - D("1e-12") <= t <= high + D("1e-12"):
                result.append((index, path.fraction(index, t), option))
    return result


def check_mesh(geometry, source, mesh, frozen_groups):
    assert mesh["doc_revision"] == source["project"]["revision"], "stale mesh installed"
    curves_by_piece = {c: p["id"] for p in source["project"]["pieces"] for c in p["boundary"]}
    for record in mesh["boundary"]:
        assert record["curve"] in geometry.curves and curves_by_piece[record["curve"]] == record["piece"], "bad mesh source provenance"
        t = finite(record["t"])
        assert ZERO <= t <= ONE, "mesh boundary parameter outside curve"
        point = geometry.point(record["curve"], t) + (ZERO,)
        actual = tuple(map(finite, mesh["positions"][record["vertex"]]))
        assert len(actual) == 3 and distance(point, actual) <= ABS_BUDGET, "boundary vertex is not on its authored source"
    groups = {g["id"]: g for g in source["project"]["sewing_groups"]}
    seams = {g["id"]: g for g in source["project"]["seams"]}
    paths = {key: (SewingPath(geometry, g["a"]), SewingPath(geometry, g["b"])) for key, g in groups.items()}
    paired = {key: [] for key in groups}
    legacy_seen = {key: 0 for key in seams}
    max_chord, max_pair = ZERO, ZERO
    for stitch in mesh["stitches"]:
        group_id = stitch["seam"]
        ao, bo = (weighted_options(geometry, mesh, stitch[label]) for label in ("a", "b"))
        if group_id in groups:
            g = groups[group_id]
            a, b = paths[group_id]
            matches = []
            for ai, u, av in owning_options(a, ao):
                for bi, v, bv in owning_options(b, bo):
                    error = abs(v - interpolate(g["correspondence"], u)) * b.total
                    if error <= ABS_BUDGET:
                        matches.append((ai, bi, u, v, av, bv, error))
            assert matches, "mesh stitch does not implement authored arc-length pairing"
            for ai, bi, u, v, av, bv, error in matches:
                paired[group_id].append((ai, bi, av["t"], bv["t"]))
                max_pair = max(max_pair, error)
                for option in (av, bv):
                    authored = geometry.point(option["curve"], option["t"]) + (ZERO,)
                    max_chord = max(max_chord, distance(authored, option["rest"]))
        else:
            assert group_id in seams, "stitch identity is neither group nor legacy seam"
            seam = seams[group_id]
            matches = []
            for av in ao:
                for bv in bo:
                    if av["curve"] != seam["a"]["curve"] or bv["curve"] != seam["b"]["curve"]:
                        continue
                    u = (av["t"] - dec(seam["a"]["t0"])) / (dec(seam["a"]["t1"]) - dec(seam["a"]["t0"]))
                    v = (bv["t"] - dec(seam["b"]["t0"])) / (dec(seam["b"]["t1"]) - dec(seam["b"]["t0"]))
                    if -D("1e-12") <= u <= ONE + D("1e-12") and abs(v - interpolate(seam["correspondence"], u)) <= D("1e-11"):
                        matches.append((u, v))
            assert matches, "legacy seam was reinterpreted as arc length"
            legacy_seen[group_id] += 1
    for group_id, g in groups.items():
        assert paired[group_id], "group produces no real stitches"
        expected = frozen_groups[group_id]
        for detail in expected["independent_details"]:
            ai, bi = detail["owners"]
            for end in (0, 1):
                ta, tb = D(detail["a_t"][end]), D(detail["b_t"][end])
                ap = geometry.point(g["a"][ai]["curve"], ta)
                bp = geometry.point(g["b"][bi]["curve"], tb)
                assert any(aindex == ai and bindex == bi
                           and distance(geometry.point(g["a"][ai]["curve"], at), ap) <= ABS_BUDGET
                           and distance(geometry.point(g["b"][bi]["curve"], bt), bp) <= ABS_BUDGET
                           for aindex, bindex, at, bt in paired[group_id]), "mesh omitted a required cell endpoint/one-sided location"
        for label, path in (("a", paths[group_id][0]), ("b", paths[group_id][1])):
            for index, interval in enumerate(path.intervals):
                low, high = sorted((dec(interval["t0"]), dec(interval["t1"])))
                for sample in mesh["boundary"]:
                    if sample["curve"] != interval["curve"] or not low <= finite(sample["t"]) <= high:
                        continue
                    target = geometry.point(interval["curve"], finite(sample["t"]))
                    slot, owner_slot = (2, 0) if label == "a" else (3, 1)
                    assert any(pair[owner_slot] == index and distance(geometry.point(interval["curve"], pair[slot]), target) <= ABS_BUDGET
                               for pair in paired[group_id]), "mesh omitted a relevant boundary sample"
    assert all(count >= 3 for count in legacy_seen.values()), "legacy runtime metric needs interior mesh witnesses"
    return {"stitches": len(mesh["stitches"]), "maximum_pair_error_m": str(max_pair),
            "maximum_weighted_rest_chord_deviation_m": str(max_chord),
            "legacy_stitches": legacy_seen}


def inspect(case, receipt, require_mesh=True):
    expected = load(ROOT / "expectations.json")["cases"][case]
    assert receipt["kind"] == "sewing_inspection", "not an actual inspect-sewing receipt"
    same(expected["source"], receipt["source"])
    geometry = Geometry(expected["source"])
    frozen = {g["id"]: g for g in expected["sewing_groups"]}
    actual = receipt["sewing_groups"]
    authored = expected["source"]["project"]["sewing_groups"]
    assert [g["id"] for g in actual] == [g["id"] for g in authored], "group list identity/order changed"
    checks = [check_group(geometry, g, a, frozen[g["id"]]) for g, a in zip(authored, actual)]
    mesh = check_mesh(geometry, expected["source"], receipt["mesh"], frozen) if "mesh" in receipt else None
    assert not require_mesh or mesh is not None, "missing actual rest-mesh capture; acceptance incomplete"
    return {"case": case, "status": "pass", "authoring": checks, "mesh": mesh,
            "reference_parity": "unknown", "native_gesture_history_reopen": "unknown",
            "physics_quality": "unverified"}


def self_check():
    expectations = load(ROOT / "expectations.json")["cases"]
    witnesses = []
    for case, expected in expectations.items():
        receipt = {"kind": "sewing_inspection", "source": deepcopy(expected["source"]),
                   "sewing_groups": [{k: deepcopy(g[k]) for k in ("id", "lengths", "portions")} for g in expected["sewing_groups"]]}
        inspect(case, receipt, require_mesh=False)
        def reject(name, mutate):
            bad = deepcopy(receipt)
            mutate(bad)
            try:
                inspect(case, bad, require_mesh=False)
            except (AssertionError, ValueError, KeyError, ArithmeticError):
                witnesses.append({"case": case, "mutation": name, "rejected": True})
            else:
                raise AssertionError(f"negative witness escaped: {case}/{name}")
        reject("revision-corruption", lambda r: r["source"]["project"].update({"revision": r["source"]["project"]["revision"] + 1}))
        reject("f64-corruption", lambda r: r["source"]["project"]["points"][0]["position"].__setitem__(0, math.nextafter(r["source"]["project"]["points"][0]["position"][0], 1.0)))
        if receipt["sewing_groups"]:
            reject("lost-disconnected-or-map-cell", lambda r: r["sewing_groups"][0]["portions"].pop(0))
            reject("wrong-span-order", lambda r: r["sewing_groups"][0]["portions"].reverse())
            reject("wrong-direction", lambda r: r["sewing_groups"][0]["portions"][0]["a"].update({"t0": r["sewing_groups"][0]["portions"][0]["a"]["t1"], "t1": r["sewing_groups"][0]["portions"][0]["a"]["t0"]}))
            reject("wrong-total-metric", lambda r: r["sewing_groups"][0]["lengths"].__setitem__(0, r["sewing_groups"][0]["lengths"][0] + 1e-5))
            reject("artificial-source-bridge", lambda r: r["source"]["project"]["sewing_groups"][0]["a"].append({"curve": 7, "t0": 0.0, "t1": 1.0}))
        if case.startswith("collinear"):
            group = expected["source"]["project"]["sewing_groups"][0]
            def parameter_shortcut(r):
                for portion, detail in zip(r["sewing_groups"][0]["portions"], expected["sewing_groups"][0]["independent_details"]):
                    for k in (0, 1):
                        portion["a"][f"t{k}"] = group["a"][0]["t0"] + (group["a"][0]["t1"] - group["a"][0]["t0"]) * float(detail["u"][k])
            reject("collinear-t-equals-length-shortcut", parameter_shortcut)
        if case.startswith("legacy"):
            reject("legacy-map-reinterpretation-in-source", lambda r: r["source"]["project"]["seams"][0].update({"correspondence": [[0.0, 0.0], [1.0, 1.0]]}))
            reject("migration-drops-seam", lambda r: r["source"]["project"].update({"seams": []}))
    source = expectations["line-identity-coarse"]["source"]
    geometry = Geometry(source)
    mesh = {"doc_revision": source["project"]["revision"],
            "positions": [[0.0, 0.0, 0.0], [0.4, 0.0, 0.0], [1.0, 1.0, 0.0]],
            "boundary": [{"curve": 7, "piece": 2, "t": 0.0, "vertex": 0},
                         {"curve": 7, "piece": 2, "t": 1.0, "vertex": 1},
                         {"curve": 16, "piece": 11, "t": 0.0, "vertex": 2}], "stitches": []}
    option = weighted_options(geometry, mesh, {"vertices": [0, 1], "weight": 0.25})[0]
    assert option["curve"] == 7 and option["t"] == D("0.25")
    assert distance(option["rest"], geometry.point(7, D("0.25")) + (ZERO,)) < D("1e-30")
    for name, action in (
        ("weighted-artificial-bridge", lambda: weighted_options(geometry, mesh, {"vertices": [0, 2], "weight": 0.5})),
        ("missing-real-constraints", lambda: check_mesh(geometry, source, mesh, {g["id"]: g for g in expectations["line-identity-coarse"]["sewing_groups"]})),
        ("stale-rest-mesh", lambda: check_mesh(geometry, source, {**mesh, "doc_revision": 0}, {})),
    ):
        try:
            action()
        except AssertionError:
            witnesses.append({"case": "mesh-mechanics", "mutation": name, "rejected": True})
        else:
            raise AssertionError(f"negative witness escaped: {name}")
    return {"kind": "oracle_self_check", "candidate_outputs_inspected": False,
            "synthetic_positive_receipts": len(expectations), "negative_witnesses": witnesses,
            "claim": "evaluator mechanics only; no production acceptance or reference parity"}


def main():
    if sys.argv[1] == "self-check":
        result, output = self_check(), Path(sys.argv[2])
    elif sys.argv[1] == "inspect":
        case, path, output = sys.argv[2], Path(sys.argv[3]), Path(sys.argv[4])
        try:
            result = inspect(case, load(path))
        except (AssertionError, ValueError, KeyError, ArithmeticError, IndexError, TypeError) as error:
            result = {"case": case, "status": "fail", "reason": str(error), "reference_parity": "unknown"}
        result["receipt_sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
    else:
        raise SystemExit("usage: evaluate.py self-check OUT | inspect CASE ACTUAL_RECEIPT OUT")
    with output.open("x") as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
        stream.write("\n")
    print(json.dumps({"output": output.name, "status": result.get("status", "self-check")}))
    if result.get("status") == "fail":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
