"""Bounded serial jobs with durable, confirmed process handles and output logs."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "source"
stage = sys.argv[1]
assert stage in ["build", "run"]
plan_bytes = (ROOT / "plan.json").read_bytes()
assert hashlib.sha256(plan_bytes).hexdigest() == json.loads((ROOT / "plan.sha256").read_text())["plan.json"]
for relative, expected in json.loads(plan_bytes)["frozen_code_sha256"].items():
    assert hashlib.sha256((ROOT / relative).read_bytes()).hexdigest() == expected, relative
env = dict(os.environ, CARGO_BUILD_JOBS="1", CARGO_INCREMENTAL="0", CARGO_TARGET_DIR=str(ROOT / "target"))
command = ["cargo", "build", "--locked", "--release", "--offline", "--manifest-path", str(ROOT / "harness/Cargo.toml")] if stage == "build" else [str(ROOT / "target/release/physics-full-path-diagnostic"), str(SOURCE / "fixtures/grain-shirt.weftforge"), str(ROOT / "runs")]
ledger_path = ROOT / "job-ledger.json"
ledger = json.loads(ledger_path.read_text()) if ledger_path.exists() else {"jobs": []}
assert not any(job["status"] == "running" for job in ledger["jobs"])
assert not any(job["stage"] == stage for job in ledger["jobs"]), "Never overwrite/restart an existing job"

def save():
    temporary = ROOT / "job-ledger.tmp"
    temporary.write_text(json.dumps(ledger, indent=2) + "\n")
    temporary.replace(ledger_path)

with (ROOT / f"{stage}.log").open("x") as log:
    started = time.monotonic()
    proc = subprocess.Popen(command, cwd=SOURCE, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    job = {"stage": stage, "status": "running", "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "launcher_pid": os.getpid(), "child_pid": proc.pid, "child_process_group": proc.pid, "command": command, "cwd": str(SOURCE), "environment_overrides": {key: env[key] for key in ["CARGO_BUILD_JOBS", "CARGO_INCREMENTAL", "CARGO_TARGET_DIR"]}, "log": f"{stage}.log", "wall_cap_seconds": 180 if stage == "build" else 600}
    ledger["jobs"].append(job)
    save()
    print(json.dumps(job), flush=True)
    timed_out = False
    try:
        code = proc.wait(timeout=job["wall_cap_seconds"])
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(proc.pid, signal.SIGTERM)
        try:
            code = proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            code = proc.wait()
    job.update(status="timed_out" if timed_out else "completed", exit_code=code, completed_at=datetime.datetime.now(datetime.timezone.utc).isoformat(), wall_seconds=time.monotonic() - started)
    save()
    print(json.dumps(job), flush=True)
    sys.exit(code if not timed_out else 124)
