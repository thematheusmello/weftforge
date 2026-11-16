#!/usr/bin/env python3
"""Independent, single-run D003 file checker; never imports application code."""
import copy
from datetime import datetime, timezone
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import stat
import sys

HERE = Path(__file__).resolve().parent
CASE = HERE.parent / 'native-point-release-20261116' / 'D003'
SEED_SHA = 'ba14733d505c8f9ee0bc71b0d38c7a3236c31ba25e9640ae6dd77d4124040077'
TARGETS_SHA = '7ce9327027ac4b48a9ae5435e447eff9ae3a6d942c5607f653d8e24d3fc205ac'
TOL = Decimal('1e-12')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def strict_json(data):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError('duplicate JSON key: ' + repr(key))
            result[key] = value
        return result

    def constant(value):
        raise ValueError('nonfinite JSON number: ' + value)

    result = json.loads(data.decode('utf-8'), object_pairs_hook=pairs,
                        parse_float=Decimal, parse_constant=constant)

    def finite(value):
        if type(value) in (int, Decimal):
            try:
                valid = math.isfinite(float(value))
            except (OverflowError, ValueError):
                valid = False
            if not valid:
                raise ValueError('JSON number is not finite in f64')
        elif isinstance(value, dict):
            for item in value.values():
                finite(item)
        elif isinstance(value, list):
            for item in value:
                finite(item)

    finite(result)
    return result


def is_position(path):
    return (len(path) == 5 and path[:2] == ('project', 'points')
            and type(path[2]) is int and path[3] == 'position'
            and path[4] in (0, 1))


def differences(actual, expected, path=()):
    label = '/' + '/'.join(map(str, path))
    if type(expected) is dict:
        if type(actual) is not dict:
            return [label + ': expected object']
        failures = []
        if actual.keys() != expected.keys():
            failures.append(label + ': object keys differ; missing=' + repr(sorted(expected.keys() - actual.keys()))
                            + ', extra=' + repr(sorted(actual.keys() - expected.keys())))
        for key in expected:
            if key in actual:
                failures.extend(differences(actual[key], expected[key], path + (key,)))
        return failures
    if type(expected) is list:
        if type(actual) is not list:
            return [label + ': expected array']
        failures = []
        if len(actual) != len(expected):
            failures.append(label + ': array length differs')
        for index, (value, target) in enumerate(zip(actual, expected)):
            failures.extend(differences(value, target, path + (index,)))
        return failures
    if type(expected) is int:
        valid = type(actual) is int and actual == expected
    elif type(expected) is Decimal:
        valid = type(actual) in (int, Decimal)
        if valid:
            error = abs(Decimal(actual) - expected)
            valid = error <= TOL if is_position(path) else error == 0
    else:
        valid = type(actual) is type(expected) and actual == expected
    return [] if valid else [label + ': actual=' + repr(actual) + ', expected=' + repr(expected)]


def polygon_area(vertices):
    if len(vertices) < 3:
        raise ValueError('polygon requires three vertices')
    return abs(sum(a[0] * b[1] - b[0] * a[1]
                   for a, b in zip(vertices, vertices[1:] + vertices[:1]))) / 2


def piece_area(document):
    project = document['project']
    points = {point['id']: point['position'] for point in project['points']}
    curves = {curve['id']: curve for curve in project['curves']}
    pieces = [piece for piece in project['pieces'] if piece['id'] == 2]
    if len(points) != len(project['points']) or len(curves) != len(project['curves']) or len(pieces) != 1:
        raise ValueError('duplicate IDs or missing piece 2')
    piece = pieces[0]
    if piece['holes']:
        raise ValueError('expected no holes')
    boundary = [curves[curve_id] for curve_id in piece['boundary']]
    if not boundary or any(curve['controls'] is not None for curve in boundary):
        raise ValueError('expected a straight-edge boundary')
    if any(a['end'] != b['start'] for a, b in zip(boundary, boundary[1:] + boundary[:1])):
        raise ValueError('boundary is not a connected closed loop')
    return polygon_area([points[curve['start']] for curve in boundary])


def expected_state(seed, target):
    result = copy.deepcopy(seed)
    result['project']['revision'] = target['revision']
    if target['point_position_m'] is not None:
        point = next(point for point in result['project']['points'] if point['id'] == 3)
        point['position'] = list(map(Decimal, target['point_position_m']))
    return result


def inspect(document, expected, target):
    failures = differences(document, expected)
    metrics = {}
    if failures:
        return failures, metrics
    project = document['project']
    coordinates = [abs(Decimal(a) - Decimal(b))
                   for point, reference in zip(project['points'], expected['project']['points'])
                   for a, b in zip(point['position'], reference['position'])]
    area = piece_area(document)
    area_target = Decimal(target['area_m2'])
    metrics = {'revision': project['revision'], 'point_ids': [p['id'] for p in project['points']],
               'curve_ids': [c['id'] for c in project['curves']],
               'point_3_position_m': next(p['position'] for p in project['points'] if p['id'] == 3),
               'max_position_error_m': max(coordinates), 'area_m2': area,
               'area_target_m2': area_target, 'area_error_m2': abs(area - area_target),
               'all_other_authoring_fields_unchanged': True}
    if abs(area - area_target) > TOL:
        failures.append('piece 2 area exceeds declared tolerance')
    return failures, metrics


def declarations():
    seed_bytes = (HERE / 'seed.expected.json').read_bytes()
    target_bytes = (HERE / 'targets.json').read_bytes()
    if sha(seed_bytes) != SEED_SHA or sha(target_bytes) != TARGETS_SHA:
        raise ValueError('frozen seed or targets hash differs')
    return strict_json(seed_bytes), strict_json(target_bytes)


def self_check():
    seed, targets = declarations()
    tests = []
    for payload in [b'{"x":1,"x":2}', b'{"a":{"x":1,"x":2}}',
                    b'{"x":NaN}', b'{"x":Infinity}', b'{"x":-Infinity}', b'{"x":1e309}',
                    b'{"x":1} trailing', b'\xff']:
        try:
            strict_json(payload)
        except (ValueError, UnicodeError):
            pass
        else:
            raise AssertionError('invalid JSON was accepted: ' + repr(payload))
    tests.append('strict parser rejects duplicate keys, nonfinite/overflow numbers, trailing data, invalid UTF-8')
    rectangle = [[Decimal(x), Decimal(y)] for x, y in
                 [('-0.01', '-0.01'), ('0.41', '-0.01'), ('0.41', '0.61'), ('-0.01', '0.61')]]
    assert polygon_area(rectangle) == Decimal('0.2604')
    trapezoid = copy.deepcopy(rectangle)
    trapezoid[0][0] = Decimal('-0.02')
    assert polygon_area(trapezoid) == Decimal('0.2635')
    assert polygon_area(list(reversed(trapezoid))) == Decimal('0.2635')
    assert abs(piece_area(seed) - Decimal(targets['seed_area_m2'])) <= TOL
    tests.append('analytic rectangle/trapezoid shoelace areas and seed straight-edge geometry')
    for target in targets['states'].values():
        expected = expected_state(seed, target)
        failures, metrics = inspect(copy.deepcopy(expected), expected, target)
        assert not failures, failures
        assert metrics['area_error_m2'] <= TOL
    tests.append('all four independently declared revision/position/area targets')
    expected = expected_state(seed, targets['states']['edited'])
    tiny = copy.deepcopy(expected)
    tiny['project']['points'][0]['position'][0] += TOL / 2
    assert not differences(tiny, expected)
    tiny['project']['points'][0]['position'][0] += TOL * 2
    assert differences(tiny, expected)
    tests.append('position tolerance accepts half-tolerance and rejects 2.5 times tolerance')
    mutations = [
        lambda d: d['project']['materials'][0].update(areal_density=Decimal('0.3')),
        lambda d: d['project']['pieces'][0]['placement']['translation'].__setitem__(2, Decimal('0.1')),
        lambda d: d['project'].update(next_id=12),
        lambda d: d['project'].update(resolution=Decimal('0.08')),
        lambda d: d['project']['curves'][0].update(end=5),
        lambda d: d['project']['points'][1]['position'].__setitem__(0, Decimal('0.42')),
        lambda d: d['project'].update(revision=True),
        lambda d: d['project'].update(points={}),
        lambda d: d['project'].update(unexpected_field=None),
    ]
    for mutate in mutations:
        bad = copy.deepcopy(expected)
        mutate(bad)
        assert differences(bad, expected)
    assert differences([], expected)
    tests.append('full authoring comparison rejects non-position loss, ID/topology/other-point changes and malformed structure')
    broken = copy.deepcopy(expected)
    broken['project']['curves'][0]['end'] = 5
    try:
        piece_area(broken)
    except ValueError:
        pass
    else:
        raise AssertionError('disconnected geometry accepted')
    tests.append('geometry rejects a disconnected boundary')
    report = {'passed': True, 'checks': tests, 'source_seed_sha256': SEED_SHA,
              'targets_sha256': TARGETS_SHA, 'output_files_inspected': []}
    (HERE / 'self-check.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, sort_keys=True))


def capture(path):
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode):
        raise ValueError('input is not a regular file')
    data = path.read_bytes()
    after = path.lstat()
    if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_mode) != (
            after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_mode):
        raise ValueError('input changed during read')
    return {'sha256': sha(data), 'bytes': len(data), 'mode': oct(stat.S_IMODE(after.st_mode))}, data


def run_once():
    with (HERE / 'run.lock').open('x') as handle:
        handle.write(datetime.now(timezone.utc).isoformat() + '\n')
    paths = {'checker': Path(__file__), 'targets': HERE / 'targets.json',
             'frozen_seed': HERE / 'seed.expected.json', 'freeze_manifest': HERE / 'freeze.json',
             'source_seed': CASE / 'input' / 'seed.weftforge',
             'edited': CASE / 'snapshots' / 'edited.weftforge',
             'undo': CASE / 'snapshots' / 'undo.weftforge',
             'redo': CASE / 'snapshots' / 'redo.weftforge',
             'final_work': CASE / 'work' / 'point-release.weftforge'}
    report = {'started_utc': datetime.now(timezone.utc).isoformat(),
              'before': {}, 'after': {}, 'states': {}, 'relations': {}, 'failures': []}
    data = {}
    for name, path in paths.items():
        try:
            report['before'][name], data[name] = capture(path)
        except Exception as error:
            report['before'][name] = {'error': str(error)}
            report['failures'].append(name + ': ' + str(error))
    try:
        seed, targets = declarations()
        report['scope'] = targets['scope']
        report['tolerance_si'] = targets['tolerance_si']
        manifest = strict_json(data['freeze_manifest'])
        for name in ('checker', 'targets', 'frozen_seed'):
            if report['before'][name]['sha256'] != manifest['files'][name]['sha256']:
                raise ValueError(name + ': frozen manifest SHA differs')
        if sha(data['source_seed']) != SEED_SHA or data['source_seed'] != data['frozen_seed']:
            raise ValueError('supplied source seed differs from frozen independent seed')
        parsed = {}
        for name, target in targets['states'].items():
            try:
                document = strict_json(data[name])
                parsed[name] = document
                failures, metrics = inspect(document, expected_state(seed, target), target)
                report['states'][name] = {'passed': not failures, 'metrics': metrics, 'failures': failures}
                report['failures'].extend(name + ': ' + failure for failure in failures)
            except Exception as error:
                report['states'][name] = {'passed': False, 'failures': [str(error)]}
                report['failures'].append(name + ': ' + str(error))
        for name, left, right in [('undo_equals_seed_except_revision', parsed.get('undo'), seed),
                                  ('redo_equals_edited_except_revision', parsed.get('redo'), parsed.get('edited'))]:
            if left is None or right is None:
                report['relations'][name] = False
                report['failures'].append(name + ': parsed input unavailable')
                continue
            expected = copy.deepcopy(right)
            expected['project']['revision'] = left['project']['revision']
            failures = differences(left, expected)
            report['relations'][name] = not failures
            report['failures'].extend(name + ': ' + failure for failure in failures)
        equal = 'final_work' in data and 'redo' in data and data['final_work'] == data['redo']
        report['relations']['final_work_equals_redo_bytes'] = equal
        if not equal:
            report['failures'].append('final work is not byte-identical to redo')
    except Exception as error:
        report['failures'].append('checker prerequisites: ' + str(error))
    for name, path in paths.items():
        try:
            report['after'][name], _ = capture(path)
        except Exception as error:
            report['after'][name] = {'error': str(error)}
    report['all_input_hash_size_modes_unchanged'] = report['before'] == report['after']
    if not report['all_input_hash_size_modes_unchanged']:
        report['failures'].append('an input hash, size, mode or availability changed during grading')
    report['passed'] = not report['failures']
    report['finished_utc'] = datetime.now(timezone.utc).isoformat()
    (HERE / 'result.json').write_text(json.dumps(report, indent=2, sort_keys=True, default=str) + '\n')
    print(json.dumps(report, sort_keys=True, default=str))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    if sys.argv[1:] == ['--self-check']:
        self_check()
    elif sys.argv[1:] == ['--run']:
        raise SystemExit(run_once())
    else:
        raise SystemExit('usage: check_outputs.py --self-check | --run')
