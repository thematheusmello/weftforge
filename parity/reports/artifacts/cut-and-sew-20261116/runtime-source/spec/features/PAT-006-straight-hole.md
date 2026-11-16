# PAT-006 — First editable straight hole

**Engineering contract; native and reference acceptance are separate.**
Prepared before implementation from WeftForge's own source and public primary manuals.
Design baseline: clean metadata `982ad0340d3c848bb8a959dca7a934886f1d7ae9`; the scoped
crates/apps/manifests/lock have no diff from runtime
`19b1e88245dc627cfec8164f4924b335fe44853d`. The original design has retained source hashes.
The current cut candidate, accepted cut specification and native packets are
separate and must stay byte-exact. This proposal neither qualifies nor changes them.

## 1. Smallest integrated result

One editable, simple, straight closed hole in a simple straight pattern. The
piece remains one physical piece with one connected surface and two free boundary
loops. Annotations remain annotations. Native v5 preserves the hole, ordinary
corner/edge edits, history and physical area. CLI/MCP use the same commands.
The native first route creates a rectangular hole numerically; subsequent vertex
edits and split/join can make a general straight polygon. API creation accepts
3–4096 vertices, matching AddPolygon's existing bound.

Retain the full ROADMAP target: multiple/curved/nested holes, closed internal
shape conversion/fill-back, curved owners, hole sewing, slits, cut-through-hole
partition, Cut & Sew and advanced edits remain explicit later work. No generic
topology graph, new triangulator, dependency, solver algorithm or web frontend.

## 2. Document/version and identity decision

Choose **strict native v5**, with required `Piece.holes: Vec<Hole>` and ordinary
`Hole { id: Id, boundary: Vec<Id> }` objects nested under their owning piece.
This increment permits zero or one Hole per piece. Hole IDs join the existing
global nonzero/unique ID set, object count and `next_id` high-water invariant.
There is no new material, placement or sewing field on Hole: it inherits its piece.

Rejected alternatives: v4 annotations cannot own a closed chain because their
endpoints are exclusive per segment, and annotations cannot remove material.
A disconnected outer chain, negative-area second piece or display mask is not a
hole. `Vec<Vec<Id>>` saves one small struct but has no stable feature address:
joining the last curve with the first may retire the first curve. An index or
first-curve-as-hole-ID breaks removal/selection and eventual multiple-hole APIs.
A separate polygon/boolean framework would add unnecessary ownership machinery.

Each hole edge is an ordinary Curve with `controls: None`; consecutive edges
share their common Point ID and the final end equals the first start ID.
Curves have exactly one outer/hole/annotation owner. Points may be shared only by
the two adjacent edges of their own loop; never across loops, pieces or annotations.
Every construction curve/point is owned; no new orphan exception. IDs do not
change when moving corners, resizing, transforming or changing mesh resolution.

Keep old outer-loop winding and order exactly. AddHole accepts either input
winding, retains the first input vertex, and reverses only the remaining vertex
order when necessary so the new loop winds opposite its outer loop. Allocate
after normalization. Native v5 rejects a same-winding hole rather than silently
rewriting IDs/order on load. A valid edit must retain both nonzero signed areas
and opposite winding. Output triangles are CCW in material UV as today.

## 3. Shared commands and supported edits

Add exactly two nontransaction variants, using existing closed `type`/`data`
envelopes: `AddHole { piece: Id, vertices: Vec<[f64; 2]> }` (`add_hole`) and
`RemoveHole { hole: Id }` (`remove_hole`). No extra MCP tool or top-level CLI verb
is needed: existing execute/project_execute suffice. Required fields, unknown
fields, duplicate keys, numeric bounds and trailing input stay strict. The new
schema must advertise 3–4096 vertices, finite metre coordinates in ±1000 and
positive u64 IDs. Current command/schema totals are 30/29 including/excluding
Transaction; these two become 32/31, with the existing nine MCP tools unchanged.

AddHole requires an existing straight owner with no hole and consumes `1+2n`
IDs: Hole first, then input-order Points, then ordered Curves. It changes no old
object or list except the target's holes list. RemoveHole addresses its stable
Hole ID, fills the opening and deletes only that hole and its owned curves/points;
it preserves `next_id`. RemovePiece also removes its hole objects/geometry.

MovePoint works on outer or hole vertices with final whole-domain validation.
SplitCurve works within either closed boundary loop; its existing first-edge ID
survives, followed by the new second edge, with new Point then Curve IDs.
JoinCurve stays within the selected loop, needs more than three edges and uses
the existing shape-preserving straight-join predicate; selected ID survives,
next edge/common point retire, including a last→first join. Hole ID survives.
Corners cannot be joined away as a shape change. Hole SetCurveControls is refused;
setting owner controls while a hole exists is refused. Internal annotation
split/join rules stay unchanged. CutPiece explicitly refuses a holed target
before mutation; its existing hole-free contract remains unchanged.

ResizePiece scales outer, hole and annotation geometry around the outer bounds
minimum; bounds still mean the exterior. TransformPiece includes all owned
geometry and uses existing placement/grain compensation to preserve world
arrangement. SetPlacement moves the entire annular surface; material/grain apply
to its material region. Each gesture is one Editor transaction/revision/history
entry. Reuse snapshot undo/redo, 128-entry limit and abandoned-branch ID protection.
Failed commands, cancelled/identity previews and later failure in a transaction
preserve accepted geometry, allocator, history, selection and dirty state.

## 4. Eligibility, annotation domain and sewing

Validate at every trust boundary: direct typed commands/transactions, native
load/recovery/save and importer results. Use authored f64 straight segments, not
screen picks, rest-mesh vertices or tessellated expected answers, for hole
eligibility. While a hole exists, both loops have 3–4096 authored edges and its
straight-domain validator replaces resolution-dependent outer contour sampling;
hole-free pieces retain their existing validator. Require simple closed loops, distinct vertices,
nondegenerate edges, no backtracking/self-touch/crossing, wholly interior hole
edges, and a strictly positive remaining material region. Existing contour
area floor is `abs(area2) > 1e-10 m²`; apply it to each loop and the net area2.
Straight edges must exceed `max(1e-9 m, reserve)`. Compute area in coordinates
relative to a loop vertex to bound large-translation cancellation.

Freeze `S=max(1, abs(all target outer/hole/proposed/annotation endpoint
coordinates))`, `reserve=next_up(256*f64::EPSILON*S)` and hole-to-outer clearance
`next_up(1e-6 m + reserve)`. Require greater distance, not equality. Include
segment/segment tests and winding classification; testing only corners or a
centroid is insufficient for a concave owner. Reject uncertainty; do not snap,
clip, repair, move or silently discard geometry. Support reversed outer winding
and large translations by the same rules.

An annotation's whole segment must lie in `interior(outer) minus closed(hole)`.
Retain existing outer annotation clearance and conservatism; for the straight
hole use the same `next_up(1e-6 + 1e-7 + reserve)` clearance as the existing
annotation validator. Reject a hole surrounding an annotation, any crossing,
touch or near pass, and later edits that cause them. Existing valid annotations
retain exact IDs, endpoints, order and owner. Crossing annotations cannot be
reassigned, deleted or turned into hole edges.

Permit existing target **outer-edge** legacy seams/ordered sewing unchanged:
their curve IDs, intervals, correspondence, orientation and physical lengths do
not change when adding/removing an interior hole. No remap is needed. Continue
requiring outer boundaries for every sewing creation/update/load/length route;
hole curves are explicitly ineligible, even though they have mesh boundary
samples. Outer split/join retain their existing sewing remaps; hole split/join
do not touch unrelated sewing. Other pieces' relationships remain exact.

Keep ±1000 m, finite fields, 20,000 total objects, checked ID allocation, 16 MiB
native files, 1 MiB adapter input and 1–1024 command/nonnested transaction limits.
Count Hole as an object and preflight `1+2n` objects/ID headroom. Reuse the
1,000,000-step ContainmentWork budget pattern for pair/containment work, shared
across hole operations in one execute and across holes/annotations in final
validation; do not reset it per loop/pair. Keep existing 20,000 boundary-sample,
100,000 mesh-vertex, 200,000 triangle and 20 refinement-level limits; the sampled
outer-plus-hole boundary remains at most 20,000 vertices per piece at meshing.
These sampling limits must not leak into the raw straight-domain eligibility.
No cap is
raised to make a fixture pass. Authoring eligibility and mesh success are separate:
a valid document can report a bounded meshing failure and remain editable/savable.

## 5. IO, rendering, mesh and physics

Add strict ProjectV4/PieceV4 decoding; migrate v1–v4 with empty holes and no
renumbering or geometry/relationship changes. Preserve previous version-specific
strictness. All new writes are v5, even with no hole. Required v5 holes/Hole fields,
unknown/duplicate fields, malformed ownership, nonfinite/huge values, bad winding,
multiple holes and future versions refuse the whole load. Failed open/recover
leaves the old in-memory project. Failed validation or early refusal preserves the
native primary, export destination and existing recovery bytes. A late native-save
I/O failure leaves the primary byte-exact and may advance recovery to the exact
previous valid primary. Unsupported DXF export refuses before staging and leaves
destination and existing recovery unchanged. Retain the current staged
fsync/replace, nonregular/symlink/hardlink/native export guards, interrupted-save
and same-path execute behavior; no atomic replacement of both primary and recovery
is promised.
Do not serialize meshes, caches, drape state or undo stacks as authored holes.

Draw/select/edit both contours in native 2D with the existing PhotoCraft-directed
Rust/egui studio shell; hole edges are solid construction boundaries, distinct
from annotation styling. Identify Hole #id and its edge/vertex IDs in the
inspector. Never offer hole edges in sewing/cut boundary pickers. Piece selection
still owns its hole; Remove Hole and Delete Owner Piece must be separate actions.
Reuse current retained numeric text/caret, focus, modal shielding and epoch/revision
checks. Preview/cancel never publishes a candidate mesh or consumes accepted IDs.

earcutr **0.5.0 is already a workspace/core dependency and present locally**.
Its public source/API accepts flattened vertices plus hole-start vertex offsets;
current WeftForge passes `&[]`. For each piece flatten outer then hole, supply
the actual sampled outer vertex count as the hole offset, and preserve every
curve's t=0/t=1 associations. All vertices keep the same piece ID. Existing
BoundarySample fields suffice: unique curve ownership plus doc_revision resolves
the Hole. Recheck collinear recovery, quality flips and boundary-aware refinement
on the annulus; no bridge may become a material boundary, close the hole or gain
stitches/bending across empty space. A triangulator's success/deviation is not
by itself proof of domain/topology correctness.

Rebuild rest mesh, cloth and display/export state after committed edits; reject
old-revision jobs/meshes. Report failed current meshing plainly; never show a
stale filled surface as current or remove the hole to recover. Core/solver stay
GUI-independent. Existing triangle-area mass and two-face bending construction
should suffice once the actual annular mesh is correct; review rather than add a
new solver model. Hole edges are free boundaries, with no cap or automatic seam.

Native saves preserve the editable Hole. OBJ exports the actual current/rest
annular surface in metres, Y-up, with material-space metre UVs, consistent normals
and one piece group. SVG exports **one compound path per piece**, outer and hole
closed subpaths, physical mm and inverted SVG Y, with explicit evenodd semantics;
never export the hole as another filled pattern path. Annotations remain omitted.
Existing one-contour SVG import must explicitly reject this compound path until
hole-aware import is implemented; document the round-trip gap. **DXF v1 export
refuses any holed project before staging**, because its current one-contour
piece-layer profile/importer cannot preserve a hole. Existing destination stays
exact. Update GUI/CLI/MCP format descriptions and errors; do not quietly export
only the exterior or invent an unreviewed DXF profile. Hole-aware DXF/SVG import
and later profiles remain roadmap requirements.

## 6. Original fixtures and predeclared criteria

No expected result is obtained by running WeftForge. ORACLE-SUMMARY.json records
only direct rational/analytic arithmetic. Base R240: outer
`[(0,0),(.24,0),(.24,.36),(0,.36)]`; hole input
`[(.08,.12),(.08,.24),(.16,.24),(.16,.12)]`. Outer area .0864, hole .0096,
net .0768 m²; default density .18 gives **.013824 kg** (uncut .015552 kg).
Default placement maps `[u,v]→[u,v,0]`; 3D is Y-up, metres. One piece/material;
no sewn hole, extra piece, annotation, island, triangle cap or proprietary asset.

| Independent construction/history recipe | Exact expectation |
| --- | --- |
| Empty → AddRectangle R240 | R1; piece2, points3–6, curves7–10, next11, holes[] |
| AddHole above once | R2; Hole11, points12–15, curves16–19, next20; loop16:12→13,17:13→14,18:14→15,19:15→12; old objects exact |
| MovePoint12 to(.06,.12) | R3; same IDs/next20; hole .0108, net .0756 m²; mass .013608 kg |
| Undo / Redo that edit | R4/R5; original/edited coordinates and same Hole11; next20 |
| Separate R2 branch: SplitCurve16 at.5, then JoinCurve16 | New point20=(.08,.18), curve21; loop[16,21,17,18,19], next22; join restores four-edge geometry, retains Hole11/curve16, retires20/21, next22 |
| Separate R2 branch: RemoveHole11 / Undo | Filled outer only / exact Hole11 and edges restored; next20 throughout |
| Separate R2: Undo add, then AddHole again | Revisions3/4; no hole/next20, then Hole20, points21–24, curves25–28, next29; no reused11–19 |
| Separate R2: ResizePiece(.48,.72) | Every UV doubles, net .3072 m², mass .055296 kg; old IDs/placement retained |

World-transform case on a separate R2 copy: set placement translation(.4,1,.2),
zero rotation; then TransformPiece translation(.03,-.02), angle π/2, pivot(.12,.18).
Every outer/hole/annotation UV becomes `(.33-v,u+.04)` and grain 0 becomes π/2;
compensated world points remain `(u+.4,v+1,.2)` within 1e-12 m. Compare physical
points, not a particular Euler representation. IDs, areas and masses stay exact
within their declared tolerances; placement-only edits instead move the hole
and exterior together by the declared Placement transform.

Wrap-join fixture, authored independently: same R240 owner, hole vertices
`[(.12,.12),(.08,.12),(.08,.24),(.16,.24),(.16,.12)]`. Hole11, points12–16,
curves17–21, next22. JoinCurve21 consumes next/first17 and shared point12;
loop becomes[18,19,20,21], Hole11 remains addressable/removable, area .0096.
Also exercise clockwise outer, reversed hole input with first vertex retained,
concave owner, one unreferenced outer/hole point reuse, orphan geometry, shared
curve/point, duplicate hole ID and a hole below three edges. Invalid cases must
not consume IDs/history. A malformed native v5 must not load partially.

Annotation fixtures: preserve `(.02,.06)→(.22,.06)`; refuse a new hole around
`(.10,.16)→(.14,.20)`, across `(.06,.18)→(.18,.18)`, touching its edge, or a
parallel segment only .5e-6 m outside it. Correcting to a clear segment must work.
Test simple/ordered outer sewing unchanged through hole add/remove and hole edit;
refuse every hole sewing route and a cut on the holed owner. Test invalid point
move, invalid outer resize/control edit, failed split/join and failure after a
valid AddHole in one transaction; retain accepted state and any existing outputs.

At **.005/.07/.5 m**, base R240 and the edited R3 must produce actual meshes and
fresh `simulation::create_cloth` objects. Independent checks: finite attributes;
every used triangle has positive UV area, correct piece ownership and default
world embedding; one connected component, exactly two closed boundary components,
and `V−E+F=0`. No face interior intersects the open hole, leaves the exterior or
overlaps another face; do not substitute a centroid-only check. Boundary edges
trace exactly the authored loops and complete curve-t associations through
refinement; outer/hole perimeter sums are 1.2/.4 m for base R240. Area and summed
**actual Cloth::masses()** must agree with the analytic oracle within 1e-10 m² and
1e-10 kg respectively, with no reference to production-computed area as oracle.
World/UV/boundary point tolerance is 1e-12 m. Old pre-hole mesh must refuse
create_cloth; fresh annular mesh succeeds. Init/reset retain finite rest state
and analytic mass. No drape/contact-quality or reference-physics acceptance is
claimed by these constructor checks; do not demand a fixed triangulation/count.

Resource refusal case: straight 10×10 m owner, interior 2×2 m hole at(4,4)–(6,6),
net 96 m², resolution .005 m. Geometry is eligible; mesh success is **not expected**.
Any completed refinement satisfying the current maximum-edge rule(.007 m) would
need over four million triangles by the equilateral-area bound, far above the
200,000 cap. Require an explicit bounded meshing/resource error with the authored
hole/project intact, never a successful over-cap mesh/cloth. Exercise aggregate
object/ID/byte/work refusal separately; keep unsupported cases visible.

## 7. Concrete future native path; not a frozen UI packet

Launch a fresh actual native app empty; author R240 through its existing precise
rectangle flow. Select the piece; `Add hole…` opens the reused numeric rectangle
pattern with origin(.08,.12), width.08,height.12 and live candidate outline/net
area. Cancel/Escape/backdrop returns to R1/next11. Reopen; invalid `1e-` and an
outside origin(.20,.12) refuse Apply while preserving raw text/caret and R1.
The modal blocks Save, background authoring and shortcuts; its opening Return
only opens review. Correct the numbers; one later Apply creates R2/Hole11 once.
Select a hole edge/vertex; numeric corner edit moves point12 to(.06,.12) in one
gesture, then Undo/Redo yield R4/R5 above. Save; quit and verify the owned process
ends; reopen the saved v5 in a fresh process, inspect/edit the same Hole11 and
export the R5 annular OBJ/SVG. DXF refusal preserves a preexisting sentinel output.
Retain files, IO results and actual mesh/cloth checks independently. Close test
windows after retaining outcomes; record cleanup separately. No operation count,
frozen UI packet or runtime acceptance is created by this prose.

## 8. Implementation reach and validation

In this table, core/ means crates/weftforge-core/src/, sim/ means
crates/weftforge-sim/src/, cli/ means apps/weftforge-cli/src/, and native/ means
apps/weftforge/src/. Test names refer to their respective crate/app tests folders.

| Existing surface | Required bounded reach |
| --- | --- |
| core/document.rs | Hole/Piece serde; object/ID/ownership validation; AddHole/RemoveHole; loop-aware MovePoint/SplitCurve/JoinCurve, resize/transform/remove; explicit cut/control guards; Editor atomic/history invariants |
| core/geometry.rs | Authored straight domain/clearance/work checks, annotation exclusion; outer+hole sampling/earcut offsets, collinear recovery, quality/refinement and boundary association; no new mesh type |
| core/sewing.rs; simulation.rs; sim/lib.rs | Audit outer-only require_boundary_curve callers (all intervals/length routes), unaffected remaps; current-revision adapter, actual area masses and free-boundary bending; no new solver algorithm |
| core/persistence.rs; persistence/svg.rs,dxf.rs,dxf_import.rs | Strict v4 migration/v5 write; atomic/recovery guards; SVG compound export/current import refusal; DXF export refusal/current import scope; existing OBJ surface writer |
| core/lib.rs and every Piece literal | Existing wildcard document export exposes Hole; initialize holes in polygon/import/ellipse/sample/test/migration constructors; preserve imports without holes |
| cli/main.rs | execute uses shared commands; inspect patterns includes Hole IDs/loops (project_inspect already returns full Project); help/export descriptions, same-path/failed IO and mesh-error semantics |
| cli/mcp.rs | Two closed command schemas and limits; existing nine tools, export descriptions; update all variant-count assertions to32/31 as appropriate |
| native/main.rs | Rectangle/cut-style retained draft; menu/search/inspector action, candidate outline, hole-aware selected-curve/point ownership, numeric edits, split/join/remove, patterns rendering/picking; restrict free_sewing_pick/cut_piece_pick; resize/transform selection/rebuild/job/history/load/save/export/autosave/exit lifecycles; garment consumes current annular triangles |
| core tests | New focused holes fixture/tests; workflow/internal_lines/joins/pattern_transform/migration/DXF/SVG/mesh/simulation regressions; do not rewrite original cut specification/fixtures/expected/native packets |
| cli tests + native tests | mcp_stdio/curves/export strict schema/IO cases, round-trip identity, modal raw text/caret/selection and one-gesture history; later actual native restart path above |
| docs/status/map/registry/xtask | On approved implementation, document v5 and export limits, exact source/evidence freshness and next handles; keep existing failures/stale/unknown coverage; update implementation counts/maps without parity credit; run ordinary registry/map verification only at the authorized gate |

Before a future integrated implementation is accepted, run locally:
`cargo fmt --all --check`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo xtask verify`.
Use a separate target per implementation worktree. Publish evidence only after
artifacts are Git-tracked and registry/maps pass from a clean checkout. Keep
README product-focused, Actions disabled and the static Pages site unchanged.
Implementation and verification status belong in STATUS.md and the feature registry.

## 9. Public documented intent and remaining runtime unknown

The public [Convert to Hole/Internal Shape manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358440603161-Convert-to-Hole-Internal-Shape)
documents converting an internal shape into a hole and converting it back through
the shape/hole context menu. The [Internal Polygon/Line manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358276422297-Internal-Polygon-Line)
documents creating an internal polygon inside a pattern and closing it at its
start point. The [Cut/Cut & Sew manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358433146393-Cut-Cut-Sew)
describes dividing patterns and optionally adding sewing. These documented routes
and purposes were read 2026-11-16; they establish no build-specific eligibility,
IDs, history, mesh, timing, import/export or native interaction behavior.

**One next reference-runtime uncertainty:** after authoring the original R240
rectangle and internal rectangular shape, does one Convert to Hole action leave
one editable owner pattern whose free inner boundary survives save/quit/reopen?
Observe only this bounded lifecycle once foreground access works, with declared
baseline/settings and original fixtures. Current reference access needs user
activation; no reference app, REA session, binary, installer or account was touched.
Until then this lifecycle is unobserved, and conversion-back, curved/sewn/multiple
cases and detailed native semantics remain unknown. Engineering choices above
are prospective WeftForge policy, not inferred reference facts.

V5 is a deliberate format change. SVG hole reimport and DXF holes remain explicit
gaps; implementing this contract does not establish reference parity.
