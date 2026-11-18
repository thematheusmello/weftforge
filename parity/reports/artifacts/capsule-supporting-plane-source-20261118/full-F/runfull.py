import time
STARTED = time.monotonic()
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import subprocess
import sys
import resources as r

ROOT = Path(__file__).resolve().parent
WT = ROOT.parent.parent
RSS_CAP = 1610612736
OUTPUT_CAP = 8388608


def read(path):
    return json.loads(path.read_text())


def cached_registry_tree_digest(paths,deadline=None):
    digest=hashlib.sha256();count=0;size=0
    for root in paths:
        base=Path(root)
        for p in r.files_below(base):
            r.deadline_check(deadline)
            entry=[str(base),str(p.relative_to(base)),p.stat().st_size,p.stat().st_mode&0o7777,r.sha(p,deadline)]
            digest.update(json.dumps(entry,separators=(',',':')).encode()+b'\n');count+=1;size+=entry[2]
    return {'sha256':digest.hexdigest(),'files':count,'bytes':size,'package_trees':len(paths)}


def main():
    r.require(sys.version_info[:2] == (3, 13) and sys.dont_write_bytecode and sys.flags.optimize == 0, 'Require Python3.13 -B optimization0')
    phase = sys.argv[1]
    r.require(phase == 'full-001', 'Unknown bounded phase')
    plan = read(ROOT / (phase + '.json'))
    deadline = STARTED + plan['wall_seconds']
    runtime = ROOT / phase
    runtime.mkdir()

    def persist(name, value):
        path = runtime / name
        r.write_new(path, r.encoded(value))
        with path.open('rb') as stream:
            os.fsync(stream.fileno())

    def volume():
        return sum(path.stat().st_size for path in r.files_below(runtime))

    def git(*argv):
        return subprocess.check_output(['/usr/bin/git', *argv], cwd=WT, text=True).strip()

    def guard():
        r.require(git('rev-parse', 'HEAD') == plan['head'], 'HEAD changed')
        r.require(git('status', '--porcelain=v1', '--untracked-files=no') == plan['tracked_status'], 'Tracked status changed')
        for name, entry in plan['source91'].items():
            path = WT / name
            r.require(r.sha(path, deadline) == entry['sha256'] and (path.stat().st_mode & 0o777) == entry['mode'], 'Source changed: ' + name)
        for name, digest in plan['config_inputs'].items():
            path = Path(name)
            r.require((r.sha(path, deadline) if path.is_file() else None) == digest, 'Config changed: ' + name)
        for name, digest in plan['controller_inputs'].items():
            r.require(r.sha(ROOT / name, deadline) == digest, 'Controller changed: ' + name)
        for tool in plan['tools'].values():
            r.require(r.sha(tool['path'], deadline) == tool['sha256'], 'Pinned tool changed')
        for path, expected in plan['locked_registry_archives'].items():
            r.require(r.sha(path, deadline) == expected, 'Locked registry archive changed: ' + path)
        r.require(cached_registry_tree_digest(plan['cached_registry_trees'], deadline) == plan['cached_registry_tree_digest'], 'Cached source tree changed')
        r.deadline_check(deadline)

    stages = []
    errors = []
    summary = None
    binaries = {}
    env = dict(os.environ)
    for name in plan['rejected_ambient_keys']:
        r.require(env.get(name) in (None, ''), 'Unexpected ambient flags/wrapper: ' + name)
    for name in ('CARGO_BUILD_RUSTC_WRAPPER', 'CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER'):
        env.pop(name, None)
    env.update(plan['child_env'])
    persist('STARTED.json', {'pid': os.getpid(), 'phase': phase, 'started_monotonic': STARTED, 'deadline': deadline, 'plan_sha256': r.sha(ROOT / (phase + '.json'), deadline), 'target_absent_before': not (WT / 'target').exists(), 'cold_target_expected': plan['cold_target_expected']})
    try:
        guard()
        r.require((not (WT / 'target').exists()) == plan['cold_target_expected'], 'Target cold/warm state changed')
        for command in plan['commands']:
            guard()
            name = command['name']
            def on_start(value):
                persist(name + '-STARTED.json', value)
                print(json.dumps({'phase': phase, 'stage': name, 'pid': value['pid'], 'process_group': value['process_group']}), flush=True)
            stage = r.monitor(command['argv'], WT, runtime / (name + '.stdout'), runtime / (name + '.stderr'), deadline, volume,
                              rss_cap=RSS_CAP, output_cap=OUTPUT_CAP, reserve=262144, child_env=env, on_start=on_start, disk_floor=8000000000)
            stages.append(stage)
            persist(name + '-RESULT.json', stage)
            r.require(stage['stop_reason'] is None, 'Resource/monitor stop: ' + str(stage['stop_reason']))
            r.require(stage['exit_code'] == command['expected_exit'], 'Unexpected command exit')
            guard()
            if 'expected_tests' in command:
                stdout = (runtime / (name + '.stdout')).read_text()
                stderr = (runtime / (name + '.stderr')).read_text()
                matches = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', stdout + '\n' + stderr)
                summary = {'summaries': len(matches), 'passed': sum(int(x[1]) for x in matches), 'failed': sum(int(x[2]) for x in matches), 'ignored': sum(int(x[3]) for x in matches)}
                r.require(summary == command['expected_tests'], 'Unexpected test counts')
                for fragment in command.get('required_output', []):
                    r.require(fragment in stdout + '\n' + stderr, 'Expected assertion/summary absent: ' + fragment)
        guard()
        for name in ('weftforge', 'weftforge-cli'):
            path = WT / 'target/debug' / name
            r.require(path.is_file() and not path.is_symlink(), 'Missing own built binary: ' + name)
            binaries[name] = {'path': str(path), 'sha256': r.sha(path, deadline), 'bytes': path.stat().st_size, 'mode': path.stat().st_mode & 0o777, 'built_not_launched': True}
        status = os.statvfs(WT)
        r.require(int(status.f_bavail) * int(status.f_frsize) >= 8000000000, 'Terminal disk below8GB')
        guard()
    except BaseException as error:
        errors.append(type(error).__name__ + ': ' + str(error))
    self_rss = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    if self_rss > RSS_CAP:
        errors.append('Supervisor exact RSS cap')
    result = {'phase': phase, 'head': plan['head'], 'stages': stages, 'errors': errors, 'test_summary': summary, 'built_binaries': binaries, 'all_source91_exact': not errors, 'source91': plan['source91'], 'supervisor_exact_peak_RSS_bytes': self_rss, 'plan_sha256': r.sha(ROOT / (phase + '.json')), 'expected_red_is_not_product_pass': phase == 'red-001', 'no_GUI_xtask_garment_or_reference_acceptance': True}
    persist('RESULT.json', result)
    remaining = []
    raw = subprocess.check_output(['/bin/ps', '-axo', 'pid=,pgid='], text=True)
    groups = {stage['process_group'] for stage in stages}
    for line in raw.splitlines():
        parts = line.split()
        if len(parts) == 2 and int(parts[1]) in groups:
            remaining.append({'pid': int(parts[0]), 'process_group': int(parts[1])})
    persist('TERMINAL.json', {'result_sha256': r.sha(runtime / 'RESULT.json'), 'all_direct_children_reaped': True, 'remaining_owned_groups': remaining})
    witness_hash = r.sha(runtime / 'TERMINAL.json')
    counted_volume = volume()
    closed = time.monotonic()
    accepted = not errors and not remaining and len(stages) == len(plan['commands']) and self_rss <= RSS_CAP and closed < deadline and counted_volume + 65536 < OUTPUT_CAP
    print(json.dumps({'phase': phase, 'protocol_complete_under_caps': accepted, 'expected_red_observed': accepted and phase == 'red-001', 'elapsed_after_persistence_hash_volume_seconds': closed - STARTED, 'terminal_sha256': witness_hash, 'total_retained_bytes': counted_volume, 'errors': errors}), flush=True)
    return 0 if accepted else 1


if __name__ == '__main__':
    raise SystemExit(main())
