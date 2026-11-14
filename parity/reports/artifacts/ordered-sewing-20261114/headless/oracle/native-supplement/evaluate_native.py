"""Frozen native-source checkpoints plus required standalone observation records."""
from copy import deepcopy
from pathlib import Path
import hashlib
import json
import re
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from evaluate import load, same


def artifact(base, relative):
    assert isinstance(relative, str), "artifact path must be a relative string"
    path = (base / relative).resolve()
    assert path.is_relative_to(base.resolve()) and path.is_file(), "missing or escaping capture artifact"
    return path


def evaluate(manifest_path):
    manifest = load(manifest_path)
    assert manifest["kind"] == "native-chain-sewing-capture-v1"
    assert re.fullmatch(r"[0-9a-f]{40}", manifest["candidate"]["commit"])
    assert re.fullmatch(r"[0-9a-f]{64}", manifest["candidate"]["binary_sha256"])
    assert manifest["standalone_native"] is True, "in-process/headless UI is insufficient"
    base = manifest_path.parent
    expected = load(HERE / "expected-checkpoints.json")
    assert set(manifest["checkpoints"]) == set(expected["checkpoints"]), "native checkpoint set incomplete"
    checked = []
    for name, source in expected["checkpoints"].items():
        record = manifest["checkpoints"][name]
        source_path = artifact(base, record["source"])
        same(source, load(source_path), name)
        assert record["observed_ui_action"] == expected["actions"][name], "checkpoint action binding changed"
        for key in ("source", "tool_transcript", "screenshot", "accessibility"):
            path = artifact(base, record[key])
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            assert manifest["artifacts"][record[key]] == digest, f"{name}: capture hash mismatch"
            assert path.stat().st_size > 0, f"{name}: empty observation artifact"
        assert record["observer_review"] is True, "native event/visible-state review remains unknown"
        checked.append(name)
    processes = manifest["processes"]
    assert len(processes) >= 2, "save/quit/reopen requires actual native process relaunch"
    assert len({process["handle"] for process in processes}) == len(processes), "process handles must identify distinct launches"
    for process in processes:
        assert process["handle"] and process["launched_utc"] and process["executable"]
        assert process["binary_sha256"] == manifest["candidate"]["binary_sha256"], "native executable changed between launches"
        assert process["commit"] == manifest["candidate"]["commit"], "native candidate changed between launches"
        artifact(base, process["launch_record"])
    assert processes[0]["observed_native_quit"] is True, "native quit was not observed"
    assert manifest["visible_pending_and_directions_reviewed"] is True
    assert manifest["visible_group_identity_lengths_mismatch_metric_guides_reviewed"] is True
    return {"kind": "native-chain-sewing-evaluation-v1", "status": "pass", "checkpoints": checked,
            "candidate": manifest["candidate"], "meaning": "concrete original native gesture/edit/history/save/quit/reopen case only",
            "still_unknown": ["broader native invalid save/load preservation", "native cubic precise editing case", "reference parity", "physical gathering quality", "complete roadmap gate"]}


def self_check():
    expected = load(HERE / "expected-checkpoints.json")
    assert expected["checkpoints"]["loaded"]["project"]["revision"] == 73
    created = expected["checkpoints"]["created"]["project"]
    assert created["revision"] == 74 and created["next_id"] == 1002
    assert created["sewing_groups"][0]["id"] == 1001
    assert expected["checkpoints"]["undo_edit"]["project"]["sewing_groups"] == created["sewing_groups"]
    edited = expected["checkpoints"]["edited"]["project"]["sewing_groups"]
    assert expected["checkpoints"]["redo_edit"]["project"]["sewing_groups"] == edited
    assert expected["checkpoints"]["reversed_twice"]["project"]["sewing_groups"] == edited
    assert expected["checkpoints"]["reopened"] == expected["checkpoints"]["saved"]
    rejected = []
    for label, mutate in (
        ("wrong-created-allocator", lambda d: d["project"].update({"next_id": 1001})),
        ("wrong-created-revision", lambda d: d["project"].update({"revision": 73})),
        ("lost-disconnected-span", lambda d: d["project"]["sewing_groups"][0]["a"].pop()),
        ("wrong-map", lambda d: d["project"]["sewing_groups"][0].update({"correspondence": [[0.0, 0.0], [1.0, 1.0], [1.0, 1.0]]})),
    ):
        bad = deepcopy(expected["checkpoints"]["created"])
        mutate(bad)
        try:
            same(expected["checkpoints"]["created"], bad)
        except AssertionError:
            rejected.append(label)
        else:
            raise AssertionError(label)
    return {"kind": "native-supplement-self-check", "candidate_outputs_inspected": False,
            "checkpoint_count": len(expected["checkpoints"]), "rejected_mutations": rejected,
            "claim": "checkpoint/evaluator mechanics only; no native action performed"}


if __name__ == "__main__":
    if sys.argv[1] == "self-check":
        result, output = self_check(), Path(sys.argv[2])
    else:
        manifest_path, output = Path(sys.argv[1]), Path(sys.argv[2])
        try:
            result = evaluate(manifest_path)
        except (AssertionError, ValueError, KeyError, TypeError, IndexError) as error:
            result = {"kind": "native-chain-sewing-evaluation-v1", "status": "fail", "reason": str(error)}
    with output.open("x") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(json.dumps({"status": result.get("status", "self-check"), "output": output.name}))
    if result.get("status") == "fail":
        raise SystemExit(1)
