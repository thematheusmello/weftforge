"""Observe only the one leased native process; never author or signal them."""
import ctypes
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import preview_owner as p

CASE = Path(__file__).resolve().parent


def write(name, value):
    with (CASE / name).open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())


def volume():
    retained = 0
    for path in CASE.rglob('*'):
        try:
            if path.is_file() and not path.is_symlink():
                retained += path.lstat().st_size
        except FileNotFoundError:
            pass  # Atomic publication may remove its temporary name during the sample.
    return retained


def failure(now, deadline, size, gap, rss, cap):
    if now >= deadline:
        return 'whole-case deadline'
    if size > 536870912:
        return 'whole-case output cap'
    if gap > 5:
        return 'RSS gap over5s; memory UNKNOWN'
    if cap is not None and rss > cap:
        return 'native RSS cap'
    return None


def probe(pid, expected):
    # Reuse the prior independent observer's libproc + ps identity/RSS probe.
    lib = ctypes.CDLL('/usr/lib/libproc.dylib', use_errno=True)
    lib.proc_pidpath.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_uint32]
    lib.proc_pidpath.restype = ctypes.c_int
    buffer = ctypes.create_string_buffer(4096)
    count = lib.proc_pidpath(pid, buffer, len(buffer))
    result = subprocess.run(['/bin/ps', '-p', str(pid), '-o', 'rss=', '-o', 'pgid=', '-o', 'ppid=', '-o', 'lstart='],
                            capture_output=True, text=True, timeout=1)
    if not result.stdout.strip() and result.returncode in (0, 1):
        return None
    if result.returncode != 0 or count <= 0 or os.path.realpath(buffer.value.decode()) != expected:
        raise RuntimeError('Leased PID executable/RSS observation mismatch')
    fields = result.stdout.strip().split(maxsplit=3)
    if len(fields) != 4 or not all(value.isdigit() for value in fields[:3]) or not fields[3].strip():
        raise RuntimeError('Leased PID RSS/group/parent/start schema')
    return {'executable': expected, 'start_token': fields[3], 'rss_bytes': int(fields[0]) * 1024,
            'pgid': int(fields[1]), 'ppid': int(fields[2])}


def same_process(observed, lease):
    return (observed is not None and isinstance(lease.get('native_start_token'), str) and bool(lease['native_start_token'].strip()) and
            observed['executable'] == lease['executable'] and observed['start_token'] == lease['native_start_token'] and
            observed['pgid'] == lease['process_group'] and observed['ppid'] == lease['parent_pid'])


def main():
    if sys.argv[1:] == ['--selfcheck']:
        assert failure(1, 2, 536870912, 5, 10, 10) is None
        assert failure(2, 2, 0, 0, 0, None) == 'whole-case deadline'
        assert failure(1, 2, 536870913, 0, 0, None) == 'whole-case output cap'
        assert failure(1, 2, 0, 5.0001, 0, None) == 'RSS gap over5s; memory UNKNOWN'
        assert failure(1, 2, 0, 0, 11, 10) == 'native RSS cap'
        observed = probe(os.getpid(), os.path.realpath(sys.executable))
        assert observed['rss_bytes'] > 0 and observed['start_token']
        print(json.dumps({'status': 'six_control_checks_passed', 'native_invocations': 0,
                          'own_interpreter_probe': observed}, sort_keys=True))
        return
    plan = json.loads((CASE/'PLAN.json').read_text())
    started = json.loads((CASE/'STARTED.json').read_text())
    expected = plan['native']['path']
    if hashlib.sha256(Path(expected).read_bytes()).hexdigest() != plan['native']['sha256']:
        raise RuntimeError('Bound native changed')
    write('RESOURCE-READY.json', {'pid': os.getpid(), 'ready_epoch': time.time(), 'case': plan['case'],
          'plan_sha256': hashlib.sha256((CASE/'PLAN.json').read_bytes()).hexdigest(),
          **{key: started[key] for key in ['start_epoch', 'action_cutoff_epoch', 'deadline_epoch']},
          'scope': 'Native PID RSS only, not helper/driver/descendant or process-wide performance evidence'})
    samples, cap, peak, last_pid, last_time, identities, absent = [], None, 0, None, None, {}, set()
    with (CASE/'RSS-SAMPLES.jsonl').open('x') as log:
        while not (CASE/'RESOURCE-STOP').exists():
            now = time.time()
            reason = failure(now, started['deadline_epoch'], volume(), 0, 0, None)
            in_flight = CASE/'CALL-IN-FLIGHT.json'
            try:
                call = json.loads(in_flight.read_text())
            except FileNotFoundError:
                call = None
            if call is not None:
                if now >= call['deadline_epoch']:
                    reason = 'Root call lifecycle exceeded60s; no MCP cancellation claimed'
            active = CASE/'OWNED-IDENTITY.json'
            if active.exists():
                lease = json.loads(active.read_text()); pid = lease['pid']
                p.direct_child_identity(plan, started, lease)
                observed = probe(pid, expected)
                if observed is None:
                    if pid not in absent:
                        absent.add(pid)
                        write('RSS-PID-'+str(pid)+'-ABSENT.json', {'pid': pid, 'observed_epoch': now})
                        if not (CASE/('QUIT-EXPECTED-'+str(pid)+'.json')).exists():
                            reason = 'Unexpected leased native process termination'
                    last_time = None
                else:
                    token = observed['start_token']
                    if not same_process(observed, lease):
                        raise RuntimeError('Leased native start token/group/parent changed, including first RSS sample')
                    if identities.setdefault(pid, token) != token:
                        reason = 'Leased PID was reused'
                    gap = 0 if last_time is None or last_pid != pid else now-last_time
                    last_pid, last_time = pid, now
                    peak = max(peak, observed['rss_bytes'])
                    reason = reason or failure(now, started['deadline_epoch'], volume(), gap, observed['rss_bytes'], cap)
                    row = dict(observed, pid=pid, observed_epoch=now, gap_seconds=gap, cap_bytes=cap)
                    log.write(json.dumps(row, sort_keys=True)+'\n'); log.flush(); os.fsync(log.fileno())
                    if cap is None and (CASE/'SAMPLE-SETTLED.json').exists():
                        empty = json.loads((CASE/'SAMPLE-SETTLED.json').read_text())
                        if empty['pid'] != pid:
                            raise RuntimeError('Settled public sample baseline lease mismatch')
                        samples.append(row)
                        if len(samples) == 5:
                            baseline = max(s['rss_bytes'] for s in samples); cap = baseline+536870912
                            reason = reason or failure(now, started['deadline_epoch'], volume(), gap, peak, cap)
                            write('RSS-BASELINE.json', {'B_bytes': baseline, 'cap_bytes': cap, 'samples': samples,
                                  'one_baseline_for_single_launch': True, 'baseline_state': 'Loaded public Harbor sample; not Empty-project evidence'})
            if reason:
                raise RuntimeError(reason)
            time.sleep(1)
    write('RESOURCE-TERMINAL.json', {'status': 'observer_stopped_after_RESOURCE_STOP', 'peak_observed_RSS_bytes': peak,
          'baseline_samples': len(samples), 'cap_bytes': cap, 'closed_epoch': time.time()})


if __name__ == '__main__':
    try:
        if sys.flags.optimize != 0:
            raise RuntimeError('Python optimization is forbidden')
        main()
    except BaseException as error:
        if not (CASE/'RESOURCE-ALARM.json').exists():
            write('RESOURCE-ALARM.json', {'status': 'FAILED_OR_UNKNOWN_RESOURCE_CONTROL',
                  'error': type(error).__name__+': '+str(error), 'epoch': time.time(), 'pid': os.getpid()})
        raise
