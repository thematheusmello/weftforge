#!/usr/bin/env python3
"""Build a pinned public-API driver, then capture its actual operations separately."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
WORKSPACE = ROOT.parent.parent
SOURCE = ROOT / "source"
DRIVER = ROOT / "driver"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def checked_plan():
    plan = json.loads((ROOT / "prepared-plan.json").read_text())
    for name, digest in plan["frozen_files"].items():
        assert sha(ROOT / name) == digest, f"Prepared file changed: {name}"
    assert len(plan["commands"]) == 9
    assert len(plan["checkpoints"]) == 11
    assert len(set(plan["checkpoints"])) == 11
    return plan


def invoke(folder, name, argv, cwd, env=None):
    record = {
        "argv": [str(arg) for arg in argv], "cwd": str(cwd),
        "environment_overrides": env or {},
        "started_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    }
    with (folder / f"{name}.stdout").open("xb") as stdout:
        with (folder / f"{name}.stderr").open("xb") as stderr:
            process = subprocess.Popen(record["argv"], cwd=cwd, stdout=stdout,
                                       stderr=stderr, env={**os.environ, **(env or {})})
            record["process_id"] = process.pid
            record["exit_code"] = process.wait()
    record["ended_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    record["stdout"] = f"{name}.stdout"
    record["stderr"] = f"{name}.stderr"
    write(folder / f"{name}.invocation.json", record)
    print(json.dumps({"step": name, "process_id": process.pid,
                      "exit_code": record["exit_code"]}), flush=True)
    if record["exit_code"]:
        raise RuntimeError(f"{name} failed; complete transcripts preserved in {folder}")
    return (folder / f"{name}.stdout").read_text()


def build(commit):
    checked_plan()
    folder = ROOT / "build-001"
    folder.mkdir()
    invoke(folder, "worktree", ["git", "worktree", "add", "--detach", SOURCE, commit], WORKSPACE)
    actual = invoke(folder, "source-commit", ["git", "rev-parse", "HEAD"], SOURCE).strip()
    assert actual == commit
    status = invoke(folder, "source-status", ["git", "status", "--porcelain"], SOURCE)
    assert not status, "Detached source must be clean"
    files = invoke(folder, "source-files", ["git", "ls-files", "-z"], SOURCE).split("\0")
    tracked = {name: sha(SOURCE / name) for name in files if name}
    write(folder / "source-binding.json", {
        "commit": actual, "clean": True, "tracked_file_sha256": tracked,
        "prepared_plan_sha256": sha(ROOT / "prepared-plan.json"),
    })
    target = ROOT / f"target-{commit}"
    assert not target.exists(), "Target must be exclusive and new"
    env = {"CARGO_TARGET_DIR": str(target)}
    invoke(folder, "rustc-version", ["rustc", "--version", "--verbose"], SOURCE)
    invoke(folder, "cargo-version", ["cargo", "--version"], SOURCE)
    invoke(folder, "generate-lockfile", ["cargo", "generate-lockfile", "--offline",
           "--manifest-path", DRIVER / "Cargo.toml"], ROOT, env)
    invoke(folder, "build", ["cargo", "build", "--offline", "--locked", "--release",
           "--manifest-path", DRIVER / "Cargo.toml"], ROOT, env)
    binary = target / "release" / "chain-sewing-operations-producer"
    assert binary.is_file()
    assert not invoke(folder, "source-status-after-build", ["git", "status", "--porcelain"], SOURCE)
    write(ROOT / "binary-binding.json", {
        "commit": actual, "binary": str(binary), "binary_sha256": sha(binary),
        "driver_lockfile_sha256": sha(DRIVER / "Cargo.lock"),
        "source_binding": "build-001/source-binding.json",
        "source_binding_sha256": sha(folder / "source-binding.json"),
        "prepared_plan_sha256": sha(ROOT / "prepared-plan.json"),
        "target_directory": str(target), "execution_held_for_final_gates": True,
    })


def capture(gates_path, run_name):
    checked_plan()
    binding = json.loads((ROOT / "binary-binding.json").read_text())
    binary = Path(binding["binary"])
    assert sha(binary) == binding["binary_sha256"]
    gates_bytes = Path(gates_path).read_bytes()
    gates = json.loads(gates_bytes)
    assert gates["commit"] == binding["commit"], "Gates must bind the exact source pin"
    assert gates["all_required_passed"] is True, "Final local gates must be confirmed"
    folder = ROOT / run_name
    folder.mkdir()
    (folder / "local-gates.json").write_bytes(gates_bytes)
    status = invoke(folder, "source-status-before-capture", ["git", "status", "--porcelain"], SOURCE)
    assert not status, "Detached source must remain clean"
    invoke(folder, "driver", [binary, ROOT, folder / "capture"], SOURCE)
    capture_dir = folder / "capture"
    index = json.loads((capture_dir / "index.json").read_text())
    files = [path for path in capture_dir.iterdir() if path.is_file()]
    artifacts = {path.name: sha(path) for path in sorted(files)}
    manifest = {
        "kind": "headless-chain-sewing-capture-v1",
        "candidate": {"commit": binding["commit"], "binary_sha256": binding["binary_sha256"]},
        "checkpoints": index["checkpoints"], "commands": index["commands"],
        "artifacts": artifacts,
        "binding": {"prepared_plan_sha256": binding["prepared_plan_sha256"],
                    "source_binding_sha256": binding["source_binding_sha256"],
                    "local_gates_sha256": sha(folder / "local-gates.json"),
                    "binary_binding_sha256": sha(ROOT / "binary-binding.json"),
                    "driver_invocation_sha256": sha(folder / "driver.invocation.json"),
                    "observer_review": False},
    }
    write(capture_dir / "capture-manifest.json", manifest)
    print(json.dumps({"capture_manifest": str(capture_dir / "capture-manifest.json"),
                      "checkpoints": len(index["checkpoints"]),
                      "operations": len(index["commands"])}), flush=True)


if __name__ == "__main__":
    if sys.argv[1:] == ["self-check"]:
        checked_plan()
        print("Frozen producer plan self-check passed; no product commands executed.")
    elif len(sys.argv) == 3 and sys.argv[1] == "build":
        build(sys.argv[2])
    elif len(sys.argv) == 4 and sys.argv[1] == "capture":
        capture(sys.argv[2], sys.argv[3])
    else:
        raise SystemExit("Usage: producer.py self-check | build COMMIT | capture GATES.json NEW_RUN_NAME")
