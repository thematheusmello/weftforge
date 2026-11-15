"""Compile exact helper bytes, statically replay saved f64s, and compare exact bits."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import time

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[1]
WORKTREE = ROOT / '.local/diagnostic-peak-stability-20261115'
KEYS = ('case_id', 'frame', 'maximum_bits', 'triangle', 'axis', 'signed_bits')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(name, value):
    with (OUT / name).open('x') as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write('\n')


def compare_peak(actual, expected):
    failures = [key for key in KEYS if type(actual.get(key)) is not int or actual.get(key) != expected[key]]
    if actual.get('maximum_bits') != (actual.get('signed_bits', 0) & ((1 << 63) - 1)):
        failures.append('maximum/witness coherence')
    return failures


def compare_axes(actual, expected):
    if len(actual) != len(expected) or len(actual) % 8:
        return {'count_mismatch': True, 'actual_bytes': len(actual), 'expected_bytes': len(expected)}
    failures = [i for i in range(len(actual) // 8) if actual[i*8:i*8+8] != expected[i*8:i*8+8]]
    return {'mismatch_count': len(failures), 'first_mismatches': failures[:20]}


def comparator_self_checks():
    peak = {'case_id': 0, 'frame': 0, 'maximum_bits': 4602678819172646912, 'triangle': 0, 'axis': 0, 'signed_bits': 4602678819172646912}
    assert not compare_peak(peak, peak)
    for key in KEYS:
        assert compare_peak(dict(peak, **{key: peak[key] + 1}), peak)
    wrong_coherent = dict(peak, signed_bits=peak['signed_bits'] + 1, maximum_bits=peak['maximum_bits'] + 1)
    assert compare_peak(wrong_coherent, peak)
    assert compare_peak(dict(peak, triangle=0.0), peak)
    values = struct.pack('<2d', .5, -.25)
    assert compare_axes(values, values)['mismatch_count'] == 0
    flipped = bytearray(values)
    flipped[0] ^= 1
    assert compare_axes(bytes(flipped), values)['mismatch_count'] == 1
    assert compare_axes(values[:-1], values)['count_mismatch']
    return {'peak_field_and_ulp_mutations_rejected': True, 'coherent_wrong_peak_rejected': True,
            'axis_ulp_and_count_mutations_rejected': True}


def verify(plan):
    for name, digest in plan['frozen_files'].items():
        assert sha(OUT / name) == digest, name
    for name, digest in plan['source_sha256'].items():
        assert sha(WORKTREE / name) == digest, name
    for name, digest in json.loads((OUT / 'OLD-INPUT-BINDING.json').read_text())['files'].items():
        assert sha(ROOT / name) == digest, name
    compiler_path = subprocess.check_output(['rustup', 'which', '--toolchain', '1.94.1', 'rustc']).decode().strip()
    assert sha(Path(compiler_path)) == plan['rustc_sha256']


def run(command, label):
    started = time.monotonic()
    with (OUT / (label + '.stdout')).open('xb') as stdout, (OUT / (label + '.stderr')).open('xb') as stderr:
        result = subprocess.run(command, cwd=OUT, stdout=stdout, stderr=stderr)
    return {'command': command, 'exit_code': result.returncode, 'seconds': time.monotonic() - started}


def main():
    self_checks = comparator_self_checks()
    plan = json.loads((OUT / 'PLAN.json').read_text())
    verify(plan)
    preflight = {'utc': datetime.now(timezone.utc).isoformat(), 'plan_sha256': sha(OUT / 'PLAN.json'),
                 'runner_sha256': sha(Path(__file__)), 'comparator_self_checks': self_checks,
                 'input_source_and_old_bindings_exact': True}
    write_json('RUN-PREFLIGHT.json', preflight)
    result = {'kind': 'AFTER-CLOCK actual compiled unmodified diagnostic helper static replay',
              'status': 'failed', 'new_physics_steps': 0, 'states': plan['states'],
              'triangle_axis_length_minus_one_values': plan['triangle_axes'],
              'preflight': preflight, 'cases': plan['cases'], 'failures': []}
    result['compilation'] = run(plan['compile_command'], 'compile')
    if result['compilation']['exit_code']:
        result['failures'].append('Static Rust harness did not compile; outputs retained.')
    else:
        result['executable_sha256'] = sha(OUT / 'diagnostic-static-check')
        result['execution'] = run(plan['execution_command'], 'actual')
        if result['execution']['exit_code']:
            result['failures'].append('Static Rust execution exited nonzero; outputs retained.')
        else:
            actual = [json.loads(line) for line in (OUT / 'actual.stdout').read_text().splitlines()]
            expected = json.loads((OUT / 'expected-peaks.json').read_text())
            result['peak_rows'] = []
            if len(actual) != len(expected) or len(actual) != 362:
                result['failures'].append('Peak state count mismatch')
            for observed, frozen in zip(actual, expected):
                failures = compare_peak(observed, frozen)
                result['peak_rows'].append({'actual': observed, 'frozen': frozen, 'failures': failures})
                if failures:
                    result['failures'].append({'case': frozen['case'], 'frame': frozen['frame'], 'fields': failures})
            actual_axes = (OUT / 'actual-axes.bin').read_bytes()
            expected_axes = (OUT / 'expected-axes.bin').read_bytes()
            result['all_triangle_axis_comparison'] = compare_axes(actual_axes, expected_axes)
            result['actual_axes_sha256'] = sha(OUT / 'actual-axes.bin')
            result['expected_axes_sha256'] = sha(OUT / 'expected-axes.bin')
            if result['all_triangle_axis_comparison'].get('mismatch_count') != 0:
                result['failures'].append('Individual triangle/axis f64 values differ from frozen evaluator')
            if not result['failures']:
                result['status'] = 'passed_all_362_frozen_peaks_and_838392_triangle_axes_exactly'
    verify(plan)
    result['postflight_input_source_and_old_bindings_exact'] = True
    result['old_raw_33A_plus_3B_failures_and_strict_false_results_retained'] = True
    result['limits'] = 'Posthoc static arithmetic evidence only. No Cloth initialization, integration, garment/fit/UI run, physical threshold change, evaluator modification, calibration, release or reference-parity acceptance.'
    write_json('RESULT.json', result)
    print(json.dumps({key: result[key] for key in ['status', 'states', 'triangle_axis_length_minus_one_values', 'failures', 'postflight_input_source_and_old_bindings_exact']}))
    raise SystemExit(0 if not result['failures'] else 1)


if __name__ == '__main__':
    main()
