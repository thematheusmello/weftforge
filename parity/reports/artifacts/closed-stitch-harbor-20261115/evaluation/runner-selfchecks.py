"""Stub-only runner checks. Never launches the garment driver or evaluator."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace

sys.dont_write_bytecode = True
ROOT = Path(__file__).absolute().parent
spec = importlib.util.spec_from_file_location("capsule_runner", ROOT / "runner.py")
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


def expect_error(function, *args):
    try:
        function(*args)
    except (ValueError, FileExistsError, FileNotFoundError):
        return
    raise AssertionError("Expected closed refusal")


def main():
    workspace = Path(tempfile.mkdtemp(prefix="runner-selfcheck-", dir=ROOT))
    os.chmod(workspace, 0o700)
    results = []
    def check(name, function):
        result = function()
        results.append({"name": name, "status": "passed", "observations": result})
    def stub(name, code, *, seconds=3., rss_cap=runner.CAP_BYTES, output_cap=1024*1024,
             reserve=1024, ps_low=False):
        directory = workspace / name
        directory.mkdir()
        counter = lambda: sum(path.stat().st_size for path in runner.files_below(directory))
        command = [sys.executable, "-B", "-c", code, str(directory)]
        real_run = runner.subprocess.run
        if ps_low:
            runner.subprocess.run = lambda *args, **kwargs: SimpleNamespace(returncode=0, stdout="1\n", stderr="")
        try:
            result = runner.monitor(command, directory, directory / "stdout", directory / "stderr",
                                    time.monotonic()+seconds, counter, rss_cap=rss_cap,
                                    output_cap=output_cap, reserve=reserve)
        finally:
            runner.subprocess.run = real_run
        result["actual_output_bytes"] = counter()
        runner.write_new(directory / "monitor.json", runner.encoded(result))
        if result["pid"] is not None:
            try:
                os.wait4(result["pid"], os.WNOHANG)
            except ChildProcessError:
                pass
            else:
                raise AssertionError("Monitor did not own exactly one terminal reap")
        return directory, result
    def success():
        directory, result = stub("success", "import sys; print('accepted stub stdout'); print('accepted stub stderr',file=sys.stderr)")
        assert result["exit_code"] == 0 and result["stop_reason"] is None
        assert type(result["peak_rss_bytes"]) is int and 0 < result["peak_rss_bytes"] <= runner.CAP_BYTES
        assert "accepted stub stdout" in (directory / "stdout").read_text()
        assert "accepted stub stderr" in (directory / "stderr").read_text()
        return result
    def failure():
        directory, result = stub("exit-seven", "import sys; print('retained failure',file=sys.stderr); sys.exit(7)")
        assert result["exit_code"] == 7 and result["stop_reason"] is None
        assert "retained failure" in (directory / "stderr").read_text()
        return result
    def whole_group():
        code = "import os,time; pid=os.fork(); print('grandchild='+str(pid),flush=True) if pid else None; time.sleep(5)"
        directory, result = stub("wall-group", code, seconds=.25)
        assert result["exit_code"] == -9 and result["stop_reason"] == "whole_case_wall_cap"
        line = (directory / "stdout").read_text().strip()
        assert line.startswith("grandchild=")
        grandchild = int(line.split("=")[1])
        observed = subprocess.run(["/bin/ps", "-o", "stat=", "-p", str(grandchild)], capture_output=True, text=True, check=False)
        state = observed.stdout.strip()
        assert not state or state.startswith("Z"), f"Grandchild still running: {state}"
        result["grandchild_pid"] = grandchild
        result["grandchild_terminal_state"] = state or "absent"
        return result
    def sampled_rss():
        _, result = stub("sampled-rss", "import time; memory=bytearray(64*1024*1024); time.sleep(2)", rss_cap=32*1024*1024)
        assert result["stop_reason"] == "sampled_producer_rss_cap" and result["exit_code"] == -9
        assert result["sampled_peak_rss_bytes"] > 32*1024*1024 and result["peak_rss_bytes"] > 32*1024*1024
        return result
    def exact_rss():
        _, result = stub("exact-rss", "import time; memory=bytearray(64*1024*1024); time.sleep(.1)", rss_cap=32*1024*1024, ps_low=True)
        assert result["exit_code"] == 0 and result["stop_reason"] == "exact_producer_peak_rss_cap"
        assert result["sampled_peak_rss_bytes"] <= 32*1024*1024 < result["peak_rss_bytes"]
        return result
    def output_stop():
        code = "from pathlib import Path; import sys,time; Path(sys.argv[1],'numeric-rows.jsonl').write_bytes(b'x'*100000); time.sleep(2)"
        directory, result = stub("output-stop", code, output_cap=32768)
        assert result["stop_reason"] == "numerical_output_cap_stop_budget" and result["exit_code"] == -9
        assert (directory / "numeric-rows.jsonl").stat().st_size == 100000 and result["actual_output_bytes"] > 32768
        result["overshoot_retained_and_rejected"] = True
        return result
    def prelaunch():
        directory = workspace / "prelaunch"
        directory.mkdir()
        command = [sys.executable, "-B", "-c", "raise AssertionError('must not launch')"]
        result = runner.monitor(command, directory, directory / "stdout", directory / "stderr",
                                time.monotonic()+1., lambda: 100, output_cap=100, reserve=0)
        assert result["pid"] is None and result["exit_code"] is None and result["stop_reason"].startswith("monitor_error:")
        assert not (directory / "stdout").exists()
        late = runner.monitor(command, directory, directory / "late.stdout", directory / "late.stderr",
                              time.monotonic()-1., lambda: 0, output_cap=100, reserve=0)
        assert late["pid"] is None and late["stop_reason"].startswith("monitor_error:")
        return {"volume_refusal": result, "clock_refusal": late}
    def nonoverwrite():
        directory = workspace / "nonoverwrite"
        directory.mkdir()
        original = directory / "original"
        runner.write_new(original, b"preserve these bytes")
        expect_error(runner.write_new, original, b"overwrite forbidden")
        assert original.read_bytes() == b"preserve these bytes"
        result = runner.monitor([sys.executable, "-B", "-c", "raise AssertionError('must not launch')"], directory,
                                original, directory / "stderr", time.monotonic()+1., lambda: 0, reserve=0)
        assert result["pid"] is None and original.read_bytes() == b"preserve these bytes"
        return {"original_sha256": runner.sha(original), "monitor_refused_without_spawn": True}
    def numerical_scope():
        directory = workspace / "numerical-scope"
        directory.mkdir()
        for name in runner.OUTPUT_DIRECTORIES:
            (directory / name).mkdir()
            runner.write_new(directory / name / "data", b"abc")
        (directory / "source").mkdir()
        runner.write_new(directory / "source/excluded-binary-input", b"x"*100)
        runner.write_new(directory / "runs/analysis-numeric-rows.json", b"12345")
        os.link(directory / "inputs/data", directory / "runs/hardlink-counted-again")
        assert runner.numerical_bytes(directory) == 20
        link = directory / "runs/symlink"
        link.symlink_to(directory / "inputs/data")
        expect_error(runner.numerical_bytes, directory)
        expect_error(runner.safe_file, link, directory)
        link.unlink()
        fifo = directory / "runs/nonregular"
        os.mkfifo(fifo)
        expect_error(runner.numerical_bytes, directory)
        fifo.unlink()
        return {"exact_count": runner.numerical_bytes(directory), "hardlinks_counted_per_path": True,
                "symlinks_and_nonregular_refused": True, "source_excluded": True, "analysis_rows_counted": True}
    def self_size():
        observed = []
        for existing in (0, 90, 999, 65535, runner.CAP_BYTES-100):
            receipt, data = runner.receipt_bytes({"schema": 1, "status": "failed", "failure": "stub only"}, existing)
            assert receipt["numerical_output_bytes"] == existing+len(data)
            assert json.loads(data)["numerical_output_bytes"] == receipt["numerical_output_bytes"]
            observed.append({"existing_bytes": existing, "receipt_bytes": len(data), "counted_bytes": receipt["numerical_output_bytes"]})
        directory = workspace / "receipt-write"
        (directory / "inputs").mkdir(parents=True)
        (directory / "runs").mkdir()
        runner.write_new(directory / "inputs/data", b"abc")
        real_counter = runner.numerical_bytes
        runner.numerical_bytes = lambda: real_counter(directory)
        try:
            path = directory / "runs/receipt.json"
            receipt = runner.write_receipt(path, {"schema": 1, "status": "failed", "failure": "stub only"})
            assert receipt["numerical_output_bytes"] == real_counter(directory)
            original = path.read_bytes()
            expect_error(runner.write_receipt, path, {"schema": 1, "status": "overwritten"})
            assert path.read_bytes() == original
            observed.append({"actual_write_total_bytes": real_counter(directory), "exclusive_receipt_preserved": True})
        finally:
            runner.numerical_bytes = real_counter
        return observed
    def auth_refusals():
        valid = {"schema": 1, "status": "authorized", "source_commit": runner.SOURCE,
                 "cases": ["base", "height"], "attempts_per_case": 1, "explicit_root_authorization": True}
        runner.verify_scope(valid)
        for key, value in (("schema", True), ("status", "prepared"), ("attempts_per_case", True),
                           ("explicit_root_authorization", False), ("cases", ["height", "base"]),
                           ("cases", ["base", "height", "extra"]), ("source_commit", "stale")):
            changed = copy.deepcopy(valid)
            changed[key] = value
            expect_error(runner.verify_scope, changed)
        expect_error(runner.verify_scope, {})
        expect_error(runner.verify_bindings, workspace / "missing-authorization.json")
        for index, text in enumerate(('{"x":1,"x":2}', '{"x":NaN}', '{"x":1e999}')):
            path = workspace / f"invalid-json-{index}.json"
            runner.write_new(path, text.encode())
            expect_error(runner.read_json, path)
        return {"valid_scope_only_has_no_execution_authority_without_bindings": True,
                "invalid_scopes_refused": 8, "missing_authorization_refused": True, "malformed_numeric_json_refused": 3}
    def quality_scheduling():
        assert runner.schedule_next("complete", 0) and runner.schedule_next("complete", 2)
        assert not runner.schedule_next("complete", 1) and not runner.schedule_next("failed", 2)
        path = workspace / "quality-failed-analysis.json"
        producer = {"authorization_sha256": "stub-auth", "artifacts": {}, "start_monotonic_seconds": 1.}
        report = {"status": "analyzed_complete", "case": "base", "source_commit": runner.SOURCE,
                  "authorization_sha256": "stub-auth", "producer_artifacts": {}, "case_start_monotonic_seconds": 1.,
                  "rows": [{}]*181, "quality_status": "failed", **{key: False for key in runner.QUALITY_KEYS}}
        runner.write_new(path, runner.encoded(report))
        _, flags = runner.analysis_result(path, "base", producer, 2)
        assert flags["strict_delivery_passed"] is False
        expect_error(runner.analysis_result, path, "base", producer, 1)
        expect_error(runner.analysis_result, path, "base", producer, 0)
        return {"quality_failure_retained_and_allows_height": True, "operational_stop_disallows_height": True}
    status = "passed"
    error = None
    try:
        assert sys.platform == "darwin", "These exact-RSS checks require Darwin"
        for name, function in (("successful_child_exact_reap", success), ("nonzero_exit_retained_stderr", failure),
                               ("wall_stop_kills_whole_group", whole_group), ("sampled_rss_early_stop", sampled_rss),
                               ("exact_wait4_rss_rejects_even_when_samples_low", exact_rss), ("output_overshoot_retained_and_rejected", output_stop),
                               ("prelaunch_cap_and_deadline_refusal", prelaunch), ("nonoverwrite_before_spawn", nonoverwrite),
                               ("aggregate_scope_and_unsafe_entries", numerical_scope), ("receipt_self_size_fixed_point", self_size),
                               ("strict_authorization_and_json_refusal", auth_refusals), ("quality_failure_scheduling", quality_scheduling)):
            check(name, function)
    except BaseException as caught:
        status, error = "failed", f"{type(caught).__name__}: {caught}"
    report = {"schema": 1, "status": status, "new_garment_execution": False, "real_driver_launches": 0,
              "evaluator_launches": 0, "scope": "Python stub children and pure runner helpers only",
              "runner_sha256": runner.sha(ROOT / "runner.py"), "method_sha256": runner.sha(ROOT / "RUNNER-METHOD.md"),
              "selfchecks_sha256": runner.sha(Path(__file__)), "checks": results, "error": error,
              "outputs": str(workspace.relative_to(ROOT))}
    runner.write_new(workspace / "report.json", runner.encoded(report))
    print(json.dumps({"status": status, "passed_checks": len(results), "report": str(workspace / "report.json"), "error": error}))
    return 0 if status == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
