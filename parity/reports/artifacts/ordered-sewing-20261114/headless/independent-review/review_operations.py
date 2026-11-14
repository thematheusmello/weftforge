"""Independent provenance review and declared key-only operations adapter."""
from copy import deepcopy
from pathlib import Path
import hashlib
import json
import shutil
import sys

from evaluate_actual_outputs import COMMIT, FINAL, ORACLE, OUT, PRODUCER, ROOT, frozen_snapshot, read, run, sha, utc, write


def relative(path):
    return str(path.relative_to(ROOT))


def main():
    before = frozen_snapshot()
    for supplement in ("operations-supplement", "native-supplement"):
        for name, expected in read(ORACLE / supplement / "freeze.json")["shared_inputs"].items():
            assert sha(ORACLE / name) == expected, f"frozen shared input changed: {name}"
            before[name] = expected
    original = PRODUCER / "run-001/capture"
    manifest = read(original / "capture-manifest.json")
    original_sha = sha(original / "capture-manifest.json")
    assert original_sha == "7ac6e04afe1dae0815bc0f1dd25d03a3849dacbbe4bac9ec29172e5bb66bb082"
    plan = read(PRODUCER / "prepared-plan.json")
    binding = read(PRODUCER / "binary-binding.json")
    source_binding = read(PRODUCER / "build-001/source-binding.json")
    gates = read(PRODUCER / "run-001/local-gates.json")
    driver_invocation = read(PRODUCER / "run-001/driver.invocation.json")
    assert manifest["candidate"] == {"commit": COMMIT, "binary_sha256": binding["binary_sha256"]}
    assert binding["commit"] == source_binding["commit"] == gates["commit"] == COMMIT
    assert source_binding["clean"] is True
    assert sha(Path(binding["binary"])) == binding["binary_sha256"] == "ab64abad99f02ee2408b3e4b63221cb3b78c200651bc25d78617a1f88cf3c6ef"
    assert sha(PRODUCER / "driver/Cargo.lock") == binding["driver_lockfile_sha256"]
    assert sha(PRODUCER / "prepared-plan.json") == binding["prepared_plan_sha256"] == source_binding["prepared_plan_sha256"]
    assert sha(PRODUCER / "build-001/source-binding.json") == binding["source_binding_sha256"]
    for name, expected in plan["frozen_files"].items():
        assert sha(PRODUCER / name) == expected, f"producer preregistration changed: {name}"
    for name, expected in plan["contract_sha256"].items():
        assert sha(ORACLE / name) == expected
    for name, expected in plan["input_original_sha256"].items():
        assert sha(PRODUCER / "inputs" / name) == expected == sha(ORACLE / "operations-supplement" / name)
    assert sha(PRODUCER / "run-001/local-gates.json") == sha(FINAL / "gates/all-local-gates.json")
    assert gates["all_required_passed"] is True
    for name, expected in gates["artifacts"].items():
        assert sha(ROOT / name) == expected
    for key, file in (("prepared_plan_sha256", "prepared-plan.json"), ("source_binding_sha256", "build-001/source-binding.json"),
                      ("local_gates_sha256", "run-001/local-gates.json"), ("binary_binding_sha256", "binary-binding.json"),
                      ("driver_invocation_sha256", "run-001/driver.invocation.json")):
        assert manifest["binding"][key] == sha(PRODUCER / file)
    assert manifest["binding"]["observer_review"] is False
    source = PRODUCER / "source"
    provenance = OUT / "operations/provenance"
    git_records = []
    for name, argv in (("source-head", ["git", "-C", relative(source), "rev-parse", "HEAD"]),
                       ("source-status", ["git", "-C", relative(source), "status", "--porcelain"]),
                       ("source-tree", ["git", "-C", relative(source), "ls-tree", "-rz", "--full-tree", COMMIT])):
        git_records.append(run(name, argv, provenance / name))
    assert (provenance / "source-head/stdout.txt").read_text().strip() == COMMIT
    assert (provenance / "source-status/stdout.txt").read_bytes() == b""
    assert all(r["exit_code"] == 0 for r in git_records)
    tree = {}
    for line in (provenance / "source-tree/stdout.txt").read_bytes().split(b"\0"):
        if not line:
            continue
        metadata, filename = line.split(b"\t", 1)
        mode, kind, blob = metadata.split()
        assert kind == b"blob" and mode in (b"100644", b"100755")
        tree[filename.decode()] = blob.decode()
    assert set(tree) == set(source_binding["tracked_file_sha256"])
    for name, expected in source_binding["tracked_file_sha256"].items():
        raw = (source / name).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == expected, f"source binding differs: {name}"
        actual_git_blob = hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()
        assert actual_git_blob == tree[name], f"source differs from pinned commit: {name}"
    build = read(PRODUCER / "build-001/build.invocation.json")
    assert build["exit_code"] == 0
    assert build["argv"][:5] == ["cargo", "build", "--offline", "--locked", "--release"]
    assert Path(build["argv"][-1]) == PRODUCER / "driver/Cargo.toml"
    assert Path(build["environment_overrides"]["CARGO_TARGET_DIR"]) == Path(binding["target_directory"])
    assert Path(binding["target_directory"]).name == f"target-{COMMIT}"
    assert driver_invocation["exit_code"] == 0 and driver_invocation["process_id"] == 7742
    assert list(map(Path, driver_invocation["argv"])) == [Path(binding["binary"]), PRODUCER, original]
    assert Path(driver_invocation["cwd"]) == source
    assert (PRODUCER / "run-001/driver.stderr").read_bytes() == b""
    assert (PRODUCER / "build-001/source-status.stdout").read_bytes() == b""
    assert (PRODUCER / "build-001/source-status-after-build.stdout").read_bytes() == b""
    assert (PRODUCER / "run-001/source-status-before-capture.stdout").read_bytes() == b""
    completion = read(PRODUCER / "run-001/tool-completion.json")
    assert completion["session_id"] == 3209 and completion["exit_code"] == 0 and completion["driver_process_id"] == 7742
    main_rs = (PRODUCER / "driver/src/main.rs").read_text()
    lines = main_rs.splitlines()
    assert "weftforge_core::{Command, Editor, Project}" in main_rs
    assert "let command: Command = serde_json::from_value" in main_rs
    assert "let mesh = editor.project.mesh()?;" in main_rs and "mesh.validate()?;" in main_rs
    assert "serde_json::to_vec(&editor.project)" in main_rs
    assert all(word not in main_rs for word in ("evaluate", "oracle", "expectations", "expected-checkpoints", "include_str!"))
    assert len(manifest["artifacts"]) == 45
    assert len(manifest["checkpoints"]) == 11 and len(manifest["commands"]) == 9
    assert [c["name"] for c in manifest["commands"]] == plan["commands"]
    for name, expected in manifest["artifacts"].items():
        assert sha(original / name) == expected
    events = [json.loads(line) for line in (PRODUCER / "run-001/driver.stdout").read_text().splitlines()]
    operation_events = [e for e in events if e["event"] == "operation"]
    checkpoint_events = [e for e in events if e["event"] == "checkpoint"]
    assert [e["operation"] for e in operation_events] == plan["commands"]
    assert [e["checkpoint"] for e in checkpoint_events] == plan["checkpoints"]
    review = []
    checkpoint_for = {"split": "split", "join": "joined", "undo_join": "undo_join", "reject_overlap": "rejected_overlap",
                      "redo_join": "redo_join", "remesh": "remeshed", "remove_piece": "removed", "undo_removal": "undo_removal", "redo_removal": "redo_removal"}
    preceding = {"split": "split_loaded", "join": "split", "undo_join": "joined", "reject_overlap": "undo_join",
                 "redo_join": "rejected_overlap", "remesh": "redo_join", "remove_piece": "removal_loaded", "undo_removal": "removed", "redo_removal": "undo_removal"}
    for command, event in zip(manifest["commands"], operation_events):
        name = command["name"]
        request = read(original / command["request"])
        response = read(original / command["response"])
        invocation = read(original / command["invocation"])
        assert (original / command["request"]).read_bytes() == (PRODUCER / "requests" / f"{name}.json").read_bytes()
        assert command["observer_review"] is False and command["shared_editor_command"] is True
        assert invocation["kind"] == "public_api_invocation" and invocation["process_id"] == 7742
        assert invocation["method"] == request["method"] == event["method"]
        assert invocation["request"] == command["request"]
        method = request["method"]
        callsite = {"Editor::execute": 103, "Editor::undo": 110, "Editor::redo": 119}[method]
        assert invocation["callsite"] == f"src/main.rs:{callsite}"
        assert "let callsite = concat!(file!(), \":\", line!());" in lines[callsite - 1]
        assert {"Editor::execute": "match editor.execute(command)", "Editor::undo": "editor.undo()", "Editor::redo": "editor.redo()"}[method] in lines[callsite]
        assert response["before"] == invocation["before"] == event["before"]
        assert response["after"] == invocation["after"] == event["after"]
        assert response["accepted"] == invocation["returned"]["accepted"] == event["accepted"]
        assert response["error"] == invocation["returned"]["error"] == event["error"] == command["error"]
        assert response["rejected"] == command["rejected"] == (name == "reject_overlap")
        assert response["accepted"] is (name != "reject_overlap")
        before_receipt = read(original / manifest["checkpoints"][preceding[name]])
        after_receipt = read(original / manifest["checkpoints"][checkpoint_for[name]])
        assert before_receipt["source"]["project"]["revision"] == response["before"]["revision"]
        assert after_receipt["source"]["project"]["revision"] == response["after"]["revision"]
        if name == "reject_overlap":
            assert request["command"]["data"]["group"] == read(ORACLE / "operations-supplement/invalid-overlap-group.json")
            assert "overlap" in response["error"].lower()
            assert response["before"] == response["after"] == {"can_redo": True, "can_undo": True, "revision": 76}
            assert (original / manifest["checkpoints"]["undo_join"]).read_bytes() == (original / manifest["checkpoints"]["rejected_overlap"]).read_bytes()
        review.append({"operation": name, "method": method, "process_id": 7742, "callsite": invocation["callsite"],
                       "request_sha256": sha(original / command["request"]), "response_sha256": sha(original / command["response"]),
                       "invocation_sha256": sha(original / command["invocation"]), "before": response["before"], "after": response["after"],
                       "accepted": response["accepted"], "error": response["error"], "independently_reviewed": True})
    destination = OUT / "operations/reviewed-capture"
    destination.mkdir(exist_ok=False)
    for name in manifest["artifacts"]:
        shutil.copyfile(original / name, destination / name)
        assert sha(destination / name) == manifest["artifacts"][name]
    shutil.copyfile(original / "capture-manifest.json", destination / "capture-manifest.original.json")
    receipt = {"kind": "independent-shared-editor-invocation-review-v1", "reviewed_utc": utc(),
        "original_manifest_sha256": original_sha, "candidate": manifest["candidate"], "operations": review,
        "scope": "Independent static driver/callsite and actual request/response/invocation/receipt/log/hash review. Same live Editor across each declared context. No candidate results computed from oracle.",
        "source": {"commit": COMMIT, "tracked_files_checked_against_commit_blobs_and_sha256_binding": len(tree), "clean": True,
                   "source_binding_sha256": sha(PRODUCER / "build-001/source-binding.json")},
        "driver": {"source_sha256": sha(PRODUCER / "driver/src/main.rs"), "binary_sha256": binding["binary_sha256"],
                   "build_process_id": build["process_id"], "capture_process_id": 7742, "capture_tool_session": 3209, "capture_exit_code": 0,
                   "exclusive_target_name": Path(binding["target_directory"]).name},
        "artifacts_copied_unchanged": manifest["artifacts"], "frozen_artifacts": before,
        "metadata_adapter": "Reviewed manifest adds operation equal to each exact original name; changes observer_review false to true only for independently reviewed commands and binding; adds this hashed review receipt. No source receipt/request/response/invocation/index bytes or evaluator/math changed.",
        "limitations": ["native UI not reviewed", "broader deep-history and native file-failure preservation unknown", "cancellation/staleness/exhaustion unknown", "reference parity unknown", "physical quality unknown", "complete roadmap gate unknown"]}
    write(destination / "review-receipt.json", receipt)
    adapted = deepcopy(manifest)
    for command in adapted["commands"]:
        command["operation"] = command["name"]
        command["observer_review"] = True
    adapted["binding"]["observer_review"] = True
    adapted["independent_review"] = {"receipt": "review-receipt.json", "sha256": sha(destination / "review-receipt.json"),
                                     "original_manifest": "capture-manifest.original.json", "original_manifest_sha256": original_sha,
                                     "metadata_key_correction": "operation = exact original name"}
    write(destination / "capture-manifest.reviewed.json", adapted)
    result_folder = OUT / "operations/reviewed-manifest-evaluation"
    result_path = result_folder / "evaluation.json"
    evaluation = run("operations-reviewed-manifest", [sys.executable, str(ORACLE / "operations-supplement/evaluate_operations.py"),
                     str(destination / "capture-manifest.reviewed.json"), str(result_path)], result_folder)
    result = read(result_path)
    assert sha(original / "capture-manifest.json") == original_sha
    assert all(sha(original / name) == expected == sha(destination / name) for name, expected in manifest["artifacts"].items())
    after = frozen_snapshot()
    for supplement in ("operations-supplement", "native-supplement"):
        for name, expected in read(ORACLE / supplement / "freeze.json")["shared_inputs"].items():
            assert sha(ORACLE / name) == expected
            after[name] = expected
    assert before == after
    write(OUT / "operations/review-summary.json", {"original_manifest_sha256": original_sha,
        "original_manifest_evaluation": read(OUT / "operations/original-manifest-evaluation/evaluation.json"),
        "reviewed_manifest_sha256": sha(destination / "capture-manifest.reviewed.json"), "review_receipt_sha256": sha(destination / "review-receipt.json"),
        "reviewed_evaluation_sha256": sha(result_path), "reviewed_evaluation_status": result["status"], "reviewed_evaluation_exit_code": evaluation["exit_code"],
        "source_files_verified": len(tree), "exact_capture_artifacts": len(manifest["artifacts"]), "frozen_unchanged": True,
        "frozen_artifact_count_including_shared_inputs": len(before), "original_flags_unchanged": True, "all_operations_independently_reviewed": True,
        "qualification": "Metadata adapter only. Original missing-key failure retained; candidate outputs and unchanged frozen assertions determine actual geometry result."})
    print(json.dumps({"event": "operations_review_complete", "status": result["status"], "exit_code": evaluation["exit_code"],
                      "exact_capture_artifacts": len(manifest["artifacts"]), "source_files_verified": len(tree), "frozen_unchanged": True}), flush=True)


if __name__ == "__main__":
    main()
