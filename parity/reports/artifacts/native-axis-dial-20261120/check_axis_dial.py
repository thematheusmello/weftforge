#!/usr/bin/env python3
"""Check the saved Harbor documents and native workspace export using only the recorded inputs."""
from copy import deepcopy
from hashlib import sha256
import json
import math
from pathlib import Path
import sys

from PIL import Image


def multiply(a, b):
    return [[sum(x*y for x, y in zip(row, column)) for column in zip(*b)] for row in a]


def apply(matrix, vector):
    return [sum(x*y for x, y in zip(row, vector)) for row in matrix]


def axis_rotation(index, angle):
    c, s = math.cos(angle), math.sin(angle)
    r = [[float(i == j) for j in range(3)] for i in range(3)]
    u, v = (index+1) % 3, (index+2) % 3
    r[u][u], r[u][v], r[v][u], r[v][v] = c, -s, s, c
    return r


def basis(placement):
    x, y, z = [axis_rotation(i, angle) for i, angle in enumerate(placement["rotation"])]
    return multiply(z, multiply(y, x))


def transform(placement, uv):
    return [a+b for a, b in zip(apply(basis(placement), [*uv, 0]), placement["translation"])]


def matrix_error(a, b):
    return max(abs(x-y) for ra, rb in zip(a, b) for x, y in zip(ra, rb))


def check(fixture, directory):
    names = ["world-x", "local-z", "undo", "redo"]
    paths = [fixture, *(directory / f"{name}.weftforge" for name in names)]
    docs = [json.loads(path.read_bytes()) for path in paths]
    revs = [doc["project"]["revision"] for doc in docs]
    assert revs == [23, 24, 25, 26, 27]
    original = docs[0]["project"]
    assert len(original["pieces"]) == 2 and len(original["seams"]) == 8
    selected = next(i for i, piece in enumerate(original["pieces"]) if piece["id"] == 2)
    piece = original["pieces"][selected]
    assert piece["name"] == "Harbor front"
    placements = [doc["project"]["pieces"][selected]["placement"] for doc in docs]
    assert placements[0]["rotation"] == [0, 0, 0]
    for doc, revision, placement in zip(docs, revs, placements):
        expected = deepcopy(docs[0])
        expected["project"]["revision"] = revision
        expected["project"]["pieces"][selected]["placement"] = placement
        assert doc == expected
    assert placements[3] == placements[1] and placements[4] == placements[2]

    first, combined = map(basis, placements[1:3])
    # Test the complete orientation matrices. The second axis is local to the first placement.
    a = math.atan2(first[2][1], first[1][1])
    relative = multiply(list(zip(*first)), combined)
    b = math.atan2(relative[1][0], relative[0][0])
    wx, lz = axis_rotation(0, a), axis_rotation(2, b)
    expected_bases = [wx, multiply(wx, lz)]
    orientation_errors = [matrix_error(m, r) for m, r in zip([first, combined], expected_bases)]
    assert max(orientation_errors) < 1e-12
    world_equivalent = matrix_error(combined, multiply(lz, wx))
    assert world_equivalent > 0.5
    # Compare signed turns to independently observed drag rays; the pivot is recorded to half a pixel.
    center = [1022.5, 512.5]
    start, ends = [1053, 513], [[1044, 491], [1023, 483]]
    def ray(p):
        return math.atan2(center[1] - p[1], p[0] - center[0])
    sweeps = [ray(end) - ray(start) for end in ends]
    pointer_errors = [abs(s-t) for s, t in zip(sweeps, [a, b])]
    assert max(pointer_errors) < math.radians(1)

    pts = {p["id"]: p["position"] for p in original["points"]}
    curves = {c["id"]: c for c in original["curves"]}
    boundary = [curves[idx] for idx in piece["boundary"]]
    assert all(c["controls"] is None for c in boundary)
    vertices = [pts[c["start"]] for c in boundary]
    pivot = [(min(axis) + max(axis))/2 for axis in zip(*vertices)]
    anchored = transform(placements[0], pivot)
    anchor_errors = [math.dist(transform(p, pivot), anchored) for p in placements[1:3]]
    vertex_errors = []
    for r, placement in zip(expected_bases, placements[1:3]):
        errors = []
        for uv in vertices:
            delta = [p-q for p, q in zip(transform(placements[0], uv), anchored)]
            expected = [p+q for p, q in zip(apply(r, delta), anchored)]
            errors.append(math.dist(expected, transform(placement, uv)))
        vertex_errors.append(max(errors))
    assert max(anchor_errors + vertex_errors) < 1e-12

    png = directory / "workspace.png"
    with Image.open(png) as image:
        assert (image.format, image.mode, image.size, image.info["srgb"]) == ("PNG", "RGBA", (1640, 1000), 0)
        # The exported viewport omits the macOS title bar (32 points). Sample the visible dial.
        pts = [(x, y) for y in range(440, 521) for x in range(982, 1063)
               if image.getpixel((x,y)) == (243,182,101,255)]
    cx, cy = center[0], center[1]-32
    rim = [(x, y) for x, y in pts if 28 < math.hypot(x-cx, y-cy) < 32]
    quadrants = [sum((x >= cx) == right and (y >= cy) == down for x,y in rim)
                 for right, down in [(False,False),(True,False),(False,True),(True,True)]]
    assert min(quadrants) > 5
    assert any(2.5 <= math.hypot(x-cx, y-cy) <= 5 for x,y in pts)
    assert any(x > cx+32 and abs(y-cy) <= 4 for x,y in pts)
    return {
        "revisions": revs, "piece": piece["name"],
        "world_x_radians": a, "local_z_radians": b,
        "world_x_degrees": math.degrees(a), "local_z_degrees": math.degrees(b),
        "local_z_axis_in_world": [row[2] for row in first],
        "screen_turn_errors_degrees": list(map(math.degrees, pointer_errors)),
        "maximum_matrix_component_errors": orientation_errors,
        "difference_from_second_turn_about_world_z": world_equivalent,
        "fixed_pivot_m": anchored, "pivot_errors_m": anchor_errors,
        "boundary_vertices_checked_per_turn": len(vertices), "worst_vertex_errors_m": vertex_errors,
        "other_authored_fields_match_entire_original": True,
        "undo_and_redo_match_complete_documents_except_revision": True,
        "placements": dict(zip(names, placements[1:])),
        "dial_rim_pixels_by_quadrant": quadrants,
        "workspace_png_size": [1640, 1000],
        "sha256": {path.name: sha256(path.read_bytes()).hexdigest() for path in [*paths, png]},
    }


if __name__ == "__main__":
    print(json.dumps(check(Path(sys.argv[1]), Path(__file__).resolve().parent), indent=2))
