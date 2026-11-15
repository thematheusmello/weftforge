What this change does: It avoids starting a Transform drag after a same-pass click has already released, and captures current inspector metadata when opening a retained Transform in the numeric editor. It preserves only the retained translation, angle and pivot across that adoption.

Independent read-only source review of immutable `dd5a16826dafe0d4f32c251f7c351197ad0936af` against `bc599b13548d8ffb843968f12d2dc63cff412357`. Verified `apps/weftforge/src/main.rs` SHA-256 `7ba6b482cfcf20f00b2734159f8c44ee56138fae2665b9c31b50020435d59589`. Original reports remain unchanged. No Cargo commands or native runtime were executed by this reviewer.

Looks good. Ship this scoped fix.

- **Prior fix-review finding 1 resolved in source:** The start predicate now requires `primary_down()` as well as a press event (`apps/weftforge/src/main.rs:3095`). A press+release batch with the pointer already up therefore follows the existing selection/handle-click paths without leaving an identity gesture. The added regression sends both raw events in one UI pass, checks no gesture/history/Redo changes over idle frames, then confirms File → New Empty is not blocked by a false unfinished gesture.
- **Prior fix-review finding 2 resolved in source:** `open_transform` first discards stale state, builds a fresh draft through the existing `transform_edit`, and copies only translation/angle/pivot from the retained gesture (`:2467`–`:2484`). Current focus, raw inspector text, caret state, revision, modal epoch and validation state therefore come from the live opening. The added regression changes `0.47` to `0.57` or `1e-` after detachment, checks opening and failed-apply retention over later frames, then checks cancellation retention or a successful transaction containing the newer property edit plus the retained transform, followed by Undo.

Read the two production changes, both `open_transform` callers, live draft construction, stale-state guard, gesture ownership/update/release ordering, existing selection/handle-click fallthrough, modal restore/cancel, and shared apply/transaction paths. The previous detached-pointer protection remains in place. No additional concrete findings in this targeted review.

Verdict: Ship the two targeted source fixes.

Not checked: Cargo/full gates, native app/CUA, OS accessibility or reference parity. The producer reports both cases reproduced RED before the fix and the two focused checks passing afterward; those executions were not rerun or counted here. The reported clean-source 76-native/Clippy run was still running at handoff and is not claimed passed by this review.
