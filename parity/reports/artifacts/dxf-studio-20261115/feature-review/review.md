# Independent DXF import source review — 2026-11-15

Reviewed candidate `69e7e57a7175977dcffa159ecb6e83944f04259e` against public base
`224e7b44b2f40fa479397ae50443efa63550da9c`. Candidate checkout was clean at
review time. This review read the complete diff and connected writer,
document/Editor, native numeric/modal, CLI/MCP schema, persistence and test code.
It used the original hand-authored CC0 fixture and existing writer source only;
no proprietary material or reference-parity inference was used.

## Final decision after follow-up

No outstanding source-review blocker found at
`e4236d1da4a34874e75f08cac7e5e4d1a23cf779`. The complete follow-up diff changes
only the three Markdown files named below and the candidate checkout remains
clean. It accurately separates atomic authoring, output-primary retention,
independent source-path guards and possible advancement of recovery before a
final primary replacement failure. It also preserves the history of the
prospective-contract correction and leaves full persistence acceptance open.
No Rust behavior or test changed; no Cargo or GUI job was run for this review.

## Original candidate decision

One acceptance-contract wording gap requires resolution before claiming the
contract as written. No separate blocker was found in the new DXF parser,
atomic authoring command, adapters, native modal implementation or shared SVG
refactoring.

### P2 — Failed-save recovery retention exceeds the existing native guarantee — resolved by follow-up documentation

The new prospective contract at
`spec/features/FILE-005-dxf-pattern-import.md:74–76` promises that failed saves
preserve the old project and output/recovery files; the public format notes at
`docs/file-formats.md:249–250` also describe output/recovery protection on failure.
`apps/weftforge-cli/src/main.rs:333–336` routes DXF imports through the existing
`Project::save` path.

That path replaces `<output>.recovery` with the current primary before the final
primary replacement (`crates/weftforge-core/src/persistence.rs:239`, then `:246`).
If the final replacement fails, the caller returns an error with the primary
retained, but a distinct older recovery has already been replaced. The existing
private injected-interruption branch at `:241–244` has exactly this ordering;
its test does not begin with a distinct previous recovery.

This predates the candidate and does not compromise DXF source retention. It is
nevertheless incompatible with the newly written all-files-unchanged guarantee.
The smallest resolution is to state the actual existing native guarantee, as
the accepted SVG import contract already does: parse/load/validation failures
write nothing; failed saves preserve the existing primary under native save
guarantees. If byte-for-byte retention of the previous recovery on every failed
save is required, persistence needs a separate transaction fix and a failure
test starting with distinct primary/recovery contents. No such test was run by
this reviewer.

## Reviewed behavior

- The reader follows the writer's complete ordered HEADER/CLASSES/TABLES/BLOCKS/
  ENTITIES/OBJECTS scaffold. Unknown fields/records and extra authoring content
  fail at the positional expectation instead of being discarded. Only 999
  comment pairs are ignored, and they still count toward the pair bound.
- Handles are parsed as bounded hexadecimal integers, record handles are
  positive/unique, owner/layout/style links are checked against typed roles,
  table counts are enforced, and HANDSEED exceeds every record handle.
- Input bounds cover 1 MiB plus sentinel file reads, 131072 pairs, 64 pieces and
  4096 spans. Regular-file checks, ASCII validation, finite coordinates and native
  ±1000 m coordinate validation remain at the input boundary.
- Each declared piece layer must be used by at least three spans. Source-mm
  endpoint equality checks enforce directed adjacency and final closure before
  any conversion. LINE and the exact clamped cubic SPLINE profile preserve
  source entity order. Every endpoint/control uses one division by 1000.
- Declaration order, negative coordinates and relative layout survive without
  packing, inversion or origin subtraction. Literal layer names become names;
  native allocation supplies fresh IDs, first material, zero grain and default
  placement.
- Both import formats use the internal ImportedPattern shape and existing
  AddPolygon construction path. Editor executes on a clone and validates the
  complete resulting project before replacing state/history. A later command
  failure, invalid curve/contour, capacity failure or revision/ID failure leaves
  the prior document untouched.
- CLI and MCP expose the same strict typed command; dedicated source paths are
  protected against output/recovery identity and symbolic aliases, with native
  save protection refusing nonregular replacements. Discovery and schemas carry
  the new tool and 24th shared command; SVG remains present.
- Native toolbar/search actions share one review flow. The review names scale,
  layout, layer-name/fresh-ID construction scope and omissions. Inspector drafts
  and import combine into one transaction on accept; failed/cancelled actions
  retain drafts. Modal input shielding covers background pointer/keyboard and
  AccessKit actions.
- The four native tests now iterate over SVG and DXF. The pointer test also
  iterates over chooser None and review Cancel, checks both history directions
  and retained numeric drafts over subsequent frames, and then verifies ordinary
  Tab blur commits the retained edit. Existing SVG focused tests are retained.

## Evidence read, not independently rerun

The supplied local job manifest and terminal logs report:

- Core DXF/SVG import tests: 6 + 5 passed.
- Dedicated CLI import and MCP suite: CLI passed; initial MCP run preserved one
  stale 23-vs-24 discovery-count failure, with the corrected targeted test passed.
- Native pattern import tests: 4 passed, each covering both formats.
- Formatting and strict Clippy for core/CLI/native targets: passed.

These are implementation checks only. Full workspace tests, full workspace
Clippy/xtask verification, real native chooser/edit/history/save acceptance,
independent consumers and any reference comparison remain separate gates. The
candidate itself does not claim those passes or general CAD/reference parity.

Skipped: all builds, test execution, GUI and physics work as requested; no source,
registry, STATUS or publication files were modified. Existing sampled cubic
contour validation retains its documented precision limit; this review does not
establish exact CAD intersection behavior or native runtime acceptance.
