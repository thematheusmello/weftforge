#!/usr/bin/env python3
"""Capture actual packaged CLI results; independent evaluators run separately."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import time

OWN = Path(__file__).resolve().parent
ROOT = OWN.parents[1]
ORIGINAL = ROOT / '.local/chain-sewing-acceptance-20261114'
COMMIT = '3eddef067c8a1f03f192c8e6822a8da2a94f9bf9'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, data):
    with path.open('x') as stream:
        stream.write(json.dumps(data, indent=2) + '\n')


def relative(path):
    return str(path.relative_to(ROOT))


def invoke(cli, arguments, folder, name):
    request = [relative(cli), *arguments]
    save(folder / f'{name}-request.json', {'argv': request, 'cwd': '.'})
    start = time.monotonic()
    with (folder / f'{name}-stdout.json').open('xb') as stdout, (folder / f'{name}-stderr.txt').open('xb') as stderr:
        process = subprocess.Popen(request, cwd=ROOT, stdout=stdout, stderr=stderr)
        timed_out = False
        try:
            code = process.wait(timeout=30)
        except subprocess.TimeoutExpired:
            timed_out = True
            process.kill()
            code = process.wait()
    row = {'pid': process.pid, 'exit_code': code, 'timed_out': timed_out,
           'elapsed_s': time.monotonic() - start,
           'stdout_sha256': sha(folder / f'{name}-stdout.json'),
           'stdout_bytes': (folder / f'{name}-stdout.json').stat().st_size,
           'stderr_sha256': sha(folder / f'{name}-stderr.txt'),
           'stderr_bytes': (folder / f'{name}-stderr.txt').stat().st_size}
    save(folder / f'{name}-invocation.json', row)
    return row


def main():
    binding = json.loads((OWN / 'binary-binding.json').read_bytes())
    assert binding['source_commit'] == COMMIT
    cli = ROOT / binding['cli_path']
    assert sha(cli) == binding['cli_sha256']
    precheck = json.loads((ROOT / '.local/chain-sewing-producer-20261114/debug-precheck/producer-plan.json').read_bytes())
    inputs = {case: ROOT / item['path'] for case, item in precheck['inputs'].items()}
    for case, path in inputs.items():
        assert sha(path) == precheck['inputs'][case]['sha256']
    negative = json.loads((ORIGINAL / 'negative-cases.json').read_bytes())['must_reject']
    bad_inputs = {case: ORIGINAL / 'fixtures' / f'invalid-{case}.weftforge' for case in negative}
    assert all(path.is_file() for path in bad_inputs.values())
    all_inputs = {relative(path): sha(path) for path in [*inputs.values(), *bad_inputs.values()]}
    capture = OWN / 'headless'
    capture.mkdir(exist_ok=False)
    save(capture / 'pretest.json', {'kind': 'actual-packaged-cli-pretest-v1',
         'declared_utc': datetime.now(timezone.utc).isoformat(), 'binding': binding,
         'binding_sha256': sha(OWN / 'binary-binding.json'), 'inputs': all_inputs,
         'producer_sha256': sha(Path(__file__)), 'case_timeout_s': 30,
         'valid_cases': list(inputs), 'negative_cases': negative,
         'negative_control_command': {'type': 'set_resolution', 'data': {'resolution': 0.037}},
         'negative_rule': 'All13 reject with exit1 and empty stdout; original inputs, owned preexisting native destination and recovery bytes stay exact. A valid source control must succeed with the same command/path setup.',
         'scope': 'Headless authoring/rest mesh and malformed file preservation only; no UI/history/physics/reference claims.'})
    results = []
    for case, source in inputs.items():
        folder = capture / case
        folder.mkdir()
        row = invoke(cli, ['inspect-sewing', relative(source)], folder, 'inspect')
        results.append({'case': case, **row})
    negative_results = []
    for case, source in [('valid-control', inputs['line-identity-coarse']), *bad_inputs.items()]:
        folder = capture / 'negative' / case
        folder.mkdir(parents=True)
        copied = folder / 'input.weftforge'
        output = folder / 'destination.weftforge'
        recovery = folder / 'destination.weftforge.recovery'
        commands = folder / 'commands.json'
        copied.write_bytes(source.read_bytes())
        output.write_bytes(inputs['line-identity-coarse'].read_bytes())
        recovery.write_bytes(inputs['legacy-v1-f64'].read_bytes())
        save(commands, [{'type': 'set_resolution', 'data': {'resolution': 0.037}}])
        protected = [copied, output, recovery]
        before = {relative(path): sha(path) for path in protected}
        row = invoke(cli, ['execute', relative(copied), relative(output), relative(commands)], folder, 'execute')
        after = {relative(path): sha(path) for path in protected}
        preserved = before == after
        row.update({'case': case, 'protected_before': before, 'protected_after': after,
                    'all_protected_preserved': preserved,
                    'accepted_expected_outcome': (row['exit_code'] == 0 and before[relative(output)] != after[relative(output)]) if case == 'valid-control' else (row['exit_code'] == 1 and row['stdout_bytes'] == 0 and row['stderr_bytes'] > 0 and preserved)})
        save(folder / 'preservation.json', row)
        negative_results.append(row)
    preserved_inputs = all(sha(ROOT / path) == digest for path, digest in all_inputs.items())
    unchanged_cli = sha(cli) == binding['cli_sha256']
    report = {'kind': 'actual-packaged-cli-results-v1', 'source_commit': COMMIT,
              'valid_cases': results, 'negative_cases': negative_results,
              'all_original_inputs_preserved': preserved_inputs, 'cli_unchanged': unchanged_cli,
              'finished_utc': datetime.now(timezone.utc).isoformat()}
    save(capture / 'results.json', report)
    assert all(row['exit_code'] == 0 and not row['timed_out'] and row['stderr_bytes'] == 0 for row in results)
    assert all(row['accepted_expected_outcome'] for row in negative_results)
    assert preserved_inputs and unchanged_cli
    print(json.dumps({'valid_outputs': len(results), 'negative_rejections': len(negative_results) - 1,
                      'valid_pipeline_control': True, 'inputs_preserved': preserved_inputs,
                      'cli_unchanged': unchanged_cli}, indent=2))


if __name__ == '__main__':
    main()
