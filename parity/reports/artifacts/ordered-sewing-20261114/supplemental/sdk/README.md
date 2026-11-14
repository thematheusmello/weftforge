# Prospective official-SDK ordered sewing-group smoke

Prepared independently from `SEW-002-PATHS-v1` and the original
`line-identity-coarse.weftforge` fixture. No production Rust, candidate executable,
candidate output, native UI or independent evaluator implementation was read.
The original source SHA-256 is
`9472a3ff2faaf8543eb26e3edd0d05a7ac068b3e91c1108082ab93ea37640067`.

`expectation.json` freezes commands, outcomes and scope. The two expected native
files include every field. Their numeric manifests record exact IEEE f64 bits,
with only material colors recorded as IEEE f32. The comparator checks exact
object fields, array order, IDs, allocator, revision and all other values; it
rejects duplicate JSON fields. Preparation self-checks exercise f64 precision,
signed zero, f32 color semantics and strict integer/duplicate-field handling.

The first atomic call renames group 1000, reverses its B list and each interval,
and adds independent partial 2:3 group 1001 with nonlinear correspondence. It
must advance revision 73 to 74 once and allocator 1001 to 1002. Inspection must
return that exact authoritative project. Removing group 1001 must produce
revision 75 while preserving the edited group 1000 and allocator 1002.

An overlapping-group transaction, an unsupported-version source and malformed
native JSON must fail and preserve the preexisting owned native output sentinel.
The overlap transaction first performs a valid edit, so its rejection also checks
that no earlier command reaches the output. Load-rejection calls use a valid
rectangle command that would succeed on a recovered empty document; silent
recovery must not make these tests pass. Original input and successful outputs
must remain unchanged, with no recovery-file side effect.

The runner uses the already installed official Python MCP SDK 1.28.1 and legacy
`2025-11-25` over local stdio. It expects six tools and 21 closed command variants,
including the three complete sewing-group schemas. No server has been started.

After the integrating owner supplies the final packaged CLI identity, create
`binding.json` with exactly `cli_path`, `cli_sha256` and `source_commit`. Paths
may be absolute or relative to the workspace. The binding is recorded separately
from the frozen criteria. Do not run before that binding is confirmed. A single
live invocation from the workspace root is:

```sh
python3 -B .local/chain-sewing-sdk-20261114/run-official-sdk.py > .local/chain-sewing-sdk-20261114/client-stdout.txt 2> .local/chain-sewing-sdk-20261114/client-stderr.txt
```

The runner refuses existing execution artifacts and preserves decoded SDK
streams, stderr, identities and failures. Do not repair or rerun in this retained
directory. A new attempt needs separate artifacts and explicit provenance.

This proves a bounded authored-file/legacy-client subset only. It controls no GUI
and establishes no native gesture, undo/Redo, continuous metric/mesh accuracy,
physical sewing, current-profile interoperability or reference parity. Native
and fixture CLI comparisons remain separate. No installation, network, listener,
Cargo, CI or export call is needed here. These local files are not published
evidence until the integrating owner completes tracked clean-checkout checks.
