"""Compile once, run the static primitive once, preserve output, compare frozen rules."""
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import subprocess
import time

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[1]
PRIOR = ROOT / '.local/capsule-independent-review-20261115'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def verify(plan):
    for name, expected in dict(plan['frozen_files'], **plan['engine_sha256']).items():
        assert sha((OUT / name).read_bytes()) == expected, name
    for name, expected in plan['engine_sha256'].items():
        raw = subprocess.check_output(['git', 'show', plan['commit'] + ':crates/weftforge-sim/src/' + Path(name).name], cwd=ROOT)
        assert sha(raw) == expected, name
    prior_manifest = (PRIOR / 'MANIFEST.json').read_bytes()
    assert sha(prior_manifest) == plan['prior_manifest_sha256']
    for name, expected in json.loads(prior_manifest)['files'].items():
        data = (PRIOR / name).read_bytes()
        assert len(data) == expected['bytes'] and sha(data) == expected['sha256'], name
    compiler = subprocess.check_output(['rustup', 'which', '--toolchain', '1.94.1', 'rustc'], cwd=OUT).decode().strip()
    assert sha(Path(compiler).read_bytes()) == plan['rustc_sha256']


def compare(actual, expected):
    failures = []
    if actual.get('label') != expected['label']:
        return ['case label/order mismatch']
    distance = actual.get('distance')
    bary = actual.get('bary', [])
    fraction = actual.get('axis_fraction')
    if not isinstance(distance, (int, float)) or not math.isfinite(distance) or distance < 0.:
        failures.append('distance is not finite/nonnegative')
    if len(bary) != 3 or not all(isinstance(x, (int, float)) and math.isfinite(x) and 0. <= x <= 1. for x in bary) or abs(sum(bary) - 1.) >= 1e-13:
        failures.append('invalid convex barycentric coordinates')
    if not isinstance(fraction, (int, float)) or not math.isfinite(fraction) or not 0. <= fraction <= 1.:
        failures.append('invalid axis fraction')
    if not isinstance(actual.get('uncertain'), bool):
        failures.append('uncertain is not boolean')
    if failures:
        return failures
    tolerance, truth = expected['tolerance'], expected['independent_qp_distance']
    if distance > truth + tolerance:
        failures.append('distance exceeds independent minimum plus original tolerance')
    if not actual['uncertain'] and abs(distance - truth) > tolerance:
        failures.append('exact distance differs beyond original tolerance')
    if actual['uncertain'] != expected['expected_uncertain']:
        failures.append('uncertainty classification differs from frozen prior case')
    return failures


def self_check():
    expected = {'label': 'boundary', 'independent_qp_distance': 1., 'tolerance': 0., 'expected_uncertain': False}
    actual = {'label': 'boundary', 'distance': 1., 'uncertain': False, 'bary': [.5, .5, 0.], 'axis_fraction': .5}
    assert compare(actual, expected) == []
    assert compare(dict(actual, distance=math.nextafter(1., math.inf)), expected)
    assert compare(dict(actual, distance=math.nan), expected)
    assert compare(dict(actual, bary=[1., 1., 0.]), expected)
    assert compare(dict(actual, uncertain=True), expected)
    uncertain = dict(expected, expected_uncertain=True)
    assert compare(dict(actual, distance=0., uncertain=True), uncertain) == []


def run(command, name):
    start = time.monotonic()
    with (OUT / (name + '.stdout')).open('wb') as stdout, (OUT / (name + '.stderr')).open('wb') as stderr:
        result = subprocess.run(command, cwd=OUT, stdout=stdout, stderr=stderr, check=False)
    return {'command': command, 'exit_code': result.returncode, 'wall_seconds': time.monotonic() - start}


def main():
    self_check()
    assert not (OUT / 'RUN-PREFLIGHT.json').exists(), 'One attempt already started'
    plan = json.loads((OUT / 'PLAN.json').read_text())
    verify(plan)
    preflight = {'utc': datetime.now(timezone.utc).isoformat(), 'plan_sha256': sha((OUT / 'PLAN.json').read_bytes()),
                 'runner_sha256': sha(Path(__file__).read_bytes()), 'all_bindings_exact': True,
                 'comparator_self_checks': 'passed'}
    (OUT / 'RUN-PREFLIGHT.json').write_text(json.dumps(preflight, indent=2) + '\n')
    compilation = run(plan['compile_command'], 'compile')
    result = {'kind': 'actual_unmodified_rust_static_capsule_comparison', 'commit': plan['commit'],
              'preflight': preflight, 'compilation': compilation, 'status': 'failed', 'case_results': [], 'failures': []}
    if compilation['exit_code'] != 0:
        result['failures'].append('Include harness did not compile. No engine alteration or retry.')
    else:
        result['executable_sha256'] = sha((OUT / 'capsule-static-check').read_bytes())
        result['execution'] = run(plan['execution_command'], 'actual')
        if result['execution']['exit_code'] != 0:
            result['failures'].append('Static executable exited nonzero. No retry.')
        else:
            actual = [json.loads(line) for line in (OUT / 'actual.stdout').read_text().splitlines()]
            expected = json.loads((OUT / 'expected.json').read_text())
            if len(actual) != len(expected) or len(expected) != 83:
                result['failures'].append('Actual/frozen case count mismatch')
            for row, oracle in zip(actual, expected):
                failures = compare(row, oracle)
                result['case_results'].append({'label': oracle['label'], 'actual': row,
                    'independent_qp_distance': oracle['independent_qp_distance'],
                    'original_tolerance': oracle['tolerance'], 'expected_uncertain': oracle['expected_uncertain'],
                    'absolute_oracle_difference': abs(row['distance'] - oracle['independent_qp_distance']),
                    'failures': failures})
                if failures:
                    result['failures'].append({'label': oracle['label'], 'failures': failures})
            if not result['failures']:
                result['status'] = 'passed_all_83_static_cases'
    verify(plan)
    result['postflight_bindings_exact'] = True
    result['prior_report_unchanged'] = True
    result['actual_stdout_sha256'] = sha((OUT / 'actual.stdout').read_bytes()) if (OUT / 'actual.stdout').exists() else None
    result['limits'] = 'No garment, response stepping, Cargo, workspace build, GUI, convergence or parity claim.'
    (OUT / 'RESULT.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'status': result['status'], 'cases': len(result['case_results']),
                      'failures': result['failures'], 'executable_sha256': result.get('executable_sha256'),
                      'actual_stdout_sha256': result['actual_stdout_sha256'], 'all_bindings_exact': True,
                      'prior_report_unchanged': True}))


if __name__ == '__main__':
    main()
