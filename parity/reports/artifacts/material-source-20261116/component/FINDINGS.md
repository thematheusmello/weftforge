# Coupled material projection candidate for independent review

The candidate corrects a demonstrated incremental-energy force-balance error in
the scalar material projection. It does not establish garment convergence,
clearance, calibration or any roadmap acceptance. No old garment was rerun or
regraded, and no garment setting, geometry, material, contact, sewing speed or
15%/5mm/2mm quality screen was changed.

Source starts at `381c58c5ddf0eff5a587412a2656662cfc62877d` in the isolated
`feat/coupled-cloth-solve` worktree. Only sim constraints/lib/tests and
docs/physics.md change. No public command, DTO, dependency, GUI, registry, map,
README, workflow, Pages or publication change is included. All files here remain
private ignored review inputs; they are not delivered public evidence.

## Root cause and minimal correction

The old scalar updates track lambda while each nonlinear material gradient J
changes. Even solving all three lambda coordinates together can converge to the
wrong inertial/elastic balance because accumulated historical position changes
are not equal to W*J_current^T*lambda. The original
[XPBD derivation](https://matthias-research.github.io/pages/publications/XPBD.pdf)
explicitly identifies its omission of the primal residual;
[Chen et al.](https://physbam.stanford.edu/papers/stanford2024-02_2.pdf) discuss
residual stagnation from that approximation. This candidate is derived from the
existing energy, not asserted to implement either paper's complete method.

For each triangle retain its three actual positional corrections p, resetting
them at each substep. Other constraints may change x; local anchor z=x-p removes
this triangle's previous contribution. For this local projection minimize

`Phi(q) = (q-z)^T M (q-z)/(2h²) + sum_a(k_a*C_a(q)²)/2`.

One coupled Gauss–Newton step uses the same three existing material constraints.
Woodbury elimination of the positive mass term gives a three-by-three system:

`A = J W J^T + diag(1/(k_a*h²))`,
`A*eta = J*p-C`,
`d = -p + W*J^T*eta`.

This is a single material step per triangle per existing outer iteration. A
diagonally scaled Cholesky solve handles A. A bounded Armijo line search on the
exact local incremental energy uses 1e-4 sufficient decrease and up to 16 trial
fractions (1,1/2,...); those trials only evaluate energy. Accepted p updates use
the rounded displacement actually applied to positions. Unusable factorization
or exhausted backtracking leaves the projection unchanged. No hidden settings
search or extra material-iteration budget is used.

The scratch state increases from three to nine f64 values per triangle. Bending,
sewing, analytic contact, self/layer contact, friction and pins keep their prior
algorithms and order. Full coupled-contact equilibrium is not proved. Local
backtracking and approximate/nonconvex projections do not guarantee global
convergence at finite budgets; large-mesh performance is unmeasured.

## Independent equilibrium regression

One declared skinny triangle has UV (0,0),(.01,0),(.003,.07) m; vertices 0 and 2
are fixed, vertex 1 has inertial predicted position (.012,.002) m. Density is
.18 kg/m², warp/weft/shear stiffnesses 400/400/100 N/m, grain pi/2, h=(1/60)/8 s,
and 32 material iterations. Free mass is .18*.00035/3 kg.

The independent oracle forms F=Ds*inverse(Dm) directly and minimizes the physical
incremental energy with SciPy least_squares/complex-step derivatives, refined
by a dense root solve of its energy gradient. It uses no runtime helpers. Its
minimum is (0.010007132086944948,0.000027306978065735514) m, force norm below
1.5e-15 N; the local energy Hessian eigenvalues are about 358.059 and 1408.523 N/m.
The retained Rust regression independently evaluates the deformation-gradient
force balance instead of copying the solver's residual calculation.

Limits were fixed at position error <1e-8 m and stationarity <1e-6 N before
the algorithm comparison. Nothing was relaxed after either failure.

| Context | Position error | Force imbalance | Result |
| --- | ---: | ---: | --- |
| Unchanged production Rust | 2.843544361e-6 m | 1.763599204e-3 N | RED |
| Private plain block-lambda prototype | 3.693374855e-6 m | 1.834378430e-3 N | Rejected candidate |
| Private retained-correction prototype | 1.542040858e-12 m | 1.898195883e-9 N | Component prototype |
| First Rust candidate | 1.694065895e-20 m | 1.457316403e-15 N | GREEN |
| Final reviewed Rust candidate | 8.810015435e-16 m | 5.759318500e-13 N | GREEN |

The first private least_squares oracle stopped before the tighter oracle-only
1e-9 N force check. Its source and stop explanation are retained. A dense root
refinement supplies the high-precision oracle; no test tolerance, geometry,
physical coefficient or runtime iteration count changed.

The existing uniaxial single-step expectations remain exact. Its former scalar
shear-step expectation necessarily changes with the algorithm: the new expected
coupled step comes from independent exact Fraction Gaussian elimination of the
declared 3x3 Gram matrix. That rational result and matrix are retained separately.
It is not an adjusted garment answer or a weakened assertion.

## Cancellation and source identities

Root review requested a poll between material triangles because each now includes
a 3x3 solve and bounded backtracking. `check_cancel` now runs in that loop. The
existing rollback/recovery test uses a strained patch and cancels at the second
material triangle, after real projection work. It compares complete retained
Cloth state, then tests invalid settings, numerical failure and successful
recovery. Scratch corrections remain within the candidate step; no failed step
can install them or partially mutate accepted state. One local solve/backtracking
can finish before a cooperative cancel is observed.

The later cancellation/doc change is NOT attributed to earlier red/green inputs.
Separate source bindings and raw logs retain all stages:

- RED: 66899 terminal 101; `RED-INPUTS.json`, `RED-TERMINAL.json`, `red.log`.
- Initial GREEN: 59422 terminal 0, three focused membrane tests;
  `GREEN-INPUTS.json`, `green-focused.log`.
- Initial sim suite: 39802 terminal 0, 57 tests; initial strict sim Clippy:
  36997 terminal 0. These precede rounded correction-history tightening.
- Rounded-history suite: 77995 terminal 0, 57 tests; `FINAL-INPUTS.json` and
  `final-sim-tests.log`. This precedes the cancellation/doc increment.
- Final review freeze: `REVIEW-INPUTS.json`; 2632 terminal 0, 57 tests,
  `review-sim-tests.log`. Formatting and strict sim all-target Clippy both exit
  0 in `review-fmt.log` and `review-clippy.log`. All final frozen inputs remain
  byte-identical after those checks.

The full workspace tests/Clippy, xtask verify, core/CLI/app builds, old garment
producers and native/reference cases were intentionally not run before root's
independent review. Existing source maps and acceptance records are not updated
by this candidate. docs/physics now describes the material method, cap, scratch
memory and remaining hybrid/contact limits without regrading historical cases.

The previous physics worktree and both diagnostic packets remain frozen. This
worktree uses its own compiled target; no binary/cache was copied. No GUI window
or background job is owned after the final check/commit handoff.

Skipped: broad integration gates and garment reruns. Risk: the local force-balance
regression is corrected, but inter-element/coupled-contact convergence and runtime
cost still require independent review and separately bound experiments.
