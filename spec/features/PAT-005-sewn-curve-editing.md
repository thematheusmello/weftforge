# PAT-002 / PAT-005 — selected curves and sewing-preserving split/join

This is an independent WeftForge implementation contract. Reference interaction,
defaults and post-split/join sewing behavior remain unobserved. No implementation
check establishes reference parity. The whole garment-studio target remains
open; this increment covers numeric boundary controls and bounded split/join.

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
Internal lines, attachments, general curve fitting/merging and authoring
correspondence breakpoints in the native inspector remain outside this increment. The documented
parameter-versus-arc-length discrepancy in issue #9 remains unresolved.

## Shape-preserving join

`JoinCurve { curve }` joins that boundary to the next in its piece's oriented
chain, including last-to-first. At least three curves must remain. The selected
curve ID and its start point survive; its end becomes the next curve's end. The
next curve and shared point retire, without allocating or reusing IDs. Other
point references make the operation fail. Unaffected construction, pieces,
placement, grain and material IDs/values survive. Native **Join with next
boundary** includes pending inspector changes and the join in one typed
command/transaction, one revision and one undo step. It uses the existing worker
cancellation, mesh invalidation and fresh selection/draft handling.
Rejection retains the pending draft, numeric edit text and input focus as well
as the accepted document/history. A successful join clears the prior numeric
edit buffer and focus because the retained ID now has parent control values;
later Tab/blur must not replay the child's old text.
Focus may transfer during rendering (including Tab/AX Focus and Join in the
same input frame). Success clears both the remembered input and the numeric
input actually focused at the command callback. Rejection retains the current
input, or the remembered input if the pointer gesture already released focus.
Suppress the superseded inputs' delayed blur signals through the following UI
frame; actual value changes/drags and a later Tab from the retained input still
commit normally. A rejected join must not commit its pending draft on idle.

Only two straight curves or two compatible cubic children are supported. For
straight curves, recover `t` as left length / combined length, and check that the
shared point equals the candidate line at `t` within `1e-12 m` Euclidean distance.
For cubic children `L=[A,C,D,M]` and `R=[M,E,F,B]`, the common-endpoint derivative
vectors, omitting common factorial factors, are:

- First order: `M-D` and `E-M`.
- Second order: `M-2D+C` and `F-2E+M`.
- Third order: `M-3D+3C-A` and `B-3F+3E-M`.

If both vectors of order `k` have norm greater than `1e-12 m`, their norms give
`t = norm(L)^(1/k) / (norm(L)^(1/k) + norm(R)^(1/k))`. This follows because
restriction scales derivatives by `t^k` and `(1-t)^k`. Higher derivatives permit
stationary joins. Each candidate must satisfy `1e-6 < t < 1-1e-6`, matching the
split command's supported range. Reconstruct the parent controls as
`P1=A+(C-A)/t` and `P2=B+(F-B)/(1-t)`.

Subdivide that candidate at `t` with de Casteljau. Bound the Euclidean norm of
every difference between corresponding original/reconstructed child controls,
including endpoints. The Bernstein weights are
nonnegative and sum to one, so the largest control difference bounds the error
at **every** child parameter; finite point samples do not determine acceptance.
That geometric bound alone is insufficient for f64 evaluation and remapping.
Let `P` be the maximum Euclidean norm of any original or candidate-parent
control point (including endpoints). Let `T` be the Euclidean norm of the
owning piece's unchanged 3D placement translation. For each rotation angle,
the actual f64 sine/cosine coefficients give the rotation block an operator
norm at most `max(1,hypot(sin(angle),cos(angle)))`; their product is `K`.
Norms, products and sums used for these bounds round upward. This accounts for
the actual placement coefficients without assuming exact orthogonality.

Reserve `256 * f64::EPSILON * S` metres, with `S=K*P+T`. Require each control
difference multiplied by `K`, **plus** that reserve, to be at most the unchanged
`1e-12 m` limit; round the final product and sum upward. The same bound applies
to the straight shared-point difference. This preserves the physical point-pair
guarantee after `Placement::transform`, as well as the pattern-space bound.
Large local coordinates **or placement translations** can exhaust the precision
budget even for otherwise exact joins; they are conservatively rejected.

With IEEE unit roundoff `u=EPSILON/2`, the reserve is `512*u*S`. The loose
allocation covers both Bernstein evaluations (`64*u*S`), three levels of
de Casteljau arithmetic (`64*u*S`), old/new interval and child-parameter affine
arithmetic (`128*u*S`, using the transformed-curve derivative bound
`|C'| <= 6*K*P <= 6*S`), and
control-difference/norm comparison (`32*u*S`). Both placement transforms add
`64*u*S` for their rotation multiplies/adds and final translation additions.
Their sum is below the reserve, leaving room for higher-order rounding terms.
Candidate parameter reconstruction
need not be exact: the geometric bound is checked using the actual recovered
f64 parameter and actual stored parent controls.

Candidates that pass must agree on `t` within `1e-12`, or the join is ambiguous
and fails. No surviving candidate, mixed straight/cubic representation,
degenerate geometry, corners and shape changes fail without fitting or snapping.
These numerical limits are a bounded inverse-construction contract, not general
reference join semantics.

For **each** seam side on the first curve, map its interval endpoints by `u ->
t*u`; for each side on the next curve, map by `u -> t+(1-t)*u` and replace that
curve ID with the retained ID. Preserve the seam ID, name, correspondence and
record order. An affine map commutes with interval interpolation, so unchanged
normalized correspondence preserves the full physical point-pair locus for
partial, reversed and gathered sewing on either child, within the geometric
bound. Separate seam records stay separate. A resulting self-seam or interval
separation at/below `1e-8` fails existing validation atomically. Every failed
join retains the old document and undo/redo, including all commands in a pending
inspector transaction. The v2 native schema remains unchanged.

## Reproduction and acceptance boundary

CLI `inspect` exposes point and curve IDs/values under `construction`.
`curve-controls INPUT OUTPUT CURVE_ID X1 Y1 X2 Y2`,
`curve-controls INPUT OUTPUT CURVE_ID straight`,
`split INPUT OUTPUT CURVE_ID T`, and `join INPUT OUTPUT CURVE_ID` execute the
same commands as the native editor. MCP `project_execute` advertises the same
`join_curve` command and its single `curve` field in `tools/list`.
No failed edit writes over its previous output. Existing v2 save/reopen preserves
the exact authored fields; no file-format change is needed.

Before comparing results, the implementation tests declare a `1e-12 m` absolute
tolerance for analytical cubic and sewn point-pair evaluation and `1e-12` for
normalized parameters. They test either seam side, all interval direction
combinations, nonlinear gathered maps, exact breakpoints/endpoints, multiple
seams, changed-resolution positive meshes, history/ID branching, failed split
rollback and v2 save/reopen. Native draft tests verify one numeric gesture's
revision/history behavior, fresh selected identity and useful validation errors.
CLI subprocess tests check the controls, split and join commands and output
preservation. Join checks independently restrict analytic cubics in a power
basis, reconstruct regular and stationary children (including the closing
boundary pair), and compare full control bounds. They check both children and
seam sides, every orientation combination, gathered breakpoints, stable IDs,
v2 reopen and two mesh resolutions. A control perturbation invisible at the
endpoints/midpoint is rejected; mixed controls, corners, ambiguous/extreme
parameter recovery, self-seams, compressed intervals and failing transactions
preserve history. An in-process egui button
test checks a join with pending inspector changes commits once, invalidates a
worker, and preserves document/redo on rejection. It is source evidence, not an
observed desktop run.
Additional regressions retain the concrete high-coordinate case where the
geometric guard passes but a child's near-end f64 evaluation exceeds `1e-12 m`,
and exercise focused numeric text, pending drafts, AX/pointer Join, Tab and
undo/redo through the full in-process app UI.
They also retain a 1000 m placement-translation witness where the pattern-space
guard passes but transformed points exceed `1e-12 m`, compare rotated/translated
sewn point pairs, and exercise coalesced Tab/AX Focus plus Join in one UI frame.

The broader native curve-editing scenario below remains open on the current
source; it includes branches beyond the later bounded join run. Start
empty; create two 0.4 by 0.6 m rectangles at X=0 and X=1. Select the first top
boundary, enable cubic controls and enter P1=(0.3,0.8), P2=(0.1,0.8) m. Sew it to
the other top boundary with reversed orientation. Split at t=0.4 and inspect the
new boundary and two seam records. One undo restores the single sewn cubic; redo
restores the split. Change resolution 0.09 to 0.035 m, save, close, reopen and
verify control geometry, IDs and seam intervals. Capture actual errors for an
invalid split and an invalid control edit. Record source/executable hashes and
observed outcomes; retain failures. This procedure is not a claimed completed run.
For join acceptance, select the first split child and **Join with next boundary**.
The original selected curve ID/control shape should return while both sewing
records survive with their remapped intervals. Undo restores both children and
their intervals; redo restores the join. Save/reopen and change mesh resolution;
then attempt a corner join and retain the reported error/unchanged document.
The separately declared join subset was subsequently executed on source
`26dd585`, native `9c8b0936`, and passed its independent saved-output checks.
[WF-20261114-NATIVE-JOIN-001](../../parity/reports/WF-20261114-NATIVE-JOIN-001.json)
records focused child input, Join, one Undo/Redo, refinement, save/process restart/
reopen and rejected corner geometry. That run did not repeat the broader
invalid-split/control branches or split Undo/Redo above. Native large-placement
and coalesced-event cases remain outside its executed scope.

Historical reports keep their IDs, tested commits, source/artifact hashes and
outcomes. Their prior freshness flags remain stale after this increment changes
the native/CLI/document scopes. New grain-authoring reports must be reviewed
for the same scope changes before integration; this increment does not rewrite
those reports or fabricate native evidence.
