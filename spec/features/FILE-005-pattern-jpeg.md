# FILE-005 scale-aware pattern JPEG

**Save 2D pattern JPEG…** opens the desktop crop dialog from File and
Command search. Set the crop with a preset, fit the patterns or enter
dimensions and origin in millimetres, centimetres, inches or pixels.
The 2D pane shows the crop. Turn the contours and holes, internal lines,
notches and grainline on or off independently.

The CLI and MCP take the same typed options as PNG, with the crop as
lower-left and upper-right coordinates in material-space metres. JPEG
rounds the requested density to whole pixels per inch in `1..=65535`.
The desktop and MCP show the effective density. The renderer and the
sizing controls use that density divided by 0.0254 for pixels per metre.
Each raster dimension is rounded to the nearest pixel. Raster X increases
with pattern U and raster Y increases downward from the crop's maximum V.

The encoder writes an 8-bit baseline grayscale JPEG on white paper at
quality 100. JFIF stores equal inch densities on both axes. Reading the
size with that density recovers each physical side within half a pixel.
The crop shares the PNG raster limits of 16,384 pixels per side, 32 million
pixels, the bounded stroke work budget and 80 MiB of encoded output.

The desktop stages the image in a background worker and commits it only
for the initiating document revision. A failed, cancelled or stale export
preserves the previous file. Editable construction stays in the project.

Compare native, CLI and MCP bytes for identical options. Independently
read the JFIF and baseline-frame metadata and decode the curves, hole,
annotation, notch and rotated grainline. Check an asymmetric crop and a
fractional density with a 2 m crop at 72 pixels per inch, which produces
5,669 JPEG pixels across. Physical printer and reference checks have
separate acceptance criteria.
