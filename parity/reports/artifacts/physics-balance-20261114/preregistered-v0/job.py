"""Exclusive local job runner retaining immutable invocation and terminal status."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

base = Path(__file__).resolve().parent
stage = sys.argv[1]
if stage == 'build':
    command = ['cargo', 'build', '--locked', '--release', '--offline', '--manifest-path', str(base / 'harness/Cargo.toml')]
    cap = 240
elif stage == 'control':
    command = [str(base / 'target/release/cloth-balance-diagnostic'), str(base / 'original-grain-shirt.weftforge'), str(base / 'control')]
    cap = 420
elif stage == 'clippy':
    command = ['cargo', 'clippy', '--locked', '--release', '--offline', '--manifest-path', str(base / 'harness/Cargo.toml'), '--all-targets', '--', '-D', 'warnings']
    cap = 240
else:
    raise ValueError(stage)
env_delta = {'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0', 'CARGO_TARGET_DIR': str(base / 'target')}
env = os.environ.copy()
env.update(env_delta)
started = time.monotonic()
ledger = {'stage': stage, 'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'launcher_pid': os.getpid(), 'command': command, 'cwd': str(base / 'source'), 'environment_overrides': env_delta, 'wall_cap_seconds': cap, 'status': 'running'}
ledger_path = base / (stage + '-job.json')
assert not ledger_path.exists(), ledger_path
with (base / (stage + '.stdout')).open('xb') as out, (base / (stage + '.stderr')).open('xb') as err:
    proc = subprocess.Popen(command, cwd=base / 'source', env=env, stdout=out, stderr=err, start_new_session=True)
    ledger.update(child_pid=proc.pid, child_process_group=os.getpgid(proc.pid))
    ledger_path.write_text(json.dumps(ledger, indent=2) + '\n')
    print(json.dumps(ledger), flush=True)
    try:
        code = proc.wait(timeout=cap)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGTERM)
        try:
            code = proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            code = proc.wait()
        ledger['timed_out'] = True
ledger.update(exit_code=code, completed_at=datetime.datetime.now(datetime.timezone.utc).isoformat(), wall_seconds=time.monotonic()-started, status='completed' if code == 0 else 'failed')
if stage == 'build' and code == 0:
    binary = base / 'target/release/cloth-balance-diagnostic'
    ledger['binary_sha256'] = hashlib.sha256(binary.read_bytes()).hexdigest()
ledger_path.write_text(json.dumps(ledger, indent=2) + '\n')
print(json.dumps(ledger), flush=True)
sys.exit(code)
