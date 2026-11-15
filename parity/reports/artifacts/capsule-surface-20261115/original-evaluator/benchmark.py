"""Full already-saved old-gold analysis only, never launches any producer."""
import argparse
import json
from pathlib import Path
import sys
import time

sys.dont_write_bytecode = True
import evaluator as e


def main(output):
    before = e.verify_preparation()
    e.verify_readonly_inputs()
    rows = []
    for case in e.REVISIONS:
        started = time.monotonic()
        report = e.analyze_saved(case, e.GOLD / "runs" / case, legacy=True)
        path = e.ROOT / f"old-gold-{case}-analysis.json"
        e.write_report(path, report)
        digest = e.sha(path)
        inspected = e.read(path)
        assert inspected["status"] == "analyzed_complete" and len(inspected["rows"]) == 181
        failures = inspected["failed_frames_by_screen"]["material_axis_strain_below_0_15"]
        assert failures == ([29] if case == "base" else [])
        witness = inspected["rows"][180]["independent_full_static_capsule"]
        historical = e.HISTORY[f"{case}_frame180_capsule_surface_m"]
        assert witness["penetration_lower_m"] <= historical <= witness["penetration_upper_m"]
        row = {"case": case, "kind": "Already-savedold3aa gold only, not candidate physics", "states": 181,
               "analysis_export_hash_inspection_seconds": time.monotonic()-started,
               "analysis_path": str(path.relative_to(e.ROOT)), "analysis_sha256": digest,
               "strain_failed_frames": failures, "exact_attribution_failed_frames": inspected["exact_attribution_failed_frames"],
               "diagnostic_disagreement_frames": inspected["diagnostic_disagreement_frames"],
               "historical_surface_failure_retained_m": historical,
               "independent_unknown_pair_total": sum(r["independent_full_static_capsule"]["uncertain_pairs"] for r in inspected["rows"]),
               "policy_zero_proof_all_frames": all(r["product_uncertainty_policy_zero_certificate"]["zero_count_proved"] for r in inspected["rows"]),
               "decimal100_pair_total": sum(r["independent_full_static_capsule"]["counts"]["decimal100_pairs"] for r in inspected["rows"])}
        rows.append(row)
        print(json.dumps(row), flush=True)
    assert e.verify_preparation() == before
    e.verify_readonly_inputs()
    result = {"status": "passed_saved_gold_only", "new_garment_execution": False, "new_product_or_engine_execution": False,
              "new_physics_initializations_or_steps": 0, "source_input_driver_bindings_exact_before_after": True,
              "cases": rows, "fixed_whole_case_cap_seconds": 420.,
              "feasibility": "Saved-stateanalysis cost only; candidate producer/garment cost remains unmeasured and total420second cap unchanged",
              "scope": e.SCOPE, "historical_failures": e.HISTORY}
    e.write_report(output, result)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    main(parser.parse_args().output)
