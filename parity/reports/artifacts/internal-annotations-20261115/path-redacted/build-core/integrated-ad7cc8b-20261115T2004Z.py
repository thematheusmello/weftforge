"""Run the frozen Rust gates and build; final metadata verification is separate."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path('<WORKSPACE>')
CHECKOUT = ROOT / '.local/internal-segments-20261115'
OUT = ROOT / '.local/internal-segments-implementation-20261115'
SOURCE = 'ad7cc8bbbfc7a1ae735a400c21dcc7a37c80a5c5'
PREFIX = 'integrated-ad7cc8b-20261115T2004Z'
MANIFEST = OUT / (PREFIX + '-manifest.json')
EXTRA = [
    'spec/features/APP-001-new-empty.md',
    'spec/features/PAT-004-piece-rigid-transform.md',
    'spec/features/PAT-004-native-transform-selection.md',
    'spec/features/SEW-002-physical-length.md',
    'spec/features/PAT-006-internal-segments.md',
    'docs/architecture.md', 'docs/automation.md', 'docs/file-formats.md',
]

def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

def git(*args):
    return subprocess.check_output(['git', *args], cwd=CHECKOUT, text=True).strip()

def binding(path):
    with Path(path).open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    return {'sha256': digest, 'bytes': Path(path).stat().st_size}

assert not MANIFEST.exists(), 'Preserve every run; manifest already exists'
assert git('rev-parse', 'HEAD') == SOURCE and not git('status', '--porcelain=v1')
assert not os.environ.get('CARGO_TARGET_DIR'), 'Only this worktree default target is allowed'
paths = [p for p in git('ls-files').splitlines()
         if p.startswith(('apps/', 'crates/', 'xtask/', '.cargo/'))
         or p in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', *EXTRA]]
frozen = {p: binding(CHECKOUT / p) for p in paths}
assert frozen['spec/features/PAT-006-internal-segments.md']['sha256'] == '84014cffd5ec0c4fc3375b552d6238bd0b7dcda7a7ac5b10d7fc383621f69d66'
state = {'source': SOURCE, 'tree': git('rev-parse', 'HEAD^{tree}'),
         'checkout': str(CHECKOUT), 'cargo_target': str(CHECKOUT / 'target'),
         'runner_pid': os.getpid(), 'runner_script_binding': binding(__file__),
         'started_at': utc(), 'status': 'running', 'source_inputs': frozen,
         'source_input_count': len(frozen), 'guards': [], 'steps': [],
         'scope': 'Fresh fmt/workspace tests/strict workspace-all-target Clippy and native+CLI debug build. Required xtask verify remains a separate gate after publication metadata/source-map refresh; no full integrated/native/reference acceptance.',
         'cache_policy': 'Reuse only this worktree default target; no cross-worktree Cargo cache copy.',
         'package_or_release_run': False}

def save():
    temp = MANIFEST.with_suffix('.json.tmp')
    temp.write_text(json.dumps(state, indent=2) + '\n')
    temp.replace(MANIFEST)

def guard(label):
    changed = [p for p, expected in frozen.items() if binding(CHECKOUT / p) != expected]
    result = {'label': label, 'at': utc(), 'head': git('rev-parse', 'HEAD'),
              'git_status': git('status', '--porcelain=v1'), 'changed_source_inputs': changed}
    state['guards'].append(result)
    save()
    assert result['head'] == SOURCE and not result['git_status'] and not changed, label

print('RUNNER', os.getpid(), SOURCE, flush=True)
try:
    guard('initial')
    commands = [
        ('01-fmt', ['cargo', 'fmt', '--all', '--check']),
        ('02-workspace-tests', ['cargo', 'test', '--workspace']),
        ('03-workspace-clippy', ['cargo', 'clippy', '--workspace', '--all-targets', '--', '-D', 'warnings']),
        ('04-debug-binaries', ['cargo', 'build', '-p', 'weftforge', '-p', 'weftforge-cli']),
    ]
    for name, argv in commands:
        guard('before-' + name)
        log = OUT / (PREFIX + '-' + name + '.log')
        step = {'name': name, 'argv': argv, 'log': str(log), 'started_at': utc()}
        state['steps'].append(step)
        start = time.monotonic()
        with log.open('xb') as stream:
            child = subprocess.Popen(argv, cwd=CHECKOUT, stdout=stream, stderr=subprocess.STDOUT)
            step['child_pid'] = child.pid
            state['active_child_pid'] = child.pid
            save()
            print('START', name, child.pid, flush=True)
            code = child.wait()
        step.update(exit_code=code, ended_at=utc(), elapsed_s=time.monotonic() - start,
                    log_binding=binding(log))
        state.pop('active_child_pid', None)
        if name == '02-workspace-tests':
            rows = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', log.read_text())
            assert rows, 'Missing Rust harness summaries'
            state['rust_tests'] = {'passed': sum(int(x[1]) for x in rows),
                                   'failed': sum(int(x[2]) for x in rows),
                                   'ignored': sum(int(x[3]) for x in rows),
                                   'harnesses': len(rows), 'predicted_total_used': False}
        save()
        guard('after-' + name)
        print('END', name, code, flush=True)
        if code:
            raise RuntimeError(name + ' failed')
    state['binaries'] = {name: binding(CHECKOUT / name) for name in
                         ['target/debug/weftforge', 'target/debug/weftforge-cli']}
    guard('final')
    state['status'] = 'passed_rust_gates_and_debug_build_only'
except Exception as error:
    state['status'] = 'failed'
    state['error'] = str(error)
finally:
    state['ended_at'] = utc()
    save()
print('MANIFEST', MANIFEST, flush=True)
sys.exit(0 if state['status'] == 'passed_rust_gates_and_debug_build_only' else 1)
