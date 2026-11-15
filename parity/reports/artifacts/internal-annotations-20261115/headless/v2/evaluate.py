#!/usr/bin/env python3
"""Prospective original internal-annotation evaluator; no production geometry."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess
import sys
import time

import protocol

from protocol import (MODERN, encoded, execute, finite, keys, loads, mcp, preserved,
                      require, run_process, sha, snapshot, weighted_t)

ROOT = Path(__file__).resolve().parent
CRITERIA = loads((ROOT / "CRITERIA.json").read_bytes())
CASES = loads((ROOT / "CASES.json").read_bytes())
TOOLS = {"project_inspect", "project_lengths", "project_create", "project_execute", "project_import_svg",
         "project_import_dxf", "project_export_svg", "project_export_obj", "project_export_dxf"}


def fixture(name):
    return loads((ROOT / "fixtures" / (name + ".weftforge")).read_bytes())


def exact(actual, expected, where="document"):
    require(type(actual) is type(expected), f"type changed at {where}")
    if type(expected) is float:
        require(finite(actual) and struct.pack(">d", actual) == struct.pack(">d", expected), f"f64 bits changed at {where}")
    elif type(expected) is dict:
        keys(actual, expected)
        for key in expected:
            exact(actual[key], expected[key], where + "." + key)
    elif type(expected) is list:
        require(len(actual) == len(expected), f"list length changed at {where}")
        for i, (a, b) in enumerate(zip(actual, expected)):
            exact(a, b, f"{where}[{i}]")
    else:
        require(actual == expected, f"value changed at {where}")


def close(a, b, tol):
    require(finite(a) and abs(a - b) <= tol, f"{a!r} differs from independent {b!r} beyond {tol}")


def distance(a, b):
    return math.dist(a, b)


def point_segment(p, a, b):
    d = [b[i] - a[i] for i in range(2)]
    length2 = sum(x * x for x in d)
    u = max(0.0, min(1.0, sum((p[i] - a[i]) * d[i] for i in range(2)) / length2))
    return distance(p, [a[i] + u * d[i] for i in range(2)])


def cross(a, b, c):
    return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])


def segment_distance(a, b, c, d):
    if cross(a, b, c) * cross(a, b, d) < 0 and cross(c, d, a) * cross(c, d, b) < 0:
        return 0.0
    return min(point_segment(a, c, d), point_segment(b, c, d), point_segment(c, a, b), point_segment(d, a, b))


def analytic_contains(model, a, b):
    """Only original R/L/C models: exact straight edges or quadratic extrema."""
    if not all(finite(x) for p in [a, b] for x in p) or distance(a, b) <= 1e-9:
        return False
    threshold = 1e-6 + 1e-7 + 256 * sys.float_info.epsilon * max(1.0, *(abs(x) for p in [a, b] for x in p))
    if model in ["R", "L"]:
        poly = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] if model == "R" else [[0.0, 0.0], [1.0, 0.0], [1.0, 0.4], [0.4, 0.4], [0.4, 1.0], [0.0, 1.0]]
        def inside(p):
            return 0 < p[0] < 1 and 0 < p[1] < 1 and (model == "R" or p[0] < 0.4 or p[1] < 0.4)
        cuts = {0.0, 1.0}
        for axis in [0, 1]:
            if b[axis] != a[axis]:
                t = (0.4 - a[axis]) / (b[axis] - a[axis])
                if 0 < t < 1:
                    cuts.add(t)
        ts = sorted(cuts)
        tests = ts + [(lo + hi) / 2 for lo, hi in zip(ts, ts[1:])]
        if not all(inside([a[i] + (b[i] - a[i]) * t for i in range(2)]) for t in tests):
            return False
        return all(segment_distance(a, b, c, d) > threshold for c, d in zip(poly, poly[1:] + poly[:1]))
    require(model == "C", "unsupported independent model")
    if not all(threshold < p[0] < 1 - threshold and p[1] < 1 - threshold for p in [a, b]):
        return False
    dx, dy = b[0] - a[0], b[1] - a[1]
    qa, qb, qc = 2 * dx * dx, dy - 2 * dx * (1 - 2 * a[0]), a[1] - 2 * a[0] * (1 - a[0])
    ts = [0.0, 1.0]
    if qa and 0 < -qb / (2 * qa) < 1:
        ts.append(-qb / (2 * qa))
    gap = min(qa * t * t + qb * t + qc for t in ts)
    # |q'|<=2 gives this lower bound on Euclidean clearance to the parabola.
    # All frozen positive C cases are far above the uncertainty band.
    return gap / math.sqrt(5) > threshold


def mm(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]


def mv(a, v):
    return [sum(row[k] * v[k] for k in range(3)) for row in a]


def rotation(angles):
    x, y, z = angles; sx, cx = math.sin(x), math.cos(x); sy, cy = math.sin(y), math.cos(y); sz, cz = math.sin(z), math.cos(z)
    return mm([[cz, -sz, 0.0], [sz, cz, 0.0], [0.0, 0.0, 1.0]],
              mm([[cy, 0.0, sy], [0.0, 1.0, 0.0], [-sy, 0.0, cy]], [[1.0, 0.0, 0.0], [0.0, cx, -sx], [0.0, sx, cx]]))


def check_document(actual, expected, transform=None, before=None, tolerance=1e-10):
    if transform is None:
        exact(actual, expected)
        return
    expected = copy.deepcopy(expected)
    pc_before = next(p for p in before["project"]["pieces"] if p["id"] == transform["piece"])
    pc_after = next(p for p in actual["project"]["pieces"] if p["id"] == transform["piece"])
    angle, pivot, delta = transform["angle"], transform["pivot"], transform["translation"]
    c, s = math.cos(angle), math.sin(angle)
    def map_point(q):
        x, y = q[0] - pivot[0], q[1] - pivot[1]
        return [pivot[0] + c * x - s * y + delta[0], pivot[1] + s * x + c * y + delta[1]]
    old_points = {p["id"]: p["position"] for p in before["project"]["points"]}
    matrix_before = rotation(pc_before["placement"]["rotation"])
    matrix_expected = mm(matrix_before, [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]])
    matrix_actual = rotation(pc_after["placement"]["rotation"])
    for a, b in zip(sum(matrix_actual, []), sum(matrix_expected, [])):
        close(a, b, 1e-12)
    world_before = lambda q: [v + t for v, t in zip(mv(matrix_before, [*q, 0.0]), pc_before["placement"]["translation"])]
    world_after = lambda q: [v + t for v, t in zip(mv(matrix_actual, [*q, 0.0]), pc_after["placement"]["translation"])]
    for actual_point, expected_point in zip(actual["project"]["points"], expected["project"]["points"]):
        target = map_point(old_points[expected_point["id"]])
        require(distance(actual_point["position"], target) <= tolerance, "transform omitted/changed an endpoint")
        require(distance(world_before(old_points[expected_point["id"]]), world_after(actual_point["position"])) <= tolerance, "arranged endpoint moved")
        expected_point["position"] = actual_point["position"]
    old_grain, actual_grain = pc_before["grain_angle"], pc_after["grain_angle"]
    require(finite(actual_grain) and -math.pi <= actual_grain <= math.pi, "grain is not canonical")
    target_grain = [c * math.cos(old_grain) - s * math.sin(old_grain), s * math.cos(old_grain) + c * math.sin(old_grain), 0.0]
    for a, b in zip([math.cos(actual_grain), math.sin(actual_grain)], target_grain):
        close(a, b, 1e-12)
    require(distance(mv(matrix_before, [math.cos(old_grain), math.sin(old_grain), 0.0]), mv(matrix_actual, [math.cos(actual_grain), math.sin(actual_grain), 0.0])) <= tolerance, "world grain moved")
    epc = expected["project"]["pieces"][0]
    epc["placement"] = pc_after["placement"]; epc["grain_angle"] = actual_grain
    exact(actual, expected)


def migrated(doc):
    out = copy.deepcopy(doc); version = out["version"]; out["version"] = 4
    if version < 3:
        out["project"]["sewing_groups"] = []
    for pc in out["project"]["pieces"]:
        pc["internal_curves"] = []
        if version == 1:
            pc["grain_angle"] = 0.0
    if version == 1:
        for mat in out["project"]["materials"]:
            stretch = mat.pop("stretch_stiffness")
            mat["warp_stiffness"] = stretch; mat["weft_stiffness"] = stretch
    return out


def check_mesh(before, after, revision):
    require(after["doc_revision"] == revision, "stale rest mesh revision")
    expected = copy.deepcopy(before); expected["doc_revision"] = revision
    exact(after, expected, "rest_mesh")


def normalized_dxf_revision(raw, revision):
    require(type(revision) is int and revision >= 0, "invalid declared DXF revision")
    lines = raw.splitlines(keepends=True)
    require(len(lines) % 2 == 0, "DXF group/value pairing changed")
    matches = [i + 1 for i in range(0, len(lines), 2)
               if lines[i].strip() == b"999" and lines[i + 1].startswith(b"Source document revision")]
    require(len(matches) == 1, "DXF must have exactly one group999 source-revision comment")
    i = matches[0]; value = lines[i].rstrip(b"\r\n")
    prefix = b"Source document revision: "
    require(value == prefix + str(revision).encode("ascii"), "DXF source revision differs from its own project")
    lines[i] = prefix + b"<validated source revision>" + lines[i][len(value):]
    return b"".join(lines)


def check_dxf_export(before, after, before_revision, after_revision):
    require(normalized_dxf_revision(before, before_revision) == normalized_dxf_revision(after, after_revision),
            "annotation changed nonrevision DXF bytes")


def legacy_pair(u, v):
    expected = 2.5 * u if u <= 0.25 else 0.625 + (u - 0.25) * 0.5
    close(v, expected, 1e-10)


def check_legacy_stitches(report):
    mesh = report["mesh"]; count = 0
    for stitch in mesh["stitches"]:
        if stitch["seam"] == 20:
            a = weighted_t(mesh["boundary"], stitch["a"], 7)
            b = weighted_t(mesh["boundary"], stitch["b"], 16)
            legacy_pair((a - 0.125) / 0.75, (b - 0.875) / -0.75); count += 1
    require(count >= 2, "missing nonlinear legacy t-map stitches")


def check_schema(result):
    require(len(result["tools"]) == 9 and {t["name"] for t in result["tools"]} == TOOLS, "MCP tool surface changed")
    schema = next(t["inputSchema"] for t in result["tools"] if t["name"] == "project_execute")
    choices = schema["properties"]["commands"]["items"]["oneOf"]
    require(len(choices) == 28 and len({c["properties"]["type"]["const"] for c in choices}) == 28, "need 28 distinct shared variants")
    for kind, fields in [("add_internal_segment", ["piece", "start", "end"]), ("remove_internal_curve", ["curve"])]:
        c = next(c for c in choices if c["properties"]["type"]["const"] == kind)
        require(c["additionalProperties"] is False and set(c["required"]) == {"type", "data"}, "open command envelope")
        d = c["properties"]["data"]
        require(d["additionalProperties"] is False and set(d["required"]) == set(fields) and set(d["properties"]) == set(fields), "open/missing command fields")
        for name in fields:
            f = d["properties"][name]
            if name in ["piece", "curve"]:
                require(f["type"] == "integer" and f["minimum"] == 1 and f["maximum"] == 18446744073709551615, "nonzero u64 ID schema")
            else:
                require(f["type"] == "array" and f["minItems"] == 2 and f["maxItems"] == 2 and f["items"]["type"] == "number", "fixed metre coordinate schema")


def bad_document(name):
    doc = fixture("S_line" if name == "cross_piece_internal_owner" else "R_line"); p = doc["project"]
    if name == "missing_internal_list": p["pieces"][0].pop("internal_curves")
    elif name == "missing_groups": p.pop("sewing_groups")
    elif name == "missing_point": p["points"] = p["points"][:-1]
    elif name == "missing_curve": p["curves"] = p["curves"][:-1]
    elif name == "duplicate_ID": p["points"][-1]["id"] = 11
    elif name == "duplicate_ownership": p["pieces"][0]["internal_curves"].append(13)
    elif name == "both_roles": p["pieces"][0]["internal_curves"].append(7)
    elif name == "shared_endpoint":
        p["points"].append(dict(id=15, position=[0.5, 0.875])); p["curves"].append(dict(id=16, start=11, end=15, controls=None)); p["pieces"][0]["internal_curves"].append(16); p["next_id"] = 17
    elif name == "cross_piece_internal_owner": p["pieces"][1]["internal_curves"] = [24]
    elif name == "identical_endpoint_ids": p["curves"][-1]["end"] = 11; p["points"] = p["points"][:-1]
    elif name == "orphan_point": p["points"].append(dict(id=14, position=[0.5, 0.5])); p["next_id"] = 15
    elif name == "internal_controls": p["curves"][-1]["controls"] = [[0.25, 0.375], [0.75, 0.625]]
    elif name == "invalid_next_id": p["next_id"] = 13
    elif name == "unknown_piece_field": p["pieces"][0]["internal_shapes"] = []
    else: raise AssertionError(name)
    return doc


def stress_document(edges=512, lines=2048):
    from prepare_inputs import base, polygon, add
    doc = base(); n = edges // 4
    vertices = [[i / n, 0.0] for i in range(n)] + [[1.0, i / n] for i in range(n)] + [[1 - i / n, 1.0] for i in range(n)] + [[0.0, 1 - i / n] for i in range(n)]
    polygon(doc, "Original bounded stress square", vertices)
    for i in range(lines):
        y = 0.25 + (i + 0.5) / (2 * lines)
        add(doc, [0.25, y], [0.75, y])
    return doc


def expected_failure(ok, before, after):
    require(not ok, "unsupported/rejected input was accepted")
    preserved(before, after)


def selfcheck():
    decisions = [("R", [0.125, 0.25], [0.875, 0.75], True), ("L", [0.2, 0.8], [0.8, 0.2], False),
                 ("L", [0.2, 0.8], [0.2, 0.2], True), ("C", [0.1, 0.2], [0.9, 0.2], False),
                 ("C", [0.1, 0.75], [0.9, 0.75], True), ("R", [0.25, 1.05e-6], [0.75, 1.05e-6], False),
                 ("R", [0.25, 2e-6], [0.75, 2e-6], True)]
    for model, a, b, accepted in decisions:
        require(analytic_contains(model, a, b) is accepted, "independent analytic decision changed")
    require(abs(math.dist([0.125, 0.25], [0.875, 0.75]) - math.sqrt(13) / 4) < 1e-15, "line formula disagreement")
    mutants = []
    def rejects(ident, fn):
        try: fn()
        except (AssertionError, KeyError, ValueError, TypeError): mutants.append(dict(id=ident, rejected=True))
        else: mutants.append(dict(id=ident, rejected=False))
    for ident in ["endpoint_only_L_accepted", "endpoint_only_C_accepted", "internal_sewing_accepted", "mixed_v3_accepted", "v4_missing_list_accepted"]:
        rejects(ident, lambda: expected_failure(True, {}, {}))
    rejects("failed_recovery_changed", lambda: expected_failure(False, {"recovery": "before"}, {"recovery": "after"}))
    rejects("failed_geometry_changed", lambda: expected_failure(False, {"project": "before"}, {"project": "after"}))
    for ident, case_id, mutate in [
        ("resize_second_endpoint_omitted", "R_resize_both_endpoints", lambda d: d["project"]["points"][-1].update(position=[0.875, 0.75])),
        ("remove_second_endpoint_orphaned", "R_remove_line_both_points", lambda d: d["project"]["points"].append(dict(id=12, position=[0.875, 0.75]))),
        ("owner_remove_second_endpoint_orphaned", "S_remove_owner_both_points_and_dependents", lambda d: d["project"]["points"].append(dict(id=23, position=[0.875, 0.75]))),
        ("wrong_internal_ID", "R_add", lambda d: d["project"]["pieces"][0].update(internal_curves=[12])),
        ("wrong_revision", "R_add", lambda d: d["project"].update(revision=3)),
        ("allocator_consumed_extra", "R_add", lambda d: d["project"].update(next_id=15)),
    ]:
        case = next(c for c in CASES["positive"] if c["id"] == case_id); bad = copy.deepcopy(case["expected"]); mutate(bad)
        rejects(ident, lambda bad=bad, case=case: check_document(bad, case["expected"]))
    before = fixture("R_line"); expected_transform = copy.deepcopy(before); expected_transform["project"]["revision"] += 1
    good_transform = copy.deepcopy(expected_transform)
    for p in good_transform["project"]["points"]:
        p["position"] = [p["position"][0] + 0.125, p["position"][1] + 0.25]
    good_transform["project"]["pieces"][0]["placement"]["translation"] = [-0.125, -0.25, 0.0]
    transform = dict(piece=2, translation=[0.125, 0.25], angle=0.0, pivot=[0.0, 0.0])
    check_document(good_transform, expected_transform, transform, before)
    bad_transform = copy.deepcopy(good_transform); bad_transform["project"]["points"][-1]["position"] = [0.875, 0.75]
    rejects("transform_second_endpoint_omitted", lambda: check_document(bad_transform, expected_transform, transform, before))
    for version in [1, 2, 3]:
        expected = migrated(fixture(f"legacy_v{version}")); bad = copy.deepcopy(expected)
        bad["project"]["points"][0]["position"][0] = 0.0
        rejects(f"v{version}_signed_zero_lost", lambda bad=bad, expected=expected: exact(bad, expected))
    expected = fixture("R_bits"); bad = copy.deepcopy(expected); bad["project"]["points"][-2]["position"][1] = math.nextafter(bad["project"]["points"][-2]["position"][1], math.inf)
    rejects("hard_decimal_one_ulp_loss", lambda: exact(bad, expected))
    for field in ["positions", "uv", "triangles", "boundary", "stitches"]:
        before = dict(doc_revision=4, positions=[[0.0, 0.0, 0.0]], uv=[[0.0, 0.0]], triangles=[], boundary=[], stitches=[])
        after = copy.deepcopy(before); after["doc_revision"] = 5; after[field].append(0)
        rejects("annotation_changed_" + field, lambda before=before, after=after: check_mesh(before, after, 5))
    before = dict(doc_revision=4, uv=[]); after = copy.deepcopy(before)
    rejects("old_mesh_retag_not_current", lambda: check_mesh(before, after, 5))
    expected = migrated(fixture("legacy_v3")); bad = copy.deepcopy(expected); bad["project"]["seams"][0]["correspondence"][1][1] = 0.5
    rejects("legacy_nonlinear_t_map_reinterpreted", lambda: exact(bad, expected))
    legacy_pair(0.5, 0.75)
    rejects("legacy_stitch_paired_by_wrong_fraction", lambda: legacy_pair(0.5, 0.5))
    dxf_before = b"0\nSECTION\n2\nHEADER\n999\nSource document revision: 4\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nEOF\n"
    dxf_after = dxf_before.replace(b"Source document revision: 4", b"Source document revision: 5")
    check_dxf_export(dxf_before, dxf_after, 4, 5)
    rejects("DXF_wrong_source_revision", lambda: check_dxf_export(dxf_before, dxf_after, 4, 6))
    duplicate = dxf_after.replace(b"999\nSource document revision: 5\n", b"999\nSource document revision: 5\n999\nSource document revision: 5\n")
    rejects("DXF_duplicate_source_revision", lambda: check_dxf_export(dxf_before, duplicate, 4, 5))
    changed = dxf_after.replace(b"70\n4\n", b"70\n5\n")
    rejects("DXF_nonrevision_content_changed", lambda: check_dxf_export(dxf_before, changed, 4, 5))
    require(all(x["rejected"] for x in mutants), "checker accepted deliberate wrong output")
    return dict(kind="independent_internal_line_synthetic_selfcheck", status="passed", product_CUA_or_Cargo_executed=False,
                analytic_decisions=len(decisions), wrong_outputs_rejected=len(mutants), mutants=mutants,
                qualification="Synthetic assertion checks only; no product or native acceptance.")


def check_inspection(cli, work, output, doc):
    cp = run_process(cli, ["inspect", output], work)
    require(cp.returncode == 0 and not cp.stderr, "CLI inspect failed")
    report = loads(cp.stdout)
    require(report["revision"] == doc["project"]["revision"], "stale inspect")
    require(len(report["patterns"]) == len(doc["project"]["pieces"]), "CLI omitted per-piece summaries")
    for actual, pc in zip(report["patterns"], doc["project"]["pieces"]):
        require(actual["id"] == pc["id"] and actual["boundary"] == pc["boundary"] and actual["internal_curves"] == pc["internal_curves"], "CLI omits/changes per-piece ownership")


def verify_freeze():
    manifest = loads((ROOT / "FREEZE.json").read_bytes())
    for e in manifest["files"]:
        require(sha((ROOT / e["path"]).read_bytes()) == e["sha256"], f"frozen acceptance changed: {e['path']}")
    return sha((ROOT / "FREEZE.json").read_bytes())


def evaluate(binding_path, out):
    freeze_sha = verify_freeze(); binding = loads(binding_path.read_bytes())
    require(binding.get("root_authorized") is True and binding["freeze_sha256"] == freeze_sha, "root binding/GO missing")
    cli = Path(binding["cli"]).resolve(); require(sha(cli.read_bytes()) == binding["cli_sha256"], "immutable CLI hash differs")
    source = Path(binding["source_worktree"])
    for name, h in binding["source_hashes"].items():
        require(sha((source / name).read_bytes()) == h, "explicit source-worktree hash differs")
        raw = subprocess.check_output(["git", "show", binding["source_commit"] + ":" + name], cwd=source)
        require(sha(raw) == h, "bound commit source differs")
    require(not out.exists(), "run directory must be fresh; no retry or overwrite")
    out.mkdir(parents=True); records = []
    protocol.AUDIT_ROOT = out / "processes"; protocol.AUDIT_ROOT.mkdir(); protocol.AUDIT_STARTED = time.monotonic()
    def record(ident, fn):
        try: detail = fn(); records.append(dict(id=ident, status="passed", detail=detail))
        except Exception as error: records.append(dict(id=ident, status="failed", error=f"{type(error).__name__}: {error}"))
        (out / "RESULT.json").write_bytes(encoded(dict(kind="bounded_original_internal_annotation_acceptance", freeze_sha256=freeze_sha,
            binding_sha256=sha(binding_path.read_bytes()), cli_sha256=binding["cli_sha256"], observations=records,
            separate_required_gates=CRITERIA["required_separate_gates"], limits=CRITERIA["limits"])))
    def schemas():
        for modern in [True, False]:
            data, transcript = mcp(cli, out, "tools/list", {}, modern); check_schema(data)
            (out / ("schema-modern.json" if modern else "schema-legacy.json")).write_bytes(encoded(dict(data=data, transcript=transcript)))
        return 2
    record("closed_schema_28_variants_9_tools", schemas)
    for case in CASES["positive"]:
        def positive(case=case):
            raw_outputs = []
            for channel in CRITERIA["channels"]:
                work = out / case["id"] / channel; work.mkdir(parents=True)
                before = fixture(case["input"])
                if "input_resolution" in case: before["project"]["resolution"] = case["input_resolution"]
                (work / "input.weftforge").write_bytes(encoded(before))
                report, transcript = execute(cli, work, channel, case["commands"])
                require(report["revision"] == case["expected"]["project"]["revision"] and report["commands_applied"] == len(case["commands"]), "not one transaction revision")
                (work / "transcript.json").write_bytes(encoded(transcript))
                raw = (work / "output.weftforge").read_bytes(); actual = loads(raw)
                check_document(actual, case["expected"], case.get("transform"), before, case.get("transform_tolerance_m", 1e-10)); raw_outputs.append(raw)
                for query in case.get("length_queries", []):
                    if channel == "cli_execute":
                        cp = run_process(cli, ["lengths", "output.weftforge", "interval", str(query["curve"]), "0", "1"], work)
                        require(cp.returncode == 0 and not cp.stderr, "annotation length query failed")
                        length = loads(cp.stdout)
                    else:
                        length, _ = mcp(cli, work, "tools/call", {"name": "project_lengths", "arguments": {"path": "output.weftforge", "query": {"type": "interval", "data": {"curve": query["curve"], "t0": 0.0, "t1": 1.0}}}}, channel.endswith(MODERN), True)
                    require(length["source_revision"] == actual["project"]["revision"] and length["length_unit"] == "m", "stale/non-SI length")
                    exact(length["query"], {"type": "interval", "data": {"curve": query["curve"], "t0": 0.0, "t1": 1.0}})
                    close(length["length_m"], query["length_m"], 1e-12)
                if case["id"] in ["R_add", "S_add"]:
                    check_inspection(cli, work, "output.weftforge", actual)
                if channel != "cli_execute":
                    inspected, _ = mcp(cli, work, "tools/call", {"name": "project_inspect", "arguments": {"path": "output.weftforge"}}, channel.endswith(MODERN), True)
                    exact(inspected["project"], actual["project"])
            require(raw_outputs[0] == raw_outputs[1] == raw_outputs[2], "CLI/current/legacy MCP native bytes differ")
            return dict(channels=3, native_sha256=sha(raw_outputs[0]))
        record(case["id"], positive)
    def failure(ident, doc, commands, channel, same=False, raw_commands=None, raw_input=None):
        work = out / "negative" / ident / channel / ("same" if same else "separate"); work.mkdir(parents=True)
        (work / "input.weftforge").write_bytes(raw_input if raw_input is not None else encoded(doc))
        output = "input.weftforge" if same else "output.weftforge"
        if not same: (work / output).write_bytes(encoded(fixture("R")))
        (work / (output + ".recovery")).write_bytes(encoded(fixture("legacy_v1")))
        before = snapshot(work)
        (work.parent / (("same" if same else "separate") + "-before.json")).write_bytes(encoded(before))
        _, transcript = execute(cli, work, channel, commands, False, output_name=output, raw_commands=raw_commands)
        expected_failure(False, before, snapshot(work)); (work / "transcript.json").write_bytes(encoded(transcript))
    for case in CASES["negative"]:
        def negative(case=case):
            doc = fixture(case["input"])
            for field in ["resolution", "next_id", "revision"]:
                if field in case: doc["project"][field] = case[field]
            for channel in CRITERIA["channels"]:
                for same in ([False, True] if case.get("same_file") else [False]): failure(case["id"], doc, case["commands"], channel, same)
            return dict(rejected=True, channels=3)
        record(case["id"], negative)
    for name in CRITERIA["malformed_v4_cases"]:
        record("v4_" + name, lambda name=name: [failure("v4_" + name, bad_document(name), [dict(type="set_resolution", data=dict(metres=0.07))], c) for c in CRITERIA["channels"]])
    def versions():
        for version in [1, 2, 3]:
            old = fixture(f"legacy_v{version}"); expected = migrated(old)
            work = out / f"migration-v{version}"; work.mkdir()
            original = (ROOT / f"fixtures/legacy_v{version}.weftforge").read_bytes()
            (work / "old.weftforge").write_bytes(original); (work / "old.weftforge.recovery").write_bytes(original)
            cp = run_process(cli, ["recover", "old.weftforge", "v4.weftforge"], work)
            require(cp.returncode == 0 and not cp.stderr, "explicit read/save migration failed")
            exact(loads((work / "v4.weftforge").read_bytes()), expected)
            require((work / "old.weftforge").read_bytes() == original and (work / "old.weftforge.recovery").read_bytes() == original, "migration source bytes changed")
            for modern in [True, False]:
                inspected, _ = mcp(cli, work, "tools/call", {"name": "project_inspect", "arguments": {"path": "old.weftforge"}}, modern, True)
                exact(inspected["project"], expected["project"])
            for channel in CRITERIA["channels"]:
                same = work / channel; same.mkdir(); (same / "input.weftforge").write_bytes(original)
                execute(cli, same, channel, [dict(type="set_resolution", data=dict(metres=old["project"]["resolution"]))], output_name="input.weftforge")
                target = copy.deepcopy(expected); target["project"]["revision"] += 1
                exact(loads((same / "input.weftforge").read_bytes()), target)
                require((same / "input.weftforge.recovery").read_bytes() == original, "old-primary recovery bytes lost")
            mixed = copy.deepcopy(old); mixed["project"]["pieces"][0]["internal_curves"] = []
            for channel in CRITERIA["channels"]: failure(f"mixed-v{version}", mixed, [dict(type="set_resolution", data=dict(metres=0.07))], channel)
        return dict(versions=[1, 2, 3], unchanged_read_revision=True, exact_bits=True)
    record("strict_legacy_migration_and_old_primary_recovery", versions)
    def separation():
        work = out / "physical-separation"; work.mkdir()
        base_doc = fixture("S"); (work / "before.weftforge").write_bytes(encoded(base_doc))
        def inspect(path):
            cp = run_process(cli, ["inspect-sewing", path], work); require(cp.returncode == 0 and not cp.stderr, "rest readout failed")
            (work / (path + ".inspection.json")).write_bytes(cp.stdout)
            return loads(cp.stdout)
        before = inspect("before.weftforge"); check_legacy_stitches(before); current = "before.weftforge"
        stages = [([dict(type="add_internal_segment", data=dict(piece=2, start=[0.125, 0.25], end=[0.875, 0.75]))], 5),
                  ([dict(type="move_point", data=dict(point=22, position=[0.25, 0.375])), dict(type="move_point", data=dict(point=23, position=[0.75, 0.625]))], 6),
                  ([dict(type="remove_internal_curve", data=dict(curve=24))], 7)]
        for i, (commands, revision) in enumerate(stages):
            path = f"stage-{i}.weftforge"; execute(cli, work, "cli_execute", commands, input_name=current, output_name=path)
            after = inspect(path); check_mesh(before["mesh"], after["mesh"], revision)
            check_legacy_stitches(after)
            exact(after["sewing_groups"], before["sewing_groups"])
            for ext in ["obj", "svg", "dxf"]:
                cp = run_process(cli, ["export-" + ext, path, f"after-{i}.{ext}"], work); require(cp.returncode == 0, "surface/exterior export failed")
                cp = run_process(cli, ["export-" + ext, "before.weftforge", f"before-{i}.{ext}"], work); require(cp.returncode == 0, "baseline export failed")
                after_raw = (work / f"after-{i}.{ext}").read_bytes(); before_raw = (work / f"before-{i}.{ext}").read_bytes()
                if ext == "dxf":
                    check_dxf_export(before_raw, after_raw, base_doc["project"]["revision"], revision)
                else:
                    require(after_raw == before_raw, "annotation changed surface/exterior export")
            current = path
        (work / "before-inspection.json").write_bytes(encoded(before))
        return dict(annotation_stages=3, compared="all exposed mesh arrays/UV/ownership/weighted stitches; full OBJ/SVG bytes; DXF bytes except its validated own-revision group999 comment", limits="mass/constraints/old-mesh lifecycle remain source gates")
    record("annotation_only_mesh_sewing_and_export_separation", separation)
    def raw_schema():
        command = json.dumps([dict(type="add_internal_segment", data=dict(piece=2, start=[0.125, 0.25], end=[0.875, 0.75]))], separators=(",", ":"))
        raws = {"NaN": command.replace("0.125", "NaN", 1), "Infinity": command.replace("0.125", "Infinity", 1), "overflow": command.replace("0.125", "1e400", 1), "duplicate": command.replace('"piece":2', '"piece":2,"piece":2', 1)}
        for ident, raw in raws.items():
            for channel in CRITERIA["channels"]: failure("raw-" + ident, fixture("R"), [], channel, raw_commands=raw)
        invalid = fixture("R"); invalid["version"] = 5
        for channel in CRITERIA["channels"]: failure("future-version5", invalid, [dict(type="set_resolution", data=dict(metres=0.07))], channel)
        native = json.dumps(fixture("R"), separators=(",", ":"))
        for ident, raw in [("native-duplicate-field", native.replace('"internal_curves":[]', '"internal_curves":[],"internal_curves":[]', 1)),
                           ("native-NaN", native.replace('"resolution":0.07', '"resolution":NaN', 1)),
                           ("native-overflow", native.replace('"resolution":0.07', '"resolution":1e400', 1))]:
            for channel in CRITERIA["channels"]: failure(ident, fixture("R"), [dict(type="set_resolution", data=dict(metres=0.07))], channel, raw_input=raw.encode())
        return len(raws)
    record("nonfinite_duplicate_future_schema_refusal", raw_schema)
    def bits_roundtrip():
        work = out / "v4-bits"; work.mkdir(); original = fixture("R_bits")
        (work / "input.weftforge").write_bytes(encoded(original))
        execute(cli, work, "cli_execute", [dict(type="set_resolution", data=dict(metres=0.07))])
        expected = copy.deepcopy(original); expected["project"]["revision"] += 1
        exact(loads((work / "output.weftforge").read_bytes()), expected)
        return dict(signed_zero=True, hard_decimal=True)
    record("v4_signed_zero_hard_decimal_roundtrip", bits_roundtrip)
    def input_limits():
        command = [dict(type="set_resolution", data=dict(metres=0.07))]
        too_many = stress_document(edges=4, lines=6664)
        count = sum(len(too_many["project"][k]) for k in ["points", "curves", "pieces", "materials", "seams", "sewing_groups"])
        require(count == 20002, "object-cap fixture changed")
        failure("object-limit", too_many, command, "cli_execute")
        raw = encoded(fixture("R")); raw += b" " * (16 * 1024 * 1024 + 1 - len(raw))
        failure("file-limit", fixture("R"), command, "cli_execute", raw_input=raw)
        return dict(objects=20002, file_bytes=16777217)
    record("object_and_file_limits", input_limits)
    for kind in CRITERIA["writer_failure_cases"]:
        def writer_failure(kind=kind):
            for channel in CRITERIA["channels"]:
                work = out / "writer" / kind / channel; work.mkdir(parents=True)
                (work / "input.weftforge").write_bytes(encoded(fixture("R")))
                (work / "output.weftforge.recovery").write_bytes(encoded(fixture("legacy_v1")))
                if kind == "invalid_existing_destination": (work / "output.weftforge").write_bytes(b"old invalid primary must survive\n")
                elif kind == "destination_directory":
                    (work / "output.weftforge").mkdir(); (work / "output.weftforge/keep").write_bytes(b"directory marker")
                elif kind == "destination_symlink":
                    (work / "protected.weftforge").write_bytes(encoded(fixture("R"))); (work / "output.weftforge").symlink_to("protected.weftforge")
                else:
                    (work / "output.weftforge").write_bytes(encoded(fixture("R")))
                    (work / "output.weftforge.recovery").unlink(); (work / "output.weftforge.recovery").mkdir()
                    (work / "output.weftforge.recovery/keep").write_bytes(b"recovery marker")
                before = snapshot(work)
                _, transcript = execute(cli, work, channel, next(c for c in CASES["positive"] if c["id"] == "R_add")["commands"], False)
                preserved(before, snapshot(work)); (work / "transcript.json").write_bytes(encoded(transcript))
            return dict(channels=3, primary_and_recovery_retained=True, qualification="These cases fail before a late primary replacement; injected interruption semantics remain separate.")
        record(kind, writer_failure)
    def stress():
        work = out / "bounded-stress"; work.mkdir(); doc = stress_document()
        (work / "input.weftforge").write_bytes(encoded(doc)); (work / "output.weftforge").write_bytes(encoded(fixture("R")))
        (work / "output.weftforge.recovery").write_bytes(encoded(fixture("legacy_v1")))
        before = snapshot(work)
        (work / "commands.json").write_text('[{"type":"set_resolution","data":{"metres":0.07}}]')
        cp = run_process(cli, ["execute", "input.weftforge", "output.weftforge", "commands.json"], work)
        if cp.returncode:
            preserved(before, snapshot(work)); reason = cp.stderr.decode().lower()
            require(any(w in reason for w in ["work", "budget", "sample", "depth", "certificate", "precision"]), "stress refused for an unqualified reason")
            outcome = "refused_with_explicit_work_or_certificate_budget"
        else:
            expected = copy.deepcopy(doc); expected["project"]["revision"] += 1
            exact(loads((work / "output.weftforge").read_bytes()), expected); outcome = "accepted_with_exact_supported_construction"
        (work / "transcript.json").write_bytes(encoded(dict(returncode=cp.returncode, stdout=cp.stdout.decode(), stderr=cp.stderr.decode())))
        return dict(outcome=outcome, qualification=CRITERIA["stress"]["qualification"])
    record("bounded_stress_observation_not_workcap_proof", stress)
    record("immutable_source_and_binary_retained", lambda: require(sha(cli.read_bytes()) == binding["cli_sha256"] and all(sha((source / n).read_bytes()) == h for n, h in binding["source_hashes"].items()), "source/binary changed"))
    failed = [r["id"] for r in records if r["status"] != "passed"]
    summary = dict(bounded_adapter_status="failed" if failed else "passed", observations=len(records), failed=failed,
                   full_contract_acceptance="not_established_separate_required_gates")
    (out / "SUMMARY.json").write_bytes(encoded(summary)); return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__); sub = parser.add_subparsers(dest="mode", required=True)
    check = sub.add_parser("selfcheck"); check.add_argument("--out", type=Path, required=True)
    run = sub.add_parser("evaluate"); run.add_argument("--binding", type=Path, required=True); run.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.mode == "selfcheck":
        require(not args.out.exists(), "preserve prior selfcheck attempts")
        try: result = selfcheck(); code = 0
        except Exception as error: result = dict(status="failed", product_CUA_or_Cargo_executed=False, error=f"{type(error).__name__}: {error}"); code = 1
        result["bindings"] = {name: sha((ROOT / name).read_bytes()) for name in ["evaluate.py", "protocol.py", "prepare_inputs.py", "CASES.json", "CRITERIA.json", "SOURCE-INPUTS.json", "PLAN.json", "METHOD.txt", "PROSPECTIVE-CORRECTION.json", "STATIC-SUBPROCESS-COUNT.json"]}
        result["fixture_bindings"] = {p.relative_to(ROOT).as_posix(): sha(p.read_bytes()) for p in sorted((ROOT / "fixtures").glob("*.weftforge"))}
        args.out.write_bytes(encoded(result))
    else:
        result = evaluate(args.binding.resolve(), args.out.resolve()); code = int(bool(result["failed"]))
    print(json.dumps({k: v for k, v in result.items() if k != "mutants"}, allow_nan=False)); return code


if __name__ == "__main__":
    sys.exit(main())
