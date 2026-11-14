#!/usr/bin/env python3
"""Derive declared inputs from original native evidence, never native outputs."""
import copy
from decimal import Decimal
import hashlib
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT.parent.parent / 'parity/reports/artifacts/curve-join-20261114/native'
SOURCE_SHA256 = '8d4090142551f037003e3c531d524be5e80ef870f4c706a185ad1ee89e9205ae'
HELPER_SHA256 = '90148a51d1bc7f6357908f804cf7308fd4780d6219694271dd103dfc7f821350'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load(data):
    return json.loads(data, parse_float=Decimal)


def main():
    original = (SOURCE / 'fine.weftforge').read_bytes()
    assert digest(original) == SOURCE_SHA256, 'original native baseline hash changed'
    baseline = load(original)
    assert baseline['format'] == 'weftforge' and baseline['version'] == 2
    project = baseline['project']
    assert project['revision'] == 14 and project['next_id'] == 24
    assert [s['id'] for s in project['seams']] == [20, 23]
    assert len(project['pieces']) == 2
    assert all(p['placement'] == {'translation': [0, 0, 0], 'rotation': [0, 0, 0]} for p in project['pieces'])
    prefix, last = original.rsplit(b'"correspondence": [', 1)
    old_map = b'''[
          [
            0.0,
            0.0
          ],
          [
            1.0,
            1.0
          ]
        ]'''
    new_map = b'''[
          [
            0.0,
            0.0
          ],
          [
            0.2,
            0.7
          ],
          [
            1.0,
            1.0
          ]
        ]'''
    tail = b'[' + last
    assert tail.startswith(old_map)
    assert original.count(b'"revision": 14,') == 1
    nonlinear = (prefix + b'"correspondence": ' + tail.replace(old_map, new_map, 1)).replace(b'"revision": 14,', b'"revision": 15,', 1)
    expected = copy.deepcopy(baseline)
    expected['project']['revision'] = 15
    expected['project']['seams'][1]['correspondence'] = [[Decimal('0.0'), Decimal('0.0')], [Decimal('0.2'), Decimal('0.7')], [Decimal('1.0'), Decimal('1.0')]]
    assert load(nonlinear) == expected, 'derived nonlinear input changed an undeclared typed field'
    limited = nonlinear.replace(b'"revision": 15,', b'"revision": 18446744073709551615,', 1)
    expected_limit = copy.deepcopy(expected)
    expected_limit['project']['revision'] = 18446744073709551615
    assert load(limited) == expected_limit, 'revision-limit input changed an undeclared typed field'
    (ROOT / 'original-inputs').mkdir(exist_ok=True)
    (ROOT / 'original-evaluators').mkdir(exist_ok=True)
    provenance = {
        'kind': 'declared_derived_inputs_not_native_outputs',
        'source': 'parity/reports/artifacts/curve-join-20261114/native/fine.weftforge',
        'source_sha256': SOURCE_SHA256,
        'source_native_scenario': 'UI-BOUNDED-JOIN-001',
        'source_build_commit': '26dd58535868aadc90e92636656c49426e02dc9c',
        'source_provenance': 'Original GUI-authored Curve A/Curve B panels with default zero placements. See exact copied original-inputs/ui-declaration.json.',
        'asserted_baseline': {'revision': 14, 'next_id': 24, 'retained_parent_curve': 9, 'piece_ids': [2, 11], 'seam_ids': [20, 23], 'placements': 'default_zero'},
        'derived': {
            'input-nonlinear.weftforge': {'sha256': digest(nonlinear), 'bytes': len(nonlinear), 'only_typed_changes': {'project.revision': [14, 15], 'project.seams[id=23].correspondence': [[0, 0], [0.2, 0.7], [1, 1]]}, 'other_typed_fields_exact': True},
            'input-revision-limit.weftforge': {'sha256': digest(limited), 'bytes': len(limited), 'derived_from': 'input-nonlinear.weftforge', 'only_typed_changes': {'project.revision': [15, 18446744073709551615]}, 'other_typed_fields_exact': True},
        },
        'exact_original_copies': {},
        'native_outputs_created': False,
    }
    for name in ('fine.weftforge', 'fine.svg', 'ui-declaration.json'):
        target = ROOT / 'original-inputs' / name
        shutil.copyfile(SOURCE / name, target)
        assert target.read_bytes() == (SOURCE / name).read_bytes()
        provenance['exact_original_copies'][str(target.relative_to(ROOT))] = {'sha256': digest(target.read_bytes()), 'bytes': target.stat().st_size}
    helper = SOURCE / 'original-evaluators/verify_curve_outputs.py'
    assert digest(helper.read_bytes()) == HELPER_SHA256
    helper_target = ROOT / 'original-evaluators/verify_curve_outputs.py'
    shutil.copyfile(helper, helper_target)
    provenance['exact_original_copies'][str(helper_target.relative_to(ROOT))] = {'sha256': digest(helper_target.read_bytes()), 'bytes': helper_target.stat().st_size}
    (ROOT / 'input-nonlinear.weftforge').write_bytes(nonlinear)
    (ROOT / 'input-revision-limit.weftforge').write_bytes(limited)
    (ROOT / 'fixture-provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
    print(json.dumps({'status': 'passed_derived_inputs_only', 'derived': provenance['derived'], 'native_outputs_created': False}))


if __name__ == '__main__':
    main()
