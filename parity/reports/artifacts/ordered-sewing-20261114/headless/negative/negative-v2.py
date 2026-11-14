#!/usr/bin/env python3
"""Correct only the malformed test-driver command; original failures stay intact."""
from datetime import datetime, timezone
import json
from pathlib import Path
from produce import ROOT, OWN, ORIGINAL, COMMIT, sha, save, relative, invoke


def main():
    binding = json.loads((OWN / 'binary-binding.json').read_text())
    cli = ROOT / binding['cli_path']
    assert binding['source_commit'] == COMMIT and sha(cli) == binding['cli_sha256']
    first = OWN / 'headless/results.json'
    original = json.loads(first.read_text())
    assert original['negative_cases'][0]['case'] == 'valid-control'
    assert original['negative_cases'][0]['exit_code'] == 1
    assert not original['negative_cases'][0]['accepted_expected_outcome']
    assert 'unknown field `resolution`, expected `metres`' in (OWN / 'headless/negative/valid-control/execute-stderr.txt').read_text()
    cases = json.loads((ORIGINAL / 'negative-cases.json').read_text())['must_reject']
    source = ORIGINAL / 'fixtures/line-identity-coarse.weftforge'
    old = ORIGINAL / 'fixtures/legacy-v1-f64.weftforge'
    invalid = {case: ORIGINAL / 'fixtures' / f'invalid-{case}.weftforge' for case in cases}
    input_hashes = {relative(p): sha(p) for p in [source, old, *invalid.values()]}
    folder = OWN / 'negative-v2'
    folder.mkdir(exist_ok=False)
    command = [{'type': 'set_resolution', 'data': {'metres': 0.037}}]
    save(folder / 'pretest.json', {'kind': 'corrected-negative-driver-pretest-v1',
         'declared_utc': datetime.now(timezone.utc).isoformat(), 'binding': binding,
         'producer_sha256': sha(Path(__file__)), 'shared_driver_sha256': sha(OWN / 'produce.py'),
         'original_failed_attempt_sha256': sha(first), 'original_failed_attempt_retained': True,
         'correction': 'Use actual SetResolution.metres field; first attempt failed its valid control before source loading, so none of its13negative results qualifies as a load rejection.',
         'inputs': input_hashes, 'command': command, 'negative_cases': cases,
         'rules_unchanged': 'Valid same-pipeline control succeeds. All13 malformed original sources reject with exit1/empty stdout/useful stderr while source,destination,recovery bytes stay exact. No UI/history/reference/physics inference.'})
    rows = []
    for case, input_path in [('valid-control', source), *invalid.items()]:
        out = folder / case
        out.mkdir()
        copied, destination, recovery, commands = [out / n for n in ('input.weftforge', 'destination.weftforge', 'destination.weftforge.recovery', 'commands.json')]
        copied.write_bytes(input_path.read_bytes())
        destination.write_bytes(source.read_bytes())
        recovery.write_bytes(old.read_bytes())
        save(commands, command)
        protected = [copied, destination, recovery]
        before = {relative(p): sha(p) for p in protected}
        row = invoke(cli, ['execute', relative(copied), relative(destination), relative(commands)], out, 'execute')
        after = {relative(p): sha(p) for p in protected}
        if case == 'valid-control':
            accepted = (row['exit_code'] == 0 and row['stderr_bytes'] == 0
                        and before[relative(copied)] == after[relative(copied)]
                        and before[relative(destination)] != after[relative(destination)])
        else:
            accepted = (row['exit_code'] == 1 and not row['timed_out'] and row['stdout_bytes'] == 0
                        and (out / 'execute-stderr.txt').read_text().startswith('weftforge-cli: ')
                        and before == after)
        row.update({'case': case, 'protected_before': before, 'protected_after': after,
                    'all_protected_preserved': before == after, 'accepted_expected_outcome': accepted})
        save(out / 'preservation.json', row)
        rows.append(row)
    result = {'kind': 'corrected-negative-driver-results-v1', 'source_commit': COMMIT,
              'cases': rows, 'all_original_inputs_preserved': all(sha(ROOT / p) == h for p, h in input_hashes.items()),
              'cli_unchanged': sha(cli) == binding['cli_sha256'],
              'finished_utc': datetime.now(timezone.utc).isoformat()}
    save(folder / 'results.json', result)
    assert all(row['accepted_expected_outcome'] for row in rows)
    assert result['all_original_inputs_preserved'] and result['cli_unchanged']
    print(json.dumps({'valid_pipeline_control': True, 'qualified_negative_rejections': len(rows) - 1,
                      'all_original_inputs_preserved': True, 'cli_unchanged': True}, indent=2))


if __name__ == '__main__':
    main()
