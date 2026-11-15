# FILE-005 — Bounded editable DXF pattern import

Prospective profile `FILE-005-DXF-import-v1`, written 2026-11-15 before
implementation. This is an implementation-only acceptance contract for the
existing [FILE-005-v1 writer subset](FILE-005-dxf-pattern-export.md). It establishes
no reference parity, general CAD compatibility, apparel interchange or full
garment round trip.

The post-implementation review corrected the save-failure wording below to match
inherited native primary/recovery ordering. The original prospective wording is
retained in Git; no stronger two-file save transaction is implemented here.

## Accepted drawing and construction

Accept ASCII group-code/value pairs with LF or CRLF line endings, optional ASCII
horizontal padding, and a terminal `EOF`. Require R2000 `$ACADVER=AC1015`, physical
millimetres `$INSUNITS=4`, metric `$MEASUREMENT=1`, decimal `$LUNITS=2`, and a valid
`$HANDSEED`. Require the writer's ordered `HEADER`, empty `CLASSES`, `TABLES`,
`BLOCKS`, `ENTITIES` and `OBJECTS` sections. Group-999 comments are descriptive
only. No binary DXF, BOM, non-ASCII payload, unknown header variable, section,
record, group field, external reference, XDATA or extension dictionary is accepted.

The nine standard tables, visible/unlocked `0` and `WF_PIECE_<id>` layers,
two empty standard space blocks, root/group/layout dictionaries and Model/Layout1
layouts must use the writer's fixed fields and ordering. Handles may differ from
the writer's allocation but must be positive, unique and correctly linked; table
counts and the hand seed must agree. No paper-space or block-contained geometry,
extra named layer, group, layout, style, linetype or authoring record is discarded.
This strict reader is not a general DXF structural auditor or repair tool.

Only model-space `LINE` and `SPLINE` entities on declared `WF_PIECE_<id>` layers
are accepted. A layer suffix is unpadded decimal, positive and representable as
`u64`. Each used layer supplies exactly one closed, ordered exterior contour with
at least three spans; adjacent endpoints, including final-to-first, must compare
equal in source millimetres. Preserve entity order and direction within each
piece. No snapping, sorting endpoints, reversing edges, gap filling, curve fitting,
sampling, contour merging or extra closing edge is permitted. Layer declarations
and pieces are imported in declaration order; every piece layer must be used.
An empty drawing fails because there is no pattern to add.

`LINE` endpoints are WCS X/Y with explicitly zero Z and thickness. `SPLINE`
requires planar/open/nonperiodic/nonrational flag `8`, degree `3`, eight knots
`[0,0,0,0,1,1,1,1]`, four WCS controls, zero fit points, normal `(0,0,1)`, and no
weights, fit points, tangents or nondefault tolerances. This is precisely one cubic
Bezier: controls 0/3 are boundary endpoints and controls 1/2 remain editable.
Reject other entity types, flags, dimensions, normals and knot definitions.
Autodesk documents [LINE WCS endpoints](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-FCEF5726-53AE-4C43-B4EA-C84EB8686A66.htm),
[SPLINE groups and flags](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm)
and [header units/version](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm).

Convert finite f64 millimetres to SI metres by division by `1000.0` exactly once,
using the same conversion for endpoints and controls. Map `(X,Y)` to pattern
`(U,V)` without Y inversion or origin subtraction. Preserve negative coordinates,
relative arrangement and overlaps between pieces. No packing or 3D transform
applies. All converted coordinates must pass the native ±1000 m bound and native
construction/contour validation, including rejection of degenerate or intersecting
pieces. Conversion carries ordinary f64 rounding; no f32 or fixed rounding occurs.

The writer omits original names and native IDs. Imported pieces receive literal
layer names such as `WF_PIECE_17` and fresh native IDs. They use the first material,
zero grain angle and default 3D placement. Sewing/correspondence, materials,
original names/IDs, grain, placement, drape, meshes, internal features, labels,
allowances, grading, printing and apparel conventions are outside this profile.

## Trust boundary and transaction

Read at most 1 MiB plus a sentinel byte from a regular file. Bound the parsed
input to 131072 group pairs, 64 pieces and 4096 total boundary entities. Reject
malformed pairs, duplicate fields/handles, unresolved or wrong-owner links,
nonfinite/overflow/extreme coordinates and unsupported content before committing.
Never fetch external resources. Keep the source file unchanged.

Expose one shared typed `import_dxf_patterns { dxf }` command, CLI
`import-dxf INPUT OUTPUT PATTERNS.dxf`, MCP `project_import_dxf` and native
**Import DXF…** toolbar/search action. Report the fixed physical scale, origin
convention, count and explicit construction-only scope. Schema/discovery and
adapters must agree. Validate all pieces before one atomic authoring transaction;
all pieces and accepted pending inspector edits consume one Undo step. Parsing,
validation and execution failures preserve the existing authored project and
history. CLI/MCP save failures retain the existing output primary under the
inherited native-save guarantees. Saving copies the valid current primary to
recovery before the final primary replacement; if that replacement fails,
recovery may already have advanced to that valid current-primary snapshot.
Primary and recovery are not one atomic two-file transaction. Separately,
CLI/MCP refuse output or recovery paths that replace the source DXF, including
aliases handled by existing native save protection.

The native review shows count, millimetre scale, preserved layout, layer-name
convention and omitted content before Import/Cancel. Opening either chooser or
review, Cancel, invalid source, Escape and rejected Import retain pending numeric
drafts, document revision, history and dirty state; a later ordinary blur may
commit the retained draft. Background actions are disabled during review.

## Required implementation checks

Use an original hand-authored redistributable DXF fixture independent of writer
output, with asymmetric cubic controls, negative coordinates and two pieces.
Check known dimensions, controls, adjacency, names, fresh IDs, default material/
placement, editability, save/reopen, one Undo/Redo and retained prior history.
Own-writer round trips are supplemental checks only. Exercise malformed pairs,
missing/other units, open/gapped/reordered contours, unsupported entities/flags/
weights/fields, paper/block geometry, duplicate handles/fields, wrong references,
extreme/nonfinite values and byte/entity/piece bounds. Test atomic batch failure,
CLI/MCP discovery/schema/strict deserialization, source/primary retention,
unchanged recovery for the exercised failures before recovery replacement, and
native pointer/accessibility review/Cancel/accept behavior. Those checks do not
establish unchanged recovery after every save failure; full persistence
acceptance remains open.

Tests establish bounded implementation behavior only. Actual native chooser/
editing/history/save acceptance, independent consumer checks and any reference
comparison require separate identified runtime evidence. Full DXF coverage and
reference parity remain open.
