#!/usr/bin/env python3
"""Saved-snapshot cancellation checks; no worker-exit or constructor timing claim."""
import collections
import copy
from decimal import Decimal as D, getcontext
import json
from pathlib import Path
from verify_initial_numeric import load, rejected, require, sha

getcontext().prec = 70
ROOT = Path(__file__).parent
TOL = D('1e-12')


def migrated_v2(prepared):
    expected = copy.deepcopy(prepared)
    require(expected['version'] == 1, 'Prepared original must retain its v1 schema')
    expected['version'] = 2
    for piece in expected['project']['pieces']:
        require('grain_angle' not in piece, 'Original v1 piece has unexpected grain field')
        piece['grain_angle'] = D(0)
    for material in expected['project']['materials']:
        stiffness = material.pop('stretch_stiffness')
        material['warp_stiffness'] = stiffness
        material['weft_stiffness'] = stiffness
    return expected


def source_compare(expected, actual):
    a, b = copy.deepcopy(expected), copy.deepcopy(actual)
    ep, ap = a['project']['points'], b['project']['points']
    require(len(ep) == len(ap), 'Migrated source point count')
    deltas, maximum = [], D(0)
    for e, v in zip(ep, ap):
        old, new = e.pop('position'), v.pop('position')
        require(len(old) == len(new) == 2, 'Migrated source point dimensions')
        for axis, (x, y) in enumerate(zip(old, new)):
            error = abs(y - x)
            require(error <= TOL, 'Native source coordinate exceeds unchanged1e-12m tolerance')
            maximum = max(maximum, error)
            if x != y:
                deltas.append({'point_id': e['id'], 'axis': axis, 'before_m': x, 'after_m': y, 'signed_delta_m': y - x})
    require(a == b, 'Native migration changed source fields beyond explicit schema migration')
    return {'maximum_coordinate_error_m': maximum, 'coordinate_values_exactly_equal': not deltas,
            'coordinate_deltas': deltas, 'all_other_source_fields_identical_after_schema_mapping': True}


def snapshot_equal(before, after):
    require(before == after, 'Cancelled accepted snapshot bytes changed')


def mesh_check(text):
    vertices, uv, normals, faces = [], [], [], []
    for line in text.splitlines():
        row = line.split()
        if not row or row[0].startswith('#'):
            continue
        kind, values = row[0], row[1:]
        if kind in ('v', 'vt', 'vn'):
            numbers = tuple(D(v) for v in values)
            require(len(numbers) == (2 if kind == 'vt' else 3) and all(v.is_finite() for v in numbers), 'Finite OBJ coordinate dimensions')
            {'v': vertices, 'vt': uv, 'vn': normals}[kind].append(numbers)
        elif kind == 'f':
            require(len(values) == 3, 'Triangle OBJ faces')
            corners = [tuple(int(i) for i in v.split('/')) for v in values]
            require(all(len(c) == 3 for c in corners), 'Complete OBJ UV/normal indices')
            faces.append(corners)
    require(vertices and len(vertices) == len(uv) == len(normals) and faces, 'OBJ arrays')
    require(max(abs(v) for p in vertices for v in p) < 10, 'Expected fixed metre-scale original fixture')
    edges, adjacent, minimum = collections.Counter(), collections.defaultdict(set), D('Infinity')
    for face in faces:
        require(all(1 <= c[0] <= len(vertices) and 1 <= c[1] <= len(uv) and 1 <= c[2] <= len(normals) for c in face), 'Valid positive OBJ indices')
        ids = [c[0] - 1 for c in face]
        require(len(set(ids)) == 3, 'Repeated OBJ triangle corner')
        a, b, c = [vertices[i] for i in ids]
        x, y = [q - p for p, q in zip(a, b)], [q - p for p, q in zip(a, c)]
        cross = (x[1] * y[2] - x[2] * y[1], x[2] * y[0] - x[0] * y[2], x[0] * y[1] - x[1] * y[0])
        twice_area = sum(v * v for v in cross).sqrt()
        require(twice_area > 0, 'Positive 3D OBJ triangle area')
        require(all(sum(v * n for v, n in zip(cross, normals[corner[2] - 1])) > 0 for corner in face), 'OBJ normals aligned to winding')
        p, q, r = [uv[corner[1] - 1] for corner in face]
        require((q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]) > 0, 'Positive OBJ UV triangle area')
        minimum = min(minimum, twice_area)
        for a, b in zip(ids, ids[1:] + ids[:1]):
            edges[tuple(sorted((a, b)))] += 1
            adjacent[a].add(b); adjacent[b].add(a)
    require(all(abs(sum(v * v for v in n).sqrt() - 1) <= TOL for n in normals), 'Unit OBJ normals')
    require(len(adjacent) == len(vertices) and all(n in (1, 2) for n in edges.values()), 'Used manifold-indexed vertices')
    unseen, components = set(adjacent), 0
    while unseen:
        components += 1
        todo = [unseen.pop()]
        while todo:
            for neighbor in adjacent[todo.pop()]:
                if neighbor in unseen:
                    unseen.remove(neighbor); todo.append(neighbor)
    require(components == 2, 'Two original rest-piece components')
    return {'vertices': len(vertices), 'uvs': len(uv), 'normals': len(normals), 'triangles': len(faces),
            'edges': len(edges), 'components': components, 'euler_characteristic_sum': len(vertices) - len(edges) + len(faces),
            'bounds_m': [[min(p[i] for p in vertices), max(p[i] for p in vertices)] for i in range(3)],
            'minimum_twice_triangle_area_m2': minimum, 'finite_positive_faces_and_uvs': True, 'aligned_unit_normals': True}


def main():
    declaration, observation = load(ROOT / 'cancel-declaration.json'), load(ROOT / 'cancel-observation.json')
    original, prepared, before, after = [load(ROOT / name) for name in ('original-shirt.weftforge', 'init-cancel-input.weftforge', 'init-before.weftforge', 'init-after.weftforge')]
    require(sha(ROOT / 'original-shirt.weftforge') == declaration['derived_from_sha256'], 'Original source SHA')
    require(sha(ROOT / 'init-cancel-input.weftforge') == declaration['input_sha256'], 'Prepared input SHA')
    expected = copy.deepcopy(original); expected['project']['resolution'] = D('.015')
    require(original['project']['resolution'] == D('.12') and prepared == expected, 'Prepared fixture changed fields other than resolution .12 to .015')
    expected_save = migrated_v2(prepared)
    source_comparison = source_compare(expected_save, before)
    require(after == before, 'Cancellation changed native document or revision')
    snapshot_equal((ROOT / 'init-before.weftforge').read_bytes(), (ROOT / 'init-after.weftforge').read_bytes())
    snapshot_equal((ROOT / 'init-before.obj').read_bytes(), (ROOT / 'init-after.obj').read_bytes())
    text = (ROOT / 'init-before.obj').read_text()
    mesh = mesh_check(text)
    changed = copy.deepcopy(before); changed['project']['materials'][0]['weft_stiffness'] += 1
    witnesses = {'unintended_migration_material_change': rejected(lambda value: source_compare(expected_save, value), changed),
                 'same_json_but_changed_snapshot_bytes': rejected(lambda value: snapshot_equal((ROOT / 'init-before.weftforge').read_bytes(), value), (ROOT / 'init-after.weftforge').read_bytes() + b' ')}
    changed_point = copy.deepcopy(before); changed_point['project']['points'][0]['position'][0] += D('.000001')
    witnesses['source_point_exceeds_fixed_tolerance'] = rejected(lambda value: source_compare(expected_save, value), changed_point)
    lines = text.splitlines(); i = next(i for i, line in enumerate(lines) if line.startswith('f '))
    face = lines[i].split(); face[-1] = face[1]; lines[i] = ' '.join(face)
    witnesses['obj_repeated_corner'] = rejected(mesh_check, '\n'.join(lines))
    require(observation['executable_sha256'] == declaration['executable_sha256'] and observation['source_commit'] == declaration['source_commit'], 'Cancellation identity records')
    names = ['original-shirt.weftforge', 'init-cancel-input.weftforge', 'init-before.weftforge', 'init-before.obj',
             'init-after.weftforge', 'init-after.obj', 'cancel-declaration.json', 'cancel-observation.json']
    result = {'scenario': 'UI-BATCH-CANCEL-001', 'kind': 'implementation_only', 'status': 'passed_saved_snapshot_cancel_checks',
              'evaluator_sha256': sha(Path(__file__)), 'candidate_executable_sha256': declaration['executable_sha256'],
              'prepared_source': {'raw_only_change': 'project.resolution .12 -> .015', 'other_raw_source_fields_identical': True},
              'native_save_migration': {'input_version': 1, 'saved_version': 2, 'explicit_changes': ['version 1 -> 2',
                   'pieces gain grain_angle 0', 'stretch_stiffness copied to warp_stiffness and weft_stiffness'],
                   'source_field_comparison': source_comparison, 'input_and_saved_bytes_equal': False},
              'cancel_preservation': {'native_bytes_identical': True, 'obj_bytes_identical': True, 'revision_before': before['project']['revision'],
                   'revision_after': after['project']['revision'], 'native_shared_sha256': sha(ROOT / 'init-before.weftforge'),
                   'obj_shared_sha256': sha(ROOT / 'init-before.obj')},
              'mesh': mesh, 'negative_witnesses': witnesses,
              'parent_ui_observation': {'acknowledged': observation['acknowledged'], 'reported_ui_ack_ms': observation['cancel_click_to_observed_ack_ms'],
                   'timing_basis': 'Parent native click plus subsequent AX acknowledgement; includes CUA/AX overhead.'},
              'artifacts': {name: {'sha256': sha(ROOT / name), 'bytes': (ROOT / name).stat().st_size} for name in names},
              'limits': ['Exact initialization versus step phase and worker-thread termination latency are unmeasured.',
                        'UI acknowledgement and Rest arrangement observations are parent evidence; this checker verifies retained bytes and geometry only.',
                        'No constructor-only latency, live collision speed, garment quality, reference contract or parity claim.',
                        'Prepared original fixture is confined to this separate subcase, not empty-project authoring evidence.']}
    output = ROOT / 'independent-cancellation-check.json'; output.write_text(json.dumps(result, indent=2, default=str) + '\n')
    print(json.dumps({'status': result['status'], 'report': output.name, 'report_sha256': sha(output), 'evaluator_sha256': result['evaluator_sha256']}))


if __name__ == '__main__':
    main()
