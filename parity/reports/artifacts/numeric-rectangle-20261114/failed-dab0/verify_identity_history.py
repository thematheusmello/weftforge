#!/usr/bin/env python3
"""Bounded exact checks of rectangle additions, actual curved state and grain history."""
import copy
from decimal import Decimal as D, getcontext
import hashlib
import json
from pathlib import Path

getcontext().prec = 70
ROOT = Path(__file__).parent
TOL = D('1e-12')
NAMES = ['empty.weftforge', 'numeric-after-errors.weftforge', 'identity-a.weftforge',
         'identity-b.weftforge', 'identity-curved.weftforge', 'focused-undone.weftforge', 'focused-redone.weftforge']


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def expected_add(before, ident, anchor_x, name):
    result = copy.deepcopy(before)
    project = result['project']
    project['revision'] += 1
    project['next_id'] = ident + 9
    positions = [(anchor_x, D(0)), (anchor_x + D('.4'), D(0)),
                 (anchor_x + D('.4'), D('.6')), (anchor_x, D('.6'))]
    project['points'].extend({'id': ident + 1 + i, 'position': list(p)} for i, p in enumerate(positions))
    project['curves'].extend({'id': ident + 5 + i, 'start': ident + 1 + i,
                              'end': ident + 1 + (i + 1) % 4, 'controls': None} for i in range(4))
    project['pieces'].append({'id': ident, 'name': name, 'boundary': list(range(ident + 5, ident + 9)),
                              'placement': {'translation': [0, 0, 0], 'rotation': [0, 0, 0]},
                              'material': 1, 'grain_angle': 0})
    return result


def check_focus(curved, undone, redone):
    expected = copy.deepcopy(curved)
    expected['project']['revision'] += 2
    require(undone == expected, 'Focused undo/blur changed data beyond draft-commit/undo revision')
    expected['project']['revision'] += 1
    next(p for p in expected['project']['pieces'] if p['id'] == 20)['grain_angle'] = D('.25')
    require(redone == expected, 'Focused redo changed fields other than intended B grain and revision')


def main():
    docs = {name: json.loads((ROOT / name).read_text(), parse_float=D) for name in NAMES}
    initial, a, b, curved, undone, redone = (docs[name] for name in NAMES[1:])
    require(a == expected_add(initial, 11, D(1), 'Identity A'), 'Identity A differs from exact declared rectangle addition')
    require(b == expected_add(a, 20, D(2), 'Identity B'), 'Identity B differs from exact declared rectangle addition')
    curve = next(c for c in curved['project']['curves'] if c['id'] == 27)
    require(curve['start'] == 23 and curve['end'] == 24 and curve['controls'], 'Cubic relationships')
    actual_controls = curve['controls']
    require([p[1] for p in actual_controls] == [D('.8'), D('.8')], 'Intended cubic control heights')
    expected_curved = copy.deepcopy(b)
    expected_curved['project']['revision'] = curved['project']['revision']
    next(c for c in expected_curved['project']['curves'] if c['id'] == 27)['controls'] = copy.deepcopy(actual_controls)
    require(curved == expected_curved, 'Curve edit changed unrelated native fields')
    expected_x = [D(34) / 15, D(32) / 15]
    errors = [abs(p[0] - wanted) for p, wanted in zip(actual_controls, expected_x)]
    thirds_preserved = all(error <= TOL for error in errors)
    check_focus(curved, undone, redone)
    changed = copy.deepcopy(undone)
    next(p for p in changed['project']['pieces'] if p['id'] == 20)['grain_angle'] = D('.25')
    try:
        check_focus(curved, changed, redone)
    except AssertionError as error:
        witness = {'status': 'rejected', 'reason': str(error)}
    else:
        raise AssertionError('Negative witness accepted focus-blur grain mutation')
    for name, doc in docs.items():
        require(doc['project']['materials'] == docs['empty.weftforge']['project']['materials'], 'Material drift in ' + name)
    result = {'scenario': 'UI-NUMERIC-RECTANGLE-001', 'kind': 'implementation_only',
              'evaluator_sha256': sha(Path(__file__)), 'status': 'passed' if thirds_preserved else 'partial_pass_with_control_precision_defect',
              'rectangle_additions': {'a_revision': a['project']['revision'], 'b_revision': b['project']['revision'],
                  'expected_size_m': [D('.4'), D('.6')], 'expected_anchors_m': [[1, 0], [2, 0]],
                  'exact_native_addition_checks': 'passed', 'original_numeric_and_other_fields_preserved': True},
              'curve27': {'revision': curved['project']['revision'], 'revision_increase_from_straight_b': curved['project']['revision'] - b['project']['revision'],
                  'endpoints_m': [[D('2.4'), D('.6')], [D(2), D('.6')]], 'actual_controls_m': actual_controls,
                  'expected_control_x_rationals_m': ['34/15', '32/15'], 'expected_control_x_decimal_m': expected_x,
                  'control_x_absolute_errors_m': errors, 'authoring_tolerance_m': TOL,
                  'unchanged_standard_third_x': thirds_preserved, 'only_changed_native_fields': ['revision', 'curves[id=27].controls'],
                  'failure': None if thirds_preserved else 'Unedited P2 X is 2.13333 instead of the standard third; do not normalize this value.'},
              'focused_undo': {'revision': undone['project']['revision'], 'equals_actual_curved_except_revision': True, 'all_grains_zero': True},
              'focused_redo': {'revision': redone['project']['revision'], 'only_intended_b_grain_and_revision_changed': True,
                  'b_grain_angle_rad': D('.25'), 'actual_quantized_curve_controls_preserved': True},
              'materials_identical_to_empty_in_all_checkpoints': True, 'negative_witness_focus_blur_mutates_grain': witness,
              'artifacts': {name: {'sha256': sha(ROOT / name), 'bytes': (ROOT / name).stat().st_size,
                                  'revision': doc['project']['revision']} for name, doc in docs.items()},
              'limitations': ['Inspector creation-field text persistence and focus/blur gestures are parent-observed UI evidence.',
                              'The observed standard-third control precision expectation fails for this identified candidate.',
                              'No reference parity, cloth fitting or rendering quality claim.']}
    result['portable_derivative'] = {'original_evaluator_sha256': '884e22a442efc1f1ff6754247e653ad9d253ed4c91cefdd6afb7b384b13c021b', 'note': 'Bundle-relative fixtures; process metadata omitted; fixture bytes and tolerances unchanged.'}
    output = ROOT / 'independent-identity-history-check.json'
    output.write_text(json.dumps(result, indent=2, default=str) + '\n')
    print(json.dumps({'status': result['status'], 'report': output.name, 'report_sha256': sha(output),
                      'evaluator_sha256': result['evaluator_sha256']}))
    raise SystemExit(0 if thirds_preserved else 1)


if __name__ == '__main__':
    main()
