#!/usr/bin/env python3
"""Reproduce bounded final-SVG and v3-history checks; no native/physics replay."""
import argparse
import copy
import hashlib
import importlib.util
import inspect
import json
from pathlib import Path
import re
import sys
import tempfile
import xml.etree.ElementTree as ET
from datetime import datetime

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
PRIVACY = re.compile(rb'/' + rb'Users/|:' + rb'Users:|/' + rb'home/|:' + rb'home:|/' + rb'private/var/|/' + rb'var/folders/')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    def unique(items):
        result = {}
        for key, value in items:
            require(key not in result, 'Duplicate JSON key')
            result[key] = value
        return result
    return json.loads(path.read_text(), object_pairs_hook=unique)


def child(root, name):
    relative = Path(name)
    require(not relative.is_absolute() and '..' not in relative.parts, 'Invalid relative artifact path')
    path = root / relative
    require(not path.is_symlink() and path.resolve().is_relative_to(root.resolve()), 'Artifact escaped root')
    return path


def verify_tree(root, expected_manifest=None):
    if expected_manifest:
        require(sha(root / 'MANIFEST.json') == expected_manifest, 'Prior packet manifest changed')
    manifest = read(root / 'MANIFEST.json')
    inventory = {p.relative_to(root).as_posix() for p in root.rglob('*') if p.is_file()}
    require(inventory == set(manifest['files']) | {'MANIFEST.json'}, 'Packet inventory changed')
    for path in root.rglob('*'):
        require(not path.is_symlink(), 'Packet symlink refused')
        if path.is_file():
            require(not PRIVACY.search(path.read_bytes()), 'Private absolute path found')
    for name, expected in manifest['files'].items():
        require(sha(child(root, name)) == expected, 'Bound packet artifact changed: ' + name)
    return manifest


def bindings(repo, prior):
    verify_tree(HERE)
    bound = read(HERE / 'BINDINGS.json')
    verify_tree(prior, bound['prior_packet_manifest_sha256'])
    source = read(prior / 'gates-v2/source-pretest.json')
    package = read(prior / 'gates-v2/package-binding.json')
    require(source['source_commit'] == package['source_head'] == bound['source_commit'], 'Runtime source identity differs')
    require(len(source['source_hashes']) == package['source_files'] == 45, 'Wrong runtime manifest scope')
    for name, expected in source['source_hashes'].items():
        require(sha(child(repo, name)) == expected, 'Current runtime source bytes differ: ' + name)
    require(package['binary_sha256']['target/release/weftforge'] == bound['package_native_sha256'], 'Archived package identity differs')
    require(read(prior / 'native-v2/RESULT.json')['status'] == 'failed_strict_checkpoint_expectation', 'Original v2 strict failure lost')
    require(read(HERE / 'native-v3/NATIVE-OUTCOME.json')['all_observer_assertions_passed'] is False, 'Observer failure lost')
    require(read(HERE / 'native-v3/ROOT-POSTFLIGHT.json')['all_observer_assertions_passed'] is False, 'Root observer qualifier lost')
    require(sha(prior / 'independent-evaluator/check.py') == bound['original_checker_sha256'], 'Original checker source changed')
    require(sha(prior / 'independent-evaluator/FROZEN.json') == bound['original_checker_freeze_sha256'], 'Original checker freeze changed')
    require(sha(HERE / 'history-evaluator/FROZEN.json') == bound['history_freeze_sha256'], 'History freeze changed')
    return bound


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def snapshot(module):
    functions = {name: value for name, value in vars(module).items() if inspect.isfunction(value)}
    return {'globals': dict(vars(module)), 'functions': functions,
            'code': {name: value.__code__ for name, value in functions.items()},
            'defaults': {name: value.__defaults__ for name, value in functions.items()},
            'bodies': {name: inspect.getsource(value) for name, value in functions.items()}}


def unchanged(module, before, allowed_globals=(), directory_default=None):
    require(vars(module).keys() == before['globals'].keys(), 'Checker globals added or removed')
    for name, value in before['globals'].items():
        if name not in allowed_globals:
            require(vars(module)[name] is value, 'Checker global replaced: ' + name)
    for name, value in before['functions'].items():
        require(getattr(module, name) is value and value.__code__ is before['code'][name], 'Checker function/code replaced')
        require(inspect.getsource(value) == before['bodies'][name], 'Checker function body changed')
        expected = (directory_default,) if name == 'originals' and directory_default is not None else before['defaults'][name]
        require(value.__defaults__ == expected, 'Unapproved checker default changed')


def fixed_history_paths(bound):
    filenames = {'baseline': 'baseline.weftforge', 'imported': 'imported.weftforge',
                 'undo': 'after-undo.weftforge', 'redo': 'after-redo.weftforge',
                 'edited': 'edited.weftforge', 'rejected': 'after-rejected.weftforge',
                 'reopened': 'reopened.weftforge'}
    require(set(bound['history_documents']) == set(filenames), 'Wrong history checkpoint keys')
    paths = {}
    for name, filename in filenames.items():
        expected = bound['history_documents'][name]
        require(expected['path'] == 'native-v3/' + filename, 'History wrapper is bound only to fixed v3 paths')
        path = child(HERE, expected['path'])
        require(path.parent == HERE / 'native-v3', 'History path is outside v3 case')
        require(sha(path) == expected['sha256'], 'History checkpoint bytes changed')
        paths[name] = path
    require(len({p.resolve() for p in paths.values()}) == 7, 'History checkpoints alias')
    return paths


def final_geometry(checker, root, baseline, old, geometry, source_scale):
    paths = [root / 'edited.weftforge', root / 'reopened.weftforge', root / 'edited.svg']
    require(len({p.resolve() for p in paths}) == 3, 'Final artifacts alias')
    saved, resaved = checker.load_json(paths[0]), checker.load_json(paths[1])
    checker.check_native(saved, baseline, old, geometry)
    checker.check_native(resaved, baseline, old, geometry)
    checker.check_reopen(saved, resaved, 0, None)
    exported, export_scale = checker.read_svg(paths[2].read_bytes())
    checker.compare_geometry(exported, geometry, 'fixed public final SVG')
    checker.originals()
    return {'status': 'STATIC_ARTIFACT_CHECKS_PASSED',
            'saved_sha256': sha(paths[0]), 'reopened_sha256': sha(paths[1]), 'exported_svg_sha256': sha(paths[2]),
            'saved_revision': saved['project']['revision'], 'reopened_revision': resaved['project']['revision'],
            'allowed_resave_revision_delta': 0, 'source_scale': source_scale, 'export_scale': export_scale}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo-root', type=Path, default=Path.cwd())
    parser.add_argument('--prior-packet', type=Path, help='Defaults to the bound public v1/v2 packet inside repo-root')
    args = parser.parse_args()
    require(__debug__, 'Run Python without optimization')
    repo = args.repo_root.resolve()
    initial = read(HERE / 'BINDINGS.json')
    prior = args.prior_packet.resolve() if args.prior_packet else repo / initial['prior_packet_path']
    bound = bindings(repo, prior)
    try:
        child(HERE, '../escape')
    except ValueError:
        pass
    else:
        raise ValueError('Path boundary negative failed')
    temporary_root = Path(tempfile.gettempdir()).resolve()
    require(not temporary_root.is_relative_to(HERE) and not temporary_root.is_relative_to(repo),
            'Synthetic-check scratch must be outside packet and repository')

    checker_path = prior / 'independent-evaluator/check.py'
    checker = load('archived_original_native_svg_checker', checker_path)
    original = snapshot(checker)
    numerical = copy.deepcopy((checker.HASHES, checker.POINT_TOLERANCE, checker.SAMPLE_TOLERANCE))
    fixture_directory = prior / 'native-v1'
    checker.HERE = prior / 'independent-evaluator'
    checker.ROOT = repo
    checker.ORIGINALS = fixture_directory
    checker.SPEC = repo / 'spec/features/FILE-005-svg-pattern-import.md'
    # Python captured this directory at import; only its directory binding changes.
    checker.originals.__defaults__ = (fixture_directory,)
    freeze = checker.frozen()
    baseline = checker.originals()
    old, geometry, source_scale = checker.expected_geometry(baseline)
    original_self = checker.self_checks(baseline, old, geometry)
    require(original_self == checker.load_json(checker.HERE / 'SELF_CHECKS.json'), 'Original synthetic-negative receipt differs')

    supplement_path = HERE / 'history-evaluator/check.py'
    supplement = load('archived_v3_history_supplement', supplement_path)
    history_before = snapshot(supplement)
    history_freeze = checker.load_json(HERE / 'history-evaluator/FROZEN.json')
    for name, expected in history_freeze['supplement_sha256'].items():
        require(sha(child(HERE / 'history-evaluator', name)) == expected, 'Frozen history source changed')
    for name, expected in history_freeze['original_evaluator_sha256'].items():
        require(sha(child(prior / 'independent-evaluator', name)) == expected, 'History original-checker pin differs')
    require(supplement.PINNED_CHECKER == bound['original_checker_sha256'] and
            supplement.PINNED_MANIFEST == bound['original_checker_freeze_sha256'], 'Supplement source pins differ')
    require(history_freeze['case_directory'] == '.local/native-svg-acceptance-v3-20261115', 'Prospective history case changed')
    require(history_freeze['declared_revisions'] == supplement.REVISIONS, 'Frozen revisions changed')
    expected = supplement.expectations(checker)
    require(expected[4] == history_freeze['analytic_allocator']['high_water_next_id'] == 29, 'Prospective allocator changed')
    history_self = supplement.self_checks(checker, expected)
    require(history_self == checker.load_json(HERE / 'history-evaluator/SELF_CHECKS.json'), 'History synthetic-negative receipt differs')

    cases = {'v2_final_geometry': final_geometry(checker, prior / 'native-v2', baseline, old, geometry, source_scale),
             'v3_final_geometry': final_geometry(checker, HERE / 'native-v3', baseline, old, geometry, source_scale)}
    paths = fixed_history_paths(bound)
    require(sha(HERE / 'native-v3/edited.svg') == bound['final_svg_sha256'], 'Bound v3 final SVG changed')
    documents = {name: checker.load_json(path) for name, path in paths.items()}
    supplement.check_history(checker, documents, expected)
    checker.originals()
    cases['v3_history'] = {'status': 'STATIC_HISTORY_CHECKS_PASSED',
        'actual_checkpoint_sha256': {name: sha(path) for name, path in paths.items()},
        'declared_revisions': supplement.REVISIONS, 'allocator_baseline': 11,
        'allocator_after_import_and_history': expected[4], 'analytic_allocation': expected[5],
        'fixed_v3_paths_enforced': True, 'v2_history_not_graded': True}
    geometry_receipt = read(HERE / 'root-grading/final-geometry.json')
    history_receipt = read(HERE / 'root-grading/history.json')
    require(geometry_receipt['status'] == 'STATIC_ARTIFACT_CHECKS_PASSED' and
            history_receipt['status'] == 'STATIC_HISTORY_CHECKS_PASSED', 'Original root grader failure')
    require(history_receipt['actual_checkpoint_sha256'] == cases['v3_history']['actual_checkpoint_sha256'],
            'Original root history receipt differs')
    require(sorted(geometry_receipt['actual_artifact_sha256'].values()) == sorted(
        cases['v3_final_geometry'][key] for key in ['saved_sha256', 'reopened_sha256', 'exported_svg_sha256']),
        'Original root final-SVG receipt differs')
    projection = read(HERE / 'quit-bundle-count-projection.json')
    require(projection['selector'] == {'array': 'apps', 'field': 'id',
        'exact_value': 'org.weftforge.studio.svg.v3.20261115'}, 'Quit selector differs')
    require([row['matching_bundle_count'] for row in projection['observations']] == [1, 1, 0],
            'Projected quit counts differ')
    quit_time = read(HERE / 'native-v3/trace/064-after.json')['completed_utc']
    launch_time = read(HERE / 'native-v3/trace/065-before.json')['started_utc']
    parse_time = lambda value: datetime.fromisoformat(value.replace('Z', '+00:00'))
    require(projection['quit_input_completed_utc'] == quit_time and
            projection['fresh_launch_started_utc'] == launch_time and
            parse_time(quit_time) < parse_time(projection['observations'][-1]['recorded_utc']) < parse_time(launch_time),
            'Settled quit observation is outside the quit/fresh-launch interval')

    unchanged(checker, original, ('HERE', 'ROOT', 'ORIGINALS', 'SPEC'), fixture_directory)
    unchanged(supplement, history_before)
    require((checker.HASHES, checker.POINT_TOLERANCE, checker.SAMPLE_TOLERANCE) == numerical,
            'Original hashes or numerical tolerances changed')
    require(sha(checker_path) == bound['original_checker_sha256'], 'Original checker source mutated')
    require(sha(supplement_path) == history_freeze['supplement_sha256']['check.py'], 'History source mutated')
    bindings(repo, prior)
    report = {'kind': 'Portable static replay of unchanged final-SVG checker and prospective v3 history supplement',
        'status': 'BOUNDED_STATIC_CHECKS_PASSED', 'cases': cases,
        'original_checker_sha256': sha(checker_path), 'history_checker_sha256': sha(supplement_path),
        'original_synthetic_negatives': original_self, 'history_synthetic_negatives': history_self,
        'function_sources_code_objects_and_all_unapproved_defaults_unchanged': True,
        'original_directory_globals_only': ['HERE', 'ROOT', 'ORIGINALS', 'SPEC'],
        'single_directory_default_mapping': 'originals.__defaults__ = (public original-fixture directory,)',
        'all_numerical_functions_constants_and_tolerances_unchanged': True,
        'source45_hashes_and_archived_package_binding_match': True,
        'both_original_root_grading_receipt_hash_sets_reproduced': True,
        'public_quit_projection_timestamps_and_matching_counts_checked': True,
        'packet_prior_and_source_bindings_exact_before_after': True,
        'prior_v2_strict_status': 'failed_strict_checkpoint_expectation',
        'all_v3_observer_assertions_passed': False,
        'new_ui_actions': 0, 'new_physical_steps': 0, 'new_cloth_initializations': 0,
        'scratch_scope': 'Only original synthetic checker temporary directories outside packet/repository',
        'tracked_clean_publication_certified': False, 'reference_parity': '0/56',
        'limits': 'Exit0 reproduces bounded static artifact/history checks. It does not replay native actions, erase observer failures, regrade v2 history, certify original timing, or establish physical, release or reference parity.'}
    print(json.dumps(report, indent=2, allow_nan=False))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, TypeError, IndexError, OSError, ET.ParseError) as error:
        print(json.dumps({'status': 'FAILED', 'error_type': type(error).__name__,
                          'scope': 'Public static replay refused; no native or physical replay was performed'}), file=sys.stderr)
        sys.exit(1)
