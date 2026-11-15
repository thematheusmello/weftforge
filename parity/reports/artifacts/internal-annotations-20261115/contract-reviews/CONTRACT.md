# First editable internal segments, then bounded cuts

Source-only engineering proposal, 2026-11-15. Review target, not implementation
or delivery evidence. All source conclusions below use immutable commit
`c4be919391efb8c39e035bb7c40e74ea52c592f4` (physical-sewing candidate
`corephysicalsewing26f`, `nativefixed`) through `git show`. No mutable GUI source,
Cargo build/check, application, reference, CUA, registry edit or public change was
used for this proposal. The independently assigned NewEmpty fix is outside this
scope; any later implementation/runtime binding must name its actual integrated
source and binary rather than inherit this baseline's identity.

## Bounded result and evidence boundary

The next integrated increment creates, selects, edits, removes, saves and reopens
one or more **straight internal annotations owned by an existing piece**. It has
a native two-point tool and precise endpoint controls, the shared command engine,
CLI/MCP execution, one gesture per undo entry, validation and strict persistence.
The lines remain editable construction. They do not split cloth, constrain its
triangulation, create a hole, sew, fold, bend, perforate, topstitch or affect mass.
There is no simulation or reference-parity claim.

This advances the internal-primitives dependency of ROADMAP Gate 2 and the
PAT-006/PAT-007 work, while leaving the complete studio target intact. The pinned
registry contains 56 coarse feature groups: 39 partial and 17 planned, with no
group verified against the reference. PAT-006 is planned / unobserved / not_run /
missing. Do not create another coarse feature or turn this subset into completed
PAT-006, PAT-007, Gate 1, Gate 2 or reference coverage.

Reviewed public context, read from that same commit:

- `ROADMAP.md`: internal lines/holes, complete curve editing, cut/split/join,
  relationship preservation, revision-tagged derived state, strict IO and actual
  native acceptance remain required.
- `spec/inventory/weftforge-implementation-map.json`: `internal-primitives`
  (`PACK-internal-primitives`) and `cut-and-sew` (`PACK-cut-and-sew`) are planned.
  WP-06 requires an independently checked original object, retained attachment
  IDs, supported editing/round-trip and atomic rejection. The `concave-internal`
  fixture is a recipe, not an executed result.
- `spec/inventory/md-2026.1.237-components.json`: toolbar control
  `MDUI-AE49A7523060`, Internal Polygon/Line, and its displayed G hint were
  observed; behavior is `not_executed`. The map also contains Internal Rectangle,
  Internal Ellipse, Cut and Cut & Sew menu presence. These are not observed
  creation, containment, cutting or history semantics.
- `spec/features/SEW-001-oriented-segment-sewing.md` distinguishes the manual's
  internal-line/shape sewing description from the bounded observed boundary
  cases. It supplies no executed internal-sewing contract.
- `spec/features/PAT-005-sewn-curve-editing.md`,
  `PAT-004-piece-rigid-transform.md` and
  `PAT-004-native-transform-selection.md` supply the existing shape-preserving
  boundary split/join, arrangement/grain compensation, single transaction and raw
  numeric input policies. Their reference unknowns remain unknown.
- `docs/file-formats.md` defines strict v1/v2/v3 reads and exterior-only SVG/DXF
  profiles. No public executed internal-line behavior observation or reviewed PAT-006 runtime
  behavior contract exists at this baseline.

The rules below are explicit **WeftForge engineering policy for this proposed
increment**. They must not be relabeled as reference behavior. Reference questions
remain: outside/on-boundary/crossing behavior, snapping and attachment, grouping,
curve/shape creation, internal sewing/folds, cut topology, ID distribution,
generated sewing and automatic 2D cut offset. The observed offset preference has
not been executed; no offset distance, direction or transaction rule is inferred.

## Current source facts that the change must reach

| Source | Existing behavior and implication |
| --- | --- |
| `crates/weftforge-core/src/document.rs` | `Point` is an ID and f64 U/V position. `Curve` references start/end IDs and optional absolute cubic controls. `Piece.boundary` is an ordered closed chain. `Project::validate` counts all objects, enforces globally unique IDs, rejects cross-piece shared geometry and requires every point/curve to belong to a boundary. Simply adding free points/curves currently creates rejected orphans. |
| Same file, piece edits | `ResizePiece`, `TransformPiece` and `RemovePiece` traverse boundary curves and collect only their start IDs, which suffices for a closed chain. An open internal segment also needs its end ID. `piece_bounds` uses analytic outer cubic bounds and must remain outer-only. |
| Same file, topology/sewing | `SplitCurve` and `JoinCurve` find ownership through `Piece.boundary`. Seams/groups currently validate curve existence, which implicitly means boundary-only because orphans are forbidden. After adding internals, existence alone would wrongly admit physical sewing to an annotation. |
| Same file, history | `Editor::execute` clones, applies and validates before replacing construction/history. Undo/redo increment revision and keep the allocator high-water mark; discarded IDs are not reused. A transaction validates the final result and commits once. |
| `crates/weftforge-core/src/geometry.rs` | Meshing samples only outer boundary curves, triangulates with no holes, refines, then derives sewing. `validate_contour` uses bounded polyline approximation tied to `resolution.min(0.02)`; it is not a sufficiently resolution-independent containment predicate for a new annotation. |
| `sewing.rs` / `simulation.rs` | Authored length helpers can measure a straight curve without mesh ownership. Actual weighted stitches require mesh boundary samples. `create_cloth` rejects a mesh from an obsolete document revision. Measuring an internal line must not make it a sewing target. |
| `crates/weftforge-core/src/persistence.rs` | Current writer version is 3; unknown fields are denied. V1 has a frozen piece type, but V2 currently reuses current `Piece`. Adding a required field to current `Piece` therefore needs an explicit frozen legacy piece type for both v2 and v3. |
| `apps/weftforge/src/main.rs` | `selected` + `selected_curve`, boundary picking, inspector drafting and `rebuild` assume the selected curve is in `boundary`. Point drag searches global points. Sewing routes reuse boundary picking. Extending one shared nearest-curve result indiscriminately would make internals sewable. |
| Same native file | `apply_inspector_action` combines pending valid edits and an action atomically. `RetainedInspectorInput`, `numeric_value`, `numeric_commit`, `finish_numeric_action`, `clear_numeric_fields` and transform modal handling already protect raw text, focus/caret and delayed blur. `execute` may commit an inspector draft separately; it is not the route for a new compound one-gesture action. |
| Existing save limits | `Project::save_impl` can refresh recovery to the previous valid primary before an injected interruption/failed primary replacement; it guarantees the primary survives, not byte-identical old recovery on every IO failure. Native `Studio::save` calls `commit_draft` before the chooser/IO. General Save-cancel/failed-save draft atomicity is not established by this source and must not be inferred from the export path. |
| CLI/MCP | CLI `execute_commands` and MCP `project_execute` already execute 1–1024 shared commands atomically and save only after success. MCP `command_schema` is manual and must change with the command enum. CLI `inspect` exposes every point/curve but its pattern summaries currently omit boundary/internal ownership. |

## Smallest durable representation

| Option | Cost / consequence | Decision |
| --- | --- | --- |
| Keep line coordinates only in UI, mesh, or an unowned project list | Cannot preserve editable ownership, stable relationships or reopen; contradicts authoritative construction. | Reject. |
| Add a separate line object duplicating endpoint coordinates | Needs another geometry model, transform/validation/length paths and later conversion for cubic editing. | Reject. |
| Add `Piece.internal_curves: Vec<Id>` using existing points and curves | One new authored ownership field; existing curve ID is the annotation identity. Supports the first segment without copying geometry or inventing shape semantics. | Recommend. |
| Add a path/shape ID, role enum, graph or generic feature framework now | Adds grouping, closure and topology semantics that the first segment does not use and the reference has not established. | Defer. |

Each piece gains the required `internal_curves` array. A first-increment entry
references exactly one ordinary `Curve` with `controls: None`, backed by two new
ordinary `Point`s. Its curve ID is the selectable/deletable annotation ID; the
point IDs remain stable during edits. Allocate start point, end point, then curve
from the existing monotonic allocator and append the curve ID to its owner list.
No new object type, name field, persistent tool draft, asset, dependency or mesh
vertex ID is needed. Display a useful label such as `Internal line #13`.

The vector preserves authored/display order. It is **not** an ordered connected
path, a group, a region or a hole. Crossings and coincident coordinates between
different annotations are permitted and remain independent: no auto-split,
auto-weld or inferred closed shape. Each segment's two endpoints are exclusive
to that segment in this first schema; commands never reuse boundary or other
annotation endpoint IDs. Equal coordinates do not imply equal IDs.

Validation keeps every existing boundary rule and adds:

1. Each curve occurs exactly once in all boundary/internal ownership lists. An
   internal ID cannot repeat locally, belong to two pieces or also be a boundary.
   A point cannot be shared between pieces; internal endpoints cannot share IDs
   with a boundary or another internal segment in this first increment.
2. All referenced objects exist; global ID uniqueness and `next_id` checks remain.
   The used-point set includes **both** endpoints of each open segment. All
   points/curves still need an owner; unrelated orphans remain rejected.
3. Existing finite ±1000 m f64 coordinate validation applies to both endpoints.
   Distinct IDs are insufficient: Euclidean segment length must exceed `1e-9 m`.
   Internal cubic controls are rejected even though the reusable Curve type can
   represent them. Boundary cubics remain supported without reinterpretation.
4. Count the new points/curves in the existing 20,000 object cap; do not give
   annotations an independent unlimited collection. File size remains 16 MiB.
5. Enforce the whole-segment containment policy below on load and every command's
   final candidate, including outer-boundary edits.

This leaves a direct future extension: internal cubics use the same curve/point
identity and absolute controls. Later open/closed shapes can explicitly group
retained curve IDs, introduce shared joints and their own stable shape ID under a
reviewed migration. Existing loose segments can become one-segment groups without
renumbering points/curves. Do not preempt that design by treating flat list order,
crossing coordinates or repeated clicks as a persistent group now. Topological
cuts will consume or copy authored geometry with an explicit relationship policy;
they will not reinterpret all internal annotations as mesh boundaries.

## Whole-segment containment, with fixed conservative limits

Policy: a committed annotation must be strictly inside its owning outer contour,
with at least `1e-6 m` clearance. No endpoint or subsegment may touch, overlap or
cross the outline. Reject outside/uncertain cases with the internal curve ID and
a useful reason; retain construction and history. This is a scope choice, not a
reference rule. Boundary attachment and cut anchors are deferred deliberately.

Endpoints, midpoint tests and piece bounds are not enough for a concave piece or
a bowed cubic boundary. The predicate must establish containment of **the entire
straight segment**, independent of mesh/particle resolution:

- Use the existing de Casteljau/point-to-segment geometry approach to produce a
  closed boundary polyline with a certified maximum curve-to-chord deviation of
  `1e-7 m`. Do not use the current mesh-spacing stopping condition. The maximum
  distance of all four restricted cubic controls to the leaf chord bounds the
  cubic in that chord's tube by its convex hull; subdivision, not a fixed finite
  set of point samples, establishes the bound. Lines need no subdivision.
- Reserve `r = 256 * f64::EPSILON * max(1, maximum absolute coordinate)` metres
  for the boundary controls and candidate endpoints, rounding the comparison
  conservatively. Require every polyline-edge/segment distance to exceed
  `1e-6 + 1e-7 + r` metres. Equality and unresolved near-threshold cases reject.
  The extra tube width makes acceptance conservative for the actual cubic.
- Use a winding/ray predicate with the normal half-open vertex convention to
  establish that one candidate endpoint is inside the closed polyline. Since the
  whole connected segment is separated from every contour edge by the bound,
  its inside/outside classification cannot change along its length. Reject an
  invalid/self-touching or uncertifiable fine contour rather than guess.
- Build the bounded contour once per piece with internal annotations per
  validation, and reuse it for that piece's segments. Reuse existing depth-20 and 20,000-sample
  limits. Cap new containment work at 1,000,000 subdivision/predicate steps per
  project validation; count subdivision, winding and candidate-edge checks and
  fail explicitly on exhaustion. No user tolerance option, global spatial index
  or persistent containment cache is required for this increment.

Do not quietly loosen the clearance or flatness thresholds to pass a fixture.
Large/high-curvature boundaries or many annotations can hit these explicit
resource limits even when a mathematical answer exists. The existing outer
contour's approximate validity model is not upgraded into an exact CAD topology
claim by this work. If the bounded certificate cannot be implemented soundly and
within the declared work, stop that acceptance case and report the failure; do
not substitute endpoint-only, AABB or resolution-dependent acceptance.

## Shared commands and effects on existing operations

Add only these two enum variants and strict snake_case JSON schema alternatives:

```rust
AddInternalSegment { piece: Id, start: [f64; 2], end: [f64; 2] }
RemoveInternalCurve { curve: Id }
```

JSON types are `add_internal_segment` and `remove_internal_curve`; every field
above is required, coordinates are metre arrays of exactly two finite numbers,
IDs are nonzero u64s, and extra/duplicate fields reject. Removal finds and checks
an internal owner before deleting; a boundary curve ID is a useful error, never
an invitation to damage the outline. It removes that curve, its two exclusive
points and its owner-list entry while keeping the allocator high-water mark.

Reuse `MovePoint` for an internal endpoint. Editing both endpoints is one
`Transaction` of the changed MovePoint commands, not a new update object or four
separate scalar commands. Final-candidate validation permits an atomic edit whose
intermediate endpoint state would be invalid. Failure retains the entire prior
project, revision, undo/redo and allocator; it consumes no observable IDs.

| Existing operation | Required first-increment behavior |
| --- | --- |
| `MovePoint` on outer or internal geometry | Validate all retained annotations against the resulting owner contour. Reject an outside/near-boundary or degenerate result atomically; do not clamp, delete, project or detach an annotation silently. |
| `SetCurveControls` | Remains boundary-only for this increment. Reject an internal target explicitly; v4 input also rejects `Some` internal controls. Reuse Curve now without claiming internal cubic editing. |
| `ResizePiece` | Use the existing analytic **outer** min/bounds and affine scaling. Transform every owned boundary/internal endpoint once, including both ends of an open segment, and absolute controls wherever applicable. Preserve IDs/list order and existing placement/grain/sewing policy. Revalidate clearance; a very small resize may reject. |
| `TransformPiece` | Apply the existing original-coordinate rigid transform to both annotation endpoints with the boundary. Keep its established placement compensation and grain rotation, so annotation landmarks on the authored plane have the same arranged world positions within the PAT-004 tolerance. Do not add a transform-policy flag. |
| `SetPlacement`, grain/material/resolution edits | Internal coordinates/ownership remain unchanged. Placement changes their conceptual arranged plane with its piece; there is no cloth-following internal 3D display claim. Mesh resolution cannot change containment acceptance. |
| Boundary `SplitCurve` / `JoinCurve` | Preserve the internal ID list and all annotation coordinates. Existing seam/group remapping remains authoritative. Validate retained annotations after the final shape-preserving result. These commands remain boundary-only and reject an internal ID before mutation; internal split/join is a later increment. |
| `RemovePiece` | Remove all of that piece's boundary and internal curves and all their endpoint IDs; use both endpoint references. Keep existing whole-dependent-seam/group removal and unrelated construction. Undo restores the annotations with the owner in one step. |
| Legacy seams, groups, AddSewingByLength and imported saved sewing | Require every referenced curve to be in an outer boundary, rather than merely present in Project.curves. Reject internal annotation targets through the core validator, not just a disabled native button. No internal sewing or automatic cut-and-sew claim. |
| Authored length queries | Existing `interval_length(CurveInterval::full(id))` can measure an annotation. Show it as applied authored/rest length. This read-only geometric ability is separate from physical sewability. |
| Rest mesh / cloth / display state | Internal annotations are absent from triangulation vertices, triangles, BoundarySample, stitches, mass and constraints. A successful annotation edit still creates a new document revision and invalidates old jobs/rest mesh/cloth through existing rules; a rebuild has the new revision. Do not retag old state as current. For annotation-only edits the fresh mesh's geometry/topology/UV/sewing fields should be identical, apart from its source revision. |

## Strict v4 persistence and interoperability

Set `FORMAT_VERSION` to 4 for the new authored field. Every v4 piece requires an
explicit `internal_curves` array, even when empty; project fields such as
`sewing_groups` remain required. Retain `deny_unknown_fields`; no `serde(default)`
on the new field to make a changed v3 shape look compatible. A v1/v2/v3 document
with `internal_curves`, even `[]`, must reject as a mixed schema.

Use explicit old-format types: freeze a PieceV2V3 with exactly the current v3
piece fields; use it in ProjectV2 and a new ProjectV3. Keep PieceV1/MaterialV1.
Convert old pieces to current Piece by adding only `internal_curves: []`.

| Read version | Lossless semantic conversion to the v4 model |
| --- | --- |
| v1 | Keep the established stretch-to-both-axes mapping and zero-grain default; add empty sewing groups and empty internal lists. Preserve all existing IDs, order, curve controls, seam maps, revision, allocator and supported numeric values. |
| v2 | Preserve existing grain, orthotropy, legacy seams and all other authored fields; add empty groups/internal lists. |
| v3 | Preserve every legacy seam and ordered group, interval/map/list order, grain/material/placement and numeric field; add empty internal lists only. |
| v4 | Preserve exact supported authored objects, relationships and endpoint f64 values, including signed zero and hard decimal round trips where permitted. Validate internal ownership, straightness, containment and all existing limits. |

Reading is not an authoring transaction: do not allocate IDs, change revision,
create undo history or overwrite the source. Saving any successfully read old
document writes v4; a primary replacement retains the previous valid file's
original bytes in recovery using the existing staging/rename path. No v3 writer
or export-to-old-native option is necessary.

Unknown versions, unknown/duplicate fields, missing required fields, nonfinite or
overflowing numbers, invalid IDs/geometry, oversized input and unsupported saved
internal controls reject. Preserve the old open document and pending numeric raw
text/history on failed load; preserve existing primary/output files on failed
save. Validation/schema/command failure must not start file replacement and
leaves recovery bytes unchanged. Preserve the established interruption behavior:
if the valid primary has already been copied to recovery before a later failure,
the primary remains unchanged and recovery holds that valid prior primary; do
not promise byte-identical older recovery or an atomic two-file transaction.
Keep explicit recovery and rejection of invalid/nonregular existing destinations;
never silently load recovery or strip unfamiliar fields and resave a partial
project. A future version number remains a hard stop.

New annotation editors keep their draft outside construction, and their modal
blocks background Save. General native Save currently commits an ordinary
inspector draft before chooser/IO; fixing that broader pre-existing behavior is
not hidden in this increment. A claim that failed/cancelled Save preserves every
uncommitted general inspector draft needs a separately reviewed candidate-save
change and actual evidence. Keep that limitation visible rather than report a
global Save atomicity pass from annotation tests.

V4 is a strict supported-model format, not a promise to understand every future
internal shape or role. A later grouping/hole/cut field requires an explicit
schema/version decision and migration. Enabling cubics in the reusable Curve
type also requires a reviewed supported-model/version decision: an older v4
reader must fail on unsupported data, never silently straighten or discard it.

Keep current interchange profiles bounded:

- Native v4 and its autosave/recovery paths retain the annotations completely.
- Existing SVG and FILE-005-v1 DXF remain **exterior-boundary-only**. They omit
  internal annotations. Do not add open SVG paths, DXF layers, a profile option or
  an internal export claim in this increment. Make the omission explicit in the
  format matrix, native export help, CLI help and MCP tool descriptions; the SVG
  description may state it too. Import remains exterior-only and creates pieces
  with empty internal arrays. Preserve existing source/destination guards.
- OBJ remains the rest/current cloth surface profile, without annotation line
  primitives or authored features. Annotation-only edits do not alter OBJ surface
  geometry; native project saving is required to keep editable internals.
- Existing public v1/v2/v3 fixtures and historical evidence retain their original
  version/bytes/hashes. Do not bulk-upgrade fixtures or rewrite prior evidence to
  make tests pass.

## Native tool, precise editing and input lifetime

Use a single Internal line tool/action in the existing tool enum/rail, applicable
2D creation menu and command search. It operates on one selected owner piece.
Do not guess the reference's G shortcut or create a general path tool. Disable
or explain the missing-owner case. A line preview has a clear annotation style;
no vendor palette or fold display is implied.

1. First primary click captures the selected piece, accepted input revision and
   snapped start coordinate. The second captures the snapped end and dispatches
   one AddInternalSegment through `apply_inspector_action`. Capture snap/Alt for
   each click using the house PositionSnap conversion; numeric endpoints bypass
   snap. There is no revision/allocator change before accepted commit.
2. Draw a transient preview from the original captured data. A failed second
   point keeps the anchor/candidate and useful error, allowing correction or
   precise entry; it does not manufacture partial authored points. Escape or
   Cancel abandons only the line gesture, with no history entry.
3. Provide precise start U/V and end U/V controls in metres, both for creation and
   the selected annotation. Reuse the existing numeric widget and retained-input
   helpers/modal pattern. Apply/Return submits one command or one transaction of
   changed MovePoint commands. A cancelled or unchanged endpoint editor creates
   no history entry. The applied authored length is reusable feedback.
4. Select a line by its visible 2D segment or a clearly separate inspector list
   under its owner, using the existing selected piece + selected curve identity.
   Show both endpoint IDs/positions and allow endpoint dragging; a drag previews
   on a clone and commits one MovePoint with any valid pending inspector edits
   as one transaction. Pointer press/blur must not commit those edits early.
   The Remove internal line action removes only the annotation. Piece deletion
   remains explicitly labeled; a shared Delete action must not unexpectedly
   delete the owning piece when an internal line is selected.
5. Keep selection/draw picking separate from **boundary-only** sewing/free-sewing/
   ordered-sewing/transform handles. In particular, keep a boundary pick result
   for sewing and do not pass the broader Select pick to those tools. Rebuild's
   selection validity accepts a selected internal ID only for its actual owner.
   Render and pick both endpoints, not just the start of an open curve. Hide
   boundary cubic/split/join controls for an internal selection.
6. Bind transient endpoint/creation/drag state to revision, owner and target ID.
   Tool/owner change, successful new/open/recover, undo/redo or a relevant
   revision change discards stale previews before Apply/release. A failed IO
   operation preserves the document and still-valid gesture. Cross-piece
   selection with staged or incomplete inspector text must preserve/refuse the
   change using the existing Transform selection rule rather than accidentally
   commit or discard that text. No multi-piece draft framework is added.
7. Preserve raw invalid text, f64 backing values, focus and caret on validation
   failure/cancel, including nonnumeric text that has not changed the numeric
   backing value. Reuse RetainedInspectorInput and delayed-blur suppression;
   do not reconstruct text from the last parsed number. Include same-frame
   Tab/AX focus + Apply, separate pointer press/release and unfocused Return.
   Successful commits clear superseded edit buffers so later Tab/blur cannot
   replay an old endpoint value after selection, undo/redo or a new creation.
8. The endpoint modal blocks background pointer, AccessKit and shortcut
   authoring. Cancel restores the previously retained inspector input where its
   revision/identity is still valid. Accepted command actions alone cancel
   workers/mark dirty/rebuild; rejected or cancelled actions retain accepted
   rest/cloth output, redo and pending work.

These are prospective behavior requirements. In-process egui/AccessKit tests do
not establish actual native pointer/focus/rendering behavior or reference parity.

## Original fixtures and prospective acceptance

Freeze these cases and tolerances before implementation/evaluation. Generate
small original constructions in core tests; use an independent analytic evaluator
for retained runtime outputs. No vendor project, binary, asset or extracted code
is needed.

| Case | Original construction and decisive expectation |
| --- | --- |
| R: straight baseline | Unit square at `[0,0]`, segment A=`[1/8,1/4]`, B=`[7/8,3/4]`. Length is `sqrt(13)/4 m`. Starting from the default material/allocator and one AddRectangle, the segment allocates point IDs 11/12, curve 13, next ID 14. The boundary stays IDs 7–10; the internal owner list is `[13]`. |
| L: concave escape | Six-edge L: `[0,0],[1,0],[1,0.4],[0.4,0.4],[0.4,1],[0,1]`. `[0.2,0.8]` to `[0.8,0.2]` has inside endpoints but crosses the missing upper-right quadrant: reject. `[0.2,0.8]` to `[0.2,0.2]` is valid. A bounds/endpoints-only implementation fails this case. |
| C: analytic curved escape | First boundary from `[0,0]` to `[1,0]` has controls `[1/3,2/3]`, `[2/3,2/3]`, giving `q(t)=(t,2t(1-t))`; close through `[1,1],[0,1]`. The region is above the parabola and has area `2/3 m²`. Segment `[0.1,0.2]` to `[0.9,0.2]` has inside endpoints but an outside middle: reject. The same x endpoints at y=`0.75` pass. This is an analytic answer, not agreement with our tessellator. |
| Clearance | On R, endpoint on outline, collinear outline overlap and a segment at y=`0.5e-6` reject; a horizontal interior segment at y=`2e-6` with x in `[0.25,0.75]` passes. Include a candidate in the conservative uncertainty band and require refusal. Repeat all containment cases at resolution 0.005, 0.07 and 0.5 m: outcomes agree. |
| Independence | Two crossing inside segments and two segments with coincident endpoint coordinates remain separate curve/point IDs. Moving/removing one does not mutate or group the other. Zero/near-zero length, identical endpoint IDs and explicitly shared internal IDs reject. |
| Piece edits | Resize R to 2 by 0.5 m: A becomes `[0.25,0.125]`, B `[1.75,0.375]`, with unchanged IDs/order. Rigid transform, inverse and noncommuting `[0.31,-0.47,0.21]` placement follow PAT-004's independent mapping/arrangement checks. Remove owner deletes both internal endpoints; remove line preserves owner/boundary/other annotations. |
| Boundary editing | Boundary split then compatible join preserve annotation IDs and coordinates, with established seam/group remapping. An outer point/control edit that puts an annotation outside fails atomically; a later failing command in a compound transaction also restores all earlier changes. Internal SplitCurve, JoinCurve and SetCurveControls targets reject explicitly. |
| Physical separation | Add/edit/remove an annotation on a sewn two-piece fixture with reversed partial legacy seam and ordered group. Fresh mesh arrays, topology, UVs, BoundarySample and stitches agree exactly with the before mesh except doc_revision. Old mesh is rejected by create_cloth after the edit; new mesh is accepted. An internal ID used by legacy seam, group or length-based sewing rejects without writes. |
| Persistence/history | R → create → endpoint transaction → remove → undo/redo and save/reopen preserves every authored ID/value/list. One accepted gesture is one revision and one undo entry. An abandoned create/undo branch never reuses its IDs. Include exact raw f64 bit checks for permitted hard decimals and signed zero, and existing legacy numeric fixtures. |
| Malformed/bounded input | Missing owner/point/curve, duplicate IDs/list entries/JSON fields, curve in both roles, cross-piece sharing, unowned geometry, nonzero internal controls, invalid next ID, NaN/infinities/overflow tokens, coordinate/object/file limits and containment work exhaustion all fail with the prior state/output/recovery intact. |
| Strict versions | Original v1, v2 and v3 read semantically unchanged and save/reopen as v4 with empty internals. V1/v2/v3 reject a new internal field, even empty; v4 rejects a missing internal list, missing group list, unfamiliar field or unsupported controls; version 5/unknown rejects. Same-path failed command/schema edits leave output/recovery unchanged; interrupted saves preserve the primary and a valid prior-primary recovery. A successful old-primary replacement retains the original old bytes in recovery. |

Numerical acceptance, fixed in advance:

- Stored IDs, relationships, ordering, revision/history transitions and unchanged
  numeric fields: exact equality; use `to_bits` where preserving numeric input is
  the requirement. Integer/dyadic affine fixture outputs above are exact bit
  expectations. Line-length comparison on these metre-scale fixtures is absolute
  `1e-12 m` against the analytic formula.
- Containment uses the fixed clearance/certificate/rounding bounds above, not a
  screenshot tolerance. Positive/negative fixtures have margins much larger than
  the uncertainty band; retain refusal rather than change thresholds afterward.
- Transform endpoint/arranged landmarks use the established PAT-004 `1e-10 m`
  absolute bound on ordinary fixtures and separately `1e-9 m` at ±1000 m scale.
  Use non-right-angle and tilted/gimbal placement cases already covered there.
- At a fixed `1000 px/m` native canvas, pointer-coordinate comparison allows
  `2e-6 m` for screen f32 conversion, separate from numeric controls' core bounds.
  It does not relax containment or alter committed precision. No pixel-only
  check establishes submillimetre geometry or reference agreement.
- For annotation-only edits, compare actual rest/export fields, not just counts
  or bounds: extra holes/triangles can leave both counts/bounds misleading.

CLI `execute` and MCP `project_execute` must create/edit/remove the same R result
and report one transaction revision. `tools/list` must advertise the exact two
variants, required fields, closed objects and fixed coordinate-array lengths.
MCP `project_inspect` already returns serialized construction; CLI `inspect`
must also expose per-piece `boundary` and `internal_curves` ownership, not leave
the user to guess from the global curve collection. New read-only line lengths
reuse existing length queries. Do not add a dedicated CLI subcommand or MCP tool.
Preserve strict parsing of original JSON bytes, tool count and failed output /
recovery bytes; test both late transaction failure and schema rejection.

Actual native runtime is a separate required gate on the integrated candidate:

1. Bind source commit, clean source snapshot, binary hash, own target directory,
   OS/architecture, screen scale, process and run/job handles. Start the real app
   empty after the accepted NewEmpty fix; create the first piece and internal
   line through native controls. Record both two-click and precise routes.
2. Select and numerically edit both endpoints, drag an endpoint, inspect one-step
   undo/redo and exact saved values. Start a failing outside line then correct or
   cancel it. Exercise invalid focused raw text/caret, pointer press/release,
   Return/Tab/AX focus and modal background blocking without losing the source
   draft or old redo branch.
3. Check tool/owner changes and undo/new/open/recover invalidate stale previews;
   failed open retains the old state and relevant raw text. Check failed save on
   already accepted construction and blocked background Save in the endpoint
   modal; retain the general Save draft-atomicity limitation above. Test both valid
   resize/transform and a boundary edit rejected because of the retained line.
   Delete line separately from deleting its owner.
4. Use the original concave/curved outer fixtures for the bounded failure cases;
   record which fixtures were loaded. Those loaded cases are not an empty-project
   garment workflow. Save, close the actual process, restart/reopen, edit again,
   and independently check the retained v4 construction and omission-aware
   SVG/DXF/OBJ profiles.
5. Retain any focus/rendering/cancel/failure gap as a gap. Core tests, an egui test
   process, adapter success or an old native binary cannot substitute for these
   observations. No reference run or physical fold/cut result is implied.

## Exact implementation reach after review

| File(s) | Required work or explicit regression responsibility |
| --- | --- |
| `crates/weftforge-core/src/document.rs` | Piece field/constructor, new typed commands/apply arms, ownership/orphan/role checks, both-endpoint traversal in resize/transform/removal, boundary-only topology/sewing guards, final-candidate validation. Existing Editor atomic/history boundary remains. |
| `crates/weftforge-core/src/geometry.rs` | Private bounded straight-segment containment using certified curved contour flattening and fixed work/tolerance limits. Existing mesh construction stays exterior-only. No general arrangement/cut engine. |
| `crates/weftforge-core/src/persistence.rs` | Version 4, frozen legacy PieceV2V3/ProjectV3 and conversions, strict dispatch and IO failure regression coverage. Preserve staging/recovery guards. SVG description may clarify omissions. |
| `crates/weftforge-core/tests/internal_lines.rs` (new) | Original R/L/C fixtures, containment/ownership, commands, both-endpoint transforms/removal, sewn relationships, mesh separation, stable IDs/history and atomic failures. Use independent analytic expected results. |
| `crates/weftforge-core/tests/migration.rs` | V3 semantic/bit-preserving read → v4, strict mixed-schema and duplicate-field cases, IO/old-primary preservation. Old-version construction helpers must remove new fields when intentionally creating v1/v2/v3, not relabel current data. |
| `crates/weftforge-core/tests/ellipse.rs`, `joins.rs`, `pattern_transform.rs`, `seam_reversal.rs`, `sewing_groups.rs`, `workflow.rs` | Current-output version expectations become FORMAT_VERSION/v4. Preserve intentional old v3 inputs. A formerly unsupported-v4 case must use a genuinely unsupported version (for example 5) and gain a valid-v4 counterpart. Extend only regression cases needed by new ownership. |
| `apps/weftforge/src/main.rs` | Tool label/icon/rail/menu/search, revision-bound two-point/endpoint draft, precise editor and selection/drawing, both-endpoint drag, boundary-only tool picking, reset/rebuild/load/history/input lifetime and role-aware delete. Add meaningful egui/AccessKit regressions beside house draft/focus tests; integrate the separately owned NewEmpty result without changing its scope here. |
| `apps/weftforge-cli/src/main.rs` | Add ownership fields to inspect pattern summaries and explicit exterior-export help. Existing execute/length adapters suffice; no new verb. |
| `apps/weftforge-cli/src/mcp.rs` | Two command_schema variants, required closed fields/arrays and nonzero IDs; clarify export annotation omissions. project_execute/inspect/lengths reuse existing handlers. |
| `apps/weftforge-cli/tests/mcp_stdio.rs` | Schema discovery, CLI execute + MCP create/edit/remove/inspect/lengths, current save-version assertions, malformed JSON and late-failure preservation. Use existing temporary-file/server helpers; update current-output literal v3 expectations only. |
| `docs/architecture.md`, `docs/automation.md`, `docs/file-formats.md` | Explicit annotation role/ownership, command JSON, strict v4/old migrations and forward limits; boundary-only export profiles and no simulation effect. Update recovery/autosave format statements. |
| Prospective `spec/features/PAT-006-internal-segments.md`; `spec/inventory/weftforge-implementation-map.json` / `.md`; `parity/features.yaml`; `STATUS.md` | Publish the reviewed bounded contract only when approved as source work; map actual implementation/evidence and exact next steps/confirmed handles. Keep all unknown/failed/blocked/stale states, coarse counts and no-reference claim accurate. Generated progress changes only through the existing generator after truthful registry updates. |
| Review-only: `crates/weftforge-core/src/lib.rs`, `sewing.rs`, `simulation.rs`; `persistence/dxf.rs`, `persistence/dxf_import.rs`, `persistence/svg.rs`; CLI import tests and core import/export tests | Wildcard exports already expose Piece/Command. Generic metrics and the document-to-solver boundary need no new physical algorithm. Existing exterior import profiles initialize empty internals through AddPolygon; existing export loops stay boundary-only. Verify these callers, guards and omission behavior rather than add features. |

No planned edits to Cargo manifests/lock, `weftforge-sim`, root README, reference
inventory observations, historical artifacts, GitHub workflows or the static
Pages site. Actions and automated Pages deployment stay disabled. Re-read direct
callers after implementation/rebase; the immutable source scope above does not
authorize overlooking a newly introduced caller.

Run the four integrated local gates only after implementation in its own worktree
target: `cargo fmt --all --check`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo xtask verify`.
Then execute the bounded actual-native gate on that same identified candidate.
Before publishing evidence, require every artifact/fixture/evaluator to be
Git-tracked and rerun registry/implementation-map checks from a clean checkout.
Ignored local proposal files, cached binaries or unbound run handles are not
delivery. Record failures and tested scope instead of extending checks until a
different scenario happens to pass.

## Follow-on cut boundary, after the first increment

The next proposal should be a distinct bounded **topological cut**, following
accepted internal annotations/persistence/runtime evidence. It must specify its
own typed command, geometry construction, native gesture, migration decision,
relationship/invalidation report and independent output evaluator before code.
No cut command, dormant role enum or cut flag belongs in the first increment.

The smallest useful later candidate is one straight boundary-to-boundary chord
on one simple piece: two explicit authored `(curve ID, t)` boundary anchors,
distinct non-vertex/interior parameters, a chord strictly inside between its ends,
and exactly two nondegenerate simple output pieces. This is separate from the
first increment's strictly-inside annotation endpoints; do not silently extend,
snap or attach those endpoints to make a cut possible. A user may use an
annotation as an explicit geometric guide only after the cut operation resolves
and reports its anchors.

Keep that first cut candidate unsewn and reject affected sewing/attachments or an
internal annotation crossing/touching the cut until reviewed remapping exists.
Retain and assign annotations wholly on one output side; do not drop them.
Duplicated cut edges/points need new IDs because output pieces cannot share
construction. Decide prospectively which output retains the old piece ID,
boundary IDs, names, placement/material/grain and which source annotations remain;
report all retained/created/retired relationships. The whole operation, including
any expressly chosen layout offset, must be one undo step, preserve old files on
failure and invalidate derived state. A no-offset engineering choice must be
declared separately from the still-unobserved reference offset preference.

That cut candidate still does not implement holes, open slits, arbitrary paths,
cubic cuts, cut-and-sew, internal sewing, folds, darts/pleats or linked symmetry.
Those remain required roadmap work with explicit contracts and evidence. Avoid
claiming a generic cut model from an unsewn straight-chord case.

Skipped here: implementation, Cargo/native/reference runs and public publication.
Main risks: reference semantics remain unknown; bounded curved-containment work
can refuse difficult geometry, and v4 must never be read by silently extending v3.
