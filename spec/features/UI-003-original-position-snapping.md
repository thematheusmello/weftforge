# UI-003 — Original 2D position snapping

Prospective implementation-only contract, written before product changes on
2026-11-15. This is original WeftForge behavior; no reference observation or
parity is claimed.

The native studio toolbar offers one compact inline **Snap** selector with **Off**,
**1 mm**, **5 mm**, and **10 mm**. A new studio defaults to **10 mm**, preserving
the previous 1 cm snap. The preference belongs to the current studio session,
survives project open/recovery, and is never stored in `.weftforge` or autosaves.
Changing it does not edit construction, history, selection, simulation state or
pending numeric input. Existing authoring modals disable the toolbar.

Interaction amendment, 2026-11-15: all four choices remain visible together using
the existing toolbar selectable-label pattern, with one highlighted choice and
normal keyboard focus/activation. Choosing a mode retains unfinished numeric raw
text, caret/selection and parsed draft as well as document/history. For example,
typing `1e-`, choosing a mode, then typing `2` must complete `1e-2` to 0.01 m.

Convert the pointer through the existing view-to-pattern path in SI metres.
For 1, 5 and 10 mm, independently compute each coordinate as
`round(coordinate * factor) / factor`, with factors 1000, 200 and 100 per metre.
Use f64 rounding to nearest, with exact scaled half-way values rounded away
from zero: at 10 mm, +0.005 m becomes +0.01 m and -0.005 m becomes -0.01 m.
Negative coordinates use the same origin-centred lattice. Off returns the
unrounded inverse-view coordinate, retaining existing pointer/screen precision.

Holding **Alt / Option** when starting a primary drag temporarily chooses Off
for that entire drag; releasing the modifier before release does not change its
preview or result. Capture the selected preference and modifier together when
the existing canvas recognises the drag, and retain them until release/cancel.
A polygon click or rectangle-dialog anchor click captures its current effective
preference individually, so later preference/modifier changes do not rewrite
already accepted vertices or anchors. Bypass never changes the selector.

Only the positions already snapped are covered: rectangle drag start/end,
polygon pointer vertices, dragged pattern endpoints, and the pointer anchor of
the numeric rectangle dialog. Rectangle/point previews and final commands use
the same captured rule. Each completed drag or closed polygon retains the existing
shared typed command and undo/redo boundary. Escape/tool changes abandon drag capture.

Numeric coordinates/dimensions, inspector edits, ellipse creation, curve controls,
sewing selections, hit testing, selection, scroll zoom, middle-button pan and
Frame all keep their existing behavior. Rectangle dialog dimensions remain exact
numeric millimetres converted to metres; only its clicked anchor is snapped.
The visible 0.1 m guide grid does not change with this position preference.

Focused checks will drive the native studio's egui pointer path to exercise
selector changes, snapped/bypassed geometry, retained raw text/caret and drafts,
and exact history
boundaries. A small arithmetic check covers all spacings, negative values and
half-way ties. These implementation tests establish neither actual window-level
usability nor reference parity. Workspace gates and previous acceptance cases
are outside this bounded increment.
