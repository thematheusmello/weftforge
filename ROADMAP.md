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
