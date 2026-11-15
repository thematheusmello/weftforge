# UI-001 — frame authored pattern outlines

Original prospective contract `UI-001-pattern-framing-v1`, written before code
on 2026-11-15. This is WeftForge's own view-navigation behavior; no reference
parity is claimed.

A small **Frame all** action in the native studio's top toolbar fits every
authored exterior pattern outline into the actual current 2D canvas, centered
with 28 logical pixels of padding on each side. The toolbar placement lets the
action preserve pending inspector input before the inspector handles focus.
Use construction coordinates, including negative origins and relative piece
positions, and the existing exact cubic-extrema bounds. Do not fit control
handles, labels, grid, 3D placement, selection or pending draft geometry.

The action changes only the 2D center and pixels-per-metre scale. It must not
commit pending inspector edits, change construction/revision/IDs/undo/redo,
change selection, invalidate a mesh, queue work or run simulation. It remains
available while authoring is otherwise available. An empty project disables the
action. Empty, invalid or padding-exhausted viewports are safe no-ops. Zero-width
or zero-height bounds stay finite; point-like bounds use the existing maximum
zoom. The scroll zoom minimum also permits the full valid +/-1000 metre
construction range to remain framed; Frame all must not silently clamp a fit
upward and crop large outlines.

Acceptance is an analytic native test, independent of a window or physics:
separated negative-origin outlines and a cubic whose interior extremum exceeds
its endpoints must lie inside the inset canvas at normal and minimum native
window sizes. Assert finite positive view state, retained pending draft and
selection, exact document/history preservation, safe degenerate/empty cases,
and a fit covering the full allowed coordinate range. This automated evidence
cannot establish actual native button usability or reference parity. Root-owned
combined checks and native acceptance remain separate pending work. Command
search, saved camera state and new authoring commands are outside this increment.
