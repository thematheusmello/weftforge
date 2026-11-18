"""Bindings and persistence for one prospective case; no cloth execution here."""
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time
if sys.flags.optimize != 0 or os.environ.get('PYTHONOPTIMIZE') not in (None,'0'):
    raise RuntimeError('Python optimization guard: require optimization0 and absent/literal0 PYTHONOPTIMIZE')

ROOT = Path(__file__).resolve().parent
_candidate = json.loads((ROOT / "CANDIDATE.json").read_text())
REPO = Path(_candidate["checkout"]).resolve() if isinstance(_candidate.get("checkout"), str) else None
sys.path.insert(0, str(ROOT / "consumer"))
import evaluator as c
import resources as r

EXPERIMENT = "WF-20261118-CAPSULE-SUPPORTING-PLANE-HARBOR-A8-001"
SOURCE = _candidate.get("source_commit")
OWN_TARGET = "target"
CAP_SECONDS = 420.0
CAP_BYTES = 268435456


def read(path):
    return c.read(path)


CHECKED_SOURCE = "c766f3fa48c03ac698da8b22cc3bf927ee71f195"


def validate_workspace(plan, result, terminal, external, post, stdout, hashes):
    r.require(plan["head"] == result["head"] == CHECKED_SOURCE and plan["tracked_status"] == "",
              "Workspace checks must retain their actual c766 execution identity")
    r.require(result["errors"] == [] and result["all_source91_exact"] is True and
              result["source91"] == plan["source91"] and
              {name: entry["sha256"] for name, entry in plan["source91"].items()} == hashes,
              "Workspace source roster changed or source checks failed")
    commands = plan["commands"]
    r.require([q["name"] for q in commands] ==
              ["full-fmt", "workspace-tests", "workspace-clippy", "native-cli-build"] and
              len(result["stages"]) == 4 and
              all(stage["argv"] == command["argv"] and stage["exit_code"] == 0 and stage["stop_reason"] is None
                  for stage, command in zip(result["stages"], commands)),
              "Actual fmt/test/Clippy/native+CLI source checks did not all pass")
    raw = re.findall(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", stdout)
    summary = {"summaries": len(raw), "passed": sum(int(q[1]) for q in raw),
               "failed": sum(int(q[2]) for q in raw), "ignored": sum(int(q[3]) for q in raw)}
    r.require(len(raw) == 32 and all(q[0] == "ok" for q in raw) and
              summary == result["test_summary"] == {"summaries":32,"passed":480,"failed":0,"ignored":0},
              "Actual current workspace test summaries differ")
    r.require(terminal["all_direct_children_reaped"] is True and terminal["remaining_owned_groups"] == [] and
              external["controller_exit_code"] == 0 and external["launcher_timeout"] is False and
              external["controller_terminal"]["protocol_complete_under_caps"] is True and post["accepted"] is True,
              "Workspace source-check terminal was not accepted")



def validate_execution_chain(commit, result_sha, terminal, terminal_sha, external, external_sha, post):
    r.require(terminal["result_sha256"] == result_sha and
              external["metadata_commit"] == commit and
              external["controller_terminal"]["terminal_sha256"] == terminal_sha and
              post["external_terminal_sha256"] == external_sha,
              "Selected result/terminal/external/post execution hash chain differs")
    r.require(terminal["all_direct_children_reaped"] is True and terminal["remaining_owned_groups"] == [] and
              external["controller_exit_code"] == 0 and external["launcher_timeout"] is False and
              external["controller_terminal"]["protocol_complete_under_caps"] is True and post["accepted"] is True,
              "Selected execution chain was not accepted")


def validate_xtask_result(result, stage):
    r.require(result["head"] == SOURCE and result["errors"] == [] and
              any(c.same(stage, entry) for entry in result["stages"]),
              "Selected xtask stage is not in its actual current-source result")


def validate_xtask(facts, stage, external, hashes):
    r.require(facts["commit"] == SOURCE and facts["clean_before"] is True and facts["clean_after"] is True and
              facts["source_hashes_before"] == facts["source_hashes_after"] == hashes,
              "Clean current xtask source binding differs")
    r.require(len(stage["argv"]) >= 3 and Path(stage["argv"][0]).name == "cargo" and
              stage["argv"][1:3] == ["xtask", "verify"] and
              stage["exit_code"] == 0 and stage["stop_reason"] is None and external["accepted"] is True,
              "Mandatory current cargo xtask verify is missing, failed or unaccepted")


def require_clean_checkout(deadline=None):
    def git(*argv):
        timeout = max(.001, deadline-time.monotonic()) if deadline is not None else None
        return subprocess.check_output(["git",*argv],cwd=REPO,text=True,timeout=timeout).strip()
    r.require(git("rev-parse","HEAD") == SOURCE and git("status","--porcelain") == "",
              "Current checkout changed or is dirty")


def require_source(deadline=None):
    r.require(_candidate.get("status") == "root_reviewed_candidate" and
              _candidate.get("root_confirms_green_source_and_review") is True and
              _candidate.get("root_install_and_source_bind_authorized") is True and REPO is not None and
              isinstance(SOURCE, str) and re.fullmatch(r"[0-9a-f]{40}", SOURCE) is not None and
              _candidate.get("test_source_commit") == CHECKED_SOURCE,
              "Current source/root approval is pending; no build/freeze/initialization/dynamics authorized")
    r.require(ROOT == REPO / ".local/harbor-capsule-supporting-plane-preparation-20261118", "Wrong case installation")
    r.require(c.same(read(ROOT / "CANDIDATE.json"), _candidate), "Candidate binding changed")
    for path_key, sha_key in [("source_manifest", "source_manifest_sha256"),
                              ("source_review_evidence", "source_review_sha256"),
                              ("green_checks_evidence", "green_checks_sha256")]:
        r.require(r.sha(r.safe_file(_candidate[path_key], REPO), deadline) == _candidate[sha_key],
                  "Reviewed candidate evidence changed: " + path_key)
    for name, digest in _candidate["evidence_hashes"].items():
        r.require(r.sha(r.safe_file(name, REPO), deadline) == digest, "Source evidence changed: " + name)
    source = read(REPO / _candidate["source_manifest"])
    hashes = source["source_hashes"]
    r.require(source["commit"] == SOURCE and type(source["source_count"]) is int and
              source["source_count"] == len(hashes) > 0, "Wrong current source roster")
    for name, digest in hashes.items():
        r.require(r.sha(r.safe_file(name, REPO), deadline) == digest, "Reviewed source changed: " + name)
    review = read(REPO / _candidate["source_review_evidence"])
    r.require(review["commit"] == SOURCE and review["actual_workspace_execution_commit"] == CHECKED_SOURCE and
              review["root_read_production_diff_validation_callers_final_tests_physics_and_independent_review"] is True and
              review["root_has_reviewed_v6_empty_notch_extension_and_original_metadata_fields"] is True and
              review["root_read_actual_current_required_check_receipts"] is True and
              review["root_reviewed_initial_recovery_policy_counter_and_remaining_nonlinear_limits"] is True and
              review["source_hashes"] == hashes and
              review["source_prerequisites_sha256"] == _candidate["green_checks_sha256"] and
              review["independent_review_sha256"] == _candidate["independent_review_sha256"] and
              review["old_case_regrade"] is False and review["full_garment_or_reference_acceptance"] is False,
              "Genuine current root source review differs")
    facts = read(REPO / _candidate["green_checks_evidence"])
    r.require(facts["commit"] == SOURCE and facts["actual_workspace_execution_commit"] == CHECKED_SOURCE and
              facts["execution_transfer_claimed"] is False and facts["source_hashes"] == hashes,
              "Current source equivalence and actual execution attribution differ")
    def artifact(description):
        return c.descriptor(description, REPO)
    w = facts["workspace"]
    plan_path, result_path = artifact(w["plan"]), artifact(w["result"])
    plan, result = read(plan_path), read(result_path)
    terminal_path, external_path, post_path = [artifact(w[n]) for n in ["terminal", "external_terminal", "post_terminal_clock"]]
    terminal, external, post = [read(path) for path in [terminal_path, external_path, post_path]]
    stdout = artifact(w["stdout"]).read_text()
    r.require(result["plan_sha256"] == r.sha(plan_path, deadline) and
              terminal["result_sha256"] == r.sha(result_path, deadline), "Workspace result/plan terminal hash differs")
    validate_workspace(plan, result, terminal, external, post, stdout, hashes)
    validate_execution_chain(CHECKED_SOURCE, r.sha(result_path, deadline), terminal,
                             r.sha(terminal_path, deadline), external, r.sha(external_path, deadline), post)
    from compiler_inputs import compiler_input_roster, studio_input_roster, validate_compile_coverage
    approved_producer = read(ROOT / "REUSED-SOURCES.json")["producer_compile_inputs"]
    r.require(review["approved_producer_compile_inputs"] == approved_producer,
              "Standalone producer compile roster differs from the hash-bound root review")
    validate_compile_coverage(studio_input_roster(REPO), plan["source91"],
                              compiler_input_roster(ROOT, ("producer",)), approved_producer)
    x = facts["xtask"]
    stage = read(artifact(x["stage"]))
    result_path, terminal_path, external_path, post_path = [artifact(x[n]) for n in
        ["result", "terminal", "external_terminal", "post_terminal_clock"]]
    result, terminal, external, post = [read(path) for path in
        [result_path, terminal_path, external_path, post_path]]
    validate_xtask(x, stage, post, hashes)
    validate_xtask_result(result, stage)
    validate_execution_chain(SOURCE, r.sha(result_path, deadline), terminal,
                             r.sha(terminal_path, deadline), external, r.sha(external_path, deadline), post)
    require_clean_checkout(deadline)
    return source


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
    require_source()
    expected = {"status": "authorized", "explicit_root_review_and_go": True, "experiment": EXPERIMENT,
                "freeze_sha256": freeze_sha, "source_commit": SOURCE, "cases": ["base"],
                "build_binding_sha256": read(ROOT / "SOURCE-PREFLIGHT.json")["build_binding_sha256"],
                "init_only_initializations": 1, "actual_run_initializations": 1,
                "actual_dynamics_runs": 1, "init_only_step_calls": 0, "retries": 0, "continuations": 0}
    r.require(c.same(value, expected), "Distinct exact root GO absent or workload/hash differs")


def verify_bound(deadline=None):
    source = require_source(deadline)
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
    r.require(frozen["source_path"] == _candidate["source_manifest"], "Wrong reviewed source path")
    from source_inputs import verify_preflight
    verify_preflight(deadline)
    require_clean_checkout(deadline)
    binary = r.safe_file(frozen["producer_binary"]["path"], REPO)
    r.require(binary.parent == REPO / OWN_TARGET / "release", "Producer is outside own candidate target")
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
    r.require(receipt["source_commit"] == SOURCE and receipt["source_manifest_sha256"] == frozen["repo_files"][frozen["source_path"]], "Wrong producer source receipt")
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
    volume = r.numerical_bytes(deadline=deadline)
    closed = time.monotonic()
    r.require(closed < deadline and volume <= CAP_BYTES, "Cap exceeded after final receipt persistence/guards")
    return {"status": "complete", "operational_complete": True,
            "strict_quality_passed": receipt["strict_quality_passed"],
            "final_after_receipt_monotonic_seconds": closed,
            "whole_case_elapsed_seconds": closed-receipt["started"],
            "supervisor_receipt_sha256": digest, "numerical_output_bytes": volume,
            "native_or_reference_acceptance": False, "continuation_allowed": False}
