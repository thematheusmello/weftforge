# WeftForge status — active development, 2026-11-13

The complete independent garment studio and one-to-one recorded-reference parity
remain the goal. The first bounded garment workflow has a complete single-candidate
implementation replay; reference acceptance and all remaining gates are open. No feature has verified reference parity. The56-contract inventory is
incomplete and must expand with discovery.

## Current recoverable state

Published source: `ff3914474c8acc30d10c70a8489cae56cfa2c872` on main. The public
repository and origin are verified: https://github.com/thematheusmello/weftforge.
Scoped reports and the evidence-CI toolchain correction merged in
[PR10](https://github.com/thematheusmello/weftforge/pull/10) at
`8341048fa0bcedea9fab10a178a579db4b103630`. A separate complete single-candidate
UI replay and retained output are on `evidence/single-candidate-workflow`. Reports retain exact source/input/artifact
hashes and the single-candidate GUI replay gap. No unrelated source existed.

Delivery issues [#1–#7](https://github.com/thematheusmello/weftforge/issues) and six
milestones cover discovery and every roadmap gate. Reproductions:
[tight-shirt/convergence #8](https://github.com/thematheusmello/weftforge/issues/8)
and [arc-length partial sewing #9](https://github.com/thematheusmello/weftforge/issues/9).
GitHub Pages deployed successfully in [run43512419050](https://github.com/thematheusmello/weftforge/actions/runs/43512419050);
https://thematheusmello.github.io/weftforge/ returned HTTP200 with the correct title.
A later [Pages run43515229132](https://github.com/thematheusmello/weftforge/actions/runs/43515229132)
failed an observed artifact-visibility race: upload14080172593 succeeded, deploy
immediately listed0 artifacts, and the same run later listed1 unexpired artifact.
The single-candidate branch adds a bounded API visibility check before deploy;
its actual successful deployment remains required.
The first source CI run is [43512419034](https://github.com/thematheusmello/weftforge/actions/runs/43512419034).
Linux/macOS/Windows source builds/tests/clippy passed; the initial evidence job
failed Rustup component installation (`cargo-fmt` conflict). Explicit manifest
components fixed it. Both [pushCI43513630203](https://github.com/thematheusmello/weftforge/actions/runs/43513630203)
and [PRCI43513640378](https://github.com/thematheusmello/weftforge/actions/runs/43513640378)
finished green on the merged fix, including all3 platforms and evidence checks. Project board creation failed:
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
not full product/reference parity. That first composite run is retained unchanged. A fresh full replay then used
the SAME identified f00e executable from empty through authoring/simulation/save/
quit/relaunch/reopen/edit/history/resimulation/export. It saved1.08m widths and
verified them after reopening, edited both to1.10m, and produced independently
checked outputs: strain.08071753, seam2.79e-8m, contact~1.50e-16m. Native source
identity and screenshot are in docs/images/studio-single-candidate.*; exact actions
and original points are in the new candidate scenario/report.

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

1. Finish the single-candidate evidence PR, verify report consistency and Pages
   publication, and retain all failed scenes. Source code is unchanged from the
   fully tested candidate and merged evidence-CI fix.
2. Continue professional editing/sewing and resolve per-run evidence freshness
   before code evolves: current validator conservatively checks every registered
   run source against current files, so historical runs must remain attributable
   without falsely blocking a newly rerun current check. Narrow source scopes and
   retain per-run stale status rather than dropping failures.
3. Improve authored sewing length semantics, robust relationship-preserving sewn
   splits and curve UI; add local command automation for reproducible workflows.
4. Fix garment convergence/contact beyond these fixtures; measure layered shirt+
   jacket, contrasting fabrics and responsiveness. Continue all six roadmap gates.
5. When reference access is available, freeze baseline preferences and run the
   same controlled scenarios; no implementation-only evidence counts as parity.

## Last confirmed live state

At latest native capture the WeftForge PID1550 was alive, paused after a completed3s
drape, no background job. Reference PID18283 was alive at last investigator poll,
terms/license still pending. No research/benchmark process remains live. Recheck
actual processes/handles before assuming either app or any job is still running.
