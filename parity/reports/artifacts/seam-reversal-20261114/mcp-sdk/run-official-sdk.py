#!/usr/bin/env python3
"""One bounded local SDK transaction and one revision-limit rejection."""
import anyio
from datetime import datetime, timedelta, timezone
from decimal import Decimal
import hashlib
import importlib.metadata
import json
from pathlib import Path
import struct
import traceback
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

OWN = Path(__file__).resolve().parent
ROOT = OWN.parents[2]
BASE = OWN.parent
CLI = ROOT / '.local/candidates/9d30a71bbc2b/weftforge-cli'
PLAN_SHA = 'd21c769518c65253c83a7d357fcf41c8cef29e92603de081f301274205c1efe9'
TRANSCRIPT = []


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(value, message):
    if not value:
        raise AssertionError(message)


def save(name, value):
    (OWN / name).write_text(json.dumps(value, indent=2) + '\n')


def read_doc(path):
    return json.loads(path.read_text(), parse_float=Decimal)


def compare_docs(expected, actual):
    counts = {'binary64': 0, 'binary32': 0}
    def visit(a, b, path=()):
        if isinstance(a, dict):
            require(isinstance(b, dict) and a.keys() == b.keys(), f'Object keys differ: {path}')
            for key in a:
                visit(a[key], b[key], (*path, key))
        elif isinstance(a, list):
            require(isinstance(b, list) and len(a) == len(b), f'Array differs: {path}')
            for i, (x, y) in enumerate(zip(a, b)):
                visit(x, y, (*path, i))
        elif isinstance(a, Decimal):
            require(type(b) in (Decimal, int) and a == b, f'Decimal construction differs: {path}')
            color = len(path) == 5 and path[:2] == ('project', 'materials') and path[-2] == 'color'
            code = '>f' if color else '>d'
            require(struct.pack(code, float(a)) == struct.pack(code, float(b)), f'Typed bits differ: {path}')
            counts['binary32' if color else 'binary64'] += 1
        else:
            require(type(a) is type(b) and a == b, f'Nonfloat metadata differs: {path}')
    visit(expected, actual)
    return {**counts, 'all_decimal_construction_exact': True, 'all_nonfloat_fields_exact': True,
            'color_note': 'Unchanged fixture colors are ordinary serialized binary32 values; no new midpoint-parser test.'}


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
        require(not isinstance(value, Exception), repr(value))
        TRANSCRIPT.append({'direction': self.direction, 'message': value.message.model_dump(by_alias=True, mode='json', exclude_none=True)})
        return value

    async def send(self, value):
        TRANSCRIPT.append({'direction': self.direction, 'message': value.message.model_dump(by_alias=True, mode='json', exclude_none=True)})
        return await self.wrapped.send(value)

    async def aclose(self):
        return await self.wrapped.aclose()


async def main():
    plan = json.loads((OWN / 'expectation.json').read_text())
    source = BASE / 'input-nonlinear.weftforge'
    maximum = OWN / 'max-revision-input.weftforge'
    sentinel = OWN / 'sdk-rejection-sentinel.weftforge'
    output = OWN / 'sdk-reversed.weftforge'
    report = {'kind': 'bounded_official_sdk_reversal_comparison', 'started_at': datetime.now(timezone.utc).isoformat(),
              'script_sha256': sha(Path(__file__)), 'expectation_sha256': sha(OWN / 'expectation.json'),
              'client': 'official Python mcp', 'client_version': importlib.metadata.version('mcp'),
              'cli_sha256': sha(CLI), 'launch_argv': [str(CLI), 'mcp'], 'cli_new_implementation_claimed': False,
              'cli_byte_identical_to_previous_join_sdk': True, 'input_before_sha256': sha(source),
              'max_revision_input_before_sha256': sha(maximum), 'sentinel_before_sha256': sha(sentinel),
              'transcript_kind': 'Decoded official SDK stream messages; not literal transport bytes.',
              'limits': ['Legacy 2025-11-25 local stdio only; no modern-client or reference-parity claim.',
                         'No native GUI actions, Cargo, installation, network or listeners.',
                         'Native output was read only after root confirmed it immutable, and after expectation freeze.',
                         'Independent reversal oracle was not read or modified.']}
    failure = None
    try:
        require(report['expectation_sha256'] == PLAN_SHA, 'Frozen comparison plan changed')
        require(report['client_version'] == '1.28.1', 'Unexpected official SDK version')
        require(report['cli_sha256'] == plan['cli_sha256'], 'CLI identity differs')
        require(report['input_before_sha256'] == plan['input_sha256'], 'Frozen input changed')
        require(report['max_revision_input_before_sha256'] == plan['max_revision_input_sha256'], 'Prepared rejection input changed')
        require(report['sentinel_before_sha256'] == plan['sentinel_sha256'], 'Sentinel changed before call')
        require(sha(OWN / 'expected-reversed.weftforge') == plan['expected_document_sha256'], 'Frozen expectation changed')
        require(sha(BASE / 'ui-declaration.json') == plan['native_declaration_sha256'], 'Native declaration changed')
        require(not output.exists(), 'SDK output already exists')
        with (OWN / 'server-stderr.txt').open('x') as stderr, anyio.fail_after(30):
            async with stdio_client(StdioServerParameters(command=str(CLI), args=['mcp'], cwd=str(OWN)), errlog=stderr) as (read, write):
                async with ClientSession(RecordedStream(read, 'server_to_client'), RecordedStream(write, 'client_to_server'), read_timeout_seconds=timedelta(seconds=10)) as session:
                    initialized = await session.initialize()
                    report['negotiated_protocol'] = initialized.protocolVersion
                    report['server_info'] = initialized.serverInfo.model_dump(mode='json')
                    require(initialized.protocolVersion == '2025-11-25', 'Unexpected protocol')
                    listed = await session.list_tools()
                    report['tool_names'] = [tool.name for tool in listed.tools]
                    require(report['tool_names'] == ['project_inspect', 'project_create', 'project_execute', 'project_export_svg', 'project_export_obj'], 'Unexpected tool list')
                    execute = next(t for t in listed.tools if t.name == 'project_execute')
                    variants = execute.inputSchema['properties']['commands']['items']['oneOf']
                    report['update_seam_schema'] = next(v for v in variants if v['properties']['type'].get('const') == 'update_seam')
                    schema = report['update_seam_schema']['properties']['data']
                    require(schema['required'] == ['seam'] and schema['additionalProperties'] is False, 'Unexpected update_seam arguments')
                    result = await session.call_tool('project_execute', {'input': str(source), 'output': str(output), 'commands': plan['normal_commands']})
                    report['transaction_result'] = result.model_dump(by_alias=True, mode='json')
                    require(result.isError is False and result.structuredContent['revision'] == 16 and result.structuredContent['commands_applied'] == 2, 'Transaction did not commit exactly once')
                    report['sdk_output_sha256'] = sha(output)
                    report['frozen_expected_comparison'] = compare_docs(read_doc(OWN / 'expected-reversed.weftforge'), read_doc(output))
                    # Root explicitly confirmed this snapshot ready before this script ran.
                    native = BASE / 'after-reverse.weftforge'
                    report['native_after_reverse_sha256'] = sha(native)
                    report['native_sdk_bytes_equal'] = native.read_bytes() == output.read_bytes()
                    require(report['native_sdk_bytes_equal'], 'Native and SDK bytes differ')
                    report['native_sdk_typed_comparison'] = compare_docs(read_doc(native), read_doc(output))
                    rejected = await session.call_tool('project_execute', {'input': str(maximum), 'output': str(sentinel), 'commands': plan['rejection_commands']})
                    report['rejection_result'] = rejected.model_dump(by_alias=True, mode='json')
                    text = '\n'.join(item.text for item in rejected.content if hasattr(item, 'text'))
                    require(rejected.isError is True and 'revision limit' in text.lower(), 'Maximum-revision update did not reject as expected')
                    report['sentinel_after_sha256'] = sha(sentinel)
                    require(report['sentinel_after_sha256'] == plan['sentinel_sha256'], 'Rejected update overwrote sentinel')
        report['stderr_bytes'] = (OWN / 'server-stderr.txt').stat().st_size
        require(report['stderr_bytes'] == 0, 'Server stderr is nonempty')
        report['status'] = 'passed_legacy_sdk_reversal_and_native_comparison'
    except BaseException as error:
        report['status'] = 'failed'
        report['failure'] = {'type': type(error).__name__, 'message': str(error)}
        (OWN / 'client-error.txt').write_text(traceback.format_exc())
        failure = error
    finally:
        report['input_after_sha256'] = sha(source)
        report['max_revision_input_after_sha256'] = sha(maximum)
        report['cli_after_sha256'] = sha(CLI)
        report['stdout_decoded_messages'] = sum(m['direction'] == 'server_to_client' for m in TRANSCRIPT)
        report['finished_at'] = datetime.now(timezone.utc).isoformat()
        save('sdk-transcript.json', TRANSCRIPT)
        report['transcript_sha256'] = sha(OWN / 'sdk-transcript.json')
        save('sdk-report.json', report)
    require(report['input_before_sha256'] == report['input_after_sha256'] == plan['input_sha256'], 'Native input changed')
    require(report['max_revision_input_before_sha256'] == report['max_revision_input_after_sha256'], 'Rejection input changed')
    require(report['cli_after_sha256'] == plan['cli_sha256'], 'CLI changed')
    if failure:
        raise failure
    print(json.dumps({k: report[k] for k in ['status', 'negotiated_protocol', 'sdk_output_sha256', 'native_sdk_bytes_equal', 'native_sdk_typed_comparison', 'sentinel_after_sha256', 'stdout_decoded_messages', 'stderr_bytes']}, indent=2))


if __name__ == '__main__':
    anyio.run(main)
