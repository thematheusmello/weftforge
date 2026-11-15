# PAT-003 — Original numeric ellipse contract

Prospective contract, declared before implementation or evaluation. This is an
independent WeftForge primitive. Reference ellipse representation, gestures,
defaults and later-edit semantics remain unobserved; implementation checks cannot
establish reference parity.

## Authoring and numeric scope

`Command::AddEllipse { name, origin, width, height }` adds one closed editable
piece in one editor transaction. The JSON command is `add_ellipse`. Dimensions
and the two-dimensional origin use f64 SI metres. As for `AddRectangle`, `origin`
is the minimum X/Y corner of the bounding box; the opposite corner is
`origin + [width, height]`. Width and height must be finite and greater than
`1e-6` metres. Authored endpoints and absolute controls must satisfy the existing
finite coordinate limit of ±1000 metres. Names, IDs, project object counts,
contour degeneracy and boundary sampling retain the existing validation.
The native W/H fields retain their existing 0.01–5 metre control range; CLI/MCP
commands use the core limits above.

Let `rx = width/2`, `ry = height/2`, `cx = origin.x + rx`,
`cy = origin.y + ry`, and `k = 4*(sqrt(2)-1)/3`. The ordered boundary starts at
the bottom midpoint, then visits right, top and left midpoints, closing at the
bottom. The four endpoint/control polygons are:

| Span | Start | Control 1 | Control 2 | End |
| --- | --- | --- | --- | --- |
| Bottom → right | `(cx,y)` | `(cx+k*rx,y)` | `(x+w,cy-k*ry)` | `(x+w,cy)` |
| Right → top | `(x+w,cy)` | `(x+w,cy+k*ry)` | `(cx+k*rx,y+h)` | `(cx,y+h)` |
| Top → left | `(cx,y+h)` | `(cx-k*rx,y+h)` | `(x,cy+k*ry)` | `(x,cy)` |
| Left → bottom | `(x,cy)` | `(x,cy-k*ry)` | `(cx-k*rx,y)` | `(cx,y)` |

The piece uses four stable point IDs, four stable curve IDs, and the existing
default placement, first material and zero grain angle. Points and absolute
cubic controls are authoritative. Meshes are derived. Existing project version
3 can save this construction without a schema change. Later point/control edits
are ordinary cubic edits; there is no persistent ellipse constraint.

The exact authored axis-aligned bounds are the declared box, subject to f64
rounding. Adjacent spans share endpoints and equal endpoint tangent vectors.
This is a four-cubic approximation to a conic, including when equal dimensions
approximate a circle. It is not an exact analytic ellipse.

## Declared independent checks

Before evaluation, require normalized radial error
`abs(sqrt(((X-cx)/rx)^2 + ((Y-cy)/ry)^2) - 1) <= 0.001`.
Check independently using power-basis polynomial evaluation and/or the analytic
unit-quarter identity
`R(t)^2 - 1 = (34-24*sqrt(2))*t^2*(1-t)^2*(2*t-1)^2`.
Its interior extrema occur at `t=(3±sqrt(3))/6` and `t=1/2`; the maximum
radial error is `sqrt(1+(34-24*sqrt(2))/108)-1`, below the declared bound.
Coordinate and tangent checks use absolute tolerance `1e-12` metres on the
original metre-scale fixtures, with bounds checked against the declared extrema.

Tests must cover invalid, nonfinite, overflowing and degenerate input without
changing the project, stable-ID allocator, revision or history; one undo/redo;
save/reopen of authored cubic controls; and seams/remeshing through the existing
engine while preserving boundary IDs and authored geometry. The existing contour
area threshold (`abs(twice area) > 1e-10 m²`) and bounded sampling can reject
tiny, excessively eccentric or large contours even when dimensions exceed the
primitive minimum. Meshing retains its separate finite-area and resource limits.
Acceptance does not promise all aspect ratios or the entire coordinate box are
meshable. First failures remain visible and do not loosen the criteria.

## Required surfaces and boundaries

- Core: additive command in the existing `Editor` atomic execution path; reuse
  polygon allocation/defaults and existing cubic representation.
- CLI: `ellipse INPUT OUTPUT NAME X Y WIDTH HEIGHT`, with the rectangle's units,
  argument validation, atomic save and old-file preservation on failure; command
  help includes the new operation.
- Local MCP: typed-command execution and discovery include `add_ellipse`, strict
  fields and existing limits; create/save/inspect and failure preservation are
  exercised over stdio.
- Native: accessible `Add ellipse` control uses existing name, X/Y and W/H
  inspector fields. One activation dispatches one transaction and selects the
  created piece. Existing rectangle modal/canvas gestures retain their behavior.
- Documentation: describe approximation and independent scope in feature and
  automation docs. No generalized primitive framework, dependencies, format
  bump, physics change, exact-conic claim or reference-parity promotion.

Focused package checks belong to this increment. Full integrated gates, real
native interaction, clean-checkout registry/map review and parity status updates
remain separate acceptance work; unit harness output is not a native observation.
