"""Bindings and persistence for one prospective case; no cloth execution here."""
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]
sys.path.insert(0, str(ROOT / "consumer"))
import evaluator as c
import resources as r

EXPERIMENT = "WF-20261116-COUPLED-HARBOR-A8-001"
SOURCE = "96d3baee4543c7edbdb804df6090697a9f65da09"
CAP_SECONDS = 420.0
CAP_BYTES = 268435456


def read(path):
    return c.read(path)


def persist(path, value, deadline=None):
    r.deadline_check(deadline)
    data = r.encoded(value)
    r.write_new(path, data)
    with Path(path).open("rb") as stream:
        os.fsync(stream.fileno())
    r.deadline_check(deadline)
    return r.sha(path, deadline)


def environment():
    return {"implementation": sys.implementation.name, "version": list(sys.version_info[:3]),
            "binary_path": str(Path(sys.executable).resolve()),
            "binary_sha256": r.sha(Path(sys.executable).resolve()),
            "sum_probe": sum([1., 1e-16, 1e-16]), "optimize": sys.flags.optimize,
            "bytecode_disabled": sys.dont_write_bytecode, "platform": platform.system(),
            "rss_units": "bytes" if platform.system() == "Darwin" else "unsupported"}


def validate_environment(expected, deadline=None):
    r.require(sys.implementation.name == expected["implementation"] == "cpython", "Wrong Python implementation")
    r.require(list(sys.version_info[:3]) == expected["version"] == [3, 13, 0], "Wrong Python version")
    r.require(sys.flags.optimize == expected["optimize"] == 0, "Optimized Python disables numerical assertions")
    r.require(sys.dont_write_bytecode is True and expected["bytecode_disabled"] is True, "Use the bound Python with -B")
    r.require(sum([1., 1e-16, 1e-16]) == expected["sum_probe"] == 1.0000000000000002, "Python arithmetic context changed")
    r.require(platform.system() == expected["platform"] == "Darwin" and expected["rss_units"] == "bytes", "Exact wait4 RSS units changed")
    r.require(str(Path(sys.executable).resolve()) == expected["binary_path"], "Python path changed")
    r.require(r.sha(Path(sys.executable).resolve(), deadline) == expected["binary_sha256"], "Python binary changed")


def validate_go(value, freeze_sha):
    expected = {"status": "authorized", "explicit_root_review_and_go": True, "experiment": EXPERIMENT,
                "freeze_sha256": freeze_sha, "source_commit": SOURCE, "cases": ["base"],
                "init_only_initializations": 1, "actual_run_initializations": 1,
                "actual_dynamics_runs": 1, "init_only_step_calls": 0, "retries": 0, "continuations": 0}
    r.require(c.same(value, expected), "Distinct exact root GO absent or workload/hash differs")


def verify_bound(deadline=None):
    frozen = read(r.safe_file("FREEZE.json", ROOT))
    r.require(frozen["source_commit"] == SOURCE and frozen["experiment"] == EXPERIMENT, "Wrong source/experiment freeze")
    validate_environment(frozen["python"], deadline)
    actual_code = sorted(str(p.relative_to(ROOT)) for directory, pattern in
                         [(ROOT, "*.py"), (ROOT / "consumer", "*.py"), (ROOT / "producer/src", "*.rs")]
                         for p in directory.glob(pattern))
    r.require(actual_code == frozen["code_paths"], "Program/import source roster changed")
    for name, digest in frozen["files"].items():
        r.require(r.sha(r.safe_file(name, ROOT), deadline) == digest, "Prepared file changed: " + name)
    for name, digest in frozen["repo_files"].items():
        r.require(r.sha(r.safe_file(name, REPO), deadline) == digest, "Read-only repository binding changed: " + name)
    source = read(REPO / frozen["source88_path"])
    r.require(source["commit"] == SOURCE and source["source_count"] == len(source["source_hashes"]) == 88, "Wrong source roster")
    for name, digest in source["source_hashes"].items():
        r.require(r.sha(r.safe_file(name, REPO), deadline) == digest, "Actual source changed: " + name)
    timeout = max(.001, deadline-time.monotonic()) if deadline is not None else None
    r.require(subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True, timeout=timeout).strip() == SOURCE, "Source checkout changed")
    timeout = max(.001, deadline-time.monotonic()) if deadline is not None else None
    r.require(subprocess.check_output(["git", "status", "--porcelain"], cwd=REPO, text=True, timeout=timeout).strip() == "", "Tracked source checkout is dirty")
    binary = r.safe_file(frozen["producer_binary"]["path"], REPO)
    r.require(binary.parent == REPO / "target/release", "Producer is outside own worktree target")
    r.require(r.sha(binary, deadline) == frozen["producer_binary"]["sha256"], "Actual release producer changed")
    r.deadline_check(deadline)
    return frozen


def verify_session(deadline):
    frozen = verify_bound(deadline)
    session = read(r.safe_file("runtime/SESSION.json", ROOT))
    go = r.safe_file(session["go_path"], ROOT)
    r.require(r.sha(go, deadline) == session["go_sha256"], "Root GO changed")
    validate_go(read(go), r.sha(ROOT / "FREEZE.json", deadline))
    r.require(c.number(session["started"]) and c.number(session["deadline"]) and
              session["deadline"] == session["started"] + CAP_SECONDS and
              deadline == session["deadline"] and session["started"] <= time.monotonic(), "Wrong shared monotonic clock")
    r.require(r.sha(ROOT / "inputs/base.weftforge", deadline) == frozen["input_sha256"], "Actual input copy changed")
    return frozen, session


def verify_producer_receipt(deadline):
    frozen, session = verify_session(deadline)
    receipt = read(r.safe_file("runtime/producer-receipt.json", ROOT))
    r.require(receipt["source_commit"] == SOURCE and receipt["source88_sha256"] == frozen["repo_files"][frozen["source88_path"]], "Wrong producer source receipt")
    r.require(receipt["input_sha256"] == frozen["input_sha256"] and receipt["producer_sha256"] == frozen["producer_binary"]["sha256"], "Wrong producer/input receipt")
    stage = receipt["stage"]
    r.require(stage["exit_code"] == 0 and stage["stop_reason"] is None and
              type(stage["peak_rss_bytes"]) is int and 0 < stage["peak_rss_bytes"] <= CAP_BYTES and
              stage["end_monotonic_seconds"] <= deadline, "Producer operational/resource failure")
    actual = r.artifact_hashes(ROOT / "runs/base", deadline)
    original = {name: digest for name, digest in actual.items() if name != "analysis.json"}
    r.require(original == receipt["artifacts"], "Producer artifact coverage or hash changed")
    approved = read(r.safe_file("expected/approved-metadata.json", ROOT))
    r.require(c.same(read(ROOT / "runs/base/rest-metadata.json"), approved), "Run initialization differs from independently approved original metadata")
    r.require(r.sha(ROOT / "expected/approved-metadata.json", deadline) == receipt["approved_metadata_sha256"], "Approved metadata changed")
    gate = read(r.safe_file("runtime/initialized-original-gate.json", ROOT))
    r.require(r.sha(ROOT / "runtime/initialized-original-gate.json", deadline) == receipt["original_initialized_gate_sha256"], "Independent initialized-state gate changed")
    r.require(gate["status"] == "original_rest_independently_verified" and gate["source_commit"] == SOURCE and
              gate["comparison_is_to_original_gold_not_only_current_initialization"] is True and
              type(gate["cloth_step_calls"]) is int and gate["cloth_step_calls"] == 0 and
              gate["approved_metadata_sha256"] == receipt["approved_metadata_sha256"], "Original initialized-state approval missing")
    r.require(r.sha(ROOT / "preflight-base/rest-metadata.json", deadline) == gate["preflight_rest_sha256"] and
              r.sha(ROOT / "preflight-base/initialized-state.json", deadline) == gate["initialized_state_sha256"], "Initialized original-rest inputs changed")
    r.require(c.same(approved, read(ROOT / "preflight-base/rest-metadata.json")), "Approved metadata is not the independently checked initialization")
    return frozen, session, receipt


def close_supervisor(receipt, deadline):
    """The positive terminal verdict is sampled only after the receipt is fsynced and hashed."""
    r.require(len(r.encoded(receipt)) <= r.RECEIPT_RESERVE, "Supervisor receipt exceeds reserved size")
    digest = persist(ROOT / "runtime/SUPERVISOR.json", receipt, deadline)
    volume = r.numerical_bytes()
    closed = time.monotonic()
    r.require(closed < deadline and volume <= CAP_BYTES, "Cap exceeded after final receipt persistence/guards")
    return {"status": "complete", "operational_complete": True,
            "strict_quality_passed": receipt["strict_quality_passed"],
            "final_after_receipt_monotonic_seconds": closed,
            "whole_case_elapsed_seconds": closed-receipt["started"],
            "supervisor_receipt_sha256": digest, "numerical_output_bytes": volume,
            "native_or_reference_acceptance": False, "continuation_allowed": False}
