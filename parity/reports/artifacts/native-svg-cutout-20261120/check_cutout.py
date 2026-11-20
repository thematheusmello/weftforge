"""Check the rectangular pocket fixture, its saved native edit, and two rest OBJs.

Usage: python3 check_cutout.py pocket.svg imported.weftforge imported.obj edited.weftforge edited.obj
Uses the Python standard library. Coordinates are metres.
"""

import argparse
from collections import defaultdict
import copy
import hashlib
import json
import math
from pathlib import Path
import re
import xml.etree.ElementTree as ET


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def cyclic(points):
    return zip(points, points[1:] + points[:1])


def signed_area(loop):
    return sum(a[0] * b[1] - b[0] * a[1] for a, b in cyclic(loop)) / 2


def close(a, b):
    return math.isclose(a, b, rel_tol=1e-10, abs_tol=1e-12)


def fixture_loops(path):
    root = ET.parse(path).getroot()
    x, y, width, height = map(float, root.attrib["viewBox"].split())
    assert (x, y) == (0, -360)
    assert root.attrib["width"].endswith("mm") and root.attrib["height"].endswith("mm")
    u = float(root.attrib["width"][:-2]) / (1000 * width)
    v = float(root.attrib["height"][:-2]) / (1000 * height)
    assert close(u, 0.001) and close(v, 0.001)
    (shape,) = root.findall("{http://www.w3.org/2000/svg}path")
    assert shape.attrib["fill-rule"] == "evenodd"
    outline = shape.attrib["d"]
    paths = re.findall(r"M([^MZ]+)Z", outline)
    assert len(paths) == 2 and "".join("M" + s + "Z" for s in paths).replace(
        " ", ""
    ) == outline.replace(" ", "")
    rings = []
    for coords in paths:
        numbers = list(map(float, coords.replace("L", " ").split()))
        assert len(numbers) == 8
        rings.append([(a * u, -b * v) for a, b in zip(numbers[::2], numbers[1::2])])
    return sorted(rings, key=lambda ring: abs(signed_area(ring)), reverse=True)


def rings_and_ids(doc):
    project = doc["project"]
    (piece,) = project["pieces"]
    (hole,) = piece["holes"]
    points = {p["id"]: p["position"] for p in project["points"]}
    curves = {c["id"]: c for c in project["curves"]}
    rings = []
    ring_ids = []
    for curve_ids in [piece["boundary"], hole["boundary"]]:
        spans = [curves[i] for i in curve_ids]
        assert all(
            c["controls"] is None and c["end"] == following["start"]
            for c, following in cyclic(spans)
        )
        ids = [c["start"] for c in spans]
        assert len(ids) == 4 and len(set(ids)) == 4
        ring_ids.append(ids)
        rings.append([tuple(points[i]) for i in ids])
    assert set(ring_ids[0]).isdisjoint(ring_ids[1]) and set(sum(ring_ids, [])) == set(
        points
    )
    assert len(points) == 8 and len(curves) == 8
    return piece, hole, rings, ring_ids


def matches_ring(a, b):
    # Starting vertex and winding may change in the stored authoring document.
    return any(
        all(math.dist(p, q) < 1e-12 for p, q in zip(a, turn[i:] + turn[:i]))
        for turn in (b, b[::-1])
        for i in range(len(b))
    )


def on_segment(point, a, b):
    return close(math.dist(a, point) + math.dist(point, b), math.dist(a, b))


def connected(graph, start):
    seen, pending = {start}, [start]
    while pending:
        for vertex in graph[pending.pop()]:
            if vertex not in seen:
                seen.add(vertex)
                pending.append(vertex)
    return seen


def inspect_obj(path, rings):
    vertices, uvs, normals, faces = [], [], [], []
    groups = []
    for line in path.read_text().splitlines():
        words = line.split()
        if not words or words[0].startswith("#"):
            continue
        kind, args = words[0], words[1:]
        if kind in ("v", "vt", "vn"):
            numbers = tuple(map(float, args))
            assert len(numbers) == {"v": 3, "vt": 2, "vn": 3}[kind] and all(
                map(math.isfinite, numbers)
            )
            {"v": vertices, "vt": uvs, "vn": normals}[kind].append(numbers)
        elif kind == "f":
            assert len(args) == 3
            face = []
            for corner in args:
                i, t, n = (int(index) - 1 for index in corner.split("/"))
                assert (
                    0 <= i < len(vertices)
                    and 0 <= t < len(uvs)
                    and 0 <= n < len(normals)
                )
                assert (
                    len(uvs[t]) == 2 and math.dist(vertices[i], (*uvs[t], 0.0)) < 1e-12
                )
                assert math.dist(normals[n], (0.0, 0.0, 1.0)) < 1e-12
                face.append((i, t))
            faces.append(face)
        elif kind == "g":
            groups.append(" ".join(args))
        else:
            raise AssertionError(f"Unexpected OBJ record {kind}")
    assert (
        len(groups) == 1
        and len(vertices) == len(set(vertices))
        and len(uvs) == len(set(uvs))
    )
    assert {i for f in faces for i, _ in f} == set(range(len(vertices)))
    assert {t for f in faces for _, t in f} == set(range(len(uvs)))
    edges = defaultdict(list)
    total_xyz = total_uv = 0.0
    for ordinal, face in enumerate(faces):
        vi, ti = zip(*face)
        xyz = [vertices[i][:2] for i in vi]
        uv = [uvs[t] for t in ti]
        area = signed_area(xyz)
        assert area > 0 and close(area, signed_area(uv))
        total_xyz += area
        total_uv += signed_area(uv)
        for a, b in cyclic(vi):
            edges[tuple(sorted((a, b)))].append((ordinal, a, b))
    adjacency = defaultdict(set)
    boundary = defaultdict(set)
    for entries in edges.values():
        if len(entries) == 1:
            _, a, b = entries[0]
            boundary[a].add(b)
            boundary[b].add(a)
        else:
            assert len(entries) == 2
            (i, a, b), (j, c, d) = entries
            assert (a, b) == (d, c)
            adjacency[i].add(j)
            adjacency[j].add(i)
    assert len(connected(adjacency, 0)) == len(faces)
    assert all(len(neighbors) == 2 for neighbors in boundary.values())
    remaining = set(boundary)
    boundary_sizes = []
    while remaining:
        component = connected(boundary, next(iter(remaining)))
        remaining -= component
        boundary_sizes.append(len(component))
    assert len(boundary_sizes) == 2 and len(vertices) - len(edges) + len(faces) == 0
    sides = [edge for ring in rings for edge in cyclic(ring)]
    lengths = [0.0] * len(sides)
    for a, neighbors in boundary.items():
        for b in neighbors:
            if a < b:
                p, q = vertices[a][:2], vertices[b][:2]
                (side,) = [
                    i
                    for i, (u, v) in enumerate(sides)
                    if on_segment(p, u, v) and on_segment(q, u, v)
                ]
                lengths[side] += math.dist(p, q)
    assert all(close(value, math.dist(u, v)) for value, (u, v) in zip(lengths, sides))
    expected = abs(signed_area(rings[0])) - abs(signed_area(rings[1]))
    assert close(total_xyz, expected) and close(total_uv, expected)
    return {
        "vertices": len(vertices),
        "triangles": len(faces),
        "unique_edges": len(edges),
        "boundary_loop_vertex_counts": sorted(boundary_sizes),
        "connected_surfaces": 1,
        "euler_characteristic": 0,
        "boundary_matches_every_authored_side": True,
        "outer_area_m2": abs(signed_area(rings[0])),
        "hole_area_m2": abs(signed_area(rings[1])),
        "polygon_material_area_m2": expected,
        "obj_area_m2": total_xyz,
        "obj_uv_area_m2": total_uv,
    }


def check(svg, imported, imported_obj, edited, edited_obj):
    before, after = [json.loads(p.read_bytes()) for p in [imported, edited]]
    piece, hole, rings, ids = rings_and_ids(before)
    _, _, new_rings, new_ids = rings_and_ids(after)
    assert ids == new_ids
    assert all(
        matches_ring(ring, original)
        for ring, original in zip(rings, fixture_loops(svg))
    )
    assert signed_area(rings[0]) * signed_area(rings[1]) < 0
    p1, p2 = before["project"], after["project"]
    assert p2["revision"] == p1["revision"] + 1
    expected = copy.deepcopy(before)
    expected["project"]["revision"] = p2["revision"]
    changed = []
    for a, b in zip(expected["project"]["points"], p2["points"]):
        if a["position"] != b["position"]:
            assert a["id"] == b["id"] and a["id"] in ids[1]
            changed.append(
                {"point_id": a["id"], "from_m": a["position"], "to_m": b["position"]}
            )
            a["position"] = b["position"]
    assert len(changed) == 1 and expected == after
    before_mesh = inspect_obj(imported_obj, rings)
    after_mesh = inspect_obj(edited_obj, new_rings)
    delta = (
        after_mesh["polygon_material_area_m2"] - before_mesh["polygon_material_area_m2"]
    )
    assert close(before_mesh["polygon_material_area_m2"], 0.0768) and close(
        delta, -0.0012
    )
    return {
        "revisions": [p1["revision"], p2["revision"]],
        "piece_id": piece["id"],
        "hole_id": hole["id"],
        "exterior_curve_ids": piece["boundary"],
        "hole_curve_ids": hole["boundary"],
        "exterior_point_ids": ids[0],
        "hole_point_ids": ids[1],
        "changed_points": changed,
        "other_saved_fields_equal": True,
        "imported": before_mesh,
        "edited": after_mesh,
        "material_area_change_m2": delta,
        "sha256": dict(
            zip(
                [
                    "svg",
                    "imported_project",
                    "imported_obj",
                    "edited_project",
                    "edited_obj",
                ],
                map(digest, [svg, imported, imported_obj, edited, edited_obj]),
            )
        ),
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("svg", "imported", "imported_obj", "edited", "edited_obj"):
        parser.add_argument(name, type=Path)
    print(json.dumps(check(**vars(parser.parse_args())), indent=2))
