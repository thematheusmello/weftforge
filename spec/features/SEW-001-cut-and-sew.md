# First straight Cut & Sew engineering contract

Approved as a bounded WeftForge engineering increment on 2026-11-16, before its
implementation. This contract is a requirement, not an execution result or
reference equivalence claim. Historical straight-cut, sewing and hole contracts
and their qualified evidence remain unchanged. Native workflow and reference
acceptance must be recorded separately from source tests.

## Shared authoring command

```rust
CutAndSew {
    piece: Id,
    start_curve: Id, start_t: f64,
    end_curve: Id, end_t: f64,
}
```

```json
{"type":"cut_and_sew","data":{"piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}}
```

All five fields are required. The adjacent type/data envelope and data object
are closed; original-byte decoding rejects duplicate/unknown fields. IDs are
positive u64s and parameters are finite f64 values strictly between `1e-6` and
`1 - 1e-6`. Use existing CLI `execute` and MCP `project_execute`, with 1 MiB
adapter inputs and 1–1024 nonnested commands. The same nine tools expose 32
command schemas; the shared Rust enum has 33 alternatives including Transaction.
There is no new verb, tool, dependency or saved format.

The command uses the same private cut helper as `CutPiece`, including canonical
boundary-address ordering, private SplitCurve calls and whole-annotation
assignment. Reserve ten new objects/IDs before either split, then append one
ordinary legacy Seam through the private AddSeam path on the same candidate.
Do not execute two Editor transactions. Ordinary CutPiece still reserves nine,
creates no seam and preserves its existing construction/history behavior.

The retained chord is side A with full `[0,1]`; the child's oppositely directed
chord is side B with reversed `[1,0]`. Correspondence is `[[0,0],[1,1]]`, without
easing. Name the appended seam `Cut seam #<id>`. The pieces retain independent
point/curve IDs and disjoint mesh indices and particles. Weighted sewing constrains
their boundary samples; it does not weld IDs, add triangles/material, introduce cross-cut
bending, create a linked edit or advance simulation.

## Eligibility, atomicity and persistence

All [straight-cut](PAT-006-straight-cut.md) eligibility and conservative numerical
refusals remain in force: a hole-free simple straight owner, 3–4096 outer edges,
two distinct exterior curves, strict interior addresses, whole chord inside,
and valid children. Refuse curved, same-edge, vertex/near-vertex, internal, hole,
cross-owner, crossing, touching, ambiguous or work-exhausted cases. Preserve
±1000 m coordinates, 20,000 total objects, 16 MiB native input, current mesh
limits and one shared 1,000,000-step containment budget through the outer
transaction and final validation. Do not reset work at the cut/seam boundary.

Any legacy seam or ordered group on any target outer boundary refuses, even
away from the cut. Unrelated relationships stay exact. Whole annotations move
unchanged to exactly one child; crossing, on-cut or near-cut annotations refuse
instead of becoming sewing targets. Cutting a sewn child requires removing its
seam first. Target relationship remapping remains future work.

One success is one revision and one Undo entry. Failure, including a later
transaction command or final validation/revision overflow, preserves accepted
construction, allocator/history/redo and existing primary/output/recovery bytes.
IO failure preserves the old primary and accepted construction; a late writer
failure may advance recovery to the valid previous primary. Primary and recovery
are not promised as a two-file atomic transaction.

RemoveSeam removes only the constraint, retaining both pieces, cut boundaries,
attributes and allocator high-water. Undo restores the same seam ID. Existing
RemovePiece cleanup removes dependent sewing. Native v5 already stores all
resulting ordinary objects, intervals and maps. Save/reopen retains them; history
and the transient cut-review option are not serialized. No migration or
historical fixture rewrite is needed. SVG/DXF/OBJ remain geometry exports, not
seam round trips.

## Original independent acceptance recipe

Use original R240 `[0,0],[.24,0],[.24,.36],[0,.36]` m at revision1/next11,
owner2/points3–6/curves7–10. The command above cuts at U=.09 m, with .36 m chord.
Swapped input addresses must produce identical output.

| Result | Expectation |
| --- | --- |
| Retained2 / new15 boundary | `[7,18,14,10]` / `[12,8,9,19]` |
| Anchor/copy points | 11/13 belong to2; exact coordinate-bit copies16/17 belong to15 |
| Chords | 18:11→13; 19:17→16 |
| Seam20 | A18:[0,1], B19:[1,0], `[[0,0],[1,1]]`; name `Cut seam #20` |
| Child/union areas | .0324 / .0540 / .0864 m² |
| Density .18 kg/m²: child/total mass | .005832 / .009720 / .015552 kg |
| History | R2/next21; UndoR3 one piece/no seam/next21; RedoR4 same IDs/next21 |
| Abandoned branch | UndoR5; new CutAndSew R6 starts21, child25/chords28–29/seam30/next31 |
| Separate RemoveSeam branch from R2 | R3/next21, both pieces intact; UndoR4 restores seam20 |

Unchanged f64 fields compare IEEE bits, including signed zero; RGB remains f32.
Copied anchors compare exact bits to their sources. Independent ordinary lengths,
areas and masses use absolute 1e-12 m/m²/kg, tilted world landmarks 1e-10 m and
large-offset R240 budgets 1e-9 m/m²/kg. These measurement budgets never relax
strict eligibility. Ten-object exact headroom starts at 19,990 objects; 19,991
must refuse. Ten-ID exact headroom starts at 18446744073709551605 and finishes at
u64::MAX; start 18446744073709551606 must refuse before splitting.

At .005/.07/.5 m resolution, independently inspect actual fresh R240 UV triangles,
complete child boundary walks and duplicated interface, disjoint material
components and per-owner Cloth mass sums before stepping. Every actual Mesh and
retained Cloth stitch must use only piece2/chord18 and piece15/chord19 boundary
samples, disjoint vertex sets, finite convex weights and existing 1e-8 compliance.
Independently evaluate weighted UV/world pairs: normalized fraction f must give
`(.09,.36*f)` on both sides under unchanged placement. Require both endpoints,
all boundary sample locations, monotone noncrossing coverage and no off-chord
stitches. A deliberately forward-B control has .36 m endpoint mismatch despite
equal lengths and a matching midpoint, and must fail this check. Counts, UI
guides, production correspondence or area×density alone are insufficient proof.
The read-only `Cloth::stitches()` accessor exposes retained constructor input;
it changes no solver algorithm. Old-revision mesh refuses Cloth initialization.

Repeat L eligibility/sewing at .07/.5 m; preserve the established .005 m vertex-cap
refusal and make no fine-L Cloth claim. Keep ordinary unsewn CutPiece's exact
construction and zero-stitch behavior, annotation/target-sewing/hole refusals,
object/ID/work/revision limits, late transaction failure, strict adapter schemas,
both MCP profiles, CLI and separate/same-path failed-IO byte preservation.

## Native integration and evidence boundary

The existing Cut piece review has one unchecked-by-default `Sew cut edges`
option. Off selects CutPiece; on selects CutAndSew. Include it in the preview
cache key. Toggles rebuild only a cloned candidate, preserve raw text/caret and
consume no accepted IDs, revision/history, dirty/rest mesh or simulation state.
Show the candidate seam ID, edge directions and paired lengths. Preserve
owner-only unsnapped exterior picking, typed IDs, invalid/corrected input,
property-entry guard, stale tool/owner/revision/history/file guards, modal Save
and background shields, and explicit/Escape/backdrop Cancel. Opening Return
only opens review; a later Return/Apply commits once. Checkbox interaction must
not accidentally Apply. Success selects the appended child, clears curve
selection and returns to Select; Undo/Redo retain house selection behavior.

Separate native qualification must observe off/on previews, invalid/corrected
input, Cancel, one Apply/Undo/Redo, save, real process restart/reopen and seam
inspection/removal with both editable pieces retained. Required local Rust gates
and source tests do not establish this native lifecycle or reference parity.
Full physics, calibrated fabric, drape and the remaining studio roadmap stay open.

Public intent comes from the official [Cut/Cut & Sew manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358433146393-Cut-Cut-Sew),
reviewed before this implementation. It describes dividing patterns using internal
lines/shapes with optional sewing; it does not establish this boundary-address
workflow, eligibility, orientation, identities, offsets or physical results.
One later reference question remains: does original R240 with the straight
internal line `(.09,0)→(.09,.36)` retain two separate editable patterns and their
connecting sewing across save/quit/reopen? That reference lifecycle is unobserved.
Multiple/curved/vertex cuts, holes/slits, internal-shape conversion and target
sewing remapping remain future work.
