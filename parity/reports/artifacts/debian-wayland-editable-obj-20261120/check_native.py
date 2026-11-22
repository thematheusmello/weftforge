"""Replay retained native editable projects and OBJ geometry, using Python's standard library."""

import argparse
from collections import Counter, defaultdict
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_scaled(source, scaled):
    original, converted = [], []
    for obj, result in [(source, original), (scaled, converted)]:
        for line in obj.read_text().splitlines():
            if line.startswith("v "):
                result.append(tuple(Decimal(x) for x in line.split()[1:]))
            else:
                result.append(line)
    assert len(original) == len(converted)
    for a, b in zip(original, converted):
        assert (tuple(v * 1000 for v in a) if isinstance(a, tuple) else a) == b


def surfaces(path, unit):
    positions, groups = [], {}
    group = material = None
    palette = {}
    for line in path.read_text().splitlines():
        columns = line.split()
        if not columns:
            continue
        op, *args = columns
        if op == "v":
            positions.append(tuple(float(v) * unit for v in args))
        elif op == "mtllib":
            mtl = path.parent / " ".join(args)
            for row in mtl.read_text().splitlines():
                values = row.split()
                if not values:
                    continue
                if values[0] == "newmtl":
                    mat = " ".join(values[1:])
                if values[0] == "Kd":
                    palette[mat] = tuple(float(c) for c in values[1:])
        elif op == "g":
            group = " ".join(args)
        elif op == "usemtl":
            material = " ".join(args)
        elif op == "f":
            assert group and material and len(args) == 3
            groups.setdefault((group, material), []).append(
                tuple(int(token.split("/")[0]) - 1 for token in args)
            )
    result = {}
    for (group, material), tris in groups.items():
        edges = Counter()
        areas = []
        moments = [[], [], []]
        unique = set()
        for tri in tris:
            a, b, c = (positions[i] for i in tri)
            u = [b[i] - a[i] for i in range(3)]
            v = [c[i] - a[i] for i in range(3)]
            normal = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ]
            area = math.hypot(*normal) / 2
            assert area > 0
            areas.append(area)
            for i in range(3):
                moments[i].append((a[i] + b[i] + c[i]) * area / 3)
            for i in range(3):
                edges[tuple(sorted((tri[i], tri[(i + 1) % 3])))] += 1
            unique.update(tri)
        adjacency = defaultdict(set)
        for (a, b), count in edges.items():
            assert count in (1, 2)
            if count == 1:
                adjacency[a].add(b)
                adjacency[b].add(a)
        assert all(len(v) == 2 for v in adjacency.values())
        perimeters = []
        remaining = set(adjacency)
        while remaining:
            first = min(remaining)
            current = first
            previous = None
            length = []
            while current in remaining:
                remaining.remove(current)
                nxt = min(adjacency[current] - {previous})
                length.append(math.dist(positions[current], positions[nxt]))
                previous, current = current, nxt
            assert current == first
            perimeters.append(math.fsum(length))
        area = math.fsum(areas)
        assert group not in result
        result[group] = {
            "faces": len(tris),
            "vertices": len(unique),
            "area_m2": area,
            "boundary_loops_m": sorted(perimeters),
            "rgb": palette[material],
            "centroid_m": [math.fsum(m) / area for m in moments],
            "extent_m": [
                max(positions[j][i] for j in unique)
                - min(positions[j][i] for j in unique)
                for i in range(3)
            ],
        }
    return result


def construction(project):
    points = {p["id"]: p["position"] for p in project["points"]}
    curves = {c["id"]: c for c in project["curves"]}
    materials = {m["id"]: m for m in project["materials"]}
    groups = {}
    for part in project["pieces"]:
        areas = []
        perimeters = []
        for loop in [part["boundary"], *(h["boundary"] for h in part["holes"])]:
            ring = []
            for i, idx in enumerate(loop):
                c = curves[idx]
                assert (
                    c["controls"] is None
                    and c["end"] == curves[loop[(i + 1) % len(loop)]]["start"]
                )
                ring.append(points[c["start"]])
            a = (
                math.fsum(
                    x * v - y * u for (x, y), (u, v) in zip(ring, ring[1:] + ring[:1])
                )
                / 2
            )
            areas.append(abs(a))
            perimeters.append(
                math.fsum(math.dist(a, b) for a, b in zip(ring, ring[1:] + ring[:1]))
            )
        groups[part["name"]] = {
            "export_group": "piece_" + str(part["id"]),
            "area_m2": areas[0] - sum(areas[1:]),
            "boundary_loops_m": sorted(perimeters),
            "rgb": tuple(materials[part["material"]]["color"]),
        }
    return groups


def check_geometry(source, scale, project, exported):
    incoming = surfaces(source, scale)
    outgoing = surfaces(exported, 1.0)
    draft = construction(project)
    assert (
        incoming.keys() == draft.keys()
        and {v["export_group"] for v in draft.values()} == outgoing.keys()
    )
    pairs = []
    for name, authored in draft.items():
        expected = incoming[name]
        actual = outgoing[authored["export_group"]]
        assert expected["rgb"] == actual["rgb"] == authored["rgb"]
        for measure in [actual, authored]:
            assert abs(expected["area_m2"] - measure["area_m2"]) < 1e-10
            assert len(expected["boundary_loops_m"]) == len(measure["boundary_loops_m"])
            assert (
                math.dist(expected["boundary_loops_m"], measure["boundary_loops_m"])
                < 1e-9
            )
        for key in ["centroid_m", "extent_m"]:
            assert math.dist(expected[key], actual[key]) < 1e-9
        pairs.append(
            {
                "pattern": name,
                "authored": authored,
                "source_faces": expected["faces"],
                "export": actual,
            }
        )
    return pairs


def read(path, name):
    return json.loads((path / name).read_bytes())["project"]


def check(case, source_root):
    originals = source_root / "parity/reports/artifacts"
    raw_pocket = originals / "debian-wayland-multiple-cutouts-20261120/Imported.obj"
    pocket = case / "fixtures/Pocket in millimetres.obj"
    harbor = case / "fixtures/Harbor.obj"
    check_scaled(raw_pocket, pocket)
    assert (
        sha(pocket)
        == "2c068bf5a34fb0cec320b36a74638780b021f59c9fd0ad4d5671c390b22b9a1b"
    )
    assert (
        harbor.read_bytes()
        == (originals / "native-obj-fabrics-20261120/Harbor.obj").read_bytes()
    )
    for obj, src in [
        (pocket, raw_pocket),
        (harbor, originals / "native-obj-fabrics-20261120/Harbor.obj"),
    ]:
        mtl = next(
            s.split(" ", 1)[1]
            for s in obj.read_text().splitlines()
            if s.startswith("mtllib ")
        )
        assert (obj.parent / mtl).read_bytes() == (src.parent / mtl).read_bytes()
    first = read(case, "Pocket imported.weftforge")
    undo = read(case, "Pocket undo.weftforge")
    redo = read(case, "Pocket redo.weftforge")
    reopened = read(case, "Pocket reopened.weftforge")
    assert [p["revision"] for p in [first, undo, redo, reopened]] == [1, 2, 3, 3]
    assert all(
        not undo[k] for k in ["points", "curves", "pieces", "seams", "sewing_groups"]
    )
    assert len(undo["materials"]) == 1 and undo["materials"] == first["materials"][:1]
    assert {k: v for k, v in first.items() if k != "revision"} == {
        k: v for k, v in redo.items() if k != "revision"
    } and redo == reopened
    result = check_geometry(pocket, 0.001, reopened, case / "Pocket exported.obj")
    p = result[0]["export"]
    assert len(result) == 1 and p["faces"] == 64 and len(p["boundary_loops_m"]) == 3
    assert (
        math.dist(p["extent_m"], [0.24, 0.36, 0]) < 1e-10
        and abs(p["area_m2"] - 0.0756) < 1e-10
        and abs(sum(p["boundary_loops_m"]) - 1.74) < 1e-10
    )
    harbor_doc = read(case, "Harbor imported.weftforge")
    harbor_result = check_geometry(
        harbor, 1.0, harbor_doc, case / "Harbor exported.obj"
    )
    assert (
        len(harbor_result) == 2
        and len({p["authored"]["rgb"] for p in harbor_result}) == 2
    )
    return {
        "pocket_revisions": [1, 2, 3, 3],
        "undo_empty": True,
        "redo_preserves_authoring": True,
        "reopened_matches_redo": True,
        "pocket": result,
        "harbor_revision": harbor_doc["revision"],
        "harbor": harbor_result,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("case", type=Path)
    parser.add_argument("source_root", type=Path)
    args = parser.parse_args()
    print(json.dumps(check(args.case, args.source_root), indent=2))
