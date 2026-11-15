# Prospective independent DXF import evaluator

This private preparation is implementation-only and has not executed WeftForge.
The behavior oracle is the corrected public FILE-005-DXF-import-v1 contract at
e4236d1da4a34874e75f08cac7e5e4d1a23cf779 and the original hand-authored CC0
`original-editable-patterns.dxf`. No DXF importer, writer, runtime implementation,
or product test has been read to derive expectations. Prior SVG/DXF product
outputs have not been read. The original preexisting native baseline is reused
with permission; it is an input, not an imported output. The unchanged prior
independent SVG checker supplies only its JSON/native-boundary and SVG-consumer
functions. No WeftForge code is imported or invoked by this evaluator.

`EXPECTED.json` records the analytic U/V spans before product execution. The
first layer is WF_PIECE_17, preceding WF_PIECE_4 despite suffix order 17 > 4.
Each has four spans/four shared endpoints. There are seven lines and one cubic.
Physical source coordinates divide by 1000 once without Y inversion, packing,
origin subtraction, or placement transform. Cubic P1/P2 remain distinct and
editable. Negative coordinates and inter-piece arrangement must survive.
The original straight first edge measures .2 m and the second piece .2 x .2 m;
the first piece's endpoint rectangle is .2 x .125 m, while its asymmetric cubic
extends beyond that rectangle. An endpoint bounding box is not a cubic bound.

Imported names are the literal source layers; fresh IDs are discovered through
actual references rather than guessed from layer suffixes or allocation order.
Exactly eight points, eight curves, and two pieces are added. Global uniqueness,
freshness, adjacency by shared IDs, all-used geometry, first material, zero grain,
zero translation/rotation, and every surviving original authored field are checked.
Allocator 11 + 2*(piece1 + point4 + curve4) = 29 must persist through Undo/Redo.
Synthetic checks deliberately permute the 18 fresh IDs to exclude guessed order.

Endpoints/controls compare with independent Euclidean tolerance 1e-12 m. Every
line/cubic also compares at 17 parameters 0/16 through 16/16, tolerance 1e-8 m,
using independent Bernstein evaluation. The original source is consumed by
ordinary installed ezdxf1.4.5 readfile, then checked against the analytic oracle.
The same consumer checks exported DXF units, ordered layers, LINE/SPLINE type,
control count/coordinates, nonrational degree/flags/knots, zero Z and thickness,
normal, exact closure and absence of extra blocks/paper geometry. Audit errors
and audit fixes must both be zero; recovery/repair loading is never used.
The consumer adds a Defpoints layer in memory even for the original fixture;
an independent small raw group-pair scan checks declared layer names/order,
so a physically present extra layer cannot be hidden by consumer normalization.
The ezdxf B-spline construction tool samples the cubic separately from Bernstein.
The existing environment is reused; no package is installed.

Direct exported SVG uses the unchanged independent XML/path consumer: explicit
physical mm, uniform viewBox scale, V-up to SVG Y-down mapping, preserved origin,
names/order, and unchanged straight/cubic construction. Export expectations come
from native geometry only after that native document has passed the independent
analytic input checks; no expected product output bytes are hardcoded.
DXF export layer names use actual native piece IDs in sorted stable-ID order,
as documented, not the source layer names that became native piece names.

The original native baseline has revision1, allocator11, existing rectangle name
"Existing baseline", origin(.5,.1), width.1, height.12, existing material1 and
unchanged IDs. Actual native execution must follow PLAN.json. Open clears prior
history, so two width commits and Undo explicitly create both history directions:
.11 at revision2, .12 at revision3, Undo to .11 at revision4. Preimport is saved
before entering pending width .125. Cancel must retain that pending draft,
revision4, dirty state, and both history directions, proven by separate raw trace.
Accept commits pending width .125 and both imported pieces together at revision5.

Nine named actual native files are required: preimport4, imported5,
undo_import6 (one old rectangle width.11), undo_prior7 (width.1), redo_prior8
(width.11), redo_import9 (all imported geometry/IDs and pending width.125),
edited10 (only first imported cubic P1X -.01 -> 0), rejected10, reopened10.
Preimport rectangle corners use analytic tolerance1e-12 for the two changed X
values and recursive exact comparison of everything else. Imported pending
rectangle corners likewise use analytic tolerance; native imported DXF controls
remain checked independently. Undo/Redo then compare complete restored checkpoint
documents exactly apart from declared revision/high-water fields, not a new
approximation. Both Undos retain allocator29. Rejected and actual fresh-process
reopened/resaved documents are recursively exact edited10. JSON whitespace is
ignored; arrays, fields, types, IDs, numeric values, and revisions are preserved.

Headless shared-command, dedicated CLI and MCP paths form separate attempts from
the original baseline: unchanged old width.1, imported revision2, allocator29.
They each require raw command/protocol logs, native output, and direct SVG/DXF
export files graded by this evaluator. Schema/discovery, strict deserialization,
source/primary/recovery byte retention and batch atomicity are assessed from
explicit before/after files and protocol transcripts, not inferred from native
geometry files. PLAN.json defines the cases and expected distinctions.

Native JSON files cannot establish chooser use, toolbar/search accessibility,
modal background disabling, pending drafts, Cancel/Escape, action causes, single
history steps, actual quit/relaunch or save UI behavior. Separate raw pointer/AX
trace review must establish those. A static pass reports that requirement rather
than inventing native acceptance. No reference, general CAD, apparel, printing,
physics, cross-platform or full DXF profile acceptance is claimed.

Failed CLI/MCP saves must retain the existing output primary. Recovery is a
separate ordered operation: it can already have advanced to the valid old primary
before final primary rename fails. Only failures explicitly exercised before
recovery replacement may claim unchanged recovery. A final-rename failure would
be graded as primary retained plus recovery unchanged OR advanced to the exact
valid pre-save primary, according to the observed failure stage. It must never
be reported as an atomic two-file transaction. Crash durability and unexercised
save-failure stages remain open. Source/recovery path-alias protection is separate.

All frozen files are hash-checked before/after grading, original source files
remain unchanged, missing actual artifacts fail, and self-check cannot be combined
with actual paths. SELF_CHECKS.json is synthetic checker evidence only. Product
execution remains prohibited until root supplies exact source/binary binding.

Run with the preexisting `.local/dxf-acceptance-20261114/venv/bin/python`:

```
evaluate.py --self-check
evaluate.py --imported OUTPUT.weftforge --exported-svg OUTPUT.svg --exported-dxf OUTPUT.dxf
evaluate.py --native-checkpoints CASE_DIRECTORY --exported-svg FINAL.svg --exported-dxf FINAL.dxf
```

Actual product files must be distinct, outside the frozen evaluator directory.
The evaluator writes only JSON to stdout and never edits an input artifact.
Publication is a later step: ignored local files alone are not delivered evidence.
