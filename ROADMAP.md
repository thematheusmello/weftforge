# WeftForge roadmap

The target is the complete independent native garment studio, with one-to-one
functional parity against an explicitly frozen reference baseline. These six
gates organize delivery; the first working garment is the first gate, not the
end of the project. [parity/features.yaml](parity/features.yaml) is the status
source of truth. [Generated progress](docs/progress.md) separates implementation
coverage, reference observation, and verification.

Current state: **Gate 1 in progress; no delivery gate accepted.** The native app
supports bounded construction, simulation, saving, recovery and export workflows.
Complete garment fitting, distribution and end-to-end acceptance remain in
progress. See [current status](STATUS.md) for tested builds, known failures and
remaining work.

## Current execution order — 22 November 2026

The [live registry](docs/progress.md) tracks implementation and verification
as discovery continues. All six gates remain open. On native build
`4b15c2ec`, the operator drafted and sewed a Harbor shirt from an empty
project, arranged it, fitted it, changed its front height, saved, quit,
reopened, edited again and exported. Both 400 N/m trajectories first exceeded
15% strain at frame 28. [Native journey](parity/reports/WF-20261122-HARBOR-STUDIO-WORKFLOW-001.json) · [Fit measurements](parity/reports/WF-20261122-HARBOR-STUDIO-FIT-001.json).

In the separate 520 N/m shoulder study, the recorded local sliding traction
raises front warp above 15% in both saved substeps. Measure the friction
coefficient for an identified fabric and contact surface at the recorded loads
and speeds before changing the fit model. [Shoulder contact controls](parity/reports/artifacts/harbor-shoulder-support-20261122/README.md).

1. Finish the native UI and interaction foundation in the PhotoCraft-inspired
   visual direction: consistent panels/tools/dialogs, linked selection, precise
   input, focus, history and responsive job feedback. Contextual tabs, input
   retirement and aligned constructor metre labels have source checks. Historical
   native P005 accepted the bounded layout/normal-close component on E/D-built
   native; its image-free public package cannot independently verify that layout.
   New native workspace-PNG capture has source/pixel/lifecycle/input-preservation
   checks, including the resolved shutdown wait and modal Escape behavior.
   A native File-menu save on the `59501451` build produced an independently
   decoded 1640 × 1000 workspace PNG and preserved the editable project.
   The polygon close transaction now keeps the draft and staged points on
   rejection and records one Undo step on success. The two UI-authored Harbor
   panels were arranged, sewn and saved in the native workspace. The reopened
   project passed the height edit and history checks. Independent readback
   verified its SVG, DXF, OBJ rest surfaces and workspace PNG.
   [Native Harbor case](parity/reports/WF-20261118-HARBOR-NATIVE-AUTHORING-REOPEN-001.json).
   The four Fabric numeric fields expose individual names and SI units in the
   native macOS accessibility tree.
   [Fabric accessibility case](parity/reports/WF-20261118-FABRIC-ACCESSIBLE-NAMES-001.json).
   Scene can hide or isolate patterns across both canvases. The native check
   observed both actions, restored visibility, and exported the complete GLB
   from the hidden view while preserving the editable project.
   [Native visibility case](parity/reports/WF-20261119-NATIVE-PATTERN-VISIBILITY-001.json).
   View and Command search offer six standard 3D orientations. The native
   check switched to Front and Top, scrolled a long command list and restored
   the exact default orbit capture. Custom cameras and garment focus remain.
   [Native viewpoint case](parity/reports/WF-20261119-NATIVE-STANDARD-CAMERA-VIEWS-001.json).
   Command search has a named, auto-focused query and keyboard execution. The
   native macOS check reopened the Harbor document and exported its workspace
   preview using the palette.
   [Native search case](parity/reports/artifacts/command-search-native-20261118/NATIVE-OBSERVATION.json).
   Earlier 8aa notch workflow and genuine modal accessibility-setter observations
   remain historical; they are not transferred to the changed source.
2. Close the first garment path. Shared soft sewing retains every energy in one
   balanced nodal history. Contact now replaces its own nodal actions and exposes
   separately attributed initial-repair projections; fitting requires zero repair
   at initial and every accepted whole-step state. The new complete 181-state
   case on a70c0df0 failed strain in 153 frames, 28 through 180, with peak
   20.243551% and final 15.349281%; its final external-clock proof remains
   unsealed. The capsule correction's fresh complete G case also fails all
   153 strain frames: peak 19.770147%, final 15.721208%. Its original final
   external verdict is retained at 379.697 seconds / 420. Supported contact,
   finite states, uncertainty, zero recovery and closed-target seams pass. Keep
   both failed grades: the new peak is lower while final strain is higher, with
   no broad improvement or old-clock repair. Original inputs/settings/criteria
   and earlier 16.063338%/50.9602% failures remain intact. Diagnose the actual
   edge-contact/sewing/material/history coupling before choosing another engine
   change. The optimized native studio completed the same assembly and drape
   phases on the reopened UI-authored Harbor construction. It exceeded 15%
   strain at both phase endpoints. Moving-triangle nonlinear contact and
   calibration remain open.
   [Native phase endpoints](parity/reports/WF-20261118-HARBOR-NATIVE-FIT-001.json).
   The independent soft-stitch update completed all 181 CLI fit states in
   258.064 seconds on the saved native construction. The first failed state
   was 28. Final strain was 15.65% against the 15% limit. The next contact
   increment checks the active normal projection and friction on the saved
   six-node case before testing another full garment.
   [Independent soft-stitch fit](parity/reports/WF-20261118-HARBOR-ISOLATED-SEWING-FIT-001.json).
3. A native Harbor Fabric edit passed Undo, Redo, editable save, fresh reopen
   and an initial simulation step. Independent parsing verified the native
   simulated OBJ after that step, including physical scale and preserved
   topology and UVs. Finish complete fit and export a fitted mesh. Repeat the
   integrated empty-document journey on one current build and compare the
   recorded reference case.
   A native 50 mm front hem edit survived reopening and reached both the
   assembled and draped garment. The longer front exceeded the strain limit at
   both phase endpoints. Each complete trajectory first exceeded the strain limit at
   state 28. [Dimension edit](parity/reports/WF-20261120-HARBOR-HEM-REGENERATION-001.json) · [Drape comparison](parity/reports/WF-20261120-HARBOR-HEM-DRAPE-001.json).
   [Native Fabric case](parity/reports/WF-20261118-HARBOR-NATIVE-FABRIC-001.json).
   [Native simulated OBJ case](parity/reports/WF-20261118-HARBOR-NATIVE-SIMULATED-OBJ-001.json).
   The current studio also exports the rest and stepped Harbor surfaces as
   embedded JSON glTF. Both matched the validated GLB scene and mesh data.
   A separate single-panel fit checked the successful JSON output route.
   [Native glTF case](parity/reports/WF-20261119-NATIVE-GLTF-JSON-001.json).
4. Work through Gates 2–4 below: professional editing/construction and fitting,
   fabric/contact robustness, avatars/animation/materials/rendering and output.
   Run reference discovery and comparison alongside implementation throughout.
5. Close Gate 5 across every required feature and format, including additional
   capabilities discovered during the full reference audit. Keep file compatibility
   research distinct from successful mesh import or export.
6. Complete Gate 6 locally across macOS/Apple Silicon, Windows and Linux; finish
   recovery, packaging, documentation, assets/licenses and reproducible releases.
   The Apple Silicon package now uses the executables selected by Cargo and
   has a local launch and CLI smoke check with a separate build directory.
   The Debian 13.7 ARM64 package launched offline under X11 with Vulkan software
   rendering. Its CLI completed one simulation step and exported the original
   shirt in OBJ, SVG and GLB.
   [Debian ARM64 package check](parity/reports/WF-20261119-LINUX-ARM64-PACKAGE-001.json).
   The updated package shows Ctrl+K and opens and closes Command search with
   native X11 keyboard input. The JSON glTF package also shows Export glTF in
   the palette and its CLI writes the same JSON glTF as macOS for the original
   shirt. Check the changed visibility build on Linux as well. Windows,
   Linux x86-64, Wayland, hardware GPU and signing checks remain.
   [Debian JSON glTF package check](parity/reports/WF-20261119-DEBIAN-ARM64-GLTF-001.json).
   Keep GitHub Actions disabled throughout development. Re-enable it only after
   the complete project is finished, as the user requested.

Simulator source `e5f043ef` passed formatting, 489 workspace tests across
32 summaries, strict all-target Clippy and optimized studio and CLI builds.
[Independent soft-stitch source checks](parity/reports/WF-20261118-HARBOR-ISOLATED-SEWING-FIT-001.json).
Earlier source `6078c2af` passed formatting, 488 workspace tests, strict
all-target Clippy and native and CLI builds.
[Polygon close source checks](parity/reports/WF-20261118-POLYGON-CLOSE-SOURCE-001.json).
Earlier source `59501451` passed formatting, 486 workspace tests across
32 summaries with zero failed/ignored, strict all-target Clippy and native/CLI
debug builds in 386.000 seconds within 900. Its 102 simulator tests are included
within 486. These executions belong to H. The actual G xtask pass in 306.716
seconds, separate optimized build and failed full numerical case retain their
own source and exact archived reports; they do not transfer to H. Current
source/runtime evidence and the updated 457-anchor map passed clean verification
on `6872e7c3`. Native PNG P003 verifies one actual File-menu capture on macOS.
The complete garment workflow, full native feature contracts and reference
acceptance still require their own cases. Push each atomic development commit
after its local checks.
Earlier CLI/native notch results retain their 8aa source identity. All full
feature contracts and release gates remain open.

## Before and throughout every gate

- Freeze reference build, installer/executable hashes, OS/architecture, units,
  locale, graphics, and relevant preferences. Audit documentation and the running
  UI, including all menus, tools, dialogs, ranges, shortcuts, errors, and workflows.
- Preserve implementation-independent contracts, original hashed fixtures, inputs,
  exact actions, measured outcomes, commit identities, and evidence freshness.
  Keep blocked, unobserved, partial, failed, and stale results visible.
- Match exact authoring/interaction semantics where meaningful. Declare numerical
  metrics and tolerances before running a comparison; distinguish repeatability
  from agreement within tolerance across CPUs/GPUs.
- Keep editable construction authoritative and separate from rest meshes, current
  solver state, display/export meshes, and caches. Preserve correspondence through
  edits/remeshing or issue an explicit invalidation report.
- Use SI metres/kilograms/seconds, f64 authoring and CPU reference simulation,
  explicit coordinate/scale conversions, stable identifiers, validated typed
  commands, coherent transactions, and bounded input/resource use.
- Share operations through native UI, CLI, and the planned local MCP interface.
  Keep core libraries GUI-independent. Jobs carry input revisions, progress and
  cancellation; reject stale results and preserve recoverable state on failure.
- Maintain public issues/PRs, milestones, generated reports/dashboard, CI and
  benchmark commands. Use a Project board/Pages where permissions permit; retain
  usable issues/reports and explicit blockers independently.

## Gate 1 — First complete garment workflow

- [ ] Launch the actual native desktop app with connected 2D pattern and 3D garment
  views, hierarchy, contextual tools, properties, materials and simulation controls.
- [ ] From an empty document, author editable shirt pieces through the UI, sew
  them, arrange around an original/redistributable avatar, simulate and fit.
- [ ] Change a garment dimension and fabric parameter; regenerate/resimulate the
  actual garment while preserving construction and showing errors clearly.
- [ ] Save, close, reopen, edit again, undo/redo, and export a supported mesh.
- [ ] Independently inspect exported scale/axes/UVs/normals and compare the declared
  reference scenario. A pre-generated sample or screenshot cannot substitute.

Acceptance: witnessed empty-project workflow through the real UI/engine, editable
reopen, independently checked output, and retained discrepancies. Current source
contains a partial integrated document/solver/UI path; acceptance is pending.

## Gate 2 — Professional editing and construction

- [ ] Complete lines/curves/control points, common primitives, free drawing,
  dimensions/measurements, grain direction, names, notches, internal lines and holes.
- [ ] Complete move/rotate/scale/mirror, linked symmetry, split/join/cut, offsets,
  seam allowance, darts, pleats, and precise numeric dimension changes.
- [ ] Preserve curves and relationships through triangulation/refinement. Reject
  self-intersection, degenerate geometry and unsupported edits with useful errors.
- [ ] Complete edge/free/internal sewing, partial boundaries, ordered edge chains,
  reversed and curved seams, deliberate easing/gathering, edit/removal, direction
  and length-mismatch diagnostics. Reference oriented authored intervals and
  explicit correspondence rather than tessellation indices.
- [ ] Implement folds, elastic, buttons/buttonholes, zippers, trims and attachments;
  distinguish physical constraints, visible topstitching and export welding.
- [ ] Complete linked 2D/3D selection/editing, arrangement aids, translation/rotation,
  pins/temporary constraints, cloth grabbing, multi-piece/outfit layers,
  hide/isolate, reset arrangement and drape-to-construction workflows.
- [ ] Complete shortcuts, command search, contextual tools/dialogs, snapping,
  accessible keyboard/numeric input, high-DPI behavior and single-gesture history.

Acceptance: curved seams with unequal tessellations/reversed orientation/gathering;
curve edit/split/resolution/undo tests preserve supported relationships or report
invalidation; sleeve-length fitting remains editable after reopening.

## Gate 3 — Fabric and contact quality

- [ ] Document equations, assumptions, integrator, constraint/contact treatment,
  convergence and failure behavior. Establish the repeatable CPU reference path.
- [ ] Validate gravity, mass per area, anisotropic stretch/shear, bending, damping,
  friction, grain direction and collision thickness; keep appearance separate.
- [ ] Establish reliable seams, avatar/ground/self-contact and interacting garment
  layers. Never silently disable contact or substitute canned deformation.
- [ ] Support play/pause/reset/controlled steps/resimulation, quality/resolution,
  cancellation, non-finite/instability/intersection diagnostics and recovery.
- [ ] Calibrate measured fabrics where evidence exists; clearly label approximate
  presets. Check material consistency under refinement and smaller timesteps.
- [ ] Measure convergence, rest/mass/constraints, contact errors, drape/strain,
  shirt-and-jacket interaction and temporal behavior on original fixtures.
- [ ] Benchmark garment sizes, solver step time, simulated/wall time, viewport
  frame time, memory, cancellation and save responsiveness separately.
- [ ] Add GPU acceleration only with measured benefit and CPU comparisons; record
  backend/hardware limits and tolerance-based reproducibility.

Acceptance: predeclared analytical/numerical/reference metrics pass for small
cases and meaningful multi-piece/layered scenes; unresolved failures stay visible.

## Gate 4 — Animation and professional presentation

- [ ] Import validated static/animated avatars with explicit units/axes/scale,
  inspectable collision geometry, skeletal animation and supported rigging paths.
- [ ] Integrate timeline/frame ranges, recording, simulation caches, playback and
  scrubbing; changes to relevant authoring inputs explicitly invalidate caches.
- [ ] Complete material assignment by piece/region, physical presets and grain,
  texture transforms, relevant PBR maps and appearance-only edits.
- [ ] Add practical lighting/shadows, wireframe, UV inspection, deformation/contact
  diagnostics, strain views and scene capture, with responsive navigation.
- [ ] Complete output mesh density, pattern-derived UVs, normals, thickness,
  seam treatment, retopology and other output-preparation workflows.

Acceptance: animated-avatar simulation and cache scrubbing, followed by material
or seam changes and a fresh simulation; validated time/units/temporal contact and
independently checked animation output.

## Gate 5 — Reference-workflow coverage and interoperability

- [ ] Complete the runtime feature inventory and close all remaining required
  contracts/dependencies across interaction, authoring, workflows,
  simulation/output, files and operation. Record intentional differences.
- [ ] Cover advanced drawing on avatars, garment-shape-preserving edits,
  measurements, arrangement/construction, modular workflows, graphics,
  topstitch/puckering, animation/rigging, scripting and other discovered features.
- [ ] Inventory substantive hosted creation features and provide offline local
  substitutes. Vendor billing, activation and storefront administration are the
  declared exclusions; creation features are not excluded for being hosted.
- [ ] Complete OBJ and glTF/GLB avatar/mesh workflows, SVG and a precise DXF
  pattern subset, physical-scale printing, animation/cache interchange, and
  further production formats identified by the reference audit.
  The native pocket case covers one editable SVG cutout through save, reopen,
  a corner edit and rest-surface export. Curved and multiple cutouts remain open.
  [Pocket import and edit](parity/reports/WF-20261120-NATIVE-SVG-CUTOUT-001.json).
- [ ] Keep import/export, editable construction versus mesh-only, static versus
  animated data, supported entities, scale/axes, UVs/normals/material assignments,
  welding/thickness and data-loss behavior explicit in the format matrix.
- [ ] Research native Marvelous Designer project compatibility independently.
  Preserve available specs/test evidence, unsupported data and compatibility gaps;
  mesh import or renaming extensions does not qualify.

Acceptance: independent consumers/parsers confirm advertised interchange;
complete reference contracts have fresh qualifying evidence and unresolved
compatibility requirements remain visible rather than silently discarded.

## Gate 6 — Distribution and release readiness

- [ ] Validate native build, launch, high-DPI/accessibility, performance,
  cancellation and recovery on macOS/Apple Silicon, Windows and Linux.
- [ ] Complete portable versioned projects, assets/references, authored constraints,
  attachments, avatars, animation/simulation settings and supported saved state.
  Keep optional caches separate; validate migrations, bounded malformed input,
  missing/changed assets, atomic saves, autosave and explicit recovery.
- [ ] Inject an interrupted save on disposable original data, verify the prior
  valid project survives, and test unsupported versions without silent replacement.
  The core interruption test and the native invalid-open and explicit
  recovery paths now have Apple Silicon evidence.
  [Harbor recovery case](parity/reports/WF-20261118-HARBOR-NATIVE-RECOVERY-001.json).
- [ ] Package actual applications/CLI, original sample projects, screenshots,
  installation/development/user documentation, licenses, asset/dependency notices,
  and reproducible release artifacts/checksums.
- [ ] Audit bundled native dependencies and exceptions precisely; the application
  must not require a browser/Electron/JS frontend, Python service, Blender
  subprocess, proprietary runtime SDK, account, subscription or paid API.
- [ ] Maintain public build/test workflows and redistributable regressions.
  Licensed-reference runs use trusted revisions on a configured environment,
  isolated from untrusted forks and public credentials.
- [ ] Publish accurately labeled prereleases with platform evidence and known
  limitations; keep latest builds, CI/release links and continuation steps current.

Acceptance: recovery, packaging, interoperability and platform regression evidence
for the declared release scope. Complete parity may be claimed only when every
required contract/dependency in the frozen baseline has current qualifying evidence.
