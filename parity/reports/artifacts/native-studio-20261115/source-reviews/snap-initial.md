What this change does: The candidate adds a session-only Off/1/5/10 mm pointer snapping selector and Alt/Option bypass. Existing rectangle, polygon, endpoint and numeric-rectangle-anchor callers share the conversion, while numeric dimensions and authoring commands remain unchanged.

Must fix

1. [P2] Snap discards unfinished numeric text and caret placement (apps/weftforge/src/main.rs:1627-1659).
   - What this is: The new popup remembers only the focused widget ID, then calls finish_numeric_action(..., false) to restore its focus.
   - Problem: Enter `1e-` into Mesh edge length, open Snap, then select Off, press Escape, or click away. In all three paths the String cache changes from `1e-` to `0.07` and the caret changes from 3/3 to selection 0/4. The parsed draft and editor stay equal, so the new valid-f64 assertions at lines 3326-3328 miss this. Next typing `2` produces raw `2` and resolution 0.5, instead of completing `1e-2` to 0.01. `-` and `0.` also lose raw text in all three paths.
   - Fix: Retain the raw String and text-edit cursor state with the focused ID for the current revision/selection, restore them after egui's blur/regain handling, and discard the saved record when that identity changes. Extend the existing pointer test to assert raw text/cursor plus continued typing for select/Escape/click-away.
   - If we skip it: A view preference deletes unfinished input and changes how continued typing is interpreted, contrary to UI-003's pending-input promise.

Evidence: `src/main.rs` includes the candidate main.rs without modifying it, and drives its actual toolbar/inspector/pattern code through egui RawInput. `cargo run --offline --target-dir target` used this review package's own initially absent target directory and exited 101. reproduction.log records 9/9 raw-text failures, with matching pre/post f64 draft and editor state and restored focus. This is a headless private reproduction, not a native-window or parity acceptance result. The harness was corrected once for numeric text lookup before the final recorded run.

Checked statically: every to_pattern caller; default and metre factors; negative/tie/Off behavior; drag snapshot and reset_gestures cancellation; selection/hit-testing independence; one-command drag/polygon completion; exact numeric dimensions; session preservation on open/recovery; new popup identity keys and existing numeric guards. No other concrete source regression found. Candidate HEAD remained 2950d545ada7b838a008850a7e37c32f49f67584 and clean after the reproduction. No edits to candidate, root source, DXF delivery, or public evidence.

Verdict: fix 1 first.
Not checked: supplied native suite/fmt/Clippy were not rerun; no workspace gates, native window, screenshots, garment/physics acceptance or reference parity. Same/new revision and selection restoration were traced statically, not driven in extra UI cases.
Risk: raw input and caret loss is confirmed; candidate should stay held pending the focused fix.
