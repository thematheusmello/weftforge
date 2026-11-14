#!/usr/bin/env python3
"""Bounded native/SVG checks for UI-NUMERIC-RECTANGLE-002; no fitted units."""
import copy
from decimal import Decimal as D
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

ROOT = Path(__file__).parent
TOL_M = D('1e-12')


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    def invalid(value):
        raise AssertionError('Non-finite JSON: ' + value)
    return json.loads(path.read_text(), parse_float=D, parse_constant=invalid)


def native_check(empty, numeric):
    require(set(numeric) == {'format', 'version', 'project'} and numeric['format'] == 'weftforge' and numeric['version'] == 2, 'Native v2 envelope')
    a, b = empty['project'], numeric['project']
    require(a['revision'] == 0 and not any(a[k] for k in ('points', 'curves', 'pieces', 'seams')), 'Empty baseline')
    require(b['revision'] == 1 and b['next_id'] == 11, 'One creation revision and IDs')
    for key in set(a) | set(b):
        if key not in ('revision', 'next_id', 'points', 'curves', 'pieces'):
            require(a.get(key) == b.get(key), 'Unexpected baseline change: ' + key)
    expected = {3: [D(0), D(0)], 4: [D('.2'), D(0)], 5: [D('.2'), D('.3')], 6: [D(0), D('.3')]}
    points = {p['id']: p for p in b['points']}
    require(set(points) == set(expected) and len(b['points']) == 4, 'Four stable rectangle corners')
    error = D(0)
    for ident, position in expected.items():
        point = points[ident]
        require(set(point) == {'id', 'position'} and len(point['position']) == 2, 'Point schema')
        for actual, wanted in zip(point['position'], position):
            require(D(actual).is_finite(), 'Non-finite authored point')
            error = max(error, abs(actual - wanted))
    require(error <= TOL_M, 'Authored rectangle dimension or position mismatch')
    require(b['curves'] == [{'id': 7, 'start': 3, 'end': 4, 'controls': None},
                            {'id': 8, 'start': 4, 'end': 5, 'controls': None},
                            {'id': 9, 'start': 5, 'end': 6, 'controls': None},
                            {'id': 10, 'start': 6, 'end': 3, 'controls': None}], 'Straight ordered boundary')
    require(b['pieces'] == [{'id': 2, 'name': 'Pattern 1', 'boundary': [7, 8, 9, 10],
        'placement': {'translation': [0, 0, 0], 'rotation': [0, 0, 0]}, 'material': 1, 'grain_angle': 0}], 'Single placed rectangle')
    return {'revision': b['revision'], 'next_id': b['next_id'], 'points': len(points), 'curves': len(b['curves']),
            'pieces': len(b['pieces']), 'seams': len(b['seams']), 'width_m': points[4]['position'][0] - points[3]['position'][0],
            'height_m': points[5]['position'][1] - points[4]['position'][1], 'max_authored_coordinate_error_m': error,
            'resolution_m': b['resolution'], 'materials_identical_to_empty': True,
            'material_color': b['materials'][0]['color'], 'other_baseline_data_identical': True}


def svg_check(svg):
    require(svg.tag == '{http://www.w3.org/2000/svg}svg', 'SVG namespace')
    require(all('transform' not in node.attrib for node in svg.iter()), 'Unexpected SVG transform')
    require(svg.attrib['width'].endswith('mm') and svg.attrib['height'].endswith('mm'), 'SVG must declare millimetres')
    view = [D(v) for v in svg.attrib['viewBox'].split()]
    require(len(view) == 4 and all(v.is_finite() for v in view), 'Finite SVG viewBox')
    require(D(svg.attrib['width'][:-2]) == view[2] and D(svg.attrib['height'][:-2]) == view[3], 'One user unit is one millimetre')
    paths = svg.findall('{http://www.w3.org/2000/svg}path')
    require(len(paths) == 1 and paths[0].attrib['id'] == 'piece-2' and paths[0].attrib['data-name'] == 'Pattern 1', 'Editable named rectangle path')
    data = paths[0].attrib['d']
    token = r'[A-Za-z]|[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?'
    require(not re.sub(token, '', data).replace(',', '').strip(), 'Unparsed SVG path data')
    actual = [value if value.isalpha() else D(value) for value in re.findall(token, data)]
    expected = ['M', 0, 0, 'L', 200, 0, 'L', 200, -300, 'L', 0, -300, 'L', 0, 0, 'Z']
    require(actual == expected, 'SVG rectangle physical dimensions or boundary changed')
    return {'closed_editable_paths': 1, 'physical_user_unit': 'millimetre',
            'width_mm': 200, 'height_mm': 300, 'coordinate_error_mm': 0, 'pattern_y_inverted': True}


def rejected(function, value):
    try:
        function(value)
    except AssertionError as error:
        return {'status': 'rejected', 'reason': str(error)}
    raise AssertionError('Negative witness was accepted')


def main():
    empty, numeric = load(ROOT / 'empty.weftforge'), load(ROOT / 'numeric.weftforge')
    svg = ET.parse(ROOT / 'numeric.svg').getroot()
    declaration, process = load(ROOT / 'declaration.json'), load(ROOT / 'process.json')
    expected_native_sha = '3ed0ab22b4e7335c8dff9a58350b7f68eb2aa9b5ba1c39059c9043a7210c6880'
    expected_source = '359ac5b2fedfece5cfef9f22861c279bd2708801'
    require(declaration['executable_sha256'] == expected_native_sha == process['executable_sha256'], 'Candidate SHA records disagree')
    require(declaration['source_commit'] == expected_source == process['source_commit'], 'Candidate source records disagree')
    empty_bytes = (ROOT / 'empty.weftforge').read_bytes()
    for name in ('after-cancel.weftforge', 'after-errors.weftforge'):
        require((ROOT / name).read_bytes() == empty_bytes, 'Cancel/error changed accepted native bytes: ' + name)
    undone, redone = load(ROOT / 'numeric-undone.weftforge'), load(ROOT / 'numeric-redone.weftforge')
    expected_undone = copy.deepcopy(empty)
    expected_undone['project']['revision'] = 2
    expected_undone['project']['next_id'] = 11
    require(undone == expected_undone, 'Undo differs beyond expected revision/high-water allocation')
    expected_redone = copy.deepcopy(numeric)
    expected_redone['project']['revision'] = 3
    require(redone == expected_redone, 'Redo differs beyond expected revision')
    result = {'scenario': 'UI-NUMERIC-RECTANGLE-002', 'kind': 'implementation_only',
              'evaluator_sha256': sha(Path(__file__)), 'authored_tolerance_m': TOL_M,
              'candidate': {'source_commit': expected_source, 'executable_sha256': expected_native_sha,
                  'identity_basis': 'Parent-observed declaration/process records; this checker does not query native processes.'},
              'cancel_error': {'empty_revision': 0, 'after_cancel_revision': 0, 'after_errors_revision': 0,
                  'all_native_bytes_identical': True},
              'history': {'undo_revision': 2, 'redo_revision': 3, 'undone_geometry_empty': True,
                  'initial_next_id': 2, 'undone_next_id': 11, 'revision_only_empty_equality': False,
                  'undo_equals_empty_except_revision_and_allocator': True, 'redo_equals_numeric_except_revision': True},
              'native': native_check(empty, numeric), 'svg': svg_check(svg), 'negative_witnesses': {}}
    changed = copy.deepcopy(numeric)
    changed['project']['points'][1]['position'][0] += D('.001')
    result['negative_witnesses']['authored_width_perturbation_1mm'] = rejected(lambda x: native_check(empty, x), changed)
    changed = copy.deepcopy(numeric)
    changed['project']['materials'][0]['color'][0] = D('.21999997')
    result['negative_witnesses']['phantom_rgb_drift'] = rejected(lambda x: native_check(empty, x), changed)
    changed_svg = copy.deepcopy(svg)
    changed_svg.attrib['width'] = '1000cm'
    result['negative_witnesses']['incorrect_svg_physical_unit'] = rejected(svg_check, changed_svg)
    result['negative_witnesses']['cancel_preservation_byte_witness'] = rejected(
        lambda data: require(data == empty_bytes, 'Cancel/error native bytes changed'), empty_bytes + b' ')
    for label in ('bounds', 'boundary'):
        measured = load(ROOT / f'independent-numeric-{label}.json')
        require(measured['status'] == 'passed' and measured['input_sha256'] == sha(ROOT / 'numeric.obj'), 'Independent OBJ result input/status mismatch')
        require(measured['declared_export_unit'] == 'm', 'OBJ unit must remain fixed at metres')
        result['obj_' + label] = measured
    names = ['declaration.json', 'process.json', 'empty.weftforge', 'numeric.weftforge', 'numeric.obj', 'numeric.svg',
             'after-cancel.weftforge', 'after-errors.weftforge', 'numeric-undone.weftforge', 'numeric-redone.weftforge',
             'independent-numeric-bounds.json', 'independent-numeric-boundary.json']
    result['artifacts'] = {name: {'sha256': sha(ROOT / name), 'bytes': (ROOT / name).stat().st_size} for name in names}
    result['status'] = 'passed_for_supplied_initial_outputs_with_explicit_allocator_delta'
    result['pending'] = ['Dynamic field identity and exact control preservation, focused history, 5x6mm selected-size preservation, restart/reopen, separately measured initialization/cancellation']
    result['limitations'] = ['No full reference contract/parity claim; physical-dimension comparison only, with fixed units and no normalization.',
                             'No inferred world placement, reference topology/material equivalence, cloth fitting or rendering quality.',
                             'Modal state, focused fields and native gestures are parent-observed UI evidence.']
    result['portable_derivative'] = {'original_evaluator_sha256': 'd00578044086411cd4438db657bc7440696e03ef1c8921841feaa2af2b95c2d4', 'note': 'Bundle-relative paths; original fixtures and fixed criteria unchanged; raw original report retained.'}
    output = ROOT / 'independent-initial-check.json'
    output.write_text(json.dumps(result, indent=2, default=str) + '\n')
    print(json.dumps({'status': result['status'], 'report': output.name, 'report_sha256': sha(output),
                      'evaluator_sha256': result['evaluator_sha256']}))


if __name__ == '__main__':
    main()
