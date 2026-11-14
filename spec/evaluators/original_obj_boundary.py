#!/usr/bin/env python3
"""Check the boundary of one original unsimulated axis-aligned rectangle OBJ.

Expected size and fixed units are external inputs. Translation is ignored; scale
is never fitted. This bounded check requires a consistently oriented triangle
disk and does not establish a general OBJ or cloth-physics contract.
"""
import argparse
import collections
import hashlib
import json
import math
import pathlib

import original_obj_bounds


def _components(adjacency):
    unseen = set(adjacency)
    count = 0
    while unseen:
        count += 1
        todo = [unseen.pop()]
        while todo:
            for neighbor in adjacency[todo.pop()]:
                if neighbor in unseen:
                    unseen.remove(neighbor)
                    todo.append(neighbor)
    return count


def _distance_to_segment(point, start, end):
    direction = [b - a for a, b in zip(start, end)]
    t = sum((p - a) * d for p, a, d in zip(point, start, direction))
    t = min(1.0, max(0.0, t / sum(d * d for d in direction)))
    return math.dist(point, [a + t * d for a, d in zip(start, direction)])


def measure_boundary(path, unit, expected_mm, tolerance_mm=0.01):
    dimensions = original_obj_bounds.measure(path, unit, expected_mm, tolerance_mm)
    factor = {"mm": 1.0, "cm": 10.0, "m": 1000.0}[unit]
    data = pathlib.Path(path).read_bytes()
    if hashlib.sha256(data).hexdigest() != dimensions["input_sha256"]:
        raise ValueError("Original OBJ changed during the bounded measurement")
    vertices, faces = [], []
    for line in data.decode("utf-8").splitlines():
        tokens = line.split()
        if tokens and tokens[0] == "v":
            vertices.append([float(v) * factor for v in tokens[1:]])
        elif tokens and tokens[0] == "f":
            faces.append([int(v.split("/")[0]) - 1 for v in tokens[1:]])
    if any(len(face) != 3 or len(set(face)) != 3 for face in faces):
        raise ValueError("This bounded boundary check requires non-repeated triangle corners")
    edges = collections.Counter()
    adjacency = collections.defaultdict(set)
    for face in faces:
        for a, b in zip(face, face[1:] + face[:1]):
            edges[tuple(sorted((a, b)))] += 1
            adjacency[a].add(b)
            adjacency[b].add(a)
    boundary = [edge for edge, count in edges.items() if count == 1]
    boundary_adjacency = collections.defaultdict(set)
    for a, b in boundary:
        boundary_adjacency[a].add(b)
        boundary_adjacency[b].add(a)

    extent = dimensions["bounds_mm"]["extent"]
    axes = sorted(range(3), key=lambda axis: extent[axis])[1:]
    origin = [dimensions["bounds_mm"]["min"][axis] for axis in axes]
    projected = [[vertex[axis] - offset for axis, offset in zip(axes, origin)] for vertex in vertices]
    width, height = sorted(expected_mm)
    corners = [(0.0, 0.0), (width, 0.0), (width, height), (0.0, height)]
    sides = list(zip(corners, corners[1:] + corners[:1]))
    # Both endpoints must be close to one declared side. Testing only vertex
    # distance would incorrectly accept a diagonal joining opposite corners.
    boundary_error = max((
        min(max(_distance_to_segment(projected[index], a, b) for index in edge) for a, b in sides)
        for edge in boundary
    ), default=math.inf)
    corner_error = max((
        min((_distance_to_segment(corner, projected[a], projected[b]) for a, b in boundary), default=math.inf)
        for corner in corners
    ), default=math.inf)
    twice_areas = []
    for a, b, c in faces:
        p, q, r = projected[a], projected[b], projected[c]
        twice_areas.append((q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]))
    signs = {1 if area > 0 else -1 if area < 0 else 0 for area in twice_areas}
    triangle_area = sum(abs(area) for area in twice_areas) / 2
    expected_area = width * height
    area_tolerance = 2 * (width + height) * tolerance_mm
    used_vertices = set(adjacency)
    euler = len(used_vertices) - len(edges) + len(faces)
    loops = _components(boundary_adjacency)
    checks = {
        "physical_dimensions": dimensions["status"] == "passed",
        "all_vertices_used": len(used_vertices) == len(vertices),
        "one_connected_mesh": _components(adjacency) == 1,
        "edge_incidence_one_or_two": all(count in (1, 2) for count in edges.values()),
        "boundary_degree_two": bool(boundary) and all(len(neighbors) == 2 for neighbors in boundary_adjacency.values()),
        "one_boundary_loop": loops == 1,
        "disk_euler_characteristic": euler == 1,
        "consistent_nondegenerate_projected_winding": len(signs) == 1 and 0 not in signs,
        "boundary_on_declared_rectangle": boundary_error <= tolerance_mm,
        "all_declared_corners_covered": corner_error <= tolerance_mm,
        "projected_area": abs(triangle_area - expected_area) <= area_tolerance,
    }
    return {
        "kind": "independent-original-rectangle-boundary-measurement",
        "status": "passed" if all(checks.values()) else "failed",
        "input_filename": pathlib.Path(path).name,
        "input_sha256": dimensions["input_sha256"],
        "parser_sha256": hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
        "dimension_parser_sha256": dimensions["parser_sha256"],
        "declared_export_unit": unit,
        "expected_dimensions_mm": sorted(expected_mm),
        "tolerance_mm": tolerance_mm,
        "unit_basis": "Observed export control; fixed conversion, never fitted to bounds",
        "translation_policy": "Ignore translation using exported minima; no authored position or rotation result is claimed",
        "checks": checks,
        "vertices": len(vertices),
        "faces": len(faces),
        "edges": len(edges),
        "boundary_edges": len(boundary),
        "boundary_vertices": len(boundary_adjacency),
        "boundary_components": loops,
        "euler_characteristic": euler,
        "max_boundary_error_mm": boundary_error if math.isfinite(boundary_error) else None,
        "max_corner_error_mm": corner_error if math.isfinite(corner_error) else None,
        "projected_triangle_area_mm2": triangle_area,
        "expected_area_mm2": expected_area,
        "area_tolerance_mm2": area_tolerance,
        "limitations": [
            "One original unsimulated axis-aligned rectangle, positive-index triangle OBJ subset",
            "Manifold incidence, winding, boundary and area checks; no general polygon/intersection proof",
            "Texture, material, normals, native format and cloth physics are not qualified",
            "Independent fixture measurements supply zero verified WeftForge parity claims",
        ],
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("obj", type=pathlib.Path)
    parser.add_argument("--unit", required=True, choices=["mm", "cm", "m"])
    parser.add_argument("--expected-mm", required=True, type=float, nargs=2)
    parser.add_argument("--tolerance-mm", type=float, default=0.01)
    args = parser.parse_args()
    result = measure_boundary(args.obj, args.unit, args.expected_mm, args.tolerance_mm)
    print(json.dumps(result, indent=2, allow_nan=False))
    raise SystemExit(0 if result["status"] == "passed" else 1)
