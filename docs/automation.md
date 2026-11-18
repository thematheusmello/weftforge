# Local automation

`weftforge-cli mcp` exposes a bounded, synchronous file-authoring toolset over
MCP stdio. It calls the same `Project`, `Editor`, and typed `Command` operations
as the desktop. It does not connect to an open desktop document. This is a first
integrated subset of automation, not whole-product or reference parity.

The CLI simulation commands (`simulate`, `benchmark`, `fit`, `fit-benchmark`)
report `diagnostics.peak_strain`: current zero-based solver triangle, `warp` or
`weft`, and signed dimensionless length strain. `peak_strain_piece` resolves the
current editable pattern's `id` and `name`; it is null when attribution cannot
be resolved. Fitting reports include this at each phase and in the final result.
The strain, seam and penetration limits remain unchanged. Contact error
includes static triangle/capsule occupancy. The additive
`diagnostics.capsule_surface_uncertain_pairs` counts pairs using conservative
uncertain-geometry bounds; legacy diagnostic JSON defaults it to zero.
`fit` and `fit-benchmark` also require this count to be zero, so uncertain
geometry cannot produce a successful fit verdict. The additive
`diagnostics.initial_contact_recovery_projections` counts initial-placement repair
projections that actually moved nodes, including repeated visits. It persists
through accepted steps until simulation reset; failed or cancelled steps preserve
the prior count. Repair motion contributes to velocity and does not certify
physical convergence. Legacy JSON defaults an absent field to zero for decoding;
this does not establish historical zero-repair evidence.

`fit` and `fit-benchmark` check the initial state and every successfully completed
whole-step state: finite diagnostics, finite nonnegative assembly target, strain
below 0.15, contact error below 0.002 m, zero uncertain pairs and zero repair
projections. When the assembly target is exactly zero, seam error must be below
0.005 m; the final assembly target must be zero. The report's
`quality_states_checked` includes the initial state and
`first_quality_failure_state` is its zero-based first failing index, or null.
Reports state this scope in `criteria`. A transient quality failure remains a
failure even after a valid final frame, but does not truncate the 60 assembly
and 120 drape steps. A solver error ends the protocol. `fit` preserves any prior
output on failure. Internal substep states are not independently certified.

The native inspector displays both counts. These are CLI reports; the MCP file tools below do not run simulation
jobs and their authoring schemas are unchanged.

Build with `cargo build --release -p weftforge-cli`. A client that launches MCP
servers can use this configuration when the executable is on its PATH:

```json
{
  "mcpServers": {
    "weftforge": { "command": "weftforge-cli", "args": ["mcp"] }
  }
}
```

There are no listeners, network requests, accounts, credentials, or extra
runtime dependencies. Relative file paths resolve from the process working
directory; absolute paths also work. File access uses that process's ordinary OS
permissions. Choose a new output path when reviewing an edit before replacing
its input. Concurrent desktop/file writers are not coordinated by this adapter.

## Supported protocol profiles

The official specifications were checked on 2026-11-13. The adapter supports the
current **2026-07-28** request metadata profile and the **2025-11-25** legacy
initialization profile. They share the nine tools below. Only `tools: {}` is
advertised; no resources, prompts, subscriptions, tasks, logging, sampling, or
elicitation capabilities are advertised.

In the current profile each request supplies its own version and client
capabilities in `params._meta`. No initialization is required and previous
requests do not supply missing metadata. `server/discover` returns supported
versions, capabilities, server identity, and zero-duration public cache hints.
Successful current-profile results include `resultType: "complete"`.
The [current versioning specification](https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning)
defines this profile; the [schema reference](https://modelcontextprotocol.io/specification/2026-07-28/schema)
defines the discovery, metadata, tool, and result fields.

```json
{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}
```

Unsupported per-request protocol versions return `-32022` with `requested` and
`supported` in error data. Missing required current metadata returns `-32602`.
Discovery and version errors advertise only `2026-07-28` for per-request use.
The separately supported legacy version uses the handshake below.

In the legacy profile, send `initialize`, read its response, then send
`notifications/initialized` before using tools. The server answers with
`2025-11-25`, including when the requested revision is unsupported, following
[legacy version negotiation](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle).
The client must disconnect if it cannot use the returned revision. Legacy
`ping` returns an empty object, including before initialization. Ping is absent
from the current protocol; a current-profile ping is an unknown method.

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"local-client","version":"1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
```

The [current stdio specification](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio)
describes dual-era clients: probe `server/discover` with current metadata; a
discovery result selects the current profile, a recognized version error selects
one of the supported revisions, and other errors/timeouts permit fallback to
legacy initialization. This server handles both profiles in the same process.

## Tools and command format

`tools/list` returns the complete deterministic list in one page, including
JSON Schema 2020-12 input descriptions. No cursor is issued; supplied cursors
are rejected. The tool names and arguments are:

| Tool | Required arguments | Result in `structuredContent` |
| --- | --- | --- |
| `project_inspect` | `path` | Complete validated `project`, SI `units`, `up: "Y"`; includes stable points, curves, piece boundaries/internal ownership, materials, placements, grain, legacy seams, sewing groups, mannequin, resolution, revision and next ID |
| `project_lengths` | `path`, `query` | Applied `source_revision`, `length_unit: "m"`, echoed query, and unrounded authored/rest length(s); pairs include signed `mismatch_b_minus_a_m` |
| `project_create` | `output` | Saved `output` and `revision`; existing output paths are refused |
| `project_execute` | `input`, `output`, `commands` | Saved paths, new `revision`, and `commands_applied` |
| `project_import_svg` | `input`, `output`, `svg` | Added `pieces_imported`, new `revision`, `metres_per_user_unit`, preserved-origin convention and explicit construction-only `scope` |
| `project_import_dxf` | `input`, `output`, `dxf` | Added `pieces_imported`, new `revision`, `metres_per_user_unit: 0.001`, preserved WCS X/Y origin/layout convention and explicit construction-only `scope` |
| `project_export_svg` | `input`, `output` | Paths, `source_revision`, `kind: "authored_svg"`, `length_unit: "mm"`, pattern/SVG axis convention |
| `project_export_obj` | `input`, `output` | Paths, `source_revision`, `kind: "rest_obj"`, `length_unit: "m"`, `up: "Y"` |
| `project_export_dxf` | `input`, `output` | Paths, `source_revision`, `kind: "authored_dxf"`, `length_unit: "mm"`, pattern U/V to XY axes, and boundary-only `scope` |

Results include a text block containing the same JSON for older clients.
Inspection returns authoritative authored state without building a mesh.
Its `project` uses the native writer's JSON number representation, including f32
display colors and f64 authored values. For v6 files written by this version,
parsed inspection and saved-project values agree exactly without changing stored
numeric precision. Legacy-file inspection returns the migrated current model.
DXF exports authored exterior curves from projects without holes or notches, independently of mesh resolution
or 3D placement. Its [R2000 profile](../spec/features/FILE-005-dxf-pattern-export.md)
preserves straight/cubic geometry and stable piece layers. Sewing, materials,
grain, internal annotations, drape and native editing relationships require the `.weftforge` project.
Each successful execute creates a single document revision through
`Editor::execute(Command::Transaction(commands))`, then uses native atomic save
and recovery handling. Commands may refer to stable IDs returned by inspection.
An array contains 1–1024 commands; nested transactions are rejected.

Shared command objects use an adjacent tag: `type` is snake_case and `data`
contains the fields of that core command. A legacy-profile call that creates an
original rectangle is:

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"project_create","arguments":{"output":"original.weftforge"}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"project_execute","arguments":{"input":"original.weftforge","output":"rectangle.weftforge","commands":[{"type":"add_rectangle","data":{"name":"Original front","origin":[0,0],"width":0.6,"height":0.8}}]}}}
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"project_inspect","arguments":{"path":"rectangle.weftforge"}}}
```

Use the inspected piece ID in `{"type":"set_grain","data":{"piece":2,"angle":0.37}}`.
The example's newly created default project assigns its first piece ID 2;
existing projects may assign different IDs. Current-profile calls use the same
arguments and additionally include the per-request `_meta` shown above.

The supported shared commands are `import_svg_patterns`, `import_dxf_patterns`, `add_polygon`, `add_rectangle`, `add_ellipse`, `add_hole`, `remove_hole`, `add_internal_segment`, `remove_internal_curve`, `add_notch`, `update_notch`, `remove_notch`, `move_point`,
`set_curve_controls`, `split_curve`, `join_curve`, `cut_piece`, `cut_and_sew`, `offset_outline`, `resize_piece`, `transform_piece`, `set_placement`, `set_grain`,
`set_material`, `add_material`, `update_material`, `add_seam`, `update_seam`,
`remove_seam`, `add_sewing_group`, `add_sewing_by_length`, `update_sewing_group`, `remove_sewing_group`,
`remove_piece`, `set_resolution`, and `set_mannequin`.
Their exact data fields and nested types are in `tools/list`; the shared core
validation remains authoritative. Authoring uses f64 SI units, placement/grain
angles use radians, and membrane stiffness uses N/m. See
[file formats](file-formats.md) for the native representation and physical units.

`add_internal_segment` takes one owner piece ID and two metre coordinate arrays;
`remove_internal_curve` takes the annotation's stable curve ID:

```json
{"type":"add_internal_segment","data":{"piece":2,"start":[0.125,0.25],"end":[0.875,0.75]}}
{"type":"remove_internal_curve","data":{"curve":13}}
```

The default unit square uses endpoint IDs 11/12 and curve ID 13; inspect existing
projects rather than assuming IDs. Every field is required, IDs are nonzero u64s,
coordinate arrays contain exactly two finite numbers, and extra/duplicate fields
reject. Edit both endpoints with two `move_point` entries in one command array;
only the final candidate is validated and the array creates one revision. Failed
creation/removal/edit retains output and recovery, including allocator state.
Each segment must remain wholly inside its owner with the fixed conservative
clearance/certificate/work limits in the [engineering contract](../spec/features/PAT-006-internal-segments.md).
Outer edits revalidate retained annotations. Boundary split/join/controls and all
sewing commands reject internal targets. Existing read-only interval length queries
can measure them. They do not affect triangulation, stitches, cloth or mass; an
accepted edit still invalidates old derived revisions. CLI `inspect` exposes each
pattern's `boundary`, `holes`, `internal_curves` and `notches`; MCP inspection returns full construction.
These are two alternatives on the existing nine tools, with no new CLI verb.

Authored notch commands use the same validated engine and no extra tool or CLI
verb. Each piece owns its notch IDs and curve attachments. For a newly created
rectangle, with no later allocations, the first notch has ID 11:

```json
{"type":"add_notch","data":{"piece":2,"curve":7,"t":0.25,"length":0.005}}
{"type":"update_notch","data":{"notch":{"id":11,"curve":7,"t":0.75,"length":0.008}}}
{"type":"remove_notch","data":{"notch":11}}
```

Position is finite curve parameter `t` in `[0,1]`, not arc length; length is
finite total perpendicular I marker length in `[0.0001,0.1]` metres. Curve
ownership, stable IDs, usable tangent and evaluated marker coordinates validate
before publication. Update retains the owner and ID and may reattach within that
owner. Invalid input rejects the complete transaction without replacing output.
Split/join remap positions; cut/offset and deletion of an attached hole/internal
line refuse explicitly. SVG preserves physical-scale markers; DXF refuses them
before staging and OBJ remains mesh-only. See the [bounded notch contract](../spec/features/PAT-006-authored-notches.md)
for migration, numeric decisions and the unverified reference boundary.

`cut_piece` takes an owner and two distinct straight boundary addresses. Each
address uses a dimensionless curve parameter strictly between `1e-6` and
`1 - 1e-6`. For an original 0.24 by 0.36 m rectangle at the origin, this example
cuts at U = 0.09 m:

```json
{"type":"cut_piece","data":{"piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}}
```

Inspect IDs in existing projects before using the example. The result is two
ordinary editable pieces, with independent boundary points and unchanged fabric,
placement and grain. The original piece stays on the arc containing its first
vertex; the appended child is named `Cut piece #<id>`. Swapping the two addresses
preserves identity. Each cut allocates nine IDs and commits one revision. Undo
retains allocator high-water; a rejected cut preserves the old document, history
and output/recovery bytes. A later command failure rolls back the entire batch.

The chord must lie strictly inside a certified simple straight contour. Curved,
vertex, same-edge, touching, ambiguous and work-exhausted cuts refuse. Any target
legacy seam or ordered sewing group also refuses, including remote intervals.
Existing annotations move unchanged to exactly one child; crossing, near-cut or
ambiguous annotations refuse. See the [engineering contract](../spec/features/PAT-006-straight-cut.md)
for clearance and work limits. This is one schema alternative on `project_execute`
and CLI `execute`; the nine MCP tools are retained. Native v6 stores the resulting
ordinary pieces. Curved cuts, cuts through holes and target relationship
remapping remain open.

`cut_and_sew` requires the same five fields and strict bounds as `cut_piece`:

```json
{"type":"cut_and_sew","data":{"piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}}
```

It makes the same two independent pieces and appends one ordinary seam in the
same candidate and revision. For this R240 example, retained chord 18 runs bottom
to top and child chord 19 runs top to bottom. Seam 20, named `Cut seam #20`, uses
A `{curve:18,t0:0,t1:1}`, B `{curve:19,t0:1,t1:0}` and correspondence
`[[0,0],[1,1]]`, pairing coincident endpoints without easing. Ten IDs are reserved
before construction; next ID becomes 21. The command adds no material, welded
points, linked editing or simulation step. Removing seam 20 with `remove_seam`
keeps both editable pieces and their cut boundaries; it does not fill or join
them. Already sewn targets still refuse, including a child until its seam is
removed. Save/reopen preserves the ordinary seam in native v6 with no format bump.
See the [Cut & Sew engineering contract](../spec/features/SEW-001-cut-and-sew.md).
The existing `execute`/`project_execute` routes and nine tools remain unchanged.
Command/schema failures preserve primary/output/recovery bytes; on a late IO
failure the primary stays exact while recovery may advance to its valid prior
primary. This bounded behavior does not establish native or reference parity.

`offset_outline` changes a piece's actual material boundary. Its three required
fields are a stable piece ID, a positive width in metres, and an explicit direction:

```json
{"type":"offset_outline","data":{"piece":2,"width":0.01,"outward":true}}
```

`outward:true` extends the material; `false` retracts it. One application replaces
the whole exterior with ordinary editable straight/cubic construction. The piece,
fabric, grain and 3D arrangement remain attached. Preview is transient; applying
through either shared command route creates one revision, including when combined
with other commands in the same array. A 0.4 × 0.6 m rectangle extended by 10 mm
has 0.42 × 0.62 m material bounds and 0.2604 m² area.

The operation prepares the actual rest mesh and Cloth before committing. It
refuses unsupported sewing, holes or internal construction on the target,
collapse, topology changes, uncertain geometry and exhausted resource budgets.
Finite positive width is required; zero, negative, missing, mistyped, duplicate
or extra fields reject without saving. Earlier commands in a failing array also
remain unapplied. Persistent seam allowances, selected-edge offsets and retaining
the old outline as internal construction remain future work.

`add_hole` creates one straight closed hole in a hole-free straight owner;
`remove_hole` fills it by its stable Hole ID:

```json
{"type":"add_hole","data":{"piece":2,"vertices":[[0.08,0.12],[0.08,0.24],[0.16,0.24],[0.16,0.12]]}}
{"type":"remove_hole","data":{"hole":11}}
```

The example assumes an original 0.24 by 0.36 m rectangle at the origin. Inspect
existing IDs first. Vertices contain 3–4096 finite metre pairs within ±1000;
the loop must remain wholly inside the exterior with the contract's strict
clearance. Input winding is normalized while preserving the first vertex.
The new Hole ID survives corner edits and split/join, including last-to-first
joins. One hole per piece is supported; hole curves cannot be sewn, and a holed
piece cannot be cut. Existing exterior sewing remains attached.

Native v6 preserves both editable loops. SVG exports one compound evenodd path
per piece, but current SVG import refuses hole subpaths. DXF export refuses a
holed project before staging; OBJ exports its actual annular surface. See the
[hole contract](../spec/features/PAT-006-straight-hole.md). With `offset_outline`,
the shared schema has 33 nontransaction alternatives; the core enum includes
Transaction as its 34th variant. No CLI verb or MCP tool was added.

`import_svg_patterns` takes `{"svg":"...original SVG XML..."}` and adds all
supported pieces as one typed authoring action. The dedicated CLI
`import-svg INPUT OUTPUT PATTERNS.svg` and MCP `project_import_svg` file tool read
the SVG with the same 1 MiB bound, execute this shared command and save the native
output atomically. `input` and `output` may identify the same native project;
the SVG source cannot alias output or its recovery destination. A failed import
leaves the existing document and files intact under the native-save guarantees.
The [SVG profile](../spec/features/FILE-005-svg-pattern-import.md) requires physical
mm dimensions with uniform viewBox scale and direct closed absolute `M/L/C/Z`
paths. It preserves editable straight/cubic geometry, names, user-space origin
and relative layout, inverting SVG Y into upward pattern V. The response states
the scale and omitted content; imported pieces use the first fabric and default
3D placement. Unsupported content rejects the entire drawing. The inline shared
command remains subject to the command file/MCP line-size bounds, including JSON
escaping; the file tool can read the full SVG byte allowance.

`import_dxf_patterns` takes `{"dxf":"...original ASCII DXF..."}` and adds the
supported patterns atomically. CLI `import-dxf INPUT OUTPUT PATTERNS.dxf` and
MCP `project_import_dxf` read the source with a 1 MiB bound, execute the same
command and use native primary/recovery handling. Failed saves retain the output
primary; recovery may already have advanced to the valid current-primary snapshot
if final primary replacement fails. This is not an atomic two-file transaction.
The separate path guard refuses output or recovery replacement of the DXF source.
The [DXF import profile](../spec/features/FILE-005-dxf-pattern-import.md)
requires the existing R2000 millimetre `LINE`/clamped cubic `SPLINE` structure,
declared `WF_PIECE_<id>` layers and exactly closed ordered contours. X/Y maps to
U/V with origin/layout preserved and no Y inversion. Layer names become piece
names with fresh IDs; original names/IDs, sewing, materials, grain and simulation
are not restored. Pieces use the first fabric and default 3D placement. Unsupported
content fails the entire drawing; command file/MCP line-size limits still include
JSON escaping, while the dedicated file tool can read the full source allowance.

`add_ellipse` uses the same `name`, `origin`, `width`, and `height` fields as
`add_rectangle`: origin is the bounding box's minimum X/Y in metres. It creates
four editable cubic Bézier spans approximating an ellipse, with normalized
radial error below 0.001; equal dimensions approximate a circle. The CLI form is
`ellipse INPUT OUTPUT NAME X Y WIDTH HEIGHT`; the native `Add ellipse` button
and command-search action use the existing name, X/Y and W/H fields. Later
point/control edits are ordinary cubic edits, without a persistent ellipse
constraint. See the [original contract](../spec/features/PAT-003-original-ellipse-20261114.md)
for validation limits and the independent numeric criterion. Reference ellipse
semantics remain unobserved.

`transform_piece` changes one piece's 2D layout through the existing CLI `execute`
and MCP `project_execute` adapters:

```json
{"type":"transform_piece","data":{"piece":2,"translation":[0.5,-0.25],"angle":1.5707963267948966,"pivot":[0.25,0.5]}}
```

All four data fields are required. Translation and pivot are authored U/V metres;
the pivot is in the original coordinates. Angle is counterclockwise radians in
the V-up canvas. Rotation occurs about the explicit pivot before translation.
Endpoints and absolute cubic controls move together; IDs, links and sewing maps
remain unchanged. The command compensates existing 3D placement and rotates the
UV grain direction to retain the arranged surface and world warp/weft directions.
The saved Euler placement values can consequently change. Nonzero rotation stores
canonical grain in `[-pi,pi]`; pure translation retains grain and Euler bits and
uses a single addition per changed authoring component. Inputs must be finite,
translation/pivot components within ±1000 m, and angle within ±1e6 rad. Resulting
coordinates, placement, contour and affected sewing metrics must also validate.
A rejected command leaves the entire command array and output/recovery bytes unchanged.

The layout command adds no native fields. Remeshed UVs equal the new authored
coordinates; SVG/DXF layouts and OBJ UVs change. The arranged continuous surface
is retained within the reviewed numeric limits, but a fresh revision invalidates
rest mesh, display and simulation state. Identical triangulation, solver paths or
texture placement are not promised. This is an explicit WeftForge layout policy;
reference transform behavior remains unobserved. See the
[rigid-transform contract](../spec/features/PAT-004-piece-rigid-transform.md).

`join_curve` takes `{"curve":CURVE_ID}` and joins that oriented boundary with
its next boundary, including the closing pair. It retains the selected ID and
sewing records; incompatible geometry or unsafe remapping fails atomically.
The [bounded join contract](../spec/features/PAT-005-sewn-curve-editing.md#shape-preserving-join)
defines its geometry and parameter limits.

## Ordered sewing and headless inspection

Authored length queries read editable construction directly, without a rest mesh,
simulation, editor/history changes, save or recovery writes. Full curves are
intervals from 0 to 1; reversed endpoints retain their supplied order in the echo
and return the same positive length. Legacy seam and ordered-group pairs retain
authored A/B order, including ordered disconnected spans; gaps add no length and
correspondence does not change either side's raw length.

```sh
weftforge-cli lengths original.weftforge interval 7 0.2 0.8
weftforge-cli lengths original.weftforge seam 23
weftforge-cli lengths original.weftforge sewing-group 42
```

The shared MCP query uses closed `type`/`data` variants. For example:

```json
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"project_lengths","arguments":{"path":"original.weftforge","query":{"type":"interval","data":{"curve":7,"t0":0.2,"t1":0.8}}}}}
{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"project_lengths","arguments":{"path":"original.weftforge","query":{"type":"seam","data":{"id":23}}}}}
{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"project_lengths","arguments":{"path":"original.weftforge","query":{"type":"sewing_group","data":{"id":42}}}}}
```

These are legacy-profile requests after initialization; current-profile requests
also include per-request metadata. CLI JSON, MCP structured content and MCP text
JSON return the same report for the same saved revision/query. An interval report
contains `source_revision`, `length_unit: "m"`, `query` and `length_m`. A pair report
contains the same revision/unit/query fields, `lengths_m: [A, B]`, and
`mismatch_b_minus_a_m`, computed directly as B − A. JSON retains SI f64 precision.
Values do not depend on mesh resolution, 3D placement or current cloth positions.
The desktop presents applied values separately from drafts/previews, as
millimetres to three decimal places with a signed B − A value.

Unknown IDs, missing/unknown/duplicate/null fields, nonfinite or out-of-range
endpoints, and endpoint differences of at most `1e-8` fail explicitly. Existing
bounded metric precision/work failures are preserved; no sampled fallback or
previous result is returned as a new success. Curved legacy `inspect.lengths_m`
now uses the same accurate authored metric and propagates errors. These reports
add no angle, perimeter, warning threshold, ratio or persistent measurement.
They are implementation feedback under the
[authored-length contract](../spec/features/PAT-007-authored-length-feedback.md),
with reference numerical/rounding/selection behavior still unobserved.

`add_sewing_group` takes `name`, ordered interval arrays `a`/`b`, and a monotone
`correspondence`. `update_sewing_group` takes the complete `group` object;
`remove_sewing_group` takes its stable `group` ID. Each interval stores
`curve`, `t0` and `t1`, but a new group's map relates cumulative authored arc
lengths. Legacy seam maps retain their original normalized-t meaning. Different
pieces/disconnected spans are supported; gaps never add sewing length. Group
identity and map survive supported split/join/remesh operations. The native
writer saves v6; strict v1/v2/v3/v4/v5 readers preserve legacy data and semantics.

`add_sewing_by_length` creates one ordinary sewing group with one interval per
side and the normalized physical arc-length map `[[0,0],[1,1]]`:

```json
{"type":"add_sewing_by_length","data":{"name":"Free sewing","a":{"curve":7,"start_t":0,"direction":"forward","length_m":0.1},"b":{"curve":16,"start_t":0.75,"direction":"reverse","length_m":0.1}}}
```

Use inspected curve IDs. Each side requires finite `start_t` in `[0,1]`, closed
`direction` (`forward` increases authored t; `reverse` decreases it), and positive
finite `length_m` in SI metres. Both sides resolve against the same candidate
construction before an ID is allocated. An exact full-remaining-length request
retains the exact curve endpoint; excess length is rejected without clamping,
wrapping or traversing another boundary. Missing curves, collapsed spans,
overlapping same-curve coverage, invalid names and existing metric precision/work
errors reject the whole transaction and preserve existing output/recovery bytes.
Disjoint spans on one curve remain valid. Different side lengths use the existing
group's normalized proportional semantics. Only resolved t intervals are saved;
later geometry edits retain those attachments and recompute lengths, without
persistent length locks. Native v6, CLI `execute`, MCP `project_execute` and the
nine-tool surface stay unchanged. This is bounded WeftForge engineering behavior;
reference easing, continuous correspondence and numeric parity remain unqualified.
See the [physical-length contract](../spec/features/SEW-002-physical-length.md).

The same atomic command array can be supplied without an MCP client:

```sh
weftforge-cli execute input.weftforge output.weftforge commands.json
weftforge-cli inspect-sewing output.weftforge
```

`commands.json` is a strict JSON array of 1–1024 shared commands, at most 1 MiB.
Nested transactions and duplicate fields are rejected. CLI and MCP use the same
transaction/save function; a failed command batch never partially replaces its
output. Inspection reports the authoritative native-v6 source, group lengths,
source-owned proportional portions, rest vertices/UVs/triangles, boundary
parameters and actual weighted stitches. It builds a rest mesh and performs no
simulation. Its native source uses native scalar serialization, including f32
appearance colors; geometry and physical values retain f64 precision. This is
separate from the MCP `project_inspect` tool, which does not build a mesh.

## Failure and scope boundaries

Input is UTF-8 newline-delimited JSON-RPC, one object per line, at most 1 MiB
including its newline. Oversize lines are drained before processing the next
line. Complete final JSON at EOF is also processed. Closing stdin exits cleanly.
Stdout contains only protocol messages. IDs must be strings or signed/unsigned
64-bit JSON integers; null, fractional, and boolean IDs are rejected.

Malformed JSON returns `-32700`; invalid envelopes return `-32600`; unknown
methods return `-32601`; invalid method parameters and unknown tools return
`-32602`. Notifications produce no response. Unknown fields in requests, method
parameters, tool arguments, and shared commands are rejected. MCP metadata and
capability extension objects remain extensible. Native files and typed commands
are parsed from their original bytes so duplicate fields are not erased by a
JSON value conversion.

Tool validation, load, execution, mesh, and write failures return `isError: true`
with an error string in both text and structured content. The server remains
usable after ordinary failures. An invalid command or partially failing command
array writes nothing. Invalid/unsupported native input is never silently
recovered. Native saves retain existing invalid project output. Existing
symbolic links, including dangling links, and other nonregular destinations
are refused before staging or changing recovery files.
Successful replacement of a valid native output retains its previous bytes in
`<output>.recovery`. Filesystem durability has the limits described in
[native saves](file-formats.md#validation-and-saves).

Exports reject source aliases, native project/autosave/recovery destination
names case-insensitively, resolved symlink target names, and existing valid
native projects at any extension. The shared core writers enforce these guards
for CLI and desktop exports too. Ordinary non-native export files can be
replaced atomically. SVG preserves authored boundary curves at mm scale; OBJ
contains the rebuilt arranged rest mesh, UVs, normals and piece groups in metres.
SVG and DXF omit internal annotations; OBJ emits no annotation line primitives.
Native project saving is required to preserve editable internals. These exports
are not editable native documents or saved simulation states.

There is no persistent cross-request undo, live selection/GUI control,
simulation job API, progress stream, active cancellation implementation, or
whole-product automation coverage. Requests run synchronously; cancellation
notifications read after a completed call are ignored. Long simulation and live
desktop automation remain future work. Tests exercise real stdio subprocesses,
compare authored documents with the shared editor, independently parse the
original rectangle's OBJ/SVG tokens and physical scale, and cover transaction,
protocol, duplicate-field, bounds and file-preservation failures. Those checks
establish this implementation subset only; they do not establish reference
parity or external MCP-client interoperability.
