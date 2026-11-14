# Official SDK reversal supplement

The official Python MCP SDK 1.28.1 negotiated legacy `2025-11-25`, listed five
tools and used the existing `update_seam` schema. One atomic request set
resolution to 0.04 m and reversed only seam 23's B interval. Its output is
byte-identical to the separately observed native `after-reverse.weftforge`:
`c5299308d13cc078c7e29aec3ccca0c353045b6bb913be9508842cf64d7b99fb`.
The independently frozen expectation and all 103 binary64, three binary32 and
nonfloat fields agree. Four decoded responses and zero stderr were recorded.

A second request used `UpdateSeam` alone at maximum revision. It returned
`Document revision limit reached` and preserved the preexisting sentinel bytes.
The SDK-specific compact `max-revision-input.weftforge` has SHA-256
`62c630043f9c6c1afdc2d410644e6497aa31bed39fd45926f442803593930ec6`.
It was derived from the frozen nonlinear input by changing only revision. It is
not the native fixture `input-revision-limit.weftforge`, whose SHA-256 is
`2106593bab0a4ddc4d78265baee82b72f5201fb4cb40e004d29736751b7ce28d`.
Their complete parsed fields agree, but their serialization and origin differ.

The CLI SHA-256 is
`ca49402e497f71a2785bd96a6ef936cb320ffef856acb682824fe66e108fa25a`,
byte-identical to the preceding join smoke. This supplies no new CLI
implementation, GUI input, modern-client, physical-sewing or reference-parity
claim. The transcript contains decoded SDK stream messages, not wire bytes.

Only absolute execution-workspace prefixes in the report and transcript were
replaced by literal `${WORKSPACE}`. All other execution files retain their exact
bytes. `supplement-provenance.json` maps original and published SHA-256 values.
Embedded hashes in the report and `original-private-manifest.json` continue to
identify private execution originals; they were not rewritten. Use
`publish-manifest.json` to verify the public files.

## Saved-output replay

No CLI, SDK, native UI or network is needed for these checks. From this directory,
set `WEFTFORGE_NATIVE_CASE` to the corresponding published native case directory
containing `input-nonlinear.weftforge` and `after-reverse.weftforge`, then run:

```sh
python3 -B - <<'PY'
import copy, hashlib, json, os
from decimal import Decimal
from pathlib import Path
b = Path('.')
n = Path(os.environ['WEFTFORGE_NATIVE_CASE'])
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_text(), parse_float=Decimal)
manifest = read(b/'publish-manifest.json')
for name, entry in manifest['files'].items():
    assert sha(b/name) == entry['sha256']
plan = read(b/'expectation.json')
source = read(n/'input-nonlinear.weftforge')
assert sha(n/'input-nonlinear.weftforge') == plan['input_sha256']
expected = copy.deepcopy(source)
expected['project']['revision'] += 1
expected['project']['resolution'] = Decimal('.04')
seam = next(s for s in expected['project']['seams'] if s['id'] == 23)
seam['b']['t0'], seam['b']['t1'] = seam['b']['t1'], seam['b']['t0']
assert expected == read(b/'expected-reversed.weftforge') == read(b/'sdk-reversed.weftforge')
assert (b/'sdk-reversed.weftforge').read_bytes() == (n/'after-reverse.weftforge').read_bytes()
maximum = copy.deepcopy(source)
maximum['project']['revision'] = 2**64 - 1
assert maximum == read(b/'max-revision-input.weftforge')
assert sha(b/'sdk-rejection-sentinel.weftforge') == plan['sentinel_sha256']
report = read(b/'sdk-report.json')
assert report['rejection_result']['isError'] is True
assert 'revision limit' in report['rejection_result']['content'][0]['text'].lower()
assert report['negotiated_protocol'] == '2025-11-25'
assert report['stdout_decoded_messages'] == 4 and report['stderr_bytes'] == 0
assert (b/'server-stderr.txt').read_bytes() == b''
print('Passed saved-output supplement checks; no live SDK or UI was run.')
PY
```

## Historical live-client script

`run-official-sdk.py` is the unchanged execution script, not a new portable
runner. It expects the original relative layout under a workspace:

```text
.local/candidates/9d30a71bbc2b/weftforge-cli
.local/native-reversal-acceptance/input-nonlinear.weftforge
.local/native-reversal-acceptance/ui-declaration.json
.local/native-reversal-acceptance/after-reverse.weftforge
.local/native-reversal-acceptance/mcp-sdk/
```

A fresh live replay requires the exact identified CLI, SDK 1.28.1 and immutable
native case files. In a fresh execution directory copy only the script,
`expectation.json`, `expected-reversed.weftforge`, `max-revision-input.weftforge`
and `sdk-rejection-sentinel.weftforge`; leave output/log/report filenames absent.
Run `python3 -B run-official-sdk.py` there. Do not execute it in this published
evidence directory or overwrite retained evidence. Binaries and SDK installers
are not included. A fresh run would produce separate evidence, rather than
reproduce the historical timestamps or decoded transcript verbatim.
