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

[Component catalog](md-2026.1.237-components.json) contains 166 observed toolbar,
panel, tab, property and dialog entries. The original 61 IDs remain unchanged.
The next 105 entries describe already captured Simulation Properties, selected
original-pattern fields, the rectangle constructor, OBJ export options and Fit
Properties. Their source records are pinned by hash in `observation_sources`.
The [reviewed surface notes](../features/UI-property-surfaces-20261114.md) explain
the capture context and remaining gaps. Ranges and most edit/error behavior
remain unknown.

The combined current inventory has 755 records: 589 menu nodes and 166 component
observations. This is an expanding observation index, not a complete product
feature count. Adding a field does not complete its behavior contract.

Every record separates menu-presence evidence from the REA workspace identity
capture. No raw UI, proprietary asset or implementation technique is supplied to
implementers. Activating Rectangle is recorded as tool activation only; the
[bounded original rectangle run](../features/PAT-003-runtime-20261114.md) now
records numeric creation, one-step history, document save/close/open and physical
OBJ bounds. Particle-distance edits and curved sewing remain pending. No menu
entry qualifies as verified parity from presence alone.

Continue with numeric rectangle resize, controlled particle distance, process
restart, and reversed partial sewing on curves. The initial CUA coordinate route
failed; a pinned target-guarded native pointer fallback supports controlled
canvas input. Retain the failed attempts and remaining modal-capture/AX
limitations while testing each route. Expand the catalog with real canvas
context menus, toolbar flyouts, all panels/tabs, dialogs, shortcuts,
checked/enabled states, input limits, error/cancellation and end-to-end results.
