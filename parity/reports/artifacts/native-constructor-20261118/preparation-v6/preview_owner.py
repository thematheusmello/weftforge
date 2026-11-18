"""Fresh-case baseline, owned cleanup, provisional seal and final read-only audit."""
import base64
import hashlib
import json
import math
import os
from pathlib import Path
import re
import struct
import subprocess
import sys
import tempfile
import time
import zlib

CASE = Path(__file__).resolve().parent
OUTPUT_CAP = 536870912
DISK_FLOOR = 8000000000
ADMISSION_RESERVE = 65536
IMAGE_CAP = 8388608


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


def read(name):
    return json.loads((CASE / name).read_text())


def write(name, value):
    raw = (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + '\n').encode()
    fd, temporary = tempfile.mkstemp(prefix='.__retain-', suffix='.tmp', dir=CASE)
    try:
        with os.fdopen(fd, 'wb') as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        os.link(temporary, CASE / name, follow_symlinks=False)
    finally:
        os.unlink(temporary)


def sha(path, deadline=None):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        while block := stream.read(1048576):
            require(deadline is None or time.time() < deadline, 'Deadline during hashing')
            digest.update(block)
    require(deadline is None or time.time() < deadline, 'Deadline after hashing')
    return digest.hexdigest()


def files():
    return sorted(p for p in CASE.rglob('*') if p.is_file() and not p.is_symlink())


def volume():
    retained = 0
    for path in files():
        try:
            retained += path.stat().st_size
        except FileNotFoundError:
            pass
    return retained


def bounds(started, reserve=ADMISSION_RESERVE):
    retained = volume()
    disk = os.statvfs(CASE)
    available = disk.f_bavail * disk.f_frsize
    checked = time.time()
    return {'checked_epoch': checked, 'retained_bytes': retained, 'available_disk_bytes': available,
            'reserved_admission_bytes': reserve, 'deadline_epoch': started['deadline_epoch'],
            'within_bounds': checked < started['deadline_epoch'] and retained + reserve <= OUTPUT_CAP
            and available >= DISK_FLOOR + reserve}


def validate_started(plan, started):
    require(plan['case_path'] == str(CASE), 'Case path mismatch')
    require(all(type(started[k]) in (int, float) and math.isfinite(started[k])
                for k in ['start_epoch', 'action_cutoff_epoch', 'deadline_epoch']), 'Malformed STARTED epochs')
    require(started['action_cutoff_epoch'] == started['start_epoch'] + 240
            and started['deadline_epoch'] == started['start_epoch'] + 300, 'Actual clock bounds changed')


def direct_child_identity(plan, started, lease=None):
    child, producer = read('DIRECT-CHILD.json'), read('LAUNCH-PRODUCER-LEASE.json')
    digest = sha(CASE / 'PLAN.json')
    require(child['case'] == producer['case'] == plan['case'] and
            child['plan_sha256'] == producer['plan_sha256'] == digest and
            child['start_epoch'] == producer['start_epoch'] == started['start_epoch'] and
            type(child['pid']) is int and child['pid'] > 0 and type(producer['pid']) is int and producer['pid'] > 0 and
            child['pid'] != producer['pid'] and child['pgid'] == producer['pgid'] == producer['pid'] == child['producer_pid'] and
            isinstance(producer['start_token'], str) and bool(producer['start_token'].strip()) and
            child['producer_start_token'] == producer['start_token'] and
            producer['executable'] == os.path.realpath(plan['python']) and
            producer['command'] == [plan['python'], '-B', str(CASE / 'preview_launch.py'), 'launch'] and
            child['executable'] == plan['native']['path'] and child['sha256'] == plan['native']['sha256'],
            'Missing complete original direct-child/producer identity')
    require(all(type(child[key]) in (int, float) and math.isfinite(child[key]) for key in
                ['created_epoch', 'startup_begin_epoch', 'startup_deadline_epoch']) and
            started['start_epoch'] <= child['startup_begin_epoch'] <= child['created_epoch'] < child['startup_deadline_epoch'] and
            child['startup_deadline_epoch'] == min(child['startup_begin_epoch'] + 8, started['action_cutoff_epoch'] - 120),
            'Invalid original direct-child epochs')
    if lease is not None:
        require(isinstance(lease, dict) and type(lease.get('pid')) is int and lease['pid'] == child['pid'] and
                lease['executable'] == plan['native']['path'] and lease['bundle_id'] == plan['bundle_id'] and
                lease['app_path'] == plan['app'] and lease['source_binary_sha256'] == plan['native']['sha256'] and
                lease['process_group'] == child['pgid'] and lease['parent_pid'] == child['producer_pid'] and
                isinstance(lease['native_start_token'], str) and bool(lease['native_start_token'].strip()) and
                lease['direct_child_created_epoch'] == child['created_epoch'] and
                type(lease['native_identity_observed_epoch']) in (int, float) and math.isfinite(lease['native_identity_observed_epoch']) and
                child['created_epoch'] <= lease['native_identity_observed_epoch'] <= time.time() and
                'launch_time' in lease and (lease['launch_time'] is None or
                type(lease['launch_time']) in (int, float) and math.isfinite(lease['launch_time'])),
                'Missing complete retained native identity witness')
    return child


def cleanup_receipts(lease):
    direct_child_identity(read('PLAN.json'), read('STARTED.json'), lease)
    receipts = {}
    for path in sorted(CASE.glob('*CLEANUP*.json')):
        value = json.loads(path.read_text())
        require(isinstance(value, dict) and type(value.get('absent')) is bool
                and isinstance(value.get('signals'), list), 'Unknown or malformed cleanup receipt: ' + path.name)
        require(value.get('case') == read('PLAN.json')['case'] and value.get('start_epoch') == read('STARTED.json')['start_epoch'],
                'Cleanup receipt is not same-case: ' + path.name)
        for signal in value['signals']:
            require(isinstance(signal, dict) and signal.get('signal') in ['SIGTERM', 'SIGKILL'], 'Malformed cleanup signal')
            if signal.get('target') == 'producer_group':
                producer = read('LAUNCH-PRODUCER-LEASE.json')
                require(signal.get('pid') == producer['pid'] and signal.get('pgid') == producer['pgid'], 'Unbound producer signal')
            elif signal.get('target') == 'native_child':
                require(signal.get('pid') == lease['pid'] and signal.get('pgid') == lease['process_group'], 'Unbound child signal')
            elif signal.get('target') == 'same_case_app':
                require(any(app['pid'] == signal.get('pid')
                            and app['app_path'] == read('PLAN.json')['app'] and app['executable'] == read('PLAN.json')['native']['path']
                            for app in value.get('before', {}).get('apps', [])) and
                        signal.get('pid') == lease['pid'] and signal.get('native_start_token') == lease['native_start_token'] and
                        signal.get('parent_pid') == lease['parent_pid'] and signal.get('pgid') == lease['process_group'],
                        'Unbound same-case fallback signal')
            else:
                require(False, 'Malformed or unbound cleanup signal target')
        receipts[path.name] = value
    require('CLEANUP.json' in receipts, 'Missing owner cleanup receipt')
    return receipts


def accepted(metadata, cleanups, exact, resources_ok, in_bounds, capture_ok):
    return (metadata.get('preview_error') is None and metadata.get('quit_error') is None
            and metadata.get('constructor_response') == 'VISUAL-03-create'
            and metadata.get('normal_quit_helper_completed') is True
            and bool(cleanups) and 'CLEANUP.json' in cleanups
            and all(c.get('absent') is True and c.get('signals') == [] for c in cleanups.values())
            and exact and resources_ok and in_bounds and capture_ok)


def decoded_image(block):
    require(block.get('mimeType') in ['image/png', 'image/jpeg'], 'Unsupported provider image MIME')
    data = block.get('data')
    require(isinstance(data, str) and 0 < len(data) <= (IMAGE_CAP + 2) // 3 * 4,
            'Empty, missing or oversized image base64')
    raw = base64.b64decode(data, validate=True)
    require(raw and len(raw) <= IMAGE_CAP and base64.b64encode(raw).decode() == data, 'Noncanonical image base64')
    if raw.startswith(b'\x89PNG\r\n\x1a\n'):
        offset, dimensions, seen_data, ended = 8, None, False, False
        while offset < len(raw):
            require(offset + 12 <= len(raw), 'Truncated PNG chunk')
            size = struct.unpack('>I', raw[offset:offset + 4])[0]
            kind = raw[offset + 4:offset + 8]
            end = offset + 12 + size
            require(end <= len(raw), 'Truncated PNG chunk bytes')
            content = raw[offset + 8:end - 4]
            require(zlib.crc32(kind + content) & 0xffffffff == struct.unpack('>I', raw[end - 4:end])[0], 'PNG CRC mismatch')
            if dimensions is None:
                require(kind == b'IHDR' and size == 13, 'PNG must start with IHDR')
                dimensions = struct.unpack('>II', content[:8])
            elif kind == b'IHDR':
                raise RuntimeError('Duplicate PNG IHDR')
            seen_data |= kind == b'IDAT' and size > 0
            offset = end
            if kind == b'IEND':
                require(size == 0 and offset == len(raw), 'PNG IEND/trailing data')
                ended = True
                break
        require(ended and seen_data, 'PNG lacks image data/end')
        actual = 'image/png'
    elif raw.startswith(b'\xff\xd8') and raw.endswith(b'\xff\xd9'):
        offset, dimensions, scan = 2, None, False
        while offset < len(raw) - 2:
            require(raw[offset] == 0xff, 'JPEG marker missing')
            while offset < len(raw) and raw[offset] == 0xff:
                offset += 1
            require(offset < len(raw), 'JPEG marker truncated')
            marker = raw[offset]
            offset += 1
            require(marker not in [0, 0xd8, 0xd9], 'Unexpected JPEG marker')
            require(offset + 2 <= len(raw), 'JPEG segment truncated')
            size = struct.unpack('>H', raw[offset:offset + 2])[0]
            require(size >= 2 and offset + size <= len(raw), 'JPEG segment out of range')
            if marker in [0xc0, 0xc1, 0xc2]:
                require(size >= 8 and dimensions is None, 'Malformed JPEG frame')
                height, width = struct.unpack('>HH', raw[offset + 3:offset + 7])
                dimensions = (width, height)
            offset += size
            if marker == 0xda:
                scan = offset < len(raw) - 2
                break
        require(dimensions is not None and scan, 'JPEG lacks frame/scan')
        actual = 'image/jpeg'
    else:
        raise RuntimeError('Decoded bytes are not PNG/JPEG')
    require(all(0 < n <= 16384 for n in dimensions) and dimensions[0] * dimensions[1] <= 67108864,
            'Image dimensions outside bounded positive range')
    return raw, actual, dimensions


def image_probe(path):
    result = subprocess.run(['/usr/bin/sips', '-g', 'format', '-g', 'pixelWidth', '-g', 'pixelHeight', str(path)],
                            capture_output=True, text=True, timeout=3, check=True)
    fields = dict(re.findall(r'^\s*(format|pixelWidth|pixelHeight):\s*(\S+)\s*$', result.stdout, re.M))
    require(fields.get('format') in ['png', 'jpeg'], 'Decoder did not identify PNG/JPEG')
    return 'image/' + fields['format'], (int(fields['pixelWidth']), int(fields['pixelHeight']))


def retain_captures(metadata):
    captures, errors = [], []
    if metadata.get('constructor_response') != 'VISUAL-03-create':
        return captures, ['Missing exact constructor response']
    for index, block in enumerate(read('VISUAL-03-create.json').get('content', [])):
        if block.get('type') != 'image':
            continue
        try:
            raw, actual, dimensions = decoded_image(block)
            require(bounds(read('STARTED.json'), len(raw) + ADMISSION_RESERVE)['within_bounds'],
                    'Insufficient capture persistence byte/disk/time reserve')
            path = CASE / 'captures' / ('constructor-' + str(index) + ('.jpg' if actual == 'image/jpeg' else '.png'))
            fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            with os.fdopen(fd, 'wb') as stream:
                stream.write(raw)
                stream.flush()
                os.fsync(stream.fileno())
            require(image_probe(path) == (actual, dimensions), 'Actual decoder type/dimensions mismatch')
            captures.append({'path': str(path.relative_to(CASE)), 'sha256': sha(path), 'bytes': len(raw),
                             'provider_mime_type': block['mimeType'], 'actual_mime_type': actual,
                             'provider_type_mismatch': block['mimeType'] != actual, 'dimensions': list(dimensions),
                             'validation': 'strict canonical base64 + structural PNG/JPEG + sips format/dimensions',
                             'unedited_tool_image': True})
        except (ValueError, RuntimeError, OSError, subprocess.SubprocessError, KeyError) as error:
            errors.append({'block': index, 'error': type(error).__name__ + ': ' + str(error)})
    return captures, errors if captures or errors else ['No returned image blocks']


def postguards(plan, started):
    deadline = started['deadline_epoch']
    checks = {name: sha(binding['path'], deadline) == binding['sha256']
              for name, binding in plan['required_root_files'].items()}
    checks['native'] = sha(plan['native']['path'], deadline) == plan['native']['sha256']
    checks['retained_native'] = sha(plan['retained_native']['path'], deadline) == plan['retained_native']['sha256'] == plan['native']['sha256']
    checks['fixture_sha256'] = sha(plan['fixture']['path'], deadline) == plan['fixture']['sha256']
    checks['fixture_mode'] = (Path(plan['fixture']['path']).stat().st_mode & 0o777) == plan['fixture']['mode']
    return checks


def candidate(plan, started, captures, errors):
    metadata = read('SEQUENCE-METADATA.json')
    lease = read('OWNED-IDENTITY.json')
    direct_child_identity(plan, started, lease)
    require(lease == read('OWNED-IDENTITY-001.json'), 'Retained native identity changed')
    cleanups = cleanup_receipts(lease)
    checks = postguards(plan, started)
    monitors = {name: read(name + '-COMPLETION.json')['exit_code'] for name in ['LAUNCH', 'WATCHDOG', 'RESOURCE']}
    producer = read('LAUNCH-PRODUCER-TERMINAL.json')
    close = read('OWNER-close-COMPLETION.json')
    counts = read('COUNTS.json')
    resources_ok = (not (CASE / 'RESOURCE-ALARM.json').exists() and all(type(code) is int and code == 0 for code in monitors.values())
                    and counts['actions'] <= 20 and counts['observations'] <= 40
                    and producer['case'] == plan['case'] and producer['child_pid'] == read('OWNED-IDENTITY.json')['pid']
                    and producer['ready'] is True and type(producer['child_exit']) is int and producer['child_exit'] == 0 and producer['failure'] is None
                    and producer['signals'] == [] and read('CLEANUP.json').get('producer_quiescent') is True
                    and type(close.get('exit_code')) is int and close['exit_code'] == 0 and close.get('session_id') is None)
    current = bounds(started)
    ok = accepted(metadata, cleanups, all(checks.values()), resources_ok, current['within_bounds'], bool(captures) and not errors)
    return ok, {'metadata': metadata, 'captures': captures, 'capture_errors': errors, 'cleanup_receipts': cleanups,
                'source_postguards': checks, 'monitor_exits': monitors, 'bounds_snapshot': current,
                'status': 'PROVISIONAL_SEAL_REQUIRES_FINAL_CALLER_ADMISSION', 'provisional_candidate': ok,
                'alignment_accepted': False, 'no_reference_or_garment_workflow_credit': True}


def manifest(started):
    return {str(p.relative_to(CASE)): {'sha256': sha(p, started['deadline_epoch']), 'bytes': p.stat().st_size} for p in files()}


def seal(plan, started):
    require(bounds(started)['within_bounds'], 'Insufficient final persistence reserve')
    captures, errors = retain_captures(read('SEQUENCE-METADATA.json'))
    ok, provisional = candidate(plan, started, captures, errors)
    write('PREVIEW-RESULT.json', provisional)
    write('PREVIEW-MANIFEST.json', manifest(started))
    write('PREVIEW-EXTERNAL-TERMINAL.json', {'status': 'PROVISIONAL_OWNER_TERMINAL_REQUIRES_CALLER_POST_RETENTION_AUDIT',
          'provisional_candidate': ok, 'alignment_accepted': False, 'snapshot': bounds(started)})
    require(ok and bounds(started)['within_bounds'], 'Provisional seal failed final persistence/time/byte/disk checks')
    return {'status': 'PROVISIONAL_SEAL_ONLY', 'provisional_candidate': True}


def audit(plan, started):
    terminal = read('OWNER-seal-COMPLETION.json')
    require(type(terminal.get('exit_code')) is int and terminal['exit_code'] == 0 and terminal.get('session_id') is None
            and not list(CASE.glob('*-LATE.json')) and not list(CASE.glob('*-PENDING.json')),
            'Seal/controller terminal is failed, late or pending')
    previous = read('PREVIEW-RESULT.json')
    ok, current = candidate(plan, started, previous['captures'], previous['capture_errors'])
    expected = read('PREVIEW-MANIFEST.json')
    actual = manifest(started)
    require(all(actual.get(name) == value for name, value in expected.items()), 'Persisted protocol/manifest changed')
    allowed = re.compile(r'^(PREVIEW-MANIFEST\.json|PREVIEW-EXTERNAL-TERMINAL\.json|OWNER-seal-(INITIAL|COMPLETION|WAIT-\d{3})\.json)$')
    require(all(name in expected or allowed.fullmatch(name) for name in actual), 'Unexpected post-manifest artifact')
    require(all(sha(CASE / c['path'], started['deadline_epoch']) == c['sha256'] for c in previous['captures']), 'Capture changed')
    plan_digest = sha(CASE / 'PLAN.json', started['deadline_epoch'])
    manifest_digest = sha(CASE / 'PREVIEW-MANIFEST.json', started['deadline_epoch'])
    files_digest = hashlib.sha256(json.dumps(actual, sort_keys=True).encode()).hexdigest()
    final = bounds(started)
    require(ok and previous['provisional_candidate'] is True and final['within_bounds'], 'Final persisted-case bounds/admission failed')
    return {'status': 'PROVISIONAL_FINAL_AUDIT', 'provisional_candidate': True, 'case': plan['case'],
            'plan_sha256': plan_digest, **final, 'persisted_manifest_sha256': manifest_digest,
            'all_persisted_files_digest': files_digest,
            'persisted_file_count': len(actual), 'filesystem_writes_after_final_probe': 0,
            'caller_must_check_post_terminal_retention_deadline': True, 'alignment_accepted': False}


def main():
    plan, started = read('PLAN.json'), read('STARTED.json')
    validate_started(plan, started)
    phase = sys.argv[1]
    if phase == 'baseline':
        limit = min(time.time() + 8, started['action_cutoff_epoch'] - 120)
        while not (CASE / 'RSS-BASELINE.json').exists() and time.time() < limit:
            require(not (CASE / 'RESOURCE-ALARM.json').exists(), 'Resource alarm before baseline')
            time.sleep(.1)
        baseline = read('RSS-BASELINE.json')
        require(len(baseline['samples']) == 5 and not (CASE / 'RESOURCE-ALARM.json').exists(), 'Five-sample baseline unavailable')
        print(json.dumps({'baseline': baseline, 'elapsed_seconds': time.time() - started['start_epoch']}))
        return
    if phase == 'close':
        try:
            write('OWNER-NEGATIVE-FALLBACK.json', {'status': 'NEGATIVE_UNLESS_FINAL_CALLER_ADMISSION_SUCCEEDS',
                  'purpose': 'Durable default for exceptions, partial outputs and unknown tool outcomes'})
        finally:
            import cleanup as owned
            outcome = owned.close()
            require(outcome['absent'] and outcome['producer_quiescent'], 'Owned app/producer/window absence not established')
            lease_path = CASE / 'OWNED-IDENTITY.json'
            if lease_path.exists():
                lease = read('OWNED-IDENTITY.json')
                ps = subprocess.run(['/bin/ps', '-p', str(lease['pid']), '-o', 'pid=,pgid=,state='],
                                    capture_output=True, text=True, timeout=2)
                write('OWNER-OS-ABSENCE.json', {'PID': lease['pid'], 'PID_query_exit': ps.returncode,
                      'stdout': ps.stdout, 'stderr': ps.stderr, 'window_inventory': outcome['after']})
                require(ps.returncode in (0, 1) and not ps.stdout.strip(), 'Owned PID absence not established')
            else:
                require(not outcome['before']['apps'], 'Missing lease for observed native process')
            (CASE / 'DONE').write_text('Owned app/window absent; producer terminal/barrier established.\n')
            (CASE / 'RESOURCE-STOP').write_text('Owned cleanup reached; stop observer.\n')
            print(json.dumps({'absent': True, 'fallback_used': bool(outcome['signals']), 'normal_close_credit': False}))
        return
    require(phase in ['seal', 'audit'], 'Unknown owner phase')
    value = seal(plan, started) if phase == 'seal' else audit(plan, started)
    print(json.dumps(value, sort_keys=True))


if __name__ == '__main__':
    require(sys.flags.optimize == 0 and os.environ.get('PYTHONOPTIMIZE') in (None, '0'), 'Python optimization forbidden')
    main()
