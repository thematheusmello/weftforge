"""Bounded metadata/analysis/finalization phases; never starts a cloth producer."""
import argparse
import os
import time

import common as b
from common import ROOT, REPO, c, r
from metadata_adapter import check_current_metadata


def initialized_state(meta, state, compared):
    r.require(set(state) == {"positions_m", "velocities_m_per_s", "simulation_time_s", "diagnostics", "cloth_step_calls", "metadata_compared_before_steps"}, "Wrong initialized-state schema")
    r.require(c.same(state["positions_m"], meta["rest_positions_m"]), "Initialization moved the rest mesh")
    r.require(c.same(state["velocities_m_per_s"], [[0., 0., 0.] for _ in meta["piece_ids"]]), "Nonzero or differently typed initial velocity")
    r.require(c.same(state["simulation_time_s"], 0.) and type(state["cloth_step_calls"]) is int and state["cloth_step_calls"] == 0, "Initialization performed a step")
    r.require(state["metadata_compared_before_steps"] is compared, "Wrong initialized metadata gate status")
    r.require(state["diagnostics"]["finite"] is True and c.same(state["diagnostics"]["simulation_time"], 0.), "Invalid initial diagnostics")


def metadata(deadline):
    b.verify_session(deadline)
    meta = b.read(r.safe_file("preflight-base/rest-metadata.json", ROOT))
    state = b.read(r.safe_file("preflight-base/initialized-state.json", ROOT))
    check = check_current_metadata(meta, "base")
    initialized_state(meta, state, False)
    (ROOT / "expected").mkdir()
    approved_sha = b.persist(ROOT / "expected/approved-metadata.json", meta, deadline)
    b.persist(ROOT / "runtime/initialized-original-gate.json", {
        "status": "original_rest_independently_verified", "source_commit": b.SOURCE,
        "preflight_rest_sha256": r.sha(ROOT / "preflight-base/rest-metadata.json", deadline),
        "initialized_state_sha256": r.sha(ROOT / "preflight-base/initialized-state.json", deadline),
        "approved_metadata_sha256": approved_sha, "cloth_step_calls": 0,
        "comparison_is_to_original_gold_not_only_current_initialization": True, "check": check,
    }, deadline)
    return 0


def analyze(deadline):
    _, session, receipt = b.verify_producer_receipt(deadline)
    meta = b.read(ROOT / "runs/base/rest-metadata.json")
    initialized_state(meta, b.read(ROOT / "runs/base/initialized-state.json"), True)
    c.REPO, c.EXPERIMENT = REPO, ROOT
    c.REVISIONS = {"base": 20}
    c.OUTPUT_DIRECTORIES = r.OUTPUT_DIRECTORIES
    c.check_metadata = check_current_metadata
    result = c.analyze_saved("base", ROOT / "runs/base", deadline=deadline)
    result.update(source_commit=b.SOURCE, experiment=b.EXPERIMENT,
                  producer_receipt_sha256=r.sha(ROOT / "runtime/producer-receipt.json", deadline),
                  actual_current_numerical_producer_executed=True, native_UI_executed=False,
                  case_start_monotonic_seconds=session["started"], old_cases_regraded=False,
                  fresh_producer_qualification="Inherited saved-state wording describes this independent reader. These 181 states were just produced by the separately fresh current candidate run after a zero-step original-rest preflight; no archived dynamic state was loaded.")
    c.write_report(ROOT / "runs/base/analysis.json", result)
    with (ROOT / "runs/base/analysis.json").open("rb") as stream:
        os.fsync(stream.fileno())
    digest = r.sha(ROOT / "runs/base/analysis.json", deadline)
    b.verify_producer_receipt(deadline)
    b.persist(ROOT / "runtime/analysis-written.json", {
        "analysis_sha256": digest, "persisted_then_hash_checked_at_monotonic_seconds": time.monotonic(),
        "deadline": deadline, "current_case_only": True,
    }, deadline)
    if result["status"] != "analyzed_complete" or result["errors"] or result["diagnostic_disagreement_frames"] or result["blocked_diagnostic_verification_frames"]:
        return 1
    return 0 if result["strict_delivery_passed"] else 2


def finalize(deadline):
    frozen, session, producer = b.verify_producer_receipt(deadline)
    analysis = b.read(r.safe_file("runs/base/analysis.json", ROOT))
    written = b.read(r.safe_file("runtime/analysis-written.json", ROOT))
    r.require(analysis["source_commit"] == b.SOURCE and analysis["actual_current_numerical_producer_executed"] is True and analysis["old_cases_regraded"] is False, "Fresh candidate producer/source qualification missing")
    r.require(r.sha(ROOT / "runs/base/analysis.json", deadline) == written["analysis_sha256"], "Independent report changed")
    stage = b.read(r.safe_file("runtime/analysis-stage.json", ROOT))
    r.require(stage["exit_code"] in (0, 2) and not stage["stop_reason"] and stage["end_monotonic_seconds"] < deadline, "Independent analysis did not finish under the cap")
    r.require(analysis["status"] == "analyzed_complete" and len(analysis["rows"]) == 181 and not analysis["errors"] and not analysis["diagnostic_disagreement_frames"] and not analysis["blocked_diagnostic_verification_frames"], "Independent verification incomplete")
    r.require(type(analysis["strict_delivery_passed"]) is bool and (stage["exit_code"] == 0) is analysis["strict_delivery_passed"], "Quality flag and analysis exit disagree")
    b.verify_session(deadline)
    outputs = {str(path.relative_to(ROOT)): r.sha(path, deadline) for path in r.numerical_files()}
    r.require(r.numerical_bytes() + 2*r.RECEIPT_RESERVE < b.CAP_BYTES, "Final receipt reserve unavailable")
    final = {"experiment": b.EXPERIMENT, "source_commit": b.SOURCE,
             "status": "complete" if analysis["strict_delivery_passed"] else "complete_quality_failed",
             "strict_quality_passed": analysis["strict_delivery_passed"],
             "quality_status": analysis["quality_status"], "failed_frames_by_screen": analysis["failed_frames_by_screen"],
             "analysis_sha256": written["analysis_sha256"], "output_hashes_before_final_receipt": outputs,
             "started": session["started"], "deadline": deadline,
             "guard_completion_before_receipt_monotonic_seconds": time.monotonic(),
             "two_fresh_initializations": True, "init_only_steps": 0, "actual_dynamics_runs": 1,
             "native_or_reference_acceptance": False, "old_cases_regraded": False,
             "continuation_allowed": False, "authoritative_terminal_clock": "supervisor observes this process after receipt persistence and then checks its own receipt persistence"}
    final_sha = b.persist(ROOT / "runtime/FINAL-NUMERICAL.json", final, deadline)
    b.verify_session(deadline)
    b.persist(ROOT / "runtime/finalized.json", {"final_receipt_sha256": final_sha,
              "final_receipt_persisted_and_hash_checked_at_monotonic_seconds": time.monotonic(),
              "strict_quality_passed": analysis["strict_delivery_passed"]}, deadline)
    r.deadline_check(deadline)
    return 0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("phase", choices=["metadata", "analyze", "finalize"])
    parser.add_argument("--deadline", type=float, required=True)
    args = parser.parse_args()
    r.require(c.number(args.deadline), "Invalid absolute deadline")
    r.deadline_check(args.deadline)
    return {"metadata": metadata, "analyze": analyze, "finalize": finalize}[args.phase](args.deadline)


if __name__ == "__main__":
    raise SystemExit(main())
