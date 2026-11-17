"""One explicitly root-authorized component test, with no solver edits or retries."""
import hashlib
import json
import math
import os
from pathlib import Path
import re
import stat
import subprocess
import time
from datetime import datetime, timezone

import red_resources as r

PROOF = Path(__file__).absolute().parent
ROOT = PROOF.parent.parent
WT = ROOT / '.local/material-network-regression-20261117'
PREVIOUS = ROOT / '.local/coupled-edge-regression-20261116/.local/harbor-material-groups-preparation-20261116-v2'
EXPECTED_FREEZE = 'ad4d326900deda803a76e0f42686e46048abaf21f4589dac7e1aeefdb96e0c67'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def snapshot(names):
    return {name: {'sha256': sha(WT / name), 'bytes': (WT / name).stat().st_size,
                   'mode': format(stat.S_IMODE((WT / name).stat().st_mode), '04o')}
            for name in names}


def persist(path, value):
    r.write_new(path, r.encoded(value))
    with path.open('rb') as stream:
        os.fsync(stream.fileno())


def main():
    freeze = json.loads((PROOF / 'FREEZE.json').read_bytes())
    assert sha(PROOF / 'FREEZE.json') == EXPECTED_FREEZE
    go = json.loads((PROOF / 'ROOT-INPUT-REFERENCE-REVIEW-GO.json').read_bytes())
    assert go['explicit_root_input_reference_review_and_GO'] is True and go['attempts'] == 1
    assert go['input_reference_freeze_sha256'] == EXPECTED_FREEZE
    for name, expected in freeze['files'].items():
        p = PROOF / name
        assert sha(p) == expected['sha256'] and p.stat().st_size == expected['bytes']
        assert stat.S_IMODE(p.stat().st_mode) == int(expected['mode'], 8)
    before = snapshot(freeze['source_hashes'])
    assert {p: value['sha256'] for p, value in before.items()} == freeze['source_hashes']
    assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=WT, text=True).strip() == freeze['base_commit']
    assert subprocess.check_output(['git', 'diff', '--name-only'], cwd=WT, text=True).splitlines() == ['crates/weftforge-sim/src/tests.rs']
    assert not (WT / 'target').exists(), 'Own target must be new; never copy a compiled cache'
    preflight = json.loads((PREVIOUS / 'SOURCE-PREFLIGHT.json').read_bytes())['build_binding']
    for name in ('cargo', 'rustc'):
        assert sha(preflight[name]['path']) == preflight[name]['sha256']
    env = dict(os.environ)
    for name in ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_BUILD_RUSTC_WRAPPER', 'CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER'):
        assert env.get(name) in (None, ''), 'Unexpected inherited wrapper'
        env.pop(name, None)
    env.update(CARGO_BUILD_JOBS='1', CARGO_TARGET_DIR=str(WT / 'target'),
               RUSTFLAGS='', CARGO_ENCODED_RUSTFLAGS='', RUSTC=preflight['rustc']['path'],
               CARGO_HOME=preflight['cargo_home'], PYTHONOPTIMIZE='0', PYTHONDONTWRITEBYTECODE='1')
    command = json.loads((PROOF / 'PROPOSED-RED-COMMAND.json').read_bytes())
    argv = [preflight['cargo']['path'], *command['argv'][1:]]
    directory = PROOF / 'RED'
    directory.mkdir()  # Exclusive namespace: a failed or completed attempt is never retried.
    persist(PROOF / 'RED-SOURCE-BEFORE.json', before)
    persist(PROOF / 'RED-RUNNING.json', {'status': 'ONE_AUTHORIZED_RED_STARTING',
            'at_utc': datetime.now(timezone.utc).isoformat(), 'root_GO_sha256': sha(PROOF / 'ROOT-INPUT-REFERENCE-REVIEW-GO.json'),
            'freeze_sha256': EXPECTED_FREEZE, 'argv': argv, 'cwd': str(WT),
            'environment': {k: env[k] for k in ('CARGO_BUILD_JOBS', 'CARGO_TARGET_DIR', 'RUSTC', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_HOME')},
            'compiler_binaries': {k: preflight[k] for k in ('cargo', 'rustc')},
            'runner_sha256': sha(PROOF / 'run_red.py'), 'monitor_sha256': sha(PROOF / 'red_resources.py')})
    started = time.monotonic()
    deadline = started + 120.
    count = lambda: sum(p.stat().st_size for p in r.files_below(directory))
    stage = r.monitor(argv, WT, directory / 'stdout.log', directory / 'stderr.log', deadline, count,
                      rss_cap=1073741824, output_cap=8388608, reserve=2 * r.RECEIPT_RESERVE,
                      child_env=env, on_start=lambda handle: persist(directory / 'started.json', handle))
    after = snapshot(freeze['source_hashes'])
    proofs_unchanged = all(sha(PROOF / name) == meta['sha256'] for name, meta in freeze['files'].items())
    stdout, stderr = (directory / 'stdout.log').read_text(), (directory / 'stderr.log').read_text()
    match = re.search(r'position_error=([^\s]+)m force_residual=([^\s]+)N', stdout)
    numbers = {'position_error_m': float(match[1]), 'force_residual_N': float(match[2])} if match else None
    bounded = stage['stop_reason'] is None and stage['end_monotonic_seconds'] < deadline
    if not bounded:
        classification = 'OPERATIONAL_RESOURCE_FAILURE'
    elif numbers and not all(math.isfinite(value) for value in numbers.values()):
        classification = 'NUMERICAL_NONFINITE_FAILURE'
    elif numbers and numbers['force_residual_N'] >= 1e-6:
        classification = 'RED_PHYSICAL_ENERGY_STATIONARITY_FAILURE'
    elif numbers and numbers['position_error_m'] >= 1e-8:
        classification = 'POSITION_ONLY_FAILURE_REQUIRES_BRANCH_REVIEW'
    elif stage['exit_code'] == 0 and numbers:
        classification = 'GREEN_HYPOTHESIS_NOT_SHOWN'
    elif 'could not compile' in stderr or 'error[E' in stderr:
        classification = 'COMPILE_FAILURE_NO_NUMERICAL_RED'
    else:
        classification = 'REPRESENTATION_OR_OTHER_TEST_FAILURE_NO_CLASSIFIED_FORCE_RED'
    terminal = {'status': 'terminal', 'classification': classification, 'at_utc': datetime.now(timezone.utc).isoformat(),
                'stage': stage, 'numbers': numbers, 'actual_test_executions': 1 if 'running 1 test' in stdout else 0,
                'production_Cargo_invocations': 1, 'source_before_after_exact': before == after,
                'frozen_inputs_reference_and_proofs_unchanged': proofs_unchanged,
                'source_after': after, 'stdout_sha256': sha(directory / 'stdout.log'),
                'stderr_sha256': sha(directory / 'stderr.log'), 'started': started, 'deadline': deadline,
                'elapsed_before_terminal_persistence_seconds': time.monotonic() - started,
                'followup_implementation_authorized': False, 'no_retry': True}
    persist(PROOF / 'RED-TERMINAL.json', terminal)
    final = time.monotonic()
    final_guard = {'after_terminal_fsync_hash_seconds': final - started, 'deadline': deadline,
                   'within120s_after_persistence': final < deadline, 'terminal_sha256': sha(PROOF / 'RED-TERMINAL.json'),
                   'source_proof_guards_passed': before == after and proofs_unchanged,
                   'log_bytes': count(), 'owned_child_reaped': stage['exit_code'] is not None,
                   'classification': classification}
    persist(PROOF / 'RED-FINAL-GUARD.json', final_guard)
    final_guard['after_final_guard_fsync_seconds'] = time.monotonic() - started
    assert before == after and proofs_unchanged
    print(json.dumps({'classification': classification, 'numbers': numbers,
                      'stage_exit': stage['exit_code'], 'stop_reason': stage['stop_reason'],
                      'final_guard': final_guard}, allow_nan=False), flush=True)
    return 0 if bounded and final_guard['after_final_guard_fsync_seconds'] < 120. else 1


if __name__ == '__main__':
    raise SystemExit(main())
