# Verification and acceptance

Implementation checks and reference comparisons are separate. No garment
workflow has qualifying reference comparison evidence. The reference workspace
and complete build are observed, along with a bounded original rectangle and
physical export measurement. Global settings and broader contracts remain
incomplete; the baseline is provisional. See the dedicated reference setup and
reviewed behavior documents.

## Reproducible commands

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 -m pip install -r requirements-dev.txt
cargo xtask progress
cargo xtask verify
cargo xtask scenario
cargo xtask benchmark
cargo run --release -p weftforge
cargo run --release -p weftforge-cli -- --help
cargo xtask package
```

`scenario` authors an original two-panel short-sleeve shirt using the shared
command engine, saves the editable project and exports OBJ and SVG. The
independent Python standard-library consumer checks declared physical bounds,
axes, UV indices, unit normals, piece groups and millimetre SVG dimensions/paths.
Core tests also use independent `tobj`. This does not prove material assignments,
animated export, editable SVG import or the empty-project desktop workflow.

The regression shirt starts 0.44 m apart in Z around an original capsule
mannequin. It has 152 vertices and 232 triangles at 0.12 m target resolution.
Its eight oriented seams produce 30 weighted stitch samples. Geometry is tested
at 0.12/0.07/0.035 m with physical area, conformity and positive incident mass.
The first integration exposed roundoff-zero ear-clipping faces; regression and
strict area checks retain that failure's scope. Subsequent simulation exposed
instantaneous seam closure instability at default quality. Interior edge flips and
explicit progressive sewing allow finite runs; tight-shirt strain and refinement
still fail the declared criteria. A separate relaxed shirt passes the explicit
60-step gravity-off assembly plus 120-step gravity-on fitting protocol at 8
substeps/32 iterations. See physics.md, STATUS.md and attributable run reports.
Elementary cloth tests do not prove garment fitting quality.

## Numerical evidence

See physics.md for equations, initial conditions, tolerances declared before
judging results, analytic freefall, area mass, material behavior, refinement,
weighted seam momentum, dihedral gradients and bounded contact fixtures. Unit
fixtures test original independently justified geometry, not generated reference
answers. Maintain failure thresholds; do not disable contact or reduce scene
coverage to pass a benchmark.

The bounded sewn-curve split checks compare original cubic evaluation and
original/partitioned sewn physical point pairs at a predeclared 1e-12 m tolerance
(and 1e-12 for normalized parameter comparisons). They exercise either seam side,
both interval orientations, gathered piecewise-linear mappings, partial ranges,
splits on correspondence breakpoints and interval endpoints, several seams, and
the explicit same-curve-both-sides validation exclusion. Remeshing at 0.09/0.035 m
must retain the authored maps and positive triangles. History branching, failed
transaction rollback and actual v2 save/reopen are tested. Native inspector state
and real CLI subprocess checks are implementation evidence; no new native
interaction capture or reference comparison is implied. The identified native
acceptance steps and remaining limits are in
[PAT-005](../spec/features/PAT-005-sewn-curve-editing.md).

## Desktop acceptance scope

Start empty. Author shirt boundaries through actual UI actions, sew, arrange,
simulate, edit a dimension, save, close, reopen, edit again, and export; parse
output independently. Record executable/source hash, viewport logical/physical
size, DPI, camera, lighting, scene state and action log. A screenshot of an opened
CLI-created sample is only a viewport observation and cannot substitute for this
workflow. Use original redistributable assets only.

Additional release acceptance remains open: curved reversed gathered seams with
different tessellations; relationship-preserving splitting/refinement/history;
connected 2D/3D sleeve edit; measured interacting shirt+jacket/ground/avatar;
contrasting fabric mesh/timestep refinement; animated cache invalidation;
interrupted save, missing assets and malformed/unsupported inputs; independent
all-advertised export attributes; and representative responsiveness/performance.

### Shutdown and recovery boundary

The preliminary curve-candidate probe found that macOS Cmd+Q quit a dirty
untitled panel without the window-close warning; see
[issue #18](https://github.com/thematheusmello/weftforge/issues/18). The retained
`1ed6d18d…` executable did not provide last-second shutdown retention. The pinned
winit 0.30.13 application-termination path emits `LoopExiting`, while window close
emits `CloseRequested`. eframe 0.35.0 calls `App::on_exit` on application
termination; its exposed viewport `CancelClose` cannot cancel that global Quit.
The window-close guard must not be described as a universal unsaved-quit prompt.

The follow-up uses `on_exit` to commit valid pending numeric drafts, finish any
older autosave writer, then synchronously retain the latest dirty authoritative
document. Saved projects use a `.autosave` sidecar; untitled projects use the
operating-system recovery directory. If that destination fails, one fresh
`Exit-…weftforge.autosave` path in the recovery directory is attempted. Recovery
requires explicit opening of a copy; the primary is never saved by shutdown.
Invalid drafts retain the last valid construction. Clean exit and explicit
Discard do not write a new latest-state snapshot; older autosaves remain.
Exhausted recovery destinations produce a native error and stderr report, but
global Quit is already in progress and cannot be canceled. Forced termination
and storage failure are not covered by a successful-shutdown guarantee.

Focused implementation checks load retained files and recover them exactly after
last-second curve/grain edits, a pending older writer and valid drafts. They also
check primary-byte preservation, explicit discard, clean exit, unique fallback,
invalid drafts and all-destination failure. Actual macOS Cmd+Q/relaunch/recovery
now passes the bounded focused-draft scenario on the identified `4fbd66af…77c9d`
candidate. The accepted primary remains unchanged; a valid still-focused grain
draft is retained, explicitly recovered as a copy and edited again. This does
not execute every fallback/storage-failure path through the native UI.

## Host and performance scope

Initial host: Apple M3 Max, 16 CPU cores, 40 GPU cores, 48 GiB RAM, macOS 27.2
26B123 arm64; Rust 1.94.1. Report solver settings and mesh size with each result.
`benchmark` reports actual CPU step times and simulated seconds per wall second.
It explicitly leaves memory and viewport timing null until separately measured.
A responsive viewport does not establish real-time simulation.

Public CI is configured to test Linux/macOS/Windows source builds on hosted runners; build success
alone does not prove GUI launch, platform dialogs, notarization or hardware
behavior. Licensed reference runs are local/trusted, never fork CI jobs.

## Witnessed native increment (macOS, 2026-11-13)

The actual unsigned eframe/wgpu executable was launched from an empty project.
CUA canvas actions authored two 14-point short-sleeve panels; eight same-direction
interval seams were created through the sewing tool and pieces arranged at
Z±0.22m. The project was saved, closed and reopened through native dialogs.
Resolution was changed from0.07m to0.12m (511→158vertices), followed by explicit
60-step gravity-off assembly and120-step drape. Both panel widths were changed
from1.06m to1.08m after reopening; one Undo restored1.06m and Redo restored1.08m.
The edited construction was saved and resimulated with the same declared stages.
OBJ and SVG exports came from the actual UI. No prebuilt garment replaced those
authoring actions. See fixtures/ui-authored-shirt.weftforge, ui-shirt-drape.obj,
ui-shirt-patterns.svg and the run report for exact hashes and source revision.

An independent Python OBJ/authoring evaluator measured the edited output:
158vertices,244triangles, maximum material-axis strain0.08541719869 (<0.15),
seam gap3.06875e-7m (<0.005m), and avatar/ground thickness error~1.64e-16m
(<0.002m). It checks explicit straight seams, UV-space coordinates, physical
metre bounds and unit normals; it does not verify self-contact, animation,
curved-seam behavior or reference equivalence. The final displayed clock was3s.

Default camera: yaw−0.45rad, pitch0.12rad, scale270logicalpoints/m; orthographic
projection; directional flat triangle shading; no texture or PBR. Screenshot and
selected-window provenance are in docs/images/. This is implementation-only
acceptance for this original scene. Full discovery, all reference comparisons,
professional quality, interacting clothing, animation and other platforms remain
open. The scalar strain threshold is a scene acceptance criterion, not material
calibration or a universal garment-quality guarantee.

Input injection needed an observed UI state between text entry and Tab/commit;
unflushed rapid automation could send text to the next field. Preview contours
were checked before committing; rejected invalid drafts did not change the
project. Native AX SetValue is not currently wired for all egui numeric/text
controls; ordinary keyboard and pointer paths were used and observed.

Attribution limit: initial canvas creation was observed in a local build before
the bounded Assembly/Drape buttons were added; its executable digest was not
retained. The final reopen/edit/simulation/export build is hashed and matches
the committed source candidate. A fresh full empty-project replay on a single
identified release candidate remains required before accepting the full gate.

## Complete replay on one identified executable

A separate fresh run closes the earlier composite-build attribution gap.
`WF-20261113-SINGLE-CANDIDATE-001` starts with an empty project in the retained
`f00e0775…45111b` executable; all23 Rust/workspace/lock/toolchain files were
independently compared with source `ff3914474c8acc30d10c70a8489cae56cfa2c872`.
It authored both14-point panels in the canvas, arranged and sewed8 boundaries,
simulated, adjusted widths1.06→1.08m, saved, quit/relaunched/reopened, observed
persisted1.08m, edited to1.10m (Undo1.08/Redo1.10), saved and resimulated. Both
simulation stages and native OBJ/SVG export use the same identified executable.
Exact original canvas points and settings are in the candidate scenario.

The final independently parsed result is158vertices/244triangles, material-axis
strain0.08071752985 (<0.15), seam gap2.78859e-8m (<0.005m), avatar/ground thickness
error~1.50e-16m (<0.002m), and authored SVG paths at1mm per user unit. The native
capture/provenance are `docs/images/studio-single-candidate.*`; final construction
and actual UI exports are `fixtures/single-candidate-*`. The prior1.08m save is
retained in the run artifacts. Original tight-shirt failures are untouched.

This witnesses the first bounded garment integration workflow. Reference parity,
complete professional authoring, animation and all remaining acceptance workflows
are still open. Windows/Linux source builds/tests/lint passed hostedCI, but GUI
launch/operation there is unverified. See mergedPR10 and its two green CI runs;
the initial evidence-job setup failure remains linked in STATUS.md.

## Authored grain candidate

The identified native `03d44055…b895` executable exercised the retained v1 shirt
through grain edit/Undo/Redo, separate shared stiffness controls, recoloring,
v2 save/quit/relaunch/reopen, resimulation and OBJ/SVG export. All26 Rust/workspace/
lock/toolchain files byte-match public source `28cae66b684e368e7fbe6b3240069ae0eb6e16e6`;
the executable was retained from the earlier b8c9a1a build, not rebuilt at the
later publication commit. The original v1 fixture remains unchanged.

[Authoring/persistence evidence](../parity/reports/WF-20261113-GRAIN-AUTHORING-001.json)
passes only that declared subset. Before/after saved data differ only in revision
and color; front grainπ/2, back0 and shared warp50/weft1000N/m persist.
[The same replay's fitting evidence](../parity/reports/WF-20261113-GRAIN-FIT-001.json)
**fails** the unchanged0.15 strain screen: independent strain0.26930346. Seam gap
5.55e-6m and avatar/ground error0 pass their0.005m/0.002m limits. OBJ attributes,
metre/Y-up bounds, authored SVG points and physical millimetre scale pass.

This changed material is uncalibrated; a fitting-screen failure does not by itself
establish an incorrect constitutive model. No physical inputs or limits were
changed to obtain a pass. See [issue16](https://github.com/thematheusmello/weftforge/issues/16).
The actual own-app screenshot and redacted capture/source provenance are
[studio-grain.png](images/studio-grain.png) and [studio-grain.json](images/studio-grain.json).
The original empty-project workflow remains historical stale evidence; this new
run began by loading that authored shirt. Neither is a reference comparison.

The independent headless repeat returned exit1 and the identical0.2693034615
strain (agreement within1e-12). It simulated3s in22.19s wall, about0.135 simulated
s/wall s on the documented host; memory and viewport frame time are unmeasured.
The retained CLI JSON includes per-step timing/settings and finite diagnostics.


## Sewn-curve and shutdown replay

[WF-20261114-SEWN-CURVE-001](../parity/reports/WF-20261114-SEWN-CURVE-001.json)
records the declared construction/recovery subset on native executable
`4fbd66af…77c9d`, built from `eb153cd`. Its scoped runtime source is byte-identical
to published main `68bdb9f`; no rebuild at the later publication commit is claimed.

From an empty native document, two original 0.4 × 0.6 m panels were created,
one top edge made cubic and paired to the other with reversed sewing. Splitting
at t = 0.4, one Undo/Redo, rejected endpoint/nonfinite edits, 0.09 → 0.035 m
refinement, save/quit/relaunch/reopen and actual OBJ/SVG export were exercised.
Independent Bernstein bounds cover the entire authored curve and every sewn
point pair at 1e-16 m against the predeclared 1e-12 m threshold. Both failed edits
leave identical saved bytes and revision. Exported meshes increase from
207 vertices / 343 triangles to 2330 / 4516, with finite coordinates, valid UVs,
positive faces and aligned unit normals. SVG retains both cubic segments.

A separate focused grain draft remained uncommitted at global Quit: primary
revision 14/grain 0 stayed byte-identical, while the new recovery contained
revision 15/grain 0.25. After relaunch, explicit Recover copy and a new save
preserved those recovery bytes. Editing recovered curve 9 P1 X from 0.36 to
0.37 changed only that coordinate and revision 16, demonstrating retained editing.

The [original fixtures](../fixtures/sewn-curve),
[independent evaluator](../parity/reports/artifacts/sewn-curve-20261114/verify_curve_outputs.py),
[exact rerun arguments](../parity/reports/artifacts/sewn-curve-20261114/rerun-arguments.json)
and [result](../parity/reports/artifacts/sewn-curve-20261114/independent-final-check.json)
are retained. Negative witnesses reject a 0.001 m control change, wrong seam
orientation and unsupported SVG syntax. Static REA before/final captures are
untruncated; intervening interactions/history/errors are parent native CUA
observations, not a REA action recording or video.

This is a construction and recovery pass, not a complete garment/fitting replay,
a full feature contract or reference parity. No cloth simulation ran. The
separately reproduced creation edit-buffer defect
[issue22](https://github.com/thematheusmello/weftforge/issues/22) remains visible;
its correction was accepted in PR27 and that scoped evidence is now historical
after later source changes. Historical garment fitting
failures and stale complete-workflow reports remain unchanged.


## Bounded native join replay

[WF-20261114-NATIVE-JOIN-001](../parity/reports/WF-20261114-NATIVE-JOIN-001.json)
records source `26dd585`, desktop `9c8b0936` and CLI `ca49402e`. Starting empty,
two original panels were created, one boundary made cubic and sewn with reversed
pairing. Split and Join retained the original curve identity and both sewing
records. Focus/Tab, one Undo/Redo, refinement, actual OBJ/SVG export, process
restart/reopen and rejected corner geometry were checked.

Independent exact rational control/interval checks bound the authored parent
error by 2.119e-16 m and reconstructed-child/sewn-locus error by 8.695e-17 m against
the preregistered 1e-12 m limit. Focus, Tab and corner rejection preserve exact
saved bytes. Meshes increase from 194 vertices/325 positive triangles to 2296/4448;
both SVG exports retain two closed piece paths and one editable cubic.

The official Python MCP SDK 1.28.1 negotiated legacy 2025-11-25, advertised the
join command and produced bytes identical to the native joined document. A
rejected corner command preserved its output sentinel and all native inputs.
This is five-tool local stdio evidence, not a modern-client or active-GUI claim.

The [portable bundle](../parity/reports/artifacts/curve-join-20261114/native/README.md)
contains exact native inputs, frozen evaluators, meaningful negative witnesses
and a gap-free read-only REA corner-error capture. Parent observations retain
AX focus/pagination and temporary full-disk input-driver qualifications. No
reference parity, cloth fitting, large-placement native case or coalesced-event
native case is claimed. Earlier source-pinned evidence remains historical/stale.

## Bounded native seam reversal

[WF-20261114-NATIVE-REVERSAL-001](../parity/reports/WF-20261114-NATIVE-REVERSAL-001.json)
records desktop source `9d30a71`, executable `afcb154d` and unchanged CLI
`ca49402e`. The declared input derives from the original GUI-authored two-panel
project, adding an explicit nonlinear seam map. With neither piece selected,
Reverse on seam 23 commits a focused resolution edit and the B-only interval
swap together. One Undo, Tab, Redo and a second Reverse preserve all expected
document fields, stable IDs and allocator 24. Actual process restart/reopen and
revision-limit rejection preserve exact saved bytes.

The frozen independent evaluator checks all nine document snapshots and both
nonlinear map intervals. Actual OBJ export contains 1,159 vertices and 2,192
positive triangles with valid UVs/normals and metre scale. SVG retains the exact
original geometry and bytes: two closed piece paths and one editable cubic.
The official Python MCP SDK 1.28.1/legacy 2025-11-25 produces bytes identical to
the native two-command transaction and preserves a rejected-output sentinel.
The CLI binary is unchanged; this is an additional executed case.

Passive REA captures retain the before/after guides and visible revision-limit
error. The fixed 2D viewport differs only at 371 pixels in the guide region;
this is an observed raster difference, not a reference comparison. The after
capture exposes only four AX nodes despite reporting no truncation or gaps;
its screenshot and parent CUA observations supply row semantics. Focus and
event order require the parent observations. The source-pinned implementation
contract remains its pre-acceptance declaration; this report records the
subsequent bounded desktop result.

This is implementation-only acceptance. Simulation, physical easing/twist,
continuous arc-length pairing, broad native editing and full reference parity
remain unverified. GitHub Actions and Pages updates are disabled during
development at the user's request; checks continue locally.
