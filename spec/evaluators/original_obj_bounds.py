#!/usr/bin/env python3
"""Measure an original, axis-aligned, unsimulated rectangle OBJ independently.

Units and expected dimensions come from observed authoring/export controls, never
from fitting the parsed bounds. This is not a general OBJ/interoperability test.
"""
import argparse
import collections
import hashlib
import json
import math
import pathlib


def _index(value, count):
    index = int(value)
    if not 1 <= index <= count:
        raise ValueError("This bounded evaluator requires valid positive OBJ indices")


def _hull(points):
    def cross(o, a, b):
        return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])

    lower, upper = [], []
    points = sorted(set(points))
    for point in points:
        while len(lower) >= 2 and cross(lower[-2], lower[-1], point) <= 0:
            lower.pop()
        lower.append(point)
    for point in reversed(points):
        while len(upper) >= 2 and cross(upper[-2], upper[-1], point) <= 0:
            upper.pop()
        upper.append(point)
    return lower[:-1] + upper[:-1]


def measure(path, unit, expected_mm, tolerance_mm=0.01):
    factor = {"mm": 1.0, "cm": 10.0, "m": 1000.0}[unit]
    if len(expected_mm) != 2 or not all(math.isfinite(v) and v > 0 for v in expected_mm):
        raise ValueError("Two finite positive expected dimensions are required")
    if not math.isfinite(tolerance_mm) or tolerance_mm <= 0:
        raise ValueError("Tolerance must be finite and positive")
    path = pathlib.Path(path)
    if path.stat().st_size > 50_000_000:
        raise ValueError("Input exceeds the bounded 50 MB observation allowance")
    data = path.read_bytes()
    vertices, uv, normals, faces = [], [], [], []
    for line in data.decode("utf-8").splitlines():
        tokens = line.split()
        if not tokens or tokens[0].startswith("#"):
            continue
        if tokens[0] in ("v", "vt", "vn"):
            values = [float(v) for v in tokens[1:]]
            required = 2 if tokens[0] == "vt" else 3
            if len(values) != required or not all(map(math.isfinite, values)):
                raise ValueError("Unexpected or nonfinite original OBJ coordinates")
            {"v": vertices, "vt": uv, "vn": normals}[tokens[0]].append(values)
        elif tokens[0] == "f":
            if len(tokens) < 4:
                raise ValueError("Face has fewer than three corners")
            faces.append(tokens[1:])
    if not vertices or not faces:
        raise ValueError("Original OBJ must contain vertices and faces")
    for face in faces:
        for corner in face:
            indices = corner.split("/")
            if len(indices) > 3:
                raise ValueError("Unexpected face corner syntax")
            _index(indices[0], len(vertices))
            if len(indices) > 1 and indices[1]:
                _index(indices[1], len(uv))
            if len(indices) > 2 and indices[2]:
                _index(indices[2], len(normals))
    vertices_mm = [[value * factor for value in vertex] for vertex in vertices]
    low = [min(v[k] for v in vertices_mm) for k in range(3)]
    high = [max(v[k] for v in vertices_mm) for k in range(3)]
    extent = [b - a for a, b in zip(low, high)]
    expected = [0.0, *sorted(expected_mm)]
    errors = [abs(a - b) for a, b in zip(sorted(extent), expected)]
    axes = [i for i, value in enumerate(extent) if value > tolerance_mm]
    if len(axes) != 2:
        raise ValueError("This observation requires an axis-aligned near-planar rectangle")
    hull = _hull((v[axes[0]], v[axes[1]]) for v in vertices_mm)
    pairs = list(zip(hull, hull[1:] + hull[:1]))
    perimeter = sum(math.dist(a, b) for a, b in pairs)
    area = abs(sum(a[0] * b[1] - a[1] * b[0] for a, b in pairs)) / 2
    return {
        "kind": "independent-original-export-dimension-measurement",
        "status": "passed" if max(errors) <= tolerance_mm else "failed",
        "input_filename": path.name,
        "input_sha256": hashlib.sha256(data).hexdigest(),
        "parser_sha256": hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
        "declared_export_unit": unit,
        "unit_basis": "Observed export control; fixed conversion, never fitted to bounds",
        "vertices": len(vertices),
        "uv_coordinates": len(uv),
        "normals": len(normals),
        "faces": len(faces),
        "face_sizes": dict(collections.Counter(map(len, faces))),
        "bounds_mm": {"min": low, "max": high, "extent": extent},
        "expected_sorted_extent_mm": expected,
        "extent_errors_mm": errors,
        "declared_extent_tolerance_mm": tolerance_mm,
        "convex_hull_vertices": len(hull),
        "convex_hull_perimeter_mm": perimeter,
        "convex_hull_area_mm2": area,
        "limitations": [
            "Original unsimulated axis-aligned rectangle only; positive-index OBJ subset",
            "Dimension result does not establish interior topology, cloth physics or parity",
            "Native project, vendor resources and materials are not decoded or consumed",
        ],
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("obj", type=pathlib.Path)
    parser.add_argument("--unit", required=True, choices=["mm", "cm", "m"])
    parser.add_argument("--expected-mm", required=True, type=float, nargs=2)
    parser.add_argument("--tolerance-mm", type=float, default=0.01)
    args = parser.parse_args()
    result = measure(args.obj, args.unit, args.expected_mm, args.tolerance_mm)
    print(json.dumps(result, indent=2))
    raise SystemExit(0 if result["status"] == "passed" else 1)
