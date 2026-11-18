# Native Fabric editing — 18 November 2026

The current Apple Silicon studio changed Harbor's shared warp stiffness from
400 to 520 N/m. Undo and Redo restored each value in a single action.
The Save As copy reopened in a fresh process at revision 23. The back panel
displayed 520 N/m and the reopened project completed one manual simulation step.
Reset restored the rest arrangement. Both test windows closed.

Independent readback verified that the saved file differed in revision and warp
stiffness and retained both pieces and all eight seams. The native capture
decoded as a 1640 × 1000 RGBA8 sRGB PNG.
[Fabric case and artifacts](parity/reports/WF-20261118-HARBOR-NATIVE-FABRIC-001.json).

Complete fit and simulated-mesh export for the shirt remain open. The next
contact check captures sewing and contact history at the first failed substep
with a positive seam target. GitHub Actions and automated Pages deployments
stay disabled.

# Independent soft stitches — 18 November 2026

The solver now solves soft stitches analytically wherever their movable nodes
are independent. Shared rows still use the coupled update, and the sewing
stage commits both updates and their history together. Source `e5f043ef`
passed the targeted mixed-topology regression, formatting, 489 workspace tests,
strict all-target Clippy and the optimized studio and CLI builds.

The optimized CLI completed all 181 quality states on the saved Harbor project
in 258.064 seconds. The first failed state was 28. The final strain was 15.65%,
above the 15% limit. Final seam gap was 0.575 mm and supported penetration was
0.0062 mm. [Source checks and failed fit](parity/reports/WF-20261118-HARBOR-ISOLATED-SEWING-FIT-001.json).

The verified source and fit report landed in
[PR #69](https://github.com/thematheusmello/weftforge/pull/69).

# Native save and cancellation — 18 November 2026

The optimized Apple Silicon studio saved a byte-identical Harbor project while
assembly was at step 23 of 60. Task Details recorded cancellation at step 35,
discarded the simulation result and retained the rest arrangement at revision
20. The studio quit normally.
[Native job-control case](parity/reports/WF-20261118-HARBOR-NATIVE-JOB-CONTROL-001.json).

The native job-control case landed in [PR #68](https://github.com/thematheusmello/weftforge/pull/68).

# Native project recovery — 18 November 2026

The optimized Apple Silicon studio kept Harbor revision 20 open after a
malformed project and a project from an unsupported future version. Recover
copy opened revision 19 as an untitled editable document with all eight seams.
Save as wrote a byte-identical copy of the backup. The four input files stayed
exact and the studio quit normally.

A separate engine test injected an interruption before atomic replacement and
confirmed that the prior project survived. Missing external asset recovery and
other platforms still need their own cases.
[Native recovery and engine check](parity/reports/WF-20261118-HARBOR-NATIVE-RECOVERY-001.json).

The native fit failure is recorded in [PR #66](https://github.com/thematheusmello/weftforge/pull/66).
The direct contact experiment reduced the sewing residual but missed the
normal and friction bounds. Continue with a solver that explicitly handles
the active, sticking and slipping contacts on the saved six-node case.
Gate 1 remains open. GitHub Actions and automated Pages deployments stay disabled.

# Native Harbor fit — 18 November 2026

The optimized Apple Silicon studio completed 60 assembly steps and 120 drape
steps on the reopened Harbor project. It displayed 15.71% strain after assembly
and 15.72% after drape, both above the strict 15% limit. The displayed seam gaps
were 0.5 mm and 0.6 mm. Project bytes stayed exact and the studio quit normally
after 338.044 seconds. [Native phase endpoints](parity/reports/WF-20261118-HARBOR-NATIVE-FIT-001.json).

These are phase endpoint observations. The separate 181-state garment failure
retains its original source and grade. The next solver check uses the saved
six-node case to test whether a direct solve can satisfy both sewing and contact.
Gate 1 remains open. GitHub Actions and automated Pages deployments stay disabled.

# Native Harbor authoring and exports — 18 November 2026

On Apple Silicon the studio arranged the two recovered UI-authored Harbor
panels, sewed eight matching boundary pairs and saved the editable project.
A fresh studio process reopened the project. It saved a front height change
from 0.640 m to 0.680 m, then saved Undo, Redo and the restored original.
Independent readback checked all eight seam relationships across the edits.

The native File menu saved the SVG and DXF patterns, the two OBJ rest surfaces
and a 1640 × 1000 workspace PNG. Both test windows closed normally.
[Case and verified files](parity/reports/WF-20261118-HARBOR-NATIVE-AUTHORING-REOPEN-001.json).

The strict full-garment strain failure remains open. The polygon fix and its 488-test
source check landed in [PR #64](https://github.com/thematheusmello/weftforge/pull/64).
GitHub Actions and automated Pages deployments remain disabled.

# Polygon close transaction — 18 November 2026

Closing a polygon now commits pending inspector edits and the new piece in one
transaction. A rejected close preserves the staged vertices, inspector input,
undo history and preview. A successful close selects the new piece and puts both
edits in one Undo step. The tests cover the button press and release and Return.

Source `6078c2af` passed formatting, 488 workspace tests, strict all-target
Clippy and the studio and CLI builds on Apple Silicon. [Source check report](parity/reports/WF-20261118-POLYGON-CLOSE-SOURCE-001.json).
The verified workspace screenshot landed in [PR #63](https://github.com/thematheusmello/weftforge/pull/63).
Its native File-menu observation belongs to the earlier `59501451` build.

The studio reopened the two UI-authored Harbor panels. The macOS input driver
lost access during the Front depth edit, so that session closed with the saved
panels intact and no seams added. The next native case starts from those panels
and checks placement, sewing, editable save, reopen and export.

The complete Harbor run still exceeds the 15% strain limit. A private diagnostic
hit its event recorder limit. Review of the complete early windows is in progress
before the next solver change. The six delivery gates remain open. GitHub Actions
and automated Pages deployment are disabled.

# Native workspace capture — 18 November 2026

**File → Save workspace PNG… passed on Apple Silicon.** The `59501451` native
binary was launched from the merged checkout with all 93 source inputs unchanged.
It saved an original 1640 × 1000 client PNG in case P003, confirmed completion in
the UI and quit normally in 70.269 seconds. Independent decoding verified RGBA8,
dimensions and the standard sRGB chunk. Project bytes, visible revision and rest
arrangement stayed unchanged. [Native case and image](parity/reports/WF-20261118-NATIVE-WORKSPACE-PNG-P003.json).

The earlier P001 truncated inspection and P002 startup timeouts remain recorded
as stops before a PNG request. The P003 run verifies the File route on this build.
Command search, cross-platform capture and the full scene-capture contract remain
open. The separate tooling receipt records the actual clean `6872e7c3` metadata
verification. [Tooling report](parity/reports/WF-20261118-WORKSPACE-PNG-I-TOOLING-001.json).

The latest complete garment run still exceeds the 15% strain limit. Next, author
and sew the two Harbor panels in a fresh native project and verify editable
save/reopen and export. The separately reviewed, private garment diagnostic
remains outside the product. Keep Actions disabled and land each checked increment
in a small PR. The full roadmap remains active.

# Main landing — 18 November 2026

The checked 95-commit development stack is landed in **main** through
[PR #62](https://github.com/thematheusmello/weftforge/pull/62), merge
`0b2c5eb994177e0c2835537e66031c73cba28fc7`. Its tree is exactly checked head
`6872e7c3`; every head from PRs #50, #51, #53 and #55–62 is included. PR #50
closed as merged automatically, and the nine other covered stack entries were
closed with their delivery location recorded. No open PR remained after this
landing. Historical branch references remain available for existing evidence
links. The closed-stitch precursor `35afac60` is superseded by integrated sewing
and retained only for frozen provenance: historical source checks require its
Git object. It contains no additional feature work to land.

Actual source `59501451` retains its formatting, **486 workspace tests / 32
summaries, zero failed/ignored**, strict Clippy and native/CLI build pass. The
102 simulator tests are included within 486. Actual clean `6872e7c3` metadata
verification passed in **312.499 seconds / 360**, including registry/maps,
generated reports, 54 script tests, six evaluator tests and CLI help. Source93
is identical in the landing tree. These executions keep their original commit
identities; this status-only update claims no new source execution.

The full garment case remains failed at **19.770147% peak / 15.721208% final
strain**, with 153 frames above the unchanged 15% screen. Actual native PNG
saving/backend verification and the complete empty-project garment journey
remain open. Newer controller observations await public selection and provide
no acceptance; the private history diagnostic is prepared and source-reviewed
but has not compiled or run. No private diagnostic code is included in main.

Next: finish the actual native capture/creation/save/reopen/export journeys,
compile the separately reviewed diagnostic when admitted, investigate the
preserved strain failure and continue the full ROADMAP. Land each finished,
checked increment promptly in main through a small PR; close covered work as
it lands instead of accumulating a draft stack. All 56 full feature groups,
reference parity, six gates and platform releases remain unaccepted. The
end-to-end goal stays active. No owned Cargo, model or native job is running.

GitHub Actions and both workflows remain disabled: 54 historical runs, none
active at the post-merge check. The existing hosted Pages site is preserved;
no deployment or release was performed. The earlier checkpoints below retain
their original identities, pending actions and outcomes as historical records.

# Workspace PNG integration — 18 November 2026

The reviewed source increment adds **Save workspace PNG…** to File and command
search. It uses actual renderer pixels, a single staging worker and a validated
UI-side atomic commit. Pending authoring remains untouched. Shutdown stops
waiting for a worker result, and Escape cancels capture through popups/modals
without consuming their normal event. The original shutdown review finding is
preserved; its source resolution is reviewed. Pixel, lifecycle, input-retention
and file-preservation tests are included. Actual clean source `59501451` passes
formatting, **486 workspace tests across 32 summaries, zero failed/ignored**,
strict all-target Clippy and native/CLI debug builds in **386.000 seconds / 900**.
Its 102 simulator tests are included within 486. Clean metadata verification and
actual native capture remain pending. See the [bounded capture contract](spec/features/MAT-003-workspace-capture.md)
and [source report](parity/reports/WF-20261118-WORKSPACE-PNG-INTEGRATED-SOURCE-001.json).

The fresh G `e30a71a5` all-181-state numerical case completed operationally in
**379.697 seconds / 420**, with its exact final external tool verdict retained.
Strict quality remains **failed**: 153 strain-failing frames, 28 through 180;
peak **19.770147%** at frame 29/front triangle 6, final **15.721208%** at back
triangle 654. The peak is lower than D's 20.243551%, while final strain is higher
than D's 15.349281%. Finite states, supported contact, uncertainty, zero recovery
and closed-target seams pass. Original physical inputs, settings and thresholds
are unchanged. Initial open arrangements are not completed-fit failures. See the
[failed complete case](parity/reports/WF-20261118-CAPSULE-SUPPORTING-PLANE-HARBOR-A8-001.json).

An earlier external admission stopped on an incorrect parent PATH before GO
validation, the numerical clock/controller or any model initialization. Its
original failure remains frozen. A separate source/environment check and
reviewed admission supplied the exact bound child and ambient environment for
the first actual numerical run. A premature wrapper invocation during an
admission deferral is also retained separately. Neither failure is a garment
result, and no prior numerical result or clock gap is repaired or regraded.

Current G optimized native/CLI builds passed in 63.854 seconds; source binding
passed in 2.563 seconds and its standalone producer build in 23.183 seconds.
All owned process groups are absent. These executions belong to G and do not
transfer to the PNG source. See the [G tooling report](parity/reports/WF-20261118-CAPSULE-SUPPORTING-PLANE-G-TOOLING-001.json).
The new source map preserves all 441 prior occurrence identities: 218 are
repinned and 223 are exact, with 16 new capture anchors for 457 total. It has
40 rechecks, 25 existing surfaces and 25 reviewed documents; all prior 24 document
rows, 949 catalog IDs, 205 packets, 56 historical landmarks and command schemas
are preserved. The private static map check completed its hash/literal/line and
preservation checks, then failed an incorrect wording assertion; its tail was
not run. This is not a map-check pass. Actual clean map/registry/xtask verification
is required before publication.

Selected runtime/source evidence and generated progress are installed. The first
generation refused ten feature rows whose current labels had no current H run;
only those freshness fields were corrected to stale, preserving all outcomes
and feature scopes. The corrected generation passed separately in 82.533 seconds;
the [freshness correction](parity/reports/artifacts/workspace-png-integrated-source-20261118/REGISTRY-FRESHNESS-CORRECTION-001.json)
retains the original failure's identity. Next: commit the coherent metadata
checkpoint, run clean `cargo xtask verify` and push each checked atomic commit
promptly. Then test File-to-PNG through the actual native
app, inspect the original pixels/metadata and unchanged project, and close the
owned window. Empty-project creation/sewing/simulation/edit/save/reopen/export
and the remaining reference and release journeys remain open. The actual G
strain failure is under separate saved-state diagnosis; no broad fitting
improvement, full feature, native journey, reference parity or delivery gate is
accepted. GitHub Actions remain disabled. The full end-to-end goal stays active.

The earlier source/runtime checkpoints below retain their original identities.

# Capsule contact integration — 18 November 2026

The reviewed capsule-contact correction is integrated. Current vertices and
retained barycentric supports use exact capsule supporting planes, with their
current closest-axis normals and independently retained nodal actions. This
addresses the demonstrated stiff-sewing split instability. Scaled point queries
preserve nonzero axes, and unusable geometry fails before it can be mistaken
for separation. Physical parameters, closure speed, outer iterations and garment
quality criteria are unchanged. Ground, moving-triangle and radial Torso limits
remain separate; the capsule method follows a local current-side branch and
provides no general global-projection or continuous-collision guarantee.

Actual clean source `c766f3fa` passes formatting, **480 workspace tests across
32 summaries, zero failed/ignored**, strict all-target Clippy and native/CLI
debug builds in **413.527 seconds / 900**. The 102 simulator tests are included
within 480. Independent review accepts the bounded correction, including the
large-finite-axis fix; the earlier numeric finding remains preserved. See the
[source report](parity/reports/WF-20261118-CAPSULE-SUPPORTING-PLANE-SOURCE-001.json).
These executions belong to `c766f3fa`; metadata commits do not inherit execution.

Historical native P005 completed its bounded constructor observation and normal
close in **37.025 seconds / 300** on the E checkpoint, using the D-built native.
Separate review of the unchanged original capture accepted aligned metre labels
and controls in the charcoal/blue workspace. The public text-only package omits
all image-bearing originals for metadata privacy and cannot independently verify
that layout. It provides no current-F or complete native-workflow acceptance.
P002/P003/P004 retain their negative harness outcomes, without an inferred product
visual defect. See the [bounded native report](parity/reports/WF-20261118-NATIVE-CONSTRUCTOR-VISUAL-P005-001.json)
and [earlier stops](parity/reports/WF-20261118-NATIVE-CONSTRUCTOR-HARNESS-STOPS-001.json).

The updated source map preserves all 441 occurrence identities: three capsule
contact anchors are repinned and 438 are exact. The 949 catalog IDs, 205 packets,
56 historical landmarks, 24 reviewed contract documents and command schemas
remain unchanged. Prior C/D reports are conservatively stale for their complete
source scope; exact original report archives preserve their outcomes and hashes.
Clean E registry/map/xtask verification passed separately in 253.837 seconds / 360.
Current metadata still requires its own clean verification before publication.

Next: finish generated progress, commit this evidence/metadata atomically, run
clean current `cargo xtask verify` and push promptly. Then build current optimized
native/CLI and numerical producer binaries and execute the unchanged full
181-state case. Its independent prospective source review is ready; actual
current bindings, builds, runtime freeze, admission and GO are pending. Original
inputs, 15% strain, 5 mm closed-seam and 2 mm contact limits, all states, zero
repair requirement and physical settings stay fixed. The D full case below
remains failed; source/component success claims no garment improvement.

Then complete the empty-project native creation/sewing/simulation/edit/save/
quit/reopen/export journey and independently inspect its output. No Cargo,
native or model job is running at this checkpoint. Latest published development
checkpoint is [draft PR #62](https://github.com/thematheusmello/weftforge/pull/62)
at `023b45a8`; push every checked atomic increment to keep remote synchronized.
GitHub Actions and both workflows stay disabled, with 54 historical runs and
none active. Main and existing static Pages remain unchanged. All 56 broad
groups remain partial/planned (42 partial, 14 planned); discovery, six gates,
ten acceptance journeys and three platform targets remain open.

The runtime checkpoints below retain their original source, pending actions and
results as historical records.

# Current runtime checkpoint — 18 November 2026

The current full 181-state garment case on `a70c0df0` is **failed**.
Strain exceeds 15% in 153 frames, 28 through 180: the peak is 20.24355055%
at frame 29, front neckline triangle 6; final strain is 15.34928093%.
Both are worse than the earlier 16.06333827% peak and 9.43073967% final.
All states are finite and pass the supported contact, uncertainty and present
integer zero-initial-recovery screens. Closed-target seams pass; initial open
arrangements are not completed-fit failures. Original inputs, settings and
physical thresholds are unchanged. See the [failed full-case report](parity/reports/WF-20261118-CONTACT-RECOVERY-HARBOR-A8-001.json).

The raw root witness records operational completion at 353.193 seconds within
420 seconds, with outer exit 2. The outer external witness still marks its final
clock verdict pending. A separate post-case record reports 353.245 seconds;
there is no retained independently sealed final external-clock verdict, and
that later record does not repair the missing proof or change the failed grade.
All owned process groups are absent; no model job is running.

Clean D `cargo xtask verify` passed in 264.142 seconds within 360, including
registry/maps/generated progress, 54 script tests, six evaluator tests and CLI
help. A separate optimized native/CLI build passed in 53.293 seconds within
900. Actual formatting, 472 workspace tests and strict Clippy remain the
separate `f30c588d` source execution; all 91 source inputs are unchanged on D.
See the [D tooling/build report](parity/reports/WF-20261118-CONTACT-RECOVERY-D-TOOLING-001.json).
These source/build results provide no native workflow or garment acceptance.

Next: finish the fresh native constructor observation; review and integrate the
capsule-contact correction; run all integrated local checks and the unchanged
full numerical case. Then complete empty-document native creation, sewing,
simulation, edits, save/quit/reopen and independently checked export. The new
tracked evidence and metadata still require clean registry/map verification
before draft publication. No Cargo or native job is running at this checkpoint.
GitHub Actions remain disabled at repository and workflow levels; there are
54 historical runs and none active. Main and existing Pages remain unchanged.
All 56 broad groups remain partial/planned (42 partial, 14 planned), with
discovery, six gates, ten acceptance journeys and three platform targets open.

The earlier checkpoints below retain their original source and timing; their
then-pending runtime actions are historical.

# Contact recovery integration — 18 November 2026

The end-to-end delivery goal remains active. Local source commit `f30c588d`
integrates the reviewed contact recovery and all diagnostic adapters; the latest
published draft remains PR #62 at `3e1f9ef8`. No garment, reference, complete
feature or delivery gate is accepted.

Actual clean `f30c588d2602eabc3f8c79f63c74f50a6bd2ee5f` passes formatting,
**472 workspace tests across 32 summaries, zero failed/ignored**, strict
all-target Clippy and native/CLI debug builds in **381.554 seconds / 900**.
All source91 hashes/modes remain exact and owned process groups are absent.
The first attempt failed preflight on the previous STATUS hash, before any
Cargo command; its original failure is preserved. The accepted invocation has
external terminal `5e50ff3e7f291e517a7e33d07b9f3b7f99a9a42eb7dd5cd5b226455504d334c3`;
separate post-case closure is `86ab30790efa715a3ed69763ee00ee62e43afb0cf74743f76d15b4ac5e4813d9`.
These executions belong to `f30c588d`, not a later metadata commit.
See the [source report](parity/reports/WF-20261118-CONTACT-RECOVERY-INTEGRATED-SOURCE-001.json).

The actual built native identity is
`d93f90127b02fc4ee3049738946cb2466a0a2c00c4a287eebf3f1664088e7e74`;
the CLI is `26949051c4a60d43042c99229d1b98d458a3369f6a7adfeaae4f286551fb6131`.
Both are retained as immutable copies inside their original checkout, outside
Cargo inputs. Their standalone binaries have not been launched. The workspace
checks execute their normal implementation tests, including 94 simulator tests
and the shared CLI quality-state recorder test.

Ordinary contacts now replace their own displacement history; capsule surface
pairs retain fixed material-point supports. Frozen initially intersecting
nonlinear pairs use separately attributed first-substep kinematic repair. Its
persistent projection counter appears in native/CLI reports. CLI fitting checks
the initial and every accepted whole-step state, retains transient failures and
requires zero repair projections, with the original 15% strain, 5 mm closed-seam
and 2 mm contact limits. No authoring command or MCP file-tool schema changes.
Physics and automation documentation describe the policy and its limits.

The constructor's 459-test report remains stale for its full dependency scope,
with exact original bytes archived. The new source overlay retains 441 occurrence
identities, 949 catalog IDs, 205 packets and 56 historical landmarks; 239 source
hashes/lines are repinned and 202 remain exact. A private map-preparation failure
on an absent optional metadata key is preserved; its successor completed all
preservation assertions. This preparation is not a map-check pass. Public source
curation and registry freshness are installed. Progress generation passed in
60.450 seconds / 120, changing only the two established generated documents;
all 13 frozen inputs and the index remained exact and owned groups are absent.
Clean registry/map/xtask verification is required before publication and numerical GO.

Moving-triangle single-feature nonlinear contact can still stall. Initial repair
contributes to velocity and supplies no normal-dynamics or calibration proof.
The original 181-state Harbor case remains failed at frame 29, 16.063338% strain
against 15%; no new garment execution or improvement is claimed. Its successor
must check all 181 states, including a present zero recovery counter, with the
original inputs, physics and quality thresholds.

Next: finish the tracked evidence/metadata checkpoint and clean local
`cargo xtask verify`, then build exact-current optimized native/CLI/producer
binaries, run the unchanged full numerical case, and execute native creation/
sewing/simulation/edit/save/quit/reopen/export with independent output inspection.
The constructor preview controller is being corrected before a fresh observation;
older failures remain unchanged. No Cargo, native or standalone model job is
running. GitHub Actions remain disabled at repository level and both workflows
are disabled manually. Main and existing Pages remain unchanged. All 56
inventoried groups remain partial/planned; discovery, all six gates, ten
acceptance journeys and macOS/Windows/Linux completion remain open.

The prior checkpoint below retains its own source and timing attribution.

# Current development checkpoint — 18 November 2026

The latest published checkpoint remains [draft PR #62](https://github.com/thematheusmello/weftforge/pull/62),
head `3e1f9ef8`, stacked on draft #61. New UI code is local commit `f6144bce`.
No garment, reference, full feature or delivery gate is accepted.

Constructor Width, Height and Position X/Y fields now have aligned metre labels
and accessibility associations. Numeric helpers, ranges, field order and shared
authoring paths are retained. Actual clean `f6144bce` passes **459 workspace tests
across 32 summaries, zero failed/ignored**, formatting, strict all-target Clippy
and native/CLI debug builds. The first invocation passed formatting, then stopped
at the disk floor after 59 partial ok test lines, without a complete summary.
Its 193.072-second failure stays failed. After inactive owned caches were cleared
with same-worktree executable preservation, a distinct unchanged-source invocation
passed only the unfinished tests/lint/builds in **378.958 seconds / 900**.
There is no single successful four-command clock. See the
[source report](parity/reports/WF-20261118-CONSTRUCTOR-FORM-SOURCE-001.json).

The current debug native identity is
`ba7c80382a78acac937c367f223778b8c6e2898476985340b2ac10c6fa99134f`;
the CLI is `1626b5af3a0b460b993e34271f4613dbc40eedb30f3786b56e0972f7519cf4be`.
Immutable copies remain inside this checkout, outside Cargo inputs. Earlier
optimized constructor build `fbf91108fa828597a6c16fac476970b4844c24234b303a5a1dae7a27c5fd4592`
retains its pre-commit source attribution. Generated progress updated in
73.032 seconds / 120; clean metadata/xtask verification is next.

Preview P001 captured the loaded original Harbor sample, but failed before
Frame all or opening the constructor. The two state-call lifecycles totalled
2.141 seconds; preparation/operator gaps consumed the dispatch allowance.
The watchdog closed the owned process, without normal-Quit credit; sealing also
exceeded 300 seconds. It remains failed. The continuous successor is unarmed
while its binding, image validation, cleanup and final admission are corrected.
No constructor appearance or complete native workflow is accepted. Earlier
N009/N010 failures and their cleanup outcomes remain unchanged.

A separate simulation successor now passes **94 package tests, zero
failed/ignored**, including unchanged capsule, pinned-layer and swept-crossing
criteria. Its independent ground-equilibrium result is Y approximately 1 metre,
force residual -9.77e-15 N. Source review separates balanced contact actions from
explicit kinematic recovery of frozen initially intersecting nonlinear pairs in
the first substep. A persistent recovery counter records motion; it must remain
zero at every state of the future full Harbor case. Recovery contributes to
velocity and supplies no normal-dynamics or calibration proof. The 85/2 and 88/2
predecessors remain failed, with thresholds and outer iterations unchanged.

That simulator source is still isolated. GUI/CLI/default-data compatibility,
zero-recovery fitting criteria, documentation and integrated source checks must
be completed before its full garment/native runs. Moving-triangle nonlinear
fixed-point and broader contact/calibration limits remain open. The original
181-state garment case still fails at 16.063338% strain against 15%; no new
garment improvement is claimed from package tests.

Next: commit the tracked UI metadata checkpoint; integrate the reviewed simulator
and all adapters/docs; run full local source checks, the unchanged all-181-state
quality case and the genuine native empty-project create/sew/simulate/edit/save/
reopen/export workflow. No Cargo, native or standalone simulation job is running.
Actions are disabled at repository level and both workflows are disabled manually;
54 historical runs remain, none active. Main and existing Pages are unchanged.
All six gates and ten acceptance journeys remain open, with 42 partial and 14
planned broad groups; discovery is incomplete.

The published checkpoint below retains its original source and timing attribution.

# Development integration checkpoint — 18 November 2026

The `feat/dual-context-integration-20261117` branch combines dual soft sewing
with contextual Pattern, Fabric and Simulation tabs, independent scrolling,
Scene and contextual creation/sewing groups in the PhotoCraft-inspired direction.
A shared cache fix retires discarded sewing-row input before reused controls or
an authoring action can consume it, while preserving unrelated edits.

Actual combined source `12ecdc13ba47a20d77bf52d1757815e39f0ea380` passes
**459 workspace tests across 32 summaries, zero failed/ignored**, formatting,
strict all-target Clippy and native/CLI builds. These results belong to two
separately bound invocations. The first sequence completed the tests/lint, then
its build was killed after a verification output stream broke. That failure and
its original false combined-result flag remain intact. A distinct build-only
invocation passed in **4.179 seconds / 180 seconds** on unchanged source;
no passed tests or lint checks were repeated. No single all-four-success clock
is claimed. See the [combined source report](parity/reports/WF-20261117-DUAL-CONTEXT-INTEGRATED-SOURCE-001.json).

The newly built native binary is
`08e682166be52aadb915d3560a16bef2569893737f182f4980600bebfe2db4a9`;
the separately built CLI is
`1626b5af3a0b460b993e34271f4613dbc40eedb30f3786b56e0972f7519cf4be`.
The native binary has not yet run the current shirt workflow. Immutable
same-worktree copies retain both build-only identities; no Cargo target was
copied. The later clean verification rebuilt and ran CLI help with post-run
identity `532288b895e439fcd223a20430e6d74d775e0d5121cd082ea8e1972ca0769ee5`;
that execution is separate from the earlier CLI build.

The numeric-cache regression has an actual first Down failure on `bd08d511` and
passing eight-guard successors on `cb291881` and `99734576`. The latest check
includes actual App keyboard Delete/Escape, an appended row before redraw,
reused IDs and exact preservation of other raw text/caret, pending commands,
history and accepted mesh/cloth identities. Its 24.987-second check is separate
from the full workspace result. All original prototype compile, discovery and
lint failures remain attributed to their own sources. The earlier 166-test pass
belongs to `351bf28b`; it is never relabeled as an execution on `a66c90ed`.

The independent full garment case on actual `ef313fa8` remains **strict quality
failed**. It completed 180 steps and analyzed all 181 states in 323.212 seconds
within 420 seconds, but frame 29 reaches **16.063338% warp strain**, exceeding
the unchanged 15% limit. Finite-value, supported-contact and uncertainty screens
pass; assembly targets are zero at frames 60/180. Final-frame 9.430740% does
not override the trajectory failure. The historical 50.9602% failure remains
unchanged. See the [full numerical report](parity/reports/WF-20261117-DUAL-SEWING-HARBOR-A8-001.json).
The ef313/UI997 source reports are stale for their declared full dependency
scopes after integration; the new combined source report is current.

The source crosswalk maps 441 occurrences, preserving all preceding identities,
949 catalog IDs, 205 packets and 56 historical landmarks. Its separate read-only
check passed in 14.892 seconds. Progress generation returned zero on clean `4ffb20e3` and changed only the two
expected reports. Its 60.9998-second case failed the 8 MiB evidence-volume cap
with a 10,527,765-byte ledger; all source/original/public hash guards passed.
That failure remains unchanged. A separate clean `cargo xtask verify` on
`317565b6e270aacaba8b6cc26e5305d2148ddaeb` passed in **222.885 seconds / 360**:
registry, generated reports, maps, 54 script tests, 6 evaluator tests and CLI help.
All 188 immutable inputs, 8 supporting tools, 237 originals and 242 public
increment artifacts remained exact. HEAD/index stayed clean; owned controller
42270 and Cargo 42273 and their process groups are absent. The source tests and
generator were not replayed, and the generator volume failure is not regraded.
The [tooling packet](parity/reports/artifacts/dual-context-tooling-20261118/PROVENANCE.json)
retains both outcomes, with explicit descriptors for the six omitted full ledgers.

N006/N007 native workflow evidence still belongs to 8aa/c332. The private N008
preview showed the three tabs but missed its 180-second final-seal bound
(193.534 seconds), so it provides no workflow acceptance. Finished owned test,
preview and reference processes are closed. A current-v6 empty-project shirt
route is prepared against the new binary. Its private independent consumer passed
29 synthetic checks; actual native outputs and UI execution remain pending
clock/control preparation. No new native app is running. The separate fixed-input
observer passed independent source/math review and its one private build-only
attempt; numerical execution and diagnostic evidence publication remain pending.

Next: retain the compact tooling evidence and final clean registry/map closure,
publish only the reviewed safe draft checkpoint, and execute current native authoring, sewing,
simulation, save/restart/edit/export with independent output checks. Continue
all six gates and ten acceptance journeys in [ROADMAP.md](ROADMAP.md).
GitHub Actions remain disabled with 54 runs. Main and existing Pages are unchanged;
no new branch has been pushed, merged or deployed. The registry remains 42 partial,
14 planned, zero fully implemented/reference verified, with no accepted delivery
gate. Discovery remains incomplete; 56 is not the final product denominator.

Earlier published checkpoints and retained evidence follow. Their current/pending,
source, job and next-step statements belong to those historical checkpoints.

# Current development integration — 17 November 2026

The `feat/notch-bend-integration` branch combines the notch/text-input and
coupled-bending source on safe published PR #60. Actual combined source
`8aa850975fa388c3c4bd206db74adf7171916451` passes 446 workspace tests across
32 summaries, formatting, strict Clippy and native/CLI builds. All four commands
exited zero in 496.191 seconds within the declared 900-second bound. Source
hashes/modes, test summaries, binary identities and cleanup were independently
checked. No dependency or toolchain changes are added.

The actual desktop binary `c3327b3f9b7b6ed300787e4d113fa9f095969628416748155e3b79452f5e056f`
now passes the [bounded native notch workflow](parity/reports/WF-20261117-NATIVE-NOTCH-N006-001.json).
From an empty UI document, it creates a rectangle, refuses invalid raw notch
input without changing the revision, creates and edits the same notch #11,
performs one Undo/Redo, saves native v6 and exports a 10 mm SVG marker. After
confirming replacement of an owned DXF sentinel, the application refuses the
unsupported export and preserves the file. A fresh process reopens the saved
project with editable notch identity/values; both processes close normally.
Independent root checks cover all 94 retained manifest entries, 91 source inputs,
native output, SVG geometry and the preserved sentinel. This is keyboard/pointer
workflow evidence; N006 did not exercise application-modal AXSetValue.
N004/N005's empty accessibility outcomes remain historical, unchanged.

A separate N007 case subsequently verifies five genuine native AXValue writes
with changed raw readback, invalid-input refusal, stable-ID create/edit,
Undo/Redo, native saving and editable fresh-process reopen. Root independently
checks its 93 retained entries, 91 source inputs, actual before/after AX nodes,
saved v6 data and normal process cleanup. That case sealed in 565.902 seconds
within its 600-second bound; its raw evidence remains private pending a scoped
publication selection. It does not regrade N006 or establish complete keyboard,
high-DPI, screen-reader, feature or reference coverage.

Clean `cargo xtask verify` on metadata revision `1ff1b7bf` passed in 199.814
seconds after the native evidence and generated progress were Git-tracked:
registry/generated reports, source and discovery maps, 54 script tests,
6 evaluator tests and actual CLI help. All 145 protected inputs stayed exact.
[Retained tooling evidence](parity/reports/artifacts/notch-integration-verification-20261117/PROVENANCE.json)
records that actual revision and its separate invocation. The prior clean check
on `f9ade463` remains historical. The CLI-only rebuild/help is distinct from the
earlier independent CLI export case, which retains its original binary identity.
Source/garment tests were not replayed by this metadata check.

The full coupled-bending garment case remains **strict quality failed**: all
180 steps and 181 states completed, but strain failed at frames 28/29, with
50.9602% peak against the unchanged 15% limit. Finite-value and supported-contact
screens passed. No garment or reference acceptance follows.

A separate coupled soft-sewing candidate fixes the independent duplicate-stitch
stationarity case: its original 0.0877711 N residual becomes 2.21057e-11 N under
the unchanged 1e-6 N criterion. Its first full workspace sequence on `1029d52e`
then **failed** an existing core sewing-group test: one simulation step exceeded
the 500% instability guard with 798.7% material-axis extension. The partial run
recorded 348 passed, 1 failed, zero ignored across 25 summaries. Formatting
passed; Clippy and native/CLI build were not run after the failure. The candidate
is not integrated, and no new garment run has started. Original tests and limits
remain unchanged while the solver interaction is diagnosed.

Next: verify and publish the bounded native evidence; repair the coupled-sewing
integration failure, then repeat the affected source checks. Before a new full
garment case, independently approve the unchanged original physical rest with
only the explicit empty v6 notch metadata extension. Continue the empty-document
shirt workflow and the complete six-gate delivery sequence in ROADMAP.md.

GitHub Actions are disabled; their run count remains 54. Main remains
`1fc1d71993cf193801e44982b757fca8abc5aba7` and the existing Pages site is unchanged.
The N006, N007 and reference test processes are closed; no new garment run is
active. A separate UI worktree is improving contextual property grouping while
the solver repair continues independently. Feature discovery remains incomplete: 42 partial,
14 planned, zero fully implemented or reference verified
among 56 inventoried broad contracts. No delivery gate is accepted.

Earlier checkpoints and retained evidence follow.

# WeftForge status — active development, 2026-11-17

## Clean sewing-history verification

Checked candidate `bf100c3310bf6acf53df18e07052780312372965` passed one pinned, clean `cargo xtask verify`: **54 script tests, 6 evaluator unit tests**, registry/generated checks, historical/current maps and CLI help. Session 4054 exited 0 in **310.25626837508753 s / 360 s**, through external receipt persistence. The tooling cap was expanded before execution; it is not a performance or garment threshold. All **176 protected inputs, 8 supporting tools, 425 public artifacts and 424 originals** stayed exact; HEAD/index and checkout stayed clean. Owned processes/groups are absent. The own target now exists from verification; no cache/binary was copied.

The [QC manifest](parity/reports/artifacts/sewing-history-publication-checks-20261117/SELECTED-PUBLIC-MANIFEST-001.json) binds **61 original records plus itself**. Focused-001 executed zero tests and was rejected despite child exit 0; focused-002 passed all 10 intended tests. One generator passed in **79.09636899991892 s / 120 s**. Its pre-execution roster failure/correction, actual logs, guards, and concurrent activity remain recorded.

Actual tested source is `40817073cd38e58a8438bb397d09a0ab9d95f9bb`; its original **420-test source gates** are reused. Public `d28fb129857e97424cccb1001d1ad7f8899bdb74` matches all 88 source bytes/modes and is source-equivalent, not executed. The complete Harbor case remains **STRICT QUALITY FAILED**: 181 analyzed states, 153 failed strain frames (first 16, last 180), peak **+2.706940454071794 weft strain / 270.6940454071794% at frame 170**, triangle 915/piece 31/nodes 71,514,512. Operational completion at 335.2994576250203 s / 420 s does not override that failure. Finite/contact/uncertainty screens pass within their declared supported scope; initial 0–28 seam/target failures remain.

The published base is draft PR59, `f941dfa168162d7d003da7614fab019e691b2896`. Later notch/coupled-solver source work is a separate, excluded increment. All 56 scopes/statuses/scenarios/dependencies remain unchanged: **41 partial, 15 planned, 0 implemented, 0 reference-verified**. README/ROADMAP/gallery and static Pages remain unchanged; Actions/automated deployments stay disabled.

Next: resolve complete-garment quality and coupled physics, continue material/contact calibration and dependable editing/sewing, complete native/reference workflow coverage, and finish the remaining roadmap. This tooling result grants no full garment, feature, reference, physics-calibration or roadmap acceptance.

## Historical sewing-history preparation checkpoint

Everything below is the byte-exact preceding STATUS body. Its unrun/pending/next wording describes that earlier phase; the bounded tooling verification above is the later result.

# WeftForge status — active development, 2026-11-17

## Tested sewing source and complete Harbor failure

Public source integration `d28fb129857e97424cccb1001d1ad7f8899bdb74`, based only on safe PR59, preserves all88 bytes/modes of actually tested `40817073cd38e58a8438bb397d09a0ab9d95f9bb`. Five source/physics files changed; no private source ancestry or compiled payload was imported. The [source report](parity/reports/WF-20261117-SEWING-HISTORY-SOURCE-001.json) records bounded N32/subnormal RED-to-GREEN evidence and original420 tests/31 summaries, full fmt, strict all-target Clippy and native+CLI builds passing in260.4531114998972s. Original controller/command/format failures remain. Source/component passes establish no garment fix.

The [new Harbor report](parity/reports/WF-20261117-SEWING-HISTORY-HARBOR-A8-001.json) is **FAILED STRICT QUALITY** on actual408: all181 states were independently analyzed, with **153 failed strain frames, first16 and last180**. The trajectory peak is **frame170, +2.706940454071794 weft strain (270.6940454071794%)**, triangle915/piece31/nodes71,514,512. Final180 has absolute compression0.7423001748261551 and maximum extension0.7021055752845875; those final values are not the trajectory peak or acceptance. This is worse than the preserved9d one-frame15.580887% failure under the unchanged screen. All finite/supported-contact/uncertainty screens pass; seam/target pass after first exactzero29, with initial0..28 failures retained.

Actual session74101 exited2 after externally persisted operational completion at **335.2994576250203s/420s**. The later468.40192691702396s closure observation is preservation only. All owned groups are reported absent in the retained closure. A pre-admission launcher refusal made zero initializations/steps/analysis before missing admission; it is retained as an orchestration refusal, not a model retry. Historical d98260steps/61states monitor failure stays ungraded; its416 source gates and G005 native result remain separate unpublished facts. Frozen historical SOURCE/PROTOCOL pending annotations remain explicit in outer admission, with actual executable bindings authoritative. No old case was continued or regraded.

The [selected manifest](parity/reports/artifacts/sewing-history-source-20261117/SELECTED-PUBLIC-MANIFEST-001.json) binds424 originals plus itself:425 artifacts, including all117 case members and both closure records. Only479 ROOT,367 HOME,4 account-specific TMPDIR,1 unrelated-CWD and8 account-name exact substitutions and public0644 modes differ; private originals remain retained. The two affected9d reports are archived exact and change only freshness/reason. The map keeps394 earlier identities/history, re-pins197 records and adds11 actual anchors for405; all56 full scopes/statuses/scenarios/dependencies stay exact:41partial/15planned/0full/0reference-verified. Original8 inventories remain byte/mode-exact. No native/raw reference/image/binary/cache/installer/account payload is selected.

The narrow progress-tool proposal preserves actual408 as report.commit and uses a hash-bound complete88 public-source bridge only for incomplete implementation-only lookup. Public d28 source links are labeled source-equivalent, never executed. Available original Git trees must also match all88 paths/modes/blob IDs; reference/full-workflow promotion is refused. Six new focused test methods are authored but **not run**. Generated progress/dashboard remain at the preceding checkpoint. This preparation launched no job, helper/test/generator, Cargo, model/evaluator, native/GUI or network action. README/ROADMAP/gallery, Actions and existing Pages remain unchanged; Actions/automated Pages stay disabled.

Next: root reviews the source lookup, selection/report/registry/map proposal, then separately admits focused tooling tests, one bounded metadata generation and clean committed verification before publication. Complete garment quality, coupled-physics diagnosis/calibration, native workflow, reference parity and all roadmap acceptance remain open. No push/PR/merge is performed here.

## Historical material polling publication checkpoint

Everything below is the byte-exact preceding STATUS file. Its current/pending/next wording, source/case identities and resource claims belong to that historical checkpoint.

# WeftForge status — active development, 2026-11-17

## Clean material polling publication verification

Checked commit `92ac19094aed5adbf38fca95f26fed834c3c46a2` passed one fully pinned `cargo xtask verify`: session 12097, controller 20716 and Cargo/xtask 20721, exit 0 in **239.45643333299085 s / 300 s** after all receipt persistence. All **156 inputs** (88 source, 63 reports, five metadata including committed generated outputs), 286 originals and 287 public artifact copies stayed exact; HEAD/index remained fixed and the checkout stayed clean. Registry/generated progress, historical/current maps, 48 script unit tests, six evaluator unit tests and actual CLI help passed. See the [terminal receipt](parity/reports/artifacts/material-polling-publication-checks-20261117/verify-closure-001/TERMINAL-RECEIPT-001.json), [stdout](parity/reports/artifacts/material-polling-publication-checks-20261117/verify-001/runtime/stdout.log), [stderr](parity/reports/artifacts/material-polling-publication-checks-20261117/verify-001/runtime/stderr.log) and [input freeze](parity/reports/artifacts/material-polling-publication-checks-20261117/verify-001/VERIFICATION-FREEZE-156.json). Both owned PIDs are confirmed absent. The own target now exists from verification; no cache or binary was copied.

The one [bounded generator](parity/reports/artifacts/material-polling-publication-checks-20261117/generated-delta-review-001/EXTERNAL-TERMINAL-OBSERVATION.json) passed in 59.89680149999913 s on `bbbedb37`: 154 immutable inputs stayed exact and only the two declared progress/dashboard outputs changed. Root [accepted the delta](parity/reports/artifacts/material-polling-publication-checks-20261117/ROOT-GENERATED-DELTA-ACCEPTANCE-001.json) before those exact outputs were committed as 92ac. Intermediate pending-clock fields remain in the raw receipts alongside final terminal observations. The [independent metadata review](parity/reports/artifacts/material-polling-publication-checks-20261117/independent-review/REVIEW-RECEIPT.json) found no blocker before generation; root separately reviewed output hashes and labels.

The [full whitespace result](parity/reports/artifacts/material-polling-publication-checks-20261117/whitespace-001/RESULT.json) remains **exit 2**: 19 findings on eight exact raw evidence paths. The [waiver](parity/reports/artifacts/material-polling-publication-checks-20261117/whitespace-001/EXACT-RAW-WAIVER.json) binds patch blank-context lines and original Cargo.lock/test stdout EOF blanks. The [scoped check](parity/reports/artifacts/material-polling-publication-checks-20261117/whitespace-001/SCOPED-RESULT.json), excluding only those eight paths, passed 0. Raw diagnostic excerpts retain their whitespace; no raw artifact was edited and no full whitespace pass is claimed.

Tested source remains exact `9d4b629c077f06c348f6615fe90df7bbeacedb24`, reusing its original 412-test/fmt/strict-Clippy/native+CLI source gates. The [full Harbor report](parity/reports/WF-20261117-MATERIAL-POLLING-HARBOR-A8-001.json) remains **FAILED STRICT QUALITY**: all 181 states were analyzed; frame 29 warp strain 0.1558088708602663 exceeds 0.15 at triangle 618/piece 31. Operational completion at 298.4210457500303 s / 420 s and final 9.430147% strain do not override that failure. All 181 finite/contact/uncertainty screens pass. The old global 153/180 failure remains ungraded; the paired trajectory failure remains historical. All 56 full scopes/statuses/scenarios/dependencies stay exact: 41 partial, 15 planned, zero full or reference-verified.

This closure adds only the QC derivatives in the [manifest](parity/reports/artifacts/material-polling-publication-checks-20261117/MANIFEST.json) and this leading STATUS section; all 156 checked inputs and the prior STATUS body remain exact. Copies use only declared ROOT/HOME locator substitutions and 0644 modes. Separate unpublished source/native increments stay unselected. Actions/automated Pages remain disabled; README/ROADMAP/gallery and static Pages stay exact. No additional check, generator, Cargo, physical/native execution or network action occurs for this closure.

Separate unpublished local work: G005 SaveReset on `d982d3af` passed its bounded scope (seal `8fb231f4`). A new full-case attempt on the same source stopped at 60 completed steps / 61 states because the directory-volume monitor raced an atomic OBJ temporary-file rename (seal `a2a3686c`). This incomplete prefix is **UNGRADED**; no full-quality result exists. No native/model payload is selected, and neither local result supplies parity or gate credit.

Next: complete independent full-garment quality verification, calibrate material and contact behavior, extend reference workflow coverage, and finish the remaining roadmap work. Full garment, calibrated physics, reference and roadmap acceptance remain open.

## Historical material polling metadata preparation

Everything below preserves the prior STATUS body and its original current/pending/next wording. Its target-absent, generation-pending and earlier checkpoint descriptions are historical; the clean 92ac verification above is current.

## Material polling source and strict Harbor failure checkpoint

Current local integration `07674f14190a24af0e1c28cb503e0dec0d8dd6a8` normally merges public PR58 `b5d6e4d25ff1607c511e26da8af6c2642800eec0` with exact reviewed `9d4b629c077f06c348f6615fe90df7bbeacedb24`. All88 runtime/source files match the actual412-gate freeze; only constraints.rs,tests.rs and docs/physics.md change. The [source report](parity/reports/WF-20261117-MATERIAL-POLLING-SOURCE-001.json) passes its bounded scope:64-checkpoint cancellation cadence, entry/pre-success checks and transactional rollback, unchanged material arithmetic/PCG32/Armijo16. Original four full source gates passed412 Rust tests/fmt/strictClippy/native+CLI debug builds. The first formatting-only failure and separate corrected focused10+8+3 checks/16unique tests remain exact; no green gate is rerun here.

The [new full Harbor report](parity/reports/WF-20261117-MATERIAL-POLLING-HARBOR-A8-001.json) is **FAILED STRICT QUALITY**. One fresh180-frame producer supplies181 independently analyzed states, operationally accepted at **298.4210457500303s/420s** after all root persistence; the independent external terminal observation is398.1007409580052s, also below420s. Session93936 exits2. **Frame29 warp strain0.1558088708602663 exceeds0.15**, triangle618/piece31/nodes228,106,107. Final-frame9.430147percent does not override that trajectory failure. All181 finite/supported contact<2mm/uncertainty screens pass, completed-fit seam<5mm from exactzero29 passes and assembly targets are exactlyzero at60/180; initial seam/target failures0..28 remain. Producer253.2211085s/analysis37.571655499981716s and all raw states are retained. Owned PIDs21369/21379/21626/21562/34803/32881/21590 are confirmed absent in the original terminal record.

The initial new outer clock-order blocker was found and fixed before numerical GO; initial/fix/final reviews and raw preparation errors remain. The [case closure](parity/reports/artifacts/material-polling-source-20261117/harbor/closure/FINAL-MANIFEST.json) preserves all118 original members plus its own manifest. Published52099 global153/180 wall-cap failure remains ungraded; old paired full strain failure remains historical. Only the two affected global reports change freshness/reason after exact archival. New31-file scoped reports pin tested9d4; the full88 gate ledger remains separately attributable.

The [selected manifest](parity/reports/artifacts/material-polling-source-20261117/SELECTED-PUBLIC-MANIFEST-001.json) binds286 originals plus itself:287 artifacts,970 ROOT/652 HOME locator substitutions and declared public0644 modes. The kernel5.476845906730992x median ratio is bounded component-only evidence with all6pairs/32x-p boundaries bit-identical, not whole-garment speed. The original release build was externally stopped for disk protection; separately authorized recovery002 passed149.98046270909254s using its own partial target. Its inherited cold_own_target:true banner is explicitly wrong and remains frozen with the authoritative correction. No target/cache/binary/image/installer/account/OS-dialog or unfinished native evidence is selected.

The source map retains all390 prior identities/history, re-pins7 surviving records and adds4 actual helper/test definitions for394 active anchors;383 records remain exact. Original8/949/205/56,39rows/23surfaces/23contracts/34variants/33schemas/nine tools and all56 full normative/coarse statuses/scopes/scenarios/dependencies remain unchanged:41partial/15planned/0full/0reference-verified. Earlier reports and all prior failures remain visible.

This is reviewable metadata preparation only. Generated progress/dashboard remain at the preceding checkpoint until root GO; no generator, registry/map checker, xtask, Cargo/build, numerical/evaluator/UI/native replay or service write is run. New integration target is absent; no compiled copy occurs. Source-only local merge and read-only PR58 metadata lookup are the only Git/service integration actions. No push/PR/service merge/deploy is authorized. Actions/automated Pages stay disabled; README/ROADMAP/gallery/existing static Pages remain exact.

Local verification after this preparation: native case `WF-20261117-NATIVE-HARBOR-G003` completed the empty-project route on exact `9d4b`: 28 typed points, two editable pieces, eight oriented seams, assembly/drape, dimension and density edits, a fresh-process reopen with toolbar Undo/Redo, and native project/OBJ/SVG exports. The independent checker passed all eight saved outputs. Root separately checked action order, both process lifecycles and the original resource limits. Its retained manifest is `d97959f7a5be4064816b9688f286c73bccf8fe053fd68fe8c7f920b5476989ef`; the private traces are not selected in this publication. This bounded workflow/output result does not override the failed frame-29 physics trajectory or establish reference parity or a roadmap gate.

Two separate unpublished source increments have been combined locally: `72219ad5` adds current rest-area/actual Cloth-mass readouts with focused native checks; `22a7cb99` corrects bending displacement history and passes a fixed independent pinned-hinge oracle plus cancellation/reset checks and independent source review. Combined candidate `d982d3af` passed formatting, 416 workspace tests with zero failures or ignored tests, strict all-target Clippy and native/CLI builds in 253.43 seconds. All 88 source, 59 inherited metadata and eight original-input guards passed before and after every stage. Native visual verification of the readout and a newly frozen garment test of the bending change remain pending. These local checks do not change this publication's tested `9d4b` source or its preserved outcomes.

Next: review this selection/report/map/freshness tree, then run one metadata generation cycle and clean committed verification before publication. Publish the separately reviewed native evidence, verify the mass inspector in the app, and evaluate the bending change on a fresh complete trajectory. Calibrated physics, reference parity and full garment/feature/roadmap acceptance remain open.

## Historical global material publication checkpoint

Everything below preserves the preceding STATUS body and its original current/pending/next wording. Its source/check/case identities remain historical and are not silently refreshed.

## Clean global material publication verification

Checked metadata commit `03cf087a6b67c6656e2a7974669fb01e06e0c645` passed root's fully pinned `cargo xtask verify`: session5276 / PID34530, exit0 in **218.61593933298718s**. Registry/generated progress, historical/current source maps,48 script tests,6 evaluator tests and actual CLI help passed. All **154 inputs** (88source,61reports,5metadata including the committed generated outputs) remained exact and the checkout stayed clean. See the [clean verification receipt](parity/reports/artifacts/global-material-publication-checks-20261117/xtask-verify-001/RESULT.json), [stdout](parity/reports/artifacts/global-material-publication-checks-20261117/xtask-verify-001/stdout.log), [stderr](parity/reports/artifacts/global-material-publication-checks-20261117/xtask-verify-001/stderr.log) and [derivative manifest](parity/reports/artifacts/global-material-publication-checks-20261117/MANIFEST.json). Original source409/fmt/strictClippy/nativeCLI evidence is reused on tested `52099b6f6b7aff43b3e9ada8bad4f830cb987134` and all88 exact source bytes; those gates are not rerun.

The [original generator result](parity/reports/artifacts/global-material-publication-checks-20261117/generator-cycle-001/RESULT.json) keeps actual generator exit0 and wrapper exit2: session15672 / PID28417,48.24499995901715s. Its preparation roster included the two explicitly allowed generated outputs inside the154-input all-unchanged guard. The [qualified assessment](parity/reports/artifacts/global-material-publication-checks-20261117/generator-cycle-001/POST-GENERATION-ASSESSMENT-002.json) separates **152 immutable non-output inputs** from the **two expected changed outputs**. All source88/61reports/3non-output metadata,227 originals,228 artifact copies,HEAD/index/GO/freeze stayed exact. Both assessment versions and the original failed wrapper receipt remain unchanged; no generator retry occurred. Root reviewed the two outputs and committed them as03cf. Counts and full56 coarse states stay unchanged:41partial/15planned/0full/0reference-verified.

The [full whitespace check](parity/reports/artifacts/global-material-publication-checks-20261117/whitespace-001/RESULT.json) remains **exit2**, with65 findings on8 exact raw patch/test-log paths. The later [exact raw-byte waiver](parity/reports/artifacts/global-material-publication-checks-20261117/whitespace-001/EXACT-RAW-WAIVER.json) binds single-ASCII-space unified-patch context lines or blank EOF test-log bytes; the separately retained scoped check excluding only those8 paths passed0. The original pending-waiver field remains historical. No raw artifact was edited and no full whitespace pass is claimed. The copied QC diagnostic stdout also retains its61 literal plus-space excerpt lines as raw output. The [independent metadata review](parity/reports/artifacts/global-material-publication-checks-20261117/independent-review/REVIEW-RECEIPT.json) found no blocker at0cad before generation; root separately reviewed the generated delta before03cf verification.

The [global Harbor case](parity/reports/WF-20261117-GLOBAL-MATERIAL-HARBOR-A8-001.json) remains **FAILED OPERATIONALLY** at420.0070624999935s/420s,153/180 completed steps. Its incomplete trajectory is **ungraded**: no full181-state analysis or computed quality grade exists. Raw strict_quality_passed:false remains nonacceptance. The bounded eight-face source/component and409-gate pass supplies no full garment, calibrated physics, native workflow, reference, feature or roadmap acceptance. All prior failures and full contracts remain visible.

This closure adds only QC derivatives and this leading STATUS section; all154 checked inputs and the previous STATUS body remain exact. Only declared ROOT/HOME locator substitutions and public0644 modes change selected copies. The local target now exists from root's clean verification and remains untouched; no compiled cache/binary is copied. QC selection itself ran no additional source tests, generator, Cargo, model/evaluator, UI/native or network action. Actions and automated Pages remain disabled; existing Pages,README/ROADMAP/gallery stay exact.

Next: finish the separately reviewed cancellation improvement, its complete source checks and the unchanged full-garment test. The next performance/cancellation candidate and current native garment work remain separate, unaccepted increments; their private partial traces are unselected and supply no public workflow credit. Complete current authoring/sewing/fitting/save-reopen-edit/export and independent full-garment quality, calibrated physics and reference acceptance remain open.

## Historical global material delivery checkpoint

Everything below preserves the preceding STATUS body and its original current/pending/next wording. Earlier pending checks,target-absence statements and case identities describe only their historical checkpoint.

## Global material source and incomplete Harbor checkpoint

Integrated source `97a152468ca34fa7edae9e4680167ba02bdccbb7` normally merges public PR57 `56959aeed92efbe67f64df8e0cc8a24e289ca7cb` and reviewed `52099b6f6b7aff43b3e9ada8bad4f830cb987134`. All88 runtime/source inputs match the actual409-gate freeze. The [source/component report](parity/reports/WF-20261117-GLOBAL-MATERIAL-SOURCE-001.json) passes its bounded scope: original eight-face stationarity RED0.0008125005684045307m/7.071746584708191N, first-candidate compile failure with zero tests, then annotation-only GREEN2.0770362583566576e-10m/1.2857517012326884e-7N at unchanged1e-8m/1e-6N bounds. Final62 sim tests and actual409 workspace tests/strictClippy/native+CLI debug gates passed originally; full fmt is reused from identical88 source, not repeated here. Original descriptor correction, preparation guard failures and every failed attempt remain exact.

The [new global Harbor A8 report](parity/reports/WF-20261117-GLOBAL-MATERIAL-HARBOR-A8-001.json) is **FAILED OPERATIONALLY**, root session86284 terminal1 at **420.0070624999935s /420s**, with **153/180 completed steps**. The incomplete raw trajectory is retained **ungraded**: no full181-state analysis or computed quality grade exists. Raw `strict_quality_passed:false` is retained as nonacceptance, never presented as a computed quality failure. Original settings/caps/screens and source88/metadata59/original8/final-freeze/producer binary stayed unchanged. Retained postcheck finds owned PIDs89614/89644/89688/89726/89763 absent; no native window was launched. Post-case closure cannot create a successful420s verdict. No retry, continuation, cap extension or prefix/final-frame grade occurred.

The [derivative manifest](parity/reports/artifacts/global-material-source-20261117/SELECTED-PUBLIC-MANIFEST-002.json) binds227 originals plus itself:228 artifacts, complete new case packet/root/closure/reviews and bounded component/source logs. Only91 ROOT/611 HOME locator substitutions and public0644 modes differ; no target, cache, binary, image, private native dialog or proprietary reference research is selected. Both formerly current paired-material reports are archived byte-exact and change only freshness/reason; prior strict strain failure28..40 and all old outcomes remain historical. The new31-file scoped reports pin tested52099; the full88/409 ledger remains separate.

The map explicitly retires four removed current literals, adds two truthful successors and two actual new regression definitions, and re-pins four surviving records:382 unchanged records and390 active occurrences, with all390 prior identities retained in exact PR57 history. All56 full normative/coarse statuses, scopes, scenarios and dependencies remain exact:41partial/15planned/0full/0reference-verified; original8/949/205/56 and39rows/23surfaces/23contracts/34variants/33schemas/nine tools remain fixed.

This metadata checkpoint has **not** run a progress generator, registry/map checker, xtask, Cargo, model/evaluator or UI/native case. Progress/dashboard retain the preceding generated checkpoint pending root review GO; preceding clean verification receipts below apply only to their exact earlier trees. The new integration target is absent and no cached target/binary was copied. No push/network/publication occurs. Actions and automated Pages remain disabled; README/ROADMAP/gallery and static Pages remain unchanged.

Next: root independently reviews the exact tracked metadata/provenance, then authorizes one bounded progress generation and later clean committed verification. Address whole-case performance in a separate reviewed source increment; complete current native authoring/sewing/fitting/save-reopen-edit/export and independent full-garment quality remain open. No calibrated-physics, reference, full-feature or roadmap acceptance follows.

## Historical paired-material publication checkpoint

Everything below preserves the preceding STATUS body and its original current/pending/next wording. Earlier source/case/check identities and failures remain historical; this closure does not transfer their acceptance.

## Clean publication verification checkpoint

Clean metadata commit `bb8ae0d2b3db6e09c96c65f3ac5dddefc9695267` passed the fully pinned `cargo xtask verify`: session 28423 / PID 39198, terminal exit 0 in **223.09856833296362 s**. Registry/generated progress, historical/current maps, 48 script tests, six evaluator tests and actual CLI help passed. All **152 inputs** (88 source, 59 reports, five metadata) remained exact and the checkout stayed clean. See the [passing receipt](parity/reports/artifacts/material-group-publication-checks-20261117/XTASK-PINNED-RESULT-002.json), [raw log](parity/reports/artifacts/material-group-publication-checks-20261117/XTASK-PINNED-002.log) and [derivative manifest](parity/reports/artifacts/material-group-publication-checks-20261117/MANIFEST.json). This receipt/status-only closure awaits final root review and publication.

The [first attempt](parity/reports/artifacts/material-group-publication-checks-20261117/XTASK-RESULT-001.json) remains failed: session 53727 / PID 12989, exit 1 in 226.15059099998325 s. Its 48+6 Python tests passed before CLI-help compilation encountered mixed rustc 1.100/1.94 crates. Root pinned RUSTUP_TOOLCHAIN, RUSTC, RUSTDOC, CARGO and PATH to 1.94 and used a fresh own nested target. No source, input or criterion changed and no cache/binary was copied. Both attempts kept all 152 inputs exact and clean. The worktree target now exists from verification and remains retained. No check is repeated for this closure.

The [independent review](parity/reports/artifacts/material-group-publication-checks-20261117/independent-review/review-receipt.json) reports **NO BLOCKER** on checked tree `24a8752bf4ceb032274581d2d93c90c82cf2a623`. Root corrected only the active Markdown `solve_stitch` locator from 164 to 387 before committing. The [whitespace review](parity/reports/artifacts/material-group-publication-checks-20261117/ROOT-WHITESPACE-REVIEW.json) retains the original full-check failure: 31 findings on seven exact raw-artifact paths. The scoped check excluding only those preserved raw paths passed; no full whitespace pass or raw-byte repair is claimed.

The tested paired solver remains `13fb43fe08c045f08a1b85007aff3c15388a1e24`, with 407 workspace Rust tests and all 88 source identities retained. Its [full Harbor case](parity/reports/WF-20261116-MATERIAL-GROUPS-HARBOR-A8-001.json) still **fails strict strain at frames 28–40**, peaking at 21.96202527055029% in frame 29. All 181 states were analyzed; operational completion at 251.83977058297023 s / 420 s does not override strict quality failure. Source407 gates are reused. Native, full-physics, reference, full-garment and roadmap acceptance remain open.

All source, registry, reports, maps, progress/dashboard, README/ROADMAP and historical evidence inputs remain unchanged. This closure adds only QC derivatives and this leading status section. Actions and automated Pages deployments remain disabled; the existing static Pages site stays exact. No Cargo, generator, test, UI/native, model/evaluator, network or publication action occurs here.

Next: root reviews and publishes this bounded checkpoint. The separate global solver correction still needs its own workspace and full-garment evidence.

## Historical material-group delivery checkpoint

Everything below preserves the preceding status body and its original current/pending/next wording. Its source/case identities and failures remain historical and exact; this closure adds no numerical or native result.

## Material-group source and failed Harbor checkpoint

Current integration `8fe1c7b3018e807a61afffc3d0af77f5bbbe45d7` merges safe PR56 with exact reviewed `13fb43fe08c045f08a1b85007aff3c15388a1e24`. All88 source inputs match the actual407-gate freeze. The [bounded component/source report](parity/reports/WF-20261117-MATERIAL-GROUP-SOURCE-001.json) is **passed**: the independent two-face physical-energy counterexample changes from RED4.266274346685227e-7m/2.7341292831298387e-3N to first GREEN5.683890753411727e-18m/1.0747913121812962e-14N at fixed1e-8m/1e-6N bounds. First-GREEN source and later test/docs stabilization remain separately mapped, including the failed first Clippy attempt. Final60 sim tests and full407 workspace tests/fmt/strictClippy/native+CLI builds passed originally; no source or numerical gate is rerun here.

The [new full Harbor A8 report](parity/reports/WF-20261116-MATERIAL-GROUPS-HARBOR-A8-001.json) is **failed**. One fresh dynamics run/180frames supplies181 independently analyzed states. Root operational completion is accepted at authoritative251.83977058297023s/420s, after all root persistence. The slightly earlier raw ROOT-RESULT251.83914029202424s remains exact. Both fresh initializations share that clock; the independent original-rest gate takes zero steps. Original654 vertices/1172 triangles, mass0.21322800000000003kg and all geometry/material/settings/counts/screens remain unchanged.

**Strict trajectory failure remains:** strain fails frames28..40, peak21.96202527055029percent at frame29/triangle214/piece2/weft, nodes39/500/501. Final-frame14.03608985percent does not override those failures. All181 finite/contact/uncertainty screens pass; completed-fit seam from exactzero frame29 passes; initial seam/target failures0..28 remain. No fabric calibration, full inter-group/bend/sew/contact equilibrium, native garment, reference, full-feature or roadmap acceptance follows. The raw inherited `new_garment_execution:false`/saved-state wording remains, qualified by actual fresh producer/root receipts rather than rewritten.

The [selected manifest](parity/reports/artifacts/material-group-source-20261117/SELECTED-PUBLIC-MANIFEST-001.json) binds258 selected originals/derivatives plus the generated manifest. It preserves the complete raw trajectory and analysis, exact13-keyGO/template resolution/final freeze/preflight/protocol review/root witness/terminal closure, V1 build-only monitor failure and V2 fix/build. V1's observation timeout killed live Cargo before any model ran; it establishes no product compile/physical failure. Only declared108 ROOT/200 HOME locator substitutions and public0644 modes differ;47 original files were0600. No binary/target/cache/image/proprietary reference evidence is selected.

Five formerly current reports are archived byte-exact and change only freshness/reason: coupled material source, coupled Harbor A8 and all three Rectangle reports. Their results/scopes/artifact maps stay historical, including Rectangle's original900s deadline failure and separate file-only pass. Rectangle-related feature freshness becomes stale for changed declared sim dependencies despite unchanged GUI bytes. New physical source/full-case reports bind31 files; the full88/407 ledger remains separate. All56 normative/coarse statuses remain exact:41partial/15planned/0full/0reference-verified. Original949/205/56 inventory and all eight original files remain exact. The current source map retains385 old anchor identities, re-pins3 existing sim occurrences and adds5 real method/test anchors for390;39/23/23 and34/33/9 remain fixed.

Confirmed original jobs are terminal: root session13979/supervisor70464/controller70473 and all five children are absent in the retained closure. No native window was opened. The new integration has no target/cache copy. The one authorized metadata progress generator passed: PID89362/session31944, exit0 in61.893072708975524s, with all150 immutable source/report/registry/map inputs unchanged. Progress and dashboard are its separately recorded outputs. No generator retry or standalone registry/map/Cargo/numerical/native check occurred. Root owns independent staged review, one later clean committed `cargo xtask verify` and publication. Actions/automated Pages remain disabled; README/ROADMAP/gallery and existing static Pages stay exact. No remote operation occurs.

Next: review the exact bounded metadata/selected artifacts and retain strict trajectory failure; then complete root-owned committed metadata verification. Diagnose remaining material strain from the retained outputs under a separate reviewed source/protocol. Full native authoring/sewing/fitting/save-reopen-edit/export, calibrated physics/reference and later roadmap gates remain open.

## Historical safe Rectangle checkpoint

Everything below preserves the preceding Rectangle/material checkpoints and their original current/pending/next wording. Their source/case/verification attributions stay historical; no result or scope is repaired or transferred.

## Safe Rectangle publication checkpoint

Clean metadata commit `87843ed51bf1d325bf11a2a7a39dd1150dc3f625` passed
`cargo xtask verify` in 226.473 seconds: registry and generated progress, historical
and current maps, 48 script tests, six evaluator tests and CLI help. All 150 inputs
(88 source, 57 reports, five metadata) remained unchanged, and the checkout stayed
clean. The actual tested source commit `7d19` is retained in ancestry. See the
[verification receipt](parity/reports/artifacts/rectangle-publication-checks-20261117/SAFE-XTASK-RESULT-002.json)
and [publication manifest](parity/reports/artifacts/rectangle-publication-checks-20261117/MANIFEST.json).

Before the first push, privacy review found display-profile identifiers in nine
screenshots. Public copies remove eight metadata bytes per image; encoded scans,
color-characterization tags and all other bytes match the private originals.
The [current index](parity/reports/artifacts/rectangle-gesture-20261116/PUBLICATION-CURRENT.json)
records the correction. Original cases and outcomes remain exact. The native
900-second case still failed its finalization limit; the separate file check
passed. No whole garment, physics, full feature or reference acceptance follows.
The [preview gallery](docs/studio-preview.md) shows the actual native creation
and reopened editing captures with these limits.

This receipt and status closure changes no checked source/report/map input.
Local verification PID 32431/session 47796 is terminal exit 0. No owned native
window or Cargo job remains in this checkout. Publishing targets a draft on
PR #55 with Actions disabled and the existing Pages site preserved. The later
material-group repair and its 407-test gates belong to a separate increment.

## Rectangle staged delivery checkpoint

Integrated source `ae8dd6ebe6da760501f507248a11b069a02cb453` merges PR55 math metadata with Rectangle `496b9bc2b336c9193d0508837c17b6170582490e`. All **88 source inputs** exactly match actual tested `7d19f86a006601133d457aee42cca3b115b994aa`; [the integration receipt](parity/reports/artifacts/rectangle-gesture-20261116/source-integration/ROOT-INTEGRATION-001.json) retains that binding. `main.rs` is the only Rectangle fix file. One original source cycle passed formatting, **404 workspace Rust tests**, strict all-target Clippy and native+CLI debug builds. The original Rectangle filter is six new plus two existing tests; the original red covered two new cases only. Intermediate guard/snap-fixture failures remain exact. No source/Cargo/native/checker/evaluator case was repeated for this staged metadata checkpoint.

The [source report](parity/reports/WF-20261116-RECTANGLE-SOURCE-001.json) is **passed**. The [original native report](parity/reports/WF-20261116-RECTANGLE-NATIVE-001.json) is **failed**: RG001 finalization closed at **1008.460155082983 s against900 s**, `INCOMPLETE_FINALIZATION_DEADLINE_EXCEEDED`, whole-case bounded acceptance false. It retains actual emptyR0 → one400×600mm RectangleR1 → UndoemptyR2 → RedoR3 → native save/quit → explicit freshP2 reopenR3 → one10mm corner editR4/save/quit. UI ended at560.473s/760s; cleanup/source guard at649.864573250059s is separate. Three clock-helper errors preceded the first app API call; no app call was replayed. All35 app calls,18 observations,9 original images,19 actions and both owned-process identities remain attributable to that original case. Retained cleanup says both owned PIDs were absent; this checkpoint launches no native process.

The [separate file-only supplement](parity/reports/WF-20261116-RECTANGLE-FILE-SUPPLEMENT-001.json) is **passed**: FS001 ran the previously frozen checker exactly once in0.07889999996405095s/60s, after68 earlier synthetic self-checks, with all inputs/outputs/original-case seals unchanged. This cannot regrade native timing or establish UI, gesture, lifecycle, native reopen, reference, physics, full-feature, full-garment or roadmap acceptance. The native case remains failed.

The [historical selected manifest](parity/reports/artifacts/rectangle-gesture-20261116/SELECTED-PUBLIC-MANIFEST-001.json) remains exact, with its nine original JPEG hashes/whole-byte wording explicitly superseded by [the current public index](parity/reports/artifacts/rectangle-gesture-20261116/PUBLICATION-CURRENT.json). The current union has **212 artifacts**: the previous210 plus the redaction journal and current index. All other201 prior artifact entries remain exact. Original text derivatives retain71 ROOT/0 HOME substitutions. The9 public JPEG copies clear only eight ICC display identifier bytes each; encoded scans, color-characterization tags and every other byte remain exact. [The metadata journal](parity/reports/artifacts/rectangle-gesture-20261116/JPEG-METADATA-REDACTION.json) binds original/current hashes and allowed fields. All209 private original inputs/case/results/clocks remain unchanged. The app bundle, executable/font/resource payloads, targets/caches and external commercial reference evidence remain excluded.

New report source bindings use the complete declared feature/dependency closures (**13/17 files**); the full88-source/404-gate identity remains a separate historical ledger. Only PAT-001/PAT-003/UI-004/FILE-001/APP-001 receive bounded evidence refs, explicit current freshness and appended gap notes. All **56 coarse implementation/verification/reference observation statuses and normative contracts** remain exact. Existing math31-current scopes/gaps/freshness and all54 preceding report bytes, including endpoint stales, remain unchanged. The current source overlay preserves379 prior anchor identities and appends six actual test anchors for385 current occurrences;39 rows/23 surfaces/23 contracts,34 variants/33 schemas/9 tools and the original949/205/56 inventory stay exact.

The first progress generator (PID79164/session43218, exit0,51.572899500024505s) and original unredacted checked state remain private historical evidence. Root found a concrete ICC display identifier before publication and applied the established metadata-only cleanup. Its old verification result does not verify this corrected public tree. The one corrected privacy progress generator passed: PID12159/session50755, exit0 in73.14317645796109s, with normal stdout and empty stderr. Its150-input guard retained149 inputs exactly; generated progress changed only the five approved privacy gap lines. The wrapper exit2 records its over-strict generated-output assertion and is retained privately; no generator or standalone check was repeated. Source88, the385-anchor map and all prior outcomes remain fixed. No Cargo/native/checker/evaluator/standalone registry/map/full-whitespace/remote action is run for this amendment. Root owns narrow delta review, safe commit/actual tested7d19 ancestry and the justified later clean metadata verification. Actions and automated Pages deployments stay disabled; existing Pages, README and ROADMAP stay exact.

Next: retain the original native timing failure, separate file-only pass and private pre-redaction history; independently review this exact privacy/gallery delta, then complete the root-owned safe commit/ancestry and clean metadata verification. Complete current empty-project garment authoring, sewing/fitting/save-reopen-edit/export, calibrated physics/reference behavior, advanced editing and later roadmap gates remain open.

## Historical material publication checkpoint

Everything below retains the preceding material/endpoint checkpoint and its original current/pending/next wording. Its source/case/verification identities remain historical and exact; the Rectangle scope above supplies no additional math, physics, reference or whole-workflow acceptance.

## Reviewed publication checkpoint

Metadata commit `2ac29945142f6675d606b480331c3e2062d972ba` passed one clean
committed `cargo xtask verify` in 204.238 seconds: registry and generated progress,
historical/current maps, 48 script tests, six evaluator tests and actual CLI help.
All **147 verification inputs** (88 source, 54 reports, five metadata) remained
unchanged, and the checkout was clean afterward. See the [retained local
verification receipt](parity/reports/artifacts/material-publication-checks-20261116/XTASK-RESULT-001.json)
and [public derivative manifest](parity/reports/artifacts/material-publication-checks-20261116/MANIFEST.json).
The independent review corrected only two predecessor digest literals against
actual f79 Git objects. Original whitespace failures remain retained; the passing
scoped check excludes exactly five preserved raw-artifact paths. Source/math gates
are reused, and the complete Harbor strain failure remains unchanged. This later
receipt/status-only checkpoint adds no source or report changes and repeats no
checks. Local verification PID 54511/session 52320 is terminal exit 0; no owned
Cargo or WeftForge GUI job is active. Publication is prepared on
`feat/cloth-delivery-integration`, stacked on PR #53, with Actions disabled and
existing Pages preserved. The full 56-contract target remains unfinished.

Current physical source is actual tested `96d3baee4543c7edbdb804df6090697a9f65da09`, preserved byte-for-byte across all **88 source inputs** at delivery integration `a9d5cf72cc966e6a2b8fc76fc245951f7c7ef031`. The reviewed material correction `36524293b89686c2368f22d13ad09289fc70af03` and actual endpoint source remain in ancestry; trace47 stays excluded. Later Rectangle/UI source and its separate 404-test gates belong to another increment.

The [bounded source/math report](parity/reports/WF-20261116-COUPLED-MATERIAL-SOURCE-001.json) is **passed**. Its independent small-patch inertial/elastic equilibrium reference changes from RED on unchanged production (2.843544361e-6 m position error / 1.763599204e-3 N force residual) to GREEN on reviewed material source (8.810015435e-16 m / 5.759318500e-13 N). Material sweeps retain positional corrections and use a coupled 3×3 Gauss–Newton/Woodbury descent with at most 16 Armijo halvings; material-loop cancellation preserves whole-call rollback. Later cancellation/docs changes have their own final source maps. Bending, sewing and contact retain their prior algorithms; full coupled-contact equilibrium and fabric calibration remain unproved.

Actual96 retained formatting, **398 workspace Rust tests**, strict all-target workspace Clippy and native+CLI debug builds, each passed once. These are exact-source historical gates, not new execution or current whole-workspace/native acceptance. The native executable was built without launch. CLI invocation occurred in workspace integration tests; the original build-only `launched:false` field remains qualified by its separate launch-attribution record. No executable, target or cache is distributed.

The [new original Harbor A8 numerical report](parity/reports/WF-20261116-COUPLED-HARBOR-A8-001.json) is **failed**. One root-authorized fresh headless case operationally completed: controller exit 2, all 181 states analyzed without errors/disagreement/blocked diagnostics, and 234.47424437501468 s through the root outer receipt against the unchanged 420 s cap. Both initializations share that clock; init-only makes zero steps and exactly one fresh dynamics run occurs. Original geometry/material/rest/contact, 1/60 s timestep, 8 substeps, 32 iterations, 1 m/s seam closure and strict 15% strain / 5 mm seam / 2 mm contact screens remain unchanged. Original rest has 654 vertices / 1172 triangles and 0.21322800000000003 kg mass; only four empty v5 piece fields adapt serialization.

**Strict trajectory failure remains:** material strain fails frames 28–37. All 181 finite, supported static contact and uncertainty screens pass; target first reaches exact zero at frame 29 and is zero at 60/180. Initial target/seam failures remain visible and final-frame pass cannot override the complete strict failure. The analyzer's inherited saved-state wording remains exact and is qualified by fresh producer/controller/root receipts. Bounded contact samples do not establish whole physics, equilibrium, continuous/edge-edge/moving-avatar collision, native garment workflow or reference parity. No old case or archived replay identity failure is repaired, replayed or regraded.

The [selected manifest](parity/reports/artifacts/material-source-20261116/SELECTED-PUBLIC-MANIFEST-001.json) binds **167 selected originals plus the generated manifest**: 40 source/math files, all 125 preparation/case/root files, and two exact endpoint-report archives. It retains raw 15,389,222-byte trajectory data and all invalid synthetic fixture, superseded build, failed first seal and second refusal history. Only declared plain ROOT/HOME locator substitutions change text bytes (87/25); original/public hashes, sizes and modes distinguish derivatives. Public mode 0644 differs from 33 recorded original case permissions. Raw vendor/reference R011/R012/R013 and compiled binaries/targets/caches remain wholly excluded. This is an audit packet, not replay authorization.

The full target remains **56 complete contracts: 41 partial, 15 planned, 0 full implementations and 0 reference-verified**, with discovery incomplete. All original scopes, scenarios, criteria, dependencies and coarse implementation/verification/reference observation statuses remain exact. Relevant physical freshness uses 31 source files; the full 88-source/398-gate identity remains a separate ledger. Current evidence covers only the agreed bounded SIM scopes; SIM-005 receives an attribution cross-reference without a current feature verification claim. Evidence freshness is 5 current / 30 stale / 21 missing. Both endpoint reports are stale solely because their original sim-source scope changed; exact originals precede two-field freshness/reason transitions. Other 50 prior reports and all eight original inventory files stay exact.

The source map pins a9 with **39 rows / 23 surfaces / 23 contracts / 379 current anchors**, preserving all 378 old identities, including two sim hash/line repins, and adding the one actual material-method anchor. Historical 378/predecessor counts and 949/205/56 inventory remain unchanged. No source-map anchor confers execution or acceptance.

Confirmed jobs are terminal: combined gate shell 9326 / children 15044, 15161, 20377, 20932 all exit 0; release V2 shell 7200 / child 71066 exit 0; the separate root case controller exits 2 with bounded operational acceptance true / strict quality false. No owned Cargo job or GUI window is active. The first progress-generator attempt (PID 28555) stopped before generation because the approved UTC timestamp used an offset rejected by the existing schema. Its failed receipt/log remain exact. Root authorized the identical instant in Z form only for two new reports and seven new run mirrors; corrected generator PID 29565 passed (session 92347, exit 0, 50.893143 s). No source, schema, scientific result or existing timestamp changed. This metadata checkpoint awaits root staged review and one later clean committed `cargo xtask verify`; no standalone registry/map test or Cargo gate is repeated here.

Next: retain the failed strict case and diagnose its remaining strain under a distinct reviewed source/protocol. Complete current empty-project native garment authoring, sewing/fitting, save-reopen-edit/export, physics/reference acceptance, advanced editing and later roadmap gates remain open. GitHub Actions and automated Pages deployments stay disabled; the existing static Pages site, README and ROADMAP stay exact. No remote, issue, PR or deployment operation occurs in this metadata step.

## Historical endpoint metadata checkpoint

Everything below retains the exact preceding endpoint checkpoint and its earlier historical material, including original current/pending/next wording. Its 397-test/native outcomes remain attributable only to that original source and cases; the source report freshness is now stale.

Current tested endpoint source is `3fddeb9a72e5b4395e8e874a628086937331af2f`, preserved in integrated
checkpoint `ac6c2e53b47f2d19772d2bfb341e1500dae2cf0c` with all 87 source inputs byte-identical.
The [source report](parity/reports/WF-20261116-ENDPOINT-EDIT-SOURCE-001.json) retains **397 workspace Rust tests**,
strict all-target workspace Clippy and native debug build, each passed once.
Full formatting passed on the exact main source and is reused with its explicit
qualification. Focused final green has eight test functions; original red 6134
had 5 passed/3 failed at 787fcadc and excludes later ownership/Escape/pending-grain
variants. No old red or native case was rerun, repaired or regraded.

The [bounded native/file report](parity/reports/WF-20261116-ENDPOINT-EDIT-NATIVE-001.json) retains one 10 mm endpoint
drag R5→R6/P0#3(-.02,-.01), one Undo R7 and Redo R8, three native saves and native
Open of final R8 in the second independently verified fresh owned process.
The separately frozen, single-run file check passes all authoring/ID/revision and
straight-edge area targets within 1e-12 SI; maximum area error 1.942e-17 m² and
maximum coordinate error 0. It transfers no UI/lifecycle/physics acceptance.

**Lifecycle qualification:** after first Quit, an AX query returned a new empty
process rather than App quit error. Its startup mechanism was untraced;
observer auto-launch is inference. Verified second PID 3213 was used for the
predeclared read-only reopen; no third launch or second drag occurred. Both owned
PIDs 2719/3213 were independently absent afterward. Native saves, working project,
autosave and recovery writes remain retained. D003 does not qualify as a compliant
full workflow or native-broad/reference/physics/full-feature/garment/roadmap pass.
Its 532.051439 s / 900 s, 24 semantic actions, 22 physical calls, 55 app calls,
33 observations and 8 original images remain exact; public selection retains
42 of 165 files, 9 standalone AX and 3 unchanged own-app images, with omissions mapped.

The complete target remains **56 full contracts: 41 partial, 15 planned, 0 full
implementations and 0 reference-verified**; no roadmap gate is accepted.
Four b776 reports are stale with exact originals archived and only freshness/reason
changed; all other 46 prior reports and eight immutable inventory files stay exact.
The current source map pins integrated ac6 with 39 rows / 23 surfaces / 23 contracts and
378 actual anchors; command/schema/tool identities remain 34/33/9.
E001 retains its failed point drag and budget overruns; D001/D002 remain failed
no-commit cases. D002 trace47 is historical diagnostic-only and excluded from
shipping ancestry. See [endpoint issue #52](https://github.com/thematheusmello/weftforge/issues/52).

Confirmed source handles are terminal: focused green 35485 / Cargo 86429;
workspace 89394 / Cargo 91355; Clippy 60306 / Cargo 95040;
native build 12453 / Cargo 95894, all exit 0. Original red 6134 / Cargo 78888 exited 101.
No Cargo job or native window is active. The one local metadata cycle passed:
current-map PID 22434, progress generation 23370, progress check26435 and 39 registry
tests PID 29660, all terminal/exit 0. The [tracked checkpoint](parity/reports/artifacts/endpoint-edit-source-20261116/METADATA-CHECKPOINT-001.json)
retains exact commands, raw logs and before/after invariants. Full staged whitespace
checking exits 2 on preserved raw log EOFs and valid single-space blank unified-diff
context lines; no shipping source changed or full whitespace pass is claimed.
Root authorizes a scoped check excluding only these exact preserved artifacts:

- [workspace-tests.log](parity/reports/artifacts/endpoint-edit-source-20261116/broad-gates/workspace-tests.log): original blank EOF.
- [green.log](parity/reports/artifacts/endpoint-edit-source-20261116/green.log): original blank EOF.
- [red.patch](parity/reports/artifacts/endpoint-edit-source-20261116/red.patch): structural blank context markers.
- [green.patch](parity/reports/artifacts/endpoint-edit-source-20261116/green.patch): structural blank context markers.

All raw bytes remain exact. The single scoped staged check passed (PID 39171,
exit 0); every other staged file was included. Root independently reviewed all
56 preserved full scopes, eight original inventory files, 46 unchanged reports,
four freshness-only archives and 85 new report artifact references.

Clean committed `cargo xtask verify` passed at `f79f88e5749c24fb642bee5f75979c957c600f40`
(session 34874 / PID 47646, terminal exit 0, 206.643517 s). It checked the registry,
generated progress, source map, 48 script tests, six independent evaluator tests
and CLI help. All 144 frozen source/report/metadata inputs stayed exact and the
checkout remained clean. The [publication evidence](parity/reports/artifacts/endpoint-edit-publication-20261116/PUBLICATION-MANIFEST-001.json)
retains the actual logs, hashes, root review and input guards; published locator
normalization is declared separately. The endpoint's 397 Rust tests, formatting
and strict Clippy are reused from byte-identical tested source, not rerun here.
This verification supplies no additional native, physics or reference acceptance.

Remaining work includes Rectangle/Transform/middle-pan gesture behavior, complete offset
first-phase/advanced authoring tools, current empty-project garment authoring,
sewing/fitting/save-reopen-edit/export, physics/reference acceptance and all later
roadmap gates. No physics3652429 merge belongs to this endpoint increment.
GitHub Actions and automated Pages deployments stay disabled; the existing static
Pages site is preserved. This checkpoint makes no remote or deployment change.

## Historical source and evidence checkpoints

Everything below records its original earlier context, including prior current,
pending and next-step wording. Those source identities/results remain historical;
b776 and the392-test4d8 checkpoint are not the current endpoint source.

The source follow-up for [endpoint input issue #52](https://github.com/thematheusmello/weftforge/issues/52)
reconstructs one completed primary endpoint drag when its press/move/release
arrives in one frame. It uses the existing boundary/internal/hole hit selection
and typed transaction, with enabled, pause, clip, layer ownership, movement and
unambiguous-input guards. Both normal and reconstructed endpoints use the
release event coordinate. Pending inspector edits stay in that same transaction
when later hover leaves the canvas; invalid final geometry preserves history.

The exact original paused-test input
`787fcadcbb1ceda7bb6f68d7f62ee291fa45e44b29b4a881ed9f17fb963a7a83`
produced **5 passed / 3 failed**
on unchanged production (session **6134**, terminal exit **101**); it was not
rerun. Final source `3ddd1a624e7dd9af0c6228a12065782cc8dadef5bddb85f31d4958458f3adf21`
passed **eight focused tests** (session **35485**, terminal exit **0**) and full
formatting. Later added ownership, Escape and pending-property variants are
adjacent green coverage and were not part of that original red input. Exact
source snapshots and private receipts retain that distinction. No Cargo job or
GUI run is active.

This increment changes one source input; the source/native gates below remain
identified baseline evidence. Next: root diff review, the required broad source
gates and a separately bound native check, then source-map/report freshness and
clean committed verification. No new native acceptance or reference parity is
claimed. Rectangle, transform and pan gesture recognition remain outside this
endpoint fix. Frozen cases, full contracts, inventory, Actions and Pages remain
unchanged; temporary pointer tracing is not included.

An isolated pointer-release source increment updates endpoint preview positions
on the release frame before the existing typed `MovePoint` transaction. The
shared native handler covers exterior, internal-line and hole endpoints;
captured snap, paused-gesture handling and project validation remain in place.
Three full-Studio pointer regressions each exercise all three owners: a snapped
no-op preview followed by a 10 mm release move, a release beyond a changed
preview, and invalid final geometry preserving the old project and history.
The unchanged handler failed all three tests once (session **81295**, terminal
exit **101**); the two-line condition change passed all three (session **29690**,
terminal exit **0**). Private red/green inputs and receipts await root review
and publication; ignored local logs are not delivered evidence. No Cargo job
or GUI window is active.

This changes one frozen source input. At reviewed source checkpoint
`4d8c2c381f7df1995720e3677b9b26df6ee063cf`, full formatting, **392 workspace
Rust tests**, strict all-target workspace Clippy and the native debug build
passed once, sequentially with one Cargo build job and this checkout's own
target. All **87 source inputs** remained byte-identical before/after every
gate. Sessions **88837 / 56926 / 11098 / 93104** are terminal, all exit **0**.
Native binary identity is
`3cac4e20fa13b4b9d4305ac2fb97375a64266e1860524244d018fab24a7742f2`;
it was built but never launched. Actual raw logs, hashes and command receipts
remain private review inputs pending tracked publication. The b776 source/native
receipts below identify the published baseline.

Exact next steps: reconcile source-map/report freshness, select and track the
reviewed evidence, then run registry checks and clean committed `cargo xtask
verify`. No new native UI run or reference comparison has occurred. Completely
compressed gestures and
CUA coordinate delivery remain separate unknowns; this source fix does not
establish the cause or resolution of the closed exploratory E001 no-commit
observation. All full contracts, inventory files, disabled Actions and static
Pages remain unchanged.

Published UI source is `b7767bc94c89196f93b4ddbd1e06245f569936ff`, integrated on
`feat/photocraft-ui-integration`. Runtime source checkpoint `160562d` and
ancestry checkpoint `d4c220d` match all **87 frozen b776 source inputs**;
the tested b776 commit is reachable in the integrated history. Current-source
map checkpoint `a9b6db2` retains **39 rows / 23 surfaces / 23 contracts /
370 anchors**, **34 Rust variants / 33 schemas / nine MCP tools**. Its one
clean current-map check passed (session 7345, terminal exit 0); the original
949/205/56 inventory and eight historical files remain exact.

The PhotoCraft-inspired polish groups Pattern, Arrangement, Fabric, Create
pattern and Simulation settings; adds original Scene row icons/selection
styling; and makes existing Sewing controls a document/tool-aware disclosure.
It changes no shared authoring command, schema, file format or dependency.
Retained b776 formatting, **389 workspace Rust tests** (including 137 app tests),
strict all-target workspace Clippy and native debug build passed on that
source. These were **not rerun in this integration checkout**. Native identity:
`1bfb3b966b5ac8dbf5f16c49d58b16cf2759bbb66b0b18307a6908cc9365145c`.
Original source-gate handles 65030/40882/13931/55188 are terminal exit 0;
no Cargo job is active and no binary or compiled target crossed worktrees.

Three separately attributable implementation-only reports retain the source
and native scopes:

- [Source gates](parity/reports/WF-20261116-PHOTOCRAFT-WORKSPACE-SOURCE-001.json)
  pass only the declared source checks and four headless regression definitions.
- [N003 bounded preview](parity/reports/WF-20261116-PHOTOCRAFT-WORKSPACE-PREVIEW-001.json)
  observes one screenshot-grounded name double-click/paste, exact name retained
  after Tab, then one default Add rectangle: revision 1, selected piece #2,
  141 displayed rest vertices and matching actual Properties/Scene/2D/3D rest
  display. Four screenshots and seven AX files remain original bytes.
  Inclusive observation count 13 / cap 14 and elapsed 225.141 s / cap 420 s are qualified
  in the selected SUMMARY; the original OUTCOME is unchanged.
- [P001/N002 stops](parity/reports/WF-20261116-PHOTOCRAFT-WORKSPACE-NATIVE-STOPS-001.json)
  remain blocked before authoring. P001 retained Pattern 1 instead of the planned
  name. N002 fresh combined AX showed chrome only; no Tab/paste/Add occurred.
  Causes remain unknown; N003 is a distinct case and supplies no repair/regrade.

N003's initial standalone AX was chrome-only; full AX followed one observed
Raise. That temporal sequence establishes no product/controller cause or source
fix. No Save, Export, offset, simulation, independent geometry-output grade,
reference comparison or complete feature/workflow was executed by N003.
Broad `native_ui_verified` remains false; only its explicitly bounded preview
observation is true. Separate cleanup receipts retain absence of owned PIDs
85382/11422/16412; no WeftForge test window or WeftForge GUI lease is active.
No process/UI probe
was repeated during publication preparation.

All **129 reviewed public derivative files** are retained under
`parity/reports/artifacts/photocraft-workspace-polish-20261116/` as sibling
case and headless-gate namespaces. Case bytes/manifests remain untouched.
Original locator hashes and declared ROOT/HOME transformations retain distinct
meanings; two redundant raw_ps omissions are explicit. The two workspace raw
test logs retain their original blank EOF; only those two paths are excluded
from the scoped whitespace check. Exact old 4f80 reports are archived in the
separate history namespace before freshness/reason-only transitions to stale.
No old failed outcome, grade, identity, run or artifact map is rewritten.

## Rectangle-offset controller follow-up

[Current b776 controller stops](parity/reports/WF-20261116-OFFSET-NATIVE-CONTROLLER-STOPS-001.json)
retain two separate closed cases on unchanged source87/native1bfb. All **54
reviewed derivative files** are tracked under
`parity/reports/artifacts/offset-native-stops-followup-20261116/`.

- R004 reached clean activation AX, then the local screenshot return-type guard
  stopped storage. **Zero images** were retained. The outcome's 14.556 s and event
  ledger's 14.555 s / 16:40:41.846Z remain distinct. Delayed 0005 metadata at 16:43:04.862Z
  is not screenshot return time or API latency. The secondary wrong-record
  wx/EEXIST attempt preserved the correct 0004 AX record.
- R005 retained 13 original AX returns (seven standalone files selected, with all
  original AX strings in the selected ledger) and three original images. Draft
  name OFFSET-R1 / W0.40 / H0.60 / X0.00 / Y0.00 remained revision 0/zero pieces. Local
  `r5DraftAX is not defined` stopped before Add. Its exact failure UTC/pair
  remain missing, bounded only after 17:03:25.585Z and before 17:06:18.659Z.
  The 41 paired app calls plus final post-Quit error remain 42 app calls; the local
  unpaired JavaScript failure is not an additional app API call.

Neither case creates a rectangle/material, saves/exports, invokes offset or
simulation, or reaches a formal grader. No product defect/source fix, geometry,
native-full, physics, reference, feature or roadmap acceptance follows. The
separate cleanup receipts retain owned PIDs 68629/76165 absence without a new
process/UI probe. R005 images do not restore or regrade R004 lost return.
All 49 prior reports, 87 source inputs, 370 source-map anchors and 56 full scopes
remain exact.

Follow-up metadata cycle passed once: full staged whitespace check, progress
generation/check and **39 registry tests**. Session **47760** is terminal,
exit **0**. Report/feature inputs stayed unchanged during every command; raw
outputs and before/after input hashes are retained privately for root review.
No Cargo, build, native or reference action ran in this follow-up.

Progress remains **41 partial / 15 planned / 0 full contracts / 0 reference
contracts verified**. UI-001/UI-002/PAT-003 whole verification remains
`not_run`; PAT-009 remains `partial/blocked` with current b776 controller-stop
evidence; the earlier 4f80 source/native reports remain stale.
All 56 full behavior/scenario/dependency/comparison scopes remain exact. This
follow-up changes only PAT-009 evidence/freshness; the other 55 feature objects
remain exact. UI-006 stays planned. No roadmap gate is accepted.

The preceding UI metadata cycle passed once: progress generation, progress check and **39
registry tests**. Session **73628** is terminal, exit **0**. Raw command/output
and exact-input records are retained privately for root review; no source gates
were rerun.

Clean committed `cargo xtask verify` passed at
`195c609c3ce459045c02c30e9b08ac4491e513c9` in 183.9 seconds, with all
87 source inputs, 49 reports and five qualifying metadata files unchanged.
Session **53753** is terminal, exit **0**; original receipt
`57ad108caccf0ae50286760bdb13cbdaeb702083e72c9885a8d0553638491514`.
Selected logs are under `photocraft-workspace-publication-checks-20261116`.
That publication append changed only status and those logs; no native, reference or
complete-workflow acceptance transfers from the CLI verification.

Clean committed `cargo xtask verify` passed at
`8f1c64916d21f562f489ecd4555011b9aaa46bf4` in 202.8 seconds. All 87 source
inputs, 50 reports and five qualifying metadata files stayed unchanged, and the
tracked checkout remained clean. Session **49585** / Cargo PID **48516** is
terminal, exit **0**. Selected original/locator-normalized logs are retained under
`offset-native-stops-publication-checks-20261116/clean-8f1c649-verify-001`.
This final append changes only status and those logs; source gates were not
repeated and no native, reference or complete-workflow credit transfers.

Next: continue the separately bound pointer-input diagnosis and full native
rectangle-offset workflow, then independently assess its required saved files
and exports. The reviewed controller evidence remains a bounded draft update
to [PR #51](https://github.com/thematheusmello/weftforge/pull/51). Prior closed
cases remain unchanged. GitHub Actions stays disabled and the
existing static Pages site remains unchanged. Full editable allowances,
native offset acceptance, curved/concave/cancellation cases and the complete
ROADMAP target remain due.

## Earlier offset-publication context (retained)

The latest public main is `1fc1d71993cf193801e44982b757fca8abc5aba7`:
[PR #49](https://github.com/thematheusmello/weftforge/pull/49) merged the hole,
Cut & Sew and SVG canvas work. Actions remains disabled; the existing Pages
configuration and recent run IDs were unchanged in the last remote check.

Active offset integration is on `feat/cancellable-triangulation`, runtime
`4f80d064a60cbdba1358df9074beb74b803e1ee8`, based on the frozen `89e6d53`
offset checkpoint (runtime `d8cef84`). It combines the editable
material-outline preview/Apply operation with a locally licensed triangulator
that supports fallible internal callbacks, shared geometry work accounting,
sewing and Cloth initialization control, and shared offset-transaction budgets.
Independent source review prompted fixes for sewing-length inversion, split/join
certificates and bounds calculations in transaction siblings. The pending-refinement guard is integrated and independent source review found
no remaining blocker. Final formatting, **385 workspace Rust tests**, strict
all-target Clippy, native/CLI debug builds and **30 vendored triangulator tests**
passed with all **87 source inputs unchanged**. These are source checks; native
responsiveness and complete offset workflow acceptance remain unverified.

The original offset source passed formatting, **363 workspace Rust tests**,
strict all-target Clippy and native/CLI debug builds with 79 inputs unchanged:
receipt `6257e853ec5404dd7d4cd6446beb1bf3648345c09f02fcac62903a5809eae66a`.
Clean committed verification/build at `89e6d53` also passed, receipt
`160ac382f1590dc05214f5e7540fdc4a6734cef0a1a174b35813bc166d4f78d9`.
Those results identify the frozen source and do not validate this later runtime.
The current source map/progress cycle passed with 39 rows, 23 surfaces,
23 contracts, **363 anchors**, 34 Rust commands, 33 schemas and nine tools.
All 39 registry tests passed (session 19049, terminal exit 0; receipt
`c42aaf4b693a1faef796d84bb950937f4506d55ed3f7b5fc3fc3933e6dc71c25`).
Two separately attributable reports retain current source checks and the
closed native stops; the older offset source report is stale with its original
bytes archived. Progress remains
**41 partial / 15 planned / 0 of 56 reference contracts verified**, with no
roadmap gate accepted.

Two closed native attempts on that frozen source remain visible. OFFSET-R1-001
stopped at controller setup because a bare executable was not an app selector.
OFFSET-R1-002 selected its own app bundle but stopped before rectangle authoring:
an empty document advanced from revision 0 to 2 and New Empty refused unsaved
changes. No geometry, saves, exports or acceptance were obtained. A separate
instrumented diagnostic did not reproduce the problem: its empty document stayed
clean and New succeeded. Its missed scheduled observation times are retained.
The cause remains unexplained; no production fix or repaired result is claimed.
Three ordinary headless idle/focus/precision regressions are retained; all
temporary tracing has been removed. Finished test windows are closed.

Final source receipt: `4843ca67cac78b1bd2e1d091631ef18671e8a2847c2f2da3fa2dca44185d490a`;
public logs and independent review are selected under
`parity/reports/artifacts/material-outline-offset-work-control-20261116/`.
The native binary is `c05bac974a4f8b40e2c5c46e4f857d5e00c270410f3a817f83439dbbb1f5d71e`;
CLI is `a7f23bd8876a04456e0d668e11eabefe54f510757664d0180cc17e03c51e058c`.
Final sessions 11517/6268/70912/10226 are terminal exit 0; no Cargo job remains.
The earlier constructor-handoff compile failure and fixture-assumption failures
remain recorded separately. No binaries or target caches crossed worktrees.

Clean committed `cargo xtask verify` passed at `7a8df7e` in 154.7 seconds,
with all 87 source inputs, 45 reports and qualifying metadata unchanged.
Its selected logs are under the work-control artifact namespace; original
receipt: `e5819d8eb479eaf833da5e1c3fa426b3e2172aae6544ae5be99d2e41fd3663c3`.
Session 7611 is terminal exit 0; the native binary is unchanged.

OFFSET-R1-003 is now closed before authoring. New Empty succeeded cleanly,
but the planned name replacement remained “Pattern 1”; the width replacement
showed 0.40. No rectangle, material edit, offset, saves, exports or grading
followed. The controller/input cause remains unknown; this establishes no
product defect or geometry result. Its selected packet and current blocked
report are retained under `material-outline-offset-input-003-20261116`.
PAT-009 verification is blocked by that entry route; overall verification states
are **48 not run / 5 failed / 3 blocked**, with **41 partial / 15 planned** and
**0/56 reference contracts verified**. All 56 full scopes remain unchanged.
No test window is open; owned PID 34836 was independently confirmed absent.

[Draft PR #50](https://github.com/thematheusmello/weftforge/pull/50) publishes
source/checkpoint `2483c24`; Actions remains disabled, zero new runs were
created, and Pages is unchanged. This follow-up adds the closed003 packet and
blocked status while keeping all 45 prior reports and source87 exact. Its
progress/check/39-registry-test cycle passed (session 50454, exit 0; receipt
`61d79aae5dad4325269b73bc2ec94a728917a9f64ac567cb050d7d556bc1a9ce`).

Clean committed `cargo xtask verify` passed at `3f16230368bfed61ebd0bb3228f85bcf09cf40bc`
in 166.4 seconds, with all 87 source inputs, 46 reports and five qualifying
metadata files unchanged. Session 35392 is terminal exit 0; original receipt
`0b77063021dd9940d58e347688fc3994ae8a291119727e3482381ae4a6113a32`.
Selected logs are in `offset-input-publication-checks-20261116`.
This append changes only status and those check artifacts. It supplies no new
native, reference, physics or complete-workflow acceptance.

Next: publish this evidence follow-up to draft PR #50, then integrate the
separately checked UI polish. Calibrate name entry before binding another full
offset workflow.

A further PhotoCraft-inspired inspector/Scene polish is isolated on
`feat/photocraft-workspace-polish`, runtime `b7767bc94c89196f93b4ddbd1e06245f569936ff`.
Formatting, all **389 workspace tests**, strict workspace all-target Clippy and
the native build passed. An actual empty OS-window preview was captured. Its
name-entry calibration retained “Pattern 1”, so no rectangle or populated Scene
was accepted. That closed P001 result has unknown input/controller cause; owned
PID 85382 was confirmed absent. A separately bound N002 keyboard focus/paste
probe is diagnostic only. The UI candidate does not alter this frozen offset
source or grant any complete-workflow acceptance.
Closed native cases are never replayed or regraded. Curved/concave native cases,
full editable allowances and the rest of the roadmap remain due.

## Merged hole and Cut & Sew checkpoint

Current integrated runtime source is `947bfbe4dcb74fb253d584709376ace5a08db838`.
PR #49 contains the hole, Cut & Sew and SVG canvas changes. The tracked evidence
checkpoint is `7abfa075b4f6685c43618a7b0b881fd3dcf7ef5f`; final local integration
checks passed from that clean checkout on `feat/holes-sewn-cuts-publication`.
Registry progress remains **40 partial / 16 planned**,
**0/56 coarse reference contracts verified**, discovery incomplete and **no
roadmap gate accepted**. All 56 feature statuses/contract verification and all
39 earlier report bytes/results/artifact hashes stay exact. Actions remains
disabled and the existing static Pages configuration is preserved.

## Three separately attributable bounded reports

- [Historical straight hole](parity/reports/WF-20261116-STRAIGHT-HOLE-001.json):
  runtime `a021a2581c8e1dd000e535a417dcd964c3835884`, source73, metadata521,
  native4b56. Accepted H1-002 covers actual empty creation, review/refusal/cancel,
  one Apply, point edit/history and five saves. Its sole saved grade7c939 and
  separate root UI b6d79 remain bounded and stale on changed947. H1-001 remains
  FAILED/unaccepted at `expected_binary` preflight, before geometry assessment.
  H2-001 remains FAILED/unaccepted at sole grade
  `5457c733571b632dcc8cbde58a878d86035b6701db4284063ba8d19f9fe7db1d`:
  old SVG canvas1000x1000mm instead of240x360mm. Path scale was correct; old
  seed/control-hull bounds added blank padding, not observed clipping. Neither
  failed case was repaired/regraded, and no per-export H2 formal pass is claimed.
- [Current hole export](parity/reports/WF-20261116-HOLE-EXPORT-001.json):
  runtime947/source74/metadataa362/nativef743, H3-001 only. Actual accepted
  historical H1-002 R5 is the explicit strict-v5 compatibility input; it supplies
  no current947 empty-creation authority. Sole grade44c414 passes own R6/full-v5
  bits and the unchanged56ab export consumer: OBJ.0768m2/one component/two loops/
  Euler0/metre positions+UV/Y-up; compound-evenodd SVG240x360mm with
  `viewBox="0 -360 240 360"`; planned DXF refusal preserves both sentinels.
  Separate root UI aa8491/root0a3f and all34-image review6988 support only that
  bounded native route. Original postguarde135 STILL FAILED/exited1 with29
  permission string/integer false rows; failed invocation88ad/code/report stay
  unchanged. Separate e3cf direct277-pin/435-raw hash/length/permbit audit and UI
  review qualify unchanged boundary without turning the original guard green.
  Actor435/canonical363 manifests share361 and retain separate receipts; the
  canonical set contains two later root files.79 pairs+0 docs+1 immediate summary
  =80/101, conservative upper804.867825666s/1200. History corroboration is
  retrospective, not exact invocation UTC; cleanup remains separate.
- [Historical Cut & Sew](parity/reports/WF-20261116-CUT-AND-SEW-001.json):
  sealed runtime5898/source74/metadata385/native7809/freeze5ab. Accepted S1 covers
  default off/on-off-on, raw/caret/refusal/cancel/shields, one Apply/history/save;
  accepted S2 uses only actual S1R4, reopens, removes only seam20, edits child15
  height.40m and preserves retained2/anchors11+13 independently. S1 sole
  grade174cb/UI37a/rootd741 and S2 grade2782/UI3673/rootefd756 remain separately
  bounded, stale on changed947. S1:59 images,132 pairs+1 doc+1 summary=134/161,
  upper1070.583932333s/1800. S2:25 images,57+1+1=59/76,
  upper602.617138334s/900. No current947 native Sew authority transfers.

All three are implementation-only passes of declared subsets; coarse PAT-006,
SEW-001 and FILE-005 remain partial/not_run. Saved area*density is analytic only.
No actual native Cloth/stitch/particle mass, simulation/drape/calibration,
reference parity, complete feature, garment or roadmap acceptance is added.
Saved-grade native_ui_verified:false remains unchanged beside separate UI
acceptances. Every cleanup receipt stays outside frozen case accounting.

## Current source and publication gates

Shared SVG export now unions ALL existing analytic exterior bounds, including
true cubic extrema, preserving absolute U/-V, millimetres and path bytes. Only
empty export keeps the1m fallback. Regressions cover R240, all-negative/large-
offset/multipiece layouts, cubic extrema and empty fallback; the legacy500x700mm
expectation matches its literal geometry. No command/schema/tool/version/UI/
dependency or print-margin contract was added. Current source remains **74
immutable inputs**, **33 Rust alternatives / 32 schemas / 9 tools** and **38
source rows / 22 native surfaces / 22 contracts / 321 anchor occurrences**.
Historical949/205/56/8 maps and5898/385/521/982/ad7/B657 lineage stay exact.

Retained947 source gates passed fmt, **339 Rust tests**, strict all-target Clippy
and fresh debug builds: runtime receipt
`15d3225ef669621690f7aabfe47d481509d2b78a232c3b4d61779898e83b78bd`;
clean a362 xtask/debug receipt
`c50c3a39a1782250b1fdc201e1ce36716ff9817995e877efa244b743f0aea9aa`.
Native identity is
`f74350126898e07729f898cb641442baecacc29404f5620b1cfabb03926e282a`
(62,231,992B). Historical hole329 and Sew336 source gates retain their tested
build identities. Root's publication-worktree fresh debug job37501 is terminal
exit0 with source74 unchanged; receipt SHA-256
`a9b7dea7e659feafb56a32f74144c619be91277ef63fcd8ea4b952bca13ec648`.
No cross-worktree Cargo target was copied. That locally built executable is
separate from reviewed native f743 and receives no transferred UI acceptance.
No Cargo job, GUI lease or native preview window is active.

The indexed archives are Hole v3 **2,863 nodes** and Sew **1,195 nodes**. Hole
CURRENT003 supersedes exact historical CURRENT/002 only at this version. Whole
chooser/PID/bundle/session omissions, literal-path/result/history derivatives
and selected JPEG8-byte ICC identifier redactions retain original/public hashes,
all other encoded bytes and identical pixels. No whole-byte-original JPEG claim
is made. Both independent private privacy/byte reviews passed; Hole v3 is bound to
review df08c016. Root copied exactly2,863 Hole+1,195 Sew files/64,545,387B
without overwrites and rechecked every destination SHA/size/mode; copy receipt
`902d78b7fb77527130f1ae1157092e8ad56498105a84c7c645dca0e7b438f51f`.
All 4,058 approved archive files are tracked; every staged Git blob matches its
approved hash, length and mode. The initial tracking check found 15 reviewed
autosave/recovery files excluded by generic ignore rules. Those exact files were
explicitly added; the ignore rules and archive bytes were unchanged. The passing
blob receipt is `44d3d44af2136dc22aa74107d2a62e87dcf8c84eda233cfe29fe491ae08baad2`.
Old stage/case/source/packet bytes remain immutable. See the
[publication privacy scope](parity/reports/artifacts/holes-sewn-cuts-publication-20261116/PUBLICATION-PRIVACY-001.json).

The single metadata-only Python cycle passed: implementation-map check, progress
generation/check and **39/39 registry tests**. Preservation audit confirms all39
older report bytes/results/artifact hashes, five historical inventory files, all74
current source bytes, all56 feature authority records and prior run attribution
remain exact. Metadata-cycle receipt SHA-256 is
`33ad5319a6b229a3830068e9d906ebdf5ff8060d17cc67605269ab19a8ccd137`.

Root independently preserved all 39 older reports, eight historical map/check
files, 56 feature authority records and the exact 74 source inputs. The immutable
older Cut report and Git tree contain 1,262 artifacts; an earlier continuation
summary's prediction of 1,266 was incorrect. Every actual artifact remains exact.
Root preservation receipt:
`acbb48cc701148dc6a086ba3ad6db530b2dbda6e897d63689e37912a0f2ebe3e`.

Final clean-checkout `cargo xtask verify` and native/CLI debug builds passed at
7abfa07. Session 40358 is terminal, exit 0; receipt:
`e90c6a412b2fc04f924ec1d165c774887bbd87250c4fd18c6387e35aac7c6677`.
The source inputs and locally built executable hashes are unchanged. Formatting,
339 Rust tests and strict Clippy retain their exact-source 947 gate above;
no native case, evaluator or saved grade was replayed. This status-only update
does not change the tested source, registry, maps, generated docs or artifacts.

Next: publish this reviewed checkpoint through PR #49, preserve disabled Actions
and the existing Pages site, then implement PAT-009 material-outline offsets from
the independent reviewed contract. Persistent seam allowances and the complete
roadmap remain required. No Cargo job or GUI lease is active.

## Published bounded v4 cut qualification — historical source 19b1

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

The clean runtime checkpoint 19b1e88245dc627cfec8164f4924b335fe44853d passed
workspace formatting, **304 Rust tests / zero failures**, and strict
workspace/all-target Clippy. All 69 runtime/build/test/doc/contract inputs remained
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
behavior unobserved. Full cuts and reference comparison remain open; this v4 collection supplies no
hole/Cut & Sew acceptance, native concave or failed-I/O acceptance. Saved analytic area/density is not a native particle-
mass or physics acceptance. Source and case originals remain isolated. Finished
cut windows are closed and their jobs are terminal. Clean tracked checkpoint
4fb6c47 passed its [publication gate](parity/reports/artifacts/straight-cut-publication-20261116/clean-4fb6c47/PUBLICATION.json):
all 1, 262 evidence Git blobs match, source 69 is unchanged, and registry/map/Python
checks plus CLI help pass through `cargo xtask verify`. Public screenshot copies
remove only eight device-identifier metadata bytes each; decoded pixels, scan data
and color characterization remain exact. Original private captures stay untouched.
GitHub Actions remains disabled and the existing static Pages site is preserved.
The separate strict-v5 hole source passed329 Rust tests and local source gates;
its native/private qualification is outside this published v4 cut collection.
Current candidate and H1/H2 boundaries are stated above.
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
processes**. Root collection verified all 3, 100 original regular files, three
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

## Earlier continuation notes (historical)

Root independently audits the clean tracked publication candidate, mappings,
source 65, scopes and check receipts before remote publication. Continue the
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
