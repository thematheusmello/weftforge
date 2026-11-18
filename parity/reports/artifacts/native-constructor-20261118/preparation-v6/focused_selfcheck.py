"""One bound-basename check and the focused caller VM cases; no GUI/native invocation."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
from unittest.mock import patch

import preview_launch as launch
import preview_owner as p
import resource_observer

ROOT = Path(__file__).resolve().parent
OUT = ROOT / 'stub-check-001'
case = OUT / 'bound-basename'
case.mkdir()
fixture = case / 'different-readonly-input.weftforge'
fixture.write_bytes(b'inert readonly fixture bytes; no project authoring')
fixture.chmod(0o444)
before = hashlib.sha256(fixture.read_bytes()).hexdigest()
started = {'start_epoch': 1000., 'action_cutoff_epoch': 1240., 'deadline_epoch': 1300.}
plan = {'case': 'WF-V6-BOUND-STUB', 'case_path': str(case), 'python': '/stub/python', 'app': str(case / 'Inert.app'),
        'fixture': {'path': str(fixture)}, 'native': {'path': str(case / 'inert-native'), 'sha256': '2' * 64}}
digest, token = '1' * 64, 'original producer creation token'
lease = {'pid': 42, 'app_path': plan['app'], 'executable': plan['native']['path'], 'bundle_id': 'org.weftforge.stub',
         'native_start_token': 'original native creation token', 'parent_pid': 4242, 'process_group': 4242,
         'direct_child_created_epoch': 1001., 'source_binary_sha256': '2' * 64, 'window_id': 99, 'window_geometry': {'Width': 1640, 'Height': 1032}}
producer = {'case': plan['case'], 'plan_sha256': digest, 'start_epoch': 1000., 'pid': 4242, 'pgid': 4242,
            'start_token': token, 'executable': '/stub/python', 'command': ['/stub/python', '-B', str(case / 'preview_launch.py'), 'launch']}
ready = {'case': plan['case'], 'app': plan['app'], 'plan_sha256': digest, 'start_epoch': 1000.,
         'producer_pid': 4242, 'producer_pgid': 4242, 'lease': lease, 'ready_epoch': 1002., 'startup_begin_epoch': 1001., 'startup_deadline_epoch': 1009.}
with patch.object(p, 'CASE', case), patch.object(launch, 'CASE', case):
    for name, value in [('LAUNCH-PRODUCER-LEASE.json', producer), ('LAUNCH-READY.json', ready),
                        ('OWNED-IDENTITY.json', lease), ('OWNED-IDENTITY-001.json', lease)]:
        p.write(name, value)
    p.write('LAUNCH-READINESS-ADMISSION.json', {'case': plan['case'], 'plan_sha256': digest, 'start_epoch': 1000.,
            'producer_pid': 4242, 'producer_pgid': 4242, 'producer_start_token': token, 'child_pid': 42,
            'ready_sha256': p.sha(case / 'LAUNCH-READY.json'), 'startup_begin_epoch': 1001., 'startup_deadline_epoch': 1009.,
            'ready_persistence_observed_epoch': 1003.})
    with patch.object(launch, 'time', SimpleNamespace(time=lambda: 1010., sleep=lambda _: None)), \
            patch.object(launch, 'live_monitors', return_value={'WATCHDOG': {}, 'RESOURCE': {}}), \
            patch.object(resource_observer, 'probe', return_value={'start_token': token}), \
            patch.object(launch, 'inventory', return_value={'apps': [], 'windows': []}), \
            patch.object(launch, 'lease_from_inventory', return_value=lease), patch.object(p, 'bounds', return_value={'within_bounds': True}):
        bound = launch.bound(plan, started, digest)
    assert bound['fixture_basename'] == fixture.name
    assert bound['fixture_basename'] not in ['harbor-shirt.weftforge', 'original-harbor.weftforge']
    assert hashlib.sha256(fixture.read_bytes()).hexdigest() == before
node = subprocess.run(['<HOME>/.nvm/versions/node/v26.12.1/bin/node', str(ROOT / 'focused_selfcheck.cjs')],
                      cwd=ROOT, capture_output=True, text=True, timeout=20, check=True)
node_result = json.loads(node.stdout)
print(json.dumps({'status': 'PASSED_V6_FOCUSED_CALLER_CHECKS', 'Python_scenarios': 1, 'Node_scenarios': node_result['count'],
                  'count': 1 + node_result['count'], 'Python_check': 'actual bound response returns readonly fixture Path.name; bytes unchanged',
                  'node': node_result, 'actual_GUI_native_app_model_Cargo_network_calls': 0, 'old_proof_rerun_or_regraded': False}, indent=2))
