# CPU cloth model

This is WeftForge's independently authored CPU solver. All checks below are
implementation-only. No reference drape,
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

The engineering choice is a deterministic sequential CPU path. Compliant sewing,
membrane and bending use the incremental-energy updates described below.
Hard sewing retains the scalar projection from
[Macklin, Müller and Chentanez, XPBD (2016)](https://matthias-research.github.io/pages/publications/XPBD.pdf).
The small dependency surface and explicit physical energy coefficients make it
suitable for repeatable checks and future accelerated-path comparisons. It is not
a promise of time-step, mesh or iteration independence at finite convergence.

One public step of duration dt is subdivided into h = dt/substeps. For free nodes:

```text
v* = exp(-damping*h) * (v + h*gravity)
x* = x + h*v*
```

The solver clears material, bending, soft-sewing and ordinary contact displacement
histories and hard-seam multipliers at each substep, then iterates membrane, bending, sewing,
analytic contact, cloth contact and pins in that order. The scalar radial projection for hard sewing
uses zero compliance in the following formula:

```text
a = 0
delta_lambda = (-C - a*lambda) / (sum_i(w_i*|gradient_i(C)|²) + a)
x_i += w_i*gradient_i(C)*delta_lambda
lambda += delta_lambda
```

The first substep can also apply the separately attributed initial-overlap repairs
described below. These repairs use the existing iteration budget. The new velocity
is the entire solved displacement divided by h, including any initial repair.
Damping is exponential
velocity damping in s^-1, rather than a frame-dependent interpolation. Its
operator splitting introduces a finite-step error in forced trajectories. The
zero-damping gravity test has the expected symplectic Euler error. There is no
restitution, air drag, wind, calibrated viscoelasticity or fully coupled implicit solve.

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

### Global material projection

Warp, weft and shear share triangle positions, and adjacent triangles share nodal
inertia. A scalar multiplier history alone does not retain the nonlinear inertial
relation when gradients rotate. The original XPBD derivation explicitly omits
that primal residual;
[Chen et al. (2024)](https://physbam.stanford.edu/papers/stanford2024-02_2.pdf)
also discuss its effect on backward-Euler force balance. This material stage is
derived directly from this crate's existing energy. It is not a calibrated
implementation or a convergence claim for either paper's full method.

One actual material displacement p per vertex resets to zero each substep.
During a material sweep its anchor z=x-p stays fixed; displacements from the
other stages move x without changing p, so the next material anchor follows them.
All triangles enter one physical incremental objective, with each triangle's
unchanged energy once and each free vertex's physical lumped inertia once:

```text
Phi(q) = sum_free_i m_i*|q_i-z_i|²/(2*h²) + sum_triangles sum_a k_a*C_a(q)²/2
```

Pinned coordinates stay fixed. There are no material pairs or separate group
anchors. Bending and compliant sewing use their separate incremental-energy
updates below; contact uses its own displacement histories and pins remain exact.
Each outer iteration makes one global Gauss–Newton material update before those
stages, in the same order. The garment acceptance settings
remain dt=1/60 s, eight substeps and 32 outer iterations; the standalone DTO
defaults above are unchanged.

Let J contain every triangle's three existing mode gradients, K the diagonal of
its area-scaled stiffnesses and W the inverse nodal masses, zero at pins. Using
mass-weighted displacement coordinates gives:

```text
B = h*sqrt(K)*J*sqrt(W)
r = h*sqrt(K)*C
A = I + B^T*B
b = -p/sqrt(W) - B^T*r
A*u = b
d = sqrt(W)*u
```

The inertia term in b is zero at pins. Matrix products visit the cached sparse
three-node rows of B and the identity; no dense global matrix is stored. A
symmetric diagonal preconditioner uses S_ii=1/sqrt(A_ii), solving S*A*S and
recovering u=S*v. Repeated `hypot` computes sqrt(A_ii) without first squaring
large coefficients. The preconditioned right-hand side is divided by its largest
absolute component before conjugate gradients and restored afterward. This
normalization does not change the objective or direction. An exactly zero
right-hand side leaves positions and p unchanged, including an all-pinned mesh.

The linear solve starts at zero, takes at most 32 conjugate-gradient iterations,
and stops when the recursive residual norm is at most 1e-12 of its initial norm
(squared comparison 1e-24), with no absolute tolerance floor. Reaching the cap
uses the finite approximate direction and still requires descent. This internal
linear work is additional to the public outer-iteration count; 32 outer iterations
do not mean only 32 local projections. Finite convergence and the recursive
residual do not certify a physical force tolerance on an arbitrary mesh.

The same complete objective h²*Phi governs Armijo descent, with coefficient
1e-4 and at most 16 fractions starting at one and halving. Each trial evaluates
all triangle energies and one inertia per free vertex. Only an accepted finite
trial changes x and p; p accumulates the actual rounded position displacement.
An exhausted search or non-descent direction leaves that sweep unchanged.
Current non-finite coefficients, diagonal scales, objective, linear curvature,
iterates or reconstructed directions abort with `Unstable`. Non-finite trial
energies are rejected during backtracking. Positive linear curvature is required.
Products, accumulated dot products and energies must fit finite f64; diagonal
scaling and RHS normalization reduce overflow exposure but do not supply arbitrary
range arithmetic. Underflow, rounding, extreme compression and ill conditioning
can still stall. Existing strain/contact/finite diagnostics remain necessary.

For N vertices and T triangles, each sweep stores 3*T sparse rows and O(N+T)
workspace, plus the retained three-component p per vertex. Its fixed budget is
at most 32 matrix products and 17 complete objective scans (current plus trials),
with at most 168*N + 119*T + 48 cancellation checkpoints over all vertex,
row, CG-iteration and backtracking scans. Each unit has bounded scalar work;
vector allocations/copies add O(N+T) work. Every input vector is finite in length;
the solver introduces no smaller geometry/input limit. A complete step runs at
most substeps*iterations such sweeps under the existing 1..=1000 settings limits.
Rows, vertices, every CG iteration and every trial visit these checkpoints. The
material stage polls the original cancellation callback at entry, at every 64th
checkpoint, and immediately before each successful return or accepted x/p copy.
At most 63 checkpoints can skip the callback between polls; the callback budget
is at most 2 + floor((168*N + 119*T + 48)/64) per sweep. Vector allocation and
copying remain uninterruptible, so this is not a wall-time latency guarantee.
A cancelled direct material stage returns before changing x or p. Candidate
arrays and p stay local to the whole-call transaction, so cancellation or numerical
failure preserves accepted positions, velocities, seam targets, time and
diagnostics. These bounds do not assert a wall-time or aggregate-memory limit.

The global objective resolves material coupling across the entire mesh in each
direction. It does not establish convergence of the separately projected bending,
sewing, friction or contact equations, nor general garment equilibrium.

The focused pinned/free-node regression compares the projected positions with
an independent dense Newton solution of the inertial-plus-material energy and
evaluates its force balance from a direct deformation gradient, with bounds of
1e-8 m and 1e-6 N. A second fixed regression uses two triangles sharing two free
edge nodes and independently derived UV-area masses. Its six-DOF energy reference
has a positive Hessian bound on an inertia ball containing every minimizer; an
independent cubic reduction and high-precision/finite-difference checks support
the same predeclared position/force bounds. Mixed anisotropy/grain, fixed pins,
once-only face coverage and whole-call cancellation remain separate checks.
These are small incremental-energy component checks, not garment answers.
A frozen eight-face component retains exact original masses, material data and
saved predictor positions at two free nodes. Its test invokes this production
global stage for 32 sweeps and checks the same predeclared 1e-8 m position and
1e-6 N independently differenced physical-force bounds. Its 80/100-digit
reference establishes a strict local minimum near the expected point, without
global uniqueness. Obsolete pair/order guards are replaced with exact face-order
and free/pinned-partition guards; the physical data and oracle stay unchanged.
Material convergence at finite budgets, the separate bending/sewing updates,
friction, and complete coupled-contact equilibrium remain separate
questions. Earlier garment results below retain their original source
and algorithm attribution; this source change does not regrade them.

`max_stretch_error` reports max(abs(length(f_w)-1), abs(length(f_f)-1)). It is a
material-axis length diagnostic, not a bound on all principal strains or shear.
`peak_strain` identifies the current solver triangle, `warp` or `weft`, and
signed `length(f_axis)-1`; positive means extension, negative compression.
Exact ties choose the first input triangle and then warp. The triangle index
belongs to the current rest mesh, while the shared core bridge resolves its
authored pattern ID. Default/legacy diagnostics can have no attribution. The
native inspector shows the pattern, direction and signed percentage; CLI reports
include the same typed metric and pattern ID/name. This adds no physical force
and changes no fitting threshold. See the [bounded contract](../spec/features/SIM-005-strain-attribution.md).

For reproducible f64 reporting, diagnostics form both current F columns from
relative current edges using the rest-UV difference-of-products and division,
then rotate those columns to the stored grain angle without snapping it.
Each squared norm uses Neumaier compensation over the three rounded component
squares, then square root and subtraction of one. The maximum, signed value,
triangle and axis come from the same candidate across every triangle and both
axes. Non-finite candidates invalidate the diagnostic; squared-sum overflow stays
non-finite. Constraint projection retains its own cached material weights and
ordinary dot products. Nearly equal real strains can still select different
triangles under other f64 conventions; this deterministic reporting rule supplies
no unique real-arithmetic winner, physical acceptance or reference-parity claim.

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

The bending stage retains one actual displacement `p_b` per node, reset to zero
once every substep. This is a displacement, not a force or scalar multiplier.
Its anchor `z=x-p_b` stays fixed during the stage's update and follows
displacements from other stages. All active hinges share the same inertial
term in the incremental objective:

```text
h²*Phi_b(q) = sum_free_i m_i*|q_i-z_i|²/2 + h²*sum_b k_b*C_b(q)²/2
```

Writing `a_i=p_b_i/sqrt(w_i)`, each row
`J_bi=h*sqrt(k_b*w_i)*gradient_i(C_b)` and residual
`r_b=h*sqrt(k_b)*C_b`, the mass-normalized Gauss–Newton system is:

```text
(I + J^T J) y = -(a + J^T r)
d_i = sqrt(w_i)*y_i
```

Pinned nodes have zero `a_i`, row entries and displacement. Shared-node rows
are summed in one matrix-free solve, using diagonal preconditioning, at most
32 PCG iterations and relative residual `1e-12`. This replaces separately
iterated hinge objectives whose shared-node coupling can remain unresolved
after 32 outer sweeps. It changes neither rigidity nor the angular energy.
These histories remain separate from the material and sewing histories; they
are not extra physical masses. Finite-budget convergence of the combined
stages is not established.

The complete block objective accepts an Armijo descent with coefficient `1e-4`
and at most 16 fractions, starting at one and halving. Current degenerate
hinges are omitted from that invocation. Any trial degenerating an active
hinge is rejected for the whole block, as is a non-finite trial. Non-finite
current quantities abort; an exhausted search leaves the stage unchanged.
Only an accepted finite trial changes positions and history, using actual
rounded displacement. The original cancellation callback is checked on entry,
at most every 64 bounded internal checkpoints and immediately before every
successful return or accepted position/history copy. No cancellation result
is returned after that copy. All history and scratch storage is local to the
candidate substep/call, preserving the whole-Cloth transaction.

An independent pinned-hinge check uses one free 0.01 kg node, a 0.1 s substep,
0.5 N m bend coefficient and a predictor at radius 1 m and angle 1 radian.
Eliminating radius gives `r=cos(theta-1)` and the strictly increasing equation
`0.5*theta+0.5*sin(2*(theta-1))=0` on `[0,1]`. Its unique minimum supplies a
64-bisection position oracle; direct polar energy derivatives supply force
residuals without the production dihedral gradient. The fixed bounds are
`1e-8 m` and `1e-6 N`. At 32 sweeps the old scalar history misses by
0.1185816818 m / 0.1065595925 N; displacement history gives
3.835502563e-9 m / 3.349162043e-9 N. Those numbers describe the earlier
single-hinge implementation, not a run of the coupled stage. Its checks now
call the shared production stage while retaining their independent criteria.

A fresh five-node/two-hinge case has one free node, mass
`6.627272727272728e-5 kg`, substep `1/480 s`, two coefficients
`0.024750031175957105 N m`, and a `1e-5 m` normal predictor. Independent
Cartesian derivatives and a strict-convexity proof give the unique minimum
`[0,0,5.1358750877942195e-8] m`. At 32 ordered local-hinge sweeps, the
compiled RED misses by `2.661405569226554e-8 m` with force residual
`7.912503711575184e-5 N`, exceeding the fixed `1e-8 m` / `1e-6 N` bounds.
On coupled-stage commit `9643ad0a430380f85b7c42f915c5ead3352c98b8`, the one
filtered component check passes with position error `5.360130369343619e-22 m`
and independent Cartesian force residual `1.6534083130403943e-18 N`.
The reference-force norm is `2.710505431213761e-20 N`; pins and finiteness
pass after each of the unchanged 32 sweeps. The subsequent `82cc77ee` workspace
check passed 425 tests, including the four guards for shared history/pins and
cancellation, no-op/invalid-input atomicity, active-trial degeneration and
whole-call rollback. Formatting, strict Clippy and native/CLI builds also passed.
These results describe that separate source checkpoint; the integrated notch/v6
source still requires its own checks.

A fresh full original-shirt run on `82cc77ee` completed 180 steps and independently
analyzed all 181 saved states in 358.903 seconds within its predeclared 420-second
bound. At dt=1/60 s, eight substeps, 32 iterations and 1 m/s sewing closure, the
unchanged 15% strain screen fails at frames 28 and 29. The peak is 50.9601855%
warp extension at frame 28; the final maximum is 9.43082%. All finite-value,
supported-contact and uncertainty screens pass; seam error passes after exact
closure at frame 29. The final frame does not override trajectory failure.
This is a failed implementation-only garment result, not measured-fabric,
reference, complete-feature or garment-workflow acceptance. Earlier failed and
incomplete runs retain their original source attribution and grades.

## Sewing, pins and construction correspondence

A stitch constrains the distance between two convex weighted points. Each point
lies on a mesh segment through two vertex indices and weights summing to one.
The stitch uses compliance in m/N. Repeated indices are
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

For positive compliance c, all input soft stitches share one incremental-energy
block and one actual displacement history `p_s` per node. This history resets
once each substep and remains separate from the material/bending histories.
Its anchor is `z=x-p_s`. The unchanged energy counts every input spring,
including duplicate rows, and one nodal inertia term:

```text
h²*Phi_s(q) = sum_free_i |q_i-z_i|²/(2*w_i)
             + h²*sum_soft_s (|weighted_a(q)-weighted_b(q)|-target_s)²/(2*c_s)
```

Here `w_i=1/m_i`; pins have zero inverse mass and remain fixed. Repeated endpoint
indices merge signed coefficients for gradient scatter, without deleting,
deduplicating or renormalizing any physical spring/weight. Target zero is the
same vector energy, represented by three Cartesian residuals. Positive targets
retain the radial energy. With merged coefficients beta, mass-normalized rows
and residuals are:

```text
zero target: r_axis=(h/sqrt(c))*d_axis
             J_axis,i=(h/sqrt(c))*sqrt(w_i)*beta_i*e_axis
positive:    r=(h/sqrt(c))*(|d|-target)
             J_i=(h/sqrt(c))*sqrt(w_i)*beta_i*unit(d)
(I+J^T*J)u = -p_s/sqrt(W)-J^T*r
Delta_x_i = sqrt(w_i)*u_i
```

`p_s` is a displacement in metres, not a force or multiplier. Both inertia
`p_s/sqrt(w)` and each scaled spring residual have units m*sqrt(kg), so this
objective is h² times physical energy.

Exactly one soft spring uses the analytic proximal path in production. This is
a rank-one weighted difference, regardless of endpoint IDs or the number of
hard rows. Eliminating nodal inertia gives the unchanged local objective
`|D-d_z|²/(2*W)+(|D|-target)²/(2*alpha)`, with
`d_z=weighted_a(z)-weighted_b(z)`, `W=sum(w_i*beta_i²)` and `alpha=c/h²`.
For nonzero d_z its minimizing impulse is
`H=unit(d_z)*(target-|d_z|)/(alpha+W)`; at target0 it is
`H=-d_z/(alpha+W)`, preserving the original division/weighted-endpoint arithmetic.
At zero anchor the current direction is retained if defined, otherwise+X.
A private candidate starts at z and scatters `w_i*beta_i*H`; pins stay bit-exact.
Finite trials must not increase the same complete inertial/spring energy.
The actual nodal displacement is retained, not a reconstructed per-row multiplier.
This preserves the isolated analytic guards without normal-solver rounding.
Both duplicate inputs remain separate energies and therefore take the shared path.

For two or more soft springs, the block solves the equivalent row-space
Woodbury system. Here J is the physical scaled residual Jacobian,
W is the inverse nodal mass (zero at pins), and p is the retained nodal history:

```text
(I + J*W*J^T) eta = J*p_free - r
delta_free = -p_free + W*J^T*eta
delta_pin = 0
p_new = (1-alpha)*p + alpha*W*J^T*eta   # free nodes, exact arithmetic
```

The old history appears in both RHS and recovery. A zero dual RHS does not
mean no movement: eta=0 still requires delta=-p. This distinction is tested
with a nonzero old history and a zero-energy manufactured minimum.
Each finite dual iterate produces an actual weighted row scatter. With
balanced endpoint weights this retains equal/opposite internal actions;
more generally it respects the actual coefficients rather than recentering COM
or silently normalizing accepted weights. Floating position rounding remains.

With B=J*sqrt(W), a=p_free/sqrt(W), the row diagonal is
D_j=sqrt(1+sum_i |B_ji|^2), accumulated using hypot. Diagonal PCG solves
D^-1*(I+B*B^T)*D^-1 with at most32 iterations and relative residual1e-12.
The operator scatters B^T into nodal workspace then gathers B back to rows;
every duplicate physical row remains present. The true primal gradient
`a+B^T*r` dotted with the recovered mass-normalized direction controls descent.
The unchanged full radial/vector objective controls Armijo1e-4 with at most16
fractions. Only finite accepted trials copy x/p, retaining actual rounded delta.
A zero/non-descent direction or exhausted search leaves the stage unchanged.
The singleton, material and bending numerical paths are unchanged.

This replaces finite diagonal-primal PCG for soft sewing: that approximation
can move outside the weighted row-action space before convergence. In a frozen
five-panel core integration test, first ground/capsule contact produces59.26%
compression; subsequent sewing stages amplify the coupled state, reaching798.7%
extension and the existing500% instability rejection. Temporary read hooks
reproduce the exact failure and show non-roundoff sewing-history action drift,
even though every sewing update decreases its own complete objective. This is
an integration failure, not a new accepted garment case. The row-space repair
addresses the demonstrated invariant gap; complete stability still needs the
unchanged integration test and broader checks.

In the shared block, positive zero-gap rows choose the nonzero weighted
inertial-anchor direction, or world+X when that anchor is also zero. Nonzero current directions first divide
components by their maximum absolute value before unit normalization, preserving
subnormal45-degree directions. This deterministic choice supplies a trial at the
radial cusp, not differentiability or a global-minimum guarantee. Non-finite
current quantities abort atomically; non-finite trials are rejected. An exhausted
search or non-descent direction leaves the stage unchanged. All-pinned or exact
zero-RHS stages preserve positions and displacement history; pin reactions remain
available from the physical spring energy without storing a fictitious pin history.

The sewing slot now applies the coupled soft block, then the original hard
projections in input order. This makes mixed hard/soft ordering explicit; it
changes their finite-iteration interaction. All-hard order/arithmetic, hard scalar
history remain unchanged; the subsequent contact treatment is described below.
No targets, closure speed, stiffness, time step, substeps or outer
iteration limits change. No fully coupled sewing/contact equilibrium is claimed.

For N vertices, S input stitches and R=3*S_zero+S_positive<=3*S soft rows,
workspace is O(N+R), with one retained Vec3 history per node. One sweep performs
at most32 matrix products and17 complete objective scans. Bounded internal
checkpoint count is at most71*N+166*R+19*S+48 (at most71*N+517*S+48).
The original cancellation callback is polled at entry, every64th checkpoint,
and immediately before each successful return or x/p_s copy: at most63 skipped
checkpoints and at most2+floor(bound/64) calls. Allocation/copying remain
uninterruptible, so this is no wall-time latency guarantee. There is no Cancelled
return after the accepted copy. Histories/workspaces remain local to each
whole-Cloth transaction; cancellation/errors preserve all accepted persistent fields.

The earlier local vector-history algorithm repaired rotating-direction local
stationarity, and its .01kg, h=.1s positive+zero two-spring component keeps its
independent1e-8m/1e-6N physical bounds. It did not resolve general stiff sharing.
A fresh two-free-node/two-duplicate-row component at masses
`4.5714285714285506e-5`/`4.571428571428556e-5 kg`, h=`1/480 s`, c=`1e-8 m/N`
and positive target `0.01333333333333439 m` has a unique independent Cartesian
minimum. At32 old local sweeps, compiled source `d8bcb3b7` passes position with
`2.1942782660322457e-10 m` error but fails full force balance with
`0.08777113262758984 N` residual. Both physical spring energies remain in the
new block and the independent oracle/32/1e-8m/1e-6N/COM-finite checks stay fixed.
The primal component passed the same fixed criteria, but its subsequent full
source checks stopped at the separate integration failure above; remaining
stages were not run. The dual candidate is unexecuted. A new independent guard
uses a six-node/two-row weighted network with a nonconstant kernel vector,
nonzero old row-scattered history, free/pinned partitions, and analytically
manufactured strictly convex zero-target minima. It checks nonconstant-kernel
action orthogonality rather than just COM, exact pins, finite/history transport,
and1e-12 position/force bounds. A second configuration has exactly zero dual
RHS but nonzero p. These are source/component facts, not a garment-spike causal
verdict, garment acceptance or reference parity.

This explicitly models assembly, separate from fabric stiffness. A zero target
requests coincidence; finite compliance can leave a gap. The default speed is
0.5 m/s; zero pauses the target reduction while allowing cloth dynamics.
`seam_assembly_remaining` reports
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
particle/triangle path. Each projection pass builds the existing triangle
hierarchy from the same swept, thickness-expanded AABBs. A swept particle query
collects candidates, filters the exact AABB predicate and sorts triangle indices
into the original projection order.

If a contact moves the queried particle, the remaining triangle indices use the
original linear scan: that correction can reach a later bound outside the first
query. Bounds retain their original per-pass snapshot even when triangle nodes
move. Static diagnostics use the hierarchy without changing positions; tests
retain a separate exhaustive search oracle. Dense overlap, sorting and the
linear fallback can still be expensive. This optimization adds no collision law,
contact toggle or continuous-collision completeness. See
[SIM-004](../spec/features/SIM-004-contact-candidates.md).

Ordinary contact retains each support's actual nodal displacement p for the current
substep. At a support visit its anchor is z_i=x_i-p_i for free nodes; pinned
coordinates stay fixed. Ground and particle/triangle geometry is queried at that
anchor. Capsules use the current-side supporting plane described below. For unit
normal n, the selected stencil's signed gap c_a at the anchor, coefficients a_i
and inverse masses w_i:

```text
D = sum_i w_i*a_i²
lambda = max(0, -c_a/D)
t = tangential_part(sum_i a_i*(z_i-previous_i))
tau = min(mu*lambda, |t|/D)
q_i = z_i + w_i*a_i*(lambda*n - tau*t/|t|)
p_i = q_i-z_i
```

The tangential term is zero at negligible |t|; an immovable support applies no
motion and keeps its reported conflict. This replaces the support's old action
rather than accumulating a positive-only correction. An inactive stencil releases
both normal and friction action; exactly zero actions are removed. For capsules,
this means separation from the selected current-side halfspace. The multiplier
has units kg m, not momentum or force. A force interpretation additionally depends
on the substep's h² scaling and model. Previously active particle/triangle pairs
are revisited even when their current bounds leave the hierarchy query. Unusable
active geometry returns an error and preserves the last accepted whole-step state.

Particle/triangle closest points include face, edge and vertex regions. Their
coefficients are [1, -beta_0, -beta_1, -beta_2], transferring motion between layers.
Mesh one-ring
neighbors and directly stitched endpoint neighborhoods are excluded because
contact between adjacent elements would oppose their shared surface or seam.

An additional linearized swept face test preserves the previous side when a
particle endpoint has crossed a triangle interior. This is **not full continuous
collision detection**: deforming triangle trajectories, tangential crossings and
edge/edge intersections can be missed. Very thin, fast or initially entangled
scenes can interpenetrate. The ordinary particle/triangle path uses one closest
feature per pair and can stall at an infeasible fixed point even for an isolated
moving triangle. Its action bookkeeping does not establish nonlinear convergence;
a controlled nonlinear contact block remains open. Initial intersections are
counted where this model detects them; the constructor permits such arrangements
and freezes the recovery eligibility described below. An undetected edge/edge crossing
cannot be interpreted as an intersection-free scene.

Ground is an exact horizontal offset plane: a linear triangle's minimum Y is at
a vertex. Capsules test their vertices and additionally test
every complete cloth triangle against the closed capsule axis segment. The
closest-feature search covers edges, axis endpoints projected onto faces, and
face/axis crossings, including parallel and coplanar axes; a zero-length capsule
is a sphere. It uses f64 in a scaled local frame. The excluded radius is the
capsule radius plus the maximum existing incident vertex physical offset on the
triangle. This conservative material-boundary policy agrees with a uniform
offset; rendered thickness does not enter contact.

Vertex/capsule and retained material-point/capsule visits choose a unit outward
normal at the **current** point's closest axis feature. They evaluate that
normal's supporting halfspace at the retracted anchor; they do not choose a new
radial projection branch from the anchor. For capsule endpoints A, B and excluded
radius R (including the existing physical offset), the translated gap is:

```text
current_point = sum_i a_i*x_i
anchor_point = sum_i a_i*z_i
c_a = dot(n, anchor_point-A) - max(0, dot(n, B-A)) - R
```

The full-axis support value matters even when the current point is exactly on
an endpoint and a deterministic fallback supplies n. For a movable resolved
support, the resulting plane excludes the complete convex capsule. Finite
friction stays in that plane. Point features scale both the axis and offset
before the projection dot product, preserving tiny nonzero axes without an
absolute length cutoff. Unusable axis, offset or distance arithmetic returns a
simulation error before it can be mistaken for separation.

This is a local current-side contact law. A same-side separated anchor releases
its action. An anchor on the body's opposite side can retain a reaction on the
current side; this deliberately does not implement a global nearest exterior
projection. Branch selection, complete continuous collision detection and
general nonlinear convergence remain open. A single finite friction visit can
have positive curved-surface clearance with a nonzero reaction. Current-normal
complementarity and Coulomb balance follow at a converged fixed point, rather
than from a penetration-only diagnostic or an arbitrary finite iteration count.

The two-mass regression compares the free sewing block with its exact quadratic
minimum, then compares 32 unchanged sewing/contact passes with an independently
bracketed constrained-energy minimum on a capsule hemisphere. It checks final
force balance as well as position, history, feasibility and complementarity.
Separate vertex and pinned barycentric tests retain the first-visit plane
friction oracle and check progress toward a known sphere Coulomb fixed point
after 32 visits. These component checks do not establish garment equilibrium,
calibration or reference parity.

Ordinary triangle/capsule contact keeps a family of fixed material points beta
per pair. Each owns three nodal actions and uses a_i=beta_i in the update above,
with barycentrically interpolated physical friction. Each pass revisits retained
supports in insertion order and then discovers at most one new current closest
point, or revisits its exact existing beta. Zero-action supports are removed.
Thus I passes retain O(I) supports and make O(I²) support visits per pair in the
worst case. This avoids discarding another material point's action when the closest
feature changes, but does not prove whole-triangle feasibility or global equilibrium
at a finite iteration count. The capsule phase scans all triangle/capsule pairs
without a broad phase.

Skinny faces (cross-product magnitude at most 1e-7 times the longest squared
edge length) use a conservative lower distance bound: closed-boundary distance
minus the face altitude to its longest edge, including a rounding allowance.
Arithmetic underflow retains the trivial conservative lower distance bound zero.
Confidently collinear/point triangles reduce to their closed edges/vertices.
`capsule_surface_uncertain_pairs` counts current triangle/capsule pairs using
these conservative uncertain-geometry bounds, including separated pairs; exact
closed-edge degeneracy is not counted. Such bounds can retain a small false
positive penetration, and missing stable face arithmetic cannot certify clearance.
The native inspector displays this count. CLI fitting conservatively requires
zero uncertain pairs in addition to the existing strain, seam and penetration
bounds, including when a conservative pair bound reports no penetration.

At zero distance, a previously axis-separated triangle prefers the previous
same-barycentric point's separation. If the previous triangle already crosses
the axis, that point has no consistent previous side as closest features change;
recovery instead anchors to the previous face/axis geometry for the substep.
The face winding chooses its normal when usable. An interior-axis fallback has
a radial component; an unusable face normal falls back to the cross product of
the axis and its least-aligned coordinate basis (ties X, Y, Z). A point axis with
no usable direction falls back to +X. These finite deterministic side/tie rules
are engineering recovery, not continuum contact-normal or convergence proofs.
Pins and finite iteration budgets can leave explicitly reported penetration.
Capsule surface contact is static: between-step tunnelling remains possible.

The original torso collider remains a vertex-tested ellipsoid with inflated
radii and radial projection, an approximation to a constant signed-distance
offset. Away from principal axes its radial direction is not the ellipsoid's
surface normal; the associated friction step can re-enter even that approximate
surface. This separate unresolved limit is not repaired by the capsule law.
The current core mannequin adapter constructs Ground and Capsules. No
proprietary avatar is bundled. Arbitrary triangle-mesh avatars and animated or
moving collider trajectories are not supported by this crate yet. Collider
contact friction uses the cloth's physical friction coefficient.

Because contacts are sequential and pins are authoritative, the final projection
can leave a previously resolved contact slightly penetrated or conflict with a
pin. Construction, reset and final `max_contact_error` evaluate current full
static triangle/capsule occupancy as well as all existing vertex collider and
particle/triangle residuals. `initial_intersections` counts those supported
vertex/collider, triangle/capsule and nonexcluded particle/triangle pairs with
depth above 1e-8 m, including conservative uncertain pairs; overlapping vertex
and surface coverage is counted separately. `contacts` counts projections
including repeated iterations, not unique colliding pairs. Historical zero
vertex-only capsule residuals retain their original narrower scope. Ellipsoid
surface coverage, self edge/edge contact and continuous collision remain open.
An exact zero residual in a small fixture does not establish production
collision robustness.

### Initial-overlap repair

Construction and reset freeze the specific nonexcluded particle/triangle and
triangle/capsule pairs whose supported static depth exceeds 1e-8 m, using the
same completed analysis as `initial_intersections`. Eligibility is per pair;
ground and vertex/collider contacts remain ordinary balanced contacts. New pairs
created by later motion are not eligible.

Only the first substep at simulated time zero uses the former positive-only
geometric normal/friction correction for those eligible nonlinear pairs. It adds
no passes or iterations. This is a kinematic initial-placement repair, not an
equilibrium reaction: cumulative actual nodal motion is attributed separately and
ordinary balanced contact does not retract it. Later substeps use ordinary contact.
The usual velocity update includes repair motion, so it can inject kinetic energy.
Pins stay exact, and immovable or uncertain conflicts remain visible.

`initial_contact_recovery_projections` counts repair projections that actually
moved nodes, including repeated visits, since construction/reset. It is zero
initially, persists through later accepted calls, and reset clears and rearms it.
Cancellation or failure preserves both its prior value and the recovery phase,
positions, velocities and provenance. Legacy diagnostic JSON defaults the absent
field to zero for decoding compatibility; absence does not prove that an older
execution used no initial repair. Fitting requires a present zero count at the
initial state and every accepted whole-step boundary; repairs cannot qualify a
garment quality pass.

## Failure, cancellation and recovery

`Cloth::new_cancellable` builds a complete candidate or returns `Cancelled`;
`Cloth::new` delegates with a non-cancelling callback. Construction polls within
input validation, constraint/mass/seam setup, contact bounds and initial
intersection analysis. Contact analysis polls per vertex and hierarchy node,
with bounded intervals during bounds construction, partitioning and collider
scans. Rejected candidates still participate in cancellation. Its initial count and
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
substeps, iterations, hierarchy work, bounded intervals in vertex/capsule surface
work, every 32 particles of the cloth contact loop, and inside final contact
analysis. Poll counts are implementation details. A failed or
cancelled call preserves positions, velocities, time, diagnostics and initial
recovery provenance from the
last successful call. The caller receives the explicit error. Non-finite state or
derived diagnostics, displacement above 100 m in one substep, or material-axis
extension above 500% rejects the complete call as unstable. These are runaway
guards, not a complete inversion or energy-stability detector. Large finite seam
or contact residuals are returned without disguising them as successful fitting.
Reset remains available after errors and uses non-cancelling contact analysis.
Final linear strain/energy aggregation retains before/after cancellation checks;
latency in that phase and direct reset depends on mesh size. Background worker
ownership is required by the application.

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

The static capsule surface increment adds analytic cap/face/edge/vertex cases,
parallel and coplanar segments, spheres, tiny/degenerate faces, rigid transforms,
and central-difference barycentric gradients (error below 1e-8). Saved original
triangle140 and Harbor A1164/B678 witness gaps reproduce within the prospectively
declared 1e-10 m tolerance; all three saved centroid occupancies are also retained
in the primitive tests. Response checks cover unequal inverse masses, physical
friction, pinned conflicts, sequential capsules undoing prior separation, initial/
reset/final residuals, visible conservative uncertainty, and cancellation during
projection and final analysis preserving every accepted state field. The original
30-iteration capsule exterior test keeps its original <1e-6 m requirement. These
checks establish the bounded implementation contract, not garment clearance,
calibration, convergence, continuous contact or reference parity.

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
the mannequin torso's bare 0.18 m radius has 1.131 m circumference. Including the
uniform 0.003 m physical offset gives radius0.183 m and circumference1.149823 m,
which would require about19.77% circumferential extension for that horizontal
cross-section. Redistribution and diagonal drape can change the actual local
strain; this conditional bound does not explain the measured strain by itself.
Harbor has different dimensions and must be assessed separately. Coarse cuff
edges can also make an exterior wrap geometrically infeasible within the15%
material-axis screen. A low vertex residual cannot establish that such edges
stay outside the arm capsule. Keep geometric fit, contact coverage and numerical
convergence as separately measured questions.

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
A quality failure still prints the report and exits nonzero. Quality now covers
the initial state and every successfully completed whole-step boundary: finite
diagnostics, finite nonnegative assembly target, strain below 15%, penetration
below 2 mm, zero uncertain capsule pairs and zero initial-repair projections.
States with an exactly zero assembly target also require seam gap below 5 mm;
the final target must be zero. `quality_states_checked` includes the initial
state, and `first_quality_failure_state` identifies the first failed boundary
(zero is initial). A quality failure does not truncate the 180-step protocol;
a solver error stops it. Substep-internal quality is not certified. `fit` uses the same
protocol and writes the actual simulated OBJ only after the criteria pass; failed
quality preserves any prior output. Neither command claims reference parity.

The following historical runs used final-state CLI quality checks. They have not
been rerun or regraded against the trajectory and initial-repair requirements.
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

The separate [full-path experiment](../parity/reports/WF-20261114-PHYSICS-FULL-PATH-001.json)
starts each quality setting from the unchanged original arrangement at source
`e342471b`, rather than refining only the continuation. It retains all 39 sampled
position/velocity states and 540 timed steps. Independent geometric measurements
give:

| Full-path quality | Strain after 1 s assembly | Strain after 3 s total | Final screen |
| --- | --- | --- | --- |
| 8 substeps / 32 iterations | 25.969213% | 26.930346% | Failed |
| 8 / 64 | 26.126438% | 27.168563% | Failed |
| 16 / 32 | 26.546938% | 26.698742% | Failed |

Before gravity, 16/32 already differs from control by 0.005777 strain and up to
11.087 mm in position, exceeding both separately declared sensitivity triggers.
At three seconds the refined paths differ by up to 2.547 and 23.748 mm, while
their maximum-strain differences remain below 0.005. The control reproduces all
13 shared historical samples and the original native OBJ within 6.94e-18 m.
Both assembly and drape were refined, so the final differences do not isolate
assembly alone. These results establish path sensitivity, not static equilibrium,
convergence, calibration or a sufficient production fix. All original fitting
failures and the 15% screen remain unchanged; replay instructions accompany the
tracked original outputs.

The [current-source balance diagnosis](../parity/reports/WF-20261114-PHYSICS-BALANCE-001.json)
uses runtime `3eddef067c8a` and the unchanged original shirt. Its new mesh has
150 vertices / 228 triangles, compared with the historical 158 / 244; all 28
stitch mappings are unchanged. Nodal arrays across those different topologies
are not compared as equivalent. The original 8/32 control fails the same strain
screen at 24.1185% after assembly and 32.3519% after drape.

Two fixed causal probes clarify this failure without changing production inputs
or acceptance. Continuing the identical first 60 frames for 120 more zero-gravity
frames ends at 32.1699%. During exactly 2.5–3.0 s, strain rises from 30.7541% to
32.1699%, maximum node displacement is 3.62048 mm, and membrane-plus-bending
energy rises by 0.00383853 J. Low RMS speed does not establish equilibrium.
All 61 shared prefix states match exactly apart from wall-clock timing.

Separately omitting only mannequin capsules in memory gives 0.242573% strain at
1 s with the same cloth, ground and self-contact settings. That counterfactual
isolates avatar-loaded assembly as a contributor; it cannot count as a garment
quality pass. Neither initial prestrain nor incomplete seam closure explains the
original 1 s failure. Geometric impossibility and a solver defect remain unproven.
Independent conservative force gradients omit contact/friction reactions and
per-substep impulses, so full force balance, convergence and calibration remain
open. All three original runs, 423 saved states, 420 timings and pre-run plans
are retained with separate independent review and read-only replay.

## Static capsule surface checkpoint, 2026-11-15

Runtime `0770204a54ab` retains the static surface-contact model described above
and exposes conservative uncertainty in CLI fitting rejection and native
diagnostics. Independent source/static review and an actual unmodified Rust
detector run each cover83original static cases, including9conservative uncertain
cases. These checks establish their bounded static scope only.

The [closed original Harbor experiment](../parity/reports/WF-20261115-CAPSULE-SURFACE-001.json)
retains unchanged base revision20 and height revision17, with181saved states and
180steps each. Both use60zero-gravity assembly steps then120gravity steps,
dt1/60s,8substeps/32iterations and1m/s seam closure. All saved positions,
velocities, raw diagnostics/settings, rest metadata and0/60/180OBJ exports are
publicly hash-bound. No new physics or timed benchmark runs during publication.

Both cases pass independent full static triangle/capsule and supported contact
below2mm at all181saved boundaries. Maximum contact upper bounds are
4.552257086565259e-7m for base and3.7889283330261314e-7m for height, both at29.
All independent unresolved-pair and raw conservative-policy counts are zero;
initial-intersection counts are independently verified. Their original whole-case
times167.26820654096082s and162.04855137504637s include numerical analysis,
export/hash/inspection and remain below420s, with producerRSS and aggregate
numerical-output each separately below256MiB.

**Both overall strict results remain failed.** Base exceeds15% material-axis
strain at28–37, reaching19.90919497559327%at31, and has33 exact peak triangle/axis
attribution failures. Height passes all trajectory physical and completed-fit
screens, with14.611755162261497% maximum strain at30, but fails exact attribution
at6,10,18. Open seams/targets at0–28 describe initial arrangement; exact zero
targets begin at29. Final-only producer pass flags do not qualify delivery.
Every failure list and per-frame diagnostic disagreement remains in the packet.

The separate public saved-state replay changes only evaluator directory mapping;
all numerical functions, thresholds and cutoffs match the frozen original, and
every deterministic analysis field must match the original saved reports. The
original24-file evaluator freeze remains separately archived. Public replay
does not execute binaries or recertify historical clocks/resource counters.
Prior3aa strain/contact failures, native7fe Save18/raw metadata failures, full
garment acceptance, edge-edge/continuous contact, convergence, calibration and
reference parity remain open. No current native uncertainty-label observation
is supplied by these headless cases.
