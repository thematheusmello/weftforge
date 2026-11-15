#!/usr/bin/env python3
"""Invented file/trace plumbing counterexamples, never a WeftForge/native run."""
import copy
import json
import shutil
from pathlib import Path

import grade_saved as G
import retain_saved as R

ROOT = Path(__file__).resolve().parent
BASE = ROOT / 'SYNTHETIC-FILE-GRADE-001'
BASE.mkdir()
GOOD = BASE / 'accepted-synthetic-A'
(GOOD / 'work').mkdir(parents=True)
(GOOD / 'trace').mkdir()
models = G.parsed(G.read(ROOT / 'EXPECTATIONS.json'))
freeze = b'original invented selfcheck freeze marker; not a frozen native packet'

def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')

write(GOOD / 'ROOT-BINDING.json', {'synthetic_only': True, 'root_authorized_for_case': True,
      'case': 'A', 'freeze_sha256': G.sha(freeze), 'source_commit': '0' * 40,
      'native_binary_sha256': '0' * 64, 'own_target_directory': str(BASE / 'not-a-real-target'), 'inputs': {}})
write(GOOD / 'LAUNCH-CLOCK.json', {'monotonic_ns': '1000000000'})
write(GOOD / 'NATIVE-LEASE-CLOSED.json', {'case': 'A', 'native_input_lease_closed': True,
      'outcome': 'synthetic_only_no_native_lease_ever_existed'})
prior = None
for index, (label, model) in enumerate(models['states']['A'].items()):
    save, capture = f'{2*index+1:04d}', f'{2*index+2:04d}'
    for identity, method in ((save,'click'),(capture,'getAXStateAndScreenshot')):
        write(GOOD / 'trace' / (identity + '-before.json'), {'id': identity, 'method': method, 'monotonic_ns': str(1000000000+int(identity)*2)})
        write(GOOD / 'trace' / (identity + '-after.json'), {'id': identity, 'status': 'returned', 'monotonic_ns': str(1000000001+int(identity)*2)})
    raw = (json.dumps(model, indent=2) + '\n').encode()
    if prior is not None:
        (GOOD / 'work/A.weftforge.recovery').write_bytes(prior)
    (GOOD / 'work/A.weftforge').write_bytes(raw)
    R.retain(GOOD, 'A', label, save, capture, 'synthetic_selfcheck')
    prior = raw
G.grade_run('A', GOOD, freeze, synthetic=True)
checks = []

def bad(name, mutate):
    run = BASE / name
    shutil.copytree(GOOD, run)
    mutate(run)
    try:
        G.grade_run('A', run, freeze, synthetic=True)
    except (AssertionError, ValueError, KeyError, TypeError):
        checks.append(name)
    else:
        raise AssertionError('file selfcheck accepted ' + name)

def replace(path, value):
    path.chmod(0o600)
    write(path, value)

bad('missing-final-native-output', lambda p: (p / 'snapshots/A05-redo.weftforge').unlink())
bad('missing-returned-trace', lambda p: (p / 'trace/0004-after.json').unlink())
bad('changed-final-primary', lambda p: (p / 'work/A.weftforge').write_bytes(b'changed'))
bad('wrong-recovery-bytes', lambda p: ((p / 'snapshots/A05-redo.recovery').chmod(0o600), (p / 'snapshots/A05-redo.recovery').write_bytes(b'changed')))
bad('wrong-snapshot-hash', lambda p: replace(p / 'snapshots/A05-redo.receipt.json', G.parsed(G.read(p / 'snapshots/A05-redo.receipt.json')) | {'snapshot_sha256': 'f'*64}))
bad('reused-save-origin', lambda p: replace(p / 'snapshots/A05-redo.receipt.json', G.parsed(G.read(p / 'snapshots/A05-redo.receipt.json')) | {'native_save_record':'0001'}))
bad('wrong-plan-binding', lambda p: replace(p / 'ROOT-BINDING.json', G.parsed(G.read(p / 'ROOT-BINDING.json')) | {'freeze_sha256': 'f'*64}))
bad('elapsed-cap-overrun', lambda p: replace(p / 'trace/0012-after.json', G.parsed(G.read(p / 'trace/0012-after.json')) | {'monotonic_ns': str(2000000000000)}))
report = {'status': 'passed_synthetic_file_plumbing_only', 'candidate_executed': False,
          'native_executed': False, 'accepted_synthetic_native_save_copies': 6,
          'rejected_bad_output_or_provenance_cases': len(checks), 'checks': checks,
          'all_synthetic_cases_retained': str(BASE.relative_to(ROOT))}
with (ROOT / 'SELF-CHECK-FILES-001.json').open('x') as output:
    json.dump(report, output, indent=2)
    output.write('\n')
print(json.dumps(report))
