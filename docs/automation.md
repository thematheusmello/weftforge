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
The existing strain, seam and penetration limits remain unchanged. Contact error
now includes static triangle/capsule occupancy. The additive
`diagnostics.capsule_surface_uncertain_pairs` counts pairs using conservative
uncertain-geometry bounds; legacy diagnostic JSON defaults it to zero.
`fit` and `fit-benchmark` also require this count to be zero, so uncertain
geometry cannot produce a successful fit verdict. The native inspector displays
the count. These are CLI reports; the MCP file tools below do not run simulation
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
initialization profile. They share the eight tools below. Only `tools: {}` is
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
| `project_inspect` | `path` | Complete validated `project`, SI `units`, `up: "Y"`; includes stable points, curves, piece boundaries, materials, placements, grain, legacy seams, sewing groups, mannequin, resolution, revision and next ID |
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
display colors and f64 authored values. For v3 files written by this version,
parsed inspection and saved-project values agree exactly without changing stored
numeric precision. Legacy-file inspection returns the migrated current model.
DXF exports authored exterior curves directly, independently of mesh resolution
or 3D placement. Its [R2000 profile](../spec/features/FILE-005-dxf-pattern-export.md)
preserves straight/cubic geometry and stable piece layers. Sewing, materials,
grain, drape and native editing relationships require the `.weftforge` project.
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

The supported shared commands are `import_svg_patterns`, `import_dxf_patterns`, `add_polygon`, `add_rectangle`, `add_ellipse`, `move_point`,
`set_curve_controls`, `split_curve`, `join_curve`, `resize_piece`, `set_placement`, `set_grain`,
`set_material`, `add_material`, `update_material`, `add_seam`, `update_seam`,
`remove_seam`, `add_sewing_group`, `update_sewing_group`, `remove_sewing_group`,
`remove_piece`, `set_resolution`, and `set_mannequin`.
Their exact data fields and nested types are in `tools/list`; the shared core
validation remains authoritative. Authoring uses f64 SI units, placement/grain
angles use radians, and membrane stiffness uses N/m. See
[file formats](file-formats.md) for the native representation and physical units.

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
`join_curve` takes `{"curve":CURVE_ID}` and joins that oriented boundary with
its next boundary, including the closing pair. It retains the selected ID and
sewing records; incompatible geometry or unsafe remapping fails atomically.
The [bounded join contract](../spec/features/PAT-005-sewn-curve-editing.md#shape-preserving-join)
defines its geometry and parameter limits.

## Ordered sewing and headless inspection

`add_sewing_group` takes `name`, ordered interval arrays `a`/`b`, and a monotone
`correspondence`. `update_sewing_group` takes the complete `group` object;
`remove_sewing_group` takes its stable `group` ID. Each interval stores
`curve`, `t0` and `t1`, but a new group's map relates cumulative authored arc
lengths. Legacy seam maps retain their original normalized-t meaning. Different
pieces/disconnected spans are supported; gaps never add sewing length. Group
identity and map survive supported split/join/remesh operations. The native
writer saves v3; strict v1/v2 readers preserve legacy data and semantics.

The same atomic command array can be supplied without an MCP client:

```sh
weftforge-cli execute input.weftforge output.weftforge commands.json
weftforge-cli inspect-sewing output.weftforge
```

`commands.json` is a strict JSON array of 1–1024 shared commands, at most 1 MiB.
Nested transactions and duplicate fields are rejected. CLI and MCP use the same
transaction/save function; a failed command batch never partially replaces its
output. Inspection reports the authoritative native-v3 source, group lengths,
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
Neither is an editable native document or a saved simulation state.

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
