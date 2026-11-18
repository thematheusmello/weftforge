"""Frozen direct-child producer, exact app binding and post-getter ownership check."""
import json
import math
import os
from pathlib import Path
import plistlib
import subprocess
import sys
import time

import preview_owner as p

CASE = p.CASE
SOURCES = ['preview_launch.py', 'preview_launch.js', 'preview_sequence.js', 'preview_owner.py',
           'native_call.js', 'clock.py', 'cleanup.py', 'call_recorder.py', 'resource_observer.py', 'inventory.js']


def frozen():
    plan = p.read('PLAN.json')
    p.require(plan['case_path'] == str(CASE) and plan['root_release_ready'] is True, 'Unbound prospective case')
    digest = p.sha(CASE / 'PLAN.json')
    go = p.read('ROOT-NATIVE-GO.json')
    p.require(go['native_case_authorized'] is True and go['plan_sha256'] == digest, 'No exact root native GO')
    p.require([plan['bounds'][key] for key in ['seconds_from_before_launch', 'action_cutoff_seconds', 'cleanup_reserve',
              'max_actions', 'max_observations', 'retained_output_bytes']] == [300, 240, 60, 20, 40, 536870912], 'Frozen bounds changed')
    integration = Path(plan['integration_root']).resolve()
    native, retained, app = Path(plan['native']['path']), Path(plan['retained_native']['path']), Path(plan['app'])
    p.require(integration.is_dir() and CASE.is_relative_to(integration / '.local'), 'Fresh runtime must stay in original integration worktree')
    p.require(native == native.resolve() and retained == retained.resolve() and retained.is_relative_to(integration / '.local')
              and native.parent == app / 'Contents/MacOS' and app.parent == CASE and not native.is_symlink(),
              'Native/app left original integration worktree or is not an actual bundle executable')
    p.require(p.sha(native) == plan['native']['sha256'] == plan['retained_native']['sha256'] == p.sha(retained)
              and native.stat().st_mode & 0o111, 'Retained/native bundle executable changed')
    p.require(p.sha(plan['fixture']['path']) == plan['fixture']['sha256'] and
              Path(plan['fixture']['path']).stat().st_mode & 0o777 == plan['fixture']['mode'], 'Public fixture changed')
    for name in SOURCES:
        binding = plan['required_root_files'][name]
        p.require(binding['path'] == str(CASE / name) and not Path(binding['path']).is_symlink() and
                  p.sha(binding['path']) == binding['sha256'], 'Source binding changed: ' + name)
    for name, binding in plan['required_root_files'].items():
        p.require(p.sha(binding['path']) == binding['sha256'], 'Required binding changed: ' + name)
    plist_path = app / 'Contents/Info.plist'
    info = plistlib.loads(plist_path.read_bytes())
    p.require(info['CFBundleIdentifier'] == plan['bundle_id'] and
              app / 'Contents/MacOS' / info['CFBundleExecutable'] == native and
              p.sha(plist_path) == plan['wrapper']['plist_sha256'] and str(native) == plan['wrapper']['executable'],
              'Exact app/bundle/executable binding changed')
    p.require((CASE / 'captures').is_dir() and not (CASE / 'captures').is_symlink(), 'Missing fresh captures directory')
    return plan, digest


def inventory(plan, child_pid=None, timeout=3):
    argv = ['/usr/bin/osascript', '-l', 'JavaScript', str(CASE / 'inventory.js'), plan['bundle_id']]
    if child_pid is not None:
        argv.append(str(child_pid))
    result = subprocess.run(argv, capture_output=True, text=True, timeout=timeout, check=True)
    value = json.loads(result.stdout)
    return value


def main_window(value, pid):
    windows = [w for w in value['windows'] if w['kCGWindowOwnerPID'] == pid and
               w['kCGWindowLayer'] == 0 and w['kCGWindowName'] == 'WeftForge' and w['kCGWindowIsOnscreen'] is True and
               w['kCGWindowBounds']['Width'] > 100 and w['kCGWindowBounds']['Height'] > 100]
    p.require(len(windows) == 1, 'Missing or ambiguous owned content window')
    return windows[0]


def live_monitors(plan, started, digest, wait=False):
    limit = min(time.time() + 8, started['action_cutoff_epoch'] - 120)
    names = ['WATCHDOG', 'RESOURCE']
    while wait and not all((CASE / (name + '-READY.json')).exists() for name in names) and time.time() < limit:
        p.require(not (CASE / 'RESOURCE-ALARM.json').exists(), 'Resource alarm before monitor readiness')
        time.sleep(.05)
    result = {}
    for name in names:
        ready = p.read(name + '-READY.json')
        p.require(ready['case'] == plan['case'] and ready['plan_sha256'] == digest and
                  all(ready[key] == started[key] for key in ['start_epoch', 'action_cutoff_epoch', 'deadline_epoch']) and
                  started['start_epoch'] <= ready['ready_epoch'] <= time.time() and
                  type(ready['pid']) is int and ready['pid'] > 0, 'Mismatched same-case READY: ' + name)
        file = 'cleanup.py' if name == 'WATCHDOG' else 'resource_observer.py'
        expected = [plan['python'], '-B', str(CASE / file)] + (['--watchdog'] if name == 'WATCHDOG' else [])
        observed = subprocess.run(['/bin/ps', '-p', str(ready['pid']), '-ww', '-o', 'command='],
                                  capture_output=True, text=True, timeout=1, check=True)
        p.require(observed.stdout.strip() == ' '.join(expected), 'Original monitor PID/command not live: ' + name)
        result[name] = ready
    p.require(result['WATCHDOG']['pid'] != result['RESOURCE']['pid'] and not (CASE / 'RESOURCE-ALARM.json').exists(),
              'Distinct live monitors unavailable')
    return result


def lease_from_inventory(plan, started, value, child_pid, pgid):
    p.require(len(value['apps']) == 1, 'Exactly one fresh owned native process required; extras refuse GUI')
    app = value['apps'][0]
    p.require(app['pid'] == child_pid and app['bundle_id'] == plan['bundle_id'] and app['app_path'] == plan['app'] and
              app['executable'] == plan['native']['path'] and 'launch_time' in app and
              (app['launch_time'] is None or type(app['launch_time']) in (int, float) and math.isfinite(app['launch_time'])) and
              p.sha(app['executable']) == plan['native']['sha256'], 'Launch identity mismatch')
    child = p.direct_child_identity(plan, started)
    from resource_observer import probe
    observed = probe(child_pid, plan['native']['path'])
    checked = time.time()
    p.require(child['pid'] == child_pid and child['pgid'] == pgid and observed is not None and
              observed['pgid'] == pgid and observed['ppid'] == child['producer_pid'] and
              isinstance(observed['start_token'], str) and bool(observed['start_token'].strip()) and
              child['created_epoch'] <= checked, 'Original direct-child OS identity unavailable or changed')
    window = main_window(value, child_pid)
    return dict(app, process_group=pgid, source_binary_sha256=plan['native']['sha256'],
                native_start_token=observed['start_token'], parent_pid=observed['ppid'],
                native_identity_observed_epoch=checked, direct_child_created_epoch=child['created_epoch'],
                window_id=window['kCGWindowNumber'], window_geometry=window['kCGWindowBounds'], window_metadata=window)


def same_lease(actual, expected):
    return all(actual[key] == expected[key] for key in ['pid', 'native_start_token', 'parent_pid', 'direct_child_created_epoch',
               'executable', 'bundle_id', 'app_path',
               'process_group', 'source_binary_sha256', 'window_id', 'window_geometry'])


def startup_lease(plan, started, child, pgid, deadline):
    for attempt in range(1, 4):
        p.require(child.poll() is None and not (CASE / 'LAUNCH-STOP.json').exists() and
                  not (CASE / 'RESOURCE-ALARM.json').exists(), 'Original native child ended/stopped/alarmed before readiness')
        remaining = deadline - time.time()
        p.require(remaining > 0, 'Original native startup exceeded eight-second readiness bound')
        value = inventory(plan, child.pid, timeout=min(2, remaining))
        # Keep the actual failed observation too; validation must never erase its identity/count.
        p.write('STARTUP-INVENTORY-' + str(attempt).zfill(3) + '.json', value)
        p.require(time.time() < deadline, 'Startup inventory retention exceeded original readiness deadline')
        p.require(isinstance(value, dict) and all(isinstance(value.get(key), list) for key in
                  ['apps', 'child_apps', 'windows']), 'Malformed startup inventory')
        apps, children = value['apps'], value['child_apps']
        p.require(len(apps) <= 1 and len(children) <= 1, 'Extra native identity; no startup retry or GUI')
        expected = {'pid': child.pid, 'bundle_id': plan['bundle_id'], 'app_path': plan['app'],
                    'executable': plan['native']['path']}
        for app in apps + children:
            p.require(isinstance(app, dict) and type(app.get('pid')) is int and
                      all(app.get(key) == expected[key] for key in expected), 'Wrong native identity; no startup retry or GUI')
            p.require('launch_time' in app and (app['launch_time'] is None or
                      type(app['launch_time']) in (int, float) and math.isfinite(app['launch_time'])),
                      'Malformed raw native launch date; no startup retry or GUI')
        if apps:
            p.require(children == apps, 'Missing or inconsistent by-child identity; no startup retry or GUI')
            windows = [w for w in value['windows'] if w['kCGWindowOwnerPID'] == child.pid and
                       w['kCGWindowLayer'] == 0 and w['kCGWindowName'] == 'WeftForge' and w['kCGWindowIsOnscreen'] is True and
                       w['kCGWindowBounds']['Width'] > 100 and w['kCGWindowBounds']['Height'] > 100]
            p.require(len(windows) <= 1, 'Ambiguous owned content window; no startup retry or GUI')
            if windows:
                lease = lease_from_inventory(plan, started, value, child.pid, pgid)
                p.require(child.poll() is None and time.time() < deadline, 'Original native child ended or readiness completed late')
                return value, lease
        p.require(attempt < 3, 'Original native child not ready after three retained inventories; no GUI')
        time.sleep(min(2, max(0, deadline - time.time())))
    raise RuntimeError('Unreachable startup readiness')


def native_producer(plan, started, digest):
    # The group exists and its complete lease is visible before any native child can be created.
    try:
        os.setpgid(0, 0)
    except PermissionError:
        p.require(os.getpgrp() == os.getpid(), 'Cannot own a dedicated producer process group')
    pid, pgid = os.getpid(), os.getpgrp()
    p.require(pid == pgid, 'Producer group is not exclusively owned')
    from resource_observer import probe
    observed = probe(pid, os.path.realpath(plan['python']))
    p.require(observed is not None and observed['pgid'] == pgid, 'Producer identity unavailable')
    producer = {'case': plan['case'], 'plan_sha256': digest, 'start_epoch': started['start_epoch'],
                'pid': pid, 'pgid': pgid, 'start_token': observed['start_token'], 'executable': os.path.realpath(plan['python']),
                'command': [plan['python'], '-B', str(CASE / 'preview_launch.py'), 'launch']}
    p.write('LAUNCH-PRODUCER-LEASE.json', producer)
    child, signals, failure, ready = None, [], None, False
    try:
        p.require(not (CASE / 'LAUNCH-STOP.json').exists(), 'Permanent stop barrier; no native child')
        p.require(not (CASE / 'LAUNCH-INTENT.json').exists(), 'No second launch; never replay')
        counts = p.read('LAUNCH-GATE.json')['counts']
        p.require(counts['actions'] == 1 and counts['observations'] == 31 and counts['label'] == 'LAUNCH',
                  'Missing actual precharged startup reservation')
        live_monitors(plan, started, digest, wait=True)
        before = inventory(plan)
        p.write('PRELAUNCH-ABSENCE.json', before)
        p.require(not before['apps'] and not before['windows'], 'Unique bundle already present; no native child')
        p.write('LAUNCH-INTENT.json', {'case': plan['case'], 'plan_sha256': digest, 'start_epoch': started['start_epoch'],
                'one_launch_only': True, 'intent_epoch': time.time(), 'executable': plan['native']['path'],
                'binary_sha256': plan['native']['sha256'], 'bundle_id': plan['bundle_id'], 'app': plan['app'],
                'producer_pid': pid, 'producer_pgid': pgid, 'method': 'direct Popen; no LaunchServices request'})
        p.require(not (CASE / 'LAUNCH-STOP.json').exists() and time.time() + 120 < started['action_cutoff_epoch'],
                  'Stop/clock gate before the single direct Popen')
        argv = [plan['native']['path'], plan['fixture']['path']]
        with (CASE / 'native.stdout.log').open('xb') as stdout, (CASE / 'native.stderr.log').open('xb') as stderr:
            startup_begin = time.time()
            startup_deadline = min(startup_begin + 8, started['action_cutoff_epoch'] - 120)
            child = subprocess.Popen(argv, cwd=CASE, stdout=stdout, stderr=stderr, process_group=pgid)
            p.write('DIRECT-CHILD.json', {'case': plan['case'], 'plan_sha256': digest, 'start_epoch': started['start_epoch'],
                    'producer_pid': pid, 'producer_start_token': producer['start_token'],
                    'pid': child.pid, 'pgid': pgid, 'created_epoch': time.time(),
                    'executable': plan['native']['path'], 'sha256': plan['native']['sha256'],
                    'startup_begin_epoch': startup_begin, 'startup_deadline_epoch': startup_deadline})
            value, lease = startup_lease(plan, started, child, pgid, startup_deadline)
            p.write('OWNED-IDENTITY-001.json', lease)
            p.write('OWNED-IDENTITY.json', lease)
            p.write('LAUNCH-INVENTORY.json', value)
            p.write('LAUNCH.json', {'argv': argv, 'pid': child.pid, 'pgid': pgid, 'direct_owned_child': True})
            p.require(time.time() < startup_deadline, 'Startup receipt retention exceeded original readiness deadline')
            p.write('LAUNCH-READY.json', {'case': plan['case'], 'plan_sha256': digest, 'start_epoch': started['start_epoch'],
                    'app': plan['app'], 'lease': lease, 'producer_pid': pid, 'producer_pgid': pgid, 'ready_epoch': time.time(),
                    'startup_begin_epoch': startup_begin, 'startup_deadline_epoch': startup_deadline})
            ready_persisted = time.time()
            p.require(ready_persisted < startup_deadline, 'Final READY persistence exceeded original readiness deadline')
            # This witness may persist later; its criterion is the completed READY write observed above.
            p.write('LAUNCH-READINESS-ADMISSION.json', {'case': plan['case'], 'plan_sha256': digest,
                    'start_epoch': started['start_epoch'], 'producer_pid': pid, 'producer_pgid': pgid,
                    'producer_start_token': producer['start_token'], 'child_pid': child.pid,
                    'ready_sha256': p.sha(CASE / 'LAUNCH-READY.json'), 'startup_begin_epoch': startup_begin,
                    'startup_deadline_epoch': startup_deadline, 'ready_persistence_observed_epoch': ready_persisted})
            p.require(time.time() < started['deadline_epoch'], 'Readiness witness persisted after whole-case deadline')
            ready = True
            while child.poll() is None and not (CASE / 'LAUNCH-STOP.json').exists() and time.time() < started['action_cutoff_epoch']:
                p.require(not (CASE / 'RESOURCE-ALARM.json').exists(), 'Resource alarm during direct native lease')
                time.sleep(.05)
    except BaseException as error:
        failure = type(error).__name__ + ': ' + str(error)
    finally:
        if child is not None and child.poll() is None:
            try:
                child.wait(timeout=2)
            except subprocess.TimeoutExpired:
                child.terminate()
                signals.append({'pid': child.pid, 'pgid': pgid, 'signal': 'SIGTERM', 'target': 'native_child'})
                try:
                    child.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    child.kill()
                    signals.append({'pid': child.pid, 'pgid': pgid, 'signal': 'SIGKILL', 'target': 'native_child'})
                    child.wait(timeout=2)
        p.write('CLEANUP-PRODUCER.json', {'case': plan['case'], 'start_epoch': started['start_epoch'], 'signals': signals,
                'absent': child is None or child.returncode is not None, 'direct_child_reaped': True})
        p.write('LAUNCH-PRODUCER-TERMINAL.json', {'case': plan['case'], 'pid': pid, 'pgid': pgid, 'ready': ready,
                'child_pid': None if child is None else child.pid, 'child_exit': None if child is None else child.returncode,
                'failure': failure, 'signals': signals, 'terminal_epoch': time.time()})
    p.require(failure is None and ready and child.returncode == 0 and not signals, 'Direct producer negative: ' + str(failure))
    return {'status': 'DIRECT_CHILD_REAPED_NORMAL_EXIT', 'app': plan['app'], 'pid': child.pid}


def bound(plan, started, digest):
    limit = min(time.time() + 8, started['action_cutoff_epoch'] - 120)
    while not (CASE / 'LAUNCH-READINESS-ADMISSION.json').exists() and time.time() < limit:
        p.require(not (CASE / 'LAUNCH-PRODUCER-TERMINAL.json').exists() and not (CASE / 'RESOURCE-ALARM.json').exists(),
                  'Producer ended/alarmed before launch readiness')
        time.sleep(.05)
    ready, producer = p.read('LAUNCH-READY.json'), p.read('LAUNCH-PRODUCER-LEASE.json')
    admission = p.read('LAUNCH-READINESS-ADMISSION.json')
    p.require(ready['case'] == plan['case'] and ready['plan_sha256'] == digest and ready['app'] == plan['app'] and
              ready['start_epoch'] == started['start_epoch'] and ready['producer_pid'] == producer['pid'] and
              ready['producer_pgid'] == producer['pgid'] and producer['pid'] == producer['pgid'] and
              producer['case'] == plan['case'] and producer['plan_sha256'] == digest and producer['start_epoch'] == started['start_epoch'] and
              producer['executable'] == os.path.realpath(plan['python']) and
              producer['command'] == [plan['python'], '-B', str(CASE / 'preview_launch.py'), 'launch'], 'Producer READY binding mismatch')
    begin, deadline, completed = [admission.get(key) for key in
                                 ['startup_begin_epoch', 'startup_deadline_epoch', 'ready_persistence_observed_epoch']]
    p.require(all(type(value) in (int, float) and math.isfinite(value) for value in [begin, deadline, completed, ready['ready_epoch']]) and
              admission['case'] == plan['case'] and admission['plan_sha256'] == digest and
              admission['start_epoch'] == started['start_epoch'] and admission['producer_pid'] == producer['pid'] and
              admission['producer_pgid'] == producer['pgid'] and admission['producer_start_token'] == producer['start_token'] and
              admission['child_pid'] == ready['lease']['pid'] and admission['ready_sha256'] == p.sha(CASE / 'LAUNCH-READY.json') and
              begin == ready['startup_begin_epoch'] and deadline == ready['startup_deadline_epoch'] and
              deadline == min(begin + 8, started['action_cutoff_epoch'] - 120) and
              started['start_epoch'] <= begin <= ready['ready_epoch'] <= completed < deadline and completed <= time.time(),
              'Missing timely same-producer post-READY-persistence admission')
    monitors = live_monitors(plan, started, digest)
    from resource_observer import probe
    actual = probe(producer['pid'], os.path.realpath(plan['python']))
    p.require(actual is not None and actual['start_token'] == producer['start_token'], 'Original producer is not live')
    lease = p.read('OWNED-IDENTITY.json')
    current = inventory(plan)
    p.write('BIND-INVENTORY.json', current)
    observed = lease_from_inventory(plan, started, current, ready['lease']['pid'], producer['pgid'])
    p.require(lease == p.read('OWNED-IDENTITY-001.json') == ready['lease'] and same_lease(observed, lease),
              'Observed original process/window lease changed')
    p.require(time.time() + 120 < started['action_cutoff_epoch'] and p.bounds(started)['within_bounds'], 'Bound case no longer admissible')
    return {'case': plan['case'], 'case_path': str(CASE), 'app': plan['app'], 'plan_sha256': digest, 'started': started, 'inventory': current,
            'fixture_basename': Path(plan['fixture']['path']).name,
            'lease': lease, 'producer': producer, 'monitors': monitors, 'readiness_admission': admission, 'checked_epoch': time.time(),
            'sources_exact': True, 'same_case_ready_and_processes_live': True}


def main():
    plan, digest = frozen()
    phase = sys.argv[1]
    if phase == 'arm':
        p.require(not any((CASE / name).exists() for name in ['STARTED.json', 'COUNTS.json', 'LAUNCH-GATE.json', 'LAUNCH-INTENT.json',
                  'LAUNCH-PRODUCER-LEASE.json', 'LAUNCH-STOP.json', 'OWNED-IDENTITY.json', 'WATCHDOG-READY.json', 'RESOURCE-READY.json',
                  'DONE', 'RESOURCE-STOP']), 'Case is not fresh; never rearm')
        subprocess.run([plan['python'], '-B', str(CASE / 'clock.py'), 'arm'], check=True, timeout=3, capture_output=True)
        started = p.read('STARTED.json')
        p.validate_started(plan, started)
        gate = subprocess.run([plan['python'], '-B', str(CASE / 'clock.py'), 'reserve', '1', '16', '60', '8388608', 'LAUNCH'],
                              capture_output=True, text=True, timeout=3, check=True)
        p.write('LAUNCH-GATE.json', json.loads(gate.stdout))
        print(json.dumps({'case': plan['case'], 'app': plan['app'], 'plan_sha256': digest, 'started': started}, sort_keys=True))
        return
    started = p.read('STARTED.json')
    p.validate_started(plan, started)
    if phase == 'launch':
        result = native_producer(plan, started, digest)
    elif phase == 'bind':
        result = bound(plan, started, digest)
    else:
        p.require(phase == 'post-getter', 'Expected arm, launch, bind or post-getter')
        subprocess.run([plan['python'], '-B', str(CASE / 'clock.py'), 'reserve', '0', '1', '60', '65536', 'POST-GETTER'],
                       capture_output=True, text=True, timeout=3, check=True)
        lease, producer = p.read('OWNED-IDENTITY.json'), p.read('LAUNCH-PRODUCER-LEASE.json')
        value = inventory(plan)
        observed = lease_from_inventory(plan, started, value, lease['pid'], producer['pgid'])
        p.require(same_lease(observed, lease) and not (CASE / 'LAUNCH-STOP.json').exists() and
                  not (CASE / 'RESOURCE-ALARM.json').exists(), 'Post-getter ownership changed; no navigation or replay')
        p.write('POST-GETTER-INVENTORY.json', value)
        result = {'app': plan['app'], 'pid': lease['pid'], 'window_id': lease['window_id'],
                  'same_observed_lease': True, 'checked_epoch': time.time()}
    print(json.dumps(result, sort_keys=True))


if __name__ == '__main__':
    p.require(sys.flags.optimize == 0 and os.environ.get('PYTHONOPTIMIZE') in (None, '0'), 'Python optimization forbidden')
    main()
