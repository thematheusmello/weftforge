#!/usr/bin/env python3
"""Prospective independent R240 saved-byte assessment; never imports WeftForge.

Models exist only in memory for assessment. They are not native input fixtures.
"""
import argparse
import copy
import importlib.util
import json
import math
import plistlib
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
OLD = REPO / '.local/internal-native-plan-v2-20261115/grade_saved.py'
OLD_SHA = '6b3aefcca60fcc12d78a75e967f63c582d76e7a2d134975393a72a3a9c184c78'
import hashlib
assert hashlib.sha256(OLD.read_bytes()).hexdigest() == OLD_SHA, 'old narrow helper changed'
SPEC = importlib.util.spec_from_file_location('independent_v2_helpers', OLD)
G = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(G)
require, read, sha, parsed, exact, item, bits, near = (
    G.require, G.read, G.sha, G.parsed, G.exact, G.item, G.bits, G.near)
ORIGINAL = REPO / '.local/straight-cut-contract-20261115/approved-v2'
ORIGINAL_PINS = {
    'MANIFEST.json': '1144d1e955cd77bd3e8690364555e7f0e9fec1a921b7ee410966a45fd0edd942',
    'CONTRACT.md': 'da88a248482dd45dd702b57e33dab13c38b885c222042863e973bf809c886b0c',
    'ORACLE.json': '13846d635a1432c3e5bfd77eb66802cc633bd29a3661c940972783dc2f5dfea2',
    'NATIVE-R.md': 'c810155dd3df45dbce5576d6e8e925e82f07c9a685063ea3ce941d6cfa146158',
    'fixtures/R240.weftforge': 'afcd39b8e65caa76a4fe7e556d10d58a0117c5af6954859cfa6ad35d50bb1d50',
    'fixtures/L.weftforge': 'a8ddc0c3b9e625e6bc77d72ec382cc333dcd4225e87a6c91261636af28828e7d',
}
CASES = {
    'CUT-R1': {'labels': ['parent-r1', 'cut-r2', 'undo-r3', 'redo-r4'],
               'revisions': [1, 2, 3, 4], 'cap': 175, 'seconds': 1200,
               'primary': 'work/cut-r240.work.weftforge'},
    'CUT-R2': {'labels': ['child-height-r5'], 'revisions': [5], 'cap': 65,
               'seconds': 600, 'primary': 'work/cut-r240-reopened.work.weftforge'},
}
SIDECARS = ('.recovery', '.autosave', '.autosave.recovery')
HEX = r'[0-9a-f]{64}'
CLOSURE_SCHEMA = 'weftforge.straight-cut.native-input-closure.v2'


def originals():
    for name, digest in ORIGINAL_PINS.items():
        require(sha(read(ORIGINAL / name)) == digest, 'original packet changed: ' + name)
    return parsed(read(ORIGINAL / 'fixtures/R240.weftforge')), parsed(read(ORIGINAL / 'ORACLE.json'))


def models():
    """Only the original oracle's literal allocation/incidence/walk recipes."""
    parent, oracle = originals()
    cut = copy.deepcopy(parent)
    project, recipe = cut['project'], oracle['R240']['cut']
    project['revision'], project['next_id'] = recipe['revision'], recipe['next_id']
    project['points'] += [{'id': int(identity), 'position': position}
                          for identity, position in recipe['new_points'].items()]
    project['curves'] = [{'id': identity, 'start': recipe['all_curve_incidences'][str(identity)][0],
                          'end': recipe['all_curve_incidences'][str(identity)][1], 'controls': None}
                         for identity in recipe['curve_order']]
    retained = project['pieces'][0]
    retained['boundary'] = recipe['retained']['boundary']
    child = copy.deepcopy(retained)
    child.update(id=recipe['new']['id'], name=recipe['new']['name'], boundary=recipe['new']['boundary'])
    project['pieces'].append(child)
    undo, redo = copy.deepcopy(parent), copy.deepcopy(cut)
    undo['project'].update(revision=3, next_id=20)
    redo['project']['revision'] = 4
    return {'parent-r1': parent, 'cut-r2': cut, 'undo-r3': undo, 'redo-r4': redo}, oracle


def tolerant_points(actual, expected, identities, tolerance):
    """Permit only declared changed point components; keep every other field exact."""
    adjusted = copy.deepcopy(expected)
    for identity in identities:
        point, target = item(actual, 'points', identity), item(adjusted, 'points', identity)
        near(point['position'], target['position'], tolerance, 'declared point differs: ' + str(identity))
        target['position'] = copy.deepcopy(point['position'])
    exact(actual, adjusted)


def walk(doc, identity):
    piece = item(doc, 'pieces', identity)
    curves = [item(doc, 'curves', curve) for curve in piece['boundary']]
    require(len(curves) >= 3 and len({c['id'] for c in curves}) == len(curves), 'invalid boundary list')
    require(all(c['controls'] is None and c['end'] == curves[(n + 1) % len(curves)]['start']
                for n, c in enumerate(curves)), 'directed walk is not closed/straight')
    point_ids = [c['start'] for c in curves]
    require(len(set(point_ids)) == len(point_ids), 'repeated boundary point')
    polygon = [item(doc, 'points', identity)['position'] for identity in point_ids]
    edges = list(zip(polygon, polygon[1:] + polygon[:1]))
    area = math.fsum(a[0] * b[1] - b[0] * a[1] for a, b in edges) / 2
    require(area > 0, 'parent winding was not preserved')
    perimeter = math.fsum(math.hypot(a[0] - b[0], a[1] - b[1]) for a, b in edges)
    return set(point_ids), {c['id'] for c in curves}, area, perimeter


def measure_cut(doc, oracle, edited=False):
    recipe, tolerance = oracle['R240']['cut'], oracle['tolerances']
    left, right = walk(doc, 2), walk(doc, 15)
    require(left[0].isdisjoint(right[0]) and left[1].isdisjoint(right[1]), 'children share ownership')
    require(left[0] | right[0] == {p['id'] for p in doc['project']['points']} and
            left[1] | right[1] == {c['id'] for c in doc['project']['curves']}, 'unowned construction')
    for copied, source in recipe['copy_bits']:
        # The intentional R5 height edit breaks only the top coordinate coincidence.
        if not edited or copied == 16:
            exact(item(doc, 'points', copied)['position'], item(doc, 'points', source)['position'])
    targets = ([recipe['retained']['area'], oracle['R240']['reopen_edit']['new_area']] if edited
               else [recipe['retained']['area'], recipe['new']['area']])
    near([left[2], right[2]], targets, tolerance['ordinary_area_m2'], 'analytic child areas differ')
    density = item(doc, 'materials', 1)['areal_density']
    target_masses = ([recipe['retained']['mass'], oracle['R240']['reopen_edit']['new_mass']] if edited
                     else [recipe['retained']['mass'], recipe['new']['mass']])
    near([left[2] * density, right[2] * density], target_masses,
         tolerance['ordinary_mass_kg'], 'analytic material masses differ')
    if not edited:
        near([left[3], right[3]], [recipe['retained']['perimeter'], recipe['new']['perimeter']],
             tolerance['ordinary_new_point_length_m'], 'child perimeters differ')
        near([math.fsum([left[2], right[2]])], [recipe['union_area']],
             tolerance['ordinary_area_m2'], 'pre-edit area conservation differs')
        for identity in (11, 13, 16, 17):
            near([item(doc, 'points', identity)['position'][0]], [.09],
                 tolerance['ordinary_new_point_length_m'], 'interface is not at declared x')
        # Full literal walks plus these side bounds certify disjoint rectangle interiors.
        require(all(item(doc, 'points', n)['position'][0] <= item(doc, 'points', 11)['position'][0] for n in (3, 6)) and
                all(item(doc, 'points', n)['position'][0] >= item(doc, 'points', 16)['position'][0] for n in (4, 5)),
                'child interiors do not occupy opposite declared sides')
    return {'areas_m2': [left[2], right[2]], 'analytic_material_masses_kg': [left[2] * density, right[2] * density]}


def grade_states(case, documents, input_r4=None):
    expected, oracle = models()
    require(list(documents) == CASES[case]['labels'], 'missing/extra/reordered state labels')
    tolerance = oracle['tolerances']['ordinary_new_point_length_m']
    measured = {}
    if case == 'CUT-R1':
        for label in ('parent-r1', 'undo-r3'):
            exact(documents[label], expected[label])
        for label in ('cut-r2', 'redo-r4'):
            tolerant_points(documents[label], expected[label], (11, 13, 16, 17), tolerance)
            measured[label] = measure_cut(documents[label], oracle)
        same_cut = copy.deepcopy(documents['cut-r2'])
        same_cut['project']['revision'] = 4
        exact(documents['redo-r4'], same_cut)
    else:
        require(input_r4 is not None, 'actual accepted R4 input required')
        tolerant_points(input_r4, expected['redo-r4'], (11, 13, 16, 17), tolerance)
        measure_cut(input_r4, oracle)
        edited = copy.deepcopy(input_r4)
        edited['project']['revision'] = oracle['R240']['reopen_edit']['revision']
        for identity, position in oracle['R240']['reopen_edit']['only_changed_points'].items():
            target = item(edited, 'points', int(identity))
            target['position'][1] = position[1]
            actual = item(documents['child-height-r5'], 'points', int(identity))['position']
            require(bits(actual[0]) == bits(target['position'][0]), 'reopen changed an unchanged x bit')
            near([actual[1]], [position[1]], tolerance, 'reopen height differs')
            target['position'][1] = actual[1]
        exact(documents['child-height-r5'], edited)
        measured['child-height-r5'] = measure_cut(documents['child-height-r5'], oracle, True)
    return {'full_v4_construction': True, 'ids_order_walks_ownership_highwater_revision': 'checked',
            'unchanged_fields': 'exact f64 bits including signed zero; color f32 bits',
            'declared_point_tolerance_m': tolerance, 'analytic_saved_geometry': measured,
            'solver_mesh_or_particle_mass_verified': False}


def paired(run, record, returned=True):
    require(type(record) is str and re.fullmatch(r'\d{4,}', record), 'bad source record')
    before = parsed(read(run / 'trace' / (record + '-before.json')))
    require(before['id'] == record, 'wrong before record')
    if returned:
        after = parsed(read(run / 'trace' / (record + '-after.json')))
        require(after['id'] == record and after['status'] == 'returned', 'unpaired/failed source operation')
        return before, after
    return before


def closure(run, case, binding_raw):
    closed = parsed(read(run / 'NATIVE-LEASE-CLOSED.json'))
    keys = {'schema', 'case', 'attempt', 'native_input_lease_closed', 'outcome', 'root_binding_sha256',
            'terminal_source_record', 'planned_quit_source_record', 'immediate_completion_summary_calls',
            'completion_summary_return_receipt', 'postflight_receipt'}
    require(closed.keys() == keys and closed['schema'] == CLOSURE_SCHEMA and closed['case'] == case and
            closed['native_input_lease_closed'] is True and closed['root_binding_sha256'] == sha(binding_raw) and
            type(closed['immediate_completion_summary_calls']) is int and closed['immediate_completion_summary_calls'] == 1 and
            closed['completion_summary_return_receipt'] == 'COMPLETION-SUMMARY-RETURN.json' and
            closed['postflight_receipt'] == 'POSTFLIGHT.json' and
            closed['outcome'] in ('completed', 'stopped_unverified') and
            type(closed['attempt']) is str and bool(closed['attempt']), 'wrong/incomplete input closure schema')
    require(closed['outcome'] != 'completed' or closed['planned_quit_source_record'] == closed['terminal_source_record'],
            'completed case lacks final planned quit')
    return closed


def absolute(value, directory=False):
    require(type(value) is str and Path(value).is_absolute(), 'missing/nonabsolute bound path')
    path = Path(value)
    require(not path.is_symlink() and path.resolve() == path and
            (path.is_dir() if directory else path.is_file()), 'missing/noncanonical bound path')
    return path


def file_sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def inventory_sha(bound):
    return sha(json.dumps(bound['source_inputs'], sort_keys=True, separators=(',', ':')).encode())


def receipt(descriptor, expected_digest):
    require(descriptor.keys() == {'path', 'sha256'} and descriptor['sha256'] == expected_digest,
            'wrong bound receipt descriptor')
    path = absolute(descriptor['path'])
    raw = read(path)
    require(sha(raw) == expected_digest, 'bound receipt bytes differ')
    return parsed(raw), path


def grade_scope(run, report, case, bound, closed):
    require(run == absolute(str(run), True) and report == run / 'SAVED-GRADE-001.json' and
            not report.exists() and not report.is_symlink(), 'formal grade needs exact fresh owned report path')
    require(bound['schema'] == 'weftforge.straight-cut.root-binding.v2' and bound['case'] == case and
            bound['root_authorized_for_case'] is True and bound['native_lease_granted'] is True and
            type(bound['attempt']) is str and re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]{0,79}', bound['attempt']) and
            bound['attempt'] == closed['attempt'] and bound['controller_configuration']['root'] == str(run) and
            bound['controller_configuration']['evidenceKind'] == 'native', 'wrong/unbound native case scope')


def identity_preflight(run, bound, source_model):
    """Validate bound files/receipts, never execute their product binaries."""
    require(bound['source_commit'] == source_model['source_commit'] and
            bound['clean_metadata_commit'] == source_model['clean_metadata_commit'] and
            bound['source_input_count'] == source_model['source_input_count'] == 69 and
            bound['source_inputs'] == source_model['source_inputs'], 'wrong named source69/commit inventory')
    source = absolute(bound['source_worktree'], True)
    target = absolute(bound['own_target_directory'], True)
    require(target == source / 'target', 'own target differs from the bound fresh default build target')
    bundle = absolute(bound['bundle'], True)
    binary = absolute(bound['native_binary_path'])
    require(bundle.is_relative_to(run) and bundle.suffix == '.app' and
            binary == bundle / 'Contents/MacOS/weftforge', 'binary/bundle is not owned by this case')
    for name, digest in bound['source_inputs'].items():
        require(re.fullmatch(HEX, digest) and file_sha(absolute(str(source / name))) == digest,
                'bound source file changed: ' + name)
    require(bound['native_binary_sha256'] == source_model['native_binary_sha256'] == file_sha(binary) and
            binary.stat().st_size == source_model['native_binary_bytes'], 'bound native binary changed')
    plist_raw = read(bundle / 'Contents/Info.plist')
    require(re.fullmatch(HEX, bound['bundle_plist_sha256']) and sha(plist_raw) == bound['bundle_plist_sha256'],
            'bound plist changed')
    plist = plistlib.loads(plist_raw)
    require(type(bound['bundle_identifier']) is str and re.fullmatch(r'[A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)+', bound['bundle_identifier']) and
            plist['CFBundleIdentifier'] == bound['bundle_identifier'] and plist['CFBundleExecutable'] == 'weftforge' and
            plist['CFBundlePackageType'] == 'APPL' and type(bound['os_version']) is str and
            re.fullmatch(r'\d+(?:\.\d+)+', bound['os_version']), 'missing/mismatched bundle/OS identity')
    expected_argv = ['-n', str(bundle)] + (['--args', *bound['input_argv']] if bound['input_argv'] else [])
    require(bound['launcher_path'] == '/usr/bin/open' and bound['launcher_argv'] == expected_argv,
            'launcher argv is unbound or differs')
    for name, relative, digest in (
        ('controller', '.local/native-transform-controller-20261115/controller.cjs', '9debdc6b1d42b49dce16c8b1227df0621b579532ca7c5fd76c3a8a5db537790b'),
        ('recorder', '.local/native-length-readouts-v2-20261115/recorder.cjs', '45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc'),
        ('observer', '.local/native-internal-annotations-20261115/C002/OBSERVER-FRAGMENT.cjs', 'd580450b45fc2fe42b19c146fb7023f84d04e7b9596ac24a8f5afe3e81be7729')):
        require(bound[name] == str(REPO / relative) and bound[name + '_sha256'] == digest == file_sha(absolute(bound[name])),
                'bound module identity differs: ' + name)
    actor = absolute(bound['actor_filename'])
    require(actor.is_relative_to(run) and re.fullmatch(HEX, bound['actor_sha256']) and
            file_sha(actor) == bound['actor_sha256'], 'actual actor filename/hash is unbound')
    config = bound['controller_configuration']
    expected_context = {key: value for key, value in config.items() if key != 'root'}
    expected_context['recorderSha256'] = bound['recorder_sha256']
    exact(parsed(read(run / 'CONTEXT.json')), expected_context)
    gates = bound['integrated_local_gate_receipts']
    require(gates.keys() == {'runtime', 'clean_build'} and bound['clean_source_metadata_receipt'] == gates['clean_build'],
            'missing/ambiguous bound metadata/build receipts')
    runtime, runtime_path = receipt(gates['runtime'], source_model['runtime_checks_sha256'])
    build, build_path = receipt(gates['clean_build'], source_model['clean_build_checks_sha256'])
    require(runtime['source_commit'] == build['runtime_commit'] == bound['source_commit'] and
            build['checked_clean_commit'] == bound['clean_metadata_commit'] and
            runtime['source_hashes'] == build['source_hashes'] == bound['source_inputs'] and
            runtime['source_count'] == build['source_count'] == 69 and
            runtime['checkout_status_before'] == runtime['checkout_status_after'] == '' and
            runtime['runtime_inputs_unchanged'] is True and runtime['all_three_passed'] is True and
            build['source69_unchanged'] is True and build['checkout_clean_after'] is True and build['all_passed'] is True and
            build['fresh_owned_default_target_no_cache_copy'] is True, 'source/clean gate results differ')
    expected_steps = [(['fmt', 'test', 'clippy'], runtime, runtime_path), (['xtask', 'debug-build'], build, build_path)]
    for names, checks, checks_path in expected_steps:
        require([step['name'] for step in checks['steps']] == names and
                all(type(step['exit_code']) is int and step['exit_code'] == 0 for step in checks['steps']), 'gate did not pass')
        for step in checks['steps']:
            for output in ('stdout', 'stderr'):
                require(file_sha(absolute(str(checks_path.parent / (step['name'] + '.' + output)))) == step[output + '_sha256'],
                        'gate raw log differs')
    built = build['binaries']['weftforge']
    built_path = absolute(built['path'])
    require(built_path.is_relative_to(target) and built['sha256'] == bound['native_binary_sha256'] == file_sha(built_path) and
            built['bytes'] == binary.stat().st_size == built_path.stat().st_size, 'built/owned binary identities differ')


def runtime_preflight(run, bound, binding_raw):
    proof_raw = read(run / 'ACTOR-RUNTIME-PROOF.json')
    proof = parsed(proof_raw)
    expected = {'schema': 'weftforge.straight-cut.actor-runtime-proof.v1', 'case': bound['case'],
                'attempt': bound['attempt'], 'root_binding_sha256': sha(binding_raw), 'evidence_kind': 'native_cua_runtime',
                'actual_cua_runtime': True, 'native_UI_calls': 0, 'createRequire': True, 'fs_writable': True,
                'source_record': '0002', 'actor_filename': bound['actor_filename'], 'actor_sha256': bound['actor_sha256'],
                'controller_configuration': bound['controller_configuration'],
                'observer_explicit_arguments': 5, 'marker_initialized_synchronous_before_observer': True}
    expected.update({name + '_sha256': bound[name + '_sha256'] for name in ('controller', 'recorder', 'observer')})
    require(proof.keys() == expected.keys() | {'offline_receipt'}, 'wrong actual runtime proof fields')
    exact({key: proof[key] for key in expected}, expected)
    require(paired(run, '0001')[0]['method'] == 'getState', 'first operation is not standalone getState')
    before, after = paired(run, proof['source_record'])
    require(before['method'] == 'local-filesystem-proof' and int(before['monotonic_ns']) >=
            int(parsed(read(run / 'LAUNCH-CLOCK.json'))['monotonic_ns']), 'runtime proof was not clocked second FS operation')
    exact(after['result'], {'proof_sha256': sha(proof_raw), 'actual_cua_runtime': True, 'native_UI_calls': 0})
    artifact = proof['offline_receipt']
    require(artifact['file'] == 'captures/binding-raw.json' and artifact['source_record'] == '0001' and
            artifact['unedited'] is True and artifact['bytes'] == len(binding_raw) and artifact['sha256'] == sha(binding_raw),
            'opaque recorder binding-byte receipt differs')
    offline = run / 'offline-proof'
    require(read(offline / artifact['file']) == binding_raw, 'opaque recorder bytes differ from binding')
    exact(parsed(read(offline / (artifact['file'] + '.sha256.json'))), artifact)
    offline_before, offline_after = paired(offline, '0001')
    require(offline_before['method'] == 'opaque-binding-proof' and
            offline_after['result'] == {'raw_binding_sha256': sha(binding_raw)}, 'offline recorder proof did not return')
    returned = parsed(read(run / 'ACTOR-RUNTIME-TOOL-HISTORY.json'))
    expected_return = {'schema': 'weftforge.straight-cut.actor-runtime-tool-history.v1', 'tool': 'mcp__cua_repl.js',
                       'case': bound['case'], 'attempt': bound['attempt'], 'root_binding_sha256': sha(binding_raw),
                       'source_record': '0002', 'proof_sha256': sha(proof_raw), 'root_correlated_with_actual_CUA_tool_history': True,
                       'evidence_kind': 'post_case_tool_history_corroboration', 'captured_after_input_closure': True,
                       'invocation_timestamp_exact': False, 'input_closure_sha256': sha(read(run / 'NATIVE-LEASE-CLOSED.json'))}
    require(returned.keys() == expected_return.keys() | {'raw_tool_history'}, 'missing/wrong retained actual CUA tool return')
    exact({key: returned[key] for key in expected_return}, expected_return)
    raw_tool = returned['raw_tool_history']
    require(raw_tool.keys() == {'path', 'sha256'} and raw_tool['path'] == 'ACTOR-RUNTIME-RAW-TOOL-HISTORY.json' and
            re.fullmatch(HEX, raw_tool['sha256']) and sha(read(run / raw_tool['path'])) == raw_tool['sha256'] and
            sha(proof_raw).encode() in read(run / raw_tool['path']), 'actual CUA raw tool return missing/changed')


def postflight_preflight(run, bound, binding_raw, closed):
    postflight = parsed(read(run / 'POSTFLIGHT.json'))
    expected = {'schema': 'weftforge.straight-cut.postflight.v1', 'case': bound['case'], 'attempt': bound['attempt'],
                'root_binding_sha256': sha(binding_raw), 'input_closure_sha256': sha(read(run / 'NATIVE-LEASE-CLOSED.json')),
                'source_commit': bound['source_commit'], 'native_binary_sha256': bound['native_binary_sha256'],
                'source_inputs_sha256': inventory_sha(bound), 'controller_context_sha256': sha(read(run / 'CONTEXT.json')),
                'runtime_proof_sha256': sha(read(run / 'ACTOR-RUNTIME-PROOF.json')),
                'runtime_tool_history_sha256': sha(read(run / 'ACTOR-RUNTIME-TOOL-HISTORY.json')),
                'terminal_source_record': closed['terminal_source_record'],
                'paired_native_external_operations': len(list((run / 'trace').glob('*-before.json'))),
                'completion_summary_return_sha256': sha(read(run / 'COMPLETION-SUMMARY-RETURN.json')),
                'source_inputs_unchanged': True, 'native_binary_unchanged': True}
    exact(postflight, expected)


def dependency(run, bound):
    dep = bound['accepted_cut_r1_dependency']
    require(dep['case'] == 'CUT-R1' and dep['snapshot_label'] == 'redo-r4' and dep['root_accepted'] is True,
            'CUT-R2 needs accepted actual CUT-R1 dependency')
    input_path = run / 'inputs/accepted-redo-r4.weftforge'
    raw = read(input_path)
    require(not input_path.stat().st_mode & 0o222 and sha(raw) == dep['snapshot_sha256'], 'R4 input differs/not read-only')
    require(bound['inputs'] == {'inputs/accepted-redo-r4.weftforge': sha(raw)}, 'bound input inventory differs')
    require(raw == read(Path(dep['snapshot_path'])), 'R4 staged bytes differ from actual retained snapshot')
    grade_raw, review_raw = read(Path(dep['saved_grade_report'])), read(Path(dep['root_native_ui_review']))
    require(sha(grade_raw) == dep['saved_grade_report_sha256'] and sha(review_raw) == dep['root_native_ui_review_sha256'],
            'accepted dependency report hashes differ')
    grade, review = parsed(grade_raw), parsed(review_raw)
    r1_binding_path = absolute(dep['root_binding_path'])
    r1_raw = read(r1_binding_path)
    r1_bound = parsed(r1_raw)
    require(r1_binding_path.name == 'ROOT-BINDING.json' and sha(r1_raw) == dep['root_binding_sha256'] and
            r1_bound['schema'] == 'weftforge.straight-cut.root-binding.v2' and r1_bound['case'] == 'CUT-R1' and
            r1_bound['attempt'] == dep['attempt'] and r1_bound['root_authorized_for_case'] is True and
            r1_bound['native_lease_granted'] is True and r1_bound['controller_configuration']['root'] == str(r1_binding_path.parent) and
            Path(dep['snapshot_path']) == r1_binding_path.parent / 'snapshots/redo-r4.weftforge' and
            Path(dep['saved_grade_report']) == r1_binding_path.parent / 'SAVED-GRADE-001.json' and
            Path(dep['root_native_ui_review']) == r1_binding_path.parent / 'ROOT-UI-ACCEPTANCE.json', 'R1 dependency run identity differs')
    require(grade['status'] == 'passed_saved_files_only' and grade['case'] == 'CUT-R1' and
            grade['attempt'] == review['attempt'] == dep['attempt'] and
            grade['root_binding_sha256'] == review['root_binding_sha256'] == sha(r1_raw) and
            review['single_saved_grade_report_sha256'] == dep['saved_grade_report_sha256'] == sha(grade_raw) and
            grade['source_commit'] == review['source_commit'] == r1_bound['source_commit'] == bound['source_commit'] and
            grade['native_binary_sha256'] == review['native_binary_sha256'] == r1_bound['native_binary_sha256'] == bound['native_binary_sha256'] and
            grade['snapshot_sha256']['redo-r4'] == sha(raw), 'CUT-R1 actual saved grade/acceptance does not identify this run')
    require(review['schema'] == 'weftforge.straight-cut.native-ui-review.v2' and review['case'] == 'CUT-R1' and
            review['accepted'] is True and review['reviewed_raw_images_and_traces'] is True and
            review['accepted_redo_r4_sha256'] == sha(raw), 'CUT-R1 raw UI review did not accept this R4')
    return parsed(raw)


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


def grade_run(case, run, bound, binding_raw, closed):
    spec = CASES[case]
    require(bound['schema'] == 'weftforge.straight-cut.root-binding.v2' and bound['case'] == case and
            bound['root_authorized_for_case'] is True and bound['native_lease_granted'] is True and
            bound['freeze_sha256'] == sha(read(ROOT / 'FREEZE.json')), 'missing/wrong final root binding')
    require(re.fullmatch(r'[0-9a-f]{40}', bound['source_commit']) and
            len(bound['source_inputs']) == bound['source_input_count'] == 69 and
            all(re.fullmatch(HEX, digest) for digest in bound['source_inputs'].values()) and
            re.fullmatch(HEX, bound['native_binary_sha256']), 'source69/binary binding incomplete')
    config = bound['controller_configuration']
    require(config.keys() == {'root', 'recorderPath', 'evidenceKind', 'callCap', 'elapsedCapS', 'baselineCalls'} and
            config['root'] == str(run) and config['recorderPath'] == str(REPO / '.local/native-length-readouts-v2-20261115/recorder.cjs') and
            config['evidenceKind'] == 'native' and config['callCap'] == spec['cap'] and
            config['elapsedCapS'] == spec['seconds'] and config['baselineCalls'] == 0, 'six-key controller binding differs')
    require(bound['input_argv'] == ([] if case == 'CUT-R1' else [str(run / 'inputs/accepted-redo-r4.weftforge')]),
            'launch argv differs from original empty/dependent case')
    require(case != 'CUT-R1' or bound['inputs'] == {}, 'actual-empty case must have no staged project input')
    identity_preflight(run, bound, parsed(read(ROOT / 'SOURCE-69.json')))
    runtime_preflight(run, bound, binding_raw)
    postflight_preflight(run, bound, binding_raw, closed)
    launch = parsed(read(run / 'LAUNCH-CLOCK.json'))
    records = sorted((run / 'trace').glob('*-before.json'))
    require(0 < len(records) <= spec['cap'], 'missing trace/operation cap exceeded')
    latest = int(launch['monotonic_ns'])
    for index, path in enumerate(records, 1):
        record = str(index).zfill(4)
        require(path.name == record + '-before.json', 'noncontiguous operation trace')
        before, after = paired(run, record)
        require(int(before['monotonic_ns']) >= latest and int(after['monotonic_ns']) >= int(before['monotonic_ns']),
                'clock chronology differs')
        latest = int(after['monotonic_ns'])
    require(closed['terminal_source_record'] == str(len(records)).zfill(4), 'closure is not after last native/external operation')
    summary = parsed(read(run / closed['completion_summary_return_receipt']))
    require(summary['schema'] == 'weftforge.straight-cut.completion-summary-return.v1' and summary['case'] == case and
            summary['attempt'] == bound['attempt'] and summary['root_binding_sha256'] == sha(binding_raw), 'wrong summary case/binding identity')
    require(type(summary['summary_calls']) is int and summary['summary_calls'] == 1 and
            summary['paired_native_external_operations'] == len(records) and
            type(summary['documentation_refresh_calls']) is int and summary['documentation_refresh_calls'] >= 0 and
            summary['overall_actual_calls'] == len(records) + 1 + summary['documentation_refresh_calls'] <= spec['cap'] + 1 and
            summary['summary_invocation_time_exact'] is False and summary['native_controller_pair_claimed_for_summary'] is False,
            'completion summary/refresh count differs')
    lower = int(summary['lower_bound_before_summary_invocation']['monotonic_ns'])
    upper = int(summary['upper_bound_after_summary_return']['monotonic_ns'])
    require(int(launch['monotonic_ns']) <= lower <= upper and latest <= upper and
            0 <= (upper - int(launch['monotonic_ns'])) / 1e9 <= spec['seconds'], 'continuous elapsed cap exceeded')
    documents, hashes = check_receipts(run, case)
    input_r4 = dependency(run, bound) if case == 'CUT-R2' else None
    result = grade_states(case, documents, input_r4)
    require(closed['outcome'] == 'completed', 'native case stopped unverified')
    return {'status': 'passed_saved_files_only', 'case': case, 'attempt': bound['attempt'],
            'source_commit': bound['source_commit'], 'native_binary_sha256': bound['native_binary_sha256'], 'native_ui_verified': False,
            'reference_verified': False, 'full_feature_accepted': False,
            'root_binding_sha256': sha(binding_raw), 'freeze_sha256': sha(read(ROOT / 'FREEZE.json')),
            'snapshot_sha256': hashes, 'result': result}


def verify_freeze():
    freeze = parsed(read(ROOT / 'FREEZE.json'))
    for name, digest in freeze['files'].items():
        require(sha(read(ROOT / name)) == digest, 'prospective packet changed: ' + name)
    for name, digest in freeze['external_dependencies'].items():
        require(sha(read(REPO / name)) == digest, 'pinned dependency changed: ' + name)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--case', choices=CASES, required=True)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    require(args.run.is_absolute() and args.run == args.run.resolve(), 'formal run must be an absolute canonical path')
    run = args.run
    require(not args.report.exists(), 'refuse to overwrite a saved grade')
    # No formal attempt is allowed until exact native-input closure already exists.
    binding_raw = read(run / 'ROOT-BINDING.json')
    bound = parsed(binding_raw)
    closed = closure(run, args.case, binding_raw)
    grade_scope(run, args.report, args.case, bound, closed)
    attempt = run / 'SAVED-GRADE-ATTEMPT.json'
    with attempt.open('x') as output:
        json.dump({'case': args.case, 'formal_grade_attempt': 1, 'input_closure_sha256':
                   sha(read(run / 'NATIVE-LEASE-CLOSED.json'))}, output)
    try:
        verify_freeze()
        result = grade_run(args.case, run, bound, binding_raw, closed)
    except Exception as problem:
        result = {'status': 'failed_saved_files_or_preflight', 'case': args.case, 'attempt': bound['attempt'],
                  'root_binding_sha256': sha(binding_raw), 'native_ui_verified': False,
                  'reference_verified': False, 'error_type': type(problem).__name__, 'error': str(problem)}
    with args.report.open('x') as output:
        json.dump(result, output, indent=2, allow_nan=False)
        output.write('\n')
    print(json.dumps(result))
    return 0 if result['status'] == 'passed_saved_files_only' else 1


if __name__ == '__main__':
    sys.exit(main())
