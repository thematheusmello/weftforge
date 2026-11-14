#!/usr/bin/env python3
"""Independent native reversal oracle; public contracts and original evidence only."""
import argparse
import copy
from decimal import Decimal as D
from fractions import Fraction as F
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tempfile
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent
SCENARIO = 'UI-NATIVE-SEAM-REVERSAL-001'
SOURCE_COMMIT = '9d30a71bbc2b0e051d9eaddc42b5f3b7dab08bde'
SOURCE_SHA = '8d4090142551f037003e3c531d524be5e80ef870f4c706a185ad1ee89e9205ae'
HELPER_SHA = '90148a51d1bc7f6357908f804cf7308fd4780d6219694271dd103dfc7f821350'
SVG_SHA = '5339421b64714244b81d7dfbe1629c3a4f877806fbb3a4b8ba09018dd41127a4'
ORIGINAL_DECLARATION_SHA = '59e88b5e5e234af6576f6263ba6a5753eb5663eba64ddd84da161f3d73b1706f'
FIXTURE_SHA = {
    'input-nonlinear.weftforge': 'ea89bb6ad55c402eb3370888cd4ec2187b82d6b89ae9e6c4113617249b56c4f8',
    'input-revision-limit.weftforge': '2106593bab0a4ddc4d78265baee82b72f5201fb4cb40e004d29736751b7ce28d',
}
PROJECT_FILES = ('before-reverse', 'after-reverse', 'undo', 'undo-tab', 'redo', 'restored', 'reopened', 'max-loaded', 'max-rejected')
NATIVE_FILES = tuple(f'{name}.weftforge' for name in PROJECT_FILES) + ('restored.obj', 'restored.svg')
INPUT_FILES = tuple(FIXTURE_SHA) + NATIVE_FILES + ('candidate-pretest.json', 'ui-declaration.json', 'preregistration-freeze.json')
LIMITS = [
    'Implementation-only native saved-output acceptance; no reference execution or parity claim.',
    'Oriented curve parameters only; no arc-length equivalence, physical easing/twist, cloth fitting or simulation quality claim.',
    'Default zero placements only; no large-translation coverage.',
    'Saved files cannot establish selected/focused widgets, event timing, endpoint-guide rendering, visible revision-limit error or executing-process identity. Parent UI/process observations are required.',
    'OBJ checks establish finite valid triangles/UVs/normals and bounded metre scale; they do not prove remesher sewing-point usage or simulation behavior.',
]


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


HELPER = ROOT / 'original-evaluators/verify_curve_outputs.py'
require(digest(HELPER) == HELPER_SHA, 'unchanged original evaluator hash differs')
sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('original_curve_output_oracle', HELPER)
original = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(original)


def read_json(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, f'duplicate JSON field: {key}')
            result[key] = value
        return result

    def nonfinite(value):
        raise AssertionError(f'non-finite JSON constant: {value}')

    return json.loads(Path(path).read_text(), parse_float=D, parse_constant=nonfinite, object_pairs_hook=unique)


def exact(actual, expected, label):
    if isinstance(expected, dict):
        require(type(actual) is dict and set(actual) == set(expected), f'{label}: field set changed')
        for key, value in expected.items():
            exact(actual[key], value, f'{label}.{key}')
    elif isinstance(expected, list):
        require(type(actual) is list and len(actual) == len(expected), f'{label}: array length changed')
        for i, (a, e) in enumerate(zip(actual, expected)):
            exact(a, e, f'{label}[{i}]')
    elif isinstance(expected, D):
        require(type(actual) in (int, D) and D(actual).is_finite() and actual == expected, f'{label}: numeric value changed')
    else:
        require(type(actual) is type(expected) and actual == expected, f'{label}: typed value changed')


def seam(document):
    matches = [item for item in document['project']['seams'] if item['id'] == 23]
    require(len(matches) == 1, 'seam 23 must exist once')
    return matches[0]


def source_document(case_dir):
    expected_hashes = {'original-inputs/fine.weftforge': SOURCE_SHA, 'original-inputs/fine.svg': SVG_SHA,
                       'original-inputs/ui-declaration.json': ORIGINAL_DECLARATION_SHA}
    for name, expected_hash in expected_hashes.items():
        require(digest(case_dir / name) == expected_hash, f'original evidence bytes changed: {name}')
    document = read_json(case_dir / 'original-inputs/fine.weftforge')
    p = document['project']
    require(document['format'] == 'weftforge' and document['version'] == 2, 'expected v2 original')
    require(p['revision'] == 14 and p['next_id'] == 24, 'original revision/allocator')
    require([s['id'] for s in p['seams']] == [20, 23], 'original seam IDs/order')
    require([p['id'] for p in p['pieces']] == [2, 11], 'original piece IDs/order')
    require(any(c['id'] == 9 and c['controls'] is not None for c in p['curves']), 'retained parent cubic 9 missing')
    require(all(p['placement'] == {'translation': [0, 0, 0], 'rotation': [0, 0, 0]} for p in p['pieces']), 'requires default zero placements')
    return document


def expected_inputs(source):
    nonlinear = copy.deepcopy(source)
    nonlinear['project']['revision'] = 15
    seam(nonlinear)['correspondence'] = [[D('0'), D('0')], [D('.2'), D('.7')], [D('1'), D('1')]]
    limited = copy.deepcopy(nonlinear)
    limited['project']['revision'] = 2**64 - 1
    return nonlinear, limited


def expected_snapshots(nonlinear, limited):
    result = {}
    states = (('before-reverse', 15, '.035', False), ('after-reverse', 16, '.04', True),
              ('undo', 17, '.035', False), ('undo-tab', 17, '.035', False),
              ('redo', 18, '.04', True), ('restored', 19, '.04', False), ('reopened', 19, '.04', False))
    for name, revision, resolution, flipped in states:
        document = copy.deepcopy(nonlinear)
        document['project'].update(revision=revision, resolution=D(resolution))
        if flipped:
            b = seam(document)['b']
            b['t0'], b['t1'] = b['t1'], b['t0']
        result[name] = document
    result['max-loaded'] = copy.deepcopy(limited)
    result['max-rejected'] = copy.deepcopy(limited)
    return result


def fixture_check(case_dir, source):
    nonlinear, limited = expected_inputs(source)
    for name, expected in zip(FIXTURE_SHA, (nonlinear, limited)):
        require(digest(case_dir / name) == FIXTURE_SHA[name], f'derived input bytes changed: {name}')
        exact(read_json(case_dir / name), expected, name)
    return nonlinear, limited


def pairing_proof(before, after):
    old, new = seam(before), seam(after)
    exact(new['a'], old['a'], 'pairing.side_a')
    exact(new['correspondence'], old['correspondence'], 'pairing.map')
    exact(new['b'], {'curve': old['b']['curve'], 't0': old['b']['t1'], 't1': old['b']['t0']}, 'pairing.side_b')
    intervals = []
    for (u0, v0), (u1, v1) in zip(old['correspondence'], old['correspondence'][1:]):
        u0, v0, u1, v1 = map(F, (u0, v0, u1, v1))
        require(0 <= u0 < u1 <= 1 and 0 <= v0 < v1 <= 1, 'nonmonotone normalized map')
        a0, a1 = map(F, (old['a']['t0'], old['a']['t1']))
        b0, b1 = map(F, (old['b']['t0'], old['b']['t1']))
        c0, c1 = map(F, (new['b']['t0'], new['b']['t1']))
        a_coefficients = (a0 + (a1-a0)*u0, (a1-a0)*(u1-u0))
        reversed_b = (c0 + (c1-c0)*v0, (c1-c0)*(v1-v0))
        original_at_one_minus_v = (b0 + (b1-b0)*(1-v0), -(b1-b0)*(v1-v0))
        require(reversed_b == original_at_one_minus_v, 'Bnew(v) != Boriginal(1-v) on full affine map interval')
        intervals.append({'normalized_u_interval': [str(u0), str(u1)], 'normalized_v_interval': [str(v0), str(v1)],
                          'a_t_local_affine_coefficients': [str(x) for x in a_coefficients],
                          'b_t_local_affine_coefficients': [str(x) for x in reversed_b], 'exact_polynomial_identity': True})
    require(len(intervals) == 2, 'expected two nonlinear-map intervals')
    return {'model': 'A(u), Boriginal(1-v(u)); local s ranges over [0,1] in each map interval.',
            'proof': 'Exact rational affine coefficients: b1+(b0-b1)*v = b0+(b1-b0)*(1-v). Unchanged curve fields lift identical parameters to identical curve points for every s.',
            'intervals_proved': intervals, 'curve_parameter_not_arc_length': True}


def svg_geometry(path):
    root = ET.parse(path).getroot()
    ns = '{http://www.w3.org/2000/svg}'
    require(root.tag == ns + 'svg', 'SVG namespace')
    require(all('transform' not in node.attrib for node in root.iter()), 'unexpected SVG transform')
    require(all(node.tag in {ns + name for name in ('svg', 'title', 'desc', 'path')} for node in root.iter()), 'unexpected SVG element')
    require(root.attrib['width'].endswith('mm') and root.attrib['height'].endswith('mm'), 'SVG units must be millimetres')
    width, height = (D(root.attrib[k][:-2]) for k in ('width', 'height'))
    view = tuple(D(x) for x in root.attrib['viewBox'].split())
    require(len(view) == 4 and all(x.is_finite() for x in (width, height, *view)), 'non-finite SVG extent')
    require(width == view[2] > 0 and height == view[3] > 0, 'SVG physical/user-unit mismatch')
    paths = root.findall(ns + 'path')
    require(len(paths) == 2 and [p.attrib['id'] for p in paths] == ['piece-2', 'piece-11'], 'SVG piece paths/order')
    commands = [original.svg_path_commands(p.attrib['d']) for p in paths]
    require(all(len(c) == 6 and c[0][0] == 'M' and c[-1] == ('Z', ()) for c in commands), 'SVG closed editable boundaries')
    require(sum(command == 'C' for c in commands for command, _ in c) == 1, 'SVG must have exactly one editable cubic')
    return (width, height, view, tuple((p.attrib['id'], p.attrib['data-name'], tuple(c)) for p, c in zip(paths, commands)))


def declaration_check(case_dir):
    path = case_dir / 'ui-declaration.json'
    declaration = read_json(path)
    require(declaration['id'] == SCENARIO and declaration['source_build_commit'] == SOURCE_COMMIT, 'declaration scenario/source identity')
    require(declaration['status_at_declaration'] == 'preregistered_before_candidate_native_launch', 'declaration was not preregistered')
    for field in ('source_tree_sha256', 'executable_sha256', 'cli_executable_sha256', 'candidate_pretest_sha256'):
        require(isinstance(declaration.get(field), str) and re.fullmatch('[0-9a-f]{64}', declaration[field]), f'candidate identity pending: {field}')
    require(declaration.get('declared_at') and declaration.get('executable_basename') == 'weftforge', 'declaration timestamp/basename pending')
    require(declaration['evaluator_sha256'] == digest(__file__), 'declaration evaluator hash differs')
    require(declaration['original_evaluator_sha256'] == HELPER_SHA and declaration['source_baseline_sha256'] == SOURCE_SHA, 'declaration original evidence hash differs')
    require(declaration['fixture_sha256'] == FIXTURE_SHA and declaration['required_files'] == list(INPUT_FILES), 'declaration fixture/file list differs')
    require(digest(case_dir / 'candidate-pretest.json') == declaration['candidate_pretest_sha256'], 'candidate identity metadata bytes differ')
    metadata = read_json(case_dir / 'candidate-pretest.json')
    require(metadata['source_commit'] == SOURCE_COMMIT and metadata['source_tree_sha256'] == declaration['source_tree_sha256'], 'candidate source identity differs')
    tree = json.dumps(metadata['source_hashes'], sort_keys=True, separators=(',', ':')).encode()
    require(hashlib.sha256(tree).hexdigest() == declaration['source_tree_sha256'], 'candidate source hash aggregate differs')
    require(len(metadata['source_hashes']) == declaration['scoped_source_file_count'] == 32, 'candidate source scope count differs')
    require(metadata['native_sha256'] == declaration['executable_sha256'] and metadata['cli_sha256'] == declaration['cli_executable_sha256'], 'candidate binary identity metadata differs')
    require(metadata['package_file_hashes'] == declaration['package_file_hashes'], 'candidate package identity metadata differs')
    freeze = read_json(case_dir / 'preregistration-freeze.json')
    require(freeze['declaration_sha256'] == digest(path), 'frozen declaration changed')
    require(freeze['evaluator_sha256'] == digest(__file__), 'frozen evaluator changed')
    return {'sha256': digest(path), 'freeze_sha256': digest(case_dir / 'preregistration-freeze.json'),
            'declared_at': declaration['declared_at'], 'source_build_commit': SOURCE_COMMIT,
            'executable_sha256': declaration['executable_sha256'], 'identity_is_claimed_by_declaration_requires_parent_process_record': True}


def same_bytes(case_dir, before, after):
    a, b = (case_dir / f'{name}.weftforge' for name in (before, after))
    require(a.read_bytes() == b.read_bytes(), f'no-change snapshot bytes differ: {before}, {after}')
    return {'before': a.name, 'after': b.name, 'sha256': digest(a), 'bytes_identical': True}


def evaluate(case_dir):
    source = source_document(case_dir)
    nonlinear, limited = fixture_check(case_dir, source)
    expected = expected_snapshots(nonlinear, limited)
    snapshots = {name: read_json(case_dir / f'{name}.weftforge') for name in PROJECT_FILES}
    for name in PROJECT_FILES:
        exact(snapshots[name], expected[name], name)
    unchanged = [same_bytes(case_dir, a, b) for a, b in (('undo', 'undo-tab'), ('restored', 'reopened'), ('max-loaded', 'max-rejected'))]
    svg = svg_geometry(case_dir / 'restored.svg')
    require(svg == svg_geometry(case_dir / 'original-inputs/fine.svg'), 'restored SVG geometry differs from original fine.svg')
    obj = original.obj_check(case_dir / 'restored.obj')
    return {'declaration': declaration_check(case_dir), 'whole_document_checkpoints': {
                name: {'all_fields_exact': True, 'revision': snapshots[name]['project']['revision'],
                       'resolution_m': snapshots[name]['project']['resolution'], 'allocator': snapshots[name]['project']['next_id']}
                for name in PROJECT_FILES},
            'exact_no_change_byte_pairs': unchanged,
            'input_save_byte_equality_informational_only': {
                'nonlinear_input_to_before': (case_dir / 'input-nonlinear.weftforge').read_bytes() == (case_dir / 'before-reverse.weftforge').read_bytes(),
                'max_input_to_loaded': (case_dir / 'input-revision-limit.weftforge').read_bytes() == (case_dir / 'max-loaded.weftforge').read_bytes()},
            'nonlinear_oriented_interval_pairing': pairing_proof(snapshots['before-reverse'], snapshots['after-reverse']),
            'two_reversals': 'Only declared revision and resolution differ; all other document fields exact.',
            'obj': obj, 'obj_checker': 'Exact unchanged original evaluator obj_check.',
            'svg': {'geometry_equal_original_fine': True, 'closed_editable_piece_paths': 2, 'editable_cubic_segments': 1, 'physical_user_unit': 'millimetre', 'sha256': digest(case_dir / 'restored.svg')},
            'native_ui_execution_requires_parent_observation_record': True}


def self_check():
    source = source_document(ROOT)
    nonlinear, limited = fixture_check(ROOT, source)
    snapshots = expected_snapshots(nonlinear, limited)
    for name, document in snapshots.items():
        exact(copy.deepcopy(document), document, f'positive.{name}')
    proof = pairing_proof(snapshots['before-reverse'], snapshots['after-reverse'])
    require(proof['intervals_proved'][0]['b_t_local_affine_coefficients'] == ['0', '21/50'], 'independently known first nonlinear B interval')
    negative = {}
    for label, checkpoint in (('wrong_side_a_swapped', 'after-reverse'), ('map_mutated', 'after-reverse'),
                              ('extra_field', 'after-reverse'), ('extra_revision', 'after-reverse'),
                              ('stale_numeric_replay', 'undo-tab'), ('allocator_changed', 'restored'),
                              ('revision_limit_mutation', 'max-rejected')):
        bad = copy.deepcopy(snapshots[checkpoint])
        if label == 'wrong_side_a_swapped':
            bad = copy.deepcopy(snapshots['before-reverse'])
            bad['project'].update(revision=16, resolution=D('.04'))
            a = seam(bad)['a']
            a['t0'], a['t1'] = a['t1'], a['t0']
        elif label == 'map_mutated':
            seam(bad)['correspondence'][1][1] = D('.3')
        elif label == 'extra_field':
            bad['project']['unexpected_field'] = 1
        elif label == 'extra_revision':
            bad['project']['revision'] += 1
        elif label == 'stale_numeric_replay':
            bad['project']['resolution'] = D('.04')
        elif label == 'allocator_changed':
            bad['project']['next_id'] += 1
        else:
            b = seam(bad)['b']
            b['t0'], b['t1'] = b['t1'], b['t0']
        try:
            exact(bad, snapshots[checkpoint], label)
        except AssertionError as error:
            negative[label] = {'status': 'rejected', 'reason': str(error)}
        else:
            raise AssertionError(f'negative witness accepted: {label}')
    with tempfile.TemporaryDirectory(prefix='synthetic-oracle-', dir=ROOT) as directory:
        path = Path(directory) / 'synthetic.txt'
        path.write_text('v 0 0 0\nv 1.4 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nvn 0 0 1\nf 1/1/1 2/2/1 3/3/1\n')
        obj = original.obj_check(path)
        path.write_text(path.read_text().replace('v 0 0 0', 'v NaN 0 0', 1))
        try:
            original.obj_check(path)
        except AssertionError as error:
            negative['nonfinite_obj'] = {'status': 'rejected', 'reason': str(error)}
        else:
            raise AssertionError('nonfinite OBJ witness accepted')
        expected_svg = svg_geometry(ROOT / 'original-inputs/fine.svg')
        path.write_bytes((ROOT / 'original-inputs/fine.svg').read_bytes())
        require(svg_geometry(path) == expected_svg, 'positive original SVG failed')
        path.write_text(path.read_text().replace('100 -800', '100.001 -800', 1))
        require(svg_geometry(path) != expected_svg, 'mutated SVG witness accepted')
        negative['svg_control_drift_1e_6_m'] = {'status': 'rejected', 'reason': 'SVG geometry differs from original.'}
    return {'kind': 'synthetic_in_memory_oracle_checks_only', 'status': 'passed', 'positive_whole_document_checkpoints': len(snapshots),
            'exact_nonlinear_interval_proof': proof, 'synthetic_obj_positive': obj,
            'negative_witnesses': negative, 'native_outputs_created': False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--case-dir', type=Path, default=ROOT)
    parser.add_argument('--self-check-only', action='store_true')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    result = {'scenario': SCENARIO, 'kind': 'implementation_only', 'evaluator_sha256': digest(__file__),
              'original_evaluator_sha256': HELPER_SHA, 'limits': LIMITS}
    initial_hashes = {name: digest(args.case_dir / name) for name in INPUT_FILES if (args.case_dir / name).is_file()}
    try:
        result['self_check'] = self_check()
        if args.self_check_only:
            result['status'] = 'passed_synthetic_oracle_checks_only'
        else:
            missing = [name for name in INPUT_FILES if not (args.case_dir / name).is_file()]
            if missing:
                result.update(status='incomplete_missing_native_outputs', missing_files=missing)
            else:
                result['checks'] = evaluate(args.case_dir)
                result['status'] = 'passed_for_supplied_outputs'
        require(all(digest(args.case_dir / name) == value for name, value in initial_hashes.items()), 'evaluator changed an input artifact')
        result['inputs_preserved'] = initial_hashes
    except (AssertionError, KeyError, TypeError, ValueError, OSError, ET.ParseError, ArithmeticError) as error:
        result.update(status='failed', failure={'type': type(error).__name__, 'message': str(error)})
    if args.output:
        protected = INPUT_FILES + ('original-inputs/fine.weftforge', 'original-inputs/fine.svg', 'original-inputs/ui-declaration.json',
                                   'original-evaluators/verify_curve_outputs.py', 'verify_native_reversal.py')
        require(args.output.resolve() not in {(args.case_dir / name).resolve() for name in protected}, 'report must not overwrite input artifacts')
        args.output.write_text(json.dumps(result, indent=2, default=str) + '\n')
    print(json.dumps(result if args.output is None else {'status': result['status'], 'report': str(args.output.name),
                     'report_sha256': digest(args.output), 'failure': result.get('failure'), 'missing_files': result.get('missing_files', [])}, default=str))
    return 0 if result['status'].startswith('passed') else 2 if result['status'].startswith('incomplete') else 1


if __name__ == '__main__':
    sys.exit(main())
