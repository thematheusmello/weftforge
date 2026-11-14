# WeftForge status — active development, 2026-11-14

The complete independent native garment studio and recorded one-to-one parity
remain the goal. Verified parity is **0/56** coarse inventoried contracts.
Discovery is incomplete; observed interface entries are not a feature denominator.
Every remaining roadmap gate stays in scope.

## Published and recoverable state

Public repository: https://github.com/thematheusmello/weftforge.
Published main: `d34c9d18a2f4c672eb3130f2c80f24d59c4f95ce`
([native evidence PR24](https://github.com/thematheusmello/weftforge/pull/24));
all eight push/PR platform and evidence checks passed.
[Source PR21](https://github.com/thematheusmello/weftforge/pull/21) is retained
as ancestor `68bdb9fb774177f074f69edc1b2965336670cc9f`.
The root README remains product focused following
[PR20](https://github.com/thematheusmello/weftforge/pull/20). Reference laboratory
and account-status details belong in dedicated research documents.

The source-only
[alpha.1](https://github.com/thematheusmello/weftforge/releases/tag/v0.1.0-alpha.1)
remains at older `8419363` and discloses its inspector defect. No native binaries
ship yet. Distribution notices, platform launch/package QA and signing remain.
Historical tags and failed or stale evidence have not been rewritten.

Current branch: `feat/numeric-rectangle-and-contact-init`.
Frozen local candidate source: `f348223d5dadfaa7312cb93c1f2ec09a0e7e8dec`;
native executable `dab0f0f3fbda1e778dde45c230bd944cf140b9c99a86e9376d0078ac6e80bff3`,
CLI `3b0977567273c57d74edaf2ac470e0740b55fb2f7a3fec56e53b8b0463aab32f`.
This unpublished candidate integrates the numeric rectangle dialog, stable widget
scopes, modal input isolation, focused-history/RGB fixes, initial-contact BVH and
893-entry implementation map. Native investigation found a separate numeric
focus/blur precision defect; this candidate is not fully accepted.

Preserve merge ancestry: source-pinned maps retain `7e59f2a`, `9acb265` and
`3548de48983bd1a897c0a6433771cd695398ee60`. Do not squash away these anchors.
Only generated Cargo caches were relocated to the external volume; file sizes
and identified binary hashes were checked. Local target symlinks require that
volume. CI uses fresh targets; compiled targets must stay separate by worktree.

## Integrated behavior and checks

The editable core retains stable points, straight/cubic curves, pieces, materials
and oriented seam intervals with explicit correspondence. Typed transactions,
undo/redo and revision invalidation remain shared by adapters. Subdivision
preserves reversed, partial and gathered physical point pairs at parameter t;
joining, internal features and arc-length semantics remain open. Undo retains
the allocator high-water mark intentionally, preventing reuse on abandoned history.

The f64 CPU XPBD solver includes orthotropic membrane strain, dihedral bending,
area-based mass, damping/friction, bounded-speed assembly, analytic avatar/ground
and discrete vertex/triangle self/layer contact. Initialization polls the native
worker's cancellation token. The new BVH accelerates only initial static contact
diagnostics; live collision projection and step diagnostics remain quadratic.
Some allocation/validation/final phases remain non-interruptible. Actual native
cancellation latency, edge/edge CCD, GPU cloth and calibrated fabrics remain open.

Fresh serial before/after measurements on an original 15,459-vertex / 30,488-face
fixture reduced median initialization diagnostics from 5.618584042 s to
0.056252833 s (99.88×). All six runs retained the same 71,927 intersections and
0.00536994076963222 m maximum error. Exact committed sources, executable hashes,
producer, driver and original raw timing output are retained locally. This is
initialization evidence, not live-simulation speed or fitting evidence.

Native v2 stores editable construction and strictly migrates v1. Atomic save and
export guards preserve primary/recovery files and reject nonregular destinations
and protected aliases. Concurrent external writers are not coordinated.
On macOS, global Quit commits valid drafts and retains the latest dirty state as
recovery without overwriting primary. Actual focused-draft Quit/relaunch/recovery
passed on the prior identified candidate. Forced termination and storage failure
remain outside the retention guarantee.

Five stateless MCP tools share validated Rust authoring and file operations.
Modern 2026-07-28 and legacy 2025-11-25 profiles have stdio/schema/error tests.
Official Python MCP SDK 1.28.1 negotiated legacy, listed five tools and
created/inspected an original file on the new identified CLI, with four decoded
responses and zero stderr. A modern external client, live GUI control, MCP
simulation jobs and persistent undo remain open.

The frozen combined code passes **70 Rust tests**, formatting, strict Clippy and
`cargo xtask verify`: **54 Python checks** (48 script and six independent
reference evaluator tests), the **893-ID** map and current CLI help. Its unsigned
native package builds. Subsequent source changes require appropriate new checks.
Earlier temporary-directory collisions and compile-time xtask root selection were
fixed with regression coverage; wrong-checkout output is not current evidence.

## Native evidence and known failures

Published report [WF-20261114-SEWN-CURVE-001](parity/reports/WF-20261114-SEWN-CURVE-001.json)
records native `4fbd66af…77c9d`, runtime source equal to `68bdb9f`: original
rectangle/cubic/reversed sewing, subdivision, Undo/Redo, rejected edits,
refinement, clean restart, actual OBJ/SVG export and focused-draft Quit recovery.
Independent full-curve and sewn-pair bounds were 1e-16 m. Primary bytes survived
Quit; explicit recovery and further curve editing preserved intended fields.
It establishes a bounded workflow, without simulation or complete-contract
parity, and is stale for relevant source changes in the current branch.

New native `dab0f0f3` has exercised the 500 × 500 mm rectangle dialog, independent
200 × 300 mm entry, Cancel, zero/NaN rejection, disabled background controls,
one Undo/Redo and actual OBJ/SVG export. Independent measurements found exact
200 × 300 mm geometry, 43 vertices / 68 faces and a closed editable SVG path.
Cancel/error bytes and material data are unchanged. Two additional 0.4 × 0.6 m
rectangles retain creation fields through dynamic curve controls. Focused grain
Undo, subsequent blur and Redo preserve exactly the intended document fields.

The same run exposed an **unaccepted precision defect**: Tab focused an untouched
cubic P2 X field; leaving it rounded 2.1333333333333333 to 2.13333 m. Original
fixtures and the independent failing result are retained, without normalization.
A separate source worker is fixing shared numeric focus/blur handling in
[issue25](https://github.com/thematheusmello/weftforge/issues/25). Clean Quit,
relaunch and reopen preserve the actual saved bytes, including that defect.
Restart and cancellation acceptance remain pending for the final fixed build.

Older original shirt workflow `f00e` / `ff391447` passed its declared fitting
limits: strain 0.08071753, seam 2.79e-8 m, contact about 1.50e-16 m. It is stale.
The later anisotropic shirt **failed**: strain 0.26930346 > 0.15, despite passing
seam/contact limits; the headless repeat matched within 1e-12. See
[issue16](https://github.com/thematheusmello/weftforge/issues/16).
Tight-shirt/refinement failures remain in
[issue8](https://github.com/thematheusmello/weftforge/issues/8); its 0.96 m body
circumference is below the 1.131 m torso. No fixture or tolerance was substituted
to turn a failed result into a pass.

[Issue18](https://github.com/thematheusmello/weftforge/issues/18) is closed by
native recovery evidence; [issue19](https://github.com/thematheusmello/weftforge/issues/19)
is closed by shared file guards. Creation-field identity and focused-history
fixes are local pending publication. Parameter-versus-arc-length sewing remains
in [issue9](https://github.com/thematheusmello/weftforge/issues/9).

## Reference investigation and implementation map

REA 6.9.0 observes the installed Personal 2026.1.237 (r58920), executable
`e7696388…c76e`. Baseline revision 1 remains provisional: selected units are
Millimeter, but numeric locale, language and wider behavioral preferences are
not fully frozen. Raw account/commercial records and vendor artifacts stay private.

The current reviewed catalog has **589 menu nodes plus 304 component records**.
The [crosswalk](spec/inventory/weftforge-implementation-map.md) assigns all
**893 unique IDs** once to **201 bounded packets and 50 source landmarks**,
retaining all prior entries. This maps observed surfaces to implementation work;
it is not a feature denominator or a parity pass. A later nine-record sewing
observation batch is retained separately, not yet integrated into this map.

Original reference rectangle entry, Undo/Redo, save/reopen and PD20→PD10 edits
were executed. Independent exports stayed within the declared 0.01 mm dimension
limit: 290→1127 vertices, with separately checked closed boundary topology.
Normal PD10 restart temporarily showed a black window before resolving in the
same process; final target fields persisted, with explicit AX gaps/truncation.
An original two-panel curved fixture exists. First-side free-sewing selection
is preview evidence only; the paired/reversed seam remains unconfirmed.

Reference qualification still requires technical identity, behavior settings,
contracts, scenarios, original fixtures, criteria and transitive dependency pins.
No current report qualifies as a complete reference parity pass.

## Active work and next steps

1. Fix and independently review numeric focus/blur precision. Retain the failing
   candidate; rerun affected native scenarios on a newly identified executable.
2. Complete save/relaunch/reopen and measured native initialization cancellation
   acceptance; publish original fixtures, checks and explicit limits.
3. Publish the combined source, reproducible initialization timings and expanded
   map with all eight CI checks passing; preserve source-anchor ancestry.
4. Continue reference sewing, preferences, contextual dialogs/ranges/errors and
   end-to-end workflows. Integrate the next catalog batch with a complete map.
5. Continue all construction, simulation/contact, animation, avatar, rendering,
   interchange, printing, automation and delivery gates. Verified parity stays zero.

Root exclusively owns desktop input. Reference_lab completed a coordinated
read-only capture: 107 nodes, no gaps/truncation, identified digest matched.
Numeric_precision owns an isolated
source worktree; Curve_output_check independently evaluates original native files.
No root Cargo build/test is live. The relaunched saved app PID16510 remains open; do not
overwrite its isolated executable. Low internal disk space requires external
generated build caches; do not remove unrelated files or change system protections.
