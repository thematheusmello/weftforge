"""Replay the native Harbor fabric snapshots using Python's standard library."""

import argparse
import copy
import json
import struct
from pathlib import Path


def check(original, directory):
    base = json.loads(original.read_bytes())
    initial = base["project"]
    source, = initial["materials"]
    assert source["id"] == 1 and source["areal_density"] == 0.18
    assert initial["revision"] == 24
    assert [(p["id"], p["material"]) for p in initial["pieces"]] == [(2, 1), (31, 1)]
    allocated = initial["next_id"]
    assert allocated == 68
    states = [
        ("copied", "Harbor lining", 0.18, 1),
        ("renamed", "Harbor twill", 0.18, 1),
        ("independent", "Harbor twill", 0.29, 1),
        ("assigned", "Harbor twill", 0.29, allocated),
        ("undo", "Harbor twill", 0.29, 1),
        ("redo", "Harbor twill", 0.29, allocated),
    ]
    snapshots = {}
    for revision, (stage, name, density, front_material) in enumerate(states, 25):
        expected = copy.deepcopy(base)
        project = expected["project"]
        project["revision"] = revision
        project["next_id"] += 1
        duplicate = dict(source, id=allocated, name=name, areal_density=density)
        project["materials"].append(duplicate)
        project["pieces"][0]["material"] = front_material
        project["pieces"][1]["material"] = allocated
        actual = json.loads((directory / (stage + ".weftforge")).read_bytes())
        assert actual == expected, stage
        snapshots[stage] = actual

    for previous, restored in [("independent", "undo"), ("assigned", "redo")]:
        expected = copy.deepcopy(snapshots[previous])
        expected["project"]["revision"] = snapshots[restored]["project"]["revision"]
        assert expected == snapshots[restored], restored

    png = (directory / "harbor-independent.png").read_bytes()
    assert png.startswith(b"\x89PNG\r\n\x1a\n") and png[12:16] == b"IHDR"
    dimensions = struct.unpack(">II", png[16:24])
    assert dimensions == (1640, 1000)
    return {
        "revisions": list(range(24, 31)),
        "piece_ids": [2, 31],
        "seam_count": len(initial["seams"]),
        "original_material_id": 1,
        "allocated_material_id": allocated,
        "final_next_id": allocated + 1,
        "inherited_fields": sorted(set(source) - {"id", "name"}),
        "density_kg_per_square_metre": {"original": 0.18, "edited_copy": 0.29},
        "assignments_by_stage": {
            stage: [p["material"] for p in obj["project"]["pieces"]]
            for stage, obj in snapshots.items()
        },
        "complete_snapshots_equal_expected": True,
        "undo_redo_restore_all_fields_except_revision": True,
        "workspace_png_dimensions": list(dimensions),
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("original", type=Path)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    print(json.dumps(check(args.original, args.directory), indent=2))
