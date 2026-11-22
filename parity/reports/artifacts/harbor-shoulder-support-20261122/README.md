# Sliding contact at the Harbor shoulder

At the final substep of frame 28, removing tangential avatar traction reduced
the extension of the 50 mm front material line by 0.465 mm in the original
520 N/m Harbor and by 0.508 mm with the extended hem. Both normal-only
equilibria stayed outside the avatar and fell below the 15% strain limit
for this line.

| Contact in the 14-node patch | Original front warp | Extended front warp | Original / extended shoulder tension |
| --- | ---: | ---: | ---: |
| Recorded Coulomb coefficient 0.4 | 15.220% | 15.844% | 2.544 / 2.634 N |
| Normal support only | 14.290% | 14.829% | 2.174 / 2.252 N |
| All local avatar forces released | 9.388% | 9.436% | 1.277 / 1.332 N |

Releasing normal support as well allowed 8.535 and 8.955 mm of penetration.
That control estimates how much of the reaction comes from supporting the
garment. The normal-only control isolates the contribution of tangential drag.

All eight loaded contacts in the two recorded roots were sliding on the upper
spherical end of the torso capsule. Normal force ranged from 0.809 to 2.086 N.
The contacting cloth traveled 0.397–1.428 mm in 1/480 s. The traction opposed
slip and dissipated 2.051 mJ in the original and 1.631 mJ in the extended
patch. The direction of the exact shortest arc between the projected positions
and the recovered force at coefficient 0.4 agreed within 1.1 × 10⁻¹³ N.
This rules out endpoint tangent projection as the source of the extra stretch
in the two saved equilibria.

The four counterfactual roots use the same predictor, previous positions,
sewing targets, material constants and fixed exterior as the recorded
14-vertex root. We changed only the indicated contact forces in the reduced
system. Central five-point differences of membrane and bending energy,
mass-weighted predictor displacement, sewing distance and capsule forces
recover all six mobile force balances within 2.6 × 10⁻⁹ N. The whole-mesh
swept self-contact check found zero new contact rows at each return.

The earlier trajectory and the exterior already carry the effects of
avatar contact. This single-substep result cannot predict a passing full fit
or establish the physical sliding coefficient for the uncalibrated cotton.
The next discriminating test is an independent sliding coupon for a stated
fabric and avatar surface or a controlled reference slip measurement. Apply
0.81–2.09 N of normal load and measure steady tangential force at
0.19–0.69 m/s. Compare the measured force-to-normal-load ratio with 0.4 before
changing the registered model. The official 181-state failures remain in force.

Run `python3 -B parity/reports/artifacts/harbor-shoulder-support-20261122/check_support.py`
from the repository root and compare its output with `results.json`. The
solver predictor came from historical CPU source `6707eb78`.
[Prior saved material coordinates, contact and force balance](../harbor-shoulder-balance-20261122/README.md).
