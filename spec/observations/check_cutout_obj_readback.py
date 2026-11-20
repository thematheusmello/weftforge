"""Measure one planar metre-scale cutout before and after an OBJ round trip."""

from collections import Counter, defaultdict
import hashlib
import json
import math
from pathlib import Path
import sys


def components(neighbors):
    remaining = set(neighbors)
    result = []
    while remaining:
        start = remaining.pop()
        group = {start}
        stack = [start]
        while stack:
            unseen = neighbors[stack.pop()] & remaining
            remaining.difference_update(unseen)
            group.update(unseen)
            stack.extend(unseen)
        result.append(group)
    return result


def measure(path):
    vertices, texcoords, normals, edges = [], [], [], Counter()
    areas, corner_uv, corner_normals = [], [], []
    groups = materials = 0
    for line in path.read_text().splitlines():
        fields = line.split()
        if not fields:
            continue
        tag = fields[0]
        if tag in ("v", "vt", "vn"):
            values = tuple(map(float, fields[1:]))
            assert all(map(math.isfinite, values))
            {"v": vertices, "vt": texcoords, "vn": normals}[tag].append(values)
        elif tag == "g":
            groups += 1
        elif tag == "usemtl":
            materials += 1
        elif tag == "f":
            assert len(fields) == 4
            corners = [tuple(int(i) - 1 for i in f.split("/")) for f in fields[1:]]
            for i, uv, n in corners:
                assert i >= 0 and uv >= 0 and n >= 0
                corner_uv.append(texcoords[uv][:2])
                corner_normals.append(normals[n])
            ids = [c[0] for c in corners]
            a, b, c = [vertices[i] for i in ids]
            assert a[2] == b[2] == c[2] == 0.0
            areas.append(
                ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) / 2
            )
            assert areas[-1] > 0
            edges.update(tuple(sorted((i, j))) for i, j in zip(ids, ids[1:] + ids[:1]))
    all_edges, boundary = defaultdict(set), defaultdict(set)
    for (a, b), count in edges.items():
        assert count in (1, 2)
        all_edges[a].add(b)
        all_edges[b].add(a)
        if count == 1:
            boundary[a].add(b)
            boundary[b].add(a)
    assert len(vertices) == len(all_edges) and all(
        len(v) == 2 for v in boundary.values()
    )
    loops = []
    for ring in components(boundary):
        perimeter = math.fsum(
            math.dist(vertices[a], vertices[b])
            for a in ring
            for b in boundary[a]
            if a < b
        )
        loops.append(
            {
                "vertices": len(ring),
                "perimeter_m": perimeter,
                "bounds_xy_m": [
                    [
                        min(vertices[i][k] for i in ring),
                        max(vertices[i][k] for i in ring),
                    ]
                    for k in range(2)
                ],
            }
        )
    return {
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "vertices": len(vertices),
        "triangles": len(areas),
        "connected_components": len(components(all_edges)),
        "euler_characteristic": len(vertices) - len(edges) + len(areas),
        "material_area_m2": math.fsum(areas),
        "boundary_loops": sorted(loops, key=lambda r: -r["perimeter_m"]),
        "uv_bounds": [
            [min(v[k] for v in corner_uv), max(v[k] for v in corner_uv)]
            for k in range(2)
        ],
        "all_normals_positive_z": all(n == (0.0, 0.0, 1.0) for n in corner_normals),
        "group_statements": groups,
        "usemtl_statements": materials,
    }


if __name__ == "__main__":
    print(
        json.dumps(
            {
                "input": measure(Path(sys.argv[1])),
                "returned": measure(Path(sys.argv[2])),
            },
            indent=2,
        )
    )
