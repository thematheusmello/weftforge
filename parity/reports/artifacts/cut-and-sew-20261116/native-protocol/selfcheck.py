#!/usr/bin/env python3
"""Offline synthetic selfchecks only; no product, controller, clock or real case."""
import copy
import json
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
import grade_saved as G
import retain_saved as R


def run():
    models = G.models()
    s1 = {label: copy.deepcopy(models[label]) for label in G.CASES['SEW-S1']['labels']}
    s2 = {label: copy.deepcopy(models[label]) for label in G.CASES['SEW-S2']['labels']}
    G.grade_states('SEW-S1', s1, models['parent-r1'])
    G.grade_states('SEW-S2', s2, models['redo-r4'])
    rejected = []

    def bad(name, invoke):
        try:
            invoke()
        except (AssertionError, ValueError, KeyError, TypeError, FileExistsError):
            rejected.append(name)
        else:
            raise AssertionError('synthetic mutant accepted: ' + name)

    def s1_mutant(name, label, edit):
        altered = copy.deepcopy(s1); edit(altered[label])
        bad(name, lambda: G.grade_states('SEW-S1', altered, models['parent-r1']))

    s1_mutant('v5 holes field missing', 'sewn-r2', lambda doc: G.item(doc, 'pieces', 2).pop('holes'))
    s1_mutant('unknown project field', 'sewn-r2', lambda doc: doc['project'].update(unknown=True))
    s1_mutant('wrong allocator after previews', 'sewn-r2', lambda doc: doc['project'].update(next_id=31))
    s1_mutant('wrong retained parent name', 'sewn-r2', lambda doc: G.item(doc, 'pieces', 2).update(name='Original Cut R240'))
    s1_mutant('global point order changed', 'sewn-r2', lambda doc: doc['project']['points'].reverse())
    s1_mutant('chord incidence changed', 'sewn-r2', lambda doc: G.item(doc, 'curves', 19).update(start=16, end=17))
    s1_mutant('seam ID reused', 'sewn-r2', lambda doc: G.item(doc, 'seams', 20).update(id=19))
    s1_mutant('seam interval wrongly forward B', 'sewn-r2', lambda doc: G.item(doc, 'seams', 20)['b'].update(t0=0., t1=1.))
    s1_mutant('seam map changed', 'sewn-r2', lambda doc: G.item(doc, 'seams', 20).update(correspondence=[[0., 1.], [1., 0.]]))
    s1_mutant('Undo lost allocator highwater', 'undo-r3', lambda doc: doc['project'].update(next_id=11))
    s1_mutant('Redo assigned new seam ID', 'redo-r4', lambda doc: G.item(doc, 'seams', 20).update(id=30))
    s1_mutant('unchanged signed zero changed', 'sewn-r2', lambda doc: G.item(doc, 'points', 3)['position'].__setitem__(0, -0.))
    s1_mutant('RGB f32 changed', 'sewn-r2', lambda doc: G.item(doc, 'materials', 1)['color'].__setitem__(0, .23))
    s1_mutant('new anchor outside 1e-12m budget', 'sewn-r2', lambda doc: G.item(doc, 'points', 11)['position'].__setitem__(0, .090001))
    s1_mutant('copied anchor not bit exact', 'sewn-r2', lambda doc: G.item(doc, 'points', 16)['position'].__setitem__(0, .09 + 1e-16))
    s1_mutant('retained piece gained hole', 'sewn-r2', lambda doc: G.item(doc, 'pieces', 2)['holes'].append({'id': 21, 'boundary': []}))
    wrong_orientation = copy.deepcopy(models['sewn-r2'])
    G.item(wrong_orientation, 'seams', 20)['b'].update(t0=0., t1=1.)
    bad('independent endpoint rule rejects equal-length matching-midpoint forward B', lambda: G.measure_cut(wrong_orientation))
    good = copy.deepcopy(s1)
    for label in ('sewn-r2', 'redo-r4'):
        for identity in (11, 13, 16, 17):
            G.item(good[label], 'points', identity)['position'][0] += 2e-13
    G.grade_states('SEW-S1', good, models['parent-r1'])
    bad_redo = copy.deepcopy(good)
    for identity in (11, 13, 16, 17):
        G.item(bad_redo['redo-r4'], 'points', identity)['position'][0] += 1e-16
    bad('Redo must retain actual R2 anchor bits even within tolerance', lambda: G.grade_states('SEW-S1', bad_redo, models['parent-r1']))
    rgb_equivalent = copy.deepcopy(s1)
    for doc in rgb_equivalent.values():
        G.item(doc, 'materials', 1)['color'][0] = .2199999988079071
    G.grade_states('SEW-S1', rgb_equivalent, models['parent-r1'])
    altered = copy.deepcopy(s2)
    altered['removed-r5']['project']['seams'] = copy.deepcopy(models['redo-r4']['project']['seams'])
    bad('RemoveSeam left constraint', lambda: G.grade_states('SEW-S2', altered, models['redo-r4']))
    altered = copy.deepcopy(s2)
    G.item(altered['removed-r5'], 'points', 13)['position'][1] = .40
    bad('RemoveSeam edited geometry', lambda: G.grade_states('SEW-S2', altered, models['redo-r4']))
    altered = copy.deepcopy(s2)
    G.item(altered['child-height-r6'], 'points', 13)['position'][1] = .40
    bad('child height changed retained anchor13', lambda: G.grade_states('SEW-S2', altered, models['redo-r4']))
    altered = copy.deepcopy(s2)
    G.item(altered['child-height-r6'], 'points', 5)['position'][0] += 1e-16
    bad('child height changed unchanged U bit', lambda: G.grade_states('SEW-S2', altered, models['redo-r4']))
    altered = copy.deepcopy(s2); altered['child-height-r6']['project']['next_id'] = 22
    bad('height edit allocated ID', lambda: G.grade_states('SEW-S2', altered, models['redo-r4']))
    template = G.parsed(G.read(G.ROOT / 'ROOT-BINDING.SEW-S1.template.json'))
    bad('false grants and pending fields block final binding', lambda: G.final_binding_complete(template))
    extra = copy.deepcopy(template); extra['draft_ready'] = False
    bad('extra top-level draft flag forbidden', lambda: G.closed_keys(extra, template))
    extra = copy.deepcopy(template); extra['controller_configuration']['draft_ready'] = False
    bad('extra nested draft flag forbidden', lambda: G.closed_keys(extra, template))
    bad('duplicate input JSON field', lambda: G.parsed(b'{"version":5,"version":5}'))
    bad('nonfinite input JSON', lambda: G.parsed(b'{"coordinate":NaN}'))

    with tempfile.TemporaryDirectory(prefix='SYNTHETIC-ONLY-', dir=G.ROOT) as name:
        root = Path(name)

        def write(path, value):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(value))

        def pair(record, method, arguments=None, result=None, returned=True):
            write(root / 'trace' / (record + '-before.json'),
                  {'id': record, 'method': method, 'arguments': arguments or {}, 'monotonic_ns': int(record) * 2})
            if returned:
                write(root / 'trace' / (record + '-after.json'),
                      {'id': record, 'status': 'returned', 'result': result or {}, 'monotonic_ns': int(record) * 2 + 1})

        pair('0001', 'click'); pair('0002', 'getAXStateAndScreenshot'); pair('0003', 'fs.raw_snapshot', returned=False)
        primary = root / G.CASES['SEW-S1']['primary']; primary.parent.mkdir()
        primary.write_bytes(b'SYNTHETIC OPAQUE BYTES; deliberately not project JSON\n -0.0\n')
        receipt = R.retain(root, 'SEW-S1', 'parent-r1', '0001', '0002', '0003', 'synthetic_selfcheck')
        G.require((root / 'snapshots/parent-r1.weftforge').read_bytes() == primary.read_bytes() and
                  receipt['construction_graded'] is False and receipt['evidence_kind'] == 'synthetic_selfcheck', 'opaque copy altered/graded bytes')
        bad('opaque retainer cannot overwrite snapshots', lambda: R.retain(root, 'SEW-S1', 'parent-r1', '0001', '0002', '0003', 'synthetic_selfcheck'))
        bound = {'case': 'SEW-S1', 'attempt': 'SYNTHETIC-ONLY', 'bundle': str(root / 'mock.app'),
                 'native_binary_path': str(root / 'mock.app/Contents/MacOS/weftforge'),
                 'launcher_path': '/usr/bin/open', 'input_argv': [str(root / 'inputs/accepted-parent-r1.weftforge')],
                 'controller_configuration': {'root': str(root), 'recorderPath': 'SYNTHETIC-ONLY', 'evidenceKind': 'native',
                                              'callCap': G.CASES['SEW-S1']['cap'], 'elapsedCapS': G.CASES['SEW-S1']['seconds'], 'baselineCalls': 0}}
        bound['launcher_argv'] = ['-n', bound['bundle'], '--args', *bound['input_argv']]
        binding_raw = json.dumps(bound).encode(); proof_raw = b'{"SYNTHETIC_ONLY_NOT_ACTUAL_PROOF":true}'
        (root / 'ACTOR-RUNTIME-PROOF.json').write_bytes(proof_raw)
        ready = {'schema': 'weftforge.straight-cut-and-sew.actor-readiness.v1', 'case': bound['case'], 'attempt': bound['attempt'],
                 'root_binding_sha256': G.sha(binding_raw), 'source_record': '0003', 'actual_cua_runtime': True, 'native_UI_calls': 0,
                 'helper_types': G.HELPER_TYPES, 'helper_flags': {key: True for key in G.HELPER_TYPES}, 'helpers_ready': True,
                 'root': {'expected': str(root), 'actual': str(root)},
                 'controller_configuration': {'expected': bound['controller_configuration'], 'actual': bound['controller_configuration']},
                 'context_schema_ready': True, 'proof_schema_ready': True, 'proof_result_ready': True,
                 'offline_artifact_ready': True, 'controller_status_ready': True, 'proof_sha256': G.sha(proof_raw)}
        write(root / 'ACTOR-PERSISTENCE-CHECK-RESULT.json', ready)
        pair('0003', 'local-runtime-bindings-check', result=ready)
        command = ' '.join([bound['native_binary_path'], *bound['input_argv']])
        pair('0004', 'external.open', {'executable': bound['launcher_path'], 'argv': bound['launcher_argv']})
        pair('0005', 'external.pid', {'expected_binary': bound['native_binary_path'], 'expected_command': command},
             {'status': 0, 'exact_matches': [{'pid': 1, 'command': command}]})
        pair('0006', 'getApp', {'args': [bound['bundle']]}); pair('0007', 'getAXStateAndScreenshot')
        G.readiness_preflight(root, bound, binding_raw)
        ready['helper_flags']['sewObserve'] = False; write(root / 'ACTOR-PERSISTENCE-CHECK-RESULT.json', ready)
        bad('readiness requires every cross-cell helper', lambda: G.readiness_preflight(root, bound, binding_raw))
        ready['helper_flags']['sewObserve'] = True; write(root / 'ACTOR-PERSISTENCE-CHECK-RESULT.json', ready)
        before = G.parsed(G.read(root / 'trace/0005-before.json'))
        del before['arguments']['expected_command']; write(root / 'trace/0005-before.json', before)
        bad('PID proof requires full expected_command including input argv', lambda: G.readiness_preflight(root, bound, binding_raw))
        before['arguments']['expected_command'] = command; write(root / 'trace/0005-before.json', before)
        after = G.parsed(G.read(root / 'trace/0005-after.json'))
        after['result']['exact_matches'][0]['command'] = bound['native_binary_path']; write(root / 'trace/0005-after.json', after)
        bad('binary-only PID match is insufficient', lambda: G.readiness_preflight(root, bound, binding_raw))
        after['result']['exact_matches'][0]['command'] = command; write(root / 'trace/0005-after.json', after)
        G.readiness_preflight(root, bound, binding_raw)
        write(root / 'LAUNCH-CLOCK.json', {'monotonic_ns': 0, 'synthetic_only': True})
        summary = {'schema': 'weftforge.straight-cut.completion-summary-return.v1', 'case': bound['case'], 'attempt': bound['attempt'],
                   'root_binding_sha256': G.sha(binding_raw), 'summary_calls': 1, 'paired_native_external_operations': 7,
                   'documentation_refresh_calls': 1, 'overall_actual_calls': 9, 'summary_invocation_time_exact': False,
                   'native_controller_pair_claimed_for_summary': False,
                   'lower_bound_before_summary_invocation': {'monotonic_ns': 15}, 'upper_bound_after_summary_return': {'monotonic_ns': 17}}
        write(root / 'COMPLETION-SUMMARY-RETURN.json', summary)
        G.trace_preflight(root, bound['case'], bound, binding_raw, {'terminal_source_record': '0007'})
        summary.update(documentation_refresh_calls=0, overall_actual_calls=8); write(root / 'COMPLETION-SUMMARY-RETURN.json', summary)
        bad('mandatory documentation refresh cannot disappear from count', lambda: G.trace_preflight(root, bound['case'], bound, binding_raw, {'terminal_source_record': '0007'}))
        G.require(not (root / 'SAVED-GRADE-ATTEMPT.json').exists(), 'selfcheck created formal marker')
    G.require(not (G.ROOT / 'FREEZE.json').exists(), 'draft selfcheck may not create/use a freeze')
    return {'status': 'passed_offline_synthetic_state_metadata_readiness_opaque_copy_checks_only',
            'rejected_mutants': rejected, 'allowed_new_anchor_roundoff_with_exact_copy_and_redo_bits': True,
            'RGB_f32_equivalent_decimal_accepted': True, 'product_native_CUA_controller_clock_or_actual_case_grade_executed': False,
            'synthetic_clock_numbers_are_literals_only': True, 'temporary_synthetic_files_deleted': True,
            'freeze_created': False, 'source_or_native_acceptance_claim': False}


if __name__ == '__main__':
    report = Path(sys.argv[1]); G.require(not report.exists(), 'fresh synthetic report required')
    result = run()
    with report.open('x') as output:
        json.dump(result, output, indent=2, allow_nan=False); output.write('\n')
    print(json.dumps(result))
