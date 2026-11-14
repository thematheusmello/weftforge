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
Accepted bounded native candidate source: `359ac5b2fedfece5cfef9f22861c279bd2708801`;
native executable `3ed0ab22b4e7335c8dff9a58350b7f68eb2aa9b5ba1c39059c9043a7210c6880`,
CLI `3b0977567273c57d74edaf2ac470e0740b55fb2f7a3fec56e53b8b0463aab32f`.
Runtime/test sources remain identical after the later evidence/catalog merges.
This integrates the numeric rectangle dialog, stable widget scopes, modal input
isolation, focused-history/RGB fixes, precision-preserving numeric editing,
valid small/large size preservation and initial-contact BVH. The current map has
902 entries; the executable was packaged at its 893-entry source snapshot.

The preceding `f348223d` / `dab0f0f3` candidate and its precision failure remain
retained in [the failed run](parity/reports/WF-20261114-NUMERIC-RECTANGLE-001.json).
New bounded [main acceptance](parity/reports/WF-20261114-NUMERIC-RECTANGLE-002.json)
and [batch cancellation](parity/reports/WF-20261114-BATCH-CANCEL-001.json) reports
pass their declared subsets and retain original/invalid/failed states.
No complete feature, fitting, release-ready binary or reference parity is claimed.

Preserve merge ancestry: source-pinned maps retain `7e59f2a`, `9acb265` and
`3548de48983bd1a897c0a6433771cd695398ee60` and `13f1d81d238801da617f68525660ae6fa1d01500`.
Measured benchmark source `461f2e2` is also retained. Do not squash away these anchors.
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
Some allocation/validation/final phases remain non-interruptible. Actual native batch cancellation acknowledged a click in 890 ms including CUA/AX
overhead; exact preparation/step phase and worker exit latency remain unmeasured.
Edge/edge CCD, GPU cloth and calibrated fabrics remain open.

Fresh serial before/after measurements on an original 15,459-vertex / 30,488-face
fixture reduced median initialization diagnostics from 5.618584042 s to
0.056252833 s (99.88×). All six runs retained the same 71,927 intersections and
0.00536994076963222 m maximum error. Exact committed sources, executable hashes,
producer and original raw timing output are retained; the portable public bundle
and reproduction procedure are in [the benchmark](docs/initial-contact-benchmark.md). This is
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

The combined runtime code passes **73 Rust tests**, formatting, strict Clippy and
`cargo xtask verify`: **54 Python checks** (48 script and six independent
reference evaluator tests), the **893-ID** packaged snapshot and current CLI help.
The final combined evidence gate passes all54 Python checks, the **902-ID** map
and registry after public artifact packaging. Its unsigned
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

The corrected `3ed0ab22` candidate passes the original main construction slice:
exact 200 × 300 mm OBJ/SVG, independent fields, Cancel/error protection, one
Undo/Redo, stable creation values, full cubic controls and focused grain history.
Untouched cubic X values equal their original f64 inputs; independent rational
errors are below 6.67e-17 m. The native 5 × 6 mm rectangle survives untouched
focus/Tab/blur and invalid-zero restoration. Four saves, including clean
Quit/relaunch/reopen, are byte-identical at revision12. Final exports contain
475 vertices / 846 positive triangles and four editable SVG paths; full cubic
bounds are below 2.83e-16 m. Materials remain unchanged.

A stale automation helper briefly reopened the old build; its mixed-instance
attempt is explicitly invalid and retained. A double-click on a moving field
left an earlier Y-edit incomplete; that state is also retained. Explicit app
arguments and an observed single click completed the corrected target; neither
aborted attempt is credited as a completed workflow.

A separate prepared original 15 mm shirt exercised native batch cancellation:
15,459 vertices, eight seams, active zero-gravity assembly, Cancel acknowledgement
and retained rest arrangement. Native saves and actual OBJ exports are byte-identical
before/after cancellation. The 890 ms UI measurement includes automation/AX
overhead and does not measure worker exit or constructor-only cancellation.
Its v1-to-v2 save has expected schema migration plus two 1e-16 m coordinate
deltas. Those are below the unchanged 1e-12 m criterion, but exact source-coordinate
equality is false; [issue26](https://github.com/thematheusmello/weftforge/issues/26)
tracks parser precision. No bit-perfect migration claim is made.

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

The current reviewed catalog has **589 menu nodes plus 313 component records**.
The [crosswalk](spec/inventory/weftforge-implementation-map.md) assigns all
**902 unique IDs** once to **202 bounded packets and 52 source landmarks**,
retaining all prior entries. This maps observed surfaces to implementation work;
it is not a feature denominator or a parity pass. All 893 previous entries and
201 packets are preserved; the new sewing packet covers first-side preview only.

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

1. Publish the completed source/evidence checkpoint and require all eight CI
   checks before merging. Local final evidence gates pass. Preserve the failed
   and invalid attempts, source ancestry and the product README.
2. Keep native preparation-phase timing, worker termination, JSON float precision,
   complete garment fitting and reference comparison gates open.
3. Continue actual reference whole-edge/partial sewing, preferences, contextual
   dialogs/ranges/errors and end-to-end workflows; map each new observation batch.
4. Continue every construction, simulation/contact, animation, avatar, rendering,
   interchange, printing, automation and delivery gate. Verified parity stays zero.

Reference_lab owns exclusive desktop input for the next original sewing workflow.
Root's saved own candidate PID79057 is idle on the cancellation fixture; do not
overwrite its isolated package. The read-only reopened-small capture has 94 nodes,
no gaps/truncation and a matched executable digest. Curve_output_check is packaging
independent public evidence (complete); Integrated_review checks attribution and reproduction.
No root Cargo job is live. External generated caches are retained; do not remove
unrelated files or change system protections.

A separate solver review found no justified production math change for issue16.
Its two analytic regressions and review note pass 22 sim tests/strict Clippy on
isolated commit `c248903655a4c30dbab93d004f9c0229101e3f22`; they are not merged
into this 73-test native checkpoint. The fitting failure remains unresolved.
