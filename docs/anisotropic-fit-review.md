# Anisotropic fitting review

This is an implementation-only review of the original public
`fixtures/grain-shirt.weftforge` and retained
`fixtures/grain-shirt-drape.obj`, based on source commit
`359ac5b2fedfece5cfef9f22861c279bd2708801`. It is not reference execution,
fabric calibration, or fresh fitting acceptance. The failed, stale report
`WF-20261113-GRAIN-FIT-001` remains unchanged.

## Retained output

Recomputing deformation from each exported face's world and material coordinates
reproduces the reported maximum axis strain, `0.26930346146570994`. Its location
and sign are useful when diagnosing the failure:

| OBJ face, zero based | Piece | Warp length change | Weft length change | Material area |
| --- | --- | --- | --- | --- |
| 1 | Front, 2 | +0.2693034615 | -0.0130278450 | 0.0048579009 m² |
| 202 | Back, 31 | +0.2577678216 | +0.0008776979 | 0.0048773585 m² |
| 219 | Back, 31 | +0.2415446054 | +0.0020087225 | 0.0048773585 m² |

The worst front face is at the sleeve opening; the next two are near the back
neckline. All three extend along the authored weak warp axis, whose stiffness
is 50 N/m. The front's quarter-turn makes that axis material +V; the back's
unrotated warp is material +U. This is actual tensile length change, rather than
a swapped diagnostic axis or a compression-only failure.

All eight authored pairs of straight seam lengths match within `2e-16 m`.
The retained CLI report already records `0.2596921306` strain after zero-gravity
assembly, before gravity is enabled, and final kinetic energy of about
`1.43e-6 J`. These observations do not establish convergence or prove that the
fixture is physically overconstrained; they narrow the failure beyond a simple
gravity drop or unequal authored seam lengths.

## Math and mapping

Inspection found no concrete defect that justifies changing the stated model
in [CPU cloth model](physics.md):

- The document bridge keeps material UV separate from arranged positions and
  passes each piece's grain and assigned material to its triangles.
- Membrane derivatives implement `F = Ds * inverse(Dm)`, followed by the authored
  grain rotation. The three constraint gradients differentiate the documented
  warp, weft and engineering-shear strains. Area scales both membrane energy and
  rest mass; substep duration scales XPBD compliance by `1 / h²`.
- The signed-dihedral gradient has an existing independent finite-difference
  check. Bending uses arranged rest geometry and the documented approximate
  `D*l/H` coefficient.
- Weighted stitching combines repeated degrees of freedom before projection.
  Assembly reduces each initial separation at the declared speed. Capsule
  projection uses closest distance to its segment and summed radius/offset.

No production solver code, fixture, material value, fitting protocol, quality
threshold, unit or diagnostic definition changed in this review.

## Added analytic regressions

`crates/weftforge-sim/src/tests.rs` adds two checks independent of garment fitting:

1. An affine map with columns `(1.2,0,0)` and `(0.25,0.7,0)` has closed-form
   material-axis lengths after any grain rotation. A rigid 3D arrangement and
   reversed triangle winding must preserve those lengths and its 0.11 m² area.
2. A pinned unit right triangle extended to 1.2 m has one free degree of freedom.
   Its first XPBD update is `x = 1.2 - 1.2*0.22/(1.44 + alpha)`, with `alpha=4`
   for the 50 N/m axis and `alpha=0.2` after a quarter-turn activates 1000 N/m.
   A separate unit-axis shear of 0.3 gives `delta_lambda=-1/32` with 10 N/m shear
   stiffness, 0.5 m² area and 0.2 s duration; every node's displacement is checked.

Verified on 2026-11-14 with one build job, a dedicated external Cargo target and
an APFS temporary directory:

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo test -p weftforge-sim membrane_ -- --nocapture` | Both new tests passed |
| `cargo test -p weftforge-sim` | All 22 tests passed; no doc-test failures |
| `cargo clippy -p weftforge-sim --all-targets -- -D warnings` | Passed |

No fitting replay or performance measurement was run because production math
is unchanged. The package checks do not supply a new passing garment result;
workspace integration checks remain separate. Uncalibrated constitutive values,
bending refinement, finite-iteration convergence, and incomplete cloth continuous
collision remain unresolved.
