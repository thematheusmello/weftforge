#!/usr/bin/env python3
"""Portable AFTER-CLOCK diagnostic replay; this never constructs or steps Cloth."""
import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def strict_loads(text):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'Duplicate JSON key')
            result[key] = value
        return result
    def floating(token):
        value = float(token)
        require(math.isfinite(value), 'Nonfinite JSON number')
        return value
    def constant(_):
        raise ValueError('Nonfinite JSON constant')
    return json.loads(text, object_pairs_hook=pairs, parse_float=floating, parse_constant=constant)


def read(path):
    return strict_loads(path.read_text())


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def function(source, declaration):
    start = source.index(declaration)
    end = source.index('{', start) + 1
    depth = 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end].encode()


def safe_child(root, name):
    relative = Path(name)
    require(not relative.is_absolute() and '..' not in relative.parts, 'Invalid relative artifact path')
    path = root / relative
    require(not path.is_symlink() and path.resolve().is_relative_to(root.resolve()), 'Artifact escaped packet')
    return path


def verify_bindings(repo, runtime):
    manifest = read(ROOT / 'MANIFEST.json')
    inventory = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob('*') if p.is_file()}
    require(inventory == set(manifest['files']) | {'MANIFEST.json'}, 'Packet inventory changed')
    require(not any(p.is_symlink() for p in ROOT.rglob('*')), 'Packet symlink refused')
    for name, digest in manifest['files'].items():
        require(sha(safe_child(ROOT, name)) == digest, 'Packet artifact changed: ' + name)
    binding = read(ROOT / 'BINDINGS.json')
    original_plan = read(ROOT / 'original-static/PLAN.json')
    require(binding['source_sha256'] == original_plan['source_sha256'], 'Original/runtime source binding differs')
    require(sha(ROOT / 'original-static/MANIFEST.json') == binding['original_static_manifest_sha256'], 'Closed static manifest changed')
    require(sha(ROOT / 'harness.rs') == binding['original_harness_sha256'], 'Original harness changed')
    closed_files = read(ROOT / 'original-static/MANIFEST.json')['files']
    for item in read(ROOT / 'PUBLICATION-PROVENANCE.json')['copies']:
        observed = sha(safe_child(ROOT, item['public_path']))
        require(observed == item['public_sha256'] == item['original_sha256'] and item['byte_identical'], 'Original copy changed')
        if 'original_static_member' in item:
            require(closed_files[item['original_static_member']]['sha256'] == observed, 'Closed copy provenance differs')
    for name, digest in binding['source_sha256'].items():
        require(sha(safe_child(ROOT / 'sources', Path(name).name)) == digest, 'Captured source changed')
        require(sha(safe_child(runtime, name)) == digest, 'Runtime sim source differs: ' + name)
    capsule = safe_child(repo, binding['capsule_packet'])
    for name, digest in binding['capsule_dependencies'].items():
        require(sha(safe_child(capsule, name)) == digest, 'Public capsule dependency changed: ' + name)
    capsule_manifest = read(capsule / 'MANIFEST.json')
    for name, digest in binding['capsule_dependencies'].items():
        if name != 'MANIFEST.json':
            require(capsule_manifest['files'][name] == digest, 'Capsule manifest dependency differs')
    freeze = read(capsule / 'original-evaluator/FREEZE.json')
    metrics_sha = freeze['files']['metrics.py']
    require(sha(capsule / 'original-evaluator/metrics.py') == sha(capsule / 'public-evaluator/metrics.py') == metrics_sha, 'Frozen evaluator math changed')
    return binding, capsule


def verify_unchanged_physics(capsule):
    previous = capsule / 'experiment/source/crates/weftforge-sim/src'
    receipt = read(ROOT / 'original-static/UNCHANGED-PHYSICS.json')
    old = (previous / 'constraints.rs').read_text()
    new = (ROOT / 'sources/constraints.rs').read_text()
    dead = function(old, '    pub fn signed_strains(').decode()
    require(new == old.replace(dead + '\n', '', 1), 'Constraint changes exceed dead diagnostic removal')
    hashes = {}
    for declaration in ['    fn deformation(', '    pub fn solve(', 'pub(crate) fn dihedral(', 'fn project', 'pub(crate) fn solve_stitch(']:
        before, after = function(old, declaration), function(new, declaration)
        require(before == after, 'Projection arithmetic changed')
        hashes[declaration] = hashlib.sha256(after).hexdigest()
    old = (previous / 'lib.rs').read_text()
    new = (ROOT / 'sources/lib.rs').read_text()
    for declaration in ['    pub fn step(', '    pub fn reset(', 'fn validate(']:
        before, after = function(old, declaration), function(new, declaration)
        require(before == after, 'Transaction, cancellation or guard policy changed')
        hashes[declaration] = hashlib.sha256(after).hexdigest()
    require(hashes == receipt['byte_identical_function_sha256'], 'Unchanged-physics function receipt differs')
    require((previous / 'math.rs').read_bytes() == (ROOT / 'sources/math.rs').read_bytes(), 'Common solver math changed')
    require(sha(ROOT / 'sources/math.rs') == receipt['common_math_sha256'], 'Common math receipt differs')
    return hashes


def prepare(scratch, capsule, binding, metrics):
    expected_peaks, cases = [], []
    axis_count = 0
    bits = lambda value: struct.unpack('<Q', struct.pack('<d', value))[0]
    with (scratch / 'inputs.bin').open('xb') as data, (scratch / 'expected-axes.bin').open('xb') as expected:
        data.write(b'WFSTRN01')
        def integer(value):
            data.write(struct.pack('<Q', value))
        def floating(value):
            data.write(struct.pack('<d', value))
        integer(2)
        for case_id, case in enumerate(['base', 'height']):
            run = capsule / 'experiment/runs' / case
            meta, analysis = read(run / 'rest-metadata.json'), read(run / 'analysis.json')
            trajectory = (run / 'trajectory.jsonl').read_text().splitlines()
            require(len(trajectory) == len(analysis['rows']) == 181, 'Wrong saved state count')
            require(analysis['quality_status'] == 'failed' and analysis['strict_delivery_passed'] is False, 'Original strict failure lost')
            integer(case_id)
            integer(len(meta['uv_m']))
            integer(len(meta['triangles']))
            integer(len(trajectory))
            for point in meta['uv_m']:
                for value in point:
                    floating(value)
            for triangle in meta['triangles']:
                for vertex in triangle:
                    integer(vertex)
            grains = {piece['id']: piece['grain_angle'] for piece in meta['project']['pieces']}
            for triangle in meta['triangles']:
                floating(grains[meta['piece_ids'][triangle[0]]])
            for frame, line in enumerate(trajectory):
                state = strict_loads(line)
                positions = state['positions_m']
                require(len(positions) == len(meta['uv_m']), 'Saved vertex count changed')
                integer(frame)
                for point in positions:
                    for value in point:
                        floating(value)
                independent = None
                for ti in range(len(meta['triangles'])):
                    axes = metrics.signed_triangle_axes(meta, positions, ti)
                    for name in ['warp', 'weft']:
                        value = axes[name]
                        expected.write(struct.pack('<d', value))
                        axis_count += 1
                        if independent is None or abs(value) > abs(independent['signed_strain']):
                            independent = {'triangle_index': ti, 'axis': name, 'signed_strain': value, 'absolute_strain': abs(value)}
                stored = analysis['rows'][frame]['independent_material_axis']['peak']
                require(independent['triangle_index'] == stored['triangle_index'] and independent['axis'] == stored['axis'], 'Frozen evaluator peak attribution changed')
                require(all(bits(independent[key]) == bits(stored[key]) for key in ['signed_strain', 'absolute_strain']), 'Frozen evaluator peak bits changed')
                expected_peaks.append({'case_id': case_id, 'case': case, 'frame': frame,
                    'maximum_bits': bits(stored['absolute_strain']), 'triangle': stored['triangle_index'],
                    'axis': ['warp', 'weft'].index(stored['axis']), 'signed_bits': bits(stored['signed_strain'])})
            cases.append({'case': case, 'case_id': case_id, 'states': len(trajectory), 'triangles': len(meta['triangles']),
                'triangle_axes': len(trajectory) * len(meta['triangles']) * 2,
                'original_exact_attribution_failed_frames_retained': analysis['exact_attribution_failed_frames'],
                'original_quality_status_retained': analysis['quality_status'],
                'original_strict_delivery_passed_retained': analysis['strict_delivery_passed']})
    require(cases == binding['cases'], 'Original failure lists or quality flags changed')
    require(len(expected_peaks) == 362 and axis_count == 838392, 'Wrong replay scope')
    (scratch / 'expected-peaks.json').write_text(json.dumps(expected_peaks, indent=2, allow_nan=False) + '\n')
    require(sha(scratch / 'inputs.bin') == binding['original_input_sha256'], 'Regenerated input f64 bytes differ')
    require(sha(scratch / 'expected-axes.bin') == binding['original_axis_values_sha256'], 'Regenerated frozen axis bytes differ')
    require(sha(scratch / 'expected-peaks.json') == binding['original_expected_peaks_sha256'], 'Regenerated frozen peak bytes differ')
    return expected_peaks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo-root', type=Path, default=Path.cwd())
    parser.add_argument('--runtime-root', type=Path, help='Defaults to repository root; must contain exact captured 2079411 sim source')
    args = parser.parse_args()
    require(__debug__, 'Run without Python optimization')
    require(sys.implementation.name == 'cpython' and sys.version_info[:2] == (3, 13), 'Use CPython 3.13 for the frozen sum convention')
    repo = args.repo_root.resolve()
    runtime = (args.runtime_root or repo).resolve()
    for malformed in ['{"frame":1,"frame":2}', '[NaN]', '[1e400]']:
        try:
            strict_loads(malformed)
        except ValueError:
            pass
        else:
            raise ValueError('Strict JSON self-check failed')
    try:
        safe_child(ROOT, '../escape')
    except ValueError:
        pass
    else:
        raise ValueError('Path boundary self-check failed')
    binding, capsule = verify_bindings(repo, runtime)
    physics_hashes = verify_unchanged_physics(capsule)
    original = load_module('original_diagnostic_comparator', ROOT / 'original-static/run_static.py')
    comparator_checks = original.comparator_self_checks()
    metrics = load_module('frozen_public_metrics', capsule / 'public-evaluator/metrics.py')
    started = time.monotonic()
    with tempfile.TemporaryDirectory(prefix='weftforge-diagnostic-') as temporary:
        scratch = Path(temporary).resolve()
        require(not scratch.is_relative_to(repo) and not scratch.is_relative_to(ROOT), 'Scratch must be outside repository and packet')
        (scratch / 'sources').mkdir()
        shutil.copyfile(ROOT / 'harness.rs', scratch / 'harness.rs')
        shutil.copyfile(ROOT / 'sources/strain.rs', scratch / 'sources/strain.rs')
        expected = prepare(scratch, capsule, binding, metrics)
        plan = read(ROOT / 'original-static/PLAN.json')
        version = subprocess.check_output(['rustup', 'run', '1.94.1', 'rustc', '--version', '--verbose'], cwd=scratch, text=True)
        compilation = subprocess.run(plan['compile_command'], cwd=scratch, capture_output=True)
        require(compilation.returncode == 0, 'Static harness compilation failed')
        execution = subprocess.run(plan['execution_command'], cwd=scratch, capture_output=True)
        require(execution.returncode == 0, 'Static harness assertion or execution failed')
        actual = [strict_loads(line) for line in execution.stdout.decode().splitlines()]
        require(len(actual) == len(expected) == 362, 'Compiled peak state count mismatch')
        peak_failures = [{'case': frozen['case'], 'frame': frozen['frame'], 'fields': original.compare_peak(observed, frozen)}
                         for observed, frozen in zip(actual, expected) if original.compare_peak(observed, frozen)]
        axis_comparison = original.compare_axes((scratch / 'actual-axes.bin').read_bytes(), (scratch / 'expected-axes.bin').read_bytes())
        require(not peak_failures and axis_comparison.get('mismatch_count') == 0, 'Exact compiled f64 comparison failed')
        harness_checks = [strict_loads(line) for line in execution.stderr.decode().splitlines()]
        require(harness_checks == [{'static_self_checks': 30, 'status': 'passed'}, {'states': 362, 'triangle_axes': 838392, 'new_physics_steps': 0}], 'Static self-check receipt changed')
        result = {'kind': 'portable AFTER-CLOCK compiled diagnostic replay', 'status': 'passed_all_362_frozen_peaks_and_838392_triangle_axes_exactly',
            'source_commit': binding['source_commit'], 'source_sha256': binding['source_sha256'],
            'runtime_source_hashes_exact': True, 'unchanged_physics_function_sha256': physics_hashes,
            'states': len(actual), 'triangle_axes': 838392, 'peak_failures': peak_failures, 'axis_comparison': axis_comparison,
            'new_cloth_initializations': 0, 'new_physics_steps': 0, 'new_timed_benchmarks': 0,
            'comparator_self_checks': comparator_checks, 'strict_json_and_path_self_checks': 'passed',
            'harness_self_checks': harness_checks, 'cases': binding['cases'],
            'input_sha256': sha(scratch / 'inputs.bin'), 'actual_axis_sha256': sha(scratch / 'actual-axes.bin'),
            'expected_axis_sha256': sha(scratch / 'expected-axes.bin'),
            'harness_sha256': sha(scratch / 'harness.rs'), 'compiled_helper_sha256': sha(scratch / 'sources/strain.rs'),
            'scratch_executable_sha256': sha(scratch / 'diagnostic-static-check'),
            'compiler_version': version, 'python_version': sys.version,
            'compile_command': plan['compile_command'], 'execution_command': plan['execution_command'],
            'compile_exit_code': compilation.returncode, 'execution_exit_code': execution.returncode,
            'scratch_outside_repository': True, 'scratch_retained': False,
            'limits': 'Static arithmetic replay only. Original A physical strain failures at28..37, 33A+3B exact attribution failures and both strict false results remain unchanged. No calibration, convergence, native UI, release or reference-parity acceptance.'}
    verify_bindings(repo, runtime)
    result['bindings_exact_before_and_after'] = True
    result['scratch_removed'] = not scratch.exists()
    result['posthoc_wall_seconds'] = time.monotonic() - started
    print(json.dumps(result, indent=2, allow_nan=False))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print('Portable diagnostic replay refused: ' + str(error), file=sys.stderr)
        sys.exit(1)
