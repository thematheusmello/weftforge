import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path('<WORKSPACE>')
CHECKOUT = ROOT / '.local/pattern-transform-20261115'
OUT = ROOT / '.local/pattern-transform-implementation-20261115'
PREFIX = 'integrated-133c6d3-20261115T1651Z'
MANIFEST = OUT / (PREFIX + '-manifest.json')
SOURCE = '133c6d3fd91601b897f584e03c92765f79f97993'
ACCEPTED = ROOT / '.local/transform-independent-evaluation-20261115/candidate/weftforge-cli'
CONTRACTS = {
    'spec/features/APP-001-new-empty.md': 'ce6de41684f98772c69bf8e0117e22f5f2cdf6b15952e4394f6756dd0077deb7',
    'spec/features/PAT-004-piece-rigid-transform.md': 'afffdf4e68ca5afb7f0038d2568576feb66824f873a8aaa659f60bfc072bf557',
    'spec/features/PAT-004-native-transform-selection.md': 'b5cb53682e0b2da486393852bc84e9edb2ba9a6c7d6e1fa16763ed4fffde19c7',
}

def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

def run_output(*args):
    return subprocess.check_output(args, cwd=CHECKOUT, text=True).strip()

def binding(path):
    path = Path(path)
    digest = hashlib.sha256()
    with path.open('rb') as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b''):
            digest.update(block)
    return {'sha256': digest.hexdigest(), 'bytes': path.stat().st_size}

def optional_binding(relative):
    path = CHECKOUT / relative
    return binding(path) if path.is_file() else None

if MANIFEST.exists():
    raise SystemExit('Refusing to replace an existing integrated manifest')
tracked = run_output('git', 'ls-files').splitlines()
scope = [name for name in tracked if name.startswith(('apps/', 'crates/', 'xtask/', '.cargo/')) or name in {'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml'} or name in CONTRACTS]
frozen = {name: binding(CHECKOUT / name) for name in scope}
accepted = binding(ACCEPTED)
state = {
    'source': SOURCE,
    'tree': run_output('git', 'rev-parse', 'HEAD^{tree}'),
    'checkout': str(CHECKOUT),
    'cargo_target': str(CHECKOUT / 'target'),
    'started_at': utc(),
    'runner_pid': os.getpid(),
    'status': 'running',
    'completion_scope': 'Fresh fmt/workspace tests/workspace-all-target Clippy/xtask verify/package/debug run on clean133c6d3; no prior-gate component reuse.',
    'runner_script_binding': binding(__file__),
    'runtime_and_contract_files': frozen,
    'reviewed_contracts': CONTRACTS,
    'independent_evaluation_cli': {'path': str(ACCEPTED), **accepted},
    'toolchain': {'cargo': run_output('cargo', '--version'), 'rustc': run_output('rustc', '--version')},
    'guards': [],
    'steps': [],
    'notes': [
        'The accepted independent-evaluation CLI artifact is read-only for this run and is never replaced or relabeled.',
        'Cargo feature contexts may produce different CLI executables; each gate records current CLI identities.',
        'Packaging is unsigned and all outputs remain local pending root review and tracked publication.',
    ],
}

def save():
    temporary = MANIFEST.with_suffix('.json.tmp')
    temporary.write_text(json.dumps(state, indent=2) + '\n')
    temporary.replace(MANIFEST)

def guard(label):
    head = run_output('git', 'rev-parse', 'HEAD')
    status = run_output('git', 'status', '--porcelain=v1')
    changed = [name for name, expected in frozen.items() if binding(CHECKOUT / name) != expected]
    independent_unchanged = binding(ACCEPTED) == accepted
    result = {'label': label, 'at': utc(), 'source': head, 'git_status': status, 'changed_frozen_files': changed, 'independent_cli_unchanged': independent_unchanged}
    state['guards'].append(result)
    save()
    if head != SOURCE or status or changed or not independent_unchanged:
        raise RuntimeError('Source/clean/independent-artifact guard failed: ' + label)

try:
    if os.environ.get('CARGO_TARGET_DIR'):
        raise RuntimeError('Unexpected external CARGO_TARGET_DIR')
    for name, expected in CONTRACTS.items():
        if frozen[name]['sha256'] != expected:
            raise RuntimeError('Reviewed contract hash mismatch: ' + name)
    guard('initial-clean-frozen-source')
    commands = [
        ('01-fmt', ['cargo', 'fmt', '--all', '--check']),
        ('02-workspace-tests', ['cargo', 'test', '--workspace']),
        ('03-workspace-clippy', ['cargo', 'clippy', '--workspace', '--all-targets', '--', '-D', 'warnings']),
        ('04-xtask-verify', ['cargo', 'xtask', 'verify']),
        ('05-xtask-package', ['cargo', 'xtask', 'package']),
        ('06-debug-native', ['cargo', 'build', '-p', 'weftforge']),
    ]
    for name, argv in commands:
        guard('before-' + name)
        log_path = OUT / (PREFIX + '-' + name + '.log')
        step = {'name': name, 'command': argv, 'log': str(log_path), 'started_at': utc(), 'status': 'running', 'cli_before': optional_binding('target/debug/weftforge-cli')}
        state['steps'].append(step)
        print('START ' + name + ' ' + utc(), flush=True)
        started = time.monotonic()
        with log_path.open('xb') as handle:
            child = subprocess.Popen(argv, cwd=CHECKOUT, stdout=handle, stderr=subprocess.STDOUT)
            step['child_pid'] = child.pid
            state['active_child_pid'] = child.pid
            save()
            code = child.wait()
        step.update({'exit_code': code, 'status': 'passed' if code == 0 else 'failed', 'ended_at': utc(), 'duration_seconds': time.monotonic() - started, 'log_binding': binding(log_path), 'cli_after': optional_binding('target/debug/weftforge-cli'), 'release_cli_after': optional_binding('target/release/weftforge-cli')})
        state.pop('active_child_pid', None)
        save()
        guard('after-' + name)
        print('END ' + name + ' exit=' + str(code) + ' ' + utc(), flush=True)
        if code:
            raise RuntimeError(name + ' failed')
    package = CHECKOUT / 'target/package'
    state['package_artifacts'] = {str(path.relative_to(CHECKOUT)): binding(path) for path in sorted(package.rglob('*')) if path.is_file()}
    state['final_binaries'] = {name: optional_binding(name) for name in ['target/debug/weftforge', 'target/debug/weftforge-cli', 'target/release/weftforge', 'target/release/weftforge-cli', 'target/package/weftforge', 'target/package/weftforge-cli', 'target/package/WeftForge.app/Contents/MacOS/weftforge']}
    state['font_notice_comparisons'] = []
    for filename in ['OFL-Inter.txt', 'OFL-JetBrainsMono.txt']:
        source = binding(CHECKOUT / 'apps/weftforge/assets/fonts' / filename)
        regular = binding(package / 'licenses' / filename)
        app = binding(package / 'WeftForge.app/Contents/Resources/licenses' / filename)
        state['font_notice_comparisons'].append({'name': filename, 'source': source, 'regular': regular, 'app': app, 'equal': source == regular == app})
        if source != regular or source != app:
            raise RuntimeError('Packaged font notice mismatch: ' + filename)
    native = state['final_binaries']['target/release/weftforge']
    if native != state['final_binaries']['target/package/weftforge'] or native != state['final_binaries']['target/package/WeftForge.app/Contents/MacOS/weftforge']:
        raise RuntimeError('Native release/package executable mismatch')
    if state['final_binaries']['target/release/weftforge-cli'] != state['final_binaries']['target/package/weftforge-cli']:
        raise RuntimeError('CLI release/package executable mismatch')
    guard('final-clean-frozen-source')
    state['status'] = 'passed'
except Exception as error:
    state['status'] = 'failed'
    state['error'] = str(error)
    print('FAILED ' + str(error), flush=True)
finally:
    state['ended_at'] = utc()
    state['independent_evaluation_cli_after'] = binding(ACCEPTED)
    save()
print('MANIFEST ' + str(MANIFEST), flush=True)
sys.exit(0 if state['status'] == 'passed' else 1)
