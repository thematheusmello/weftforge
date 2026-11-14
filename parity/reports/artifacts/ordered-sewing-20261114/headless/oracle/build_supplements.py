"""Original prospective supplements; never reads production source or output."""
from copy import deepcopy
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent
ORIGINAL_FREEZE = "d84f38184dded9a574ace4e30c6a3860029d1b4fa9ae98e0e50d52bf73ac1ed0"


def write(name, value):
    with (ROOT / name).open("x") as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write("\n")


def revision(source, number):
    result = deepcopy(source)
    result["project"]["revision"] = number
    return result


def main():
    freeze_path = ROOT / "freeze.json"
    assert hashlib.sha256(freeze_path.read_bytes()).hexdigest() == ORIGINAL_FREEZE
    freeze = json.loads(freeze_path.read_text())
    for name, record in freeze["artifacts"].items():
        assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == record["sha256"], name
    original = json.loads((ROOT / "fixtures/line-identity-coarse.weftforge").read_text())
    baseline = deepcopy(original)
    baseline["project"]["sewing_groups"] = []
    assert baseline["project"]["revision"] == 73 and baseline["project"]["next_id"] == 1001
    write("native-supplement/input-no-groups-v3.weftforge", baseline)
    actions = {
        "loaded": "load_original_no_group_fixture",
        "empty_a_enter": "enter_with_empty_a",
        "delete_last_a": "select_two_partial_a_intervals_then_delete_last",
        "escape_a": "escape_pending_a",
        "empty_b_enter": "confirm_two_partial_a_intervals_then_enter_empty_b",
        "tool_reset": "switch_tool_while_draft_pending",
        "load_reset": "load_original_fixture_while_draft_pending",
        "confirmed_a_pending_b": "select_ordered_partial_a_then_enter",
        "created": "select_ordered_partial_b_then_enter",
        "undo_create": "one_native_undo_after_create",
        "redo_create": "one_native_redo_after_create",
        "edited": "commit_complete_group_name_endpoint_map_edit_once",
        "undo_edit": "one_native_undo_after_edit",
        "redo_edit": "one_native_redo_after_edit",
        "reversed_once": "one_native_reverse_b",
        "reversed_twice": "second_native_reverse_b",
        "saved": "native_save",
        "reopened": "native_quit_then_relaunch_and_reopen_saved_file",
    }
    checkpoints = {name: deepcopy(baseline) for name in list(actions)[:8]}
    created = revision(baseline, 74)
    group = deepcopy(original["project"]["sewing_groups"][0])
    group.update({"id": 1001, "name": "Sewing group 1"})
    created["project"].update({"sewing_groups": [group], "next_id": 1002})
    checkpoints["created"] = created
    checkpoints["undo_create"] = revision(baseline, 75)
    checkpoints["undo_create"]["project"]["next_id"] = 1002
    checkpoints["redo_create"] = revision(created, 76)
    edited = revision(created, 77)
    edit = edited["project"]["sewing_groups"][0]
    edit["name"] = "Native ordered acceptance edited"
    edit["a"][0].update({"t0": 0.15, "t1": 0.4})
    edit["correspondence"] = [[0.0, 0.0], [0.2, 0.12], [0.55, 0.72], [1.0, 1.0]]
    checkpoints["edited"] = edited
    checkpoints["undo_edit"] = revision(created, 78)
    checkpoints["redo_edit"] = revision(edited, 79)
    reversed_once = revision(edited, 80)
    reversed_once["project"]["sewing_groups"][0]["b"] = [
        {"curve": i["curve"], "t0": i["t1"], "t1": i["t0"]} for i in reversed(edit["b"])]
    checkpoints["reversed_once"] = reversed_once
    checkpoints["reversed_twice"] = revision(edited, 81)
    checkpoints["saved"] = revision(edited, 81)
    checkpoints["reopened"] = revision(edited, 81)
    write("native-supplement/expected-checkpoints.json", {"kind": "native-chain-sewing-expected-v1", "actions": actions, "checkpoints": checkpoints})
    split = revision(original, 74)
    p = split["project"]
    p["points"].append({"id": 1001, "position": [1.3, 1.0]})
    left = next(c for c in p["curves"] if c["id"] == 16)
    left["end"] = 1001
    p["curves"].append({"id": 1002, "start": 1001, "end": 13, "controls": None})
    next(piece for piece in p["pieces"] if piece["id"] == 11)["boundary"] = [16, 1002, 17, 18, 19]
    p["sewing_groups"][0]["a"] = [deepcopy(p["sewing_groups"][0]["a"][0]),
                                  {"curve": 1002, "t0": 0.75, "t1": 0.0},
                                  {"curve": 16, "t0": 1.0, "t1": 0.5}]
    p["next_id"] = 1003
    joined = revision(original, 75)
    joined["project"]["next_id"] = 1003
    joined["project"]["sewing_groups"][0]["a"] = [deepcopy(joined["project"]["sewing_groups"][0]["a"][0]),
                                                                  {"curve": 16, "t0": 0.9, "t1": 0.6},
                                                                  {"curve": 16, "t0": 0.6, "t1": 0.3}]
    split_undo = revision(split, 76)
    remeshed = revision(joined, 78)
    remeshed["project"]["resolution"] = 0.017
    operation_checkpoints = {"split_loaded": original, "split": split, "joined": joined,
                             "undo_join": split_undo, "rejected_overlap": deepcopy(split_undo),
                             "redo_join": revision(joined, 77), "remeshed": remeshed}
    write("operations-supplement/input-split-join-v3.weftforge", original)
    removal = deepcopy(original)
    survivor = deepcopy(original["project"]["sewing_groups"][0])
    survivor.update({"id": 1001, "name": "Independent surviving relationship",
                     "a": [deepcopy(survivor["a"][0])], "b": [deepcopy(survivor["b"][0])],
                     "correspondence": [[0.0, 0.0], [0.35, 0.2], [1.0, 1.0]]})
    removal["project"]["sewing_groups"].append(survivor)
    removal["project"]["next_id"] = 1002
    write("operations-supplement/input-dependent-removal-v3.weftforge", removal)
    removed = revision(removal, 74)
    p = removed["project"]
    p["pieces"] = [piece for piece in p["pieces"] if piece["id"] != 11]
    p["curves"] = [c for c in p["curves"] if c["id"] not in (16, 17, 18, 19)]
    p["points"] = [point for point in p["points"] if point["id"] not in (12, 13, 14, 15)]
    p["sewing_groups"] = [deepcopy(survivor)]
    operation_checkpoints.update({"removal_loaded": removal, "removed": removed,
                                  "undo_removal": revision(removal, 75), "redo_removal": revision(removed, 76)})
    invalid = deepcopy(split_undo["project"]["sewing_groups"][0])
    invalid["a"].append({"curve": 7, "t0": 0.2, "t1": 0.6})
    write("operations-supplement/invalid-overlap-group.json", invalid)
    write("operations-supplement/expected-checkpoints.json", {"kind": "headless-chain-sewing-expected-v1",
          "checkpoints": operation_checkpoints, "new_geometry_tolerance_m": "1e-12",
          "rewritten_t_tolerance": "1e-14", "continuous_pair_budget_m": "1e-8",
          "probe_u": ["0", "0.07", "0.125", "0.2", "0.25", "0.3333333333333333333333333333333333333333", "0.49", "0.55", "0.625", "0.7", "0.83", "0.8333333333333333333333333333333333333333", "0.9", "1"]})
    print(json.dumps({"native_checkpoints": len(checkpoints), "headless_checkpoints": len(operation_checkpoints), "original_freeze_unchanged": True}))


if __name__ == "__main__":
    main()
