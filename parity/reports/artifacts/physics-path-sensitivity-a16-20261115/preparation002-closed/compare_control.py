"""Compare new output with saved control values; never re-evaluate the control."""
import argparse
import itertools
import math
from pathlib import Path

import consumer16 as c
import resources as r

ROOT = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--deadline", type=float, required=True)
    args = parser.parse_args()
    r.deadline_check(args.deadline)
    binding = c.read(ROOT / "READONLY-BINDING.json")
    repo = Path(binding["repo"])
    old = repo / "parity/reports/artifacts/closed-stitch-harbor-20261115/experiment/runs/base"
    for name in ("rest-metadata.json", "analysis.json", "trajectory.jsonl"):
        path = old / name
        c.require(c.sha(path) == binding["files"][str(path.relative_to(repo))], "Control bytes changed")
    original = c.read(old / "analysis.json")
    actual = c.read(ROOT / "runs/base/analysis16.json")
    c.require(actual["status"] == "analyzed_complete" and len(actual["rows"]) == len(original["rows"]) == 181,
              "Complete independent rows required")
    masses = c.read(old / "rest-metadata.json")["masses_kg"]
    total_mass = sum(masses)
    rows = []
    with (old / "trajectory.jsonl").open() as before, (ROOT / "runs/base/trajectory.jsonl").open() as after:
        for frame, pair in enumerate(itertools.zip_longest(before, after)):
            r.deadline_check(args.deadline)
            c.require(frame <= 180 and all(line is not None for line in pair), "State counts differ")
            first, second = map(c.strict_loads, pair)
            c.require(first["global_frame"] == second["global_frame"] == frame, "State frames differ")
            c.require(len(first["positions_m"]) == len(second["positions_m"]) == len(masses), "Node counts differ")
            squared = [c.metrics.dot(delta, delta) for delta in
                       (c.metrics.sub(a, b) for a, b in zip(first["positions_m"], second["positions_m"]))]
            if frame == 0:
                c.require(c.same(first["positions_m"], second["positions_m"]) and
                          c.same(first["velocities_m_per_s"], second["velocities_m_per_s"]), "Initial state differs")
                left, right = dict(first["diagnostics"]), dict(second["diagnostics"])
                left.pop("step_seconds"); right.pop("step_seconds")
                c.require(c.same(left, right), "Initial physical diagnostics differ")
            old_row, new_row = original["rows"][frame], actual["rows"][frame]
            old_strain = old_row["independent_material_axis"]["peak"]["absolute_strain"]
            new_strain = new_row["independent_material_axis"]["peak"]["absolute_strain"]
            rows.append({"frame": frame, "old_peak": old_row["independent_material_axis"]["peak"],
                         "new_peak": new_row["independent_material_axis"]["peak"],
                         "peak_strain_difference": new_strain-old_strain,
                         "max_nodal_difference_m": math.sqrt(max(squared)),
                         "mass_weighted_rms_difference_m": math.sqrt(sum(m*d for m, d in zip(masses, squared))/total_mass),
                         "old_energy_J": old_row["independent_material_axis"]["membrane_energy_J"],
                         "new_energy_J": new_row["independent_material_axis"]["membrane_energy_J"],
                         "old_rms_speed_m_per_s": old_row["rms_speed_m_per_s"],
                         "new_rms_speed_m_per_s": new_row["rms_speed_m_per_s"]})
    c.require(len(rows) == 181, "Exactly181 states required")
    window = [row for row in rows if 20 <= row["frame"] <= 45]
    result = {"status": "comparison_complete", "new_control_evaluation": False,
              "control_source": "2e9795a3cb92c79ad3b406e675dcbdb49c60ce67",
              "candidate_core_source": c.SOURCE, "control_original_quality": "failed",
              "old_failed_frames": original["failed_frames_by_screen"],
              "new_failed_frames": actual["failed_frames_by_screen"],
              "old_peak": max((row["old_peak"] | {"frame": row["frame"]} for row in rows), key=lambda p: p["absolute_strain"]),
              "new_peak": max((row["new_peak"] | {"frame": row["frame"]} for row in rows), key=lambda p: p["absolute_strain"]),
              "max_absolute_same_frame_peak_difference": max(abs(row["peak_strain_difference"]) for row in rows),
              "max_nodal_difference_m": max(row["max_nodal_difference_m"] for row in rows),
              "transient_strain_sensitivity_above_0_005": any(abs(row["peak_strain_difference"]) > .005 for row in window),
              "nodal_sensitivity_above_0_001_m": any(row["max_nodal_difference_m"] > .001 for row in rows),
              "candidate_strict_screen_passed": actual["strict_delivery_passed"], "rows": rows,
              "limits": "A single numerical-path comparison; no convergence, sufficient fix, native/contact completeness or reference acceptance."}
    r.deadline_check(args.deadline)
    data = r.encoded(result)
    r.require(r.numerical_bytes() + len(data) + r.RECEIPT_RESERVE < r.CAP_BYTES, "Comparison exceeds output stop budget")
    r.write_new(ROOT / "runs/base/comparison.json", data)
    print({key: result[key] for key in ("status", "old_peak", "new_peak", "candidate_strict_screen_passed")})


if __name__ == "__main__":
    main()
