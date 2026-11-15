"""Local SVG studio integration source checks; no GitHub workflows or UI input."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import time

BASE = Path(__file__).resolve().parent
ROOT = BASE.parents[1] / ".local/svg-studio-integration-20261115"
PRE = json.loads((BASE / "source-pretest.json").read_bytes())
STEPS = [
    ("fmt", ["cargo", "fmt", "--all", "--check"]),
    ("package", ["cargo", "xtask", "package"]),
    ("tests", ["cargo", "test", "--workspace"]),
    ("clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]),
    ("xtask", ["cargo", "xtask", "verify"]),
]


def source_unchanged():
    assert (ROOT / "target").is_dir() and not (ROOT / "target").is_symlink()
    assert not any(path.is_symlink() for path in (ROOT / "target").rglob("*")), "Target must contain no shared symlinks"
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT).decode().strip() == PRE["source_commit"]
    assert not subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT)
    for name, expected in PRE["source_hashes"].items():
        assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == expected, name


def save(name, data):
    (BASE / name).write_text(json.dumps(data, indent=2) + "\n")


assert not (BASE / "jobs.json").exists()
jobs = []
for name, argv in STEPS:
    source_unchanged()
    job = {"name": name, "source_commit": PRE["source_commit"], "wrapper_pid": os.getpid(),
           "started_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
           "command": argv, "status": "running"}
    started = time.monotonic()
    with (BASE / (name + ".log")).open("xb") as log:
        child = subprocess.Popen(argv, cwd=ROOT, env=dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target"), CARGO_INCREMENTAL="0"), stdout=log, stderr=subprocess.STDOUT)
        job["child_pid"] = child.pid
        save("current-job.json", job)
        print(json.dumps(job), flush=True)
        code = child.wait()
    source_unchanged()
    job.update(exit_code=code, status="completed" if code == 0 else "failed",
               wall_seconds=time.monotonic() - started,
               completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    jobs.append(job)
    save("jobs.json", jobs)
    save("current-job.json", job)
    print(json.dumps(job), flush=True)
    if code:
        raise SystemExit(code)
save("current-job.json", {"status": "completed", "all_exit_zero": True,
                          "source_commit": PRE["source_commit"]})
