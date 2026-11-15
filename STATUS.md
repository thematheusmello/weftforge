# WeftForge status — active development, 2026-11-15

The native workspace now follows PhotoCraft’s compact charcoal editor structure:
menus, left tools, context controls, document tab, paired 2D/3D canvases,
Properties/Scene docks and slim simulation/status rows. The complete garment
studio remains the goal: **0/56 coarse reference contracts verified**, discovery
incomplete, and **no roadmap gate accepted**. The 949 observed interface entries
are an engineering inventory. See [roadmap](ROADMAP.md) and [registry](parity/features.yaml).

## Current UI delivery

The visual delivery is [PR43](https://github.com/thematheusmello/weftforge/pull/43).
Runtime source is `7ab2107aaa51f2c0fa9c9fd2ada972a08c433882`, after
[PR42](https://github.com/thematheusmello/weftforge/pull/42) main `f8be9b7`.
The [gallery](docs/studio-preview.md) shows actual unedited 1640 × 1032 captures.
The original Harbor fixture stays paused in Rest at revision 20: two pieces,
eight seams, 654 rest vertices. Only framing, selection and dropdown actions ran.
Selected Scene, Snap and boundary dropdown text is readable; Scene rows are
left aligned. No authoring, numeric entry, Save, history or physics ran here.

The [scoped UI report](parity/reports/WF-20261115-PHOTOCRAFT-STUDIO-001.json)
is a bounded implementation-only pass. All five final local gates pass from a
clean source checkpoint using that worktree’s own target:195Rust tests
(49 native), 54 Python tests, formatting, strict workspace/all-target Clippy,
xtask registry/map verification and unsigned packaging. Both embedded font OFLs
match in regular and app-bundle package locations. Debug capture and release
package identities are separate; there is no new binary release. Smaller windows,
other platforms and screen-reader use remain unverified.

Two reproduced popup regressions were fixed: open menus no longer dispatch
canvas/transport shortcuts, and keyboard menu activation no longer restores stale
numeric focus. Pointer text/caret retention and ordinary canvas shortcuts remain
covered. Independent source review found one remaining dropdown contrast issue;
the shared light selection stroke fix is followed by final gates and actual
dropdown capture. See the [packet](parity/reports/artifacts/photocraft-studio-20261115/README.md).

Source 21b’s [native 001/002 report](parity/reports/WF-20261115-NATIVE-STUDIO-001.json)
remains failed/unaccepted and is now stale. Its setup and focus-witness stops are
unchanged, as are all original garment/physics failures. This visual increment
does not rerun or transfer them. UI-001 stays partial/not_run with current bounded
visual evidence; UI-003/UI-005 retain stale prior evidence. All complete-contract,
reference, single-workflow and full-native-plan flags remain false.

GitHub Actions and automated Pages deployments stay disabled until the complete
project is done. The existing static Pages site is preserved. Local gates are
terminal; no Cargo, CUA or physical acceptance job from this UI pass remains active.

## Failures and historical checkpoints

The following records keep their original source, limits and outcomes. They are
not rerun or regraded by this publication. Full immutable reports and artifacts
remain linked; freshness can be stale while original outcomes remain true.

| Source/checkpoint | Retained result and boundary |
|---|---|
|2950/7a1 Snap|Unfinished raw text/caret lost in 9/9 headless cases; corrective 7a1 then stole keyboard focus in M:N. Final 539 fixes are source-reviewed. Original 2950 snapshot is retrospective, not a pre-edit cryptographic freeze.|
|35b14/a6f77 telemetry|Late cancel/obsolete revision masked genuine failures; RED then 5ab5 GREEN. A 10.5 px transport-layout failure remained until 221f's stable disclosure row.|
|221f/c493/21b9 intent|A superseded failure cleared replacement Step/Play; controlled RED precedes 21b9 GREEN. Actual native cancellation/error lifecycle remains unverified.|
|76be native Harbor|Intended 0.84 first observed 84.00 at 0198; actor missed the numeric stop barrier, continued and Close rejected the staged back. Failed/stopped before dynamics; cause unresolved.|
|76be DXF, PR41|Four headless geometry/export grades, 14 retention negatives, 4 exact MCP inspections and 9 native saved/export checkpoints pass their bounded scope; the full frozen native plan remains unaccepted.|
|5950 DXF|First .11 Return produced revision 3 rather than 2 before import; 4 MCP inspections failed on 3 f32 color decimals each. These original failures remain closed.|
|2e closed stitch, PR40|Five gates 177 Rust/54 Python; operator 51 tests/810 exact legacy comparisons and N32 RED/GREEN. A/pair physical screen failed; B passes only its bounded protocol.|
|32f9 SVG, PR39|Bounded explicit-mm SVG native file/history/control-edit/reopen checks pass; earlier v1/v2 failures and v3 input/observer corrections remain.|
|6c455 original Harbor|Native authoring/reopen/restoration passes bounded geometry/history checks. A/B physical attempts hit 420 s limits; accepted prefixes cannot establish final quality.|

The [source 76be garment failure](parity/reports/WF-20261115-NATIVE-HARBOR-FAILED-001.json)
selects 36 original candidates and 3 review derivatives from a reviewed 90-file
allowlist. Both declared 14-point designs are independently non-self-intersecting;
replacing only back X7 with 84 conditionally creates a crossing. This reconstruction
is consistent with rejection, not direct proof of every unsaved staged coordinate.
The failure is not evidence that the validator rejects the declared valid design.
Snapshot-time live sidecar reads are not whole-case immutability claims.

The [DXF report](parity/reports/WF-20261115-DXF-STUDIO-001.json) remains stale on
later native source. Its three gaps stay visible: printing is not explicit in
the review modal; old .12 Redo after Cancel is runtime-unverified, not failed;
temporary revision 2/.126 reached background autosave/recovery while input primaries
remained unchanged. Chooser basename, closure assertion and preparation/staging
corrections remain retained. Source 76be packet manifest starts fe82650c; its
single clean saved-output replay and later metadata checks are archival receipts,
not newly executed acceptance. APP-001/FILE-005 remain stale/partial/not_run.

The [closed-stitch report](parity/reports/WF-20261115-CLOSED-STITCH-HARBOR-001.json)
remains stale because later authoring/adapters changed its full scoped source,
although solver bytes are unchanged. Actual A peaks 19.9092% strain at frame 31 and
fails frames 28–37; B peaks 14.6118% at frame 30 and passes its bounded strict screen.
The pair is failed. Original 077 strict failures, controller launch refusal,
qualified proof reuse and saved-state/publication corrections remain immutable.
SIM-001/003/005 remain failed/stale. This increment adds no physics evidence.

Original Harbor authoring/restoration and incomplete physical prefixes are kept
in the [Harbor bundle](parity/reports/artifacts/harbor-workflow-20261114/README.md),
with [authoring](parity/reports/WF-20261114-HARBOR-AUTHORING-001.json) and
[physical failure](parity/reports/WF-20261114-HARBOR-PHYSICS-001.json) separate.
Source 6c455 A/B accepts 163/158 prefix frames before 420 s cancellation; native
Drape first post-cap observation at 422.705 s retains its overrun. No frame 180/full
quality pass follows. Later 7fe strict-trace incompleteness, A29 strain and 9–11 mm
contact failures remain in the physics evidence. General convergence, calibration,
surface contact, full persistence and full garment acceptance remain open.

## Next implementation and reference questions

UI/visual work takes priority over the old focus diagnostics. The reviewed next
feature is precise authored/rest boundary and sewing-length feedback: reuse the
existing bounded arc metric, report raw A/B lengths and signed B−A in explicit
units, and expose the same read-only query through CLI/MCP. Preserve drafts,
geometry/history and revision-aware display. Source implementation is assigned
to a separate worktree; its results are not yet accepted or published.

Then implement 2D piece translation/rotation, physical-length partial Free Sewing,
internal straight lines and a bounded topological cut. Their remaining reference
semantics stay explicit. Presence of a reference menu item alone is not executed
behavior or parity. The executed 100 mm Free Sewing pair is recorded in the
[reviewed behavior contract](spec/features/SEW-002-paired-runtime-20261114.md).

The legitimate reference baseline is provisional. The old Rectangle manual
handoff has no inferred outcome. Keep the
[dated runtime notes](spec/features/SEW-002-runtime-notes-20261114.md). Raw
commercial/account material stays private; fresh implementers receive reviewed
public behavior contracts. Old closed cases are not restarted or regraded.

Next checks: review length implementation and independent analytic/adapter
outcomes, run the required integrated gates, inspect its actual native readout,
and publish only source-bound tracked evidence. Continue full garment/contact
work under new explicit source/input/criteria bindings after these authoring
increments. No roadmap gate is accepted by a screenshot or unit-test count.
