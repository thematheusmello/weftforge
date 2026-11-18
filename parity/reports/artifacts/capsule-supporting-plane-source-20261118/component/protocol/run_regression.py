"""One root-reviewed targeted run, reusing the retained process-group monitor."""
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
NAME = "contact::tests::zero_target_stitch_capsule_n32_analytic_coupling_diagnostic"
ARGV = ["cargo", "test", "-p", "weftforge-sim", "--lib", NAME, "--offline", "--locked", "-j", "1", "--", "--exact", "--ignored", "--nocapture", "--test-threads=1"]
OUT = ROOT / ".local/runs/baseline001"
TARGET = ROOT / ".local/cargo-target"
OUT.mkdir(parents=True)
started = time.monotonic()
deadline = started + 180.
handle = {}
peak = {"aggregate_rss_bytes": 0, "target_bytes": 0}


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


def record_start(value):
    handle.update(value)
    r.write_new(OUT / "started.json", r.encoded(value))


source = ROOT / "crates/weftforge-sim/src/contact.rs"
binding = {"head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "diff_sha256": hashlib.sha256(subprocess.check_output(["git", "diff", "--binary"], cwd=ROOT)).hexdigest(), "test_source_sha256": r.sha(source), "guard_sha256": r.sha(GUARD), "runner_sha256": r.sha(Path(__file__))}
environment = dict(os.environ, CARGO_TARGET_DIR=str(TARGET), CARGO_BUILD_JOBS="1", CARGO_INCREMENTAL="0", CARGO_NET_OFFLINE="true", CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0", PYTHONDONTWRITEBYTECODE="1", PYTHONOPTIMIZE="0")
result = r.monitor(ARGV, ROOT, OUT / "stdout", OUT / "stderr", deadline, observe, rss_cap=1_073_741_824, output_cap=8_388_608, reserve=65_536, on_start=record_start, child_env=environment)
receipt = {"binding": binding, "stage": result, "peaks": peak, "elapsed_seconds": time.monotonic() - started, "diagnostic_pass_is_not_coupled_convergence_pass": True, "source_unchanged": r.sha(source) == binding["test_source_sha256"], "budgets": {"whole_seconds": 180, "observed_aggregate_rss_bytes": 1_073_741_824, "output_bytes": 8_388_608, "target_disk_bytes": 8_000_000_000}}
r.write_new(OUT / "receipt.json", r.encoded(receipt))
print(json.dumps(receipt, sort_keys=True))
sys.exit(0 if result["exit_code"] == 0 and result["stop_reason"] is None and receipt["source_unchanged"] else 1)
