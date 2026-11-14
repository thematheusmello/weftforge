#!/usr/bin/env python3
"""Private bounded official-SDK join smoke; local stdio only."""
import anyio
from datetime import datetime, timedelta, timezone
import hashlib
import importlib.metadata
import json
from pathlib import Path
import runpy
import traceback

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client


ROOT = Path(__file__).resolve().parents[3]
OWN = Path(__file__).resolve().parent
NATIVE = OWN.parent
CLI = ROOT / '.local/candidates/26dd58535868/weftforge-cli'
CLI_SHA = 'ca49402e497f71a2785bd96a6ef936cb320ffef856acb682824fe66e108fa25a'
EXPECTED = {
    'after-split.weftforge': 'd904801435a660fa754b83ad088c477f385d040df36b6d4b7865a43f28e30f55',
    'after-join.weftforge': '00d8d318b888d41b89bd9d8d4fef770ba04abb71c8e12dbb01c69e8291e6ee9a',
    'reopened.weftforge': '8d4090142551f037003e3c531d524be5e80ef870f4c706a185ad1ee89e9205ae',
}
JOINED = OWN / 'sdk-joined.weftforge'
SENTINEL = OWN / 'sdk-rejected-sentinel.weftforge'
SENTINEL_BYTES = b'Official SDK rejected-join sentinel; preserve these exact bytes.\n'
TRANSCRIPT = []


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(name, data):
    (OWN / name).write_text(json.dumps(data, indent=2) + '\n')


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
        TRANSCRIPT.append({'direction': self.direction, 'message': value.message.model_dump(
            by_alias=True, mode='json', exclude_none=True)})
        return value

    async def send(self, value):
        TRANSCRIPT.append({'direction': self.direction, 'message': value.message.model_dump(
            by_alias=True, mode='json', exclude_none=True)})
        return await self.wrapped.send(value)

    async def aclose(self):
        return await self.wrapped.aclose()


async def main():
    report = {
        'kind': 'implementation_only_official_sdk_stdio_smoke',
        'started_at': datetime.now(timezone.utc).isoformat(),
        'client': 'official Python mcp',
        'client_version': importlib.metadata.version('mcp'),
        'script_sha256': sha(Path(__file__)),
        'cli_sha256': sha(CLI),
        'inputs_before': {name: sha(NATIVE / name) for name in EXPECTED},
        'transcript_kind': 'Decoded MCP messages at the official SDK stream boundary; not literal transport bytes.',
        'limits': ['Local stdio only; no network or listener, installation, Cargo or native UI.',
                   'SDK output is not a native UI output.',
                   'Legacy 2025-11-25 only; no modern external-client or reference-parity claim.'],
    }
    error = None
    try:
        check(report['client_version'] == '1.28.1', 'Unexpected official SDK version')
        check(report['cli_sha256'] == CLI_SHA, 'CLI identity mismatch')
        check(report['inputs_before'] == EXPECTED, 'Native input identity mismatch')
        check(not JOINED.exists() and not SENTINEL.exists(), 'Disposable outputs already exist')
        SENTINEL.write_bytes(SENTINEL_BYTES)
        report['sentinel_before_sha256'] = sha(SENTINEL)
        save('input-identities.json', {key: report[key] for key in
             ('started_at', 'script_sha256', 'cli_sha256', 'inputs_before', 'sentinel_before_sha256')})
        with (OWN / 'server-stderr.txt').open('x') as stderr:
            with anyio.fail_after(30):
                params = StdioServerParameters(command=str(CLI), args=['mcp'], cwd=str(OWN))
                async with stdio_client(params, errlog=stderr) as (read, write):
                    async with ClientSession(RecordedStream(read, 'server_to_client'),
                                             RecordedStream(write, 'client_to_server'),
                                             read_timeout_seconds=timedelta(seconds=10)) as client:
                        initialized = await client.initialize()
                        report['negotiated_protocol'] = initialized.protocolVersion
                        report['server_info'] = initialized.serverInfo.model_dump(mode='json')
                        check(initialized.protocolVersion == '2025-11-25', 'Unexpected negotiated protocol')
                        listed = await client.list_tools()
                        report['tool_names'] = [tool.name for tool in listed.tools]
                        check(report['tool_names'] == ['project_inspect', 'project_create', 'project_execute',
                              'project_export_svg', 'project_export_obj'], 'Unexpected tool list')
                        execute = next(tool for tool in listed.tools if tool.name == 'project_execute')
                        variants = execute.inputSchema['properties']['commands']['items']['oneOf']
                        joins = [v for v in variants if v['properties']['type'].get('const') == 'join_curve']
                        check(len(joins) == 1, 'Missing or duplicate join_curve schema')
                        schema = joins[0]['properties']['data']
                        check(schema['required'] == ['curve'] and set(schema['properties']) == {'curve'}
                              and schema['additionalProperties'] is False, 'Incorrect join_curve arguments')
                        report['join_curve_schema'] = joins[0]
                        joined = await client.call_tool('project_execute', {
                            'input': str(NATIVE / 'after-split.weftforge'), 'output': str(JOINED),
                            'commands': [{'type': 'join_curve', 'data': {'curve': 9}}]})
                        report['join_result'] = joined.model_dump(by_alias=True, mode='json')
                        check(joined.isError is False, 'SDK join unexpectedly failed')
                        check(joined.structuredContent['revision'] == 10
                              and joined.structuredContent['commands_applied'] == 1, 'Unexpected join revision/count')
                        native_join = (NATIVE / 'after-join.weftforge').read_bytes()
                        report['sdk_joined_sha256'] = sha(JOINED)
                        report['sdk_joined_equals_native_after_join_bytes'] = JOINED.read_bytes() == native_join
                        check(report['sdk_joined_equals_native_after_join_bytes'], 'SDK/native join bytes differ')
                        oracle_path = ROOT / 'parity/reports/artifacts/document-f64-20261114/native/ieee_document_oracle.py'
                        oracle = runpy.run_path(str(oracle_path))
                        report['typed_comparison_oracle_sha256'] = sha(oracle_path)
                        report['typed_comparison'] = oracle['compare_native'](
                            oracle['load'](NATIVE / 'after-join.weftforge'), oracle['load'](JOINED))
                        rejected = await client.call_tool('project_execute', {
                            'input': str(NATIVE / 'reopened.weftforge'), 'output': str(SENTINEL),
                            'commands': [{'type': 'join_curve', 'data': {'curve': 7}}]})
                        report['rejected_result'] = rejected.model_dump(by_alias=True, mode='json')
                        texts = '\n'.join(item.text for item in rejected.content if hasattr(item, 'text'))
                        check(rejected.isError is True and 'collinear' in texts.lower(),
                              'Rejected join did not return the expected collinear error')
                        report['sentinel_after_sha256'] = sha(SENTINEL)
                        check(SENTINEL.read_bytes() == SENTINEL_BYTES, 'Rejected join overwrote sentinel')
        report['stderr_bytes'] = (OWN / 'server-stderr.txt').stat().st_size
        check(report['stderr_bytes'] == 0, 'Server emitted stderr')
        report['status'] = 'passed_bounded_legacy_join_and_error_smoke'
    except BaseException as exc:
        report['status'] = 'failed'
        report['failure'] = {'type': type(exc).__name__, 'message': str(exc)}
        (OWN / 'client-error.txt').write_text(traceback.format_exc())
        error = exc
    finally:
        report['inputs_after'] = {name: sha(NATIVE / name) for name in EXPECTED}
        report['all_native_inputs_byte_unchanged'] = report['inputs_before'] == report['inputs_after'] == EXPECTED
        report['cli_after_sha256'] = sha(CLI)
        report['stdout_decoded_messages'] = sum(row['direction'] == 'server_to_client' for row in TRANSCRIPT)
        report['client_messages'] = sum(row['direction'] == 'client_to_server' for row in TRANSCRIPT)
        report['finished_at'] = datetime.now(timezone.utc).isoformat()
        save('sdk-transcript.json', TRANSCRIPT)
        report['transcript_sha256'] = sha(OWN / 'sdk-transcript.json')
        save('sdk-report.json', report)
    check(report['all_native_inputs_byte_unchanged'], 'Native input changed during smoke')
    check(report['cli_after_sha256'] == CLI_SHA, 'CLI identity changed during smoke')
    if error is not None:
        raise error
    print(json.dumps({key: report[key] for key in ['status', 'negotiated_protocol', 'tool_names',
          'sdk_joined_sha256', 'sdk_joined_equals_native_after_join_bytes', 'typed_comparison',
          'sentinel_before_sha256', 'sentinel_after_sha256', 'stderr_bytes', 'stdout_decoded_messages',
          'all_native_inputs_byte_unchanged']}, indent=2))


if __name__ == '__main__':
    anyio.run(main)
