# Native transform source review

What this change does: The native studio gains a Transform pattern tool, numeric
modal, boundary translation and a rotation handle. It sends the shared typed
command, combines pending inspector commands into one transaction, and keeps drag
previews separate from the accepted document and 3D output.

Reviewed immutable commit `1dbe5bc0265c3f169cbc2b71af7709db5c5c30f0` against
`02bcd8c833376d2ba7c007c7dcd2516337d8a02f`. All line references below refer to
that immutable commit's `apps/weftforge/src/main.rs`. The checkout was clean at
the start of review. The frozen PAT-004 contract SHA-256 was verified as
`afffdf4e68ca5afb7f0038d2568576feb66824f873a8aaa659f60bfc072bf557`; the root
`NATIVE-SELECTION-CLARIFICATION.md` supplement was also read. Assumed load: one
person editing one local studio document.

## Must fix

1. **Scene and 3D selections bypass Transform's staged-edit refusal**
   (`main.rs:2344-2372`, callers at `main.rs:1897-1900,3296-3305`).
   - **What this is:** The clarification requires a cross-piece click in Transform
     mode to keep the current selection, pending property edit, text/caret and
     Redo, and tell the user to apply or cancel the property edit first.
   - **Problem:** Only 2D canvas selections call `select_transform_piece`
     (`2962,3035`). Scene rows and picked 3D triangles still call `select`, which
     invokes `commit_draft` before changing the selection (`792-796`). For a
     concrete event case, use the existing transform fixture, stage the selected
     piece's warp angle as `0.47` with `focus_property`, and send an AccessKit
     Click to the Scene row named `Separate panel`: this route commits the angle,
     consumes the draft, clears Redo and selects the other piece instead of
     refusing. With unparseable text, the old piece's draft/text focus can instead
     be consumed during selection. The inspector's Transform blur guard also
     covers only pointer activity within `pattern_canvas` (`1912-1920`), so a
     pointer fix must protect the press/blur frame as well as the release.
   - **Fix:** Use the existing Transform selection refusal and input-retention
     behavior for Scene rows and 3D picks while Transform is active, including
     their initiating pointer frame. Extend the existing cross-piece test to
     cover both routes, valid pending text, `1e-`/`NaN`, and normal selection with
     no pending edit.
   - **If we skip it:** The same selection action has different history and
     draft-retention behavior depending on the clicked view. A user can lose
     Redo or their correctable input by selecting the other piece.

## Other checked paths

- Tool label/icon/rail and search route (`21-38,1612-1616,1626-1645,3566,3600`)
  enter the same tool state. The named rotation handle opens the same numeric
  modal as the contextual button (`2920-2930,2990-2991`). Select retains its
  original individual-point path (`3000-3017`).
- Numeric fields use metres/radians, explicit original-coordinate pivot, and a
  bounds-center default. Their modal identity includes piece/revision/opening
  epoch; cached text is separate from accepted authoring values
  (`2311-2342,2413-2515`). A valid identity closes without dispatch, preserving
  pending inspector edits; invalid pivot/text does not use that bypass
  (`2479-2507`).
- Success calls `apply_inspector_action`, whose transaction includes pending
  inspector commands before the transform (`2392-2411,648-667`). An error leaves
  `Editor`, draft and accepted mesh/cloth/job untouched by that action; only
  success enters `rebuild` and clears accepted-input caches (`614-667,824-859`).
- A gesture owns an original f64 `Project`, captured pending commands, source
  revision/piece/pivot and a snap mode selected once with Alt at press. Every
  update computes displacement or signed angle from the original start;
  preview constructs a disposable editor and uses the shared command
  (`255-279,2645-2672,2945-2977`). Release dispatches once, and failure retains the
  attempted transform/error in the modal (`2979-2988`).
- Escape, tool/reset, successful load, revision/selection mismatch, search and
  popup paths discard active previews (`670-701,1009-1030,2374-2390,2611-2614,
  3349-3364`). Modal cancellation restores the retained inspector text and caret
  through the existing focus/cache guard (`224-251,2517-2547,3886-3942`).
- Authoring-modal state blocks global shortcuts and disables the background.
  Input events, including AccessKit requests, are withheld while background
  widgets render and restored before the modal renders (`3349-3354,3456-3464,
  3617-3624`). This covers egui's otherwise active disabled numeric widgets.
- Transform sewing-guide previews evaluate the preview document, while point
  editing continues to use accepted construction for ordered sewing guides
  (`2777-2810`). The `paint_sewing_interval` signature change has both callers
  updated (`2793,2822,3104-3127`).
- Read all seven new tests (`10727-11509`), including their event helpers,
  fixture, transaction/geometry checks, raw text/caret checks, failed-release
  correction, no-op modal and stale-gesture cases. The cross-piece refusal test
  currently exercises only the 2D boundary route (`11034-11089`).

Verdict: fix 1 first.

Not checked: Cargo/test binaries, native/CUA or reference execution, full workspace
gates, publication/registry/map checks, PR/push or Actions. Reported 62/62 native
test and Clippy results were not rerun or treated as proof. This source review
establishes no reference parity; ignored local reports are not published evidence.

## Selection fix review

What this fix does: All native piece selection routes now use one shared
`select(ctx, target)` method. In Transform mode it refuses a cross-piece
selection before committing or consuming the current property edit, and the
Scene/3D pointer paths keep the pending numeric input through their press frame.

Reviewed immutable commit `9c54688393e1b8b94a6854af47dab24835d18e46` against
`1dbe5bc0265c3f169cbc2b71af7709db5c5c30f0`. The candidate checkout was clean at
the start of this fix review. This section's line references refer to the
immutable `9c54688393e1b8b94a6854af47dab24835d18e46` version of
`apps/weftforge/src/main.rs`; the original finding above remains a record of the
earlier `1dbe` source.

Finding 1 is resolved in this source. No concrete remaining findings in the fix.

- `794-830`: The shared selection method performs Transform's staged-command
  and invalid-raw-text checks before any selection/draft mutation or call to
  `commit_draft`. Its refusal restores numeric focus and returns `false`.
  Same-piece Transform selection remains a no-op, while a clean cross-piece
  selection updates selection without creating history. Non-Transform behavior
  retains the prior commit-before-selection handling and failure return.
- `832-841`: A cross-piece `select_curve` request in Transform mode must pass
  that same refusal before it can commit or change the selected curve. The
  existing same-piece curve route keeps its established behavior.
- `1934-1940`: Scene rows retain inspector input on pointer press and dispatch
  their pointer/AccessKit selection through the shared method. Their input guard
  runs before the property widgets can process blur in that frame.
- `1952-1962,3226-3229,3309-3318`: The studio captures the garment canvas ID and
  includes it beside the pattern canvas in pre-render blur deferral. A 3D
  triangle pick now calls the shared selection method, so both the initial
  press and later selection release retain pending edits.
- Audited every production `select`/`select_curve` caller, including 2D gesture
  start/click, Scene rows, 3D picks and successful authoring result selections.
  They pass their current Context (`694,1177,1939,1992,2204,2612,2974,3039,
  3044,3047,3318`). Direct production selection assignments outside the shared
  methods are existing document-load, invalid-selection cleanup, and successful
  polygon-creation paths (`881,1052,1468`), rather than alternate click-pick routes.
- Read the two added regression tests (`11144-11302`). Scene coverage uses real
  AccessKit Click and pointer press/release on the named row; 3D coverage finds
  an isolated rendered triangle and sends pointer press/release through the
  actual garment picker. Both use staged `0.47`, raw `1e-` and `NaN`, check the
  editor/selection/draft/text/caret/Redo after settling frames, and assert no
  commit on press. The 3D test also checks accepted mesh identity. Clean Scene
  selection and clean 3D selection in both Transform and Select remain covered.

Verdict: Ship the selection fix to the integrated checks.

Not checked: I did not run Cargo/tests, inspect native/reference execution, or
perform source edits. The reported nine focused tests, 64 full native tests,
strict Clippy and rustfmt passes are separate evidence owned by the root/native
agent; this source review does not establish reference parity or replace the
root's required integrated gates.
