# Combined source integration and next numerical proposal

Combined commit: `96d3baee4543c7edbdb804df6090697a9f65da09`, branch
`feat/cloth-material-integration`. Its actual parents are endpoint source
`ac6c2e53b47f2d19772d2bfb341e1500dae2cf0c` and reviewed material source
`36524293b89686c2368f22d13ad09289fc70af03`. Actual endpoint commit
`3fddeb9a72e5b4395e8e874a628086937331af2f` remains an ancestor. Diagnostic trace
`47eaa53ef61f4c6db73cf874402cb41be44c827f` is explicitly not an ancestor.

The combined diff from ac6 is exactly the four reviewed files: sim
constraints/lib/tests and docs/physics.md. No endpoint source, metadata, registry,
map, scope, Actions or Pages file was edited by this integration. No other
worktree was mutated and no compiled binary/cache crossed worktrees.

SOURCE88.json uses the existing 87-path canonical source roster plus
docs/physics.md, which the old list omitted. All 88 paths are Git-tracked.
METADATA-FREEZE.json records all eight historical inventory files byte-identical
to 381c, the original 949-entry/205-packet/56-landmark map, all 56 full behavior,
scenario, comparison and dependency scopes exact against 381c, and the entire
feature registry byte-identical to ac6. These are read-only identity checks;
source-map/report freshness remains deliberately unrefreshed pending review.

## One sequential combined gate run

Overall tool session **9326** is terminal, exit **0**. Own target and one Cargo
build job were used. Every command ran once; every before/after source and
metadata pin matched, and the tracked checkout stayed clean.

| Command | Child PID | Exit | Seconds |
| --- | ---: | ---: | ---: |
| cargo fmt --all --check | 15044 | 0 | 0.787881 |
| cargo test --workspace | 15161 | 0 | 331.395645 |
| cargo clippy --workspace --all-targets -- -D warnings | 20377 | 0 | 69.138279 |
| cargo build -p weftforge -p weftforge-cli | 20932 | 0 | 15.039148 |

The actual test-log total is **398 passed / 0 failed / 0 ignored**, summed from
31 unit/integration/doc-test summaries. This count is derived, not inferred from
the expected count. Exact argv, UTC/monotonic duration, process handles, source
after states, raw logs and log hashes are retained in per-command receipts and
RESULT.json. All child handles are terminal.

Fresh debug binaries (the explicit build stage launched neither):

- Native: SHA256 `eb82a84295f8558cdb094119e2a3c9796313efc17bae87c20ed4b4fa1c133933`,
  62,736,440 bytes.
- CLI: SHA256 `9544cbcb2635c1b29a8e4de7729f6a9b13a526fd97ae40a6a5638ffbe5147b25`,
  10,626,608 bytes.

The native GUI was never launched. Workspace integration tests did invoke the
CLI through CARGO_BIN_EXE_weftforge-cli. The original RESULT.json's unqualified
`launched:false` entries are retained unchanged; LAUNCH-ATTRIBUTION.json qualifies
them as build-stage information, not proof that the CLI was never executed.

No GUI, old garment producer/evaluator, cargo xtask verify, registry validation,
progress generation or public publication operation was executed. No native UI,
garment, reference, calibration or complete-workflow acceptance follows.

## New Harbor A8 proposal, unexecuted

The sibling `next-harbor-proposal/PROTOCOL-PROPOSAL.json` gives a concrete new
case, `WF-20261116-COUPLED-HARBOR-A8-001`. It is proposed only and has no GO file.
It uses exactly the original Harbor A input fc4949... and physical rest d96697...,
with 654 vertices/1,172 triangles/48 stitches, original geometry/material/contact,
dt1/60, 8 substeps, 32 iterations, sewing1m/s, 60 zero-gravity assembly frames and
120 gravity frames. There is one attempt, no sweep/reset/continuation/height case.

All 181 states are independently evaluated under the original strict 15% strain
and 2mm supported-contact screens, finite/uncertainty requirements, and 5mm seam
screen after exact closure. Targets must be exactly zero at frames60 and180;
final-only success cannot replace trajectory quality. Original 420s whole-case,
256MiB producer RSS and numerical-output caps are retained. The closing clock
must be sampled after final source/artifact guards and report writing.

The proposed producer is an exact copy of the existing public A8 API driver,
not a copied engine or binary. The proposed independent evaluator's mathematical
modules and original A8 measurement/summarizing functions are copied byte-for-
byte. REUSED-SOURCES.json records their original paths and hashes. An explicit
strict adapter permits only holes=[] and internal_curves=[] on the two pieces,
plus the already-known f32 color storage; all physical/rest fields remain exact.
The consumer does not import production solver/contact routines or execute cloth.

The producer/evaluator candidates have not been built or executed. Python files
were syntax-parsed without imports; no old saved case was graded. Before any new
case execution, root must review this protocol and independence, then separately
prepare/review the controller and receipts, qualify analytical compatibility
self-checks in a new namespace, build/freeze the release producer and private
lockfile in this worktree, and issue a GO binding their exact hashes. The current
proposal entry point refuses without a distinct authorization binding. The
producer's final-only flag is consistency data, not the independent grade.

Historical original/Harbor A/B/A16 failures and session96213's failed archived
identity stay unchanged. This next case supplies no native or full-garment gate
acceptance even if its bounded numerical screens eventually pass.

All gate/proposal artifacts remain private ignored review inputs. They are not
delivery or public evidence; root owns any later tracked curation and freshness
updates. No owned job or test window remains active at handoff.
