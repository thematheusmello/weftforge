#!/usr/bin/env python3
"""Replay the recorded event distribution and check the saved Harbor placements."""
import copy
import json
import math
from pathlib import Path

CASE = Path(__file__).resolve().parent
ARTIFACTS = CASE.parent


def read(path):
    return json.loads(path.read_bytes())


def check():
    frames = read(CASE / "FRAMES.json")
    baseline, corrected = frames["baseline"], frames["corrected"]
    assert [row["rev_after"] for row in baseline] == [23, 23, 24]
    assert [row["rev_after"] for row in corrected] == [24, 24, 25]
    for sequence in [baseline[:2], corrected[1:]]:
        press, release = sequence
        assert press["frame"] + 1 == release["frame"]
        assert press["canvas"] == release["canvas"]
        assert press["ready"] and release["ready"] and press["world_x"] and release["world_x"]
        assert press["armed_after"] and release["armed_before"] and not release["armed_after"]
        assert not release["fully_coalesced"] and not release["egui_drag_stopped"]
        for frame, kind in [(press, "down"), (release, "up")]:
            button, = [event for event in frame["raw"] if event["kind"] == kind]
            assert button["button"] == "Primary"
            assert frame["os_buttons"] == [{"kind":kind, "xy":button["xy"], "key_window":True}]
    assert [r["raw"] for r in baseline[:2]] == [r["raw"] for r in corrected[1:]]
    assert baseline[-1]["fully_coalesced"] and corrected[0]["fully_coalesced"]
    start = corrected[1]["os_buttons"][0]["xy"]
    end = corrected[2]["os_buttons"][0]["xy"]
    cx, cy = corrected[1]["dial"]
    assert corrected[2]["dial"] == [cx, cy]
    angle = math.atan2(cy-end[1], end[0]-cx) - math.atan2(cy-start[1], start[0]-cx)

    original = read(ARTIFACTS / "native-harbor-simulated-obj-20261118/inputs/Harbor.weftforge")
    prior = ARTIFACTS / "native-axis-dial-20261120"
    # The saved first turn and Undo matched these previously retained files byte for byte.
    paths = [prior / "world-x.weftforge", CASE / "split-release.weftforge",
             prior / "undo.weftforge", CASE / "redo.weftforge"]
    docs = [read(path) for path in paths]
    project = original["project"]
    front = project["pieces"][0]
    assert front["id"] == 2 and len(project["pieces"]) == 2 and len(project["seams"]) == 8
    points = {p["id"]: p["position"] for p in project["points"]}
    curves = {c["id"]: c for c in project["curves"]}
    assert all(curves[i]["controls"] is None for i in front["boundary"])
    outline = [points[curves[i]["start"]] for i in front["boundary"]]
    uv_center = [(min(coords)+max(coords))/2 for coords in zip(*outline)]
    initial = front["placement"]
    assert initial["rotation"] == [0, 0, 0]
    pivot = [a+b for a, b in zip(initial["translation"], [*uv_center, 0])]
    worst = 0.
    for doc, revision, turns in zip(docs, [24, 25, 26, 27], [1, 2, 1, 2]):
        placement = doc["project"]["pieces"][0]["placement"]
        expected = copy.deepcopy(original)
        expected["project"]["revision"] = revision
        expected["project"]["pieces"][0]["placement"] = placement
        assert expected == doc
        x, y, z = placement["rotation"]
        assert max(abs(x-turns*angle), abs(y), abs(z)) < 1e-12
        tx, ty, tz = placement["translation"]
        for u, v in [uv_center, *outline]:
            actual = [tx+u, ty+v*math.cos(x), tz+v*math.sin(x)]
            dv = v-uv_center[1]
            predicted = [pivot[0]+u-uv_center[0], pivot[1]+dv*math.cos(turns*angle), pivot[2]+dv*math.sin(turns*angle)]
            worst = max(worst, math.dist(actual, predicted))
    assert worst < 1e-12
    for a, b in [(0, 2), (1, 3)]:
        expected = copy.deepcopy(docs[a])
        expected["project"]["revision"] = docs[b]["project"]["revision"]
        assert expected == docs[b]
    return {"baseline_revisions": [23, 23, 24], "fixed_revisions": [23, 24, 25, 26, 27],
            "same_split_frame_events": True, "turn_radians": angle,
            "saved_two_turns_radians": docs[1]["project"]["pieces"][0]["placement"]["rotation"][0],
            "fixed_pivot_m": pivot, "boundary_vertices": len(outline), "maximum_position_error_m": worst,
            "undo_redo_match_complete_documents_except_revision": True,
            "all_other_authored_fields_match_original": True}


if __name__ == "__main__":
    print(json.dumps(check(), indent=2))
