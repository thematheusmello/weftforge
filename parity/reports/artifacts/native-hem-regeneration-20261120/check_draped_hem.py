"""Measure both garments' native phase endpoints independently, in material coordinates."""
import json
import math
from pathlib import Path
import runpy
import sys


def check(repo, case):
    read_obj = runpy.run_path(str(repo / "parity/reports/artifacts/static-glb-20261119/compare_glb.py"))["read_obj"]
    measure = runpy.run_path(str(repo / "scripts/check_drape.py"))["measure"]
    metrics = {}
    trajectories = {}
    corners = {}
    rest = {}
    for kind, v, baseline in [("original", 0.96, "original-step"), ("extended", 0.91, "edited-rest")]:
        directory = case / "full-fit" / kind
        project = directory / "Harbor.weftforge"
        data = json.loads(project.read_bytes())["project"]
        assert len(data["pieces"]) == 2 and len(data["seams"]) == 8
        assert [point["position"] for point in data["points"][:2]] == [[-1.08, v], [-0.36, v]]
        original_attributes, original_faces = read_obj(case / (baseline + ".obj"))
        rest[kind] = v
        for phase in ("assembled", "draped"):
            path = directory / (phase + ".obj")
            report = measure(project, path)
            attr, faces = read_obj(path)
            assert faces == original_faces and attr["vt"] == original_attributes["vt"]
            front = {i for face in faces["2"] for i in face}
            ids = [[i for i in front if attr["vt"][i] == (u, v)] for u in (-1.08, -0.36)]
            assert all(len(matches) == 1 for matches in ids)
            corners[kind, phase] = [attr["v"][matches[0]] for matches in ids]
            metrics[kind + "_" + phase] = {
                key: report[key] for key in (
                    "status", "vertices", "triangles", "max_material_axis_strain",
                    "max_seam_gap_m", "max_avatar_ground_thickness_error_m", "criteria",
                )
            }
        trajectory = json.loads((directory / "trajectory.json").read_bytes())
        assert trajectory["quality_states_checked"] == 181 and trajectory["solver_error"] is None
        assert trajectory["criteria"]["max_stretch_error_below"] == 0.15
        assert trajectory["status"] == "failed" and trajectory["export_written"] is False
        assert trajectory["wall_seconds"] < 420
        assert [phase["steps_completed"] for phase in trajectory["phases"]] == [60, 120]
        for name, phase in zip(("assembled", "draped"), trajectory["phases"], strict=True):
            native_peak = metrics[kind + "_" + name]["max_material_axis_strain"]
            assert abs(phase["diagnostics"]["max_stretch_error"] - native_peak) < 1e-12
        first = trajectory["first_quality_failure"]
        assert first["state"] == trajectory["first_quality_failure_state"]
        assert first["diagnostics"]["peak_strain"]["axis"] == "warp"
        assert first["diagnostics"]["max_stretch_error"] >= 0.15
        trajectories[kind] = {
            "checked_states": trajectory["quality_states_checked"],
            "first_failed_state": first["state"],
            "first_failed_time_s": first["diagnostics"]["simulation_time"],
            "triangle": first["diagnostics"]["peak_strain"]["triangle"],
            "piece": first["peak_strain_piece"]["name"],
            "peak_strain_at_first_failure": first["diagnostics"]["max_stretch_error"],
            "status": trajectory["status"],
        }
    delta = {}
    for phase in ("assembled", "draped"):
        pairs = zip(corners["original", phase], corners["extended", phase], strict=True)
        delta[phase] = [dict(y_change_m=new[1] - old[1], distance_m=math.dist(old, new)) for old, new in pairs]
    return {
        "front_rest_hem_height_change_m": rest["extended"] - rest["original"],
        "left_right_hem_world_changes_by_phase": delta,
        "phase_endpoint_metrics": metrics,
        "complete_trajectory_checks": trajectories,
    }


if __name__ == "__main__":
    print(json.dumps(check(Path(sys.argv[1]), Path(__file__).resolve().parent), indent=2))
