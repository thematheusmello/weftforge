#!/usr/bin/env python3
"""Independent checks of the UI-SEWN-CURVE-002 saved construction and exports."""
import argparse
import copy
from decimal import Decimal as D, getcontext
import hashlib
import json
from pathlib import Path
import re
import sys
import xml.etree.ElementTree as ET

getcontext().prec = 70
TOL = D('1e-12')
SPLIT = D('.4')
EXPECTED = ((D('.4'), D('.6')), (D('.3'), D('.8')), (D('.1'), D('.8')), (D(0), D('.6')))


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def add(a, b):
    return tuple(x + y for x, y in zip(a, b))


def scale(a, t):
    return tuple(x * t for x in a)


def lerp(a, b, t):
    return add(scale(a, 1 - t), scale(b, t))


def bezier(c, t):
    while len(c) > 1:
        c = tuple(lerp(a, b, t) for a, b in zip(c, c[1:]))
    return c[0]


def derivative(c, t):
    return bezier(tuple(scale(add(b, scale(a, -1)), D(3)) for a, b in zip(c, c[1:])), t)


def subcurve(c, a, b):
    start, end = bezier(c, a), bezier(c, b)
    return (start, add(start, scale(derivative(c, a), (b - a) / 3)),
            add(end, scale(derivative(c, b), (a - b) / 3)), end)


def curve_bound(a, b):
    # The Bernstein convex hull bounds every point, not just a sample grid.
    return max(sum((x - y) ** 2 for x, y in zip(p, q)).sqrt() for p, q in zip(a, b))


def close(a, b, message):
    require(abs(a - b) <= TOL, f'{message}: {a} != {b}')


def read_project(path):
    def bad_constant(value):
        raise AssertionError(f'non-finite JSON value: {value}')
    doc = json.loads(Path(path).read_text(), parse_float=D, parse_int=int, parse_constant=bad_constant)
    require(doc['format'] == 'weftforge' and doc['version'] == 2, 'expected native v2 document')
    return doc['project']


def index(project, kind):
    return {item['id']: item for item in project[kind]}


def controls(project, curve_id):
    points, curves = index(project, 'points'), index(project, 'curves')
    c = curves[curve_id]
    a, b = tuple(points[c['start']]['position']), tuple(points[c['end']]['position'])
    inner = c['controls'] or (lerp(a, b, D(1) / 3), lerp(a, b, D(2) / 3))
    return (a, tuple(inner[0]), tuple(inner[1]), b)


def project_summary(project):
    return {'revision': project['revision'], 'resolution_m': project['resolution'],
            'next_id': project['next_id'],
            'ids': {kind: sorted(index(project, kind)) for kind in ('points', 'curves', 'pieces', 'materials', 'seams')}}


def baseline_check(project):
    points, pieces, seams = (index(project, k) for k in ('points', 'pieces', 'seams'))
    expected = {3: [D(0), D(0)], 4: [D('.4'), D(0)], 5: [D('.4'), D('.6')], 6: [D(0), D('.6')],
                12: [D(1), D(0)], 13: [D('1.4'), D(0)], 14: [D('1.4'), D('.6')], 15: [D(1), D('.6')]}
    require(len(points) == 8 and all(points[k]['position'] == v for k, v in expected.items()), 'baseline rectangle corners')
    require(pieces[2]['name'] == 'Curve A' and pieces[2]['boundary'] == [7, 8, 9, 10], 'baseline first piece')
    require(pieces[11]['name'] == 'Curve B' and pieces[11]['boundary'] == [16, 17, 18, 19], 'baseline second piece')
    require(curve_bound(controls(project, 9), EXPECTED) == 0, 'baseline cubic controls')
    require(len(seams) == 1 and seams[20] == {'id': 20, 'name': 'Seam 1',
            'a': {'curve': 9, 't0': D(0), 't1': D(1)},
            'b': {'curve': 18, 't0': D(1), 't1': D(0)},
            'correspondence': [[D(0), D(0)], [D(1), D(1)]]}, 'baseline oriented seam')


def original_check(base, project):
    for key in base:
        if key not in ('revision', 'resolution', 'next_id'):
            require(project[key] == base[key], f'original construction changed: {key}')
    return project_summary(project)


def split_check(base, project):
    old_points, points = index(base, 'points'), index(project, 'points')
    old_curves, curves = index(base, 'curves'), index(project, 'curves')
    old_pieces, pieces = index(base, 'pieces'), index(project, 'pieces')
    seams = index(project, 'seams')
    require(set(points) >= set(old_points) and len(points) == 9, 'split point count or IDs')
    require(all(points[k] == v for k, v in old_points.items()), 'original points changed')
    require(set(curves) >= set(old_curves) and len(curves) == 9, 'split curve count or IDs')
    require(all(curves[k] == v for k, v in old_curves.items() if k != 9), 'unrelated curve changed')
    require(set(pieces) == set(old_pieces), 'piece IDs changed')
    right_ids = set(curves) - set(old_curves)
    right = next(iter(right_ids))
    join_id = next(iter(set(points) - set(old_points)))
    require(curves[9]['start'] == 5 and curves[9]['end'] == join_id, 'first subcurve relationships')
    require(curves[right]['start'] == join_id and curves[right]['end'] == 6, 'second subcurve relationships')
    require(pieces[2]['boundary'] == [7, 8, 9, right, 10], 'ordered split boundary')
    for piece_id in pieces:
        a, b = dict(pieces[piece_id]), dict(old_pieces[piece_id])
        if piece_id == 2:
            a.pop('boundary'); b.pop('boundary')
        require(a == b, f'unrelated piece data changed: {piece_id}')
    for key in ('materials', 'mannequin'):
        require(project[key] == base[key], f'unrelated {key} changed')
    expected = {9: subcurve(EXPECTED, D(0), SPLIT), right: subcurve(EXPECTED, SPLIT, D(1))}
    geometry_bound = max(curve_bound(controls(project, k), c) for k, c in expected.items())
    require(geometry_bound <= TOL, f'cubic preservation bound {geometry_bound} m')
    require(20 in seams and len(seams) == 2, 'split seam count or original seam ID')
    ids = [item['id'] for kind in ('points', 'curves', 'pieces', 'materials', 'seams') for item in project[kind]]
    require(len(ids) == len(set(ids)) and project['next_id'] > max(ids), 'global stable IDs or next_id')
    require(min(set(ids) - set(v['id'] for kind in ('points', 'curves', 'pieces', 'materials', 'seams') for v in base[kind])) >= base['next_id'], 'new IDs precede baseline next_id')
    spans, physical_bound, parameter_bound, interval_count = [], D(0), D(0), 0
    curve_domains = {9: (D(0), SPLIT), right: (SPLIT, D(1))}
    original_b = controls(base, 18)
    for seam in seams.values():
        a, b = seam['a'], seam['b']
        require(a['curve'] in curve_domains and b['curve'] == 18, 'split seam curve relationships')
        lo, hi = curve_domains[a['curve']]
        pairs = seam['correspondence']
        require(len(pairs) >= 2 and pairs[0] == [0, 0] and pairs[-1] == [1, 1], 'normalized correspondence endpoints')
        close(a['t0'], D(0), 'subcurve span start'); close(a['t1'], D(1), 'subcurve span end')
        spans.append((lo, hi))
        for (u0, v0), (u1, v1) in zip(pairs, pairs[1:]):
            require(0 <= u0 < u1 <= 1 and 0 <= v0 < v1 <= 1, 'monotonic normalized correspondence')
            at0, at1 = (a['t0'] + (a['t1'] - a['t0']) * u for u in (u0, u1))
            bt0, bt1 = (b['t0'] + (b['t1'] - b['t0']) * v for v in (v0, v1))
            g0, g1 = (lo + (hi - lo) * t for t in (at0, at1))
            parameter_bound = max(parameter_bound, abs(bt0 - (1 - g0)), abs(bt1 - (1 - g1)))
            physical_bound = max(physical_bound,
                curve_bound(subcurve(controls(project, a['curve']), at0, at1), subcurve(EXPECTED, g0, g1)),
                curve_bound(subcurve(controls(project, 18), bt0, bt1), subcurve(original_b, 1 - g0, 1 - g1)))
            interval_count += 1
    require(sorted(spans) == [(D(0), SPLIT), (SPLIT, D(1))], 'seam domain coverage')
    require(parameter_bound <= TOL, f'normalized seam parameter bound {parameter_bound}')
    require(physical_bound <= TOL, f'all sewn pairs bound {physical_bound} m')
    return {**project_summary(project), 'join_point_id': join_id, 'second_curve_id': right,
            'join_point_m': points[join_id]['position'], 'subcurve_controls_m': expected,
            'cubic_all_points_error_bound_m': geometry_bound,
            'sewn_all_point_pairs_error_bound_m': physical_bound,
            'normalized_parameter_error_bound': parameter_bound,
            'seam_correspondence_intervals_proved': interval_count}


def obj_check(path):
    vertices, uvs, normals, faces = [], [], [], []
    for line in Path(path).read_text().splitlines():
        fields = line.split()
        if not fields or fields[0].startswith('#'):
            continue
        kind, values = fields[0], fields[1:]
        if kind in ('v', 'vt', 'vn'):
            row = tuple(D(v) for v in values)
            require(all(v.is_finite() for v in row), f'non-finite OBJ {kind}')
            require(len(row) == (2 if kind == 'vt' else 3), f'OBJ {kind} dimensions')
            {'v': vertices, 'vt': uvs, 'vn': normals}[kind].append(row)
        elif kind == 'f':
            require(len(values) == 3, 'non-triangle OBJ face')
            rows = [tuple(int(i) for i in value.split('/')) for value in values]
            require(all(len(row) == 3 for row in rows), 'missing OBJ UV or normal references')
            faces.append(rows)
    require(vertices and faces and uvs and normals, 'empty OBJ data')
    require(max(abs(v) for row in vertices for v in row) < 2, 'OBJ exceeds declared metre scale')
    min_cross, min_uv = D('Infinity'), D('Infinity')
    for face in faces:
        require(all(1 <= row[0] <= len(vertices) and 1 <= row[1] <= len(uvs) and 1 <= row[2] <= len(normals) for row in face), 'OBJ reference outside arrays')
        a, b, c = (vertices[row[0] - 1] for row in face)
        x, y = tuple(q - p for p, q in zip(a, b)), tuple(q - p for p, q in zip(a, c))
        cross = (x[1] * y[2] - x[2] * y[1], x[2] * y[0] - x[0] * y[2], x[0] * y[1] - x[1] * y[0])
        cross_norm = sum(v * v for v in cross).sqrt()
        require(cross_norm > 0, 'non-positive OBJ triangle area')
        require(all(sum(x * y for x, y in zip(cross, normals[row[2] - 1])) > 0 for row in face), 'OBJ face normal disagrees with winding')
        p, q, r = (uvs[row[1] - 1] for row in face)
        uv_area = (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
        require(uv_area > 0, 'non-positive OBJ UV triangle area')
        min_cross, min_uv = min(min_cross, cross_norm), min(min_uv, uv_area)
    require(all(abs(sum(x * x for x in n).sqrt() - 1) <= TOL for n in normals), 'OBJ normals are not unit vectors')
    bounds = [[min(p[i] for p in vertices), max(p[i] for p in vertices)] for i in range(3)]
    require(D('1.3') < max(b - a for a, b in bounds) < D('1.5'), 'OBJ full pattern scale')
    return {'sha256': digest(path), 'vertices': len(vertices), 'triangles': len(faces), 'uvs': len(uvs),
            'normals': len(normals), 'bounds_m': bounds, 'minimum_twice_triangle_area_m2': min_cross,
            'minimum_twice_uv_triangle_area': min_uv, 'finite_coordinates_uvs_and_normals': True}


def svg_path_commands(data):
    token = r'[MLCZ]|[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?'
    require(not re.sub(token, '', data).replace(',', '').strip(), 'unsupported SVG path syntax')
    tokens, commands, cursor = re.findall(token, data), [], 0
    sizes = {'M': 2, 'L': 2, 'C': 6, 'Z': 0}
    while cursor < len(tokens):
        command = tokens[cursor]
        require(command in sizes, 'SVG command must be explicit')
        values = tuple(D(v) for v in tokens[cursor + 1:cursor + 1 + sizes[command]])
        require(len(values) == sizes[command] and all(v.is_finite() for v in values), 'SVG command dimensions')
        commands.append((command, values))
        cursor += 1 + sizes[command]
    return commands


def svg_check(path, project):
    root = ET.parse(path).getroot()
    require(root.tag == '{http://www.w3.org/2000/svg}svg', 'SVG root namespace')
    require(all('transform' not in node.attrib for node in root.iter()), 'unexpected SVG transform')
    require(root.attrib['width'].endswith('mm') and root.attrib['height'].endswith('mm'), 'SVG physical units')
    view = tuple(D(v) for v in root.attrib['viewBox'].split())
    require(len(view) == 4 and all(v.is_finite() for v in view), 'SVG viewBox')
    require(D(root.attrib['width'][:-2]) / view[2] == 1 and D(root.attrib['height'][:-2]) / view[3] == 1, 'one SVG user unit must be one millimetre')
    paths = {node.attrib['id']: node for node in root.findall('{http://www.w3.org/2000/svg}path')}
    require(set(paths) == {'piece-2', 'piece-11'}, 'editable SVG piece paths')
    pieces, curves = index(project, 'pieces'), index(project, 'curves')
    saved_bound, original_bound, cubic_count = D(0), D(0), 0
    for piece_id, piece in pieces.items():
        path_node = paths[f'piece-{piece_id}']
        require(path_node.attrib['data-name'] == piece['name'], 'SVG piece name')
        commands = svg_path_commands(path_node.attrib['d'])
        require(len(commands) == len(piece['boundary']) + 2, 'SVG boundary command count')
        require(commands[0][0] == 'M' and commands[-1] == ('Z', ()), 'SVG closed editable boundary')
        current = (commands[0][1][0] / 1000, -commands[0][1][1] / 1000)
        expected_start = controls(project, piece['boundary'][0])[0]
        saved_bound = max(saved_bound, sum((a - b) ** 2 for a, b in zip(current, expected_start)).sqrt())
        for curve_id, (command, values) in zip(piece['boundary'], commands[1:-1]):
            expected = controls(project, curve_id)
            require(command == ('C' if curves[curve_id]['controls'] else 'L'), 'SVG flattened or changed curve type')
            actual_points = tuple((values[i] / 1000, -values[i + 1] / 1000) for i in range(0, len(values), 2))
            if command == 'C':
                actual = (current, *actual_points)
                saved_bound = max(saved_bound, curve_bound(actual, expected))
                original = subcurve(EXPECTED, D(0), SPLIT) if curve_id == 9 else subcurve(EXPECTED, SPLIT, D(1))
                original_bound = max(original_bound, curve_bound(actual, original))
                cubic_count += 1
            else:
                saved_bound = max(saved_bound, sum((a - b) ** 2 for a, b in zip(actual_points[0], expected[-1])).sqrt())
            current = actual_points[-1]
    require(cubic_count == 2, 'SVG must retain both editable cubic segments')
    require(saved_bound <= TOL and original_bound <= TOL, 'SVG authored geometry changed')
    return {'sha256': digest(path), 'editable_piece_paths': len(paths), 'editable_cubic_segments': cubic_count,
            'physical_user_unit': 'millimetre', 'pattern_y_direction': 'upward via negated SVG Y',
            'saved_authoring_error_bound_m': saved_bound, 'original_cubic_all_points_error_bound_m': original_bound}


def self_check(base):
    left, right = subcurve(EXPECTED, D(0), SPLIT), subcurve(EXPECTED, SPLIT, D(1))
    require(left[-1] == (D('.2448'), D('.744')), 'independent split self-check')
    require(left[1] == (D('.36'), D('.68')) and left[2] == (D('.304'), D('.728')), 'left controls self-check')
    require(right[1] == (D('.156'), D('.768')) and right[2] == (D('.06'), D('.72')), 'right controls self-check')
    fixture = copy.deepcopy(base)
    fixture['points'].append({'id': 21, 'position': list(left[-1])})
    index(fixture, 'curves')[9].update(end=21, controls=[list(p) for p in left[1:3]])
    fixture['curves'].append({'id': 22, 'start': 21, 'end': 6, 'controls': [list(p) for p in right[1:3]]})
    index(fixture, 'pieces')[2]['boundary'] = [7, 8, 9, 22, 10]
    fixture['seams'][0]['b']['t1'] = 1 - SPLIT
    fixture['seams'].append({'id': 23, 'name': 'Self-check second seam',
                             'a': {'curve': 22, 't0': D(0), 't1': D(1)},
                             'b': {'curve': 18, 't0': 1 - SPLIT, 't1': D(0)},
                             'correspondence': [[D(0), D(0)], [D(1), D(1)]]})
    fixture['next_id'] = 24
    split_check(base, fixture)
    witnesses = {}
    for label in ('changed_control', 'wrong_orientation'):
        changed = copy.deepcopy(fixture)
        if label == 'changed_control':
            index(changed, 'curves')[9]['controls'][0][0] += D('.001')
        else:
            changed['seams'][0]['b']['t0'], changed['seams'][0]['b']['t1'] = D(0), SPLIT
        try:
            split_check(base, changed)
        except AssertionError as error:
            witnesses[label + '_negative_witness'] = {'status': 'rejected', 'reason': str(error)}
        else:
            raise AssertionError(f'negative witness accepted: {label}')
    require(svg_path_commands('M0 0 C1 2 3 4 5 6 Z') == [('M', (D(0), D(0))), ('C', tuple(D(i) for i in range(1, 7))), ('Z', ())], 'SVG parser self-check')
    try:
        svg_path_commands('M0 0 Q1 2 3 4 Z')
    except AssertionError:
        witnesses['unsupported_svg_command_negative_witness'] = 'rejected'
    else:
        raise AssertionError('SVG parser accepted an unsupported command')
    return {'de_casteljau_known_values': 'passed', 'valid_split_fixture': 'passed', 'svg_parser_known_path': 'passed', **witnesses}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', required=True)
    parser.add_argument('--original', action='append', default=[])
    parser.add_argument('--split', action='append', default=[])
    parser.add_argument('--unchanged', nargs=2, action='append', default=[])
    parser.add_argument('--obj', action='append', default=[])
    parser.add_argument('--mesh-pair', nargs=2, action='append', default=[])
    parser.add_argument('--refinement-pair', nargs=2, action='append', default=[])
    parser.add_argument('--svg', nargs=2, action='append', default=[], metavar=('SVG', 'PROJECT'))
    parser.add_argument('--grain-edit', nargs=3, action='append', default=[], metavar=('BEFORE', 'AFTER', 'ANGLE'))
    parser.add_argument('--preserved-file', nargs=2, action='append', default=[], metavar=('FILE', 'SHA256'))
    parser.add_argument('--control-edit', nargs=3, action='append', default=[], metavar=('BEFORE', 'AFTER', 'P1_X'))
    parser.add_argument('--evidence', action='append', default=[])
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    baseline = read_project(args.baseline)
    baseline_check(baseline)
    result = {'scenario': 'UI-SEWN-CURVE-002', 'kind': 'implementation_only',
              'evaluator_sha256': digest(__file__), 'invocation_arguments': sys.argv[1:],
              'tolerance_absolute_m': TOL, 'tolerance_normalized_parameter': TOL,
              'self_check': self_check(baseline), 'baseline': {}, 'original_checkpoints': {},
              'split_checkpoints': {}, 'unchanged_pairs': [], 'obj': {}, 'mesh_pairs': [], 'refinement_pairs': [], 'svg': {}, 'grain_edits': [], 'control_edits': [], 'preserved_files': [], 'artifacts': {},
              'limitations': ['No reference execution or parity claim.',
                              'No cloth fitting or simulation quality check.',
                              'UI gestures, error display, undo/redo, process identity and recovery interaction require the parent UI record.',
                              'The final recovered control edit intentionally changes geometry; preservation bounds concern the earlier split and its unchanged saves/exports.']}
    result['baseline'] = {'sha256': digest(args.baseline), **project_summary(baseline),
                          'authored_cubic_m': EXPECTED, 'oriented_seam': baseline['seams'][0]}
    for path in args.original:
        result['original_checkpoints'][Path(path).name] = {'sha256': digest(path), **original_check(baseline, read_project(path))}
    for path in args.split:
        result['split_checkpoints'][Path(path).name] = {'sha256': digest(path), **split_check(baseline, read_project(path))}
    for a, b in args.unchanged:
        require(read_project(a) == read_project(b), f'unchanged project changed: {a}, {b}')
        require(Path(a).read_bytes() == Path(b).read_bytes(), f'unchanged document bytes changed: {a}, {b}')
        result['unchanged_pairs'].append({'before': Path(a).name, 'after': Path(b).name,
                                          'before_sha256': digest(a), 'after_sha256': digest(b),
                                          'project_including_revision_unchanged': True, 'document_bytes_identical': True})
    for coarse, fine in args.refinement_pair:
        a, b = read_project(coarse), read_project(fine)
        changed = sorted(key for key in set(a) | set(b) if a.get(key) != b.get(key))
        require(changed == ['resolution', 'revision'], 'refinement changed authored construction')
        require(a['resolution'] == D('.09') and b['resolution'] == D('.035'), 'declared refinement resolutions')
        result['refinement_pairs'].append({'coarse': Path(coarse).name, 'fine': Path(fine).name,
            'coarse_sha256': digest(coarse), 'fine_sha256': digest(fine), 'changed_project_keys': changed,
            'coarse_resolution_m': a['resolution'], 'fine_resolution_m': b['resolution'],
            'coarse_revision': a['revision'], 'fine_revision': b['revision'], 'passed': True})
    for path in set(args.obj + [p for pair in args.mesh_pair for p in pair]):
        result['obj'][Path(path).name] = obj_check(path)
    for coarse, fine in args.mesh_pair:
        a, b = result['obj'][Path(coarse).name], result['obj'][Path(fine).name]
        require(b['vertices'] > a['vertices'], 'finer mesh did not increase vertices')
        result['mesh_pairs'].append({'coarse': Path(coarse).name, 'fine': Path(fine).name,
                                     'vertex_increase': b['vertices'] - a['vertices'], 'passed': True})
    for path, project in args.svg:
        result['svg'][Path(path).name] = svg_check(path, read_project(project))
    for before, after, angle in args.grain_edit:
        a, b, angle = read_project(before), read_project(after), D(angle)
        require(angle.is_finite(), 'non-finite expected grain angle')
        expected = copy.deepcopy(a)
        expected['revision'] += 1
        index(expected, 'pieces')[2]['grain_angle'] = angle
        require(b == expected, 'grain edit changed unexpected project fields or revision')
        result['grain_edits'].append({'before': Path(before).name, 'after': Path(after).name,
            'before_sha256': digest(before), 'after_sha256': digest(after),
            'before_revision': a['revision'], 'after_revision': b['revision'],
            'before_grain_angle_rad': index(a, 'pieces')[2]['grain_angle'], 'after_grain_angle_rad': angle,
            'only_project_changes': ['revision', 'pieces[id=2].grain_angle'], 'all_other_project_values_identical': True})
    for before, after, p1_x in args.control_edit:
        a, b, p1_x = read_project(before), read_project(after), D(p1_x)
        require(p1_x.is_finite(), 'non-finite expected control coordinate')
        expected = copy.deepcopy(a)
        expected['revision'] += 1
        index(expected, 'curves')[9]['controls'][0][0] = p1_x
        require(b == expected, 'control edit changed unexpected project fields or revision')
        result['control_edits'].append({'before': Path(before).name, 'after': Path(after).name,
            'before_sha256': digest(before), 'after_sha256': digest(after),
            'before_revision': a['revision'], 'after_revision': b['revision'],
            'before_curve9_p1_x_m': index(a, 'curves')[9]['controls'][0][0], 'after_curve9_p1_x_m': p1_x,
            'only_project_changes': ['revision', 'curves[id=9].controls[0][0]'], 'all_other_project_values_identical': True})
    for path, expected_sha in args.preserved_file:
        actual_sha = digest(path)
        require(actual_sha == expected_sha, f'preserved file bytes changed: {path}')
        result['preserved_files'].append({'file': Path(path).name, 'sha256': actual_sha, 'matches_prior_sha256': True})
    artifact_paths = [args.baseline, *args.original, *args.split, *args.obj, *args.evidence]
    for groups in (args.unchanged, args.mesh_pair, args.refinement_pair, args.svg):
        artifact_paths.extend(path for group in groups for path in group)
    for groups in (args.grain_edit, args.control_edit):
        artifact_paths.extend(path for group in groups for path in group[:2])
    artifact_paths.extend(path for path, _ in args.preserved_file)
    result['artifacts'] = {Path(path).name: {'sha256': digest(path), 'bytes': Path(path).stat().st_size}
                           for path in sorted(set(artifact_paths))}
    result['status'] = 'passed_for_supplied_outputs'
    Path(args.output).write_text(json.dumps(result, indent=2, default=str) + '\n')
    print(json.dumps({'status': result['status'], 'report': str(args.output),
                      'report_sha256': digest(args.output), 'evaluator_sha256': result['evaluator_sha256'],
                      'original_checkpoints': len(args.original), 'split_checkpoints': len(args.split),
                      'obj_outputs': len(result['obj'])}))


if __name__ == '__main__':
    main()
