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
                                           'baseline_revision': 1, 'baseline_status': 'frozen',
                                           'product': 'Synthetic test product',
                                           'installer': {'sha256': '1' * 64},
                                           'application': {'bundle_id': 'invalid.example.synthetic',
                                                           'executable_sha256': '2' * 64,
                                                           'observed_ui_version': '1.0', 'observed_ui_build': '123',
                                                           'observed_runtime_architecture': 'synthetic'},
                                           'host': {'os': 'Synthetic OS', 'version': '1.0',
                                                    'build': '123', 'architecture': 'synthetic'},
                                           'settings': {'units': 'metres', 'locale': 'en-US',
                                                        'preferences': {'default_length': 1.0},
                                                        'graphics': {'renderer': 'synthetic'},
                                                        'license_entitlement': None}})
        for path in ['src/feature.rs', 'src/dependency.rs', 'tools/evaluator.py', 'tools/generator.py']:
            self.write(path, 'original synthetic source\n')
        self.write('fixtures/input.txt', 'original synthetic fixture\n')
        self.write('parity/scenarios/test.json', {'actions': ['Run original synthetic scenario']})
        self.write('spec/features/test.md', 'Reviewed synthetic behavior contract; never real reference evidence.\n')
        feature.update(id='PAT-001', dependencies=[], implementation_locations=['src/feature.rs'],
                       evidence_ids=[], reference_baseline='SYNTHETIC-1', verification_runs=[],
                       contract_documents={'spec/features/test.md': self.digest('spec/features/test.md')},
                       implementation_status='implemented', verification_status='passed',
                       reference_observation_status='observed', freshness='current', scenarios=[{
                           'id': 'SYNTHETIC-001', 'path': 'parity/scenarios/test.json',
                           'fixture_hashes': {'fixtures/input.txt': self.digest('fixtures/input.txt')},
                           'initial_conditions': 'Original synthetic input',
                           'expected_outcomes': 'Synthetic regression only; never published as evidence'}])
        self.feature = feature
        self.data = {'schema_version': 2, 'discovery_complete': False, 'features': [feature]}
        self.write('parity/features.yaml', self.data)
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
        if kind == 'reference_comparison' and baseline == 'SYNTHETIC-1':
            report['reference_qualification'] = progress.reference_qualification(
                report, self.data, progress.read('parity/reference.json'))
        if freshness == 'stale':
            report['freshness_reason'] = 'Superseded synthetic source or baseline; retained history'
        path = f'parity/reports/{name}.json'
        self.write(path, report)
        feature['verification_runs'].append({key: report[key] for key in
                                             ['id', 'commit', 'timestamp', 'platform', 'artifact_hashes']}
                                            | {'path': path})
        return report

    def test_publication_requires_tracked_evidence_even_when_ignored_file_exists(self):
        self.report('UNPUBLISHED')
        self.write('.gitignore', 'artifacts/*.txt\n')
        self.git('add', 'parity/reports')
        with self.assertRaisesRegex(ValueError, 'untracked public evidence: artifacts/UNPUBLISHED.txt'):
            progress.validate(self.data, require_tracked=True)
        self.git('add', '-f', 'artifacts/UNPUBLISHED.txt')
        progress.validate(self.data, require_tracked=True)

    def parity(self, feature=None):
        return progress.parity_verified(feature or self.feature,
                                        {f['id']: f for f in self.data['features']},
                                        progress.read('parity/reference.json'))

    def public_source_bridge(self, available_origin=None):
        for index in range(84):
            self.write(f'bridge-inputs/{index}.rs', f'original synthetic input {index}\n')
        tested = 'a' * 40
        if available_origin:
            tested = self.commit()
            if available_origin == 'mismatch':
                self.write('src/feature.rs', 'public source differs from the available origin\n')
            else:
                self.write('public-metadata.txt', 'public metadata does not change source inputs\n')
        public = self.commit()
        report = self.report('BRIDGED', commit=public)
        report['commit'] = tested
        self.feature['verification_runs'][-1]['commit'] = report['commit']
        hashes = report['source_hashes'] | {
            f'bridge-inputs/{index}.rs': self.digest(f'bridge-inputs/{index}.rs')
            for index in range(84)}

        def artifact(name, value):
            path = f'artifacts/bridge/{name}.json'
            self.write(path, value)
            digest = self.digest(path)
            report['artifact_hashes'][path] = digest
            return {'path': path, 'sha256': digest, 'original_sha256': digest}

        frozen = artifact('freeze', {'commit': report['commit'], 'source88': hashes,
                                    'input_modes': dict.fromkeys(hashes, 0o644)})
        result = artifact('result', {'commit': report['commit'], 'source88': hashes,
                                    'source_freeze_sha256': frozen['original_sha256'],
                                    'all4_stages_passed': True})
        final = artifact('final', {'commit': report['commit'], 'all4_stages_passed': True,
                                  'source_freeze_sha256': frozen['original_sha256'],
                                  'result_sha256': result['original_sha256']})
        integration = artifact('integration', {'integration_commit': public,
            'all88_origin_gate_public_hashes_exact': True,
            'source88_mapping': {path: {'source_gate_sha256': digest,
                                       'public_integration_sha256': digest}
                                 for path, digest in hashes.items()}})
        proof = {'kind': 'implementation_source_equivalence', 'tested_commit': report['commit'],
                 'public_commit': public, 'source_count': 88, 'source_hashes': hashes,
                 'source_modes': dict.fromkeys(hashes, '0644'),
                 'anchors': {'source_gate_freeze': frozen, 'source_gate_result': result,
                             'source_gate_final_manifest': final,
                             'source_integration_receipt': integration}}
        report['public_source_equivalence'] = {'path': 'artifacts/bridge/proof.json', 'sha256': ''}
        self.store_public_source_bridge(report, proof)
        return report, proof, public

    def store_public_source_bridge(self, report, proof):
        descriptor = report['public_source_equivalence']
        self.write(descriptor['path'], proof)
        descriptor['sha256'] = self.digest(descriptor['path'])
        report['artifact_hashes'][descriptor['path']] = descriptor['sha256']
        self.write('parity/reports/' + report['id'] + '.json', report)

    def test_public_source_bridge_retains_tested_identity_without_private_git_object(self):
        report, _, public = self.public_source_bridge()
        reference = progress.validate(self.data)
        generated = progress.generate(self.data, reference)
        self.assertIn('commit `' + report['commit'] + '`', generated['docs/progress.md'])
        self.assertIn('Source-equivalent public revision `' + public + '` (not executed)',
                      generated['docs/progress.md'])
        dashboard = generated['docs/dashboard/index.html']
        self.assertIn('tested commit <code>' + report['commit'] + '</code>', dashboard)
        self.assertNotIn('/commit/' + report['commit'], dashboard)
        self.assertIn('/blob/' + public + '/tools/evaluator.py', dashboard)
        self.assertIn('source-equivalent: tools/evaluator.py', dashboard)
        self.assertFalse(self.parity())

    def test_public_source_bridge_accepts_matching_available_origin(self):
        report, _, public = self.public_source_bridge(available_origin='matching')
        self.assertNotEqual(report['commit'], public)
        progress.validate(self.data)
        self.assertFalse(self.parity())

    def test_public_source_bridge_rejects_available_origin_blob_mismatch(self):
        report, _, _ = self.public_source_bridge(available_origin='mismatch')
        with self.assertRaisesRegex(ValueError, 'available tested tree differs from public inputs'):
            progress.validate(self.data)

    def test_public_source_bridge_rejects_changed_hash_revision_and_public_bytes(self):
        report, proof, _ = self.public_source_bridge()
        original_report, original_proof = copy.deepcopy(report), copy.deepcopy(proof)
        report['source_hashes']['src/feature.rs'] = '0' * 64
        self.store_public_source_bridge(report, proof)
        with self.assertRaisesRegex(ValueError, 'report source hashes differ'):
            progress.validate(self.data)
        report = copy.deepcopy(original_report)
        proof['public_commit'] = self.initial_commit
        self.store_public_source_bridge(report, proof)
        with self.assertRaisesRegex(ValueError, 'frozen source evidence differs'):
            progress.validate(self.data)
        proof = copy.deepcopy(original_proof)
        self.write('src/feature.rs', 'different public source bytes\n')
        proof['public_commit'] = self.commit()
        anchor = proof['anchors']['source_integration_receipt']
        integration = progress.read(anchor['path'])
        integration['integration_commit'] = proof['public_commit']
        self.write(anchor['path'], integration)
        anchor['sha256'] = anchor['original_sha256'] = self.digest(anchor['path'])
        report['artifact_hashes'][anchor['path']] = anchor['sha256']
        self.store_public_source_bridge(report, proof)
        with self.assertRaisesRegex(ValueError, 'public tree differs from original inputs'):
            progress.validate(self.data)

    def test_public_source_bridge_rejects_missing_extra_roster_and_scope_paths(self):
        report, proof, _ = self.public_source_bridge()
        for change in ['missing', 'extra', 'mode']:
            with self.subTest(change=change):
                altered = copy.deepcopy(proof)
                if change == 'missing':
                    del altered['source_hashes']['bridge-inputs/0.rs']
                elif change == 'extra':
                    altered['source_hashes']['extra.rs'] = '0' * 64
                else:
                    altered['source_modes']['bridge-inputs/0.rs'] = '0755'
                self.store_public_source_bridge(report, altered)
                with self.assertRaisesRegex(ValueError, 'complete source roster differs'):
                    progress.validate(self.data)
        self.store_public_source_bridge(report, proof)
        del report['source_hashes']['src/feature.rs']
        self.store_public_source_bridge(report, proof)
        with self.assertRaisesRegex(ValueError, 'source scope lacks tested source identities'):
            progress.validate(self.data)
        report['source_hashes']['src/feature.rs'] = proof['source_hashes']['src/feature.rs']
        report['source_scope'].append('bridge-inputs')
        self.store_public_source_bridge(report, proof)
        with self.assertRaisesRegex(ValueError, 'source scope lacks tested source identities'):
            progress.validate(self.data)
        report['source_hashes']['../outside.rs'] = '0' * 64
        self.store_public_source_bridge(report, proof)
        with self.assertRaisesRegex(ValueError, 'report source hashes differ'):
            progress.validate(self.data)

    def test_public_source_bridge_rejects_unbound_artifact_and_acceptance_promotion(self):
        report, proof, _ = self.public_source_bridge()
        descriptor = report['public_source_equivalence']
        descriptor['sha256'] = '0' * 64
        self.write('parity/reports/BRIDGED.json', report)
        with self.assertRaisesRegex(ValueError, 'artifact hash differs'):
            progress.validate(self.data)
        for key, value in [('kind', 'reference_comparison'), ('complete_contract', True),
                           ('reference_executed', True), ('reference_verified', True),
                           ('single_identified_candidate_workflow', True),
                           ('full_garment_accepted', True), ('reference_qualification', {})]:
            with self.subTest(promotion=key):
                altered = copy.deepcopy(report)
                altered[key] = value
                self.store_public_source_bridge(altered, proof)
                with self.assertRaisesRegex(ValueError, 'cannot qualify reference or full workflow'):
                    progress.validate_sources(altered)

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
        with self.assertRaisesRegex(progress.jsonschema.ValidationError, 'reference_qualification'):
            progress.validate(self.data)

    def assert_requirement_change(self, change):
        report = self.report('REFERENCE', kind='reference_comparison')
        progress.validate(self.data)
        self.assertTrue(self.parity())
        original = copy.deepcopy(report)
        change()
        self.write('parity/features.yaml', self.data)
        with self.assertRaisesRegex(ValueError, 'reference qualification requires rerun'):
            progress.validate(self.data)
        self.assertFalse(self.parity())
        self.assertEqual(progress.read('parity/reports/REFERENCE.json'), original)
        report.update(freshness='stale', freshness_reason='Normative inputs changed; retained original comparison')
        self.write('parity/reports/REFERENCE.json', report)
        self.feature.update(freshness='stale', verification_status='stale')
        progress.validate(self.data)
        self.assertEqual(report['reference_qualification'], original['reference_qualification'])
        self.assertEqual(report['artifact_hashes'], original['artifact_hashes'])
        self.assertFalse(self.parity())

    def change_reference(self, key, value):
        reference = progress.read('parity/reference.json')
        reference[key] = value
        self.write('parity/reference.json', reference)

    def test_baseline_revision_change_requires_stale_comparison(self):
        self.assert_requirement_change(lambda: self.change_reference('baseline_revision', 2))

    def test_reference_executable_identity_change_requires_stale_comparison(self):
        application = progress.read('parity/reference.json')['application'] | {'executable_sha256': '3' * 64}
        self.assert_requirement_change(lambda: self.change_reference('application', application))

    def test_reference_observed_build_change_requires_stale_comparison(self):
        application = progress.read('parity/reference.json')['application'] | {'observed_ui_build': '124'}
        self.assert_requirement_change(lambda: self.change_reference('application', application))

    def test_reference_host_identity_change_requires_stale_comparison(self):
        host = progress.read('parity/reference.json')['host'] | {'version': '2.0'}
        self.assert_requirement_change(lambda: self.change_reference('host', host))

    def test_reference_installer_identity_change_requires_stale_comparison(self):
        self.assert_requirement_change(lambda: self.change_reference('installer', {'sha256': '4' * 64}))

    def test_new_evidence_and_provenance_do_not_stale_reference_comparison(self):
        self.report('REFERENCE', kind='reference_comparison')
        reference = progress.read('parity/reference.json')
        reference.update(evidence=[{'id': 'synthetic observation'}], latest_access_observation={'state': 'synthetic'},
                         blockers=['Synthetic delivery status'])
        reference['host']['support_assessment'] = 'New synthetic support prose'
        reference['application']['garment_workspace_observed'] = True
        reference['installer']['retrieval'] = 'New synthetic provenance prose'
        self.write('parity/reference.json', reference)
        progress.validate(self.data)
        self.assertTrue(self.parity())

    def test_settings_change_requires_stale_comparison(self):
        settings = progress.read('parity/reference.json')['settings']
        settings['units'] = 'centimetres'
        self.assert_requirement_change(lambda: self.change_reference('settings', settings))

    def test_contract_text_change_requires_stale_comparison(self):
        self.assert_requirement_change(lambda: self.feature.update(required_reference_behavior='Revised synthetic behavior'))

    def test_reviewed_specification_change_requires_stale_comparison(self):
        def change():
            self.write('spec/features/test.md', 'Revised reviewed synthetic contract.\n')
            self.feature['contract_documents']['spec/features/test.md'] = self.digest('spec/features/test.md')
        self.assert_requirement_change(change)

    def test_scenario_definition_change_requires_stale_comparison(self):
        self.assert_requirement_change(lambda: self.write('parity/scenarios/test.json', {'actions': ['Revised action']}))

    def test_scenario_registry_conditions_change_requires_stale_comparison(self):
        self.assert_requirement_change(lambda: self.feature['scenarios'][0].update(initial_conditions='Revised conditions'))

    def test_comparison_units_and_tolerances_change_require_stale_comparison(self):
        def change():
            self.feature['comparison_criteria'].update(units='millimetres', numerical_tolerances=[{
                'metric': 'synthetic distance', 'unit': 'mm', 'absolute': 0.1,
                'justification': 'Synthetic regression only'}])
        self.assert_requirement_change(change)

    def test_original_fixture_change_requires_stale_comparison(self):
        def change():
            self.write('fixtures/input.txt', 'revised original synthetic fixture\n')
            self.feature['scenarios'][0]['fixture_hashes']['fixtures/input.txt'] = self.digest('fixtures/input.txt')
        self.assert_requirement_change(change)

    def test_complete_contract_requires_known_frozen_settings_and_complete_feature_state(self):
        report = self.report('REFERENCE', kind='reference_comparison')
        progress.validate(self.data)
        self.assertTrue(self.parity())

        reference = progress.read('parity/reference.json')
        for key, value in [('frozen', False), ('settings', {}),
                           ('settings', reference['settings'] | {'units': None}),
                           ('settings', reference['settings'] | {'graphics': {'renderer': None}})]:
            with self.subTest(key=key, value=value):
                self.write('parity/reference.json', reference | {key: value})
                self.assertFalse(self.parity())
        self.write('parity/reference.json', reference)
        for key, value in [('implementation_status', 'partial'), ('verification_status', 'not_run'),
                           ('reference_observation_status', 'documentation_only'), ('freshness', 'stale')]:
            original = self.feature[key]
            self.feature[key] = value
            self.assertFalse(self.parity())
            self.feature[key] = original
        for key in ['complete_contract', 'reference_executed']:
            report[key] = False
            self.write('parity/reports/REFERENCE.json', report)
            self.assertFalse(self.parity())
            report[key] = True
        self.write('parity/reports/REFERENCE.json', report)
        reference['settings']['license_entitlement'] = 'synthetic account state excluded from behavior settings'
        self.write('parity/reference.json', reference)
        progress.validate(self.data)
        self.assertTrue(self.parity())

    def test_frozen_settings_require_nonempty_typed_technical_records(self):
        reference = progress.read('parity/reference.json')
        self.assertTrue(progress.reference_frozen(reference))
        for key, values in {
                'units': [None, '', ' \t ', {}, [], 1, {'length': ' '}],
                'locale': [None, '', ' \t ', {}, []],
                'preferences': [None, {}, '', ' ', []],
                'graphics': [None, {}, '', ' ', []]}.items():
            for value in values:
                with self.subTest(key=key, value=value):
                    changed = copy.deepcopy(reference)
                    changed['settings'][key] = value
                    self.assertFalse(progress.reference_frozen(changed))
        reference['settings'].update(units={'length': 'metres'}, preferences={'custom_shortcuts': []})
        self.assertTrue(progress.reference_frozen(reference))

    def test_frozen_reference_requires_known_binary_build_and_host_anchors(self):
        reference = progress.read('parity/reference.json')
        for section, keys in {
                None: ['product'], 'installer': ['sha256'],
                'application': ['bundle_id', 'executable_sha256', 'observed_ui_version',
                                'observed_ui_build', 'observed_runtime_architecture'],
                'host': ['os', 'version', 'build', 'architecture']}.items():
            for key in keys:
                for value in [None, '', ' \t ']:
                    with self.subTest(section=section, key=key, value=value):
                        changed = copy.deepcopy(reference)
                        (changed if section is None else changed[section])[key] = value
                        self.assertFalse(progress.reference_frozen(changed))
        for section, key in [('installer', 'sha256'), ('application', 'executable_sha256')]:
            changed = copy.deepcopy(reference)
            changed[section][key] = 'not-a-sha256'
            self.assertFalse(progress.reference_frozen(changed))
        reference['application']['observed_ui_build'] = 123
        self.assertTrue(progress.reference_frozen(reference))

    def test_matching_comparison_with_unknown_binary_identity_cannot_promote(self):
        reference = progress.read('parity/reference.json')
        del reference['application']['executable_sha256']
        self.write('parity/reference.json', reference)
        self.report('UNKNOWN-BINARY', kind='reference_comparison', commit=self.commit())
        progress.validate(self.data)
        self.assertFalse(self.parity())

    def test_complete_pass_requires_reviewed_documents_but_exploration_does_not(self):
        del self.feature['contract_documents']
        self.write('parity/features.yaml', self.data)
        report = self.report('NO-REVIEWED-SPEC', kind='reference_comparison', commit=self.commit())
        with self.assertRaisesRegex(ValueError, 'complete comparison requires reviewed contract documents'):
            progress.validate(self.data)
        self.assertFalse(self.parity())
        report['complete_contract'] = False
        self.write('parity/reports/NO-REVIEWED-SPEC.json', report)
        progress.validate(self.data)
        self.assertFalse(self.parity())

    def test_failed_executed_reference_run_requires_matching_qualification(self):
        report = self.report('REFERENCE-FAIL', kind='reference_comparison', status='failed')
        self.feature['verification_status'] = 'failed'
        progress.validate(self.data)
        self.change_reference('baseline_revision', 2)
        with self.assertRaisesRegex(ValueError, 'reference qualification requires rerun'):
            progress.validate(self.data)
        report.update(freshness='stale', freshness_reason='Baseline revised; retain failed diagnostics')
        self.write('parity/reports/REFERENCE-FAIL.json', report)
        self.feature['freshness'] = 'stale'
        progress.validate(self.data)
        self.assertEqual(progress.read('parity/reports/REFERENCE-FAIL.json')['status'], 'failed')

    def test_executed_reference_outcomes_need_qualification_even_when_failed(self):
        for status in ['passed', 'failed']:
            report = self.report('UNQUALIFIED-' + status, kind='reference_comparison', status=status)
            del report['reference_qualification']
            self.write('parity/reports/' + report['id'] + '.json', report)
            with self.assertRaisesRegex(progress.jsonschema.ValidationError, 'reference_qualification'):
                progress.validate(self.data)
            report.update(freshness='stale', freshness_reason='Legacy unqualified comparison; retain history')
            self.write('parity/reports/' + report['id'] + '.json', report)
        self.feature.update(freshness='stale', verification_status='stale')
        progress.validate(self.data)
        self.assertFalse(self.parity())

    def test_nonexecuted_blocked_reference_attempt_may_remain_unqualified(self):
        report = self.report('BLOCKED', status='blocked')
        report['kind'] = 'reference_comparison'
        self.write('parity/reports/BLOCKED.json', report)
        self.feature['verification_status'] = 'blocked'
        progress.validate(self.data)
        self.assertFalse(self.parity())

    def test_qualification_cannot_be_rewritten_to_new_requirements_even_in_stale_history(self):
        report = self.report('REFERENCE', kind='reference_comparison')
        self.feature['required_reference_behavior'] = 'Later synthetic behavior'
        report['reference_qualification'] = progress.reference_qualification(
            report, self.data, progress.read('parity/reference.json'))
        for freshness in ['current', 'stale']:
            report.update(freshness=freshness, freshness_reason='Retained synthetic history')
            self.write('parity/reports/REFERENCE.json', report)
            with self.assertRaisesRegex(ValueError, 'qualification differs from tested commit'):
                progress.validate(self.data)
            self.assertFalse(self.parity())

    def test_transitive_dependency_contract_change_stales_parent_even_after_dependency_rerun(self):
        dependency = copy.deepcopy(self.feature)
        dependency.update(id='PAT-002', dependencies=['PAT-003'], verification_runs=[],
                          implementation_locations=['src/dependency.rs'])
        transitive = copy.deepcopy(dependency)
        transitive.update(id='PAT-003', dependencies=[])
        self.write('spec/features/dependency.md', 'Original reviewed transitive synthetic contract.\n')
        transitive['contract_documents'] = {'spec/features/dependency.md': self.digest('spec/features/dependency.md')}
        self.data['features'].extend([dependency, transitive])
        self.feature['dependencies'] = ['PAT-002']
        self.write('parity/features.yaml', self.data)
        commit = self.commit()
        reports = [self.report(feature['id'] + '-REFERENCE', feature=feature, commit=commit,
                               kind='reference_comparison') for feature in self.data['features']]
        progress.validate(self.data)
        self.assertTrue(self.parity())
        self.assertEqual(set(reports[0]['reference_qualification']['contracts']), {'PAT-001', 'PAT-002', 'PAT-003'})
        self.write('spec/features/dependency.md', 'Revised reviewed transitive synthetic contract.\n')
        transitive['contract_documents']['spec/features/dependency.md'] = self.digest('spec/features/dependency.md')
        reports[-1].update(freshness='stale', freshness_reason='Dependency specification revised')
        self.write('parity/reports/PAT-003-REFERENCE.json', reports[-1])
        self.write('parity/features.yaml', self.data)
        self.report('TRANSITIVE-RERUN', feature=transitive, commit=self.commit(), kind='reference_comparison')
        self.assertTrue(self.parity(transitive))
        self.assertFalse(self.parity())
        with self.assertRaisesRegex(ValueError, 'reference qualification requires rerun'):
            progress.validate(self.data)
        for feature, report in zip(self.data['features'][:2], reports[:2]):
            report.update(freshness='stale', freshness_reason='Dependency specification revised; rerun parent')
            self.write('parity/reports/' + report['id'] + '.json', report)
            feature.update(freshness='stale', verification_status='stale')
        progress.validate(self.data)

    def test_dependency_without_reviewed_documents_cannot_promote_complete_parent(self):
        dependency = copy.deepcopy(self.feature)
        dependency.update(id='PAT-002', verification_runs=[], implementation_locations=['src/dependency.rs'])
        del dependency['contract_documents']
        self.data['features'].append(dependency)
        self.feature['dependencies'] = ['PAT-002']
        self.write('parity/features.yaml', self.data)
        self.report('PARENT-REFERENCE', kind='reference_comparison', commit=self.commit())
        with self.assertRaisesRegex(ValueError, 'complete comparison requires reviewed contract documents'):
            progress.validate(self.data)
        self.assertFalse(self.parity())

    def test_current_reference_scope_and_dependencies_must_qualify_together(self):
        dependency = copy.deepcopy(self.feature)
        dependency.update(id='PAT-002', implementation_locations=['src/dependency.rs'], verification_runs=[])
        self.data['features'].append(dependency)
        self.feature['dependencies'] = ['PAT-002']
        self.write('parity/features.yaml', self.data)
        commit = self.commit()
        report = self.report('FEATURE-REFERENCE', commit=commit, kind='reference_comparison', scope=['src/feature.rs'])
        self.report('DEPENDENCY-REFERENCE', commit=commit, kind='reference_comparison', feature=dependency)
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
