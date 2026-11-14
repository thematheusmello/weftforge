#!/usr/bin/env python3
"""Fixed independent completion checks for UI-NUMERIC-RECTANGLE-002."""
import collections
import copy
from decimal import Decimal as D, getcontext
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET
from verify_initial_numeric import load, rejected, require, sha
from verify_dynamic_numeric import check_focus

getcontext().prec = 70
ROOT = Path(__file__).parent
TOL = D('1e-12')
NATIVE = ['empty.weftforge', 'after-cancel.weftforge', 'after-errors.weftforge', 'numeric.weftforge',
          'numeric-undone.weftforge', 'numeric-redone.weftforge', 'identity-a.weftforge', 'identity-b.weftforge',
          'curve-collinear.weftforge', 'identity-curved.weftforge', 'focused-undone.weftforge', 'focused-redone.weftforge',
          'precision-curved.weftforge', 'precision-focused-undone.weftforge', 'precision-focused-redone.weftforge',
          'small-before-focus.weftforge', 'small-after-focus.weftforge', 'small-after-invalid.weftforge', 'reopened.weftforge']


def item(doc, kind, ident):
    return next(value for value in doc['project'][kind] if value['id'] == ident)


def small_check(before, after):
    project = after['project']
    points = [item(after, 'points', ident) for ident in range(30, 34)]
    require(all(set(p) == {'id', 'position'} and len(p['position']) == 2 for p in points), 'Small point schema')
    a, b, c, d = [p['position'] for p in points]
    require(a[1] == b[1] and b[0] == c[0] and c[1] == d[1] and d[0] == a[0], 'Small axis-aligned boundary')
    width, height = b[0] - a[0], d[1] - a[1]
    require(abs(width - D('.005')) <= TOL and abs(height - D('.006')) <= TOL, 'Small must remain 5x6mm')
    expected = copy.deepcopy(before)
    ep = expected['project']
    ep['revision'] += 1
    ep['next_id'] = 38
    ep['points'].extend(copy.deepcopy(points))
    ep['curves'].extend({'id': 34 + i, 'start': 30 + i, 'end': 30 + (i + 1) % 4, 'controls': None} for i in range(4))
    ep['pieces'].append({'id': 29, 'name': 'Small', 'boundary': [34, 35, 36, 37],
                         'placement': {'translation': [0, 0, 0], 'rotation': [0, 0, 0]}, 'material': 1, 'grain_angle': 0})
    require(after == expected, 'Small creation changed unrelated native fields')
    return {'revision': project['revision'], 'next_id': project['next_id'], 'width_m': width, 'height_m': height,
            'measured_anchor_m': a, 'anchor_policy': 'Recorded source position; no independent UI placement claim.',
            'width_height_errors_m': [abs(width - D('.005')), abs(height - D('.006'))],
            'only_declared_small_construction_added': True}


def obj_check(data):
    vertices, uv, normals, faces = [], [], [], []
    for line in data.splitlines():
        row = line.split()
        if not row or row[0].startswith('#'):
            continue
        kind, values = row[0], row[1:]
        if kind in ('v', 'vt', 'vn'):
            numbers = tuple(D(value) for value in values)
            require(len(numbers) == (2 if kind == 'vt' else 3) and all(v.is_finite() for v in numbers), 'OBJ finite coordinate dimensions')
            {'v': vertices, 'vt': uv, 'vn': normals}[kind].append(numbers)
        elif kind == 'f':
            require(len(values) == 3, 'OBJ triangular faces')
            face = [tuple(int(i) for i in token.split('/')) for token in values]
            require(all(len(corner) == 3 for corner in face), 'OBJ complete UV/normal references')
            faces.append(face)
    require(vertices and faces and len(vertices) == len(uv) == len(normals), 'OBJ arrays')
    edges, adjacent = collections.Counter(), collections.defaultdict(set)
    minimum = D('Infinity')
    for face in faces:
        require(all(1 <= c[0] <= len(vertices) and 1 <= c[1] <= len(uv) and 1 <= c[2] <= len(normals) for c in face), 'OBJ indices')
        ids = [c[0] - 1 for c in face]
        require(len(set(ids)) == 3, 'OBJ repeated triangle vertex')
        p, q, r = [vertices[i] for i in ids]
        area = (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
        require(area > 0, 'OBJ positive projected triangles')
        minimum = min(minimum, area)
        up, uq, ur = [uv[c[1] - 1] for c in face]
        require((uq[0] - up[0]) * (ur[1] - up[1]) - (uq[1] - up[1]) * (ur[0] - up[0]) > 0, 'OBJ positive UV triangles')
        for a, b in zip(ids, ids[1:] + ids[:1]):
            edges[tuple(sorted((a, b)))] += 1
            adjacent[a].add(b); adjacent[b].add(a)
    require(len(adjacent) == len(vertices) and all(n in (1, 2) for n in edges.values()), 'OBJ used manifold vertices')
    require(all(abs(p[2]) <= TOL for p in vertices), 'OBJ expected unsimulated XY plane')
    require(all(abs(n[0]) <= TOL and abs(n[1]) <= TOL and abs(n[2] - 1) <= TOL for n in normals), 'OBJ finite aligned unit normals')
    require(all(max(abs(p[i] - t[i]) for i in (0, 1)) <= TOL for p, t in zip(vertices, uv)), 'OBJ UVs remain metres')
    unseen, components = set(adjacent), 0
    while unseen:
        components += 1
        todo = [unseen.pop()]
        while todo:
            for neighbor in adjacent[todo.pop()]:
                if neighbor in unseen:
                    unseen.remove(neighbor); todo.append(neighbor)
    require(components == 4 and len(vertices) - len(edges) + len(faces) == 4, 'OBJ four components and expected summed Euler characteristic')
    bounds = [[min(p[i] for p in vertices), max(p[i] for p in vertices)] for i in range(3)]
    require(abs(bounds[0][0]) <= TOL and abs(bounds[0][1] - D('2.4')) <= TOL and bounds[1][1] <= D('.75') + TOL, 'OBJ fixed metre bounds')
    return {'vertices': len(vertices), 'uvs': len(uv), 'normals': len(normals), 'triangles': len(faces), 'edges': len(edges),
            'components': components, 'euler_characteristic_sum': 4, 'bounds_m': bounds,
            'minimum_twice_projected_area_m2': minimum, 'all_faces_positive': True, 'finite_unit_normals_and_uvs': True}


def svg_check(svg, doc):
    require(svg.tag == '{http://www.w3.org/2000/svg}svg' and all('transform' not in n.attrib for n in svg.iter()), 'SVG namespace/no implicit transform')
    require(svg.attrib['width'].endswith('mm') and svg.attrib['height'].endswith('mm'), 'SVG fixed millimetres')
    view = [D(v) for v in svg.attrib['viewBox'].split()]
    require(len(view) == 4 and D(svg.attrib['width'][:-2]) == view[2] and D(svg.attrib['height'][:-2]) == view[3], 'SVG one unit equals one millimetre')
    paths = {n.attrib['id']: n for n in svg.findall('{http://www.w3.org/2000/svg}path')}
    require(set(paths) == {f'piece-{p["id"]}' for p in doc['project']['pieces']}, 'SVG editable piece IDs')
    points = {p['id']: p['position'] for p in doc['project']['points']}
    curves = {c['id']: c for c in doc['project']['curves']}
    maximum, cubics = D(0), 0
    token = r'[A-Za-z]|[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?'
    for piece in doc['project']['pieces']:
        path = paths[f'piece-{piece["id"]}']
        require(path.attrib['data-name'] == piece['name'], 'SVG piece names')
        data = path.attrib['d']
        require(not re.sub(token, '', data).replace(',', '').strip(), 'SVG complete syntax')
        actual = [v if v.isalpha() else D(v) for v in re.findall(token, data)]
        start = points[curves[piece['boundary'][0]]['start']]
        expected = ['M', start[0] * 1000, -start[1] * 1000]
        for ident in piece['boundary']:
            curve = curves[ident]
            controls = curve['controls']
            expected.append('C' if controls else 'L')
            if controls:
                cubics += 1
                for p in controls:
                    expected.extend([p[0] * 1000, -p[1] * 1000])
            end = points[curve['end']]
            expected.extend([end[0] * 1000, -end[1] * 1000])
        expected.append('Z')
        require(len(actual) == len(expected), 'SVG editable boundary commands')
        for a, b in zip(actual, expected):
            if isinstance(b, str):
                require(a == b, 'SVG curve type or closure changed')
            else:
                require(not isinstance(a, str) and a.is_finite(), 'SVG finite point')
                maximum = max(maximum, abs(a - b) / 1000)
    require(len(paths) == 4 and cubics == 1 and maximum <= TOL, 'SVG fixed-unit authored geometry')
    return {'editable_piece_paths': len(paths), 'editable_cubic_segments': cubics, 'physical_user_unit': 'millimetre',
            'maximum_authored_coordinate_error_m': maximum, 'cubic_all_points_error_bound_m': maximum * D(2).sqrt()}


def main():
    docs = {name: load(ROOT / name) for name in NATIVE}
    first = load(ROOT / 'independent-initial-check.json')
    partial_report = load(ROOT / 'independent-dynamic-check.json')
    require(first['status'].startswith('passed_for_supplied_initial_outputs'), 'Initial phase result')
    require(partial_report['status'] == 'passed_precision_and_focused_history_with_incomplete_curve_target', 'Partial phase retained')
    for report in (first, partial_report):
        for name, info in report['artifacts'].items():
            require(sha(ROOT / name) == info['sha256'], 'Earlier phase artifact changed: ' + name)
    partial, complete = docs['identity-curved.weftforge'], docs['precision-curved.weftforge']
    expected = copy.deepcopy(partial)
    expected['project']['revision'] += 1
    item(expected, 'curves', 27)['controls'][1][1] = D('.8')
    require(complete == expected, 'Complete curve changed fields other than P2 Y and revision')
    collinear = item(docs['curve-collinear.weftforge'], 'curves', 27)['controls']
    controls = item(complete, 'curves', 27)['controls']
    require([p[0] for p in controls] == [p[0] for p in collinear], 'Untouched control X changed')
    rational_errors = [abs(p[0] - wanted) for p, wanted in zip(controls, (D(34) / 15, D(32) / 15))]
    require(max(rational_errors) <= TOL and [p[1] for p in controls] == [D('.8'), D('.8')], 'Independent fixed cubic target')
    undo, redo = docs['precision-focused-undone.weftforge'], docs['precision-focused-redone.weftforge']
    check_focus(complete, undo, redo)
    small = docs['small-before-focus.weftforge']
    small_result = small_check(redo, small)
    stable = ['small-before-focus.weftforge', 'small-after-focus.weftforge', 'small-after-invalid.weftforge', 'reopened.weftforge']
    for name in stable:
        require((ROOT / name).read_bytes() == (ROOT / stable[0]).read_bytes(), 'Small no-edit/invalid/reopen bytes changed')
    for name, doc in docs.items():
        require(doc['project']['materials'] == docs['empty.weftforge']['project']['materials'], 'Material drift in ' + name)
    changed = copy.deepcopy(small)
    for ident in (31, 32):
        item(changed, 'points', ident)['position'][0] += D('.005')
    witnesses = {'selected_size_clamped_to_10mm': rejected(lambda value: small_check(redo, value), changed)}
    obj_text = (ROOT / 'final.obj').read_text()
    obj_result = obj_check(obj_text)
    lines = obj_text.splitlines()
    face_line = next(i for i, line in enumerate(lines) if line.startswith('f '))
    face = lines[face_line].split(); face[-1] = face[1]; lines[face_line] = ' '.join(face)
    witnesses['obj_repeated_vertex_face'] = rejected(obj_check, '\n'.join(lines))
    svg = ET.parse(ROOT / 'final.svg').getroot()
    svg_result = svg_check(svg, small)
    changed_svg = copy.deepcopy(svg); changed_svg.attrib['width'] = '2400cm'
    witnesses['svg_wrong_physical_unit'] = rejected(lambda value: svg_check(value, small), changed_svg)
    restart = load(ROOT / 'restart-process.json')
    require(restart['executable_sha256'] == first['candidate']['executable_sha256'] and restart['source_commit'] == first['candidate']['source_commit'], 'Restart identity records')
    classification = load(ROOT / 'invalid-mixed-instance-attempt' / 'classification.json')
    require(classification['status'] == 'invalid_automation_attempt', 'Mixed-instance classification must remain invalid')
    invalid_names = ['invalid-mixed-instance-attempt/classification.json']
    for name, digest in classification['files'].items():
        relative = 'invalid-mixed-instance-attempt/' + name
        require(sha(ROOT / relative) == digest, 'Mixed-instance evidence changed')
        invalid_names.append(relative)
    artifacts = [*NATIVE, 'declaration.json', 'process.json', 'restart-process.json', 'numeric.obj', 'numeric.svg',
                 'final.obj', 'final.svg', 'independent-numeric-bounds.json', 'independent-numeric-boundary.json',
                 'independent-initial-check.json', 'independent-dynamic-check.json', *invalid_names]
    result = {'scenario': 'UI-NUMERIC-RECTANGLE-002', 'kind': 'implementation_only', 'status': 'passed_main_native_saved_output_acceptance',
              'evaluator_sha256': sha(Path(__file__)), 'candidate': first['candidate'], 'fixed_authoring_tolerance_m': TOL,
              'fixed_initial_obj_tolerance_mm': D('.01'), 'initial_phase_report_sha256': sha(ROOT / 'independent-initial-check.json'),
              'partial_phase_retained': {'report_sha256': sha(ROOT / 'independent-dynamic-check.json'), 'both_y_target_reached': False,
                   'completion_is_separate_file': 'precision-curved.weftforge'},
              'completed_curve': {'revision': complete['project']['revision'], 'controls_m': controls,
                   'untouched_x_exactly_equal_to_collinear': True, 'independent_third_x_errors_m': rational_errors,
                   'only_completion_changes': ['revision', 'curves[id=27].controls[1][1]']},
              'focused_history': {'undo_revision': undo['project']['revision'], 'redo_revision': redo['project']['revision'],
                   'undo_matches_complete_except_revision': True, 'redo_changes_only_b_grain_to_point25_and_revision': True},
              'small': {**small_result, 'focus_invalid_restore_reopen_bytes_identical': stable, 'shared_sha256': sha(ROOT / stable[0])},
              'materials_exact_empty_baseline_all_qualified_native_files': True, 'final_obj': obj_result, 'final_svg': svg_result,
              'negative_witnesses': witnesses, 'invalid_mixed_instance': {'qualification': 'excluded; no candidate execution attribution',
                   'classification_sha256': sha(ROOT / invalid_names[0]), 'retained_artifacts': invalid_names},
              'artifacts': {name: {'sha256': sha(ROOT / name), 'bytes': (ROOT / name).stat().st_size} for name in artifacts},
              'separate_initialization_cancellation': 'Not evaluated by this saved-output checker; requires separately measured parent evidence.',
              'limits': ['No reference parity, reference placement/topology/material equivalence, cloth fitting or rendering-quality claim.',
                        'UI gestures, field text, explicit app/process identity and reopen interaction remain parent-observed evidence.',
                        'Final OBJ validates finite positive mesh/UV/normals and bounded topology; no cubic tessellation accuracy at 1e-12m is claimed.',
                        'Earlier incomplete action and mixed-instance attempt are retained separately, without normalization.']}
    output = ROOT / 'independent-completed-check.json'
    output.write_text(json.dumps(result, indent=2, default=str) + '\n')
    print(json.dumps({'status': result['status'], 'report': output.name, 'report_sha256': sha(output),
                      'evaluator_sha256': result['evaluator_sha256']}))


if __name__ == '__main__':
    main()
