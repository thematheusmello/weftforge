# Independent manufactured contact oracles

These are WeftForge implementation checks. They establish neither Harbor improvement nor reference parity. The frozen Rust source and logs are included under `component/`; no numerical program was run during curation.

For two equal masses, let `m = 0.00012 kg`, `h = 1/480 s`, `c = 1e-8 m/N`, `k = m/h²`, and predictions `y± = (±a,b,0)` with `a = 0.1 m`, `b = 0.025 m`. One zero-target stitch gives the independent SI energy

`E(q) = (k/2)(|q0-y0|² + |q1-y1|²) + |q0-q1|²/(2c)`.

Without contact its unique quadratic minimum solves `(M + h² Aᵀ C⁻¹ A)q = My`, where `A = [I,-I]`, `M = mI`, and `C = cI`. Thus `q± = (±ka/(k+2/c), b, 0)`. This controls the exact one-soft-stitch branch; it does not exercise PCG.

The capsule has endpoints `(0,-0.2,0)` and `(0,0,0)`, radius `R = 0.1 m`, no friction and no thickness. Both predictions are outside; coincident points at `(0,R,0)` are feasible. In the symmetric active upper-cap family `q± = (±R sinθ,R cosθ,0)`, the energy is

`E(θ) = k[(R sinθ-a)² + (R cosθ-b)²] + 2R² sin²θ/c`.

Its stationarity equation is `k(b sinθ-a cosθ) + (2R/c)sinθ cosθ = 0`. Dividing by positive `k cosθ` gives `a = b tanθ + (2R/(kc))sinθ`; the right side is strictly increasing on the bracket used by the retained independent bisection. This identifies the unique minimum on that symmetric active-cap family, without claiming a global nonconvex contact solve.

For an actual final state, differentiate the Cartesian energy directly: `g0 = k(q0-y0) + (q0-q1)/c`, `g1 = k(q1-y1) - (q0-q1)/c`. Compare these gradients with actual saved contact reactions `r_i = k p_contact,i`, and check the final normal, gap, nonnegative multiplier and complementarity. The retained history identity is `q-y = p_sewing+p_contact`.

The baseline ran the original 32 sewing/contact passes. Its Rust diagnostic exited successfully because analytical controls, finiteness and history checks passed; it deliberately reported rather than asserted coupled convergence. Its printed final force-balance residual was `17320512.487939887 N`, and the retained independent geometric KKT residual was `8660255.568583531 N`. This is a convergence failure, not a failing Rust process. Every sewing block was individually stationary. It is a manufactured split-contact counterexample, not evidence that PCG or this mechanism caused the historical Harbor failure.

Candidate001 made the declared oracle criteria ordinary regressions and passed 100 simulator tests. Its later source review retained the separate F1 large-finite-axis defect. Candidate002 corrected only the numeric projection/error handling and added its exact regressions; it passed 102 tests. Its physical settings, 32 passes and prior tolerances stayed fixed. Those 102 tests are included within the actual F workspace total of 480.

For the supporting-plane law, a unit current closest-axis normal `n` uses exact capsule support `H(n) = R+t+max(n·a,n·b)`. The translated gap is `n·(anchor-a)-max(0,n·(b-a))-(R+t)`. The same normalized `n` is used for the gap and replacement action. This chooses the current local branch, including deliberate retention when an anchor lies across the body; same-side separating halfspaces release. At zero distance the exact support value keeps arbitrary fallback planes valid.

The friction oracle uses a unit sphere, anchor `(0,0,0.5)`, previous point `(1,0,0.5)`, `μ = 0.5`, and a unit-second substep. The independent fixed point is `(0.6,0,0.8)`. For effective inverse mass `D`, its normal multiplier is `0.6/D` and tangent magnitude `0.3/D`. Substitution in the retained Coulomb plane update maps that target to itself. The first visit remains the independent plane answer `(0.25,0,1)`; sphere complementarity is checked only after the unchanged 32 visits. N1 and unequal-mass N3/pinned cases use the declared `1e-3 m`, `1e-3 N`, and `1e-8 J` final tolerances.

Limits remain explicit: current-side local contact is not a global exterior projection or CCD; finite friction is not an exact one-visit nonlinear solve; retained-point discovery and general multiple-contact convergence are not certified. F source QA and toy success do not establish full-surface, calibration, garment, native-workflow or reference acceptance. The historical D full181 peak remains failed; the corresponding F full181 execution is pending.
