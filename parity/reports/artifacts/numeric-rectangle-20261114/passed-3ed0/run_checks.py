#!/usr/bin/env python3
"""Reproduce fixed saved-output evidence only; no UI or timing experiment."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
SIZES = ['--unit', 'm', '--expected-mm', '200', '300', '--tolerance-mm', '.01']
CASES = [
    ('numeric_obj_bounds', ['original_obj_bounds.py', 'numeric.obj', *SIZES], 'independent-numeric-bounds.json'),
    ('numeric_obj_boundary', ['original_obj_boundary.py', 'numeric.obj', *SIZES], 'independent-numeric-boundary.json'),
    ('initial_numeric', ['verify_initial_numeric.py'], None),
    ('earlier_partial_curve_and_history', ['verify_dynamic_numeric.py'], None),
    ('completed_main', ['verify_completed_numeric.py'], None),
    ('separate_cancel_snapshots', ['verify_cancellation.py'], None),
]
checks = []
for label, args, redirected in CASES:
    completed = subprocess.run([sys.executable, '-B', *args], cwd=ROOT, text=True, capture_output=True)
    row = {'case': label, 'command': ['python3', '-B', *args], 'expected_exit': 0,
           'actual_exit': completed.returncode, 'matches_expected_exit': completed.returncode == 0, 'stderr': completed.stderr}
    if redirected:
        (ROOT / redirected).write_text(completed.stdout)
        row['output_report'] = redirected
        row['output_report_sha256'] = hashlib.sha256((ROOT / redirected).read_bytes()).hexdigest()
    else:
        row['stdout'] = completed.stdout
    checks.append(row)
read = lambda name: json.loads((ROOT / name).read_text())
initial, partial, main, cancel = [read(name) for name in ('independent-initial-check.json', 'independent-dynamic-check.json',
                                                         'independent-completed-check.json', 'independent-cancellation-check.json')]
witnesses = {}
for label, report in [('initial', initial), ('main', main), ('cancel', cancel)]:
    witnesses.update({label + '/' + key: value for key, value in report['negative_witnesses'].items()})
witnesses['partial/old_quantization'] = partial['negative_witness_old_p2_x_quantization']
witnesses['partial/focus_blur_grain'] = partial['negative_witness_focus_blur_mutates_grain']
source = cancel['native_save_migration']['source_field_comparison']
limits_preserved = (source['coordinate_values_exactly_equal'] is False and len(source['coordinate_deltas']) == 2
                    and {v['point_id'] for v in source['coordinate_deltas']} == {40, 41}
                    and all(v['signed_delta_m'] == '1E-16' for v in source['coordinate_deltas'])
                    and partial['curve27']['both_y_target_reached'] is False
                    and main['invalid_mixed_instance']['qualification'].startswith('excluded'))
matched = (all(row['matches_expected_exit'] and not row['stderr'] for row in checks)
           and len(witnesses) == 13 and all(v['status'] == 'rejected' for v in witnesses.values())
           and main['status'] == 'passed_main_native_saved_output_acceptance'
           and cancel['status'] == 'passed_saved_snapshot_cancel_checks' and limits_preserved)
log = {'bundle_kind': 'explicit_portable_derivative', 'status': 'passed_scoped_checks' if matched else 'unexpected_check_result',
       'overall_exit_code': 0 if matched else 1, 'checks': checks, 'negative_witnesses': witnesses,
       'all_13_negative_witnesses_rejected': len(witnesses) == 13 and all(v['status'] == 'rejected' for v in witnesses.values()),
       'scopes': {'main_saved_output_acceptance': main['status'], 'separate_cancellation_snapshot_check': cancel['status'],
                 'earlier_partial_y_action': 'retained incomplete; not credited as the completed target',
                 'mixed_instance_attempt': 'retained invalid; excluded from candidate attribution',
                 'source_migration_bit_lossless': False, 'source_coordinate_deltas_m': source['coordinate_deltas'],
                 'prior_failed_candidate': 'separate retained provenance, not executed by this new-candidate runner'},
       'limits': ['No UI, Cargo, timing experiment or simulation is performed by this runner.',
                  'Parent-observed 890 ms is CUA/AX UI acknowledgement only; exact phase and worker exit are unmeasured.',
                  'No full reference parity, fitting or rendering-quality claim.']}
(ROOT / 'selfcheck-run.log').write_text(json.dumps(log, indent=2) + '\n')
print(json.dumps({'status': log['status'], 'overall_exit_code': log['overall_exit_code'], 'run_log': 'selfcheck-run.log'}))
raise SystemExit(log['overall_exit_code'])
