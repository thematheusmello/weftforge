"""Independent source/metric checks for declared shared-command operations."""
from copy import deepcopy
from pathlib import Path
import hashlib
import json
import re
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from evaluate import check_group, check_mesh, distance, load, same
from oracle import ABS_BUDGET, D, Geometry, ONE, Path as SewingPath, ZERO, dec, group_expectation, interpolate


def source_check(expected, actual, name):
    adjusted = deepcopy(actual)
    rewritten = {"split", "joined", "undo_join", "rejected_overlap", "redo_join", "remeshed"}
    if name in rewritten:
        eg = expected["project"]["sewing_groups"][0]
        ag = adjusted["project"]["sewing_groups"][0]
        assert len(ag["a"]) == len(eg["a"]), "split/join lost an authored span boundary"
        for ai, ei in zip(ag["a"][1:], eg["a"][1:]):
            for key in ("t0", "t1"):
                assert abs(dec(ai[key]) - dec(ei[key])) <= D("1e-14"), "wrong recovered child-to-parent parameter"
                ai[key] = ei[key]
        expected_point = next((p for p in expected["project"]["points"] if p["id"] == 1001), None)
        if expected_point is not None:
            point = next(p for p in adjusted["project"]["points"] if p["id"] == 1001)
            assert distance(tuple(map(dec, point["position"])), tuple(map(dec, expected_point["position"]))) <= D("1e-12"), "wrong split point"
            point["position"] = expected_point["position"]
    same(expected, adjusted, name)


def points_at(path, fraction):
    result = []
    tolerance = ABS_BUDGET / path.total
    for owner, (low, high) in enumerate(zip(path.knots, path.knots[1:])):
        if low - tolerance <= fraction <= high + tolerance:
            clamped = max(low, min(high, fraction))
            result.append(path.resolve(clamped, owner)[1])
    assert result, "probe has no source owner"
    return result


def continuity(original, current, probes):
    bg, cg = Geometry(original), Geometry(current)
    before = original["project"]["sewing_groups"][0]
    after = next(g for g in current["project"]["sewing_groups"] if g["id"] == before["id"])
    same(before["correspondence"], after["correspondence"])
    max_error = ZERO
    for side in ("a", "b"):
        old, new = SewingPath(bg, before[side]), SewingPath(cg, after[side])
        assert abs(old.total - new.total) <= ABS_BUDGET, "operation changed path length"
        fractions = set(old.knots + new.knots)
        fractions.update(dec(u) if side == "a" else interpolate(before["correspondence"], dec(u)) for u in probes)
        for fraction in sorted(fractions):
            op, np = points_at(old, fraction), points_at(new, fraction)
            for left, right in ((op, np), (np, op)):
                for point in left:
                    error = min(distance(point, other) for other in right)
                    max_error = max(max_error, error)
                    assert error <= ABS_BUDGET, "split/join/remesh changed continuous physical pairing"
    return str(max_error)


def inspect_checkpoint(expected, receipt, name, probes, original):
    assert receipt["kind"] == "sewing_inspection"
    source = receipt["source"]
    source_check(expected, source, name)
    geometry = Geometry(source)
    groups = source["project"]["sewing_groups"]
    frozen = {g["id"]: group_expectation(geometry, g) for g in groups}
    assert [g["id"] for g in receipt["sewing_groups"]] == [g["id"] for g in groups]
    metric = [check_group(geometry, g, actual, frozen[g["id"]]) for g, actual in zip(groups, receipt["sewing_groups"])]
    mesh = check_mesh(geometry, source, receipt["mesh"], frozen)
    pair_error = continuity(original, source, probes) if name in {"split", "joined", "undo_join", "rejected_overlap", "redo_join", "remeshed"} else None
    return {"checkpoint": name, "metric": metric, "mesh": mesh, "maximum_continuity_error_m": pair_error}


def artifact(base, relative):
    assert isinstance(relative, str)
    path = (base / relative).resolve()
    assert path.is_relative_to(base.resolve()) and path.is_file(), "missing or escaping capture artifact"
    return path


def evaluate(path):
    capture, base = load(path), path.parent
    assert capture["kind"] == "headless-chain-sewing-capture-v1"
    assert re.fullmatch(r"[0-9a-f]{40}", capture["candidate"]["commit"])
    assert re.fullmatch(r"[0-9a-f]{64}", capture["candidate"]["binary_sha256"])
    expected = load(HERE / "expected-checkpoints.json")
    assert set(capture["checkpoints"]) == set(expected["checkpoints"]), "missing operation checkpoint"
    receipts, reports = {}, []
    for name, source in expected["checkpoints"].items():
        receipt_path = artifact(base, capture["checkpoints"][name])
        assert hashlib.sha256(receipt_path.read_bytes()).hexdigest() == capture["artifacts"][capture["checkpoints"][name]]
        receipt = load(receipt_path)
        receipts[name] = receipt
        reports.append(inspect_checkpoint(source, receipt, name, expected["probe_u"], expected["checkpoints"]["split_loaded"]))
    same(receipts["undo_join"]["source"], receipts["rejected_overlap"]["source"], "failed edit preserves every source bit")
    same(receipts["undo_join"]["sewing_groups"], receipts["rejected_overlap"]["sewing_groups"], "failed edit preserves derived state")
    commands = capture["commands"]
    assert [c["operation"] for c in commands] == ["split", "join", "undo_join", "reject_overlap", "redo_join", "remesh", "remove_piece", "undo_removal", "redo_removal"]
    for command in commands:
        assert command["shared_editor_command"] is True
        assert command["observer_review"] is True, "actual shared-command invocation not yet reviewed"
        for key in ("request", "response", "invocation"):
            p = artifact(base, command[key])
            assert capture["artifacts"][command[key]] == hashlib.sha256(p.read_bytes()).hexdigest()
        if command["operation"] == "reject_overlap":
            assert command["rejected"] is True and "overlap" in command["error"].lower()
        else:
            assert command["rejected"] is False
    return {"kind": "headless-chain-sewing-evaluation-v1", "status": "pass", "candidate": capture["candidate"], "checkpoints": reports,
            "observed": ["reversed crossing split traversal", "stable group ID/name/map", "retained joined entries", "continuous source pairing", "remesh identity", "whole dependent removal", "unrelated common-interval group preservation", "one-step removal Undo/Redo", "error source/revision preservation and surviving Redo"],
            "still_unknown": ["broader deep history and native file failure preservation", "bounded cancellation/staleness/exhaustion", "reference parity", "physical quality", "complete roadmap gate"]}


def self_check():
    expected = load(HERE / "expected-checkpoints.json")
    sources = expected["checkpoints"]
    for name, source in sources.items():
        source_check(source, source, name)
    errors = [continuity(sources["split_loaded"], sources[name], expected["probe_u"]) for name in ("split", "joined", "undo_join", "redo_join", "remeshed")]
    rejected = []
    for label, name, mutate in (
        ("wrong-child-traversal", "split", lambda d: d["project"]["sewing_groups"][0]["a"].reverse()),
        ("joined-entries-merged", "joined", lambda d: d["project"]["sewing_groups"][0].update({"a": [{"curve": 7, "t0": 0.125, "t1": 0.375}, {"curve": 16, "t0": 0.9, "t1": 0.3}]})),
        ("wrong-recovered-t", "joined", lambda d: d["project"]["sewing_groups"][0]["a"][1].update({"t1": 0.61})),
        ("allocator-rewound", "joined", lambda d: d["project"].update({"next_id": 1001})),
        ("partial-dependent-trim", "removed", lambda d: d["project"]["sewing_groups"].append(deepcopy(sources["removal_loaded"]["project"]["sewing_groups"][0]))),
        ("unrelated-group-lost", "removed", lambda d: d["project"].update({"sewing_groups": []})),
        ("rejection-mutates-revision", "rejected_overlap", lambda d: d["project"].update({"revision": 77})),
    ):
        bad = deepcopy(sources[name])
        mutate(bad)
        try:
            source_check(sources[name], bad, name)
        except (AssertionError, KeyError, IndexError):
            rejected.append(label)
        else:
            raise AssertionError(label)
    return {"kind": "headless-supplement-self-check", "candidate_outputs_inspected": False,
            "checkpoints": len(sources), "continuity_errors_m": errors, "rejected_mutations": rejected,
            "claim": "independent prospective mechanics only; no shared command executed"}


if __name__ == "__main__":
    if sys.argv[1] == "self-check":
        result, output = self_check(), Path(sys.argv[2])
    else:
        try:
            result = evaluate(Path(sys.argv[1]))
        except (AssertionError, ValueError, KeyError, IndexError, TypeError, ArithmeticError) as error:
            result = {"kind": "headless-chain-sewing-evaluation-v1", "status": "fail", "reason": str(error)}
        output = Path(sys.argv[2])
    with output.open("x") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(json.dumps({"status": result.get("status", "self-check"), "output": output.name}))
    if result.get("status") == "fail":
        raise SystemExit(1)
