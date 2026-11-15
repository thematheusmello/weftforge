#!/usr/bin/env python3
"""Read-only publication replay; exit zero means reproduction, not garment acceptance."""
import ast
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[3]
sys.path.insert(0, str(ROOT / "public-evaluator"))
import evaluator as e


def require(condition, message):
    if not condition:
        raise ValueError(message)


def file(path):
    return e.safe_file(path, ROOT)


def read(path):
    return e.read(file(path))


def digest(path):
    return e.sha(file(path))


def verify_packet():
    manifest = read("MANIFEST.json")
    inventory = {str(p.relative_to(ROOT)) for p in ROOT.rglob("*") if p.is_file()}
    require(inventory == set(manifest["files"]) | {"MANIFEST.json"}, "Public packet inventory changed")
    for name, expected in manifest["files"].items():
        require(digest(name) == expected, f"Public artifact changed: {name}")
    relative = ROOT.relative_to(REPO).as_posix()
    tracked = set(subprocess.check_output(["git", "ls-files", "-z", "--", relative], cwd=REPO).decode().split("\0"))
    require(all(relative + "/" + name in tracked for name in inventory), "Public artifact is not Git-tracked")
    provenance = read("PUBLICATION-PROVENANCE.json")
    copies = {item["private_logical_path"]: item for item in provenance["copies"] if "private_logical_path" in item}
    for item in provenance["copies"]:
        require(digest(item["public_path"]) == item["public_sha256"], "Publication derivative changed")
        if item["byte_identical"]:
            require(item["private_sha256"] == item["public_sha256"], "False byte-identity claim")
    freeze = read("original-evaluator/FREEZE.json")
    require(digest("original-evaluator/FREEZE.json") == "4b22c7109de54f10c3f133fd1c9eced52421c5dd753cf50ac6520f8140f5b4b3", "Original evaluator freeze changed")
    require(len(freeze["files"]) == 24, "Wrong frozen evaluator inventory")
    for name, expected in freeze["files"].items():
        require(digest("original-evaluator/" + name) == expected, "Frozen evaluator bytes changed")
    trees = []
    for folder in ("original-evaluator", "public-evaluator"):
        tree = ast.parse(file(folder + "/evaluator.py").read_text())
        for node in tree.body:
            if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id in {"REPO", "EXPERIMENT", "GOLD"} for t in node.targets):
                node.value = ast.Constant(value="PUBLIC_DIRECTORY_MAPPING")
        trees.append(ast.dump(tree))
    require(trees[0] == trees[1], "Public evaluator changed beyond three directory assignments")
    for name in ("capsule_geometry.py", "decimal_oracle.py", "geometry_math.py", "metrics.py", "static_geometry.py"):
        require(digest("original-evaluator/" + name) == digest("public-evaluator/" + name), "Numerical routine changed")
    original_rest = file("original-evaluator/REST-BINDING.json").read_bytes()
    require(original_rest.replace(b".local/harbor-contact-candidates-20261114/public-safe-candidate/", b"historical/") == file("public-evaluator/REST-BINDING.json").read_bytes(), "Rest binding changed beyond path mapping")
    plan = read("experiment/PREPARATION-PLAN-v2.json")
    binding = read("experiment/SOURCE-INPUT-BINDING.json")
    gates = read("integrated-gates/package-binding.json")
    require(plan["source_commit"] == binding["source_commit"] == gates["source_commit"] == e.SOURCE, "Source identities differ")
    require(len(plan["source_hashes"]) == 40 and gates["source_hashes"] == plan["source_hashes"], "Tested forty-source binding differs")
    require(binding["files"] == {**{"source/" + n: h for n, h in plan["source_hashes"].items()}, **plan["inputs_sha256"]}, "Source/input bindings differ")
    for name, expected in binding["files"].items():
        require(digest("experiment/" + name) == expected, "Bound source/input changed")
    for name, expected in plan["source_hashes"].items():
        require(hashlib.sha256((REPO / name).read_bytes()).hexdigest() == expected, "Current runtime differs from tested source")
    require(all(j["status"] == "completed" and j["exit_code"] == 0 for j in read("integrated-gates/jobs.json")), "Original integration gate failed")
    return manifest, copies


def verify_closed_receipt(case, copies):
    prefix = "experiment/runs/" + case + "/"
    receipt = read(prefix + "case-receipt.json")
    producer = read(prefix + "producer-receipt.json")
    analysis = read(prefix + "analysis.json")
    auth_sha = digest("experiment/execution-authorization.json")
    driver = read("experiment/PRODUCER-BINDING-v2.json")["driver_sha256"]
    for record in (receipt, producer):
        require(record["source_commit"] == e.SOURCE and record["case"] == case and type(record["attempt"]) is int and record["attempt"] == 1, "Wrong closed receipt scope")
        require(record["authorization_sha256"] == auth_sha and record["driver_sha256"] == driver, "Closed authorization/driver binding differs")
        require(record["evaluator_freeze_sha256"] == digest("original-evaluator/FREEZE.json") and record["input_sha256"] == digest("experiment/inputs/" + case + ".weftforge"), "Closed evaluator/input binding differs")
        require(record["producer_exit_code"] == 0 and record["stop_reason"] is None and 0 < record["producer_peak_rss_bytes"] <= e.RSS_CAP and 0 <= record["numerical_output_bytes"] <= e.OUTPUT_CAP, "Original producer/resource screen failed")
        for name, expected in record["artifacts"].items():
            require(digest(prefix + name) == expected, "Closed producer artifact changed")
    require(receipt["status"] == "complete" and producer["status"] == "producer_complete", "Original case was not operationally complete")
    start, end, closed = (receipt[k] for k in ("start_monotonic_seconds", "producer_end_monotonic_seconds", "final_monotonic_seconds"))
    require(0 <= start <= end <= closed and abs(closed-start-receipt["whole_case_elapsed_seconds"]) <= 1e-9 and closed-start <= e.CAP_SECONDS, "Original closed case clock failed")
    require(receipt["analysis"]["sha256"] == digest(prefix + "analysis.json") and receipt["inspection"]["sha256"] == digest(prefix + "inspection.json"), "Original final receipt descriptors differ")
    require(analysis["authorization_sha256"] == auth_sha and analysis["receipt_sha256"] == digest(prefix + "producer-receipt.json") and analysis["producer_artifacts"] == receipt["artifacts"], "Original analysis/receipt chain differs")
    require(analysis["analysis_completed_monotonic_seconds"] <= closed and analysis["case_start_monotonic_seconds"] == start, "Original analysis clock differs")
    require(receipt["quality_flags"] == {key: analysis[key] for key in receipt["quality_flags"]}, "Original quality flags differ")
    inspection = read(prefix + "inspection.json")
    require(sum(v["bytes"] for v in inspection["numerical_files"].values()) == inspection["numerical_output_bytes_at_inventory"] <= e.OUTPUT_CAP, "Original recorded inventory total differs")
    for name, expected in inspection["numerical_files"].items():
        copied = copies[name]
        require(copied["private_sha256"] == expected["sha256"] and copied["private_bytes"] == expected["bytes"], "Original inventory/public copy provenance differs")
    audit = read("experiment/runner-audits/" + case + "-validation.json")
    require(audit["execution_receipt_passed"] is True and audit["receipt_sha256"] == digest(prefix + "case-receipt.json"), "Original closed receipt audit differs")
    return analysis, receipt


def self_check():
    require(not e.screen(.15, 0., 0., 0., True, 0)["material_axis_strain_below_0_15"], "Strain equality accepted")
    require(not e.screen(0., .005, 0., 0., True, 0)["sampled_seam_gap_below_0_005_m"], "Seam equality accepted")
    require(not e.screen(0., 0., .002, 0., True, 0)["physical_contact_below_0_002_m"], "Contact equality accepted")
    try:
        e.strict_loads('{"frame":1,"frame":2}')
    except ValueError:
        pass
    else:
        raise ValueError("Duplicate JSON key accepted")
    try:
        file("../escape")
    except ValueError:
        pass
    else:
        raise ValueError("Parent traversal accepted")


def main():
    self_check()
    manifest, copies = verify_packet()
    results = {}
    for case in ("base", "height"):
        original, receipt = verify_closed_receipt(case, copies)
        replay = e.analyze_saved(case, ROOT / "experiment/runs" / case)
        # Replay wall time is a new read-only measurement, never the original execution clock.
        deterministic = {k: v for k, v in replay.items() if k != "analysis_seconds"}
        require(e.same(deterministic, {k: original[k] for k in deterministic}), "Saved numerical replay differs: " + case)
        require(replay["strict_delivery_passed"] is False and replay["quality_status"] == "failed", "Strict failure was lost")
        results[case] = {"saved_frames_reproduced": len(replay["rows"]), "all_deterministic_analysis_fields_exact": True,
                         "strict_delivery_passed": False, "quality_status": "failed", "operational_completion": True,
                         "original_whole_case_elapsed_seconds": receipt["whole_case_elapsed_seconds"],
                         "original_producer_peak_rss_bytes": receipt["producer_peak_rss_bytes"],
                         "original_numerical_output_bytes": receipt["numerical_output_bytes"]}
    verify_packet()
    print(json.dumps({"kind": "public_saved_state_replay", "status": "reproduced_strict_failures", "source_commit": e.SOURCE,
                      "packet_files_verified": len(manifest["files"])+1, "cases": results,
                      "new_physical_executions": 0, "new_timed_benchmarks": 0,
                      "limits": "Original producer and native/CLI binaries remain private. Closed execution/resource receipts are audited as archived records; no new physical execution, original timing certification, UI acceptance, full CCD, convergence, calibration or reference parity."}, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print("Public replay refused: " + str(error), file=sys.stderr)
        sys.exit(1)
