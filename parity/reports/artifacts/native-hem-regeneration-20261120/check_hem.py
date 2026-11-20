"""Replay the native Harbor dimension edit against its saved construction and OBJ surfaces."""
import copy
import json
import math
import runpy
import sys
from pathlib import Path


def project(path):
    document = json.loads(path.read_bytes())
    assert document["version"] == 6
    return document["project"]


def geometry(attributes, indices):
    return {attributes["vt"][index]: attributes["v"][index] for index in indices}



def seam_side_lengths(doc):
    points = {p["id"]: p["position"] for p in doc["points"]}
    curves = {c["id"]: c for c in doc["curves"]}
    result = {}
    for seam in doc["seams"]:
        lengths = []
        for side in ("a", "b"):
            span = seam[side]
            curve = curves[span["curve"]]
            assert curve["controls"] is None
            lengths.append(math.dist(points[curve["start"]], points[curve["end"]]) * abs(span["t1"] - span["t0"]))
        result[seam["id"]] = lengths
    return result

def verify(repo, case):
    artifacts = repo / "parity/reports/artifacts"
    original = project(artifacts / "native-harbor-simulated-obj-20261118/inputs/Harbor.weftforge")
    edited = project(case / "Harbor hem.weftforge")
    reopened_edit = project(case / "reopened-edit.weftforge")

    # Reconstruct the complete expected document, including all unaffected authored entities.
    assert original["revision"] == 23 and len(original["seams"]) == 8
    expected = copy.deepcopy(original)
    for point, u in zip(expected["points"][:2], (-1.08, -0.36), strict=True):
        assert point["position"] == [u, 0.96]
        point["position"][1] = 0.91
    expected["revision"] = 25
    assert edited == expected
    expected["points"][0]["position"][1] = 0.96
    expected["revision"] = 26
    assert reopened_edit == expected

    read_obj = runpy.run_path(str(artifacts / "static-glb-20261119/compare_glb.py"))["read_obj"]
    maps = {}
    counts = {}
    topology = {}
    for name in ["original-step", "edited-rest", "reopened-step"]:
        attributes, faces = read_obj(case / (name + ".obj"))
        assert set(faces) == {"2", "31"}
        assert len(attributes["v"]) == len(attributes["vt"]) == len(attributes["vn"])
        assert all(math.isfinite(x) for values in attributes.values() for row in values for x in row)
        indices = {group: {i for triangle in triangles for i in triangle} for group, triangles in faces.items()}
        assert indices["2"].isdisjoint(indices["31"])
        assert indices["2"] | indices["31"] == set(range(len(attributes["v"])))
        maps[name] = {group: geometry(attributes, ids) for group, ids in indices.items()}
        assert all(len(maps[name][group]) == len(ids) for group, ids in indices.items())
        topology[name] = {group: [tuple(attributes["vt"][i] for i in face) for face in faces[group]] for group in faces}
        counts[name] = {"vertices": len(attributes["v"]), "triangles": sum(map(len, faces.values()))}

    assert topology["reopened-step"] == topology["edited-rest"]
    assert topology["original-step"]["31"] == topology["edited-rest"]["31"]
    rest = maps["edited-rest"]
    stepped = maps["reopened-step"]
    original_stepped = maps["original-step"]
    height = lambda coords: max(v for _, v in coords) - min(v for _, v in coords)
    assert math.isclose(height(rest["2"]), 0.69, abs_tol=1e-12)
    assert math.isclose(height(original_stepped["2"]), 0.64, abs_tol=1e-12)
    translations = {str(p["id"]): p["placement"]["translation"] for p in edited["pieces"]}
    assert all(p["placement"]["rotation"] == [0, 0, 0] for p in edited["pieces"])
    max_rest_error = max(
        math.dist(pos, (u + translations[group][0], v + translations[group][1], translations[group][2]))
        for group, vertices in rest.items() for (u, v), pos in vertices.items()
    )
    assert max_rest_error < 2e-6
    before_lengths, after_lengths = map(seam_side_lengths, (original, edited))
    changed_pairs = {seam: {"before_m": pair, "after_m": after_lengths[seam]}
                     for seam, pair in before_lengths.items() if pair != after_lengths[seam]}
    assert set(changed_pairs) == {60, 67}
    hem_shifts = []
    hem_sim_displacements = []
    for u in [-1.08, -0.36]:
        old = original_stepped["2"][u, 0.96]
        new = stepped["2"][u, 0.91]
        hem_shifts.append(new[1] - old[1])
        hem_sim_displacements.append(math.dist(new, rest["2"][u, 0.91]))
    assert all(d > 1e-6 for d in hem_sim_displacements)
    # Report measured world-space effects separately from the exact 50 mm material-space edit.
    return {
        "revisions": [original["revision"], edited["revision"], reopened_edit["revision"]],
        "front_height_before_after_m": [height(original_stepped["2"]), height(stepped["2"])],
        "counts": counts,
        "unchanged_seams": len(edited["seams"]),
        "seams_with_changed_rest_lengths": changed_pairs,
        "unchanged_authored_points": len(original["points"]) - 2,
        "maximum_rest_placement_error_m": max_rest_error,
        "hem_left_right_stepped_y_change_m": hem_shifts,
        "hem_left_right_simulated_displacement_from_rest_m": hem_sim_displacements,
        "edited_rest_and_stepped_uv_topology_equal": True,
        "back_uv_topology_equal_across_all_exports": True,
    }


if __name__ == "__main__":
    print(json.dumps(verify(Path(sys.argv[1]), Path(__file__).resolve().parent), indent=2))
