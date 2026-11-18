"""Bindings and persistence for one prospective case; no cloth execution here."""
import json
import os
from pathlib import Path
import platform
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

EXPERIMENT = "WF-20261117-MATERIAL-POLLING-HARBOR-A8-001"
SOURCE = _candidate.get("source_commit")
OWN_TARGET = "target"
CAP_SECONDS = 420.0
CAP_BYTES = 268435456


def read(path):
    return c.read(path)


def require_source(deadline=None):
    r.require(_candidate.get("status") == "root_reviewed_candidate" and _candidate.get("root_confirms_green_source_and_review") is True and REPO is not None and
              isinstance(SOURCE, str) and len(SOURCE) == 40 and all(q in "0123456789abcdef" for q in SOURCE),
              "Reviewed candidate source is pending; no build/freeze/initialization/dynamics authorized")
    r.require(ROOT == REPO / ".local/harbor-material-polling-preparation-20261117",
              "Install the source-only packet in the reviewed candidate checkout before final preparation")
    r.require(c.same(read(ROOT / "CANDIDATE.json"), _candidate), "Candidate binding changed")
    for path_key, sha_key in [("source_manifest", "source_manifest_sha256"),
                              ("source_review_evidence", "source_review_sha256"),
                              ("green_checks_evidence", "green_checks_sha256")]:
        r.require(r.sha(r.safe_file(_candidate[path_key], REPO), deadline) == _candidate[sha_key],
                  "Reviewed candidate evidence changed: " + path_key)
    source = read(REPO / _candidate["source_manifest"])
    r.require(source["commit"] == SOURCE and type(source["source_count"]) is int and source["source_count"] == len(source["source_hashes"]) == 88,
              "Wrong reviewed source manifest")
    for name, digest in source["source_hashes"].items():
        r.require(r.sha(r.safe_file(name, REPO), deadline) == digest, "Reviewed source changed: " + name)
    review = read(REPO / _candidate["source_review_evidence"])
    r.require(review["commit"] == SOURCE and review["root_read_production_diff_validation_callers_final_tests_physics_and_independent_review"] is True and
              review["author_manifest_sha256"] == _candidate["green_checks_sha256"] and
              review["independent_review_sha256"] == _candidate["independent_review_sha256"], "Root source-review/component evidence differs")
    for name, digest in review["source_hashes"].items():
        r.require(source["source_hashes"][name] == digest, "Reviewed component/source roster differs")
    for path_key, sha_key in [("independent_review_evidence", "independent_review_sha256"),
                              ("workspace_gate_start_freeze", "workspace_gate_start_sha256"),
                              ("workspace_gates_evidence", "workspace_gates_sha256"),
                              ("original8_guard", "original8_guard_sha256")]:
        r.require(r.sha(r.safe_file(_candidate[path_key], REPO), deadline) == _candidate[sha_key], "Source/metadata guard origin changed")
    gate = read(REPO / _candidate["workspace_gate_start_freeze"])
    r.require(gate["commit"] == SOURCE and gate["source88"] == source["source_hashes"], "Workspace source freeze differs")
    workspace = read(REPO / _candidate["workspace_gates_evidence"])
    summary = workspace["workspace_test_summary"]
    r.require(workspace["commit"] == SOURCE and workspace["status"] == "GREEN_FOUR_WORKSPACE_SOURCE_GATES" and
              workspace["all4_stages_passed"] is True and type(workspace["errors"]) is list and workspace["errors"] == [] and
              workspace["source_freeze_sha256"] == _candidate["workspace_gate_start_sha256"] and
              workspace["source88"] == source["source_hashes"] and
              workspace["metadata_frozen"] == gate["metadata_frozen"] and workspace["original8"] == gate["original8"] and
              summary["all_status_ok"] is True and
              all(type(summary[key]) is int and summary[key] == expected for key, expected in
                  [("passed",412),("failed",0),("ignored",0),("summary_count",31)]) and
              type(summary["raw_summaries"]) is list and len(summary["raw_summaries"]) == 31 and
              all(entry["status"] == "ok" and all(type(entry[key]) is int and entry[key] >= 0
                  for key in ["passed","failed","ignored"]) for entry in summary["raw_summaries"]) and
              all(sum(entry[key] for entry in summary["raw_summaries"]) == summary[key]
                  for key in ["passed","failed","ignored"]),
              "Actual four-stage412 source result/counts do not match the full88 freeze")
    r.require(type(workspace["stages"]) is list and len(workspace["stages"]) == 4 and
              [stage["name"] for stage in workspace["stages"]] ==
                  ["full-fmt","workspace-tests","workspace-clippy","native-cli-debug"] and
              [{"name":stage["name"],"argv":stage["argv"]} for stage in workspace["stages"]] == gate["commands"] and
              all(type(stage["exit_code"]) is int and stage["exit_code"] == 0 and stage["stop_reason"] is None and
                  stage["all_source88_metadata59_original8_exact"] is True and
                  all(stage[where] == {"all_source88_metadata59_original8_exact":True,
                      "clean_exact_commit":True,"tools_configs_compiler_inventory_exact":True}
                      for where in ["pre_guard","post_guard"]) for stage in workspace["stages"]) and
              workspace["final_guard"] == {"all_source88_metadata59_original8_exact":True,
                  "clean_exact_commit":True,"tools_configs_compiler_inventory_exact":True},
              "Four actual workspace stages/full88 source guards are not green")
    r.require(review["workspace_gates_result_sha256"] == _candidate["workspace_gates_sha256"] and
              review["old_case_regrade"] is False and review["full_garment_or_reference_acceptance"] is False,
              "Root source/workspace review qualification differs")
    final = read(REPO / _candidate["workspace_gates_final_manifest"])
    r.require(r.sha(r.safe_file(_candidate["workspace_gates_final_manifest"],REPO),deadline) ==
                  _candidate["workspace_gates_final_sha256"] and final["commit"] == SOURCE and
              final["result_sha256"] == _candidate["workspace_gates_sha256"] and
              final["source_freeze_sha256"] == _candidate["workspace_gate_start_sha256"] and
              final["all4_stages_passed"] is True and final["source88_metadata59_original8_exact_all_stages"] is True and
              final["postclose_guards"]["source88"] == 88 and
              final["postclose_guards"]["all_hashes_modes_exact"] is True,
              "Closed four-gate manifest/full source binding differs")
    r.require(r.sha(r.safe_file(_candidate["fmt_receipt_evidence"], REPO), deadline) ==
                  _candidate["fmt_receipt_sha256"], "Actual full88 FMT receipt changed")
    fmt = read(REPO / _candidate["fmt_receipt_evidence"])
    r.require(fmt == workspace["stages"][0] and fmt["name"] == "full-fmt" and
              type(fmt["exit_code"]) is int and fmt["exit_code"] == 0 and fmt["stop_reason"] is None and
              fmt["argv"] == gate["commands"][0]["argv"] and fmt["argv"][1:] == ["fmt","--all","--check"] and
              fmt["all_source88_metadata59_original8_exact"] is True,
              "Actual full FMT receipt is not the first passed stage under the exact88 source freeze")
    for name, digest in {**gate["metadata_frozen"], **read(REPO / _candidate["original8_guard"])}.items():
        r.require(r.sha(r.safe_file(name, REPO), deadline) == digest, "Frozen pre-existing metadata/inventory changed: " + name)
    timeout = max(.001, deadline-time.monotonic()) if deadline is not None else None
    r.require(subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True, timeout=timeout).strip() == SOURCE and
              subprocess.check_output(["git", "status", "--porcelain"], cwd=REPO, text=True, timeout=timeout).strip() == "",
              "Reviewed candidate checkout changed or is dirty")
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
    timeout = max(.001, deadline-time.monotonic()) if deadline is not None else None
    r.require(subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True, timeout=timeout).strip() == SOURCE, "Source checkout changed")
    timeout = max(.001, deadline-time.monotonic()) if deadline is not None else None
    r.require(subprocess.check_output(["git", "status", "--porcelain"], cwd=REPO, text=True, timeout=timeout).strip() == "", "Tracked source checkout is dirty")
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
    volume = r.numerical_bytes()
    closed = time.monotonic()
    r.require(closed < deadline and volume <= CAP_BYTES, "Cap exceeded after final receipt persistence/guards")
    return {"status": "complete", "operational_complete": True,
            "strict_quality_passed": receipt["strict_quality_passed"],
            "final_after_receipt_monotonic_seconds": closed,
            "whole_case_elapsed_seconds": closed-receipt["started"],
            "supervisor_receipt_sha256": digest, "numerical_output_bytes": volume,
            "native_or_reference_acceptance": False, "continuation_allowed": False}
