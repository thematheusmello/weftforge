# Observed reference inventory

This directory contains reviewed public behavior observations from the accessible
macOS 2026.1.237 (r58920) workspace. Raw commercial/account UI stays private.

[Main-menu catalog](md-2026.1.237-main-menus.json) records 17 main menus, 572
nonroot entries and 505 leaf-like nodes in an empty project. The
[readable outline](md-2026.1.237-main-menus.txt) uses `!` for an exposed disabled
marker. IDs derive from exact surface and label path, so insertion/reordering
does not renumber existing entries. Parent IDs preserve the observed hierarchy.

These counts cover one exposed menu tree state. Disabled headings, empty
submenus, duplicate entry points, context-sensitive variants and external OS
services make them unsuitable as a product-feature denominator. `Edit > Context
Menu` lists commands but does not verify a canvas right-click menu. Checked
states, shortcuts, ranges and behavior stay unknown until separately observed.

[Component catalog](md-2026.1.237-components.json) adds 61 observed toolbar, panel,
tab and property entries. It records current Graphic Options and millimetre grid
controls, with ranges/error behavior still unknown.

Every record separates menu-presence evidence from the REA workspace identity
capture. No raw UI, proprietary asset or implementation technique is supplied to
implementers. Activating Rectangle is recorded as tool activation only; the
[bounded original rectangle run](../features/PAT-003-runtime-20261114.md) now
records numeric creation, one-step history, document save/close/open and physical
OBJ bounds. Particle-distance edits and curved sewing remain pending. No menu
entry qualifies as verified parity from presence alone.

Continue with numeric rectangle create/edit/Undo/Redo/save-close-reopen, controlled
particle distance, and reversed partial sewing on curves. The initial CUA coordinate route failed; a pinned target-guarded native pointer
fallback now supports controlled canvas input. Retain the failed attempts and
remaining modal-capture/AX limitations while testing each route. Expand the catalog with real
canvas context menus, toolbar flyouts, all panels/tabs, dialogs, shortcuts,
checked/enabled states, input limits, error/cancellation and end-to-end results.
