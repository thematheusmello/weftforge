# Independent pattern PDF check

This check uses an original WeftForge v6 document at revision 2. Juniper α contains one cubic contour edge, an 8 mm notch and a 150 mm dashed annotation. The contour spans two rows and two columns on each supported paper size. Basin β is a separate piece with a 40 × 35 mm hole. The two pieces have distinct authoring coordinates.

A4 and US Letter each export six pages. The first page contains a 100 × 100 mm calibration square. A4 was exported through File in the desktop studio and Letter through keyboard command search. The corresponding CLI files match the desktop files byte for byte. The project SHA-256 stayed the same in the desktop copy.

`CHECK-SPEC.json` freezes the fixture SHA-256, page counts, geometry, and acceptance bounds. The checker derives vectors and cubic extrema independently from the project and inspects PDF dimensions, clipping, translations, an exact 1:1 pattern scale, 10 mm adjacent tile overlaps, matching marks, calibration and Unicode outline destinations. The absolute tolerance is 0.0001 pt for vector measurements and 0.012 pt for Poppler page dimensions rounded to two decimal places. Both files request `/PrintScaling /None`.

Poppler 26.03.0 rendered all twelve pages at 72 dpi. Visual inspection confirmed the calibration text and square, unbroken tile contours, dashed internal mark, notch, registration crosses and separate piece with a hole. pypdf 6.15.0 rejected controls with an altered print scaling directive and with a stretched tile. This evidence covers digital dimensions and rendering. Physical printer and driver scaling still need a measured paper print.

## Reproduce

Use Python 3.10 or newer with pypdf, plus `pdfinfo`, `pdftotext` and `pdftoppm` on PATH. Run the checks from this directory. The render destination must be fresh.

```sh
python3 check_pattern_pdf.py a4 final-control/cli-a4.pdf --render-dir replay
python3 check_pattern_pdf.py letter final-control/cli-letter.pdf --render-dir replay
```

Supply `--native` with a desktop export path to verify byte equality. The exact executed CLI arguments, tool hashes and artifact hashes are recorded in the receipts. The two typed authoring recipes can regenerate the document with a compatible WeftForge CLI in a scratch directory.

```sh
weftforge-cli new empty.weftforge
weftforge-cli execute empty.weftforge rectangle-scaffold.weftforge 01-pieces.json
weftforge-cli execute rectangle-scaffold.weftforge print-original.weftforge 02-details.json
weftforge-cli inspect print-original.weftforge
```
