"""Check tracked retained evidence from a clean checkout; never launch the product."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys

BUNDLE = Path(__file__).resolve().parent
REPO = BUNDLE.parents[3]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_bytes())


def inventory(base, records):
    for name, record in records.items():
        path = base / name
        assert not Path(name).is_absolute() and '..' not in Path(name).parts
        assert path.resolve().is_relative_to(base) and not path.is_symlink()
        assert path.stat().st_size == record['bytes'] and digest(path) == record['sha256'], name


def main():
    output = Path(sys.argv[1]).resolve()
    assert not output.is_relative_to(BUNDLE), 'Results must remain outside immutable evidence'
    assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=REPO), 'Use a clean checkout'
    manifest = load(BUNDLE / 'bundle-manifest.json')
    actual = {p.relative_to(BUNDLE).as_posix() for p in BUNDLE.rglob('*') if p.is_file()}
    assert actual == set(manifest['artifacts']) | {'bundle-manifest.json'}
    inventory(BUNDLE, manifest['artifacts'])
    tracked = subprocess.check_output(
        ['git', 'ls-files', '-z', '--', BUNDLE.relative_to(REPO).as_posix()], cwd=REPO
    ).decode().split('\0')
    assert {p for p in tracked if p} == {(BUNDLE.relative_to(REPO) / p).as_posix() for p in actual}
    source = load(BUNDLE / 'supplemental/runtime/source-pretest.json')
    assert source['source_commit'] == manifest['candidate']['commit']
    subprocess.run(['git', 'merge-base', '--is-ancestor', source['source_commit'], 'HEAD'], cwd=REPO, check=True)
    scope = subprocess.check_output(
        ['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z', '--', *source['source_scope']], cwd=REPO
    ).decode().split('\0')
    assert {p for p in scope if p} == set(source['source_hashes']) and len(source['source_hashes']) == 37
    for name, expected in source['source_hashes'].items():
        committed = subprocess.check_output(['git', 'show', source['source_commit'] + ':' + name], cwd=REPO)
        assert hashlib.sha256(committed).hexdigest() == expected == digest(REPO / name), name
    native = load(BUNDLE / 'native/native-bundle-manifest.json')
    assert {p.relative_to(BUNDLE / 'native').as_posix() for p in (BUNDLE / 'native').rglob('*') if p.is_file()} == set(native['artifacts']) | {'native-bundle-manifest.json'}
    inventory(BUNDLE / 'native', native['artifacts'])
    output.mkdir(parents=True, exist_ok=False)
    jobs = [
        ('headless', ['headless/replay.py', str(output / 'headless')], 0, None),
        ('sdk-and-supplemental', ['supplemental/check_bundle.py'], 0, None),
        ('native-unreviewed', ['headless/oracle/native-supplement/evaluate_native.py',
                               'native/capture-manifest.unreviewed.publication.json', str(output / 'native-unreviewed.json')], 1,
         'native event/visible-state review remains unknown'),
        ('native-reviewed', ['headless/oracle/native-supplement/evaluate_native.py',
                             'native/capture-manifest.reviewed.json', str(output / 'native-reviewed.json')], 0, None),
    ]
    for case in ('asymmetric-coarse', 'asymmetric-fine'):
        jobs.append((case, ['supplemental/fine-mesh/check-mesh.py',
                           f'supplemental/fine-mesh/fixtures/{case}.weftforge',
                           f'headless/inspect/{case}/inspect-stdout.json', str(output / f'{case}.json')], 0, None))
    results = []
    for name, arguments, expected_exit, reason in jobs:
        with (output / f'{name}-stdout.txt').open('xb') as stdout, (output / f'{name}-stderr.txt').open('xb') as stderr:
            process = subprocess.Popen([sys.executable, '-B', *arguments], cwd=BUNDLE,
                                       env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'), stdout=stdout, stderr=stderr)
            code = process.wait()
        record = {'name': name, 'pid': process.pid, 'exit_code': code, 'expected_exit_code': expected_exit,
                  'stderr_bytes': (output / f'{name}-stderr.txt').stat().st_size}
        assert code == expected_exit and record['stderr_bytes'] == 0, record
        if reason:
            refused = load(output / f'{name}.json')
            assert refused['status'] == 'fail' and refused['reason'] == reason
        elif name == 'native-reviewed':
            assert load(output / 'native-reviewed.json')['status'] == 'pass'
        results.append(record)
        print(json.dumps(record), flush=True)
    assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=REPO)
    with (output / 'summary.json').open('x') as stream:
        json.dump({'kind': 'tracked-ordered-sewing-output-replay-v1', 'all_expected_outcomes': True,
                   'tracked_artifacts': len(actual), 'source_commit': source['source_commit'],
                   'source_files': 37, 'results': results, 'candidate_binary_executed': False,
                   'native_ui_executed': False, 'reference_parity': 'unknown', 'physics_quality': 'unverified'}, stream, indent=2)
        stream.write('\n')


if __name__ == '__main__':
    main()
