"""Freeze input f64s and unchanged evaluator expectations for static Rust replay."""
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import subprocess
import sys

sys.dont_write_bytecode = True
OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[1]
WORKTREE = ROOT / '.local/diagnostic-peak-stability-20261115'
REVIEW = ROOT / '.local/capsule-attribution-review-20261115'
EXPERIMENT = ROOT / '.local/capsule-garment-20261115'
FROZEN = ROOT / '.local/capsule-garment-evaluator-20261115'
CONTRACT_SHA = 'e2762b168085a4d2c2c738819083b2be9311c10946af1251ab21ffebf61f18d7'
BASE = '0770204a54abdf5f929b6626b01d7fcfb0e13ca4'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(name, value):
    with (OUT / name).open('x') as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write('\n')


def bits(value):
    return struct.unpack('<Q', struct.pack('<d', value))[0]


def function(source, declaration):
    start = source.index(declaration)
    opening = source.index('{', start)
    depth = 1
    end = opening + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end].encode()


assert not (OUT / 'PLAN.json').exists(), 'Preparation is already frozen'
assert sha(REVIEW / 'DIAGNOSTIC-CONTRACT.md') == CONTRACT_SHA
sys.path.insert(0, str(FROZEN))
import evaluator as e

e.verify_frozen()
old_binding = json.loads((REVIEW / 'INPUT-BINDING.json').read_text())['files']
for name, digest in old_binding.items():
    assert sha(ROOT / name) == digest, name
source_binding = json.loads((EXPERIMENT / 'SOURCE-INPUT-BINDING.json').read_text())['files']
for name, digest in source_binding.items():
    assert sha(EXPERIMENT / name) == digest, name
old_files = dict(old_binding)
for name in ['DIAGNOSTIC-CONTRACT.md', 'REPORT.md', 'ARITHMETIC-RESULT.json', 'arithmetic_followup.py', 'INPUT-BINDING.json', 'DATA-PHASE-RECEIPT.json', 'FOLLOWUP-FAILURE.json']:
    old_files[str((REVIEW / name).relative_to(ROOT))] = sha(REVIEW / name)
for name in e.read(FROZEN / 'FREEZE.json')['files']:
    old_files[str((FROZEN / name).relative_to(ROOT))] = sha(FROZEN / name)
old_files[str((FROZEN / 'FREEZE.json').relative_to(ROOT))] = sha(FROZEN / 'FREEZE.json')
for name, digest in source_binding.items():
    old_files[str((EXPERIMENT / name).relative_to(ROOT))] = digest
write_json('OLD-INPUT-BINDING.json', {'files': old_files, 'scope': 'Immutable pre-existing captures, reviewed own-math contract, all frozen evaluator files and 40 source snapshots'})

(OUT / 'sources').mkdir()
source_files = ['crates/weftforge-sim/src/' + name for name in ['strain.rs', 'lib.rs', 'constraints.rs', 'math.rs', 'tests.rs']]
source_hashes = {}
for name in source_files:
    path = WORKTREE / name
    raw = path.read_bytes()
    (OUT / 'sources' / path.name).write_bytes(raw)
    source_hashes[name] = sha(path)
old_constraints = subprocess.check_output(['git', 'show', BASE + ':crates/weftforge-sim/src/constraints.rs'], cwd=ROOT).decode()
new_constraints = (WORKTREE / 'crates/weftforge-sim/src/constraints.rs').read_text()
dead_method = function(old_constraints, '    pub fn signed_strains(').decode()
assert new_constraints == old_constraints.replace(dead_method + '\n', '', 1)
invariants = {}
for declaration in ['    fn deformation(', '    pub fn solve(', 'pub(crate) fn dihedral(', 'fn project', 'pub(crate) fn solve_stitch(']:
    old, new = function(old_constraints, declaration), function(new_constraints, declaration)
    assert old == new
    invariants[declaration] = hashlib.sha256(new).hexdigest()
old_lib = subprocess.check_output(['git', 'show', BASE + ':crates/weftforge-sim/src/lib.rs'], cwd=ROOT).decode()
new_lib = (WORKTREE / 'crates/weftforge-sim/src/lib.rs').read_text()
for declaration in ['    pub fn step(', '    pub fn reset(', 'fn validate(']:
    old, new = function(old_lib, declaration), function(new_lib, declaration)
    assert old == new
    invariants[declaration] = hashlib.sha256(new).hexdigest()
old_math = subprocess.check_output(['git', 'show', BASE + ':crates/weftforge-sim/src/math.rs'], cwd=ROOT)
assert old_math == (WORKTREE / 'crates/weftforge-sim/src/math.rs').read_bytes()
write_json('UNCHANGED-PHYSICS.json', {'constraints_only_change': 'Removed unused signed_strains diagnostic method', 'byte_identical_function_sha256': invariants, 'common_math_sha256': hashlib.sha256(old_math).hexdigest()})

peaks, cases = [], []
axis_count = 0
with (OUT / 'inputs.bin').open('xb') as data, (OUT / 'expected-axes.bin').open('xb') as expected:
    data.write(b'WFSTRN01')
    def integer(value):
        data.write(struct.pack('<Q', value))
    def floating(value):
        data.write(struct.pack('<d', value))
    integer(2)
    for case_id, case in enumerate(['base', 'height']):
        run = EXPERIMENT / 'runs' / case
        meta, analysis = e.read(run / 'rest-metadata.json'), e.read(run / 'analysis.json')
        trajectory = (run / 'trajectory.jsonl').read_text().splitlines()
        assert len(trajectory) == len(analysis['rows']) == 181
        assert analysis['quality_status'] == 'failed' and analysis['strict_delivery_passed'] is False
        assert len(analysis['exact_attribution_failed_frames']) == (33 if case == 'base' else 3)
        integer(case_id)
        integer(len(meta['uv_m']))
        integer(len(meta['triangles']))
        integer(len(trajectory))
        for point in meta['uv_m']:
            for q in point:
                floating(q)
        for triangle in meta['triangles']:
            for vertex in triangle:
                integer(vertex)
        grains = {p['id']: p['grain_angle'] for p in meta['project']['pieces']}
        for triangle in meta['triangles']:
            floating(grains[meta['piece_ids'][triangle[0]]])
        for frame, line in enumerate(trajectory):
            state = e.strict_loads(line)
            integer(frame)
            positions = state['positions_m']
            assert len(positions) == len(meta['uv_m'])
            for point in positions:
                for q in point:
                    floating(q)
            independent_peak = None
            for ti in range(len(meta['triangles'])):
                axes = e.metrics.signed_triangle_axes(meta, positions, ti)
                for axis, name in enumerate(['warp', 'weft']):
                    value = axes[name]
                    expected.write(struct.pack('<d', value))
                    axis_count += 1
                    if independent_peak is None or abs(value) > abs(independent_peak['signed_strain']):
                        independent_peak = {'triangle_index': ti, 'axis': name, 'signed_strain': value, 'absolute_strain': abs(value)}
            stored = analysis['rows'][frame]['independent_material_axis']['peak']
            for key in independent_peak:
                assert e.same(independent_peak[key], stored[key]), (case, frame, key)
            peaks.append({'case_id': case_id, 'case': case, 'frame': frame,
                          'maximum_bits': bits(stored['absolute_strain']), 'triangle': stored['triangle_index'],
                          'axis': ['warp', 'weft'].index(stored['axis']), 'signed_bits': bits(stored['signed_strain'])})
        cases.append({'case': case, 'case_id': case_id, 'states': len(trajectory), 'triangles': len(meta['triangles']),
                      'triangle_axes': len(trajectory) * len(meta['triangles']) * 2,
                      'original_exact_attribution_failed_frames_retained': analysis['exact_attribution_failed_frames'],
                      'original_quality_status_retained': analysis['quality_status'],
                      'original_strict_delivery_passed_retained': analysis['strict_delivery_passed']})
assert len(peaks) == 362 and axis_count == 838392
write_json('expected-peaks.json', peaks)
version = subprocess.check_output(['rustup', 'run', '1.94.1', 'rustc', '--version', '--verbose'])
(OUT / 'compiler-version.txt').write_bytes(version)
compiler_path = subprocess.check_output(['rustup', 'which', '--toolchain', '1.94.1', 'rustc']).decode().strip()
frozen_names = ['METHOD.md', 'OLD-INPUT-BINDING.json', 'UNCHANGED-PHYSICS.json', 'prepare.py', 'harness.rs', 'inputs.bin', 'expected-axes.bin', 'expected-peaks.json', 'compiler-version.txt']
frozen_names += ['sources/' + Path(name).name for name in source_files]
plan = {'base_commit': BASE, 'kind': 'AFTER-CLOCK static compiled diagnostic-only arithmetic verification',
        'new_physics_steps': 0, 'cases': cases, 'states': len(peaks), 'triangle_axes': axis_count,
        'source_sha256': source_hashes,
        'rustc_sha256': sha(Path(compiler_path)),
        'compile_command': ['rustup', 'run', '1.94.1', 'rustc', '--edition=2024', '--crate-name', 'diagnostic_static_check', '-D', 'warnings', 'harness.rs', '-o', 'diagnostic-static-check'],
        'execution_command': ['./diagnostic-static-check'],
        'frozen_files': {name: sha(OUT / name) for name in frozen_names}}
write_json('PLAN.json', plan)
for name, digest in old_files.items():
    assert sha(ROOT / name) == digest, name
e.verify_frozen()
print(json.dumps({'status': 'prepared', 'states': len(peaks), 'triangle_axes': axis_count,
                  'cases': cases, 'source_sha256': source_hashes, 'plan_sha256': sha(OUT / 'PLAN.json')}))
