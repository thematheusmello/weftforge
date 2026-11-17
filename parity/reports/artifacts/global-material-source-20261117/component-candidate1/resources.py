"""Retained Harbor guards, with bounded same-child RSS observation recovery."""
import hashlib
import json
import os
from pathlib import Path
import signal
import stat
import subprocess
import sys
import time
if sys.flags.optimize != 0 or os.environ.get('PYTHONOPTIMIZE') not in (None,'0'):
    raise RuntimeError('Python optimization guard: require optimization0 and absent/literal0 PYTHONOPTIMIZE')
ROOT = Path(__file__).absolute().parent
CAP_BYTES = 268435456
RECEIPT_RESERVE = 65536
OUTPUT_DIRECTORIES = ("inputs", "expected", "preflight-base", "runs", "runtime")

def require(condition, message):
    if not condition:
        raise ValueError(message)

def deadline_check(deadline):
    if deadline is not None:
        require(time.monotonic() < deadline, "Whole-case deadline closed; no numerical continuation")

def no_links(path, base):
    path, base = Path(path).absolute(), Path(base).absolute()
    require(".." not in path.parts and path.is_relative_to(base), "Path escaped bound directory")
    current = path
    while True:
        require(not current.is_symlink(), f"Symlink rejected: {current.name}")
        if current == base:
            break
        current = current.parent
    return path

def safe_file(path, base):
    path = Path(path)
    if not path.is_absolute():
        path = Path(base) / path
    path = no_links(path, base)
    require(path.is_file() and stat.S_ISREG(path.stat().st_mode), f"Missing regular file: {path.name}")
    return path

def files_below(directory):
    directory = Path(directory)
    require(not directory.is_symlink(), f"Symlink directory: {directory.name}")
    if not directory.exists():
        return []
    require(directory.is_dir(), f"Not a directory: {directory.name}")
    result = []
    for path in sorted(directory.iterdir()):
        mode = path.lstat().st_mode
        require(not stat.S_ISLNK(mode), f"Symlink output/source: {path.name}")
        if stat.S_ISDIR(mode):
            result.extend(files_below(path))
        else:
            require(stat.S_ISREG(mode), f"Nonregular output/source: {path.name}")
            result.append(path)
    return result

def sha(path, deadline=None):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        while True:
            deadline_check(deadline)
            block = stream.read(1024 * 1024)
            if not block:
                break
            digest.update(block)
    deadline_check(deadline)
    return digest.hexdigest()

def numerical_files(root=ROOT):
    return [path for name in OUTPUT_DIRECTORIES for path in files_below(Path(root) / name)]

def numerical_bytes(root=ROOT):
    return sum(path.stat().st_size for path in numerical_files(root))

def encoded(value):
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()

def write_new(path, data):
    path = Path(path)
    no_links(path, path.anchor)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(descriptor, "wb") as stream:
        stream.write(data)
        stream.flush()

def kill_group(pid):
    try:
        os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass

def monitor(argv, cwd, stdout, stderr, deadline, count_bytes, *, rss_cap=None,
            output_cap=CAP_BYTES, reserve=RECEIPT_RESERVE, pending_copy_bytes=lambda: 0, on_start=None, child_env=None):
    """Own wait4 for this child; do not mix Popen poll/wait with exact rusage."""
    child, usage, wait_status = None, None, None
    result = {"argv": [str(value) for value in argv], "pid": None, "process_group": None,
              "exit_code": None, "peak_rss_bytes": None, "sampled_peak_rss_bytes": 0, "stop_reason": None,
              "rss_observation": {"snapshot_timeout_max_seconds": 1., "consecutive_failure_limit": 3,
                                  "scope": "Best-effort process-group snapshots with gaps; only direct-child wait4 RSS is exact.",
                                  "successful_snapshots": 0, "failures": 0, "timeouts": 0,
                                  "max_consecutive_failures": 0, "max_snapshot_gap_seconds": 0.,
                                  "failure_events": [], "omitted_failure_events": 0}}
    observation = result["rss_observation"]
    consecutive_failures, last_snapshot = 0, None
    try:
        deadline_check(deadline)
        require(count_bytes() + pending_copy_bytes() + reserve < output_cap, "Numerical output stop budget reached before launch")
        no_links(stdout, Path(stdout).anchor)
        no_links(stderr, Path(stderr).anchor)
        with Path(stdout).open("xb") as out, Path(stderr).open("xb") as err:
            child = subprocess.Popen(argv, cwd=cwd, env=dict(child_env or os.environ, PYTHONDONTWRITEBYTECODE="1", PYTHONOPTIMIZE="0"),
                                     stdout=out, stderr=err, start_new_session=True)
            result.update(pid=child.pid, process_group=child.pid)
            last_snapshot = time.monotonic()
            if on_start is not None:
                on_start(dict(result))
            while True:
                pid, status, observed = os.wait4(child.pid, os.WNOHANG)
                if pid:
                    wait_status, usage = status, observed
                    break
                if time.monotonic() >= deadline:
                    result["stop_reason"] = "whole_case_wall_cap"
                elif count_bytes() + pending_copy_bytes() + reserve >= output_cap:
                    result["stop_reason"] = "numerical_output_cap_stop_budget"
                if result["stop_reason"]:
                    break
                if rss_cap is not None:
                    remaining = min(1., deadline - time.monotonic())
                    if remaining <= 0:
                        result["stop_reason"] = "whole_case_wall_cap"
                        break
                    try:
                        sample = subprocess.run(["/bin/ps", "-axo", "pid=,pgid=,rss="],
                                                capture_output=True, text=True, timeout=remaining, check=False)
                        group = [int(parts[2])*1024 for line in sample.stdout.splitlines()
                                 if len(parts := line.split()) == 3 and int(parts[1]) == child.pid]
                        require(sample.returncode == 0 and bool(group), "RSS snapshot unavailable")
                        sampled = max(group)
                        require(sampled >= 0, "Invalid sampled RSS")
                    except (subprocess.TimeoutExpired, OSError, ValueError) as error:
                        consecutive_failures += 1
                        observation["failures"] += 1
                        observation["timeouts"] += int(isinstance(error, subprocess.TimeoutExpired))
                        observation["max_consecutive_failures"] = max(consecutive_failures, observation["max_consecutive_failures"])
                        event = {"pid": child.pid, "kind": type(error).__name__, "at_monotonic_seconds": time.monotonic(),
                                 "timeout_seconds": remaining, "consecutive_failures": consecutive_failures}
                        if len(observation["failure_events"]) < 16:
                            observation["failure_events"].append(event)
                        else:
                            observation["omitted_failure_events"] += 1
                        # An observation timeout says nothing about the child's termination.
                        pid, status, observed = os.wait4(child.pid, os.WNOHANG)
                        if pid:
                            wait_status, usage = status, observed
                            break
                        if time.monotonic() >= deadline:
                            result["stop_reason"] = "whole_case_wall_cap"
                        elif count_bytes() + pending_copy_bytes() + reserve >= output_cap:
                            result["stop_reason"] = "numerical_output_cap_stop_budget"
                        elif consecutive_failures >= 3:
                            result["stop_reason"] = "rss_observation_unavailable"
                        if result["stop_reason"]:
                            break
                        time.sleep(min(.02, max(0., deadline - time.monotonic())))
                        continue
                    observed_at = time.monotonic()
                    observation["max_snapshot_gap_seconds"] = max(observed_at - last_snapshot, observation["max_snapshot_gap_seconds"])
                    last_snapshot = observed_at
                    consecutive_failures = 0
                    observation["successful_snapshots"] += 1
                    result["sampled_peak_rss_bytes"] = max(sampled, result["sampled_peak_rss_bytes"])
                    if sampled > rss_cap:
                        result["stop_reason"] = "sampled_producer_rss_cap"
                        break
                time.sleep(min(.02, max(0., deadline - time.monotonic())))
    except BaseException as error:
        result["stop_reason"] = f"monitor_error:{type(error).__name__}:{error}"
    finally:
        if child is not None:
            kill_group(child.pid)
            if wait_status is None:
                _, wait_status, usage = os.wait4(child.pid, 0)
            child.returncode = os.waitstatus_to_exitcode(wait_status)
            result.update(exit_code=child.returncode, peak_rss_bytes=int(usage.ru_maxrss))
            if rss_cap is not None and result["peak_rss_bytes"] > rss_cap and not result["stop_reason"]:
                result["stop_reason"] = "exact_producer_peak_rss_cap"
    final_bytes = count_bytes() + pending_copy_bytes()
    result["end_monotonic_seconds"] = time.monotonic()
    if rss_cap is not None and last_snapshot is not None:
        observation["max_snapshot_gap_seconds"] = max(result["end_monotonic_seconds"] - last_snapshot, observation["max_snapshot_gap_seconds"])
    result["terminal_phase_checks"] = {"deadline": deadline, "output_bytes": final_bytes,
                                       "reserve_bytes": reserve, "output_cap_bytes": output_cap}
    if result["end_monotonic_seconds"] >= deadline and not result["stop_reason"]:
        result["stop_reason"] = "terminal_phase_wall_cap"
    if final_bytes + reserve >= output_cap and not result["stop_reason"]:
        result["stop_reason"] = "terminal_phase_output_cap"
    return result

def artifact_hashes(run, deadline):
    return {str(path.relative_to(run)): sha(path, deadline) for path in files_below(run)}
