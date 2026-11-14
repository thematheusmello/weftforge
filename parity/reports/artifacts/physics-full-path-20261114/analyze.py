"""Verify and independently measure the bounded full assembly-path experiment."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess
import sys

sys.dont_write_bytecode = True
from measure import drift, measure, residuals, self_check

ROOT = Path(__file__).resolve().parent
CASES = {"control_8x32": (8, 32), "iteration_8x64": (8, 64), "substep_16x32": (16, 32)}


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def near(a, b):
    assert math.isfinite(a) and math.isfinite(b) and abs(a - b) <= 1e-12, (a, b)


def compute(source, runs):
    self_check()
    plan = read(ROOT / "plan.json")
    assert sha(ROOT / "plan.json") == read(ROOT / "plan.sha256")["plan.json"]
    assert sha(ROOT / "source-manifest.json") == plan["source_manifest_sha256"]
    assert sha(ROOT / "retained-inputs.json") == plan["retained_input_manifest_sha256"]
    assert subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip() == plan["source_commit"]
    assert not subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip()
    for relative, expected in read(ROOT / "source-manifest.json")["files"].items():
        assert sha(source / relative) == expected, relative
    for relative, expected in read(ROOT / "retained-inputs.json").items():
        assert sha(source / relative) == expected, relative
    for relative, expected in plan["frozen_code_sha256"].items():
        assert sha(ROOT / relative) == expected, relative
    meta = read(runs / "rest-metadata.json")
    fixture_project = read(source / plan["fixture"])["project"]
    for material in fixture_project["materials"]:
        material["color"] = [struct.unpack("f", struct.pack("f", v))[0] for v in material["color"]]
    assert meta["project"] == fixture_project
    mass = meta["masses_kg"]
    assert len(meta["rest_positions_m"]) == len(meta["uv_m"]) == len(meta["piece_ids"]) == len(mass)
    assert all(math.isfinite(m) and m > 0 for m in mass)
    pieces = {p["id"]: p for p in fixture_project["pieces"]}
    materials = {m["id"]: m for m in fixture_project["materials"]}
    expected_mass = [0.] * len(mass)
    for tri in meta["triangles"]:
        uv = [meta["uv_m"][v] for v in tri]
        a, b = [[p[i] - uv[0][i] for i in range(2)] for p in uv[1:]]
        area = abs(a[0] * b[1] - a[1] * b[0]) / 2
        material = materials[pieces[meta["piece_ids"][tri[0]]]["material"]]
        for v in tri:
            expected_mass[v] += area * material["areal_density"] / 3
    for a, b in zip(mass, expected_mass):
        near(a, b)
    records = [json.loads(line) for line in (runs / "trajectory.jsonl").read_text().splitlines()]
    steps = [json.loads(line) for line in (runs / "steps.jsonl").read_text().splitlines()]
    assert len(records) == 39 and len(steps) == 540
    by_key = {(r["case"], r["global_frame"]): r for r in records}
    assert len(by_key) == len(records)
    assert set(by_key) == {(case, frame) for case in CASES for frame in range(0, 181, 15)}
    assert {(r["case"], r["global_frame"]) for r in steps} == {(case, frame) for case in CASES for frame in range(1, 181)}
    for row in steps:
        settings = row["settings"]
        assert (settings["substeps"], settings["iterations"]) == CASES[row["case"]]
        assert settings["dt"] == 1 / 60 and settings["seam_closure_speed"] == 1
        assert settings["gravity"] == ([0., 0., 0.] if row["global_frame"] <= 60 else [0., -9.81, 0.])
        near(row["diagnostics"]["simulation_time"], row["global_frame"] / 60)
        assert math.isfinite(row["measured_step_seconds"]) and row["measured_step_seconds"] >= 0
    calculated = {}
    summaries = []
    previous = {}
    for row in records:
        case, frame = row["case"], row["global_frame"]
        positions, velocities, diag = row["positions_m"], row["velocities_m_per_s"], row["diagnostics"]
        assert len(positions) == len(velocities) == len(mass)
        assert all(len(p) == 3 and all(math.isfinite(q) for q in p) for p in positions + velocities)
        if frame == 0:
            assert positions == meta["rest_positions_m"]
            assert all(q == 0 for v in velocities for q in v)
        near(diag["simulation_time"], frame / 60)
        measured = measure(meta, positions)
        gaps = residuals(meta, positions)
        near(measured["peak"]["absolute_strain"], diag["max_stretch_error"])
        near(gaps["sampled_stitch_gap_m"], diag["max_seam_error"])
        near(sum(m * sum(q * q for q in v) for m, v in zip(mass, velocities)) / 2, diag["kinetic_energy"])
        near(sum(mass), row["total_mass_kg"])
        near(math.sqrt(2 * diag["kinetic_energy"] / sum(mass)), row["rms_speed_m_per_s"])
        near(max(math.sqrt(sum(q * q for q in v)) for v in velocities), row["max_speed_m_per_s"])
        assert gaps["avatar_ground_penetration_m"] <= diag["max_contact_error"] + 1e-12
        quality = diag["finite"] and diag["seam_assembly_remaining"] == 0 and measured["peak"]["absolute_strain"] < .15 and gaps["sampled_stitch_gap_m"] < .005 and gaps["avatar_ground_penetration_m"] < .002 and diag["max_contact_error"] < .002
        summary = {key: row[key] for key in ["case", "global_frame", "phase", "case_wall_seconds", "diagnostics", "rms_speed_m_per_s", "max_speed_m_per_s", "total_mass_kg"]}
        summary.update(measured)
        summary["independent_geometric_residuals"] = gaps
        summary["quality_passed"] = quality
        if case in previous:
            summary["nodal_drift_from_previous_sample"] = drift(positions, previous[case], mass)
        previous[case] = positions
        calculated[case, frame] = summary
        summaries.append(summary)
    comparisons = []
    for frame in range(0, 181, 15):
        control = by_key["control_8x32", frame]
        for case in ["iteration_8x64", "substep_16x32"]:
            row = by_key[case, frame]
            delta = calculated[case, frame]["peak"]["absolute_strain"] - calculated["control_8x32", frame]["peak"]["absolute_strain"]
            moved = drift(row["positions_m"], control["positions_m"], mass)
            velocity = drift(row["velocities_m_per_s"], control["velocities_m_per_s"], mass)
            comparisons.append({"case": case, "global_frame": frame, "time_s": frame / 60, "max_axis_strain_difference_from_control": delta, "nodal_difference_from_control": moved, "velocity_difference_from_control": {"max_m_per_s": velocity["max_m"], "mass_weighted_rms_m_per_s": velocity["mass_weighted_rms_m"]}, "strain_sensitivity_exceeded": abs(delta) > .005, "displacement_sensitivity_exceeded": moved["max_m"] > .001})
    obj = [line.split() for line in (source / "fixtures/grain-shirt-drape.obj").read_text().splitlines()]
    vertices = [list(map(float, r[1:])) for r in obj if r and r[0] == "v"]
    uv = [list(map(float, r[1:])) for r in obj if r and r[0] == "vt"]
    triangles = [[int(x.split("/")[0]) - 1 for x in r[1:]] for r in obj if r and r[0] == "f"]
    assert len(vertices) == len(mass) and len(uv) == len(meta["uv_m"])
    native_drift = drift(vertices, by_key["control_8x32", 180]["positions_m"], mass)
    native_strain = measure(meta, vertices)["peak"]["absolute_strain"]
    uv_difference = max(abs(a - b) for x, y in zip(uv, meta["uv_m"]) for a, b in zip(x, y))
    native = {"artifact": "fixtures/grain-shirt-drape.obj", "max_nodal_difference_m": native_drift["max_m"], "mass_weighted_rms_nodal_difference_m": native_drift["mass_weighted_rms_m"], "max_uv_component_difference_m": uv_difference, "triangle_indices_match_exactly": triangles == meta["triangles"], "native_independent_max_axis_strain": native_strain, "strain_difference": calculated["control_8x32", 180]["peak"]["absolute_strain"] - native_strain}
    native["declared_match"] = native_drift["max_m"] <= 1e-12 and abs(native["strain_difference"]) <= 1e-12 and uv_difference <= 1e-12 and triangles == meta["triangles"]
    prior = source / "parity/reports/artifacts/physics-refinement-20261114"
    old_meta = read(prior / "runs/rest-metadata.json")
    old_records = [json.loads(line) for line in (prior / "runs/trajectory.jsonl").read_text().splitlines()]
    old_base = {}
    for row in old_records:
        if row["phase"].startswith("baseline_"):
            frame = row["frame"] + (60 if "drape" in row["phase"] else 0)
            if frame in old_base:
                assert row["positions_m"] == old_base[frame]["positions_m"] and row["velocities_m_per_s"] == old_base[frame]["velocities_m_per_s"]
            old_base[frame] = row
    historical = []
    for frame, old in old_base.items():
        row = by_key["control_8x32", frame]
        moved = drift(row["positions_m"], old["positions_m"], mass)
        velocity = drift(row["velocities_m_per_s"], old["velocities_m_per_s"], mass)
        delta = calculated["control_8x32", frame]["peak"]["absolute_strain"] - measure(old_meta, old["positions_m"])["peak"]["absolute_strain"]
        historical.append({"global_frame": frame, "time_s": frame / 60, "strain_difference": delta, "nodal_difference": moved, "velocity_difference": {"max_m_per_s": velocity["max_m"], "mass_weighted_rms_m_per_s": velocity["mass_weighted_rms_m"]}, "declared_match": abs(delta) <= 1e-12 and moved["max_m"] <= 1e-12 and velocity["max_m"] <= 1e-12})
    old_finals = {r["phase"]: r for r in old_records if r["phase"] in ["continuation_8x32", "iteration_8x64", "substep_16x32"] and r["frame"] == 60}
    old_control = old_finals["continuation_8x32"]
    old_comparisons = []
    for case in ["iteration_8x64", "substep_16x32"]:
        row = old_finals[case]
        delta = measure(old_meta, row["positions_m"])["peak"]["absolute_strain"] - measure(old_meta, old_control["positions_m"])["peak"]["absolute_strain"]
        moved = drift(row["positions_m"], old_control["positions_m"], mass)
        old_comparisons.append({"case": case, "time_s": 4., "scope": "historical continuation only from t=3s; different terminal time than new full-path experiment", "independent_max_axis_strain": measure(old_meta, row["positions_m"])["peak"]["absolute_strain"], "max_axis_strain_difference_from_control": delta, "nodal_difference_from_control": moved, "strain_sensitivity_exceeded": abs(delta) > .005, "displacement_sensitivity_exceeded": moved["max_m"] > .001})
    cases = read(runs / "completed-cases.json")
    assert len(cases) == 3
    for case in cases:
        assert case["completed_frames"] == 180 and case["final_diagnostics"] == by_key[case["case"], 180]["diagnostics"]
        times = sorted(r["measured_step_seconds"] for r in steps if r["case"] == case["case"])
        case["measured_step_median_seconds"] = (times[89] + times[90]) / 2
        case["measured_step_p95_seconds"] = times[math.ceil(.95 * len(times)) - 1]
        case["simulated_seconds_per_wall_second"] = 3 / case["case_wall_seconds"]
    return {"kind": "implementation_only_full_path_diagnostic", "source_commit": plan["source_commit"], "plan_sha256": sha(ROOT / "plan.json"), "all_support_contacts_enabled": True, "source_clean_and_hashes_verified": True, "input_project_exactly_matches_original_fixture": True, "independent_mass_max_difference_kg": max(abs(a - b) for a, b in zip(mass, expected_mass)), "sample_count": len(records), "step_count": len(steps), "native_original_comparison": native, "historical_control_metadata_exact": {key: meta[key] == old_meta[key] for key in ["project", "rest_positions_m", "uv_m", "triangles", "piece_ids", "masses_kg"]}, "historical_control_shared_time_comparisons": historical, "cases": cases, "samples": summaries, "same_time_refinement_comparisons": comparisons, "historical_t4_continuation_only_comparisons": old_comparisons, "limits": ["No force-balance or static-equilibrium measurement", "No general convergence from two resolutions in numerical parameters", "Analytic avatar/ground checks and sampled-stitch checks do not establish full cloth self-contact/edge-edge/CCD correctness", "Old continuation endpoints are t=4s; new full-path endpoints are t=3s", "Both assembly and drape are refined, so post-assembly differences cannot isolate assembly alone", "No calibrated fabric, reference execution or parity result"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "source")
    parser.add_argument("--runs", type=Path, default=ROOT / "runs")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    result = compute(args.source.resolve(), args.runs.resolve())
    if args.check:
        assert result == read(ROOT / "analysis.json"), "Retained derived analysis differs"
    else:
        with (ROOT / "analysis.json").open("x") as file:
            file.write(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"checks": "passed", "native_original_comparison": result["native_original_comparison"], "checkpoints": [{key: r[key] for key in ["case", "global_frame", "peak", "independent_geometric_residuals", "quality_passed"]} for r in result["samples"] if r["global_frame"] in [60, 180]], "same_time_comparisons": [r for r in result["same_time_refinement_comparisons"] if r["global_frame"] in [60, 180]], "timings": result["cases"]}, indent=2))


if __name__ == "__main__":
    main()
