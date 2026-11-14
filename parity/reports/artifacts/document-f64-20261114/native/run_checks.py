#!/usr/bin/env python3
"""Portable fixed-case runner; current executed subset plus historical expected failure."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from ieee_document_oracle import load, compare_native, nearest_bits, sha

ROOT = Path(__file__).resolve().parent
checks = []
for script in ('ieee_document_oracle.py', 'verify_document_io.py'):
    done = subprocess.run([sys.executable, '-B', script], cwd=ROOT, capture_output=True, text=True)
    checks.append({'command': ['python3', '-B', script], 'expected_exit': 0, 'actual_exit': done.returncode,
                   'stdout': done.stdout, 'stderr': done.stderr})
result = load(ROOT / 'independent-document-io-check.json')
original_expected = ROOT / 'original-results' / 'expected-bits.json'
expected_unchanged = (ROOT / 'expected-bits.json').read_bytes() == original_expected.read_bytes()
prior_input = load(ROOT / 'prior-evidence' / 'prior-v1-input.weftforge')
prior_output = load(ROOT / 'prior-evidence' / 'prior-rounded-v2.weftforge')
try:
    compare_native(prior_input, prior_output)
except AssertionError as error:
    historical = {'status': 'rejected_as_expected', 'reason': str(error), 'qualification': 'Separate source359/3ed0 historical failure, never current candidate output.'}
else:
    historical = {'status': 'unexpectedly_accepted'}
prior_points = []
for ident in (40, 41):
    a = next(p for p in prior_input['project']['points'] if p['id'] == ident)['position'][0]
    b = next(p for p in prior_output['project']['points'] if p['id'] == ident)['position'][0]
    prior_points.append({'point_id': ident, 'source_literal': str(a), 'historical_saved_literal': str(b),
                         'source_expected_binary64': f'0x{nearest_bits(a, 64):016x}',
                         'historical_actual_binary64': f'0x{nearest_bits(b, 64):016x}', 'signed_decimal_delta_m': str(b - a)})
witnesses = result['negative_witnesses']
matched = (all(c['actual_exit'] == c['expected_exit'] and not c['stderr'] for c in checks)
           and expected_unchanged and result['status'] == 'passed_executed_native_io_subset'
           and result['full_contract_verified'] is False and len(result['not_executed']) == 3
           and len(witnesses) == 5 and all(v['status'] == 'rejected' for v in witnesses.values())
           and historical['status'] == 'rejected_as_expected'
           and all(p['signed_decimal_delta_m'] == '1E-16' for p in prior_points))
log = {'bundle_kind': 'portable_public_evidence_copy', 'status': 'passed_executed_subset_with_historical_failure_retained' if matched else 'unexpected_check_result',
       'overall_exit_code': 0 if matched else 1, 'checks': checks, 'frozen_expected_bits_unchanged': expected_unchanged,
       'full_contract_verified': False, 'not_executed': result['not_executed'], 'negative_witnesses': witnesses,
       'all_five_negative_witnesses_rejected': len(witnesses) == 5 and all(v['status'] == 'rejected' for v in witnesses.values()),
       'historical_failed_candidate': {'check': historical, 'point_deltas': prior_points},
       'limits': ['No native UI, Cargo, simulation, fitting, performance or reference-parity test is run here.',
                  'Native gestures/process/recovery observations remain parent evidence; only the declared executed subset passes.',
                  'Historical rounded values are retained unchanged; a parser cannot reconstruct values already saved after rounding.']}
(ROOT / 'selfcheck-run.log').write_text(json.dumps(log, indent=2) + '\n')
print(json.dumps({'status': log['status'], 'overall_exit_code': log['overall_exit_code'], 'run_log': 'selfcheck-run.log'}))
raise SystemExit(log['overall_exit_code'])
