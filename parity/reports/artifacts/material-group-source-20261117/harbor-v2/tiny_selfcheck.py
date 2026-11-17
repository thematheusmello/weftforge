"""Tiny fake-child/mock-ps checks only; no Cargo, model or actual evaluator."""
import ast
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from types import SimpleNamespace
from unittest.mock import patch

import common as b
from common import ROOT, r


def main():
    reused = b.read(ROOT / "REUSED-SOURCES.json")
    same = []
    for name, digest in reused["original_files"].items():
        if name.startswith(("consumer/", "producer/")):
            r.require(r.sha(ROOT / name) == digest, "Changed producer/evaluator: " + name)
            same.append(name)
    for file in [*ROOT.glob("*.py"), *(ROOT / "consumer").glob("*.py")]:
        ast.parse(file.read_text(), filename=str(file))
    b.require_source()
    out = ROOT / "checks/tiny"
    out.mkdir()
    real_wait4, real_popen = os.wait4, subprocess.Popen
    results = []

    def child(name, code, *, observe, seconds=3., cap=1048576, rss=268435456, wait=real_wait4):
        directory = out / name
        directory.mkdir()
        launches, starts = [], []
        def launch(*args, **kwargs):
            process = real_popen(*args, **kwargs)
            launches.append(process.pid)
            return process
        def started(value):
            starts.append(value["pid"])
        def ps(*args, **kwargs):
            return observe(launches[0], *args, **kwargs)
        with patch.object(r.os, "wait4", wait), patch.object(r.subprocess, "Popen", launch), patch.object(r.subprocess, "run", ps):
            result = r.monitor([sys.executable, "-B", "-c", code], ROOT,
                directory / "stdout", directory / "stderr", time.monotonic() + seconds,
                lambda: sum(p.stat().st_size for p in r.files_below(directory)),
                rss_cap=rss, output_cap=cap, reserve=2 * r.RECEIPT_RESERVE, on_start=started)
        b.persist(directory / "receipt.json", result)
        r.require(launches == starts == [result["pid"]], "Fake child was restarted or PID changed")
        r.require(result["peak_rss_bytes"] is not None, "Missing exact direct-child wait4 RSS")
        try:
            real_wait4(result["pid"], os.WNOHANG)
        except ChildProcessError:
            pass
        else:
            raise ValueError("Fake child was not reaped")
        results.append({"name": name, **result})
        return result

    def good(pid, *args, **kwargs):
        r.require(0 < kwargs["timeout"] <= 1., "Unbounded ps timeout")
        return SimpleNamespace(returncode=0, stdout=f"{pid} {pid} 1024\n")
    calls = 0
    def transient(pid, *args, **kwargs):
        nonlocal calls
        calls += 1
        if calls == 1:
            raise subprocess.TimeoutExpired(args[0], kwargs["timeout"])
        return good(pid, *args, **kwargs)
    result = child("transient_same_pid", "import time;time.sleep(.12);print('fake completed')", observe=transient)
    r.require(result["exit_code"] == 0 and result["stop_reason"] is None and result["rss_observation"]["timeouts"] == 1 and result["rss_observation"]["successful_snapshots"] > 0, "Transient timeout killed/restarted/failed the child")

    saved_terminal = None
    def terminal_wait(pid, options):
        nonlocal saved_terminal
        if saved_terminal is not None:
            value, saved_terminal = saved_terminal, None
            return value
        return (0, 0, None)
    def ends_during_timeout(pid, *args, **kwargs):
        nonlocal saved_terminal
        # Real wait4 supplies the exact fake-child receipt; the mock delivers it on re-poll.
        saved_terminal = real_wait4(pid, 0)
        raise subprocess.TimeoutExpired(args[0], kwargs["timeout"])
    result = child("terminal_during_timeout", "print('fake terminal')", observe=ends_during_timeout, wait=terminal_wait)
    r.require(result["exit_code"] == 0 and result["stop_reason"] is None and result["rss_observation"]["timeouts"] == 1, "Child terminating during ps timeout was not accepted normally")

    def unavailable(pid, *args, **kwargs):
        raise subprocess.TimeoutExpired(args[0], kwargs["timeout"])
    result = child("three_unavailable", "import time;time.sleep(3)", observe=unavailable)
    r.require(result["exit_code"] == -9 and result["stop_reason"] == "rss_observation_unavailable" and result["rss_observation"]["failures"] == 3, "Repeated unavailable observations were accepted")

    def slow_timeout(pid, *args, **kwargs):
        time.sleep(kwargs["timeout"] + .01)
        raise subprocess.TimeoutExpired(args[0], kwargs["timeout"])
    result = child("wall_during_timeout", "import time;time.sleep(3)", observe=slow_timeout, seconds=.08)
    r.require(result["stop_reason"] == "whole_case_wall_cap", "Observation recovery relaxed the wall guard")
    def large(pid, *args, **kwargs):
        return SimpleNamespace(returncode=0, stdout=f"{pid} {pid} 262145\n")
    result = child("sampled_rss_cap", "import time;time.sleep(3)", observe=large)
    r.require(result["stop_reason"] == "sampled_producer_rss_cap", "RSS guard relaxed")

    def completed(pid, options):
        return real_wait4(pid, 0)
    result = child("terminal_output", "print('x'*4096)", observe=good,
                   cap=2 * r.RECEIPT_RESERVE + 1024, wait=completed)
    r.require(result["exit_code"] == 0 and result["stop_reason"] == "terminal_phase_output_cap", "Terminal output accepted")
    def late(pid, options):
        time.sleep(.08)
        return real_wait4(pid, 0)
    result = child("late_terminal", "print('fake completed child')", observe=good, seconds=.05, wait=late)
    r.require(result["exit_code"] == 0 and result["stop_reason"] == "terminal_phase_wall_cap", "Terminal deadline accepted")
    result = child("exact_direct_child_rss", "print('fake terminal')", observe=good, rss=1, wait=completed)
    r.require(result["exit_code"] == 0 and result["stop_reason"] == "exact_producer_peak_rss_cap", "Exact terminal wait4 RSS guard relaxed")

    for script, args in [("controller.py", ["--go", "missing", "--go-sha256", "0"*64]), ("bind_source.py", [])]:
        result = subprocess.run([sys.executable, "-B", str(ROOT / script), *args],
            env=dict(os.environ, PYTHONOPTIMIZE="1", PYTHONDONTWRITEBYTECODE="1"),
            capture_output=True, text=True, timeout=5)
        r.require(result.returncode != 0 and "Python optimization guard" in result.stderr, "Optimized launch not explicitly rejected")
    r.require(not any((ROOT / n).exists() for n in r.OUTPUT_DIRECTORIES)
              and not (ROOT / "SOURCE-PREFLIGHT.json").exists() and not (ROOT / "FREEZE.json").exists(),
              "Source-only check created an actual case")
    report = {"status": "passed_source_bound_fake_children_only", "source_commit": b.SOURCE,
        "exact_reused_producer_and_consumer_files": same, "Python_AST": "passed",
        "reviewed_source_guard": True, "same_pid_observation_recovery": True,
        "terminal_during_timeout_normally_reaped": True, "three_observation_failures_stop_negatively": True,
        "wall_sampled_and_exact_RSS_terminal_output_guards": True, "optimization_guards": True,
        "fake_children_reaped": len(results), "fake_child_results": results,
        "actual_model_initializations": 0, "actual_steps": 0, "actual_evaluator_runs": 0,
        "Cargo_invocations": 0, "real_case_clock_started": False,
        "old_qualification_reused_only_for_exact_sources": reused["original_qualification"]}
    b.persist(ROOT / "checks/TINY-SELFCHECKS.json", report)
    print(json.dumps({key: value for key, value in report.items() if key != "fake_child_results"}, indent=2))


if __name__ == "__main__":
    main()
