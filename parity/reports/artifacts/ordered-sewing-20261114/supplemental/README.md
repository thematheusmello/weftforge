# Ordered sewing supplemental evidence candidate

This small package retains implementation-only development evidence for runtime
source `3eddef067c8a1f03f192c8e6822a8da2a94f9bf9`. Its 37-file source snapshot,
packaged CLI SHA-256 `5755ecd4aefb29b0060370c8369ee823fdbbd19cae85d5dd675edec48de954e1`,
and unsigned native executable SHA-256
`e935a4331cd884be7c81078f82da9119bea87211bf702cfc8959ba6eaca54053` are retained
under `runtime/`. The saved four required local gates and unsigned package passed:
122 Rust tests, 54 Python checks, map 949 / 205 work packets / 56 landmarks.
These are historical gate records, not a new clean-checkout publication run.

`manifest.json` records every retained file's bytes, SHA-256, original source
path and original source hash. Safe originals remain exact. Files explicitly
named `.public.*` replace only literal machine-specific path prefixes with
`<WORKSPACE>`, `<HOME>`, `<FINE_WORKTREE>` or `<CARGO_TARGET>`; no lines are dropped
or normalized. The manifest records replacement counts. Those whole-file
derivatives are **not complete raw-byte logs**. Source-hash references inside
them still name original bytes, not derivative hashes. SDK transcripts preserve
decoded SDK stream messages; they were never literal transport bytes.

Run `python3 -B check_bundle.py` in this directory for integrity, original SDK
freeze consistency, full typed saved-result comparisons, six-call transcript
checks, preservation hashes, review classifier self-tests and public-path scan.
This uses only Python's standard library and starts no server or product binary.
The saved-report/decoded-stream check permits only the SDK's omission of null
`calls[i].result.content[j].annotations` and
`calls[i].result.content[j]._meta` protocol block metadata. It requires those
saved fields to be null, removes only those two keys from content blocks, and
compares all authoritative `structuredContent` project values including nulls
without normalization. `packaging-review/` retains the unchanged initial checker,
its observed failure and the explicit correction; no frozen runner or criteria
changed. The captured failure repeats the first direct failure with unchanged
checker bytes, as its record states.
The manifest excludes itself to avoid a recursive digest; the integrator's
separate local closure manifest hashes every file including this manifest.

`sdk/` retains the unchanged independently prepared official Python MCP SDK
runner, every expected native field, f64/f32 bit manifests, actual native input
and output bytes, malformed/future fixtures, initial sentinel, original pretest
freeze `0bde0fc960a0c3e1c9f93238cb9c9e899b455da7dde72dab663105e02f76e285`, final
binding and saved results. Six actual tool calls passed using SDK **1.28.1**,
legacy **2025-11-25** local stdio, six listed tools and 21 closed command variants.
They exercised one atomic update/add, inspection, removal and three rejected
calls with preserved sentinel and successful outputs. The frozen SDK README is
historical: its statement that no server had started describes preparation, and
the later bound report/transcripts document the actual run. The runner's original
relative workspace layout is retained as evidence; do not invoke it from this
relocated subset or overwrite these records. No SDK install or network is needed
to review the retained results.

`reviews/core/` is the exact unchanged 13-file independent core subset: original
`70897ee5378cb36edb9647b4c0e5fdcfb884c54a` accepted a short-span counterexample
with 5.01998207145255645e-7 m paired error against a 1e-8 m budget; fix
`9cd0a215df4a5977b1b3263061a6cb98183eb2d4` rejects it, with four precision cases
and 68 ownership cases. Its README distinguishes literal retained program-output
transcripts from complete logs. Its checker contains a generic private-temp
prefix test literal, not an actual personal path. No core build was rerun here.

`reviews/ui/` preserves the exact six-file follow-up: both frozen harnesses fail
on `65c1e22fbff6ba528921629e4926caed0be397b0` and pass on
`ed0dd37dc06789d531db5091917c7d64afeaa4d4`. This covers the reversed-B numeric
buffer regression and accessible append action in an in-process UI harness.
It is not standalone native UI acceptance; no app was launched here.

`fine-mesh/` retains the original failed fine input (0.017 m resolution;
100000 vertices / 200000 triangles), zero-stdout debug failure, source plan,
diagnostic sliver progression, necessary area bound, exact mathematical mesh
validator, completed reports, source diff and core gate provenance. Fix
`aca60af071d5a099f4054c2793e4f6d7dfb089e8` schedules quality improvement before
each subdivision; it is an ancestor of the final runtime source. The initial
validator correction is preserved verbatim, including its then-pending status:
it incorrectly mixed dimensionless boundary t and metre margin; the corrected
validator checks the interpolated endpoint t on one curve. Separate later
reports show successful validation at 51906 vertices / 102685 triangles / 273
stitches for fine, and 851 / 1480 / 73 for coarse. The path-redacted area driver
is a historical source transcript, not a portable replay script. Saved elapsed
times are observations under unqualified local load, not isolated benchmarks.

The separate output-review bundle supplies all ten final actual CLI receipts.
This package records hashes for the omitted large fine/coarse success outputs;
their bytes equal the final-runtime receipts. The original debug precheck lists
nine other successes whose large receipts/evaluations are omitted here, so this
package makes no published replay claim for those nine.

`regressions/` retains the original source-pinned f32 snapshot RED assertion,
historical GREEN log, exact `ec28db041479de6040ef4d1245ceb166b2ddfc84` fix diff,
the initial workspace grain literal-version failure, historical grain GREEN and
exact `6417d5aebc8dd87d8f5711f0173318d40518ba2a` old-caller fix. The historical
GREEN logs do not embed commit pins; their association is qualified in
`scope.json`. The separate source-bound final workspace log provides current
integration validation.

No reference parity, physical sewing, GUI gesture, modern-profile SDK client,
cross-platform or roadmap acceptance claim follows from this package. Native
18-checkpoint evidence remains separate. No binary, target cache, proprietary
material, credentials or personal path is included. Actions remain disabled.
These ignored local files become published evidence only after the owner tracks
every artifact and runs registry/map checks from a clean checkout.
