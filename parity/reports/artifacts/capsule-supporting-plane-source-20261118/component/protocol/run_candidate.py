"""One candidate: the complete sim package, including focused capsule regressions."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).absolute().parents[1]
GUARD = ROOT.parent / "<COMPONENT_GUARD>"
spec = importlib.util.spec_from_file_location("retained_guards", GUARD)
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)
COMMANDS = [
    ("sim-package", ["cargo", "test", "-p", "weftforge-sim", "--offline", "--locked", "-j", "1", "--", "--nocapture", "--test-threads=1"]),
]
OUT = ROOT / ".local/runs/candidate001"
TARGET = ROOT / ".local/cargo-target"
OUT.mkdir(parents=True)
started = time.monotonic()
deadline = started + 180.
handle = {}
peak = {"aggregate_rss_bytes": 0, "target_bytes": 0}
sources = [ROOT / "crates/weftforge-sim/src" / name for name in ("contact.rs", "capsule.rs")]
binding = {"head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "diff_sha256": hashlib.sha256(subprocess.check_output(["git", "diff", "--binary"], cwd=ROOT)).hexdigest(), "sources": {str(path.relative_to(ROOT)): r.sha(path) for path in sources}, "guard_sha256": r.sha(GUARD), "runner_sha256": r.sha(Path(__file__))}
environment = dict(os.environ, CARGO_TARGET_DIR=str(TARGET), CARGO_BUILD_JOBS="1", CARGO_INCREMENTAL="0", CARGO_NET_OFFLINE="true", CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0", PYTHONDONTWRITEBYTECODE="1", PYTHONOPTIMIZE="0")


def observe():
    r.require(shutil.disk_usage(ROOT).free >= 8_000_000_000, "8GB free disk floor reached")
    disk = r.directory_bytes([TARGET], deadline=deadline)
    peak["target_bytes"] = max(peak["target_bytes"], disk)
    r.require(disk < 8_000_000_000, "8GB own-target disk budget reached")
    if handle:
        sample = subprocess.run(["/bin/ps", "-axo", "pid=,pgid=,rss="], capture_output=True, text=True, check=True, timeout=min(1., max(.001, deadline - time.monotonic())))
        aggregate = sum(int(parts[2]) * 1024 for line in sample.stdout.splitlines() if len(parts := line.split()) == 3 and int(parts[1]) == handle["process_group"])
        peak["aggregate_rss_bytes"] = max(peak["aggregate_rss_bytes"], aggregate)
        r.require(aggregate <= 1_073_741_824, "1GiB observed process-group RSS budget reached")
    return r.directory_bytes([OUT], deadline=deadline)


stages = []
for name, argv in COMMANDS:
    directory = OUT / name
    directory.mkdir()
    handle.clear()

    def record_start(value):
        handle.update(value)
        r.write_new(directory / "started.json", r.encoded(value))

    result = r.monitor(argv, ROOT, directory / "stdout", directory / "stderr", deadline, observe, rss_cap=1_073_741_824, output_cap=8_388_608, reserve=65_536, on_start=record_start, child_env=environment)
    stages.append({"name": name, "result": result})
    r.write_new(directory / "receipt.json", r.encoded(stages[-1]))
    if result["exit_code"] != 0 or result["stop_reason"] is not None:
        break
unchanged = all(r.sha(path) == binding["sources"][str(path.relative_to(ROOT))] for path in sources)
receipt = {"binding": binding, "stages": stages, "peaks": peak, "elapsed_seconds": time.monotonic() - started, "source_unchanged": unchanged, "budgets": {"whole_seconds": 180, "observed_aggregate_rss_bytes": 1_073_741_824, "output_bytes": 8_388_608, "target_disk_bytes": 8_000_000_000, "free_disk_floor_bytes": 8_000_000_000}}
r.write_new(OUT / "receipt.json", r.encoded(receipt))
print(json.dumps(receipt, sort_keys=True))
sys.exit(0 if len(stages) == 1 and all(stage["result"]["exit_code"] == 0 and stage["result"]["stop_reason"] is None for stage in stages) and unchanged else 1)
