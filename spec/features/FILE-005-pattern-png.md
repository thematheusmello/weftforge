# FILE-005 scale-aware pattern PNG

**Save 2D pattern PNG…** in File and Command search opens a crop and density
dialog. Fit the editable patterns, choose a paper preset or enter arbitrary
dimensions. Width and height can be swapped around the crop centre. The
2D pane outlines the crop while the dialog is open. The dialog reports
pixel dimensions and lets the user toggle outlines and holes, internal
lines, notch marks and grainlines.

The crop uses the lower-left and upper-right material coordinates. The CLI
and MCP use metres. In the desktop dialog, choose millimetres, centimetres,
inches or pixels for the dimensions and origin. Switching units keeps the
physical crop fixed. The writer rounds pixels per inch divided by 0.0254
to an integer pixels-per-metre value. Both axes use that density for the
pixel fields, drawing and the PNG pHYs chunk. Raster X increases with
pattern U. Raster Y increases downward from the crop's maximum V.

The image uses opaque white and antialiased black marks. It preserves
physical dimensions to within half a pixel of crop rounding per side.
Dimensions up to 16384 pixels per side and 32 million total pixels are
supported within the raster's stroke budget and 80 MiB output bound.
The stored density lets consumers recover the physical dimensions.

When **Grainline** is enabled, each arrow starts at the centre of the
piece's exterior bounds and points in its authored warp direction, as in
the 2D pane. The CLI and MCP `grainline` option defaults to `false`.

The desktop uses a background worker. Cancelling or changing the source
document discards an outstanding result before replacing the destination.
A failed export preserves the previous file. The SVG, PDF and native
project retain their respective vector and editing roles.

Verify the physical pixel density and a known asymmetric crop with an
independent PNG decoder. Check the positions of straight and cubic curves,
holes and marks, toggle layers, and compare the desktop export with the
CLI and typed MCP operations. Actual printer measurements and the reference
snapshot's presentation controls have separate acceptance checks.
