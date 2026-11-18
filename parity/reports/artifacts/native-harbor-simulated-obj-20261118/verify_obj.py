"""Independent standard-library native OBJ comparison; no WeftForge imports."""
import hashlib
import json
import math
from pathlib import Path
import time

HERE = Path(__file__).resolve().parent
START = time.monotonic()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sub(a, b): return tuple(x-y for x, y in zip(a, b))
def dot(a, b): return math.fsum(x*y for x, y in zip(a, b))
def norm(a): return math.sqrt(dot(a, a))
def cross(a, b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def parse(path):
    assert path.stat().st_size < 1024**2
    vertices, uv, normals, faces, groups = [], [], [], [], []
    group = None
    raw = path.read_text()
    assert raw.splitlines()[0] == '# WeftForge OBJ: metres; Y up; material UV coordinates in metres'
    for line in raw.splitlines():
        values = line.split()
        if not values or values[0] == '#':
            continue
        tag = values[0]
        assert tag in ('v', 'vt', 'vn', 'g', 'f'), tag
        if tag in ('v', 'vt', 'vn'):
            row = tuple(map(float, values[1:]))
            assert len(row) == (2 if tag == 'vt' else 3) and all(map(math.isfinite, row))
            {'v': vertices, 'vt': uv, 'vn': normals}[tag].append(row)
        elif tag == 'g':
            assert len(values) == 2 and values[1].startswith('piece_')
            group = values[1]
            groups.append(group)
        else:
            assert group is not None and len(values) == 4
            face = tuple(tuple(map(int, corner.split('/'))) for corner in values[1:])
            assert all(len(corner) == 3 for corner in face)
            faces.append((group, face))
    assert len(vertices) == len(uv) == len(normals) == 654 and len(faces) == 1172
    assert groups == ['piece_2', 'piece_31']
    assert all(1 <= v <= len(vertices) and 1 <= u <= len(uv) and 1 <= n <= len(normals)
               for _, face in faces for v, u, n in face)
    assert all(v == u == n for _, face in faces for v, u, n in face)
    unit_error = max(abs(dot(n, n)-1) for n in normals)
    assert unit_error < 1e-5
    min_face_area, min_winding_alignment = math.inf, math.inf
    for _, face in faces:
        a, b, c = (vertices[v-1] for v, _, _ in face)
        normal = cross(sub(b, a), sub(c, a))
        area = norm(normal)/2
        assert area > 1e-20
        min_face_area = min(min_face_area, area)
        min_winding_alignment = min(min_winding_alignment, *(dot(normal, normals[n-1])/(2*area) for _, _, n in face))
    assert min_winding_alignment > 0.
    return {'vertices': vertices, 'uv': uv, 'normals': normals, 'faces': faces, 'groups': groups,
            'certificate': {'vertices': len(vertices), 'uv': len(uv), 'normals': len(normals),
                'triangles': len(faces), 'groups': groups, 'max_normal_squared_unit_error': unit_error,
                'minimum_triangle_area_m2': min_face_area, 'minimum_normal_winding_cosine': min_winding_alignment,
                'bounds_m': [[min(v[i] for v in vertices), max(v[i] for v in vertices)] for i in range(3)],
                'finite_data_and_positive_valid_indices': True}}


def main():
    project_path = HERE / 'inputs/Harbor.weftforge'
    assert sha(project_path) == '6e8d3dcad320d3570719cb62417af1c4efb003fe30940938b16a484b44441e1f'
    project = json.loads(project_path.read_text())['project']
    assert project['revision'] == 23 and len(project['pieces']) == 2 and len(project['seams']) == 8
    assert project['materials'][0]['warp_stiffness'] == 520.
    rest, simulated = (parse(HERE / 'artifacts' / name) for name in ('rest.obj', 'simulated.obj'))
    assert rest['faces'] == simulated['faces'] and rest['uv'] == simulated['uv'] and rest['groups'] == simulated['groups']
    # Original Harbor has zero rotations: every rest point must be its metre UV plus its piece placement.
    placements = {'piece_'+str(p['id']): p['placement'] for p in project['pieces']}
    assert all(p['rotation'] == [0., 0., 0.] for p in placements.values())
    assigned, placement_error = {}, 0.
    for group, face in rest['faces']:
        tx, ty, tz = placements[group]['translation']
        for vertex, uv_index, _ in face:
            assert vertex not in assigned or assigned[vertex] == group
            assigned[vertex] = group
            u, v = rest['uv'][uv_index-1]
            placement_error = max(placement_error, norm(sub(rest['vertices'][vertex-1], (u+tx, v+ty, tz))))
    assert len(assigned) == 654 and placement_error <= 1e-12
    displacements = [norm(sub(a, b)) for a, b in zip(simulated['vertices'], rest['vertices'])]
    changed = sum(a != b for a, b in zip(simulated['vertices'], rest['vertices']))
    assert changed > 0 and max(displacements) > 1e-9
    output = {'status': 'PASS_one_native_simulated_OBJ_path_only',
        'input_sha256': sha(project_path), 'native_document_revision_observed_before_and_after': 23,
        'revision_embedded_in_OBJ': False,
        'revision_attribution': 'Unchanged versioned editable project and retained native AX observations; OBJ has no revision field.',
        'source_project_unchanged': True,
        'rest_obj_sha256': sha(HERE / 'artifacts/rest.obj'),
        'simulated_obj_sha256': sha(HERE / 'artifacts/simulated.obj'),
        'rest': rest['certificate'], 'simulated': simulated['certificate'],
        'topology_uvs_and_piece_groups_exactly_preserved': True,
        'rest_axis_and_metre_placement_max_error_m': placement_error,
        'changed_vertex_count': changed, 'max_displacement_m': max(displacements),
        'rms_displacement_m': math.sqrt(math.fsum(d*d for d in displacements)/len(displacements)),
        'changed_normal_count': sum(a != b for a, b in zip(rest['normals'], simulated['normals'])),
        'position_samples_m': [{'vertex_zero_based': i, 'rest': rest['vertices'][i],
                               'simulated': simulated['vertices'][i]} for i in (0, 42, 108, 653)],
        'parser_uses_production_code': False,
        'repository_verifier_note': 'scripts/check_exports.py provided the standard OBJ checks; its fixed original-shirt bounds and required SVG are inapplicable to this Harbor pair, so this independent small parser uses the actual authored placements.',
        'full_garment_fit_or_reference_or_gate_acceptance': False,
        'elapsed_seconds': time.monotonic()-START}
    with (HERE / 'EVALUATION.json').open('x') as stream:
        json.dump(output, stream, indent=2, allow_nan=False)
        stream.write('\n')
    print(json.dumps({'status': output['status'], 'changed_vertices': changed,
        'max_displacement_m': output['max_displacement_m'], 'rms_displacement_m': output['rms_displacement_m'],
        'triangles': len(rest['faces']), 'placement_error_m': placement_error,
        'evaluation_sha256': sha(HERE / 'EVALUATION.json')}, sort_keys=True))


if __name__ == '__main__':
    main()
