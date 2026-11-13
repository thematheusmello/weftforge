# WeftForge status — active development, 2026-11-13

The complete independent native garment studio and recorded one-to-one reference
parity remain the goal. No feature has verified reference parity: 0/56 inventoried
contracts, with discovery incomplete and the full-product denominator unknown.
All six roadmap gates remain open beyond the bounded implementation evidence.

## Recoverable implementation and publication

Verified public repository/origin: https://github.com/thematheusmello/weftforge.
Published main: `69ad78ce018674db4595c5ddf6e0a18b00275c82` ([PR14](https://github.com/thematheusmello/weftforge/pull/14)).
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

Current root branch `feat/grain-native-acceptance` integrates the grain increment
from recoverable commit `b8c9a1a615787c851d6fea15265faf30541f1024` in its separate
`feat/grain-authoring` worktree. It adds authored grain, orthotropic materials,
strict v1→v2 migration, native/CLI controls and the inspector fix. Source integration checks passed (formatting,44 Rust tests, strict clippy and
24 Python/registry checks). New native acceptance files are retained locally for a
reviewed report after the source checkpoint is public. The merged
[reference-access PR14](https://github.com/thematheusmello/weftforge/pull/14) records
the latest read-only sign-in observation; all eight push/PR checks passed. Main
CI/Pages for its merge are being verified separately. No source build/probe is
running locally.

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
animation checks remain absent. New screenshot/report review is in progress.

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

1. Finish source integration checks and publish the grain/inspector fix. Retain a
   public source checkpoint, then publish new scoped native reports and the
   anisotropic failure. Preserve all old fixtures/artifacts and accurate freshness.
2. Continue the isolated `feat/sewn-curve-editing` increment: native curve controls
   and exact sewn-interval partitioning on split, with relationship/history/remesh
   checks. Its agent has explicit separate ownership and no publication authority.
3. Improve sewing semantics and garment/contact convergence, then measure layered
   shirt+jacket, contrasting fabrics/refinement and real responsiveness. Continue
   professional construction, avatar imports, animation/caches, PBR, interchange,
   printing, output preparation, automation and all distribution gates.
4. When reference access is available, freeze settings and execute the controlled
   original scenarios. Implementation-only checks never count as reference parity.

Last observed own-app PID52560 is paused at3s Drape with no active-job indicator.
Historical reference PID18283 showed sign-in at23:06UTC. Grain build/check sessions
completed; the curve agent is active in its isolated worktree. Recheck actual
processes and CI handles before resuming; do not duplicate live work.
