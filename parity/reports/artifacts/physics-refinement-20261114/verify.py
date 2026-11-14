#!/usr/bin/env python3
"""Verify retained garment refinement evidence without simulation or file writes."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess
import struct

ROOT = Path(__file__).resolve().parent
COMMIT = "16e680ad774119e1ce3609b7d6c42c317f8677ef"

def read(path):
    return json.loads(path.read_text())

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def near(a, b):
    assert math.isfinite(a) and math.isfinite(b) and abs(a - b) <= 1e-12, (a, b)

def sub(a, b):
    return [x - y for x, y in zip(a, b)]

def dot(a, b):
    return sum(x * y for x, y in zip(a, b))

def drift(a, b, mass):
    squared = [dot(sub(x, y), sub(x, y)) for x, y in zip(a, b)]
    return {"max_m": math.sqrt(max(squared)), "mass_weighted_rms_m": math.sqrt(sum(m * d for m, d in zip(mass, squared)) / sum(mass))}

def measure(meta, positions):
    pieces = {p["id"]: p for p in meta["project"]["pieces"]}
    materials = {m["id"]: m for m in meta["project"]["materials"]}
    peak = None
    extension, compression, energies = -math.inf, math.inf, [0., 0., 0.]
    for index, triangle in enumerate(meta["triangles"]):
        piece = pieces[meta["piece_ids"][triangle[0]]]
        material = materials[piece["material"]]
        uv = [meta["uv_m"][v] for v in triangle]
        world = [positions[v] for v in triangle]
        a, b = sub(uv[1], uv[0]), sub(uv[2], uv[0])
        det = a[0] * b[1] - b[0] * a[1]
        assert abs(det) > 1e-12
        e, f = sub(world[1], world[0]), sub(world[2], world[0])
        u = [(e[j] * b[1] - f[j] * a[1]) / det for j in range(3)]
        v = [(f[j] * a[0] - e[j] * b[0]) / det for j in range(3)]
        sine, cosine = math.sin(piece["grain_angle"]), math.cos(piece["grain_angle"])
        warp = [cosine * x + sine * y for x, y in zip(u, v)]
        weft = [-sine * x + cosine * y for x, y in zip(u, v)]
        signed = [math.sqrt(dot(axis, axis)) - 1. for axis in [warp, weft]]
        for axis, value in enumerate(signed):
            extension, compression = max(extension, value), min(compression, value)
            if peak is None or abs(value) > peak["absolute_strain"]:
                peak = {"triangle_index": index, "piece_id": piece["id"], "axis": ["warp", "weft"][axis], "stiffness_N_per_m": [material["warp_stiffness"], material["weft_stiffness"]][axis], "vertices_0based": triangle, "signed_strain": value, "absolute_strain": abs(value)}
        constraints = [(dot(warp, warp) - 1) / 2, (dot(weft, weft) - 1) / 2, dot(warp, weft)]
        for j, stiffness in enumerate([material["warp_stiffness"], material["weft_stiffness"], material["shear_stiffness"]]):
            energies[j] += abs(det) * stiffness * constraints[j] ** 2 / 4
    return {"peak": peak, "maximum_signed_extension": extension, "minimum_signed_compression": compression, "membrane_energy_J": dict(zip(["warp", "weft", "shear", "total"], energies + [sum(energies)]))}

def quality(d):
    return d["finite"] and d["seam_assembly_remaining"] == 0 and d["max_stretch_error"] < .15 and d["max_seam_error"] < .005 and d["max_contact_error"] < .002

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "source")
    parser.add_argument("--runs", type=Path, default=ROOT / "runs")
    args = parser.parse_args()
    manifest, provenance = read(ROOT / "MANIFEST.json"), read(ROOT / "provenance.json")
    for relative, item in manifest["files"].items():
        assert sha(ROOT / relative) == item["published_sha256"], relative
    assert subprocess.check_output(["git", "-C", str(args.source), "rev-parse", "HEAD"], text=True).strip() == COMMIT
    assert not subprocess.check_output(["git", "-C", str(args.source), "status", "--porcelain"], text=True).strip()
    for relative, expected in provenance["files"].items():
        path = ROOT / relative if relative.startswith("harness/") else args.source / relative.removeprefix("source/")
        assert sha(path) == expected, relative
    for relative, item in read(ROOT / "retained-artifact-hashes.json").items():
        assert item["expected"] == item["actual"] == sha(args.source / relative), relative
    meta = read(args.runs / "rest-metadata.json")
    fixture_project = read(args.source / "fixtures/grain-shirt.weftforge")["project"]
    # Document colors are f32; serde_json::Value exposes their promoted f64 values.
    for material in fixture_project["materials"]:
        material["color"] = [struct.unpack("f", struct.pack("f", value))[0] for value in material["color"]]
    assert meta["project"] == fixture_project
    records = [json.loads(line) for line in (args.runs / "trajectory.jsonl").read_text().splitlines()]
    stored = read(ROOT / "analysis.json")
    expected = {(r["phase"], r["frame"]): r for r in stored["samples"]}
    assert len(records) == len(expected) == 29
    assert {(r["phase"], r["frame"]) for r in records} == set(expected)
    base = next(r for r in records if r["phase"] == "baseline_drape_8x32" and r["frame"] == 120)
    mass = meta["masses_kg"]
    is_retained = args.runs.resolve() == (ROOT / "runs").resolve()
    previous, ends = {}, {}
    for r in records:
        calculated = measure(meta, r["positions_m"])
        near(calculated["peak"]["absolute_strain"], r["diagnostics"]["max_stretch_error"])
        near(sum(mass), r["total_mass_kg"])
        near(math.sqrt(2 * r["diagnostics"]["kinetic_energy"] / sum(mass)), r["rms_speed_m_per_s"])
        assert quality(r["diagnostics"]) == r["quality_passed"]
        if r["phase"] in ["continuation_8x32", "iteration_8x64", "substep_16x32"] and r["frame"] == 0:
            assert r["positions_m"] == base["positions_m"] and r["velocities_m_per_s"] == base["velocities_m_per_s"]
        if is_retained:
            old = expected[r["phase"], r["frame"]]
            for key in ["maximum_signed_extension", "minimum_signed_compression"]:
                near(calculated[key], old[key])
            for key in ["triangle_index", "piece_id", "axis", "stiffness_N_per_m", "vertices_0based"]:
                assert calculated["peak"][key] == old["peak"][key]
            for key in calculated["membrane_energy_J"]:
                near(calculated["membrane_energy_J"][key], old["membrane_energy_J"][key])
            for key, value in drift(r["positions_m"], base["positions_m"], mass).items():
                near(value, old["nodal_drift_from_t3_baseline"][key])
            if r["phase"] in previous:
                for key, value in drift(r["positions_m"], previous[r["phase"]], mass).items():
                    near(value, old["nodal_drift_from_previous_sample"][key])
        previous[r["phase"]] = r["positions_m"]
        ends[r["phase"]] = r
    near(base["diagnostics"]["max_stretch_error"], .26930346146570994)
    if is_retained:
        obj = [line.split() for line in (args.source / "fixtures/grain-shirt-drape.obj").read_text().splitlines()]
        vertices = [list(map(float, row[1:])) for row in obj if row and row[0] == "v"]
        uv = [list(map(float, row[1:])) for row in obj if row and row[0] == "vt"]
        triangles = [[int(x.split("/")[0]) - 1 for x in row[1:]] for row in obj if row and row[0] == "f"]
        old = stored["retained_native_OBJ_comparison"]
        near(drift(vertices, base["positions_m"], mass)["max_m"], old["max_nodal_difference_m"])
        near(max(abs(a-b) for x,y in zip(uv,meta["uv_m"]) for a,b in zip(x,y)), old["max_UV_component_difference_m"])
        assert triangles == meta["triangles"] and old["triangle_indices_match_exactly"]
    control, comparisons = ends["continuation_8x32"], {}
    for phase in ["continuation_8x32", "iteration_8x64", "substep_16x32"]:
        r = ends[phase]
        delta = r["diagnostics"]["max_stretch_error"] - control["diagnostics"]["max_stretch_error"]
        moved = drift(r["positions_m"], control["positions_m"], mass)
        if is_retained:
            old = stored["comparison_to_t4_control"][phase]
            near(delta, old["strain_difference_from_t4_control"])
            for key, value in moved.items():
                near(value, old["nodal_difference_from_t4_control"][key])
            assert (abs(delta) > .005) == old["strain_sensitivity_exceeded"]
            assert (moved["max_m"] > .001) == old["displacement_sensitivity_exceeded"]
        comparisons[phase] = {"strain_delta_vs_control": delta, "nodal_difference_vs_control": moved}
    cases = [{"phase": phase, "status": "PASSED" if quality(r["diagnostics"]) else "FAILED", "max_axis_strain": r["diagnostics"]["max_stretch_error"], "strain_delta_vs_retained": r["diagnostics"]["max_stretch_error"] - expected[phase, r["frame"]]["diagnostics"]["max_stretch_error"]} for phase, r in ends.items()]
    if is_retained:
        assert all(c["status"] == "FAILED" for c in cases)
    print(json.dumps({"checks": "passed", "scope": "bounded historical source 16e diagnostic; no equilibrium or parity result", "source_commit": COMMIT, "retained": is_retained, "cases": cases, "sensitivity_comparisons": comparisons}, indent=2))

if __name__ == "__main__":
    main()
