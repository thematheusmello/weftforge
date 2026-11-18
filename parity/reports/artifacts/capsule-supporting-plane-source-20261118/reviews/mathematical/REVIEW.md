# Current-side capsule supporting-plane review

2026-11-18. Math/source review only: no arithmetic program, simulation,
constructor, Cargo, GUI, source change, grade, or new numerical result.
The previous contact-stationarity review remains frozen.

**Verdict:** the proposed update is a valid local current-side contact law
when it uses the exact capsule support plane and a true current closest-axis
normal. It removes the manufactured mode's local anchor-normal amplification
and enforces true normal complementarity at a converged fixed point. It is
not an exact nonlinear prox, a one-visit friction solve, or a general
32-pass convergence guarantee. Root explicitly chose current-side branch
retention; a far-side clear anchor does not by itself release contact.

## Equations and per-visit feasibility

Let K be the segment [a,b] inflated by R>0. For any unit n, its exact support is

    H(n) = R + max(n dot a, n dot b).

For an N1 vertex or one retained N3 material point beta, take the geometry
**before retraction**:

    q = sum_i beta_i x_i
    z_i = x_i - old_p_i       (free nodes; copy pinned x exactly)
    z = sum_i beta_i z_i
    D = sum_i w_i beta_i^2.

Use beta=1 for N1. Choose n from the true closest axis point to current q,
with an explicit unit fallback at zero distance. Then set

    g = n dot z - H(n)
    lambda = max(0, -g/D)
    t = (I-n n^T) sum_i beta_i (z_i-previous_i)
    eta = min(mu*lambda, |t|/D)
    J = lambda*n - eta*t/|t|
    trial_i = z_i + w_i beta_i J
    new_p_i = trial_i-z_i.

The tangent term is zero at negligible |t|, as in the existing law. For D>0,

    n dot sum beta_i trial_i - H(n) = g + D*lambda >= 0.

Thus that material point is outside the true closed capsule after the visit,
including finite tangent motion. This proof needs only an exact support value
and a unit n; it does not require n to be the closest normal. True closest
normals are additionally needed for the fixed-point stationarity claim below.

When d=dist(q,[a,b])>0 and n is the true closest-axis normal,
`H(n)=R+n dot axis(q)` and `g=(d-R)+n dot(z-q)`. At d=0 an arbitrary fallback
does not satisfy that identity. Therefore use H directly rather than relying
on the distance identity for all fallbacks. A local-coordinate equivalent is

    g = n dot (z-a) - R - max(0, n dot (b-a)),

which avoids subtracting large absolute world-coordinate dot products.
Use the same normalized n when computing g and applying J. Validate the
complete trial and retain actual occupancy diagnostics; exact-arithmetic
feasibility is not a universal f64 rounding certificate.
For mu=0, fixed anchors and a currently feasible point, current nodal x is
itself feasible for this halfspace. Its weighted projection therefore cannot
increase that isolated anchor-distance objective. This is a local descent
fact, not a coupled solver convergence proof; it does not apply unchanged
to friction or an input point that another stage has moved inside.

## Fixed points and friction

At a fixed point the point action is `q-z=D*(lambda*n+tau)`, with `tau dot n=0`.
The next normal replacement therefore satisfies

    lambda_new = max(0, lambda - (n dot q-H(n))/D).

With a true current closest normal, `n dot q-H(n)=dist(q,[a,b])-R`.
A fixed point has nonnegative true gap, and positive lambda forces gap=0.
The same replacement computes the tangent action at that final normal;
its clipping gives either zero final relative tangent slip inside the cone,
or maximum-dissipation sliding at `|tau|=mu*lambda`.

This addresses the prior curved-friction complementarity counterexample
**at convergence**. With empty history, current q=anchor z, so the first
normal/friction update is the same as the existing method for regular geometry.
For sphere R=1, z=(0,0,1/2), previous=(1,0,1/2), w=1 and mu=1/2, that first
update still gives `(1/4,0,1)` with positive gap and positive normal action.
The next visit changes the plane and reaction. Preserve the legacy one-call
friction oracle; evaluate final-feature complementarity after the unchanged
32 visits with an explicit residual, not as an exact one-call guarantee.
Convergence under general friction/multiple contacts remains unproved.

The current law permits arbitrarily large positive lambda when the anchor
lies far behind the current plane. The old anchor-radial point prox restricts
normal point displacement to at most R because it chooses the anchor's nearest
exit. Large balancing reactions on a chosen local branch must not be clipped
to that global-prox bound.

## Manufactured split mode: why this addresses it

For the proposed two-equal-mass, mu=0 test let a_x=R, b_y=R/4 and

    alpha = 1 / (1 + 2*h^2/(m*c)).

Let s be one node's owned contact x displacement, using symmetry to give
the other the opposite s. Exact zero-target sewing produces

    u_sew = alpha*(a_x+s).

Near the axial top-cap solution, with contact y displacement R-b_y,
the old anchor-radial replacement has first-order map

    s_new = (R/b_y-1) * [alpha*a_x - (1-alpha)*s] + higher-order terms,

so its tangential amplification is `-3*(1-alpha)`. In the stated stiff-seam
regime alpha is small, yielding alternating amplification near -3. This
comes from recomputing a radial normal at the retracted anchor at height b_y.

The current-plane update instead has first-order map

    s_new = (1-b_y/R)*alpha*(a_x+s) + higher-order terms,

with amplification `(3/4)*alpha`, near zero in the same regime. This removes
that local flip mechanism without changing m, h, c, R, mu, fixtures or passes.
These are analytic local expansions, not executed source results or a finite
32-pass error bound. The independently prepared baseline/candidate and angle
oracle must decide whether the actual implementation passes its fixed bounds.

For the symmetric cap branch, `q_±=(±R*sin(theta),R*cos(theta),0)`, the exact
physical incremental objective and its stationary equation are

    F(theta) = (m/h^2)*[(R*sin(theta)-a_x)^2+(R*cos(theta)-b_y)^2]
               + (2*R^2/c)*sin(theta)^2
    -a_x*cos(theta) + b_y*sin(theta)
      + (2*h^2*R/(m*c))*sin(theta)*cos(theta) = 0.

The independent oracle must retain its stated branch/domain and check its
minimum, not merely any root of this equation. This review asserts no global
3D asymmetric/multiple-body minimum from that one-angle equation alone.

## Concrete limits and release policy

1. **Local branch, not global metric projection.** Sphere R=1, w=1,
   current q=(1,0,0), old_p=(3,0,0), anchor z=(-2,0,0) is an exact fixed point:
   n=+X, H=1, g=-3, lambda=3, trial q=+X. The anchor is already outside, but
   on the far side. The isolated metric objective is smaller at z and has a
   tangential maximum at this opposite boundary point. Root accepts this as
   intentional current-side retention, not global optimality. It also does
   not prove continuous non-crossing when other blocks can move current q
   between planes; swept/CCD policy stays separate.

2. **Same-side release is exact.** If `n(q) dot z >= H(n(q))`, lambda=0,
   friction is zero and the entire old nodal action is removed, including a
   rotated old normal/tangent. Do not advertise “release iff anchor is anywhere
   outside the capsule.” The existing translated-release test at
   `contact.rs:1628–1637` translates +Z on its +Z branch; retain it unchanged
   and verify actual candidate histories satisfy this same-side release.

3. **Zero-axis-distance fallback must use H.** For axis `(0,-1,0)` to
   `(0,1,0)`, R=1, current q=(0,-1,0), old_p=0, fallback n=+Y, the naive
   distance/axis(q) formula moves to `(0,0,0)`, still inside. The exact support
   H=2 instead requires y>=2 and produces a feasible trial. Any unit fallback
   gives a valid supporting plane through H; which branch it selects is an
   explicit deterministic recovery/tie policy. For a=b the same formula
   naturally reduces to a sphere.

4. **Exact support alone does not supply the true final normal.** Existing
   N1 `collider_feature` at `contact.rs:1033–1039` replaces an axis with point a
   when its squared length is <=1e-20. Take a=0, b=(epsilon,epsilon,0),
   R=epsilon, q=(2*epsilon,0,0), and `2*epsilon^2<=1e-20`. That approximation
   supplies n=+X. Exact H=2*epsilon allows a positive-action plane fixed point
   at q, although true closest-axis distance is `sqrt(2)*epsilon>R` and its
   normal is `(1,-1,0)/sqrt(2)`. Feasibility survives; true complementarity
   does not. A claim about the actual capsule requires a robust true point/
   segment closest query (the N3 scaled capsule query already avoids this
   absolute short-axis substitution), including underflow/uncertainty handling.

5. **Pins and multiple supports retain their limits.** For D>0 the proof
   includes partial pins without moving them. D=0 cannot repair a conflict;
   preserve fixed bits and visible residuals. Each N3 beta keeps its own action
   and uses the current point for that beta. A later support/collider/material
   visit may undo an earlier point's clearance. Finite support discovery does
   not establish whole-triangle feasibility or global equilibrium. N4 finite-
   triangle cloth contact and the Torso's radial normal are separate laws;
   this capsule plane argument does not repair either. Torso requires its true
   ellipsoid support/normal rather than reusing the radial direction.

## Smallest implementation/test boundary

Keep the shared actual-action retraction/replacement and its transactional
commit. Only capsule feature evaluation/gap construction must change: N1
current closest-axis geometry, and N3 current fixed-beta point geometry.
Ground remains its identical affine plane; N4 and initial-recovery policy
remain separate. No command, public diagnostic, CLI/MCP adapter or schema
change is required merely for this internal law.

Retain the existing one-call barycentric/mass/friction test
(`tests.rs:3218–3248`), rotating/releasing action and exact-pin tests
(`contact.rs:1384–1520`), 30-visit manifold/translated release test
(`contact.rs:1569–1637`), unchanged 30-iteration exterior assertion, final
occupancy, cancellation and rollback. Add only the manufactured split/angle
oracle plus focused same-side/far-side branch, finite-friction convergence,
zero-distance fallback and tiny/point-axis cases needed for this changed law.
No existing thresholds, outer passes, physical settings or velocities should
change to obtain green. A source implementation still needs the repository's
four required integrated checks; none was run by this read-only review.

Remaining risk: sequential current-side planes can select an undesirable
nonconvex branch or fail to converge. A successful manufactured component
would be bounded source evidence, not Harbor causation, whole-garment quality,
calibration, reference parity or roadmap completion.
