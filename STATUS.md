# WeftForge status — active development, 2026-11-16

The last published runtime is ad7cc8bbbfc7a1ae735a400c21dcc7a37c80a5c5, merged in
[PR #47](https://github.com/thematheusmello/weftforge/pull/47) at
bbada3f7f5228a9479a76c0c300d3aa716e57d77. GitHub Actions remains disabled and the
existing Pages configuration is preserved. The source/evidence
publication candidate retains **0/56 coarse reference contracts verified**,
**40 partial / 16 planned**, discovery incomplete and **no roadmap gate accepted**.
The historical 949 entries / 205 packets / 56 landmarks remain an engineering
inventory, not a complete product denominator.

## Current implementation: first straight piece cut

The straight-cut runtime is complete for its bounded engineering subset; its
publication branch is `feat/straight-cut-publication` in a separate worktree.
Core/CLI and native UI have separate owners using the
[reviewed engineering contract](spec/features/PAT-006-straight-cut.md). Original
R240/L fixtures, literal geometry/ID/mass oracles and bounded R author/restart
native procedures were declared before implementation. These are engineering
criteria; no reference cut behavior or implementation acceptance follows from them.
The original native-check worktree stays unchanged at db8/ad7. Its annotation C001
case retained a prospective Select-tool correction and 170+1 operations / 1200 s
caps. C001 stopped on an observer setup error before input. C002 reached R6, then
its single drag produced no expected R7; all four required saved states are
missing. Both windows were separately closed after retention. These private
checks supply no new native acceptance; pointer delivery/origin remains unresolved.

Core/CLI package checks passed 136 tests and strict Clippy; native headless checks
passed 116 tests and strict Clippy. Independent source review found one opening-
Return defect, fixed with two RED/GREEN regressions: opening review preserves
construction and only a later Return applies a cut. The source review found no
remaining blocker. Actual R240 meshes/initial cloth/analytic mass pass at
0.005/0.07/0.5 m; L passes those checks at 0.07/0.5 m. At L 0.005 m, both original
and cut meshes hit the existing vertex cap; cut eligibility still passes and no
fine-L cloth/mass acceptance is claimed. Caps and tolerances are unchanged.

The clean runtime checkpoint19b1e88245dc627cfec8164f4924b335fe44853d passed
workspace formatting, **304 Rust tests / zero failures**, and strict
workspace/all-target Clippy. All69 runtime/build/test/doc/contract inputs remained
exact through those checks. Source-map/registry integration, clean xtask validation
and the native build also passed at metadata checkpoint 982ad03.

Actual native CUT-R1/R003 passed its separate raw-UI review and one-shot saved-file
assessment: empty authoring, precise refusal/caret/cancellation, one accepted cut,
four native saves, Undo and Redo. CUT-R2/R2-001 opened that actual accepted R4 in a
fresh process, edited only child #15 to height 0.40 m, saved R5 and inspected the
unchanged retained piece. Its separate review and one-shot assessment passed the
original barriers. R2 retains a contradictory extra draft-only metadata flag;
the canonical accepted dependency files/hashes were present before launch, and
the clarification and qualified acceptance remain visible. Neither case was
repaired or regraded. Both processes quit and exit was separately corroborated.
See the [bounded report](parity/reports/WF-20261116-STRAIGHT-CUT-001.json) and
[native gallery](docs/studio-preview.md).

PAT-006 is now partial; whole-contract verification remains not_run and reference
behavior unobserved. Full cuts, holes, target sewing, Cut & Sew, native concave and
failed-I/O cases remain open. Saved analytic area/density is not a native particle-
mass or physics acceptance. Source and case originals remain isolated. Finished
cut windows are closed and their jobs are terminal. Root's next publication gate
is privacy-safe evidence, Git blob tracking and clean registry/map/xtask checks.
The separate strict-v5 straight-hole candidate has passed 329 Rust tests and local
source gates; its native acceptance remains unexecuted and is outside this cut
collection. Continue that workflow after the bounded cut publication checkpoint.
Prior reports with changed scoped source are marked stale without altering their
original results, tested commits or artifacts. Fresh default targets stay separate across worktrees.

## Prior straight-annotation evidence — bounded headless and native A

The [implementation-only report](parity/reports/WF-20261115-INTERNAL-ANNOTATIONS-001.json)
retains its original passed result for the 86-observation headless run and
root-accepted native A003; it is now stale on the straight-cut candidate. PAT-007, UI-004 and APP-001 remain partial/not_run for their coarse
contracts. APP-001 attribution covers the exercised shared-command/input/history
adapters and A's empty launch; it supplies no native New/Recover, full automation
or process-restart acceptance. Straight annotations add no cut/hole credit;
PAT-006's partial status comes from the separate bounded cut report above.

The source implements piece-owned straight annotations, native two-point creation,
precise endpoint editing, selection/removal, applied lengths and one transaction
with valid staged properties. Strict native v4 stores ownership; supported v1/v2/v3
reads add empty internal lists. Annotations remain separate from exterior topology,
sewing targets, mesh constraints and simulation. Internal cubics/shapes, notches,
persistent measurements/constraints, folds, general cuts and internal sewing remain open.
The [approved engineering contract](spec/features/PAT-006-internal-segments.md)
provides policy rather than reference behavior despite its PAT-006 filename.

At the published checkpoint all **65 runtime/build/test/contract/doc inputs**
were byte-identical to ad7.
Original formatting, **282 workspace Rust tests / zero failures**, strict
workspace/all-target Clippy and native+CLI debug builds are reused by exact source
identity. Their manifest SHA is
d47c3a7c35e89e8b9e0688703b8e3e97120e3fb3f3c14e48a4ea038eb643c872.
Source runner PID 56057 completed. The retained clean db8ee77 final local gate
passed registry/maps, 48+6 Python tests and CLI help in 114.020357458 s; its
original-worktree CLI feature-context change remains disclosed. The fresh
[publication gate](parity/reports/artifacts/internal-annotations-20261115/publication-checks-af69ed2/CHECKS.json)
on clean af69ed287776345aa6da09566b1ca4f1f32a3c95 passed registry/maps, 48+6 Python
tests and CLI help in 123.400316792 s. It used this publication worktree's fresh
default target, with no copied cache/target or native build. The new CLI-help
context is 0e537ecad1de86d05593d1c40e0276b0b32ef064ba0786040cb254b53f14d44d;
it does not replace the evaluated immutable CLI. The later versioned receipt and
metadata-only closure keeps that checked-commit attribution. Rust/native/headless/
saved-grader and physical consumers are not rerun for publication.

The original independent v2 run001 passed **86 observations / 434 paired target
processes**. Root collection verified all 3,100 original regular files, three
relative link nodes and 65 source pins. All 868 raw target stdout/stderr files and
link target bytes remain exact. The immutable evaluated CLI stays
71ef84c7c90e9ffdf291f3dd2e031a476ff8b99097258256bf46e17afeb8b2bd, mode 0555.
The separate original-worktree help-context CLI is
8a370548e341e4c1a2fc760b276425f5900e7a137d37cadeb268d752d1fc3a41;
it is not substituted for the accepted independent artifact.

Native A003 uses executable
a733c9952ff060a3ec96417467be50011a16039dbb0868fd8ac9890a2b5897c4.
The root [bounded acceptance](parity/reports/artifacts/internal-annotations-20261115/native/ROOT-ACCEPTANCE-A003.json)
records 142 returned pairs plus one summary, a conservative elapsed upper bound
641.657843875 s within 175+1 / 1200 caps, six exact native v4 saves and PID 13707
absent after planned quit. Root directly checked fourteen keyframes; the separate
independent review checked all 60 original images, 142 pairs/receipts, 701 original
manifest entries, 65 source / 143 new-freeze / 523 old-provenance pins and snapshot/recovery identities.
The unchanged saved grader ran once and passed all six states.

A starts empty, creates the unit square at R1 and curve #13 at R2; first-point
state remains transient. Invalid `1e-` refusal, modal Save block, backdrop Cancel,
explicit Cancel and `0.470e0` caret retention preserve R2. One compound endpoint
and grain edit reaches R3, Undo R4 and Redo R5. Full initial getApp AX is missing;
transient marker is partly cursor-occluded; immediate/exactly-once summary relies
on actor/tool history and stored timing bounds. Full accessibility, held-state,
strict same-frame, restart, B–E, full annotation, reference, physics and roadmap
acceptance are not established. See the [actual UI gallery](docs/studio-preview.md).

A001 remains a zero-execution setup error. A002 remains stopped/unaccepted after
its old procedure missed intended backdrop cancellation, with its one failed
saved grade unchanged. Neither is repaired or regraded. Later cleanup and changed
mutable autosaves have separate receipts; exact pre-cleanup snapshot copies are
retained and the later mutable sidecars excluded. The original native worktree
remains untouched for separately bound native checks. B–E coverage remains
outside this publication packet. Historical B001 initial checkpoint: actor clock
2026-11-15T22:33:46.268Z, owned PID 39198, six pairs / 47.767 s / zero errors, at R5
with one piece, line #13, 571 vertices and 10 mm snap before authoring.

Root later confirmed B001 terminal after planned Quit: PID 39198 absent, one
formal saved-state grade passed all nine states. Separate bounded B acceptance
is private, receipt SHA
2bba3c58feea568aeebd269d3bdca5c7cf2a21f6fe2f4aeab93cf275387a93db,
after direct fourteen-keyframe review and independent review of all 67 images /
155 pairs, plus 65-source/native/clean-worktree/save/grade/PID checks. None of that B evidence or
acceptance is imported into either current report or artifact packet; C–E remain
open. A separate desktop inventory found no running WeftForge app, while browser
inventory was unavailable. Raise002 timed out during compaction before any Raise
or authoring. These are separate post-case/tooling facts that rewrite no frozen clock or
original outcome; no active UI lease is implied and no new publication run or
grader is registered.

## Separate failed Harbor half-step diagnostic

The [diagnostic-only report](parity/reports/WF-20261115-HARBOR-HALF-STEP-001.json)
is failed/current solely for one declared A16×32 experiment. SIM-006 remains
partial/failed; no convergence, calibration, sufficient fix or new defaults are
established. The retained A8 control and failed A/pair results are unchanged.

Corrected freeze 58733a8 / GO c76dfd8 bound one run; exact physical rest, initialized
masses and unchanged inputs match the retained A control after only empty v4
internal arrays. All 181 states / 180 steps and 31 FINAL-bound outputs were
reviewed. Quality remains **failed**: peak 0.1654700550371433 (**16.5470055%**) at
frame 29, front/piece 2, warp, triangle 79; strain failures 29–30. Both declared
path-sensitivity triggers fire: maximum same-frame strain difference
0.08070415883928184 and nodal difference 0.02490758863765885 m. These support this
one path-sensitivity diagnosis, not convergence or a physics fix.

Whole-case 285.18800870806444 s fits 420; stage exits are 0/0/2/0. Consumer 2 explicitly
reports failed quality despite operational completion. FINAL original SHA
9dbc231b433b539306d6229bed16bb056ab4920ece70d816bd3cb980da905397
and the independent saved-record review remain bound in the
[separate packet](parity/reports/artifacts/physics-path-sensitivity-a16-20261115/README.md). Controller 0 is producer/parent observed,
consistent with FINAL/code; its later terminal copy is explicitly retrospective
post-run-appendix-40084, not an in-situ receipt. Unexecuted preparation001/f7's
deadline blocker and prospectively corrected preparation002 review remain.
No driver binary or target/cache is published. Publication performs no new
physical run and changes no production default.
No native/reference/contact-completeness or sufficient-fix credit follows.

The current source-only addendum records 30 command variants / 29 nontransaction
schemas/nine MCP tools. Historical JSON remains unchanged. Old Transform pointer,
New/recovery, garment-contact and physics gaps below remain visible. Keep only
active test windows; close finished windows after retaining evidence and record
post-case cleanup outside frozen clocks. Actions remain disabled and the existing
static Pages site is preserved.

## Historical bounded physical sewing checkpoint (B657)

The [implementation-only report](parity/reports/WF-20261115-PHYSICAL-LENGTH-SEWING-001.json)
retains its original **passed** result for the original headless scope and
root-accepted native v3 case002, and is now **stale** on ad7. Its covered coarse
contracts remain partial/not_run; current feature freshness is recorded separately
in the registry, including the new bounded annotation scope above. No full sewing,
New Empty, garment, physics, reference, process-restart or roadmap acceptance follows.

The shared command resolves oriented intervals from exact Start t and positive
physical length using the existing authored metric, then stores an ordinary
physical-arc-length group. A stays transient; B and valid staged property edits
commit as one transaction. Native boundary/Start t/direction/Length controls,
revision-bound previews and failure/cancel text/caret retention are implemented.
Length input chooses creation endpoints, not a persistent dimension lock.
Legacy Sew edges and M:N behavior are retained; internal sewing, full chains,
general native easing and sewn simulation remain open.

At that publication, all **60 runtime/build/test/contract inputs**, including four
contracts, were byte-identical to B657 through the PR45 metadata merge, then-current
source-map addendum and later window-policy change. The original build manifest SHA is
a42c5d9b88d678068a1ab804aa480f41600a93cbd03de847f48faaa5fbdcb0fc.
Its six commands exited zero with **254 Rust / 54 Python** passes, four font-notice
copies and twelve package files. The runner predicted 255: its prediction-match
flag is **false** and retained. Zero exits do not mean every bookkeeping expectation
passed. That publication reused those Rust/source gates by exact identity. Its
[clean-candidate verification](parity/reports/artifacts/physical-length-sewing-20261115/publication-checks-d670aa7/CHECKS.json) at d670aa7 passed registry, historical/current maps, 48 script tests, six evaluator tests and CLI help. The later receipt/metadata-only closure preserves those checked-commit attributions.
Debug/release CLI feature contexts remain disclosed; unsigned packaging is not
packaged launch, signing, a new binary release or cross-platform acceptance.

The original independent attempt002 passed **50 observations**: 36 positive
adapter runs, 96 rejection preservations, 18 writer failures, ten mesh configurations
and three same-file paths. Its **46** core/CLI inputs matched B657 and accepted CLI
fa4add99646544a052b6fa534ad325f1528e0572e4ee9aae344d687b85ac5cc1 is a separate immutable
artifact. Wrong-working-directory/source-context preflights stopped before candidate
execution and remain retained. Three writer-output symlinks remain relative,
typed mode120000 nodes with exact target strings, not flattened fixture copies.
No headless evidence is borrowed as changed-GUI acceptance.

Native002 uses executable
69af2254e9f1346dbd9abf018d66ff731eea4660b2efd71df195916c8b195cd0.
Root verified 732 original manifest entries, all 161 returned pairs/action arguments
with zero errors, 60 source and 20 frozen pins, twelve raw UI images and final PID
absence. Including summary, **162 calls / 748.816649 s** fit unchanged 220 / 1200
caps. The actor OUTCOME remains pending root review; the later ROOT-ACCEPTANCE-002
records bounded acceptance without rewriting it.

Length 1e- refusal and Cancel retained R41; the unrelated width/caret witness became
0.20e0 without an accepted change. Curve #6, Forward, 100 mm alone stayed transient at R41;
curve #15, Reverse, Start t 0.75 and 100 mm completed group 20 at R42 with two 100.0 mm labels/guides.
One Undo at R43 removed it and Redo at R44 restored it. File → New Empty produced R0/zero authored
construction and no-op Undo/Redo, then same-process reopen restored R44/group 20.
The single frozen saved-output grader passed its unchanged 1e-8 m parsed-field/metric
scope. All four native primaries retain first saved hashes; Redo and reopened are
byte-identical (3d0aa9cd93300ecfe69a0798363c9ace70bf2e4e36fd4ba68499de822348c08f).
Full AX accessibility, general source picking/reference behavior and full New are
not established. See the [actual UI gallery](docs/studio-preview.md).

Failed001 remains stopped before Save at garmentcreated.weftforge, with none of
four required primaries. Its autosave is not an acceptance substitute. The isolated
filename calibration accepted only one documented setValue/Cancel, without Save or
sewing credit. Frozen v1 is unexecuted; v2 belongs to failed001 and prospectively
corrected tooling v3 to fresh002. Original math/grader and all old outcomes remain
unchanged in the [tracked packet](parity/reports/artifacts/physical-length-sewing-20261115/README.md).
Original 5655 source-review findings, separate REDs/C4be fixes and B657 integration
checks are retained. Publication selects exact JPEG/project/sidecar bytes and only
literal workspace/home normalization in text; named owned/pair derivatives exclude
raw global/kernel/chooser material. No executable/app bundle is included.

Confirmed historical handles: source runner 1905 completed; native002 PID 56803 and
filename-calibration PID 41664 were absent after their planned quits. Failed001
PID 34507 was originally left at its frozen failure barrier. The later explicit
user-requested cleanup closed all ten old owned WeftForge GUI windows, including
that throwaway editor, while preserving project files/reference app and unrelated
browser tabs. This separate post-case event ends prior left-running PID states;
it is outside every frozen clock and rewrites no outcome. Future unused finished
test windows close after retaining evidence, per AGENTS. These are historical
handles, not reusable input authority.

## Historical Transform and New Empty checkpoint

[PR45](https://github.com/thematheusmello/weftforge/pull/45), merged as
550825a1f2ccd390321f9d581f16ff880cb7a441, retains the DD
[failed implementation-only A/B report](parity/reports/WF-20261115-PATTERN-TRANSFORM-001.json),
now **stale** on later B657/ad7 runtimes. Root accepted bounded numeric A4; B remained
at R40 after one declared drag, with no rotation/Save and unknown cause. No B pass
or unexecuted B2 packet is imported. The original 59 source pins, build and native 216 identities,
235 Rust/54 Python source gates, 93-case CLI/MCP run and all images/outcomes remain
unchanged. B657 physical sewing002 does not complete that Transform pointer plan.

The historical map remains 949 entries / 205 packets / 56 landmarks. The previous
B657 source addendum has 27 enum variants, 26 nontransaction schemas and
nine MCP tools; this is source inventory only. Neither map marks behavior or a
roadmap gate complete. Actions remain disabled, head Actions runs are not enabled
by this preparation, and the existing static Pages site is preserved.

## Prior measurement delivery

[PR44](https://github.com/thematheusmello/weftforge/pull/44) and the
[be8 report](parity/reports/WF-20261115-AUTHORED-LENGTH-001.json) retain the bounded
205 Rust/54 Python gates, 46-process analytic CLI/MCP run and read-only native002
length displays. Native001 stopped before input on recorder setup. Be8 and the
earlier 7ab PhotoCraft-inspired shell evidence are stale on changed runtime source;
their original outcomes are unchanged and no new authoring, reference or garment
credit is transferred. PhotoCraft informs visual structure only.

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
not newly executed acceptance. That historical DXF attribution remains stale; current APP-001/FILE-005 feature states are recorded separately in the registry.

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

## Exact next work

Root independently audits the clean tracked publication candidate, mappings,
source65, scopes and check receipts before remote publication. Continue the
remaining native C–E work under separate reviewed source/input/criteria bindings
and GO, preserving the separately accepted private B checkpoint. Actual accepted
native bytes may become bound inputs; never synthesize a model substitute. Do not
repair/regrade closed A001/A002/A003/B001 or physics cases.

After accepted annotation/persistence/runtime evidence, propose a separate bounded
topological cut with explicit ownership, relationship and atomic-command policy.
PAT-006 remains planned. Full internal shapes/constraints/notches, chain/internal
sewing, easing, native remeshing, garment physics, full New/Recover and reference
workflows remain open. No roadmap gate is completed by these engineering subsets.

Keep Actions and automated Pages deployments disabled through development. This
local preparation adds no push/PR/merge, package/release or deployment. Root owns
publication; the existing static Pages site remains unchanged.
