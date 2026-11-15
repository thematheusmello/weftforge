# SIM-001 — Closed soft stitch coordinates

Prospective implementation-only contract, written before adapting candidate
`35afac604a93174fcdc2a5538799432aa6d69286` to the current solver. This bounded
change supplies no garment-quality, calibration, convergence or reference-parity
acceptance.

For a stitch with exactly zero current target and finite compliance c > 0, let
d = A(x) - B(x), using the existing weighted-point arithmetic. Merge signed
endpoint weights into q_i for each shared vertex. The vector constraint C = d
has energy E = d·d/(2c), with no factor of three. For substep h:

```text
alpha = c / h²
D = sum(inverse_mass_i * q_i²) + alpha
delta_lambda = -(d + alpha * lambda) / D
x_i += inverse_mass_i * q_i * delta_lambda
lambda += delta_lambda
```

Positions and d are metres, h is seconds, mass is kilograms, c is m/N, alpha
and D are 1/kg, lambda is kg m, lambda/h² is force in N, and E is joules.
The constant gradients make the update rotation covariant. Accepted weights,
weighted-point evaluation and duplicate merge order stay unchanged; weights
are not renormalized. Pinned/zero-inverse-mass positions remain exact. Preserve
the existing guard that skips nonfinite D or D <= 1e-20.

Dispatch uses exact target == 0 and c > 0. All positive targets, including the
smallest positive f64, and all hard c == 0 stitches retain the old scalar radial
arithmetic in lambda[0]. The other two components are unused on that branch.
Multipliers remain private and are reset per substep after target updates. No
public type, file format, command, configuration or dependency changes.

Analytic acceptance before integration:

- Reuse the candidate's fixed N32 two-anchor quadratic oracle and its 1e-12
  position, multiplier, residual, individual-action, energy and stationarity
  tolerances. Retain the legacy scalar transport failure above 0.01 N.
- Preserve the isolated m=2 kg, h=0.5 s, c=0.125 m/N oracle: lambda=(-1,-2,0)
  kg m, free endpoint=(0.5,1,0) m, force=(-4,-8,0) N and E=5 J.
- Check two free unequal masses, shared/repeated weighted vertices, zero gap
  with zero/nonzero lambda, all-pinned vertices, accepted near-unit weight sums,
  and the inherited small/nonfinite denominator guards.
- Check translated and rotated coordinates and multipliers against the same
  analytic reaction; compare every unchanged hard/positive-target result to
  the prior radial operator exactly, including pinned/zero-gap guards.
- Retain the candidate's small Cloth substep-reset and cancelled/failed-step
  state-retention cases, with the current complete diagnostic fields.

Current capsule surface contact, canonical strain diagnostics, membrane/bending
constraints, cancellation/acceptance and severe-instability/quality thresholds
must remain unchanged. Focused solver tests establish only this operator scope;
actual garment protocols and broader integrated gates remain future work.
