"""Read-only, stdlib saved-output replay; no simulation, build, network or Git writes."""
import sys
sys.dont_write_bytecode = True
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import re

base = Path(__file__).resolve().parent
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads((base / p).read_text())
manifest = read('MANIFEST.json')
actual = {p.relative_to(base).as_posix() for p in base.rglob('*') if p.is_file()}
assert actual == set(manifest['files']) | {'MANIFEST.json'}, 'Artifact closure changed'
private_path = re.compile(r'/(?:Users|home)/[^/\s<>]+/')
for name, record in manifest['files'].items():
    path = base / name
    assert not path.is_symlink()
    data = path.read_bytes()
    assert len(data) == record['bytes'] and sha(path) == record['sha256'], name
    assert not private_path.search(data.decode('utf-8')), name
assert all(part not in ('.git', 'target', '__pycache__') for name in actual for part in Path(name).parts)
source = read('source-manifest.json')
assert source['commit'] == '3eddef067c8a1f03f192c8e6822a8da2a94f9bf9'
assert len(source['files']) == 37
assert all(sha(base / 'source' / name) == checksum for name, checksum in source['files'].items())
provenance = read('PROVENANCE.json')
by_origin = {p['origin']: p for p in provenance['files']}
for item in provenance['files']:
    assert sha(base / item['bundle_path']) == item['bundle_sha256']
    assert item['byte_identical'] == (item['raw_sha256'] == item['bundle_sha256'])
    if not item['byte_identical']:
        assert item['derivative']['kind'] == 'pathname_prefix_only'
        assert item['derivative']['numerical_fixture_state_bytes_changed'] is False
        assert item['derivative']['replacement_count'] > 0
        assert item['bundle_path'].endswith(('-job.json', '.stdout', '.stderr'))
original = read('producer-original-artifact-inventory.json')
for name, record in original['files'].items():
    assert by_origin['<PRIVATE_EXPERIMENT_ROOT>/' + name]['raw_sha256'] == record['sha256']
for case in ['control', 'zero-g-balance', 'no-avatar']:
    for name in ['rest-metadata.json', 'trajectory.jsonl', 'steps.jsonl', 'completed-case.json']:
        assert by_origin['<PRIVATE_EXPERIMENT_ROOT>/' + case + '/' + name]['byte_identical']
assert by_origin['<PRIVATE_EXPERIMENT_ROOT>/original-grain-shirt.weftforge']['byte_identical']
for name in ['plan.json', 'plan-erratum-001.json', 'plan-probes-001.json']:
    assert by_origin['<PRIVATE_EXPERIMENT_ROOT>/' + name]['byte_identical']
# The original frozen logger/wrapper files remain in preregistered-v0 after the erratum.
first = read('plan.json')
for name, checksum in first['frozen_code_sha256'].items():
    original_name = {'harness/src/main.rs': 'preregistered-v0/main.rs', 'job.py': 'preregistered-v0/job.py'}.get(name, name)
    assert sha(base / original_name) == checksum
for name in ['plan-erratum-001.json', 'plan-probes-001.json']:
    assert all(sha(base / p) == checksum for p, checksum in read(name)['frozen_code_sha256'].items())
proof = read('prefix-history-proof.json')
assert proof['source_backed_internal_history']['source_lib_sha256'] == sha(base / 'source/crates/weftforge-sim/src/lib.rs')
assert proof['source_backed_internal_history']['source_constraints_sha256'] == sha(base / 'source/crates/weftforge-sim/src/constraints.rs')
assert len(proof['direct_observation']['frames_0_through_60']) == 61
jobs = [read(name) for name in manifest['files'] if name.endswith('-job.json')]
assert len(jobs) == 9 and all(j['status'] in ['completed', 'failed'] for j in jobs)
assert [(j['stage'], j['exit_code']) for j in jobs if j['exit_code']] == [('clippy', 101)]
confirmed = read('process-confirmations.json')
assert confirmed['all_job_ledgers_terminal'] and confirmed['all_recorded_launcher_and_child_pids_absent'] and confirmed['no_live_handles']
link = read('review-support/review-link.json')
checker = base / link['bundled_unmodified_independent_checker']
assert sha(checker) == link['checker_sha256']
spec = importlib.util.spec_from_file_location('independent_saved_review', checker)
review = importlib.util.module_from_spec(spec)
spec.loader.exec_module(review)
output = io.StringIO()
old_argv = sys.argv
try:
    sys.argv = [str(checker), str(base), '--historical', str(base / 'historical')]
    with contextlib.redirect_stdout(output):
        review.main()
finally:
    sys.argv = old_argv
receipt = json.loads(output.getvalue())
assert all(receipt['cases'][name]['checkpoints'][-1]['original_quality'] is False for name in ['control', 'zero-g-balance'])
assert receipt['cases']['no-avatar']['checkpoints'][-1]['original_quality'] is None
# Recompute each retained historical control peak with the independent checker.
historical_meta = read('historical/runs/rest-metadata.json')
historical_data = review.triangle_data(historical_meta)[0]
historical_states = [json.loads(s) for s in (base / 'historical/runs/trajectory.jsonl').read_text().splitlines()]
comparisons = {s['frame']: s for s in read('control-analysis.json')['historical_shared_time_comparison']}
count = 0
for state in historical_states:
    if state['case'] != 'control_8x32':
        continue
    value = review.membrane(historical_data, state['positions_m'])[0]['peak']['absolute_strain']
    review.near(value, comparisons[state['global_frame']]['historical_independent_peak_axis_strain'])
    count += 1
assert count == 13
assert all(v is False for v in receipt['historical']['array_equality'].values())
summary = {'status': 'passed_saved_output_replay', 'source_commit': source['commit'], 'source_files_verified': 37,
           'bundle_files_including_manifest': len(actual), 'snapshot_count': 423, 'step_timing_records': 420,
           'original_control_t1_strain': next(s for s in receipt['cases']['control']['checkpoints'] if s['frame'] == 60)['material']['peak']['absolute_strain'],
           'original_control_t3_strain': receipt['cases']['control']['checkpoints'][-1]['material']['peak']['absolute_strain'],
           'zero_g_t3_strain': receipt['cases']['zero-g-balance']['checkpoints'][-1]['material']['peak']['absolute_strain'],
           'avatar_omission_t1_strain': receipt['cases']['no-avatar']['checkpoints'][-1]['material']['peak']['absolute_strain'],
           'original_quality': 'failed15percent_strain_screen', 'avatar_omission_quality': None,
           'prefix': receipt['prefix'], 'all_original_jobs_terminal': True, 'new_simulations': 0,
           'equilibrium_convergence_reference_parity': 'unknown/not established',
           'review_receipt': 'external/separate; this invocation is producer saved-output replay'}
print(json.dumps(summary, indent=2, allow_nan=False))
