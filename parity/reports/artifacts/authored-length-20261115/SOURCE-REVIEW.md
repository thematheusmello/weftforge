# Independent authored-length source review — 2026-11-15

What this change does: It reports committed interval, legacy seam and ordered
sewing-group lengths from the existing authored metric, with unrounded SI output
through CLI/MCP. The native inspector presents millimetres, identifies the applied
revision, separates drafts/previews and caches selected-curve/visible legacy
results.

Reviewed runtime source: `a8205194608ddf95a81730c1d1698d559f236cd6`, against
`7ab2107aaa51f2c0fa9c9fd2ada972a08c433882`. The inspected worktree HEAD was
`7976bfc564d947066e753d8d399e23957129c75e`; its apps/crates/Cargo files had no
diff from `a820519`. Immutable `git show a820519:…` reads anchor native line
references below. This review independently verified the suspected scroll issue;
it did not assume the parent's report established it.

The frozen contract `spec/features/PAT-007-authored-length-feedback.md` matched
SHA-256 `4f79150ed28854b8a16760af07b1558808dcca2080bc1ee8e114d5b2682f851c`.
Assumed load: one native desktop document, potentially enough authored seams to
overflow the Properties scroll area, plus explicit stateless local CLI/MCP
queries. No document-format, solver or dependency changes are present in this
runtime increment.

## Must fix

1. **Legacy feedback changes scroll geometry when it leaves the viewport**
   (`apps/weftforge/src/main.rs:1940–1953`).
   - **What this is:** Each legacy seam gains a child UI containing the applied
     revision, draft qualifier and pair lengths/error. The integrals should be
     lazy while the row's layout remains stable.
   - **Problem:** `ui.is_rect_visible(ui.min_rect())` is checked before this
     child adds any widgets. Its initial rectangle is a zero-sized point at the
     cursor. When that point is clipped, the branch adds no feedback height;
     when visible, it adds the complete block. Therefore scrolling a seam's
     feedback start above the viewport removes that block's height, shifts
     following rows upward, and changes the scroll area's total content height.
     Cached metrics do not prevent the collapse because the visibility check
     also gates rendering of cached values. With multiple seams, repeated
     transitions can move the scrollbar and the controls the user is targeting.
   - **Fix:** Allocate the feedback block consistently and gate only uncached
     metric computation on the visibility of that allocated block. Preserve the
     existing separate egui ID scope. Add one scrolling regression that moves
     already measured rows outside the viewport and checks stable content/row
     positions and reuse of cached results.
   - **If we skip it:** A normal Properties scroll changes the document's UI
     layout as rows pass out of view, making later sewing controls move and
     making partially clipped feedback disappear prematurely.

The layout conclusion follows directly from the installed, locked egui 0.35
source: `Layout::region_from_max_rect` seeds `min_rect` with
`Rect::from_center_size(seed, Vec2::ZERO)`; `Ui::scope_dyn` advances the parent
cursor using the child's final `min_rect`; `ScrollArea` derives `content_size`
from `content_ui.min_size()`. No native execution was used to establish this
finding. The new lazy-cache test at `main.rs:10310–10334` changes viewport height
but never scrolls, so it does not cover this case.

## Connected source checked

- `Project::seam_lengths` now delegates both authored intervals to
  `Project::interval_length` with `?`. Its production callers are CLI `inspect`,
  the shared `length_report` seam variant, and the new native lazy cache. The
  existing stitch-generation/correspondence path and solver consumption are
  separate and unchanged.
- `IntervalMetric::new`, its integration limits, and group metric derivation
  retain finite/positive, interval, precision, depth and work checks. The
  interval query supplies the original ordered endpoints; raw pair order is A/B
  and mismatch is exactly returned B minus returned A. No fallback replaces a
  failed metric. Group queries reuse `sewing_group_geometry`; native group
  feedback reuses mesh geometry without calling that wrapper on repaint.
- CLI arity is checked before indexing arguments. Endpoint parsing uses the
  existing finite-number parser. The shared `LengthQuery` is closed and
  `length_report` is used by both adapters. `inspect` collects metric `Result`s
  instead of turning errors into `null`.
- MCP's closed argument/query schemas, tool allowlist/discovery, annotations and
  typed original-byte reparsing reach `project_lengths`. Duplicate fields are
  not erased through a `Value` round trip. Both adapters load using the bounded,
  validated `Project::load`; the query paths create no `Editor`, mesh, save or
  recovery write. MCP success text and structured content use the same report
  value; errors use the existing explicit tool-error envelope.
- Selected-curve results, including errors, live in `InspectorDraft`; the
  retained draft is keyed by revision, selected piece and curve. Applied length
  is excluded from the authoring-command comparisons. Calling `commands` can
  rebuild a baseline and integrate during explicit actions; idle inspector
  rendering retains its draft instead.
- `apply_command`, history restoration and successful import/export candidate
  adoption call `rebuild`. `rebuild` clears both the selected draft and legacy
  snapshot. Successful open/recover clears the mesh and invokes `rebuild`, so a
  replacement with identical IDs/revision does not retain old lengths or group
  geometry. New-document construction starts with fresh state. Failed load
  leaves the previous editor and snapshots in place.
- Feedback is placed in explicit child ID scopes. Pending/success/error widget
  count changes stay within those scopes, preserving the surrounding numeric
  editor identities. Applied metrics read committed project geometry, not
  numeric text, point preview or group-edit drafts. The permanent qualifier
  discloses this distinction. No feedback path calls authoring, selection,
  cancellation, mesh or solver mutators.
- Group geometry is filtered by document revision. Mesh failures/cancellations
  are shown only when the finished mesh job belongs to the current revision,
  no job is active and no rebuild is queued. Pending/obsolete jobs cannot supply
  current-looking old values. Successful document replacement also drops the
  old mesh even when revisions match.
- Read the added core analytic/independent-oracle, validation/read-only and
  revision tests; adapter/schema/lifecycle/file-retention integration tests;
  native format/cache/focus/caret/draft/replacement/job-state tests; and the
  updated automation documentation. No other concrete correctness issue was
  found in these connected paths.

Verdict: fix 1 before accepting the native increment. The CLI/core/MCP source
paths have no further blocking finding from this review.

Not checked: No Cargo command, executable acceptance run, CUA, reference
investigation, push, Actions or publication was performed. The parent's reported
focused test/Clippy and independent 46-process query results were context, not
checks rerun by this reviewer. Full workspace gates and native acceptance remain
the root's responsibility; this review makes no reference-parity claim.

Skipped: Source edits, style-only suggestions, new frameworks and redundant
palette/implementation-mirroring tests. Risk: the scroll-layout defect above is
still present in the reviewed `a820519` runtime source.

---

# Separate fix review — `be8a031`, 2026-11-15

What this change does: It always allocates the legacy feedback block, then
measures only an uncached pair whose complete allocated block intersects the
viewport. It gives unknown, successful and failed pairs three single-line slots
and requests a repaint after a new cached result becomes available.

Reviewed fix: `be8a031b5f8342d0857f3bed34f52dd165fa86da`, atop root metadata
`4c9fca1`. The worktree was clean at that exact HEAD. Comparing apps, crates and
Cargo files with `a820519` showed only `apps/weftforge/src/main.rs` changed;
CLI/core/MCP conclusions above remain applicable. The original finding and
original verdict above are preserved as the historical `a820519` review.

**Finding 1: resolved in `be8a031`. No new concrete finding.**

- `main.rs:1955–1965` lays out the heading and pair/error/unknown slots
  unconditionally inside the existing explicit `applied-length` ID scope.
  Visibility now gates only the vacant cache insertion, using
  `feedback.response.rect` after layout. The full block rectangle detects
  partial visibility even when its heading has scrolled above the clip. Cached
  successes and errors are displayed without reintegration. Each insertion
  requests a repaint so the next frame replaces placeholders with the result;
  later frames do not repeat the insertion or repaint request.
- `main.rs:326–338` provides exactly three label slots in every pair state.
  Error CR/LF characters become spaces so a reason cannot add rows; the third
  error slot contains a space to preserve its height. `truncate()` prevents long
  values or reasons from wrapping. Unknown slots contain em dashes, so they do
  not present a zero or an old result as a successful measurement. The agreed
  three-row A/B/B-minus-A layout preserves three-decimal millimetres and the
  signed mismatch.
- Checked the locked egui 0.35 `Label` implementation: truncation limits each
  label to one row; `Label::new` defaults `show_tooltip_when_elided` to true;
  its widget code supplies the original full text on hover when elided.
  Consequently the shorter visual labels retain the full numerical string or
  error reason without custom tooltip or focus code.
- The child scope remains present exactly once in all states. Its labels do
  not alter auto IDs outside that child. Numeric authoring, selection, command,
  focus/caret and draft handling are unchanged. The corresponding group block
  uses the same pair slots plus one portions/status row, inside its existing
  scope; its revision and mesh failure/cancellation checks are unchanged.
- Read the new actual-scroll regression at `main.rs:10326–10416`: it creates
  25 seams at a fixed 420-pixel viewport, changes stored scroll offset, verifies
  the offset really moves, compares content heights before/after/return,
  confirms only some pairs are measured, checks idle reuse, and injects a long
  cached error to verify the same footprint. The partial-clip regression at
  `main.rs:10419–10463` positions a value row at the viewport top while the
  heading is clipped, then checks that exactly one pair is measured over four
  frames. These tests exercise the reported root cause without a new framework.
- Read the supplied receipts, without rerunning them:
  `scroll-layout-red-7976bfc.log` reports the old-runtime regression failing
  with content height `2198 -> 2290`; `scroll-layout-green2.log` reports all six
  focused authored-length tests passing, including fixed content height
  `3787 -> 3787 -> 3787` and only `2/25` measured pairs. The other passing tests
  include focused numeric text/caret/draft retention, replacement invalidation,
  metric caching, and mesh state handling. These are inspected implementation
  receipts, not independently executed acceptance evidence.

Verdict: Ship the source fix to the root's integrated checks. The original
blocking finding is resolved for `be8a031`; this source review approves moving
on to full workspace gates and native acceptance.

Not checked: No Cargo, executable/native acceptance, CUA, reference work, push,
Actions or publication. Formatter/strict native Clippy success was reported by
the implementer, not rerun here. The root still owns the full integrated gates,
native acceptance and evidence publication; this is no reference-parity claim.

Skipped: Source changes and additional test/framework requests. Risk: full
workspace and native acceptance results remain outstanding at this review point.
