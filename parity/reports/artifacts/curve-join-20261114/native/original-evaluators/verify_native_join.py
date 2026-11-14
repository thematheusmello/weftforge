#!/usr/bin/env python3
"""Independent bounded-join saved-output evaluator; no implementation imports."""
import argparse
import copy
from decimal import Decimal as D, ROUND_CEILING, localcontext
from fractions import Fraction as F
import hashlib
import json
from pathlib import Path
import re
import sys
import tempfile
import xml.etree.ElementTree as ET

import verify_curve_outputs as original

TOL = F(1, 10**12)
SPLIT = F(2, 5)
KINDS = ('points', 'curves', 'pieces', 'materials', 'seams')
ORIGINAL_SHA256 = '90148a51d1bc7f6357908f804cf7308fd4780d6219694271dd103dfc7f821350'
HISTORICAL_SHA256 = '1deffbc485b9c1d4b2206e3c5cbe94d0181e2afc75ebb96cd2114cae1696b4ab'
PROJECT_FILES = ('before-split', 'after-split', 'after-focus', 'after-join',
                 'after-join-tab', 'join-undone', 'join-redone', 'coarse',
                 'fine', 'reopened', 'after-corner-error')
INPUT_FILES = tuple(f'{name}.weftforge' for name in PROJECT_FILES) + (
    'coarse.obj', 'coarse.svg', 'fine.obj', 'fine.svg', 'ui-declaration.json')
require = original.require
index = original.index
digest = original.digest


def fraction(value):
    require(type(value) in (int, D, F), 'expected finite numeric construction value')
    if isinstance(value, D):
        require(value.is_finite(), 'non-finite construction value')
    return F(value)


def vector(values):
    require(len(values) == 2, 'expected two pattern coordinates')
    return tuple(fraction(v) for v in values)


def add(a, b):
    return tuple(x + y for x, y in zip(a, b))


def scale(a, t):
    return tuple(x * t for x in a)


def control_points(project, curve_id):
    curve, points = index(project, 'curves')[curve_id], index(project, 'points')
    a, b = vector(points[curve['start']]['position']), vector(points[curve['end']]['position'])
    if curve['controls'] is None:
        inner = (add(scale(a, F(2, 3)), scale(b, F(1, 3))),
                 add(scale(a, F(1, 3)), scale(b, F(2, 3))))
    else:
        require(len(curve['controls']) == 2, 'expected two inner cubic controls')
        inner = tuple(vector(v) for v in curve['controls'])
    return (a, *inner, b)


def restrict(c, a, b):
    """Exact power-basis restriction, independent of native de Casteljau code."""
    h = b - a
    p0 = c[0]
    p1 = scale(add(c[1], scale(c[0], -1)), 3)
    p2 = scale(add(add(c[2], scale(c[1], -2)), c[0]), 3)
    p3 = add(add(c[3], scale(c[2], -3)), add(scale(c[1], 3), scale(c[0], -1)))
    q0 = add(add(p0, scale(p1, a)), add(scale(p2, a*a), scale(p3, a*a*a)))
    q1 = scale(add(add(p1, scale(p2, 2*a)), scale(p3, 3*a*a)), h)
    q2 = scale(add(p2, scale(p3, 3*a)), h*h)
    q3 = scale(p3, h*h*h)
    return (q0, add(q0, scale(q1, F(1, 3))),
            add(add(q0, scale(q1, F(2, 3))), scale(q2, F(1, 3))),
            add(add(q0, q1), add(q2, q3)))


def squared_bound(a, b):
    require(len(a) == len(b) == 4, 'expected cubic Bernstein control polygons')
    return max(sum((x-y)**2 for x, y in zip(p, q)) for p, q in zip(a, b))


def upper_decimal(value, square_root=False):
    if value == 0:
        return D(0)
    with localcontext() as ctx:
        ctx.prec = 70
        ctx.rounding = ROUND_CEILING
        number = D(value.numerator) / D(value.denominator)
        # sqrt is correctly rounded; one upward ulp makes the report an enclosure.
        return number.sqrt().next_plus() if square_root else number


def bound_record(squared):
    return {'all_parameters_upper_bound_m': upper_decimal(squared, True),
            'maximum_squared_control_difference_m2_exact': str(squared),
            'proof': 'Exact rational power restriction and Bernstein convex hull.',
            'within_1e_12_m': squared <= TOL*TOL}


def parameter_record(value):
    return {'upper_bound': upper_decimal(value), 'maximum_absolute_error_exact': str(value),
            'within_1e_12': value <= TOL}


def check_bound(squared, label):
    require(squared <= TOL*TOL, f'{label}: {upper_decimal(squared, True)} m; squared exact {squared}')


def check_tree(value):
    if isinstance(value, dict):
        for child in value.values():
            check_tree(child)
    elif isinstance(value, list):
        for child in value:
            check_tree(child)
    elif isinstance(value, D):
        require(value.is_finite(), 'non-finite JSON construction field')


def check_ids(project):
    check_tree(project)
    ids = [item['id'] for kind in KINDS for item in project[kind]]
    require(all(type(i) is int and i > 0 for i in ids), 'invalid stable ID')
    require(len(ids) == len(set(ids)), 'duplicate global stable ID')
    require(type(project['next_id']) is int and project['next_id'] > max(ids), 'invalid monotone allocator')
    require(type(project['revision']) is int and project['revision'] >= 0, 'invalid document revision')
    for curve in project['curves']:
        control_points(project, curve['id'])
    for piece in project['pieces']:
        require(len(piece['boundary']) >= 3, 'piece has fewer than three boundaries')
        curves = index(project, 'curves')
        chain = [curves[curve] for curve in piece['boundary']]
        require(all(a['end'] == b['start'] for a, b in zip(chain, chain[1:]+chain[:1])), 'open piece chain')
    for seam in project['seams']:
        pairs = seam['correspondence']
        require(len(pairs) >= 2 and pairs[0] == [0, 0] and pairs[-1] == [1, 1], 'normalized correspondence endpoints')
        require(all(0 <= u0 < u1 <= 1 and 0 <= v0 < v1 <= 1
                    for (u0, v0), (u1, v1) in zip(pairs, pairs[1:])), 'nonmonotone seam correspondence')
        require(seam['a']['curve'] != seam['b']['curve'], 'self seam')
        for side in ('a', 'b'):
            interval = seam[side]
            require(interval['curve'] in index(project, 'curves'), 'seam references retired curve')
            a, b = fraction(interval['t0']), fraction(interval['t1'])
            require(0 <= a <= 1 and 0 <= b <= 1 and abs(a-b) > F(1, 10**8), 'invalid seam interval')


def baseline_check(project):
    original.baseline_check(project)
    check_ids(project)
    expected_ids = {'points': [3, 4, 5, 6, 12, 13, 14, 15],
                    'curves': [7, 8, 9, 10, 16, 17, 18, 19],
                    'pieces': [2, 11], 'materials': [1], 'seams': [20]}
    require({kind: sorted(index(project, kind)) for kind in KINDS} == expected_ids, 'unexpected baseline IDs')
    require(set(project) == {'revision', *KINDS, 'mannequin', 'resolution', 'next_id'}, 'unexpected v2 project fields')
    for piece in project['pieces']:
        require(piece['placement'] == {'translation': [0, 0, 0], 'rotation': [0, 0, 0]}, 'non-default placement outside native case')
        require(piece['grain_angle'] == 0 and piece['material'] == 1, 'non-default piece fields')
    for curve_id, start, end in ((7, 3, 4), (8, 4, 5), (10, 6, 3),
                                 (16, 12, 13), (17, 13, 14), (18, 14, 15), (19, 15, 12)):
        require(index(project, 'curves')[curve_id] == {'id': curve_id, 'start': start, 'end': end, 'controls': None}, 'baseline straight edge changed')
    return {'sha256_source': 'actual before-split snapshot', **original.project_summary(project),
            'authored_control_bound': bound_record(squared_bound(control_points(project, 9), tuple(vector(p) for p in original.EXPECTED))),
            'pattern_world_equivalence': 'Both placements are zero; pattern/world norms agree.'}


def split_check(base, split):
    check_ids(split)
    legacy = original.split_check(base, split)
    point_id, child_id = legacy['join_point_id'], legacy['second_curve_id']
    new_seams = set(index(split, 'seams')) - set(index(base, 'seams'))
    require(len(new_seams) == 1, 'split must allocate exactly one seam record')
    seam_id = next(iter(new_seams))
    expected = copy.deepcopy(base)
    expected['revision'] += 1
    expected['next_id'] = split['next_id']
    require(split['next_id'] >= base['next_id'], 'split allocator regressed')
    expected['points'].append(copy.deepcopy(index(split, 'points')[point_id]))
    expected['curves'].append(copy.deepcopy(index(split, 'curves')[child_id]))
    retained = index(expected, 'curves')[9]
    retained.update(end=point_id, controls=copy.deepcopy(index(split, 'curves')[9]['controls']))
    require(index(split, 'curves')[child_id].keys() == retained.keys(), 'new child acquired unrelated fields')
    require(index(split, 'points')[point_id].keys() == index(base, 'points')[5].keys(), 'new point acquired unrelated fields')
    index(expected, 'pieces')[2]['boundary'] = [7, 8, 9, child_id, 10]
    first = copy.deepcopy(base['seams'][0])
    first['b']['t1'] = D('.6')
    second = copy.deepcopy(first)
    second.update(id=seam_id)
    second['a']['curve'] = child_id
    second['b'].update(t0=D('.6'), t1=D(0))
    expected['seams'] = [first, second]
    parameter_error = F(0)
    for actual, wanted in zip(split['seams'], expected['seams']):
        for side in ('a', 'b'):
            for endpoint in ('t0', 't1'):
                parameter_error = max(parameter_error, abs(fraction(actual[side][endpoint])-fraction(wanted[side][endpoint])))
                wanted[side][endpoint] = actual[side][endpoint]
    require(split == expected, 'split changed unintended fields/order/name/correspondence or revision')
    require(parameter_error <= TOL, 'split affine interval error exceeds 1e-12')
    parent = control_points(base, 9)
    geometry = max(squared_bound(control_points(split, 9), restrict(parent, F(0), SPLIT)),
                   squared_bound(control_points(split, child_id), restrict(parent, SPLIT, F(1))))
    check_bound(geometry, 'split full authored-control bound')
    return {'point_id': point_id, 'child_id': child_id, 'new_seam_id': seam_id,
            'geometry': bound_record(geometry), 'interval_parameters': parameter_record(parameter_error),
            'legacy_decimal_70_check': legacy}


def seam_locus_bound(before, after):
    squared, count = F(0), 0
    require([s['id'] for s in before['seams']] == [s['id'] for s in after['seams']], 'seam record identity/order changed')
    for old, new in zip(before['seams'], after['seams']):
        require(old['name'] == new['name'] and old['correspondence'] == new['correspondence'], 'seam name/correspondence changed')
        for p0, p1 in zip(old['correspondence'], old['correspondence'][1:]):
            for side, coordinate in (('a', 0), ('b', 1)):
                a, b = old[side], new[side]
                old_t = [fraction(a['t0'])+(fraction(a['t1'])-fraction(a['t0']))*fraction(p[coordinate]) for p in (p0, p1)]
                new_t = [fraction(b['t0'])+(fraction(b['t1'])-fraction(b['t0']))*fraction(p[coordinate]) for p in (p0, p1)]
                squared = max(squared, squared_bound(restrict(control_points(before, a['curve']), *old_t),
                                                       restrict(control_points(after, b['curve']), *new_t)))
            count += 1
    check_bound(squared, 'full sewn point-pair locus bound')
    return {'both_sides_bound': bound_record(squared), 'piecewise_affine_correspondence_intervals_proved': count}


def join_check(base, split, joined, discovered):
    check_ids(joined)
    child_id, point_id = discovered['child_id'], discovered['point_id']
    require(point_id not in index(joined, 'points') and child_id not in index(joined, 'curves'), 'joined child/shared-point IDs did not retire')
    require(joined['next_id'] == split['next_id'], 'join allocated/reused IDs')
    parent_bound = squared_bound(control_points(joined, 9), control_points(base, 9))
    child_bound = max(squared_bound(restrict(control_points(joined, 9), F(0), SPLIT), control_points(split, 9)),
                      squared_bound(restrict(control_points(joined, 9), SPLIT, F(1)), control_points(split, child_id)))
    check_bound(parent_bound, 'joined original parent control bound')
    check_bound(child_bound, 'joined reconstructed child control bound')
    expected = copy.deepcopy(split)
    expected['revision'] += 1
    expected['points'] = [point for point in expected['points'] if point['id'] != point_id]
    expected['curves'] = [curve for curve in expected['curves'] if curve['id'] != child_id]
    index(expected, 'curves')[9].update(end=index(base, 'curves')[9]['end'],
                                      controls=copy.deepcopy(index(joined, 'curves')[9]['controls']))
    index(expected, 'pieces')[2]['boundary'] = [7, 8, 9, 10]
    parameter_error = F(0)
    for actual, wanted in zip(joined['seams'], expected['seams']):
        for side in ('a', 'b'):
            interval = wanted[side]
            old_curve = interval['curve']
            if old_curve in (9, child_id):
                interval['curve'] = 9
                for endpoint in ('t0', 't1'):
                    value = fraction(interval[endpoint])
                    mapped = SPLIT*value if old_curve == 9 else SPLIT+(1-SPLIT)*value
                    parameter_error = max(parameter_error, abs(fraction(actual[side][endpoint])-mapped))
                    interval[endpoint] = actual[side][endpoint]
    require(joined == expected, 'join changed unintended fields/order/name/correspondence or did not commit once')
    require(parameter_error <= TOL, f'join affine interval error exceeds 1e-12: {parameter_error}')
    return {'retained_curve_id': 9, 'retired_curve_id': child_id, 'retired_shared_point_id': point_id,
            'original_parent_geometry': bound_record(parent_bound), 'child_geometry': bound_record(child_bound),
            'normalized_affine_interval_error': parameter_record(parameter_error),
            'sewn_point_pairs': seam_locus_bound(split, joined), 'single_revision': True,
            'unintended_field_changes': [], 'allocator_unchanged': True}


def construction_equal(a, b, label, ignored=('revision', 'next_id')):
    require(set(a) == set(b), f'{label}: project field set changed')
    require({k: v for k, v in a.items() if k not in ignored} ==
            {k: v for k, v in b.items() if k not in ignored}, f'{label}: authored construction changed')
    require(b['next_id'] >= a['next_id'], f'{label}: allocator regressed')
    return {'construction_identical': True, 'ignored_fields': list(ignored),
            'before_revision': a['revision'], 'after_revision': b['revision'],
            'before_next_id': a['next_id'], 'after_next_id': b['next_id']}


def same_bytes(a, b):
    require(a.read_bytes() == b.read_bytes(), f'no-change snapshot bytes differ: {a.name}, {b.name}')
    return {'before': a.name, 'after': b.name, 'sha256': digest(a), 'document_bytes_identical': True}


def svg_check(path, project):
    root = ET.parse(path).getroot()
    require(root.tag == '{http://www.w3.org/2000/svg}svg', 'SVG root namespace')
    require(all('transform' not in node.attrib for node in root.iter()), 'unexpected SVG transform')
    require(root.attrib['width'].endswith('mm') and root.attrib['height'].endswith('mm'), 'SVG physical units')
    view = tuple(D(v) for v in root.attrib['viewBox'].split())
    require(len(view) == 4 and all(v.is_finite() for v in view) and view[2] > 0 and view[3] > 0, 'SVG viewBox')
    require(D(root.attrib['width'][:-2]) == view[2] and D(root.attrib['height'][:-2]) == view[3], 'one SVG unit must be one millimetre')
    nodes = root.findall('{http://www.w3.org/2000/svg}path')
    paths = {node.attrib['id']: node for node in nodes}
    require(len(paths) == len(nodes) == 2 and set(paths) == {'piece-2', 'piece-11'}, 'SVG editable piece paths')
    maximum, parent, cubic_count = F(0), F(0), 0
    for piece in project['pieces']:
        node = paths[f"piece-{piece['id']}"]
        require(node.attrib['data-name'] == piece['name'], 'SVG piece name')
        commands = original.svg_path_commands(node.attrib['d'])
        require(len(commands) == len(piece['boundary'])+2 and commands[0][0] == 'M' and commands[-1] == ('Z', ()), 'SVG closed editable boundary command count')
        current = (fraction(commands[0][1][0])/1000, -fraction(commands[0][1][1])/1000)
        start = control_points(project, piece['boundary'][0])[0]
        maximum = max(maximum, sum((a-b)**2 for a, b in zip(current, start)))
        for curve_id, (command, values) in zip(piece['boundary'], commands[1:-1]):
            authored = control_points(project, curve_id)
            cubic = index(project, 'curves')[curve_id]['controls'] is not None
            require(command == ('C' if cubic else 'L'), 'SVG flattened or stale curve representation')
            points = tuple((fraction(values[i])/1000, -fraction(values[i+1])/1000) for i in range(0, len(values), 2))
            if cubic:
                actual = (current, *points)
                maximum = max(maximum, squared_bound(actual, authored))
                require(curve_id == 9, 'unexpected editable cubic in joined SVG')
                parent = max(parent, squared_bound(actual, tuple(vector(p) for p in original.EXPECTED)))
                cubic_count += 1
            else:
                maximum = max(maximum, sum((a-b)**2 for a, b in zip(points[-1], authored[-1])))
            current = points[-1]
        require(current == start, 'SVG endpoint closure changed')
    require(cubic_count == 1, 'joined SVG must retain exactly one editable cubic')
    check_bound(maximum, 'SVG stale/authored geometry bound')
    check_bound(parent, 'SVG original parent bound')
    return {'sha256': digest(path), 'editable_closed_piece_paths': len(paths),
            'editable_cubic_segments': cubic_count, 'physical_user_unit': 'millimetre',
            'saved_authoring_error': bound_record(maximum), 'original_parent_error': bound_record(parent),
            'parser': 'Unchanged original evaluator svg_path_commands.'}


def synthesize():
    # Synthetic fixtures test the oracle; they are never substituted for native outputs.
    base = original.read_project(Path(__file__).with_name('historical-before-split.weftforge'))
    base = copy.deepcopy(base)
    left, right = restrict(control_points(base, 9), F(0), SPLIT), restrict(control_points(base, 9), SPLIT, F(1))
    def dec(value):
        with localcontext() as ctx:
            ctx.prec = 70
            return D(value.numerator)/D(value.denominator)
    def position(point):
        return [dec(v) for v in point]
    split = copy.deepcopy(base)
    split['revision'] += 1
    split['next_id'] = 24
    split['points'].append({'id': 21, 'position': position(left[-1])})
    index(split, 'curves')[9].update(end=21, controls=[position(p) for p in left[1:3]])
    split['curves'].append({'id': 22, 'start': 21, 'end': 6, 'controls': [position(p) for p in right[1:3]]})
    index(split, 'pieces')[2]['boundary'] = [7, 8, 9, 22, 10]
    split['seams'][0]['b']['t1'] = D('.6')
    second = copy.deepcopy(split['seams'][0])
    second['id'] = 23
    second['a']['curve'] = 22
    second['b'].update(t0=D('.6'), t1=D(0))
    split['seams'].append(second)
    joined = copy.deepcopy(base)
    joined['revision'] = split['revision']+1
    joined['next_id'] = split['next_id']
    joined['seams'] = copy.deepcopy(split['seams'])
    joined['seams'][0]['a']['t1'] = D('.4')
    joined['seams'][1]['a'].update(curve=9, t0=D('.4'), t1=D(1))
    return base, split, joined


def synthetic_svg(project):
    root = ET.Element('svg', xmlns='http://www.w3.org/2000/svg', width='1400mm', height='800mm', viewBox='0 -800 1400 800')
    def text(point):
        return ' '.join(str(upper_decimal(v*1000)) for v in (point[0], -point[1]))
    for piece in project['pieces']:
        commands = ['M'+text(control_points(project, piece['boundary'][0])[0])]
        for curve_id in piece['boundary']:
            points = control_points(project, curve_id)
            commands.append('C'+' '.join(text(p) for p in points[1:]) if index(project, 'curves')[curve_id]['controls'] is not None else 'L'+text(points[-1]))
        commands.append('Z')
        ET.SubElement(root, 'path', {'id': f"piece-{piece['id']}", 'data-name': piece['name'], 'd': ' '.join(commands)})
    return ET.tostring(root, encoding='unicode')


def self_check():
    require(digest(Path(__file__).with_name('verify_curve_outputs.py')) == ORIGINAL_SHA256, 'copied original evaluator bytes changed')
    require(digest(Path(__file__).with_name('historical-before-split.weftforge')) == HISTORICAL_SHA256, 'copied historical baseline bytes changed')
    base, split, joined = synthesize()
    baseline_check(base)
    discovered = split_check(base, split)
    good = join_check(base, split, joined, discovered)
    require(good['original_parent_geometry']['maximum_squared_control_difference_m2_exact'] == '0', 'synthetic parent is not exact')
    require(control_points(split, 9)[-1] == (F(153, 625), F(93, 125)), 'independent known split point')
    negatives = {}
    for label in ('control_drift_1e_6_m', 'seam_interval_drift_1e_6', 'id_retirement', 'unintended_material_change'):
        bad = copy.deepcopy(joined)
        measurement = None
        if label == 'control_drift_1e_6_m':
            index(bad, 'curves')[9]['controls'][0][0] += D('.000001')
            measurement = bound_record(squared_bound(control_points(bad, 9), control_points(base, 9)))
        elif label == 'seam_interval_drift_1e_6':
            bad['seams'][0]['a']['t1'] += D('.000001')
            measurement = parameter_record(F(1, 10**6))
        elif label == 'id_retirement':
            bad['points'].append(copy.deepcopy(index(split, 'points')[21]))
        else:
            bad['materials'][0]['friction'] += D('.000001')
        try:
            join_check(base, split, bad, discovered)
        except AssertionError as error:
            negatives[label] = {'status': 'rejected', 'reason': str(error), 'measured_error': measurement}
        else:
            raise AssertionError(f'negative oracle witness accepted: {label}')
    with tempfile.TemporaryDirectory(prefix='oracle-self-check-', dir=Path(__file__).parent) as directory:
        path = Path(directory)/'synthetic-joined.svg'
        svg = synthetic_svg(joined)
        path.write_text(svg)
        svg_result = svg_check(path, joined)
        require('C300 -800' in svg, 'stale-cache witness did not locate SVG control')
        path.write_text(svg.replace('C300 -800', 'C300.001 -800', 1))
        try:
            svg_check(path, joined)
        except AssertionError as error:
            negatives['stale_cache_svg_geometry_1e_6_m'] = {
                'status': 'rejected', 'reason': str(error), 'measured_error': bound_record(F(1, 10**12))}
        else:
            raise AssertionError('stale-cache SVG geometry accepted')
    return {'kind': 'synthesized_oracle_self_checks_only', 'status': 'passed',
            'exact_positive_split_and_join': good, 'synthetic_svg': svg_result,
            'negative_witnesses': negatives, 'native_outputs_created': False}


def declaration_check(path):
    declaration = json.loads(path.read_text())
    for field, size in (('source_build_commit', 40), ('source_tree_sha256', 64), ('executable_sha256', 64)):
        require(isinstance(declaration.get(field), str) and re.fullmatch('[0-9a-f]{'+str(size)+'}', declaration[field]), f'declaration identity pending/invalid: {field}')
    require(declaration.get('declared_at') and declaration.get('executable_basename'), 'declaration candidate fields pending')
    require(declaration.get('evaluator_sha256') == digest(__file__), 'declaration evaluator hash differs from invoked evaluator')
    require(declaration.get('original_evaluator_sha256') == ORIGINAL_SHA256, 'declaration original evaluator hash changed')
    require(declaration.get('required_files') == list(INPUT_FILES), 'preregistered file list changed')
    return {'sha256': digest(path), 'id': declaration['id'], 'declared_at': declaration['declared_at'],
            'source_build_commit': declaration['source_build_commit'], 'source_tree_sha256': declaration['source_tree_sha256'],
            'executable_sha256': declaration['executable_sha256'], 'candidate_identity_is_claimed_by_declaration': True,
            'limitation': 'Saved output alone cannot prove launch timing, executable process identity or GUI actions.'}


def evaluate(case_dir):
    projects = {name: original.read_project(case_dir/f'{name}.weftforge') for name in PROJECT_FILES}
    for project in projects.values():
        check_ids(project)
    base, split, joined = (projects[name] for name in ('before-split', 'after-split', 'after-join'))
    baseline = baseline_check(base)
    discovered = split_check(base, split)
    unchanged = [same_bytes(case_dir/'after-split.weftforge', case_dir/'after-focus.weftforge'),
                 same_bytes(case_dir/'after-join.weftforge', case_dir/'after-join-tab.weftforge'),
                 same_bytes(case_dir/'reopened.weftforge', case_dir/'after-corner-error.weftforge')]
    join = join_check(base, split, joined, discovered)
    undo = construction_equal(split, projects['join-undone'], 'single Undo')
    require(projects['join-undone']['next_id'] >= joined['next_id'], 'Undo released retired IDs')
    require(projects['join-undone']['revision'] == joined['revision']+1, 'Undo was not one revision')
    redo = construction_equal(joined, projects['join-redone'], 'single Redo')
    require(projects['join-redone']['revision'] == projects['join-undone']['revision']+1, 'Redo was not one revision')
    require(projects['join-redone']['next_id'] == projects['join-undone']['next_id'], 'Redo allocated/reused IDs')
    refinement = []
    previous = projects['join-redone']
    for name, resolution in (('coarse', D('.09')), ('fine', D('.035'))):
        project = projects[name]
        equality = construction_equal(previous, project, f'{name} refinement', ('revision', 'resolution'))
        require(project['revision'] == previous['revision']+1 and project['resolution'] == resolution, 'resolution edit/revision differs from preregistration')
        refinement.append({'checkpoint': name, 'resolution_m': resolution, **equality})
        previous = project
    reopened = construction_equal(projects['fine'], projects['reopened'], 'reopen', ('revision', 'next_id'))
    require(projects['reopened']['next_id'] == projects['fine']['next_id'], 'reopen changed allocator')
    objs = {name: original.obj_check(case_dir/f'{name}.obj') for name in ('coarse', 'fine')}
    require(objs['fine']['vertices'] > objs['coarse']['vertices'], 'fine mesh did not increase vertices')
    svgs = {name: svg_check(case_dir/f'{name}.svg', projects[name]) for name in ('coarse', 'fine')}
    timeline = [projects[name]['next_id'] for name in PROJECT_FILES]
    require(all(b >= a for a, b in zip(timeline, timeline[1:])), 'allocator regressed along workflow')
    return {'declaration': declaration_check(case_dir/'ui-declaration.json'), 'baseline': baseline,
            'split': discovered, 'join': join, 'unchanged_bytes': unchanged, 'undo': undo,
            'redo': redo, 'refinement': refinement, 'reopen': reopened, 'obj': objs, 'svg': svgs,
            'fine_vertex_increase': objs['fine']['vertices']-objs['coarse']['vertices'],
            'allocator_timeline': timeline, 'native_ui_execution_requires_parent_observation_record': True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--case-dir', type=Path)
    parser.add_argument('--self-check-only', action='store_true')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    require(args.case_dir is not None or args.self_check_only, '--case-dir is required unless --self-check-only')
    result = {'scenario': 'UI-BOUNDED-JOIN-001', 'kind': 'implementation_only',
              'evaluator_sha256': digest(__file__), 'original_evaluator_sha256': ORIGINAL_SHA256,
              'absolute_tolerance_m': '1e-12', 'normalized_parameter_tolerance': '1e-12',
              'arithmetic': 'Exact rational serialized-decimal geometry; outward Decimal70 reporting.',
              'limitations': ['No reference execution/parity, cloth fitting or simulation claim.',
                              'No large-translation native coverage; default placements only.',
                              'No coalesced-event native coverage; saved files do not establish GUI event timing.',
                              'GUI actions, focused buffers, error display and process identity require the parent UI record.']}
    try:
        result['self_check'] = self_check()
        if args.self_check_only:
            result['status'] = 'passed_synthesized_self_checks_only'
        else:
            missing = [name for name in INPUT_FILES if not (args.case_dir/name).is_file()]
            if missing:
                result.update(status='incomplete_missing_native_outputs', missing_files=missing)
            else:
                result['checks'] = evaluate(args.case_dir)
                result['status'] = 'passed_for_supplied_outputs'
                result['artifacts'] = {name: {'sha256': digest(args.case_dir/name), 'bytes': (args.case_dir/name).stat().st_size} for name in INPUT_FILES}
    except (AssertionError, KeyError, TypeError, ValueError, OSError, ET.ParseError, ArithmeticError) as error:
        result.update(status='failed', failure={'type': type(error).__name__, 'message': str(error)})
    output = args.output or (args.case_dir/'evaluation.json' if args.case_dir else None)
    if output:
        output.write_text(json.dumps(result, indent=2, default=str)+'\n')
    print(json.dumps(result if output is None else {'status': result['status'], 'report': output.name,
                                                   'report_sha256': digest(output), 'evaluator_sha256': result['evaluator_sha256'],
                                                   'failure': result.get('failure'), 'missing_files': result.get('missing_files', [])}, default=str))
    return 0 if result['status'].startswith('passed') else 2 if result['status'].startswith('incomplete') else 1


if __name__ == '__main__':
    sys.exit(main())
