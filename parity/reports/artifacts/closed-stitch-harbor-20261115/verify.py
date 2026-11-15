#!/usr/bin/env python3
"""Static reproduction of closed saved states; never executes a garment or build."""
import argparse
import ast
import hashlib
import importlib.util
import inspect
import json
import math
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
SOURCE = "2e9795a3cb92c79ad3b406e675dcbdb49c60ce67"
ORIGINAL = ".local/closed-stitch-garment-protocol-20261115/"
DESTINATION = "parity/reports/artifacts/closed-stitch-harbor-20261115"
PRIVACY = re.compile(rb'/' + rb'Users/|:' + rb'Users:|/' + rb'home/|:' + rb'home:|/' + rb'private/var/|/' + rb'var/folders/')
FUNCTIONS = {"evaluator.py": {"authorization", "verify_frozen", "verify_preparation", "verify_readonly_inputs"}, "runner.py": {"verify_bindings"}}
ASSIGNMENTS = {"evaluator.py": {"EXPERIMENT", "SOURCE", "PREVIOUS_PLAN_SHA", "PLAN_SHA", "PRODUCER_BINDING_SHA"},
               "runner.py": {"EVALUATOR", "SOURCE", "PLAN_SHA", "INPUT_BINDING_SHA", "PRODUCER_BINDING_SHA", "DRIVER_SHA"}}
NUMERICAL = {"metrics.py": "e4b0bf5034fa0670f39c2115e996e679316891235a3b4eb8e1240b58f3411610",
             "geometry_math.py": "5f83dd7ad4a322d5fdd9d8e62e66a1e67688f834f7c6ed7cb912ff0cc1a69a40",
             "static_geometry.py": "c4583bbda9250cfa32cdc305c1b21563c26ca46ddb75032f5c2ae4d128299b34",
             "decimal_oracle.py": "7aaf8916b53747bdc6af3056e9d8f711913d260c47841d1d63763213a670fa8d",
             "capsule_geometry.py": "f5dc3581caf3aa4f2ba2aab9ee743d641ca2313262119c55157d88170df0a39b"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def loads(text):
    def pairs(items):
        value = {}
        for key, item in items:
            require(key not in value, "Duplicate external JSON key")
            value[key] = item
        return value
    def floating(token):
        value = float(token)
        require(math.isfinite(value), "Nonfinite external number")
        return value
    return json.loads(text, object_pairs_hook=pairs, parse_float=floating,
                      parse_constant=lambda token: require(False, "Nonfinite external number"))


def child(base, name):
    relative = Path(name)
    require(not relative.is_absolute() and ".." not in relative.parts, "Artifact path escaped root")
    path, base = base / relative, base.resolve()
    require(path.resolve().is_relative_to(base), "Artifact escaped root")
    current = path
    while current != base:
        require(not current.is_symlink(), "Symlink artifact refused")
        current = current.parent
    require(path.is_file(), "Missing artifact: " + relative.name)
    return path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(name):
    return loads(child(ROOT, name).read_text())


def manifest_tree(directory, filename, expected=None):
    path = child(directory, filename)
    require(expected is None or sha(path) == expected, "External packet manifest differs")
    manifest = loads(path.read_text())
    files = manifest["files"]
    if isinstance(files, list):
        require(len({r["path"] for r in files}) == len(files), "Duplicate manifest member")
        files = {row["path"]: row["sha256"] for row in files}
    inventory = {p.relative_to(directory).as_posix() for p in directory.rglob("*") if p.is_file()}
    require(inventory == set(files) | {filename}, "Packet inventory differs")
    for p in directory.rglob("*"):
        require(not p.is_symlink(), "Packet symlink refused")
    for name, digest in files.items():
        p = child(directory, name)
        require(sha(p) == digest and not PRIVACY.search(p.read_bytes()), "Packet hash/privacy differs: " + name)
    return files


def qualified_ast(path, functions, assignments):
    tree = ast.parse(path.read_text())
    tree.body = [n for n in tree.body if not (
        isinstance(n, ast.FunctionDef) and n.name in functions
        or isinstance(n, ast.Assign) and all(isinstance(t, ast.Name) and t.id in assignments for t in n.targets)
        or isinstance(n, ast.Import) and len(n.names) == 1 and n.names[0].name == "bindings")]
    return ast.dump(tree, include_attributes=False)


def scope(record, case):
    require(type(record["schema"]) is int and record["schema"] == 1 and record["case"] == case
            and type(record["attempt"]) is int and record["attempt"] == 1 and record["source_commit"] == SOURCE, "Wrong closed case scope")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--preparation", action="store_true", help="Untracked staging replay; never publication certification")
    args = parser.parse_args()
    repo = args.repo_root.resolve()
    files = manifest_tree(ROOT, "MANIFEST.json")
    require("PRIVATE-MAPPING.json" not in files, "Private inverse map in packet")
    if not args.preparation:
        require(ROOT == repo / DESTINATION, "Default replay must use the bound public destination")
        tracked = set(subprocess.check_output(["git", "ls-files", "-z", "--", DESTINATION], cwd=repo).decode().split("\0"))
        require(all(DESTINATION+"/"+name in tracked for name in set(files)|{"MANIFEST.json"}), "Public artifact not tracked")
        require(not subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=normal"], cwd=repo), "Clean checkout required")
    refs = read("REFERENCES.json")
    for item in refs.values():
        manifest_tree(repo / item["path"], item["manifest"], item["sha256"])
    prior = repo / refs["prior"]["path"]
    source = read("experiment/SOURCE-BINDING.json")
    require(source["source_commit"] == SOURCE and len(source["source_hashes"]) == 45, "Wrong source identity/inventory")
    environment = read("experiment/PROTOCOL.json")["python"]
    require(sys.implementation.name == environment["implementation"] and list(sys.version_info[:3]) == environment["version"]
            and sum([1.0, 1e-16, 1e-16]) == environment["builtin_sum_probe"], "Original Python arithmetic environment differs")
    for name, expected in source["source_hashes"].items():
        require(not Path(name).is_absolute() and ".." not in Path(name).parts, "Source path escaped commit")
        data = subprocess.check_output(["git", "show", SOURCE+":"+name], cwd=repo)
        require(hashlib.sha256(data).hexdigest() == expected, "Published commit source hash differs")
    build_source = loads(child(repo / refs["build"]["path"], "source-pretest.json").read_text())
    require(build_source["source_commit"] == SOURCE and build_source["source_hashes"] == source["source_hashes"], "Integrated build source differs")
    provenance = read("PUBLICATION-PROVENANCE.json")
    originals, copied = {}, {}
    for row in provenance["copies"]:
        p = child(ROOT, row["public_path"])
        require(sha(p) == row["public_sha256"] and p.stat().st_size == row["public_bytes"], "Public derivative differs")
        require(len(row["substitution_counts"]) == 3 and all(type(n) is int and n >= 0 for n in row["substitution_counts"]), "Invalid literal substitution counts")
        for rule, count in zip(provenance["rules"], row["substitution_counts"]):
            require(p.read_bytes().count(rule["public_prefix"].encode()) == count, "Literal replacement count differs")
        if row["byte_identical"]:
            require(row["private_sha256"] == row["public_sha256"] and row["private_bytes"] == row["public_bytes"], "False byte-identity claim")
        originals[row["private_logical_path"]] = row
        copied[row["public_path"]] = row
    for row in provenance["references"]:
        p = child(ROOT, row["packet_path"]) if "packet_path" in row else child(repo, row["repo_path"])
        require(sha(p) == row["private_sha256"] and p.stat().st_size == row["private_bytes"], "Public byte-identical reference differs")
        require(row["private_logical_path"] not in originals, "Duplicate original mapping")
        originals[row["private_logical_path"]] = row
    def raw(name):
        return copied[name]["private_sha256"]
    def descriptor(value):
        require(originals[value["path"]]["private_sha256"] == value["sha256"], "Original descriptor chain differs")
    for name, old in (("evaluator.py", prior / "original-evaluator/evaluator.py"), ("runner.py", prior / "experiment/runner.py")):
        require(qualified_ast(old, FUNCTIONS[name], ASSIGNMENTS[name]) == qualified_ast(child(ROOT, "evaluation/"+name), FUNCTIONS[name], ASSIGNMENTS[name]), "Nonbinding module AST differs")
    review = read("experiment/ROOT-REUSE-REVIEW.json")
    require(review["qualified_comparison"] == "passed" and review["changes_to_frozen_files"] is False, "Reuse qualification lost")
    for name in FUNCTIONS:
        require(set(review["normalizations"][name]["functions"]) == FUNCTIONS[name]
                and set(review["normalizations"][name]["simple_name_assignments"]) == ASSIGNMENTS[name]
                and review["normalizations"][name]["import"] == "import bindings", "Normalization qualification changed")
    freeze, prep = read("evaluation/FREEZE.json"), read("evaluation/PREPARATION-FREEZE.json")
    require(raw("evaluation/FREEZE.json") == "94e6f33aa2518e3f1277d1e0f4bddb8ed96c8c75f531a29df003467da4a587a6"
            and raw("evaluation/PREPARATION-FREEZE.json") == "3f7b1d00bf29932baf07f06123ae5f8610bab25502ded674198147f6ba357d78"
            and freeze["files"] == prep["files"], "Original freezes differ")
    coverage = read("ARCHIVE-COVERAGE.json")
    require({r["original_name"]: r["sha256"] for r in coverage["members"]} == prep["files"], "Selective archive coverage differs")
    for row in coverage["members"]:
        if row["coverage"] == "copy_or_external_reference":
            require(originals[ORIGINAL+row["original_name"]]["private_sha256"] == row["sha256"], "Covered frozen artifact differs")
    auth = read("experiment/execution-authorization.json")
    require(auth["status"] == "authorized" and auth["explicit_root_authorization"] is True and auth["source_commit"] == SOURCE
            and auth["cases"] == ["base", "height"] and type(auth["attempts_per_case"]) is int and auth["attempts_per_case"] == 1, "Original execution scope differs")
    for key in ("runner", "plan", "source_input_binding", "producer_binding", "evaluator_freeze", "preparation_freeze", "gates"):
        descriptor(auth[key])
    producer_binding = read("experiment/PRODUCER-BINDING.json")
    require(producer_binding["source_commit"] == SOURCE and producer_binding["geometry_only_executions"] == 2
            and producer_binding["physics_initializations_or_steps"] == 0, "Preparation scope differs")
    for name, expected in producer_binding["files"].items():
        require(originals[ORIGINAL+name]["private_sha256"] == expected, "Producer source/lock binding differs")
    refusal, correction = read("experiment/ROOT-CONTROLLER-LAUNCH-REFUSAL.json"), read("experiment/ROOT-CONTROLLER-ARGUMENT-CORRECTION.json")
    require(refusal["exit_code"] == 1 and refusal["original_result"]["status"] == "refused_or_failed"
            and refusal["original_result"]["no_resume_or_retry"] is True and refusal["runs_directory_exists"] is False, "Failed first controller lost")
    require(correction["original_result_remains_failed"] is True and correction["physical_attempts_before_correction"] == 0
            and correction["root_explicit_correction_authorization"] is True, "Prospective argument correction lost")
    for case in ("base", "height"):
        old = loads(child(prior, f"experiment/runs/{case}/analysis.json").read_text())
        require(old["strict_delivery_passed"] is False, "Old source077 failure lost")
    sys.path.insert(0, str(prior / "public-evaluator"))
    spec = importlib.util.spec_from_file_location("unchanged_saved_state_math", prior / "public-evaluator/evaluator.py")
    e = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(e)
    for name, expected in NUMERICAL.items():
        require(sha(child(prior / "public-evaluator", name)) == expected, "Numerical helper differs")
    before = dict(vars(e))
    functions = {n: (v, v.__code__, v.__defaults__) for n, v in before.items() if inspect.isfunction(v)}
    e.EXPERIMENT = ROOT / "experiment"
    def unchanged():
        require(vars(e).keys() == before.keys(), "Evaluator global inventory differs")
        for name, value in before.items():
            if name != "EXPERIMENT":
                require(vars(e)[name] is value, "Unapproved evaluator global replaced")
        for name, (value, code, defaults) in functions.items():
            require(getattr(e, name) is value and value.__code__ is code and value.__defaults__ is defaults, "Function/code/default replaced")
    # Small trust-boundary mutations; no new dynamics or original timing claim.
    negatives = 0
    for text in ('{"x":1,"x":2}', '{"x":NaN}', '{"x":1e999}'):
        try:
            loads(text)
        except ValueError:
            negatives += 1
        else:
            raise ValueError("Malformed external JSON accepted")
    for values in ((.15, 0., 0.), (0., .005, 0.), (0., 0., .002)):
        require(not all(e.screen(*values, 0., True, 0).values()), "Strict equality limit accepted")
        negatives += 1
    for schema, attempt in ((True, 1), (1, True)):
        try:
            scope({"schema": schema, "case": "base", "attempt": attempt, "source_commit": SOURCE}, "base")
        except ValueError:
            negatives += 1
        else:
            raise ValueError("Boolean identity accepted")
    results = read("RESULTS.json")
    reproduced = {}
    for case in ("base", "height"):
        prefix = f"experiment/runs/{case}/"
        receipt, producer, archived = read(prefix+"case-receipt.json"), read(prefix+"producer-receipt.json"), read(prefix+"analysis.json")
        for record in (receipt, producer):
            scope(record, case)
            require(record["authorization_sha256"] == raw("experiment/execution-authorization.json")
                    and record["evaluator_freeze_sha256"] == raw("evaluation/FREEZE.json")
                    and record["driver_sha256"] == producer_binding["driver_sha256"]
                    and record["input_sha256"] == raw(f"experiment/inputs/{case}.weftforge"), "Closed source/input/driver/freeze/auth chain differs")
            require(record["producer_exit_code"] == 0 and record["stop_reason"] is None
                    and type(record["producer_peak_rss_bytes"]) is int and 0 < record["producer_peak_rss_bytes"] <= 268435456
                    and type(record["numerical_output_bytes"]) is int and 0 <= record["numerical_output_bytes"] <= 268435456, "Original execution/resource screen differs")
            for name, expected in record["artifacts"].items():
                require(raw(prefix+name) == expected, "Original producer artifact differs")
        require(receipt["status"] == "complete" and producer["status"] == "producer_complete" and receipt["failure"] is None, "Original operational status differs")
        start, end, closed = (receipt[k] for k in ("start_monotonic_seconds", "producer_end_monotonic_seconds", "final_monotonic_seconds"))
        require(0 <= start <= end <= closed and abs(closed-start-receipt["whole_case_elapsed_seconds"]) <= 1e-9 and closed-start <= 420., "Original whole-case clock differs")
        for key in ("analysis", "inspection"):
            descriptor(receipt[key])
        require(archived["receipt_sha256"] == raw(prefix+"producer-receipt.json") and archived["authorization_sha256"] == raw("experiment/execution-authorization.json")
                and archived["producer_artifacts"] == receipt["artifacts"] and archived["case_start_monotonic_seconds"] == start
                and archived["analysis_completed_monotonic_seconds"] <= closed, "Original analysis chain/clock differs")
        inspection = read(prefix+"inspection.json")
        require(sum(v["bytes"] for v in inspection["numerical_files"].values()) == inspection["numerical_output_bytes_at_inventory"] <= 268435456, "Original volume inventory differs")
        for name, expected in inspection["numerical_files"].items():
            require(originals[name]["private_sha256"] == expected["sha256"] and originals[name]["private_bytes"] == expected["bytes"], "Original inventory coverage differs")
        audit, integrity = read(f"experiment/runner-audits/{case}-validation.json"), read(f"experiment/runner-audits/{case}-integrity.json")
        require(audit["execution_receipt_passed"] is True and audit["receipt_sha256"] == raw(prefix+"case-receipt.json")
                and integrity["operational_completion"] is True and integrity["post_write_wall_within_cap"] is True and integrity["post_write_output_within_cap"] is True
                and integrity["post_write_elapsed_seconds"] <= 420. and integrity["post_write_numerical_output_bytes"] <= 268435456, "Original integrity/resource audit differs")
        replay = e.analyze_saved(case, ROOT / "experiment/runs" / case)
        deterministic = {key: value for key, value in replay.items() if key != "analysis_seconds"}
        require(e.same(deterministic, {key: archived[key] for key in deterministic}), "Every deterministic numerical field must reproduce: " + case)
        require(archived["status"] == "analyzed_complete" and len(replay["rows"]) == 181 and archived["strict_delivery_passed"] is results["cases"][case]["strict_delivery_passed"], "Complete saved result differs")
        require(receipt["quality_flags"] == {key: archived[key] for key in receipt["quality_flags"]}, "Archived quality flags differ")
        reproduced[case] = {"saved_frames_reproduced": 181, "deterministic_fields_exact": True, "strict_delivery_passed": replay["strict_delivery_passed"],
                            "quality_status": replay["quality_status"], "original_whole_case_seconds": receipt["whole_case_elapsed_seconds"],
                            "original_producer_peak_rss_bytes": receipt["producer_peak_rss_bytes"], "original_numerical_output_bytes": receipt["numerical_output_bytes"]}
        unchanged()
    require(results["paired_strict_delivery_passed"] is all(r["strict_delivery_passed"] for r in reproduced.values()) is False, "Paired failure lost")
    manifest_tree(ROOT, "MANIFEST.json")
    print(json.dumps({"status": "closed_saved_states_reproduced_paired_failure_retained", "source_commit": SOURCE,
                      "packet_files": len(files)+1, "cases": reproduced, "adapter_negative_checks": negatives,
                      "qualified_AST_and_original_receipt_chains_verified": True, "old077_strict_failures_retained": True,
                      "first_controller_refusal_retained": True, "new_physical_executions": 0, "new_timed_benchmarks": 0,
                      "tracked_clean_publication_certified": not args.preparation,
                      "limits": "Static saved-state reproduction and archived resource audit; no new dynamics, original timing recertification, UI, full garment/release gate or reference parity."}, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError, subprocess.CalledProcessError) as error:
        print(json.dumps({"status": "refused", "error": str(error), "new_physical_executions": 0}), file=sys.stderr)
        sys.exit(1)
