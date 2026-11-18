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

EXPERIMENT = "WF-20261117-DUAL-SEWING-HARBOR-A8-001"
SOURCE = _candidate.get("source_commit")
OWN_TARGET = "target"
CAP_SECONDS = 420.0
CAP_BYTES = 268435456


def read(path):
    return c.read(path)


def require_source(deadline=None):
    r.require(_candidate.get("status") == "root_reviewed_candidate" and
              _candidate.get("root_confirms_green_source_and_review") is True and
              _candidate.get("root_install_and_source_bind_authorized") is True and REPO is not None and
              SOURCE == "ef313fa869647c708a163853301ebc1d70a1d0b2",
              "Current source/root approval is pending; no build/freeze/initialization/dynamics authorized")
    r.require(ROOT == REPO / ".local/harbor-dual-sewing-preparation-20261117", "Wrong case installation")
    r.require(c.same(read(ROOT / "CANDIDATE.json"), _candidate), "Candidate binding changed")
    for path_key, sha_key in [("source_manifest", "source_manifest_sha256"),
                              ("source_review_evidence", "source_review_sha256"),
                              ("green_checks_evidence", "green_checks_sha256")]:
        r.require(r.sha(r.safe_file(_candidate[path_key], REPO), deadline) == _candidate[sha_key],
                  "Reviewed candidate evidence changed: " + path_key)
    for name, digest in _candidate["evidence_hashes"].items():
        r.require(r.sha(r.safe_file(name, REPO), deadline) == digest, "Source evidence changed: " + name)
    source = read(REPO / _candidate["source_manifest"])
    r.require(source["commit"] == SOURCE and type(source["source_count"]) is int and
              source["source_count"] == len(source["source_hashes"]) == 91, "Wrong current source roster")
    for name, digest in source["source_hashes"].items():
        r.require(r.sha(r.safe_file(name, REPO), deadline) == digest, "Reviewed source changed: " + name)
    def evidence(name):
        return read(ROOT / "evidence" / name)
    review = read(REPO / _candidate["source_review_evidence"])
    r.require(review["commit"] == SOURCE and
              review["root_read_production_diff_validation_callers_final_tests_physics_and_independent_review"] is True and
              review["root_has_reviewed_v6_empty_notch_extension_and_original_metadata_fields"] is True and
              review["source_hashes"] == source["source_hashes"] and
              review["source_prerequisites_sha256"] == _candidate["green_checks_sha256"] and
              review["independent_review_sha256"] == _candidate["independent_review_sha256"] and
              review["old_case_regrade"] is False and review["full_garment_or_reference_acceptance"] is False,
              "Genuine current root source review differs")
    current_inputs = evidence("CURRENT-SOURCE-INPUTS.json")
    for group in ["source91", "extra_compile_include_inputs", "authored_root_fixture_files", "original8_current_metadata_files"]:
        for name, expected in current_inputs[group].items():
            path = r.safe_file(name, REPO)
            r.require(r.sha(path, deadline) == expected["sha256"] and
                      path.stat().st_size == expected["bytes"] and path.stat().st_mode & 0o777 == expected["mode"],
                      "Current source/fixture bytes or mode changed: " + name)
    r.require({name: item["sha256"] for name, item in current_inputs["source91"].items()} == source["source_hashes"],
              "Current source manifest and compile-check input roster differ")
    parent = evidence("PARENT451-RESULT.json")
    parent_inputs = evidence("PARENT451-SOURCE-INPUTS.json")
    summary = parent["workspace_test_summary"]
    r.require(parent["commit"] == "526b9451b41900200359afa8b00f0abb214132bc" and
              parent["status"] == "FAILED_WORKSPACE_SOURCE_GATES" and parent["source91"] == parent_inputs["source91"] and
              summary["all_status_ok"] is True and
              all(type(summary[key]) is int and summary[key] == expected for key, expected in
                  [("passed",451),("failed",0),("ignored",0),("summary_count",32)]) and
              type(summary["raw_summaries"]) is list and len(summary["raw_summaries"]) == 32 and
              all(entry["status"] == "ok" and all(type(entry[key]) is int and entry[key] >= 0
                  for key in ["passed","failed","ignored"]) for entry in summary["raw_summaries"]) and
              all(sum(entry[key] for entry in summary["raw_summaries"]) == summary[key]
                  for key in ["passed","failed","ignored"]), "Actual parent451 test evidence differs")
    tests = [stage for stage in parent["stages"] if stage["name"] == "workspace-tests"]
    r.require(len(tests) == 1 and tests[0]["exit_code"] == 0 and tests[0]["stop_reason"] is None and
              tests[0]["workspace_test_summary"] == summary, "Parent workspace test stage did not pass")
    alias = evidence("ALIAS-EXPANSION-PROOF.json")
    constraints = "crates/weftforge-sim/src/constraints.rs"
    old = (ROOT / "evidence/CONSTRAINTS-526.rs").read_bytes()
    new = (REPO / constraints).read_bytes()
    declaration = b"type SoftSewingTrial = (Vec<Vec3>, Vec<Vec3>);\n\n"
    signature = b") -> Result<Option<SoftSewingTrial>, SimulationError> {"
    expanded = b") -> Result<Option<(Vec<Vec3>, Vec<Vec3>)>, SimulationError> {"
    r.require(alias["commit"] == SOURCE and alias["parent"] == parent["commit"] and
              new.count(declaration) == new.count(signature) == 1 and
              new.replace(declaration,b"").replace(signature,expanded) == old and
              r.sha(ROOT / "evidence/CONSTRAINTS-526.rs",deadline) == parent_inputs["source91"][constraints]["sha256"] and
              {name:item for name,item in current_inputs["source91"].items() if name != constraints} ==
              {name:item for name,item in parent_inputs["source91"].items() if name != constraints},
              "Type-only alias expansion no longer recovers the actual451 source")
    current = evidence("CURRENT-CHECKS-RESULT.json")
    current_plan = evidence("CURRENT-CHECKS-PLAN.json")
    guard = {"all_source91_extra_fixtures_original8_exact":True,"clean_exact_HEAD_index":True,
             "tools_configs_local_compiler_inventory_exact":True}
    r.require(current["commit"] == SOURCE and current["status"] == "GREEN_FMT_CLIPPY_NATIVE_CLI_BUILD" and
              current["all3_stages_passed"] is True and current["errors"] == [] and
              current["source91"] == current_inputs["source91"] and current["workspace_test_summary"] is None and
              [stage["name"] for stage in current["stages"]] == ["full-fmt","workspace-clippy","native-cli-build"] and
              [{"name":stage["name"],"argv":stage["argv"]} for stage in current["stages"]] == current_plan["commands"] and
              all(stage["exit_code"] == 0 and stage["stop_reason"] is None and
                  stage["pre_guard"] == stage["post_guard"] == guard for stage in current["stages"]) and
              current["final_guard"] == guard, "Current alias compile checks did not pass")
    for prefix, result in [("PARENT451",parent),("CURRENT-CHECKS",current)]:
        terminal = evidence(prefix + "-TERMINAL.json")
        r.require(terminal["all_direct_children_reaped"] is True and
                  terminal["result_sha256"] == r.sha(ROOT / "evidence" / (prefix + "-RESULT.json"),deadline) and
                  terminal["stage_PIDs"] == [stage["pid"] for stage in result["stages"]], "Source-check terminal differs")
    facts = read(REPO / _candidate["green_checks_evidence"])
    r.require(facts["commit"] == SOURCE and facts["workspace451_actual_source"] == parent["commit"] and
              facts["current_source_test_repeat"] is False and facts["type_only_equivalence_proved"] is True and
              facts["current_fmt_Clippy_native_CLI_build_passed"] is True, "Source prerequisite qualification changed")
    timeout = max(.001, deadline-time.monotonic()) if deadline is not None else None
    r.require(subprocess.check_output(["git","rev-parse","HEAD"],cwd=REPO,text=True,timeout=timeout).strip() == SOURCE and
              subprocess.check_output(["git","status","--porcelain"],cwd=REPO,text=True,timeout=timeout).strip() == "",
              "Current checkout changed or is dirty")
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
    volume = r.numerical_bytes(deadline=deadline)
    closed = time.monotonic()
    r.require(closed < deadline and volume <= CAP_BYTES, "Cap exceeded after final receipt persistence/guards")
    return {"status": "complete", "operational_complete": True,
            "strict_quality_passed": receipt["strict_quality_passed"],
            "final_after_receipt_monotonic_seconds": closed,
            "whole_case_elapsed_seconds": closed-receipt["started"],
            "supervisor_receipt_sha256": digest, "numerical_output_bytes": volume,
            "native_or_reference_acceptance": False, "continuation_allowed": False}
