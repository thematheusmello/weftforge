#!/usr/bin/env python3
"""Verify the native Front-view rotation against the original editable outline and captures."""
from copy import deepcopy
from hashlib import sha256
import json
import math
from pathlib import Path
import sys

from PIL import Image, ImageChops


def check(original_path, case):
    files = [original_path, *(case / f"{name}.weftforge" for name in ("drag", "undo", "redo"))]
    original, turned, undone, redone = [json.loads(path.read_bytes()) for path in files]
    project = original["project"]
    assert [doc["project"]["revision"] for doc in (original, turned, undone, redone)] == [23, 24, 25, 26]
    assert len(project["pieces"]) == 2 and len(project["seams"]) == 8
    front = next(piece for piece in project["pieces"] if piece["id"] == 2)
    curves = {curve["id"]: curve for curve in project["curves"]}
    points = {point["id"]: point["position"] for point in project["points"]}
    boundary = [curves[index] for index in front["boundary"]]
    assert len(boundary) == 14 and all(curve["controls"] is None for curve in boundary)
    vertices = [points[curve["start"]] for curve in boundary]
    pivot = [(min(axis) + max(axis)) / 2 for axis in zip(*vertices)]
    initial = front["placement"]
    placement = next(p for p in turned["project"]["pieces"] if p["id"] == front["id"])["placement"]
    assert initial["rotation"] == [0, 0, 0] and placement["rotation"][:2] == [0, 0]
    angle = placement["rotation"][2]
    # The two native pointer rays bracket a quarter turn to within one screen pixel.
    assert abs(math.degrees(angle) - 90) < 1
    cos, sin = math.cos(angle), math.sin(angle)
    tx, ty, tz = placement["translation"]
    old_x, old_y, old_z = initial["translation"]
    assert tz == old_z
    def world(x, y):
        return [tx + cos*x - sin*y, ty + sin*x + cos*y, tz]
    center = [old_x + pivot[0], old_y + pivot[1], old_z]
    pivot_error = math.dist(world(*pivot), center)
    assert pivot_error < 1e-12
    for x, y in vertices:
        dx, dy = x - pivot[0], y - pivot[1]
        predicted = [center[0] + cos*dx - sin*dy, center[1] + sin*dx + cos*dy, center[2]]
        assert math.dist(world(x, y), predicted) < 1e-12
    expected = deepcopy(original)
    next(piece for piece in expected["project"]["pieces"] if piece["id"] == front["id"])["placement"] = placement
    for doc, template, revision in [(turned, expected, 24), (undone, original, 25), (redone, expected, 26)]:
        assert doc == dict(template, project=dict(template["project"], revision=revision))

    captures = [case / f"{name}.png" for name in ("before", "after")]
    images = []
    for path in captures:
        with Image.open(path) as im:
            assert (im.format, im.mode, im.size, im.info["srgb"]) == ("PNG", "RGBA", (1640, 1000), 0)
            images.append(im.convert("RGB"))
    diff = ImageChops.difference(*images)
    assert diff.crop((46, 117, 696, 944)).getbbox() is None
    garment = diff.crop((700, 146, 1348, 944))
    changed = sum(pixel != (0, 0, 0) for pixel in garment.get_flattened_data())
    assert changed > 0
    return {
        "revisions": [23, 24, 25, 26], "piece": front["name"], "camera": "Front",
        "angle_radians": angle, "angle_degrees": math.degrees(angle), "pivot_world_m": center,
        "pivot_error_m": pivot_error, "outline_vertices_checked": len(vertices),
        "other_authored_fields_match": True, "undo_and_redo_match_complete_documents": True,
        "changed_2d_canvas_pixels": 0, "changed_3d_canvas_pixels": changed,
        "sha256": {p.name: sha256(p.read_bytes()).hexdigest() for p in [*files, *captures]},
    }


if __name__ == "__main__":
    print(json.dumps(check(Path(sys.argv[1]), Path(__file__).resolve().parent), indent=2))
