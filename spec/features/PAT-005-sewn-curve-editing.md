# PAT-002 / PAT-005 — selected curves and sewing-preserving splitting

This is an independent WeftForge implementation contract. Reference interaction,
defaults and post-split sewing behavior remain unobserved. No implementation
check establishes reference parity. The whole garment-studio target remains
open; this increment covers numeric boundary controls and splitting.

## Authoring contract

Select a boundary by clicking its 2D edge or choosing its stable ID in the
inspector. Selection identifies both piece and curve, highlights the actual edge
and control locations, and shows authored endpoint positions in metres. A cubic
checkbox creates controls on the straight segment at one-third/two-thirds or
removes cubic controls to make the segment straight. Numeric controls are absolute
f64 coordinates. `SetCurveControls` commits once per numeric gesture through the
existing command/history engine. Invalid geometry retains the prior document and
shows the validation error. No cached display geometry becomes authoritative.

`SplitCurve { curve, t }` retains the existing interior parameter range
`1e-6 < t < 1 - 1e-6`. `t` is a curve parameter, not distance or normalized arc
length. Lines stay lines; cubic subdivision uses de Casteljau and preserves
physical evaluation. The original curve ID denotes the first, start-to-split
segment. A new point and second curve receive new stable IDs.

For each affected seam, use its oriented interval to locate the split in
normalized interval coordinates. Evaluate the other coordinate through the stored
strictly monotone piecewise-linear correspondence. If the split crosses the
interval, partition that map, retain its interior breakpoints, and renormalize
each portion's coordinates and both intervals. Remap the split side to local
parameters on its new curve; physical point pairs and complete seam coverage must
remain the same. Breakpoints that round onto a partition endpoint are coalesced
to retain strict monotonicity. No arc-length conversion is introduced.

The original seam ID and name survive on the first portion in the seam's oriented
correspondence order; additional portions keep the name and receive new IDs.
Reversed, partial, endpoint-touching, outside and multiple sewing intervals all
follow the same rule. Existing validation forbids a seam joining a curve to
itself; this exclusion is explicitly tested. Pieces, placements, grain, materials
and unrelated IDs are retained. The entire split is one transaction and revision
change. Failed validation retains project/history, including redo; undo restores
the relationships while reserving allocated IDs against reuse on a new branch.

The unchanged interval validator requires endpoint separation greater than
`1e-8` in each curve's local parameter. A split that would create a smaller seam
portion is rejected atomically; coverage is never dropped to force success.
Curve joining, internal lines, attachments and authoring correspondence
breakpoints in the native inspector remain outside this increment. The documented
parameter-versus-arc-length discrepancy in issue #9 remains unresolved.

## Reproduction and acceptance boundary

CLI `inspect` exposes point and curve IDs/values under `construction`.
`curve-controls INPUT OUTPUT CURVE_ID X1 Y1 X2 Y2`,
`curve-controls INPUT OUTPUT CURVE_ID straight`, and
`split INPUT OUTPUT CURVE_ID T` execute the same commands as the native editor.
No failed edit writes over its previous output. Existing v2 save/reopen preserves
the exact authored fields; no file-format change is needed.

Before comparing results, the implementation tests declare a `1e-12 m` absolute
tolerance for analytical cubic and sewn point-pair evaluation and `1e-12` for
normalized parameters. They test either seam side, all interval direction
combinations, nonlinear gathered maps, exact breakpoints/endpoints, multiple
seams, changed-resolution positive meshes, history/ID branching, failed split
rollback and v2 save/reopen. Native draft tests verify one numeric gesture's
revision/history behavior, fresh selected identity and useful validation errors.
CLI subprocess tests check both commands and output preservation.

Actual native acceptance is pending a separately identified executable. Start
empty; create two 0.4 by 0.6 m rectangles at X=0 and X=1. Select the first top
boundary, enable cubic controls and enter P1=(0.3,0.8), P2=(0.1,0.8) m. Sew it to
the other top boundary with reversed orientation. Split at t=0.4 and inspect the
new boundary and two seam records. One undo restores the single sewn cubic; redo
restores the split. Change resolution 0.09 to 0.035 m, save, close, reopen and
verify control geometry, IDs and seam intervals. Capture actual errors for an
invalid split and an invalid control edit. Record source/executable hashes and
observed outcomes; retain failures. This procedure is not a claimed completed run.

Historical reports keep their IDs, tested commits, source/artifact hashes and
outcomes. Their prior freshness flags remain stale after this increment changes
the native/CLI/document scopes. New grain-authoring reports must be reviewed
for the same scope changes before integration; this increment does not rewrite
those reports or fabricate native evidence.
