# PAT-009 — Authored material-outline offsets

Engineering contract for a partial implementation of **seam allowances and offsets**.
Execution and remaining gaps belong to the feature registry and individual run reports.

The first operation replaces one whole material exterior with ordinary editable
f64 lines/cubics. Its rest domain and actual Cloth mass change together. Preview,
Cancel, one-command Undo/Redo, saving/reopening, physical-scale SVG/DXF/rest OBJ
and the native/CLI/MCP routes are part of the contract. A cutting guide does not
satisfy this material operation; no persistent relationship or file-version bump
is needed for this increment.

The complete PAT-009 target also includes editable persistent seam allowances,
distinct guide/material/construction roles, per-edge and interval widths,
holes, internal/open offsets and endpoint extension, multiple offsets/reference
semantics, explicit corner choices, source-to-internal retention, relationship
remapping through editing/cutting/sewing, manufacturing output/printing and full
reference comparison. Those requirements remain visible after this increment.

The recorded reference baseline is 2026.1.237 (r58920), macOS arm64. Its offset
runtime behavior is unobserved. The current official manuals for
[Offset Pattern Outline](https://support.marvelousdesigner.com/hc/en-us/articles/47358359572121-Offset-Pattern-Outline)
and [Offset as Internal Line](https://support.marvelousdesigner.com/hc/en-us/articles/47358442871705-Offset-as-Internal-Line)
distinguish material-outline changes from internal construction. Their option
labels do not establish our mathematical join semantics, exact algorithms,
numeric limits, persistence or physical equivalence. The choices below are
independent engineering rules.

[Original preimplementation recipes](fixtures/PAT-009-offset-originals.json)
retain their exact frozen bytes; the
[reviewed additions](fixtures/PAT-009-offset-reviewed-additions.json) retain all
independent targets with documentation links adjusted for this public location.
Their planned/unexecuted fields describe the original freeze, not current run
status. The reviewed mathematical supplement's original SHA-256 is
`b5e0b8c441d0fdc595840ee13ca8d56ae113b682e7c42ae499ad4a5466c2f73a`.
No expected value comes from product output.

The current implementation charges and polls its controlled geometry,
validation, rest-mesh and Cloth preparation. The existing pinned triangulator
has a monolithic call with checks before and after; its hidden predicates are
not individually charged or polled. The strict whole-request work/cancellation
criterion below remains unmet at that boundary. No hard cancellation latency,
complete first increment or full PAT-009 acceptance follows from the current
partial implementation. Instrumenting or replacing that kernel remains due.

## Orientation, joins and the valid domain

These are WeftForge choices, not equivalences to MD corner labels. Establish the orientation/material
side of the actual authored line/cubic loop, not a display polyline. For a CCW exterior put
`v=C'(t)`, `s=||v||`, `n=(v_y,-v_x)/s`, `Q=C+d*n`, with `d=+width` for Extend, `d=-width` for Retract.
For CW traversal negate this normal. Reflection/reversal changes winding and directed tangents
together, preserving material-relative side. Establish tangent regularity and all signs with numeric
error bounds; an uncertain sign is a bounded diagnostic/refusal, never guessed left/right.

For explanation normalize a corner to CCW; let `tau=cross(T_in,T_out)`:

- `d*tau>0` opens a gap. Intersect the offset endpoint tangent lines. The intersection must have the
  forward-incoming/backward-outgoing directions appropriate to extending those spans. Ratio
  `||M-V||/abs(d)` is measured from original source vertex V: **<=4** uses the miter, **>4** joins the
  offset endpoints with a bevel. Tangent arms are explicit straight join geometry. Collinear arms
  may extend the same source line; do not merge separate source spans and lose their identities.
- `d*tau<0` overlaps adjacent loci. Trim those two **actual offset loci**, not just tangent lines,
  to one certified local intersection, retaining nonempty source intervals in source order. The fit
  covers the surviving normal locus; include trim/join endpoint uncertainty in its error budget.
- Codirected smooth tangents add no corner, including an exact smooth split. Antiparallel tangents,
  a cusp, ambiguous/multiple trim roots, lost span or unproved local join refuse. Near-collinear
  cases require enclosures rather than a hidden dimensional epsilon. No nonadjacent repair or
  arbitrary component selection is allowed.

For CCW `kappa=cross(C',C'')/s^3`, `Q'=s*(1+d*kappa)*T`. Certify `1+d*kappa>0` on every retained span;
a zero/reversal/unresolved bound refuses. Use the corresponding signed formula for CW. Certify one
simple output loop with preserved source order, no nonadjacent crossing/touch/overlap, no collapsed
span and no topology change. Extend contains the old domain; Retract lies inside it. Continuing offset
lines through collapse is invalid even when a final polygon regains positive area. A retained straight
span must have strictly positive projection along its source tangent, including extended endpoints;
a trimmed cubic retains a strictly increasing source interval. The chosen source-order branch must
remain valid for signed distances between zero and d; earlier collapse/contact cannot be ignored.
A sufficient conservative no-event certificate is valid; enumerating every intermediate width is not
required. Never flip/reorder
that failed result into acceptability. Require nonempty ordered spans and valid source-order
continuation; general topology repair remains later scope. Topology checks must cover actual cubics
using conservative enclosures/bounded refinement or an equally defensible continuous test. Polyline
intersection tests alone cannot prove an inter-sample crossing absent. Uncertainty consumes work and
then refuses explicitly.

## Between-sample approximation certificate

A regular cubic's offset is generally not cubic. Fit one or more ordinary editable cubics; do not
replace construction with a display/rest polyline. Retain a transient, ordered source-parameter map
for every surviving fitted span. This need not become a saved live relationship. Required fit error
is `epsilon=min(1e-5 m,width/64)`.

The following is a sufficient implementation-independent certificate; a sharper proved one is valid.
On cell `[a,b]`, source and candidate F use the same parameter; `h=b-a`. Bounds valid throughout the
cell are `v0>0` for source speed, `A>=||C''||`, `B>=||C'''||`, `K>=||F''||`. Then

`||n''|| <= B/v0 + 6*A*A/(v0*v0)`

`||R''|| <= M = A + abs(d)*(B/v0 + 6*A*A/(v0*v0)) + K`, where `R=Q-F`.

If e bounds both endpoint residual norms, the **whole cell** satisfies

`||Q(t)-F(t)|| <= e + M*h*h/8`.

Proof: subtract the endpoint linear interpolant of vector R and apply the second-derivative
Green-kernel bound, whose integral is at most h²/8; the interpolant norm is <=e. Differentiating
`n=J*C'/||C'||` twice and grouping C''' terms gives an orthogonal projection bounded by B/v0; the
remaining three terms are bounded by (2+1+3)*A²/v0². The onto continuous parameter correspondence
bounds both point-to-contour directions on the corresponding surviving spans. Passing finite samples
or aggregate area alone is insufficient.

Derivative Bernstein convex hulls give upper A/B/K bounds. Distance from the origin to the convex
hull of the quadratic source-derivative controls gives a safe lower speed bound. A hull containing
zero does not prove a cusp: refine/use a tighter proved bound or refuse at budget. Derivatives must
use the same parameter: if both source/candidate restrictions use local [0,1], use their scaled local
derivatives and h=1. Certificate cells need not be authored segments. Include evaluation, restriction,
join and floating-point uncertainty by outward enclosures or an audited forward-error bound. Bare
floating-point estimates with a guessed tolerance factor are not a certificate. Refuse unrepresentable
tiny widths/large translations when arithmetic uncertainty exhausts epsilon. No reference minimum or
maximum width is inferred.

Independently certify rest-mesh/OBJ boundary tessellation <=1e-5 m from the **applied authored**
boundary. Cubic chord/control-hull subdivision supplies one sufficient test. The error to original
normal locus composes to <=epsilon+1e-5 m, not a single 1e-5 bound. SVG and cubic DXF preserve authored
curves directly at physical scale. Zoom never determines geometric tolerance.

## Stable identities and transactions

Preserve piece ID/name, material identity and physical values, grain and arrangement fields; unrelated
object IDs/values remain unchanged. Match existing SplitCurve convention: keep each source curve ID on
its first nonempty fitted/trimmed descendant in source-parameter order; allocate fresh IDs for other
descendants and joins. Do not attach that ID to a join arm instead of its offset locus. Retain a source
point ID when it has exactly one corresponding join/smooth output point. A bevel replaces one source
vertex by two distinct endpoints: retire its old point ID and allocate two fresh IDs. Fit knots are
fresh. Do not merge separate collinear source spans. Transient plans record source intervals/output
roles/ID decisions; clear/remap native selections of retired geometry explicitly. Mesh indices never
become authoring identity.

Preview uses transient indices, with no permanent IDs/revision/history/file mutation. Apply rechecks
source revision/request and attached relationships, prepares the complete candidate, and commits
once. Undo restores original geometry/IDs; Redo restores the same result geometry/IDs. Existing Editor
semantics advance monotonic revision and keep next_id at its high-water mark across Undo/Redo, so an
abandoned history branch cannot recycle IDs. Do not require an old revision or smaller ID counter
from Undo. Failed/Cancelled work, in contrast, preserves the old revision, ID counter, both history
stacks, files and actual old rest/simulation state exactly.

## Numeric and resource bounds

| Limit | Reviewed policy |
| --- | --- |
| Input exterior segments | <=4,096 per request |
| Output fit/join segments | <=65,536 AND remaining existing global document-object budget |
| Offset/certification subdivision | depth <=24 |
| Request geometry work | <=16,777,216 charged units, counting each fit/certificate/subdivision task and candidate pair predicate before doing/enqueuing it |
| Pending geometry tasks | <=65,536 independently of depth |
| Cancellation | Before starting, at least every 256 charged units and before publication/commit |
| Derived target-piece vertices | <=65,536 AND existing boundary/global mesh/triangle budgets |
| Coordinates/values | Finite, existing authoring validation; finite positive width |

At reviewed production source `947bfbe4dcb74fb253d584709376ace5a08db838`, global objects <=20,000,
piece boundary samples <=20,000, mesh vertices <=100,000 and triangles <=200,000. For a replacement
simple loop of N curves/N points, let O_other count all objects except the replaced target exterior
points/curves, including retained piece/material. Require `O_other+2*N<=20,000`. The otherwise empty
one-piece/one-material case permits at most 9,999 output curves. Respect smaller remaining budgets,
existing lower rest-depth cap, coordinate/ID/revision/serialization limits. No existing cap is raised.
A depth cap alone does not bound total work. Constants are conservative engineering choices, not
reference/hardware speed promises. Record timing/hardware independently. Exhaustion/cancellation
returns a typed descriptive diagnostic/source identity and preserved old state; never truncate.
Use existing worker/cancellation mechanisms and math/dependencies, without a new configuration surface.

## Actual material integration and outputs

One shared typed operation takes stable piece ID, positive SI width and Extend/Retract. Whole exterior,
count one and material role are fixed scope. Unknown fields/enums and unsupported options refuse.
Desktop/CLI/MCP invoke the same authoritative validation/planning/command path; CLI execute and MCP
project_execute can expose it without an extra frontend. Synchronize enum/serialization, schema,
CLI help/examples, native inspector/draft/jobs, inspection outputs, fixtures and cross-route/error tests.

Detect affected seams/ordered sewing groups by curve/interval ownership before mutation. Refuse holes,
attached internal construction, linked geometry or physical attachments not conservatively retainable.
Do not claim pin/tack/link testing for record types that do not yet exist: preserve those later
requirements and all current types. Unrelated relationships survive unchanged. Deleting a blocker
through generic owner cleanup is forbidden.

Prepare the candidate's actual domain, mesh and create_cloth/real Cloth construction (or equivalent
atomic preparation through those same paths) **before** replacing authoritative state. Document-only
validation followed by a failed mesh worker is insufficient for this Apply contract. Any derive,
meshing, solver-initialization or commit failure preserves the old project/history/rest/simulation.
Successful Apply stops old simulation and clears/hides stale revision display/collision/normals/export
products until coherent current products are ready. Old/cancelled jobs never publish. Core/solver stay
GUI-free. A drape or CPU simulation step is unnecessary to measure rest mass; a **real Cloth instance**
created from the new domain is required.

Retain separate measurements with source revision:

1. Authored area from exact line/cubic integrals or independent bounded integration.
2. Actual rest area summed from triangle UV cross products, with no faces outside domain/notch.
3. Actual solver total `sum(Cloth::masses())`, plus consistency with triangle area*assigned density.
   Display text or preview area*density alone does not pass. Set existing material.areal_density
   explicitly to **0.2 kg/m²**; production default is 0.18. No thickness conversion is necessary.

Keep strict straight-fixture tolerances. For curved fixtures, authored and actual rest area each
must be within 5e-5 m² of the independent target; actual Cloth mass within 1e-5 kg; and
`abs(actual_mass-0.2*actual_rest_area)<=2e-10 kg`. Record all separately, along with resolution and
geometric bounds. Save/reopen preserves editable f64 construction/IDs/material/arrangement and rebuilds
current mesh/Cloth under the same criteria; no persisted mass or cache is needed.

SVG, supported boundary DXF and rest OBJ must succeed for accepted fixtures; unsupported remains an
unmet gate. Preserve existing atomic write/load/save behavior. SVG uses physical mm, coordinates
1,000*m with V->-Y and matching width/height/viewBox scale. A standalone one-piece rectangle must have
420x620 mm material bounds and currently tight 420x620 mm viewport; stroke width is appearance. DXF
must declare mm and retain every cubic. Measure OBJ before simulation/deformation, explicitly in metres,
removing rigid placement for intrinsic bounds/area. Do not imply an unavailable rest/current selector.

Native UI labels material change, shows old/new dimensions, area and physical mass with
pending/error states, distinguishes source/preview without color alone, and supports accessible
names/focus/keyboard Apply/Escape/Cancel. Never autocommit an abandoned offset draft on unrelated
selection or quit. Required first-phase originals are SA-001–SA-010/SA-013–SA-015 plus the reviewed
additions; SA-011/SA-012 remain later. Record the four local Cargo checks, independent native/output evidence, tracked artifacts and
clean-checkout publication checks in separate attributable run reports. This specification supplies
no product execution, reference comparison or parity credit.
