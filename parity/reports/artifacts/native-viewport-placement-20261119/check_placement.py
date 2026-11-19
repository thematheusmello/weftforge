#!/usr/bin/env python3
"""Check the saved native edits and compare the two native canvas captures."""

from copy import deepcopy
from decimal import Decimal
from hashlib import sha256
import json
from pathlib import Path
import sys

from PIL import Image, ImageChops


def check(original_path, case):
    files = [original_path, *(case / name for name in ("drag.weftforge", "undo.weftforge", "redo.weftforge"))]
    original, moved, undone, redone = [
        json.loads(path.read_text(), parse_float=Decimal) for path in files
    ]
    project = original["project"]
    assert project["revision"] == 23 and len(project["pieces"]) == 2 and len(project["seams"]) == 8
    front = next(piece for piece in project["pieces"] if piece["id"] == 2)
    translation = front["placement"]["translation"]
    displacement = tuple(Decimal(value) for value in ("0.32", "0.18", "0"))
    expected = deepcopy(original)
    expected["project"]["revision"] = 24
    next(piece for piece in expected["project"]["pieces"] if piece["id"] == 2)[
        "placement"
    ]["translation"] = [coordinate + offset for coordinate, offset in zip(translation, displacement, strict=True)]
    assert expected == moved
    assert dict(original, project=dict(project, revision=25)) == undone
    assert dict(expected, project=dict(expected["project"], revision=26)) == redone

    captures = [case / name for name in ("before.png", "after.png")]
    images = []
    for path in captures:
        with Image.open(path) as image:
            assert image.format == "PNG" and image.mode == "RGBA"
            assert image.size == (1640, 1000) and image.info["srgb"] == 0
            images.append(image.convert("RGB"))
    diff = ImageChops.difference(*images)
    assert diff.crop((46, 117, 696, 944)).getbbox() is None
    region = (700, 146, 1348, 944)
    garment = diff.crop(region)
    bounds = garment.getbbox()
    assert bounds is not None
    changed = sum(pixel != (0, 0, 0) for pixel in garment.get_flattened_data())

    return {
        "revisions": [item["project"]["revision"] for item in (original, moved, undone, redone)],
        "moved_piece": {"id": 2, "name": front["name"], "displacement_m": list(map(str, displacement))},
        "undo_and_redo_match_complete_document": True,
        "unaffected_authored_fields_match": True,
        "captures": {
            "size_px": [1640, 1000],
            "format": "RGBA8 PNG with sRGB intent 0",
            "2d_difference_pixels": 0,
            "3d_difference_pixels": changed,
            "3d_difference_bounds_px": [value + region[i % 2] for i, value in enumerate(bounds)],
        },
        "sha256": {path.name: sha256(path.read_bytes()).hexdigest() for path in [*files, *captures]},
    }


if __name__ == "__main__":
    print(json.dumps(check(Path(sys.argv[1]), Path(__file__).resolve().parent), indent=2))
