"""Reproduce fixed surface counterexamples using only OBJ vertices and scalar geometry."""
from pathlib import Path
import hashlib
import json
import math

ROOT = Path(__file__).resolve().parent


def subtract(a, b):
    return [x - y for x, y in zip(a, b)]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def axis_distance(point, a, b):
    direction = subtract(b, a)
    squared = dot(direction, direction)
    fraction = max(0.0, min(1.0, dot(subtract(point, a), direction) / squared)) if squared else 0.0
    closest = [x + fraction * d for x, d in zip(a, direction)]
    return math.dist(point, closest), closest


def asset(record):
    path = ROOT / record['path']
    assert path.parent == ROOT
    contents = path.read_bytes()
    assert hashlib.sha256(contents).hexdigest() == record['sha256']
    return contents


def check(case):
    project = json.loads(asset(case['assets']['project']))['project']
    vertices, faces = [], []
    for line in asset(case['assets']['obj']).decode('utf-8').splitlines():
        fields = line.split()
        if fields and fields[0] == 'v':
            assert len(fields) == 4
            point = [float(value) for value in fields[1:]]
            assert all(math.isfinite(value) for value in point)
            vertices.append(point)
        elif fields and fields[0] == 'f':
            assert len(fields) == 4
            faces.append([int(value.split('/')[0]) - 1 for value in fields[1:]])
    assert all(0 <= index < len(vertices) for face in faces for index in face)
    triangle = faces[case['triangle_index_0based']]
    weights = case['barycentric']
    assert all(0.0 <= weight <= 1.0 for weight in weights)
    assert abs(sum(weights) - 1.0) < 1e-14
    point = [sum(weights[j] * vertices[triangle[j]][axis] for j in range(3)) for axis in range(3)]
    capsule = project['mannequin']['capsules'][case['capsule_index_0based']]
    offsets = {material['collision_thickness'] for material in project['materials']}
    assert offsets == {0.003}, 'These fixtures use a uniform physical offset'
    offset = next(iter(offsets))
    distance, axis_point = axis_distance(point, capsule['a'], capsule['b'])
    penetration = capsule['radius'] + offset - distance
    assert abs(penetration - case['expected_witness_penetration_m']) < case['static_check_tolerance_m']
    assert penetration > case['contact_screen_below_m']
    vertex_depths = [max(0.0, capsule['radius'] + offset - axis_distance(vertices[i], capsule['a'], capsule['b'])[0]) for i in triangle]
    assert max(vertex_depths) < 1e-12
    return {'case': case['case'], 'source_commit': case['source_commit'],
            'vertices': len(vertices), 'triangles': len(faces),
            'triangle_index_0based': case['triangle_index_0based'], 'triangle_vertices_0based': triangle,
            'barycentric': weights, 'surface_point_m': point, 'axis_point_m': axis_point,
            'capsule_radius_m': capsule['radius'], 'physical_offset_m': offset,
            'witness_penetration_m': penetration, 'witness_vertex_penetration_m': vertex_depths,
            'contact_screen_below_m': case['contact_screen_below_m'], 'contact_screen': 'failed',
            'scope': 'A single convex-combination point proves penetration at least this deep; no exhaustive maximum, trajectory or dynamics claim.'}


if __name__ == '__main__':
    assert axis_distance([2.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 0.0, 2.0]) == (2.0, [0.0, 0.0, 1.0])
    assert axis_distance([3.0, 4.0, 0.0], [0.0] * 3, [0.0] * 3)[0] == 5.0
    assert axis_distance([0.0, 0.0, 3.0], [0.0] * 3, [0.0, 0.0, 2.0])[0] == 1.0
    cases = json.loads((ROOT / 'cases.json').read_bytes())['cases']
    print(json.dumps({'kind': 'independently checked static output', 'cases': [check(case) for case in cases]}, indent=2))
