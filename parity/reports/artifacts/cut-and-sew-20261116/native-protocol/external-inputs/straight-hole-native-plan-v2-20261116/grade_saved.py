#!/usr/bin/env python3
"""Prospective hole saved/export grader; never imports or executes WeftForge."""
import argparse
import copy
import hashlib
import importlib.util
import json
import math
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
BASE_PATH = REPO / '.local/straight-cut-native-plan-v2-20261116/grade_saved.py'
assert hashlib.sha256(BASE_PATH.read_bytes()).hexdigest() == 'a9d7d866f7a4639ed57eee94f79428dd4026baba073c95043c2d9bd9a7931d62'
SPEC = importlib.util.spec_from_file_location('frozen_native_protocol', BASE_PATH)
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)
require, read, parsed, sha, exact, item, near, bits, paired, absolute, file_sha = (
    BASE.require, BASE.read, BASE.parsed, BASE.sha, BASE.exact, BASE.item, BASE.near,
    BASE.bits, BASE.paired, BASE.absolute, BASE.file_sha)
SIDECARS = BASE.SIDECARS
PLAN = parsed(read(ROOT / 'PLAN.json'))
CASES = {case: {'labels': spec['labels'], 'revisions': spec['revisions'], 'primary': spec['primary'],
                'cap': spec['operation_cap'], 'seconds': spec['elapsed_cap_s']} for case, spec in PLAN['cases'].items()}
HELPER_TYPES = dict(zip(
    ['hole' + name for name in ('Module', 'Require', 'Fs', 'Path', 'Crypto', 'Process', 'Root', 'Repo', 'Packet',
                                'Hash', 'Write', 'BindingBytes', 'BindingSha', 'Binding', 'Recorder', 'Controller',
                                'Observe', 'Mark', 'ProofId', 'ProofSha')],
    ['object', 'function', 'object', 'object', 'object', 'object', 'string', 'string', 'string', 'function',
     'function', 'object', 'string', 'object', 'object', 'object', 'function', 'function', 'string', 'string']))


def final_binding_complete(bound):
    template = parsed(read(ROOT / ('ROOT-BINDING.' + bound['case'] + '.template.json')))
    require(bound.keys() == template.keys() and bound['root_authorized_for_case'] is True and bound['native_lease_granted'] is True,
            'final binding has extra/missing fields or lacks canonical grants')
    def complete(value):
        return value is not None and (all(complete(v) for v in value.values()) if isinstance(value, dict) else
                                     all(complete(v) for v in value) if isinstance(value, list) else True)
    require(all(complete(value) for key, value in bound.items() if key != 'accepted_h1_dependency') and
            (bound['accepted_h1_dependency'] is None if bound['case'] == 'HOLE-H1' else
             bound['accepted_h1_dependency'].keys() == template['accepted_h1_dependency'].keys() and complete(bound['accepted_h1_dependency'])),
            'final binding retains unresolved required nulls')
    require(bound['source_input_count'] == 73 and bound['source_inputs'] == parsed(read(ROOT / 'SOURCE-73.json'))['source_inputs'],
            'final binding still lacks the canonical named source73 inventory')


def models():
    # Assessment only: original literal R240/defaults, with required v5 holes; never native input.
    parent = parsed(read(ROOT / 'fixtures/R240-v5-assessment.weftforge'))
    hole = copy.deepcopy(parent)
    hole['project'].update(revision=2, next_id=20)
    item(hole, 'pieces', 2)['holes'] = [{'id': 11, 'boundary': [16, 17, 18, 19]}]
    hole['project']['points'] += [{'id': identity, 'position': point} for identity, point in
                                zip((12, 13, 14, 15), ([.08, .12], [.08, .24], [.16, .24], [.16, .12]))]
    hole['project']['curves'] += [{'id': identity, 'start': start, 'end': end, 'controls': None}
                                for identity, start, end in ((16, 12, 13), (17, 13, 14), (18, 14, 15), (19, 15, 12))]
    edited = copy.deepcopy(hole)
    edited['project']['revision'] = 3
    item(edited, 'points', 12)['position'][0] = .06
    undo, redo = copy.deepcopy(hole), copy.deepcopy(edited)
    undo['project']['revision'], redo['project']['revision'] = 4, 5
    return dict(zip(CASES['HOLE-H1']['labels'], (parent, hole, edited, undo, redo)))


def changed_points(actual, expected, identities):
    adjusted = copy.deepcopy(expected)
    for identity in identities:
        target = item(adjusted, 'points', identity)
        position = item(actual, 'points', identity)['position']
        near(position, target['position'], 1e-12, 'declared hole point differs')
        target['position'] = copy.deepcopy(position)
    exact(actual, adjusted)


def grade_states(case, documents, input_r5=None):
    expected = models()
    require(list(documents) == CASES[case]['labels'], 'missing/extra/reordered native snapshot labels')
    if case == 'HOLE-H1':
        exact(documents['parent-r1'], expected['parent-r1'])
        changed_points(documents['hole-r2'], expected['hole-r2'], (12, 13, 14, 15))
        edited = copy.deepcopy(documents['hole-r2'])
        edited['project']['revision'] = 3
        target = item(edited, 'points', 12)['position']
        actual = item(documents['edit-r3'], 'points', 12)['position']
        near([actual[0]], [.06], 1e-12, 'edited U differs')
        target[0] = actual[0]
        exact(documents['edit-r3'], edited)
        for label, source, revision in (('undo-r4', 'hole-r2', 4), ('redo-r5', 'edit-r3', 5)):
            restored = copy.deepcopy(documents[source]); restored['project']['revision'] = revision
            exact(documents[label], restored)
    else:
        require(input_r5 is not None, 'accepted actual H1 R5 is required')
        changed_points(input_r5, expected['redo-r5'], (12, 13, 14, 15))
        edited = copy.deepcopy(input_r5); edited['project']['revision'] = 6
        actual = item(documents['reopened-r6'], 'points', 12)['position']
        near([actual[0]], [.08], 1e-12, 'reopened U differs')
        item(edited, 'points', 12)['position'][0] = actual[0]
        exact(documents['reopened-r6'], edited)
    areas = {}
    for label, doc in documents.items():
        piece = item(doc, 'pieces', 2)
        values = []
        for boundary in [piece['boundary'], *[hole['boundary'] for hole in piece['holes']]]:
            curves = [item(doc, 'curves', identity) for identity in boundary]
            require(all(c['end'] == curves[(n + 1) % len(curves)]['start'] for n, c in enumerate(curves)), 'hole walk is not closed')
            polygon = [item(doc, 'points', c['start'])['position'] for c in curves]
            values.append(math.fsum(a[0] * b[1] - b[0] * a[1] for a, b in zip(polygon, polygon[1:] + polygon[:1])) / 2)
        require(values[0] > 0 and (len(values) == 1 or values[1] < 0), 'outer/hole winding differs')
        area = math.fsum(values)
        target = .0864 if label == 'parent-r1' else (.0756 if label in ('edit-r3', 'redo-r5') else .0768)
        near([area], [target], 1e-10, 'net material area differs')
        areas[label] = area
    return {'full_v5_construction': True, 'unchanged_fields': 'exact f64/signed-zero; color f32',
            'IDs_global_order_Hole11_ownership_highwater_history': 'checked', 'areas_m2': areas,
            'analytic_area_times_density_only_kg': {label: area * .18 for label, area in areas.items()},
            'actual_Cloth_masses_verified': False, 'native_UI_verified': False, 'reference_parity': False}


def identity_preflight(run, bound):
    identity = parsed(read(ROOT / 'SOURCE-73.json'))
    require(bound['source_commit'] == identity['source_commit'] and bound['clean_metadata_commit'] == identity['clean_metadata_commit'] and
            bound['source_input_count'] == 73 and bound['source_inputs'] == identity['source_inputs'], 'wrong named source73/runtime/metadata')
    require(bound['clean_source_metadata_receipt'] == identity['clean_checks'], 'wrong clean metadata receipt')
    source, target = absolute(bound['source_worktree'], True), absolute(bound['own_target_directory'], True)
    bundle, binary = absolute(bound['bundle'], True), absolute(bound['native_binary_path'])
    require(target == source / 'target' and bundle.is_relative_to(run) and bundle.suffix == '.app' and
            binary == bundle / 'Contents/MacOS/weftforge', 'unbound own target/bundle/binary')
    for name, digest in bound['source_inputs'].items():
        require(file_sha(absolute(str(source / name))) == digest, 'source73 file changed: ' + name)
    require(file_sha(binary) == bound['native_binary_sha256'] == identity['native_binary_sha256'] and
            binary.stat().st_size == identity['native_binary_bytes'], 'native binary differs')
    plist_raw = read(bundle / 'Contents/Info.plist')
    plist = BASE.plistlib.loads(plist_raw)
    require(sha(plist_raw) == bound['bundle_plist_sha256'] and plist['CFBundleIdentifier'] == bound['bundle_identifier'] and
            type(bound['bundle_identifier']) is str and re.fullmatch(r'[A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)+', bound['bundle_identifier']) and
            plist['CFBundleExecutable'] == 'weftforge' and plist['CFBundlePackageType'] == 'APPL' and
            re.fullmatch(r'\d+(?:\.\d+)+', bound['os_version']), 'plist/OS identity differs')
    argv = [] if bound['case'] == 'HOLE-H1' else [str(run / 'inputs/accepted-redo-r5.weftforge')]
    require(bound['input_argv'] == argv and bound['launcher_path'] == '/usr/bin/open' and
            bound['launcher_argv'] == ['-n', str(bundle)] + (['--args', *argv] if argv else []), 'wrong empty/actual-input launch argv')
    for name, relative, digest in (
        ('controller', '.local/native-transform-controller-20261115/controller.cjs', '9debdc6b1d42b49dce16c8b1227df0621b579532ca7c5fd76c3a8a5db537790b'),
        ('recorder', '.local/native-length-readouts-v2-20261115/recorder.cjs', '45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc'),
        ('observer', '.local/native-internal-annotations-20261115/C002/OBSERVER-FRAGMENT.cjs', 'd580450b45fc2fe42b19c146fb7023f84d04e7b9596ac24a8f5afe3e81be7729')):
        require(bound[name] == str(REPO / relative) and file_sha(absolute(bound[name])) == bound[name + '_sha256'] == digest, 'immutable native module differs')
    for descriptor in ({'path': bound['actor_filename'], 'sha256': bound['actor_sha256']}, bound['actor_persistence_check'], bound['actor_initial_capture']):
        path = absolute(descriptor['path'])
        require(path.is_relative_to(run) and file_sha(path) == descriptor['sha256'], 'bound actor source differs')
    config = bound['controller_configuration']
    exact(config, {'root': str(run), 'recorderPath': bound['recorder'], 'evidenceKind': 'native',
                   'callCap': CASES[bound['case']]['cap'], 'elapsedCapS': CASES[bound['case']]['seconds'], 'baselineCalls': 0})
    context = {key: value for key, value in config.items() if key != 'root'}
    context['recorderSha256'] = bound['recorder_sha256']
    exact(parsed(read(run / 'CONTEXT.json')), context)
    for key, expected_names, descriptor in (('runtime', ['fmt', 'test', 'clippy', 'debug-build'], identity['runtime_checks']),
                                          ('clean_build', ['xtask', 'debug-build'], identity['clean_checks'])):
        require(bound['integrated_local_gate_receipts'][key] == descriptor, 'wrong integrated gate descriptor')
        checks, path = BASE.receipt(descriptor, descriptor['sha256'])
        require(checks['source_hashes'] == bound['source_inputs'] and checks['source_count'] == 73 and
                checks['all_passed'] is True and checks['checkout_status_after'] == '' and
                [step['name'] for step in checks['steps']] == expected_names, 'source/gate/clean receipt mismatch')
        for step in checks['steps']:
            require(step['exit_code'] == 0, 'integrated gate failed')
            for stream in ('stdout', 'stderr'):
                require(file_sha(absolute(str(path.parent / (step['name'] + '.' + stream)))) == step[stream + '_sha256'], 'gate raw log differs')
        if key == 'clean_build':
            require(checks['runtime_commit'] == bound['source_commit'] and checks['checked_clean_commit'] == bound['clean_metadata_commit'] and
                    checks['source73_unchanged'] is True and checks['checkout_clean_before'] is True and checks['checkout_clean_after'] is True,
                    'clean metadata/source receipt differs')
            built = absolute(checks['binaries']['weftforge']['path'])
            require(built == target / 'debug/weftforge' and file_sha(built) == bound['native_binary_sha256'], 'build original differs')


def readiness_preflight(run, bound, binding_raw):
    ready = parsed(read(run / 'ACTOR-PERSISTENCE-CHECK-RESULT.json'))
    expected = {'schema': 'weftforge.straight-hole.actor-readiness.v1', 'case': bound['case'], 'attempt': bound['attempt'],
                'root_binding_sha256': sha(binding_raw), 'source_record': '0003', 'actual_cua_runtime': True, 'native_UI_calls': 0,
                'helper_types': HELPER_TYPES, 'helper_flags': {key: True for key in HELPER_TYPES}, 'helpers_ready': True,
                'root': {'expected': str(run), 'actual': str(run)},
                'controller_configuration': {'expected': bound['controller_configuration'], 'actual': bound['controller_configuration']},
                'context_schema_ready': True, 'proof_schema_ready': True, 'proof_result_ready': True,
                'offline_artifact_ready': True, 'controller_status_ready': True,
                'proof_sha256': sha(read(run / 'ACTOR-RUNTIME-PROOF.json'))}
    exact(ready, expected)
    before, after = paired(run, '0003')
    require(before['method'] == 'local-runtime-bindings-check', 'independent FS readiness is not record0003')
    exact(after['result'], ready)
    launch, _ = paired(run, '0004')
    require(launch['method'] == 'external.open' and launch['arguments']['executable'] == bound['launcher_path'] and
            launch['arguments']['argv'] == bound['launcher_argv'], 'owned launch did not follow independent readiness')
    pid_before, pid_after = paired(run, '0005')
    matches = pid_after['result']['exact_matches']
    require(pid_before['method'] == 'external.pid' and pid_before['arguments']['expected_binary'] == bound['native_binary_path'] and
            pid_after['result']['status'] == 0 and len(matches) == 1 and type(matches[0]['pid']) is int and matches[0]['pid'] > 0 and
            matches[0]['command'] == ' '.join([bound['native_binary_path'], *bound['input_argv']]), 'wrong live owned PID')
    require(paired(run, '0006')[0]['method'] == 'getApp' and paired(run, '0006')[0]['arguments']['args'] == [bound['bundle']] and
            paired(run, '0007')[0]['method'] == 'getAXStateAndScreenshot', 'initial target binding/capture route differs')


def dependency(run, bound):
    dep = bound['accepted_h1_dependency']
    prior = absolute(dep['root_binding_path']); prior_run = prior.parent
    prior_raw = read(prior); prior_bound = parsed(prior_raw)
    require(prior.name == 'ROOT-BINDING.json' and sha(prior_raw) == dep['root_binding_sha256'] and
            prior_bound['schema'] == 'weftforge.straight-cut.root-binding.v2' and prior_bound['controller_configuration']['root'] == str(prior_run) and
            prior_bound['case'] == 'HOLE-H1' and prior_bound['attempt'] == dep['attempt'] and
            prior_bound['root_authorized_for_case'] is True and prior_bound['native_lease_granted'] is True, 'wrong accepted H1 binding')
    grade_raw, review_raw = read(prior_run / 'SAVED-GRADE-001.json'), read(prior_run / 'ROOT-UI-ACCEPTANCE.json')
    grade, review = parsed(grade_raw), parsed(review_raw)
    raw = read(run / 'inputs/accepted-redo-r5.weftforge')
    require(not (run / 'inputs/accepted-redo-r5.weftforge').stat().st_mode & 0o222 and
            raw == read(prior_run / 'snapshots/redo-r5.weftforge') and sha(raw) == dep['snapshot_sha256'] and
            bound['inputs'] == {'inputs/accepted-redo-r5.weftforge': sha(raw)}, 'actual H1 R5 bytes/input binding differ')
    require(sha(grade_raw) == dep['saved_grade_report_sha256'] == review['single_saved_grade_report_sha256'] and
            sha(review_raw) == dep['root_native_ui_review_sha256'] and
            grade['root_binding_sha256'] == review['root_binding_sha256'] == sha(prior_raw) and
            grade['case'] == review['case'] == 'HOLE-H1' and grade['attempt'] == review['attempt'] == dep['attempt'] and
            grade['source_commit'] == review['source_commit'] == prior_bound['source_commit'] == bound['source_commit'] and
            grade['native_binary_sha256'] == review['native_binary_sha256'] == prior_bound['native_binary_sha256'] == bound['native_binary_sha256'] and
            grade['status'] == 'passed_saved_files_only' and grade['snapshot_sha256']['redo-r5'] == sha(raw) and
            review['schema'] == 'weftforge.straight-hole.native-ui-review.v1' and review['accepted'] is True and
            review['reviewed_raw_images_and_traces'] is True and review['accepted_redo_r5_sha256'] == sha(raw), 'H1 single grade/UI/run acceptance mismatch')
    return parsed(raw)


# Frozen Cut V2 raw-copy receipt checker; local hole case table, unchanged semantics.
def check_receipts(run, case, synthetic=False):
    documents, raw_states, last_capture = {}, {}, 0
    spec = CASES[case]
    for index, label in enumerate(spec['labels']):
        snapshot = run / 'snapshots' / (label + '.weftforge')
        raw = read(snapshot)
        receipt = parsed(read(run / 'snapshots' / (label + '.receipt.json')))
        require(not snapshot.stat().st_mode & 0o222, 'snapshot is not read-only')
        previous = spec['labels'][index - 1] if index else None
        require(receipt['schema'] == 'weftforge.straight-cut.raw-copy.v1' and
                receipt['evidence_kind'] == ('synthetic_selfcheck' if synthetic else 'native_saved_bytes') and receipt['case'] == case and
                receipt['label'] == label and type(receipt['revision']) is int and receipt['revision'] == spec['revisions'][index] and
                receipt['previous_saved_label'] == previous and receipt['primary_relative_path'] == spec['primary'] and
                receipt['primary_sha256'] == receipt['snapshot_sha256'] == sha(raw) and receipt['bytes'] == len(raw) and
                receipt['construction_graded'] is False and receipt['raw_bytes_unedited'] is True,
                'raw copy receipt metadata/hash differs: ' + label)
        save, capture, retained = [int(receipt[key]) for key in
                                   ('native_save_record', 'clean_capture_record', 'raw_copy_record')]
        require(last_capture < save < capture < retained, 'save/capture/raw-copy chronology differs')
        last_capture = retained
        for key in ('native_save_record', 'clean_capture_record', 'raw_copy_record'):
            paired(run, receipt[key])
        require(paired(run, receipt['clean_capture_record'])[0]['method'] == 'getAXStateAndScreenshot',
                'clean capture is not an ordinary raw observation')
        require(paired(run, receipt['raw_copy_record'])[0]['method'] == 'fs.raw_snapshot', 'raw retention lacks FS operation')
        require(list(receipt['sidecars']) == list(SIDECARS), 'missing/extra/reordered sidecar inventory')
        for suffix, entry in receipt['sidecars'].items():
            path = run / 'snapshots' / (label + suffix)
            if entry['present']:
                sidecar = read(path)
                require(not path.stat().st_mode & 0o222 and sha(sidecar) == entry['sha256'] and
                        len(sidecar) == entry['bytes'], 'retained raw sidecar differs')
            else:
                require(entry == {'present': False, 'bytes': None, 'sha256': None} and not path.exists() and not path.is_symlink(),
                        'sidecar absence differs')
        recovery = receipt['sidecars']['.recovery']
        if previous is None:
            require(recovery['present'] is False, 'fresh SaveAs unexpectedly had recovery')
        else:
            require(recovery['present'] is True and read(run / 'snapshots' / (label + '.recovery')) == raw_states[previous],
                    'recovery is not exact previous-primary bytes')
        documents[label], raw_states[label] = parsed(raw), raw
    final = spec['labels'][-1]
    require(read(run / spec['primary']) == raw_states[final], 'final native primary differs from last retained bytes')
    final_receipt = parsed(read(run / 'snapshots' / (final + '.receipt.json')))
    for suffix, entry in final_receipt['sidecars'].items():
        path = Path(str(run / spec['primary']) + suffix)
        require(not path.is_symlink() and path.exists() == entry['present'], 'final original sidecar presence differs')
        if entry['present']:
            require(read(path) == read(run / 'snapshots' / (final + suffix)), 'final original sidecar bytes differ')
    return documents, {label: sha(raw) for label, raw in raw_states.items()}


def grade_export_receipts(run, construction):
    import grade_exports
    result = {}
    prior = int(parsed(read(run / 'snapshots/reopened-r6.receipt.json'))['raw_copy_record'])
    for name in ('hole-r6.obj', 'hole-r6.svg', 'hole-r6.dxf'):
        primary, snapshot = run / 'exports' / name, run / 'exports-retained' / name
        raw = read(snapshot)
        receipt = parsed(read(run / 'exports-retained' / (name + '.receipt.json')))
        require(not snapshot.stat().st_mode & 0o222 and raw == read(primary) and
                receipt['schema'] == 'weftforge.straight-hole.export-copy.v1' and receipt['case'] == 'HOLE-H2' and
                receipt['revision'] == 6 and receipt['label'] == name and receipt['sha256'] == sha(raw) and
                receipt['bytes'] == len(raw) and receipt['construction_graded'] is False, 'export raw-copy receipt differs')
        action, capture, copied = (int(receipt[key]) for key in ('ui_action_record', 'clean_capture_record', 'raw_copy_record'))
        require(prior < action < capture < copied, 'unordered export action/capture/copy')
        prior = copied
        for key in ('ui_action_record', 'clean_capture_record', 'raw_copy_record'):
            paired(run, receipt[key])
        require(paired(run, receipt['clean_capture_record'])[0]['method'] == 'getAXStateAndScreenshot' and
                paired(run, receipt['raw_copy_record'])[0]['method'] == 'fs.raw_snapshot', 'export observation/FS origin differs')
        for suffix in SIDECARS:
            entry = receipt['sidecars'][suffix]
            path, copied_path = Path(str(primary) + suffix), Path(str(snapshot) + suffix)
            require(not path.is_symlink() and not copied_path.is_symlink(), 'symlink export sidecar')
            if entry['present']:
                sidecar = read(copied_path)
                require(not copied_path.stat().st_mode & 0o222 and read(path) == sidecar and
                        entry['bytes'] == len(sidecar) and entry['sha256'] == sha(sidecar), 'export sidecar bytes differ')
            else:
                require(entry == {'present': False, 'bytes': None, 'sha256': None} and not path.exists() and not copied_path.exists(), 'export sidecar absence differs')
        if name.endswith('.dxf'):
            require(raw == read(ROOT / 'fixtures/hole-r6.dxf') and read(Path(str(snapshot) + '.recovery')) ==
                    read(ROOT / 'fixtures/hole-r6.dxf.recovery') and receipt['intentional_refusal'] is True, 'DXF refusal changed sentinel/recovery')
            result['DXF'] = 'intentional refusal; original destination/recovery bytes exact'
        else:
            require(receipt['intentional_refusal'] is False and not any(entry['present'] for entry in receipt['sidecars'].values()), 'fresh export unexpectedly had sidecars')
            result[name.rsplit('.', 1)[1].upper()] = grade_exports.grade_obj(raw, construction) if name.endswith('.obj') else grade_exports.grade_svg(raw, construction)
    return result


def postflight(run, bound, binding_raw, closed):
    expected = {'schema': 'weftforge.straight-cut.postflight.v1', 'case': bound['case'], 'attempt': bound['attempt'],
                'root_binding_sha256': sha(binding_raw), 'input_closure_sha256': sha(read(run / 'NATIVE-LEASE-CLOSED.json')),
                'source_commit': bound['source_commit'], 'native_binary_sha256': bound['native_binary_sha256'],
                'source_inputs_sha256': BASE.inventory_sha(bound), 'controller_context_sha256': sha(read(run / 'CONTEXT.json')),
                'runtime_proof_sha256': sha(read(run / 'ACTOR-RUNTIME-PROOF.json')),
                'runtime_tool_history_sha256': sha(read(run / 'ACTOR-RUNTIME-TOOL-HISTORY.json')),
                'actor_readiness_sha256': sha(read(run / 'ACTOR-PERSISTENCE-CHECK-RESULT.json')),
                'terminal_source_record': closed['terminal_source_record'],
                'paired_native_external_operations': len(list((run / 'trace').glob('*-before.json'))),
                'completion_summary_return_sha256': sha(read(run / 'COMPLETION-SUMMARY-RETURN.json')),
                'source_inputs_unchanged': True, 'native_binary_unchanged': True}
    exact(parsed(read(run / 'POSTFLIGHT.json')), expected)


def grade_run(case, run, bound, binding_raw, closed):
    require(bound['freeze_sha256'] == sha(read(ROOT / 'FREEZE.json')), 'wrong final hole packet binding')
    identity_preflight(run, bound)
    BASE.runtime_preflight(run, bound, binding_raw)
    readiness_preflight(run, bound, binding_raw)
    postflight(run, bound, binding_raw, closed)
    clock = int(parsed(read(run / 'LAUNCH-CLOCK.json'))['monotonic_ns'])
    latest = clock
    records = sorted((run / 'trace').glob('*-before.json'))
    spec = CASES[case]
    require(7 <= len(records) <= spec['cap'], 'missing trace/lab operation cap exceeded')
    for index, path in enumerate(records, 1):
        record = str(index).zfill(4)
        require(path.name == record + '-before.json', 'noncontiguous case trace')
        before, after = paired(run, record)
        require(latest <= int(before['monotonic_ns']) <= int(after['monotonic_ns']), 'clock chronology differs')
        latest = int(after['monotonic_ns'])
    summary = parsed(read(run / 'COMPLETION-SUMMARY-RETURN.json'))
    require(summary['schema'] == 'weftforge.straight-cut.completion-summary-return.v1' and summary['case'] == case and
            summary['attempt'] == bound['attempt'] and summary['root_binding_sha256'] == sha(binding_raw) and
            type(summary['summary_calls']) is int and summary['summary_calls'] == 1 and
            summary['paired_native_external_operations'] == len(records) and type(summary['documentation_refresh_calls']) is int and
            summary['documentation_refresh_calls'] >= 0 and summary['overall_actual_calls'] ==
            len(records) + 1 + summary['documentation_refresh_calls'] <= spec['cap'] + 1 and
            summary['summary_invocation_time_exact'] is False and summary['native_controller_pair_claimed_for_summary'] is False,
            'summary/refresh/binding count differs')
    lower, upper = (int(summary[key]['monotonic_ns']) for key in
                    ('lower_bound_before_summary_invocation', 'upper_bound_after_summary_return'))
    require(clock <= lower <= upper and latest <= upper and (upper - clock) / 1e9 <= spec['seconds'] and
            closed['terminal_source_record'] == str(len(records)).zfill(4), 'continuous lab clock/terminal record differs')
    require(case != 'HOLE-H1' or bound['inputs'] == {}, 'H1 must author from actual empty, without generated input')
    documents, hashes = check_receipts(run, case)
    result = grade_states(case, documents, dependency(run, bound) if case == 'HOLE-H2' else None)
    if case == 'HOLE-H2':
        result['exports'] = grade_export_receipts(run, documents['reopened-r6'])
    require(closed['outcome'] == 'completed', 'native case stopped unverified')
    return {'status': 'passed_saved_files_only', 'case': case, 'attempt': bound['attempt'],
            'source_commit': bound['source_commit'], 'native_binary_sha256': bound['native_binary_sha256'],
            'root_binding_sha256': sha(binding_raw), 'freeze_sha256': sha(read(ROOT / 'FREEZE.json')),
            'snapshot_sha256': hashes, 'result': result, 'native_ui_verified': False, 'reference_verified': False,
            'full_feature_accepted': False}


def verify_freeze():
    freeze = parsed(read(ROOT / 'FREEZE.json'))
    for name, digest in freeze['files'].items():
        require(sha(read(ROOT / name)) == digest, 'hole packet artifact changed: ' + name)
    for name, digest in freeze['external_dependencies'].items():
        require(sha(read(REPO / name)) == digest, 'pinned dependency changed: ' + name)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--case', choices=CASES, required=True)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    require(args.run.is_absolute() and args.run == args.run.resolve(), 'absolute canonical owned run required')
    run = args.run
    binding_raw = read(run / 'ROOT-BINDING.json'); bound = parsed(binding_raw)
    closed = BASE.closure(run, args.case, binding_raw)
    BASE.grade_scope(run, args.report, args.case, bound, closed)
    final_binding_complete(bound)
    with (run / 'SAVED-GRADE-ATTEMPT.json').open('x') as output:
        json.dump({'case': args.case, 'formal_grade_attempt': 1,
                   'input_closure_sha256': sha(read(run / 'NATIVE-LEASE-CLOSED.json'))}, output)
    try:
        verify_freeze()
        result = grade_run(args.case, run, bound, binding_raw, closed)
    except Exception as problem:
        result = {'status': 'failed_saved_files_or_preflight', 'case': args.case, 'attempt': bound['attempt'],
                  'root_binding_sha256': sha(binding_raw), 'native_ui_verified': False,
                  'error_type': type(problem).__name__, 'error': str(problem)}
    with args.report.open('x') as output:
        json.dump(result, output, indent=2, allow_nan=False); output.write('\n')
    print(json.dumps(result))
    return 0 if result['status'] == 'passed_saved_files_only' else 1


if __name__ == '__main__':
    sys.exit(main())
