# PAT-003 — Rectangle creation

Reference state: **partial runtime observed**. Candidate baseline:
`MD-2026.1.237.58920-macos-arm64-20261113`, provisional revision 1. The
[controlled numeric rectangle observation](PAT-003-runtime-20261114.md) records
constructor values, creation/history, document persistence and independent OBJ
dimensions. Platform modifiers, linked resizing and broader input behavior remain
unobserved.

The [official Rectangle manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358203451673-Rectangle)
describes a 2D tool available from the pattern-creation menu and toolbar. Dragging
between corners creates a rectangular authored pattern on release. Shift
constrains a square; Ctrl changes creation to use a center; their combination
creates a centered square. A single canvas click opens numeric width/height
entry. Linking the dimensions preserves their ratio; unlinking permits separate
values. These modifier names are the manual's wording, pending macOS observation.

A conforming implementation keeps the four editable boundary segments and
numeric dimensions. A drag is one committed authoring transaction. Our native
project format and branding are intentional differences.

Original scenario `REF-PAT-003-RECTANGLE-001` is partially executed; retain the
full plan and remaining cases: start an empty project,
record units/zoom, create 200 × 300 mm through numeric entry, inspect dimensions,
then undo/redo. Repeat square, centered rectangle, centered square, linked resize,
and canceled creation. Record screenshots plus exported measurements. Choose a
0.01 mm numeric geometry tolerance before running; display rounding, accepted
range, invalid-input behavior, snapping, and cancellation semantics require
measurement and must not be inferred from this manual.
