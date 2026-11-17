# Exact local compliant radial update

This derives the repair from the unchanged physical spring energy, independently
of scalar XPBD projection. The frozen coupled fixture/oracle remain unchanged.

After merging repeated vertex indices, let beta_i be the signed endpoint weight,
w_i the inverse mass (zero at pins), H the existing Vec3 correction history, and
d=sum(beta_i*x_i). The retained local endpoint anchor is z=d-W*H, where
W=sum(w_i*beta_i^2). Histories reset once per substep, as before. Other stages move
x without moving H, so their displacement moves this local anchor.

For W>0, the minimum nodal inertia compatible with endpoint D is obtained with
x_i-z_i = w_i*beta_i*(D-z)/W. It is |D-z|^2/(2W). The unchanged radial spring
with target t and a=compliance/h^2 therefore gives the scaled objective

    F(D) = |D-z|^2/(2W) + (|D|-t)^2/(2a).

For r=|z|>0, Cauchy-Schwarz chooses the unique minimizing direction n=z/r.
The remaining scalar quadratic has radius

    rho = (a*r+W*t)/(a+W).

Consequently H_new=(D-z)/W=n*(t-r)/(a+W), and the nodal update is exactly
w_i*beta_i*(H_new-H). Retaining H_new, rather than a scalar against the latest
gradient, preserves the vector action across direction changes. No inner solve,
iteration, new persistent state, or changed physical coefficient is required.

At t=0, H_new=-z/(a+W). Subtracting H gives
delta_H=(-d-a*H)/(a+W), exactly the existing closed compliant branch. That branch
and the entire hard-compliance branch are retained byte-for-byte. The positive
branch uses chained hypot for a representable norm and component division for a
unit direction, avoiding squared-norm/reciprocal overflow on otherwise finite data.

At r=0,W>0, all directions minimize F with radius W*t/(a+W). The existing endpoint
direction is retained when defined; otherwise +X is a deterministic tie choice.
For all-pinned W=0,r>0, positions cannot move; H_new=n*(t-r)/a retains the spring's
constitutive vector force. At all-pinned zero anchor the direction is undefined,
so both positions and history remain unchanged. Nonfinite/invalid positive
targets, rejected denominators, or nonrepresentable trial arithmetic are no-ops.
Every candidate node update is checked before any history/position commit.
Merged duplicate coefficients ensure no vertex receives two nonzero updates.

The existing positive-target scalar-history test used arbitrary unused multiplier
components; it is replaced by independently derived physical position, vector
action, and force checks at the same 1e-14 bounds, including the subnormal target.
Every old hard case and bound is retained. New small checks cover cross-side
duplicate cancellation, weighted physical force, pinned constitutive history,
both zero-anchor tie routes, all-pinned zero gap, and rejected/overflow arithmetic.

The new N32 component's only semantic observer change is its private multiplier
force logger: vector history divided by h^2. Its original inputs, loops, direct
energy/force oracle, position/force/action/pin/finite assertions and bounds remain
exactly as frozen before RED. The source proof records their byte identity.

Sole production caller: Cloth::step's sewing loop. Histories, target scheduling,
stage order, local candidate state, final validation, cancellation and whole-call
publication/rollback are unchanged; no lib.rs edit is needed. A finite local
component result would not establish garment causality, garment acceptance or
reference parity. RED snapshots and old case seals remain historical evidence.
