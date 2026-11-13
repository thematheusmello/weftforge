# Project and interchange formats

This document describes the current source implementation. Platform/workflow
verification is recorded separately in [STATUS.md](../STATUS.md) and
[verification](verification.md). Reference-native compatibility has not been
established. Never infer it from the `.weftforge` extension or a mesh export.

## Capability matrix

| Format | Import/read | Export/write | Editable construction | Static/animated | Current limits |
| --- | --- | --- | --- | --- | --- |
| `.weftforge` v1 | Yes | Yes | Supported authoring model | Static authoring | JSON, 16 MiB limit; no embedded external assets or saved simulation frames |
| `.weftforge.recovery` | Explicit recovery | Prior valid primary on save | Same v1 model | Static authoring | One prior saved document; no silent fallback |
| `.weftforge.autosave` | Same project reader | Desktop named-document autosave | Same v1 model | Static authoring | Dirty named documents only; no automatic recovery UI |
| OBJ | No application import yet | Yes | Mesh only | Static | Thin, unwelded triangles; metre positions, normals, pattern-space UVs, piece groups; no MTL/textures |
| SVG | No | Yes | Boundary curves only in output | Static 2D | `M/L/C/Z` paths; mm physical size; no sewing/materials/internal features/print tiling |
| glTF/GLB | No | No | None yet | None yet | Planned validated avatar/mesh/animation workflows |
| DXF | No | No | None yet | None yet | Supported entity/unit subset still to define |
| Animation/cache formats | No | No | None yet | None yet | Timeline, frame caches and export not implemented |
| Marvelous Designer `.zprj` / `.zpac` | No | No | None | None | Separate FILE-006 research/compatibility workstream |

The current application has an original analytic capsule mannequin. No OBJ/glTF
avatar reader is wired into the project model; an installed parsing dependency
does not establish an import feature. Export validation with independent software
must be evidenced separately from our own writer/reader round trips.

## Native v1 representation

The UTF-8 JSON envelope is:

```json
{
  "format": "weftforge",
  "version": 1,
  "project": {
    "revision": 0,
    "points": [],
    "curves": [],
    "pieces": [],
    "materials": [],
    "seams": [],
    "mannequin": { "capsules": [] },
    "resolution": 0.07,
    "next_id": 2
  }
}
```

This structural illustration omits a valid default material; generate a valid
empty project with `weftforge-cli new FILE.weftforge`. Treat the current Rust
serialized types and validation as authoritative for v1; schema migrations are
not implemented. A different format marker/version, unknown field, malformed JSON
or invalid construction is rejected without replacing the open project.

Authoring coordinates use f64 metres. 3D is right-handed/Y-up; pattern coordinates
are material-space U/V with V upward. Placements store metre translations and XYZ
Euler rotations in radians, applied in that order. Mass uses kilograms, time
seconds, density kg/m², membrane stiffness N/m, and bending rigidity N·m.

Points and straight/cubic curves retain stable IDs. A piece contains a name,
ordered closed boundary of curve IDs, placement and material ID. Cubic controls
are absolute 2D coordinates; a null control array means a straight boundary.
Sewing stores two oriented curve intervals (`curve`, `t0`, `t1`) plus strictly
increasing normalized correspondence pairs including `[0,0]` and `[1,1]`.
Decreasing interval endpoints encode reversed direction. Derived seam samples
reference those authored relationships rather than persistent vertex indices.

Materials store physical parameters and display color. The mannequin stores
analytic capsule endpoints/radii. Resolution is a document-wide mesh setting.
The format does **not** yet store solver timestep/iterations, current drape or
velocities, undo history, selection/camera, grain-angle authoring, pins,
attachments, holes/internal lines, external avatar assets, textures, animation,
lighting or cache data. These are visible persistence gaps on the roadmap.

## Validation and saves

The reader caps input at 16 MiB, rejects unknown fields and validates complete
construction. Current limits include 20,000 authored objects, finite coordinates
within ±1,000 m, globally unique nonzero IDs, 256-byte names without control
characters, document resolution 0.005–0.5 m, and at most 128 mannequin capsules.
Contour, curve-chain, material, seam-interval/correspondence and reference checks
run before accepting a project. Mesh generation separately caps 100,000 vertices
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
creates a dirty copy requiring Save As. Autosave files use the same v1 parser;
failed recovery preserves the current document. Unsaved
close is blocked until save or an explicit discard action. There is no silent
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
layout/tiling, margins, units in third-party readers, and DXF entity subsets still
need independent workflow validation.

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
