# Prospective physical-length sewing acceptance

This is a frozen, independently authored engineering evaluator for the approved
physical-length Free Sewing contract. It has **not executed a WeftForge candidate**
or any application. It cannot establish reference parity or full contract
acceptance. The fixture and every numerical expectation were prepared before
candidate execution, from the approved contract and the immutable public source
inputs recorded in `SOURCE-INPUTS.json`.

`fixture.weftforge` is an original version-3 construction with two 200 × 300 mm
panel bounds, revision 41 and allocator 20. Curve 6 is
`C(t)=(0.2t,0.1t²)`; curve 15 is `B(t)=(0.4+0.2t,0)`, both in metres.
The analytic primitive is `F(t)=0.1[t√(1+t²)+asinh(t)]`. Full curved length is
`0.22955871493926383 m`. A separate cubic derivative evaluator uses composite
Simpson quadrature at 2,048/4,096 steps and a separate 56-step inverse. Primitive
inversion uses 64 bisections. The two methods must agree within `2e-12 m`, tighter
than the frozen production acceptance of `1e-8 m`. Neither imports or executes a
production metric. `prepare_acceptance.py` records how the original fixture and
expectations were authored; do not regenerate a frozen input after seeing outputs.

`CRITERIA.json` contains 12 positive cases, 23 structured rejection cases and five
raw JSON/schema cases. They include asymmetric forward, reverse and interior
starts; near-full requests in both directions; exact straight endpoint requests;
disjoint same-curve coverage; unequal existing proportional semantics; a separate
nonuniform collinear cubic; and a nonlinear legacy seam that must keep normalized
curve-t semantics. Unequal creation is an engineering compatibility check, with
no reference easing claim.

The future run exercises `execute` and both MCP protocol revisions
`2026-07-28`/`2025-11-25`. Each positive result must store one ordinary group, one
span per side, `[0,0],[1,1]`, one ID and one revision. It checks independently
integrated lengths and locations, strict new command/side schemas, all nine tools,
and 26 nontransaction command variants. Native output **file bytes** must match
across all three adapters; their different response envelopes are checked
separately. Authored inspection and length reports must carry the owning revision.

Five representative relationships are rebuilt at 90 mm and 35 mm mesh resolution.
Their authored endpoint answers and correspondence must remain within the same
absolute budget. The existing `update_sewing_group` command inserts explicit
quarter-fraction map knots after independently validating the created endpoints.
This exposes portions at `0,¼,½,¾,1` through `inspect-sewing`. Candidate endpoints
are reused only as the input to this existing operation, never to generate an
expected numerical value. Every returned group stitch is also evaluated using
the independent arc metric; legacy stitches use their existing nonlinear t map.
The asymmetric and collinear counterexamples deliberately separate physical
midpoints from half the endpoint parameter.

All rejected operations must retain the exact input, destination and recovery
byte hashes, including representative same-file CLI and both MCP failures. ID
and revision overflow, malformed/nonfinite input, coverage overlap, short spans,
no remaining room and a small overlong excess are covered. A separately labelled
composition creates the physical pair and then asks the existing group validator
to reject an ill-conditioned map in the same transaction. It checks metric-error
rollback **after creation**, not an initial resolver metric error or work-budget
saturation. Writer cases cover an invalid existing destination, a directory,
symlink, recovery directory, absent parent and invalid input, and require that no
temporary files remain. Successful same-file editing must rotate the exact prior
primary bytes to recovery.

Run only after separate owner review and authorization of an immutable candidate:

```sh
python3 evaluate_physical_sewing.py self-check --out fresh-self-check.json
python3 evaluate_physical_sewing.py evaluate \
  --cli /path/to/reviewed/immutable/weftforge-cli \
  --sha256 REVIEWED_BINARY_SHA256 \
  --out fresh-candidate-results
```

Every report/output directory must be new; prior attempts are preserved. Evaluate
first verifies `FREEZE.json` and the supplied binary SHA-256, then records all
bounded failures and verifies the binary still has the same hash afterward.
`RESULT.json` is updated after each observation, and `SUMMARY.json` reports the
bounded adapter result. It explicitly leaves full contract acceptance pending.
Do not change the criteria after candidate execution; retain failures and submit
a new prospective contract/evaluator if a requirement itself must change.

`self-check-001.json` is a preserved initial passing synthetic attempt;
`self-check-001-binding.json` records the then-current evaluator hash.
`self-check-002.json` is the final bound synthetic run. All 23 deliberately wrong
outputs were rejected, including linear-t endpoints/midpoints, wrong direction,
missing group, changed revision/allocator, persistent length locks, unrelated
construction changes, wrong/nonfinite length reports, bad stitch pairing, stale
rest data, mismatched MCP text/IDs, an open schema and modified recovery bytes.
These invented structural outputs exercise assertions, not a real adapter runner
or mesh. No product verdict follows from their passing.

The following remain separate required gates: native transient A/B acceptance and
history; save/new/reopen; real pointer and accessible-control workflow; exact
endpoint ties/projection; Tab/OK/labels/guides; Cancel/close/Escape/tool/reset/
history/load/stale revision; modal isolation and raw text/caret retention; initial
resolver metric failure, cancellation/work exhaustion and injected writer
interruption; all four Rust checks; and registry/map checks from a clean tracked
checkout. The adapter evaluator cannot observe live Editor history, GUI drafts,
revision caches or solver state. It does not run simulation, private reference
assets, a proprietary application, or native version-1/2 migration tests.

Successful construction comparisons are exact **parsed JSON field values** except
the endpoint budget; they do not prove signed-zero bit or JSON number-spelling
preservation. Failed disk operations compare literal byte hashes. Exact curved
full-length equality is not inferred from an analytic length that may differ from
a computed f64 metric by ulps: curved coverage uses near-full tests, while both
exact endpoint directions use independent straight arithmetic. Source/native
checks must retain any stronger bit-level guarantees.

These local files are preparation, not published delivery. Before any public
evidence claim, the owner must Git-track every artifact and run clean-checkout
registry/map checks with a separate Cargo target. No Actions or Pages deployment
was enabled or changed.
