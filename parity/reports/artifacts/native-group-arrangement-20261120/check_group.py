"""Compare the native two-pattern arrangement, its original fixture, and both workspace PNGs.

Usage: python3 check_group.py original.weftforge case_directory
Requires Pillow for the workspace pixel check. Geometry uses Python's math module.
"""

import argparse
import copy
import json
import math
from pathlib import Path
from PIL import Image


def add(a, b):
    return [x + y for x, y in zip(a, b)]


def sub(a, b):
    return [x - y for x, y in zip(a, b)]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def cross(a, b):
    return [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]


def transform(placement, xyz, translate=True):
    x, y, z = xyz
    (sx, cx), (sy, cy), (sz, cz) = [
        (math.sin(a), math.cos(a)) for a in placement["rotation"]
    ]
    y, z = cx * y - sx * z, sx * y + cx * z
    x, z = cy * x + sy * z, -sy * x + cy * z
    x, y = cz * x - sz * y, sz * x + cz * y
    return add([x, y, z], placement["translation"]) if translate else [x, y, z]


def rotate(axis, v, angle):
    cos, sin = math.cos(angle), math.sin(angle)
    return [
        cos * x + sin * y + (1 - cos) * dot(axis, v) * k
        for x, y, k in zip(v, cross(axis, v), axis)
    ]


def dial_pixels(path):
    with Image.open(path) as image:
        rgb = image.convert("RGB")
    assert rgb.size == (1640, 1000)
    points = {
        (x, y)
        for y in range(413, 495)
        for x in range(1035, 1125)
        if rgb.getpixel((x, y)) == (243, 182, 101)
    }
    assert len(points) > 100
    return points


def check(original, directory):
    docs = [
        json.loads(path.read_bytes())
        for path in [
            original,
            *[
                directory / (n + ".weftforge")
                for n in ["translated", "rotated", "undo", "redo"]
            ],
        ]
    ]
    revisions = [d["project"]["revision"] for d in docs]
    assert revisions == [24, 25, 26, 27, 28]
    project = docs[0]["project"]
    assert [p["id"] for p in project["pieces"]] == [2, 31] and len(
        project["seams"]
    ) == 8
    for doc, rev in zip(docs[1:], revisions[1:]):
        expected = copy.deepcopy(docs[0])
        expected["project"]["revision"] = rev
        for old, new in zip(
            expected["project"]["pieces"], doc["project"]["pieces"], strict=True
        ):
            old["placement"] = new["placement"]
        assert expected == doc
    for a, b in [(1, 3), (2, 4)]:
        expected = copy.deepcopy(docs[a])
        expected["project"]["revision"] = revisions[b]
        assert expected == docs[b]
    original_pose, moved, turned = [
        [p["placement"] for p in d["project"]["pieces"]] for d in docs[:3]
    ]
    translation = [54 / 270, 27 / 270, 0.0]
    for a, b in zip(original_pose, moved):
        assert (
            a["rotation"] == b["rotation"]
            and math.dist(add(a["translation"], translation), b["translation"]) < 1e-14
        )
    point_by_id = {p["id"]: p["position"] for p in project["points"]}
    curve_by_id = {c["id"]: c for c in project["curves"]}
    circles = []
    for p in project["pieces"]:
        boundary = [curve_by_id[c] for c in p["boundary"]]
        assert all(c["controls"] is None for c in boundary)
        circles.append([point_by_id[c["start"]] for c in boundary])
    centers = [
        [0.5 * (min(axis) + max(axis)) for axis in zip(*ring)] + [0.0]
        for ring in circles
    ]
    world_centers = [transform(p, c) for p, c in zip(moved, centers)]
    pivot = [sum(axis) / 2 for axis in zip(*world_centers)]
    axis = transform(moved[0], [0.0, 0.0, 1.0], False)
    # Infer the signed twist from just the active panel's original and saved U
    # basis vectors. The other axes, both origins and all authored corners remain
    # independent constraints on that one rigid transform.
    old_u = transform(moved[0], [1.0, 0.0, 0.0], False)
    new_u = transform(turned[0], [1.0, 0.0, 0.0], False)
    angle = math.atan2(dot(axis, cross(old_u, new_u)), dot(old_u, new_u))
    assert abs(angle - math.pi / 2) < math.radians(2)
    pre = dial_pixels(directory / "group-before-turn.png")
    post = dial_pixels(directory / "group-after-turn.png")
    assert pre == post
    # Gold rim extremes give the saved camera's pivot center to a pixel. The
    # golden knob extends the right extreme. Use top/bottom and left rim radius.
    left = min(x for x, y in pre)
    cy = (min(y for x, y in pre) + max(y for x, y in pre)) / 2 + 32.5
    cx = left + 30.5
    screen_angle = math.atan2(cy - 456, 1077 - cx) - math.atan2(cy - 486, 1107 - cx)
    assert abs(angle - screen_angle) < math.radians(2)
    basis_error = vertex_error = pivot_error = world_z_disagreement = 0.0
    for a, b, ring, center in zip(moved, turned, circles, centers):
        for v in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]:
            old = transform(a, v, False)
            new = transform(b, v, False)
            basis_error = max(basis_error, math.dist(new, rotate(axis, old, angle)))
            world_z_disagreement = max(
                world_z_disagreement, math.dist(new, rotate([0, 0, 1], old, angle))
            )
        for xyz in [[0.0, 0.0, 0.0], center, *[[*uv, 0.0] for uv in ring]]:
            expected = add(pivot, rotate(axis, sub(transform(a, xyz), pivot), angle))
            vertex_error = max(vertex_error, math.dist(expected, transform(b, xyz)))
    new_pivot = [
        sum(coords) / 2
        for coords in zip(*(transform(p, c) for p, c in zip(turned, centers)))
    ]
    pivot_error = math.dist(new_pivot, pivot)
    assert (
        max(basis_error, vertex_error, pivot_error) < 1e-12
        and world_z_disagreement > 0.5
    )
    return {
        "revisions": revisions,
        "piece_ids": [2, 31],
        "active_piece": 2,
        "seams": 8,
        "rest_mesh_vertices_observed": 654,
        "view_plane_translation_m": translation,
        "world_pivot_m": pivot,
        "active_local_z_world": axis,
        "signed_rotation_radians": angle,
        "signed_rotation_degrees": math.degrees(angle),
        "turn_from_native_dial_pixels_degrees": math.degrees(screen_angle),
        "unchanged_dial_pixels": len(pre),
        "authored_boundary_vertices_checked": sum(map(len, circles)),
        "max_world_position_error_m": vertex_error,
        "max_basis_vector_error": basis_error,
        "shared_pivot_error_m": pivot_error,
        "world_z_alternative_max_basis_disagreement": world_z_disagreement,
        "all_other_saved_fields_identical": True,
        "undo_redo_restore_complete_documents_except_revision": True,
    }


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("original", type=Path)
    p.add_argument("directory", type=Path)
    args = p.parse_args()
    print(json.dumps(check(args.original, args.directory), indent=2))
