"""One authorized base/height attempt; preparation/selfchecks never call main."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import signal
import stat
import subprocess
import sys
import time
import bindings

sys.dont_write_bytecode = True
ROOT = Path(__file__).absolute().parent
REPO = ROOT.parents[1]
EVALUATOR = ROOT
SOURCE = bindings.SOURCE
PLAN_SHA = bindings.PROTOCOL_SHA
INPUT_BINDING_SHA = bindings.SOURCE_BINDING_SHA
DRIVER_SHA = None
CASES = ("base", "height")
CAP_SECONDS = 420.0
CAP_BYTES = 268435456
RECEIPT_RESERVE = 65536
OUTPUT_DIRECTORIES = ("inputs", "preflight-base", "preflight-height", "runs")
PRODUCER_FILES = {"rest-metadata.json", "trajectory.jsonl", "steps.jsonl", "stage-000.obj",
                  "stage-060.obj", "stage-180.obj", "completed-case.json", "producer.stdout", "producer.stderr"}
QUALITY_KEYS = ("strict_delivery_passed", "trajectory_physical_screens_passed",
                "completed_fit_all_frame_quality_passed", "final_frame_quality_passed", "all_frame_quality_passed")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def deadline_check(deadline):
    if deadline is not None:
        require(time.monotonic() < deadline, "Whole-case deadline closed; no numerical continuation")


def finite(value):
    try:
        return type(value) in (int, float) and math.isfinite(value)
    except OverflowError:
        return False


def read_json(path):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, f"Duplicate JSON key: {key}")
            result[key] = value
        return result
    def floating(token):
        value = float(token)
        require(math.isfinite(value), "Nonfinite JSON number")
        return value
    return json.loads(Path(path).read_text(), object_pairs_hook=pairs, parse_float=floating,
                      parse_constant=lambda token: require(False, f"Nonfinite JSON: {token}"))


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


def receipt_bytes(value, existing_bytes):
    value = dict(value)
    value["numerical_output_bytes"] = existing_bytes
    for _ in range(32):
        data = encoded(value)
        total = existing_bytes + len(data)
        if value["numerical_output_bytes"] == total:
            return value, data
        value["numerical_output_bytes"] = total
    raise ValueError("Receipt volume fixed point did not converge")


def write_receipt(path, value):
    value, data = receipt_bytes(value, numerical_bytes())
    write_new(path, data)
    require(numerical_bytes() == value["numerical_output_bytes"], "Numerical files changed during receipt write")
    return value


def descriptor(path, deadline=None):
    return {"path": str(Path(path).relative_to(REPO)), "sha256": sha(path, deadline)}


def verify_descriptor(value, expected, deadline=None):
    require(isinstance(value, dict) and set(value) == {"path", "sha256"}, "Descriptor needs path/sha256")
    require(isinstance(value["path"], str) and not Path(value["path"]).is_absolute(), "Descriptor must be repository-relative")
    actual = safe_file(value["path"], REPO)
    require(actual == expected and sha(actual, deadline) == value["sha256"], f"Wrong descriptor: {expected.name}")


def verify_scope(auth):
    require(isinstance(auth, dict) and type(auth.get("schema")) is int and auth["schema"] == 1,
            "Root execution authorization schema absent")
    require(auth.get("status") == "authorized" and auth.get("explicit_root_authorization") is True,
            "Separate explicit root execution authorization absent")
    require(auth.get("source_commit") == SOURCE and auth.get("cases") == list(CASES)
            and type(auth.get("attempts_per_case")) is int and auth["attempts_per_case"] == 1,
            "Authorization scope must be the frozen two one-attempt cases")


def verify_bindings(auth_path, expected_auth_sha=None, deadline=None):
    global DRIVER_SHA
    deadline_check(deadline)
    bound = bindings.authorization(auth_path, expected_auth_sha, deadline)
    verify_scope(bound["auth"])
    DRIVER_SHA = bound["producer"]["driver_sha256"]
    numerical_bytes()
    deadline_check(deadline)
    return {key: bound[key] for key in ("auth_path", "authorization_sha256", "plan", "evaluator_freeze_sha256")}


def kill_group(pid):
    try:
        os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def monitor(argv, cwd, stdout, stderr, deadline, count_bytes, *, rss_cap=None,
            output_cap=CAP_BYTES, reserve=RECEIPT_RESERVE, pending_copy_bytes=lambda: 0, on_start=None):
    """Own wait4 for this child; do not mix Popen poll/wait with exact rusage."""
    child, usage, wait_status = None, None, None
    result = {"argv": [str(value) for value in argv], "pid": None, "process_group": None,
              "exit_code": None, "peak_rss_bytes": None, "sampled_peak_rss_bytes": 0, "stop_reason": None}
    try:
        deadline_check(deadline)
        require(count_bytes() + pending_copy_bytes() + reserve < output_cap, "Numerical output stop budget reached before launch")
        no_links(stdout, Path(stdout).anchor)
        no_links(stderr, Path(stderr).anchor)
        with Path(stdout).open("xb") as out, Path(stderr).open("xb") as err:
            child = subprocess.Popen(argv, cwd=cwd, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"),
                                     stdout=out, stderr=err, start_new_session=True)
            result.update(pid=child.pid, process_group=child.pid)
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
                    remaining = max(.001, min(.25, deadline - time.monotonic()))
                    sample = subprocess.run(["/bin/ps", "-o", "rss=", "-p", str(child.pid)],
                                            capture_output=True, text=True, timeout=remaining, check=False)
                    if sample.returncode or not sample.stdout.strip():
                        pid, status, observed = os.wait4(child.pid, os.WNOHANG)
                        if pid:
                            wait_status, usage = status, observed
                            break
                        raise ValueError("Sampled producer RSS unavailable while child is live")
                    sampled = int(sample.stdout.strip()) * 1024
                    require(sampled >= 0, "Invalid sampled RSS")
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
    result["end_monotonic_seconds"] = time.monotonic()
    return result


def copy_new(source, target, deadline):
    deadline_check(deadline)
    with Path(source).open("rb") as incoming, Path(target).open("xb") as outgoing:
        while True:
            deadline_check(deadline)
            block = incoming.read(1024 * 1024)
            if not block:
                break
            outgoing.write(block)
    deadline_check(deadline)


def artifact_hashes(run, deadline):
    return {str(path.relative_to(run)): sha(path, deadline) for path in files_below(run)}


def analysis_result(path, case, producer_receipt, exit_code):
    value = read_json(path)
    require(exit_code in (0, 2) and value.get("status") == "analyzed_complete", "Independent arithmetic incomplete or refused")
    require(value.get("case") == case and value.get("source_commit") == SOURCE
            and value.get("authorization_sha256") == producer_receipt["authorization_sha256"]
            and value.get("producer_artifacts") == producer_receipt["artifacts"]
            and value.get("case_start_monotonic_seconds") == producer_receipt["start_monotonic_seconds"],
            "Independent analysis binding disagreement")
    require(len(value.get("rows", [])) == 181, "Independent arithmetic did not retain all181 boundary rows")
    flags = {key: value.get(key) for key in QUALITY_KEYS}
    require(all(type(flag) is bool for flag in flags.values()), "Independent quality flags incomplete")
    flags["quality_status"] = value.get("quality_status")
    require(flags["quality_status"] == ("passed" if flags["strict_delivery_passed"] else "failed")
            and exit_code == (0 if flags["strict_delivery_passed"] else 2), "Independent exit/quality disagreement")
    return value, flags


def schedule_next(case_status, analysis_exit):
    return case_status == "complete" and analysis_exit in (0, 2)


def run_case(case, binding):
    runs, run = ROOT / "runs", ROOT / "runs" / case
    logs, attempts = runs / ".producer-logs", runs / ".attempts"
    auth_path, auth_sha = binding["auth_path"], binding["authorization_sha256"]
    marker = attempts / f"{case}.json"
    require(not run.exists() and not run.is_symlink(), "Case output must initially be absent")
    write_new(marker, encoded({"schema": 1, "case": case, "attempt": 1, "source_commit": SOURCE,
                              "authorization_sha256": auth_sha, "wrapper_pid": os.getpid(), "no_resume_or_retry": True}))
    producer_stdout, producer_stderr = logs / f"{case}.stdout", logs / f"{case}.stderr"
    started = time.monotonic()
    deadline = started + CAP_SECONDS
    receipt = {"schema": 1, "case": case, "attempt": 1, "source_commit": SOURCE,
               "input_sha256": binding["plan"]["inputs_sha256"][f"inputs/{case}.weftforge"],
               "driver_sha256": DRIVER_SHA, "evaluator_freeze_sha256": binding["evaluator_freeze_sha256"],
               "authorization_sha256": auth_sha, "start_monotonic_seconds": started,
               "producer_end_monotonic_seconds": started, "producer_peak_rss_bytes": None,
               "producer_exit_code": None, "status": "producer_failed", "artifacts": {},
               "artifact_hashes_complete": False}
    final = None
    failure = None
    analysis_exit = None
    analysis_path, inspection_path = run / "analysis.json", run / "inspection.json"
    producer_receipt_path = None
    analysis_descriptor, inspection_descriptor = None, None
    flags = {"quality_status": "not_evaluated", **{key: None for key in QUALITY_KEYS}}
    def handle(value):
        write_new(logs / f"{case}.handle.json", encoded(value))
        print(json.dumps({"case": case, "producer_pid": value["pid"], "process_group": value["process_group"]}), flush=True)
    try:
        producer = monitor([str(ROOT / "candidate/harbor-public-driver"), "run",
                            str(ROOT / f"inputs/{case}.weftforge"), str(run)], ROOT,
                           producer_stdout, producer_stderr, deadline, numerical_bytes, rss_cap=CAP_BYTES,
                           pending_copy_bytes=lambda: sum(path.stat().st_size for path in (producer_stdout, producer_stderr) if path.exists()),
                           on_start=handle)
        receipt.update(producer_end_monotonic_seconds=producer["end_monotonic_seconds"],
                       producer_peak_rss_bytes=producer["peak_rss_bytes"], producer_exit_code=producer["exit_code"],
                       producer_sampled_peak_rss_bytes=producer["sampled_peak_rss_bytes"], stop_reason=producer["stop_reason"])
        if producer["stop_reason"]:
            receipt["status"] = "producer_stopped"
        elif producer["exit_code"] == 0:
            receipt["status"] = "producer_complete"
        deadline_check(deadline)
        require(numerical_bytes() + sum(path.stat().st_size for path in (producer_stdout, producer_stderr) if path.exists())
                + RECEIPT_RESERVE <= CAP_BYTES, "Numerical output stop budget reached after producer")
        if run.is_dir():
            no_links(run, ROOT)
            for original, name in ((producer_stdout, "producer.stdout"), (producer_stderr, "producer.stderr")):
                copy_new(original, run / name, deadline)
            receipt["artifacts"] = artifact_hashes(run, deadline)
            receipt["artifact_hashes_complete"] = True
        verify_bindings(auth_path, auth_sha, deadline)
        producer_receipt_path = (run / "producer-receipt.json") if run.is_dir() else attempts / f"{case}-producer-receipt.json"
        receipt = write_receipt(producer_receipt_path, receipt)
        require(receipt["status"] == "producer_complete" and receipt["producer_exit_code"] == 0
                and receipt["producer_peak_rss_bytes"] <= CAP_BYTES, "Producer failed or stopped; no independent arithmetic")
        require(set(receipt["artifacts"]) == PRODUCER_FILES, "Missing or unexpected producer files")
        complete = read_json(run / "completed-case.json")
        require(type(complete.get("completed_frames")) is int and complete["completed_frames"] == 180,
                "Producer did not complete180 steps")
        analyzed = monitor([sys.executable, "-B", str(EVALUATOR / "evaluator.py"), "analyze", "--case", case,
                            "--authorization", str(auth_path), "--receipt", str(producer_receipt_path),
                            "--output", str(analysis_path)], EVALUATOR, run / "analysis.stdout", run / "analysis.stderr",
                           deadline, numerical_bytes)
        analysis_exit = analyzed["exit_code"]
        require(not analyzed["stop_reason"], f"Independent arithmetic stopped: {analyzed['stop_reason']}")
        deadline_check(deadline)
        analysis, flags = analysis_result(analysis_path, case, receipt, analysis_exit)
        require(finite(analysis.get("analysis_completed_monotonic_seconds"))
                and receipt["producer_end_monotonic_seconds"] <= analysis["analysis_completed_monotonic_seconds"] <= time.monotonic(),
                "Independent arithmetic clock invalid")
        require(sha(producer_receipt_path, deadline) == analysis.get("receipt_sha256"), "Independent producer receipt changed")
        for name, digest in receipt["artifacts"].items():
            require(sha(safe_file(name, run), deadline) == digest, f"Producer artifact changed during analysis: {name}")
        verify_bindings(auth_path, auth_sha, deadline)
        inventory = {str(path.relative_to(REPO)): {"bytes": path.stat().st_size, "sha256": sha(path, deadline)}
                     for path in numerical_files()}
        inspection = {"schema": 1, "case": case, "source_commit": SOURCE, "all_bound_hashes_exact": True,
                      "numerical_files": inventory, "pending_metadata_paths": [str(path.relative_to(REPO)) for path in (inspection_path, run / "case-receipt.json")],
                      "numerical_output_bytes_at_inventory": numerical_bytes(), "no_new_numerical_acceptance_after_clock_close": True}
        write_new(inspection_path, encoded(inspection))
        analysis_descriptor, inspection_descriptor = descriptor(analysis_path, deadline), descriptor(inspection_path, deadline)
        verify_bindings(auth_path, auth_sha, deadline)
        require(numerical_bytes() + RECEIPT_RESERVE <= CAP_BYTES, "Output cap lacks final receipt reserve")
        deadline_check(deadline)
    except BaseException as error:
        failure = f"{type(error).__name__}: {error}"
    closed = time.monotonic()
    receipt_location = run if run.is_dir() and not run.is_symlink() else attempts
    if producer_receipt_path is None:
        producer_receipt_path = receipt_location / ("producer-receipt.json" if receipt_location == run else f"{case}-producer-receipt.json")
        receipt["failure"] = failure
        receipt = write_receipt(producer_receipt_path, receipt)
    final_path = receipt_location / ("case-receipt.json" if receipt_location == run else f"{case}-case-receipt.json")
    final = {**receipt, "status": "complete" if failure is None else "failed", "failure": failure,
             "producer_status": receipt["status"], "final_monotonic_seconds": closed,
             "whole_case_elapsed_seconds": closed-started, "analysis": analysis_descriptor,
             "inspection": inspection_descriptor, "analysis_exit_code": analysis_exit, "quality_flags": flags,
             "clock_close_scope": "All numerical analysis/export/hash/inspection/postbindings; final receipt serialization and audit are integrity metadata only",
             "no_new_numerical_acceptance_after_clock_close": True}
    final = write_receipt(final_path, final)
    post_write_elapsed, post_write_volume = time.monotonic()-started, numerical_bytes()
    audit = {"schema": 1, "case": case, "integrity_only": True,
             "no_new_numerical_acceptance_after_clock_close": True,
             "case_receipt": descriptor(final_path), "post_write_elapsed_seconds": post_write_elapsed,
             "post_write_numerical_output_bytes": post_write_volume,
             "post_write_wall_within_cap": post_write_elapsed <= CAP_SECONDS,
             "post_write_output_within_cap": post_write_volume <= CAP_BYTES,
             "case_clock_closed_status": final["status"], "quality_flags": flags}
    operational = failure is None and audit["post_write_wall_within_cap"] and audit["post_write_output_within_cap"]
    if operational:
        try:
            audit_dir = ROOT / "runner-audits"
            checked = monitor([sys.executable, "-B", str(EVALUATOR / "evaluator.py"), "validate-receipt", "--case", case,
                               "--authorization", str(auth_path), "--receipt", str(final_path),
                               "--output", str(audit_dir / f"{case}-validation.json")], EVALUATOR,
                              audit_dir / f"{case}-validation.stdout", audit_dir / f"{case}-validation.stderr",
                              time.monotonic()+60., numerical_bytes, reserve=0)
            audit["validation_process"] = checked
            operational = checked["exit_code"] == 0 and not checked["stop_reason"]
            require(sha(final_path) == audit["case_receipt"]["sha256"], "Final receipt changed during integrity audit")
            if analysis_descriptor is not None:
                require(descriptor(analysis_path) == analysis_descriptor, "Analysis changed during integrity-only audit")
        except BaseException as error:
            operational = False
            audit["integrity_failure"] = f"{type(error).__name__}: {error}"
    audit["operational_completion"] = operational
    audit["scheduling"] = "next_case_allowed" if operational and schedule_next(final["status"], analysis_exit) else "stop_no_resume_or_retry"
    if not operational:
        audit["failure"] = failure or "Post-write budget or integrity audit failed; no case credited as within-budget success"
    write_new(ROOT / "runner-audits" / f"{case}-integrity.json", encoded(audit))
    print(json.dumps({"case": case, "status": final["status"], "operational_completion": operational,
                      "quality_flags": flags, "receipt": str(final_path.relative_to(REPO))}), flush=True)
    return operational and schedule_next(final["status"], analysis_exit), flags["strict_delivery_passed"] is True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--authorization", type=Path, required=True)
    args = parser.parse_args()
    try:
        binding = verify_bindings(args.authorization)
        require(numerical_bytes() + RECEIPT_RESERVE < CAP_BYTES, "Initial numerical output stop budget reached")
        for name in ("runs", "runs/.attempts", "runs/.producer-logs", "runner-audits"):
            path = no_links(ROOT / name, ROOT)
            path.mkdir(exist_ok=True)
            require(path.is_dir(), "Runner directory is not real")
        for case in CASES:
            for path in (ROOT / "runs" / case, ROOT / "runs/.attempts" / f"{case}.json"):
                require(not path.exists() and not path.is_symlink(), "An attempt already exists; no resume or retry")
            for path in (list((ROOT / "runs/.producer-logs").glob(case + ".*"))
                         + list((ROOT / "runs/.attempts").glob(case + "-*"))
                         + list((ROOT / "runner-audits").glob(case + "-*"))):
                require(False, f"Prior case artifact exists: {path.name}")
        quality_passed = True
        for case in CASES:
            binding = verify_bindings(binding["auth_path"], binding["authorization_sha256"])
            operational, quality = run_case(case, binding)
            quality_passed = quality_passed and quality
            if not operational:
                return 1
        return 0 if quality_passed else 2
    except BaseException as error:
        print(json.dumps({"status": "refused_or_failed", "error": f"{type(error).__name__}: {error}",
                          "no_resume_or_retry": True}), file=sys.stderr, flush=True)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
