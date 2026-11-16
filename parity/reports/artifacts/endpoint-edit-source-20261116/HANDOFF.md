# Coalesced endpoint source increment — root review handoff

- Issue: https://github.com/thematheusmello/weftforge/issues/52
- Branch: feat/coalesced-endpoint-drag
- Base: 381c58c5ddf0eff5a587412a2656662cfc62877d
- Source checkpoint: 3fddeb9a72e5b4395e8e874a628086937331af2f; tracked checkout clean.
- Main SHA256: 3ddd1a624e7dd9af0c6228a12065782cc8dadef5bddb85f31d4958458f3adf21.
- Only apps/weftforge/src/main.rs and source-only STATUS.md changed.

## Production scope

The existing patterns endpoint handler reconstructs only one raw primary
press/release pair when ordinary egui drag recognition did not own the gesture.
It requires Select, no active endpoint, enabled/unpaused canvas, no ordinary
click or active/ended drag ownership, no held button/Escape, press inside canvas
and clip, the canvas's top interaction layer, and movement beyond egui's current
click-distance threshold. The Select canvas has no interactive children; its
only interactive child is the rotation handle in the separate Transform tool.
The original point hit/owner selection, pending numeric validation and typed
MovePoint transaction are reused for boundary/internal/hole endpoints.

Snap bypass comes from raw press modifiers. Both ordinary and reconstructed
endpoint releases use the unique raw primary release coordinate, even when a
later hover moves elsewhere. Multiple release coordinates do not commit a stale
preview. Recognized drags cannot also enter the fallback. Gesture state is
cleared when consumed, including when no valid release coordinate is available.

The inspector's acting guard now retains pending drafts for an active endpoint
and uses a Select press coordinate instead of a later hover when identifying a
canvas gesture. This prevents property blur from committing before the point
transaction or invalidating an already armed endpoint. Green coverage verifies
one revision, stable IDs and one Undo/Redo for pending grain plus point movement
on both normal and coalesced routes.

No shared command, CLI/MCP adapter, schema, dependency, option, framework, solver,
or tracing change occurred. One runtime patterns caller and three test-only
layouts remain unchanged. No temporary trace47e code is included.

## Exact evidence identities

Original red session 6134 / Cargo78888 completed naturally before the pause:
exit101, 5 passed / 3 failed, 129.097s. Main input:
787fcadcbb1ceda7bb6f68d7f62ee291fa45e44b29b4a881ed9f17fb963a7a83
Raw log: e4f84567675c433072e7efb17dc3c0b3698fc41755c06910f69d9e7b6b1348b8
PAUSED-* and red.* bytes remain exact; resume verified that source and working
diff matched their immutable copies and the old Cargo PID was absent. No red
rerun occurred. This old source/test snapshot does NOT include the later
Recovery-button ownership, Escape or pending-grain variants.

Final focused green session35485 / Cargo86429: exit0, all eight test functions
passed in 7.843s including compilation. It includes the
original assertions plus those later adjacent variants. Main input:
3ddd1a624e7dd9af0c6228a12065782cc8dadef5bddb85f31d4958458f3adf21
Raw log: e9ae2205f6957ebee81e7ee89c97d1b71d4df895bd9b182acfbe36aacab9f0c0

Tests reuse existing full-Studio frame/pointer/fixture/history helpers and two
raw-event helpers. They preserve D002's event order (move, primary down, move,
move, primary up) within one input frame, with canvas-derived test coordinates
and a 13-point movement that snaps to 10 mm. They are headless regressions, not
native replay at D002's literal screenshot coordinates. Coverage includes all
three endpoint owners, stable IDs/history/redo, invalid collapsed geometry,
post-release hover for both routes, no history on snap no-op, blocked/outside/
other-control input, wrong button, multiple pairs, Escape and press Alt bypass.

Full cargo fmt --all -- --check passed; git diff --check passed before commit.
All87 source inputs stayed unchanged during green; other86 equal381c. Actual
full source maps, source copies, diffs, raw logs and JSON receipts are siblings.
These ignored files are private review inputs, not delivered public evidence.

## Stop and remaining scope

No owned Cargo job is active; all handles are terminal. All Cargo work used one
build job and this worktree's target. No broad source gates, native build, GUI,
publication, registry/report/source-map or solver work occurred in this increment.
STATUS links issue52 and separates the exact old red from final green coverage.

Root next: review this committed production path, then authorize required broad
source gates/native build and a separately bound fresh native case. Any later
source-map/report freshness and clean registry/xtask verification remain due.
E001, D001 and D002 stay frozen and are not retrospectively repaired or regraded.
The previous release fix remains valid for its original source scenarios.

Rectangle/Transform/middle-pan still use their existing Response recognition;
this endpoint fallback does not address coalescence there. Rectangle/Transform
release handling may still depend on their existing current pointer; no broader
canvas gesture acceptance or reference parity is claimed.
