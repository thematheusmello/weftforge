# Posed garment OBJ

Flat import remains the default. With Posed selected, each connected panel keeps
its source triangles and an initial XYZ position per material vertex. Mesh
resolution applies to generated patterns. The imported panels retain the OBJ
tessellation, including folds and cutout boundaries.

The physical scale in metres per UV unit is the square root of the total posed
triangle area in square metres divided by the total UV triangle area on that
panel. This preserves surface area under one uniform chart scale. A clockwise UV
island is reflected to preserve face orientation. The source units apply to XYZ.

The two bent swatches and the draped Harbor are the original acceptance inputs.
Direct readback budgets are 1 nm per position component, 1 nm for analytically
predicted flat extents and perimeters, equal oriented triangles, and 0.000001 per
diffuse color channel. For comparisons to the reviewed reference results, grade
each flat extent separately at its reported ±0.6 mm measurement uncertainty.
The reference remeshed the surface and its scale estimator is unknown.

Pattern resize changes the material dimensions and leaves the initial 3D surface
in place. A rigid 2D transform preserves world placement. Change the arrangement
to move the entire posed surface. For edits that change the chart boundary,
choose **Clear imported pose** first. Undo restores the original chart and pose.

| Input panel | Width × height (mm) | Flat perimeter (mm) | Reference extent comparison |
| --- | --- | --- | --- |
| Bent swatch | 240.000 × 360.000 | 1200.000 | Both within tolerance |
| Extended bent swatch | 245.927 × 368.890 | 1229.634 | Width within tolerance, height 0.690 mm above reference |
| Harbor blue | 1325.707 × 642.767 | 4100.800 | Both below either measured reference outline beyond tolerance |
| Harbor terracotta | 1325.705 × 642.766 | 4100.793 | Both below either measured reference outline beyond tolerance |

All three inputs preserve the original triangle surfaces for direct export.
These comparisons concern initial geometry. The reference run did not establish
a simulation-continuation contract.
