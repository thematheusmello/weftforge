"""Bounded local jobs; physical cases require the reviewed registration and authorization."""
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
CAP_BYTES = 256 * 1024 * 1024
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()


def write(path, value):
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(value, indent=2) + "\n")
    temporary.replace(path)


def numerical_bytes():
    paths = [p for directory in ("prepared", "runs") for p in (ROOT / directory).glob("**/*") if p.is_file()]
    paths += [p for p in ROOT.glob("native-*-comparison.json") if p.is_file()]
    if (ROOT / "edit-response.json").is_file():
        paths.append(ROOT / "edit-response.json")
    return sum(p.stat().st_size for p in paths)


def main():
    stage = sys.argv[1]
    assert stage in ("build", "base", "height")
    physical = stage != "build"
    if physical:
        registration = json.loads((ROOT / "preregistration.json").read_text())
        for path, expected in registration["frozen_files"].items():
            assert sha(ROOT / path) == expected, path
        for path, expected in json.loads((ROOT / "source-manifest.json").read_text())["files"].items():
            assert sha(ROOT / "source" / path) == expected, path
        assert subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT / "source") == b""
        authorization = json.loads((ROOT / "authorization.json").read_text())
        assert authorization["preregistration_sha256"] == sha(ROOT / "preregistration.json")
        assert authorization["explicit_parent_authorization"] is True
        assert authorization["cases"] == ["base", "height"]
        commands = [
            [str(ROOT / "target/release/harbor-public-driver"), "run", str(ROOT / f"inputs/{stage}.weftforge"), str(ROOT / f"runs/{stage}")],
            ["python3", "analyze.py", stage],
        ]
        (ROOT / "runs").mkdir(exist_ok=True)
    else:
        commands = [
            ["cargo", "generate-lockfile", "--offline", "--manifest-path", "harness/Cargo.toml"],
            ["cargo", "fmt", "--manifest-path", "harness/Cargo.toml"],
            ["cargo", "fmt", "--manifest-path", "harness/Cargo.toml", "--", "--check"],
            ["cargo", "build", "--offline", "--locked", "--release", "--manifest-path", "harness/Cargo.toml"],
            ["cargo", "clippy", "--offline", "--locked", "--manifest-path", "harness/Cargo.toml", "--all-targets", "--", "-D", "warnings"],
        ]
    ledger = ROOT / "checks" / f"{stage}-job.json"
    assert not ledger.exists(), "Retain the original attempt; never overwrite/retry a case"
    cap = 420 if physical else 240
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target"), CARGO_BUILD_JOBS="1", CARGO_INCREMENTAL="0", PYTHONDONTWRITEBYTECODE="1")
    job = {"stage": stage, "wrapper_pid": os.getpid(), "status": "ready", "started_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "wall_cap_seconds": cap, "numerical_output_cap_bytes_overall": CAP_BYTES, "children": []}
    started = time.monotonic()
    write(ledger, job)
    print(json.dumps(job), flush=True)
    try:
        for index, argv in enumerate(commands):
            with (ROOT / "checks" / f"{stage}-{index}.stdout").open("xb") as out, (ROOT / "checks" / f"{stage}-{index}.stderr").open("xb") as err:
                child = subprocess.Popen(argv, cwd=ROOT, env=env, stdout=out, stderr=err, start_new_session=True)
                handle = {"pid": child.pid, "process_group": child.pid, "argv": argv, "status": "running"}
                job["children"].append(handle)
                job["status"] = "running"
                write(ledger, job)
                print(json.dumps(handle), flush=True)
                while child.poll() is None:
                    if time.monotonic() - started >= cap or (physical and numerical_bytes() >= CAP_BYTES):
                        os.killpg(child.pid, signal.SIGKILL)
                        child.wait()
                        handle.update(status="terminated_at_cap", exit_code=child.returncode)
                        raise RuntimeError("Fixed wall/output cap reached; no continuation or extension")
                    time.sleep(.05)
                handle.update(status="completed", exit_code=child.returncode)
                write(ledger, job)
                if child.returncode:
                    raise RuntimeError(f"First failing command: exit {child.returncode}")
                if time.monotonic() - started >= cap or (physical and numerical_bytes() >= CAP_BYTES):
                    raise RuntimeError("Fixed wall/output cap exceeded at completion")
        job["status"] = "completed"
    except Exception as exc:
        job.update(status="failed", first_failure=f"{type(exc).__name__}: {exc}")
    finally:
        job.update(completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), wall_seconds=time.monotonic()-started, numerical_bytes_overall=numerical_bytes())
        if physical:
            job["retained_outputs"] = {str(p.relative_to(ROOT)): {"bytes": p.stat().st_size, "sha256": sha(p)} for p in (ROOT / "runs" / stage).glob("**/*") if p.is_file()}
        write(ledger, job)
    print(json.dumps(job), flush=True)
    return 0 if job["status"] == "completed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
