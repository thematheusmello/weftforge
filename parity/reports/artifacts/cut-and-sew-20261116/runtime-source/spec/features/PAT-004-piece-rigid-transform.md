# One-piece 2D translation and rotation contract

Reviewed engineering contract, approved before implementation and evaluation,2026-11-15.
Root approves the explicit arrangement/grain policy and fixed tolerances below.
Reference behavior remains unobserved; no parity claim is made.
Source reviewed read-only: `be8a031b5f8342d0857f3bed34f52dd165fa86da`
in the authored-length-feedback worktree. No Cargo, native/reference runtime,
registry change, PR, push or Actions run was performed for this proposal.

The next integrated increment is one selected piece, translation and rotation,
an explicit pivot, native numeric and drag routes, and one shared typed command.
It is an implementation candidate for `PACK-pattern-transform-tool` / PAT-004 /
UI-004; it does not establish reference parity or complete PAT-004.

## What the current source actually does

| Concern | Source and present behavior |
|---|---|
| Authored geometry | `crates/weftforge-core/src/document.rs:10`: `Point.position` is f64 `[u,v]`; `Curve.controls` contains two **absolute** f64 cubic controls. Endpoint IDs are references, not duplicate coordinates. `Curve::evaluate` uses straight/Bernstein evaluation. |
| Ownership and bounds | `document.rs:319` / `Project::piece_bounds` collects analytic cubic bounds. `Project::validate` requires closed ordered boundaries, forbids shared boundary curves/points between pieces and orphan construction, and validates every endpoint/control within ±1000 m. |
| Existing resize | `document.rs:891`: `ResizePiece` scales the owned endpoints and controls about the analytic bounds minimum, using a `BTreeSet` so each endpoint is changed once. It leaves placement, grain, IDs and sewing records unchanged. Do not alter these established semantics in this increment. |
| 3D arrangement | `document.rs:101`: `Placement::transform` embeds `[u,v]` as `[u,v,0]`, applies XYZ Euler rotations in order (`Rz Ry Rx`), then translation. `SetPlacement` at line 934 only replaces that authored placement. |
| Rest mesh | `crates/weftforge-core/src/geometry.rs:202`: `mesh_cancellable` sends each authored sample directly to `piece.placement.transform(point)` and copies the same point into `mesh.uv`. There is **no per-piece centering, layout origin or separate local frame**. Triangulation/refinement use these physical-scale UVs. |
| Grain/solver | `Piece.grain_angle` is counterclockwise from material +U toward +V. `crates/weftforge-core/src/simulation.rs:27` passes mesh UVs and that grain to the solver. `crates/weftforge-sim/src/constraints.rs:12` and `strain.rs:3` derive warp/weft from UV differences and grain. Moving UVs preserves differences; rotating UVs with an unchanged grain changes the physical warp direction. |
| Sewing | Legacy `Seam` holds oriented curve-t intervals and normalized parameter maps; `SewingGroup` holds ordered intervals and cumulative arc-length maps. The records reference curve IDs, never mesh vertex IDs. `sewing.rs` supplies bounded authored metrics with `SEWING_METRIC_TOLERANCE = 1e-8 m`. |
| Transaction/history | `document.rs:1439` / `Editor::execute` clones, applies and validates before replacing the project or history. A transaction is one undo entry. Affected sewing metrics are revalidated. Undo/redo create fresh revisions and preserve the ID high-water mark. |
| Native edit paths | `apps/weftforge/src/main.rs`: `InspectorDraft::commands` (255), `apply_command` (529), `apply_inspector_action` (563), `reset_gestures` (585), `patterns` (2263), `numeric_value`/`numeric_commit` (3386/3399). Current Select drags one endpoint; its preview edits a clone and release dispatches `MovePoint`. Whole-piece dragging/rotation is absent. |
| Adapters | CLI `execute` deserializes original JSON bytes into shared commands; `apps/weftforge-cli/src/main.rs:365` wraps the array in one `Transaction` and saves only after success. MCP uses that function and manually lists variants in `apps/weftforge-cli/src/mcp.rs:528` / `command_schema`. Strict unknown/duplicate-field handling already exists. |

**Resolved source question:** moving every authored endpoint and control with
placement untouched also moves the arranged 3D rest garment. Rotating them also
rotates its arranged geometry in the piece's current plane. A naive new command
would therefore be a physical arrangement edit, not just a 2D layout edit.

## Proposed fixed policy

This is a deliberate **WeftForge layout policy**, approved for implementation; reference
behavior is unknown. Keep the arranged 3D authored surface and physical warp/weft
directions fixed while changing the 2D layout. Document the placement compensation
and grain update as part of this command. Do not add a policy flag or silently
modify `MovePoint`, `ResizePiece` or `SetPlacement`.

1. For each owned endpoint and each absolute cubic control, apply
   `q' = p + R2(angle) * (q - p) + translation`. The pivot `p` is in the original
   authored U/V coordinates. Positive angle is counterclockwise in the V-up canvas;
   rotation occurs first, then translation. Use the same result once per original
   coordinate, never repeated per-frame or per-curve endpoint transforms.
2. Preserve piece/point/curve/material/seam/group IDs, object ordering, boundary
   ordering, start/end links, straight-vs-cubic representation, names, materials,
   interval t endpoints/directions, correspondence maps, mannequin, resolution and
   `next_id`. Transform every cubic control with its endpoints; do not refit curves,
   reverse boundaries, split/join, or remap sewing data.
3. UVs continue to equal authored pattern coordinates after remeshing, in metres.
   They therefore rotate/translate with the piece. This is a material coordinate
   change, not an atlas or a new persistent layout frame. SVG/DXF show the changed
   construction; OBJ UVs change, while its arranged surface is preserved within
   numeric/meshing limits. No texture-placement promise is made by this increment.
4. Rotate the material warp vector by the same `R2(angle)`. For a nonzero angle,
   store the resulting direction as canonical `atan2(v,u)` in `[-pi,pi]`, computed
   from `R2(angle) * [cos(old_grain), sin(old_grain)]`. This avoids overflow or lost
   increments from adding to the currently permitted arbitrary finite grain angle.
   A pure translation retains the grain field bits. Material properties do not change.
5. Compensate the **existing** 3D placement, without adding document fields. If the
   old placement is `T(q) = A*[q,0] + t`, use `A' = A*Rz(-angle)` and
   `t' = T(p) - A'*[p + translation,0]`. Then `T'(q') = T(q)` for every point in
   the authored plane, and the rotated UV grain has its original world direction.
   Euler values may change even though arrangement is fixed. This visible change
   must be described in the feature contract/native help.
6. A successful command still invalidates all derived revisions. Keep rest mesh,
   display snapshot and simulation separate. Do not reuse a mesh/cloth because the
   continuous 3D surface is unchanged. A rebuild can choose different triangulation
   under a rigid UV change; identical vertex indices, topology, or solver trajectories
   are not promised. Authored geometry and sewing references are the invariants.

Do not implement compensation as `placement.rotation[2] -= angle`: that fails for
pieces already tilted about X/Y. The smallest sufficient math is a private helper
beside `Placement`: obtain its existing rotation basis with zero-translation
`Placement::transform`, right-compose the two in-plane columns, and extract one
XYZ Euler representation. With matrix `M = A'`, use
`y = atan2(-M20, hypot(M00,M10))`, `x = atan2(M21,M22)`,
`z = atan2(M10,M00)` away from gimbal lock. At the singular branch choose `x=0`
and `z=atan2(-M01,M11)`; a small fixed branch threshold (32 f64 epsilons) must be
declared before tests. Compare transformed geometry/orientation, not Euler field
equality. A pure translation bypasses Euler decomposition and directly uses
`t' = t - A*[translation,0]`. No matrix/quaternion dependency, general transform
framework, persistent origin or native-file version change is needed.

## Smallest shared API and validation

Add one `Command` variant in `document.rs`; existing `lib.rs` wildcard export
already exposes it:

```rust
TransformPiece {
    piece: Id,
    translation: [f64; 2], // metres in authored U/V
    angle: f64,           // radians, positive counterclockwise
    pivot: [f64; 2],      // original authored U/V, explicit even for angle=0
}
```

JSON tag: `transform_piece`. Require all four data fields; reject extra/duplicate
fields. In the MCP schema, use the existing fixed-length two-number array helper,
nonzero piece ID, and `additionalProperties:false`. Reuse CLI `execute` and MCP
`project_execute`; no separate CLI verb or MCP tool is necessary.

Before mutation, validate known piece, finite translation/pivot with the existing
±1000 m coordinate policy, and finite angle with `abs(angle) <= 1e6` rad, consistent
with the current supported placement-rotation range. Validate all resulting
endpoints, controls, grain and compensated placement, the contour, and affected
sewing metrics through the existing clone/validate/commit boundary. A valid input
can still produce unsupported coordinates or placement and must fail atomically;
do not clamp or move other pieces to make it fit. Reject nonfinite intermediates,
overflowing JSON numbers, or precision/metric failures with a useful error.

For `angle == 0`, author coordinates use exactly one componentwise addition and
must not pass through pivot subtraction/trigonometry. A zero component retains its
original bits. For an identity request, the native UI sends no command; core
`Editor::execute` keeps its existing successful-no-op history behavior.

Any invalid or overflowing request must preserve project bytes/fields, revision,
both history branches/ID allocator, every staged native draft and focused text,
accepted mesh/cloth/job state, and CLI/MCP destination/recovery bytes. A bad command
late in a command array must preserve these too. Do not first commit inspector
edits and then try a transform. `Studio::execute` calls `commit_draft` first, and
`commit_draft` takes its draft before validation, so that is not a safe new action
route. Reuse `apply_inspector_action`'s combined transaction and retained-on-failure
behavior; clear drafts/text caches only on success.

## Native scope

Keep the merged studio shell. Add one accessible `Transform pattern` tool to the
existing rail/search routes; Select continues to edit individual points. Clicking
a boundary in Transform mode selects its piece. One selected piece is enough.

- Numeric route: a small `Transform pattern…` modal using the existing authoring
  modal/focus pattern. Show translation U/V in metres, angle in radians, and pivot
  U/V in metres. Default pivot is the analytic bounds center, explicitly displayed;
  the user can type another pivot. Apply/Return dispatches the one command, combined
  with pending inspector commands as one transaction. Cancel/Escape changes no
  construction and retains unrelated drafts. Invalid input keeps the modal values,
  focus/text, error and history for correction. Block background pointer, shortcuts
  and AccessKit authoring actions while the modal is open, as the rectangle modal does.
- Drag route: show the chosen bounds-center pivot and one rotation handle in this
  tool. Boundary drag translates the whole piece; handle drag computes a signed
  angle with `atan2(cross,dot)` around the displayed pivot. The first increment's
  drag pivot is the displayed bounds center; custom pivot entry is the numeric route.
  Capture piece ID, source revision, original geometry, pivot and snap/Alt state at
  gesture start. Translation snaps the displacement with `PositionSnap::apply`;
  do not snap individual vertices/controls. Rotation has no angle snap/axis lock.
  Never accumulate transforms from the last preview. Preview is disposable and
  does not create history or replace accepted 3D state. Release commits once through
  the atomic action route. Escape or tool/document/revision change cancels a preview;
  a failed release retains a correctable attempted transform/error without a commit.
- Give the handle and modal meaningful accessible names. Use existing
  `to_screen`/`to_pattern`, `numeric_value`, focus/cache guards and gesture reset
  conventions. Pointer f32 coordinates must never become the authoritative source
  geometry. Menus, search, tool changes and history must not replay stale numeric
  buffers or leave a transform active on a later revision.

These handle/pivot/numeric decisions are proposed product behavior, not observations
of the reference. Do not add a reference keyboard shortcut solely from its presence
metadata. Mirroring, scaling, multiselection, pivot dragging, alignment, smart snap,
angle/axis locks and multi-turn gestures remain later work.

## Tolerances and meaningful checks, fixed before execution

Use one small analytic fixture, not a production/reference garment: a closed
asymmetric piece whose first cubic has endpoints `(0,0)`, `(0.75,0.75)` and controls
`(0.25,0)`, `(0.5,0.25)`, then straight edges through `(-0.25,1.25)` and
`(-0.5,0.25)` back to the start. Its first boundary is exactly
`q(t)=(0.75t,0.75t²)` in real arithmetic, with analytic interval length from
`F(t)=(3/8)t*sqrt(1+4t²)+(3/16)asinh(2t)`.

Add a second separately placed piece, a legacy seam with reversed/partial
intervals and nonidentity map, and an ordered sewing group with multiple spans,
a gap, reversal and nonidentity map. Choose nonoverlapping intervals within each
group. Use nonzero grain `0.37` and unequal warp/weft stiffness. Give the selected
piece a noncommuting 3D rotation `[0.31,-0.47,0.21]` and nonzero translation; include
identity and exact/near-gimbal placements as targeted table rows.

| Check | Predeclared acceptance |
|---|---|
| Translation | On the dyadic fixture, delta `[0.5,-0.25]` gives exact `to_bits` equality to the expected single additions for endpoints and controls; unchanged axes/other objects retain bits. No promise of exact arithmetic for arbitrary decimal inputs. |
| Rotation/order/pivot | With pivot `[0.25,0.5]`, translation `[0.5,-0.25]`, angle `pi/2`, the independent expected mapping is `(u,v) -> (1.25-v,u)` and the cubic is `(1.25-0.75t²,0.75t)`. Check all controls/endpoints and independent polynomial samples including `t=0,1/8,1/4,1/2,3/4,7/8,1`, within `1e-10 m`; verify the pivot maps to `p+translation`. Include one non-right angle and its inverse to catch a quarter-turn-only implementation. |
| Arranged surface | At the same independently chosen samples, old placement of the old point equals new placement of its transformed point within `1e-10 m`. Check a noncommuting placement, ±pi/2 gimbal cases and near-gimbal cases. At the ±1000 m supported scale use a separately declared `1e-9 m` absolute bound. These bounds allow f64 composition/argument reduction and remain below the existing 1e-8 m sewing budget; do not widen them after results. |
| Material/grain | Expected new direction is the independently rotated old unit warp vector, within `1e-12` per component. Its arranged world direction is unchanged within `1e-12`. Check translation retains grain bits, nonzero rotation canonicalizes it, and an existing very large finite grain does not overflow. Material assignments and all material fields are unchanged. |
| Curve/sewing relationships | IDs, ordered links, interval directions/t endpoints and every authored correspondence pair are exact equality. Analytic curve length error is at most `1e-8 m`; before/after measured seam/group length differences are at most `2e-8 m` (two bounded metric evaluations). Check lengths on the reversed partial parabola using `abs(F(t1)-F(t0))`. |
| Mesh/solver boundary | Build a fresh mesh, verify each new UV's compensated world embedding and current revision, valid piece/curve associations and unchanged sewing IDs. Do not require identical mesh indices/counts. Reject old rest mesh in `create_cloth`; accept the fresh one. No long drape or reference parity inference. |
| History/persistence | Exactly one success revision/undo restores original geometry, placement, grain, links/maps; redo restores the result under a new revision. `next_id` is unchanged. Save/reopen preserves all authored f64 fields and v3 format. |
| Failure | Unknown ID; NaN/±infinity in each input through typed tests; out-of-bound translation/pivot; angle beyond supported range; overflow numeric token; result/control/compensated-placement beyond bounds; stale gesture; and invalid command late in a transaction. Retain exact project/history/redo/drafts/text/caret and destination/recovery bytes. Include a control-only coordinate overflow so endpoints alone cannot validate the transform. |
| Native routes | Event/AccessKit numeric Apply/Return and pointer drag both reach the typed command, with one undo and no mid-gesture revision. At a fixed `1000 px/m` canvas and at least `100 px` rotation-handle radius, pointer expectations allow `2e-6 m` / `2e-6 rad` for screen f32 quantization; numeric routes use the core bounds above. Capture snap/Alt once; test cancel, invalid release, modal background blocking, blur/Tab after success/failure, pending inspector edits and fresh revision/cache invalidation. |

The boundary coordinates are deliberately asymmetric and the pivot is off-center;
a bounds-only check, rectangle, endpoint-only movement, constant-grain rotation or
`rotation[2] -= angle` cannot satisfy these checks.

## Exact implementation reach after approval

| File | Required work or regression responsibility |
|---|---|
| `crates/weftforge-core/src/document.rs` | New command/apply arm, small private placement compensation helper, existing validation/transaction boundaries. Reuse ResizePiece's owned endpoint/control traversal. |
| `crates/weftforge-core/src/lib.rs` | Existing wildcard command export already suffices; confirm no explicit export addition is needed. |
| `crates/weftforge-core/src/geometry.rs`, `sewing.rs`, `simulation.rs` | Trace/verify existing UV, sampling, sewing and revision consumers; no planned algorithm changes. Existing `ResizePiece`/`SetPlacement` callers and tests remain valid. |
| `crates/weftforge-core/tests/pattern_transform.rs` (new) | Analytic fixture, translation/rotation, compensation, grain, sewn relationships, atomic failures, history, v3 reopen and stale mesh checks above. Generate original fixtures in the test; no external asset. |
| `apps/weftforge/src/main.rs` | Tool enum/label/icon, rail and command search, contextual numeric action/modal and modal input guard, preview/captured gesture state, atomic action route, reset/load/history invalidation. Add focused native event/AccessKit tests alongside existing rectangle/ellipse/focus/length tests in this file. |
| `apps/weftforge-cli/src/main.rs` | Existing strict `execute_commands` adapter is sufficient; verify it with the new shared variant. Do not create a duplicate transform implementation or CLI verb. |
| `apps/weftforge-cli/src/mcp.rs` | Required `transform_piece` schema variant and concise field/semantic description; retain strict original-byte parsing and existing tool count. |
| `apps/weftforge-cli/tests/mcp_stdio.rs` | Discover exact required fields/array lengths/ranges, run the same command via MCP and CLI `execute`, inspect/save/reopen results and revisions, reject malformed/duplicate/unknown fields and late transaction failures while preserving existing output/recovery. Existing temporary-file/server helpers suffice. |
| `docs/automation.md`, `docs/architecture.md`, `docs/file-formats.md` | Add command JSON/units and fixed layout/placement/grain semantics; state v3 unchanged and UV/export effects. |
| `spec/features/PAT-004-piece-rigid-transform.md` (new), `spec/inventory/weftforge-implementation-map.json`, `.md`, `parity/features.yaml`, `STATUS.md` | Bound this increment and its unknowns, map exact new implementation/evidence, record source/check/capture handles and next steps. Never promote control-presence or implementation checks to reference parity. Update only when implementation/evidence actually exists. |

Run the required integrated local gates after implementation:
`cargo fmt --all --check`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo xtask verify`.
Use the worktree's own Cargo target. Then make one bounded native capture on the
identified candidate, including both transform routes, failure/cancel, undo/redo,
3D arrangement/grain and save/reopen. Publish only tracked artifacts and rerun
registry/map verification from a clean checkout. Keep Actions disabled and the
existing static Pages site untouched.

## Reference evidence and one next observation

Supplied evidence establishes only control presence: packet
`PACK-pattern-transform-tool`; menu `MDUI-BBF4CAF53EBE`,
`2D Pattern > Edit > Transform Pattern`; toolbar `MDUI-BAF98EAD70C1`.
The local inventory marks behavior `not_executed`. Pivot rules, handles, locks,
numeric entry, grain transformation and 3D effects remain **unobserved**. This
proposal does not fill those unknowns with assumptions.

The highest-value next runtime observation is one disposable asymmetric pattern
with a visible grain line, deliberately translated and tilted in 3D, simulation
stopped. Record its authored layout/landmarks and arranged 3D landmarks/grain,
invoke the observed Transform Pattern control, perform one visible 2D rotation,
then record the same landmarks/grain and undo. Use source-independent measurements
or application numeric/export data where available; a fixed-camera screenshot alone
can establish visible movement but not a submillimetre invariant. Log the actual
handle/pivot used and any chooser/setting encountered, without claiming that other
pivots/locks/numeric routes exist. This single before/after/undo trial resolves the
main semantic gap—whether a 2D rotation preserves the arranged 3D surface and carries
the grain with the pattern—and can expose a different reference policy before
implementation. It does not establish full transform parity.

Skipped here: implementation, runtime/reference observation, Cargo/native checks,
mirroring, scaling and multiselection. Main risk: the proposed 3D/grain policy is
explicit engineering judgment until that controlled reference observation exists.
