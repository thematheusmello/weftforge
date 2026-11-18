import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).resolve().parent
OWNER_ROOT = ROOT.parents[1]
HEAD = "7d19f86a006601133d457aee42cca3b115b994aa"
ORIGINAL = "381c58c5ddf0eff5a587412a2656662cfc62877d"
PUBLICATION = "33ea04a29b374883150d0c694353968e33e23587"
PROOF = OWNER_ROOT / ".local/rectangle-combined-source-review-20261116/ROOT-REVIEW-001.json"
PROOF_SHA = "337bc4bf191a6eff6c911fc7772013a404a536ab681c393d3ca1e9b33db0072e"
CANONICAL = OWNER_ROOT / ".local/cloth-material-integration-20261116/.local/combined-gates-20261116/SOURCE88.json"
INPUTS144 = OWNER_ROOT / ".local/endpoint-publication-20261116/clean-f79f88e/BEFORE.json"
FIELDS = ["id", "required_reference_behavior", "scenarios", "comparison_criteria", "dependencies", "implementation_status", "verification_status", "reference_observation_status"]


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


def write(name, value):
    (OUT / name).write_text(json.dumps(value, indent=2) + "\n")


def hashes(paths):
    return {path: digest((ROOT / path).read_bytes()) for path in sorted(paths)}


def scopes(registry, fields=FIELDS):
    rows = json.loads(registry)["features"]
    assert len(rows) == 56
    return [{field: row[field] for field in fields} for row in rows]


def disk():
    allocated = int(subprocess.check_output(["du", "-sk", str(ROOT / "target")], text=True).split()[0]) * 1024
    free = shutil.disk_usage(ROOT).free
    return {"target_allocated_bytes": allocated, "filesystem_free_bytes": free, "filesystem_free_gib": free / 2**30}


assert sys.argv[1:] in [[], ["fmt-only"]]
assert digest(PROOF.read_bytes()) == PROOF_SHA
proof = json.loads(PROOF.read_text())
expected_source = proof["source88"]
canonical_source = json.loads(CANONICAL.read_text())["source_hashes"]
assert len(expected_source) == 88 and set(expected_source) == set(canonical_source)
source_paths = sorted(expected_source)
source87 = set(source_paths) - {"docs/physics.md"}
assert len(source87) == 87
assert all(expected_source[path] == canonical_source[path] for path in source_paths if path != "apps/weftforge/src/main.rs")
metadata_paths = sorted(set(json.loads(INPUTS144.read_text())["inputs144"]) - source87)
top_reports = [path for path in metadata_paths if path.startswith("parity/reports/") and path.endswith(".json") and path.count("/") == 2]
assert len(metadata_paths) == 57 and len(top_reports) == 52
assert set(metadata_paths) - set(top_reports) == {"parity/features.yaml", "docs/progress.md", "docs/dashboard/index.html", "spec/inventory/weftforge-current-source-addendum.json", "spec/inventory/weftforge-current-source-addendum.md"}
inventory = git("ls-files", "spec/inventory").decode().splitlines()
addenda = {"spec/inventory/weftforge-current-source-addendum.json", "spec/inventory/weftforge-current-source-addendum.md"}
historical_paths = sorted(set(inventory) - addenda)
assert len(inventory) == 10 and len(historical_paths) == 8
historical_original = {path: digest(git("show", ORIGINAL + ":" + path)) for path in historical_paths}
registry = (ROOT / "parity/features.yaml").read_bytes()
assert registry == git("show", PUBLICATION + ":parity/features.yaml")
assert scopes(registry, FIELDS[:5]) == scopes(git("show", ORIGINAL + ":parity/features.yaml"), FIELDS[:5])
assert scopes(registry) == scopes(git("show", PUBLICATION + ":parity/features.yaml"))
tracked = set(git("ls-files", "-z").decode().split("\0"))
assert set(source_paths + metadata_paths + historical_paths) <= tracked
assert git("rev-parse", "HEAD").decode().strip() == HEAD
assert git("status", "--porcelain") == b""
assert hashes(source_paths) == expected_source
assert hashes(historical_paths) == historical_original


def freeze():
    registry_bytes = (ROOT / "parity/features.yaml").read_bytes()
    protected = scopes(registry_bytes)
    return {
        "at_utc": utc(), "head": git("rev-parse", "HEAD").decode().strip(),
        "git_status_porcelain": git("status", "--porcelain").decode(),
        "source88": hashes(source_paths), "metadata52plus5": hashes(metadata_paths),
        "original8": hashes(historical_paths), "full56_protected_fields": FIELDS,
        "full56_protected_scopes": protected,
        "full56_sha256": digest(json.dumps(protected, sort_keys=True).encode()),
        "registry_whole_bytes_sha256": digest(registry_bytes), "disk": disk(),
    }


if (OUT / "INITIAL-FREEZE.json").exists():
    initial = json.loads((OUT / "INITIAL-FREEZE.json").read_text())
else:
    initial = freeze()
    write("INITIAL-FREEZE.json", initial)
write("ROSTER-ORIGINS.json", {
    "source88": {"path": str(CANONICAL), "sha256": digest(CANONICAL.read_bytes()), "count": 88},
    "actual_reviewed_values": {"path": str(PROOF), "sha256": PROOF_SHA},
    "metadata57": {"path": str(INPUTS144), "sha256": digest(INPUTS144.read_bytes()), "selection": "inputs144 keys minus canonical source87; 52 top-level reports plus exact five named files"},
    "original8": {"excluded_only": sorted(addenda), "reference": ORIGINAL, "hashes": historical_original},
    "full56": {"five_behavior_fields_exact_vs": ORIGINAL, "all_eight_fields_exact_vs": PUBLICATION, "whole_registry_exact_vs": PUBLICATION},
    "scope": "Read-only source and identity freezes. No registry/map validation or freshness rewrite, acceptance transfer, reference/garment execution or native GUI launch.",
})
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS="1", CARGO_TARGET_DIR=str(ROOT / "target"))
environment_keys = ["CARGO_BUILD_JOBS", "CARGO_TARGET_DIR", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_INCREMENTAL", "RUSTUP_TOOLCHAIN", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_HOME"]
build_env = {key: env.get(key) for key in environment_keys}
environment = {
    "at_utc": utc(), "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
    "cargo": subprocess.check_output(["cargo", "--version"], text=True).strip(),
    "explicit_overrides": {key: env[key] for key in ["CARGO_BUILD_JOBS", "CARGO_TARGET_DIR"]},
    "build_environment": build_env, "other_environment": "Inherited; credentials and unrelated variables are not retained.",
    "target_existed_before": True, "copied_compiled_cache_or_binary": False,
    "runner_sha256": digest(Path(__file__).read_bytes()), "disk_before": initial["disk"],
    "disk_estimate": "Previous combined target about2.7GiB vs own about1.5GiB; may add about1.2GiB. Root notified before gates; no GUI launch.",
}
if not (OUT / "ENVIRONMENT.json").exists():
    write("ENVIRONMENT.json", environment)
else:
    frozen_environment = json.loads((OUT / "ENVIRONMENT.json").read_text())
    assert frozen_environment["build_environment"] == build_env
    assert frozen_environment["runner_sha256"] == environment["runner_sha256"]


def assert_frozen(value):
    assert value["head"] == HEAD
    assert value["git_status_porcelain"] == ""
    for key in ["source88", "metadata52plus5", "original8", "full56_protected_scopes", "registry_whole_bytes_sha256"]:
        assert value[key] == initial[key], key + " changed"


stages = [
    ("fmt", ["cargo", "fmt", "--all", "--check"]),
    ("workspace-tests", ["cargo", "test", "--workspace"]),
    ("workspace-clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]),
    ("native-cli-debug-build", ["cargo", "build", "-p", "weftforge", "-p", "weftforge-cli"]),
]
receipts = []
all_started = time.monotonic()
for name, argv in stages:
    receipt_path = OUT / (name + "-receipt.json")
    if receipt_path.exists():
        receipt = json.loads(receipt_path.read_text())
        assert receipt["status"] == "terminal" and receipt["exit_code"] == 0
        assert receipt["argv"] == argv
        receipts.append(receipt)
        continue
    if name != "fmt" and sys.argv[1:] == ["fmt-only"]:
        write("RESOURCE-HOLD.json", {"at_utc": utc(), "reason": "Root disk resource coordination for frozen reference R013; no further compile until explicit root resume.", "completed_receipts": receipts, "next_stage": name, "active_owned_jobs": 0})
        print(json.dumps({"status": "resource hold", "next_stage": name, "active_owned_jobs": 0}), flush=True)
        sys.exit(0)
    before = freeze()
    assert_frozen(before)
    write(name + "-BEFORE.json", before)
    begun = utc()
    stage_started = time.monotonic()
    log_path = OUT / (name + ".log")
    with log_path.open("wb") as log:
        child = subprocess.Popen(argv, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        receipt = {"stage": name, "argv": argv, "cwd": str(ROOT), "environment": build_env, "pid": child.pid, "started_at_utc": begun, "status": "running", "before_sha256": digest((OUT / (name + "-BEFORE.json")).read_bytes())}
        write("CURRENT-JOB.json", receipt)
        print(json.dumps(receipt), flush=True)
        exit_code = child.wait()
    command_seconds = time.monotonic() - stage_started
    ended = utc()
    after = freeze()
    write(name + "-AFTER.json", after)
    unchanged = all(after[key] == initial[key] for key in ["source88", "metadata52plus5", "original8", "full56_protected_scopes", "registry_whole_bytes_sha256"])
    receipt.update({"status": "terminal", "exit_code": exit_code, "finished_at_utc": ended, "command_seconds": command_seconds, "source88_unchanged": after["source88"] == initial["source88"], "metadata57_unchanged": after["metadata52plus5"] == initial["metadata52plus5"], "original8_unchanged": after["original8"] == initial["original8"], "full56_unchanged": after["full56_protected_scopes"] == initial["full56_protected_scopes"], "whole_registry_unchanged": after["registry_whole_bytes_sha256"] == initial["registry_whole_bytes_sha256"], "git_status_after": after["git_status_porcelain"], "head_after": after["head"], "after_sha256": digest((OUT / (name + "-AFTER.json")).read_bytes()), "log_sha256": digest(log_path.read_bytes()), "disk_after": after["disk"], "target_growth_bytes": after["disk"]["target_allocated_bytes"] - initial["disk"]["target_allocated_bytes"]})
    write(name + "-receipt.json", receipt)
    receipts.append(receipt)
    write("CURRENT-JOB.json", receipt)
    print(json.dumps(receipt), flush=True)
    if exit_code or not unchanged or after["head"] != HEAD or after["git_status_porcelain"]:
        write("RESULT.json", {"status": "failed", "receipts": receipts, "elapsed_seconds": time.monotonic() - all_started})
        sys.exit(exit_code or 1)

log_text = (OUT / "workspace-tests.log").read_text()
summary_rows = [{"status": status, "passed": int(passed), "failed": int(failed), "ignored": int(ignored)} for status, passed, failed, ignored in re.findall(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", log_text)]
assert summary_rows and all(row["status"] == "ok" for row in summary_rows)
binaries = {}
for name in ["weftforge", "weftforge-cli"]:
    binary = ROOT / "target/debug" / name
    binaries[name] = {"sha256": digest(binary.read_bytes()), "size_bytes": binary.stat().st_size, "launched_by_build_stage": False}
write("RESULT.json", {
    "status": "passed", "head": HEAD, "receipts": receipts, "elapsed_seconds": time.monotonic() - all_started,
    "test_summary_rows": summary_rows, "actual_test_passed_total": sum(row["passed"] for row in summary_rows),
    "actual_test_failed_total": sum(row["failed"] for row in summary_rows), "actual_test_ignored_total": sum(row["ignored"] for row in summary_rows),
    "source88_metadata57_original8_full56_unchanged": True, "native_and_cli_debug_binaries": binaries,
    "native_gui_launched": False, "cli_exercised_by_workspace_integration_tests": True,
    "xtask_or_registry_map_validation_run": False, "historical_garment_or_reference_case_run": False,
    "scope": "Integrated implementation checks only; six new Rectangle tests plus two existing tests made the earlier eight-test Rectangle filter. Not native/reference/full-feature acceptance or public delivery.",
})
print(json.dumps({"status": "passed", "actual_tests": sum(row["passed"] for row in summary_rows), "binaries": binaries}), flush=True)
