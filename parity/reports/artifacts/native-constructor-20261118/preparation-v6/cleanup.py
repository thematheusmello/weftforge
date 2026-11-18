"""Stop the owned direct producer group before final same-case app absence."""
import json
import math
import os
import signal
import subprocess
import sys
import time

import preview_owner as p

CASE = p.CASE
PLAN = p.read('PLAN.json')


def inventory():
    value = subprocess.run(['/usr/bin/osascript', '-l', 'JavaScript', str(CASE / 'inventory.js'), PLAN['bundle_id']],
                           capture_output=True, text=True, timeout=3, check=True)
    result = json.loads(value.stdout)
    p.require(isinstance(result.get('apps'), list) and isinstance(result.get('windows'), list), 'Malformed cleanup inventory')
    return result


def group_snapshot(pgid):
    result = subprocess.run(['/bin/ps', '-ww', '-axo', 'pid=,pgid=,ppid=,lstart=,command='], capture_output=True, text=True,
                            timeout=2, check=True)
    rows, seen = [], False
    for line in result.stdout.splitlines():
        fields = line.split(maxsplit=8)
        seen |= len(fields) >= 3 and all(part.isdigit() for part in fields[:3])
        if len(fields) >= 2 and fields[0].isdigit() and fields[1].isdigit() and int(fields[1]) == pgid:
            p.require(len(fields) == 9 and fields[2].isdigit(), 'Malformed owned process-group snapshot')
            rows.append({'pid': int(fields[0]), 'pgid': pgid, 'ppid': int(fields[2]),
                         'start_token': ' '.join(fields[3:8]), 'command': fields[8]})
    p.require(seen, 'Global process-group snapshot unavailable')
    return rows


def verify_group(rows, producer):
    expected_native = PLAN['native']['path'] + ' ' + PLAN['fixture']['path']
    expected_inventory = '/usr/bin/osascript -l JavaScript ' + str(CASE / 'inventory.js') + ' ' + PLAN['bundle_id']
    child = p.read('DIRECT-CHILD.json') if (CASE / 'DIRECT-CHILD.json').exists() else None
    producer_present = any(row['pid'] == producer['pid'] and row['start_token'].split() == producer['start_token'].split() for row in rows)
    native_count = 0
    for row in rows:
        if row['pid'] == producer['pid']:
            p.require(row['command'] == ' '.join(producer['command']) and
                      row['start_token'].split() == producer['start_token'].split(), 'Producer PID reused or command changed')
        elif row['command'] == expected_native:
            native_count += 1
            direct = p.direct_child_identity(PLAN, p.read('STARTED.json'))
            p.require(row['pid'] == direct['pid'] and row['pgid'] == direct['pgid'] and
                      p.sha(PLAN['native']['path']) == PLAN['native']['sha256'], 'Owned native PID/group/binary changed')
            leases = []
            for name in ['OWNED-IDENTITY-001.json', 'OWNED-IDENTITY.json']:
                try:
                    lease = p.read(name)
                    p.require(isinstance(lease, dict), 'Malformed retained native lease')
                    p.direct_child_identity(PLAN, p.read('STARTED.json'), lease)
                    leases.append(lease)
                except (FileNotFoundError, ValueError, AttributeError, KeyError, RuntimeError):
                    pass
            p.require(all(row['start_token'].split() == lease['native_start_token'].split() for lease in leases) if leases else
                      producer_present and row['ppid'] == producer['pid'], 'Native PID reused or original parent unverified')
            p.require(row['ppid'] == producer['pid'] or (leases and not producer_present and row['ppid'] == 1),
                      'Native parent changed outside verified producer shutdown')
        else:
            allowed = [expected_inventory] + ([expected_inventory + ' ' + str(child['pid'])] if child else [])
            p.require(row['command'] in allowed and row['ppid'] == producer['pid'], 'Refuse unrelated group member')
    p.require(native_count <= 1, 'Unexpected additional native group member')


def producer_barrier():
    signals, stop_error = [], None
    stop = {'case': PLAN['case'], 'plan_sha256': p.sha(CASE / 'PLAN.json'), 'start_epoch': p.read('STARTED.json')['start_epoch']}
    try:
        p.write('LAUNCH-STOP.json', stop)
    except FileExistsError:
        p.require(p.read('LAUNCH-STOP.json') == stop, 'Conflicting permanent stop barrier')
    except OSError as error:
        stop_error = type(error).__name__ + ': ' + str(error)
    path = CASE / 'LAUNCH-PRODUCER-LEASE.json'
    if not path.exists():
        # The producer publishes its lease before checking this permanent barrier or creating a child.
        return {'quiescent': stop_error is None, 'method': 'permanent pre-publication stop barrier',
                'signals': signals, 'stop_error': stop_error}
    producer = p.read('LAUNCH-PRODUCER-LEASE.json')
    p.require(producer['case'] == PLAN['case'] and producer['plan_sha256'] == stop['plan_sha256'] and
              producer['start_epoch'] == stop['start_epoch'] and type(producer['pid']) is int and producer['pid'] > 0 and
              producer['pid'] == producer['pgid'] and producer['executable'] == os.path.realpath(PLAN['python']) and
              producer['command'] == [PLAN['python'], '-B', str(CASE / 'preview_launch.py'), 'launch'],
              'Producer group lease is not same-case')
    # Give a responsive producer time to reap its child and persist its terminal after normal Quit/stop.
    limit = min(time.time() + 2, p.read('STARTED.json')['deadline_epoch'])
    while not (CASE / 'LAUNCH-PRODUCER-TERMINAL.json').exists() and stop_error is None and time.time() < limit:
        time.sleep(.05)
    time.sleep(.1)
    rows = group_snapshot(producer['pgid'])
    verify_group(rows, producer)
    if rows:
        rows = group_snapshot(producer['pgid'])
        verify_group(rows, producer)
        if rows:
            try:
                os.killpg(producer['pgid'], signal.SIGTERM)
                signals.append({'pid': producer['pid'], 'pgid': producer['pgid'], 'signal': 'SIGTERM', 'target': 'producer_group'})
            except ProcessLookupError:
                pass
        time.sleep(.3)
        rows = group_snapshot(producer['pgid'])
        verify_group(rows, producer)
        if rows:
            try:
                os.killpg(producer['pgid'], signal.SIGKILL)
                signals.append({'pid': producer['pid'], 'pgid': producer['pgid'], 'signal': 'SIGKILL', 'target': 'producer_group'})
            except ProcessLookupError:
                pass
            time.sleep(.1)
            rows = group_snapshot(producer['pgid'])
    return {'quiescent': not rows, 'method': 'verified original producer group absence',
            'producer': producer, 'signals': signals, 'stop_error': stop_error, 'remaining': rows}


def intent_owned(app):
    intent, before, started = p.read('LAUNCH-INTENT.json'), p.read('PRELAUNCH-ABSENCE.json'), p.read('STARTED.json')
    child = p.direct_child_identity(PLAN, started)
    return (not before['apps'] and not before['windows'] and intent['case'] == PLAN['case']
            and intent['plan_sha256'] == p.sha(CASE / 'PLAN.json') and intent['start_epoch'] == started['start_epoch']
            and intent['one_launch_only'] is True and intent['executable'] == PLAN['native']['path']
            and intent['binary_sha256'] == PLAN['native']['sha256'] and intent['bundle_id'] == PLAN['bundle_id']
            and intent['app'] == PLAN['app'] and type(app['pid']) is int and app['pid'] == child['pid']
            and app['executable'] == PLAN['native']['path'] and app['bundle_id'] == PLAN['bundle_id']
            and app['app_path'] == PLAN['app'] and intent['intent_epoch'] >= started['start_epoch']
            and 'launch_time' in app and (app['launch_time'] is None or type(app['launch_time']) in (int, float) and math.isfinite(app['launch_time']))
            and p.sha(app['executable']) == PLAN['native']['sha256'])


def owned_observation(app, lease=None):
    from resource_observer import probe, same_process
    child = p.direct_child_identity(PLAN, p.read('STARTED.json'))
    p.require(intent_owned(app), 'No complete direct-child intent for native observation')
    observed = probe(app['pid'], PLAN['native']['path'])
    p.require(observed is not None and observed['pgid'] == child['pgid'] and observed['ppid'] == child['producer_pid'] and
              isinstance(observed['start_token'], str) and bool(observed['start_token'].strip()), 'Native token/group/parent unavailable')
    if lease is not None:
        p.require(same_process(observed, lease), 'Native PID reused or identity changed before cleanup signal')
    else:
        producer = p.read('LAUNCH-PRODUCER-LEASE.json')
        current = probe(producer['pid'], producer['executable'])
        p.require(current is not None and current['start_token'] == producer['start_token'] and current['pgid'] == producer['pgid'],
                  'Cannot recover native token without its live original producer')
    return observed


def recover_lease(value):
    p.require(len(list(CASE.glob('OWNED-IDENTITY-???.json'))) <= 1, 'At most one original native lease')
    valid, corrupt = [], []
    for name in ['OWNED-IDENTITY-001.json', 'OWNED-IDENTITY.json']:
        if not (CASE / name).exists():
            continue
        try:
            lease = p.read(name)
            p.require(isinstance(lease, dict), 'Malformed lease identity')
            p.direct_child_identity(PLAN, p.read('STARTED.json'), lease)
            valid.append(lease)
        except (ValueError, RuntimeError, KeyError, TypeError):
            corrupt.append(name)
    p.require(not valid or all(lease == valid[0] for lease in valid), 'Conflicting complete native leases')
    lease = valid[0] if valid else None
    if lease is None and value['apps']:
        p.require(len(value['apps']) == 1 and intent_owned(value['apps'][0]), 'No exact single-launch intent for lease recovery')
        app = value['apps'][0]
        observed = owned_observation(app)
        lease = dict(app, source_binary_sha256=PLAN['native']['sha256'], process_group=observed['pgid'],
                     native_start_token=observed['start_token'], parent_pid=observed['ppid'], native_identity_observed_epoch=time.time(),
                     direct_child_created_epoch=p.read('DIRECT-CHILD.json')['created_epoch'],
                     recovered_after_launch_retention_fault=True)
    for name in corrupt:
        p.require(lease is not None or not value['apps'], 'Cannot quarantine unresolved native lease')
        p.require(not (CASE / ('CORRUPT-' + name)).exists(), 'Preserve previous corrupt-lease evidence')
        os.rename(CASE / name, CASE / ('CORRUPT-' + name))
    if lease is not None:
        for name in ['OWNED-IDENTITY-001.json', 'OWNED-IDENTITY.json']:
            try:
                p.write(name, lease)
            except FileExistsError:
                p.require(p.read(name) == lease, 'Conflicting single-launch lease')
    return corrupt


def close():
    barrier = producer_barrier()
    before = inventory()
    corrupt = recover_lease(before)
    result = {'case': PLAN['case'], 'start_epoch': p.read('STARTED.json')['start_epoch'], 'before': before,
              'method': 'producer barrier/group shutdown then strict same-case app cleanup', 'signals': list(barrier['signals']),
              'producer_barrier': barrier, 'producer_quiescent': barrier['quiescent'], 'corrupt_leases_retained': corrupt,
              'unexpected_case_apps': before['apps']}
    # An unexpected same-case app is negative evidence; only exact intent/native/bundle/time matches may be closed.
    for app in before['apps']:
        p.require(intent_owned(app), 'Refuse unrelated app during cleanup')
        check = inventory()
        p.require(any(all(x[key] == app[key] for key in ['pid', 'executable', 'bundle_id', 'app_path']) and intent_owned(x)
                      for x in check['apps']), 'Same-case app identity changed before signal')
        observed = owned_observation(app, p.read('OWNED-IDENTITY.json'))
        os.kill(app['pid'], signal.SIGTERM)
        result['signals'].append({'pid': app['pid'], 'launch_time': app['launch_time'], 'native_start_token': observed['start_token'],
                                  'pgid': observed['pgid'], 'parent_pid': observed['ppid'], 'signal': 'SIGTERM', 'target': 'same_case_app'})
    time.sleep(.3)
    after = inventory()
    for app in after['apps']:
        p.require(intent_owned(app), 'Refuse unrelated app after TERM')
        check = inventory()
        p.require(any(all(x[key] == app[key] for key in ['pid', 'executable', 'bundle_id', 'app_path']) and intent_owned(x)
                      for x in check['apps']), 'Same-case app identity changed before KILL')
        observed = owned_observation(app, p.read('OWNED-IDENTITY.json'))
        os.kill(app['pid'], signal.SIGKILL)
        result['signals'].append({'pid': app['pid'], 'launch_time': app['launch_time'], 'native_start_token': observed['start_token'],
                                  'pgid': observed['pgid'], 'parent_pid': observed['ppid'], 'signal': 'SIGKILL', 'target': 'same_case_app'})
    if after['apps']:
        time.sleep(.1)
    result['after'] = inventory()
    result['absent'] = not result['after']['apps'] and not result['after']['windows'] and barrier['quiescent']
    result['elapsed_since_case_start'] = time.time() - result['start_epoch']
    p.write('CLEANUP-WATCHDOG.json' if '--watchdog' in sys.argv else 'CLEANUP.json', result)
    return result


if __name__ == '__main__':
    p.require(sys.flags.optimize == 0 and os.environ.get('PYTHONOPTIMIZE') in (None, '0'), 'No optimized Python')
    if '--watchdog' in sys.argv:
        started = p.read('STARTED.json')
        p.write('WATCHDOG-READY.json', {'pid': os.getpid(), 'ready_epoch': time.time(), 'case': PLAN['case'],
                'plan_sha256': p.sha(CASE / 'PLAN.json'), **{key: started[key] for key in
                ['start_epoch', 'action_cutoff_epoch', 'deadline_epoch']}})
        while time.time() < started['action_cutoff_epoch'] and not (CASE / 'DONE').exists():
            time.sleep(.5)
        if not (CASE / 'DONE').exists():
            print(json.dumps(close()), flush=True)
        p.write('WATCHDOG-TERMINAL.json', {'status': 'watchdog_ended', 'closed_epoch': time.time(),
                'same_case_cleanup_record_present': (CASE / 'CLEANUP-WATCHDOG.json').exists()})
    else:
        print(json.dumps(close()), flush=True)
