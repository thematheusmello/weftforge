"""One prospective attempt; refuses without a separately hashed exact root GO."""
import argparse
import json
import os
from pathlib import Path
import sys
import time
from datetime import datetime, timezone

import common as b
from common import ROOT, REPO, c, r


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--go", required=True)
    parser.add_argument("--go-sha256", required=True)
    args = parser.parse_args()
    stages = []
    started, deadline = None, None
    try:
        go_path = r.safe_file(args.go, ROOT)
        r.require(len(args.go_sha256) == 64 and all(q in "0123456789abcdef" for q in args.go_sha256) and r.sha(go_path) == args.go_sha256, "Wrong explicit root GO hash")
        b.validate_go(b.read(go_path), r.sha(ROOT / "FREEZE.json"))
        frozen = b.verify_bound()
        r.require(not any((ROOT / name).exists() for name in r.OUTPUT_DIRECTORIES), "This case namespace already started; no retry/reset/overwrite")
        (ROOT / "runtime").mkdir()
        started = time.monotonic()
        deadline = started + b.CAP_SECONDS
        b.persist(ROOT / "runtime/SESSION.json", {"experiment": b.EXPERIMENT, "source_commit": b.SOURCE,
                  "started_at_utc": datetime.now(timezone.utc).isoformat(),
                  "started": started, "deadline": deadline, "go_path": str(go_path.relative_to(ROOT)),
                  "go_sha256": args.go_sha256, "wrapper_pid": os.getpid(), "physical_attempt": 1,
                  "build_binding": b.read(ROOT / "SOURCE-PREFLIGHT.json")["build_binding"]}, deadline)
        (ROOT / "inputs").mkdir()
        r.write_new(ROOT / "inputs/base.weftforge", (REPO / frozen["input_path"]).read_bytes())
        b.verify_session(deadline)
        binary = str(REPO / frozen["producer_binary"]["path"])
        python = frozen["python"]["binary_path"]

        def stage(name, argv):
            r.deadline_check(deadline)
            result = r.monitor(argv, ROOT, ROOT / f"runtime/{name}.stdout", ROOT / f"runtime/{name}.stderr",
                               deadline, r.numerical_bytes, rss_cap=b.CAP_BYTES,
                               reserve=2*r.RECEIPT_RESERVE,
                               on_start=lambda handle: b.persist(ROOT / f"runtime/{name}-started.json", handle, deadline))
            stages.append({"name": name, **result})
            b.persist(ROOT / f"runtime/{name}-stage.json", result, deadline)
            r.require(result["stop_reason"] is None, name + " stopped: " + str(result["stop_reason"]))
            r.require(result["peak_rss_bytes"] <= b.CAP_BYTES and result["end_monotonic_seconds"] < deadline, name + " exceeded cap")
            return result

        preflight = stage("init-only", [binary, "prepare", "inputs/base.weftforge", "preflight-base"])
        r.require(preflight["exit_code"] == 0, "Initialized preflight failed")
        checked = stage("metadata", [python, "-B", str(ROOT / "worker.py"), "metadata", "--deadline", str(deadline)])
        r.require(checked["exit_code"] == 0, "Independent original-rest gate refused; no dynamics started")
        (ROOT / "runs").mkdir()
        producer = stage("producer", [binary, "run", "inputs/base.weftforge", "runs/base", "expected/approved-metadata.json"])
        r.require(producer["exit_code"] == 0, "Actual producer incomplete; do not grade a prefix")
        b.persist(ROOT / "runtime/producer-receipt.json", {
            "source_commit": b.SOURCE, "source_manifest_sha256": frozen["repo_files"][frozen["source_path"]],
            "input_sha256": frozen["input_sha256"], "producer_sha256": frozen["producer_binary"]["sha256"],
            "approved_metadata_sha256": r.sha(ROOT / "expected/approved-metadata.json", deadline),
            "original_initialized_gate_sha256": r.sha(ROOT / "runtime/initialized-original-gate.json", deadline),
            "stage": producer, "artifacts": r.artifact_hashes(ROOT / "runs/base", deadline),
            "two_fresh_initializations": True, "init_only_step_calls": 0, "actual_dynamics_runs": 1,
        }, deadline)
        analysis = stage("analysis", [python, "-B", str(ROOT / "worker.py"), "analyze", "--deadline", str(deadline)])
        r.require(analysis["exit_code"] in (0, 2), "Independent analysis operational failure")
        finalized = stage("finalization", [python, "-B", str(ROOT / "worker.py"), "finalize", "--deadline", str(deadline)])
        r.require(finalized["exit_code"] == 0, "Final guards/report persistence failed")
        final = b.read(ROOT / "runtime/finalized.json")
        r.require(r.sha(ROOT / "runtime/FINAL-NUMERICAL.json", deadline) == final["final_receipt_sha256"], "Final numerical receipt changed")
        b.verify_session(deadline)
        receipt = {"experiment": b.EXPERIMENT, "source_commit": b.SOURCE,
                   "status": "pending_post_persistence_terminal_verdict", "started": started, "deadline": deadline,
                   "stages": stages, "strict_quality_passed": final["strict_quality_passed"],
                   "final_numerical_receipt_sha256": final["final_receipt_sha256"],
                   "finalization_process_end_monotonic_seconds": finalized["end_monotonic_seconds"],
                   "root_go_sha256": args.go_sha256, "native_or_reference_acceptance": False}
        terminal = b.close_supervisor(receipt, deadline)
        print(json.dumps(terminal, allow_nan=False), flush=True)
        return 0 if terminal["strict_quality_passed"] else 2
    except BaseException as error:
        failure = {"experiment": b.EXPERIMENT, "status": "failed_or_refused", "strict_quality_passed": False,
                   "error": f"{type(error).__name__}:{error}"[:2000], "stages": stages,
                   "started": started, "failed_at_monotonic_seconds": time.monotonic(),
                   "outputs_retained": True, "retry_or_continuation_allowed": False, "native_or_reference_acceptance": False}
        if started is not None:
            failure["whole_case_elapsed_seconds"] = time.monotonic()-started
            # A negative post-cap receipt can preserve failure, never create a successful late grade.
            path = ROOT / "runtime/FAILED.json"
        else:
            path = ROOT / "checks" / f"refusal-{time.monotonic_ns()}.json"
        if not path.exists():
            b.persist(path, failure)
        print(json.dumps(failure, allow_nan=False), flush=True)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
