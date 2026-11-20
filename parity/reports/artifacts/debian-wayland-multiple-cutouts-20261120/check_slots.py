"""Independent editable-ring, OBJ topology, boundary and UV readback for the two-slot pocket."""

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
    assert len(paths) == 3 and "".join("M" + s + "Z" for s in paths).replace(
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
    holes = piece["holes"]
    assert len(holes) == 2
    points = {p["id"]: p["position"] for p in project["points"]}
    curves = {c["id"]: c for c in project["curves"]}
    rings = []
    ring_ids = []
    for curve_ids in [piece["boundary"], *(hole["boundary"] for hole in holes)]:
        spans = [curves[i] for i in curve_ids]
        assert all(
            c["controls"] is None and c["end"] == following["start"]
            for c, following in cyclic(spans)
        )
        ids = [c["start"] for c in spans]
        assert len(ids) == 4 and len(set(ids)) == 4
        ring_ids.append(ids)
        rings.append([tuple(points[i]) for i in ids])
    connected_ids = [i for ids in ring_ids for i in ids]
    assert len(connected_ids) == len(set(connected_ids)) == len(points) == len(curves) == 12
    assert set(connected_ids) == set(points)
    return piece, holes, rings, ring_ids


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
    groups, libraries, materials = [], [], []
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
        elif kind == "mtllib":
            libraries.append(path.with_name(" ".join(args)))
        elif kind == "usemtl":
            materials.append(" ".join(args))
        elif kind == "g":
            groups.append(" ".join(args))
        else:
            raise AssertionError(f"Unexpected OBJ record {kind}")
    assert (
        len(groups) == 1
        and len(vertices) == len(set(vertices))
        and len(uvs) == len(set(uvs))
    )
    assert len(libraries) == len(materials) == 1
    lib = libraries[0]
    assert lib.name == f"weftforge-fabrics-{digest(lib)}.mtl"
    assert f"newmtl {materials[0]}" in lib.read_text().splitlines()
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
    assert len(boundary_sizes) == 3 and len(vertices) - len(edges) + len(faces) == -1
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
    expected = abs(signed_area(rings[0])) - sum(abs(signed_area(ring)) for ring in rings[1:])
    assert close(total_xyz, expected) and close(total_uv, expected)
    return {
        "vertices": len(vertices),
        "triangles": len(faces),
        "unique_edges": len(edges),
        "boundary_loop_vertex_counts": sorted(boundary_sizes),
        "connected_surfaces": 1,
        "euler_characteristic": -1,
        "boundary_matches_every_authored_side": True,
        "outer_area_m2": abs(signed_area(rings[0])),
        "hole_areas_m2": sorted(abs(signed_area(ring)) for ring in rings[1:]),
        "polygon_material_area_m2": expected,
        "obj_area_m2": total_xyz,
        "obj_uv_area_m2": total_uv,
    }


def same_ring_sets(ours, expected):
    assert len(ours) == len(expected)
    for ring in expected:
        assert sum(matches_ring(ring, other) for other in ours) == 1


def check_inspector(baseline, added, expected_rings):
    old, new = [json.loads(path.read_bytes()) for path in (baseline, added)]
    before, after = old["project"], new["project"]
    assert after["revision"] == before["revision"] + 1
    assert after["next_id"] == before["next_id"] + 9
    assert len(after["pieces"]) == len(before["pieces"]) == 1
    piece, holes, rings, ids = rings_and_ids(new)
    assert len(before["pieces"][0]["holes"]) == 1
    assert before["pieces"][0]["holes"] == holes[:1]
    same_ring_sets(rings, expected_rings)
    normalized = copy.deepcopy(new)
    n = normalized["project"]
    for name in ("curves", "points"):
        assert n[name][:-4] == before[name]
        n[name] = n[name][:-4]
    n["pieces"][0]["holes"] = n["pieces"][0]["holes"][:-1]
    n["revision"], n["next_id"] = before["revision"], before["next_id"]
    assert normalized == old
    return {"revisions": [before["revision"], after["revision"]], "piece": piece["id"],
            "existing_hole": holes[0]["id"], "added_hole": holes[1]["id"],
            "unchanged_existing_document_fields": True, "ring_point_ids": ids}


def check_case(svg, imported, imported_obj, edited, edited_obj, refused, inspector, inspector_obj=None, inspector_redo=None):
    before, after = [json.loads(p.read_bytes()) for p in (imported, edited)]
    piece, holes, rings, ids = rings_and_ids(before)
    _, _, updated, updated_ids = rings_and_ids(after)
    authored = fixture_loops(svg)
    same_ring_sets(rings, authored)
    assert ids == updated_ids
    before_project, after_project = before["project"], after["project"]
    assert (before_project["revision"], after_project["revision"]) == (3, 4)
    expected = copy.deepcopy(before)
    expected["project"]["revision"] = 4
    changes = []
    for original, change in zip(expected["project"]["points"], after_project["points"]):
        if original["position"] != change["position"]:
            assert original["id"] == change["id"]
            assert original["position"] == [0.08, 0.24] and change["position"] == [0.06, 0.24]
            changes.append(original["id"])
            original["position"] = change["position"]
    assert len(changes) == 1 and expected == after
    old_mesh = inspect_obj(imported_obj, rings)
    new_mesh = inspect_obj(edited_obj, updated)
    assert close(old_mesh["polygon_material_area_m2"], 0.0756)
    assert close(new_mesh["polygon_material_area_m2"], 0.0744)
    assert refused.read_bytes() == edited.read_bytes()
    result = {"revisions": [3, 4], "piece": piece["id"], "holes": [h["id"] for h in holes],
              "point_moved": changes[0], "other_saved_fields_identical": True,
              "after_rejected_import_sha256": digest(refused), "edited_sha256": digest(edited),
              "imported": old_mesh, "edited": new_mesh}
    if inspector:
        result["inspector"] = check_inspector(*inspector, authored)
        original = copy.deepcopy(before)
        added = json.loads(inspector[1].read_bytes())
        original["project"]["revision"] = added["project"]["revision"]
        assert original == added
        result["inspector"]["matches_import_except_revision"] = True
        if inspector_obj:
            assert inspector_obj.read_bytes() == imported_obj.read_bytes()
            result["inspector"]["matches_imported_obj_sha256"] = digest(inspector_obj)
        if inspector_redo:
            restored = json.loads(inspector_redo.read_bytes())
            assert restored["project"]["revision"] == added["project"]["revision"] + 2
            added["project"]["revision"] = restored["project"]["revision"]
            assert restored == added
            result["inspector"]["redo_revision"] = restored["project"]["revision"]

    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("svg", "imported", "imported_obj", "edited", "edited_obj", "refused"):
        parser.add_argument(name, type=Path)
    parser.add_argument("--inspector", nargs=2, type=Path,
                        metavar=("ONE_HOLE_BEFORE", "TWO_HOLES_AFTER"))
    parser.add_argument("--inspector-obj", type=Path)
    parser.add_argument("--inspector-redo", type=Path)
    print(json.dumps(check_case(**vars(parser.parse_args())), indent=2))
