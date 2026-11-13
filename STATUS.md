# WeftForge status — active development, 2026-11-13

The complete independent garment studio and one-to-one recorded-reference parity
remain the goal. Full gate1 is not accepted; current evidence is an implementation
increment. No feature has verified reference parity. The56-contract inventory is
incomplete and must expand with discovery.

## Current recoverable state

Published source: `ff3914474c8acc30d10c70a8489cae56cfa2c872` on main. The public
repository and origin are verified: https://github.com/thematheusmello/weftforge.
Scoped run reports and a one-line evidence-CI toolchain correction are on
`evidence/first-garment-checkpoint`. Reports retain exact source/input/artifact
hashes and the single-candidate GUI replay gap. No unrelated source existed.

Delivery issues [#1–#7](https://github.com/thematheusmello/weftforge/issues) and six
milestones cover discovery and every roadmap gate. Reproductions:
[tight-shirt/convergence #8](https://github.com/thematheusmello/weftforge/issues/8)
and [arc-length partial sewing #9](https://github.com/thematheusmello/weftforge/issues/9).
GitHub Pages deployed successfully in [run43512419050](https://github.com/thematheusmello/weftforge/actions/runs/43512419050);
https://thematheusmello.github.io/weftforge/ returned HTTP200 with the correct title.
The first source CI run is [43512419034](https://github.com/thematheusmello/weftforge/actions/runs/43512419034).
Linux/macOS/Windows tests and clippy passed; release builds were still running at
last poll. The evidence job failed Rustup component installation (`cargo-fmt`
conflict); the branch explicitly installs manifest components to repair it.
New CI on the fix remains required. Project board creation failed:
token lacks project/read:project scopes. Issues and registry work independently.

## What works and how it was checked

Editable core: stable points/straight and cubic curves/pieces/materials/seams;
oriented parameter intervals and explicit correspondence; conforming refinement,
interior quality flips and boundary correspondence; typed transactions/history;
actual edit→rest mesh; bounded versioned atomic saves/recovery; OBJ/SVG export.
Sewn-curve split explicitly fails pending interval rewriting. Cubic intersection
validation has a documented tessellation tolerance rather than exact predicates.

CPU solver: f64 XPBD material strain and dihedral bend, area mass, damping/friction,
weighted seams with explicit bounded-speed assembly, avatar/ground and discrete
vertex/triangle self/layer contact, diagnostic guard and transactional cancel/fail.
No GPU simulation, full edge/edge CCD, or measured/calibrated fabric presets.

Native editor: connected2D/3D views, original capsule mannequin, polygon/rectangle
creation, interval sewing, numeric edits/placement/shared material, coherent undo,
revision-tagged mesh/simulation workers, save/open/export, untitled autosave and
explicit recovery. Initial solver contact construction has only before/after
cancellation checks; file IO and validated commands retain synchronous portions.

Actual UI observations: authored two14-point relaxed-shirt pieces from empty,
sewed8 edges, arranged atZ±0.22m, saved/closed/reopened, changed resolution.07→.12m,
assembled60steps with gravity0, then draped120steps. Both widths changed1.06→1.08m,
Undo/Redo restored exact numeric values, saved, and resimulated; OBJ/SVG came from
actual UI. Final158vertices/244triangles, t3s; independent output parser measured
max axis strain.0854172, seam gap3.07e-7m, avatar/ground thickness error~1.64e-16m,
and exact authored millimetre SVG paths. These satisfy predeclared scene limits,
not full product/reference parity. Initial authoring preceded the phase-buttons
build and its digest was not retained: a fresh complete single-candidate replay
remains necessary. Final binary digest and screenshot are in docs/images/.

`cargo fmt --all --check`, workspace tests, strict all-target clippy, registry
validation and independent OBJ/SVG scenario passed on the integrated source.
Independent evaluator tests also pass; a deliberate1mm SVG-scale mutation is
rejected. Public platform CI has passed source tests and lint on all three OSes; full
workflow success and GUI launches remain unverified. See verification
and physics docs for equations, tolerances, commands, hardware and evidence scope.

## Failures and remaining scope

The original tight-shirt fixture stays unchanged. Its default simultaneous-gravity
and refinement runs fail fitting criteria despite finite output. Its0.96m body
circumference is smaller than the1.131m torso; increasing mesh quality alone does
not establish convergence. A separate relaxed fixture passes only its documented
explicit assembly/drape protocol (CPU slower than real time). Keep both results.
Partial curved-sewing parameter fractions differ from a documented boundary
arc-length API lead; runtime semantics remain unobserved.

Advanced patterns, attachments, avatar imports, grain authoring/regions,
PBR/textures/shadows, animation/timeline/caches, layers UI, glTF/DXF/printing,
MD-native decoding, local MCP and production output preparation remain open.
No distributable binary release is approved by this status: full notice collection,
three-platform launch/package QA and signing remain release work. Native bindings,
platform libraries and drivers are accurately excluded from any all-Rust claim.

## Reference laboratory and blocker

Official Marvelous Designer Personal2026.1.237 was downloaded, hashed, installed
and launched. REA6.9.0 selected-window evidence observes the terms/sign-in boundary.
User must personally review terms and sign in/activate a legitimate trial/license.
No garment reference workflow has executed. Baseline is provisional: full observed
build/settings/units/locale/graphics are not frozen. See parity/reference.json.
Raw installer/binary/account artifacts are private outside Git. The local private
manifest location is in the thread handoff, never public files. No proprietary
code analysis was used. REA CLI works with existing Node26.12.1; current-session
MCP unconnected, Ghidra unconfigured. No setup or download is still running.

## Next exact actions

1. Finish and merge the evidence/CI-fix PR after checks and repository requirements.
   Inspect authoritative CI/Pages terminal states. Preserve the initial failed
   evidence job and fix any new platform failure without weakening checks.
2. Replay the entire empty-project workflow on one identified candidate, including
   post-reopen dimension edit and actual export. Preserve observed failures.
3. Improve authored sewing length semantics, robust relationship-preserving sewn
   splits and curve UI; add local command automation for reproducible workflows.
4. Fix garment convergence/contact beyond these fixtures; measure layered shirt+
   jacket, contrasting fabrics and responsiveness. Continue all six roadmap gates.
5. When reference access is available, freeze baseline preferences and run the
   same controlled scenarios; no implementation-only evidence counts as parity.

## Last confirmed live state

At native capture the WeftForge PID64498 was alive, paused after a completed3s
drape, no background job. Reference PID18283 was alive at last investigator poll,
terms/license still pending. No research/benchmark process remains live. Recheck
actual processes/handles before assuming either app or any job is still running.
