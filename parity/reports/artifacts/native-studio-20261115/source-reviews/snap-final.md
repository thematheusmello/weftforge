What this change does: Final candidate 539635bc75f5e8a2d7b63061178e6c082a4b7c0c keeps the inline session-only Snap choices and limits numeric-focus preservation to pointer interaction. The normal selectable_value path still handles keyboard and accessibility activation.

Looks good. Ship within the bounded native position-snapping scope. No concrete remaining source blocker found.

Checked: the two-line production guard and connected inspector completion; stamped request consumption rejects stale frames/revisions/piece or curve selection; every shared snap caller was traced in the preceding reviews and is unchanged. The expanded keyboard regression starts with numeric focus in both Select and SewingPaths, then focuses and activates Off and asserts retained choice focus plus unchanged editor/history. Existing tests retain raw text/caret/continued typing and geometry/history cases. The supplied source-bound keyboard-delivery identity reports native 44/44, formatting and strict native Clippy passing; the prior regression RED log and native GREEN log were read, not rerun.

Both historical RED witnesses remain preserved with source snapshots and hash-checked logs: 2950 popup text/caret loss (9/9 cases) under red-2950, and 7a1 stale keyboard focus under corrective-7a1-main.rs / keyboard-review.log. The 2950 source freeze was retrospective and is explicitly qualified; those failures are historical, not findings against 539635b. Final candidate was confirmed clean at its reviewed commit. No production source was changed by this reviewer.

Verdict: Ship.
Not checked: no new build, native window, screenshots, workspace-wide gates, garment/physics acceptance or reference parity. Approval is limited to this source increment and supplied implementation evidence; it does not claim full-contract or window-level acceptance.
