# Observed reference inventory

This directory contains reviewed public behavior observations from the accessible
macOS 2026.1.237 (r58920) workspace. Raw commercial/account UI stays private.

[Main-menu catalog](md-2026.1.237-main-menus.json) initially recorded 17 main menus,
572 nonroot entries and 505 leaf-like nodes in an empty project. A later selected
original-seam Edit menu contributes four additional exact label paths, bringing
the observed union to **593 menu records**, including 576 nonroot entries and
509 leaf-like nodes. The
[readable outline](md-2026.1.237-main-menus.txt) uses `!` for an exposed disabled
marker. IDs derive from exact surface and label path, so insertion/reordering
does not renumber existing entries. Parent IDs preserve the observed hierarchy.

The initial 589 records retain their original state. The four later paths are
`Edit > Context Menu > 3D Garment > Flat`, the Display commands `Show 3D Pattern`
and `Hide All 3D Patterns`, and `Sewing > Apply Turned Sewing Line Type` under the
same Context Menu parent. Three initial labels are absent from that later Edit
tree. These labels remain distinct until behavior/alias review; neither new
features nor equivalent renamed commands are inferred. The union is not one
simultaneous menu state. Disabled headings, empty
submenus, duplicate entry points, context-sensitive variants and external OS
services make them unsuitable as a product-feature denominator. `Edit > Context
Menu` lists commands but does not verify a canvas right-click menu. Checked
states, shortcuts, ranges and behavior stay unknown until separately observed.

[Component catalog](md-2026.1.237-components.json) contains 356 observed toolbar,
panel, tab, property and dialog entries. The original 61 IDs remain unchanged.
The next 105 entries describe already captured Simulation Properties, selected
original-pattern fields, the rectangle constructor, OBJ export options and Fit
Properties. Their source records are pinned by hash in `observation_sources`.
The [reviewed surface notes](../features/UI-property-surfaces-20261114.md) explain
that capture context. A further 138 records cover the
[read-only preference pass](../features/UI-preferences-runtime-20261114.md),
including actual unit/display settings and configured recovery controls.
Nine further entries record the curvature/free-sewing tool labels and numeric
Create Sewing Line dialog from the [bounded original curve run](../features/SEW-002-runtime-20261114.md).
That run qualifies one curve/history operation and a 100 mm first-side preview.
The next 26 entries record Segment Sewing and the selected seam panel, including
paired numeric/slider controls and explicitly described unlabeled actions. The
[whole-edge runtime case](../features/SEW-001-runtime-20261114.md) records one
completed original pair, Undo/Redo and document reopen; the seam-property edits
and ranges remain untested.
The next 17 entries describe the original input's Open Project options: Open/Add,
content sections and checks, and Reset/OK/Cancel/Close. Open with Garment and
Display Settings checked was accepted unchanged. Gray/unchecked controls are
input-specific visual observations; no retained AX enabled flag, alternate
content, Reset, mode switch or cancellation behavior is claimed.
Ranges and most edit/error behavior remain unknown. The
[component outline](md-2026.1.237-components.txt) is a compact index of the same
records.

The combined current inventory has 949 records: 593 menu nodes and 356 component
observations. This is an expanding observation index, not a complete product
feature count. Adding a field does not complete its behavior contract.

The [source-pinned implementation map](weftforge-implementation-map.md) assigns
all 949 exact IDs while preserving the prior 928 entries, 205 bounded packets
and 56 source landmarks. Open Project and selected-seam label contexts remain
explicit; related group assignments do not establish aliases or parity.

Every record separates menu-presence evidence from the REA workspace identity
capture. No raw UI, proprietary asset or implementation technique is supplied to
implementers. Activating Rectangle is recorded as tool activation only; the
[bounded original rectangle run](../features/PAT-003-runtime-20261114.md) now
records numeric creation, one-step history, document save/close/open and physical
OBJ bounds. The [bounded resolution run](../features/PAT-011-runtime-20261114.md)
adds the 20 to 10 mm edit, one Undo/Redo, separate save/export, and target-state
reopen after process restart. Its whole-window AX gap remains explicit. The
curved first-side preview adds a numeric dialog observation. Whole-edge sewing
now has a bounded history/persistence case. The later
[paired partial case](../features/SEW-002-paired-runtime-20261114.md) accepts two
100 mm sides and observes Undo/Redo and document reopen. The separate
[reversal case](../features/SEW-002-reverse-runtime-20261114.md) switches the dashed
endpoint guides and observes Undo/Redo and document reopen. Continuous arc-length
correspondence remains unmeasured. No menu entry qualifies as verified parity
from presence alone.

Continue with numeric rectangle resize, resolution limits/cancellation and sewn
constraints, and independent continuous curve/seam measurements. The initial CUA coordinate route
failed; a pinned target-guarded native pointer fallback supports controlled
canvas input. Retain the failed attempts and remaining modal-capture/AX
limitations while testing each route. Expand the catalog with real canvas
context menus, toolbar flyouts, all panels/tabs, dialogs, shortcuts,
checked/enabled states, input limits, error/cancellation and end-to-end results.
