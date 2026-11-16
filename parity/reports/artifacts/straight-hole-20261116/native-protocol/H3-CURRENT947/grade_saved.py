#!/usr/bin/env python3
"""H3 closed-lease assessment draft; stdlib only, never imports or executes WeftForge."""
import argparse
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
FROZEN_PATH = REPO / '.local/straight-hole-native-plan-v2-20261116/grade_saved.py'
assert hashlib.sha256(FROZEN_PATH.read_bytes()).hexdigest() == '486328f13c49018e240cbbd3f52e8b1cf9dd0d97747d3653ee3b927d3b2a31b9'
SPEC = importlib.util.spec_from_file_location('frozen_hole_literal_math', FROZEN_PATH)
FROZEN = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FROZEN)
BASE = FROZEN.BASE
require, read, parsed, sha, exact, item, near, paired, absolute, file_sha = (
    FROZEN.require, FROZEN.read, FROZEN.parsed, FROZEN.sha, FROZEN.exact, FROZEN.item,
    FROZEN.near, FROZEN.paired, FROZEN.absolute, FROZEN.file_sha)
SIDECARS = FROZEN.SIDECARS
HELPER_TYPES = FROZEN.HELPER_TYPES
HEX = r'[0-9a-f]{64}'
PLAN = parsed(read(ROOT / 'PLAN.json'))
CASES = {case: {'labels': spec['labels'], 'revisions': spec['revisions'], 'primary': spec['primary'],
                'cap': spec['operation_cap'], 'seconds': spec['elapsed_cap_s']} for case, spec in PLAN['cases'].items()}
require(list(CASES) == ['HOLE-H3'], 'only fresh H3 may be assessed')
SOURCE_KEYS = {'source_commit', 'clean_metadata_commit', 'source_worktree', 'own_target_directory',
               'source_input_count', 'source_inputs', 'command_variant_count', 'command_schema_count', 'mcp_tool_count',
               'native_format_version', 'runtime_checks', 'clean_checks', 'native_binary_sha256', 'native_binary_bytes'}
MANDATORY_ARTIFACTS = {'README.md', 'PLAN.json', 'MODEL.json', 'SOURCE-74.json', 'PROVENANCE.json',
    'HISTORICAL-DEPENDENCY.json', 'DEPENDENCIES.json', 'RUNTIME-PROTOCOL.json', 'grade_saved.py', 'retain_saved.py',
    'grade_exports.py', 'selfcheck.py', 'FREEZE.template.json', 'ROOT-BINDING.HOLE-H3.template.json',
    'ACTOR-RUNTIME-PROOF.template.json', 'ACTOR-PERSISTENCE-CHECK-RESULT.template.json',
    'ACTOR-RUNTIME-TOOL-HISTORY.template.json', 'NATIVE-LEASE-CLOSED.template.json', 'POSTFLIGHT.template.json',
    'COMPLETION-SUMMARY-RETURN.template.json', 'ROOT-UI-ACCEPTANCE.template.json', 'OWNED-PROCESS-EXIT.template.json',
    'ROOT-TOOL-HISTORY-CORRELATION.template.json', 'RAW-RETENTION.template.json', 'RAW-ARTIFACT-MANIFEST.template.json',
    'fixtures/R240-v5-assessment.weftforge', 'fixtures/hole-r6.dxf', 'fixtures/hole-r6.dxf.recovery'}


def complete(value):
    if isinstance(value, dict):
        return all(complete(v) for v in value.values())
    if isinstance(value, list):
        return all(complete(v) for v in value)
    return value is not None


def closed_shape(value, template):
    if isinstance(template, dict):
        require(type(value) is dict and value.keys() == template.keys(), 'closed schema has missing/extra keys')
        for key in template:
            closed_shape(value[key], template[key])
    elif isinstance(template, list):
        require(type(value) is list and len(value) == len(template), 'closed schema list differs')
        for actual, expected in zip(value, template):
            closed_shape(actual, expected)


def source_complete(source):
    require(source.keys() == SOURCE_KEYS and complete(source), 'source74 fields missing/extra or pending root receipt')
    require(source['source_commit'] == '947bfbe4dcb74fb253d584709376ace5a08db838' and
            re.fullmatch(r'[0-9a-f]{40}', source['clean_metadata_commit']) and source['source_input_count'] == 74 and
            source['command_variant_count'] == 33 and source['command_schema_count'] == 32 and
            source['mcp_tool_count'] == 9 and source['native_format_version'] == 5, 'wrong source74 identity/schema counts')
    inputs = source['source_inputs']
    require(type(inputs) is dict and len(inputs) == 74 and all(type(name) is str and name and
            not Path(name).is_absolute() and '..' not in Path(name).parts and re.fullmatch(HEX, digest)
            for name, digest in inputs.items()), 'named source74 inventory missing/unsafe/incomplete')
    require(re.fullmatch(HEX, source['native_binary_sha256']) and type(source['native_binary_bytes']) is int and
            source['native_binary_bytes'] > 0, 'current native binary identity missing')
    for key in ('runtime_checks', 'clean_checks'):
        descriptor = source[key]
        require(descriptor.keys() == {'path', 'sha256'} and re.fullmatch(HEX, descriptor['sha256']) and
                Path(descriptor['path']).is_absolute(), 'missing canonical current gate descriptor')


def final_binding_complete(bound):
    template = parsed(read(ROOT / 'ROOT-BINDING.HOLE-H3.template.json'))
    closed_shape(bound, template)
    require(bound.keys() == template.keys() and complete(bound) and bound['root_authorized_for_case'] is True and
            bound['native_lease_granted'] is True and bound['case'] == 'HOLE-H3' and bound['attempt'] == 'H3-001',
            'final H3 binding has extra/missing/pending fields or lacks canonical grants')
    source = parsed(read(ROOT / 'SOURCE-74.json')); source_complete(source)
    for key in ('source_commit', 'clean_metadata_commit', 'source_worktree', 'own_target_directory', 'source_input_count',
                'source_inputs', 'native_binary_sha256', 'native_binary_bytes'):
        exact(bound[key], source[key])
    require(bound['source74_sha256'] == sha(read(ROOT / 'SOURCE-74.json')), 'unbound source74 artifact')
    exact(bound['accepted_h1_dependency'], parsed(read(ROOT / 'HISTORICAL-DEPENDENCY.json')))
    exact(bound['inputs'], {'inputs/accepted-h1-002-redo-r5.weftforge': bound['accepted_h1_dependency']['snapshot']['sha256']})
    require(bound['input_sidecar_policy'] == template['input_sidecar_policy'], 'unbound input sidecar disclosure policy')


def grade_states(documents, historical_r5):
    # Reuse literal v5 math only; never run the frozen entrypoint or assess an old case.
    return FROZEN.grade_states('HOLE-H2', documents, historical_r5)


def identity_preflight(run, bound):
    identity = parsed(read(ROOT / 'SOURCE-74.json')); source_complete(identity)
    source, target = absolute(bound['source_worktree'], True), absolute(bound['own_target_directory'], True)
    require(target == source / 'target' and source == REPO / '.local/svg-export-fix-20261116', 'own default target/worktree differs')
    bundle, binary = absolute(bound['bundle'], True), absolute(bound['native_binary_path'])
    require(bundle.parent == run and bundle.name == 'WeftForgeSvgHoleH3001.app' and
            binary == bundle / 'Contents/MacOS/weftforge', 'fresh owned H3 bundle/binary differs')
    for name, digest in bound['source_inputs'].items():
        require(file_sha(absolute(str(source / name))) == digest, 'current source74 file changed: ' + name)
    require(file_sha(binary) == bound['native_binary_sha256'] == identity['native_binary_sha256'] and
            binary.stat().st_size == bound['native_binary_bytes'] == identity['native_binary_bytes'], 'current native binary differs')
    plist_raw = read(bundle / 'Contents/Info.plist'); plist = BASE.plistlib.loads(plist_raw)
    require(sha(plist_raw) == bound['bundle_plist_sha256'] and
            plist['CFBundleIdentifier'] == bound['bundle_identifier'] == 'org.weftforge.preview.svghole20261116.h3001' and
            plist['CFBundleExecutable'] == 'weftforge' and plist['CFBundlePackageType'] == 'APPL' and
            re.fullmatch(r'\d+(?:\.\d+)+', bound['os_version']), 'current plist/OS identity differs')
    argv = [str(run / 'inputs/accepted-h1-002-redo-r5.weftforge')]
    require(bound['input_argv'] == argv and bound['launcher_path'] == '/usr/bin/open' and
            bound['launcher_argv'] == ['-n', str(bundle), '--args', *argv], 'wrong exact historical-input launch argv')
    for name, relative, digest in (
        ('controller', '.local/native-transform-controller-20261115/controller.cjs', '9debdc6b1d42b49dce16c8b1227df0621b579532ca7c5fd76c3a8a5db537790b'),
        ('recorder', '.local/native-length-readouts-v2-20261115/recorder.cjs', '45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc'),
        ('observer', '.local/native-internal-annotations-20261115/C002/OBSERVER-FRAGMENT.cjs', 'd580450b45fc2fe42b19c146fb7023f84d04e7b9596ac24a8f5afe3e81be7729')):
        require(bound[name] == str(REPO / relative) and file_sha(absolute(bound[name])) == bound[name + '_sha256'] == digest,
                'immutable native module differs')
    for descriptor in ({'path': bound['actor_filename'], 'sha256': bound['actor_sha256']},
                       bound['actor_persistence_check'], bound['actor_initial_capture']):
        require(descriptor.keys() == {'path', 'sha256'}, 'actor descriptor fields differ')
        path = absolute(descriptor['path'])
        require(path.parent == run and file_sha(path) == descriptor['sha256'], 'bound fresh actor source differs')
    config = bound['controller_configuration']
    exact(config, {'root': str(run), 'recorderPath': bound['recorder'], 'evidenceKind': 'native',
                   'callCap': 100, 'elapsedCapS': 1200, 'baselineCalls': 0})
    context = {key: value for key, value in config.items() if key != 'root'}; context['recorderSha256'] = bound['recorder_sha256']
    exact(parsed(read(run / 'CONTEXT.json')), context)
    exact(bound['clean_source_metadata_receipt'], identity['clean_checks'])
    for key, names, descriptor in (('runtime', ['fmt', 'test', 'clippy', 'debug-build'], identity['runtime_checks']),
                                   ('clean_build', ['xtask', 'debug-build'], identity['clean_checks'])):
        exact(bound['integrated_local_gate_receipts'][key], descriptor)
        checks, path = BASE.receipt(descriptor, descriptor['sha256'])
        require(checks['source_hashes'] == bound['source_inputs'] and checks['source_count'] == 74 and
                checks['all_passed'] is True and checks['checkout_status_after'] == '' and
                checks['source74_unchanged'] is True and checks['checkout_clean_before'] is True and
                checks['checkout_clean_after'] is True and [step['name'] for step in checks['steps']] == names,
                'source74/gate/clean receipt mismatch')
        for step in checks['steps']:
            require(step['exit_code'] == 0, 'integrated current gate failed')
            for stream in ('stdout', 'stderr'):
                require(file_sha(absolute(str(path.parent / (step['name'] + '.' + stream)))) == step[stream + '_sha256'],
                        'current gate raw log differs')
        require(checks['runtime_commit'] == bound['source_commit'], 'runtime receipt commit differs')
        if key == 'clean_build':
            require(checks['checked_clean_commit'] == bound['clean_metadata_commit'], 'clean metadata commit differs')
        built = absolute(checks['binaries']['weftforge']['path'])
        require(built == target / 'debug/weftforge' and file_sha(built) == bound['native_binary_sha256'] and
                checks['binaries']['weftforge']['bytes'] == bound['native_binary_bytes'], 'own build original differs')


def dependency(run, bound):
    dep = bound['accepted_h1_dependency']
    exact(dep, parsed(read(ROOT / 'HISTORICAL-DEPENDENCY.json')))
    for name in ('source73', 'root_binding', 'snapshot', 'saved_grade_report', 'root_native_ui_review'):
        descriptor = dep[name]; require(descriptor.keys() == {'path', 'sha256'}, 'historical descriptor differs')
        require(file_sha(absolute(descriptor['path'])) == descriptor['sha256'], 'historical dependency bytes changed: ' + name)
    prior_path = absolute(dep['root_binding']['path']); prior = parsed(read(prior_path)); prior_run = prior_path.parent
    identity = parsed(read(absolute(dep['source73']['path'])))
    require(prior_path.name == 'ROOT-BINDING.json' and prior['case'] == dep['case'] == 'HOLE-H1' and
            prior['attempt'] == dep['attempt'] == 'H1-002' and prior['root_authorized_for_case'] is True and
            prior['native_lease_granted'] is True and prior['controller_configuration']['root'] == str(prior_run),
            'wrong historical H1 accepted binding')
    for key in ('source_commit', 'clean_metadata_commit', 'source_input_count', 'native_binary_sha256'):
        exact(prior[key], dep[key]); exact(identity[key], dep[key])
    require(prior['source_inputs'] == identity['source_inputs'] and len(identity['source_inputs']) == 73 and
            prior['source_commit'] != bound['source_commit'] and prior['native_binary_sha256'] != bound['native_binary_sha256'],
            'historical source73 confused with current source74 authority')
    grade = parsed(read(absolute(dep['saved_grade_report']['path'])))
    review = parsed(read(absolute(dep['root_native_ui_review']['path'])))
    for record in (grade, review):
        require(record['case'] == dep['case'] and record['attempt'] == dep['attempt'] and
                record['root_binding_sha256'] == dep['root_binding']['sha256'] and
                record['source_commit'] == dep['source_commit'] and record['native_binary_sha256'] == dep['native_binary_sha256'],
                'historical acceptance identity differs')
    require(grade['status'] == 'passed_saved_files_only' and grade['snapshot_sha256']['redo-r5'] == dep['snapshot']['sha256'] and
            review['schema'] == 'weftforge.straight-hole.native-ui-review.v1' and review['accepted'] is True and
            review['reviewed_raw_images_and_traces'] is True and review['single_saved_grade_report_sha256'] == dep['saved_grade_report']['sha256'] and
            review['accepted_redo_r5_sha256'] == dep['snapshot']['sha256'], 'historical sole grade/UI acceptance differs')
    require(Path(dep['snapshot']['path']) == prior_run / 'snapshots/redo-r5.weftforge' and
            Path(dep['saved_grade_report']['path']) == prior_run / 'SAVED-GRADE-001.json' and
            Path(dep['root_native_ui_review']['path']) == prior_run / 'ROOT-UI-ACCEPTANCE.json', 'historical evidence run differs')
    source = absolute(dep['snapshot']['path']); staged = absolute(str(run / 'inputs/accepted-h1-002-redo-r5.weftforge'))
    raw = read(staged)
    require(not source.stat().st_mode & 0o222 and not staged.stat().st_mode & 0o222 and raw == read(source) and
            sha(raw) == dep['snapshot']['sha256'] and bound['inputs'] == {'inputs/accepted-h1-002-redo-r5.weftforge': sha(raw)},
            'actual readonly H1 source/staged primary bytes differ')
    return parsed(raw)


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
    require(pid_before['method'] == 'external.pid' and pid_before['arguments'] == {'expected_binary': bound['native_binary_path'],
            'expected_command': ' '.join([bound['native_binary_path'], *bound['input_argv']])} and
            pid_after['result']['status'] == 0 and len(matches) == 1 and type(matches[0]['pid']) is int and matches[0]['pid'] > 0 and
            matches[0]['command'] == ' '.join([bound['native_binary_path'], *bound['input_argv']]), 'wrong live owned PID')
    require(paired(run, '0006')[0]['method'] == 'getApp' and paired(run, '0006')[0]['arguments']['args'] == [bound['bundle']] and
            paired(run, '0007')[0]['method'] == 'getAXStateAndScreenshot', 'initial target binding/capture route differs')

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
                receipt['schema'] == 'weftforge.straight-hole.export-copy.v1' and receipt['case'] == 'HOLE-H3' and
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


def after_closure_evidence(run, bound, binding_raw, closed):
    """Check root corroboration and opaque preservation before any construction assessment."""
    binding_sha = sha(binding_raw)
    closure_sha = sha(read(run / 'NATIVE-LEASE-CLOSED.json'))
    correlation = parsed(read(run / 'ROOT-TOOL-HISTORY-CORRELATION.json'))
    summary = parsed(read(run / 'COMPLETION-SUMMARY-RETURN.json'))
    expected = {'schema': 'weftforge.svg-hole-successor.root-history.v1', 'case': 'HOLE-H3', 'attempt': 'H3-001',
                'root_binding_sha256': binding_sha, 'input_closure_sha256': closure_sha,
                'captured_after_input_closure': True, 'root_correlated_with_actual_CUA_tool_history': True,
                'evidence_kind': 'post_case_actual_tool_history_corroboration', 'invocation_timestamp_exact': False,
                'standalone_initial_getState': True, 'actual_CUA_proof_source_record': '0002',
                'actual_CUA_proof_sha256': sha(read(run / 'ACTOR-RUNTIME-PROOF.json')),
                'bootstrap_code_sha256': bound['actor_sha256'],
                'readiness_code_sha256': bound['actor_persistence_check']['sha256'],
                'initial_capture_code_sha256': bound['actor_initial_capture']['sha256'],
                'terminal_source_record': closed['terminal_source_record'], 'real_Quit_last_native_observer': True,
                'completion_summary_very_next_actual_tool_call': True, 'completion_summary_calls': 1,
                'paired_native_external_operations': len(list((run / 'trace').glob('*-before.json'))),
                'documentation_refresh_calls': summary['documentation_refresh_calls'],
                'overall_actual_calls': summary['overall_actual_calls'],
                'native_input_after_terminal': False}
    require(correlation.keys() == expected.keys() | {'raw_actual_tool_history'}, 'root history fields differ')
    exact({key: correlation[key] for key in expected}, expected)
    descriptor = correlation['raw_actual_tool_history']
    require(descriptor.keys() == {'path', 'sha256'} and descriptor['path'] == 'ROOT-ACTUAL-TOOL-HISTORY.json' and
            file_sha(absolute(str(run / descriptor['path']))) == descriptor['sha256'] and
            expected['actual_CUA_proof_sha256'].encode() in read(run / descriptor['path']),
            'root actual tool history is missing/changed')
    exited = parsed(read(run / 'OWNED-PROCESS-EXIT.json'))
    exit_expected = {'schema': 'weftforge.svg-hole-successor.owned-exit.v1', 'case': 'HOLE-H3', 'attempt': 'H3-001',
                     'root_binding_sha256': binding_sha, 'captured_after_input_closure': True,
                     'input_closure_sha256': closure_sha,
                     'owned_pid': paired(run, '0005')[1]['result']['exact_matches'][0]['pid'],
                     'expected_binary': bound['native_binary_path'],
                     'expected_command': ' '.join([bound['native_binary_path'], *bound['input_argv']]),
                     'owned_process_exited': True}
    require(exited.keys() == exit_expected.keys() | {'raw_process_check'}, 'owned-exit fields differ')
    exact({key: exited[key] for key in exit_expected}, exit_expected)
    descriptor = exited['raw_process_check']
    require(descriptor.keys() == {'path', 'sha256'} and Path(descriptor['path']).name == descriptor['path'] and
            file_sha(absolute(str(run / descriptor['path']))) == descriptor['sha256'], 'exit raw process check differs')
    retained = parsed(read(run / 'RAW-RETENTION.json'))
    manifest_raw = read(run / 'RAW-ARTIFACT-MANIFEST.json'); manifest = parsed(manifest_raw)
    exact(retained, {'schema': 'weftforge.svg-hole-successor.raw-retention.v1', 'case': 'HOLE-H3', 'attempt': 'H3-001',
                     'root_binding_sha256': binding_sha, 'input_closure_sha256': closure_sha,
                     'captured_after_input_closure': True, 'raw_bytes_unedited': True,
                     'manifest': {'path': 'RAW-ARTIFACT-MANIFEST.json', 'sha256': sha(manifest_raw)},
                     'artifacts_count': len(manifest['files'])})
    require(manifest.keys() == {'schema', 'files'} and manifest['schema'] == 'weftforge.svg-hole-successor.raw-manifest.v1'
            and type(manifest['files']) is dict and manifest['files'], 'raw manifest missing/empty/extra fields')
    required = set()
    for directory in ('trace', 'captures', 'snapshots', 'exports', 'exports-retained', 'work', 'inputs', 'offline-proof'):
        required.update(str(path.relative_to(run)) for path in (run / directory).rglob('*') if path.is_file())
    required.update(('ROOT-BINDING.json', 'LAUNCH-CLOCK.json', 'CONTEXT.json', 'ACTOR-RUNTIME-PROOF.json',
                     'ACTOR-PERSISTENCE-CHECK-RESULT.json', 'COMPLETION-SUMMARY-RETURN.json',
                     'NATIVE-LEASE-CLOSED.json', 'ACTOR-RUNTIME-TOOL-HISTORY.json', 'ACTOR-RUNTIME-RAW-TOOL-HISTORY.json'))
    required.update(Path(value).name for value in (bound['actor_filename'], bound['actor_persistence_check']['path'],
                                                 bound['actor_initial_capture']['path']))
    require(manifest['files'].keys() == required, 'raw manifest omits or adds evidence, including input sidecars')
    for name, entry in manifest['files'].items():
        require(not Path(name).is_absolute() and '..' not in Path(name).parts, 'unsafe raw manifest path')
        path = absolute(str(run / name))
        exact(entry, {'bytes': path.stat().st_size, 'mode': path.stat().st_mode & 0o777, 'sha256': file_sha(path)})


def verify_freeze():
    freeze = parsed(read(ROOT / 'FREEZE.json'))
    require(freeze.keys() == {'schema', 'case', 'attempt', 'files', 'external_dependencies'} and
            freeze['schema'] == 'weftforge.svg-hole-successor.freeze.v1' and freeze['case'] == 'HOLE-H3' and
            freeze['attempt'] == 'H3-001', 'wrong closed final freeze schema')
    artifacts = {str(path.relative_to(ROOT)) for path in ROOT.rglob('*') if path.is_file() and path != ROOT / 'FREEZE.json'}
    dependencies = parsed(read(ROOT / 'DEPENDENCIES.json'))
    require(dependencies.keys() == {'schema', 'files', 'source74_and_gate_coverage_rule'} and dependencies['files'] and
            dependencies['schema'] == 'weftforge.svg-hole-successor.dependencies.v1', 'empty/invalid declared dependencies')
    external = dependencies['files']
    source = parsed(read(ROOT / 'SOURCE-74.json')); source_complete(source)
    required = {str(FROZEN_PATH.relative_to(REPO)), str(FROZEN.BASE_PATH.relative_to(REPO)),
                str(BASE.OLD.relative_to(REPO)), '.local/straight-hole-native-plan-v2-20261116/PLAN.json',
                '.local/straight-hole-native-plan-v2-20261116/fixtures/R240-v5-assessment.weftforge',
                '.local/straight-hole-native-plan-v2-20261116/grade_exports.py',
                '.local/straight-hole-native-plan-v2-20261116/retain_saved.py',
                '.local/straight-hole-native-plan-v2-20261116/fixtures/hole-r6.dxf',
                '.local/straight-hole-native-plan-v2-20261116/fixtures/hole-r6.dxf.recovery',
                '.local/straight-holes-20261116/spec/features/PAT-006-straight-hole.md',
                '.local/svg-export-bounds-20261116/DIAGNOSTIC-001.md'}
    dep = parsed(read(ROOT / 'HISTORICAL-DEPENDENCY.json'))
    required.update(str(Path(dep[key]['path']).relative_to(REPO)) for key in
                    ('source73', 'root_binding', 'snapshot', 'saved_grade_report', 'root_native_ui_review'))
    template = parsed(read(ROOT / 'ROOT-BINDING.HOLE-H3.template.json'))
    required.update(str(Path(template[key]).relative_to(REPO)) for key in ('controller', 'recorder', 'observer'))
    required.update(str((Path(source['source_worktree']) / name).relative_to(REPO)) for name in source['source_inputs'])
    for key in ('runtime_checks', 'clean_checks'):
        descriptor = source[key]; checks, path = BASE.receipt(descriptor, descriptor['sha256'])
        required.add(str(path.relative_to(REPO)))
        required.update(str((path.parent / (step['name'] + '.' + stream)).relative_to(REPO))
                        for step in checks['steps'] for stream in ('stdout', 'stderr'))
    coverage_complete(freeze, artifacts, external, required)
    for names, base in ((freeze['files'], ROOT), (external, REPO)):
        for name, digest in names.items():
            require(type(name) is str and not Path(name).is_absolute() and '..' not in Path(name).parts and
                    type(digest) is str and re.fullmatch(HEX, digest) and file_sha(absolute(str(base / name))) == digest,
                    'frozen artifact/dependency bytes changed: ' + name)
    require(file_sha(ROOT / 'grade_exports.py') == '56ab4c66ea5d70846b532e3abae951e2af1e2c1ea0a9baee5a924a154d06ba11',
            'frozen export criteria weakened')


def coverage_complete(freeze, artifacts, external, required):
    require(MANDATORY_ARTIFACTS <= artifacts and freeze['files'] and freeze['files'].keys() == artifacts,
            'freeze coverage omitted mandatory PLAN/model/SOURCE/provenance/grader/retainer/templates or draft history')
    require(external and required <= external.keys() and freeze['external_dependencies'] == external,
            'freeze omitted declared/transitive/current-source/historical dependencies; no prior-source exceptions')


def grade_run(case, run, bound, binding_raw, closed):
    require(bound['freeze_sha256'] == sha(read(ROOT / 'FREEZE.json')), 'wrong final H3 packet binding')
    identity_preflight(run, bound)
    for name in ('ACTOR-RUNTIME-PROOF', 'ACTOR-PERSISTENCE-CHECK-RESULT', 'ACTOR-RUNTIME-TOOL-HISTORY',
                 'NATIVE-LEASE-CLOSED', 'POSTFLIGHT', 'COMPLETION-SUMMARY-RETURN'):
        closed_shape(parsed(read(run / (name + '.json'))), parsed(read(ROOT / (name + '.template.json'))))
    BASE.runtime_preflight(run, bound, binding_raw)
    readiness_preflight(run, bound, binding_raw)
    postflight(run, bound, binding_raw, closed)
    after_closure_evidence(run, bound, binding_raw, closed)
    clock = int(parsed(read(run / 'LAUNCH-CLOCK.json'))['monotonic_ns']); latest = clock
    records = sorted((run / 'trace').glob('*-before.json')); spec = CASES[case]
    require(7 <= len(records) <= spec['cap'], 'missing trace/lab operation cap exceeded')
    for index, path in enumerate(records, 1):
        record = str(index).zfill(4); require(path.name == record + '-before.json', 'noncontiguous H3 trace')
        before, after = paired(run, record)
        require(latest <= int(before['monotonic_ns']) <= int(after['monotonic_ns']), 'continuous clock chronology differs')
        latest = int(after['monotonic_ns'])
    summary = parsed(read(run / 'COMPLETION-SUMMARY-RETURN.json'))
    require(summary.keys() == parsed(read(ROOT / 'COMPLETION-SUMMARY-RETURN.template.json')).keys() and
            summary['schema'] == 'weftforge.straight-cut.completion-summary-return.v1' and summary['case'] == case and
            summary['attempt'] == bound['attempt'] and summary['root_binding_sha256'] == sha(binding_raw) and
            type(summary['summary_calls']) is int and summary['summary_calls'] == 1 and
            summary['paired_native_external_operations'] == len(records) and type(summary['documentation_refresh_calls']) is int and
            summary['documentation_refresh_calls'] >= 0 and summary['overall_actual_calls'] ==
            len(records) + 1 + summary['documentation_refresh_calls'] <= 101 and
            summary['summary_invocation_time_exact'] is False and summary['native_controller_pair_claimed_for_summary'] is False,
            'summary/refresh/binding counts differ')
    lower, upper = (int(summary[key]['monotonic_ns']) for key in
                    ('lower_bound_before_summary_invocation', 'upper_bound_after_summary_return'))
    require(clock <= lower <= upper and latest <= upper and (upper - clock) / 1e9 <= 1200 and
            closed['terminal_source_record'] == str(len(records)).zfill(4) and closed['outcome'] == 'completed',
            'continuous lab clock/terminal/completed outcome differs')
    documents, hashes = check_receipts(run, case)
    result = grade_states(documents, dependency(run, bound))
    result['exports'] = grade_export_receipts(run, documents['reopened-r6'])
    return {'status': 'passed_saved_files_only', 'case': case, 'attempt': bound['attempt'],
            'source_commit': bound['source_commit'], 'native_binary_sha256': bound['native_binary_sha256'],
            'historical_h1_dependency_sha256': sha(read(ROOT / 'HISTORICAL-DEPENDENCY.json')),
            'root_binding_sha256': sha(binding_raw), 'freeze_sha256': sha(read(ROOT / 'FREEZE.json')),
            'snapshot_sha256': hashes, 'result': result, 'native_ui_verified': False, 'reference_verified': False,
            'full_feature_accepted': False}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--case', choices=CASES, required=True)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    require(args.run.is_absolute() and args.run == args.run.resolve(), 'absolute canonical owned run required')
    run = args.run
    require(run == REPO / '.local/native-svg-export-successor-20261116/H3-001', 'fresh H3 owned run differs')
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
