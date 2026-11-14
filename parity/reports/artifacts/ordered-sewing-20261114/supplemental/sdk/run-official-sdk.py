#!/usr/bin/env python3
"""Frozen, bounded legacy SDK sewing-group smoke; requires final binding.json."""
from datetime import datetime, timedelta, timezone
import hashlib
import importlib.metadata
import json
import math
from pathlib import Path
import struct
import sys
import traceback

import anyio
import mcp
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

OWN = Path(__file__).resolve().parent
ROOT = OWN.parents[1]
MESSAGES = []


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(condition, message):
    if not condition:
        raise AssertionError(message)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        check(key not in result, f'Duplicate JSON field: {key}')
        result[key] = value
    return result


def load(path):
    return json.loads(path.read_bytes(), object_pairs_hook=unique_object)


def save(name, value):
    with (OWN / name).open('x') as stream:
        stream.write(json.dumps(value, indent=2, allow_nan=False) + '\n')


def float_kind(path):
    return 'f32' if len(path) == 5 and path[:2] == ('project', 'materials') and path[3] == 'color' else 'f64'


def float_bits(value, kind):
    check(isinstance(value, (float, int)) and not isinstance(value, bool)
          and math.isfinite(value), f'Expected finite {kind}: {value!r}')
    return struct.pack('>f' if kind == 'f32' else '>d', value).hex()


def compare(expected, actual, path=()):
    label = '$' + ''.join(f'[{key}]' if isinstance(key, int) else '.' + key for key in path)
    if isinstance(expected, float):
        kind = float_kind(path)
        check(float_bits(expected, kind) == float_bits(actual, kind), f'{label}: {kind} bits differ')
    elif isinstance(expected, dict):
        check(isinstance(actual, dict) and set(expected) == set(actual), f'{label}: object fields differ')
        for key, value in expected.items():
            compare(value, actual[key], path + (key,))
    elif isinstance(expected, list):
        check(isinstance(actual, list) and len(expected) == len(actual), f'{label}: list length differs')
        for index, value in enumerate(expected):
            compare(value, actual[index], path + (index,))
    else:
        check(type(expected) is type(actual) and expected == actual, f'{label}: value/type differs')


def numeric_bits(value, path=()):
    if isinstance(value, float):
        kind = float_kind(path)
        return [{'path': list(path), 'kind': kind, 'bits': float_bits(value, kind)}]
    children = value.items() if isinstance(value, dict) else enumerate(value) if isinstance(value, list) else []
    return [row for key, child in children for row in numeric_bits(child, path + (key,))]


def check_schema(listed, plan):
    check([tool.name for tool in listed.tools] == plan['tool_names'], 'Unexpected tool names/order')
    check(listed.nextCursor is None, 'Unexpected tools pagination')
    execute = next(tool for tool in listed.tools if tool.name == 'project_execute')
    variants = execute.inputSchema['properties']['commands']['items']['oneOf']
    names = [variant['properties']['type']['const'] for variant in variants]
    check(len(names) == 21 and set(names) == set(plan['command_variants']), 'Unexpected command variants')
    for variant in variants:
        check(variant['additionalProperties'] is False
              and set(variant['properties']) == set(variant['required']) == {'type', 'data'}
              and variant['properties']['data']['additionalProperties'] is False, 'Command schema is not closed')
    for name, fields in plan['new_command_data_fields'].items():
        variant = next(item for item in variants if item['properties']['type']['const'] == name)
        data = variant['properties']['data']
        check(data['additionalProperties'] is False and set(data['properties']) == set(fields)
              and set(data['required']) == set(fields), f'{name}: data schema differs')
        group = data['properties']['group'] if name == 'update_sewing_group' else data
        if name != 'remove_sewing_group':
            required = {'id', 'name', 'a', 'b', 'correspondence'} if name == 'update_sewing_group' else set(fields)
            check(group['additionalProperties'] is False and set(group['properties']) == required
                  and set(group['required']) == required, f'{name}: group schema differs')
            for side in ('a', 'b'):
                intervals = group['properties'][side]
                interval = intervals['items']
                check(intervals['type'] == 'array' and intervals['minItems'] == 1
                      and intervals['maxItems'] == 1024, f'{name}: side bounds differ')
                check(interval['additionalProperties'] is False
                      and set(interval['properties']) == set(interval['required']) == {'curve', 't0', 't1'},
                      f'{name}: interval schema is not closed')
    return execute.inputSchema


class RecordedStream:
    def __init__(self, wrapped, direction):
        self.wrapped, self.direction = wrapped, direction

    async def __aenter__(self):
        await self.wrapped.__aenter__()
        return self

    async def __aexit__(self, *args):
        return await self.wrapped.__aexit__(*args)

    def __aiter__(self):
        return self

    async def __anext__(self):
        try:
            value = await self.wrapped.receive()
        except anyio.EndOfStream:
            raise StopAsyncIteration
        check(not isinstance(value, Exception), repr(value))
        self.record(value)
        return value

    def record(self, value):
        row = {'direction': self.direction, 'message': value.message.model_dump(
            by_alias=True, mode='json', exclude_none=True)}
        MESSAGES.append(row)
        with (OWN / 'sdk-transcript.jsonl').open('a') as stream:
            stream.write(json.dumps(row) + '\n')

    async def send(self, value):
        self.record(value)
        return await self.wrapped.send(value)

    async def aclose(self):
        return await self.wrapped.aclose()


async def main():
    plan, freeze, binding = (load(OWN / name) for name in ('expectation.json', 'freeze.json', 'binding.json'))
    check(set(binding) == {'cli_path', 'cli_sha256', 'source_commit'}, 'Unexpected binding fields')
    cli = Path(binding['cli_path'])
    if not cli.is_absolute():
        cli = ROOT / cli
    identities = {name: sha(OWN / name) for name in freeze['files']}
    report = {
        'kind': 'implementation_only_official_sdk_ordered_sewing_groups_smoke',
        'started_at': datetime.now(timezone.utc).isoformat(),
        'client': 'official Python mcp', 'client_version': importlib.metadata.version('mcp'),
        'python_version': sys.version, 'python_executable': sys.executable,
        'python_executable_sha256': sha(Path(sys.executable)),
        'sdk_files': {str(Path(module.__file__)): sha(Path(module.__file__))
                      for module in (mcp, sys.modules[ClientSession.__module__],
                                     sys.modules[stdio_client.__module__])},
        'binding': binding, 'binding_sha256': sha(OWN / 'binding.json'),
        'cli_before_sha256': sha(cli), 'freeze_sha256': sha(OWN / 'freeze.json'),
        'frozen_files_before': identities,
        'limits': plan['limits'], 'calls': [],
        'transcript_kind': 'Original decoded SDK stream messages, not literal transport bytes.',
    }
    error = None
    success_hashes = {}
    try:
        check(report['client_version'] == plan['client_version'], 'Unexpected official SDK version')
        check(report['cli_before_sha256'] == binding['cli_sha256'], 'Final CLI identity mismatch')
        check(identities == {name: row['sha256'] for name, row in freeze['files'].items()}, 'Frozen criteria changed')
        for name in ('sdk-transcript.jsonl', 'sdk-transcript.json', 'sdk-report.json', 'server-stderr.txt',
                     'pre-call-identities.json', 'sdk-batch.weftforge', 'sdk-removed.weftforge'):
            check(not (OWN / name).exists(), f'Retained execution artifact exists: {name}')
        save('pre-call-identities.json', report)
        with (OWN / 'server-stderr.txt').open('x') as stderr:
            with anyio.fail_after(30):
                params = StdioServerParameters(command=str(cli), args=['mcp'], cwd=str(OWN))
                async with stdio_client(params, errlog=stderr) as (read, write):
                    async with ClientSession(RecordedStream(read, 'server_to_client'),
                                             RecordedStream(write, 'client_to_server'),
                                             read_timeout_seconds=timedelta(seconds=10)) as client:
                        initialized = await client.initialize()
                        report['initialized'] = initialized.model_dump(by_alias=True, mode='json')
                        check(initialized.protocolVersion == plan['protocol'], 'Unexpected negotiated protocol')
                        listed = await client.list_tools()
                        report['execute_input_schema'] = check_schema(listed, plan)
                        report['tool_names'] = [tool.name for tool in listed.tools]
                        for case in plan['calls']:
                            arguments = dict(case['arguments'])
                            for key in ('input', 'output', 'path'):
                                if key in arguments:
                                    arguments[key] = str(OWN / arguments[key])
                            output = OWN / case['arguments']['output'] if 'output' in case['arguments'] else None
                            before = sha(output) if output is not None and output.exists() else None
                            result = await client.call_tool(case['tool'], arguments)
                            row = {'case': case['case'], 'result': result.model_dump(by_alias=True, mode='json'),
                                   'output_before_sha256': before}
                            report['calls'].append(row)
                            check(result.isError is case['is_error'], f"Unexpected result: {case['case']}")
                            if case['is_error']:
                                check(isinstance(result.structuredContent.get('error'), str)
                                      and result.structuredContent['error'], 'Missing structured error')
                                row['output_after_sha256'] = sha(output)
                                check(row['output_after_sha256'] == before == case['preserved_sha256'],
                                      f"Rejected call changed sentinel: {case['case']}")
                            elif case['tool'] == 'project_inspect':
                                expected = load(OWN / case['expected_native'])
                                compare(expected['project'], result.structuredContent['project'], ('project',))
                                check(result.structuredContent['up'] == 'Y', 'Inspection lost Y-up metadata')
                                row['typed_project_matches_frozen_expectation'] = True
                            else:
                                expected_result = dict(case['expected_result'], input=arguments['input'],
                                                       output=arguments['output'])
                                check(result.structuredContent == expected_result, 'Unexpected execution metadata')
                                expected, actual = load(OWN / case['expected_native']), load(output)
                                compare(expected, actual)
                                expected_rows = load(OWN / case['expected_bits'])
                                actual_rows = []
                                for item in expected_rows:
                                    value = actual
                                    for key in item['path']:
                                        value = value[key]
                                    actual_rows.append(dict(item, bits=float_bits(value, item['kind'])))
                                check(actual_rows == expected_rows, 'Numeric bit manifest differs')
                                row['typed_native_matches_frozen_expectation'] = True
                                row['output_sha256'] = sha(output)
                                success_hashes[output.name] = sha(output)
        report['stderr_bytes'] = (OWN / 'server-stderr.txt').stat().st_size
        check(report['stderr_bytes'] == 0, 'Server emitted stderr')
        check(not list(OWN.glob('*.recovery*')), 'Unexpected recovery side effect')
        report['status'] = 'passed_bounded_legacy_ordered_group_smoke'
    except BaseException as exc:
        report['status'] = 'failed'
        report['failure'] = {'type': type(exc).__name__, 'message': str(exc)}
        with (OWN / 'client-error.txt').open('x') as stream:
            stream.write(traceback.format_exc())
        error = exc
    finally:
        report['frozen_files_after'] = {name: sha(OWN / name) for name in freeze['files']}
        report['all_frozen_files_unchanged'] = report['frozen_files_after'] == identities
        report['success_output_hashes_after'] = {name: sha(OWN / name) for name in success_hashes}
        report['success_outputs_unchanged_after_errors'] = report['success_output_hashes_after'] == success_hashes
        report['cli_after_sha256'] = sha(cli)
        report['binding_after_sha256'] = sha(OWN / 'binding.json')
        report['client_messages'] = sum(row['direction'] == 'client_to_server' for row in MESSAGES)
        report['server_messages'] = sum(row['direction'] == 'server_to_client' for row in MESSAGES)
        report['finished_at'] = datetime.now(timezone.utc).isoformat()
        save('sdk-transcript.json', MESSAGES)
        report['transcript_sha256'] = sha(OWN / 'sdk-transcript.json')
        save('sdk-report.json', report)
    check(report['all_frozen_files_unchanged'], 'Frozen file changed during smoke')
    check(report['success_outputs_unchanged_after_errors'], 'An error changed a successful output')
    check(report['cli_after_sha256'] == binding['cli_sha256'], 'CLI changed during smoke')
    check(report['binding_after_sha256'] == report['binding_sha256'], 'Binding changed during smoke')
    if error is not None:
        raise error
    print(json.dumps({key: report[key] for key in ('status', 'tool_names', 'client_messages',
                     'server_messages', 'stderr_bytes', 'all_frozen_files_unchanged',
                     'success_outputs_unchanged_after_errors')}, indent=2))


if __name__ == '__main__':
    anyio.run(main)
