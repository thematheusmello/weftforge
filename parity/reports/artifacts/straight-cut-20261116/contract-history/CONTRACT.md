# First straight topological cut — engineering draft

Prospective WeftForge policy, based on public ad7 source. This authorizes no code
or execution. Reference cut numbers, eligibility, selection and offset remain
unobserved. An internal annotation does not already perform a cut. The full
PAT-006/PAT-007, Cut & Sew, holes and studio roadmap remain open.

## One command, ordinary v4 construction

```rust
CutPiece {
    piece: Id,
    start_curve: Id, start_t: f64,
    end_curve: Id,   end_t: f64,
}
```

```json
{"type":"cut_piece","piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}
```

All fields required; closed objects, original-byte duplicate detection, nonzero
u64 IDs and finite f64 parameters. Add one shared command/schema alternative,
not a CLI verb, MCP tool, persistent cut object, role flag or dependency.
The result is two ordinary editable Pieces with ordinary Points/Curves; native
v4 already represents it. No format bump/defaulted field or legacy-fixture rewrite.
There is no persistent link that later moves the two cut edges together.

Eligibility is deliberately bounded:

- One valid simple outer polygon, 3–4096 authored boundary curves, all with
  `controls: None`. Distinct target boundary curves; both addresses belong to
  this owner. Reuse SplitCurve's strict `1e-6 < t < 1-1e-6`; never snap a t to a
  vertex or accept an internal curve as a boundary address.
- Preserve existing finite ±1000m coordinates, 20,000-object, u64/revision, native
  16MiB and adapter 1MiB/1–1024-command limits. The successful cut adds nine
  objects/IDs; preflight capacity and overflow. Each child has ≥3 edges and must
  pass current contour/ownership/internal validation, including its area floor.
- Use authored straight edges, not mesh samples, for eligibility at every mesh
  resolution. Let `r=256*EPSILON*max(1,max_abs_coordinate)`m and `d=1e-6+r`m.
  Endpoints must be farther than d from old vertices; chord length >2d. Reject
  collinearity/overlap and numerically ambiguous transverse intersections.
- The chord intersects its two addressed edges only at the designated endpoints;
  the far endpoint must be farther than d from each addressed edge’s support
  line, excluding near-collinear ambiguity. Its distance from every other outer edge exceeds d. Its midpoint must be
  strictly inside. These together certify the open chord stays inside; an
  endpoints/AABB or intersection-only test is insufficient. Reject uncertain,
  touching, outside, degenerate or non-simple cases rather than repair them.
- Bound new simplicity/chord/annotation predicates to 1,000,000 work steps per
  outer Editor execution, shared across cuts in its transaction; never reset
  the budget in recursive splits. Count pair, winding and intersection work.
  Existing final-validation/20,000-sample and meshing limits remain additional
  guards. Exhaustion is a useful atomic refusal, not a new user option. Named positive fixtures below
  must pass; do not loosen tolerances or declare them budget failures to hide a bug.

## Construction, identity and relationships

Canonicalize the two addresses by their indices in the ORIGINAL boundary list,
so swapping command arguments does not change identity. Call them A then B.
Use the existing private apply/SplitCurve path twice, A then B, inside the same
candidate; do not call Editor::execute twice or copy its split/ownership logic.
Distinct curves avoid parameter remapping between these two splits.

The original piece ID/name remains on the wrapping B→A arc containing the old
first boundary vertex. Rotate its final boundary list to start with its original
first curve. The other child follows A→B and starts with A's split tail. Preserve
parent winding. Close each arc with its own oppositely directed straight chord.
Original vertices each stay with their one arc. Original split-anchor points
belong to the retained piece; copy their coordinate bits into two NEW endpoints
for the new child, retargeting that child's incident split curves. No child
shares a Point/Curve ID with another piece, even where coordinates coincide.

Both children copy material ID, placement and grain EXACTLY; do not rebase UVs or
introduce an automatic layout offset. This zero-offset choice preserves the
arranged union and is separate from the unobserved reference offset preference.
Keep old global object order, replace the retained piece in place and append the
new child. Name it `Cut piece #<new_id>` to avoid truncating a long source name.
Names are not identifiers: show the selected piece ID with its name so duplicate
user names remain distinguishable; no persistent parent/child field is needed.

For allocator high-water N, fixed allocation order is:

| Allocation | IDs |
| --- | --- |
| Split A point / tail curve; split B point / tail curve | N / N+1; N+2 / N+3 |
| New piece; copied A / B endpoints | N+4; N+5 / N+6 |
| Retained chord A→B; new-child chord B′→A′ | N+7; N+8 |

Next ID becomes N+9. No original ID retires. Unchanged objects/fields stay exact;
only split incidences, required ownership lists and the listed new objects change.
One CutPiece is one revision/Undo entry. Existing Editor clone/validate/commit
provides atomic failure; Undo keeps the allocator high-water and Redo restores
these same IDs. A later failed command in a transaction rolls back the entire cut.

Refuse any legacy seam OR ordered group referencing ANY boundary of the target,
even an interval away from the chord. Do not let SplitCurve's normal relationship
remapping silently turn this into cut-and-sew. Unaffected other-piece relationships
and their interval/map/list order remain exact. Ad7 has no separate attachment
model; any future attachment type needs an explicit eligibility/remapping policy.

Retain each existing straight internal annotation unchanged in exactly one child,
filtering the original owner list in its original order. Reuse the current whole-
segment containment checks and existing `1e-6+1e-7+r` clearance. Reject the entire
cut if any annotation crosses, touches, lies too near the new boundary, or cannot
be assigned to exactly one child. Do not split, drop, weld or project annotations.

The status/inspection diff identifies retained/new pieces, split/new IDs and
annotation ownership changes; no extra persistent result/report schema is needed.
Existing inspect tools expose the authoritative lists. After native Apply select
the appended new child, clear curve selection and return to Select. Undo clears
that now-invalid selection; Redo follows the house policy and does not invent a
restored UI selection. Failed/cancelled cuts preserve the old selection/history.

## Complete native path

1. Start empty, create the original 240×360mm rectangle in metres, and save.
   Select its owner. Expose Cut piece in the rail/menu/search and a precise action;
   boundary picking remains separate from internal-line picking.
2. Refuse entry while inspector commands are pending or raw input is incomplete,
   using the existing retained-input/selection guard; preserve raw text/caret.
   The user explicitly applies/cancels those properties first. No new compound
   cross-piece draft framework or general Save-draft fix belongs here.
3. First boundary click captures owner, revision and curve/t without allocation.
   Second click opens a review/preview modal, not an immediate destructive commit.
   Use the house unsnapped boundary/t projection path, as for boundary intervals;
   position-grid snap does not alter these addresses. Refuse ambiguous/vertex
   hits, never extend a line to invent an anchor. The precise
   route has two typed boundary selectors and two dimensionless f64 t fields,
   plus read-only metre endpoints, cut length, child areas and retained/new labels.
   Never edit a u64 ID through an f64 numeric widget.
4. Build preview through the shared command on a clone. Accepted construction,
   rest mesh, cloth, allocator/history and dirty state stay separate. Validation
   failure retains raw numeric text/caret and useful reasons. Escape, explicit
   Cancel and backdrop Cancel abandon the draft without authoring. The modal
   blocks background authoring/Save. Apply/Return executes one shared command.
   Tool/owner/history/new/open/recover changes invalidate stale drafts; failed IO
   preserves still-valid state. Accepted actions alone cancel/rebuild workers.
   For the analytic rectangle workflow, enter t=3/8 and 5/8 in this same review
   modal after the two picks; do not infer submillimetre coordinates from pixels.
5. Apply gives two selectable pieces; inspect both, Undo once, Redo once and save
   actual v4 outputs. Quit the actual process, restart/reopen, select the new
   child by its displayed piece ID/name and change its height .36→.40m with one normal Tab
   commit. The old child's cut endpoints must remain exact: these are independent
   editable pieces. Area/mass conservation is assessed BEFORE that intentional edit.
6. Include invalid input, Cancel/backdrop, concave refusal/correction and save/open
   failure retention. Freeze the actual native actions/caps/grader BEFORE its run;
   source/egui/adapter tests are not native or reference evidence. Retain results,
   then close the finished window; separate any failed-case cleanup from evidence.

## Original acceptance, predeclared before implementation

R240 is independent analytic geometry only: `[0,0],[.24,0],[.24,.36],[0,.36]`.
At default allocator, rectangle owner2 has points3–6, curves7–10, revision1/next11.
Cut curve7 at3/8 and curve9 at5/8: A=(.09,0), B=(.09,.36), length .36m.

| Exact prospective result | Expectation |
| --- | --- |
| Retained2 / new15 boundaries | `[7,18,14,10]` / `[12,8,9,19]` |
| Anchor/copy point IDs | A11/B13 belong to2; A′16/B′17 belong to15 |
| Curve incidences | 7:3→11; 12:16→4; 9:5→17; 14:13→6; 18:11→13; 19:17→16 |
| Areas | .0324 / .0540; total .0864m² |
| Perimeter sum | Original1.2 + twice chord.36 =1.92m |
| Default density .18kg/m²: masses | .005832 / .009720; total .015552kg |
| History | CutR2/next20; UndoR3 one piece/next20; RedoR4 same cut IDs/next20 |
| Abandoned branch | Undo againR5, cut againR6 starts at20, new piece24, next29; never reuse11–19 |

Also freeze these small independent cases:

- Original L vertices `[0,0],[1,0],[1,.4],[.4,.4],[.4,1],[0,1]`, area .64m²,
  default boundary9–14. Valid curve10:t=.5 to curve14:t=.8 cuts y=.2: retained
  bottom area .20, new upper area .44, chord1m. Invalid curve11:t=1/3 to
  curve12:t=2/3 connects (.8,.4) to (.4,.8); midpoint(.6,.6) is outside despite
  only the two endpoint boundary hits. Refuse atomically.
- R240 annotations: (.02,.10)→(.06,.30) stays with retained child;
  (.12,.10)→(.21,.30) moves unchanged to the new child. With annotation curves13
  and16 already allocated, N17 gives new piece21/next26; annotation IDs/data stay
  exact. Crossing (.05,.18)→(.13,.18), on-cut x=.09, and near-cut x=.09+.5e-6
  annotations each refuse the entire cut without consuming IDs.
- Target legacy seam and target ordered-group refusals, including intervals remote
  from the chord. A separate pair of other pieces with reversed partial legacy
  sewing and an ordered group retains all authored relationships unchanged.
- Swapped anchors give identical output; clockwise input preserves winding;
  same-edge, vertex/near-vertex, collinear, cross-owner/internal/missing IDs,
  nonfinite/out-of-range t, degenerate child, overflow/object/work limits and late
  transaction failure refuse with old document/history/redo/primary intact.

Untouched f64 fields compare IEEE bits, including signed zero; RGB stays f32.
Copies of new anchors compare EXACT bits to their source anchor. Independently
interpolated new points/lengths use1e-12m and child/union areas use1e-12m² for the
ordinary R/L cases. Repeat with tilted placement `[.31,-.47,.21]`, translation
`[.2,1.4,-.3]`, nonzero grain: same local split, unchanged world union/landmarks
within1e-10m. A translated R240 near `[999,-999]` uses1e-9m/m² budgets. These
measurement budgets never substitute for strict interior/uncertainty refusal.

Check area by independent polygon arithmetic AND actual fresh UV triangles per
piece, disjoint interiors, the complete boundary walks and duplicated interface;
counts/bounds/area alone are insufficient. At resolutions .005/.07/.5m, eligibility
must agree. Independently sum `Cloth::masses()` by mesh vertex piece ownership,
using stable summation: ordinary absolute mass error ≤1e-12kg (large-offset case
1e-9kg), against density×ANALYTIC area, not the production mesh's own area answer.
No automatic cross-cut sewing/bending constraints or shared particles; ordinary
collision behavior remains. Measure initial mass before any simulation step.
Old-revision mesh must fail create_cloth; fresh mesh must succeed. Do not expect identical triangles
before/after a real cut or preserve old drape state by retagging it.

## Required implementation reach after separate approval

| Callers | Minimum reach / regression responsibility |
| --- | --- |
| Core document.rs / lib.rs | One enum/apply arm; guarded reuse of SplitCurve, allocation/ownership/annotation partition; same Editor atomic/history path. Wildcard exports suffice. Exercise later resize/transform/remove on each independent child. |
| Core geometry.rs | Small bounded straight-chord eligibility/assignment support; share existing predicates/budget pattern. Existing per-piece meshing must produce independent cut boundaries; no holes/arrangement engine. |
| Core sewing.rs / simulation.rs; sim crate review | Target relationship guard; unaffected authored sewing exact. Rebuild boundary samples/stitches/mass from new ownership; stale mesh rejection and existing masses() suffice. No new solver algorithm. |
| CLI main.rs / MCP mcp.rs + adapter tests | Add closed cut_piece schema, same execute/project_execute transaction, existing inspect ownership and lengths; advertise exactly one extra command variant, same tools. Strict bytes/schema/failed output/recovery tests, including same-path IO. |
| Native main.rs | Tool/rail/menu/search, boundary-only selection, revision-bound draft/precise preview and retained input, modal guards, one Apply, visible piece IDs/deterministic selection, reset/history/reopen/child edit. Actual native gate is separate. |
| Persistence.rs / SVG/DXF/OBJ paths + migration/export tests | Same strict nativev4, two ordinary pieces. SVG two closed paths, DXF two piece layers/both cut boundary copies; internal annotations still omitted. OBJ has two surface components, not annotation lines. Preserve IO/recovery guards and old v1/v2/v3 fixtures. |
| Core cut/history/geometry/migration + native/adapter tests | Freeze original recipes/IDs/oracle first; test actual authored fields, meshes, physical mass, failures and retained raw files. No replacement of a failed run by a later scenario. |
| Architecture/automation/formats/PAT006 + status/registry | Describe engineering subset/evidence/limits truthfully; no parity or full-feature promotion. Keep root README focused, historical artifacts unchanged, Actions/Pages disabled. |

Schema/command refusals never start replacement: primary/output/recovery bytes
stay exact. Failed IO must preserve the primary and accepted open construction;
late writer failure may advance recovery to the valid prior primary. Do not
promise atomic replacement of both files or fix general inspector Save semantics.

Before implementation: review this draft and freeze original fixtures/oracle and
native acceptance procedure. After implementation: run the four required local
Rust gates in its own target, then separately bound native acceptance. Before
publication: tracked artifacts and registry/map checks from a clean checkout.

Still required later: vertex/curved/multiple cuts, holes/open slits, target sewing
and attachment remapping, Cut & Sew, linked symmetry, darts/pleats/folds, persistent
cut relations where needed, and independently observed reference eligibility,
identity, selection and layout-offset behavior. Those remain product work.

Risks: dense/near-degenerate contours can receive conservative numeric/work
refusals; coincident independent cut edges exercise existing contact handling.
The original fixtures, fresh-cloth construction and mass/union checks must pass
before this subset is called implemented. A draft is neither delivery nor parity.
