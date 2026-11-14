# FILE-005 — DXF pattern-boundary export profile

Profile `FILE-005-v1`, primary sources reviewed 2026-11-14. This is an
implementation-independent contract for WeftForge's first DXF writer, not a
claim that it is implemented or independently accepted yet. The
[reference-route inspection](FILE-005-DXF-inspection-20261114.md) leaves reference
DXF options and output semantics unknown. This profile establishes no reference
parity, ASTM/AAMA apparel interchange, import, printing or garment round trip.

## Preserved content and coordinates

- Export every valid authored piece's closed, ordered exterior boundary. Each
  straight authored curve becomes one `LINE`; each authored cubic Bezier becomes
  one `SPLINE`. Preserve direction, order, corners and adjacent endpoints. Do not
  add a closing segment to an already closed chain, merge curves, fit a new
  curve, or replace a cubic with sampled lines, polylines or circular arcs.
- Coordinates are absolute pattern-space U/V, expressed in physical millimetres:
  DXF WCS `(X, Y, Z) = (1000 U, 1000 V, 0)`. Positive V stays positive Y; no SVG
  Y inversion applies. Preserve negative coordinates, relative piece positions
  and overlaps. Do not pack pieces, shift them to the origin or apply their 3D
  placements. Rest/display meshes, simulation, grain, camera and mesh resolution
  cannot determine exported geometry.
- Use finite f64 values throughout. Serialize each metre-to-millimetre result
  with enough decimal precision to recover that scaled f64 value on reading
  (signed zeros may compare equal). No f32 conversion or fixed decimal rounding.
  "Exact cubic" means the same polynomial curve representation; the required
  unit conversion still has normal f64 rounding.

DXF coordinates themselves carry no per-value unit. This profile combines numeric
millimetres with `$INSUNITS = 4`; consumers must be checked for their interpretation
of that hint. `$LUNITS` is a display setting and `$MEASUREMENT` selects metric
linetype/hatch conventions; neither substitutes for coordinate conversion.
See Autodesk's [header variables](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm)
and the independent consumer's [unit model](https://ezdxf.readthedocs.io/en/stable/concepts/units.html).

## ASCII R2000 drawing

Write a `.dxf` as ASCII group-code/value line pairs, LF line endings and a final
newline, with no BOM or binary payload. Declare `$ACADVER` (group 1) `AC1015`
(AutoCAD 2000), `$INSUNITS` (70) `4`, `$MEASUREMENT` (70) `1`, and `$LUNITS` (70)
`2`. R2000 is this profile's only version: it is supported by the independent
[SPLINE consumer](https://ezdxf.readthedocs.io/en/stable/dxfentities/spline.html),
while Autodesk identifies its version code in the
[header reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm).
An R12 declaration with R2000 entities is invalid for this profile.

Use these sections, each properly terminated, in this order:
`HEADER`, empty `CLASSES`, `TABLES`, `BLOCKS`, `ENTITIES`, `OBJECTS`, then `EOF`.
No proprietary classes, proxy entities or external references are needed. Include
the standard drawing infrastructure even though the garment geometry is small:

| Location | Required content |
| --- | --- |
| `HEADER` | The four variables above; `$HANDSEED` (5), greater than every emitted handle |
| `TABLES` | `VPORT`, `LTYPE`, `LAYER`, `STYLE`, `VIEW`, `UCS`, `APPID`, `DIMSTYLE`, `BLOCK_RECORD` |
| Standard table records | `LTYPE`: `BYBLOCK`, `BYLAYER`, `CONTINUOUS`; `LAYER`: `0` plus piece layers; `STYLE` and `DIMSTYLE`: `STANDARD`; `APPID`: `ACAD` |
| Space records | `BLOCK_RECORD`: `*Model_Space` and `*Paper_Space`, with matching empty `BLOCK`/`ENDBLK` definitions in `BLOCKS` |
| `ENTITIES` | Only boundary `LINE`/`SPLINE` entities, all owned by the model-space block record |
| `OBJECTS` | Root `DICTIONARY`, its empty `ACAD_GROUP` dictionary, and `ACAD_LAYOUT` dictionary with `Model`/`Layout1` `LAYOUT` objects linked to the two space records |

`VPORT`, `VIEW` and `UCS` may have no records. `Layout1` has no garment entities,
viewport or claimed printer/page setup. The standard space/layout infrastructure
is not per-piece block nesting. The consumer's
[minimum R13-and-later structure](https://ezdxf.readthedocs.io/en/stable/dxfinternals/filestructure.html#dxf-r13-r14-and-later)
explains why a lone `ENTITIES` fragment is insufficient; this profile explicitly
includes layout links as well.

Every table, record, block marker, entity and object that requires a handle gets
a unique positive hexadecimal handle. Allocate handles deterministically; source
object IDs are not DXF handles. Use `0` only for a DXF-defined null/root reference.
All emitted non-null references must resolve, including table-record owners,
model-space entity owners, dictionary entries and space/layout links. Table
counts must match emitted records. Include `AcDbSymbolTable` table markers,
`AcDbSymbolTableRecord` plus the specific table-record subclass, and the standard
block/dictionary/layout subclasses. `DIMSTYLE` uses handle group 105, not 5.
These are standard records, not application-defined data; see Autodesk's
[symbol-table structure](https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-A66D0ACA-3F43-4B2E-A0C2-2B490C1E5268.htm),
[common table-record fields](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5926A569-3E40-4ED2-AE06-6ACCE0EFC813.htm),
[block](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-66D32572-005A-4E23-8B8B-8726E8C14302.htm),
[block record](https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-A1FD1934-7EF5-4D35-A4B0-F8AE54A9A20A.htm),
[dictionary](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-40B92C63-26F0-485B-A9C2-B349099B26D0.htm)
and [layout](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-433D25BF-655D-4697-834E-C666EDFD956D.htm)
references. Do not include newer-version fields merely because a current
reference page lists them. In particular, omit `BLOCK_RECORD` fields 70, 280 and
281: the independent [block-management reference](https://ezdxf.readthedocs.io/en/stable/dxfinternals/block_management.html)
identifies these as R2007 additions. Physical model-space units come from this
profile's numeric coordinates and header, not those block insertion fields.

## Piece identity, entities and exact cubics

Each piece owns a declared layer named `WF_PIECE_<id>`, where `<id>` is its stable
nonzero authored ID in unpadded decimal. Do not derive layer names from user text.
Piece names, curve/point IDs and full document identity are not preserved by this
subset. Layer `0` and piece layers are visible, thawed and unlocked, with color 7
and `CONTINUOUS` linetype. Their records include `AcDbLayerTableRecord`; see
Autodesk's [layer fields](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-D94802B0-8BE8-4AC9-8054-17197688AFDB.htm).
Layers provide piece grouping; no `GROUP`, per-piece `BLOCK`/`INSERT`, XDATA or
text entity is emitted.

All boundary entities carry type (0), unique handle (5), model-space owner (330),
`AcDbEntity` (100), layer (8), model-space flag (67) `0`, layout name (410)
`Model`, and the appropriate geometry subclass (100). Their color/linetype inherit
the layer, their thickness is zero and their lineweight inherits the layer
(370 = -1). See Autodesk's [common entity fields](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm)
and [lineweight enumeration](https://help.autodesk.com/cloudhelp/2022/ENU/OARX-ManagedRefGuide/files/OARX-ManagedRefGuide-Autodesk_AutoCAD_DatabaseServices_LineWeight.html).

For a straight curve, `AcDbLine` stores its start in 10/20/30 and its end in
11/21/31, with both Z coordinates explicitly zero. Autodesk specifies these as
[WCS line endpoints](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-FCEF5726-53AE-4C43-B4EA-C84EB8686A66.htm).

For an authored cubic with endpoints `P0`, `P3` and controls `P1`, `P2`, emit
`AcDbSpline` with this complete geometric definition:

| Group | Required value |
| --- | --- |
| 70 | `8`: planar, open, nonperiodic, nonrational; no closed or linear bit |
| 71 | `3` (degree) |
| 72 / 73 / 74 | `8` knots / `4` control points / `0` fit points |
| Repeated 40 | Knot vector `[0, 0, 0, 0, 1, 1, 1, 1]`, in that order |
| Repeated 10/20/30 | The four scaled WCS points `P0`, `P1`, `P2`, `P3`, in that order, Z = 0 |
| 210 / 220 / 230 | Plane normal `(0, 0, 1)` |

Omit weights (all are implicitly one), fit points and start/end fit tangents.
Use the format defaults for spline tolerances; they are not a geometry fitting
budget. Autodesk defines these [spline group codes](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm).
The clamped degree-3 B-spline above is the single authored cubic Bezier, including
its endpoints and parameter direction; it is not an interpolating spline through
four samples. The independent consumer documents
[B-spline order, clamping and Bezier decomposition](https://ezdxf.readthedocs.io/en/stable/math/core.html#bspline).
Boundary closure comes from the ordered chain, not from marking each cubic closed.

## Determinism, loss and failure behavior

For the same valid document revision, repeated exports are byte-identical across
destination names. Emit pieces in ascending stable-ID order, each boundary in its
authored order, and piece layers in that same ascending order after layer `0`.
Use fixed infrastructure order, handle allocation and numeric formatting. Include
ASCII group-999 comments identifying `WeftForge`, `FILE-005-v1`, the source
document revision, millimetres, UV-to-XY mapping and boundary-only scope. Do not
emit timestamps, random identifiers, user names, original piece names or paths.

The exported curves are editable CAD geometry. This is not an editable WeftForge
project. Export entry points and format documentation must name the scope:
**pattern boundaries in millimetres**. Sewing/correspondence, materials, grain,
3D placement, drape, thickness, simulation/meshes, internal features, labels,
seam allowances, grading, print tiling and apparel-specific layer conventions
are outside this subset. It does not promise a consumer will retain splines or
honor physical scale on later import, conversion or printing.

Validate the source document and the scaled values before replacing any
destination. Invalid construction, unsupported future boundary geometry, invalid
paths or write/replace failures return an error; no approximation fallback is
allowed. Preserve the open project, revision, history and prior destination on
failure. Retain the existing export protections for native project/recovery files
and non-regular targets. Export has no authoring transaction and cannot overwrite
the loaded project through an export path. The standard text profile needs no
new runtime DXF dependency.

## Acceptance and evidence boundary

Implementation checks must establish: byte determinism; a known-size rectangle;
mixed straight/cubic pieces with asymmetric controls and negative coordinates;
multiple stable piece IDs and names containing non-ASCII/DXF punctuation; empty
valid projects; and failed-export preservation. Verify that changing 3D placement,
mesh resolution or simulation changes no boundary coordinates. Verify explicit
closure and that no extra edge or tessellated cubic is emitted.

An independent check must read the original, unmodified export through ordinary
`ezdxf.readfile`, never `recover.readfile` or a save/rewrite pass. Record the
consumer version and every audit error and fix; acceptance requires both lists
empty. `audit()` can repair data, so checking only `has_errors` is insufficient.
See the consumer's [normal reading workflow](https://ezdxf.readthedocs.io/en/stable/howto/document.html)
and [audit contract](https://ezdxf.readthedocs.io/en/stable/drawing/drawing.html#ezdxf.document.Drawing.audit).
Inspect the raw file as well, so default synthesis during reading cannot hide
missing declarations, wrong owners, count mismatches or duplicate handles.

Compare header units, model-space entity types/counts, per-piece layers, line
endpoints, spline flags/degree/counts/knots/controls/normal and exact adjacency.
Parsed coordinates must recover the expected scaled f64 values, treating signed
zero equally. Evaluate independent B-splines against the authored cubic at
`t = 0, 0.125, 0.25, 0.5, 0.75, 0.875, 1`; allow at most `1e-8 mm` absolute
coordinate difference for evaluation arithmetic. The exact knots and controls
remain mandatory; passing sample positions alone is insufficient. Confirm the
known dimension numerically in millimetres, rather than trusting a screenshot or
a unit label.

Publish only original fixtures and tracked outputs/reports, with source revision,
hashes, commands, consumer version and errors/fixes. Run registry/map checks from
a clean checkout before publishing evidence. Source tests and ezdxf inspection
prove only this profile's bounded writer/consumer checks. Native desktop export,
other CAD consumers, apparel workflows, printer scale and reference parity remain
separate, explicitly unknown until independently evidenced.
