"""One outer clock and persisted witness for the separately approved numerical case."""
import json
import os
from pathlib import Path
import signal
import sys
import time

OUT = Path(__file__).resolve().parent
REPO = OUT.parent / "positive-target-sewing-component-20261117"
PACKET = REPO / ".local/harbor-sewing-history-preparation-20261117"
sys.path.insert(0, str(PACKET))
import common as b
from common import r


def accepted(exit_code, stop_reason, complete, closed, deadline, volume):
    return exit_code in (0, 2) and stop_reason is None and complete and closed < deadline and volume <= b.CAP_BYTES


assert accepted(2, None, True, 419., 420., b.CAP_BYTES)
assert not accepted(0, None, True, 420., 420., 1)
assert not accepted(0, "cap", True, 1., 420., 1)
assert not accepted(1, None, False, 1., 420., 1)
assert not accepted(0, None, True, 1., 420., b.CAP_BYTES + 1)


def save(name, value):
    return b.persist(OUT / name, value)


def total_bytes(deadline=None):
    return r.directory_bytes((PACKET / name for name in r.OUTPUT_DIRECTORIES), shallow=(OUT,), deadline=deadline)


def main():
    go_path = PACKET / "ROOT-GO.json"
    go_sha = r.sha(go_path)
    frozen = b.verify_bound()
    b.validate_go(b.read(go_path), r.sha(PACKET / "FREEZE.json"))
    r.require(not (OUT / "RUNNING.json").exists(), "Outer case already started; no retry")
    r.require(not any((PACKET / n).exists() for n in r.OUTPUT_DIRECTORIES), "Numerical case already started")
    child_env = dict(os.environ, PATH=b.read(PACKET / "SOURCE-PREFLIGHT.json")["build_binding"]["parent_path"])
    argv = [frozen["python"]["binary_path"], "-B", str(PACKET / "controller.py"), "--go", "ROOT-GO.json", "--go-sha256", go_sha]
    started = time.monotonic()
    deadline = started + 420.
    save("OUTER-START.json", {"started": started, "deadline": deadline, "go_sha256": go_sha,
                             "supervisor_sha256": r.sha(Path(__file__)), "argv": argv})
    stage = r.monitor(argv, PACKET, OUT / "controller.stdout", OUT / "controller.stderr", deadline,
                      lambda: total_bytes(deadline), rss_cap=b.CAP_BYTES, reserve=4*r.RECEIPT_RESERVE, child_env=child_env,
                      on_start=lambda h: save("RUNNING.json", {"root_pid": os.getpid(), "started": started, "deadline": deadline, **h}))
    errors, cleanup = [], []
    for path in sorted((PACKET / "runtime").glob("*-started.json")):
        handle = b.read(path)
        pid = handle["pid"]
        try:
            os.killpg(pid, 0)
        except ProcessLookupError:
            cleanup.append({"pid": pid, "group_absent": True, "signal_sent": False})
        else:
            # Every group came from this attempt's exclusive stage-start receipt.
            os.killpg(pid, signal.SIGKILL)
            cleanup.append({"pid": pid, "group_absent": False, "signal_sent": True})
            errors.append("Owned stage group remained after controller terminal: " + str(pid))
    candidate, quality, hashes = False, False, {}
    try:
        r.require(stage["exit_code"] in (0, 2) and stage["stop_reason"] is None, "Controller operational failure")
        terminal = json.loads((OUT / "controller.stdout").read_text())
        r.require(terminal["status"] == "complete" and terminal["operational_complete"] is True, "No complete controller terminal")
        quality = terminal["strict_quality_passed"]
        r.require(type(quality) is bool and (stage["exit_code"] == 0) is quality, "Quality/exit mismatch")
        r.require(r.sha(PACKET / "runtime/SUPERVISOR.json", deadline) == terminal["supervisor_receipt_sha256"], "Controller receipt hash mismatch")
        session = b.read(PACKET / "runtime/SESSION.json")
        r.require(started <= session["started"] < session["deadline"] and session["deadline"] == session["started"] + 420., "Wrong inner shared clock")
        r.require(session["go_sha256"] == go_sha and r.sha(go_path, deadline) == go_sha, "GO changed")
        supervisor = b.read(PACKET / "runtime/SUPERVISOR.json")
        r.require([s["name"] for s in supervisor["stages"]] == ["init-only", "metadata", "producer", "analysis", "finalization"], "Wrong stage workload")
        for s in supervisor["stages"]:
            r.require(s["stop_reason"] is None and s["peak_rss_bytes"] <= b.CAP_BYTES and s["end_monotonic_seconds"] < session["deadline"], "Stage cap failure")
        final = b.read(PACKET / "runtime/FINAL-NUMERICAL.json")
        r.require(r.sha(PACKET / "runtime/FINAL-NUMERICAL.json", deadline) == supervisor["final_numerical_receipt_sha256"], "Final receipt mismatch")
        analysis = b.read(PACKET / "runs/base/analysis.json")
        r.require(len(analysis["rows"]) == 181 and analysis["status"] == "analyzed_complete" and analysis["strict_delivery_passed"] is quality, "Incomplete all-state analysis")
        r.require(not analysis["errors"] and not analysis["diagnostic_disagreement_frames"] and not analysis["blocked_diagnostic_verification_frames"], "Independent analysis incomplete")
        r.require(final["two_fresh_initializations"] is True and final["init_only_steps"] == 0 and final["actual_dynamics_runs"] == 1 and final["old_cases_regraded"] is False, "Wrong initialization/run scope")
        b.verify_bound(deadline)
        hashes = {str(p.relative_to(PACKET)): r.sha(p, deadline) for p in r.numerical_files()}
        candidate = not errors
    except BaseException as error:
        errors.append(f"{type(error).__name__}:{error}")
    witness = {"status": "pending_external_post_persistence_clock_verdict", "source_commit": b.SOURCE,
               "started": started, "deadline": deadline, "stage": stage, "owned_groups": cleanup,
               "operational_candidate": candidate, "strict_quality_passed": quality, "errors": errors,
               "numerical_hashes": hashes, "go_sha256": go_sha, "native_or_reference_acceptance": False,
               "retry_or_continuation_allowed": False}
    witness_sha = save("ROOT-TERMINAL-WITNESS.json", witness)
    volume = total_bytes(deadline)
    closed = time.monotonic()
    result = {"root_operational_accepted": accepted(stage["exit_code"], stage["stop_reason"], candidate, closed, deadline, volume),
              "strict_quality_passed": quality, "elapsed_through_persisted_root_witness_seconds": closed-started,
              "witness_sha256": witness_sha, "root_controller_peak_rss_bytes": stage["peak_rss_bytes"],
              "total_counted_bytes": volume, "errors": errors, "source_commit": b.SOURCE}
    result_sha = save("ROOT-RESULT.json", result)
    volume = total_bytes(deadline)
    closed = time.monotonic()
    result["root_operational_accepted"] = result["root_operational_accepted"] and closed < deadline and volume <= b.CAP_BYTES
    result.update(root_result_sha256=result_sha, total_counted_bytes_after_root_result_persistence=volume,
                  authoritative_elapsed_after_all_persistence_seconds=closed-started)
    print(json.dumps(result, allow_nan=False), flush=True)
    return (0 if quality else 2) if result["root_operational_accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
