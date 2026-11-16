#!/usr/bin/env python3
"""Small synthetic-only checks. No product, CUA, actual-case grade or marker."""
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
    states = G.models()
    G.grade_states('HOLE-H1', states)
    reopened = copy.deepcopy(states['redo-r5']); reopened['project']['revision'] = 6
    G.item(reopened, 'points', 12)['position'][0] = .08
    G.grade_states('HOLE-H2', {'reopened-r6': reopened}, states['redo-r5'])
    rejected = []

    def bad(name, invoke):
        try:
            invoke()
        except (AssertionError, ValueError, KeyError, TypeError):
            rejected.append(name)
        else:
            raise AssertionError('synthetic mutant accepted: ' + name)

    altered = copy.deepcopy(states)
    G.item(altered['hole-r2'], 'pieces', 2).pop('holes')
    bad('missing required v5 holes', lambda: G.grade_states('HOLE-H1', altered))
    altered = copy.deepcopy(states); altered['hole-r2']['project']['next_id'] = 11
    bad('wrong Hole allocator', lambda: G.grade_states('HOLE-H1', altered))
    altered = copy.deepcopy(states); G.item(altered['undo-r4'], 'points', 12)['position'][0] = .06
    bad('Undo did not restore corner bits', lambda: G.grade_states('HOLE-H1', altered))
    altered_r6 = copy.deepcopy(reopened); G.item(altered_r6, 'points', 3)['position'][0] = -0.
    bad('reopen lost unchanged signed zero', lambda: G.grade_states('HOLE-H2', {'reopened-r6': altered_r6}, states['redo-r5']))
    template = G.parsed(G.read(G.ROOT / 'ROOT-BINDING.HOLE-H1.template.json'))
    def fill(value):
        return {key: fill(v) for key, v in value.items()} if isinstance(value, dict) else [fill(v) for v in value] if isinstance(value, list) else 'SYNTHETIC-ONLY' if value is None else value
    complete = fill(template); complete.update(root_authorized_for_case=True, native_lease_granted=True, accepted_h1_dependency=None)
    complete['source_inputs'] = G.parsed(G.read(G.ROOT / 'SOURCE-73.json'))['source_inputs']
    G.final_binding_complete(complete)
    complete['missing_root_UI_dependency'] = True
    bad('extra draft diagnostic in final binding', lambda: G.final_binding_complete(complete))
    vertices = [*E.OUTER, (.08, .12), (.16, .12), (.16, .24), (.08, .24)]
    faces = [(0, 1, 5), (0, 5, 4), (1, 2, 6), (1, 6, 5), (2, 3, 7), (2, 7, 6), (3, 0, 4), (3, 4, 7)]
    obj = ''.join(f'v {u} {v} 0\n' for u, v in vertices)
    obj += ''.join(f'vt {u} {v}\n' for u, v in vertices)
    obj += 'vn 0 0 1\n' * len(vertices) + 'g piece_2\n'
    obj += ''.join('f ' + ' '.join(f'{n+1}/{n+1}/{n+1}' for n in face) + '\n' for face in faces)
    E.grade_obj(obj.encode())
    bad('OBJ cap fills open hole', lambda: E.grade_obj((obj + 'f 5/5/5 6/6/6 7/7/7\n').encode()))
    bad('OBJ wrong world/UV scale', lambda: E.grade_obj(obj.replace('v 0.0 0.0 0', 'v 0.0 0.0 1', 1).encode()))
    data = 'M 0 0 L 240 0 L 240 -360 L 0 -360 L 0 0 Z M 80 -120 L 80 -240 L 160 -240 L 160 -120 L 80 -120 Z'
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="240mm" height="360mm" viewBox="0 -360 240 360"><path id="piece-2" data-name="Original Hole R240" d="{data}" fill="none" fill-rule="evenodd" stroke="black" stroke-width="0.3"/></svg>'.encode()
    E.grade_svg(svg)
    bad('SVG missing evenodd semantics', lambda: E.grade_svg(svg.replace(b'evenodd', b'nonzero')))
    bad('SVG wrong physical scale', lambda: E.grade_svg(svg.replace(b'240mm', b'24mm')))
    with tempfile.TemporaryDirectory(prefix='SYNTHETIC-ONLY-', dir=G.ROOT) as name:
        root = Path(name)
        def write(path, value):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(value))
        for record, method in [('0001', 'click'), ('0002', 'getAXStateAndScreenshot'), ('0003', 'fs.raw_snapshot')]:
            write(root / 'trace' / (record + '-before.json'), {'id': record, 'method': method})
            if record != '0003':
                write(root / 'trace' / (record + '-after.json'), {'id': record, 'status': 'returned'})
        primary = root / G.CASES['HOLE-H1']['primary']; primary.parent.mkdir()
        primary.write_bytes(b'SYNTHETIC OPAQUE BYTES ONLY\n -0.0\n')
        receipt = R.retain(root, 'HOLE-H1', 'parent-r1', '0001', '0002', '0003', 'synthetic_selfcheck')
        G.require((root / 'snapshots/parent-r1.weftforge').read_bytes() == primary.read_bytes() and receipt['construction_graded'] is False, 'synthetic opaque copy changed')
        binding = {'case': 'HOLE-H1', 'attempt': 'SYNTHETIC-ONLY', 'bundle': str(root / 'mock.app'), 'native_binary_path': str(root / 'mock.app/Contents/MacOS/weftforge'),
                   'launcher_path': '/usr/bin/open', 'launcher_argv': ['-n', str(root / 'mock.app')], 'input_argv': [],
                   'controller_configuration': {'root': str(root), 'recorderPath': 'synthetic', 'evidenceKind': 'native', 'callCap': 200, 'elapsedCapS': 1800, 'baselineCalls': 0}}
        binding_raw = json.dumps(binding).encode(); proof_raw = b'{"SYNTHETIC_ONLY_NOT_ACTUAL_PROOF":true}'
        (root / 'ACTOR-RUNTIME-PROOF.json').write_bytes(proof_raw)
        ready = {'schema': 'weftforge.straight-hole.actor-readiness.v1', 'case': 'HOLE-H1', 'attempt': binding['attempt'], 'root_binding_sha256': G.sha(binding_raw),
                 'source_record': '0003', 'actual_cua_runtime': True, 'native_UI_calls': 0, 'helper_types': G.HELPER_TYPES,
                 'helper_flags': {key: True for key in G.HELPER_TYPES}, 'helpers_ready': True, 'root': {'expected': str(root), 'actual': str(root)},
                 'controller_configuration': {'expected': binding['controller_configuration'], 'actual': binding['controller_configuration']},
                 'context_schema_ready': True, 'proof_schema_ready': True, 'proof_result_ready': True, 'offline_artifact_ready': True, 'controller_status_ready': True,
                 'proof_sha256': G.sha(proof_raw)}
        write(root / 'ACTOR-PERSISTENCE-CHECK-RESULT.json', ready)
        write(root / 'trace/0003-before.json', {'id': '0003', 'method': 'local-runtime-bindings-check'})
        write(root / 'trace/0003-after.json', {'id': '0003', 'status': 'returned', 'result': ready})
        for record, method, args, result in [('0004', 'external.open', {'executable': binding['launcher_path'], 'argv': binding['launcher_argv']}, {}),
                    ('0005', 'external.pid', {'expected_binary': binding['native_binary_path']}, {'status': 0, 'exact_matches': [{'pid': 1, 'command': binding['native_binary_path']}]}),
                    ('0006', 'getApp', {'args': [binding['bundle']]}, {}), ('0007', 'getAXStateAndScreenshot', {}, {})]:
            write(root / 'trace' / (record + '-before.json'), {'id': record, 'method': method, 'arguments': args})
            write(root / 'trace' / (record + '-after.json'), {'id': record, 'status': 'returned', 'result': result})
        G.readiness_preflight(root, binding, binding_raw)
        ready['helper_flags']['holeObserve'] = False
        write(root / 'ACTOR-PERSISTENCE-CHECK-RESULT.json', ready)
        bad('readiness cannot omit one cross-cell helper flag', lambda: G.readiness_preflight(root, binding, binding_raw))
        G.require(not (root / 'SAVED-GRADE-ATTEMPT.json').exists(), 'synthetic check used formal marker')
    return {'status': 'passed_small_synthetic_state_export_copy_readiness_checks_only', 'rejected_mutants': rejected,
            'actual_native_CUA_product_evaluator_or_formal_grade_executed': False, 'temporary_synthetic_files_deleted': True}


if __name__ == '__main__':
    report = Path(sys.argv[1]); G.require(not report.exists(), 'fresh selfcheck report required')
    result = run()
    with report.open('x') as stream:
        json.dump(result, stream, indent=2); stream.write('\n')
    print(json.dumps(result))
