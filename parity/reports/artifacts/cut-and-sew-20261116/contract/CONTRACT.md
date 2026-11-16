# First straight Cut & Sew — engineering proposal

Prepared 2026-11-16 from public documentation and the project's own source only.
This is a private proposal for the next increment after the current hole
qualification. It contains no implementation, execution result or reference
equivalence claim. Runtime source is `a021a2581c8e1dd000e535a417dcd964c3835884`;
the inspected metadata ref is `52163adb86b9f48fdd06f042e4ddf615797c3947`.
[SOURCE-REACH.md](SOURCE-REACH.md) records actual read hashes and future reach;
[ORACLE.json](ORACLE.json) contains independent analytic expectations.

## One command, two pieces, one seam

Add this shared typed alternative without changing existing `CutPiece` fields,
callers, JSON or successful construction:

```rust
CutAndSew {
    piece: Id,
    start_curve: Id, start_t: f64,
    end_curve: Id,   end_t: f64,
}
```

```json
{"type":"cut_and_sew","data":{"piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}}
```

All five fields are required; preserve closed objects, original-byte duplicate
rejection, positive u64 IDs and finite f64 parameters. Use existing CLI `execute`
and MCP `project_execute`; add no verb, tool, dependency or document format.
The proposed totals become 33 Rust alternatives including Transaction, 32 schema
commands and the same nine MCP tools.

Extract the existing inline cut construction into one private helper returning
`[retained_chord, new_chord]` IDs. Both commands use it, including its existing
private SplitCurve applications, canonical address order and annotation
partition. CutAndSew reserves capacity for ten new objects/IDs before allocation,
then appends one ordinary legacy Seam through the existing private AddSeam path
on the same candidate. CutPiece reserves nine as today. Do not perform two
Editor executions or expose a new result/report type. A local reservation
argument is sufficient if needed; no new command mode field is required.

The helper's geometry, point duplication, global object order, retained/new
identity, material, placement, grain and annotation behavior stay exact. The new
seam is appended after the nine construction objects, named `Cut seam #<id>`:

- A is the retained chord with `t0=0`, `t1=1`.
- B is the new child's chord with `t0=1`, `t1=0`.
- Correspondence is exactly `[[0,0],[1,1]]`; there is no easing.

The authored chords run in opposite directions. Reversing B therefore pairs
the coincident A/A′ and B/B′ endpoints. Two independent pieces keep disjoint
Point/Curve IDs, mesh indices and particles. The ordinary physical seam constrains
their boundary samples; it does not weld construction, add material, create
shared triangles/bending across the cut or introduce a persistent linked edit.
No simulation step or retained drape state is part of this authoring operation.

## Existing boundaries remain in force

Reuse all straight CutPiece eligibility and conservative numeric/work refusals:
hole-free simple straight owner, 3–4096 outer edges, distinct exterior addresses,
strict `1e-6 < t < 1-1e-6`, existing upward-rounded coordinate/clearance
conventions, inside chord and whole-annotation assignment. Reject curved owners,
same-edge/vertex/near-vertex addresses, internal/hole/cross-owner addresses,
crossing/touching/ambiguous geometry and invalid children. No annotation is
converted, split, dropped or used as a sewing target. In particular, an existing
annotation on the proposed chord still refuses rather than becoming the seam.

Refuse any existing legacy seam or ordered group referencing any target outer
boundary, including intervals remote from the cut. Unrelated relationships remain
bit-for-bit exact. Cutting an already sewn child requires removal of its seam
first; target sewing remapping remains later work. Current v5 Hole representation,
all hole restrictions and Cut-on-holed refusal remain unchanged.

Keep ±1000 m authoring limits, 20,000 total objects, 16 MiB native documents,
1 MiB adapter input, 1–1024 adapter commands, current meshing limits and one shared
1,000,000-step containment budget across the outer Editor execution and final
validation. Count the extra seam in object/ID headroom; do not reset work at the
helper, seam or transaction boundary. Keep revision overflow and final validation
guards. A command/schema failure, including one after cut construction or in a
later transaction command, preserves the accepted project, allocator, history,
redo and primary/output/recovery bytes. Failed IO preserves the old primary and
accepted construction; a late writer failure may advance recovery to the valid
prior primary. Do not promise atomic replacement of both files.

One successful command is one revision and one Undo entry. For original R240,
R1/next11 becomes R2/next21 with pieces2/15, chords18/19 and seam20. Undo gives
R3, one piece, no seam and next21; Redo gives R4 and the same construction/seam IDs.
Undo again then accepting a new CutAndSew starts at21 and ends next31, with new
piece25, chords28/29 and seam30. No abandoned ID is reused.

`RemoveSeam { seam: 20 }` removes only this constraint. Both pieces, their
boundaries/endpoints, attributes, geometry and next21 remain; it must not fill,
join or merge the cut. Undo restores the same seam20. Existing RemovePiece
dependency cleanup remains in force. Native v5 already stores all required
ordinary objects: no migration or fixture rewrite is needed. Save/reopen retains
the two pieces and seam intervals/map; undo history remains session-local.

## Native reuse

Add one unchecked-by-default `Sew cut edges` option to the existing Cut piece
review. Unchecked produces exactly today's CutPiece; checked produces one
CutAndSew. Include the option in the CutPreview cache key. Changing it rebuilds
only a cloned preview, preserves raw t text/caret and consumes no accepted IDs,
revision, history, dirty state, rest mesh or simulation state. Show whether sewing
will be created and the candidate seam ID/edge directions/paired lengths using
ordinary inspection; no new picker or gesture framework is needed.

Preserve owner-only exterior picking, unsnapped t, raw incomplete/invalid input,
property-entry guard, correction, explicit/escape/backdrop Cancel, stale
owner/revision/tool/history/file guards, failed-IO retention and modal background
authoring/Save shielding. Opening Return only opens review; a later Return or
Apply commits once. Checkbox interactions must not accidentally Apply. Successful
Apply still selects the appended child, clears curve selection and returns to
Select. Undo/Redo follow existing selection policy. Native qualification must
include off/on preview, invalid/corrected input, Cancel, one Apply/Undo/Redo, save,
actual process restart/reopen and inspection/removal of the persisted seam while
both editable pieces remain. This document is not an execution procedure.

## Future acceptance checks

- Preserve the original unsewn CutPiece R240/L fixtures and every serialized
  field/ID/history result at the current v5 baseline. It still creates no seam or
  stitches, uses nine IDs and keeps its original refusals.
- Independently walk both R240 child boundaries and inspect all point/curve
  ownership, duplicated coordinates, opposite chord direction and .36 m paired
  lengths. Swapping the two command addresses gives identical output. Material,
  placement/grain and annotations follow the existing cut contract, including
  tilted/large-offset cases; no automatic layout offset is added.
- At .005/.07/.5 m, inspect actual fresh R240 UV triangles, boundary samples,
  Mesh stitches and actual retained Cloth stitches before any simulation step.
  Check two disjoint material components, full duplicated interface, child/union
  areas .0324/.0540/.0864 m² and independent per-owner stable mass sums
  .005832/.009720/.015552 kg at .18 kg/m². Existing ordinary tolerances remain
  1e-12 m/m²/kg and tilted world landmarks 1e-10 m; large-offset budgets stay
  1e-9 m/m²/kg. Stale pre-cut mesh must refuse; a fresh mesh must construct Cloth.
- For every seam20 stitch, both A indices must be actual piece2/chord18 boundary
  samples and B indices piece15/chord19 samples, with disjoint vertex sets and
  finite convex weights. Independently evaluate weighted UV/world rest points:
  for fraction f both equal `(.09,.36*f)` under the shared placement. Require
  coverage of both ends and all chord boundary sample locations, monotone
  noncrossing pairing, and no off-chord/other-piece stitches. Do not infer this
  from seam count, UI guide, length equality or a production correspondence call.
  A deliberately forward-B control has .36 m endpoint mismatch and must fail this
  independent check even though midpoint/length checks pass.
- Cloth currently has no public retained stitch accessor. Add only a read-only
  `Cloth::stitches() -> &[Stitch]`, following its existing positions/masses getters,
  so checks can verify the actual constructor input, weights and existing 1e-8
  compliance. No solver algorithm change is needed. Mass/positions and ordinary
  unsewn zero-stitch behavior must remain unchanged by the new getter.
- Repeat successful L eligibility/sewing at .07/.5 m; keep the established fine-L
  .005 m vertex-cap refusal visible. Do not raise the cap or claim fine-L cloth,
  drape, calibrated material or reference-physics acceptance.
- Cover unchanged geometric/annotation/target-sewing/hole refusals; ten-object
  exact headroom versus nine-only headroom; ten-ID exact headroom versus
  nine-only headroom; revision/work exhaustion; a later transaction failure;
  strict bytes/schema/duplicate/unknown/missing/nonfinite inputs; both MCP protocol
  profiles and CLI; separate and same-path failed IO. A failed candidate or IO
  must not replace accepted construction or existing primary output.

The exact future source/API reach is in SOURCE-REACH.md. After separately approved
implementation, run the four required local gates in a separate target:
`cargo fmt --all --check`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo xtask verify`.
Keep unknown/blocked/failed/stale evidence truthful in STATUS.md and
parity/features.yaml. Publish evidence only when every artifact is tracked and
registry/map checks pass from a clean checkout. Keep Actions disabled, static
Pages unchanged and root README focused on WeftForge.

## Public intent and one later reference question

The official [Cut/Cut & Sew manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358433146393-Cut-Cut-Sew),
read 2026-11-16, describes dividing a 2D pattern using internal lines/shapes and
optionally establishing sewing between the divided patterns. It documents
Transform/Edit Pattern and Trace routes; it does not establish this boundary-
address UI, supported numeric cases, identity, offset, undo or physical results.
The shorter supplied article ID `47358433193` was inaccessible; the official
search result and accessible page use `47358433146393`.

One bounded reference-runtime unknown for later: on original R240 with one
straight internal line `(.09,0)→(.09,.36)` m, does one Cut & Sew leave two
separate editable patterns whose connecting sewing survives save/quit/reopen?
Observe that lifecycle only after foreground access works. No reference app,
binary, installer, account or REA runtime was touched for this proposal. All
reference eligibility/orientation/offset/identity details remain unobserved.
Multiple/curved/vertex cuts, holes/slits, internal-shape conversion, target sewing
remapping and the rest of the studio roadmap remain product work.
