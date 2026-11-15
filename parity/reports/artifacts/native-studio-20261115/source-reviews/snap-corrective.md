What this change does: Corrective candidate 7a1ef8f7a9fbda9b8b1400d7a05723268153b360 replaces the popup with four inline choices. Its same-frame, revision/selection-stamped completion after inspector rendering addresses the historical raw-text/caret loss without copying edit buffers.

Must fix

1. [P2] Keyboard activation restores stale numeric focus during M:N sewing (apps/weftforge/src/main.rs:1649-1659; completion at 1590-1605).
   - What this is: Every response.clicked() runs the pointer-preservation guard and queues a numeric-focus restoration. egui defines clicked() to include keyboard and accessibility activation.
   - Problem: In Tool::SewingPaths, focus and edit Mesh edge length, move keyboard focus to Off, then press Enter. Off is selected, but focus jumps back to the previous numeric field. inspector_focus retains that cached ID while the inspector's acting flag is true for SewingPaths; the new same-frame stamp still matches. The new keyboard test starts without prior numeric focus, so it cannot see this.
   - Fix: Keep selectable_value activation unchanged; gate the numeric preservation guard/request on response.is_pointer_button_down_on() || response.clicked_by(egui::PointerButton::Primary). Extend the keyboard case to begin with prior numeric focus while SewingPaths, and assert that activating Off leaves keyboard focus on Off.
   - If we skip it: Keyboard users unexpectedly leave the Snap choices and type into an old numeric field, violating the amended normal-focus behavior.

Evidence: keyboard-review.log records focus_before_enter_is_off=true for both tools. With Select, focus_after_enter_is_off=true. With SewingPaths, focus_after_enter_is_off=false and focus_after_enter_is_old_numeric=true. Both select Off. The private driver includes the exact candidate main.rs, and cargo run --offline --target-dir target --bin keyboard-review exits 101. It is one focused headless check using the review package's own target; no native window or full suite was run. The source snapshot and frozen equivalent driver are corrective-7a1-main.rs and keyboard-review-frozen.rs.

The original 2950 raw-text 9/9 RED log remains unchanged and is also frozen with its historical source under red-2950. The source was copied from Git after the corrective candidate arrived, as identity.json explicitly records; no contemporaneous snapshot claim is made.

Reviewed statically: every shared to_pattern caller remains unchanged; captured drag setting/cancellation, negative/tie/Off metre conversion, exact numeric dimensions, session-only preference, and typed history boundaries. The new request is consumed once after inspector and rejects stale frame, revision, selected-piece and selected-curve identities. No other concrete source regression found. Supplied inline-delivery identity records native 44/44, fmt and strict native Clippy passing; these were not rerun.

Verdict: fix 1 first.
Not checked: actual window behavior, full workspace gates, screenshots, geometry/physics acceptance or reference parity. Revision/selection invalidation was traced statically only.
Risk: the keyboard focus jump is confirmed; candidate should remain held for the one-line pointer-only guard fix.
