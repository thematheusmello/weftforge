#!/usr/bin/env python3
"""Grade native-authored saved snapshots only; never execute WeftForge or CUA."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys
import types

ROOT = Path(__file__).resolve().parent
PINS = {
    "fixture.weftforge": "239009c6ae23da35d4e6fd12e53dfc07df12b721f9eb841a841f9cc30ec39831",
    "CRITERIA.json": "d748d52d5aee5bee2d57c02e81edd137c8e50ab5c9bea9f69afb1c27c31d2938",
    "evaluate_physical_sewing.py": "ac252d08d820f7aedac771502486a54ec2dc4242017911588d5e39fcaa38a384",
}


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


for name, expected in PINS.items():
    require(sha((ROOT / name).read_bytes()) == expected, f"frozen numerical input changed: {name}")

# Reuse the pre-candidate checker unchanged, with a private namespace that cannot
# invoke its __main__ candidate runner or write an import cache.
ORACLE = types.ModuleType("frozen_physical_oracle")
ORACLE.__file__ = str(ROOT / "evaluate_physical_sewing.py")
exec(compile((ROOT / "evaluate_physical_sewing.py").read_bytes(), ORACLE.__file__, "exec"), ORACLE.__dict__)
FIXTURE_RAW = (ROOT / "fixture.weftforge").read_bytes()
FIXTURE = ORACLE.loads(FIXTURE_RAW)
EXPECTED = ORACLE.loads((ROOT / "EXPECTED.json").read_bytes())
CASE = copy.deepcopy(next(c for c in ORACLE.CRITERIA["positive_cases"] if c["id"] == "initial_asymmetric"))
CASE["command"]["data"]["name"] = "Free sewing 1"
require(CASE["command"] == EXPECTED["native_command"] and CASE["expected"] == EXPECTED["independent_intervals_and_locations"],
        "native expectations differ from original frozen oracle")


def encoded(value):
    return json.dumps(value, indent=2, allow_nan=False).encode() + b"\n"


def load_snapshot(raw):
    require(0 < len(raw) <= 16 * 1024 * 1024, "native snapshot exceeds bounded project size")
    return ORACLE.loads(raw)


def grade_bytes(input_raw, created_raw, undo_raw, redo_raw, reopened_raw):
    require(sha(input_raw) == PINS["fixture.weftforge"], "actor input is not byte-identical to frozen R41 fixture")
    created, undone, redone, reopened = [load_snapshot(raw) for raw in [created_raw, undo_raw, redo_raw, reopened_raw]]
    group = ORACLE.check_document(created, FIXTURE, CASE)
    require(group["id"] == 20 and created["project"]["revision"] == 42 and created["project"]["next_id"] == 21,
            "native creation must allocate ID20 once and commit R42")
    expected_undo = copy.deepcopy(FIXTURE)
    expected_undo["project"].update(revision=43, next_id=21)
    require(undone == expected_undo, "Undo must restore original parsed construction at R43 while retaining allocator21")
    source_r43 = copy.deepcopy(FIXTURE)
    source_r43["project"]["revision"] = 43
    ORACLE.check_document(redone, source_r43, CASE)
    expected_redo = copy.deepcopy(created)
    expected_redo["project"]["revision"] = 44
    require(redone == expected_redo, "Redo changed created parsed fields/ID or lost fresh R44")
    ORACLE.check_document(reopened, source_r43, CASE)
    require(reopened == redone, "reopened parsed construction differs from Redo R44")
    require(reopened_raw == redo_raw, "same-serializer native Redo/reopened primary bytes differ")
    return {
        "status": "passed_saved_output_only",
        "snapshots": {"input": {"revision": 41, "next_id": 20, "sha256": sha(input_raw)},
                      "created": {"revision": 42, "next_id": 21, "group_id": 20, "sha256": sha(created_raw)},
                      "undo": {"revision": 43, "next_id": 21, "groups": 0, "sha256": sha(undo_raw)},
                      "redo": {"revision": 44, "next_id": 21, "group_id": 20, "sha256": sha(redo_raw)},
                      "reopened": {"revision": 44, "next_id": 21, "group_id": 20, "sha256": sha(reopened_raw)}},
        "metric_tolerance_m": 1e-8,
        "parsed_fields_only": True,
        "native_workflow_verdict": "not_established_by_saved_files",
        "limits": ["No saved file proves invalid-text refusal, caret restoration, A-only transient state, pointer actions, labels/guides, live history/cache or New Empty.",
                   "New Empty R0 and its no-op Undo/Redo require actual native trace/captures; no empty snapshot is silently inferred.",
                   "Parsed equality does not distinguish signed-zero bits or JSON numeric spelling. No simulation, process restart or reference parity claim."],
    }


def synthetic_good():
    created = ORACLE.expected_document(FIXTURE, CASE)
    undone = copy.deepcopy(FIXTURE)
    undone["project"].update(revision=43, next_id=21)
    redone = copy.deepcopy(created)
    redone["project"]["revision"] = 44
    return dict(input=FIXTURE_RAW, created=encoded(created), undo=encoded(undone),
                redo=encoded(redone), reopened=encoded(redone))


def selfcheck():
    maximum = ORACLE.crosscheck_oracles()
    good = synthetic_good()
    grade_bytes(*[good[k] for k in ["input", "created", "undo", "redo", "reopened"]])
    mutants = []

    def reject(name, blobs):
        try:
            grade_bytes(*[blobs[k] for k in ["input", "created", "undo", "redo", "reopened"]])
        except (AssertionError, KeyError, ValueError, TypeError, StopIteration) as error:
            mutants.append(dict(id=name, deliberately_wrong=True, rejected=True, diagnosis=str(error)[:320]))
        else:
            mutants.append(dict(id=name, deliberately_wrong=True, rejected=False))

    for name, stage, mutate in [
        ("linear_t_curved_endpoint", "created", lambda p: p["sewing_groups"][0]["a"][0].update(t1=0.5)),
        ("wrong_reverse_straight_endpoint", "created", lambda p: p["sewing_groups"][0]["b"][0].update(t1=1.0)),
        ("transient_side_saved_as_legacy_seam", "created", lambda p: p.update(sewing_groups=[])),
        ("wrong_source_policy_name", "created", lambda p: p["sewing_groups"][0].update(name="Original physical pair")),
        ("double_commit_R43", "created", lambda p: p.update(revision=43)),
        ("extra_ID_allocation", "created", lambda p: p.update(next_id=22)),
        ("unrelated_control_edit", "created", lambda p: p["curves"][0]["controls"][0].__setitem__(0, 0.08)),
        ("persistent_length_lock", "created", lambda p: p["sewing_groups"][0].update(length_m=0.1)),
        ("undo_reuses_allocator20", "undo", lambda p: p.update(next_id=20)),
        ("undo_stale_revision41", "undo", lambda p: p.update(revision=41)),
        ("undo_retains_group", "undo", lambda p: p.update(sewing_groups=ORACLE.loads(good["created"])["project"]["sewing_groups"])),
        ("redo_reallocated_groupID", "redo", lambda p: p["sewing_groups"][0].update(id=21)),
        ("redo_small_endpoint_drift_within_metric_budget", "redo", lambda p: p["sewing_groups"][0]["a"][0].update(t1=p["sewing_groups"][0]["a"][0]["t1"] + 1e-10)),
        ("reopen_new_revision45", "reopened", lambda p: p.update(revision=45)),
        ("reopen_small_endpoint_drift_within_metric_budget", "reopened", lambda p: p["sewing_groups"][0]["a"][0].update(t1=p["sewing_groups"][0]["a"][0]["t1"] + 1e-10)),
    ]:
        blobs = copy.deepcopy(good)
        doc = ORACLE.loads(blobs[stage])
        mutate(doc["project"])
        blobs[stage] = encoded(doc)
        reject(name, blobs)
    bad = copy.deepcopy(good)
    bad["input"] = FIXTURE_RAW + b" "
    reject("modified_readonly_input_bytes", bad)
    bad = copy.deepcopy(good)
    bad["reopened"] += b" "
    reject("reopen_primary_bytes_differ_with_equal_parsed_fields", bad)
    bad = copy.deepcopy(good)
    bad["created"] = good["created"].replace(b'"revision": 42', b'"revision": 42, "revision": 42', 1)
    reject("duplicate_native_field", bad)
    require(all(m["rejected"] for m in mutants), "grader accepted a deliberately wrong saved output")
    return dict(kind="native_saved_file_grader_synthetic_selfcheck", status="passed",
                product_CUA_or_Cargo_executed=False, wrong_outputs_rejected=len(mutants), mutants=mutants,
                oracle_maximum_crosscheck_error_m=maximum,
                note="Invented files exercise the prospective assertions; they are not native UI delivery evidence.")


def verify_freeze():
    manifest = ORACLE.loads((ROOT / "FREEZE.json").read_bytes())
    for item in manifest["files"]:
        require(sha((ROOT / item["path"]).read_bytes()) == item["sha256"], f"frozen packet changed: {item['path']}")
    return sha((ROOT / "FREEZE.json").read_bytes())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    check = sub.add_parser("selfcheck")
    check.add_argument("--out", type=Path, required=True)
    grade = sub.add_parser("grade")
    for name in ["input", "created", "undo", "redo", "reopened"]:
        grade.add_argument("--" + name, type=Path, required=True)
    grade.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    require(not args.out.exists(), "report exists; preserve every attempt and choose a fresh report")
    try:
        if args.mode == "selfcheck":
            result = selfcheck()
        else:
            freeze_sha = verify_freeze()
            result = grade_bytes(*[getattr(args, k).read_bytes() for k in ["input", "created", "undo", "redo", "reopened"]])
            result["freeze_sha256"] = freeze_sha
        code = 0
    except Exception as error:
        result = dict(kind="native_saved_file_grader", status="failed", mode=args.mode,
                      error=f"{type(error).__name__}: {error}", native_workflow_verdict="unverified")
        code = 1
    result["bindings"] = {name: sha((ROOT / name).read_bytes()) for name in
                          ["grade_saved.py", "EXPECTED.json", "PLAN.json", "SOURCE-INPUTS.json", *PINS]}
    with args.out.open("xb") as report:
        report.write(encoded(result))
    print(json.dumps({k: v for k, v in result.items() if k not in ["mutants", "snapshots"]}, allow_nan=False))
    return code


if __name__ == "__main__":
    sys.exit(main())
