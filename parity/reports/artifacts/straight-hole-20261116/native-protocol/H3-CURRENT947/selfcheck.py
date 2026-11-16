#!/usr/bin/env python3
"""Synthetic checks for this mutable draft only; no old case, entrypoint, product or CUA."""
import copy
import json
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
import grade_saved as G
import grade_exports as E
import retain_saved as R


def run():
    rejected = []

    def bad(name, invoke):
        try:
            invoke()
        except (AssertionError, ValueError, KeyError, TypeError):
            rejected.append(name)
        else:
            raise AssertionError('synthetic mutant accepted: ' + name)

    G.require(not (G.ROOT / 'FREEZE.json').exists(), 'draft unexpectedly frozen')
    G.require(G.file_sha(G.ROOT / 'grade_exports.py') ==
              '56ab4c66ea5d70846b532e3abae951e2af1e2c1ea0a9baee5a924a154d06ba11', 'export grader bytes changed')
    G.require(list(G.CASES) == ['HOLE-H3'] and len(G.HELPER_TYPES) == 20, 'case/helper scope differs')
    plan = G.PLAN['cases']['HOLE-H3']
    G.require(sum(step[1] for step in plan['steps']) == plan['static_operations'] == 95 and
              plan['operation_cap'] == 100 and plan['summary_calls'] == 1 and plan['overall_cap'] == 101 and
              plan['elapsed_cap_s'] == 1200 and plan['reserve'] == 5 and plan['steps'][0][1] == 13,
              'original bounded H3 planning envelope changed')
    source = G.parsed(G.read(G.ROOT / 'SOURCE-74.json'))
    pending = copy.deepcopy(source); pending['clean_metadata_commit'] = None
    bad('pending clean identity cannot authorize source74', lambda: G.source_complete(pending))
    bound = G.parsed(G.read(G.ROOT / 'ROOT-BINDING.HOLE-H3.template.json'))
    bad('pending ungranted binding cannot be final', lambda: G.final_binding_complete(bound))
    extra = copy.deepcopy(bound); extra['extra_draft_flag'] = True
    bad('extra final binding flag rejected', lambda: G.closed_shape(extra, bound))
    extra = copy.deepcopy(bound); extra['controller_configuration']['extra'] = True
    bad('extra nested constructor flag rejected', lambda: G.closed_shape(extra, bound))
    G.require(bound['source_commit'] != bound['accepted_h1_dependency']['source_commit'] and
              bound['native_binary_sha256'] != bound['accepted_h1_dependency']['native_binary_sha256'],
              'historical H1 collapsed into current authority')
    G.require(not (G.ROOT / 'SAVED-GRADE-ATTEMPT.json').exists(), 'formal marker unexpectedly exists')

    # Synthetic assessment model only, never written as a native input.
    r5 = G.FROZEN.models()['redo-r5']
    r6 = copy.deepcopy(r5); r6['project']['revision'] = 6
    G.item(r6, 'points', 12)['position'][0] = .08
    G.grade_states({'reopened-r6': r6}, r5)
    wrong = copy.deepcopy(r6); G.item(wrong, 'points', 12)['position'][0] = .081
    bad('incorrect H3 edit rejected by literal math', lambda: G.grade_states({'reopened-r6': wrong}, r5))
    wrong = copy.deepcopy(r6); G.item(wrong, 'points', 3)['position'][0] = -0.
    bad('unchanged signed zero retained', lambda: G.grade_states({'reopened-r6': wrong}, r5))
    wrong = copy.deepcopy(r6); G.item(wrong, 'pieces', 2).pop('holes')
    bad('required v5 Hole11 retained', lambda: G.grade_states({'reopened-r6': wrong}, r5))
    wrong = copy.deepcopy(r6); wrong['project']['next_id'] = 21
    bad('allocator cannot drift', lambda: G.grade_states({'reopened-r6': wrong}, r5))

    vertices = [(0., 0.), (.24, 0.), (.24, .36), (0., .36),
                (.08, .12), (.16, .12), (.16, .24), (.08, .24)]
    faces = [(0, 1, 5), (0, 5, 4), (1, 2, 6), (1, 6, 5),
             (2, 3, 7), (2, 7, 6), (3, 0, 4), (3, 4, 7)]

    def obj_for(points):
        raw = ''.join(f'v {u} {v} 0\n' for u, v in points)
        raw += ''.join(f'vt {u} {v}\n' for u, v in points)
        raw += 'vn 0 0 1\n' * len(points) + 'g piece_2\n'
        return raw + ''.join('f ' + ' '.join(f'{n+1}/{n+1}/{n+1}' for n in face) + '\n' for face in faces)

    obj = obj_for(vertices).encode(); E.grade_obj(obj, r6)
    data = 'M 0 0 L 240 0 L 240 -360 L 0 -360 L 0 0 Z M 80 -120 L 80 -240 L 160 -240 L 160 -120 L 80 -120 Z'
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="240mm" height="360mm" viewBox="0 -360 240 360"><path id="piece-2" data-name="Original Hole R240" d="{data}" fill="none" fill-rule="evenodd" stroke="black" stroke-width="0.3"/></svg>'.encode()
    E.grade_svg(svg, r6)
    bad('old padded one-metre canvas stays failed', lambda: E.grade_svg(svg.replace(b'240mm', b'1000mm').replace(
        b'360mm', b'1000mm').replace(b'0 -360 240 360', b'0 -1000 1000 1000'), r6))
    bad('wrong SVG scale rejected', lambda: E.grade_svg(svg.replace(b'240mm', b'24mm'), r6))
    bad('SVG evenodd cannot be weakened', lambda: E.grade_svg(svg.replace(b'evenodd', b'nonzero'), r6))
    bad('hole cap rejected', lambda: E.grade_obj(obj + b'f 5/5/5 6/6/6 7/7/7\n', r6))
    bad('wrong normal rejected', lambda: E.grade_obj(obj.replace(b'vn 0 0 1', b'vn 0 1 0'), r6))
    bad('overlapping face rejected', lambda: E.grade_obj(obj + b'f 1/1/1 2/2/2 6/6/6\n', r6))
    rounded_r5, rounded_r6 = copy.deepcopy(r5), copy.deepcopy(r6)
    for doc in (rounded_r5, rounded_r6):
        G.item(doc, 'points', 13)['position'][0] = .08 + 9e-13
    G.grade_states({'reopened-r6': rounded_r6}, rounded_r5)
    rounded_vertices = list(vertices); rounded_vertices[7] = (.08 + 9e-13, .24)
    E.grade_obj(obj_for(rounded_vertices).encode(), rounded_r6)
    E.grade_svg(svg.replace(b'L 80 -240', f'L {(.08+9e-13)*1000.:.17g} -240'.encode()), rounded_r6)
    wrong_vertices = list(rounded_vertices); wrong_vertices[7] = (.081, .24)
    bad('exports use exact already-graded R6 domain', lambda: E.grade_obj(obj_for(wrong_vertices).encode(), rounded_r6))

    artifacts = set(G.MANDATORY_ARTIFACTS)
    external = {'synthetic-prior-source-helper': '0' * 64}
    freeze = {'files': dict.fromkeys(artifacts, '0' * 64), 'external_dependencies': external}
    G.coverage_complete(freeze, artifacts, external, set(external))
    missing = copy.deepcopy(freeze); missing['files'].pop('MODEL.json')
    bad('freeze cannot omit literal model', lambda: G.coverage_complete(missing, artifacts, external, set(external)))
    empty = copy.deepcopy(freeze); empty['external_dependencies'] = {}
    bad('freeze cannot omit external map', lambda: G.coverage_complete(empty, artifacts, external, set(external)))
    bad('prior-source dependency cannot be exempted', lambda: G.coverage_complete(freeze, artifacts, external,
                                                                                 {'synthetic-omitted-prior-source'}))

    with tempfile.TemporaryDirectory(prefix='SYNTHETIC-H3-ONLY-', dir=G.ROOT) as name:
        run = Path(name)

        def write(path, value):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(value))

        for record, method in [('0001', 'click'), ('0002', 'getAXStateAndScreenshot'), ('0003', 'fs.raw_snapshot')]:
            write(run / 'trace' / (record + '-before.json'), {'id': record, 'method': method})
            if record != '0003':
                write(run / 'trace' / (record + '-after.json'), {'id': record, 'status': 'returned'})
        primary = run / G.CASES['HOLE-H3']['primary']; primary.parent.mkdir()
        primary.write_bytes(b'SYNTHETIC OPAQUE BYTES ONLY\n -0.0\n')
        receipt = R.retain(run, 'HOLE-H3', 'reopened-r6', '0001', '0002', '0003', 'synthetic_selfcheck')
        G.require((run / 'snapshots/reopened-r6.weftforge').read_bytes() == primary.read_bytes() and
                  receipt['case'] == 'HOLE-H3' and receipt['construction_graded'] is False, 'opaque H3 retention changed bytes')
        G.require((G.ROOT / 'fixtures/hole-r6.dxf').stat().st_size == 41 and
                  (G.ROOT / 'fixtures/hole-r6.dxf.recovery').stat().st_size == 42, 'literal sentinel sizes changed')
        exports = run / 'exports'; exports.mkdir()
        for suffix in ('', '.recovery'):
            (exports / ('hole-r6.dxf' + suffix)).write_bytes((G.ROOT / ('fixtures/hole-r6.dxf' + suffix)).read_bytes())
        R.retain_export(run, 'HOLE-H3', 'hole-r6.dxf', '0001', '0002', '0003')
        for suffix in ('', '.recovery'):
            G.require((run / ('exports-retained/hole-r6.dxf' + suffix)).read_bytes() ==
                      (exports / ('hole-r6.dxf' + suffix)).read_bytes(), 'opaque sentinel retention changed bytes')
        binding = {'case': 'HOLE-H3', 'attempt': 'SYNTHETIC-ONLY', 'bundle': str(run / 'mock.app'),
                   'native_binary_path': str(run / 'mock.app/Contents/MacOS/weftforge'), 'launcher_path': '/usr/bin/open',
                   'input_argv': [str(run / 'inputs/SYNTHETIC-ONLY.weftforge')],
                   'launcher_argv': ['-n', str(run / 'mock.app'), '--args', str(run / 'inputs/SYNTHETIC-ONLY.weftforge')],
                   'controller_configuration': {'root': str(run), 'recorderPath': 'SYNTHETIC-ONLY', 'evidenceKind': 'native',
                                                'callCap': 100, 'elapsedCapS': 1200, 'baselineCalls': 0}}
        binding_raw = json.dumps(binding).encode(); proof_raw = b'{"SYNTHETIC_ONLY_NOT_ACTUAL_PROOF":true}'
        (run / 'ACTOR-RUNTIME-PROOF.json').write_bytes(proof_raw)
        ready = G.parsed(G.read(G.ROOT / 'ACTOR-PERSISTENCE-CHECK-RESULT.template.json'))
        ready.update(attempt=binding['attempt'], root_binding_sha256=G.sha(binding_raw), actual_cua_runtime=True,
                     native_UI_calls=0, helper_flags=dict.fromkeys(G.HELPER_TYPES, True), helpers_ready=True,
                     root={'expected': str(run), 'actual': str(run)},
                     controller_configuration={'expected': binding['controller_configuration'], 'actual': binding['controller_configuration']},
                     context_schema_ready=True, proof_schema_ready=True, proof_result_ready=True,
                     offline_artifact_ready=True, controller_status_ready=True, proof_sha256=G.sha(proof_raw))
        write(run / 'ACTOR-PERSISTENCE-CHECK-RESULT.json', ready)
        write(run / 'trace/0003-before.json', {'id': '0003', 'method': 'local-runtime-bindings-check'})
        write(run / 'trace/0003-after.json', {'id': '0003', 'status': 'returned', 'result': ready})
        expected_command = ' '.join([binding['native_binary_path'], *binding['input_argv']])
        for record, method, args, result in [
                ('0004', 'external.open', {'executable': binding['launcher_path'], 'argv': binding['launcher_argv']}, {}),
                ('0005', 'external.pid', {'expected_binary': binding['native_binary_path'], 'expected_command': expected_command},
                 {'status': 0, 'exact_matches': [{'pid': 1, 'command': expected_command}]}),
                ('0006', 'getApp', {'args': [binding['bundle']]}, {}),
                ('0007', 'getAXStateAndScreenshot', {}, {})]:
            write(run / 'trace' / (record + '-before.json'), {'id': record, 'method': method, 'arguments': args})
            write(run / 'trace' / (record + '-after.json'), {'id': record, 'status': 'returned', 'result': result})
        G.readiness_preflight(run, binding, binding_raw)
        pid = G.parsed(G.read(run / 'trace/0005-before.json')); pid['arguments'].pop('expected_command')
        write(run / 'trace/0005-before.json', pid)
        bad('PID must bind full command including input', lambda: G.readiness_preflight(run, binding, binding_raw))
        pid['arguments']['expected_command'] = binding['native_binary_path']; write(run / 'trace/0005-before.json', pid)
        bad('PID cannot omit input from expected command', lambda: G.readiness_preflight(run, binding, binding_raw))
        pid['arguments']['expected_command'] = expected_command; write(run / 'trace/0005-before.json', pid)
        ready['helper_flags']['holeObserve'] = False; write(run / 'ACTOR-PERSISTENCE-CHECK-RESULT.json', ready)
        bad('all twenty cross-cell helpers are required', lambda: G.readiness_preflight(run, binding, binding_raw))
        G.require(not (run / 'SAVED-GRADE-ATTEMPT.json').exists(), 'selfcheck created formal attempt')
    return {'status': 'passed_new_draft_synthetic_checks_only', 'rejected_mutants': rejected,
            'allowed_roundoff_with_matching_R6_OBJ_SVG': True, 'exact_export_grader_preserved': True,
            'actual_or_old_case_grade_entrypoint_executed': False, 'product_CUA_controller_or_clock_executed': False,
            'temporary_synthetic_files_deleted': True, 'freeze_or_GO_created': False}


if __name__ == '__main__':
    path = Path(sys.argv[1]); G.require(path.parent.resolve() == G.ROOT and not path.exists(), 'fresh in-packet report required')
    try:
        result = run()
    except Exception as problem:
        result = {'status': 'failed_new_draft_synthetic_check', 'error_type': type(problem).__name__, 'error': str(problem),
                  'actual_or_old_case_grade_entrypoint_executed': False, 'freeze_or_GO_created': False}
    with path.open('x') as stream:
        json.dump(result, stream, indent=2); stream.write('\n')
    print(json.dumps(result))
    sys.exit(0 if result['status'] == 'passed_new_draft_synthetic_checks_only' else 1)
