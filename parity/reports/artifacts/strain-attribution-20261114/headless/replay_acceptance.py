"""No-simulator replay: python3 replay_acceptance.py BUNDLE [--source CHECKOUT]."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text())


def prior_diagnostics(value):
    return {k: v for k, v in value.items() if k not in ['peak_strain', 'step_seconds']}


def passed(value):
    return value['finite'] and value['seam_assembly_remaining'] == 0. and value['max_stretch_error'] < .15 and value['max_seam_error'] < .005 and value['max_contact_error'] < .002


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', type=Path)
    parser.add_argument('--source', type=Path)
    args = parser.parse_args()
    base = args.bundle
    plan, binding, manifest = [load(base / p) for p in ['plan.json', 'binary-binding.json', 'source-manifest.json']]
    assert digest(base / 'plan.json') == binding['plan_sha256']
    assert binding['source_manifest_sha256'] == digest(base / 'source-manifest.json') == plan['source_manifest_sha256']
    assert binding['source_commit'] == manifest['commit'] == plan['source_commit'] == '6c45535cb312f7fed8ad872ee87b6ba5e5974e35'
    for name, expected in plan['frozen_files_sha256'].items():
        assert digest(base / name) == expected, name
    assert digest(base / 'review-support/expected-peaks-preregistered.json') == '5f8ffcfc077d41a39aa3084466f88806c85a09b4d6ed004213ecfc16d76bbc0e'
    assert digest(base / 'original-grain-shirt.weftforge') == '82e7c6d398c7772d7e62c28196dcf11845020020d44fca9ac9113e615b9537e7'
    assert digest(base / 'review-support/replay_review.py') == 'ef19fef14b25088d951823c11da0b9f8e1733dd8c8137353e66186c0eb3eabdb'
    assert digest(base / 'harness/src/main.rs') == digest(base / 'original-driver/src/main.rs') == 'b215edd790b44bf7bc353b0f3724d025a158d11f0e6c04c9194a12feafc8c580'
    assert (base / 'harness/Cargo.toml').read_text() == (base / 'original-driver/Cargo.toml').read_text().replace('../source/', '../runtime-source/')
    assert (base / 'harness/Cargo.lock').read_bytes() == (base / 'original-driver/Cargo.lock').read_bytes()
    source_status = 'source checkout not supplied'
    if args.source:
        for name, expected in manifest['files'].items():
            assert digest(args.source / name) == expected, name
        assert subprocess.check_output(['git', '-C', str(args.source), 'rev-parse', 'HEAD'], text=True).strip() == manifest['commit']
        assert subprocess.check_output(['git', '-C', str(args.source), 'status', '--porcelain'], text=True) == ''
        source_status = '37 source hashes and clean original Git checkout verified'
    binary_status = 'not supplied; retained pre-run binding only'
    if (base / 'target/release/cloth-balance-diagnostic').is_file():
        assert digest(base / 'target/release/cloth-balance-diagnostic') == binding['executed_binary_sha256']
        binary_status = 'actual retained release binary hash verified'
    for stage in ['build', 'clippy', 'run']:
        job = load(base / f'{stage}-job.json')
        assert job['status'] == 'completed' and job['exit_code'] == 0 and not job.get('timed_out', False)
    run_job = load(base / 'run-job.json')
    assert plan['declared_at'] < binding['declared_at'] < run_job['started_at']
    assert binding['executed_binary_sha256'] == load(base / 'build-job.json')['binary_sha256']
    spec = importlib.util.spec_from_file_location('independent_original_reviewer', base / 'review-support/replay_review.py')
    independent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(independent)
    independent.self_check()
    baseline_meta, current_meta = [load(base / name / 'rest-metadata.json') for name in ['baseline-control', 'control']]
    assert baseline_meta == current_meta, 'mesh/project/masses/stitches/boundary changed'
    baseline, current, baseline_steps, current_steps = [[json.loads(row) for row in (base / folder / filename).read_text().splitlines()]
                                                     for folder, filename in [('baseline-control', 'trajectory.jsonl'), ('control', 'trajectory.jsonl'), ('baseline-control', 'steps.jsonl'), ('control', 'steps.jsonl')]]
    assert len(baseline) == len(current) == 181 and len(baseline_steps) == len(current_steps) == 180
    data, mass, *_ = independent.triangle_data(current_meta)
    pieces = {p['id']: p for p in current_meta['project']['pieces']}
    expected = {r['frame']: r for r in load(base / 'review-support/expected-peaks-preregistered.json')['rows'] if r['case'] == 'control'}
    assert all(r['trajectory_sha256'] == digest(base / 'baseline-control/trajectory.jsonl') for r in expected.values())
    maxima = {'signed_strain_absolute_error': 0., 'scalar_vs_independent_absolute_error': 0.}
    selected, roundoff, meaningful = [], [], 0
    for frame, (old, new) in enumerate(zip(baseline, current)):
        assert old['global_frame'] == new['global_frame'] == frame
        for key in ['case', 'global_frame', 'settings_last_step', 'phase', 'positions_m', 'velocities_m_per_s']:
            assert old[key] == new[key], (frame, key)
        assert set(new['diagnostics']) == set(old['diagnostics']) | {'peak_strain'}
        assert prior_diagnostics(old['diagnostics']) == prior_diagnostics(new['diagnostics']), (frame, 'prior diagnostics')
        assert passed(old['diagnostics']) == passed(new['diagnostics'])
        if frame:
            old_step, new_step = baseline_steps[frame - 1], current_steps[frame - 1]
            for key in ['global_frame', 'settings']:
                assert old_step[key] == new_step[key]
            assert prior_diagnostics(old_step['diagnostics']) == prior_diagnostics(new_step['diagnostics'])
            assert new_step['diagnostics'] == new['diagnostics']
        calculated = independent.membrane(data, new['positions_m'])[0]['peak']
        peak = new['diagnostics']['peak_strain']
        assert set(peak) == {'triangle', 'axis', 'signed_strain'} and peak['axis'] in ['warp', 'weft']
        assert isinstance(peak['triangle'], int) and 0 <= peak['triangle'] < len(current_meta['triangles'])
        triangle = current_meta['triangles'][peak['triangle']]
        assert len(set(current_meta['piece_ids'][i] for i in triangle)) == 1
        piece = pieces[current_meta['piece_ids'][triangle[0]]]
        error = independent.near(peak['signed_strain'], calculated['signed_strain'], 1e-12)
        scalar_error = independent.near(new['diagnostics']['max_stretch_error'], calculated['absolute_strain'], 1e-12)
        independent.near(abs(peak['signed_strain']), new['diagnostics']['max_stretch_error'], 1e-12)
        maxima['signed_strain_absolute_error'] = max(maxima['signed_strain_absolute_error'], error)
        maxima['scalar_vs_independent_absolute_error'] = max(maxima['scalar_vs_independent_absolute_error'], scalar_error)
        if calculated['absolute_strain'] > 1e-12:
            assert (peak['triangle'], peak['axis'], piece['id']) == (calculated['triangle'], calculated['axis'], calculated['piece']), (frame, peak, calculated)
            meaningful += 1
        else:
            roundoff.append(frame)
        if frame in expected:
            frozen = expected[frame]
            independent.near(peak['signed_strain'], frozen['signed_strain'], 1e-12)
            if frozen['absolute_strain'] > 1e-12:
                assert (peak['triangle'], peak['axis'], piece['id']) == (frozen['triangle'], frozen['axis'], frozen['piece'])
            selected.append({'frame': frame, 'peak_strain': peak, 'triangle_vertices': triangle, 'resolved_authored_piece_id': piece['id'],
                             'resolved_authored_piece_name': piece['name'], 'independent_peak': calculated, 'original_quality_passed': passed(new['diagnostics'])})
    for folder, states in [('baseline-control', baseline), ('control', current)]:
        complete = load(base / folder / 'completed-case.json')
        assert complete['completed_frames'] == 180 and complete['final_diagnostics'] == states[-1]['diagnostics']
    assert not passed(current[60]['diagnostics']) and not passed(current[180]['diagnostics'])
    receipt = {'kind': 'independent_original_control_read_only_strain_attribution_acceptance',
               'source_commit': manifest['commit'], 'source_verification': source_status, 'binary_verification': binary_status,
               'plan_sha256': digest(base / 'plan.json'), 'binary_binding_sha256': digest(base / 'binary-binding.json'),
               'executed_binary_sha256': binding['executed_binary_sha256'],
               'frozen_expected_peaks_sha256': digest(base / 'review-support/expected-peaks-preregistered.json'),
               'baseline_trajectory_sha256': digest(base / 'baseline-control/trajectory.jsonl'), 'new_trajectory_sha256': digest(base / 'control/trajectory.jsonl'),
               'frame_count': 181, 'new_simulation_case_count': 1, 'new_no_avatar_cases': 0, 'new_extra_zero_gravity_cases': 0,
               'mesh_project_masses_stitches_boundary_exact': True, 'positions_velocities_settings_all_prior_diagnostics_exact': True,
               'ignored_comparison_fields': ['peak_strain (new field only)', 'step_seconds', 'measured_step_seconds', 'case/phase/initialization wall seconds'],
               'independent_attribution_all_meaningful_frames_passed': meaningful, 'roundoff_only_frames': roundoff, 'max_absolute_errors': maxima,
               'decisive_authored_piece_mappings': selected, 'unchanged_quality_limits': {'strain_below': .15, 'seam_m_below': .005, 'contact_m_below': .002},
               'original_assembly_quality_passed': False, 'original_final_quality_passed': False,
               'run_job': {k: run_job[k] for k in ['started_at', 'completed_at', 'launcher_pid', 'child_pid', 'child_process_group', 'exit_code']},
               'limits': ['Only read-only original control attribution acceptance; not a physical quality fix, convergence or force equilibrium claim.',
                          'No fresh no-avatar or extra zero-G variants and no extension to the original three-case causal plan.',
                          'Native UI/CLI adapter acceptance and full workspace checks belong to separate root-owned evidence.',
                          'Copied source snapshots and omitted binaries need their declared hash closure; omitted actual Git/binary state remains explicit.']}
    print(json.dumps(receipt, indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
