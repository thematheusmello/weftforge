# SIM-001 — Linear coordinates for closed elastic stitches

Prospective implementation-only contract, independently reviewed in the retained
mathematical proposal. It changes iterative coordinates for exactly zero stitch
target and finite positive compliance; authoring and physical parameters remain.
It does not predict a correction to the original garment's strain failure.

Let q=A(x)-B(x), with the existing supplied weighted-point evaluation and merged
duplicate vertex coefficients beta_i. The current zero-target potential is
E=q·q/(2c). Three coordinate constraints preserve this potential with compliance
c per component, without a factor of three. For h seconds and a=c/h²:

```
D = sum(inverse_mass_i * beta_i²) + a
delta_lambda = -(q + a * lambda) / D
x_i += inverse_mass_i * beta_i * delta_lambda
```

Lambda is a vector in kg m. Its constant Jacobian removes changing-direction
transport: individual free-node projection action equals beta_i*lambda/h² to
roundoff. The endpoint constitutive residual q+a*lambda can still be nonzero
after coupled constraints. Preserve weighted-point arithmetic, duplicate merge
order and accepted weights; do not renormalize or assume their sum is exact.

Keep all positive targets, however small, and hard c=0 on the existing radial
branch. There is no epsilon dispatch. Targets update before the existing fresh
substep multipliers; no scalar/vector history conversion is required. Public
Stitch, StepSettings, Diagnostics, commands and project formats stay unchanged.

Fixed operator oracle: one free mass1 at(0,0,0), pinned anchors(1,0,0)/(0,1,0),
two stitches c=1,h=1, strict first/second order, N=32 iterations. The independent
incremental quadratic has Hessian3I and minimizer(1/3,1/3,0). With u=4^-32:

```
lambda1 = ((2-2u)/3, (-1+4u)/3, 0)
lambda2 = ((-1+u)/3, (2-2u)/3, 0)
x       = ((1-u)/3, (1+2u)/3, 0)
r1      = (-u, 2u, 0), r2=(0,0,0)
E       = (5-u+5u²)/9, Phi=2/3+5u²/6
```

Acceptance: candidate positions/lambdas/residuals/energy/stationarity and each
individual stitch's free-node transport agree within1e-12 in their declared
m/kg-m/J/N units; pins are exact. One unchanged legacy N32 observation must retain
its RED witness: stitch1 transport exceeds .01 N (independent lower bound
.011917041355797431 N). N1/N2 recurrence examples are derivations, not a sweep.

Focused edge checks cover the independently derived isolated m2,h.5,c.125 case
(lambda=(-1,-2,0), endpoint(.5,1,0), force(-4,-8,0), E5J), shared/repeated weighted
vertices, allowed near-unit weight sums, zero gap with zero/nonzero multiplier,
all-pinned participants, unchanged hard/positive targets, cancellation and
whole-call failure retention. Preserve first failures and expected values.

Before garment claims, bind source/binary/input/evaluator and rerun original
scenes under unchanged settings/quality limits. Distinguish all force/quality
results and report any remaining failure. No claim of equilibrium, calibration,
full sewing coverage or reference parity follows from this operator correction.
