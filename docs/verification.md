# Verification and acceptance

Implementation checks and reference comparisons are separate. No garment
workflow has qualifying Marvelous Designer comparison evidence. The reference
app launched at its terms/sign-in boundary; its garment workspace, units,
preferences, graphics and complete UI build are unobserved.

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
