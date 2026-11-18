"""One constructor visual-preview case clock/count gate; root must bind PLAN and grant GO first."""
import hashlib
import json
import math
import os
from pathlib import Path
import sys
import time

CASE = Path(__file__).resolve().parent


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


def read(name):
    return json.loads((CASE / name).read_bytes())


def write(name, value):
    with (CASE / name).open('x') as output:
        json.dump(value, output, indent=2, sort_keys=True, allow_nan=False)
        output.write('\n')
        output.flush()
        os.fsync(output.fileno())


def volume():
    retained = 0
    for path in CASE.rglob('*'):
        try:
            if path.is_file() and not path.is_symlink():
                retained += path.lstat().st_size
        except FileNotFoundError:
            pass  # Atomic publication may remove its temporary name during the sample.
    return retained


require(sys.flags.optimize == 0 and os.environ.get('PYTHONOPTIMIZE') in (None, '0'), 'No optimized Python')
plan_raw = (CASE / 'PLAN.json').read_bytes()
plan = json.loads(plan_raw)
bounds = plan['bounds']
require(plan['root_release_ready'] is True, 'Root bindings are pending')
require([bounds[k] for k in ('seconds_from_before_launch', 'action_cutoff_seconds', 'cleanup_reserve',
                            'max_actions', 'max_observations', 'retained_output_bytes')]
        == [300, 240, 60, 20, 40, 536870912], 'Frozen case bounds changed')
go = read('ROOT-NATIVE-GO.json')
require(go['native_case_authorized'] is True
        and go['plan_sha256'] == hashlib.sha256(plan_raw).hexdigest(), 'Missing exact root native GO')
disk = os.statvfs(CASE)
require(disk.f_bavail * disk.f_frsize >= 8000000000, 'Disk below8GB')
command = sys.argv[1]
if command == 'arm':
    require(plan['app'] and plan['bundle_id'], 'Exact own wrapper/bundle binding required')
    for name, binding in plan['required_root_files'].items():
        require(binding['path'] and binding['sha256'], 'Missing binding: ' + name)
        require(hashlib.sha256(Path(binding['path']).read_bytes()).hexdigest() == binding['sha256'], 'Binding changed: ' + name)
    require(hashlib.sha256(Path(plan['native']['path']).read_bytes()).hexdigest() == plan['native']['sha256'], 'Native changed')
    started = time.time()
    write('STARTED.json', {'start_epoch': started, 'started_monotonic_local_only': time.monotonic(),
          'action_cutoff_epoch': started + 240, 'deadline_epoch': started + 300,
          'clock_authority': 'One prelaunch epoch start; never subtract monotonic values from different processes'})
    write('COUNTS.json', {'actions': 0, 'observations': 15, 'reservations': 0,
                         'precharged_cleanup_observations': 15})
else:
    started = read('STARTED.json')
    counts = read('COUNTS.json')
    if command == 'reserve':
        actions, observations, allowance, byte_allowance = int(sys.argv[2]), int(sys.argv[3]), float(sys.argv[4]), int(sys.argv[5])
        require(actions >= 0 and observations >= 0 and math.isfinite(allowance) and 0 < allowance <= 60 and byte_allowance >= 0, 'Invalid reservation')
        require(time.time() + allowance < started['action_cutoff_epoch'], 'Insufficient pre-cutoff call allowance')
        require(counts['actions'] + actions <= 20 and counts['observations'] + observations <= 40, 'Action/observation cap')
        require(volume() + byte_allowance <= 536870912, 'Output allowance exhausted')
        counts.update(actions=counts['actions'] + actions, observations=counts['observations'] + observations,
                      reservations=counts['reservations'] + 1, label=sys.argv[6], reserved_epoch=time.time())
        write('COUNTS-next.json', counts)
        os.replace(CASE/'COUNTS-next.json', CASE/'COUNTS.json')
    else:
        require(command == 'status', 'Expected arm, reserve or status')
    retained = volume()
    finished = time.time()
    require(finished < started['deadline_epoch'] and retained <= 536870912, 'Whole-case time/output cap')
    if command == 'reserve':
        require(finished + allowance < started['action_cutoff_epoch'], 'Call allowance exhausted during reservation persistence')
        require(retained + byte_allowance <= 536870912, 'Output allowance exhausted during reservation persistence')
    print(json.dumps({'elapsed_epoch_seconds': finished - started['start_epoch'], 'counts': counts,
                      'retained_bytes': retained, 'action_cutoff_epoch': started['action_cutoff_epoch'],
                      'deadline_epoch': started['deadline_epoch']}, sort_keys=True))
