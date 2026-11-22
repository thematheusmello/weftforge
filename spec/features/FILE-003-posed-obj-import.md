# Importing a posed garment from OBJ

A garment OBJ can supply an initial three-dimensional surface and planar UV
charts for editable patterns. On the reference build 2026.1.237 (r58920),
importing as **Garment**, with **Trace 2D Patterns from UV Map** enabled, units
in metres, scale 100% and Y up, produced both views for the original fixtures
below. [Observation and measurements](../observations/REF-FILE-003-OBJ-POSED-001.json).

| Original input | Flat pattern observation | Initial surface observation |
| --- | --- | --- |
| [Bent swatch](../../fixtures/posed-obj/Bent%20swatch.obj) | 240.2 × 360.2 mm raster bounds. The displayed 2D perimeter was 1200.0 mm. | Its two equal-width halves meet at 60°. The displayed 3D perimeter was 1197.5 mm. |
| [Extended bent swatch](../../fixtures/posed-obj/Extended%20bent%20swatch.obj) | The UVs are identical to the bent swatch. The new bounds were 245.4 × 368.2 mm and the displayed 2D perimeter was 1226.7 mm. | Extending the right half by 10% changed the displayed 3D perimeter to 1218.8 mm. |
| [Draped Harbor](../../parity/reports/artifacts/native-selected-obj-20261120/Harbor%20draped.obj) | Two outlines measured 1338.4 × 649.0 and 1338.8 × 649.0 mm. Their individual material-to-outline association was not recorded. | The reference returned two nonplanar surfaces and preserved their diffuse colors. Triangle counts changed from 586 per panel to 5408 and 5428. |

The private snapshots used a 0.2 mm pixel pitch with ±0.6 mm uncertainty on
each extent. Harbor surface area changed from 0.597433 to 0.595181 m² for blue
and from 0.597431 to 0.595216 m² for terracotta. Testing all vertices and triangle
centroids against the opposite surface in both directions gave a largest sampled
distance of 5.20 mm in the original coordinate frame. These results describe
remeshing on this build. They do not identify the reference's scale estimator.

## WeftForge behavior to implement

Offer an explicit posed mode alongside the existing flat import in the studio,
CLI and MCP. Confirm source units before the edit. Preserve a UV chart per
supported panel as editable construction, with stable boundary ownership, holes
and fabric assignment. Associate the source positions with their material
coordinates so each panel can start in its imported pose. Preserve the distinction
through saving, reopening, Undo, Redo, simulation initialization and export.

Validate the original fixtures at both ends. Measure flat dimensions and
perimeters separately from the 3D positions, pose orientation and color.
Use the input positions as the target for a direct posed export. Grade any
comparison to the reference's resampled return separately. Declare tolerances
and the physical scale policy before evaluating the import. A failure to resolve
chart scale or mapping must identify the affected panel and preserve the open
project. These fixtures provide no sewing, avatar or animation reconstruction
contract.
