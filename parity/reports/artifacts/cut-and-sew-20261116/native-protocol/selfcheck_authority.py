#!/usr/bin/env python3
"""Synthetic dependency/coverage checks only; never a formal case grade or freeze."""
import copy
import hashlib
import json
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
import grade_saved as G


def run():
    rejected = []

    def bad(name, invoke, message):
        try:
            invoke()
        except AssertionError as problem:
            G.require(str(problem) == message, 'control failed at an unrelated gate: ' + name)
            rejected.append(name)
        else:
            raise AssertionError('synthetic mutant accepted: ' + name)

    identity = G.parsed(G.read(G.ROOT / 'SOURCE-74.json'))
    current_freeze = G.sha(b'SYNTHETIC-ONLY current freeze identity; no FREEZE file exists')
    input_raw = json.dumps(G.models()['redo-r4'], indent=2).encode()
    input_sha = G.sha(input_raw)

    with tempfile.TemporaryDirectory(prefix='SYNTHETIC-AUTHORITY-ONLY-', dir=G.ROOT) as name:
        temporary = Path(name)

        def write(path, value):
            path.parent.mkdir(parents=True, exist_ok=True)
            raw = (json.dumps(value, indent=2) + '\n').encode()
            path.write_bytes(raw)
            return G.sha(raw)

        def linked_pair(label, change=None):
            prior_run, current_run = temporary / label / 'prior', temporary / label / 'current'
            attempt = 'SYNTHETIC-ONLY-' + label
            prior = {'schema': 'weftforge.straight-cut.root-binding.v2', 'case': 'SEW-S1', 'attempt': attempt,
                     'root_authorized_for_case': True, 'native_lease_granted': True,
                     'controller_configuration': {'root': str(prior_run)},
                     'source_commit': identity['source_commit'], 'native_binary_sha256': identity['native_binary_sha256'],
                     'clean_metadata_commit': identity['clean_metadata_commit'],
                     'source_inputs': copy.deepcopy(identity['source_inputs']), 'source_input_count': 74,
                     'freeze_sha256': current_freeze}
            if change:
                change(prior)
            binding_sha = write(prior_run / 'ROOT-BINDING.json', prior)
            snapshot = prior_run / 'snapshots/redo-r4.weftforge'
            snapshot.parent.mkdir(); snapshot.write_bytes(input_raw); snapshot.chmod(0o444)
            staged = current_run / G.CASES['SEW-S2']['input']
            staged.parent.mkdir(parents=True); staged.write_bytes(input_raw); staged.chmod(0o444)
            closed = {'schema': G.BASE.CLOSURE_SCHEMA, 'case': 'SEW-S1', 'attempt': attempt,
                      'native_input_lease_closed': True, 'outcome': 'completed', 'root_binding_sha256': binding_sha,
                      'terminal_source_record': '0007', 'planned_quit_source_record': '0007',
                      'immediate_completion_summary_calls': 1,
                      'completion_summary_return_receipt': 'COMPLETION-SUMMARY-RETURN.json', 'postflight_receipt': 'POSTFLIGHT.json'}
            closure_sha = write(prior_run / 'NATIVE-LEASE-CLOSED.json', closed)
            write(prior_run / 'SAVED-GRADE-ATTEMPT.json', {'case': 'SEW-S1', 'formal_grade_attempt': 1,
                                                       'input_closure_sha256': closure_sha})
            write(prior_run / 'POSTFLIGHT.json', {'root_binding_sha256': binding_sha, 'input_closure_sha256': closure_sha,
                                                'source_inputs_unchanged': True, 'native_binary_unchanged': True})
            grade = {'status': 'passed_saved_files_only', 'case': 'SEW-S1', 'attempt': attempt,
                     'root_binding_sha256': binding_sha, 'source_commit': prior['source_commit'],
                     'native_binary_sha256': prior['native_binary_sha256'], 'snapshot_sha256': {'redo-r4': input_sha},
                     'freeze_sha256': prior['freeze_sha256']}
            grade_sha = write(prior_run / 'SAVED-GRADE-001.json', grade)
            review = {'schema': 'weftforge.straight-cut-and-sew.native-ui-review.v1', 'case': 'SEW-S1', 'attempt': attempt,
                      'root_binding_sha256': binding_sha, 'single_saved_grade_report_sha256': grade_sha,
                      'source_commit': prior['source_commit'], 'native_binary_sha256': prior['native_binary_sha256'],
                      'accepted': True, 'reviewed_raw_images_and_traces': True,
                      'accepted_final_snapshot_label': 'redo-r4', 'accepted_final_snapshot_sha256': input_sha}
            review_sha = write(prior_run / 'ROOT-UI-ACCEPTANCE.json', review)
            bound = {'case': 'SEW-S2', 'source_commit': identity['source_commit'],
                     'native_binary_sha256': identity['native_binary_sha256'],
                     'clean_metadata_commit': identity['clean_metadata_commit'],
                     'source_inputs': copy.deepcopy(identity['source_inputs']), 'source_input_count': 74,
                     'freeze_sha256': current_freeze, 'inputs': {G.CASES['SEW-S2']['input']: input_sha},
                     'accepted_input_dependency': {'case': 'SEW-S1', 'attempt': attempt, 'snapshot_label': 'redo-r4',
                        'root_binding_path': str(prior_run / 'ROOT-BINDING.json'), 'root_binding_sha256': binding_sha,
                        'snapshot_sha256': input_sha, 'saved_grade_report_sha256': grade_sha,
                        'root_native_ui_review_sha256': review_sha}}
            return current_run, bound

        positive_run, positive = linked_pair('positive')
        G.exact(G.dependency(positive_run, positive), G.models()['redo-r4'])
        mutants = [
            ('prior-metadata', lambda prior: prior.update(clean_metadata_commit='52163adb86b9f48fdd06f042e4ddf615797c3947')),
            ('prior-freeze', lambda prior: prior.update(freeze_sha256=G.sha(b'SYNTHETIC-ONLY other accepted packet freeze'))),
            ('prior-source-inventory', lambda prior: prior['source_inputs'].__setitem__('.cargo/config.toml', '0' * 64)),
            ('prior-source-count', lambda prior: prior.update(source_input_count=73)),
        ]
        for label, change in mutants:
            current_run, bound = linked_pair(label, change)
            bad('fully hash-linked accepted prior S1 with wrong ' + label,
                lambda: G.dependency(current_run, bound),
                'SEW-S2 prior S1 metadata/source74/freeze differs from the current packet')

    declared = G.parsed(G.read(G.ROOT / 'PROVENANCE.json'))['external_dependencies']
    frozen = {'status': 'frozen_prospective_native_cut_and_sew_acceptance',
              'files': {name: hashlib.sha256((G.ROOT / name).read_bytes()).hexdigest() for name in G.REQUIRED_PACKET_INPUTS},
              'external_dependencies': copy.deepcopy(declared)}
    G.require({path.name for path in G.ROOT.glob('*.template.json')} <= G.REQUIRED_PACKET_INPUTS,
              'an authority template was omitted from the mandatory set')
    G.freeze_coverage(frozen)
    empty = copy.deepcopy(frozen); empty['files'] = {}
    bad('empty frozen packet file inventory', lambda: G.freeze_coverage(empty), 'incomplete frozen packet input inventory')
    for name in ('EXPECTED-STATES.json', 'PLAN.json', 'SOURCE-74.json', 'grade_saved.py', 'retain_saved.py',
                 'ROOT-UI-ACCEPTANCE.template.json'):
        missing = copy.deepcopy(frozen); del missing['files'][name]
        bad('frozen packet omits ' + name, lambda: G.freeze_coverage(missing), 'incomplete frozen packet input inventory')
    empty = copy.deepcopy(frozen); empty['external_dependencies'] = {}
    bad('empty declared external dependency coverage', lambda: G.freeze_coverage(empty),
        'incomplete or changed declared frozen dependency inventory')
    missing = copy.deepcopy(frozen); del missing['external_dependencies'][next(iter(declared))]
    bad('one declared external dependency omitted', lambda: G.freeze_coverage(missing),
        'incomplete or changed declared frozen dependency inventory')
    changed = copy.deepcopy(frozen); changed['external_dependencies'][next(iter(declared))] = '0' * 64
    bad('declared external dependency hash changed', lambda: G.freeze_coverage(changed),
        'incomplete or changed declared frozen dependency inventory')
    G.require(not (G.ROOT / 'FREEZE.json').exists() and not list(G.ROOT.glob('SYNTHETIC-AUTHORITY-ONLY-*')),
              'selfcheck left a freeze or temporary metadata fixtures')
    return {'status': 'passed_offline_synthetic_dependency_and_freeze_coverage_checks_only',
            'rejected_mutants': rejected, 'positive_same385_source74_currentfreeze_dependency': True,
            'negative_dependencies_relinked_binding_grade_review_closure_attempt_and_postflight': True,
            'dependency_failures_reached_the_new_identity_gate': True,
            'positive_mandatory_file_and_declared_dependency_coverage': True,
            'all_nine_authority_templates_are_mandatory': True,
            'freeze_manifests_were_in_memory_only': True,
            'synthetic_history_files_were_temporary_fixtures_not_formal_grade_attempts': True,
            'product_native_CUA_controller_clock_actual_case_or_formal_grade_executed': False,
            'temporary_synthetic_files_deleted': True, 'freeze_created': False}


if __name__ == '__main__':
    report = Path(sys.argv[1]); G.require(not report.exists(), 'fresh synthetic authority report required')
    result = run()
    with report.open('x') as output:
        json.dump(result, output, indent=2, allow_nan=False); output.write('\n')
    print(json.dumps(result))
