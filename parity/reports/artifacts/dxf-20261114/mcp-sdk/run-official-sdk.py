#!/usr/bin/env python3
"""Bounded official-SDK DXF smoke; local stdio only, frozen pre-call contract."""
from datetime import datetime, timedelta, timezone
import hashlib
import importlib.metadata
import json
from pathlib import Path
import sys
import traceback

import anyio
import mcp
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client


OWN = Path(__file__).resolve().parent
ROOT = OWN.parents[1]
PLAN = json.loads((OWN / 'expectation.json').read_text())
MESSAGES = []


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(name, value):
    with (OWN / name).open('x') as stream:
        stream.write(json.dumps(value, indent=2) + '\n')


def check(condition, message):
    if not condition:
        raise AssertionError(message)


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
    identities = {name: sha(ROOT / row['path']) for name, row in PLAN['immutable'].items()}
    report = {
        'kind': 'implementation_only_official_sdk_stdio_dxf_smoke',
        'started_at': datetime.now(timezone.utc).isoformat(),
        'client': 'official Python mcp', 'client_version': importlib.metadata.version('mcp'),
        'python_version': sys.version, 'python_executable': sys.executable,
        'python_executable_sha256': sha(Path(sys.executable)),
        'sdk_module': str(Path(mcp.__file__)),
        'sdk_files': {str(Path(module.__file__)): sha(Path(module.__file__))
                      for module in (mcp, sys.modules[ClientSession.__module__],
                                     sys.modules[stdio_client.__module__])},
        'script_sha256': sha(Path(__file__)), 'expectation_sha256': sha(OWN / 'expectation.json'),
        'inputs_before': identities, 'limits': PLAN['limits'],
        'transcript_kind': 'Original decoded SDK stream messages, not literal transport bytes.',
        'calls': [],
    }
    error = None
    try:
        check(report['client_version'] == PLAN['client_version'], 'Unexpected official SDK version')
        check(identities == {name: row['sha256'] for name, row in PLAN['immutable'].items()},
              'Frozen binary, input, fixture or comparison-output identity mismatch')
        for name in ('sdk-transcript.jsonl', 'sdk-transcript.json', 'sdk-report.json',
                     'server-stderr.txt', 'input-identities.json', 'sdk-export.dxf'):
            check(not (OWN / name).exists(), f'Retained execution artifact exists: {name}')
        save('input-identities.json', report)
        with (OWN / 'server-stderr.txt').open('x') as stderr:
            with anyio.fail_after(30):
                params = StdioServerParameters(
                    command=str(ROOT / PLAN['immutable']['cli']['path']), args=['mcp'], cwd=str(OWN))
                async with stdio_client(params, errlog=stderr) as (read, write):
                    async with ClientSession(RecordedStream(read, 'server_to_client'),
                                             RecordedStream(write, 'client_to_server'),
                                             read_timeout_seconds=timedelta(seconds=10)) as client:
                        initialized = await client.initialize()
                        report['initialized'] = initialized.model_dump(by_alias=True, mode='json')
                        check(initialized.protocolVersion == PLAN['protocol'], 'Unexpected negotiated protocol')
                        listed = await client.list_tools()
                        report['tool_names'] = [tool.name for tool in listed.tools]
                        check(report['tool_names'] == PLAN['tool_names'], 'Unexpected tool list')
                        check(listed.nextCursor is None, 'Unexpected tools pagination')
                        schema = next(tool for tool in listed.tools
                                      if tool.name == 'project_export_dxf').inputSchema
                        report['export_input_schema'] = schema
                        check(schema['type'] == 'object' and schema['additionalProperties'] is False
                              and set(schema['required']) == {'input', 'output'}
                              and set(schema['properties']) == {'input', 'output'}
                              and all(value['type'] == 'string' for value in schema['properties'].values()),
                              'Unexpected DXF input schema')
                        for case in PLAN['calls']:
                            source, destination = ROOT / case['input'], ROOT / case['output']
                            before = sha(destination) if destination.exists() else None
                            result = await client.call_tool('project_export_dxf', {
                                'input': str(source), 'output': str(destination)})
                            decoded = result.model_dump(by_alias=True, mode='json')
                            row = {'case': case['case'], 'result': decoded, 'destination_before_sha256': before}
                            report['calls'].append(row)
                            check(result.isError is case['is_error'], f"Unexpected result: {case['case']}")
                            if case['is_error']:
                                check(isinstance(result.structuredContent.get('error'), str)
                                      and result.structuredContent['error'], 'Missing structured error')
                                row['destination_after_sha256'] = sha(destination)
                                check(row['destination_after_sha256'] == before == case['preserved_sha256'],
                                      f"Rejected call changed destination: {case['case']}")
                            else:
                                expected = dict(PLAN['success_metadata'], input=str(source), output=str(destination))
                                check(result.structuredContent == expected, 'Unexpected DXF metadata')
                                row['output_sha256'] = sha(destination)
                                row['equals_approved_cli_bytes'] = destination.read_bytes() == (
                                    ROOT / PLAN['immutable']['comparison_dxf']['path']).read_bytes()
                                check(row['equals_approved_cli_bytes'], 'SDK/approved CLI DXF bytes differ')
        report['stderr_bytes'] = (OWN / 'server-stderr.txt').stat().st_size
        check(report['stderr_bytes'] == 0, 'Server emitted stderr')
        report['status'] = 'passed_bounded_legacy_dxf_and_preservation_smoke'
    except BaseException as exc:
        report['status'] = 'failed'
        report['failure'] = {'type': type(exc).__name__, 'message': str(exc)}
        (OWN / 'client-error.txt').write_text(traceback.format_exc())
        error = exc
    finally:
        report['inputs_after'] = {name: sha(ROOT / row['path']) for name, row in PLAN['immutable'].items()}
        report['all_frozen_files_unchanged'] = report['inputs_after'] == identities
        report['client_messages'] = sum(row['direction'] == 'client_to_server' for row in MESSAGES)
        report['server_messages'] = sum(row['direction'] == 'server_to_client' for row in MESSAGES)
        report['finished_at'] = datetime.now(timezone.utc).isoformat()
        save('sdk-transcript.json', MESSAGES)
        report['transcript_sha256'] = sha(OWN / 'sdk-transcript.json')
        save('sdk-report.json', report)
    check(report['all_frozen_files_unchanged'], 'A frozen source, binary or fixture changed')
    if error is not None:
        raise error
    print(json.dumps({key: report[key] for key in ('status', 'tool_names', 'client_messages',
                     'server_messages', 'stderr_bytes', 'all_frozen_files_unchanged')}, indent=2))


if __name__ == '__main__':
    anyio.run(main)
