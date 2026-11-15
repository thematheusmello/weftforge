# Proposed material-axis peak diagnostic contract

Original mathematics and an explicit numerical reporting convention; no change
to the frozen experiment, its evaluator, its failures or its15%/5mm/2mm limits.

For a current triangle and its authoritative rest material coordinates, form
E=[x1-x0,x2-x0], D=[uv1-uv0,uv2-uv0], F=E*inverse(D). With the authored grain
theta, evaluate warp=F*(cos(theta),sin(theta)) and
weft=F*(-sin(theta),cos(theta)). Signed strain is length(axis)-1. The reported
maximum is the largest absolute signed strain over BOTH axes of EVERY triangle.
It is a material-axis length metric, not a principal-strain or convergence bound.

The signed value, axis and triangle of the reported peak must come from the same
computed candidate that supplies the scalar maximum. Exact equality chooses the
first input triangle, then warp. Attribution resolves only against the matching
rest mesh/document revision; stale/out-of-range indices remain invalid.

For reproducible f64 reporting, compute the two current deformation columns from
relative current edges and the restUV difference-of-products/division, THEN rotate
those columns to material axes. Use an explicitly documented compensated sum of
the three rounded component squares before sqrt and subtraction of1. The small
independent arithmetic_followup.py describes that original numerical convention;
its compensated norm agrees exactly with the frozen host reference for all
838,392 examined triangle/axis norms. Plain three-term left association does not.
Do not replace the largest candidate by an epsilon-tie winner, sample triangles,
snap grain angles or change any physical threshold.

This is a deterministic computed-maximum contract. It does not promise a unique
physically meaningful triangle when two true magnitudes differ below f64 roundoff.
In the actual36 disagreements, all pairs share a material-u edge; their real
magnitude gaps are at most9.69e-19. A future ambiguity annotation may explain such
conditioning, but it cannot substitute for the current exact-index criterion or
erase any frozen failure.

Required static checks before adopting a production correction:

1. Known affine warp/weft extension/compression and grain rotation, with scalar,
   signed value and attribution checked together.
2. Exact dyadic translations, coordinate permutations and reflections that
   preserve represented edges; the declared deterministic tie result survives.
3. Adjacent/shared-edge equal and near-tie patches, plus a clearly unequal patch
   that must move the winner. Stored pi/2 is used as stored, without axis snapping.
4. A squared-sum witness `[1,2^-53,2^-53]`: the corrected sum is1+2^-52 while a
   plain left fold yields1. Finite/error handling must remain explicit.
5. Compiled static diagnostic evaluation of ALL362 saved actual states against
   the unchanged frozen values/indices; this requires no new integration steps.
   Keep the old33A+3B attribution failures and all original quality flags intact.

The bounded recommendation is a diagnostic-only arithmetic correction using
existing ClothInput UV/material/grain data. Keep constraint projection arithmetic,
settings, accepted states, construction and solver transaction behavior unchanged.
The scalar and witness reduction is already internally coherent; the observable
disagreement comes from evaluation staging and squared-norm accumulation.

The attached Python comparison establishes feasibility on these saved states.
It is not compiled Rust evidence, a physical rerun, calibration or release/parity
acceptance. Root must separately review any implementation.
