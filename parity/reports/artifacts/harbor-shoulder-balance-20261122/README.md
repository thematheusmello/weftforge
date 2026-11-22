# Shoulder forces at the frame-28 strain crossing

At the last of the eight substeps of frame 28, the native shoulder stitch
spans 13.393 mm in the original Harbor and 13.167 mm in the extended hem.
Its target at this instant is 13.333 mm. With the authored bilateral seam
compliance of 10⁻⁸ m/N, the observed gaps imply +5,932 N and −16,631 N
respectively. Solving the local force equations jointly brings both
tensions below 2.7 N. All six solutions still exceed the 15% strain limit
on the front shoulder.

| Mobile vertices | Original tension | Original front warp | Extended tension | Extended front warp |
| --- | ---: | ---: | ---: | ---: |
| Two stitched vertices | 2.577 N | 15.584% | 2.672 N | 16.103% |
| Six vertices of the opposing warp faces | 2.567 N | 15.292% | 2.642 N | 16.132% |
| Fourteen vertices including their adjacent triangles | 2.544 N | 15.220% | 2.634 N | 15.844% |

Each case starts with the actual predictor, previous positions, rest
coordinates, masses, sew targets, capsules, friction, timestep and
materials from the November 20 CPU substep. The six front and back vertex
positions agree bit for bit with the later 0a4bc886 stage probe. The
exterior stays at its observed returned positions. Reactions there are free.
All membrane triangles, bending hinges and stitches incident on the mobile
nodes participate. A nonlinear solve balances momentum, bilateral stitch
force, unilateral capsule pressure and Coulomb friction on the mobile
nodes. The largest motion from the native return is 0.52 mm.

The checker independently differentiates the scalar membrane and hinge
energies with a five-point stencil. It computes inertia from the saved
predictor, tension from the true 3D stitch length, and contact reactions
from the saved pressures and friction. It checks the six coordinate
equilibria up to the full 42-coordinate ring, triangle-to-capsule
clearance, friction direction and magnitude, and the native whole-garment
swept self-contact query result. All six mobile force residual norms
are below 2.6 × 10⁻⁹ N. The native swept query detected zero self-contact
events in all six local returns.

The large constitutive mismatch at the native return is a numerical error.
At the same prior state and prescribed exterior, resolving the local
force balance still leaves the strain failure. Earlier substeps and
exterior shoulder motion can influence the peak. The independent
solutions are stationary points conditioned on each boundary. The
full trajectory, uniqueness, acceptance and resource limits remain open.
A distinct historical coupled calculation freed the exterior and advanced
from rest. At the end of frame 28, its front line measured 57.506 mm in
the original and 57.802 mm in the extension. At frame 29 the original
front line reached 58.616 mm while the anchor-spacing lower bound fell
to 57.314 mm, below the 57.5 mm screen. The rigid-anchor impossibility
at that frame is specific to the native trajectory. The extended
coupled calculation retained only the first substep of frame 29 before
its swept cloth-contact check refused the next trial. The packet retains
the two faces' coordinates and historical force-grade identities for
the sampled substeps. A future construction or constitutive correction
requires independent evidence about the intended sewing loads or the
material response during closure.

Run the independent checker with Python 3.

```sh
python3 parity/reports/artifacts/harbor-shoulder-balance-20261122/check_balance.py
```
