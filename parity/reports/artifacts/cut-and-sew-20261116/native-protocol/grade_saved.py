#!/usr/bin/env python3
"""Prospective independent saved-state grader; never imports or runs WeftForge.

No actual grade is permitted while FREEZE.json is absent. Synthetic checks call
only the pure state, readiness and opaque-copy helpers, never main/grade_run.
"""
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
FROZEN_PATH = REPO / '.local/straight-hole-native-plan-v2-20261116/grade_saved.py'
assert hashlib.sha256(FROZEN_PATH.read_bytes()).hexdigest() == '486328f13c49018e240cbbd3f52e8b1cf9dd0d97747d3653ee3b927d3b2a31b9'
SPEC = importlib.util.spec_from_file_location('frozen_saved_metadata', FROZEN_PATH)
FROZEN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FROZEN)
BASE = FROZEN.BASE
require, read, parsed, sha, exact, item, near, bits, paired, absolute, file_sha = (
    BASE.require, BASE.read, BASE.parsed, BASE.sha, BASE.exact, BASE.item, BASE.near,
    BASE.bits, BASE.paired, BASE.absolute, BASE.file_sha)
SIDECARS = BASE.SIDECARS
PLAN = parsed(read(ROOT / 'PLAN.json'))
CASES = {case: {'labels': spec['labels'], 'revisions': spec['revisions'], 'primary': spec['primary'],
                'cap': spec['operation_cap'], 'seconds': spec['elapsed_cap_s'],
                'input': spec['input_relative_path']} for case, spec in PLAN['cases'].items()}
# Reuse only these two generic receipt routines with the new strict case table.
FROZEN.CASES = CASES
check_receipts, postflight = FROZEN.check_receipts, FROZEN.postflight
HELPER_TYPES = dict(zip(
    ['sew' + name for name in ('Module', 'Require', 'Fs', 'Path', 'Crypto', 'Process', 'Root', 'Repo', 'Packet',
                               'Hash', 'Write', 'BindingBytes', 'BindingSha', 'Binding', 'Recorder', 'Controller',
                               'Observe', 'Mark', 'ProofId', 'ProofSha')],
    ['object', 'function', 'object', 'object', 'object', 'object', 'string', 'string', 'string', 'function',
     'function', 'object', 'string', 'object', 'object', 'object', 'function', 'function', 'string', 'string']))
REQUIRED_PACKET_INPUTS = {
    'README.md', 'DRAFT.json', 'PLAN.json', 'SOURCE-74.json', 'ORACLE.json', 'EXPECTED-STATES.json',
    'PROVENANCE.json', 'RUNTIME-PROTOCOL.json', 'grade_saved.py', 'retain_saved.py',
    'ROOT-BINDING.SEW-S1.template.json', 'ROOT-BINDING.SEW-S2.template.json',
    'ROOT-UI-ACCEPTANCE.template.json', 'ACTOR-RUNTIME-PROOF.template.json',
    'ACTOR-PERSISTENCE-CHECK-RESULT.template.json', 'ACTOR-RUNTIME-TOOL-HISTORY.template.json',
    'NATIVE-LEASE-CLOSED.template.json', 'POSTFLIGHT.template.json', 'COMPLETION-SUMMARY-RETURN.template.json',
}


def complete(value):
    return value is not None and (all(complete(v) for v in value.values()) if isinstance(value, dict) else
                                 all(complete(v) for v in value) if isinstance(value, list) else True)


def closed_keys(actual, template, path='binding'):
    if isinstance(template, dict):
        require(type(actual) is dict, 'wrong object: ' + path)
        if path == 'binding.source_inputs':
            return
        require(actual.keys() == template.keys(), 'missing/extra fields: ' + path)
        for key in template:
            closed_keys(actual[key], template[key], path + '.' + key)


def final_binding_complete(bound):
    require(bound.get('case') in CASES, 'wrong prospective case')
    template = parsed(read(ROOT / ('ROOT-BINDING.' + bound['case'] + '.template.json')))
    closed_keys(bound, template)
    require(complete(bound) and bound['root_authorized_for_case'] is True and bound['native_lease_granted'] is True,
            'final binding has unresolved required nulls or lacks canonical grants')
    identity = parsed(read(ROOT / 'SOURCE-74.json'))
    require(complete(identity) and re.fullmatch(r'[0-9a-f]{40}', identity['clean_metadata_commit']) and
            len(bound['source_inputs']) == bound['source_input_count'] == identity['source_input_count'] == 74 and
            bound['source_inputs'] == identity['source_inputs'], 'source74/clean metadata is still pending or differs')


def models():
    literal = parsed(read(ROOT / 'EXPECTED-STATES.json'))
    require(literal['kind'] == 'independent_literal_assessment_only' and literal['never_native_input'] is True and
            literal['derived_from_product_implementation'] is False, 'wrong independent literal model')
    return copy.deepcopy(literal['states'])


def assess_sewn(actual, expected):
    adjusted = copy.deepcopy(expected)
    for identity in (11, 13, 16, 17):
        position = item(actual, 'points', identity)['position']
        near(position, item(adjusted, 'points', identity)['position'], 1e-12, 'new anchor differs')
        item(adjusted, 'points', identity)['position'] = copy.deepcopy(position)
    exact(item(actual, 'points', 16)['position'], item(actual, 'points', 11)['position'])
    exact(item(actual, 'points', 17)['position'], item(actual, 'points', 13)['position'])
    exact(actual, adjusted)


def walk(doc, identity):
    piece = item(doc, 'pieces', identity)
    curves = [item(doc, 'curves', curve) for curve in piece['boundary']]
    require(len(curves) == 4 and all(curve['controls'] is None and
            curve['end'] == curves[(index + 1) % 4]['start'] for index, curve in enumerate(curves)),
            'straight directed boundary is not closed')
    points = [curve['start'] for curve in curves]
    require(len(set(points)) == 4 and len({curve['id'] for curve in curves}) == 4, 'repeated boundary identity')
    polygon = [item(doc, 'points', point)['position'] for point in points]
    edges = list(zip(polygon, polygon[1:] + polygon[:1]))
    area = math.fsum(a[0] * b[1] - b[0] * a[1] for a, b in edges) / 2
    require(area > 0, 'boundary winding changed')
    return set(points), {curve['id'] for curve in curves}, area, math.fsum(math.dist(a, b) for a, b in edges)


def interval_position(doc, interval, fraction):
    curve = item(doc, 'curves', interval['curve'])
    start, end = [item(doc, 'points', curve[key])['position'] for key in ('start', 'end')]
    t = interval['t0'] + fraction * (interval['t1'] - interval['t0'])
    return [a + t * (b - a) for a, b in zip(start, end)]


def measure_cut(doc, edited=False, has_seam=True):
    left, right = walk(doc, 2), walk(doc, 15)
    require(left[0].isdisjoint(right[0]) and left[1].isdisjoint(right[1]) and
            left[0] | right[0] == {point['id'] for point in doc['project']['points']} and
            left[1] | right[1] == {curve['id'] for curve in doc['project']['curves']},
            'children share construction or leave unowned points/curves')
    identities = [obj['id'] for kind in ('points', 'curves', 'pieces', 'materials', 'seams', 'sewing_groups')
                  for obj in doc['project'][kind]]
    require(len(identities) == len(set(identities)), 'duplicate global object identity')
    targets = [.0324, .0600 if edited else .0540]
    near([left[2], right[2]], targets, 1e-12, 'analytic child areas differ')
    near([left[3], right[3]], [.9, 1.1 if edited else 1.02], 1e-12, 'analytic child perimeters differ')
    near([math.fsum([left[2], right[2]])], [.0924 if edited else .0864], 1e-12, 'analytic union area differs')
    density = item(doc, 'materials', 1)['areal_density']
    masses = [left[2] * density, right[2] * density]
    near(masses, [.005832, .0108 if edited else .009720], 1e-12, 'analytic area times density differs')
    exact(item(doc, 'points', 16)['position'], item(doc, 'points', 11)['position'])
    if not edited:
        exact(item(doc, 'points', 17)['position'], item(doc, 'points', 13)['position'])
    pairs = []
    if has_seam:
        seam = item(doc, 'seams', 20)
        for fraction in (0., .25, .5, .75, 1.):
            a, b = [interval_position(doc, seam[side], fraction) for side in ('a', 'b')]
            near(a, [.09, .36 * fraction], 1e-12, 'saved seam A pairing differs')
            near(b, [.09, .36 * fraction], 1e-12, 'saved seam B pairing differs')
            near(a, b, 1e-12, 'saved seam rest endpoint/sample mismatch')
            pairs.append({'fraction': fraction, 'uv_a': a, 'uv_b': b})
        lengths = [math.dist(interval_position(doc, seam[side], 0.), interval_position(doc, seam[side], 1.))
                   for side in ('a', 'b')]
        near(lengths, [.36, .36], 1e-12, 'saved seam lengths differ')
    else:
        require(doc['project']['seams'] == [], 'removed seam remained')
    return {'areas_m2': [left[2], right[2]], 'perimeters_m': [left[3], right[3]],
            'analytic_area_times_density_only_kg': masses, 'analytic_saved_interval_pairs': pairs}


def grade_states(case, documents, accepted_input):
    expected = models()
    require(list(documents) == CASES[case]['labels'], 'missing/extra/reordered state labels')
    measured = {}
    if case == 'SEW-S1':
        exact(accepted_input, expected['parent-r1'])
        exact(documents['parent-r1'], expected['parent-r1'])
        assess_sewn(documents['sewn-r2'], expected['sewn-r2'])
        exact(documents['undo-r3'], expected['undo-r3'])
        restored = copy.deepcopy(documents['sewn-r2']); restored['project']['revision'] = 4
        exact(documents['redo-r4'], restored)
        for label in ('sewn-r2', 'redo-r4'):
            measured[label] = measure_cut(documents[label])
    else:
        assess_sewn(accepted_input, expected['redo-r4'])
        measure_cut(accepted_input)
        removed = copy.deepcopy(accepted_input); removed['project'].update(revision=5, seams=[])
        exact(documents['removed-r5'], removed)
        edited = copy.deepcopy(removed); edited['project']['revision'] = 6
        for identity in (5, 17):
            actual = item(documents['child-height-r6'], 'points', identity)['position']
            target = item(edited, 'points', identity)['position']
            require(bits(actual[0]) == bits(target[0]), 'height edit changed unchanged U bits')
            near([actual[1]], [.40], 1e-12, 'independent child height differs')
            target[1] = actual[1]
        exact(documents['child-height-r6'], edited)
        measured['removed-r5'] = measure_cut(documents['removed-r5'], has_seam=False)
        measured['child-height-r6'] = measure_cut(documents['child-height-r6'], edited=True, has_seam=False)
    return {'full_v5_closed_construction': True, 'global_ids_order_walks_ownership_highwater_history': 'checked',
            'unchanged_fields': 'IEEE f64 bits including signed zero; RGB f32 bits',
            'new_anchor_and_height_tolerance_m': 1e-12, 'anchor_copies_bit_exact_until_declared_height_edit': True,
            'analytic_saved_geometry': measured, 'actual_mesh_or_Cloth_particle_mass_verified': False,
            'native_ui_verified': False, 'reference_verified': False, 'empty_authoring_verified': False}


def identity_preflight(run, bound):
    identity = parsed(read(ROOT / 'SOURCE-74.json'))
    require(bound['source_commit'] == identity['source_commit'] == '5898f1a1274f8632d96b5fd0898f8253dee2f4db' and
            bound['clean_metadata_commit'] == identity['clean_metadata_commit'] and
            bound['source_input_count'] == 74 and bound['source_inputs'] == identity['source_inputs'] and
            bound['clean_source_metadata_receipt'] == identity['clean_checks'], 'wrong source74/runtime/clean metadata')
    source, target = absolute(bound['source_worktree'], True), absolute(bound['own_target_directory'], True)
    bundle, binary = absolute(bound['bundle'], True), absolute(bound['native_binary_path'])
    require(target == source / 'target' and bundle.is_relative_to(run) and bundle.suffix == '.app' and
            binary == bundle / 'Contents/MacOS/weftforge', 'unbound own source/default target/bundle/binary')
    for name, digest in bound['source_inputs'].items():
        require(re.fullmatch(BASE.HEX, digest) and file_sha(absolute(str(source / name))) == digest,
                'source74 file changed: ' + name)
    require(file_sha(binary) == bound['native_binary_sha256'] == identity['native_binary_sha256'] and
            binary.stat().st_size == identity['native_binary_bytes'], 'native binary differs')
    plist_raw = read(bundle / 'Contents/Info.plist'); plist = BASE.plistlib.loads(plist_raw)
    require(sha(plist_raw) == bound['bundle_plist_sha256'] and plist['CFBundleIdentifier'] == bound['bundle_identifier'] and
            type(bound['bundle_identifier']) is str and re.fullmatch(r'[A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)+', bound['bundle_identifier']) and
            plist['CFBundleExecutable'] == 'weftforge' and plist['CFBundlePackageType'] == 'APPL' and
            type(bound['os_version']) is str and re.fullmatch(r'\d+(?:\.\d+)+', bound['os_version']), 'plist/OS identity differs')
    argv = [str(run / CASES[bound['case']]['input'])]
    require(bound['input_argv'] == argv and bound['launcher_path'] == '/usr/bin/open' and
            bound['launcher_argv'] == ['-n', str(bundle), '--args', *argv], 'wrong one-input launch argv or extra draft flags')
    pins = parsed(read(ROOT / 'PROVENANCE.json'))['external_dependencies']
    for name, relative in (
        ('controller', '.local/native-transform-controller-20261115/controller.cjs'),
        ('recorder', '.local/native-length-readouts-v2-20261115/recorder.cjs'),
        ('observer', '.local/native-internal-annotations-20261115/C002/OBSERVER-FRAGMENT.cjs')):
        require(bound[name] == str(REPO / relative) and file_sha(absolute(bound[name])) == bound[name + '_sha256'] == pins[relative],
                'immutable native module differs')
    for descriptor in ({'path': bound['actor_filename'], 'sha256': bound['actor_sha256']}, bound['actor_persistence_check'], bound['actor_initial_capture']):
        path = absolute(descriptor['path'])
        require(path.is_relative_to(run) and file_sha(path) == descriptor['sha256'], 'bound actor source differs')
    config = bound['controller_configuration']
    exact(config, {'root': str(run), 'recorderPath': bound['recorder'], 'evidenceKind': 'native',
                   'callCap': CASES[bound['case']]['cap'], 'elapsedCapS': CASES[bound['case']]['seconds'], 'baselineCalls': 0})
    context = {key: value for key, value in config.items() if key != 'root'}; context['recorderSha256'] = bound['recorder_sha256']
    exact(parsed(read(run / 'CONTEXT.json')), context)
    for key, names, descriptor in (('runtime', ['fmt', 'test', 'clippy', 'debug-build'], identity['runtime_checks']),
                                   ('clean_build', ['xtask', 'debug-build'], identity['clean_checks'])):
        require(bound['integrated_local_gate_receipts'][key] == descriptor, 'wrong integrated gate descriptor')
        checks, path = BASE.receipt(descriptor, descriptor['sha256'])
        require(checks['source_hashes'] == bound['source_inputs'] and checks['source_count'] == 74 and
                checks['all_passed'] is True and checks['checkout_status_after'] == '' and
                checks['checkout_clean_before'] is True and checks['checkout_clean_after'] is True and
                checks['source74_unchanged'] is True and [step['name'] for step in checks['steps']] == names,
                'source74/gate/clean receipt mismatch')
        for step in checks['steps']:
            require(type(step['exit_code']) is int and step['exit_code'] == 0, 'integrated gate failed')
            for stream in ('stdout', 'stderr'):
                require(file_sha(absolute(str(path.parent / (step['name'] + '.' + stream)))) == step[stream + '_sha256'],
                        'gate raw log differs')
        require(checks['runtime_commit'] == bound['source_commit'], 'gate runtime differs')
        if key == 'runtime':
            require(checks['target_directory'] == str(target) and checks['cross_worktree_target_copy'] is False,
                    'runtime target provenance differs')
        else:
            require(checks['checked_clean_commit'] == bound['clean_metadata_commit'] and
                    checks['fresh_owned_default_target_no_cache_copy'] is True, 'clean metadata/default-target provenance differs')
        built = checks['binaries']['weftforge']; built_path = absolute(built['path'])
        require(built_path == target / 'debug/weftforge' and built['sha256'] == bound['native_binary_sha256'] == file_sha(built_path) and
                built['bytes'] == binary.stat().st_size == built_path.stat().st_size, 'build original/bundle identities differ')


def readiness_preflight(run, bound, binding_raw):
    ready = parsed(read(run / 'ACTOR-PERSISTENCE-CHECK-RESULT.json'))
    expected = {'schema': 'weftforge.straight-cut-and-sew.actor-readiness.v1', 'case': bound['case'], 'attempt': bound['attempt'],
                'root_binding_sha256': sha(binding_raw), 'source_record': '0003', 'actual_cua_runtime': True, 'native_UI_calls': 0,
                'helper_types': HELPER_TYPES, 'helper_flags': {key: True for key in HELPER_TYPES}, 'helpers_ready': True,
                'root': {'expected': str(run), 'actual': str(run)},
                'controller_configuration': {'expected': bound['controller_configuration'], 'actual': bound['controller_configuration']},
                'context_schema_ready': True, 'proof_schema_ready': True, 'proof_result_ready': True,
                'offline_artifact_ready': True, 'controller_status_ready': True,
                'proof_sha256': sha(read(run / 'ACTOR-RUNTIME-PROOF.json'))}
    exact(ready, expected)
    before, after = paired(run, '0003')
    require(before['method'] == 'local-runtime-bindings-check', 'independent next-cell FS readiness is not record0003')
    exact(after['result'], ready)
    launch, _ = paired(run, '0004')
    require(launch['method'] == 'external.open', 'launch did not follow independent readiness')
    exact(launch['arguments'], {'executable': bound['launcher_path'], 'argv': bound['launcher_argv']})
    pid_before, pid_after = paired(run, '0005')
    command = ' '.join([bound['native_binary_path'], *bound['input_argv']])
    require(pid_before['method'] == 'external.pid', 'wrong live-PID route')
    exact(pid_before['arguments'], {'expected_binary': bound['native_binary_path'], 'expected_command': command})
    matches = pid_after['result']['exact_matches']
    require(pid_after['result']['status'] == 0 and len(matches) == 1 and type(matches[0]['pid']) is int and
            matches[0]['pid'] > 0 and matches[0]['command'] == command, 'wrong exact own live binary+full-argv PID')
    require(paired(run, '0006')[0]['method'] == 'getApp' and paired(run, '0006')[0]['arguments']['args'] == [bound['bundle']] and
            paired(run, '0007')[0]['method'] == 'getAXStateAndScreenshot', 'initial live binding/capture route differs')


def dependency(run, bound):
    dep = bound['accepted_input_dependency']
    prior = absolute(dep['root_binding_path']); prior_run = prior.parent
    prior_raw = read(prior); prior_bound = parsed(prior_raw)
    expected_case, label = ('HOLE-H1', 'parent-r1') if bound['case'] == 'SEW-S1' else ('SEW-S1', 'redo-r4')
    require(dep['case'] == expected_case and dep['snapshot_label'] == label and prior.name == 'ROOT-BINDING.json' and
            sha(prior_raw) == dep['root_binding_sha256'] and prior_bound['schema'] == 'weftforge.straight-cut.root-binding.v2' and
            prior_bound['controller_configuration']['root'] == str(prior_run) and prior_bound['case'] == expected_case and
            prior_bound['attempt'] == dep['attempt'] and prior_bound['root_authorized_for_case'] is True and
            prior_bound['native_lease_granted'] is True, 'wrong accepted native input binding/run')
    if bound['case'] == 'SEW-S1':
        # This exact pinned H1 parent intentionally predates the current runtime/metadata/freeze.
        exact(dep, parsed(read(ROOT / 'PROVENANCE.json'))['actual_parent_acceptance'] | {'root_binding_path': str(prior)})
        require(prior_run == REPO / '.local/native-straight-hole-20261116/H1-002', 'different historical parent run')
    input_path = run / CASES[bound['case']]['input']; raw = read(input_path)
    require(not input_path.stat().st_mode & 0o222 and raw == read(prior_run / 'snapshots' / (label + '.weftforge')) and
            sha(raw) == dep['snapshot_sha256'] and bound['inputs'] == {CASES[bound['case']]['input']: sha(raw)},
            'actual accepted input bytes/readonly inventory differ')
    grade_raw, review_raw = read(prior_run / 'SAVED-GRADE-001.json'), read(prior_run / 'ROOT-UI-ACCEPTANCE.json')
    grade, review = parsed(grade_raw), parsed(review_raw)
    require(sha(grade_raw) == dep['saved_grade_report_sha256'] == review['single_saved_grade_report_sha256'] and
            sha(review_raw) == dep['root_native_ui_review_sha256'] and
            grade['root_binding_sha256'] == review['root_binding_sha256'] == sha(prior_raw) and
            grade['case'] == review['case'] == expected_case and grade['attempt'] == review['attempt'] == dep['attempt'] and
            grade['source_commit'] == review['source_commit'] == prior_bound['source_commit'] and
            grade['native_binary_sha256'] == review['native_binary_sha256'] == prior_bound['native_binary_sha256'] and
            grade['status'] == 'passed_saved_files_only' and grade['snapshot_sha256'][label] == sha(raw) and
            review['accepted'] is True and review['reviewed_raw_images_and_traces'] is True, 'prior sole grade/root UI acceptance differs')
    closed = BASE.closure(prior_run, expected_case, prior_raw)
    require(closed['outcome'] == 'completed' and closed['attempt'] == dep['attempt'], 'accepted input lease was not completed/closed')
    exact(parsed(read(prior_run / 'SAVED-GRADE-ATTEMPT.json')),
          {'case': expected_case, 'formal_grade_attempt': 1, 'input_closure_sha256': sha(read(prior_run / 'NATIVE-LEASE-CLOSED.json'))})
    prior_post = parsed(read(prior_run / 'POSTFLIGHT.json'))
    require(prior_post['root_binding_sha256'] == sha(prior_raw) and
            prior_post['input_closure_sha256'] == sha(read(prior_run / 'NATIVE-LEASE-CLOSED.json')) and
            prior_post['source_inputs_unchanged'] is True and prior_post['native_binary_unchanged'] is True,
            'prior postflight does not bind its closed lease')
    if bound['case'] == 'SEW-S1':
        require(review['schema'] == 'weftforge.straight-hole.native-ui-review.v1', 'wrong historical parent UI schema')
    else:
        require(review['schema'] == 'weftforge.straight-cut-and-sew.native-ui-review.v1' and
                review['accepted_final_snapshot_label'] == label and review['accepted_final_snapshot_sha256'] == sha(raw) and
                grade['source_commit'] == bound['source_commit'] and grade['native_binary_sha256'] == bound['native_binary_sha256'],
                'SEW-S2 input is not accepted same-runtime SEW-S1 Redo R4')
        identity = parsed(read(ROOT / 'SOURCE-74.json'))
        require(prior_bound['clean_metadata_commit'] == bound['clean_metadata_commit'] == identity['clean_metadata_commit'] ==
                '3858848afb3764014a5d305a0e5dfb5661ea6d86' and
                prior_bound['source_inputs'] == bound['source_inputs'] == identity['source_inputs'] and
                prior_bound['source_input_count'] == bound['source_input_count'] == identity['source_input_count'] == 74 and
                prior_bound['freeze_sha256'] == grade['freeze_sha256'] == bound['freeze_sha256'],
                'SEW-S2 prior S1 metadata/source74/freeze differs from the current packet')
    return parsed(raw)


def trace_preflight(run, case, bound, binding_raw, closed):
    clock = int(parsed(read(run / 'LAUNCH-CLOCK.json'))['monotonic_ns']); latest = clock
    records = sorted((run / 'trace').glob('*-before.json')); spec = CASES[case]
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
            summary['documentation_refresh_calls'] >= PLAN['cases'][case]['documentation_refresh_calls'] and
            summary['overall_actual_calls'] == len(records) + 1 + summary['documentation_refresh_calls'] <= spec['cap'] + 1 and
            summary['summary_invocation_time_exact'] is False and summary['native_controller_pair_claimed_for_summary'] is False,
            'summary/mandatory refresh/continuous operation count differs')
    lower, upper = (int(summary[key]['monotonic_ns']) for key in
                    ('lower_bound_before_summary_invocation', 'upper_bound_after_summary_return'))
    require(clock <= lower <= upper and latest <= upper and (upper - clock) / 1e9 <= spec['seconds'] and
            closed['terminal_source_record'] == str(len(records)).zfill(4), 'continuous lab clock/terminal record differs')


def grade_run(case, run, bound, binding_raw, closed):
    require(bound['freeze_sha256'] == sha(read(ROOT / 'FREEZE.json')), 'wrong final prospective packet binding')
    identity_preflight(run, bound)
    BASE.runtime_preflight(run, bound, binding_raw)
    readiness_preflight(run, bound, binding_raw)
    postflight(run, bound, binding_raw, closed)
    trace_preflight(run, case, bound, binding_raw, closed)
    accepted_input = dependency(run, bound)
    documents, hashes = check_receipts(run, case)
    result = grade_states(case, documents, accepted_input)
    require(closed['outcome'] == 'completed', 'native case stopped unverified')
    return {'status': 'passed_saved_files_only', 'case': case, 'attempt': bound['attempt'],
            'source_commit': bound['source_commit'], 'native_binary_sha256': bound['native_binary_sha256'],
            'root_binding_sha256': sha(binding_raw), 'freeze_sha256': sha(read(ROOT / 'FREEZE.json')),
            'snapshot_sha256': hashes, 'result': result, 'native_ui_verified': False, 'reference_verified': False,
            'full_feature_accepted': False}


def freeze_coverage(freeze):
    require(freeze['status'] == 'frozen_prospective_native_cut_and_sew_acceptance', 'packet is still a draft')
    require(REQUIRED_PACKET_INPUTS <= freeze['files'].keys(), 'incomplete frozen packet input inventory')
    declared = parsed(read(ROOT / 'PROVENANCE.json'))['external_dependencies']
    require(declared.keys() <= freeze['external_dependencies'].keys() and
            all(freeze['external_dependencies'][name] == digest for name, digest in declared.items()),
            'incomplete or changed declared frozen dependency inventory')


def verify_freeze():
    freeze = parsed(read(ROOT / 'FREEZE.json'))
    freeze_coverage(freeze)
    for name, digest in freeze['files'].items():
        require(sha(read(ROOT / name)) == digest, 'packet artifact changed: ' + name)
    for name, digest in freeze['external_dependencies'].items():
        require(file_sha(REPO / name) == digest, 'pinned dependency changed: ' + name)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--case', choices=CASES, required=True)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    require((ROOT / 'FREEZE.json').is_file(), 'DRAFT packet: no formal grade/attempt is authorized')
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
                  'reference_verified': False, 'error_type': type(problem).__name__, 'error': str(problem)}
    with args.report.open('x') as output:
        json.dump(result, output, indent=2, allow_nan=False); output.write('\n')
    print(json.dumps(result))
    return 0 if result['status'] == 'passed_saved_files_only' else 1


if __name__ == '__main__':
    sys.exit(main())
