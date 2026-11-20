"""Check physical OBJ triangle areas and material assignments against an editable garment."""

import argparse
from collections import Counter, defaultdict
import json
from math import dist, fsum, hypot, isclose
from pathlib import Path


def read_surface(path):
    vertices, uv, normals = [], [], []
    faces = defaultdict(list)
    groups = defaultdict(set)
    active = group = library = None
    for line in path.read_text().splitlines():
        parts = line.strip().split()
        if not parts or parts[0].startswith("#"):
            continue
        key, values = parts[0], parts[1:]
        if key in ("v", "vt", "vn"):
            {"v": vertices, "vt": uv, "vn": normals}[key].append(
                tuple(map(float, values))
            )
        elif key == "mtllib":
            assert library is None
            library = " ".join(values)
            assert Path(library).name == library
        elif key == "usemtl":
            active = " ".join(values)
        elif key == "g":
            group = " ".join(values)
        elif key == "f":
            assert active and len(values) == 3
            indices = [
                tuple(int(i) - 1 for i in corner.split("/")) for corner in values
            ]
            assert all(
                len(row) == 3
                and all(
                    0 <= idx < len(pool)
                    for idx, pool in zip(row, (vertices, uv, normals))
                )
                for row in indices
            )
            faces[active].append(tuple(row[0] for row in indices))
            if group:
                groups[active].add(group)
    assert library
    colors = {}
    for line in path.with_name(library).read_text().splitlines():
        parts = line.split()
        if parts and parts[0] == "newmtl":
            active = " ".join(parts[1:])
        elif parts and parts[0] == "Kd":
            assert active not in colors
            colors[active] = list(map(float, parts[1:]))
    assert set(faces) <= colors.keys()
    totals = {}
    for name, triangles in faces.items():
        areas, edges = [], Counter()
        for a, b, c in triangles:
            u = [x - y for x, y in zip(vertices[b], vertices[a])]
            v = [x - y for x, y in zip(vertices[c], vertices[a])]
            areas.append(
                0.5
                * hypot(
                    u[1] * v[2] - u[2] * v[1],
                    u[2] * v[0] - u[0] * v[2],
                    u[0] * v[1] - u[1] * v[0],
                )
            )
            edges.update(tuple(sorted(edge)) for edge in [(a, b), (b, c), (c, a)])
        assert all(a > 0 for a in areas) and max(edges.values()) <= 2
        boundary = [pair for pair, n in edges.items() if n == 1]
        totals[name] = {
            "groups": sorted(groups[name]),
            "triangles": len(triangles),
            "diffuse_rgb": colors[name],
            "area_m2": fsum(areas),
            "boundary_m": fsum(dist(vertices[a], vertices[b]) for a, b in boundary),
        }
    return {
        "positions": len(vertices),
        "uvs": len(uv),
        "normals": len(normals),
        "triangles": sum(len(rows) for rows in faces.values()),
        "materials": totals,
    }


def check(project_path, obj_path):
    project = json.loads(project_path.read_text())["project"]
    summary = read_surface(obj_path)
    fabrics = {m["id"]: m for m in project["materials"]}
    assert len(project["pieces"]) == 2 and len(project["seams"]) == 8
    assert (
        summary["positions"] == summary["uvs"] == summary["normals"] == 654
        and summary["triangles"] == 1172
    )
    assert len(summary["materials"]) == 2
    for piece in project["pieces"]:
        fabric = fabrics[piece["material"]]
        matches = [
            data
            for name, data in summary["materials"].items()
            if name.startswith(f"fabric_{fabric['id']}_")
        ]
        assert len(matches) == 1
        data = matches[0]
        assert data["groups"] == [f"piece_{piece['id']}"] and data["triangles"] == 586
        assert all(
            isclose(a, b, abs_tol=1e-7)
            for a, b in zip(data["diffuse_rgb"], fabric["color"], strict=True)
        )
        points = {p["id"]: p["position"] for p in project["points"]}
        curves = {c["id"]: c for c in project["curves"]}
        outline = [points[curves[c]["start"]] for c in piece["boundary"]]
        assert (
            all(curves[c]["controls"] is None for c in piece["boundary"])
            and not piece["holes"]
        )
        area = (
            abs(
                fsum(
                    p[0] * q[1] - q[0] * p[1]
                    for p, q in zip(outline, outline[1:] + outline[:1])
                )
            )
            / 2
        )
        perimeter = fsum(dist(p, q) for p, q in zip(outline, outline[1:] + outline[:1]))
        assert isclose(data["area_m2"], area, abs_tol=1e-10) and isclose(
            data["boundary_m"], perimeter, abs_tol=1e-10
        )
    return summary


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("project", type=Path)
    parser.add_argument("obj", type=Path)
    args = parser.parse_args()
    print(json.dumps(check(args.project, args.obj), indent=2, sort_keys=True))
