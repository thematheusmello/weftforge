# Local automation

`weftforge-cli mcp` exposes a bounded, synchronous file-authoring toolset over
MCP stdio. It calls the same `Project`, `Editor`, and typed `Command` operations
as the desktop. It does not connect to an open desktop document. This is a first
integrated subset of automation, not whole-product or reference parity.

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
initialization profile. They share the five tools below. Only `tools: {}` is
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
| `project_inspect` | `path` | Complete validated `project`, SI `units`, `up: "Y"`; includes stable points, curves, piece boundaries, materials, placements, grain, seams, mannequin, resolution, revision and next ID |
| `project_create` | `output` | Saved `output` and `revision`; existing output paths are refused |
| `project_execute` | `input`, `output`, `commands` | Saved paths, new `revision`, and `commands_applied` |
| `project_export_svg` | `input`, `output` | Paths, `source_revision`, `kind: "authored_svg"`, `length_unit: "mm"`, pattern/SVG axis convention |
| `project_export_obj` | `input`, `output` | Paths, `source_revision`, `kind: "rest_obj"`, `length_unit: "m"`, `up: "Y"` |

Results include a text block containing the same JSON for older clients.
Inspection returns authoritative authored state without building a mesh.
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

The supported shared commands are `add_polygon`, `add_rectangle`, `move_point`,
`set_curve_controls`, `split_curve`, `resize_piece`, `set_placement`, `set_grain`,
`set_material`, `add_material`, `update_material`, `add_seam`, `update_seam`,
`remove_seam`, `remove_piece`, `set_resolution`, and `set_mannequin`.
Their exact data fields and nested types are in `tools/list`; the shared core
validation remains authoritative. Authoring uses f64 SI units, placement/grain
angles use radians, and membrane stiffness uses N/m. See
[file formats](file-formats.md) for the native representation and physical units.

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
recovered. Existing invalid project output is retained by the native saver;
successful replacement of a valid native output retains its previous bytes in
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
