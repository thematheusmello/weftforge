#!/usr/bin/env python3
"""Run only the frozen, portable numeric-rectangle saved-output checks."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
SIZES = ['--unit', 'm', '--expected-mm', '200', '300', '--tolerance-mm', '.01']
CASES = [
    ('obj_bounds', ['original_obj_bounds.py', 'numeric.obj', *SIZES], 0, 'independent-numeric-bounds.json'),
    ('obj_boundary', ['original_obj_boundary.py', 'numeric.obj', *SIZES], 0, 'independent-numeric-boundary.json'),
    ('native_svg', ['verify_numeric_rectangle.py'], 0, None),
    ('creation_history_errors', ['verify_numeric_history.py'], 0, None),
    ('identity_focused_history', ['verify_identity_history.py'], 1, None),
    ('native_reopen', ['verify_reopen.py'], 0, None),
]
results = []
for label, args, expected_exit, redirected in CASES:
    completed = subprocess.run([sys.executable, '-B', *args], cwd=ROOT, text=True, capture_output=True)
    row = {'case': label, 'command': ['python3', '-B', *args], 'expected_exit': expected_exit,
           'actual_exit': completed.returncode, 'matches_expected_exit': completed.returncode == expected_exit,
           'stderr': completed.stderr}
    if redirected:
        (ROOT / redirected).write_text(completed.stdout)
        row['output_report'] = redirected
        row['output_report_sha256'] = hashlib.sha256((ROOT / redirected).read_bytes()).hexdigest()
    else:
        row['stdout'] = completed.stdout
    results.append(row)
identity_path = ROOT / 'independent-identity-history-check.json'
identity = json.loads(identity_path.read_text()) if identity_path.exists() else {}
expected_failure = identity.get('status') == 'partial_pass_with_control_precision_defect' and identity.get('curve27', {}).get('unchanged_standard_third_x') is False
matched = all(row['matches_expected_exit'] and not row['stderr'] for row in results) and expected_failure
status = 'reproduced_known_precision_failure' if matched else 'unexpected_check_result'
log = {'bundle_kind': 'explicit_public_safe_portable_derivative', 'status': status,
       'all_expected_exit_codes_matched': matched, 'known_failure_remains_explicit': expected_failure,
       'overall_exit_code': 1 if matched else 2, 'checks': results,
       'limits': ['Frozen native outputs only; no UI, Cargo, timing or reference-parity claim.',
                  'Exit 1 is required for the retained unedited-control precision defect.']}
(ROOT / 'selfcheck-run.log').write_text(json.dumps(log, indent=2) + '\n')
print(json.dumps({'status': status, 'overall_exit_code': log['overall_exit_code'], 'run_log': 'selfcheck-run.log'}))
raise SystemExit(log['overall_exit_code'])
