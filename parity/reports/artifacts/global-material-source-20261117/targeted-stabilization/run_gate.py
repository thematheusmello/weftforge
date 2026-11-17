"""One authorized targeted gate; preserve failures before any source changes."""
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys
import time
from datetime import datetime, timezone
import resources as r

PROOF = Path(__file__).absolute().parent
WT = PROOF.parent.parent
EXPECTED_FREEZE = '24bfc16435c64156f2468f79843e1a6f9a49d360d37d93a1d455b535f2905df3'

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def persist(path, value):
    r.write_new(path, r.encoded(value))
    with path.open('rb') as stream:
        os.fsync(stream.fileno())

def main():
    index = int(sys.argv[1])
    name = ['FMT', 'CLIPPY', 'SIM-TESTS'][index]
    assert sha(PROOF / 'SOURCE-FREEZE.json') == EXPECTED_FREEZE
    freeze = json.loads((PROOF / 'SOURCE-FREEZE.json').read_text())
    assert sha(PROOF / 'resources.py') == freeze['resources_sha256']
    before = {name: sha(WT / name) for name in freeze['source_hashes']}
    assert before == freeze['source_hashes']
    for data in freeze['toolchain_binaries'].values():
        assert sha(data['path']) == data['sha256']
    env = dict(os.environ)
    for key in ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_BUILD_RUSTC_WRAPPER', 'CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER'):
        assert env.get(key) in (None, '')
        env.pop(key, None)
    bin_dir = str(Path(freeze['toolchain_binaries']['cargo']['path']).parent)
    env.update(RUSTUP_TOOLCHAIN='1.94.1-aarch64-apple-darwin',
               RUSTC=freeze['toolchain_binaries']['rustc']['path'],
               RUSTDOC=freeze['toolchain_binaries']['rustdoc']['path'],
               RUSTFMT=freeze['toolchain_binaries']['rustfmt']['path'],
               PATH=bin_dir + os.pathsep + env['PATH'],
               CARGO_BUILD_JOBS='1', CARGO_TARGET_DIR=str(WT / 'target'),
               RUSTFLAGS='', CARGO_ENCODED_RUSTFLAGS='', CARGO_NET_OFFLINE='true',
               PYTHONDONTWRITEBYTECODE='1', PYTHONOPTIMIZE='0')
    argv = freeze['argvs'][index]
    directory = PROOF / name
    directory.mkdir()
    started = time.monotonic()
    deadline = started + freeze['command_wall_limit_seconds']
    persist(directory / 'COMMAND-FREEZE.json', {'stage': name, 'argv': argv, 'source_freeze_sha256': EXPECTED_FREEZE,
            'runner_sha256': sha(PROOF / 'run_gate.py'), 'at_utc': datetime.now(timezone.utc).isoformat(),
            'environment': {key: env[key] for key in ('RUSTUP_TOOLCHAIN', 'RUSTC', 'RUSTDOC', 'RUSTFMT', 'PATH',
                'CARGO_BUILD_JOBS', 'CARGO_TARGET_DIR', 'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_NET_OFFLINE')},
            'source_before': before})
    count = lambda: sum(path.stat().st_size for path in r.files_below(directory))
    stage = r.monitor(argv, WT, directory / 'stdout.log', directory / 'stderr.log', deadline, count,
                      rss_cap=freeze['per_process_RSS_limit_bytes'], output_cap=freeze['per_command_log_limit_bytes'],
                      reserve=2 * r.RECEIPT_RESERVE, child_env=env,
                      on_start=lambda handle: persist(directory / 'started.json', handle))
    after = {name: sha(WT / name) for name in freeze['source_hashes']}
    text = (directory / 'stdout.log').read_text() + (directory / 'stderr.log').read_text()
    test_counts = re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out', text)
    classification = 'OPERATIONAL_RESOURCE_FAILURE' if stage['stop_reason'] else 'GREEN' if stage['exit_code'] == 0 else ['FMT_FAILURE', 'STRICT_CLIPPY_FAILURE', 'SIM_TEST_FAILURE'][index]
    result = {'classification': classification, 'stage_name': name, 'stage': stage,
              'test_counts': test_counts, 'source_before_after_exact': before == after,
              'source_after': after, 'stdout_sha256': sha(directory / 'stdout.log'),
              'stderr_sha256': sha(directory / 'stderr.log'), 'elapsed_before_result_persistence_seconds': time.monotonic() - started,
              'no_source_edits_or_retry_after_command': True}
    persist(directory / 'RESULT.json', result)
    assert before == after
    print(json.dumps({'stage': name, 'classification': classification, 'exit_code': stage['exit_code'],
                     'stop_reason': stage['stop_reason'], 'test_counts': test_counts,
                     'source_exact': before == after, 'result_sha256': sha(directory / 'RESULT.json'),
                     'elapsed_seconds': time.monotonic() - started}), flush=True)
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
