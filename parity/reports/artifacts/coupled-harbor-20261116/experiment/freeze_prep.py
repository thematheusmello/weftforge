"""Seal preparation after synthetic qualification; never creates a root GO."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

import common as b
from common import ROOT, REPO, r


def main():
    r.require(not (ROOT / "FREEZE.json").exists(), "Do not overwrite a frozen preparation")
    r.require(not any((ROOT / name).exists() for name in r.OUTPUT_DIRECTORIES), "A case already started")
    r.require(not (ROOT / "ROOT-GO.json").exists(), "Preparation must not create root GO")
    checks = b.read(ROOT / "checks/SELF-CHECKS.json")
    r.require(checks["status"] == "passed" and checks["Harbor_initialized"] is False and checks["Cloth_step_calls"] == 0 and checks["old_saved_frames_read_or_graded"] is False, "Synthetic-only qualification missing")
    for name, digest in checks["tested_sources"].items():
        r.require(r.sha(ROOT / name) == digest, "Code changed after synthetic qualification: " + name)
    build = b.read(ROOT / "checks/BUILD-V2-RECEIPT.json")
    r.require(build["exit_code"] == 0 and build["source88_unchanged"] is True, "Locked release build missing")
    for name, digest in build["producer_sources"].items():
        r.require(r.sha(ROOT / name) == digest, "Built producer source/lock changed")
    r.require(r.sha(REPO / build["binary"]["path"]) == build["binary"]["sha256"] == checks["producer_binary_sha256"], "Qualified release binary changed")
    source_path = ".local/combined-gates-20261116/SOURCE88.json"
    source = b.read(REPO / source_path)
    for name, digest in source["source_hashes"].items():
        r.require(r.sha(REPO / name) == digest, "Actual source changed: " + name)
    r.require(source["commit"] == b.SOURCE and len(source["source_hashes"]) == 88, "Wrong combined source")
    r.require(subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip() == b.SOURCE and
              subprocess.check_output(["git", "status", "--porcelain"], cwd=REPO, text=True).strip() == "", "Wrong/dirty source checkout")
    python = b.environment()
    b.validate_environment(python)
    binary = str(REPO / build["binary"]["path"])
    protocol = {
        "experiment": b.EXPERIMENT, "status": "prepared_not_authorized", "source_commit": b.SOURCE,
        "case": "base", "actual_dynamics_runs": 1, "init_only_initializations": 1,
        "actual_run_initializations": 1, "init_only_step_calls": 0,
        "both_initializations_charged_to_one_case_clock": True, "no_retry_reset_continuation_or_sweep": True,
        "settings": {"dt": 1/60, "substeps": 8, "iterations": 32, "seam_closure_speed_m_per_s": 1.,
                     "assembly_steps": 60, "assembly_gravity": [0., 0., 0.], "drape_steps": 120, "drape_gravity": [0., -9.81, 0.]},
        "screens": {"axis_strain_strict_below": .15, "supported_contact_strict_below_m": .002,
                    "seam_gap_strict_below_m_after_zero_target": .005, "finite_all181_states": True,
                    "capsule_uncertain_pairs": 0, "assembly_target_exact_zero_frames": [60, 180]},
        "grading": "Exact original A8 evaluator measure_frame/screen/summarize: all181 finite/strain/contact/uncertainty screens, completed-assembly seam/target screens, no disagreement/blocked diagnostic. Final-only flags do not grade the trajectory.",
        "pre_step_gate": "Init-only mode constructs actual current mesh/cloth/masses but executes zero steps. Independent metadata worker compares all original gold physical/project fields bit/type-exact after four empty v5 piece fields. Actual run freshly constructs again and compares canonical round-trip metadata against that independently approved record before first Cloth::step.",
        "caps": {"whole_case_seconds": 420., "peak_rss_bytes_each_numerical_child": 268435456, "numerical_output_bytes": 268435456},
        "numerical_output_directories": list(r.OUTPUT_DIRECTORIES), "analysis_path": "runs/base/analysis.json",
        "receipt_reserve_bytes": 2*r.RECEIPT_RESERVE, "large_analysis_is_fully_counted": True,
        "clock": "Controller starts before input copying and both initializations; every numerical phase, analysis inner work/serialization and final guards/receipt persistence is supervised under one absolute deadline. Final positive witness is controller terminal stdout+exit after its pending receipt is fsynced/hashed and an external post-persistence clock/volume check. Late negative failure appendices cannot create a successful grade.",
        "termination": "Exact own wait4 plus sampled RSS,20ms monitor, process-group SIGKILL and reap on cap/error/completion; all failed outputs retained; no continuation.",
        "cwd": str(ROOT), "python": python,
        "argv": {
            "controller": [python["binary_path"], "-B", str(ROOT/"controller.py"), "--go", "ROOT-GO.json", "--go-sha256", "<EXACT_ROOT_GO_SHA256>"],
            "init_only": [binary, "prepare", "inputs/base.weftforge", "preflight-base"],
            "original_metadata_gate": [python["binary_path"], "-B", str(ROOT/"worker.py"), "metadata", "--deadline", "<ONE_ABSOLUTE_DEADLINE>"],
            "producer": [binary, "run", "inputs/base.weftforge", "runs/base", "expected/approved-metadata.json"],
            "analysis": [python["binary_path"], "-B", str(ROOT/"worker.py"), "analyze", "--deadline", "<ONE_ABSOLUTE_DEADLINE>"],
            "finalization": [python["binary_path"], "-B", str(ROOT/"worker.py"), "finalize", "--deadline", "<ONE_ABSOLUTE_DEADLINE>"],
        },
        "exit_semantics": {"0": "operational completion and independent strict quality pass", "2": "operational completion but independent strict quality failed", "1": "operational failure/refusal; no prefix regrade"},
        "validation_only_check_mode": "Synthetic preparation only; not an actual Harbor command or authorized dynamics attempt.",
        "scope": "New headless numerical case only. No equilibrium/calibration/CCD/edge-edge/moving-avatar/native/full-garment/reference acceptance; all old cases and replay failures remain untouched.",
    }
    b.persist(ROOT / "PROTOCOL.json", protocol)
    metadata = b.read(REPO / ".local/combined-gates-20261116/METADATA-FREEZE.json")
    repo_files = dict(metadata["metadata_hashes"])
    for name in [source_path, ".local/combined-gates-20261116/RESULT.json", ".local/combined-gates-20261116/LAUNCH-ATTRIBUTION.json",
                 "parity/reports/artifacts/closed-stitch-harbor-20261115/experiment/inputs/base.weftforge",
                 "parity/reports/artifacts/closed-stitch-harbor-20261115/experiment/runs/base/rest-metadata.json"]:
        repo_files[name] = r.sha(REPO / name)
    prior = b.read(REPO / ".local/combined-gates-20261116/FINAL-MANIFEST.json")
    for name, digest in prior["proposal_artifacts"].items():
        path = ".local/next-harbor-proposal/" + name
        r.require(r.sha(REPO / path) == digest, "Frozen proposal changed: " + name)
        repo_files[path] = digest
    code_paths = sorted(str(p.relative_to(ROOT)) for directory, pattern in
                        [(ROOT, "*.py"), (ROOT/"consumer", "*.py"), (ROOT/"producer/src", "*.rs")]
                        for p in directory.glob(pattern))
    files = {str(p.relative_to(ROOT)): r.sha(p) for p in r.files_below(ROOT) if p.name != "FREEZE.json"}
    frozen = {"schema": 1, "experiment": b.EXPERIMENT, "source_commit": b.SOURCE, "status": "prepared_no_GO_no_Harbor_execution",
              "source88_path": source_path, "repo_files": repo_files, "files": files, "code_paths": code_paths,
              "python": python, "producer_binary": build["binary"],
              "input_path": "parity/reports/artifacts/closed-stitch-harbor-20261115/experiment/inputs/base.weftforge",
              "input_sha256": "fc494974955d440aacbfde49c332a50218bd3a9354d06f4fbbf0596e5da8eda7",
              "rest_sha256": "d96697949c686fbf15f19b99b6d7551b35d3449b17157009aceb75649fbaeeb9"}
    digest = b.persist(ROOT / "FREEZE.json", frozen)
    b.verify_bound()
    print(json.dumps({"status": "prepared_not_authorized", "freeze_sha256": digest, "producer_sha256": build["binary"]["sha256"],
                      "source_commit": b.SOURCE, "Harbor_initialized": False, "Cloth_step_calls": 0}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
