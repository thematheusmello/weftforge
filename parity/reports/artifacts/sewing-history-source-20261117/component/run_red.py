"""One authorized frozen component command; no retry or source mutation."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

if sys.flags.optimize != 0 or os.environ.get("PYTHONOPTIMIZE") not in (None, "0"):
    raise RuntimeError("Require unoptimized Python and absent/literal0 PYTHONOPTIMIZE")

import resources

PACKET = Path(__file__).absolute().parent
WT = PACKET.parent.parent
RUN = PACKET / "red-001"
START = time.monotonic()
DEADLINE = START + 120.
GO_SHA = "6aec6f033a522a8d2015d9d6a092dc8dc0bece9fe01f99eac5a2b10cc9c8d7cb"
SOURCE_FREEZE_SHA = "d86c34a9c40816e514a76f8906a3db7b8c19eb4534ca5da82eda5f8ca5a45351"


def write(name, value):
    resources.write_new(RUN / name, resources.encoded(value))


def guards():
    resources.require(resources.sha(PACKET / "ROOT-GO.json") == GO_SHA, "ROOT GO changed")
    resources.require(resources.sha(PACKET / "SOURCE-ONLY-FREEZE.json") == SOURCE_FREEZE_SHA,
                      "Source-only freeze changed")
    frozen = json.loads((PACKET / "SOURCE-ONLY-FREEZE.json").read_text())
    for relative, entry in frozen["prepared_source88"].items():
        path = WT / relative
        resources.require(resources.sha(path) == entry["sha256"], f"Source changed: {relative}")
        resources.require(oct(path.stat().st_mode & 0o777) == entry["mode"], "Source mode changed")
    for relative, entry in frozen["private_payload_files"].items():
        path = PACKET / relative
        resources.require(resources.sha(path) == entry["sha256"], f"Fixture/oracle changed: {relative}")
        resources.require(oct(path.stat().st_mode & 0o777) == entry["mode"], "Input mode changed")
    execution = json.loads((PACKET / "EXECUTION-FREEZE.json").read_text())
    for relative, entry in execution["added_control_files"].items():
        resources.require(resources.sha(PACKET / relative) == entry["sha256"], "Controller changed")
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=WT, text=True).strip()
    status = subprocess.check_output(["git", "status", "--porcelain=v1", "-uno"], cwd=WT, text=True)
    resources.require(head == frozen["base_HEAD"] and status.splitlines() == frozen["tracked_status"],
                      "Unexpected tracked source change")
    return {"source88_exact": True, "test_fixture_oracle_GO_and_controllers_exact": True,
            "HEAD": head, "tracked_status": status.splitlines(),
            "source_only_freeze_sha256": SOURCE_FREEZE_SHA, "ROOT_GO_sha256": GO_SHA}


def output_bytes():
    return sum(path.stat().st_size for path in resources.files_below(RUN))


RUN.mkdir()
write("STARTED.json", {"controller_pid": os.getpid(), "start_monotonic": START,
                       "deadline_monotonic": DEADLINE, "attempt": 1})
result = {"attempt": 1, "Cargo_invocations": 0, "production_edits": 0,
          "Cloth_initializations": 0, "garment_or_old_case_runs_or_grades": 0}
try:
    plan = json.loads((PACKET / "PLAN.json").read_text())
    go = json.loads((PACKET / "ROOT-GO.json").read_text())
    resources.require(go["argv"] == plan["prospective_invocation"] and
                      go["environment"] == plan["prospective_environment"] and
                      go["attempts"] == 1 and go["sweeps"] == 32, "GO/plan mismatch")
    refused = []
    for key, value in os.environ.items():
        foreign_flag = key in ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTFLAGS",
                               "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS")
        foreign_flag |= key.startswith("CARGO_TARGET_") and key.endswith("_RUSTFLAGS")
        if foreign_flag and value:
            refused.append(key)
    resources.require(not refused, f"Unexpected nonempty compiler wrapper/flags: {refused}")
    child_env = {key: os.environ[key] for key in
                 ("HOME", "CARGO_HOME", "RUSTUP_HOME", "TMPDIR", "LANG", "LC_ALL", "USER", "LOGNAME")
                 if key in os.environ}
    child_env.update(go["environment"])
    child_env.update(PYTHONDONTWRITEBYTECODE="1", PYTHONOPTIMIZE="0")
    tools = {name: {"path": path, "sha256": resources.sha(path)} for name, path in
             (("cargo", go["argv"][0]), ("rustc", child_env["RUSTC"]), ("rustdoc", child_env["RUSTDOC"]))}
    write("PRE-GUARD.json", guards())
    resources.require(not (WT / "target").exists(), "Own target must initially be cold")
    write("COMMAND.json", {"argv": go["argv"], "cwd": str(WT), "pinned_environment": child_env,
                           "compiler_tools_hashed_not_invoked_for_probing": tools,
                           "wrappers_and_foreign_flags_refused": True, "initial_target_exists": False,
                           "caps": go["caps"], "caller_correction": go["note"]})

    def started(info):
        write("CHILD-STARTED.json", info)
        print(json.dumps({"status": "component_running", "controller_pid": os.getpid(),
                          "Cargo_pid": info["pid"], "process_group": info["process_group"]}), flush=True)

    result["Cargo_invocations"] = 1
    stage = resources.monitor(go["argv"], WT, RUN / "stdout.txt", RUN / "stderr.txt",
                              DEADLINE, output_bytes, rss_cap=1073741824, output_cap=8388608,
                              on_start=started, child_env=child_env)
    result["stage"] = stage
    write("POST-GUARD.json", guards())
    result["post_source88_test_fixture_oracle_GO_exact"] = True
    stdout = (RUN / "stdout.txt").read_text()
    result["raw_test_summaries"] = re.findall(r"^test result: .*", stdout, flags=re.MULTILINE)
    result["test_assertion_failure_observed"] = "0 passed; 1 failed;" in stdout
    result["status"] = ("RESOURCE_OR_MONITOR_FAILURE" if stage["stop_reason"] else
                        "COMPONENT_TEST_PASSED" if stage["exit_code"] == 0 else
                        "COMPONENT_TEST_FAILED" if result["test_assertion_failure_observed"] else
                        "COMPILE_OR_COMMAND_FAILURE")
except BaseException as error:
    result.update(status="CONTROLLER_OR_GUARD_FAILURE", error=f"{type(error).__name__}:{error}")
finally:
    result["end_monotonic_before_receipt"] = time.monotonic()
    result["elapsed_seconds_before_receipt"] = result["end_monotonic_before_receipt"] - START
    result["retained_output_bytes_before_receipt"] = output_bytes()
    if time.monotonic() >= DEADLINE:
        result["status"] = "WHOLE_COMMAND_DEADLINE_FAILURE"
    if output_bytes() + 65536 >= 8388608:
        result["status"] = "OUTPUT_BUDGET_FAILURE"
    write("RESULT.json", result)
    with (RUN / "RESULT.json").open("rb") as stream:
        os.fsync(stream.fileno())
    persisted = time.monotonic()
    write("PERSISTENCE-CLOCK.json", {"result_sha256": resources.sha(RUN / "RESULT.json"),
                                    "result_persisted_monotonic": persisted,
                                    "elapsed_through_result_fsync_seconds": persisted - START,
                                    "within_120_seconds": persisted < DEADLINE,
                                    "retained_output_bytes": output_bytes(),
                                    "status": result["status"]})
    print(json.dumps({"status": result["status"], "elapsed_seconds": persisted - START,
                      "result": str(RUN / "RESULT.json")}), flush=True)
sys.exit(0 if result["status"] == "COMPONENT_TEST_PASSED" else 1)
