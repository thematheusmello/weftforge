What this change does: Point-drag cancellation now keeps the live source input when its field retained focus. If that field already blurred, cancellation captures its current text and caret before using the existing refocus-retention helper.

This final bounded source follow-up reviews immutable `ad7cc8bbbfc7a1ae735a400c21dcc7a37c80a5c5` against `703a8888526790e7ebe310a8df00907a154db5a5`. Only `apps/weftforge/src/main.rs` changed between those commits. Earlier review reports remain unchanged.

Looks good in the reviewed source scope. Finding 4 is corrected; no remaining actionable defect was found in this focused pass.

Checked at `ad7cc8bbbfc7a1ae735a400c21dcc7a37c80a5c5:apps/weftforge/src/main.rs`:

- **Focused source, L4715–L4720:** The `had_focus_last_frame` branch calls `finish_numeric_action` and does not restore text or text-edit state. Consuming Escape before inspector rendering cancels the gesture while leaving same-pass text/caret events available to the source widget. No drag-start snapshot is queued for later replay.
- **Previously blurred source, L4721–L4723:** The fallback calls `free_sewing_input` at cancellation time, whose L2783–L2792 implementation reads the current cache and current text-edit state. It no longer calls `close_without_commit` on the drag-start snapshot. This uses the house refocus protection for a DragValue that can otherwise clear its raw cache when it gains focus again.
- **Delayed restoration, L3464–L3490 and L5042:** The existing end-of-pass helper checks document revision and selected owner, bounds restoration to the cancellation pass plus three passes, and stops retention for the existing key/pointer/AccessKit new-input cases. The pending numeric draft is not replaced by this fix. In the reviewed blurred sequence, the queued snapshot contains the latest source text/caret, not the earlier drag-start input.
- **Shared behavior:** Escape still clears the same drag/preview fields and runs only with global shortcuts enabled. The patch changes no modal construction, modal event shield, backdrop policy, point picking, release command, accepted-release selection, boundary-role handling, history, or rebuild path. The preceding fixes for findings 1–3 remain untouched.

Read the added regression source at L16242–L16380. It first changes the valid pending boundary control from `0.3125` to `0.4375`, then enters invalid raw `1e-` while retaining that newer numeric backing value. It explicitly distinguishes focused and prior-focus-loss states. The focused same-frame case adds text/caret input together with Escape. After cancellation it releases the pointer, advances four more frames, and checks the latest draft/raw/caret plus unchanged construction/history/selection, accepted mesh/cloth, worker cancellation flag, dirty state, and redo.

Read producer outcome `20-live-cancel-outcome.json`. It reports a matching RED failure against byte-identical `703a888` production, final GREEN coverage for the three focus/event sequences, 11 cancellation tests, 105 native Rust tests, and strict native Clippy passing. The earlier navigation attempts remain explicitly qualified diagnostics in that outcome. These are attributed producer results; this reviewer did not execute or independently reproduce them.

Source identity: native Git blob `c2fa9140bbc934a0e73f70505f63829ac41eff1c`, SHA-256 `16f92de909c94b9103612ce363c52390858dfc9f96004f13e4ae2351da195d45`. The worktree HEAD matched the reviewed commit and its tracked status was empty. Core `document.rs` and the approved PAT-006 contract blobs are unchanged. The identity receipt records these reads; the private report is handoff material, not published delivery evidence.

Verdict: scoped source clear for finding 4. The four reported source defects are now corrected; proceed to the integrated local checks and the separately required actual-native acceptance on the identified candidate.

Not checked: Cargo, test execution, evaluator, native/OS/reference/CUA behavior, or artifact evaluation. Risk: this source clearance and producer in-process results do not establish OS focus/accessibility/pointer behavior or reference parity.
