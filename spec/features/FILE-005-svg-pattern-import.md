# FILE-005 — bounded editable SVG pattern import

Prospective profile `FILE-005-SVG-import-v1`, agreed before implementation on
2026-11-15. This is an original behavior contract, not reference parity, complete
SVG support, a complete garment round trip, or physical printing acceptance.

## Accepted drawing and physical scale

Accept UTF-8 XML containing exactly one unprefixed `svg` root whose default
namespace is `http://www.w3.org/2000/svg`. Require positive finite `width` and
`height` explicitly in `mm`, and a finite four-number `viewBox` with positive
width/height. The two ratios `width_mm / viewBox_width` and
`height_mm / viewBox_height` must agree within relative tolerance `1e-12`.
Missing units, percentages, pixels, other physical units, nonuniform scale,
transforms, CSS, nested viewports and `preserveAspectRatio` overrides fail with
an explanation; there is no inferred DPI or implicit scale choice.

For the supported uniform ratio `s` millimetres per SVG user unit, construction
coordinates are `U = x * s / 1000`, `V = -y * s / 1000`, in f64 SI metres.
Keep the SVG user-space origin, negative coordinates, path order and relative
piece layout. Do not subtract `viewBox` minimum coordinates, pack pieces or apply
3D placements. The viewBox determines physical scale, not a new construction
origin. This origin policy preserves WeftForge's own absolute-coordinate SVG
exports; a rendered viewport may have a different display translation.

Each direct child `path` is one editable exterior pattern piece. Accept absolute
`M`, `L`, `C`, and `Z` commands only, including SVG numeric signs, decimals,
exponents, comma/whitespace separators and implicit repeated coordinate groups.
After the first moveto, extra coordinate pairs mean lineto. Require exactly one
moveto and a final explicit closepath. A closepath adds a straight closing edge
only if the last endpoint differs from the first; an explicit final straight or
cubic span ending at the initial coordinate reuses the initial point.
Every result must have at least three nondegenerate spans and meet the existing
project geometry constraints. Do not flatten cubics, fit new curves or silently
close an open path. Relative commands, other path commands, multiple contours,
holes and unsupported geometry elements reject the entire drawing.

Keep the decoded `data-name` when present, otherwise the path `id`, otherwise
use `SVG pattern N` in source order. Names must meet the native name constraints.
Source XML IDs are labels, never native stable IDs. Allocate fresh IDs from the
destination document. New pieces use the destination's first material, zero
grain angle and default 3D placement. Preserve all preexisting native objects.

## Explicit omissions and rejection boundaries

The root may carry `version` (`1.1` or `2.0`). Paths may carry `id`, `data-name`,
`d`, and basic literal `fill`, `stroke`, `stroke-width` attributes. Root/path
`title` and `desc` children and XML comments are accepted descriptive content.
Appearance, descriptions, source IDs, sewing, physical materials, grain,
internal details, allowances, grading, 3D arrangements and simulation are not
imported. Every native summary and CLI/MCP success report states that only
editable exterior boundaries and names are imported, with default material and
placement. Unknown attributes/elements and resource-bearing appearance fail
explicitly, rather than silently omitting geometric or hidden content.

Reject DTDs, custom entities, processing instructions, scripts, links, images,
external references and asset loading. Permit only predefined XML escapes and
numeric character references in accepted text/attributes. Parse the complete
document, including trailing data; malformed XML, duplicate attributes and
unfinished documents are errors. Never fetch entities or make network requests.

Limits: at most 1 MiB of source bytes, 64 paths, 4096 total resulting spans,
XML depth 3 and the destination's native 20,000-object/coordinate/name limits.
Reject an exhausted limit. Existing contour validation detects degenerate,
zero-area and self-touching/intersecting construction with bounded cubic
polyline sampling; it is not an exact CAD intersection predicate. Unsupported
precision remains an explicit error. These are implementation bounds, not
reference limits.

## Transaction and adapters

The shared typed command is `import_svg_patterns` with `data: {"svg": "..."}`.
All pieces commit through the shared editor as one revision and one undo entry.
An error preserves every prior object, stable-ID allocator and history entry.
Parsing and applying do not touch the input file. Native import is additive;
it never replaces the open project or its file path. Pending inspector changes
and imported pieces commit together through the existing inspector action.

CLI: `weftforge-cli import-svg INPUT OUTPUT PATTERNS.svg`.
MCP: `project_import_svg` with required `input`, `output`, `svg` file paths.
The ordinary `project_execute`/CLI command array also discovers and accepts the
same shared command. Both file adapters read bounded original SVG bytes, apply
one command and use the native atomic save/recovery behavior. Refuse aliases
between the SVG source and output/recovery destinations. Parse/load/validation
failures write nothing; failed native saves retain the existing primary under
the already documented native-save guarantees.

The native **Import SVG…** toolbar/search action chooses a file, then shows its
piece count, physical millimetres per user unit, preserved origin/upward pattern
axis and the construction-only omissions before **Import** or **Cancel**.
No scale selector is needed for this explicit-mm profile. Cancellation, invalid
input or failed command application retains the document, pending numeric
drafts and undo/redo history. Modal input shielding covers pointer, keyboard
and accessibility actions, following the existing rectangle/sewing modals.

## Acceptance planned before implementation

Original independent SVG fixtures establish endpoint/control coordinates,
names, ordering, negative origin, uniform non-unit scale and closepath behavior.
Imported original coordinates/control positions and own-export round trips
must agree within `1e-12 m` for the small in-range fixtures (signed zero may
compare equal). Independent SVG consumer checks interpret the original fixture
using its own XML/path/viewport code, confirm physical scale and compare cubic
samples within `1e-8 m`; that looser consumer bound allows float DOM APIs.
Neither a self-round-trip nor implementation-only tests establish reference
parity. Real native chooser/modal/edit/undo/save acceptance is separate from
handler tests and remains required before describing user-visible delivery.

Focused automated cases cover straight/cubic and repeated-number syntax,
malformed/unsupported SVG, unit/viewBox/namespace/entity failures, resource and
count limits, geometry rejection, transactional history/ID preservation, native
file round trip, CLI/MCP strict discovery/schemas/stdio and destination retention,
and native pending-draft/modal actions. Run all required local repository gates
once the parent clears CPU-heavy checks; do not treat unrun checks as passes.

## Public specification basis

The [W3C SVG path specification](https://www.w3.org/TR/SVG2/paths.html#PathData)
defines absolute moveto, lineto, cubic and closepath grammar, repeated coordinate
groups and compound paths. The [W3C SVG viewport transform](https://www.w3.org/TR/SVG2/coords.html#ComputingAViewportsTransform)
defines physical viewport/viewBox scale and display translation. This profile
deliberately retains the separate user-space construction origin. The
[W3C CSS absolute-length table](https://www.w3.org/TR/css-values-4/#absolute-lengths)
defines millimetres; this profile rejects other units instead of guessing DPI.
Reviewed public sources only; no proprietary installers, assets or extracts.
