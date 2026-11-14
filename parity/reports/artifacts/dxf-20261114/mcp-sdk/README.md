# Official-SDK DXF smoke — headless package

On 2026-11-14 at 12:36:22 UTC, the installed official Python MCP SDK 1.28.1
negotiated legacy `2025-11-25`, listed six tools and used the advertised
`project_export_dxf` input/output schema. Its decoded structured result matched
revision 21, `authored_dxf`, `mm`, `pattern U = +X, V = +Y; Z = 0` and the frozen
boundary-only scope. The unchanged SDK default handshake advertises client name
`mcp`, version `0.1.0`; that value is distinct from the installed package version.

This tested the coordinator-supplied **headless pre-UI-fix source `5f4124c`
package**, CLI SHA-256
`e90ac57b9bd345ad1d844b4f3f9e684c96fa58af0f795d5f7a168ddfc3fad20f`.
The SDK DXF is 4,805 bytes, SHA-256
`ca94b0235ba944c0879b51b260b600ee3a10c6bd539112b213f6b59f9a719ca7`,
byte-identical to the approved CLI geometry export. `native-input-copy.weftforge`
contains exactly the successful call's independently prepared source bytes:
SHA-256 `04c7e54d1ef459f4bf2d965d57c6e4cf816e4f9c2b049b10da0bbf8a2a984fbb`,
revision 21, resolution 0.045 m. It is also the owned copy used for same-source
rejection. Neither this fixture nor this SDK DXF is an observed native UI output.

Same-input/destination, malformed JSON and future native version 9999 each
returned `isError: true` with a structured error. The source copy and the
preexisting destination sentinel kept their exact bytes. All frozen files were
unchanged. Seven client and six server decoded messages, including the
initialization notification, were retained; both stderr files are empty.
`preparation-attempt.json` separately records a read-only pre-call `AttributeError`
from treating a producer report list as a mapping. It made no server call,
export or mutation. The live SDK smoke passed on its first attempt.

These files derive from retained private execution originals. Only exact local
path prefixes in three JSON files were replaced with `${WORKSPACE}` and
`${PYTHON_INSTALL}`. The runner, frozen expectation/pre-call manifest, native
fixtures, sentinel, DXF and stderr retain their exact bytes. See
`publication-provenance.json` for original/public SHA-256 mappings and omitted
redundant private logs. Embedded hashes in the report, pre-call manifest and
`original-private-manifest.json` still identify private originals; use
`publish-manifest.json` to verify this subset. The transcript is decoded SDK
stream data, not literal transport bytes.

The first publication-only replay found that the generated manifest incorrectly
listed itself. `failed-publication-manifest.json` and
`publication-tooling-attempt.json` preserve that failure; a complete private
snapshot was also retained. The final manifest explicitly excludes itself, as
originally intended. This tooling correction changed no live SDK artifact or
acceptance criterion and made no server call.

From this directory, the following standard-library replay checks saved files
only. It needs no CLI, SDK, network or UI:

```sh
python3 -B - <<'PY'
from pathlib import Path
import hashlib, json
b = Path('.')
read = lambda name: json.loads((b / name).read_text())
sha = lambda name: hashlib.sha256((b / name).read_bytes()).hexdigest()
manifest = read('publish-manifest.json')
for name, row in manifest['files'].items():
    assert sha(name) == row['sha256'], name
provenance = read('publication-provenance.json')
for name, row in provenance['files'].items():
    assert sha(name) == row['public_sha256'], name
attempt = read('publication-tooling-attempt.json')
assert sha('failed-publication-manifest.json') == attempt['failed_manifest_sha256']
assert attempt['server_calls'] == 0 and attempt['live_sdk_smoke_unchanged'] is True
for name, row in read('pre-call-manifest.json')['files'].items():
    assert row['sha256'] == provenance['files'][name]['original_sha256'], name
p, r = read('expectation.json'), read('sdk-report.json')
expected_hashes = {name: row['sha256'] for name, row in p['immutable'].items()}
assert r['inputs_before'] == r['inputs_after'] == expected_hashes
assert sha('native-input-copy.weftforge') == expected_hashes['source']
for name in ('future.weftforge', 'malformed.weftforge', 'invalid-source-sentinel.dxf'):
    assert sha(name) == expected_hashes[name]
source = read('native-input-copy.weftforge')
assert source['project']['revision'] == 21 and source['project']['resolution'] == 0.045
assert (b / 'future.weftforge').read_bytes() == (b / 'native-input-copy.weftforge').read_bytes().replace(b'"version": 2,', b'"version": 9999,', 1)
assert sha('sdk-export.dxf') == p['calls'][0]['expected_output_sha256']
assert sha('sdk-export.dxf') == r['calls'][0]['output_sha256']
success = r['calls'][0]['result']
assert success['isError'] is False and r['calls'][0]['equals_approved_cli_bytes'] is True
assert success['structuredContent'] == dict(p['success_metadata'], input='${WORKSPACE}/' + p['calls'][0]['input'], output='${WORKSPACE}/' + p['calls'][0]['output'])
for expected, actual in zip(p['calls'][1:], r['calls'][1:], strict=True):
    assert actual['case'] == expected['case'] and actual['result']['isError'] is True
    assert actual['result']['structuredContent']['error']
    assert actual['destination_before_sha256'] == actual['destination_after_sha256'] == expected['preserved_sha256']
messages = read('sdk-transcript.json')
assert len(messages) == 13 and messages[1]['message']['result']['capabilities'] == {'tools': {}}
assert r['initialized']['protocolVersion'] == '2025-11-25' and r['client_version'] == '1.28.1'
listed = next(row['message']['result'] for row in messages if 'tools' in row['message'].get('result', {}))
assert [tool['name'] for tool in listed['tools']] == r['tool_names'] == p['tool_names']
schema = next(tool['inputSchema'] for tool in listed['tools'] if tool['name'] == 'project_export_dxf')
assert schema == r['export_input_schema'] and schema['additionalProperties'] is False
assert set(schema['required']) == set(schema['properties']) == {'input', 'output'}
assert all(row['type'] == 'string' for row in schema['properties'].values())
assert r['script_sha256'] == provenance['files']['run-official-sdk.py']['original_sha256']
assert r['transcript_sha256'] == provenance['files']['sdk-transcript.json']['original_sha256']
assert r['status'] == 'passed_bounded_legacy_dxf_and_preservation_smoke'
assert r['client_messages'] == 7 and r['server_messages'] == 6 and r['all_frozen_files_unchanged']
assert (b / 'server-stderr.txt').read_bytes() == (b / 'client-stderr.txt').read_bytes() == b''
print('Passed saved-output checks; no live SDK or UI was run.')
PY
```

`run-official-sdk.py` is the unchanged historical live runner, not the replay
above. It expects this layout in a **fresh** workspace:

```text
.local/dxf-sdk-20261114/
  run-official-sdk.py
  expectation.json
  native-input-copy.weftforge
  malformed.weftforge
  future.weftforge
  invalid-source-sentinel.dxf
.local/dxf-acceptance-20261114/input/native-expected.weftforge
.local/dxf-producer-20261114/pre-ui-fix/
  weftforge-cli
  native-expected.dxf
```

The separate success source must equal the included native copy. The separate
comparison DXF must equal `sdk-export.dxf`. Supply the exact identified CLI and
installed official SDK 1.28.1 separately; neither a binary nor an installer is
included. Leave execution output/log/report names absent. The historical command
from that fresh workspace root is
`python3 -B .local/dxf-sdk-20261114/run-official-sdk.py`. It creates separate live
evidence and does not reproduce historical timestamps or transcript bytes. Do
not run it inside this retained publication subset.

No new server calls were made while preparing this subset. The original run
used no network, listeners, installation, Cargo, Actions or native UI. Final GUI
package/native export comparison remains pending and may be appended separately
if the CLI stays byte-identical. Current-profile clients, other CAD consumers,
apparel/printing, physical sewing and reference parity remain unestablished.
This local subset is not tracked published delivery until its artifacts and
clean-checkout registry/map checks are included by the integrating owner.
