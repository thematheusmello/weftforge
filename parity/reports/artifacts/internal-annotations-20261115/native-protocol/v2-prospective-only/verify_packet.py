#!/usr/bin/env python3
"""Static v2 packet invariants only; never invoke a product, GUI or old grader."""
import argparse
import hashlib
import json
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parent
OLD = ROOT.parent / 'internal-native-plan-20261115'
OLD_FREEZE_SHA = 'ac65bff3ab3a9c5ab006d36fa2f4db7bd385b8a70f6010cbf53e9fa8154e3a28'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def load(path):
    return json.loads(path.read_text())


def numeric_bits(value, path=()):
    if isinstance(value, dict):
        return [entry for key in sorted(value) for entry in numeric_bits(value[key], (*path, key))]
    if isinstance(value, list):
        return [entry for index, item in enumerate(value) for entry in numeric_bits(item, (*path, index))]
    if type(value) is float:
        return [(list(path), struct.pack('!d', value).hex())]
    return []


def verify():
    frozen_raw = (OLD / 'FREEZE.json').read_bytes()
    assert sha(frozen_raw) == OLD_FREEZE_SHA
    frozen = json.loads(frozen_raw)
    assert len(frozen['sha256']) == 523
    for name, digest in frozen['sha256'].items():
        assert sha((OLD / name).read_bytes()) == digest, name
    assert (ROOT / 'provenance/OLD-FREEZE-ac65.json').read_bytes() == frozen_raw
    unchanged = ('EXPECTATIONS.json', 'grade_saved.py', 'retain_saved.py', 'selfcheck_saved_files.py',
                 'SELF-CHECK-DRAFT-005.json', 'SELF-CHECK-FILES-001.json', 'SELF-CHECK-SNAPSHOT-001.json',
                 'APPROVED-CONTRACT.md', 'FINAL-SOURCE-REVIEW.json')
    unchanged_sha = {}
    for name in unchanged:
        assert (ROOT / name).read_bytes() == (OLD / name).read_bytes(), name
        unchanged_sha[name] = sha((ROOT / name).read_bytes())
    for directory in ('fixtures', 'source-context'):
        for source in (OLD / directory).iterdir():
            if source.is_file():
                assert (ROOT / directory / source.name).read_bytes() == source.read_bytes(), source.name
    model = load(ROOT / 'EXPECTATIONS.json')
    original_model = load(OLD / 'EXPECTATIONS.json')
    assert numeric_bits(model['states']) == numeric_bits(original_model['states'])
    state_sha = sha(json.dumps(model['states'], sort_keys=True, separators=(',', ':')).encode())
    a_sha = sha(json.dumps(model['states']['A'], sort_keys=True, separators=(',', ':')).encode())
    assert state_sha == frozen['original_numeric_authority']['all_24_states_sha256']
    assert a_sha == frozen['original_numeric_authority']['A_states_sha256']
    assert len(model['states']['A']) == 6
    plan, original_plan = load(ROOT / 'PLAN.json'), load(OLD / 'PLAN.json')
    assert plan['macros'] == original_plan['macros']
    for case in 'BCDE':
        assert plan['cases'][case] == original_plan['cases'][case]
    steps = plan['cases']['A']['steps']
    assert len(steps) == 34
    continuation = [s for s in steps if s['label'] in ('reopen-after-backdrop', 'reenter-invalid-after-backdrop')]
    assert [(s['macro'], s['times']) for s in continuation] == [('button', 1), ('field', 1)]
    original_steps = [s for s in steps if s not in continuation]
    assert len(original_steps) == 32
    for current, original in zip(original_steps, original_plan['cases']['A']['steps']):
        if current['label'] == 'modal-background-pointer-blocked':
            assert {k: v for k, v in current.items() if k != 'procedure'} == {k: v for k, v in original.items() if k != 'procedure'}
            assert 'Expect backdrop Cancel' in current['procedure']
        else:
            assert current == original, current['label']
    index = next(i for i, s in enumerate(steps) if s['label'] == 'modal-background-pointer-blocked')
    assert [s['label'] for s in steps[index:index + 5]] == [
        'modal-background-pointer-blocked', 'reopen-after-backdrop',
        'reenter-invalid-after-backdrop', 'cancel-invalid', 'caret-witness']
    assert 'Do NOT Apply again' in steps[index + 2]['procedure']
    static_calls = sum(plan['macros'][s['macro']]['max_calls'] * s['times'] for s in steps)
    assert static_calls == 164 == plan['cases']['A']['static_max_calls']
    assert plan['cases']['A']['call_cap'] == 175 and plan['cases']['A']['reserved_calls'] == 11
    assert plan['cases']['A']['elapsed_cap_s'] == 1200
    budget, old_budget = load(ROOT / 'STATIC-BUDGET.json'), load(OLD / 'STATIC-BUDGET.json')
    assert budget['cases']['A'] == {'static_max_calls': 164, 'call_cap': 175, 'reserved_calls': 11, 'elapsed_cap_s': 1200}
    assert budget['all_static_max_calls'] == old_budget['all_static_max_calls'] + 7 == 941
    for key in ('all_case_max_calls', 'all_elapsed_cap_s', 'completion_summary_extra_calls'):
        assert budget[key] == old_budget[key]
    source = load(ROOT / 'SOURCE-65-PINS.json')
    assert source['source_input_count'] == len(source['source_inputs']) == 65
    source_root = ROOT.parent / 'internal-segments-20261115'
    for name, digest in source['source_inputs'].items():
        assert sha((source_root / name).read_bytes()) == digest, name
    template = load(ROOT / 'ROOT-BINDING.template.json')
    assert template['root_authorized_for_case'] is False
    config = template['controller_configuration']
    assert set(config) == {'root', 'recorderPath', 'evidenceKind', 'callCap', 'elapsedCapS', 'baselineCalls'}
    assert config == {'root': None, 'recorderPath': None, 'evidenceKind': 'native', 'callCap': 175, 'elapsedCapS': 1200, 'baselineCalls': 0}
    assert template['completion_summary_bounds_policy']['bounds_and_caps'] == {
        'case_operations_max': 175, 'actual_summary_max': 1,
        'total_actual_calls_max': 176, 'conservative_elapsed_upper_bound_through_summary_return_s': 1200}
    for entry in load(ROOT / 'provenance/OLD-CASE-PINS.json').values():
        relative = entry['path_relative_to_new_packet']
        assert '/A002/work/' not in relative, 'mutable cleanup sidecar must not be old evidence'
        assert sha((ROOT / relative).read_bytes()) == entry['sha256'], relative
    return {
        'status': 'passed_static_v2_packet_invariants_only',
        'old_freeze_sha256': OLD_FREEZE_SHA, 'old523_dependencies_unchanged': True,
        'all24_original_models_and_f64_bits_unchanged': True,
        'A_six_states_sha256': a_sha, 'all24_states_sha256': state_sha,
        'numeric_f64_leaf_bits_sha256': sha(json.dumps(numeric_bits(model['states']), separators=(',', ':')).encode()),
        'fixtures_and_tested_executable_bytes_unchanged': True, 'unchanged_sha256': unchanged_sha,
        'original_A_steps': 32, 'new_ordered_items': 34, 'added_static_calls': 7,
        'static_operations': static_calls, 'operation_cap': 175, 'reserve': 11,
        'extra_actual_summary_calls': 1, 'elapsed_cap_s': 1200,
        'source65_inputs_unchanged': True, 'binding_template_is_not_grant': True,
        'unchanged42_synthetic_cases_rerun': False, 'old_outputs_regraded': False,
        'native_case_staged_or_run': False, 'product_GUI_Cargo_or_Git_mutation': False,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    assert not args.report.exists(), 'fresh report required'
    try:
        report = verify()
    except Exception as error:
        report = {'status': 'failed_static_v2_packet_invariants', 'error': type(error).__name__ + ': ' + str(error),
                  'old_outputs_regraded': False, 'native_case_staged_or_run': False}
        with args.report.open('x') as stream:
            json.dump(report, stream, indent=2)
            stream.write('\n')
        raise
    with args.report.open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    print(json.dumps({key: report[key] for key in ('status', 'static_operations', 'operation_cap', 'reserve', 'A_six_states_sha256', 'all24_states_sha256')}))


if __name__ == '__main__':
    main()
