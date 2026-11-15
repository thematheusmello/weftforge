What this change does: The native studio adds owned straight internal annotations, snapped two-click creation, a four-coordinate endpoint editor, selection, endpoint dragging, and role-aware removal. It uses the existing authoring commands and keeps annotations separate from cloth and exterior export profiles.

This is an independent source review of immutable commit `f7b7a4e660f2a5085c23980a6bb6447e067e4c63`, compared with core baseline `e6910096943e8503eb9b5bfc07119fc51353fc1f`. All line references below refer to that integrated commit, not the mutable checkout. The approved PAT-006 contract was read through Git objects and its SHA-256 matched `84014cffd5ec0c4fc3375b552d6238bd0b7dcda7a7ac5b10d7fc383621f69d66`.

## Must fix

1. **P2: Two-endpoint edits reject valid pending property edits** (`f7b7a4e660f2a5085c23980a6bb6447e067e4c63:apps/weftforge/src/main.rs:L3175–L3180`; connected helper `L846–L855`; core rejection `crates/weftforge-core/src/document.rs:L1289–L1296`).
   - **What this is:** The endpoint editor sends two `MovePoint` commands when both endpoints change. The inspector action helper also includes any staged property commands so the gesture has one undo entry.
   - **Problem:** Stage a valid grain edit, open the endpoint editor, change both endpoints, and Apply. `apply_internal_line` wraps the two moves in a transaction; `apply_inspector_action` puts that transaction inside a second transaction with `SetGrain`, and core rejects it with “Nested transactions are unsupported.”
   - **Fix:** Flatten a submitted transaction into the helper's pending command list before it creates the single final transaction. Add one regression with a valid staged property and two changed endpoints, checking one revision and one undo; also check atomic retention when the final geometry is invalid.
   - **If we skip it:** The required precise compound edit cannot complete even when every requested value is valid. Construction survives, but users must split one gesture into separate commits.

2. **P2: Beginning an internal endpoint drag discards a staged boundary edit** (`f7b7a4e660f2a5085c23980a6bb6447e067e4c63:apps/weftforge/src/main.rs:L4186–L4194`; connected draft commands `L514–L530`, release `L4233–L4237`, inspector suppression `L2333–L2345`).
   - **What this is:** Endpoint drag recognition changes the selected curve to its internal owner and replaces `InspectorDraft.curve` with the accepted internal curve. That same draft holds a selected boundary's pending cubic controls.
   - **Problem:** With an internal line present, select a boundary cubic and type a valid control-coordinate change without committing it; then drag an internal endpoint on the same piece. Canvas press suppresses the early inspector commit, but drag recognition overwrites the staged boundary curve, so release submits the endpoint move without the pending `SetCurveControls`; cancelling or rejecting the drag also leaves that staged boundary record lost.
   - **Fix:** Retain the source curve selection and inspector draft through the gesture, and select the internal target after its release command succeeds. Test a staged boundary control value followed by accepted, rejected, and cancelled internal endpoint drags; keep the existing boundary-drag selection behavior.
   - **If we skip it:** A normal mixed editing workflow silently loses an uncommitted boundary change. The old control field can also disappear while its retained raw input still points to that source widget.

3. **P2: Coincident annotations can drag the wrong selected line** (`f7b7a4e660f2a5085c23980a6bb6447e067e4c63:apps/weftforge/src/main.rs:L4146–L4154`; selection reassignment `L4182–L4186`).
   - **What this is:** Endpoint picking searches every document point and chooses only by screen distance. The approved representation permits coincident coordinates for separate annotations with different endpoint IDs.
   - **Problem:** Create two separate internal annotations with the same start coordinate, then select the second annotation in the inspector and drag that visible start endpoint. Equal distances select the first point in document order; the code switches selection to its curve and moves the first annotation instead of the selected second annotation.
   - **Fix:** When endpoint distances tie, prefer endpoints belonging to the currently selected internal curve, then preserve the existing nearest-point fallback. Add a two-annotation regression with identical endpoint coordinates and distinct IDs, selecting the later curve before dragging.
   - **If we skip it:** Users can change a different authored line from the one explicitly selected. Precise numeric editing remains available, but it does not make endpoint dragging respect the selected identity.

## Scope checked

- Read the native diff and connected production paths for tool/menu/search entry, selected owner and curve ownership, the precise editor, snapped clicks, both-endpoint iteration and press-origin picking, pending inspector commands, raw invalid text, focus/caret helpers, Tab/Return/AccessKit handling, separate pointer press/release, cancel, modal event shielding, and role-aware Delete/boundary/sewing routes.
- Traced revision/tool/owner changes, history, New Empty, successful and failed document load, recovery, save, resets, command acceptance, rebuild/worker invalidation, and native export omission help. The core review here covers the shared command variants, boundary-role guard, final candidate validation, transaction restriction, and editor history boundary; it is not a second full geometry/persistence audit.
- Read the added in-process native test source at `apps/weftforge/src/main.rs:L15242–L15994`. The precise two-endpoint case has no pending property, and the pending-property drag case changes one endpoint; neither exercises finding 1. The drag fixture has one annotation and does not exercise findings 2 or 3.
- The existing `Modal::should_close()` backdrop/Escape cancellation policy was preserved as the house policy. No new cancellation behavior is requested by this review.
- Assumed one interactive native-studio user and the existing bounded project limits. No new scale finding was established.

## Evidence limits

Source only. No Cargo commands, tests, evaluator, native process, reference application, CUA, screenshots, or filesystem IO failure injection were run. Producer reports of 98 native Rust tests and strict Clippy passing are attributed claims, not this reviewer's test results. Source tests do not prove OS focus, caret, accessibility delivery, pointer capture, rendering, chooser behavior, or reference parity.

The existing general Save behavior commits a normal inspector draft before chooser/IO. The contract explicitly retains that limitation, so this review does not claim failed/cancelled Save preserves every general pending draft. The precise endpoint modal is intended to shield background authoring, but actual native acceptance still has to establish that behavior.

The reviewed worktree reported the expected HEAD and no tracked modifications. This private untracked report and receipt are review handoff material, not published delivery or registry evidence.

Verdict: fix 1, 2, and 3 first, then run the integrated local checks and bounded actual-native gate on the identified corrected candidate.

Not checked: builds, test execution, real OS/native/reference behavior, and independent artifact/export evaluation. Risk: source fixes alone cannot establish the required native acceptance or reference parity.
