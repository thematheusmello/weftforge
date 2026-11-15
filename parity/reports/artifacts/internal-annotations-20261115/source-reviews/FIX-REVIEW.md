What this change does: The fixes flatten compound inspector actions into one transaction, retain the source selection and boundary draft until an endpoint move succeeds, and prefer a selected annotation when endpoint distances tie. They also add an Escape path that cancels a point drag while a numeric source field retains focus.

This bounded source follow-up reviews immutable commit `703a8888526790e7ebe310a8df00907a154db5a5` against original reviewed commit `f7b7a4e660f2a5085c23980a6bb6447e067e4c63`. References below use `703a8888526790e7ebe310a8df00907a154db5a5:apps/weftforge/src/main.rs`. The original report remains unchanged in `UI-REVIEW.md`.

## Original findings

| Original finding | Source result |
| --- | --- |
| 1. Nested transaction with two endpoint moves and a pending property | Corrected at L851–L858. The helper extends the pending command list with submitted transaction actions and creates one final transaction. Core's unchanged final-candidate validation and nested-transaction refusal still apply. |
| 2. Drag recognition overwrote the staged boundary curve | Corrected at L4178–L4187 and L4224–L4243. Recognition retains the source selection/draft; only a successful release changes selection to an internal target. Rejected release retains the source draft, text, and selection. |
| 3. Coincident endpoints moved the wrong selected annotation | Corrected at L4149–L4164. Distance remains the first ordering key; equal-distance candidates prefer endpoints of the selected internal curve. |

For a boundary-point target, the post-success internal lookup is empty, so the prior boundary-drag selection behavior remains. The tie preference applies only when the current selected curve has an internal role. Modal construction and the existing backdrop cancellation policy are unchanged.

## Must fix

4. **P2: Escape restores older source text and caret after edits during a drag** (`703a8888526790e7ebe310a8df00907a154db5a5:apps/weftforge/src/main.rs:L4715–L4716`; snapshot capture L4187; restoration L215–L232; delayed restoration L3464–L3490).
   - **What this is:** The source inspector remains enabled and its numeric focus is retained during a point drag. `drag_input` captures its text and caret once when the drag is recognized, and the new Escape branch restores that captured snapshot through `close_without_commit`.
   - **Problem:** Enter a pending boundary control value such as `0.3125`, start an internal endpoint drag, then replace the still-focused source text with `1e-` or move its caret while holding the pointer. Escape cancels the drag but restores the older `0.3125` text and its earlier caret; `close_without_commit` also queues that old snapshot for later passes. The later source input is silently lost even though the source selection and document revision are still valid.
   - **Fix:** Cancel this nonmodal drag with `finish_numeric_action(&ctx, input.focus, false)` without restoring its drag-start text snapshot; the source widget remains present. Add a regression that changes source text and caret after drag recognition, presses Escape, then releases the pointer and checks the latest draft/raw text/caret plus unchanged construction, redo, and accepted outputs.
   - **If we skip it:** Cancelling a drag can discard a newer pending source edit. This affects the shared boundary-point cancellation path as well as internal endpoints because the new Escape branch handles every `drag_point`.

The source path is concrete: L4187 is the only assignment to this drag's retained input; later numeric events are still processed by the enabled inspector; Escape at L4708–L4717 runs before that pass's inspector rendering and invokes `restore`, which unconditionally replaces both cached text and text-edit state. No reference or native-execution inference is needed for this overwrite.

## Regression source and limits

Read the focused production diff, all `apply_inspector_action` callers, retained-input helpers, drag recognition/release/staleness/reset paths, inspector commit suppression, and the added tests at L16013–L16362. The six new tests directly cover the original compound success/failure, boundary-draft accepted/rejected/cancelled drag, and coincident-endpoint cases. Their cancellation inputs are entered before drag recognition; they do not update source text or caret after the snapshot is captured, so they do not cover finding 4.

Producer reports six RED failures on byte-identical original production, six GREEN results, 104 passing native Rust tests, and strict Clippy passing, with evidence files 08–13 under `.local/internal-segments-native-implementation-20261115`. These remain attributed producer results: this reviewer did not run tests or independently validate those executions. The additional core change since the original review is a test-only file change; the reviewed core `document.rs` blob is unchanged.

The immutable native blob is `548f0964ea7064922ac8199d77aeb5a4eb8bf145`, SHA-256 `3b3e4328013bd1992a32fdf545ee5b0c6fe683ecad94b4b5013c27839c8b46e3`. The worktree HEAD matched the reviewed commit and its tracked status was empty. This private report and receipt are handoff material, not published delivery evidence.

Verdict: original findings 1–3 are corrected in source; fix 4 before accepting the cancellation behavior.

Not checked: Cargo, tests, evaluator, real native/OS/reference/CUA behavior, or artifact evaluation. Risk: source and in-process test coverage cannot establish actual OS focus, accessibility, pointer capture, rendering, or reference parity.
