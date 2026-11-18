import time
START = time.monotonic()
DEADLINE = START + 120
import importlib.util, json, os, signal, subprocess, sys
from pathlib import Path

B = Path(__file__).resolve().parent
ROOT = B.parent.parent
W = ROOT / '.local/notch-bend-integration-20261117'
D = W / '.local/harbor-capsule-supporting-plane-preparation-20261118'
spec = importlib.util.spec_from_file_location('capsule_build_resources', W / '.local/capsule-constructor-clean-verify-20261118/resources.py')
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)
read = lambda p: json.loads(p.read_text())
GO = 'dd6363ef5095b720a422ac14bd0ec3eff13e21db0a216226f4a03b0391484bdd'

def put(name, value):
    path = B / name
    r.write_new(path, r.encoded(value))
    with path.open('rb') as stream:
        os.fsync(stream.fileno())

def volume():
    return sum(p.stat().st_size for folder in [B, D / 'checks'] if folder.exists() for p in r.files_below(folder))

def guard():
    r.require(subprocess.check_output(['/usr/bin/git', 'rev-parse', 'HEAD'], cwd=W).decode().strip() == 'e30a71a5ffbaef9de00e1d4b10a6fa51a28ff621', 'Wrong G source')
    r.require(subprocess.check_output(['/usr/bin/git', 'status', '--porcelain'], cwd=W) == b'', 'Source/index changed')
    r.require(r.sha(D / 'SOURCE-PREFLIGHT.json', DEADLINE) == '9392e32bb6ecfeb3434b43b026550cc79c59d60f65794c99be08c460eeeae79d', 'Source preflight changed')
    r.require(r.sha(D / 'checks/ROOT-BUILD-GO.json', DEADLINE) == GO, 'Build GO changed')
    for name, entry in tested.items():
        path = W / name
        r.require(r.sha(path, DEADLINE) == entry['sha256'] and path.stat().st_mode & 0o777 == entry['mode'], 'Actual F source changed: ' + name)
    r.deadline_check(DEADLINE)

case = read(W / '.local/capsule-supporting-plane-integrated-source-20261118/full-001.json')
tested = case['source91']
env = dict(os.environ)
env.update(case['child_env'])
stage = None
errors = []
cleanup = []
put('STARTED.json', {'pid': os.getpid(), 'pgid': os.getpgrp(), 'started': START, 'deadline': DEADLINE, 'source_commit': 'e30a71a5ffbaef9de00e1d4b10a6fa51a28ff621', 'build_GO_sha256': GO, 'only_producer_build': True})
try:
    guard()
    r.require(not (D / 'checks/build').exists() and not (D / 'checks/BUILD-RECEIPT.json').exists(), 'Producer build already attempted')
    retained = W / '.local/harbor-contact-recovery-producer-binary-20261118/a073b00d4121a3de3d3096b6f03bae3494c43f628889bcab8f9ebe0be7cfc62b'
    r.require(r.sha(retained, DEADLINE) == 'a073b00d4121a3de3d3096b6f03bae3494c43f628889bcab8f9ebe0be7cfc62b', 'Original same-worktree producer retention missing')
    argv = [sys.executable, '-B', str(D / 'build_release.py'), '--go', 'checks/ROOT-BUILD-GO.json', '--go-sha256', GO]
    stage = r.monitor(argv, W, B / 'stdout', B / 'stderr', DEADLINE - 5, volume, rss_cap=1073741824, output_cap=8388608, reserve=262144, child_env=env, disk_floor=8000000000, on_start=lambda value: put('CHILD-STARTED.json', value))
    r.require(stage['exit_code'] == 0 and stage['stop_reason'] is None, 'Build helper failed or stopped; no retry')
    guard()
except BaseException as exc:
    errors.append(type(exc).__name__ + ': ' + str(exc))
ledger = D / 'checks/build/started.json'
if ledger.exists():
    group = read(ledger)['process_group']
    try:
        os.killpg(group, 0)
    except ProcessLookupError:
        cleanup.append({'group': group, 'absent': True, 'signal_sent': False})
    else:
        os.killpg(group, signal.SIGKILL)
        cleanup.append({'group': group, 'absent': False, 'signal_sent': True})
        errors.append('Nested owned Cargo group remained after helper; killed only that recorded group')
receipt_path = D / 'checks/BUILD-RECEIPT.json'
receipt = read(receipt_path) if receipt_path.exists() else None
if receipt is None or receipt.get('Cloth_initializations') != 0 or receipt.get('Cloth_steps') != 0 or 'binary' not in receipt:
    errors.append('Successful producer build receipt with zero constructors/steps missing')
put('RESULT.json', {'stage': stage, 'errors': errors, 'producer_build_receipt_sha256': r.sha(receipt_path) if receipt else None, 'producer': receipt.get('binary') if receipt else None, 'owned_Cargo_cleanup': cleanup, 'Cloth_initializations': 0, 'Cloth_steps': 0, 'runtime_GO_or_final_freeze_created': False})
put('TERMINAL.json', {'result_sha256': r.sha(B / 'RESULT.json'), 'direct_helper_reaped': stage is not None and stage['exit_code'] is not None})
elapsed = time.monotonic() - START
available = os.statvfs(W).f_bavail * os.statvfs(W).f_frsize
accepted = not errors and elapsed < 120 and volume() < 8388608 and available >= 8000000000
print(json.dumps({'accepted': accepted, 'elapsed_seconds_through_prior_persistence': elapsed, 'cap_seconds': 120, 'terminal_sha256': r.sha(B / 'TERMINAL.json'), 'counted_bytes': volume(), 'available_bytes': available, 'producer': receipt.get('binary') if receipt else None, 'errors': errors}), flush=True)
raise SystemExit(0 if accepted else 1)
