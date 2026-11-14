# CPU cloth model

This is an independently authored CPU solver, not a calibrated reconstruction of
Marvelous Designer. All checks below are implementation-only. No reference drape,
contact, numerical tolerance or complete garment workflow has been compared yet.

## State and units

`weftforge-sim` depends only on Serde and the standard library. It has no document,
rendering or GUI dependency. Its input is an immutable snapshot of the rest mesh,
physical material coordinates, per-triangle materials, weighted seam points, pins
and original analytic colliders. The application owns revision correspondence,
regeneration, worker cancellation and the decision to display a published snapshot.
A cloned solver contains an independent state; successful stepping does not mutate
its input or another clone.

Lengths and material coordinates are metres, time is seconds, mass is kilograms,
3D is Y-up, and all simulation arithmetic is `f64`. A material coordinate is an
unscaled pattern-space coordinate, not a normalized image UV. The display mesh's
rendered thickness never enters the solver. `collision_thickness` is the physical
midsurface offset; the separation between two cloth layers is the sum of their
offsets.

A triangle of material-space area A and areal density rho contributes rho*A/3 to
each incident particle. Mass is based on the unstretched cloth, independent of its
arrangement or deformed surface area. Pins have zero inverse mass during solving,
but retain their physical mass in mass/energy accounting. Every vertex requires
positive incident mass. Non-finite inputs, malformed/duplicate/non-manifold
triangles, degenerate material/rest triangles, invalid materials, seam weights,
colliders and duplicate pins are rejected before a solver is returned.

## Integration and compliant constraints

The engineering choice is a deterministic sequential CPU XPBD path, using the
compliant local projection method from
[Macklin, Müller and Chentanez, XPBD (2016)](https://matthias-research.github.io/pages/publications/XPBD.pdf).
The small dependency surface and explicit physical energy coefficients make it
suitable for repeatable checks and future accelerated-path comparisons. It is not
a promise of time-step, mesh or iteration independence at finite convergence.

One public step of duration dt is subdivided into h = dt/substeps. For free nodes:

```text
v* = exp(-damping*h) * (v + h*gravity)
x* = x + h*v*
```

The solver clears elastic/seam multipliers at each substep and iterates membrane,
bending, sewing, analytic contact, cloth contact and pins in that order. For an
energy E = k*C(x)^2/2, the constraint compliance is alpha = 1/k:

```text
a = alpha / h²
delta_lambda = (-C - a*lambda) / (sum_i(w_i*|gradient_i(C)|²) + a)
x_i += w_i*gradient_i(C)*delta_lambda
lambda += delta_lambda
```

The new velocity is the solved displacement divided by h. Damping is exponential
velocity damping in s^-1, rather than a frame-dependent interpolation. Its
operator splitting introduces a finite-step error in forced trajectories. The
zero-damping gravity test has the expected symplectic Euler error. There is no
restitution, air drag, wind, calibrated viscoelasticity or implicit global solve.

The defaults are dt=1/60 s, four substeps, eight constraint iterations, sewing
closure speed 0.5 m/s and gravity (0,-9.81,0) m/s². Step arguments require finite
gravity, nonnegative finite sewing speed, 0 < dt <= 1 s and 1..=1000
substeps/iterations. These validity limits are not stability guarantees.
Increase iterations and reduce substep duration when residuals remain high.

## Stretch, shear and grain

Each triangle derives its 3x2 deformation gradient F = Ds*inverse(Dm), using
pattern coordinates in Dm and deformed edge vectors in Ds. The editable piece's
`grain_angle` is radians counterclockwise from material +U toward +V; the document
bridge copies that angle and its assigned material's distinct warp/weft stiffnesses
into each triangle's solver input. The 2D arrow displays this same warp axis.
For unrotated deformation columns f_u and f_v:

```text
f_w = cos(angle)*f_u + sin(angle)*f_v
f_f = -sin(angle)*f_u + cos(angle)*f_v
```

The scalar constraints are:

```text
C_w = (dot(f_w,f_w) - 1)/2
C_f = (dot(f_f,f_f) - 1)/2
C_s = dot(f_w,f_f)
k_w = A * warp_stiffness
k_f = A * weft_stiffness
k_s = A * shear_stiffness
```

All three membrane stiffnesses are N/m. Their energy coefficients are N m; their
constraints are dimensionless. This is a simple orthotropic Green-strain model
with no Poisson coupling, plasticity or compression-specific wrinkling law. The
standalone solver DTO defaults of 0.2 kg/m² and 800/600/120 N/m warp/weft/shear
are uncalibrated examples. The application's document defaults remain 0.18 kg/m²
and 400/400/100 N/m; v1 migration copies the old stretch value to both axes and
sets piece grain to zero, preserving the old physical inputs. Neither set is an
identified measured fabric or a reference-application preset. Shear, bending,
density, damping, friction and collision offset remain separate parameters;
changing the display color does not change them.

`max_stretch_error` reports max(abs(length(f_w)-1), abs(length(f_f)-1)). It is a
material-axis length diagnostic, not a bound on all principal strains or shear.

The authored numerical bridge regression loads a planar 0.2 m square against the
ground, with density 0.2 kg/m², warp 50 N/m, weft 1000 N/m and damping 8 s^-1.
After one second (dt=1/120 s, two substeps, 24 iterations), its top is
0.2029735316 m at grain zero and 0.2023383529 m after a quarter-turn: 0.635179 mm
more compression, exceeding the declared 0.3 mm minimum response. A color-only
edit produces exactly the same solved positions. This bounded planar loading test
checks that authored anisotropy reaches the real CPU engine, not buckling,
calibration or general garment quality. It runs in
`crates/weftforge-core/tests/grain.rs`.

The independent output evaluator rotates its geometric deformation columns into
the piece's authored axes too; legacy files with no angle use zero. A hand-derived
45-degree fiber test under 1.2x U stretch yields sqrt((1.2²+1)/2)-1 rather than the
unrotated 0.2. It calls no solver code. This keeps its material-axis diagnostic
consistent with the stated quantity without supplying a reference drape answer.

## Bending

Each interior manifold edge creates a signed dihedral-angle constraint relative
to the arranged rest mesh. Its analytic positional gradient is independently
checked with central finite differences. The angular difference is wrapped to
[-pi,pi]. With shared edge length l, mean triangle altitude H and material
flexural rigidity D in N m:

```text
C_b = wrap(theta - theta_rest)
k_b = D*l/H
```

D is averaged across the incident materials. This edge discretization is an
approximate thin-shell bending model; it has no measured-fabric calibration and
has not demonstrated resolution-independent folding. Degenerate deformed faces
cannot provide a usable dihedral gradient and are skipped for that projection;
reported strain/residuals and the numerical checks remain relevant. Zero D
intentionally disables that material's bending energy, not collision.

## Sewing, pins and construction correspondence

A stitch constrains the distance between two convex weighted points. Each point
lies on a mesh segment through two vertex indices and weights summing to one.
The scalar distance constraint uses compliance in m/N. Repeated indices are
combined before computing the effective inverse mass. The public
constraint therefore works with different segment samplings and does not depend
on matching vertex counts. It does not itself choose curve correspondence; the
authoring/meshing layer supplies the interval samples and direction.

A sewing assembly actuator begins at each stitch's initial arranged separation.
For each substep it reduces the temporary target distance L monotonically:

```text
L_next = max(0, L_previous - seam_closure_speed*h)
C_seam = length(weighted_a - weighted_b) - L_next
```

This explicitly models assembly, separate from fabric stiffness. At zero target
the stitch enforces coincidence. The default speed is 0.5 m/s; zero pauses the
target reduction while allowing cloth dynamics. `seam_assembly_remaining` reports
the maximum temporary target, while `max_seam_error` reports the actual gap.
Changing speed cannot reopen a closed stitch. Reset restores the original targets.
The candidate target state participates in the same transactional cancellation
and error recovery as positions. Gravity remains active unless the caller
explicitly chooses a zero-gravity assembly stage. Unsewn panels can fall off the
avatar before slow assembly finishes; assembly controls do not establish fitting.

A hard stitch has zero compliance. Compliance is per sampled stitch, so finite
seam stiffness has not been homogenized against sample density. No angular
bending constraint is created across a sewn pair; that seam currently acts as
a hinge. Incompatible pins, seams and contact can leave
finite unsatisfied constraints; the returned seam/contact residuals expose this.
Pins are reapplied after each iteration. Reset restores the supplied arrangement,
zero velocity, zero simulated time and initial-intersection diagnostics; a pin's
target is applied when stepping, rather than rewriting the arrangement.

## Contact and friction

Contact is always evaluated. There is no scene-size threshold that disables self
or intergarment contact. All merged pieces and garments participate in the same
particle/triangle path. A swept AABB rejection reduces closest-point work, but
the broadphase currently remains O(vertices*triangles) per iteration.

Particle/triangle closest points include face, edge and vertex regions. Penetration
below the summed thickness creates an inverse-mass-weighted unilateral position
projection on the particle and the three triangle nodes. Relative tangential
displacement is limited by a Coulomb-style friction correction, bounded by mu
times the normal correction. This transfers motion between layers. Mesh one-ring
neighbors and directly stitched endpoint neighborhoods are excluded because
contact between adjacent elements would oppose their shared surface or seam.

An additional linearized swept face test preserves the previous side when a
particle endpoint has crossed a triangle interior. This is **not full continuous
collision detection**: deforming triangle trajectories, tangential crossings and
edge/edge intersections can be missed. Very thin, fast or initially entangled
scenes can interpenetrate. Initial intersections are counted where this contact
model detects them and are retained in diagnostics; the constructor permits such
arrangements so contact can attempt recovery. An undetected edge/edge crossing
cannot be interpreted as an intersection-free scene.

Ground is an exact horizontal offset plane. Capsules use exact closest distance
to their axis segment, plus radius and cloth thickness; a zero-length capsule is
a sphere. The original torso collider is an ellipsoid with inflated radii and
radial projection, an approximation to a constant signed-distance offset. No
proprietary avatar is bundled. Arbitrary triangle-mesh avatars and animated or
moving collider trajectories are not supported by this crate yet. Collider
contact friction uses the cloth's physical friction coefficient.

Because contacts are sequential and pins are authoritative, the final projection
can leave a previously resolved contact slightly penetrated or conflict with a
pin. `max_contact_error` recomputes the supported current contact residual after
the complete step; `contacts` counts projections including repeated iterations,
not unique colliding pairs. An exact zero residual in a small fixture does not
establish production collision robustness.

## Failure, cancellation and recovery

`Cloth::new_cancellable` builds a complete candidate or returns `Cancelled`;
`Cloth::new` delegates with a non-cancelling callback. Construction polls within
input validation, constraint/mass/seam setup, contact bounds and initial
intersection analysis. Contact analysis checks each vertex and every 256 triangle
or collider candidates, including rejected candidates. Its initial count and
maximum penetration come from the same completed analysis. The core
`create_cloth_cancellable` adapter forwards the native worker's existing atomic
cancel flag; cancellation never installs a partial cloth or alters the accepted
mesh/document. The ordinary adapter remains available for headless callers.
Project/mesh validation, vector allocation/copying and final linear diagnostic
aggregation retain before/after-only checks; this is cooperative cancellation,
not a hard latency guarantee. Deterministic callback sentinels cover cancellation
inside the initial triangle-candidate loop; actual native cancellation interaction
and measured latency still require separate acceptance.

A step builds candidate positions/velocities locally. Cancellation is checked at
substeps, iterations and every 32 particles of the cloth contact loop. A failed or
cancelled call preserves positions, velocities, time and diagnostics from the
last successful call. The caller receives the explicit error. Non-finite state or
derived diagnostics, displacement above 100 m in one substep, or material-axis
extension above 500% rejects the complete call as unstable. These are runaway
guards, not a complete inversion or energy-stability detector. Large finite seam
or contact residuals are returned without disguising them as successful fitting.
Reset remains available after errors. The final stepped residual pass and direct
reset still use the non-cancelling contact-analysis path. Cancellation latency in
those phases depends on mesh size; background worker ownership is required by the
application.

## Numerical evidence

Retained runtime reports below describe their identified historical source and
fixtures. The grain-authoring document/bridge/UI and independent-evaluator change
marks those affected runs stale in the per-report registry without changing any
outcome, source/artifact hash or failed scene. Current native grain/reopen/export
workflow evidence remains pending; the focused authoring tests above are
implementation-only.

Run the focused suite with:

```sh
cargo test -p weftforge-sim --lib -- --nocapture
cargo clippy -p weftforge-sim --all-targets -- -D warnings
```

The following original fixtures passed locally on 2026-11-13. They are analytic
or independently justified implementation checks, **not reference comparisons**.
Assertions remain in `crates/weftforge-sim/src/tests.rs`.

| Check | Declared criterion | Observed |
| --- | --- | --- |
| Rest/mass | 1 m² at 0.2 kg/m²; mass error <1e-14 kg; unforced drift <1e-12 m | Passed |
| Gravity | Symplectic discrete trajectory <1e-11 m error; 40 substeps over 0.1 s within 1.3 mm of analytic freefall | Passed |
| Weighted sewing | Unequal endpoint weights close within 1e-12 m and preserve equal-mass center of mass | Passed |
| Dihedral derivative | Central finite difference, epsilon 1e-6 m, gradient error <1e-7 | Passed |
| Bend response | Fold angle decreases, center of mass changes <1e-12 m | Passed |
| Analytic contact | Ground, capsule and ellipsoid fixture residual <1e-6 m | Passed |
| Layer interaction | Fixed lower layer resolves <1e-6 m; free lower/upper layers both respond by >1 mm | Passed |
| Swept face/friction | One fast face crossing stays outside layer; rough supported cloth slide <1e-8 m while frictionless cloth slides >0.07 m | Passed |
| Grain | Rotated soft axis extends over five times the initial stiff-axis arrangement | Passed |
| Cancellation/recovery | Mid-iteration cancellation and numerical runaway retain prior snapshot; subsequent valid stepping succeeds | Passed |
| Assembly | Target shrinks at declared speed, can pause, reaches coincidence, cancellation/reset retain or restore target | Passed |
| Finite versus quality | Pinned cloth may remain over 1 m inside a conflicting ground; finite state reports the residual and does not claim contact success | Passed |
| Input validation | Bad indices, non-finite coordinates, negative density, invalid seam weights and duplicate pins rejected | Passed |

A 1x1 m vertical patch at 0.2 kg/m², pinned across its top, was run for two seconds
with 100 N/m isotropic membrane stiffness, 8 s^-1 damping and 16 iterations:

| Discretization | Substep h | Mean bottom extension | Total mass |
| --- | --- | --- | --- |
| 2x2 cells, 8 triangles | 1/120 s | 0.009002080 m | 0.2 kg |
| 4x4 cells, 32 triangles | 1/240 s | 0.009306935 m | 0.2 kg |
| 2x2 cells, 1000 N/m | 1/120 s | 0.000914093 m | 0.2 kg |

The independently derived small-strain continuum extension under self-weight is
rho*g*L²/(2*E)=0.00981 m for the 100 N/m patch. The declared acceptance bounds were
mass error <1e-12 kg, coarse/fine extension difference <0.003 m, fine/continuum
difference <0.004 m, positive extension >0.004 m, and stiff extension <30% of the
soft result. The observed coarse/fine difference was 0.000304855 m. These bounds
and two resolutions establish a limited consistency check; they do not establish
general cloth or garment convergence. Production shirts, curved/gathered seams,
multiple garments, large initial intersections, realistic bending, performance
and calibrated fabric response remain required benchmarks.

## Garment assembly experiment

Two separate authored fixtures are retained. The original was not replaced to
make a failing result disappear:

| Fixture | SHA-256 of native project | Purpose |
| --- | --- | --- |
| `fixtures/original-shirt.weftforge` | `34bbc74a5ff78265fd503c2b586db859a56b9e985295f399673342aa304cb060` | Original two-panel tight shirt, including reproducible fitting failures |
| `fixtures/relaxed-shirt.weftforge` | `5daf847ef95a44c1f8f2924661fdb5e8aa790a983daceb6dc2964604ba839820` | Separate looser body; hem X=±0.33 m and underarm X=±0.31 m, unchanged shoulders/cuffs/neck |

Both use original editable polygons, eight interval seams, 0.12 m requested mesh
resolution, front/back arrangement Z=±0.22 m, the same uncalibrated material and
analytic mannequin. The original body's two 0.48 m widths sum to 0.96 m, whereas
the mannequin torso's 0.18 m radius has 1.131 m circumference. A horizontal body
cross-section would therefore require about 17.8% circumferential extension;
redistribution and diagonal drape can change the actual local strain. This
geometric mismatch is a design issue separate from solver convergence.

Before evaluating the fitting matrix, the bounded acceptance criteria were set
to finite state, completed assembly, max material-axis extension <15%, max seam
gap <5 mm and max supported contact penetration <2 mm. These are original fixture
engineering criteria, not measured reference tolerances. They do not evaluate
appearance, full collision freedom, every material component or general fitting.

The repeatable explicit protocol is 60 frames (one simulated second) of
**user-directed zero-gravity assembly**, followed by 120 frames (two simulated
seconds) with gravity (0,-9.81,0). Both phases use dt=1/60 s, eight substeps,
32 iterations and sewing speed 1 m/s. The solver never changes gravity silently.

```sh
cargo run --release -p weftforge-cli -- fit-benchmark fixtures/relaxed-shirt.weftforge
cargo run --release -p weftforge-cli -- fit fixtures/relaxed-shirt.weftforge target/relaxed-fit.obj
cargo run --release -p weftforge-cli -- fit-benchmark fixtures/original-shirt.weftforge
```

`fit-benchmark` prints JSON containing the declared settings, per-phase timings,
requested/completed step counts, actual diagnostics and fixed quality criteria.
A quality failure still prints the report and exits nonzero. `fit` uses the same
protocol and writes the actual simulated OBJ only after the criteria pass; failed
quality preserves any prior output. Neither command claims reference parity.

Observed release results on Apple M3 Max, 48 GiB RAM, macOS 27.2 arm64,
Rust 1.94.1, 2026-11-13:

| Explicit protocol | Vertices / triangles | Max axis extension | Seam gap | Contact penetration | Criterion |
| --- | --- | --- | --- | --- | --- |
| Original tight shirt | 152 / 232 | 23.4385% | 0.0000001973 m | <1e-15 m | **Failed: strain** |
| Separate relaxed shirt | 158 / 244 | 8.99935% | 0.00000001390 m | <1e-15 m | Passed for this fixture and protocol |

The relaxed CLI run completed all 180 frames in 24.0133 wall seconds: assembly
8.0244 s, drape 15.9889 s, median step 132.942 ms, p95 step 141.011 ms and
0.12493 simulated seconds per wall second. This quality configuration is slower
than real time on the measured host. Its final supported initial-intersection
count was zero, assembly target zero and kinetic energy 1.091e-7 J. The exported
OBJ's SHA-256 was
`b9d2c524a52f8a4e363c25917071511a015fc88a94ad94069bb02c8a726e9294`.
CLI export success itself is not independent-consumer evidence. Viewport frame
time, peak memory and isolated-machine performance were not measured here.

Default simultaneous assembly and gravity is a different, poorer workflow. A
120-frame original-shirt run completed in 1.8780 wall seconds at 1.06495 simulated
seconds per wall second (median 15.515 ms, p95 17.009 ms), but had **120.515% axis
extension and 12.769 mm seam gap**. It failed fitting quality. On the relaxed
fixture, simultaneous default assembly/drape yielded 212.889% axis extension,
31.286 mm seam gap and 3.955 mm contact penetration. Explicit zero-gravity assembly
at the lower default four-substep/eight-iteration quality reduced that fixture's
extension to 31.024%, still a failure. A finite scene or faster throughput must
not be presented as acceptable fitting.

A separate original-shirt refinement matrix retained simultaneous gravity to
make the numerical/workflow limitation visible. Every row ran 120 frames at
1/60 s. The requested resolution was changed through `Command::SetResolution`;
all other pattern dimensions, material and arrangement stayed fixed.

| Resolution | Substeps / iterations | Sewing speed | Vertices / triangles | Max extension | Seam gap | Contact penetration | Criterion |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0.12 m | 4 / 8 | 0.5 m/s | 152 / 232 | 120.515% | 12.769 mm | 0 mm | Failed |
| 0.12 m | 8 / 32 | 0.5 m/s | 152 / 232 | 111.704% | 4.444 mm | 0.196 mm | Failed |
| 0.12 m | 8 / 32 | 1 m/s | 152 / 232 | 90.500% | 2.870 mm | 1.226 mm | Failed |
| 0.07 m | 8 / 32 | 0.5 m/s | 410 / 715 | 182.844% | 21.291 mm | 0 mm | Failed |
| 0.07 m | 8 / 32 | 1 m/s | 410 / 715 | 117.146% | 6.505 mm | 0.063 mm | Failed |

Refinement did not produce convergent garment behavior in this matrix. With slow
closure, the unsewn shirt also fell below its intended shoulder arrangement. The
zero-gravity assembly protocol and relaxed design provide one bounded successful
case, while these failures remain evidence requiring investigation. Refinement
of the accepted relaxed protocol, calibrated fabrics, persistent realistic
layered outfits and general garment robustness remain open.

## Bounded weak-warp refinement, 2026-11-14

The [retained refinement analysis](../parity/reports/WF-20261114-PHYSICS-REFINEMENT-001.json)
uses historical source `16e680ad` and the unchanged original grain-shirt fixture.
It reproduces the 26.9303% strain failure at three simulated seconds. The peak is
extension along the 50 N/m warp near arm/neck contact. Strain already reaches
25.9692% after the first second of zero-gravity assembly, so gravitational settling
alone cannot explain the result.

Three continuations start from the same three-second positions and velocities.
At four seconds, 8 substeps/32 iterations gives 26.7718% maximum axis strain;
8/64 gives 26.6893%, and 16/32 gives 26.2492%. All remain above the unchanged 15%
screen. The refined continuations reduce seam residual from about 5.5 micrometres
to about 10 nanometres, while moving nodes up to 2.11/3.90 mm relative to the
same-time control. Smaller seam error therefore does not establish a well-fitted
or converged garment.

The numerical sensitivity triggers were declared separately from fitting limits.
No material, fixture, solver, tolerance or default was changed to obtain these
results. The experiment refines continuation only; static force balance, the full
assembly path, mesh convergence, calibrated fabric behavior and reference
comparison remain unmeasured. Issue 16 stays failed. The exact API driver, inputs,
stored states and read-only verification recipe are retained with the report.
