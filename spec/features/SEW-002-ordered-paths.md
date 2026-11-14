# SEW-002 — Ordered sewing paths and persistent groups

Implementation contract `SEW-002-PATHS-v1`, reviewed 2026-11-14 before production
implementation. This advances ordered and M:N sewing; the full SEW-002 contract
also requires free canvas authoring, internal relationships and further reference
behavior. None is excluded from the target by this increment.

## Evidence and boundaries

The official [M:N Segment Sewing manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358238742553-M-N-Segment-Sewing-2D)
describes selecting the first collection, confirming it with Enter, then doing
the same for the second collection. The result can be edited collectively.
Relative total lengths determine proportional pairing. Delete removes the latest
pending selection and Escape abandons the gesture. The
[M:N Free Sewing manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358268690585-M-N-Free-Sewing-2D)
extends this to chosen outline/internal intervals. The
[Free Sewing manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358239854361-Free-Sewing-2D)
documents endpoint selection, a matching-length guide and numeric length entry.
These are documentation observations, not new runtime results.

The actual reference executable/build remains the recorded 2026.1.237/r58920.
On this date its empty workspace and Rectangle tool are observed after closing
WELCOME. The separately declared original 2-to-3 sewing scenario has not yet
established group execution or continuous pairing. Later observations must be
recorded separately; do not silently rewrite this contract around their outcome.
Baseline preferences/locale and complete reference equivalence remain open.

The representation, validation, migration and precision rules below are
independent WeftForge requirements. They do not describe recovered proprietary
implementation. No proprietary source, disassembly, presets or sample garments
are implementation inputs.

## Authoritative construction and compatibility

Keep existing `Seam` and its normalized **curve-parameter** correspondence
unchanged. In particular, never reinterpret a v1/v2 nonlinear map as arc length.
The three existing legacy seam commands and their split/join behavior remain.

Add a persistent `SewingGroup` with globally unique stable `id`, `name`, ordered
`a`/`b` vectors of `CurveInterval { curve, t0, t1 }`, and explicit monotone
`correspondence` pairs. A group is one relationship, not a collection of generated
independent seams. Vectors may traverse different pieces and disconnected
boundaries; their order and each interval's direction are authored data. No
artificial geometric connector contributes length between intervals.

Three shared commands cover creation, complete editing and removal:
`AddSewingGroup { name, a, b, correspondence }`,
`UpdateSewingGroup { group }`, and `RemoveSewingGroup { group }`.
Use the existing validated Editor/history/transaction engine. Each completed
gesture or group edit commits once; rejected commands preserve construction,
revision, history/Redo and recoverable file state. Reverse B by reversing its
interval list and swapping every t0/t1, while retaining the correspondence.
Two such reversals restore every group field exactly.

Native project version 3 explicitly stores both `seams` and `sewing_groups`.
Strict v1/v2 readers migrate with an empty group vector, preserving all legacy
fields, IDs, ordering, revision, allocator, f64 values and t-map semantics.
Missing required v3 fields, duplicate/unknown fields and unsupported versions
remain errors. Do not add permissive defaults to hide missing v3 construction.
Keep historical v1/v2 fixtures/evidence unchanged. Save/recovery still uses the
existing bounded atomic writer and export protections.

## New-group physical metric

For each oriented interval i, let Li be the integral of the authored curve's
speed between its endpoints in metres. Its group domain is the cumulative Li
range divided by the side's total length. Within that range, invert **oriented
arc length** to resolve curve t. Gaps between spans have zero length. The stored
piecewise-linear correspondence maps normalized total A length to normalized
total B length. Its default `[0,0],[1,1]` permits deliberate total-length easing;
unequal lengths are not an invalid configuration.

Partial anchors remain explicit authored t endpoints. Geometry edits retain
those attachments and recompute current physical lengths/pairing. This is an
explicit independent attachment rule, not an assertion that reference serialized
ratios or every shape-edit operation use the same representation.

Use f64 and a mesh-resolution-independent metric. Exact line arithmetic and
bounded cubic length/inversion must resolve length and paired authored locations
to a declared 1e-8 m absolute budget for each evaluated path. Allocate local
budgets so summing many intervals does not multiply that path budget. Endpoints
resolve to the exact stored t0/t1. Do not linearly interpolate t merely because
the cubic is collinear: nonuniform control spacing can give nonlinear speed.

Use explicit recursion/work limits and cancellation in expensive metric/meshing
work. Nonconvergence, nonfinite results, precision-collapsed positive spans or
exhausted budgets produce useful errors; never accept an approximate fallback
without its promised bound. General interval-proof or universal hardware bitwise
identity is not claimed. UI metrics/guides must be derived once for the relevant
revision in background work, not by repeating expensive integration every frame.

## Disconnected knots, meshing and previews

A disconnected path knot has two different physical locations at one cumulative
fraction. Preserve both one-sided endpoints. Split correspondence into cells at
every A span knot, inverse-mapped B span knot and authored map knot. Each open
cell has one owning interval on each side; evaluate its inclusive endpoints
through those owners. Include relevant mesh boundary samples from both sides.

Never deduplicate a span boundary in a way that loses a selected endpoint, choose
only one of two disconnected locations, blend vertices from different spans, or
invent a connecting edge. The opposite side may intentionally share its paired
knot. Derived proportional portions and metric tables are rebuildable data.

Use existing weighted boundary vertices and `Mesh::Stitch`; the stitch's seam ID
is the group ID. The existing solver receives real constraints with ordinary
contact enabled. Rest/solver state, exported meshes and metric/preview caches
stay distinct from authored groups. Cancelled/stale jobs install no partial
result and never mutate construction. Expose side lengths, mismatch, direction
and source-derived portion guides before simulation. State the metric in the UI.

## Validation and relationship-preserving edits

Each side has 1..1024 intervals; all groups together have at most 20,000 interval
references, in addition to the existing 20,000-object bound. Map size/range,
strict monotonicity, endpoints, name, ID and interval checks reuse current rules.
Require resolvable finite positive lengths. Structural validation should not
turn normal rendering or every command into repeated expensive integration.

Within one group, reject positive-length duplicate/overlapping coverage of the
same curve, both within a side and between its sides. Touching endpoints and
disjoint intervals on one curve are permitted. Different groups may deliberately
reference a common interval. No same-piece or geometric-connectivity restriction
applies. Legacy seams retain their existing distinct-curve restriction.

Splitting rewrites every affected group interval in traversal order. A reversed
crossing interval produces the right child before the left; exact split endpoints
create no zero interval. Joining uses the recovered child-to-parent parameter
transform for every occurrence. Keep resulting list entries rather than silently
merging authored span boundaries. Group ID/name/map/list order are preserved;
continuous pair locations remain within the new metric budget. Legacy split/join
geometry/map tests retain their existing tighter criteria.

Removing a piece removes the whole group if any span depends on that piece.
Report that dependent removal and preserve unrelated groups; never trim a path
and silently redistribute pairing. The whole removal remains one undoable edit.
This is an independent deletion policy, not an observed reference claim.

## Interaction and adapters

Provide the documented two-stage Enter workflow for selecting ordered segment
collections, with visible pending sides/directions, Delete-last and Escape
cancel. Keep the draft outside the document. Enter with an empty side or invalid
combination must not create a partial group. Switching away or loading a document
must explicitly discard/reset the pending gesture without committing it.

Expose whole-group editing, B reversal and removal. The existing precise
interval authoring and shared commands must support partial/cubic paths and
explicit nonlinear easing, not only a whole-edge demonstration. Show group
identity and metric distinctly from legacy parameter-based seams.

CLI inspection and generic authoring, MCP closed command schemas and native UI
must share the same commands and errors. Include all group spans in 2D guides
and linked relationship diagnostics. SVG/DXF remain boundary-only; OBJ remains
mesh-only. These formats do not preserve groups and must not overwrite native
construction. Update the format matrix, commands, schema examples and migration
documentation with the actual v3 behavior.

## Independent acceptance

Freeze original fixtures and the evaluator before inspecting candidate outputs.
At minimum test true disconnected 2:3 sides, reversed and partial intervals,
nonidentity maps, asymmetric curved and collinear nonuniform cubics, and coarse
versus fine meshes. An exact line case uses A lengths 0.1/0.3 m and B total
0.6 m: A's first knot pairs with B length 0.15 m. Preserve both physical endpoints
at disconnected knots. Separate authored metric accuracy from weighted rest-mesh
chord approximation; a mesh approximation cannot redefine the authoring answer.

Independently integrate/invert the original cubic rather than invoking production
helpers to generate expected points. Check lengths/locations within 1e-8 m for
new paths, and exact legacy f64/map preservation. Test stable group identity and
continuous pairing through split/join/remesh, one-step create/edit/history,
reversal twice, cancellation, save/reopen and whole-group dependent removal.

Reject malformed, overlapping, empty/oversized and exhausted-work cases without
losing old construction/history/destinations. Exercise the actual native gesture
and group edit/reopen workflow separately from in-process UI tests. Retain all
failures and unknown reference behavior. A source test, passing mesh generation,
small simulation or screenshot cannot establish the full sewing contract,
physical gathering quality, internal sewing, current reference parity or an
accepted roadmap gate.
