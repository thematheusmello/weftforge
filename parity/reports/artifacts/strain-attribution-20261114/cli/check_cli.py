"""Three packaged CLI adapter checks; no settings or garment changes."""
from pathlib import Path
import datetime
import hashlib
import importlib.util
import json
import subprocess

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).resolve().parent / 'cli'
EXE = ROOT / '.local/candidates/6c45535cb312/weftforge-cli'
BASELINE = ROOT / 'parity/reports/artifacts/physics-balance-20261114'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    OUT.mkdir(exist_ok=False)
    source = ROOT / 'fixtures/grain-shirt.weftforge'
    original_hash = sha(source)
    assert original_hash == '82e7c6d398c7772d7e62c28196dcf11845020020d44fca9ac9113e615b9537e7'
    assert sha(EXE) == 'c7e63b52d60b293357ba60c49dfed3643472ab177352f2501ebfb207f7ed1092'
    incoming = OUT / 'original.weftforge'
    incoming.write_bytes(source.read_bytes())
    incoming.chmod(0o444)
    old = OUT / 'retained-output.obj'
    old.write_bytes(b'previous independently retained garment output\n')
    sentinel = sha(old)
    oracle = ROOT / '.local/cloth-balance-review-20261114/expected-peaks-preregistered.json'
    assert sha(oracle) == '5f8ffcfc077d41a39aa3084466f88806c85a09b4d6ed004213ecfc16d76bbc0e'
    plan = {'source_commit': '6c45535cb312f7fed8ad872ee87b6ba5e5974e35',
            'binary_sha256': sha(EXE), 'input_sha256': original_hash,
            'checker_sha256': sha(Path(__file__)), 'expected_peaks_sha256': sha(oracle),
            'frozen_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'commands': [['fit', 'original.weftforge', 'retained-output.obj'],
                         ['simulate', 'original.weftforge', 'one-step.obj', '1'],
                         ['benchmark', 'original.weftforge', '1']],
            'criteria': 'Preserve original failed fit/sentinel; exact source and adapter fields; independent material-gradient check within1e-12; no parameter or threshold changes.'}
    (OUT / 'plan.json').write_text(json.dumps(plan, indent=2) + '\n')
    jobs, outputs = [], {}
    for args in plan['commands']:
        name = args[0]
        record = {'command': [str(EXE), *args], 'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat()}
        with (OUT / (name + '.stdout')).open('xb') as stdout, (OUT / (name + '.stderr')).open('xb') as stderr:
            process = subprocess.Popen(record['command'], cwd=OUT, stdout=stdout, stderr=stderr)
            record['pid'] = process.pid
            (OUT / 'current-job.json').write_text(json.dumps(record, indent=2) + '\n')
            try:
                code = process.wait(timeout=120)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
                raise
        record.update(exit_code=code, completed_at=datetime.datetime.now(datetime.timezone.utc).isoformat())
        jobs.append(record)
        (OUT / 'jobs.json').write_text(json.dumps(jobs, indent=2) + '\n')
        assert code == (1 if name == 'fit' else 0), record
        outputs[name] = json.loads((OUT / (name + '.stdout')).read_bytes())
    fit = outputs['fit']
    assert fit['status'] == 'failed' and not fit['export_written'] and sha(old) == sentinel
    for row, piece_id, value in [(fit['phases'][0], 31, .24118490382884605),
                                 (fit['phases'][1], 2, .32351913672966237),
                                 (fit, 2, .32351913672966237)]:
        peak = row['diagnostics']['peak_strain']
        assert row['peak_strain_piece']['id'] == piece_id and peak['axis'] == 'warp'
        assert abs(peak['signed_strain'] - value) < 1e-12
        assert row['diagnostics']['max_stretch_error'] > .15
    spec = importlib.util.spec_from_file_location('independent_review', BASELINE / 'review-support/replay_review.py')
    review = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(review)
    metadata = json.loads((BASELINE / 'control/rest-metadata.json').read_bytes())
    positions = [[float(x) for x in line.split()[1:]] for line in (OUT / 'one-step.obj').read_text().splitlines() if line.startswith('v ')]
    independent = review.membrane(review.triangle_data(metadata)[0], positions)[0]['peak']
    for name in ['simulate', 'benchmark']:
        peak = outputs[name]['diagnostics']['peak_strain']
        assert peak['triangle'] == independent['triangle'] and peak['axis'] == independent['axis']
        assert abs(peak['signed_strain'] - independent['signed_strain']) < 1e-12
        assert outputs[name]['peak_strain_piece']['id'] == independent['piece']
    assert sha(source) == original_hash == sha(incoming)
    assert sha(EXE) == plan['binary_sha256']
    summary = {'status': 'passed', 'jobs': jobs, 'source_unchanged': True,
               'prior_output_unchanged': True, 'original_fit_quality': 'failed',
               'independent_one_step_peak': independent,
               'fit_assembly_peak': fit['phases'][0]['diagnostics']['peak_strain'],
               'fit_drape_peak': fit['diagnostics']['peak_strain']}
    (OUT / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    (OUT / 'current-job.json').write_text(json.dumps({'status': 'completed', 'all_expected_exits': True}) + '\n')
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
