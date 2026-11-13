import copy
import hashlib
import json
import pathlib
import subprocess
import tempfile
import unittest
from unittest import mock
import progress

class RegistryTests(unittest.TestCase):
    def setUp(self):self.data=progress.read('parity/features.yaml')
    def test_registry_is_schema_valid(self):progress.validate(self.data)
    def test_missing_field_rejected(self):
        del self.data['features'][0]['verification_status']
        with self.assertRaises(Exception):progress.validate(self.data)
    def test_unknown_dependency_rejected(self):
        self.data['features'][0]['dependencies']=['MISSING-999']
        with self.assertRaises(ValueError):progress.validate(self.data)
    def test_passing_claim_needs_run(self):
        self.data['features'][0]['verification_status']='passed'
        self.data['features'][0]['verification_runs']=[]
        with self.assertRaises(ValueError):progress.validate(self.data)
    def test_provisional_reference_never_establishes_parity(self):
        f=copy.deepcopy(self.data['features'][0]);f.update(implementation_status='implemented',verification_status='passed',reference_observation_status='observed',freshness='current')
        self.assertFalse(progress.parity_verified(f,{f['id']:f},{'frozen':False}))


class EvidenceHistoryTests(unittest.TestCase):
    """Synthetic git histories exercise real commit and working-source checks."""

    def setUp(self):
        schema = progress.read('parity/schema.json')
        feature = copy.deepcopy(progress.read('parity/features.yaml')['features'][0])
        directory = tempfile.TemporaryDirectory(prefix='weftforge-evidence-')
        self.addCleanup(directory.cleanup)
        self.root = pathlib.Path(directory.name).resolve()
        patch = mock.patch.object(progress, 'ROOT', self.root)
        patch.start()
        self.addCleanup(patch.stop)
        self.write('parity/schema.json', schema)
        self.write('parity/reference.json', {'baseline_id': 'SYNTHETIC-1', 'frozen': True,
                                           'baseline_revision': 1, 'baseline_status': 'frozen'})
        for path in ['src/feature.rs', 'src/dependency.rs', 'tools/evaluator.py', 'tools/generator.py']:
            self.write(path, 'original synthetic source\n')
        self.write('fixtures/input.txt', 'original synthetic fixture\n')
        self.write('parity/scenarios/test.json', {})
        feature.update(id='PAT-001', dependencies=[], implementation_locations=['src/feature.rs'],
                       evidence_ids=[], reference_baseline='SYNTHETIC-1', verification_runs=[],
                       implementation_status='implemented', verification_status='passed',
                       reference_observation_status='observed', freshness='current', scenarios=[{
                           'id': 'SYNTHETIC-001', 'path': 'parity/scenarios/test.json',
                           'fixture_hashes': {'fixtures/input.txt': self.digest('fixtures/input.txt')},
                           'initial_conditions': 'Original synthetic input',
                           'expected_outcomes': 'Synthetic regression only; never published as evidence'}])
        self.feature = feature
        self.data = {'schema_version': 2, 'discovery_complete': False, 'features': [feature]}
        self.git('init', '--quiet', '-b', 'main')
        self.initial_commit = self.commit()
        self.sequence = 0

    def git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.root, stderr=subprocess.PIPE).decode().strip()

    def write(self, path, value):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(json.dumps(value, indent=2) + '\n' if isinstance(value, dict) else value)

    def digest(self, path):
        return hashlib.sha256((self.root / path).read_bytes()).hexdigest()

    def commit(self):
        self.git('add', '.')
        self.git('-c', 'user.name=Evidence tests', '-c', 'user.email=tests@example.invalid',
                 'commit', '--no-gpg-sign', '--quiet', '-m', 'synthetic source checkpoint')
        return self.git('rev-parse', 'HEAD')

    def report(self, name, commit=None, status='passed', freshness='current',
               kind='implementation_only', baseline='SYNTHETIC-1', feature=None, scope=None):
        feature = self.feature if feature is None else feature
        commit = self.initial_commit if commit is None else commit
        self.sequence += 1
        artifact = f'artifacts/{name}.txt'
        self.write(artifact, f'Synthetic {status} diagnostics retained\n')
        source_hashes = {path: hashlib.sha256(subprocess.check_output(
            ['git', 'show', commit + ':' + path], cwd=self.root)).hexdigest()
                         for path in ['src/feature.rs', 'src/dependency.rs',
                                      'tools/evaluator.py', 'tools/generator.py']}
        report = {'id': name, 'commit': commit, 'timestamp': f'2026-01-01T00:00:{self.sequence:02}Z',
                  'platform': 'synthetic test repository', 'reference_baseline': baseline,
                  'kind': kind, 'status': status, 'feature_ids': [feature['id']],
                  'complete_contract': kind == 'reference_comparison',
                  'reference_executed': kind == 'reference_comparison',
                  'source_hashes': source_hashes, 'source_scope': scope or ['src', 'tools/evaluator.py'],
                  'artifact_hashes': {artifact: self.digest(artifact)}, 'freshness': freshness}
        if freshness == 'stale':
            report['freshness_reason'] = 'Superseded synthetic source or baseline; retained history'
        path = f'parity/reports/{name}.json'
        self.write(path, report)
        feature['verification_runs'].append({key: report[key] for key in
                                             ['id', 'commit', 'timestamp', 'platform', 'artifact_hashes']}
                                            | {'path': path})
        return report

    def parity(self, feature=None):
        return progress.parity_verified(feature or self.feature,
                                        {f['id']: f for f in self.data['features']},
                                        progress.read('parity/reference.json'))

    def test_old_failed_source_remains_attributable_after_a_current_rerun(self):
        old = self.report('OLD-FAIL', status='failed', freshness='stale')
        self.write('src/feature.rs', 'independent source correction\n')
        current = self.commit()
        self.report('CURRENT-PASS', commit=current)
        progress.validate(self.data)
        self.assertNotEqual(old['source_hashes']['src/feature.rs'], self.digest('src/feature.rs'))
        self.assertEqual(progress.read('parity/reports/OLD-FAIL.json'), old)
        self.assertEqual(self.feature['scenarios'][0]['fixture_hashes']['fixtures/input.txt'],
                         self.digest('fixtures/input.txt'))
        self.assertFalse(self.parity())

    def test_stale_reference_pass_and_current_implementation_pass_never_establish_parity(self):
        self.report('OLD-REFERENCE', kind='reference_comparison', freshness='stale')
        self.write('src/feature.rs', 'new source with implementation-only checks\n')
        self.report('CURRENT-IMPLEMENTATION', commit=self.commit())
        progress.validate(self.data)
        self.assertFalse(self.parity())

    def test_wrong_committed_source_identity_is_rejected_even_outside_stale_scope(self):
        report = self.report('TAMPERED', freshness='stale')
        report['source_hashes']['tools/generator.py'] = '0' * 64
        self.write('parity/reports/TAMPERED.json', report)
        self.feature.update(freshness='stale', verification_status='stale')
        with self.assertRaisesRegex(ValueError, 'source does not match tested commit'):
            progress.validate(self.data)

    def test_source_change_requires_rerun_or_explicit_stale_history(self):
        report = self.report('REFERENCE', kind='reference_comparison')
        progress.validate(self.data)
        self.assertTrue(self.parity())
        self.write('src/feature.rs', 'changed current runtime source\n')
        with self.assertRaisesRegex(ValueError, 'changed scoped source requires rerun'):
            progress.validate(self.data)
        self.assertFalse(self.parity())
        report.update(freshness='stale', freshness_reason='Source changed; comparison must rerun')
        self.write('parity/reports/REFERENCE.json', report)
        self.feature.update(freshness='stale', verification_status='stale')
        progress.validate(self.data)
        self.assertFalse(self.parity())

    def test_added_and_deleted_scoped_files_require_rerun(self):
        self.report('CURRENT', kind='reference_comparison')
        self.write('src/new.rs', 'new runtime module\n')
        with self.assertRaisesRegex(ValueError, 'changed scoped source requires rerun'):
            progress.validate(self.data)
        (self.root / 'src/new.rs').unlink()
        (self.root / 'src/dependency.rs').unlink()
        with self.assertRaisesRegex(ValueError, 'changed scoped source requires rerun'):
            progress.validate(self.data)

    def test_unrelated_generator_edits_do_not_stale_runtime_evidence(self):
        self.report('REFERENCE', kind='reference_comparison')
        self.write('tools/generator.py', 'unrelated progress generation correction\n')
        progress.validate(self.data)
        self.assertTrue(self.parity())

    def test_old_reference_baseline_is_retained_only_as_stale_history(self):
        self.report('OLD-BASELINE', kind='reference_comparison', freshness='stale', baseline='SYNTHETIC-OLD')
        self.report('CURRENT-IMPLEMENTATION')
        progress.validate(self.data)
        self.assertFalse(self.parity())
        report = progress.read('parity/reports/OLD-BASELINE.json')
        report['freshness'] = 'current'
        self.write('parity/reports/OLD-BASELINE.json', report)
        with self.assertRaisesRegex(ValueError, 'report baseline differs'):
            progress.validate(self.data)

    def test_current_reference_scope_and_dependencies_must_qualify_together(self):
        dependency = copy.deepcopy(self.feature)
        dependency.update(id='PAT-002', implementation_locations=['src/dependency.rs'], verification_runs=[])
        self.data['features'].append(dependency)
        self.feature['dependencies'] = ['PAT-002']
        report = self.report('FEATURE-REFERENCE', kind='reference_comparison', scope=['src/feature.rs'])
        self.report('DEPENDENCY-REFERENCE', kind='reference_comparison', feature=dependency)
        with self.assertRaisesRegex(ValueError, 'scope omits feature or dependency implementation'):
            progress.validate(self.data)
        self.assertFalse(self.parity())
        report['source_scope'] = ['src', 'tools/evaluator.py']
        self.write('parity/reports/FEATURE-REFERENCE.json', report)
        progress.validate(self.data)
        self.assertTrue(self.parity())
        dependency['verification_status'] = 'not_run'
        self.assertFalse(self.parity())

    def test_failed_current_scene_cannot_be_hidden_by_another_passing_scene(self):
        self.report('CURRENT-FAIL', status='failed')
        self.report('DIFFERENT-PASS')
        with self.assertRaisesRegex(ValueError, 'current failed verification must remain visible'):
            progress.validate(self.data)
        self.feature['verification_runs'].pop(0)
        with self.assertRaisesRegex(ValueError, 'current failed verification must remain visible'):
            progress.validate(self.data)

    def test_dashboard_and_workflow_text_use_current_reports_without_claiming_parity(self):
        report = self.report('IDENTIFIED-WORKFLOW')
        report['single_identified_candidate_workflow'] = True
        self.write('parity/reports/IDENTIFIED-WORKFLOW.json', report)
        reference = progress.validate(self.data)
        generated = progress.generate(self.data, reference)
        self.assertIn('Complete identified implementation workflow: **passed**', generated['docs/progress.md'])
        dashboard = generated['docs/dashboard/index.html']
        for text in ['Per-domain status', 'Latest attributable runs', 'Next milestone',
                     'Failed and blocked contracts', 'passed / current', 'Relevant source scope',
                     '/blob/' + self.initial_commit + '/tools/evaluator.py', '/actions', '/releases',
                     '0 / 1 inventoried contracts verified against reference']:
            self.assertIn(text, dashboard)
        report.update(freshness='stale', freshness_reason='Workflow source changed; replay is required')
        self.write('parity/reports/IDENTIFIED-WORKFLOW.json', report)
        self.feature.update(freshness='stale', verification_status='stale')
        reference = progress.validate(self.data)
        generated = progress.generate(self.data, reference)
        self.assertIn('No current complete identified implementation workflow run qualifies', generated['docs/progress.md'])
        self.assertIn('passed / stale', generated['docs/dashboard/index.html'])
