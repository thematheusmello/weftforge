#!/usr/bin/env python3
"""Draft independent saved-file grader. It never launches or imports WeftForge."""
import argparse
import copy
import hashlib
import json
import math
import re
import struct
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parent
LIMIT = 16 * 1024 * 1024


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON field: ' + key)
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError('nonfinite JSON token: ' + value)


def parsed(raw):
    require(len(raw) <= LIMIT, 'file exceeds 16 MiB')
    return json.loads(raw, object_pairs_hook=unique, parse_constant=reject_constant)


def read(path):
    require(path.is_file() and not path.is_symlink(), 'missing/nonregular/symlink file: ' + str(path))
    require(path.stat().st_size <= LIMIT, 'oversized file: ' + str(path))
    return path.read_bytes()


def bits(value, single=False):
    require(type(value) in (int, float) and math.isfinite(value), 'nonfinite or nonnumeric authoring value')
    return struct.pack('!f' if single else '!d', value)


def exact(actual, expected, path=()):
    where = '/'.join(map(str, path)) or 'document'
    if isinstance(expected, dict):
        require(isinstance(actual, dict) and actual.keys() == expected.keys(), 'fields differ: ' + where)
        for key in expected:
            exact(actual[key], expected[key], (*path, key))
    elif isinstance(expected, list):
        require(isinstance(actual, list) and len(actual) == len(expected), 'list differs: ' + where)
        for index, (a, e) in enumerate(zip(actual, expected)):
            exact(a, e, (*path, index))
    elif type(expected) is int:
        require(type(actual) is int and actual == expected, 'integer/ID differs: ' + where)
    elif type(expected) is float:
        single = len(path) >= 2 and path[-2] == 'color'
        require(bits(actual, single) == bits(expected, single), 'f32/f64 bits differ: ' + where)
    else:
        require(type(actual) is type(expected) and actual == expected, 'value differs: ' + where)


def item(doc, collection, identity):
    matches = [x for x in doc['project'][collection] if x['id'] == identity]
    require(len(matches) == 1, 'missing/duplicate ' + collection + ' ID ' + str(identity))
    return matches[0]


def near(actual, expected, tolerance, message):
    require(len(actual) == len(expected), message + ' dimensions')
    require(all(type(a) in (int, float) and math.isfinite(a) and abs(a - e) <= tolerance
                for a, e in zip(actual, expected)), message)


def multiply(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]


def rotate(angles):
    x, y, z = angles
    sx, cx, sy, cy, sz, cz = math.sin(x), math.cos(x), math.sin(y), math.cos(y), math.sin(z), math.cos(z)
    return multiply([[cz, -sz, 0.], [sz, cz, 0.], [0., 0., 1.]],
                    multiply([[cy, 0., sy], [0., 1., 0.], [-sy, 0., cy]],
                             [[1., 0., 0.], [0., cx, -sx], [0., sx, cx]]))


def world(piece, position):
    matrix = rotate(piece['placement']['rotation'])
    return [sum(row[k] * [*position, 0.][k] for k in range(3)) + t
            for row, t in zip(matrix, piece['placement']['translation'])]


def grade_turn(actual, expected, resized):
    before_piece = item(resized, 'pieces', 2)
    after_piece = item(actual, 'pieces', 2)
    require(after_piece['placement'].keys() == {'translation','rotation'}, 'unknown/missing placement field')
    expected = copy.deepcopy(expected)
    before_matrix = rotate(before_piece['placement']['rotation'])
    after_matrix = rotate(after_piece['placement']['rotation'])
    quarter_inverse = [[0., 1., 0.], [-1., 0., 0.], [0., 0., 1.]]
    near(sum(after_matrix, []), sum(multiply(before_matrix, quarter_inverse), []),
         1e-12, 'quarter-turn placement compensation differs')
    near(after_piece['placement']['translation'], [0., 0., 0.], 1e-10, 'quarter-turn placement translation differs')
    for point, target in zip(actual['project']['points'], expected['project']['points']):
        require(point['id'] == target['id'], 'transform point identity/order changed')
        near(point['position'], target['position'], 1e-10, 'quarter-turn omitted/changed an endpoint')
        original = item(resized, 'points', point['id'])['position']
        near(world(after_piece, point['position']), world(before_piece, original), 1e-10,
             'arranged endpoint moved during pattern transform')
        target['position'] = copy.deepcopy(point['position'])
    angle = after_piece['grain_angle']
    require(type(angle) in (int, float) and math.isfinite(angle) and -math.pi <= angle <= math.pi,
            'noncanonical grain angle')
    old = before_piece['grain_angle']
    near([math.cos(angle), math.sin(angle)], [-math.sin(old), math.cos(old)], 1e-12,
         'quarter-turn grain compensation differs')
    old_world = [sum(row[k] * [math.cos(old), math.sin(old), 0.][k] for k in range(3))
                 for row in before_matrix]
    new_world = [sum(row[k] * [math.cos(angle), math.sin(angle), 0.][k] for k in range(3))
                 for row in after_matrix]
    near(new_world, old_world, 1e-10, 'world warp moved')
    model_piece = item(expected, 'pieces', 2)
    model_piece['placement'] = copy.deepcopy(after_piece['placement'])
    model_piece['grain_angle'] = angle
    exact(actual, expected)


def grade_states(case, documents, models):
    expected = models['states'][case]
    require(documents.keys() == expected.keys(), 'missing or extra saved state labels')
    for label, model in expected.items():
        actual = documents[label]
        if case == 'B' and label in models['transform']['affected_states']:
            grade_turn(actual, model, expected['B10-resized'])
        else:
            exact(actual, model)
    if case == 'B':
        restored = copy.deepcopy(documents['B15-owner-restored'])
        restored['project']['revision'] = 11
        exact(restored, documents['B11-turned'])
        removed = copy.deepcopy(documents['B11-turned'])
        removed['project']['revision'] = 12
        removed['project']['points'] = [p for p in removed['project']['points'] if p['id'] not in (11, 12)]
        removed['project']['curves'] = [c for c in removed['project']['curves'] if c['id'] != 13]
        item(removed, 'pieces', 2)['internal_curves'] = []
        exact(documents['B12-line-removed'], removed)
    return {'saved_states': len(documents), 'full_v4_construction_checked': True,
            'ids_order_ownership_allocator_revision': 'exact',
            'unchanged_authoring': 'exact f64 bits, color f32 bits',
            'transform': 'independent quarter-turn/world/grain checks' if case == 'B' else None}


def normalized_dxf(raw, revision):
    require(len(raw) <= LIMIT and len(raw.splitlines()) % 2 == 0, 'invalid DXF pair structure')
    lines = raw.splitlines(keepends=True)
    comments = [i + 1 for i in range(0, len(lines), 2)
                if lines[i].strip() == b'999' and lines[i + 1].strip().startswith(b'Source document revision:')]
    require(len(comments) == 1, 'DXF must contain one source-revision comment')
    index = comments[0]
    ending = b'\r\n' if lines[index].endswith(b'\r\n') else b'\n'
    require(lines[index] == b'Source document revision: ' + str(revision).encode() + ending,
            'wrong DXF source revision/comment')
    lines[index] = b'Source document revision: <validated-own-revision>' + ending
    return b''.join(lines)


def grade_exports(files):
    for extension in ('svg', 'obj'):
        require(files['C-before.' + extension] == files['C-after.' + extension], extension + ' changed after annotation')
    require(normalized_dxf(files['C-before.dxf'], 2) == normalized_dxf(files['C-after.dxf'], 3),
            'nonrevision DXF content changed after annotation')
    svg = files['C-after.svg']
    require(b'<!DOCTYPE' not in svg and b'<!ENTITY' not in svg, 'unsupported SVG declaration')
    tree = ET.fromstring(svg)
    tag = lambda node: node.tag.rsplit('}', 1)[-1]
    require(tag(tree) == 'svg' and all(tag(x) in ('title', 'desc', 'path') for x in tree),
            'unexpected SVG primitive/nesting')
    require(tree.get('width') == '1000mm' and tree.get('height') == '1000mm', 'SVG millimetre dimensions differ')
    near([float(x) for x in tree.get('viewBox', '').split()], [0., -1000., 1000., 1000.], 1e-9, 'SVG viewBox differs')
    paths = [x for x in tree if tag(x) == 'path']
    require(len(paths) == 1 and paths[0].get('id') == 'piece-2', 'SVG contains missing/extra authored paths')
    pattern = r'[A-Za-z]|[-+]?(?:\d*\.\d+|\d+\.?\d*)(?:[eE][-+]?\d+)?'
    data = paths[0].get('d', '')
    require(not re.sub(pattern, '', data).strip(' \t\r\n,'), 'unsupported SVG path text')
    tokens = re.findall(pattern, data)
    commands = [x for x in tokens if x.isalpha()]
    require(commands == ['M', 'C', 'L', 'L', 'L', 'Z'], 'SVG is not exactly the closed C exterior')
    coordinates = [float(x) for x in tokens if not x.isalpha()]
    near(coordinates, [0., 0., 1000./3, -2000./3, 2000./3, -2000./3,
                       1000., 0., 1000., -1000., 0., -1000., 0., 0.], 1e-9,
         'SVG exterior coordinates/units differ')
    dxf_lines = files['C-after.dxf'].decode().splitlines()
    pairs = list(zip(dxf_lines[::2], dxf_lines[1::2]))
    start = next(i for i in range(len(pairs) - 1) if pairs[i] == ('0', 'SECTION') and pairs[i + 1] == ('2', 'ENTITIES')) + 2
    end = next(i for i in range(start, len(pairs)) if pairs[i] == ('0', 'ENDSEC'))
    entities = []
    for code, value in pairs[start:end]:
        if code == '0':
            entities.append({'kind': value})
        else:
            require(entities, 'DXF field before first entity')
            entities[-1].setdefault(code, []).append(value)
    require([e['kind'] for e in entities] == ['SPLINE', 'LINE', 'LINE', 'LINE'], 'DXF missing/extra/nonexterior entities')
    require(any(pairs[i] == ('9', '$INSUNITS') and pairs[i+1] == ('70', '4') for i in range(len(pairs)-1)), 'DXF units are not millimetres')
    for entity in entities:
        require(entity.get('8') == ['WF_PIECE_2'], 'DXF exterior layer differs')
    first = entities[0]
    near([float(x) for x in first.get('10', [])], [0., 1000./3, 2000./3, 1000.], 1e-9, 'DXF cubic U differs')
    near([float(x) for x in first.get('20', [])], [0., 2000./3, 2000./3, 0.], 1e-9, 'DXF cubic V differs')
    near([float(x) for x in first.get('30', [])], [0.]*4, 1e-9, 'DXF cubic Z differs')
    for entity, a, b in zip(entities[1:], [(1000.,0.), (1000.,1000.), (0.,1000.)], [(1000.,1000.), (0.,1000.), (0.,0.)]):
        near([float(entity[code][0]) for code in ('10','20','30','11','21','31')], [*a,0.,*b,0.], 1e-9, 'DXF line coordinates differ')
    obj = files['C-after.obj'].decode().splitlines()
    kinds = [line.split()[0] for line in obj if line.strip() and not line.startswith('#')]
    require(set(kinds) <= {'v', 'vt', 'vn', 'g', 'f'} and 'v' in kinds and 'f' in kinds,
            'OBJ has annotation/non-surface primitives or no cloth surface')
    vertices = [[float(v) for v in line.split()[1:]] for line in obj if line.startswith('v ')]
    require(all(len(v) == 3 and all(math.isfinite(x) for x in v) and
                -.000000000001 <= v[0] <= 1.000000000001 and
                -.000000000001 <= v[1] <= 1.000000000001 and abs(v[2]) <= 1e-12 for v in vertices),
            'OBJ is outside the original metre-scale flat C rest bounds')
    return {'annotation_omission': 'actual SVG/OBJ bytes equal; DXF equal except validated own revision',
            'surface_limit': 'OBJ comparison proves annotation invariance, not an independent meshing oracle'}


def grade_run(case, run, freeze, synthetic=False):
    models = parsed(read(ROOT / 'EXPECTATIONS.json'))
    plan = parsed(read(ROOT / 'PLAN.json'))
    bound = parsed(read(run / 'ROOT-BINDING.json'))
    require(bound.get('root_authorized_for_case') is True and bound.get('case') == case and
            bound.get('freeze_sha256') == sha(freeze), 'missing/wrong final root binding')
    require(re.fullmatch(r'[0-9a-f]{40}', bound.get('source_commit', '')) is not None and
            re.fullmatch(r'[0-9a-f]{64}', bound.get('native_binary_sha256', '')) is not None and
            Path(bound.get('own_target_directory', '')).is_absolute(), 'incomplete final source/binary/target binding')
    expected_inputs = plan['input_hashes'][case]
    require(bound.get('inputs', {}).keys() == expected_inputs.keys(), 'staged input inventory differs')
    for filename, expected_digest in expected_inputs.items():
        path = run / filename
        digest = sha(read(path))
        require(path.stat().st_mode & 0o222 == 0 and digest == bound['inputs'][filename], 'staged input changed/not read-only')
        if expected_digest is not None:
            require(digest == expected_digest, 'input differs from frozen original')
        else:
            dep = bound.get('accepted_A_dependency', {})
            require(dep.get('snapshot_sha256') == digest and
                    all(re.fullmatch(r'[0-9a-f]{64}', dep.get(key, '')) is not None
                        for key in ('saved_grade_report_sha256','root_native_ui_review_sha256')), 'unreviewed A dependency')
    records = sorted((run / 'trace').glob('*-before.json'))
    require(0 < len(records) <= plan['cases'][case]['call_cap'], 'case operation cap exceeded or no trace')
    launch = parsed(read(run / 'LAUNCH-CLOCK.json'))
    latest = int(launch['monotonic_ns'])
    for path in records:
        before = parsed(read(path))
        after = parsed(read(path.with_name(path.name.replace('-before.json','-after.json'))))
        require(before['id'] == after['id'] and after['status'] == 'returned', 'unpaired/failed case operation')
        latest = max(latest, int(after['monotonic_ns']))
    require(0 <= (latest - int(launch['monotonic_ns'])) / 1e9 <= plan['cases'][case]['elapsed_cap_s'], 'case elapsed cap exceeded')
    closed = parsed(read(run / 'NATIVE-LEASE-CLOSED.json'))
    require(closed.get('case') == case and closed.get('native_input_lease_closed') is True,
            'native case is not closed before grading')
    documents, raw_states, receipts = {}, {}, {}
    previous_capture = 0
    for label, expected in models['states'][case].items():
        path = run / 'snapshots' / (label + '.weftforge')
        raw = read(path)
        require(path.stat().st_mode & 0o222 == 0, 'snapshot is not immutable/read-only: ' + label)
        receipt = parsed(read(run / 'snapshots' / (label + '.receipt.json')))
        require(receipt.get('evidence_kind') == ('synthetic_selfcheck' if synthetic else 'native_saved_bytes'), 'wrong receipt evidence kind')
        require(int(receipt['native_save_record']) > previous_capture and
                int(receipt['clean_capture_record']) > int(receipt['native_save_record']), 'save/capture origin reused or unordered')
        previous_capture = int(receipt['clean_capture_record'])
        require(receipt['label'] == label and receipt['case'] == case and
                receipt['revision'] == expected['project']['revision'] and
                receipt['primary_sha256'] == receipt['snapshot_sha256'] == sha(raw),
                'saved snapshot provenance/hash mismatch: ' + label)
        for key in ('native_save_record', 'clean_capture_record'):
            record = receipt[key]
            require(re.fullmatch(r'\d{4,}', record) is not None, 'bad source record: ' + label)
            before = parsed(read(run / 'trace' / (record + '-before.json')))
            after = parsed(read(run / 'trace' / (record + '-after.json')))
            require(before['id'] == after['id'] == record and after['status'] == 'returned',
                    'unpaired/failed native save or capture: ' + label)
        documents[label], raw_states[label], receipts[label] = parsed(raw), raw, receipt
    result = grade_states(case, documents, models)
    # Recovery copies are bytes of the previous valid primary, never regenerated JSON.
    for label, receipt in receipts.items():
        previous = models['previous_saved'][case][label]
        require(receipt.get('previous_saved_label') == previous, 'incorrect recovery predecessor: ' + label)
        require(receipt['primary_relative_path'] == models['primary_paths'][case][label], 'wrong mutable primary: ' + label)
        if previous is not None:
            require(previous in raw_states, 'unknown previous primary snapshot')
            recovery = read(run / 'snapshots' / (label + '.recovery'))
            require(recovery == raw_states[previous] and sha(recovery) == receipt['recovery_sha256'], 'recovery is not exact previous-primary bytes: ' + label)
        else:
            require(receipt.get('recovery_sha256') is None, 'fresh primary unexpectedly had old recovery: ' + label)
    for primary in set(models['primary_paths'][case].values()):
        labels = [label for label in documents if models['primary_paths'][case][label] == primary]
        require(read(run / primary) == raw_states[labels[-1]], 'final primary differs from last immutable snapshot')
    if case == 'D':
        for name in ('invalid-save.weftforge', 'invalid-save.weftforge.recovery'):
            require(read(run / 'work' / name) == read(ROOT / 'fixtures' / name), 'failed Save mutated ' + name)
    if case == 'E':
        result['exports'] = grade_exports({name: read(run / 'exports' / name) for name in models['exports']['names']})
    result['snapshot_sha256'] = {name: sha(raw) for name, raw in raw_states.items()}
    return {'status': 'passed_saved_files_only', 'native_ui_verified': False,
            'reference_verified': False, 'case': case, 'freeze_sha256': sha(freeze),
            'root_binding_sha256': sha(read(run / 'ROOT-BINDING.json')),
            'native_case_outcome': closed.get('outcome', 'unspecified'), 'result': result}


def selfcheck():
    models = parsed(read(ROOT / 'EXPECTATIONS.json'))
    docs = copy.deepcopy(models['states'])
    # Invented exact quarter-turn representation only; never candidate output.
    for label in models['transform']['affected_states']:
        piece = item(docs['B'][label], 'pieces', 2)
        piece['placement']['rotation'] = [0., 0., -math.pi / 2.]
        piece['grain_angle'] = .47 + math.pi / 2.
    for case in docs:
        grade_states(case, docs[case], models)
    rejected = []
    def bad(name, case, label, mutate):
        altered = copy.deepcopy(docs[case])
        mutate(altered[label])
        try:
            grade_states(case, altered, models)
        except (AssertionError, ValueError, KeyError, TypeError, OverflowError):
            rejected.append(name)
        else:
            raise AssertionError('selfcheck accepted bad output: ' + name)
    bad('created missing second endpoint', 'A', 'A02-created', lambda d: d['project']['points'].pop())
    bad('both endpoint edit became two revisions', 'A', 'A03-edited', lambda d: d['project'].__setitem__('revision', 4))
    bad('pending grain lost', 'A', 'A03-edited', lambda d: item(d, 'pieces', 2).__setitem__('grain_angle', 0.))
    bad('undo failed to restore both endpoints', 'A', 'A04-undo', lambda d: item(d, 'points', 12)['position'].__setitem__(1, .625))
    bad('empty project lost mannequin', 'A', 'A00-empty', lambda d: d['project']['mannequin']['capsules'].clear())
    bad('v3 relabel', 'A', 'A05-redo', lambda d: d.__setitem__('version', 3))
    bad('missing required v4 ownership', 'A', 'A05-redo', lambda d: item(d, 'pieces', 2).pop('internal_curves'))
    bad('unexpected v4 field', 'A', 'A05-redo', lambda d: item(d, 'pieces', 2).__setitem__('cut', True))
    bad('internal controls accepted', 'A', 'A05-redo', lambda d: item(d, 'curves', 13).__setitem__('controls', [[.3, .4], [.6, .6]]))
    bad('allocator reuse', 'B', 'B06-reedited', lambda d: d['project'].__setitem__('next_id', 13))
    bad('reopen edit lost one f64 bit', 'B', 'B06-reedited', lambda d: item(d, 'points', 12)['position'].__setitem__(1, math.nextafter(.6875, 1.)))
    bad('resize omitted internal end', 'B', 'B10-resized', lambda d: item(d, 'points', 12).__setitem__('position', [.75, .6875]))
    bad('transform omitted internal end', 'B', 'B11-turned', lambda d: item(d, 'points', 12).__setitem__('position', [1.5, .34375]))
    bad('transform lost placement compensation', 'B', 'B11-turned', lambda d: item(d, 'pieces', 2)['placement'].__setitem__('rotation', [0., 0., 0.]))
    bad('transform changed material', 'B', 'B11-turned', lambda d: d['project']['materials'][0].__setitem__('damping', .3))
    bad('line deletion deleted owner', 'B', 'B12-line-removed', lambda d: d['project']['pieces'].clear())
    bad('owner deletion left endpoint', 'B', 'B14-owner-removed', lambda d: d['project']['points'].append({'id': 12, 'position': [0., 0.]}))
    bad('owner undo changed relationship', 'B', 'B15-owner-restored', lambda d: item(d, 'curves', 13).__setitem__('end', 11))
    bad('coincident drag moved first identity', 'C', 'C07-selected-identity', lambda d: item(d, 'points', 22).__setitem__('position', [.25, .375]))
    bad('staged boundary control lost', 'C', 'C08-control-and-drag', lambda d: item(d, 'curves', 7)['controls'][1].__setitem__(0, .125))
    bad('staged boundary drag split transaction', 'C', 'C08-control-and-drag', lambda d: d['project'].__setitem__('revision', 9))
    bad('legacy correspondence remapped', 'C', 'C10-redo', lambda d: d['project']['seams'][0]['correspondence'][1].__setitem__(1, .25))
    bad('ordered group changed', 'C', 'C10-redo', lambda d: d['project']['sewing_groups'][0]['b'][0].__setitem__('t0', .125))
    bad('failed open lost original endpoint', 'D', 'D03-after-io', lambda d: item(d, 'points', 11).__setitem__('position', [.25, .25]))
    bad('failed candidate consumed ID', 'D', 'D03-after-io', lambda d: d['project'].__setitem__('next_id', 20))
    bad('new retained authored geometry', 'D', 'D00-new-empty', lambda d: d['project']['points'].append({'id': 3, 'position': [0., 0.]}))
    bad('recover incremented revision', 'D', 'D05-recovered', lambda d: d['project'].__setitem__('revision', 6))
    bad('L rejection consumed candidate IDs', 'E', 'E02-L-accepted', lambda d: d['project'].__setitem__('next_id', 21))
    bad('C accepted outside-middle line', 'E', 'E03-C-accepted', lambda d: item(d, 'points', 11)['position'].__setitem__(1, .2))
    bad('lost signed zero in default geometry', 'A', 'A01-square', lambda d: item(d, 'points', 3)['position'].__setitem__(0, -0.))
    for name, raw in [('duplicate field', b'{"a":1,"a":2}'), ('NaN', b'{"x":NaN}')]:
        try:
            parsed(raw)
        except (AssertionError, ValueError):
            rejected.append(name)
        else:
            raise AssertionError('selfcheck accepted ' + name)
    dx = b'0\nSECTION\n999\nSource document revision: 2\n0\nENDSEC\n'
    require(normalized_dxf(dx, 2) == normalized_dxf(dx.replace(b'revision: 2', b'revision: 3'), 3),
            'DXF own-revision normalization selfcheck')
    for name, raw in [('wrong DXF revision', dx), ('duplicate DXF revision', dx + dx)]:
        try:
            normalized_dxf(raw, 3)
        except AssertionError:
            rejected.append(name)
        else:
            raise AssertionError('selfcheck accepted ' + name)
    svg = b'<svg xmlns="http://www.w3.org/2000/svg" width="1000mm" height="1000mm" viewBox="0 -1000 1000 1000"><path id="piece-2" d="M 0 0 C 333.3333333333333 -666.6666666666666 666.6666666666666 -666.6666666666666 1000 0 L 1000 -1000 L 0 -1000 L 0 0 Z"/></svg>'
    obj = b'# synthetic original surface, not product output\nv 0 0.75 0\nv 1 0.75 0\nv 0.5 1 0\nf 1 2 3\n'
    dpairs = [('0','SECTION'),('2','HEADER'),('999','Source document revision: 2'),('9','$INSUNITS'),('70','4'),('0','ENDSEC'),('0','SECTION'),('2','ENTITIES'),('0','SPLINE'),('8','WF_PIECE_2')]
    for x,y in [(0.,0.),(1000./3,2000./3),(2000./3,2000./3),(1000.,0.)]:
        dpairs += [('10',str(x)),('20',str(y)),('30','0')]
    for a,b in zip([(1000.,0.),(1000.,1000.),(0.,1000.)],[(1000.,1000.),(0.,1000.),(0.,0.)]):
        dpairs += [('0','LINE'),('8','WF_PIECE_2')] + list(zip(('10','20','30','11','21','31'),map(str,[*a,0.,*b,0.])))
    dpairs += [('0','ENDSEC'),('0','EOF')]
    dxf = ''.join(k+'\n'+v+'\n' for k,v in dpairs).encode()
    exports = {'C-before.svg': svg, 'C-after.svg': svg, 'C-before.obj': obj, 'C-after.obj': obj,
               'C-before.dxf': dxf, 'C-after.dxf': dxf.replace(b'revision: 2',b'revision: 3')}
    grade_exports(exports)
    def bad_exports(name, names, mutate):
        altered = copy.deepcopy(exports)
        for filename in names:
            altered[filename] = mutate(altered[filename])
        try:
            grade_exports(altered)
        except (AssertionError, ValueError, KeyError, TypeError):
            rejected.append(name)
        else:
            raise AssertionError('selfcheck accepted ' + name)
    bad_exports('SVG includes internal open path', ['C-before.svg','C-after.svg'], lambda raw: raw.replace(b'</svg>', b'<path d="M 100 750 L 900 750"/></svg>'))
    bad_exports('SVG incorrect units both outputs', ['C-before.svg','C-after.svg'], lambda raw: raw.replace(b'1000mm',b'1mm'))
    bad_exports('DXF annotation entity added', ['C-after.dxf'], lambda raw: raw.replace(b'0\nENDSEC\n0\nEOF', b'0\nLINE\n0\nENDSEC\n0\nEOF'))
    bad_exports('DXF wrong exterior both outputs', ['C-before.dxf','C-after.dxf'], lambda raw: raw.replace(b'10\n333.3333333333333', b'10\n333.0'))
    bad_exports('DXF wrong own revision', ['C-after.dxf'], lambda raw: raw.replace(b'revision: 3',b'revision: 2'))
    bad_exports('OBJ annotation line both outputs', ['C-before.obj','C-after.obj'], lambda raw: raw+b'l 1 2\n')
    bad_exports('OBJ changed surface after annotation', ['C-after.obj'], lambda raw: raw.replace(b'v 0 0.75 0',b'v 0 0.8 0'))
    bad_exports('empty OBJ pair', ['C-before.obj','C-after.obj'], lambda raw: b'# empty\n')
    return {'status': 'passed_synthetic_assertions_only', 'candidate_executed': False,
            'native_executed': False, 'accepted_original_model_states': sum(map(len, docs.values())),
            'rejected_bad_outputs': len(rejected), 'checks': rejected}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['selfcheck', 'grade'])
    parser.add_argument('--case', choices=list('ABCDE'))
    parser.add_argument('--run', type=Path)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    require(not args.report.exists(), 'refuse to overwrite a prior report')
    try:
        if args.mode == 'selfcheck':
            result = selfcheck()
        else:
            require(args.case is not None and args.run is not None, 'grade needs --case and --run')
            frozen = read(ROOT / 'FREEZE.json')
            manifest = parsed(frozen)
            for name, digest in manifest['sha256'].items():
                require(sha(read(ROOT / name)) == digest, 'prospective packet changed: ' + name)
            result = grade_run(args.case, args.run.resolve(), frozen)
    except Exception as problem:
        result = {'status': 'failed_saved_files_or_preflight', 'native_ui_verified': False,
                  'error_type': type(problem).__name__, 'error': str(problem)}
        with args.report.open('x') as output:
            json.dump(result, output, indent=2, allow_nan=False)
            output.write('\n')
        print(json.dumps(result))
        return 1
    with args.report.open('x') as output:
        json.dump(result, output, indent=2, allow_nan=False)
        output.write('\n')
    print(json.dumps({k: v for k, v in result.items() if k != 'checks'}))
    return 0


if __name__ == '__main__':
    sys.exit(main())
