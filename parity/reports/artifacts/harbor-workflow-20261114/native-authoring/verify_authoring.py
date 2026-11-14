#!/usr/bin/env python3
"""Read-only independent checks of native UI-authored JSON receipts."""
from pathlib import Path
import copy
import hashlib
import json
import math

ROOT = Path(__file__).resolve().parent
PLAN_SHA256 = "0f07ac196c8e285c1441dc3a16f0027f65c2b901b4630abfa180a216b56e1ddd"

def load(name):
    return json.loads((ROOT / name).read_text())

def digest(name):
    return hashlib.sha256((ROOT / name).read_bytes()).hexdigest()

def without_revision(doc):
    result = copy.deepcopy(doc)
    result["project"].pop("revision")
    return result

def vertices(doc, piece_index):
    q = doc["project"]
    points = {item["id"]: item["position"] for item in q["points"]}
    curves = {item["id"]: item for item in q["curves"]}
    piece = q["pieces"][piece_index]
    boundary = piece["boundary"]
    assert len(boundary) == 14 and len(set(boundary)) == 14
    for i, curve_id in enumerate(boundary):
        curve = curves[curve_id]
        assert curve["controls"] is None
        assert curve["end"] == curves[boundary[(i + 1) % 14]]["start"]
    return [points[curves[c]["start"]] for c in boundary]

def assert_frozen(doc, revision, arranged=True, sewn=True):
    q = doc["project"]
    assert doc["format"] == "weftforge" and doc["version"] == 3
    assert q["revision"] == revision
    assert len(q["points"]) == 28 and len(q["curves"]) == 28
    assert len(q["pieces"]) == 2 and q["sewing_groups"] == []
    assert q["materials"] == [PLAN["material"]]
    assert q["resolution"] == PLAN["resolution"]
    assert q["mannequin"] == EMPTY["project"]["mannequin"]
    for index, expected in enumerate(PLAN["pieces"]):
        piece = q["pieces"][index]
        assert piece["name"] == expected["name"]
        assert piece["material"] == 1
        assert vertices(doc, index) == expected["vertices"]
        if arranged:
            assert piece["placement"] == expected["placement"]
            assert piece["grain_angle"] == expected["grain_angle"]
    assert len(q["seams"]) == (8 if sewn else 0)
    if sewn:
        for seam, edge in zip(q["seams"], PLAN["seams_zero_based_matching_boundary_indices"]):
            a = q["pieces"][0]["boundary"][edge]
            b = q["pieces"][1]["boundary"][edge]
            assert seam["a"] == {"curve": a, "t0": 0.0, "t1": 1.0}
            assert seam["b"] == {"curve": b, "t0": 0.0, "t1": 1.0}
            assert seam["correspondence"] == [[0.0, 0.0], [1.0, 1.0]]

assert digest("frozen-plan.json") == PLAN_SHA256
PLAN = load("frozen-plan.json")
EMPTY = load("empty-start.weftforge")
assert EMPTY["format"] == "weftforge" and EMPTY["version"] == 3
assert EMPTY["project"]["revision"] == 0
for key in ["points", "curves", "pieces", "seams", "sewing_groups"]:
    assert EMPTY["project"][key] == []
assert EMPTY["project"]["materials"] == [PLAN["material"]]
assert EMPTY["project"]["resolution"] == PLAN["resolution"]
FRONT = load("front-created.weftforge")
assert FRONT["project"]["revision"] == 1
assert len(FRONT["project"]["pieces"]) == 1
assert vertices(FRONT, 0) == PLAN["pieces"][0]["vertices"]
assert_frozen(load("two-panels-created.weftforge"), 2, arranged=False, sewn=False)
assert_frozen(load("panels-arranged.weftforge"), 8, sewn=False)
AUTHORED = load("harbor-authored.weftforge")
assert_frozen(AUTHORED, 16)
REOPENED = load("harbor-reopened.weftforge")
assert REOPENED == AUTHORED
assert (ROOT / "harbor-reopened.weftforge").read_bytes() == (ROOT / "harbor-authored.weftforge").read_bytes()
EDITED = load("height-edited.weftforge")
assert EDITED["project"]["revision"] == 17
for key in AUTHORED["project"]:
    if key not in ["points", "revision"]:
        assert EDITED["project"][key] == AUTHORED["project"][key]
max_formula_error = 0.0
for actual, original in zip(vertices(EDITED, 0), PLAN["pieces"][0]["vertices"]):
    expected = [original[0], 0.96 + (original[1] - 0.96) * 0.68 / 0.64]
    error = max(abs(a - b) for a, b in zip(actual, expected))
    max_formula_error = max(max_formula_error, error)
    assert error < 1e-14
assert vertices(EDITED, 1) == PLAN["pieces"][1]["vertices"]
for name, rev, expected in [
    ("height-undone.weftforge", 18, AUTHORED),
    ("height-redone.weftforge", 19, EDITED),
    ("harbor-final-restored.weftforge", 20, AUTHORED),
]:
    actual = load(name)
    assert actual["project"]["revision"] == rev
    assert without_revision(actual) == without_revision(expected)
assert_frozen(load("harbor-final-restored.weftforge"), 20)
NAMES = ["empty-start.weftforge", "front-created.weftforge", "two-panels-created.weftforge", "panels-arranged.weftforge", "harbor-authored.weftforge", "harbor-reopened.weftforge", "height-edited.weftforge", "height-undone.weftforge", "height-redone.weftforge", "harbor-final-restored.weftforge"]
REPORT = {
    "checks_passed": True,
    "frozen_plan_sha256": PLAN_SHA256,
    "receipts": [{"file": name, "revision": load(name)["project"]["revision"], "sha256": digest(name)} for name in NAMES],
    "reopen_byte_identical": True,
    "undo_and_final_construction_exact_except_monotonic_revision": True,
    "redo_construction_exact_except_monotonic_revision": True,
    "height_formula_max_f64_error_m": max_formula_error,
    "quality_status": "Authoring only; physics 15%/5mm/2mm gates pending; previous weak-grain benchmark remains failing; no reference parity claim.",
}
print(json.dumps(REPORT, indent=2))
