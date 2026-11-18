"""One 60 s / 256 MiB observed RSS / 8 MiB focused V6 source-only case."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

import resources

ROOT = Path(__file__).resolve().parent
WORKSPACE = ROOT.parent.parent
OUT = ROOT / 'stub-check-001'
PYTHON = '<PYENV_HOME>/versions/3.13.0/bin/python3.13'
SOURCES = ['preview_launch.py', 'preview_sequence.js', 'preview_launch.js', 'preview_owner.py', 'native_call.js',
           'clock.py', 'cleanup.py', 'call_recorder.py', 'resource_observer.py', 'inventory.js', 'resources.py',
           'focused_selfcheck.py', 'focused_selfcheck.cjs', 'check_run.py', 'README.md', 'INT-PIN.json']


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    raw = (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + '\n').encode()
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'wb') as stream:
        stream.write(raw); stream.flush(); os.fsync(stream.fileno())


def volume():
    total = 0
    for path in OUT.rglob('*'):
        try:
            if path.is_file() and not path.is_symlink():
                total += path.stat().st_size
        except FileNotFoundError:
            pass
    return total


def free():
    disk = os.statvfs(ROOT)
    return disk.f_bavail * disk.f_frsize


def integration_snapshot(pin):
    def git(*args):
        return subprocess.run(['git', '--no-optional-locks', '-C', pin['integration_root'], *args],
                              capture_output=True, text=True, check=True, timeout=5).stdout.strip()
    index = Path(pin['index_path'])
    return {'HEAD': git('rev-parse', 'HEAD'), 'tracked_status': git('status', '--porcelain=v1', '--untracked-files=no'),
            'index_sha256': sha(index), 'index_bytes': index.stat().st_size}


def main():
    start = time.monotonic(); deadline = start + 60
    OUT.mkdir()
    sources = {name: {'sha256': sha(ROOT / name), 'bytes': (ROOT / name).stat().st_size} for name in SOURCES}
    reference = {}
    previous = WORKSPACE / '.local/native-constructor-preview-v5-20261118'
    reference[previous / 'FREEZE.json'] = '7c1f8df869968ef23f57eb9b0b9682b302cee033a712f6fe8e6d818f585680bc'
    frozen = json.loads((previous / 'FREEZE.json').read_text())
    reference.update({previous / name: entry['sha256'] for name, entry in frozen['files'].items()})
    old_manifest = json.loads((previous / frozen['recursive_case_artifacts']['path']).read_text())
    reference.update({previous / 'stub-check-002' / name: entry['sha256'] for name, entry in old_manifest.items()})
    review = WORKSPACE / '.local/native-constructor-preview-v5-root-review-20261118'
    for name, digest in {'FREEZE.json': '38c6aaf104c45133770df298132e3c1f2355cef11a32855f37103e878ea6b54a',
                         'REVIEW.md': 'df6c8da1d24db740dd762f61edf233447a2e7f107fc8d8923f3bc46515215719',
                         'RECEIPT.json': 'e61ee5cb39475819a5001e10dca7650da9fe6d4fdc78704071878bcc12ac6735'}.items():
        reference[review / name] = digest
    integration = WORKSPACE / '.local/notch-bend-integration-20261117'
    closure = integration / '.local/native-current-P004-closure-20261118'
    reference[closure / 'ROOT-OUTCOME.json'] = 'a8a4d1e92e6b210f1829e1dd38f8d63ac717b2f74b37b90b3f367898c73a57e7'
    reference[closure / 'CASE-FREEZE.json'] = 'a5760a4ac0e3861506750b00e6946786721432b7d3bd6276e08c790a9fc73511'
    case = integration / '.local/native-constructor-current-P004-20261118'
    frozen_case = json.loads((closure / 'CASE-FREEZE.json').read_text())
    resources.require(set(frozen_case) == {'case', 'files', 'kind'} and
                      frozen_case['case'] == 'WF-20261118-NATIVE-CONSTRUCTOR-VISUAL-P004-001' and
                      isinstance(frozen_case['files'], dict) and len(frozen_case['files']) == 92, 'Unexpected wrapped P004 freeze')
    reference.update({case / name: entry['sha256'] for name, entry in frozen_case['files'].items()})
    pin = json.loads((ROOT / 'INT-PIN.json').read_text())
    expected = {key: pin[key] for key in ['HEAD', 'tracked_status', 'index_sha256', 'index_bytes']}
    before = integration_snapshot(pin)
    write(OUT / 'PREFLIGHT.json', {'status': 'SOURCE_ONLY_PREFLIGHT', 'sources': sources, 'integration_before': before,
          'reference_hashes': {str(path.relative_to(WORKSPACE)): digest for path, digest in reference.items()},
          'bounds': {'seconds': 60, 'sampled_group_plus_supervisor_RSS_bytes': 268435456,
                     'retained_output_bytes': 8388608, 'disk_floor_bytes': 8000000000},
          'prior_suites_repeated': False, 'P004_replayed_or_regraded': False, 'native_GO': False})
    resources.require(sys.flags.optimize == 0 and os.path.realpath(sys.executable) == os.path.realpath(PYTHON), 'Pinned nonoptimized Python required')
    resources.require(sources['resources.py']['sha256'] == '29d9e6c0c2d12126582056d3e62d3b226f4d12bfda1860341cc9ae01f5742781', 'Resource monitor changed')
    resources.require(sources['native_call.js']['sha256'] == '9d108334cf4b1666e5e6546df221af1aba71d5c32a9a2642c2e49cac6545f890', 'Generic native helper changed')
    resources.require(sources['INT-PIN.json']['sha256'] == '8fd2ff1bcaa7005e140d4743baae20591b9ae79a4bf068802f9a4dd095f5e6e5' and
                      pin['HEAD'] == '023b45a84cfeb9248e20f620628911bc3c0c843a' and pin['tracked_status'] == '' and before == expected,
                      'Stable clean metadata HEAD/index changed')
    resources.require(all(sha(path) == digest for path, digest in reference.items()), 'Immutable prior packet changed')
    resources.require(free() >= 8000000000 + 8388608 and time.monotonic() < deadline, 'Preflight disk/time refusal; zero test producers')
    write(ROOT / 'SOURCE-FREEZE.json', {'status': 'V6_SOURCE_FROZEN_BEFORE_ONE_FOCUSED_CASE', 'files': sources, 'source_only': True, 'native_GO': False})
    stage = resources.monitor([PYTHON, '-B', str(ROOT / 'focused_selfcheck.py')], ROOT,
              OUT / 'stdout.log', OUT / 'stderr.log', deadline, volume, rss_cap=268435456,
              output_cap=8388608, reserve=65536, disk_floor=8000000000,
              on_start=lambda child: write(OUT / 'STARTED.json', {'child': child, 'start_monotonic': start,
                        'deadline_monotonic': deadline, 'source_freeze_sha256': sha(ROOT / 'SOURCE-FREEZE.json')}))
    summary = json.loads((OUT / 'stdout.log').read_text()) if stage['exit_code'] == 0 else None
    source_exact = all(sha(ROOT / name) == entry['sha256'] for name, entry in sources.items())
    prior_exact = all(sha(path) == digest for path, digest in reference.items())
    after = integration_snapshot(pin)
    passed = (stage['exit_code'] == 0 and stage['stop_reason'] is None and source_exact and prior_exact and after == expected
              and summary and summary['status'] == 'PASSED_V6_FOCUSED_CALLER_CHECKS')
    write(OUT / 'RESULT.json', {'status': 'PROVISIONAL_SOURCE_PROOF_REQUIRES_ZERO_TIMELY_ORIGINAL_TOOL_TERMINAL',
          'candidate_passed': bool(passed), 'stage': stage, 'summary': summary, 'sources_postexact': source_exact,
          'immutable_references_postexact': prior_exact, 'integration_after': after, 'integration_postexact': after == expected,
          'elapsed_before_result_seconds': time.monotonic() - start, 'actual_GUI_native_app_model_Cargo_network_calls': 0, 'native_GO': False})
    write(OUT / 'MANIFEST.json', {str(path.relative_to(OUT)): {'sha256': sha(path), 'bytes': path.stat().st_size}
          for path in sorted(OUT.rglob('*')) if path.is_file() and not path.is_symlink()})
    write(OUT / 'EXTERNAL-TERMINAL.json', {'status': 'PROVISIONAL_SUPERVISOR_TERMINAL', 'elapsed_seconds': time.monotonic() - start,
          'retained_bytes': volume(), 'disk_available_bytes': free(), 'deadline_monotonic': deadline})
    resources.require(passed and time.monotonic() < deadline and volume() + 65536 < 8388608 and free() >= 8000000000, 'Final source/time/output/disk refusal')
    print(json.dumps({'status': 'PASSED_BOUNDED_V6_FOCUSED_CASE', 'elapsed_seconds': time.monotonic() - start,
          'Python_scenarios': summary['Python_scenarios'], 'Node_scenarios': summary['Node_scenarios'],
          'sources_postexact': source_exact, 'immutable_references_postexact': prior_exact, 'integration_postexact': after == expected, 'native_GO': False}, sort_keys=True))


if __name__ == '__main__':
    main()
