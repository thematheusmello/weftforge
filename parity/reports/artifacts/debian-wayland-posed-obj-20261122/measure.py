"""Measure the three original OBJ files and native save/reopened OBJ pairs with stdlib."""

import collections
import json
import math
from pathlib import Path

ROOT = Path(__file__).parent
protocol = json.loads((ROOT / "protocol.json").read_text())
EPS = protocol["max_xyz_component_m"]
UV_EPS = protocol["max_uv_extent_and_perimeter_m"]


def read_obj(path):
    xyz, uv, groups, colors = [], [], {}, {}
    group, material = "default", None
    for line in path.read_text().splitlines():
        items = line.strip().split()
        if not items or items[0].startswith("#"):
            continue
        key, *values = items
        if key == "mtllib":
            library = path.parent / " ".join(values)
            name = None
            for entry in library.read_text().splitlines():
                terms = entry.split()
                if terms[:1] == ["newmtl"]:
                    name = " ".join(terms[1:])
                if terms[:1] == ["Kd"]:
                    colors[name] = tuple(map(float, terms[1:]))
        elif key == "v":
            xyz.append(tuple(map(float, values)))
        elif key == "vt":
            uv.append(tuple(map(float, values)))
        elif key in ("g", "o"):
            group = " ".join(values)
        elif key == "usemtl":
            material = " ".join(values)
        elif key == "f":
            tri = [
                tuple(int(term) - 1 for term in corner.split("/")[:2])
                for corner in values
            ]
            assert len(tri) == 3
            groups.setdefault((group, material), []).append(tri)
    return [
        (name, colors.get(mat), [[(xyz[i], uv[j]) for i, j in tri] for tri in faces])
        for (name, mat), faces in groups.items()
    ]


def chart(faces):
    edge_count = collections.Counter()
    uv_vertices = set()
    xyz_areas, uv_areas = [], []
    for tri in faces:
        uv = [v[1] for v in tri]
        xyz = [v[0] for v in tri]
        uv_vertices.update(uv)
        u, v, w = uv
        uv_areas.append(
            abs((v[0] - u[0]) * (w[1] - u[1]) - (v[1] - u[1]) * (w[0] - u[0])) / 2
        )
        a = [y - x for x, y in zip(xyz[0], xyz[1])]
        b = [y - x for x, y in zip(xyz[0], xyz[2])]
        xyz_areas.append(
            math.hypot(
                *(
                    a[(j + 1) % 3] * b[(j + 2) % 3] - a[(j + 2) % 3] * b[(j + 1) % 3]
                    for j in range(3)
                )
            )
            / 2
        )
        for a, b in zip(uv, uv[1:] + uv[:1]):
            edge_count[tuple(sorted((a, b)))] += 1
    assert set(edge_count.values()) <= {1, 2}
    extent = [max(axis) - min(axis) for axis in zip(*uv_vertices)]
    perimeter = math.fsum(math.dist(*edge) for edge, n in edge_count.items() if n == 1)
    return extent + [perimeter], math.sqrt(math.fsum(xyz_areas) / math.fsum(uv_areas))


def oriented(faces, indices):
    counts = collections.Counter()
    for tri in faces:
        a, b, c = (indices[p] for p, _ in tri)
        counts[min((a, b, c), (b, c, a), (c, a, b))] += 1
    return counts


def verify_face_coordinates(src, dst):
    original = sorted(set(p for tri in src for p, _ in tri))
    actual = sorted(set(p for tri in dst for p, _ in tri))
    assert len(actual) == len(original)
    lookup = {p: i for i, p in enumerate(original)}
    assigned, maximum = {}, 0.0
    for p in actual:
        # Every coordinate must name a unique source vertex within the declared budget.
        choices = [
            i
            for i, q in enumerate(original)
            if max(abs(x - y) for x, y in zip(p, q)) <= EPS
        ]
        assert len(choices) == 1, (p, choices)
        assigned[p] = choices[0]
        maximum = max(maximum, max(abs(x - y) for x, y in zip(p, original[choices[0]])))
    assert len(set(assigned.values())) == len(original)
    assert oriented(src, lookup) == oriented(dst, assigned)
    return maximum, len(original), len(src)


def saved_boundary(project, piece):
    curves = {c["id"]: c for c in project["curves"]}
    points = {p["id"]: p["position"] for p in project["points"]}
    edges = [curves[i] for i in piece["boundary"]]
    assert not piece["holes"] and all(e["controls"] is None for e in edges)
    assert all(
        e["end"] == following["start"]
        for e, following in zip(edges, edges[1:] + edges[:1])
    )
    verts = [points[e["start"]] for e in edges]
    return [max(a) - min(a) for a in zip(*verts)] + [
        math.fsum(math.dist(points[e["start"]], points[e["end"]]) for e in edges)
    ]


def measure(stem, input_name):
    original = read_obj(ROOT / "fixtures" / input_name)
    exported = read_obj(ROOT / (stem + "-reopened.obj"))
    envelope = json.loads((ROOT / (stem + ".weftforge")).read_bytes())
    assert envelope["version"] == 8
    project = envelope["project"]
    assert len(original) == len(exported) == len(project["pieces"])
    panels = []
    for group, color, faces in original:
        # The two stitched Harbor panels share seam positions to within a nanometre.
        # Match with a source vertex spatially separated from every other source panel.
        other = {
            q
            for _, _, panel in original
            if panel is not faces
            for tri in panel
            for q, _ in tri
        }
        own = {q for tri in faces for q, _ in tri}
        first = (
            max(own, key=lambda q: min(math.dist(q, x) for x in other))
            if other
            else min(own)
        )
        matches = [
            panel
            for panel in exported
            if any(
                max(abs(x - y) for x, y in zip(p, first)) <= EPS
                for tri in panel[2]
                for p, _ in tri
            )
        ]
        assert len(matches) == 1
        out_name, out_color, out_faces = matches[0]
        xyz_error, vertices, face_count = verify_face_coordinates(faces, out_faces)
        flat, scale = chart(faces)
        expected = [a * scale for a in flat]
        actual, _ = chart(out_faces)
        (piece,) = [p for p in project["pieces"] if "piece_" + str(p["id"]) == out_name]
        flat_saved = saved_boundary(project, piece)
        assert (
            piece["initial_pose"]
            and len(piece["initial_pose"]["triangles"]) == face_count
        )
        uv_error = max(abs(a - b) for a, b in zip(expected, actual))
        saved_error = max(abs(a - b) for a, b in zip(expected, flat_saved))
        assert max(uv_error, saved_error) <= UV_EPS, (expected, actual, flat_saved)
        rgb_error = None
        if color is not None:
            (material,) = [
                m for m in project["materials"] if m["id"] == piece["material"]
            ]
            rgb_error = max(
                abs(x - y)
                for target in [out_color, material["color"]]
                for x, y in zip(color, target)
            )
            assert rgb_error <= protocol["max_color_component"]
        panels.append(
            dict(
                input_group=group,
                vertices=vertices,
                oriented_faces=face_count,
                xyz_max_error_m=xyz_error,
                output_chart_extent_perimeter_m=actual,
                saved_boundary_extent_perimeter_m=flat_saved,
                expected_extent_perimeter_m=expected,
                output_chart_max_error_m=uv_error,
                saved_boundary_max_error_m=saved_error,
                material_rgb_error=rgb_error,
            )
        )
    return panels


if __name__ == "__main__":
    print(
        json.dumps(
            {
                name: measure(name, file)
                for name, file in [
                    ("bent", "Bent swatch.obj"),
                    ("extended", "Extended bent swatch.obj"),
                    ("harbor", "Harbor draped.obj"),
                ]
            },
            indent=2,
            allow_nan=False,
        )
    )
