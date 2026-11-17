# Independent subnormal-direction edge oracle

This is a separate floating-point robustness case, not another garment or a
replacement for the coupled N32 history regression. No arithmetic emulation was
used to choose its expected values or bounds.

Let epsilon be the minimum positive f64 subnormal, free inverse mass 1 kg^-1,
h=1 s, compliance 1 m/N, target 1 m, and the other endpoint pinned at the origin.
The local endpoint difference is the free-node position. W=a=1, hence

    Phi(q) = |q-z|^2/2 + (|q|-1)^2/2.

Case A starts at d=(epsilon,epsilon,0), history H=0 and z=d. In exact real
arithmetic, its minimizing direction is (1,1,0)/sqrt(2), radius
(1+sqrt(2)*epsilon)/2, and each nonzero coordinate is
1/(2sqrt(2))+epsilon/2. The epsilon/2 correction is negligible relative to the
fixed 1e-14 m absolute bound and cannot affect the ordinary-size f64 result.
The test uses expected coordinates .5/sqrt(2) and radius .5.

Case B starts at the same d but history H=d, so z=d-W*H=0 exactly. Every direction
at radius .5 minimizes Phi. The production policy explicitly retains the current
endpoint direction when defined, so this case must choose the same unit 45-degree
direction as A. It tests the separate zero-anchor fallback path.

For the returned ordinary-size q, independently differentiate the physical spring:

    f(q) = (1-|q|) q/|q|,
    R(q) = q-z-f(q).

Both subcases freeze position, radius, and independent force-residual norm bounds
of 1e-14 (metres/metres/newtons). Pinned origin equality and finite positions and
history are also checked. Each case resets its declared position and history;
there is exactly one direct stage call per case. Both actual results are printed
before the position/radius/force assertions. No Cloth initialization is involved.

The bounded counterexample is that hypot of the raw subnormal components rounds
onto their coarse lattice. Dividing those raw components by that rounded radius
can produce a vector such as (1,1,0), which is not unit length. This can yield a
finite ordinary-size update at radius sqrt(.5), missing the minimum at .5.
Scaling by max-absolute component before computing the unit direction preserves
the component ratios on both normalization paths. The radius used in the existing
spring formula remains unchanged; no stiffness, target, history, guard, iteration
or physical-fixture change is needed.

This oracle and its assertions are frozen before the first edge RED command.
RED/GREEN results apply only to this edge case. The original coupled N32 oracle,
all prior outcomes, and all failed/operational garment cases remain unchanged.
