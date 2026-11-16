What this change does: Adds the native rectangular-hole review, selection and
corner-edit workflow on the shared Hole commands. Both construction loops draw
and pick as editable boundaries, while sewing/cut routes retain their restrictions
and the 3D view draws only a current-revision surface.

**No concrete Must fix or Should fix finding in this native diff.** Reviewed
`.local/straight-holes-20261116/apps/weftforge/src/main.rs` against base
`982ad0340d3c848bb8a959dca7a934886f1d7ae9`, including connected methods and new
tests. Assumed use is the existing local desktop, single-author workflow.

Reviewed file SHA256:
`71b2ff79a315a5de5347ce36a4a07a13a3e9faca6c46e77086338bb24b8a2d48`.

Checks traced in source:

- `HoleDraft:437–495` parses finite raw strings, then builds its isolated candidate
  through the shared Editor/AddHole command. Preview never advances accepted IDs,
  revision, dirty state, mesh or cloth. Core eligibility handles outside, degenerate,
  oversized and unsupported-owner input without publishing the candidate.
- `open_hole/hole_window:3318–3472` refuse pending inspector changes, retain source
  input/caret, consume opening Return and additionally require a later frame before
  Return can apply. Invalid Apply retains the draft/error and restores field focus.
  Explicit Cancel, Escape and backdrop restore retained source input without an
  accepted command. Apply uses one command and retains owner selection while
  selecting the new hole's first curve.
- `ui:5401–5749` includes the hole in authoring-modal shielding: shortcuts are gated,
  background UI is disabled, and events are removed while background numeric widgets
  render, then restored for the modal. Existing cut/annotation handlers remain in
  their original routes. New Empty refuses an unfinished hole; failed load retains
  it, while successful load/recover, history, tool/reset and rebuild invalidate it.
  Owner/revision/tool checks reject stale drafts; opening epoch participates in
  modal identity and Enter guarding.
- `InspectorDraft:621–688`, inspector controls and `selected_hole:3474–3483` use
  stable curve/point/Hole IDs. Endpoint edits become shared MovePoint commands in
  one inspector transaction. Split/join use the loop-aware core; removal addresses
  Hole ID separately from Delete Owner Piece. Rebuild retains surviving selection
  and clears removed curves; history follows the existing no-invented-selection
  policy. No Hole ID goes through a numeric f64 widget.
- `patterns:4337–4392,4769–4948` draw/pick both solid loops and resolve point ownership
  through all construction curves. Only outer loops populate sewing candidates;
  free sewing remains outer-only and `cut_piece_pick:6305–6311` refuses a holed
  owner. Hole drag completion restores its owning piece/curve selection.
- Rebuild invalidates simulation and cancels old work; job acceptance remains
  revision/cancellation guarded. `garment:5293–5296` hides previous-revision meshes
  and reports failed current meshing plainly. OBJ still requires a current mesh.
  Autosave/exit retention clone accepted Project construction, so an uncommitted
  HoleDraft never becomes saved geometry; committed holes use the shared v5 writer.

Read the five new hole tests covering menu/search/inspector opening Return, raw
text/caret and outside refusal, cancel and background shielding, point-edit/history/
split/join/removal, pointer ownership and forbidden sewing/cut picks, same-revision
load/reset invalidation and stale mesh display. These are meaningful implementation
checks, not actual native interaction evidence. The UI agent's reported test and
Clippy results were not independently rerun here.

**Verdict: Ship this reviewed native diff after the required integrated local
gates.** This is a source-review verdict only; there is no native/reference
acceptance or parity credit.

Not checked: no Cargo/tests, evaluator, GUI/CUA, launch or runtime observation ran;
no source was changed. Actual focus/caret, OS dialog, restart and native mesh
readiness remain unobserved until separately accepted runtime work.
