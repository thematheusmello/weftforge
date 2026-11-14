# WeftForge status — active development, 2026-11-14

The complete independent native garment studio and recorded one-to-one parity
remain the goal. Verified parity is **0/56** coarse inventoried contracts.
Discovery is incomplete; the 650 observed interface entries are not a feature
denominator. Every remaining roadmap gate stays in scope.

## Published and recoverable state

Public repository: https://github.com/thematheusmello/weftforge.
Published main: `68bdb9fb774177f074f69edc1b2965336670cc9f`
([source PR21](https://github.com/thematheusmello/weftforge/pull/21)); all eight
push/PR platform and evidence checks passed. The README correction from
[PR20](https://github.com/thematheusmello/weftforge/pull/20) is retained. The root README is product focused.
Keep reference laboratory and account-status details in dedicated research docs.

[PR15](https://github.com/thematheusmello/weftforge/pull/15) published authored grain,
separate warp/weft stiffness, strict native v2 migration and the inspector fix.
[PR17](https://github.com/thematheusmello/weftforge/pull/17) retained the editing
pass and fitting failure separately at `ee2811f`. Its Pages deployment
`43532703043` succeeded; public HTTP 200 bytes exactly matched the committed
dashboard, including both reports. The source-only
[alpha.1](https://github.com/thematheusmello/weftforge/releases/tag/v0.1.0-alpha.1)
remains at older `8419363` and discloses the inspector defect. No native binaries
ship yet. Distribution notices, platform launch/package QA and signing remain.

Current branch: `docs/native-curve-evidence`, based on published main `68bdb9f`.
Identified native candidate source: `eb153cd`, byte-identical runtime code.
This integrates sewn-curve editing, stateless MCP, shared save/export safeguards,
shutdown retention, cooperative solver initialization cancellation, reference
qualification, the reviewed interface catalog and its implementation crosswalk.
The published README correction is merged, retaining its product wording and
the new adapter/curve usage instructions.

Contributor commits are retained through root cherry-picks: curve `c48cd958`,
shutdown `3014a6e`, export `db48813`, MCP `68521f9`, save `3c20e73`, cancellation
`7038aab`, qualification `0b054222`/`65dd3f5`, inventory `f26f0d6` and map `ed98b6b`.
The source-pinned map requires its ancestor `7e59f2a` to remain reachable; preserve
that history when merging this checkpoint. Identified historical executables and
all evidence remain retained. Only generated root Cargo caches were moved to the
external volume after low disk space; file sizes and identified binary hashes
were checked. Local target symlinks require that volume; CI uses fresh targets.

## Integrated behavior and checks

The editable core retains stable points, straight/cubic curves, pieces, materials
and oriented seam intervals with explicit correspondence. Typed transactions,
undo/redo and revision invalidation remain shared by adapters. Native boundary
selection exposes numeric cubic controls and straightness. Exact subdivision
partitions reversed, partial and gathered sewing while preserving physical point
pairs; focused analytical tests use 1e-12 m. Tiny resulting intervals fail
atomically. Joining, internal features and arc-length semantics remain open.

The f64 CPU XPBD solver uses orthotropic membrane strain, dihedral bending,
area-based mass, damping/friction, bounded-speed assembly, analytic avatar/ground
and discrete vertex/triangle self/layer contact. Initialization polls cancellation
inside contact analysis and receives the native worker's actual token. Some
validation, allocation and final diagnostic phases remain non-interruptible.
Native cancellation latency is unmeasured. Contact search remains quadratic;
full edge/edge CCD, GPU simulation and calibrated fabrics remain open.

Native v2 stores per-piece grain and separate warp/weft stiffness. Strict v1
migration preserves prior authored fields, IDs and revision. Atomic saves retain
prior valid recovery and refuse nonregular destinations. Shared export writers
protect native files at any extension and aliases. Regression cases cover
primary/recovery links, dangling links and directories. Concurrent external
writers are not coordinated.

On macOS, global Quit bypasses CloseRequested in pinned winit/eframe. App::on_exit
now commits valid drafts, drains an older autosave writer and synchronously
retains the latest dirty construction as recovery without overwriting primary.
Clean/discard behavior, fallback and failure reporting are tested. Actual
integrated focused-draft Cmd+Q/relaunch/explicit recovery now passes; forced termination
or storage failure is outside the retention guarantee.

Five stateless MCP file tools share typed authoring, strict original-byte parsing
and atomic commands. Modern 2026-07-28 and legacy 2025-11-25 profiles have real
stdio/schema/error tests. Official Python MCP SDK 1.28.1 independently negotiated
legacy, listed five tools and created/inspected original files with four decoded
responses and zero stderr on the prior integrated debug candidate `cc3dfea9…f1e8`.
The final packaged CLI `b2bc0999…a6396` passes the same official SDK smoke;
legacy interoperability is exercised, while a modern external client is pending. Live GUI automation, MCP simulation jobs,
persistent undo and cancellation remain open.

The current combined tree passes **61 Rust tests**, formatting and strict Clippy.
All 51 current Python checks (48 registry/export plus three independent
reference-geometry evaluator tests) and the 650-entry structural map check pass.
The integrated xtask rerun also passed against this checkout, including current
CLI MCP/curve help. The unsigned native package builds successfully, and its
CLI passes the official SDK smoke test at the same identified binary hash.
See [the integration report](parity/reports/WF-20261114-INTEGRATION-001.json).
The bounded native construction/recovery workflow now passes; see the separate
[native report](parity/reports/WF-20261114-SEWN-CURVE-001.json). Full garment fitting
acceptance and the new numeric-dialog candidate remain pending. A read-only independent source review
found no blocker; its 146,498 varied correspondence samples had maximum normalized
error 1.72e-13. This review did not execute native UI or replace runtime checks.

Two local verification problems were retained and fixed: parallel MCP fixtures
could receive the same temporary directory name, and a reused compiled xtask
could contain an older checkout's compile-time path. The new runtime-root selector
and unique fixture counter have regression coverage. Never share compiled Cargo
targets across worktrees; prior wrong-checkout output is not the current gate.

## Actual native evidence and known failures

Historical `f00e` / source `ff391447` completed the original empty-project shirt
workflow: canvas authoring, eight seams, arrangement, assembly/drape, edits,
save/quit/relaunch/reopen, Undo/Redo, resimulation and actual OBJ/SVG export.
Independent strain 0.08071753, seam error 2.79e-8 m and avatar/ground error about
1.50e-16 m passed that scene's limits. Reports remain retained as stale after
relevant source changes.

Native `03d440` / source `b8c9a1a` (Rust byte-identical to published `28cae66`)
passed the declared grain/history/shared stiffness/color-only/v2-reopen/export
subset. Its anisotropic shirt **failed** the unchanged fitting limit: axis strain
0.26930346 > 0.15; seam error 5.55e-6 m < 0.005 m and avatar/ground error 0 <
0.002 m passed. A headless repeat exits 1 with matching strain within 1e-12;
three simulated seconds took 22.19 seconds wall time. See
[issue16](https://github.com/thematheusmello/weftforge/issues/16).

Original tight-shirt/refinement failures remain in
[issue8](https://github.com/thematheusmello/weftforge/issues/8). Its 0.96 m body
circumference is below the 1.131 m torso; refinement alone did not establish
convergence. A separate relaxed sample passed its explicit protocol. No fixture,
tolerance or material substitution was made to turn a failed report into a pass.

The `1ed6d1` curve probe created one panel and exposed the global-Quit problem;
it did not complete acceptance. Follow
[issue18](https://github.com/thematheusmello/weftforge/issues/18) for actual recovery,
[issue19](https://github.com/thematheusmello/weftforge/issues/19) for native-save
link preservation and [issue9](https://github.com/thematheusmello/weftforge/issues/9)
for t-versus-arc-length sewing semantics. Retain aborted, failed and stale runs.

## Live reference investigation and handoffs

The actual garment workspace is accessible. REA 6.9.0 observes Personal
2026.1.237 (r58920), executable `e7696388…c76e`, two workspaces and native panels.
Baseline revision 1 remains provisional/unfrozen: selected global units, numeric
locale and relevant preferences are still being recorded. Initial Graphics,
Grid and Simulation Properties observations are retained without treating current
values as universal defaults. Raw account and commercial records stay private.

The reviewed catalog has 589 menu nodes across 17 menus and 61 components.
The [implementation crosswalk](spec/inventory/weftforge-implementation-map.md)
assigns all 650 unique IDs exactly once to 184
bounded packets and 36 source landmarks, linking every coarse registry feature.
It is a source map, not a second status registry. It excludes only two vendor
activation/session administration entries; hosted creation/local substitutes and
all unresolved actions remain visible. Dialogs, contextual variants, ranges,
errors and workflows still require execution.

A guarded ordinary native pointer route delivered a recorded rectangle gesture.
An original 200 × 300 mm rectangle was created numerically, removed/restored by
one Undo/Redo, saved, closed and reopened. Independent OBJ measurement found
200.000015 × 300.000122 mm: maximum error 0.000122 mm is below the predeclared
0.01 mm limit. Raw native/OBJ/MTL files remain private. Process restart,
constructor limits/cancellation, resolution editing and wider semantics remain
pending. These observations establish no WeftForge comparison or parity pass.

Reference qualification pins technical identity/revision, behavior settings,
reviewed contracts, scenarios, fixtures, criteria and transitive dependencies
against both tested commit and current inputs before any reference pass qualifies.
Historical implementation reports remain attributable.

## Latest bounded native result and next steps

Native `4fbd66af…77c9d` completed original rectangle/cubic/reversed-sewing creation,
split/history/rejected edits, refinement, clean quit/relaunch/reopen and actual
OBJ/SVG export. Independent full-curve and sewn-pair bounds are 1e-16 m; failed
edits preserve all bytes/revision; meshes increase 207 → 2330 vertices with
positive faces. A still-focused valid grain draft survives global Quit into
recovery, leaving primary unchanged. Explicit recovery and subsequent cubic
editing preserve exactly the intended fields. See the native report and retained
fixtures; no simulation, complete feature or reference-parity claim is made.

1. Publish these reviewed native fixtures, evaluator, capture provenance and report;
   preserve old failed/stale reports and the product-focused README.
2. Integrate reviewed inventory branch `docs/feature-map-expansion` by preserving
   its `9acb265` catalog/source anchor and `07d0be1` map commit. It expands the
   published 650 entries to 755 entries / 193 packets, with old mappings intact.
   Reference `1674738` adds an independent boundary evaluator/report; later PD10
   history/export/restart observations are being finalized separately.
3. Native rectangle work remains isolated on `feat/rectangle-numeric`: `51c71f3`
   numeric modal, `aefa887` stable widget scopes, `691ad146` accessibility modal
   isolation. Focused-draft history replay is confirmed in issue23 and its fix is underway. Do not merge
   until reviewer findings are fixed, combined gates pass and native acceptance
   is recorded. This source is not the native-tested 4f candidate.
4. Reference_lab owns the native UI lease. The original PD20→PD10 rectangle edit,
   one Undo/Redo and export/boundary checks passed; normal restart temporarily
   exposed a black window, then progressed without another restart/account action.
   PD10 reopen target fields persisted, with explicit AX truncation/attribute gaps.
   Baseline settings and broad sewing/workflow discovery remain incomplete.
5. Continue every construction, solver/contact, animation, avatar, rendering,
   interchange, printing, automation and delivery gate. Verified parity stays zero.
   GitHub Project scopes remain unavailable; issues, milestones and registry work.

Root is retaining the native evidence and relocating only completed generated
build caches after internal space fell again. Completed cache relocation session `62360` verified file sizes and identified
binaries; internal free space was 2.3 GiB afterwards. No root build/test is live. Reference_lab and Rectangle_numeric are active; map and
independent-output agents completed. Integrated_review's two concrete new-UI
findings are assigned to the worker. Root uses no desktop input during the
reference lease. The saved old 4f app remains running; do not overwrite its package
executable before a normal clean quit when the next UI lease is acquired.
