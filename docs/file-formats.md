# Project and interchange formats

This document describes the current source implementation. Platform/workflow
verification is recorded separately in [STATUS.md](../STATUS.md) and
[verification](verification.md). Reference-native compatibility has not been
established. Never infer it from the `.weftforge` extension or a mesh export.

## Capability matrix

| Format | Import/read | Export/write | Editable construction | Static/animated | Current limits |
| --- | --- | --- | --- | --- | --- |
| `.weftforge` v3 | Yes | Yes | Patterns, grain, orthotropy, legacy seams and ordered sewing groups | Static authoring | JSON, 16 MiB limit; no embedded external assets or saved simulation frames |
| `.weftforge` v2 | Yes, explicit migration | No | Legacy grain/orthotropy model and parameter-based seams | Static authoring | Fields and seam metrics preserved; saving writes v3 with an empty group list |
| `.weftforge` v1 | Yes, explicit migration | No | Original supported model | Static authoring | Old stretch maps to both axes; grain defaults to zero; saving writes v3; round-trip numeric decoding; tested native I/O subset |
| `.weftforge.recovery` | Explicit recovery | Prior valid primary on save | Same supported v1/v2/v3 model | Static authoring | One prior saved document; no silent fallback |
| `.weftforge.autosave` | Same project reader | Desktop named-document autosave | Same supported v1/v2/v3 model | Static authoring | Dirty named documents only; no automatic recovery UI |
| OBJ | No application import yet | Yes | Mesh only | Static | Thin, unwelded triangles; metre positions, normals, pattern-space UVs, piece groups; no MTL/textures |
| SVG | No | Yes | Boundary curves only in output | Static 2D | `M/L/C/Z` paths; mm physical size; no sewing/materials/internal features/print tiling |
| glTF/GLB | No | No | None yet | None yet | Planned validated avatar/mesh/animation workflows |
| DXF R2000 | No | Yes | Exterior boundary curves only | Static 2D | ASCII `LINE`/cubic `SPLINE`, mm, stable piece layers; no sewing, materials, grain, 3D placement, apparel conventions or print layout |
| Animation/cache formats | No | No | None yet | None yet | Timeline, frame caches and export not implemented |
| Marvelous Designer `.zprj` / `.zpac` | No | No | None | None | Separate FILE-006 research/compatibility workstream |

The current application has an original analytic capsule mannequin. No OBJ/glTF
avatar reader is wired into the project model; an installed parsing dependency
does not establish an import feature. Export validation with independent software
must be evidenced separately from our own writer/reader round trips.

## Native v3 representation

The UTF-8 JSON envelope is:

```json
{
  "format": "weftforge",
  "version": 3,
  "project": {
    "revision": 0,
    "points": [],
    "curves": [],
    "pieces": [],
    "materials": [],
    "seams": [],
    "sewing_groups": [],
    "mannequin": { "capsules": [] },
    "resolution": 0.07,
    "next_id": 2
  }
}
```

This structural illustration omits a valid default material; generate a valid
empty project with `weftforge-cli new FILE.weftforge`. Treat the current Rust
serialized types and validation as authoritative for v3. The reader accepts v1
and v2 through the explicit migrations below. A different format marker, unsupported
version, unknown or duplicate field, malformed JSON or invalid construction is
rejected without replacing the open project. Schemas are strict: a v1 envelope
cannot contain newer fields, v2 cannot contain sewing_groups, and v3 requires
the explicit group array. Modern materials cannot retain the old stretch field.

Authoring coordinates use f64 metres. 3D is right-handed/Y-up; pattern coordinates
are material-space U/V with V upward. Placements store metre translations and XYZ
Euler rotations in radians, applied in that order. Mass uses kilograms, time
seconds, density kg/m², membrane stiffness N/m, and bending rigidity N·m.

Points and straight/cubic curves retain stable IDs. A piece contains a name,
ordered closed boundary of curve IDs, placement, material ID and `grain_angle`.
The angle is f64 radians counterclockwise from material +U toward +V and identifies
the warp axis; weft is perpendicular. Zero points right in the V-up 2D canvas.
Placement changes the arranged orientation without changing the authored grain.
Angles must be finite and are retained without normalization. Cubic controls
are absolute 2D coordinates; a null control array means a straight boundary.
Legacy `seams` store two oriented curve intervals (`curve`, `t0`, `t1`) plus strictly
increasing normalized correspondence pairs including `[0,0]` and `[1,1]`.
Decreasing interval endpoints encode reversed direction. Derived seam samples
reference those authored relationships rather than persistent vertex indices.

Each `sewing_groups` entry stores `id`, `name`, ordered `a` and `b` arrays of
curve intervals, and an explicit correspondence map. Unlike legacy seam maps,
the group map uses normalized cumulative **authored arc length**. Disconnected
spans add no connecting length; both one-sided endpoints at a path knot remain
represented. Partial anchors still use curve t. Shape edits recompute current
lengths while preserving those anchors; split/join remap spans without replacing
the group identity or merging authored list entries. Removing a participating
piece removes the whole dependent group rather than redistributing its path.
The [ordered-path contract](../spec/features/SEW-002-ordered-paths.md) defines the
metric budget, precision errors and relationship limits. Derived portions,
metric tables and weighted mesh stitches are rebuildable and are not saved.

Materials store distinct `warp_stiffness` and `weft_stiffness` in N/m, plus the
separate shear/bend/density/damping/friction/collision values and display color.
The document defaults are 400 N/m for both axes. Color changes do not alter the
physical parameters. The mannequin stores
analytic capsule endpoints/radii. Resolution is a document-wide mesh setting.
The format does **not** yet store solver timestep/iterations, current drape or
velocities, undo history, selection/camera, pins,
attachments, holes/internal lines, external avatar assets, textures, animation,
lighting or cache data. These are visible persistence gaps on the roadmap.

## Explicit v1 and v2 migration

The version-1 schema stored `material.stretch_stiffness` and no per-piece grain.
Its reader uses the strict original field types, then maps:

```text
v3.piece.grain_angle = 0 radians (v1 input)
v3.material.warp_stiffness = v1.material.stretch_stiffness
v3.material.weft_stiffness = v1.material.stretch_stiffness
v3.sewing_groups = [] (v1 and v2 input)
```

These were the old document-to-solver inputs. The migration retains the other
stored fields, including IDs, names, geometry, seams, placement and revision.
The strict v2 reader preserves grain and distinct stiffness axes and adds only
an empty group vector. Legacy correspondence remains normalized curve t; it is
never silently reinterpreted as arc length.
A native check found two coordinate changes of 1e-16 m while reading/saving v1;
the source parser now enables round-trip numeric decoding, with independent
v1/v2 bit regression tests in [issue26](https://github.com/thematheusmello/weftforge/issues/26). Those original deltas remain below
that check's 1e-12 m geometric limit. The corrected native I/O subset now preserves188 legacy and103 modern typed
float fields exactly against an independent IEEE-bit oracle; see the
[native I/O report](../parity/reports/WF-20261114-NATIVE-IEEE-001.json). It cannot
restore values already saved after rounding, and is not complete format parity.
Loading v1/v2 does not rewrite it or create an authoring revision; the next
explicit save writes v3.
When saving over a valid v1 primary, recovery retains its exact original bytes.
Older builds cannot read v3, and there is no downgrade writer.

The four existing v1 fixtures and retained snapshots stay byte-identical in the
repository. Migration tests compare every authored value, rest arrangement/mass,
nondefault physical parameters, v2 unequal axes/nonzero grain, strict duplicate and
mixed-schema rejection, and exact old recovery bytes after modern saves. They do not replace
native interaction or reference-format compatibility evidence.

## Validation and saves

The reader caps input at 16 MiB, rejects unknown fields and validates complete
construction. Current limits include 20,000 authored objects, finite coordinates
within ±1,000 m, globally unique nonzero IDs, 256-byte names without control
characters, document resolution 0.005–0.5 m, and at most 128 mannequin capsules.
Contour, curve-chain, material, seam-interval/correspondence and reference checks
run before accepting a project. New groups permit 1–1024 intervals per side and
20,000 total interval references. Overlapping coverage within one group is
rejected; touching endpoints, disconnected spans and sharing between different
groups are supported. Bounded metric validation rejects unsupported precision
and exhausted work rather than returning an unqualified approximation.
Mesh generation separately caps 100,000 vertices
and 200,000 triangles. These are implementation limits, not reference limits.

Saving validates and serializes before replacing a file. It stages a unique file
in the target directory, flushes its data, preserves an existing valid primary as
`<filename>.recovery`, then renames the staged file at the commit point. Directory
flush is best-effort where the OS supports it. Existing invalid or non-regular
primary files are refused; use a new destination instead of destroying them.
This is not a promise of identical crash durability on every filesystem/platform.

Recovery is explicit:

```bash
cargo run --release -p weftforge-cli -- recover garment.weftforge recovered.weftforge
```

The desktop writes dirty named documents to `<filename>.autosave` every 30 seconds
and also retains session autosaves for untitled work in the platform data directory.
The Recover window lists session files and current project sidecars; recovering
creates a dirty copy requiring Save As. Autosave writes v3 and recovery uses the
same strict versioned reader;
failed recovery preserves the current document. A window-close request with
unsaved edits is blocked until save or an explicit discard action. macOS
application Quit bypasses that guard; shutdown synchronously retains the latest
valid dirty document as an explicit recovery copy (see issue #18 and
[verification](verification.md#shutdown-and-recovery-boundary)). There is no silent
fallback from a corrupt primary to an older version.

## Export semantics

OBJ uses arranged rest geometry, or current simulated positions when exporting
from an active desktop simulation. CLI `export-obj` exports the rest mesh;
`simulate PROJECT OUTPUT.obj STEPS` writes the advanced solver positions. Vertex
coordinates are metres with Y up. `vt` values retain material-space metres and
may lie outside `[0,1]`; `vn` values are computed vertex normals. Faces are
triangles and `g piece_ID` names separate pieces. No seam welding, rendered
thickness, texture images, material library, skeleton, animation, or editable
curves are encoded. OBJ itself has no authoritative unit field: its metre
convention is stated in the comment and this documentation.

SVG exports each authored exterior boundary as a named path, retaining straight
segments and cubic Beziers. One user unit is one millimetre; physical width/height
are expressed in mm and V is converted to downward SVG Y. It does not reconstruct
the complete garment document when imported elsewhere. Printer scaling, page
layout/tiling, margins and units in third-party readers still
need independent workflow validation.

DXF writes the [FILE-005-v1 R2000 profile](../spec/features/FILE-005-dxf-pattern-export.md):
ASCII `AC1015`, numeric millimetres and `$INSUNITS=4`. Pattern U/V maps to DXF X/Y
with positive V upward and Z zero; 3D placements and simulated positions do not
affect the drawing. Each straight boundary is one `LINE`; each authored cubic is
one degree-3 nonrational `SPLINE` with four controls and the clamped knot vector
`[0,0,0,0,1,1,1,1]`. Curves retain direction and shared endpoints without sampled
replacement or extra closing edges. Layers `WF_PIECE_ID` group each piece, sorted
by stable ID. User piece names are omitted. Repeated export of the same document
revision is byte-identical across destination names.

Choose **Export DXF** in the desktop toolbar or command search, or run:

```bash
weftforge-cli export-dxf garment.weftforge patterns.dxf
```

The CLI and local MCP return the source revision, millimetres and boundary-only
scope. DXF contains editable CAD curves rather than a complete native garment:
sewing/correspondence, physical materials, grain, placements, meshes/drape,
internal features, labels, seam allowances, grading and printing remain outside
this profile. ASTM/AAMA apparel interchange, import, printer scale and behavior
in other CAD consumers are separate requirements. See [verification](verification.md)
for consumer and actual desktop evidence; source support alone is insufficient.

Future formats must specify what they preserve and reject unsupported important
data explicitly. They must not accept a familiar extension without decoding it,
rename a mesh format as a native project, or silently flatten editable construction.


## Public reference research leads

The [official MD native-format table](https://support.marvelousdesigner.com/hc/en-us/articles/47358169252633-Marvelous-Designer-File-Format)
describes project/garment content roles without a complete native binary schema.
The [public API](https://developer.marvelousdesigner.com/list.html) now documents
binary CPX pattern exchange. That is a separate authoring-interchange lead;
it does not prove `.zprj` compatibility. No CPX parser or reference round trip
exists in WeftForge. Any future legacy-JSON conversion probe must use disposable
original data because the API documents replacing the current scene.

The [2D Snapshot manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358119414937-2D-Snapshot-2025-0)
documents physical-scale JPEG/PNG pattern snapshots. Printer scale and SVG/DXF
support remain separate work. See the [discovery map](../spec/features/discovery-inventory.md)
for runtime gaps, public API limitations and source provenance.
