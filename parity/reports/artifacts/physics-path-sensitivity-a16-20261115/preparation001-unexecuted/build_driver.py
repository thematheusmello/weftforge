"""Build the isolated driver only; never invoke the resulting executable."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    target = ROOT / "harness/target"
    assert not target.exists(), "Fresh default target required; retain any failed build"
    assert shutil.disk_usage(ROOT).free >= 512 * 1024**2, "Insufficient build reserve"
    binding = json.loads((ROOT / "SOURCE-BINDING.json").read_text())
    source = Path(binding["root"])
    assert all(digest(source / n) == h for n, h in binding["files"].items())
    inputs = {n: digest(ROOT / n) for n in (
        "harness/Cargo.toml", "harness/Cargo.lock", "harness/src/main.rs")}
    environment = dict(os.environ)
    removed = {}
    for name in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR", "RUSTC_WRAPPER",
                 "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC_WRAPPER"):
        removed[name] = environment.pop(name, None)
    argv = [shutil.which("cargo"), "build", "--release", "--locked", "--offline",
            "-j", "1", "--message-format=json"]
    job = {"kind": "isolated_build_only", "wrapper_pid": os.getpid(), "argv": argv,
           "cwd": str(ROOT / "harness"), "default_target": str(target),
           "target_initially_absent": True, "removed_environment": removed,
           "compiled_cache_copied": False, "input_hashes": inputs,
           "status": "running", "candidate_executions": 0}
    job_path = ROOT / "build/job.json"
    with job_path.open("x") as output:
        json.dump(job, output, indent=2)
    started = time.monotonic()
    with (ROOT / "build/cargo.jsonl").open("xb") as stdout, (ROOT / "build/cargo.stderr").open("xb") as stderr:
        child = subprocess.Popen(argv, cwd=ROOT / "harness", env=environment,
                                 stdout=stdout, stderr=stderr)
        job["cargo_pid"] = child.pid
        job_path.write_text(json.dumps(job, indent=2) + "\n")
        print(json.dumps({"wrapper_pid": os.getpid(), "cargo_pid": child.pid,
                          "status": "build_running", "candidate_executions": 0}), flush=True)
        job["exit_code"] = child.wait()
    job["elapsed_seconds"] = time.monotonic() - started
    job["status"] = "completed" if job["exit_code"] == 0 else "failed"
    job["source_hashes_still_match"] = all(digest(source / n) == h for n, h in binding["files"].items())
    job["build_inputs_still_match"] = all(digest(ROOT / n) == h for n, h in inputs.items())
    artifact = target / "release/harbor-public-driver"
    if job["exit_code"] == 0:
        events = [json.loads(line) for line in (ROOT / "build/cargo.jsonl").read_text().splitlines()]
        matches = [e for e in events if e.get("reason") == "compiler-artifact" and e.get("executable") == str(artifact)]
        job["actual_artifact_messages"] = matches
        job["driver_sha256"] = digest(artifact)
        job["driver_path"] = str(artifact)
        job["target_bytes"] = sum(p.stat().st_size for p in target.rglob("*") if p.is_file())
    job["log_hashes"] = {n: digest(ROOT / "build" / n) for n in ("cargo.jsonl", "cargo.stderr")}
    job_path.write_text(json.dumps(job, indent=2) + "\n")
    print(json.dumps({k: job.get(k) for k in ("status", "exit_code", "cargo_pid", "elapsed_seconds", "driver_sha256", "target_bytes", "source_hashes_still_match", "build_inputs_still_match")}), flush=True)
    assert job["exit_code"] == 0 and job["source_hashes_still_match"] and job["build_inputs_still_match"]
    assert len(matches) == 1 and matches[0]["fresh"] is False


if __name__ == "__main__":
    main()
