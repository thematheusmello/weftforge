"""Exactly one A16 attempt, refused until the root binds a separate GO to FREEZE."""
import argparse
import json
import os
from pathlib import Path
import sys
import time

import resources as r
import signatures

ROOT = Path(__file__).resolve().parent
EXPERIMENT = "Harbor-A16x32-001"
CAP_SECONDS = 420.0


def read(path):
    return signatures.c.read(path)


def validate_go(value, freeze_sha):
    r.require(set(value) == {"status", "explicit_root_review_and_go", "experiment", "freeze_sha256",
                             "physical_attempts", "geometry_preflights", "cases", "retries", "continuations"}, "Wrong GO schema")
    r.require(value["status"] == "authorized" and value["explicit_root_review_and_go"] is True,
              "Separate root review and GO absent")
    r.require(value["experiment"] == EXPERIMENT and value["freeze_sha256"] == freeze_sha,
              "GO belongs to another frozen packet")
    for name, expected in (("physical_attempts", 1), ("geometry_preflights", 1), ("retries", 0), ("continuations", 0)):
        r.require(type(value[name]) is int and value[name] == expected, "GO workload differs: " + name)
    r.require(value["cases"] == ["base"], "GO must authorize A only")


def verify_bound_inputs(deadline=None):
    frozen = read(ROOT / "FREEZE.json")
    for name, digest in frozen["files"].items():
        r.require(r.sha(r.safe_file(name, ROOT), deadline) == digest, "Frozen preparation file changed: " + name)
    source = read(ROOT / "SOURCE-BINDING.json")
    for name, digest in source["files"].items():
        r.require(r.sha(r.safe_file(name, Path(source["root"])), deadline) == digest, "Current source changed: " + name)
    control = read(ROOT / "READONLY-BINDING.json")
    for name, digest in control["files"].items():
        r.require(r.sha(r.safe_file(name, Path(control["repo"])), deadline) == digest, "Retained read-only input changed: " + name)
    r.require(list(sys.version_info[:3]) == frozen["python"]["version"], "Python version changed")
    r.require(r.sha(Path(sys.executable).resolve(), deadline) == frozen["python"]["binary_sha256"], "Python binary changed")
    r.require(sum([1., 1e-16, 1e-16]) == frozen["python"]["sum_probe"], "Python arithmetic context changed")
    return frozen


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--go", required=True)
    parser.add_argument("--go-sha256", required=True)
    args = parser.parse_args()
    job = {"experiment": EXPERIMENT, "status": "prelaunch", "wrapper_pid": os.getpid(),
           "candidate_core": "ad7cc8bbbfc7a1ae735a400c21dcc7a37c80a5c5", "stages": []}
    deadline = None
    try:
        go_path = r.safe_file(args.go, ROOT)
        r.require(len(args.go_sha256) == 64 and r.sha(go_path) == args.go_sha256, "Exact root GO hash differs")
        freeze_sha = r.sha(ROOT / "FREEZE.json")
        validate_go(read(go_path), freeze_sha)
        verify_bound_inputs()
        plan = read(ROOT / "PROTOCOL.json")
        r.require(not any((ROOT / name).exists() for name in ("runtime", "preflight-base", "runs")),
                  "This packet already started; no retry or overwrite")
        (ROOT / "runtime").mkdir()
        job.update(status="running", root_go_sha256=args.go_sha256, freeze_sha256=freeze_sha)
        r.write_new(ROOT / "runtime/ATTEMPT-001.json", r.encoded(job))
        started = time.monotonic()
        deadline = started + CAP_SECONDS
        job.update(start_monotonic_seconds=started, deadline_monotonic_seconds=deadline)

        def stage(name, argv):
            r.deadline_check(deadline)
            argv = [str(deadline) if value == "<DEADLINE>" else value for value in argv]
            result = r.monitor(argv, ROOT, ROOT / f"runtime/{name}.stdout", ROOT / f"runtime/{name}.stderr",
                               deadline, r.numerical_bytes, rss_cap=r.CAP_BYTES,
                               on_start=lambda handle: r.write_new(ROOT / f"runtime/{name}-started.json", r.encoded(handle)))
            job["stages"].append({"name": name, **result})
            r.write_new(ROOT / f"runtime/{name}-receipt.json", r.encoded(result))
            r.require(not result["stop_reason"], name + " stopped: " + str(result["stop_reason"]))
            r.deadline_check(deadline)
            r.require(r.numerical_bytes() + r.RECEIPT_RESERVE < r.CAP_BYTES, "Numerical output stop budget reached")
            return result["exit_code"]

        r.require(stage("geometry", plan["argv"]["geometry"]) == 0, "Geometry preflight process failed")
        check = signatures.check_preflight(ROOT / "preflight-base")
        r.deadline_check(deadline)
        r.write_new(ROOT / "runtime/rest-signature.json", r.encoded(check))
        (ROOT / "runs").mkdir()
        r.require(stage("producer", plan["argv"]["producer"]) == 0, "Producer incomplete or failed; do not grade its prefix")
        signatures.check_current_metadata(read(ROOT / "runs/base/rest-metadata.json"), "base")
        analysis_exit = stage("consumer", plan["argv"]["consumer"])
        analysis = read(ROOT / "runs/base/analysis16.json")
        r.require(analysis_exit in (0, 2) and analysis["status"] == "analyzed_complete", "Independent analysis incomplete or failed")
        r.require(not analysis["errors"] and not analysis["diagnostic_disagreement_frames"] and
                  not analysis["blocked_diagnostic_verification_frames"], "Independent diagnostic verification failed")
        r.require(stage("comparison", plan["argv"]["comparison"]) == 0, "Control comparison failed")
        verify_bound_inputs(deadline)
        r.require(r.sha(go_path, deadline) == args.go_sha256, "Root GO changed during execution")
        job["output_hashes"] = {str(path.relative_to(ROOT)): r.sha(path, deadline)
                                for name in r.OUTPUT_DIRECTORIES for path in r.files_below(ROOT / name)}
        r.deadline_check(deadline)
        job.update(status="complete", strict_screen_passed=analysis["strict_delivery_passed"],
                   quality_status=analysis["quality_status"], failed_frames_by_screen=analysis["failed_frames_by_screen"],
                   historical_A_and_pair="failed_unchanged", native_or_reference_acceptance=False)
    except BaseException as error:
        job.update(status="failed", strict_screen_passed=False, error=f"{type(error).__name__}: {error}",
                   incomplete_or_failed_outputs_retained=True, continuation_allowed=False)
    if deadline is not None:
        ended = time.monotonic()
        job.update(final_monotonic_seconds=ended, whole_case_elapsed_seconds=ended-job["start_monotonic_seconds"],
                   numerical_output_bytes_before_final_receipt=r.numerical_bytes(),
                   clock_scope="All geometry, physical run, independent analysis, comparison, output hashing and postbindings; final receipt is integrity metadata")
        if ended >= deadline or r.numerical_bytes() + len(r.encoded(job)) > r.CAP_BYTES:
            job.update(status="failed", strict_screen_passed=False, cap_failed_at_final_inspection=True)
        r.write_new(ROOT / "runtime/FINAL.json", r.encoded(job))
    else:
        r.write_new(ROOT / f"checks/prelaunch-refusal-{time.monotonic_ns()}.json", r.encoded(job))
    print(json.dumps(job, allow_nan=False))
    return 0 if job["status"] == "complete" else 2


if __name__ == "__main__":
    raise SystemExit(main())
