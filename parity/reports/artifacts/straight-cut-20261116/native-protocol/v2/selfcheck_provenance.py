#!/usr/bin/env python3
"""Temporary fabricated provenance only; no product/CUA or formal grade runs."""
import copy
import json
import plistlib
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
import grade_saved as G


def raw(value):
    return (json.dumps(value, indent=2) + '\n').encode()


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(raw(value))


def fixture(run):
    """Fake 69 files, binary, receipts and CUA history; none are executed."""
    model = copy.deepcopy(G.parsed(G.read(G.ROOT / 'SOURCE-69.json')))
    source = run / 'synthetic-source'
    for name in model['source_inputs']:
        path = source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(('SYNTHETIC ONLY ' + name).encode())
    model.update(source_commit='a' * 40, clean_metadata_commit='b' * 40)
    model['source_inputs'] = {name: G.file_sha(source / name) for name in model['source_inputs']}
    target = source / 'target'
    built = target / 'debug/weftforge'
    built.parent.mkdir(parents=True)
    built.write_bytes(b'SYNTHETIC ONLY NOT A PRODUCT BINARY\n')
    model.update(native_binary_sha256=G.file_sha(built), native_binary_bytes=built.stat().st_size)
    bundle = run / 'SyntheticOnly.app'
    binary = bundle / 'Contents/MacOS/weftforge'
    binary.parent.mkdir(parents=True)
    binary.write_bytes(built.read_bytes())
    plist_path = bundle / 'Contents/Info.plist'
    plist_path.write_bytes(plistlib.dumps({'CFBundleIdentifier': 'org.synthetic.only', 'CFBundleExecutable': 'weftforge', 'CFBundlePackageType': 'APPL'}))
    actor = run / 'ACTOR-RUNTIME.cjs'
    actor.write_bytes(b'// SYNTHETIC ONLY; never executed\n')
    bound = G.parsed(G.read(G.ROOT / 'ROOT-BINDING.CUT-R1.template.json'))
    bound.update(attempt='SYNTHETIC-ONLY', root_authorized_for_case=True, native_lease_granted=True,
                 source_commit=model['source_commit'], clean_metadata_commit=model['clean_metadata_commit'],
                 source_worktree=str(source), source_inputs=model['source_inputs'], own_target_directory=str(target),
                 bundle=str(bundle), native_binary_path=str(binary), native_binary_sha256=model['native_binary_sha256'],
                 bundle_identifier='org.synthetic.only', bundle_plist_sha256=G.file_sha(plist_path), os_version='99.1',
                 launcher_argv=['-n', str(bundle)], actor_filename=str(actor), actor_sha256=G.file_sha(actor))
    bound['controller_configuration']['root'] = str(run)
    gates = {}
    for key, names in [('runtime', ['fmt', 'test', 'clippy']), ('clean_build', ['xtask', 'debug-build'])]:
        checks_path = run / 'synthetic-gates' / key / 'CHECKS.json'
        checks_path.parent.mkdir(parents=True)
        steps = []
        for name in names:
            step = {'name': name, 'exit_code': 0}
            for stream in ('stdout', 'stderr'):
                path = checks_path.parent / (name + '.' + stream)
                path.write_bytes(('SYNTHETIC ONLY ' + name + stream).encode())
                step[stream + '_sha256'] = G.file_sha(path)
            steps.append(step)
        checks = {'source_hashes': model['source_inputs'], 'source_count': 69, 'steps': steps}
        if key == 'runtime':
            checks.update(source_commit=model['source_commit'], checkout_status_before='', checkout_status_after='',
                          runtime_inputs_unchanged=True, all_three_passed=True)
        else:
            checks.update(runtime_commit=model['source_commit'], checked_clean_commit=model['clean_metadata_commit'],
                          source69_unchanged=True, checkout_clean_after=True, all_passed=True,
                          fresh_owned_default_target_no_cache_copy=True,
                          binaries={'weftforge': {'path': str(built), 'sha256': model['native_binary_sha256'], 'bytes': model['native_binary_bytes']}})
        write(checks_path, checks)
        gates[key] = {'path': str(checks_path), 'sha256': G.file_sha(checks_path)}
        model['runtime_checks_sha256' if key == 'runtime' else 'clean_build_checks_sha256'] = gates[key]['sha256']
    bound['integrated_local_gate_receipts'] = gates
    bound['clean_source_metadata_receipt'] = gates['clean_build']
    binding_raw = raw(bound)
    (run / 'ROOT-BINDING.json').write_bytes(binding_raw)
    context = {key: value for key, value in bound['controller_configuration'].items() if key != 'root'}
    context['recorderSha256'] = bound['recorder_sha256']
    write(run / 'CONTEXT.json', context)
    write(run / 'LAUNCH-CLOCK.json', {'monotonic_ns': '0'})
    for record, method, before_ns in [('0001', 'getState', 1), ('0002', 'local-filesystem-proof', 3)]:
        write(run / 'trace' / (record + '-before.json'), {'id': record, 'method': method, 'monotonic_ns': str(before_ns)})
        if record == '0001':
            write(run / 'trace' / (record + '-after.json'), {'id': record, 'status': 'returned', 'monotonic_ns': '2'})
    offline = run / 'offline-proof'
    write(offline / 'trace/0001-before.json', {'id': '0001', 'method': 'opaque-binding-proof'})
    write(offline / 'trace/0001-after.json', {'id': '0001', 'status': 'returned', 'result': {'raw_binding_sha256': G.sha(binding_raw)}})
    artifact = {'file': 'captures/binding-raw.json', 'bytes': len(binding_raw), 'sha256': G.sha(binding_raw),
                'source_record': '0001', 'unedited': True, 'utc': '2026-11-16T00:00:00Z', 'monotonic_ns': '3'}
    artifact_path = offline / artifact['file']
    artifact_path.parent.mkdir()
    artifact_path.write_bytes(binding_raw)
    write(Path(str(artifact_path) + '.sha256.json'), artifact)
    proof = G.parsed(G.read(G.ROOT / 'ACTOR-RUNTIME-PROOF.template.json'))
    proof.update(attempt=bound['attempt'], root_binding_sha256=G.sha(binding_raw), evidence_kind='native_cua_runtime',
                 actual_cua_runtime=True, native_UI_calls=0, createRequire=True, fs_writable=True,
                 actor_filename=bound['actor_filename'], actor_sha256=bound['actor_sha256'],
                 controller_configuration=bound['controller_configuration'], marker_initialized_synchronous_before_observer=True,
                 offline_receipt=artifact)
    write(run / 'ACTOR-RUNTIME-PROOF.json', proof)
    proof_sha = G.file_sha(run / 'ACTOR-RUNTIME-PROOF.json')
    write(run / 'trace/0002-after.json', {'id': '0002', 'status': 'returned', 'monotonic_ns': '4',
          'result': {'proof_sha256': proof_sha, 'actual_cua_runtime': True, 'native_UI_calls': 0}})
    closed = G.parsed(G.read(G.ROOT / 'NATIVE-LEASE-CLOSED.template.json'))
    closed.update(attempt=bound['attempt'], native_input_lease_closed=True, outcome='completed',
                  root_binding_sha256=G.sha(binding_raw), terminal_source_record='0002', planned_quit_source_record='0002',
                  immediate_completion_summary_calls=1)
    write(run / 'NATIVE-LEASE-CLOSED.json', closed)
    history_path = run / 'ACTOR-RUNTIME-RAW-TOOL-HISTORY.json'
    write(history_path, {'SYNTHETIC_ONLY_NOT_REAL_CUA_HISTORY': proof_sha})
    history = G.parsed(G.read(G.ROOT / 'ACTOR-RUNTIME-TOOL-HISTORY.template.json'))
    history.update(attempt=bound['attempt'], root_binding_sha256=G.sha(binding_raw), proof_sha256=proof_sha,
                   root_correlated_with_actual_CUA_tool_history=True, evidence_kind='post_case_tool_history_corroboration',
                   captured_after_input_closure=True, input_closure_sha256=G.file_sha(run / 'NATIVE-LEASE-CLOSED.json'),
                   raw_tool_history={'path': history_path.name, 'sha256': G.file_sha(history_path)})
    write(run / 'ACTOR-RUNTIME-TOOL-HISTORY.json', history)
    write(run / 'COMPLETION-SUMMARY-RETURN.json', {'synthetic_only': True})
    post = {'schema': 'weftforge.straight-cut.postflight.v1', 'case': 'CUT-R1', 'attempt': bound['attempt'],
            'root_binding_sha256': G.sha(binding_raw), 'input_closure_sha256': G.file_sha(run / 'NATIVE-LEASE-CLOSED.json'),
            'source_commit': bound['source_commit'], 'native_binary_sha256': bound['native_binary_sha256'],
            'source_inputs_sha256': G.inventory_sha(bound), 'controller_context_sha256': G.file_sha(run / 'CONTEXT.json'),
            'runtime_proof_sha256': proof_sha, 'runtime_tool_history_sha256': G.file_sha(run / 'ACTOR-RUNTIME-TOOL-HISTORY.json'),
            'terminal_source_record': '0002', 'paired_native_external_operations': 2,
            'completion_summary_return_sha256': G.file_sha(run / 'COMPLETION-SUMMARY-RETURN.json'),
            'source_inputs_unchanged': True, 'native_binary_unchanged': True}
    write(run / 'POSTFLIGHT.json', post)
    return bound, binding_raw, model, closed


def run():
    rejected = []

    def rejects(name, invoke):
        try:
            invoke()
        except (AssertionError, ValueError, KeyError, TypeError):
            rejected.append(name)
        else:
            raise AssertionError('synthetic provenance mutant accepted: ' + name)

    with tempfile.TemporaryDirectory(prefix='SYNTHETIC-PROVENANCE-ONLY-', dir=G.ROOT) as name:
        case = Path(name)
        bound, binding_raw, model, closed = fixture(case)
        G.grade_scope(case, case / 'SAVED-GRADE-001.json', 'CUT-R1', bound, closed)
        G.identity_preflight(case, bound, model)
        G.runtime_preflight(case, bound, binding_raw)
        G.postflight_preflight(case, bound, binding_raw, closed)

        def bad_bound(name, mutate):
            altered = copy.deepcopy(bound)
            mutate(altered)
            rejects(name, lambda: G.identity_preflight(case, altered, model))

        for field in ('source_worktree', 'own_target_directory', 'bundle', 'native_binary_path', 'bundle_identifier',
                      'bundle_plist_sha256', 'native_binary_sha256', 'os_version', 'launcher_argv', 'actor_filename', 'actor_sha256'):
            bad_bound('null bound ' + field, lambda d, field=field: d.__setitem__(field, None))
        bad_bound('arbitrary named source69', lambda d: d['source_inputs'].__setitem__('arbitrary', d['source_inputs'].pop('Cargo.toml')))
        bad_bound('metadata receipt hash mismatch', lambda d: d.__setitem__('clean_source_metadata_receipt', {'path': 'wrong', 'sha256': '0' * 64}))
        bad_bound('wrong immutable controller', lambda d: d.__setitem__('controller_sha256', '0' * 64))
        bad_bound('existing directory is not own build target', lambda d: d.__setitem__('own_target_directory', str(case)))
        altered = copy.deepcopy(bound)
        altered['root_authorized_for_case'] = False
        rejects('unbound template cannot create marker scope', lambda: G.grade_scope(case, case / 'SAVED-GRADE-001.json', 'CUT-R1', altered, closed))
        rejects('wrong report path', lambda: G.grade_scope(case, case / 'elsewhere.json', 'CUT-R1', bound, closed))
        rejects('wrong actual case scope', lambda: G.grade_scope(case, case / 'SAVED-GRADE-001.json', 'CUT-R2', bound, closed))

        def bad_json(name, filename, mutate, invoke):
            path = case / filename
            original = path.read_bytes()
            altered = json.loads(original)
            mutate(altered)
            path.write_bytes(raw(altered))
            rejects(name, invoke)
            path.write_bytes(original)

        identity = lambda: G.identity_preflight(case, bound, model)
        runtime = lambda: G.runtime_preflight(case, bound, binding_raw)
        postflight = lambda: G.postflight_preflight(case, bound, binding_raw, closed)
        bad_json('CONTEXT invented root field', 'CONTEXT.json', lambda d: d.__setitem__('root', str(case)), identity)
        bad_json('CONTEXT missing recorder hash', 'CONTEXT.json', lambda d: d.pop('recorderSha256'), identity)
        bad_json('CONTEXT wrong counter baseline', 'CONTEXT.json', lambda d: d.__setitem__('baselineCalls', 1), identity)
        for field, value in [('actual_cua_runtime', False), ('evidence_kind', 'offline_mock'), ('native_UI_calls', 1),
                             ('case', 'CUT-R2'), ('attempt', 'another'), ('root_binding_sha256', '0' * 64),
                             ('actor_filename', None), ('controller_sha256', '0' * 64)]:
            bad_json('runtime proof ' + field, 'ACTOR-RUNTIME-PROOF.json', lambda d, field=field, value=value: d.__setitem__(field, value), runtime)
        bad_json('runtime proof missing six-key recorderPath', 'ACTOR-RUNTIME-PROOF.json', lambda d: d['controller_configuration'].pop('recorderPath'), runtime)
        bad_json('runtime proof wrong opaque-byte hash', 'ACTOR-RUNTIME-PROOF.json', lambda d: d['offline_receipt'].__setitem__('sha256', '0' * 64), runtime)
        bad_json('runtime second operation result mismatch', 'trace/0002-after.json', lambda d: d['result'].__setitem__('proof_sha256', '0' * 64), runtime)
        bad_json('runtime second operation not FS proof', 'trace/0002-before.json', lambda d: d.__setitem__('method', 'getApp'), runtime)
        bad_json('history claims exact invocation timestamp', 'ACTOR-RUNTIME-TOOL-HISTORY.json', lambda d: d.__setitem__('invocation_timestamp_exact', True), runtime)
        bad_json('Node report cannot be CUA history', 'ACTOR-RUNTIME-TOOL-HISTORY.json', lambda d: d.__setitem__('tool', 'standard_Node'), runtime)
        bad_json('history from another binding', 'ACTOR-RUNTIME-TOOL-HISTORY.json', lambda d: d.__setitem__('root_binding_sha256', '0' * 64), runtime)
        for field in ('root_binding_sha256', 'input_closure_sha256', 'runtime_proof_sha256', 'native_binary_sha256'):
            bad_json('postflight ' + field, 'POSTFLIGHT.json', lambda d, field=field: d.__setitem__(field, '0' * 64), postflight)
        bad_json('postflight wrong operation count', 'POSTFLIGHT.json', lambda d: d.__setitem__('paired_native_external_operations', 3), postflight)
        proof_path = case / 'ACTOR-RUNTIME-PROOF.json'
        saved = proof_path.read_bytes()
        proof_path.unlink()
        rejects('missing actual runtime proof', runtime)
        proof_path.write_bytes(saved)
        source_file = Path(bound['source_worktree']) / 'Cargo.toml'
        saved = source_file.read_bytes()
        source_file.write_bytes(b'SYNTHETIC altered source')
        rejects('actual named source bytes changed', identity)
        source_file.write_bytes(saved)
        binary = Path(bound['native_binary_path'])
        saved = binary.read_bytes()
        binary.write_bytes(b'SYNTHETIC altered binary')
        rejects('actual bound binary bytes changed', identity)
        binary.write_bytes(saved)
        # Exact same synthetic R4 bytes in two distinct fake R1 runs cannot share acceptance.
        r4 = raw({'SYNTHETIC_ONLY_OPAQUE_NOT_V4': True})
        (case / 'snapshots').mkdir()
        (case / 'snapshots/redo-r4.weftforge').write_bytes(r4)
        grade = {'status': 'passed_saved_files_only', 'case': 'CUT-R1', 'attempt': bound['attempt'],
                 'source_commit': bound['source_commit'], 'native_binary_sha256': bound['native_binary_sha256'],
                 'root_binding_sha256': G.sha(binding_raw), 'snapshot_sha256': {'redo-r4': G.sha(r4)}}
        write(case / 'SAVED-GRADE-001.json', grade)
        review = {'schema': 'weftforge.straight-cut.native-ui-review.v2', 'case': 'CUT-R1', 'attempt': bound['attempt'],
                  'accepted': True, 'reviewed_raw_images_and_traces': True, 'accepted_redo_r4_sha256': G.sha(r4),
                  'root_binding_sha256': G.sha(binding_raw), 'single_saved_grade_report_sha256': G.file_sha(case / 'SAVED-GRADE-001.json'),
                  'source_commit': bound['source_commit'], 'native_binary_sha256': bound['native_binary_sha256']}
        write(case / 'ROOT-UI-ACCEPTANCE.json', review)
        r2 = case / 'SYNTHETIC-R2'
        (r2 / 'inputs').mkdir(parents=True)
        input_path = r2 / 'inputs/accepted-redo-r4.weftforge'
        input_path.write_bytes(r4)
        input_path.chmod(0o444)
        dep = {'case': 'CUT-R1', 'snapshot_label': 'redo-r4', 'root_accepted': True, 'attempt': bound['attempt'],
               'root_binding_path': str(case / 'ROOT-BINDING.json'), 'root_binding_sha256': G.sha(binding_raw),
               'snapshot_path': str(case / 'snapshots/redo-r4.weftforge'), 'snapshot_sha256': G.sha(r4),
               'saved_grade_report': str(case / 'SAVED-GRADE-001.json'), 'saved_grade_report_sha256': G.file_sha(case / 'SAVED-GRADE-001.json'),
               'root_native_ui_review': str(case / 'ROOT-UI-ACCEPTANCE.json'), 'root_native_ui_review_sha256': G.file_sha(case / 'ROOT-UI-ACCEPTANCE.json')}
        bound2 = {'accepted_cut_r1_dependency': dep, 'inputs': {'inputs/accepted-redo-r4.weftforge': G.sha(r4)},
                  'source_commit': bound['source_commit'], 'native_binary_sha256': bound['native_binary_sha256']}
        G.dependency(r2, bound2)
        for field, value in [('root_binding_sha256', '0' * 64), ('single_saved_grade_report_sha256', '0' * 64),
                             ('attempt', 'another'), ('source_commit', 'c' * 40), ('native_binary_sha256', '0' * 64)]:
            changed = copy.deepcopy(review)
            changed[field] = value
            write(case / 'ROOT-UI-ACCEPTANCE.json', changed)
            changed_bound = copy.deepcopy(bound2)
            changed_bound['accepted_cut_r1_dependency']['root_native_ui_review_sha256'] = G.file_sha(case / 'ROOT-UI-ACCEPTANCE.json')
            rejects('R1 UI acceptance ' + field, lambda: G.dependency(r2, changed_bound))
        write(case / 'ROOT-UI-ACCEPTANCE.json', review)
        another = case / 'SYNTHETIC-OTHER-R1'
        (another / 'snapshots').mkdir(parents=True)
        other_bound = copy.deepcopy(bound)
        other_bound['attempt'] = 'ANOTHER-SYNTHETIC-RUN'
        other_bound['controller_configuration']['root'] = str(another)
        write(another / 'ROOT-BINDING.json', other_bound)
        (another / 'snapshots/redo-r4.weftforge').write_bytes(r4)
        other_grade = copy.deepcopy(grade)
        other_grade.update(attempt=other_bound['attempt'], root_binding_sha256=G.file_sha(another / 'ROOT-BINDING.json'))
        write(another / 'SAVED-GRADE-001.json', other_grade)
        write(another / 'ROOT-UI-ACCEPTANCE.json', review)  # Old run's acceptance, same R4 bytes.
        other_dep = copy.deepcopy(dep)
        other_dep.update(attempt=other_bound['attempt'], root_binding_path=str(another / 'ROOT-BINDING.json'),
                         root_binding_sha256=G.file_sha(another / 'ROOT-BINDING.json'), snapshot_path=str(another / 'snapshots/redo-r4.weftforge'),
                         saved_grade_report=str(another / 'SAVED-GRADE-001.json'), saved_grade_report_sha256=G.file_sha(another / 'SAVED-GRADE-001.json'),
                         root_native_ui_review=str(another / 'ROOT-UI-ACCEPTANCE.json'))
        other_bound2 = copy.deepcopy(bound2)
        other_bound2['accepted_cut_r1_dependency'] = other_dep
        rejects('byte-identical R4 cannot substitute another R1 run acceptance', lambda: G.dependency(r2, other_bound2))
        G.require(not list(case.rglob('SAVED-GRADE-ATTEMPT.json')), 'synthetic check created formal attempt marker')
    return {'status': 'passed_synthetic_provenance_fixtures_only', 'actual_native_CUA_product_or_case_grade_executed': False,
            'formal_entrypoint_or_attempt_marker_used': False, 'temporary_fabricated_files_deleted': True,
            'new_provenance_mutants_rejected': len(rejected), 'checks': rejected}


if __name__ == '__main__':
    parser = __import__('argparse').ArgumentParser()
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    G.require(not args.report.exists(), 'fresh synthetic report required')
    result = run()
    with args.report.open('x') as output:
        json.dump(result, output, indent=2)
        output.write('\n')
    print(json.dumps(result))
