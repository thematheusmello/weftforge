"""One bounded build, strict focused Clippy, and one registered control run."""
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
assert stage in ['build', 'clippy', 'run']
assert not (base / f'{stage}-job.json').exists()
if stage == 'run':
    plan = json.loads((base / 'plan.json').read_text())
    binding = json.loads((base / 'binary-binding.json').read_text())
    assert hashlib.sha256((base / 'plan.json').read_bytes()).hexdigest() == binding['plan_sha256']
    for name, digest in plan['frozen_files_sha256'].items():
        assert hashlib.sha256((base / name).read_bytes()).hexdigest() == digest, name
    for name, digest in json.loads((base / 'source-manifest.json').read_text())['files'].items():
        assert hashlib.sha256((base / 'runtime-source' / name).read_bytes()).hexdigest() == digest, name
    binary = base / 'target/release/cloth-balance-diagnostic'
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == binding['executed_binary_sha256']
    assert not (base / 'control').exists()
    assert json.loads((base / 'build-job.json').read_text())['exit_code'] == 0
    assert json.loads((base / 'clippy-job.json').read_text())['exit_code'] == 0
    command = [str(binary), str(base / 'original-grain-shirt.weftforge'), str(base / 'control')]
else:
    command = ['cargo', stage, '--locked', '--release', '--offline', '--manifest-path', str(base / 'harness/Cargo.toml')]
    if stage == 'clippy':
        command += ['--all-targets', '--', '-D', 'warnings']
environment = {'CARGO_TARGET_DIR': str(base / 'target'), 'CARGO_BUILD_JOBS': '2', 'CARGO_INCREMENTAL': '0'}
ledger = {'stage': stage, 'status': 'running', 'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'launcher_pid': os.getpid(), 'command': command, 'cwd': str(base / 'runtime-source'),
          'environment_overrides': environment, 'wall_cap_seconds': 420}
start = time.monotonic()
with (base / f'{stage}.stdout').open('xb') as stdout, (base / f'{stage}.stderr').open('xb') as stderr:
    process = subprocess.Popen(command, cwd=base / 'runtime-source', env={**os.environ, **environment}, stdout=stdout, stderr=stderr, start_new_session=True)
    ledger.update(child_pid=process.pid, child_process_group=os.getpgid(process.pid))
    (base / f'{stage}-job.json').write_text(json.dumps(ledger, indent=2) + '\n')
    print(json.dumps(ledger), flush=True)
    try:
        code = process.wait(timeout=420)
    except subprocess.TimeoutExpired:
        ledger['timed_out'] = True
        os.killpg(process.pid, signal.SIGTERM)
        try:
            code = process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            code = process.wait()
ledger.update(exit_code=code, status='completed' if code == 0 else 'failed',
              completed_at=datetime.datetime.now(datetime.timezone.utc).isoformat(), wall_seconds=time.monotonic() - start)
if stage == 'build' and code == 0:
    ledger['binary_sha256'] = hashlib.sha256((base / 'target/release/cloth-balance-diagnostic').read_bytes()).hexdigest()
(base / f'{stage}-job.json').write_text(json.dumps(ledger, indent=2) + '\n')
print(json.dumps(ledger), flush=True)
sys.exit(code)
