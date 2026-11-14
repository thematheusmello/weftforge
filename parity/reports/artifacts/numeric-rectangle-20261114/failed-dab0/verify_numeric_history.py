#!/usr/bin/env python3
"""Exact saved-state checks for the numeric rectangle history/error subcase."""
import copy
from decimal import Decimal
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).parent
NAMES = ['empty.weftforge', 'numeric.weftforge', 'numeric-undone.weftforge',
         'numeric-redone.weftforge', 'numeric-after-errors.weftforge']


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(empty, numeric, undone, redone, after_errors):
    expected = copy.deepcopy(empty)
    expected['project']['revision'] = numeric['project']['revision'] + 1
    expected['project']['next_id'] = numeric['project']['next_id']
    require(undone == expected, 'Undo differs from empty beyond revision and retained allocator high-water value')
    expected = copy.deepcopy(numeric)
    expected['project']['revision'] = undone['project']['revision'] + 1
    require(redone == expected, 'Redo differs from original beyond revision')
    require(after_errors == redone, 'Rejected input/cancel changed accepted document or revision')
    for value in (numeric, undone, redone, after_errors):
        require(value['project']['materials'] == empty['project']['materials'], 'Material changed from baseline')


def main():
    docs = [json.loads((ROOT / name).read_text(), parse_float=Decimal) for name in NAMES]
    empty, numeric, undone, redone, after_errors = docs
    check(*docs)
    require((ROOT / NAMES[3]).read_bytes() == (ROOT / NAMES[4]).read_bytes(), 'Rejected input/cancel changed saved bytes')
    changed = copy.deepcopy(after_errors)
    changed['project']['revision'] += 1
    try:
        check(empty, numeric, undone, redone, changed)
    except AssertionError as error:
        witness = {'status': 'rejected', 'reason': str(error)}
    else:
        raise AssertionError('Negative witness accepted an error-induced history mutation')
    result = {'scenario': 'UI-NUMERIC-RECTANGLE-001', 'kind': 'implementation_only',
              'evaluator_sha256': sha(Path(__file__)), 'status': 'passed_with_explicit_allocator_delta',
              'undo': {'revision': undone['project']['revision'], 'geometry_empty': True,
                       'all_baseline_data_identical_except': ['revision', 'next_id'],
                       'revision_only_empty_equality': False, 'initial_next_id': empty['project']['next_id'],
                       'undone_next_id': undone['project']['next_id'],
                       'allocator_note': 'Actual retained value is visible here; no implementation-intent or bug conclusion is inferred.'},
              'redo': {'revision': redone['project']['revision'], 'equals_original_except_revision': True},
              'error_cancel': {'revision': after_errors['project']['revision'], 'full_document_and_bytes_unchanged': True},
              'materials_identical_to_empty_in_all_checkpoints': True,
              'negative_witness_error_increments_revision': witness,
              'artifacts': {name: {'sha256': sha(ROOT / name), 'bytes': (ROOT / name).stat().st_size,
                                  'revision': doc['project']['revision']} for name, doc in zip(NAMES, docs)},
              'limitations': ['One-click Undo/Redo and rejected input/cancel gestures are parent-observed native UI evidence.',
                              'Empty revision-only equality is explicitly false because next_id remains at the allocated high-water value.',
                              'No reference parity, cloth fitting or rendering quality claim.']}
    result['portable_derivative'] = {'original_evaluator_sha256': 'b12e67fc3dfff4a21aebacc5eef823452510e05d79fe865b932bf091a9aede2c', 'note': 'Bundle-relative fixtures; process metadata omitted; fixture bytes and tolerances unchanged.'}
    output = ROOT / 'independent-numeric-history-check.json'
    output.write_text(json.dumps(result, indent=2, default=str) + '\n')
    print(json.dumps({'status': result['status'], 'report': output.name, 'report_sha256': sha(output),
                      'evaluator_sha256': result['evaluator_sha256']}))


if __name__ == '__main__':
    main()
