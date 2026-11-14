#!/usr/bin/env python3
"""Verify retained native output with the Python standard library; run no simulator.

Usage: verify_native_saved.py EVIDENCE_DIRECTORY ORIGINAL_CONTROL_DIRECTORY
The control directory contains the frozen rest-metadata.json and trajectory.jsonl.
"""

import datetime
import hashlib
import json
import math
from pathlib import Path
import struct
import sys


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def parse_obj(text):
    positions, uv, normals, triangles, groups, position_text, uv_text = ([] for _ in range(7))
    piece = None
    for line in text.splitlines():
        fields = line.split()
        if not fields or fields[0] == "#":
            continue
        kind, values = fields[0], fields[1:]
        if kind in ("v", "vt", "vn"):
            assert len(values) == (2 if kind == "vt" else 3), line
            numbers = list(map(float, values))
            assert all(math.isfinite(value) for value in numbers), line
            {"v": positions, "vt": uv, "vn": normals}[kind].append(numbers)
            if kind == "v":
                position_text.append(values)
            if kind == "vt":
                uv_text.append(values)
        elif kind == "g":
            assert len(values) == 1 and values[0].startswith("piece_"), line
            piece = int(values[0][6:])
        elif kind == "f":
            assert len(values) == 3 and piece is not None, line
            indices = [list(map(int, value.split("/"))) for value in values]
            assert all(len(value) == 3 and value[0] == value[1] == value[2] > 0 for value in indices), line
            triangles.append([value[0] - 1 for value in indices])
            groups.append(piece)
        else:
            raise AssertionError(f"Unexpected OBJ record: {kind}")
    assert positions and len(positions) == len(uv) == len(normals)
    assert triangles and all(0 <= i < len(positions) for tri in triangles for i in tri)
    return positions, uv, triangles, groups, position_text, uv_text


def material_peak(meta, positions):
    # Same independently reviewed UV inverse and grain rotation; no production analyzer.
    pieces = {piece["id"]: piece for piece in meta["project"]["pieces"]}
    peak = None
    for index, tri in enumerate(meta["triangles"]):
        assert len({meta["piece_ids"][i] for i in tri}) == 1
        piece = pieces[meta["piece_ids"][tri[0]]]
        a, b, c = [meta["uv_m"][i] for i in tri]
        ab, ac = [b[i] - a[i] for i in range(2)], [c[i] - a[i] for i in range(2)]
        determinant = ab[0] * ac[1] - ab[1] * ac[0]
        assert abs(determinant) > 1e-12
        inverse = [[ac[1] / determinant, -ac[0] / determinant], [-ab[1] / determinant, ab[0] / determinant]]
        gradients = [[-(inverse[0][0] + inverse[1][0]), -(inverse[0][1] + inverse[1][1])], *inverse]
        co, si = math.cos(piece["grain_angle"]), math.sin(piece["grain_angle"])
        coefficients = [[co * u + si * v, -si * u + co * v] for u, v in gradients]
        relative = [[positions[i][k] - positions[tri[0]][k] for k in range(3)] for i in tri]
        for axis, label in enumerate(("warp", "weft")):
            vector = [sum(relative[j][k] * coefficients[j][axis] for j in range(3)) for k in range(3)]
            strain = math.sqrt(sum(value * value for value in vector)) - 1
            if peak is None or abs(strain) > peak["absolute_strain"]:
                peak = {"triangle": index, "piece": piece["id"], "vertices": tri, "axis": label,
                        "signed_strain": strain, "absolute_strain": abs(strain)}
    return peak


def number_bits(value):
    if isinstance(value, dict):
        return {key: number_bits(item) for key, item in value.items()}
    if isinstance(value, list):
        return [number_bits(item) for item in value]
    return ("f64", struct.pack("!d", value).hex()) if isinstance(value, float) else value


def self_check():
    meta = {"project": {"pieces": [{"id": 1, "grain_angle": 0.0}]}, "piece_ids": [1] * 3,
            "triangles": [[0, 1, 2]], "uv_m": [[0, 0], [1, 0], [0, 1]]}
    peak = material_peak(meta, [[0, 0, 0], [1.2, 0, 0], [0, 1, 0]])
    assert peak["axis"] == "warp" and abs(peak["signed_strain"] - 0.2) < 1e-12
    try:
        parse_obj("v nan 0 0\n")
    except AssertionError:
        pass
    else:
        raise AssertionError("OBJ parser accepted nonfinite input")


def main():
    assert len(sys.argv) == 3, __doc__
    evidence, control = map(Path, sys.argv[1:])
    inputs = {"fixture": evidence / "original-grain-shirt.weftforge", "oracle": evidence / "expected-peaks-preregistered.json",
              "rest_metadata": control / "rest-metadata.json", "trajectory": control / "trajectory.jsonl"}
    hashes = {name: sha(path) for name, path in inputs.items()}
    assert hashes == {"fixture": "82e7c6d398c7772d7e62c28196dcf11845020020d44fca9ac9113e615b9537e7",
                      "oracle": "5f8ffcfc077d41a39aa3084466f88806c85a09b4d6ed004213ecfc16d76bbc0e",
                      "rest_metadata": "0c58d8aa2cb2712f004c020b558e120f7b7fc2357231527e106da45740968e80",
                      "trajectory": "feb4d50b5d40e549b48bc309fb43b8d0968357d340a3946b17898229d579b7cb"}
    self_check()
    meta = read(inputs["rest_metadata"])
    frames = {row["global_frame"]: row for row in map(json.loads, inputs["trajectory"].read_text().splitlines())}
    expected = {row["frame"]: row for row in read(inputs["oracle"])["rows"] if row["case"] == "control"}
    original = read(inputs["fixture"])
    source = original["project"]
    source["sewing_groups"] = []
    stages = []
    assert (evidence / "assembly-native.weftforge").read_bytes() == (evidence / "drape-native.weftforge").read_bytes()
    for stage, frame, screen, pattern in (("assembly", 60, "07-assembly-inspector.ax.txt", "Candidate back"),
                                          ("drape", 180, "12-drape-complete-inspector.ax.txt", "Candidate front")):
        source_path, obj_path = evidence / f"{stage}-native.weftforge", evidence / f"{stage}-native.obj"
        saved = read(source_path)
        assert number_bits(saved["project"]) == number_bits(source)
        positions, uv, triangles, groups, position_text, uv_text = parse_obj(obj_path.read_text())
        assert len(positions) == 150 and len(triangles) == 228
        assert triangles == meta["triangles"]
        assert groups == [meta["piece_ids"][tri[0]] for tri in triangles]
        baseline = frames[frame]["positions_m"]
        assert position_text == [[f"{value:.17f}" for value in row] for row in baseline]
        assert uv_text == [[f"{value:.17f}" for value in row] for row in meta["uv_m"]]
        peak = material_peak({**meta, "uv_m": uv}, positions)
        for key in ("triangle", "piece", "vertices", "axis"):
            assert peak[key] == expected[frame][key]
        strain_error = abs(peak["signed_strain"] - expected[frame]["signed_strain"])
        assert strain_error <= 1e-12
        percent = f"{peak['absolute_strain'] * 100:.2f}%"
        attribution = f"{pattern} · Warp {peak['signed_strain'] * 100:+.2f}%"
        ax = (evidence / screen).read_text()
        for label in (f"Time {frame / 60:.3f} s", f"Peak material strain {percent}", attribution,
                      "Seam gap 0.0000 m", "Contact error 0.0000 m", "Initial intersections 0"):
            assert label in ax, label
        assert "Cancel job" not in ax
        assert peak["absolute_strain"] >= 0.15
        stages.append({"stage": stage, "original_control_frame": frame, "source_version": saved["version"],
                       "source_revision": saved["project"]["revision"], "source_sha256": sha(source_path),
                       "obj_sha256": sha(obj_path), "vertices": len(positions), "triangles": len(triangles),
                       "all_obj_coordinates_and_normals_finite": True, "triangles_and_piece_groups_exact": True,
                       "position_and_uv_tokens_exact_original_fixed_17_decimal_export": True,
                       "maximum_position_difference_m": max(abs(a - b) for x, y in zip(positions, baseline) for a, b in zip(x, y)),
                       "maximum_uv_difference_m": max(abs(a - b) for x, y in zip(uv, meta["uv_m"]) for a, b in zip(x, y)),
                       "independent_peak": peak, "absolute_strain_error": strain_error,
                       "actual_native_attribution_label": attribution, "original_15_percent_strain_limit_passed": False})
    result = {"kind": "native_saved_output_no_simulator_check", "checked_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "status": "passed", "checker_sha256": sha(Path(__file__)), "frozen_input_sha256": hashes,
              "self_checks": "analytic affine extension and rejection of nonfinite OBJ input passed",
              "assembly_drape_authored_source_bytes_identical": True, "all_authored_f64_bits_order_revision_ids_unchanged": True,
              "original_fixture_version": original["version"], "source_normalization": "v2 input saved as v3; only default sewing_groups=[] is added. Existing f32 color JSON values already match; no color normalization difference was needed. All other authored numbers compare as f64 bits, arrays retain order.",
              "stages": stages, "limits": ["Native .weftforge snapshots contain editable construction, not solver positions or velocities; OBJ is the stage position artifact.",
              "OBJ fixed 17 decimal export rounds tiny coordinates; token equality proves the expected rendering of the retained original state, not bit-identical parsed OBJ floats.",
              "Finiteness is checked in retained OBJ; the native inspector has no separate finite-status label.",
              "Both original strain quality failures remain failures. This verifies diagnostic visibility and retained output, not force balance, fitting acceptance, reference parity or full SIM-005."]}
    print(json.dumps(result, indent=2, allow_nan=False))


if __name__ == "__main__":
    main()
