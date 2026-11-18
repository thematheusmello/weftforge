"""Fixed focused checks after the one frozen GREEN; stop at first failure."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time

EVIDENCE = Path(__file__).absolute().parent
MAIN = EVIDENCE.parent.parent
WT = MAIN.parent.parent
sys.path.insert(0, str(MAIN / "green-execution-002"))
import resources

START = time.monotonic()
DEADLINE = START + 120.
RUN = EVIDENCE / "runtime"
RUN.mkdir()
freeze = json.loads((EVIDENCE / "FREEZE.json").read_text())
source = json.loads((MAIN / "green-execution-002/GREEN-SOURCE-FREEZE.json").read_text())
env = json.loads((MAIN / "green-execution-002/green-001/COMMAND.json").read_text())["pinned_environment"]


def write(name, data):
    resources.write_new(RUN / name, resources.encoded(data))


def guard():
    resources.require(resources.sha(__file__) == freeze["controller_sha256"], "Controller changed")
    resources.require(resources.sha(MAIN / "green-execution-002/resources.py") ==
                      freeze["resources_sha256"], "Monitor changed")
    for relative, entry in source["prepared_source88"].items():
        path = WT / relative
        resources.require(resources.sha(path) == entry["sha256"] and
                          oct(path.stat().st_mode & 0o777) == entry["mode"], "Source88 changed")
    for relative, entry in source["private_payload_files"].items():
        resources.require(resources.sha(MAIN / relative) == entry["sha256"], "Frozen oracle/input changed")
    for relative, expected in source["preserved_red_artifacts"].items():
        resources.require(resources.sha(MAIN / relative) == expected, "RED artifact changed")
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=WT, text=True).strip()
    status = subprocess.check_output(["git", "status", "--porcelain=v1", "-uno"], cwd=WT, text=True)
    resources.require(head == source["base_HEAD"] and not status, "Committed source changed")
    return {"source88_and_immutable_inputs_exact": True, "clean_HEAD": head}


def size():
    return sum(path.stat().st_size for path in resources.files_below(RUN))


result = {"status": "RUNNING", "stages": [], "N32_component_reruns": 0}
write("STARTED.json", {"controller_pid": os.getpid(), "start_monotonic": START,
                       "deadline": DEADLINE, "frozen_commands": freeze["commands"]})
try:
    for key in ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTFLAGS", "RUSTDOCFLAGS",
                "CARGO_ENCODED_RUSTFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS"):
        resources.require(not os.environ.get(key), f"Unexpected foreign wrapper/flags: {key}")
    for name, argv in freeze["commands"]:
        write(name + "-PRE-GUARD.json", guard())

        def on_start(info):
            write(name + "-STARTED.json", info)
            print(json.dumps({"stage": name, "controller_pid": os.getpid(),
                              "Cargo_pid": info["pid"]}), flush=True)

        stage = resources.monitor(argv, WT, RUN / (name + ".stdout"), RUN / (name + ".stderr"),
                                  DEADLINE, size, rss_cap=1073741824, output_cap=8388608,
                                  on_start=on_start, child_env=env)
        stage["name"] = name
        result["stages"].append(stage)
        write(name + "-RECEIPT.json", stage)
        write(name + "-POST-GUARD.json", guard())
        if stage["stop_reason"] or stage["exit_code"] != 0:
            result["status"] = "FIRST_CHECK_FAILURE_RETAINED"
            break
    else:
        result["status"] = "ALL_FOCUSED_CHECKS_PASSED"
except BaseException as error:
    result.update(status="GUARD_OR_CONTROLLER_FAILURE", error=f"{type(error).__name__}:{error}")
finally:
    result["elapsed_seconds"] = time.monotonic() - START
    result["retained_output_bytes_before_receipt"] = size()
    write("RESULT.json", result)
    with (RUN / "RESULT.json").open("rb") as stream:
        os.fsync(stream.fileno())
    write("PERSISTENCE-CLOCK.json", {"elapsed_seconds": time.monotonic() - START,
                                    "within_120_seconds": time.monotonic() < DEADLINE,
                                    "RESULT_sha256": resources.sha(RUN / "RESULT.json")})
    print(json.dumps({"status": result["status"], "elapsed_seconds": time.monotonic() - START}), flush=True)
sys.exit(0 if result["status"] == "ALL_FOCUSED_CHECKS_PASSED" else 1)
