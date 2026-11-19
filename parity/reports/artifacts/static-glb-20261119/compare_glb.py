#!/usr/bin/env python3
"""Compare an original WeftForge scene's GLB to separately parsed OBJ and authored metadata."""
from collections import defaultdict
from pathlib import Path
import hashlib
import json
import math
import struct
import sys


def read_obj(path):
    attributes = {'v': [], 'vt': [], 'vn': []}
    faces = defaultdict(list)
    group = None
    for line in path.read_text().splitlines():
        parts = line.split()
        if not parts:
            continue
        label, *values = parts
        if label in attributes:
            attributes[label].append(tuple(map(float, values)))
        if label == 'g':
            assert values[0].startswith('piece_')
            group = values[0][len('piece_'):]
        if label == 'f':
            assert group is not None and len(values) == 3
            triplets = [tuple(map(int, value.split('/'))) for value in values]
            assert all(v == t == n and v > 0 for v, t, n in triplets)
            faces[group].append(tuple(v - 1 for v, _, _ in triplets))
    return attributes, faces


def compare(glb, obj, project_path):
    source = project_path.read_bytes()
    project = json.loads(source)['project']
    pieces = {str(piece['id']): piece for piece in project['pieces']}
    fabrics = {str(fabric['id']): fabric for fabric in project['materials']}
    attributes, faces = read_obj(obj)
    raw = glb.read_bytes()
    assert struct.unpack_from('<4sII', raw) == (b'glTF', 2, len(raw))
    json_size, header = struct.unpack_from('<I4s', raw, 12)
    assert header == b'JSON' and json_size % 4 == 0
    document = json.loads(raw[20:20 + json_size])
    bin_size, header = struct.unpack_from('<I4s', raw, 20 + json_size)
    matrix = raw[28 + json_size:]
    assert header == b'BIN\0' and len(matrix) == bin_size and bin_size % 4 == 0
    assert document['asset']['version'] == '2.0'
    assert document['asset']['extras']['weftforgeSourceRevision'] == str(project['revision'])
    assert document['buffers'] == [{'byteLength': len(matrix)}]
    observed = set()
    counts = {'vertices': 0, 'triangles': 0}
    rounding = {'position_metres': 0., 'normal': 0., 'uv': 0.}
    used_materials = set()

    def values(index, kind, vector_count, type_id):
        accessor = document['accessors'][index]
        assert accessor['componentType'] == type_id
        view = document['bufferViews'][accessor['bufferView']]
        assert view['buffer'] == 0 and view['byteOffset'] % 4 == 0
        start = view['byteOffset'] + accessor.get('byteOffset', 0)
        width = 4 * vector_count
        end = start + width * accessor['count']
        assert start >= 0 and end <= len(matrix)
        assert end - start == view['byteLength']
        return list(struct.iter_unpack('<' + kind * vector_count, matrix[start:end]))

    for node_id in document['scenes'][document['scene']]['nodes']:
        node = document['nodes'][node_id]
        assert not any(key in node for key in ('matrix', 'translation', 'rotation', 'scale'))
        mesh = document['meshes'][node['mesh']]
        piece_id = mesh['extras']['weftforgePieceId']
        piece = pieces[piece_id]
        assert piece_id not in observed and mesh['name'] == node['name'] == piece['name']
        observed.add(piece_id)
        assert len(mesh['primitives']) == 1
        primitive = mesh['primitives'][0]
        assert primitive.get('mode', 4) == 4
        property_ids = primitive['attributes']
        vertices = values(property_ids['POSITION'], 'f', 3, 5126)
        uv = values(property_ids['TEXCOORD_0'], 'f', 2, 5126)
        normals = values(property_ids['NORMAL'], 'f', 3, 5126)
        indices = [value[0] for value in values(primitive['indices'], 'I', 1, 5125)]
        assert len(vertices) == len(uv) == len(normals)
        assert len(indices) % 3 == 0 and all(i < len(vertices) for i in indices)
        assert all(math.isfinite(c) for vec in vertices + uv + normals for c in vec)
        assert all(abs(math.hypot(*n) - 1) < 1e-6 for n in normals)
        box = document['accessors'][property_ids['POSITION']]
        assert box['min'] == list(map(min, zip(*vertices)))
        assert box['max'] == list(map(max, zip(*vertices)))
        expected_indices = sorted({i for face in faces[piece_id] for i in face})
        assert len(expected_indices) == len(vertices)
        assert [expected_indices[i] for i in indices] == [i for face in faces[piece_id] for i in face]
        for original, local in zip(expected_indices, range(len(vertices))):
            expected_v = attributes['v'][original]
            expected_n = attributes['vn'][original]
            u, v = attributes['vt'][original]
            rounding['position_metres'] = max(rounding['position_metres'], math.dist(expected_v, vertices[local]))
            rounding['normal'] = max(rounding['normal'], math.dist(expected_n, normals[local]))
            rounding['uv'] = max(rounding['uv'], math.dist((u, -v), uv[local]))
        for a, b, c in zip(indices[0::3], indices[1::3], indices[2::3]):
            e = tuple(y - x for x, y in zip(vertices[a], vertices[b]))
            f = tuple(y - x for x, y in zip(vertices[a], vertices[c]))
            cross = (e[1] * f[2] - e[2] * f[1], e[2] * f[0] - e[0] * f[2], e[0] * f[1] - e[1] * f[0])
            assert math.hypot(*cross) > 0
        material_id = str(piece['material'])
        material = document['materials'][primitive['material']]
        used_materials.add(material_id)
        assert material['extras']['weftforgeMaterialId'] == material_id
        assert material['name'] == fabrics[material_id]['name'] and material['doubleSided']
        appearance = material['pbrMetallicRoughness']
        assert appearance['metallicFactor'] == 0 and appearance['roughnessFactor'] == 1
        assert math.dist(appearance['baseColorFactor'], [*fabrics[material_id]['color'], 1.]) < 1e-7
        counts['vertices'] += len(vertices)
        counts['triangles'] += len(indices) // 3
    assert observed == pieces.keys() == faces.keys()
    assert len(used_materials) == len(document['materials'])
    assert rounding['position_metres'] < 1e-6 and rounding['uv'] < 1e-6 and rounding['normal'] < 1e-6
    return {
        'fixture_sha256': hashlib.sha256(source).hexdigest(),
        'glb_sha256': hashlib.sha256(raw).hexdigest(),
        'comparison_obj_sha256': hashlib.sha256(obj.read_bytes()).hexdigest(),
        'source_revision': project['revision'],
        'pieces': {i: pieces[i]['name'] for i in sorted(observed)},
        'materials': len(used_materials),
        **counts,
        'maximum_rounding': rounding,
        'winding_indices_preserved': True,
    }


if __name__ == '__main__':
    glb_path, obj_path, source_path = map(Path, sys.argv[1:])
    print(json.dumps(compare(glb_path, obj_path, source_path), indent=2, ensure_ascii=False))
