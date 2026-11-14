#!/usr/bin/env python3
"""Exact decimal/Fraction nearest-ties-even oracle; no Core or native math inputs."""
import copy
from decimal import Decimal as D
from fractions import Fraction as F
import hashlib
import json
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).parent


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    def invalid(value):
        raise ValueError('Non-JSON/non-finite literal: ' + value)
    return json.loads(path.read_text(), parse_float=D, parse_constant=invalid)


def fraction_from_bits(bits, width):
    exponent_bits, fraction_bits = {32: (8, 23), 64: (11, 52)}[width]
    magnitude = bits & ((1 << (width - 1)) - 1)
    exponent = magnitude >> fraction_bits
    significand = magnitude & ((1 << fraction_bits) - 1)
    require(exponent != (1 << exponent_bits) - 1, 'Finite IEEE bits required')
    bias = (1 << (exponent_bits - 1)) - 1
    if exponent:
        significand += 1 << fraction_bits
    power = (exponent - bias if exponent else 1 - bias) - fraction_bits
    value = F(significand * (1 << power)) if power >= 0 else F(significand, 1 << -power)
    return -value if bits >> (width - 1) else value


def nearest_bits(value, width):
    if isinstance(value, D):
        require(value.is_finite(), 'Finite decimal required')
    negative = value < 0 or (isinstance(value, D) and value.is_zero() and value.is_signed())
    sign = int(negative) << (width - 1)
    value = abs(F(value))
    exponent_bits, fraction_bits = {32: (8, 23), 64: (11, 52)}[width]
    infinity = ((1 << exponent_bits) - 1) << fraction_bits
    maximum = infinity - 1
    if value > fraction_from_bits(maximum, width):
        last = fraction_from_bits(maximum, width)
        half_ulp = (last - fraction_from_bits(maximum - 1, width)) / 2
        return sign | (maximum if value < last + half_ulp else infinity)
    low, high = 0, maximum
    while low < high:
        middle = (low + high + 1) // 2
        if fraction_from_bits(middle, width) <= value:
            low = middle
        else:
            high = middle - 1
    lower = fraction_from_bits(low, width)
    if lower == value:
        return sign | low
    upper = fraction_from_bits(low + 1, width)
    down, up = value - lower, upper - value
    return sign | (low if down < up or (down == up and low % 2 == 0) else low + 1)


def exact_decimal(bits, width):
    value = fraction_from_bits(bits, width)
    if not value and bits >> (width - 1):
        return D('-0')
    denominator = value.denominator
    power = denominator.bit_length() - 1
    require(denominator == 1 << power, 'Binary rational denominator')
    coefficient = abs(value.numerator) * 5 ** power
    return D((int(value < 0), tuple(int(d) for d in str(coefficient)), -power))


def canonical_v2(source):
    doc = copy.deepcopy(source)
    require(doc['format'] == 'weftforge' and doc['version'] in (1, 2), 'Supported input schema')
    if doc['version'] == 1:
        doc['version'] = 2
        for p in doc['project']['pieces']:
            p.setdefault('grain_angle', D(0))
        for m in doc['project']['materials']:
            stiffness = m.pop('stretch_stiffness')
            m['warp_stiffness'] = stiffness
            m['weft_stiffness'] = stiffness
    return doc


def float_widths(doc):
    result = {('project', 'resolution'): 64}
    def vector(path, values, width=64):
        for i, _ in enumerate(values):
            result[(*path, i)] = width
    project = doc['project']
    for i, point in enumerate(project['points']):
        vector(('project', 'points', i, 'position'), point['position'])
    for i, curve in enumerate(project['curves']):
        if curve['controls'] is not None:
            for j, point in enumerate(curve['controls']):
                vector(('project', 'curves', i, 'controls', j), point)
    for i, piece in enumerate(project['pieces']):
        result[('project', 'pieces', i, 'grain_angle')] = 64
        for key in ('translation', 'rotation'):
            vector(('project', 'pieces', i, 'placement', key), piece['placement'][key])
    for i, material in enumerate(project['materials']):
        for key in material:
            if key == 'color':
                vector(('project', 'materials', i, key), material[key], 32)
            elif key not in ('id', 'name'):
                result[('project', 'materials', i, key)] = 64
    for i, capsule in enumerate(project['mannequin']['capsules']):
        for key in ('a', 'b'):
            vector(('project', 'mannequin', 'capsules', i, key), capsule[key])
        result[('project', 'mannequin', 'capsules', i, 'radius')] = 64
    for i, seam in enumerate(project['seams']):
        for side in ('a', 'b'):
            for key in ('t0', 't1'):
                result[('project', 'seams', i, side, key)] = 64
        for j, pair in enumerate(seam['correspondence']):
            vector(('project', 'seams', i, 'correspondence', j), pair)
    return result


def at(doc, path):
    for part in path:
        doc = doc[part]
    return doc


def put(doc, path, value):
    at(doc, path[:-1])[path[-1]] = value


def pointer(path):
    return '/' + '/'.join(str(p) for p in path)


def compare_native(source, actual):
    expected = canonical_v2(source)
    widths = float_widths(expected)
    checked = []
    def compare(e, a, path=()):
        if path in widths:
            width = widths[path]
            require(isinstance(a, (int, D)) and not isinstance(a, bool), 'Numeric field: ' + pointer(path))
            wanted, observed = nearest_bits(e, width), nearest_bits(a, width)
            require(wanted == observed, f'{pointer(path)} binary{width}: expected0x{wanted:0{width // 4}x}, actual0x{observed:0{width // 4}x}')
            checked.append(path)
        elif isinstance(e, dict):
            require(isinstance(a, dict) and set(e) == set(a), 'Object keys: ' + pointer(path))
            for key in e:
                compare(e[key], a[key], (*path, key))
        elif isinstance(e, list):
            require(isinstance(a, list) and len(e) == len(a), 'Array: ' + pointer(path))
            for i, (x, y) in enumerate(zip(e, a)):
                compare(x, y, (*path, i))
        else:
            require(not isinstance(e, D), 'Unclassified float field: ' + pointer(path))
            require(type(e) is type(a) and e == a, 'Nonfloat field: ' + pointer(path))
    compare(expected, actual)
    require(len(checked) == len(widths), 'Every expected float field checked')
    return {'float_fields_bit_exact': len(checked), 'all_nonfloat_fields_and_schema_exact': True}


def rounded_native(source):
    result = canonical_v2(source)
    for path, width in float_widths(result).items():
        put(result, path, exact_decimal(nearest_bits(at(result, path), width), width))
    return result


def reject(function):
    try:
        function()
    except AssertionError as error:
        return {'status': 'rejected', 'reason': str(error)}
    raise AssertionError('Negative witness accepted')


def prepare_expected():
    manifest = load(ROOT / 'input-manifest.json')
    cases = ['v1-original.weftforge', 'v1-extreme-stiffness.weftforge', 'v2-ieee-boundaries.weftforge']
    expected = {}
    for name in cases:
        require(sha(ROOT / name) == manifest['inputs'][name]['sha256'], 'Frozen input bytes: ' + name)
        source = load(ROOT / name)
        doc = canonical_v2(source)
        fields = []
        for path, width in float_widths(doc).items():
            literal = at(doc, path)
            bits = nearest_bits(literal, width)
            fields.append({'path': pointer(path), 'destination': f'binary{width}', 'raw_literal_or_schema_default': str(literal),
                           'expected_bits': f'0x{bits:0{width // 4}x}', 'expected_exact_decimal': str(exact_decimal(bits, width))})
        compare_native(source, rounded_native(source))
        expected[name] = {'input_sha256': sha(ROOT / name), 'native_version': 2, 'expected_revision': doc['project']['revision'], 'fields': fields}
    for width, fraction_bits, minimum_power in [(32, 23, 149), (64, 52, 1074)]:
        require(nearest_bits(F(1, 1 << minimum_power), width) == 1, 'Smallest subnormal self-check')
        require(nearest_bits(F(1, 1 << (minimum_power + 1)), width) == 0, 'Zero/subnormal tie-even self-check')
        one = 0x3f800000 if width == 32 else 0x3ff0000000000000
        require(nearest_bits(1 + F(1, 1 << (fraction_bits + 1)), width) == one, 'Normal tie-even self-check')
    literals = ['1.1199999999999999', '1.0999999999999999', '99999999.99999999', '1.0000000000000002',
                '0.18000000000000002', '0.0030000000000000005', '0.12345678901234568']
    for literal in literals:
        standard_bits = int.from_bytes(struct.pack('>d', float(literal)), 'big')
        require(nearest_bits(D(literal), 64) == standard_bits, 'Independent standard binary64 parser cross-check')
    color = D('0.5000000298023224')
    require(nearest_bits(color, 32) == 0x3f000001, 'Direct decimal binary32 above-midpoint self-check')
    require(nearest_bits(D('0.5000000298023223876953125'), 32) == 0x3f000000, 'Exact binary32 midpoint tie-even self-check')
    through64 = nearest_bits(fraction_from_bits(nearest_bits(color, 64), 64), 32)
    require(through64 == 0x3f000000, 'Double-round negative witness setup')
    witnesses = {}
    for label, name, path, width in [
        ('one_ulp_point40', cases[0], ('project', 'points', 22, 'position', 0), 64),
        ('one_ulp_stiffness', cases[1], ('project', 'materials', 0, 'warp_stiffness'), 64),
        ('wrong_f32_midpoint_color', cases[2], ('project', 'materials', 0, 'color', 0), 32)]:
        source = load(ROOT / name); changed = rounded_native(source)
        bits = nearest_bits(at(changed, path), width)
        wrong = through64 if width == 32 else bits + 1
        put(changed, path, exact_decimal(wrong, width))
        witnesses[label] = reject(lambda: compare_native(source, changed))
    supplement = {'id': 'DOCUMENT-IEEE-EXPECTED-001', 'kind': 'implementation_only',
                  'input_manifest_sha256': sha(ROOT / 'input-manifest.json'), 'oracle_sha256': sha(Path(__file__)),
                  'method': 'Exact Decimal -> Fraction; ordered finite IEEE bit search; nearest distance with even-bit ties. No native output or Core inputs.',
                  'cases': expected, 'recovery_grain_literal': '0.12345678901234568',
                  'recovery_grain_expected_binary64': f'0x{nearest_bits(D("0.12345678901234568"), 64):016x}',
                  'invalid_number_float64_rounding_bits': f'0x{nearest_bits(D("1e400"), 64):016x}',
                  'negative_witnesses': witnesses, 'positive_selfchecks': ['normal/subnormal ties-even', 'standard-library binary64 parser cross-check',
                  'direct binary32 above midpoint', 'exact binary32 midpoint', 'all expected native fields and metadata'],
                  'limits': ['Fixture/oracle preparation only; native output acceptance awaits explicitly provided filenames.', 'No fitting, simulation or parity claim.']}
    output = ROOT / 'expected-bits.json'; output.write_text(json.dumps(supplement, indent=2) + '\n')
    print(json.dumps({'status': 'prepared_exact_expected_bits', 'input_manifest_sha256': sha(ROOT / 'input-manifest.json'),
                      'expected_bits_sha256': sha(output), 'oracle_sha256': sha(Path(__file__)),
                      'field_counts': {name: len(value['fields']) for name, value in expected.items()}, 'negative_witnesses': witnesses}))


if __name__ == '__main__':
    if len(sys.argv) == 1:
        prepare_expected()
    else:
        require(len(sys.argv) == 4, 'Usage: oracle.py INPUT OUTPUT REPORT')
        source_name, actual_name, report_name = sys.argv[1:]
        result = {'kind': 'implementation_only', 'status': 'passed_exact_ieee_and_metadata',
                  'input': source_name, 'output': actual_name, 'input_sha256': sha(ROOT / source_name), 'output_sha256': sha(ROOT / actual_name),
                  'oracle_sha256': sha(Path(__file__)), **compare_native(load(ROOT / source_name), load(ROOT / actual_name))}
        (ROOT / report_name).write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result))
