What this change does: commit24dba7715233a7423088772d49c32a632d901a0c
adds static triangle/capsule contact alongside the previous vertex projections.
It distributes corrections through closest-point barycentric weights and
inverse masses, and includes full static capsule surface occupancy in initial,
reset and final diagnostics. Thin-face numerical uncertainty stays visible.

No actionable blocking finding was established in this bounded review. This is
an independent source/static assessment for the next integration gate, not a
claim of garment clearance, convergence, performance or reference parity.

Reviewed immutable committed capsule.rs, contact.rs, lib.rs, math.rs, tests.rs,
and the corresponding physics documentation. Root's later adapter edits were
outside this review. Exact source hashes are recorded in static-check.json.
No product files, prior evidence, fixtures, Git state, gates or UI were changed.

- **Closest features and caps.** The closed-edge candidates, both capsule-axis
  endpoints' face projections and face/axis crossings cover the minimum for a
  nondegenerate triangle and closed axis segment. A positive-distance interior
  face/interior axis minimum requires the axis parallel to the face; then a
  tied minimum reaches an edge or axis endpoint, which is already included.
  Thus parallel/coplanar cases and spherical point axes are covered without
  requiring a vertex to lie inside. The prior original140 and HarborA1164/B678
  surface witness fixtures remain explicit tests at the unchanged1e-10m static
  tolerance.
- **Conservative thin-face behavior.** The near-degenerate branch subtracts an
  upper altitude bound from closed-boundary distance. Every triangle point is
  within that altitude of its longest edge, so this gives a lower distance
  bound. Underflow returns conservative zero with uncertainty. Confidently
  collapsed triangles retain their closed edges/vertices. The returned bound
  can create false positive contact; it is not silently presented as exact
  geometry. `capsule_surface_uncertain_pairs` counts even separated uncertain
  pairs, as documented. Root owns the acceptance/CLI/UI treatment of that count.
- **Response, friction and pins.** For a closest point beta, the denominator
  sum(w_i*beta_i²) and displacement w_i*beta_i*normal*depth/denominator have the
  correct inverse-mass weighting. The same weighted point supplies tangential
  displacement, and the friction limit is capped both by the existing physical
  coefficient times the normal impulse and the remaining tangential motion.
  A pinned vertex contributes zero motion. A zero denominator leaves the
  contact unresolved for diagnostics. A single local correction can change
  the closest feature; the implementation recomputes it on subsequent passes
  and reports the remaining actual surface residual.
- **Zero-distance recovery.** The current distance normal is used away from
  zero. Fallbacks use the previous same-barycentric separation when the previous
  triangle was axis-separated; an already crossed previous face supplies a
  stable geometric anchor. Unusable radial directions fall back to a
  deterministic axis-perpendicular basis, or+X for a point axis. These rules
  avoid division by zero and give finite directions. They remain recovery
  rules with explicitly permitted residuals, not a proven unique contact law.
- **Final occupancy and cancellation.** Initial, reset and final analysis call
  the shared full triangle/capsule evaluator. Later triangle/capsule, cloth or
  pin corrections can undo earlier separation; final geometry is scanned
  again. During step, positions, velocities and seam targets remain local
  candidates; projection/final-analysis cancellation propagates before the
  accepted state/diagnostics are assigned. Reset intentionally uses the
  non-cancelling path. The new runtime tests target mid-projection and final-
  analysis cancellation and preservation of all accepted fields; those tests
  were read, not executed during this review.

The small independent static check uses a separate100-digit Decimal constrained
quadratic oracle. It minimizes distance over triangle barycentric coordinates
and axis fraction by enumerating active boundaries of their product polytope
and solving the tiny stationary linear systems. This avoids the implementation's
cross-product closest-feature formulas and thinness branch. Analytic oracle
self-checks cover face, edge, vertex, cap, intersection, parallel and point-axis
cases. A transparent Python transcription of the committed closest routine is
then compared with that oracle; this is explicitly **not executable Rust
validation** or proof of operation-order identity.

All83 bounded cases passed:48 deterministic ordinary cases,12 skinny cases,
6 parallel/nearly parallel cases,2 collapsed cases,3 saved witnesses and12
scaled/axis-permuted copies. Nine cases exercised conservative uncertainty;
none exceeded the independent true minimum beyond the predeclared rounding
allowance. The largest ordinary absolute error divided by local scale was
1.5274253848741944e-16. The separate underflow case retained uncertain zero.
The isolated static inverse-mass/friction check also preserved its pinned node
and the intended weighted-point normal/tangent displacement. Full results,
inputs for any failure(none), tolerances and source hashes are retained in
static-check.json. No garment trajectory was generated.

Expected load assessed here: the existing deterministic CPU path and current
original six-capsule analytic mannequin. The new surface phase explicitly
scans every triangle/capsule pair. No broad phase is claimed, and performance
at a complete garment or large mesh was not measured. No speed finding is
reported without a measured failure at the intended workload.

Verdict: Ship this bounded primitive to root's next reviewed integration gate.
Not checked: actual Rust execution, Cargo/workspace gates, garment dynamics,
native workflow, performance, continuous collision or root's adapter changes.
Risk: static coverage and finite deterministic recovery do not establish
whole-garment convergence or physical/reference parity.
