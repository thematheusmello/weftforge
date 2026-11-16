# Straight-cut contract review 001

Private engineering review, 2026-11-15. **Proceed after the transport example is corrected and the picker/acceptance details below are frozen.** The straight-cut subset needs no holes engine, new solver, persistent relation, new format, CLI verb, or MCP tool. This review is neither implementation evidence nor commercial-reference evidence; PAT-006 remains open.

Reviewed contract: `CONTRACT.md`, SHA-256 `f77cb6d07f2b7d3fe0d67cf60e067f28f483d6b714bf6e069e1d469ba4dd6e11`.
Read-only source: `.local/internal-segments-20261115`, clean HEAD `db8ee77f708a5088c3597030bd16de5d8912ca43`, runtime ancestor `ad7cc8bbbfc7a1ae735a400c21dcc7a37c80a5c5`. Runtime source paths listed in `SOURCE-SHA256.txt` are identical at those commits; AGENTS.md is metadata and differs. Repository AGENTS.md and the Ponytail skill were read.

## Corrections before freezing

1. **The command JSON example has the wrong envelope.** `document.rs:1542–1547` uses `tag = "type", content = "data"`; `mcp.rs:730` advertises the same envelope. Use exactly:

   ```json
   {"type":"cut_piece","data":{"piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}}
   ```

   Keep the proposed closed, required, nonzero-ID payload and original-byte duplicate checks. Do not introduce a second flat command representation or change older variants to match this typo. Existing CLI `execute` and MCP `project_execute` already deserialize shared commands directly and execute one transaction (`main.rs:365–380`, `mcp.rs:250–254,436–444`).

2. **Direct reuse of the native picker would violate cut picking policy.** `free_sewing_pick` (`native main.rs:5484–5548`) first returns any endpoint inside the picking radius, then breaks equal-distance ties by selected curve/authored order; its tests intentionally preserve these behaviors (`12618+`). It does provide exact straight projection with one segment. Reuse that projection math for the selected owner's straight boundaries, with cut-local endpoint and ambiguous-hit refusal. Do not change sewing behavior. Restrict candidates to the captured owner before resolving hits; coincident other pieces must not silently supply an address. Freeze how a tie/ambiguous hit is refused in the native procedure. The precise typed selectors remain the deterministic correction route.

3. **Freeze executable native values and uncertainty rules.** Enter the decimal strings `0.375` and `0.625` in the review modal; the house input parses f64 and does not evaluate fraction expressions. Keep `3/8` and `5/8` as the analytic description. State that `max_abs_coordinate` covers the target authored contour/annotation coordinates, comparisons are strict, and ambiguous signs at the rounding boundary refuse. Prefer the existing upward-rounded reserve/clearance convention (`geometry.rs:486–487`); it differs from the displayed formula only at representational boundaries and does not require a new tolerance option. No threshold loosening is justified by this review.

## Geometry, identity, and arithmetic

The inside certificate is sound for a certified simple polygon: removing the two endpoints leaves a connected open chord; if it has no other boundary intersection, its inside/outside classification cannot change. A strictly inside midpoint therefore certifies the entire open chord. The support-line margin on the addressed edges excludes endpoint collinearity; segment distance must cover the **whole chord**, not just its endpoints. Authored-edge simplicity must run even with zero annotations: existing `validate_contour` samples by resolution and skips adjacent pairs, while the stronger internal helper returns immediately without annotations (`geometry.rs:461–468,648+`). A bounded straight-edge pass is necessary; an arrangement engine is not.

Canonical original indices give the stated retained wrapping arc. A head and B tail stay with the retained piece; A tail and B head move to the new piece, with only their cut incidences retargeted to the copies. This keeps the original first curve and vertex on the retained piece, including when A is index zero. The two directed closing edges preserve either parent winding. Four new points, four new curves, and one piece are exactly nine objects; preflight `next_id.checked_add(9)` and object capacity before calling the two shared splits.

Independent arithmetic, using the stated decimal dimensions as exact rational values:

| Case | Independently checked result |
| --- | --- |
| R240 | A=(9/100,0), B=(9/100,9/25), length 9/25; retained area 81/2500=.0324; new area 27/500=.054; union 54/625=.0864. |
| R240 perimeter/mass | Original perimeter 6/5; child sum 48/25=1.92. With density 9/50, masses .005832 and .009720; total .015552. |
| R240 IDs | N=11 gives split anchors 11/13, tails 12/14, child 15, copies 16/17, chords 18/19, next20. The proposed walks `[7,18,14,10]` / `[12,8,9,19]` and every listed incidence are correct. |
| History | R1 rectangle → R2 cut → R3 Undo → R4 Redo → R5 Undo → R6 new branch. Existing high-water policy gives next20 after Undo/Redo, then new child24/next29 on the branch. |
| L valid | Area 1−(.6×.6)=.64. Curve10:t=.5 gives (1,.2); curve14:t=.8 gives (0,.2). The bottom retained area is .20, upper area .44, chord1. |
| L invalid | Curve11:t=1/3 gives (.8,.4); curve12:t=2/3 gives (.4,.8). Midpoint (.6,.6) lies in the missing upper-right square. This rejects through midpoint containment although no third edge is hit. |
| Two annotations | After curve IDs13 and16, N17 gives new child21, copied anchors22/23, chords24/25, next26. Left annotation13 stays; right annotation16 moves without changing points, curve, or order. Crossing/on-cut/0.5µm-near-cut cases correctly fail clearance. |

Untouched fields/copies should use bit comparisons. New interpolation and translated meshes should use the declared measurement budgets. Measure polygon area with translated-coordinate/difference arithmetic, and mesh area from UV triangle differences; naive large-offset shoelace products can cancel. Actual fresh triangle ownership, complete walks, disjoint interiors, and the duplicated interface are necessary in addition to area totals. No runtime geometry or solver result was produced in this review.

## Smallest complete implementation reach

- **Core/history:** `SplitCurve` (`document.rs:882–966`) already allocates point then tail and mutates private candidate construction. Guard all target seam/group references **before either split**; unaffected seams return unchanged from `split_seam`, and unaffected group intervals return their original value (`1304+,1458+`). Use the private apply path, not two Editor executions. `Editor::execute` clones before mutation and commits only after validation; Undo/Redo already preserve ID high-water (`1690–1819`). Pass one cut predicate budget from that outer execute through private apply recursion/Transaction and both splits; do not reset it per cut. Existing final-validation containment budget remains an additional guard. No public budget/report API is needed.
- **Annotations:** use the existing whole-segment clearance/containment logic and its winding test; partition the original owner list in order, accepting exactly one child. Do not classify solely by an annotation midpoint. Final normal validation catches orphan/shared ownership and the area floor. Reuse one budget across both candidate-child checks; avoid invoking a new complete Project validation per annotation.
- **Native:** one revision-bound cut draft and one review modal are enough. Reuse retained-input guards, typed boundary selectors, preview candidate validation, and existing modal event shielding (`native main.rs:1137–1160,2870+,3192+,4800+`). Entry must refuse pending inspector edits rather than feed them through `execute`/`apply_inspector_action`, which can commit them. Add cut state to reset/stale/modal/selection/history/open/recover guards. Existing rebuild clears a selected removed child on Undo (`1180–1215`); select the appended child after Apply and clear curve selection. Show IDs alongside names for reopening/selection. Keep preview allocations/history and worker state isolated.
- **Adapters, persistence, exports:** one schema alternative plus adapter strict-byte/failure regression cases suffices. Wildcard core exports already expose Command. Ordinary v4 Piece storage and existing exporters already walk per-piece boundaries: SVG closes each path, DXF has per-ID layers, OBJ indexes piece-specific vertices/groups (`persistence.rs:310–416`, `dxf.rs:6–105`). Verify these paths; do not rewrite them merely because a cut exists. Existing per-piece meshing and edge-index-based bends should yield independent surfaces/particles; fresh-cloth mass and no cross-cut constraints must still be measured. Native save already allows recovery to advance before a late primary failure (`persistence.rs:280–306`), matching the draft's qualified promise.
- **Evidence/docs:** freeze input recipes, expected IDs, numeric/oracle code, native actions and caps before implementation/run. Preserve historical artifacts, README product scope, disabled Actions/Pages, and full PAT-006 unknowns. The four Rust gates, actual native acceptance, and tracked clean-checkout publication gates remain separate future work.

Skipped: implementation, Cargo/tests, GUI/native execution, Git writes, remote operations, and public evidence changes. Risk: this is a static contract review; runtime meshing/mass, numeric refusal boundaries, and coincident-edge contact behavior remain unverified.
