import time
START = time.monotonic()
import hashlib, importlib.util, json, os, subprocess, sys
from pathlib import Path

B = Path(__file__).resolve().parent
ROOT = B.parent.parent
W = ROOT / '.local/notch-bend-integration-20261117'
P = ROOT / '.local/capsule-current-bind-prep-20261118'
D = W / '.local/harbor-capsule-supporting-plane-preparation-20261118'
DEADLINE = START + 120
spec = importlib.util.spec_from_file_location('capsule_install_resources', W / '.local/capsule-constructor-clean-verify-20261118/resources.py')
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)

def put(name, value):
    path = B / name
    r.write_new(path, r.encoded(value))
    with path.open('rb') as stream:
        os.fsync(stream.fileno())

def volume():
    return sum(p.stat().st_size for folder in [B, D] if folder.exists() for p in r.files_below(folder))

def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args], cwd=W).decode().strip()

approval_sha = '2299c1c4ce07d1534e45439838afc85498a5b11471388ae0b7ca5d7a433eaa69'
helper_sha = '67d5176e4805c9a94e7c378147a2ca461ab7c047c736603ea25e6d59608dac26'
controller_sha = r.sha(Path(__file__))
plan = json.loads((W / '.local/capsule-supporting-plane-integrated-source-20261118/full-001.json').read_text())
env = dict(os.environ)
env.update(plan['child_env'])
stages = []
error = None
put('STARTED.json', {'pid': os.getpid(), 'pgid': os.getpgrp(), 'start': START, 'deadline': DEADLINE, 'controller_sha256': controller_sha, 'approval_sha256': approval_sha, 'helper_sha256': helper_sha, 'scope': 'Installation and source bind only; no Cargo/build/model/GUI.'})
try:
    r.require(r.sha(B / 'ROOT-APPROVAL.json') == approval_sha and r.sha(P / 'install_source.py') == helper_sha, 'Approved inputs changed')
    r.require(git('rev-parse', 'HEAD') == 'e30a71a5ffbaef9de00e1d4b10a6fa51a28ff621' and git('status', '--porcelain') == '', 'Current G is not clean')
    commands = [('install', [sys.executable, '-B', str(P / 'install_source.py'), '--root-approval', str(B / 'ROOT-APPROVAL.json'), '--root-approval-sha256', approval_sha]), ('bind', [sys.executable, '-B', str(D / 'bind_source.py')])]
    for name, argv in commands:
        if name == 'bind':
            candidate_path = D / 'CANDIDATE.json'
            before = r.sha(candidate_path)
            candidate = json.loads(candidate_path.read_text())
            release = W / '.local/capsule-supporting-plane-release-build-closure-20261118/ROOT-CLOSURE.json'
            r.require(r.sha(release) == '69e0b10a1465c7f4d58146597413a40b2fdb9ce502169b510a0c9771ef86a315' and json.loads(release.read_text())['accepted'] is True, 'Current optimized build missing')
            candidate['optimized_native_cli_build_receipt'] = {'path': str(release.relative_to(W)), 'sha256': r.sha(release)}
            candidate['evidence_hashes'][str(release.relative_to(W))] = r.sha(release)
            candidate['workspace_gates_status_at_binding'] = 'Actual F fmt/workspace480/Clippy/debug builds passed; actual clean G xtask and optimized native+CLI builds passed. F execution does not transfer. Producer build, source bind and runtime GO remain pending.'
            candidate_path.write_bytes(r.encoded(candidate))
            put('OPTIMIZED-BUILD-METADATA-BINDING.json', {'original_installer_candidate_sha256': before, 'current_candidate_sha256': r.sha(candidate_path), 'optimized_release_closure_sha256': r.sha(release), 'qualification': 'Real optimized G result filled after source-only installation and before first source binding. Frozen installer receipt retains its earlier candidate identity. No program, physics or source91 changed.'})
            (D / 'checks/source-bind').mkdir(parents=True)
        stage = r.monitor(argv, W, B / (name + '.stdout'), B / (name + '.stderr'), DEADLINE - 5, volume, rss_cap=1073741824, output_cap=8388608, reserve=262144, child_env=env, on_start=lambda value, n=name: put(n + '-STARTED.json', value), disk_floor=8000000000)
        stages.append(stage)
        put(name + '-RESULT.json', stage)
        r.require(stage['exit_code'] == 0 and stage['stop_reason'] is None, name + ' failed or stopped; no retry')
    r.require(git('rev-parse', 'HEAD') == 'e30a71a5ffbaef9de00e1d4b10a6fa51a28ff621' and git('status', '--porcelain') == '', 'Tracked source changed')
    r.require(r.sha(B / 'ROOT-APPROVAL.json') == approval_sha and r.sha(P / 'install_source.py') == helper_sha and r.sha(Path(__file__)) == controller_sha, 'Controller/approved inputs changed')
except BaseException as exc:
    error = type(exc).__name__ + ': ' + str(exc)
put('RESULT.json', {'stages': stages, 'error': error, 'source_commit': 'e30a71a5ffbaef9de00e1d4b10a6fa51a28ff621', 'actual_workspace_execution_commit': 'c766f3fa48c03ac698da8b22cc3bf927ee71f195', 'source_preflight_sha256': r.sha(D / 'SOURCE-PREFLIGHT.json') if (D / 'SOURCE-PREFLIGHT.json').is_file() else None, 'builds': 0, 'model_initializations': 0, 'runtime_GO_created': False})
put('TERMINAL.json', {'result_sha256': r.sha(B / 'RESULT.json'), 'all_started_direct_children_reaped': all(s['exit_code'] is not None for s in stages)})
elapsed = time.monotonic() - START
available = os.statvfs(W).f_bavail * os.statvfs(W).f_frsize
accepted = error is None and len(stages) == 2 and elapsed < 120 and volume() < 8388608 and available >= 8000000000
print(json.dumps({'accepted': accepted, 'elapsed_seconds_through_prior_persistence': elapsed, 'cap_seconds': 120, 'terminal_sha256': r.sha(B / 'TERMINAL.json'), 'counted_bytes': volume(), 'available_bytes': available, 'error': error}), flush=True)
raise SystemExit(0 if accepted else 1)
