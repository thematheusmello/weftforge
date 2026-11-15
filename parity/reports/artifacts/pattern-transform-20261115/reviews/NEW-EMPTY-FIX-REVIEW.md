What this change does: It binds an active Transform drag to the widget that owned its pointer press. Interrupted drags retain their preview without rearming from later clicks, and Transform pattern… can transfer the retained values into the existing numeric editor.

Independent read-only source review of immutable `bc599b13548d8ffb843968f12d2dc63cff412357` against `133c6d3fd91601b897f584e03c92765f79f97993`. Verified `apps/weftforge/src/main.rs` SHA-256 `7c9e07ff6444acfdae47a9d5db96f46294e5427c02e431e03ca23a8201924374`. Original NEW-EMPTY-REVIEW.md remains unchanged. No tests or native runtime were executed by this reviewer.

The original Snap-after-paused-release finding is addressed in source: a paused release or capture loss clears `pointer_owner`, and a detached gesture cannot acquire a new owner from later Snap, numeric or canvas presses. The producer's reported RED revision change R8 → R9 and subsequent regression results are separate execution evidence. Two concrete edges remain in the fix.

**Must fix**

1. **[P2] A same-pass boundary click leaves an unfinished identity Transform** (`apps/weftforge/src/main.rs:2766`, `:3094`, `:3124`).
   - **What this is:** `owned_transform` and `released_transform` are computed before the later block that creates a new gesture on primary press.
   - **Problem / repro:** With Transform selected and no gesture, deliver a normal boundary click as primary press and release in the same UI pass. egui 0.35 retains both press and release events for that pass, so the press condition creates an identity gesture, but the previously computed `released_transform` is false and cannot remove it. On the next idle pass capture loss detaches that identity and leaves it pending. New Empty then refuses an unfinished gesture after an ordinary selection click. Before this fix the global release branch consumed the identity gesture in that same pass.
   - **Fix:** Do not create a drag whose pointer is already up, or evaluate release for the new owner after creation so that same-pass identity clicks are consumed. Add a raw press+release-in-one-pass boundary-click test asserting no retained gesture/history change over idle frames and no unfinished-gesture refusal from New Empty. The new regression matrix sends press and release in separate frames.
   - **If we skip it:** Fast/coalesced boundary clicks can leave a false pending construction gesture and block document replacement until the user cancels it.

2. **[P2] Opening the retained Transform editor restores an older inspector snapshot** (`apps/weftforge/src/main.rs:2467`, `:2625`).
   - **What this is:** `open_transform` now moves the entire old `gesture.edit` into the numeric editor, including the inspector text/caret snapshot from the original drag start.
   - **Problem / repro:** Start with focused Warp angle raw text `0.47`, begin a Transform drag, pause/release it through search and trigger New Empty refusal. While the detached gesture is pending, edit that property to `0.57` (or unfinished `1e-`), then click Transform pattern…. The new adoption path retains the old `inspector_text`; `transform_window` calls `restore_inspector_text` at the end of the opening pass and restores `0.47` and its old caret over the newer input. Inspector edits remain staged while `transform_gesture` exists, so no revision change prevents this sequence. The new matrix clicks a numeric field without changing its text and therefore does not cover this case.
   - **Fix:** Preserve the retained transform's translation/angle/pivot, but capture the live inspector focus/text/caret when adopting it into the modal, as a fresh numeric opening does. Add a test that edits raw property text after detachment, opens the numeric editor, and verifies the newer raw text/caret and staged commands through failed apply/cancel and later frames.
   - **If we skip it:** Explicitly finishing a paused gesture can discard a newer unrelated property edit before any transform is applied.

Read all `TransformGesture` constructors and `open_transform` callers, capture/read-response ordering, update/release/detach paths, numeric adoption/restore/cancel, shared apply/transaction handling, Escape cancellation and the new boundary/handle × File/search regression matrix. egui's `read_response` rebuilds a response from current interaction state and this/previous-pass widget geometry; reading the handle before drawing it is not itself a finding. No additional concrete findings in this bounded review.

Verdict: fix 1 and 2 first.

Not checked: No Cargo, native app/CUA, OS accessibility or reference comparison. Producer focused/full-native/Clippy results were not rerun or counted here; the two remaining cases are source-traced and need producer RED/green reproduction. This is not native acceptance or a parity claim.
