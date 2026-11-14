# WeftForge status — active development, 2026-11-14

The complete independent native garment studio and recorded one-to-one reference
parity remain the goal. No feature has verified reference parity: 0/56 inventoried
contracts, with discovery incomplete and the full-product denominator unknown.
All six roadmap gates remain open beyond the bounded implementation evidence.

## Recoverable implementation and publication

Verified public repository/origin: https://github.com/thematheusmello/weftforge.
Published main: `28cae66b684e368e7fbe6b3240069ae0eb6e16e6` ([PR15](https://github.com/thematheusmello/weftforge/pull/15)).
The preceding main2837647 Linux/macOS/Windows source checks and evidence validation passed in
[CI43526530172](https://github.com/thematheusmello/weftforge/actions/runs/43526530172).
[Pages43526530260](https://github.com/thematheusmello/weftforge/actions/runs/43526530260)
succeeded; https://thematheusmello.github.io/weftforge/ returned HTTP200 with domains,
freshness and attributable runs. Earlier CI/component and Pages/artifact-visibility
failures remain in their original runs; bounded fixes are integrated. Project
board creation lacks project/read:project token scopes; issues and registry work.

The public [v0.1.0-alpha.1 source preview](https://github.com/thematheusmello/weftforge/releases/tag/v0.1.0-alpha.1)
targets main8419363, with byte/SHA-verified original editable shirt and OBJ/SVG
assets. No compiled binaries/installers ship. Its inspector data-preservation bug
is disclosed in the release notes and [issue13](https://github.com/thematheusmello/weftforge/issues/13).

The grain, strict v2 migration and inspector fix are merged through PR15. All
eight push/PR Linux/macOS/Windows/evidence checks passed on its source-identical
head9738099. Main [CI43529986279](https://github.com/thematheusmello/weftforge/actions/runs/43529986279)
is still running at this checkpoint; [Pages43529986285](https://github.com/thematheusmello/weftforge/actions/runs/43529986285)
succeeded. Reference-access PR14 also completed all checks and merged.

Current root branch `docs/grain-evidence` retains the new native editing/persistence
pass and the same replay's fitting failure in separate scoped reports. Every
report points to public source28cae66, whose26 Rust/workspace/lock/toolchain files
byte-match the actual b8c9a1a build source. The executable was retained; no rebuild
at the later publication commit is claimed. Historical reports/fixtures remain
unchanged, including their stale flags and original outcomes.

The curve increment is recoverable as `c48cd958101c5602a09824c5836d4874c4c6b0d5`
in `feat/sewn-curve-editing`:49 Rust/24 Python checks,fmt,clippy and release builds
passed. Native curve acceptance is next. The local MCP adapter is active in its
separate worktree with explicit module/shared-export ownership; root is avoiding
concurrent shared-target Cargo work. Completed disposable debug caches were
removed after disk exhaustion; retained f00e,03d440 and1ed6d1 binaries were rehashed
unchanged. No source build/benchmark/research process remains live on root.

## What works and what was checked

The GUI-independent Rust core retains stable authored points, straight/cubic
curves, pieces, materials and oriented curve-interval seams with explicit
correspondence. Typed transactions provide undo/redo and revision invalidation.
Conforming refinement preserves seam mapping; interior flips improve mesh quality.
Sewn-curve splitting still explicitly rejects pending relationship rewriting.
Cubic intersection checking has a documented tessellation tolerance.

The native eframe/wgpu editor connects 2D/3D selection, rectangle/polygon creation,
interval sewing, dimensions, placement and materials to the real engine. It has
command search, revision-tagged cancellable workers, save/open/export, untitled
and named autosave and explicit recovery. Initial contact construction only has
before/after cancellation; file IO and validation retain synchronous portions.
The projected triangle preview still lacks professional rendering/depth/PBR.

The f64 CPU XPBD solver has orthotropic membrane strain, dihedral bending, area
mass, damping/friction, weighted seams with bounded-speed assembly, analytic
avatar/ground and discrete vertex/triangle self/layer contact. No full edge/edge
CCD, GPU solver or measured fabric calibration. Contact search is quadratic.

Native v2 writes per-piece `grain_angle` (radians, counterclockwise +U→+V) and
separate positive warp/weft stiffnesses in N/m. Strict v1 reading copies old stretch
to both axes, supplies zero grain and preserves every other authored value/ID and
revision. Existing fixtures remain unchanged. Atomic saves retain exact prior
valid bytes in recovery; invalid input/unsupported versions retain the old project.
Loading is bounded to16MiB. v2 files require the updated app. OBJ preserves static
geometry, metre/Y-up scale, UVs, normals and piece groups; SVG preserves authored
curves at physical millimetre scale. No MTL appearance or animation export yet.

All44 Rust tests and24 Python checks passed on the grain source, including strict
migration/duplicate rejection, history/rollback, CLI edits, grain-aware independent
metrics and the real-egui inspector clamping regression. Formatting, strict
all-target clippy, xtask verification and native/CLI release builds passed. The
CPU bridge patch test rotates warp50/weft1000N/m through a quarter-turn: top height
0.2029735316→0.2023383529m (0.635mm more compression, above the predeclared0.3mm
criterion). Recoloring yields bit-identical solved positions. This is an original
implementation test, not reference parity or calibration.

## Actual native workflow evidence

Historical `f00e` executable (source `ff3914474c8acc30d10c70a8489cae56cfa2c872`)
completed the full empty-project shirt workflow: authored two14-point panels,
sewed8 edges, arranged, assembled/draped, edited, saved, quit/reopened, edited
again with Undo/Redo, resimulated and exported. Final158vertices/244triangles,
3s simulation, independent strain0.08071753, seam2.79e-8m and avatar/ground error
~1.50e-16m passed that scene's declared limits. The earlier composite replay and
all original failures remain retained. These reports are now stale for the
changed runtime; they have not been rewritten as new executions.

New grain native executable SHA-256:
`03d44055eacfa20ecd6c69a051e259707848560ae0bf5db5d61282455c04b895`.
CLI SHA-256: `e84b4303caa8208be94e6aef1628efb14eaba30c85585978e23d7d6c86c7f403`.
Root opened the unchanged v1 shirt, observed both default stiffnesses400, set front
grainπ/2 and verified Undo0/Redoπ/2, set shared warp50/weft1000, recolored, saved a
new v2 file, quit/relaunched the same executable and reopened. Frontπ/2/back0 and
shared stiffness persisted. Independent JSON comparison confirms color/revision
were the only changes between the saved before/after color snapshots.

The native assembly60steps with gravity0 followed by drape120steps under gravity
−9.81, dt1/60s,8substeps/32iterations produced actual OBJ/SVG. **Fitting failed**:
independent axis strain0.26930346 exceeds the unchanged0.15 limit. Seam gap5.55e-6m
passes0.005m; avatar/ground thickness error0 passes0.002m. SVG authored paths and
millimetre scale pass. Retain this anisotropic garment failure. Successful editing
and persistence do not establish fitting quality. Independent self-contact and
animation checks remain absent. New scoped reports are `WF-20261113-GRAIN-AUTHORING-001` (passed subset) and
`WF-20261113-GRAIN-FIT-001` (failed fit), with original screenshot/provenance under
`docs/images/studio-grain.*`. A headless repeat exits1 with identical axis strain
within1e-12;3 simulated seconds took22.19s wall (~0.135 simulated s/wall s).
The fitting discrepancy is [issue16](https://github.com/thematheusmello/weftforge/issues/16).

## Reference laboratory

Official Personal2026.1.237 installer and executable identities are retained in
`parity/reference.json`; listing build2026.1.237.58920 is not yet confirmed in About.
Latest REA6.9.0 read-only capture at23:06UTC shows **CLO-SET sign-in**, with no Terms
modal or garment workspace in the selected main window. Current terms acceptance
and account entitlement remain unknown. The account holder must sign in/activate
a legitimate trial/license and review any terms presented. No investigator terms
acceptance or credential entry occurred. No garment reference scenario executed.
Baseline remains provisional: build/preferences/units/locale/graphics are unfrozen.

Raw installer, binaries, account UI and complete research evidence remain private
outside Git; no personal paths are published. No proprietary code analysis was
used. REA CLI6.9.0 works with Node26.12.1; current-session MCP is unconnected and
Ghidra unconfigured/not invoked. No installer/setup/research job is running.

## Failures, evidence integrity and next steps

The original tight shirt and refinement failures remain unchanged ([issue8](https://github.com/thematheusmello/weftforge/issues/8)).
Its0.96m body circumference is below the1.131m torso; finer meshes alone did not
establish convergence. The separate relaxed garment passed only its explicit
assembly/drape protocol and is slower than real time. Partial-sewing curve-t
versus documented arc-length semantics remain unresolved ([issue9](https://github.com/thematheusmello/weftforge/issues/9)).

Schema2 scopes freshness per run. Every original ID, tested commit, source/input/
artifact hash and outcome is preserved; changed relevant runtime/checkers mark
old evidence stale. Current failures cannot be hidden by another pass or removing
run references. Before the first actual reference pass can qualify, add baseline
revision/settings and reviewed contract/scenario/criteria digest qualification.

1. Publish the reviewed grain native reports and retained anisotropic failure,
   after registry/generated/artifact checks. Preserve the source checkpoint, all
   old artifacts/outcomes and accurate per-scope freshness.
2. Exercise the retained1ed6d1 curve executable through numeric control editing,
   sewn split, history, refinement and save/reopen; independently check geometry
   and sewing. Integrate its tested source and review grain-report staleness.
   Then integrate the local MCP adapter/shared export guard with protocol tests.
3. Improve sewing semantics and garment/contact convergence, then measure layered
   shirt+jacket, contrasting fabrics/refinement and real responsiveness. Continue
   professional construction, avatar imports, animation/caches, PBR, interchange,
   printing, output preparation, automation and all distribution gates.
4. When reference access is available, freeze settings and execute the controlled
   original scenarios. Implementation-only checks never count as reference parity.

Last observed own-app PID52560 is paused at3s Drape with no active-job indicator.
Historical reference PID18283 showed sign-in at23:06UTC. Grain and curve build/check sessions
completed; the MCP agent is active in its isolated worktree. Recheck actual
processes and CI handles before resuming; do not duplicate live work.

## Integrated curve/MCP/recovery candidate

Root branch `feat/curve-mcp-recovery` is integrating reviewed c48cd958 curve
controls and sewing-preserving subdivision with the tested local MCP adapter and
shared export guards. The native curve probe created a disposable editable panel;
its complete workflow was not accepted. Actual Cmd+Q exited while Unsaved was
visible ([issue18](https://github.com/thematheusmello/weftforge/issues/18)). The
pinned platform lifecycle bypasses window CloseRequested; latest-dirty shutdown
recovery is being implemented/tested separately, with the native Quit limitation
explicit. No unrelated user document was involved.

New grain reports retain their original outcomes, artifacts and identities but
are stale after these relevant source changes. Run the full native curve/history/
refinement/save/reopen and last-second quit/recovery scenarios on the combined
identified candidate after all source checks pass. Then publish qualified new
reports. Do not label the aborted preliminary probe a complete workflow.

Shutdown retention fix3014a6e is integrated locally: App::on_exit commits valid
drafts, drains an older autosave writer, then synchronously retains the current
dirty construction as explicit recovery without saving primary. GlobalmacOSQuit
remains uncancellable through the pinned APIs. Focused lifecycle tests passed;
combined native last-second Quit/relaunch/recovery acceptance is still pending.
The same native saver now refuses existing nonregular output and recovery
destinations; issue19 includes the reproduced dangling-link defect.

At00:15UTC the account holder replied that they will sign in and activate the
reference trial. This is intent, not confirmation of completion or entitlement.
Avoid account-UI capture or native focus changes during that action; continue
headless checks and wait for a ready signal before reference scenarios.
