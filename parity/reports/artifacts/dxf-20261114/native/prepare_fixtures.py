#!/usr/bin/env python3
"""Independent source-derived acceptance inputs; does not import WeftForge."""
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = Path('parity/reports/artifacts/seam-reversal-20261114/native/restored.weftforge')
EXPECTED = 'a240c2ae33b3d933872201817bba2f17ad4fae3fe898d2141f3ed8e0357f9a1d'
raw = SOURCE.read_bytes()
assert sha256(raw).hexdigest() == EXPECTED
original = json.loads(raw)
assert original['project']['revision'] == 19
(ROOT / 'input' / 'original-restored.weftforge').write_bytes(raw)
SHIFT = [-0.12345678901234568, -0.23456789012345678]
NAMES = {2: 'Épreuve A <>/\\":;?*|=', 11: '裁断 B <>/\\":;?*|='}
PLACEMENTS = {
    2: {'translation': [0.51, -0.37, 0.29], 'rotation': [0.19, -0.31, 0.47]},
    11: {'translation': [-0.62, 0.43, -0.28], 'rotation': [-0.23, 0.41, -0.59]},
}


def write(name, doc):
    assert doc['project']['revision'] == 20
    path = ROOT / 'input' / f'{name}.weftforge'
    path.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    return {'path': f'input/{path.name}', 'sha256': sha256(path.read_bytes()).hexdigest(),
            'revision': doc['project']['revision'], 'piece_ids_in_array_order': [p['id'] for p in doc['project']['pieces']]}


mixed = deepcopy(original)
p = mixed['project']
p['revision'] += 1
for point in p['points']:
    point['position'] = [value + shift for value, shift in zip(point['position'], SHIFT)]
for curve in p['curves']:
    if curve['controls'] is not None:
        curve['controls'] = [[value + shift for value, shift in zip(control, SHIFT)] for control in curve['controls']]
for piece in p['pieces']:
    piece['name'] = NAMES[piece['id']]
    piece['placement'] = PLACEMENTS[piece['id']]
p['pieces'].reverse()
assert [p['id'] for p in p['pieces']] == [11, 2]
assert sum(c['controls'] is not None for c in p['curves']) == 1
fixtures = {'mixed': write('mixed', mixed)}

native_expected = deepcopy(mixed)
native_expected['project']['revision'] = 21
native_expected['project']['resolution'] = 0.045
path = ROOT / 'input' / 'native-expected.weftforge'
path.write_text(json.dumps(native_expected, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
fixtures['native-expected'] = {'path': 'input/native-expected.weftforge', 'sha256': sha256(path.read_bytes()).hexdigest(), 'revision': 21, 'piece_ids_in_array_order': [11, 2]}

# This independent branch changes only placement and mesh resolution beyond mixed's edits.
variant = deepcopy(mixed)
variant['project']['resolution'] = 0.08
for piece in variant['project']['pieces']:
    piece['placement'] = {'translation': [-1.1, 1.3, -1.7], 'rotation': [0.7, -0.9, 1.1]}
fixtures['mixed-placement-resolution'] = write('mixed-placement-resolution', variant)

rectangle = deepcopy(original)
p = rectangle['project']
p['revision'] += 1
p['points'] = [{'id': i, 'position': xy} for i, xy in zip([12, 13, 14, 15], [[0.0, 0.0], [0.125, 0.0], [0.125, 0.25], [0.0, 0.25]])]
p['curves'] = [c for c in p['curves'] if c['id'] in [16, 17, 18, 19]]
p['pieces'] = [piece for piece in p['pieces'] if piece['id'] == 11]
p['pieces'][0]['name'] = NAMES[11]
p['pieces'][0]['placement'] = PLACEMENTS[11]
p['seams'] = []
fixtures['rectangle'] = write('rectangle', rectangle)

empty = deepcopy(original)
p = empty['project']
p['revision'] += 1
for field in ['points', 'curves', 'pieces', 'seams']:
    p[field] = []
fixtures['empty'] = write('empty', empty)

manifest = {
    'contract': 'FILE-005-v1', 'reviewed_spec_commit': 'c69e377',
    'reviewed_spec': 'spec/features/FILE-005-dxf-pattern-export.md',
    'original': {'path': str(SOURCE), 'sha256': EXPECTED, 'revision': 19},
    'operations': [
        'Copy original bytes unchanged to input/original-restored.weftforge.',
        'For mixed, parse JSON numbers as Python binary64; add the stated binary64 shift once to every authored point position and both controls of every cubic, coordinate by coordinate. Do not translate 3D placements, mannequin or seams.',
        'For mixed, replace piece names and placements by stable ID as stated; reverse only the pieces array; advance revision from 19 to 20 exactly once for all edits; retain all other values.',
        'mixed-placement-resolution is a separate revision-20 branch from the same original: same geometry/names/order as mixed, all placements replaced by translation [-1.1, 1.3, -1.7] and rotation [0.7, -0.9, 1.1], resolution 0.08. No second revision increment.',
        'rectangle is a separate revision-20 branch: keep only point IDs 12..15 with UV [0,0],[0.125,0],[0.125,0.25],[0,0.25], curves 16..19 and piece 11; set piece 11 name/placement as stated; remove seams. Known physical width 125 mm and height 250 mm.',
        'empty is a separate revision-20 branch: empty points, curves, pieces and seams; keep other original values.',
        'native-expected is a declared UI expectation, derived from mixed: change only resolution to binary64 0.045 and revision 20 to 21. A pending mesh-resolution field edit must commit once before native ExportDXF.'
    ],
    'mixed_shift_metres': SHIFT, 'mixed_names': NAMES, 'mixed_placements': PLACEMENTS,
    'fixtures': fixtures,
    'expected_outputs': {
        'native': ['native/source-before.weftforge', 'native/source-after-cancel.weftforge', 'native/source-after.weftforge', 'native/source-after-repeat.weftforge', 'native/reopened.weftforge', 'native/native-mixed.dxf', 'native/native-mixed-repeat.dxf', 'native/native-cancel-sentinel.dxf'],
        'headless': ['headless/mixed.dxf', 'headless/mixed-repeat.dxf', 'headless/mixed-placement-resolution.dxf', 'headless/rectangle.dxf', 'headless/empty.dxf']
    },
    'scope': 'Only FILE-005-v1 source-derived profile acceptance; no reference parity, import, garment round trip, other CAD, printing or scale-on-print claim.'
}
(ROOT / 'input' / 'manifest.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
print(json.dumps(manifest['fixtures'], indent=2))
