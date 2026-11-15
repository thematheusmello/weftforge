# Diagnostic-only strain arithmetic method

Written before product edits. This is AFTER-CLOCK static implementation validation,
not a physical rerun, threshold change, calibration or reference/parity acceptance.
The reviewed original-mathematics contract SHA256 is
`e2762b168085a4d2c2c738819083b2be9311c10946af1251ab21ffebf61f18d7`.
The worktree begins at `0770204a54abdf5f929b6626b01d7fcfb0e13ca4`.

Scope: replace the diagnostic loop's cached material-weight evaluation with a
private helper using the existing input's current relative edges, material UV
edges and triangle grain. Form both F columns by difference-of-products/division,
then rotate them, then use Neumaier compensation over the three rounded component
squares before sqrt minus one. Scan every triangle and both axes; exact ties stay
first input triangle then warp. Scalar and signed witness keep one candidate.
Callers are Cloth's diagnostic measurement and the affine unit test. Remove the
now-unused Membrane::signed_strains only; retain its deformation and solve bytes.
Tests and docs/physics.md are the other product files in scope. No configuration,
public API, dependency, force, constraint, accepted-state transaction, cancellation,
mesh, material, settings, adapter or evaluator changes are planned.

Verification: meaningful sim tests plus package Clippy and formatting in a real,
private target directory. A standalone Rust harness will include the exact private
helper bytes, with a Vec3 alias only, and evaluate all 362 saved actual states
without Cloth construction or integration. Input f64s are serialized to a
hash-bound little-endian binary; expectations are extracted unchanged from the
closed analysis and checked bit-for-bit for maximum, signed value, triangle and
axis. Also compare every triangle/axis against the unchanged frozen metrics.py.
Analytic/dyadic translation/permutation/reflection/shared-edge exact and near-tie
patches, clearly unequal winners, finite/error handling, and the squared-sum
witness [1, 2^-53, 2^-53] are required. Preserve any attempt failures and all logs.
Bind old inputs/evaluator before and after; retain the old 33A + 3B attribution
failures and A's physical strain failure at frames 28..37. Root owns integrated
full-workspace gates, registry/publication, STATUS and adapter work. No GitHub,
Actions, push, dynamics/CLI simulation, fit or UI runs are authorized here.
