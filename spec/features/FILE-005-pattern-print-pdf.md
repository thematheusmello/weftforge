# FILE-005 tiled pattern PDF

WeftForge writes 1:1 vector patterns on portrait A4 (210 × 297 mm) or US
Letter (215.9 × 279.4 mm). File and Command search offer both paper sizes.
The CLI accepts `export-pdf PROJECT OUTPUT.pdf a4|letter`, and the local MCP
tool `project_export_pdf` requires the paper as a typed argument.

The writer uses PDF 1.7 default user space at 72 units per inch. Increasing
pattern U moves right and increasing V moves up. Each tile clips a translated
vector drawing at its paper margins without changing the scale. The catalog
requests no scaling in the print dialog. [PDF 1.7 reference](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf).

- The first sheet contains a 100 × 100 mm scale square and printing instructions.
- Each pattern uses a separate grid across 10 mm horizontal and 20 mm
  vertical margins. The columns run left to right, and rows run top to
  bottom. Adjacent tiles share 10 mm of geometry and matching crosses.
- Solid strokes use the authored exterior, hole and notch geometry. Cubic
  curves retain their control points. Straight internal annotations are dashed.
- Paper labels use piece IDs and tile positions. Bookmarks carry exact
  piece names and link to each piece's first sheet.
- Exports require at least one valid pattern and allow up to 4096 sheets,
  including calibration. A failed export preserves the destination.
- The `.weftforge` project retains editable construction, sewing, grain and
  material data. The PDF uses the authored 2D geometry independently of
  placement, viewport and simulation.

For the scale check, decode both paper sizes independently. Check page
dimensions, the two square edges, cubic and straight paths, hole, annotation
and notch geometry, tile transforms, clipping and registration. Compare the
native and command-line PDF bytes for the same project and paper. A paper
measurement with the reader and printer set to Actual size is a separate check.
