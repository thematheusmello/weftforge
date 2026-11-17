# Project and interchange formats

This document describes the current source implementation. Platform/workflow
verification is recorded separately in [STATUS.md](../STATUS.md) and
[verification](verification.md). Reference-native compatibility has not been
established. Never infer it from the `.weftforge` extension or a mesh export.

## Capability matrix

| Format | Import/read | Export/write | Editable construction | Static/animated | Current limits |
| --- | --- | --- | --- | --- | --- |
| `.weftforge` v6 | Yes | Yes | Patterns, one editable straight hole per piece, straight annotations, authored notches, grain, orthotropy, seams and ordered groups | Static authoring | Required holes/notches/ownership arrays, strict JSON, 16 MiB; no embedded external assets or saved simulation frames |
| `.weftforge` v5 | Yes, explicit migration | No | Patterns, holes, annotations, grain, orthotropy, seams and ordered groups | Static authoring | Preserves all fields; adds empty notch arrays; saving writes v6 |
| `.weftforge` v4 | Yes, explicit migration | No | Patterns, straight annotations, grain, orthotropy, seams and ordered groups | Static authoring | Preserves all fields; adds empty holes/notches arrays; saving writes v6 |
| `.weftforge` v3 | Yes, explicit migration | No | Legacy patterns, grain, orthotropy, seams and ordered groups | Static authoring | Preserves all legacy fields; adds empty internal/holes/notches arrays; saving writes v6 |
| `.weftforge` v2 | Yes, explicit migration | No | Legacy grain/orthotropy model and parameter-based seams | Static authoring | Fields and seam metrics preserved; saving writes v6 with empty group/internal/holes/notches lists |
| `.weftforge` v1 | Yes, explicit migration | No | Original supported model | Static authoring | Old stretch maps to both axes; grain defaults to zero; saving writes v6; round-trip numeric decoding; tested native I/O subset |
| `.weftforge.recovery` | Explicit recovery | Prior valid primary on save | Same supported v1–v6 model | Static authoring | One prior saved document; no silent fallback |
| `.weftforge.autosave` | Same project reader | Desktop named-document autosave | Same supported v1–v6 model | Static authoring | Dirty named documents only; no automatic recovery UI |
| OBJ | No application import yet | Yes | Mesh only | Static | Thin, unwelded triangles; metre positions, normals, pattern-space UVs, piece groups; no annotation lines or MTL/textures |
| SVG | Bounded additive import of one-contour paths | Yes, compound outlines with holes and physical notch markers | Editable exterior boundaries and names on import | Static 2D | Explicit mm and uniform viewBox; holes/notches cannot yet be reimported; sewing/materials/internal lines/print tiling unsupported |
| glTF/GLB | No | No | None yet | None yet | Planned validated avatar/mesh/animation workflows |
| DXF R2000 | Bounded additive import | Yes, projects without holes or authored notches | Editable exterior boundaries with layer names | Static 2D | Holed/notched projects refuse before staging. Strict existing ASCII `LINE`/clamped cubic `SPLINE` profile, mm and `WF_PIECE` layers; no sewing, materials, grain, internal lines, 3D placement, apparel conventions or print layout |
| Animation/cache formats | No | No | None yet | None yet | Timeline, frame caches and export not implemented |
| Marvelous Designer `.zprj` / `.zpac` | No | No | None | None | Separate FILE-006 research/compatibility workstream |

The current application has an original analytic capsule mannequin. No OBJ/glTF
avatar reader is wired into the project model; an installed parsing dependency
does not establish an import feature. Export validation with independent software
must be evidenced separately from our own writer/reader round trips.

## Native v6 representation

The UTF-8 JSON envelope is:

```json
{
  "format": "weftforge",
  "version": 6,
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
serialized types and validation as authoritative for v6. The reader accepts v1,
v2, v3, v4 and v5 through the explicit migrations below. A different format marker, unsupported
version, unknown or duplicate field, malformed JSON or invalid construction is
rejected without replacing the open project. Schemas are strict: a v1 envelope
cannot contain newer fields, v2 cannot contain sewing_groups, and v3 requires
the explicit group array. V1/v2/v3 pieces reject `internal_curves`, including an
empty array; v4 requires both the project group array and every piece's explicit
internal array, and rejects `holes`. V5 additionally requires each piece's `holes`
array, including an empty one. Hole objects require exactly `id` and `boundary`.
V6 additionally requires every piece's `notches` array. Each notch requires
exactly `id`, `curve`, `t` and `length`; v1–v5 reject that newer field.
Modern materials cannot retain the old stretch field.

Authoring coordinates use f64 metres. 3D is right-handed/Y-up; pattern coordinates
are material-space U/V with V upward. Placements store metre translations and XYZ
Euler rotations in radians, applied in that order. Mass uses kilograms, time
seconds, density kg/m², membrane stiffness N/m, and bending rigidity N·m.

Points and straight/cubic curves retain stable IDs. A piece contains a name,
ordered closed `boundary` of curve IDs, a `holes` array of stable-ID closed loops,
independent `internal_curves` ownership array, `notches`, placement, material ID and `grain_angle`.
The angle is f64 radians counterclockwise from material +U toward +V and identifies
the warp axis; weft is perpendicular. Zero points right in the V-up 2D canvas.
Placement changes the arranged orientation without changing the authored grain.
Angles must be finite; explicit grain edits and native reads retain them without
normalization. A nonzero `transform_piece` layout rotation stores the rotated
unit grain direction canonically in `[-pi,pi]`, and compensates placement to
preserve the arranged surface and world warp/weft directions. A pure translation
retains grain and Euler field bits. The command rotates endpoints and absolute
controls about its explicit original U/V pivot, then translates; saved placement
Euler values can change although arrangement stays fixed. The transform itself
adds no saved fields. Cubic controls are absolute 2D coordinates; a null control
array means a straight curve. Internal annotations and hole curves require null controls.

Each internal entry identifies one independent straight annotation with two
exclusive Point IDs. The list preserves authored order and does not create a
path, region, hole or physical relationship. Every curve has exactly one boundary
or internal owner; unrelated orphan geometry and shared internal endpoints reject.
The full segment must exceed 1e-9 m and stay strictly inside its owner's contour
with at least 1e-6 m clearance. Certified 1e-7 m curve flattening, conservative
f64 reserve, depth 20, 20,000 contour samples and a one-million-step project
validation budget are fixed; uncertain/exhausted inputs reject. Mesh resolution
cannot change that decision. See the [engineering contract](../spec/features/PAT-006-internal-segments.md).

Each notch has a globally unique stable ID and belongs to one piece-owned curve.
It stores finite curve parameter `t` in `[0,1]` and total physical marker length
in `[0.0001,0.1]` metres. The perpendicular I marker is centred on the evaluated
anchor and requires a usable analytic tangent. It is annotation only, with no
mesh/solver effect. Native geometry edits preserve or explicitly refuse its
relationship; see the [notch contract](../spec/features/PAT-006-authored-notches.md).
SVG retains physical-scale markers and includes them in its view bounds; the
bounded importer refuses notch line elements. DXF refuses authored notches before
replacing output. OBJ contains triangle geometry only.
Straight topological cuts use two ordinary pieces, independent boundary
points/curves and explicit annotation ownership. The resulting exterior chords
are ordinary editable edges. No cut object or persistent cross-cut relationship
is stored. SVG/DXF export both hole-free closed boundaries and
OBJ exports the two piece surfaces. The [straight-cut contract](../spec/features/PAT-006-straight-cut.md)
defines supported construction and refusal rules.

V5 adds one simple straight hole per simple straight owner. A Hole owns its stable
ID and an ordered closed curve chain, wound opposite the exterior. Its dedicated
points cannot be shared with another loop, piece or annotation. Whole edges must
stay inside the exterior with strict clearance; annotations remain outside the
closed hole. IDs survive corner edits, split/join, resize and transform. The
derived mesh is one annular surface with two free boundary loops, and cloth mass
uses its actual material area. SVG writes both loops as one evenodd compound path;
the current SVG importer refuses this path. The current DXF profile refuses any
holed project before staging. Curved/multiple holes, internal-shape conversion,
hole sewing and cutting through a hole remain unsupported. See the
[hole contract](../spec/features/PAT-006-straight-hole.md).
Future internal cubics, grouping and persistent cut relationships require an
explicit model decision; the reader never silently discards unsupported data.
Legacy `seams` store two oriented curve intervals (`curve`, `t0`, `t1`) plus strictly
increasing normalized correspondence pairs including `[0,0]` and `[1,1]`.
Decreasing interval endpoints encode reversed direction. Derived seam samples
reference those authored relationships rather than persistent vertex indices.

`cut_and_sew` saves exactly these ordinary piece/curve/point and seam objects;
the native representation gains no fields, command tag or migration. In the original R240 example,
seam 20 joins full retained chord 18 (`t0=0,t1=1`) to reversed child chord 19
(`t0=1,t1=0`) with `[[0,0],[1,1]]`. The two chords keep disjoint IDs even though
their endpoint coordinates coincide. Removing the seam keeps both pieces and
their boundaries; undo history and the transient native `Sew cut edges` option
are not serialized. SVG/DXF retain two exterior outlines and OBJ two independent
surface components; none of those geometry exports round-trips the seam.
See the [Cut & Sew contract](../spec/features/SEW-001-cut-and-sew.md).

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
attachments, persistent cut relationships, internal cubic shapes, external avatar assets, textures, animation,
lighting or cache data. These are visible persistence gaps on the roadmap.

## Explicit v1–v5 migration

The version-1 schema stored `material.stretch_stiffness` and no per-piece grain.
Its reader uses the strict original field types, then maps:

```text
v6.piece.grain_angle = 0 radians (v1 input)
v6.material.warp_stiffness = v1.material.stretch_stiffness
v6.material.weft_stiffness = v1.material.stretch_stiffness
v6.sewing_groups = [] (v1 and v2 input)
v6.piece.internal_curves = [] (v1, v2 and v3 input)
v6.piece.holes = [] (v1, v2, v3 and v4 input)
v6.piece.notches = [] (v1, v2, v3, v4 and v5 input)
```

These were the old document-to-solver inputs. The migration retains the other
stored fields, including IDs, names, geometry, seams, placement and revision.
Frozen v2/v3 piece types preserve grain and distinct stiffness axes and add only
empty internal arrays. V2 also gains an empty group vector; v3 retains every
ordered group/map/list entry. Legacy correspondence remains normalized curve t;
it is never silently reinterpreted as arc length.
A native check found two coordinate changes of 1e-16 m while reading/saving v1;
the source parser now enables round-trip numeric decoding, with independent
v1/v2 bit regression tests in [issue26](https://github.com/thematheusmello/weftforge/issues/26). Those original deltas remain below
that check's 1e-12 m geometric limit. The corrected native I/O subset now preserves188 legacy and103 modern typed
float fields exactly against an independent IEEE-bit oracle; see the
[native I/O report](../parity/reports/WF-20261114-NATIVE-IEEE-001.json). It cannot
restore values already saved after rounding, and is not complete format parity.
V4 retains its complete annotation/group data and gains empty holes/notches arrays.
V5 retains all supported geometry, hole, annotation and group fields and gains
only empty notch arrays. Loading v1–v5 does not rewrite it or create an authoring
revision; the next explicit save writes v6.
When saving over a valid v1 primary, recovery retains its exact original bytes.
Older builds cannot read v6, and there is no downgrade writer.

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
Validation/schema/command failure starts no replacement and preserves primary
and recovery bytes. A later IO failure can occur after recovery has been refreshed
to the valid prior primary; the primary survives, but the older recovery bytes
need not. This is not an atomic two-file transaction or a promise of identical
crash durability on every filesystem/platform. Native general Save commits an
ordinary inspector draft before chooser/IO; preservation of every uncommitted
general draft on cancellation/failure is a separate known limitation.

Recovery is explicit:

```bash
cargo run --release -p weftforge-cli -- recover garment.weftforge recovered.weftforge
```

The desktop writes dirty named documents to `<filename>.autosave` every 30 seconds
and also retains session autosaves for untitled work in the platform data directory.
The Recover window lists session files and current project sidecars; recovering
creates a dirty copy requiring Save As. Autosave writes v6 and recovery uses the
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
curves or internal annotation line primitives are encoded. Native saving retains
editable annotations. OBJ itself has no authoritative unit field: its metre
convention is stated in the comment and this documentation.

Rigid 2D `transform_piece` layout edits therefore change SVG/DXF construction
and OBJ UV coordinates while compensating placement to preserve the continuous
arranged surface within numeric limits. Remeshing may change triangulation;
identical vertex ordering, solver trajectories and texture placement are outside
this increment. Every successful edit still invalidates derived revisions.

SVG exports each authored exterior boundary as a named path, retaining straight
segments and cubic Beziers. Internal annotations are omitted. One user unit is one millimetre; physical width/height
are expressed in mm and V is converted to downward SVG Y. Nonempty SVG canvases
use the union of analytic exterior curve bounds and preserve absolute authored
U/V coordinates. It does not reconstruct
the complete garment document when imported elsewhere. Printer scaling, page
layout/tiling, margins and units in third-party readers still
need independent workflow validation.

Choose **Import SVG…** in the desktop toolbar or command search to add editable
exterior patterns, or run `weftforge-cli import-svg INPUT OUTPUT PATTERNS.svg`.
The [bounded SVG import profile](../spec/features/FILE-005-svg-pattern-import.md)
requires physical `mm` width/height and a uniformly scaled `viewBox`; it accepts
direct, closed single-contour paths using absolute `M/L/C/Z` commands. Straight
and cubic construction, names and relative layout survive as editable objects.
The drawing's user-space origin is retained, without subtracting the viewBox
minimum; SVG Y is inverted into upward pattern V. The native summary shows the
physical scale before applying all pieces and pending edits as one undo step.
Missing/other units, transforms, CSS, unsupported geometry/path commands, open
or compound contours, DTDs and external resources reject the whole import.
Import is limited to 1 MiB, 64 pieces and 4096 total spans; native validation and
its bounded cubic contour predicate still apply. Appearance, descriptions,
sewing, grain, internal details and simulation are omitted explicitly. Imported
pieces use the first existing fabric, empty internal arrays and default 3D placement. Native documents
remain necessary for the complete garment.

DXF writes the [FILE-005-v1 R2000 profile](../spec/features/FILE-005-dxf-pattern-export.md):
ASCII `AC1015`, numeric millimetres and `$INSUNITS=4`. Pattern U/V maps to DXF X/Y
with positive V upward and Z zero; 3D placements and simulated positions do not
affect the drawing. Each straight boundary is one `LINE`; each authored cubic is
one degree-3 nonrational `SPLINE` with four controls and the clamped knot vector
`[0,0,0,0,1,1,1,1]`. Curves retain direction and shared endpoints without sampled
replacement or extra closing edges. Layers `WF_PIECE_ID` group each piece, sorted
by stable ID. User piece names are omitted. Repeated export of the same document
revision is byte-identical across destination names. Internal annotations are
omitted; retain the native project to preserve them.

Choose **Export DXF** in the desktop toolbar or command search, or run:

```bash
weftforge-cli export-dxf garment.weftforge patterns.dxf
```

The CLI and local MCP return the source revision, millimetres and boundary-only
scope. DXF contains editable CAD curves rather than a complete native garment:
sewing/correspondence, physical materials, grain, placements, meshes/drape,
internal features, labels, seam allowances, grading and printing remain outside
this profile. ASTM/AAMA apparel interchange, general CAD import, printer scale and behavior
in other CAD consumers are separate requirements. See [verification](verification.md)
for consumer and actual desktop evidence; source support alone is insufficient.

Choose **Import DXF…** in the toolbar or command search, or run
`weftforge-cli import-dxf INPUT OUTPUT PATTERNS.dxf`. The
[bounded DXF import profile](../spec/features/FILE-005-dxf-pattern-import.md)
accepts only this R2000 ASCII structure with explicit millimetres and one exactly
adjacent closed ordered `LINE`/clamped cubic `SPLINE` contour per `WF_PIECE_<id>`
layer. It preserves physical X/Y as U/V without inversion, origin or layout
changes. Source layer names become piece names with fresh native IDs; original
names are absent from the export. New pieces use the first fabric, zero grain
and default 3D placement. The native review shows scale, count and scope before
one atomic import/Undo step; Cancel retains pending numeric drafts.

The reader is limited to 1 MiB, 131072 group pairs, 64 pieces and 4096 boundary
entities. Other units, versions, entities, flags, fields, nonzero Z, gaps,
unsupported metadata or block/paper geometry reject the whole drawing. Native
construction validation also applies. Rejected imports preserve the existing
authored project; CLI/MCP source-path guards protect the DXF, and failed saves
retain the existing output primary. Native saving copies the valid current
primary to recovery before replacing the primary, so recovery may already have
advanced to that valid snapshot if the final primary replacement fails. The two
files are not one atomic transaction.

On source `76be95d4d7928bcb703e1f52f4a5404bc955f567`, independent headless
checks passed four import/save/export cases, fourteen rejection/retention cases
and four exact MCP/native project comparisons. Independent native-file and
export grading also passed after the observed chooser/review cancellation,
single import, two Undo/two Redo actions, cubic-control edit, rejected source
and actual quit/relaunch/reopen sequence. The [DXF studio evidence packet](../parity/reports/artifacts/dxf-studio-20261115/README.md)
records the bounded workflow and remaining gaps: the old Redo branch after
Cancel was not invoked before the new import, the modal did not explicitly
mention printing, and temporary edits reached background autosave sidecars while
original input primaries stayed unchanged. This does not establish every frozen
native-plan criterion, full persistence acceptance, general DXF/CAD or apparel
interchange, printing or reference parity.

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
