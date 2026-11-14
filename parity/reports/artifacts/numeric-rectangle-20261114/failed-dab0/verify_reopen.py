#!/usr/bin/env python3
"""Portable derivative check of retained native reopen bytes, not UI behavior."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).parent


def same_bytes(before, after):
    if before != after:
        raise AssertionError('Reopened native bytes differ')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


before, after = ROOT / 'focused-redone.weftforge', ROOT / 'reopened.weftforge'
same_bytes(before.read_bytes(), after.read_bytes())
try:
    same_bytes(before.read_bytes(), after.read_bytes() + b' ')
except AssertionError:
    witness = 'rejected'
else:
    raise AssertionError('Byte-equality negative witness accepted')
doc = json.loads(after.read_text())
empty = json.loads((ROOT / 'empty.weftforge').read_text())
assert doc['project']['materials'] == empty['project']['materials']
result = {'scenario': 'UI-NUMERIC-RECTANGLE-001', 'kind': 'implementation_only',
          'portable_derivative': True, 'evaluator_sha256': sha(Path(__file__)),
          'original_report_sha256': sha(ROOT / 'original-results' / 'independent-reopen-check.json'),
          'status': 'passed_for_reopened_saved_bytes', 'full_native_bytes_equal': True,
          'full_document_and_revision_equal': True, 'revision': doc['project']['revision'],
          'materials_exact_empty_baseline': True, 'negative_witness_semantic_only_equality': witness,
          'actual_curve27_controls': next(c for c in doc['project']['curves'] if c['id'] == 27)['controls'],
          'artifacts': {p.name: {'sha256': sha(p), 'bytes': p.stat().st_size} for p in (before, after)},
          'limitations': ['UI relaunch/Open and process identity remain parent observations.',
                          'Known P2X quantization remains unresolved for this frozen candidate.',
                          'No reference parity, cloth fitting, rendering quality or timing claim.']}
output = ROOT / 'independent-reopen-check.json'
output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'report': output.name, 'report_sha256': sha(output),
                  'evaluator_sha256': result['evaluator_sha256']}))
