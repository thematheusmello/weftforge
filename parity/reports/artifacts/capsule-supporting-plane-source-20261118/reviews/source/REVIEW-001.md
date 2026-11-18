# Candidate001 independent source review

Decision: **REVIEW_FINDINGS — one requested numeric correction before SOURCE_READY.**
The current-side capsule law, action history, pins, branch policy and independent
coupled oracle are otherwise consistent with the declared bounded scope.

Reviewed the isolated worktree diff and all callers of `axis_fraction`,
`point_feature`, `capsule_support_gap`, `collider_feature` and `collider_contact`.
Only capsule.rs and contact.rs differ from a70c0df; no public authoring commands,
CLI/MCP schemas, dependencies, core/GUI boundary or physical settings change.
Read-only source hashes match root's frozen candidate001 bindings:

| Source | SHA256 |
|---|---|
| contact.rs | f8019c9d5ad9e63e02bce8b7e67c11fe1881162aa14a460da8ca5c1dc76775b4 |
| capsule.rs | 904b16cd228126726f4dac5dc072c45148d7b78d5a08b13092aa3f518926e3da |

No product, test, arithmetic program, Cargo, model, network or GUI was run by
this review. Source/hash reads only; no source, HEAD, index, target or old
evidence changes. Prior reviews and candidate001 remain frozen.

## F1 — prevent a finite-input dot overflow becoming a silent endpoint

`capsule.rs:170–175` scales the axis but still dots it with an unscaled
`d=p-a`. Its sum can overflow even when the correct clamped fraction and
distance are finite.

Exact concrete input:

    B = 1e308
    a = (0,0,0), b = (B,B,B), p = (3B/4,3B/4,3B/4), R = 1.

The true point lies on the axis at fraction 3/4. The candidate evaluates
`dot(d,(1,1,1))` as +infinity and clamps its fraction to 1. It then reports
the finite wrong distance `sqrt(3)*B/4`. At `contact.rs:1057–1059`, N1 treats
that value as separated and returns None, rather than contact or an explicit
numeric failure. This is a silent finite-input error in the new helper,
outside the current 2^500 test range; root explicitly requested correction.

Root's proposed double-scaling formula is correct for finite usable deltas:

    as = max(abs(b-a)); e = (b-a)/as
    ds = max(abs(p-a)); dhat = (p-a)/ds
    v = dot(dhat,e)/dot(e,e)
    v <= 0          => t=0
    v >= as/ds      => t=1
    otherwise       => t=(v*ds)/as.

Handle zero as/ds separately. Algebra gives `t=v*ds/as`; the interior branch
ensures `0 < v*ds < as`, so that product cannot overflow. The two endpoint
branches avoid forming a potentially overflowing product. If `as/ds`
overflows, the axis is much larger than the displacement, and the interior
product remains safe. Underflow in that comparison does not create a positive
representable v smaller than the true positive ratio.

Also reject unusable nonfinite subtractions/features before N1's separated
filter. For example, finite p and a can have an unrepresentable p-a, and a
finite delta can have an unrepresentable Euclidean norm. `distance=+infinity`
must not become `depth=-infinity` followed by None. N3 already checks finite
distance/fraction at `contact.rs:519–520`; N1 lacks that check before filtering.
The new regression should exercise both valid extreme on-axis projection and
explicit failure for unusable arithmetic, with the old x/action unchanged.

Root/implementer own a separately bound candidate002 and any later execution.
Do not edit or reinterpret candidate001's 100-test record.

## Accepted source details

* **Current feature, retained anchor.** N1 selects x only for Capsules at
  `contact.rs:335–353`; Ground/Torso still query their own retracted anchor.
  N3 computes its fixed beta at current vertices and retracts that support's
  old actual nodal action independently (`contact.rs:510–536`). No support
  actions are merged or accumulated as positive-only repair.
* **Correct exact supporting plane.** `capsule_support_gap` uses translated
  `n dot(z-a)-max(0,n dot(b-a))-(R+thickness)`. This supports the entire closed
  capsule even when an endpoint/zero-distance fallback points along its axis.
  The helper and `Contact::replace` use the same normalization expression on
  the same raw normal. Tangential motion remains in that plane. At D>0 each
  visited point has exact-arithmetic plane clearance; later stages/supports
  can undo it, so final occupancy remains necessary.
* **History, release and pins.** Shared `contact_anchor`/`replace` are unchanged:
  pinned bits and zero pinned actions remain exact, old action is fully
  retracted, trials/actions are finite-checked before committing, and the
  stored action is actual trial-minus-anchor. Same-side supporting-plane
  separation removes both normal and friction action. Far-side feasible
  anchors deliberately retain a local branch; the explicit +X/-2X test
  documents this. The method is not a global metric prox or a CCD guarantee.
* **Zero distance and tiny axes.** `point_feature` uses a robustly normalized
  translated delta whenever nonzero, so the old 1e-20 point-axis substitution
  and 1e-12 direction cutoff are removed. `capsule::normal` is used only when
  the delta has no usable direction. Full-axis support makes any valid unit
  fallback geometrically conservative. F1 still needs the extreme-range guard.
* **Finite errors and cancellation.** N3 rejects nonfinite feature values;
  shared replacement rejects nonfinite gap/normal/weights/previous/trial before
  its local write. Whole-step x/histories remain transactional. Existing outer
  cancellation checkpoints and initial-repair provenance/eligibility policy
  are unchanged. The new point helper is constant-cost, without a new loop.
* **Shared-helper reach.** `axis_fraction` also serves scaled triangle/axis
  closest features and previous-side normal fallback. Its change can alter
  numeric results in discovery, diagnostics and initial-recovery geometry;
  “policy unchanged” does not mean those helper results are byte-identical.
  Existing conservative skinny-face/uncertainty code is retained. N4 finite
  triangle contact and the Torso radial law are not repaired by this patch.

## Independent oracle check

For two masses and the zero-target stitch, the physical quadratic is

    F(q+,q-) = K/2 * (|q+-y+|^2+|q--y-|^2)
                 + |q+-q-|^2/(2c),      K=m/h^2.

For the symmetric upper cap write `q±=(±R sin(theta),R cos(theta),0)` and
`y±=(±a,b,0)`. The test's angle derivative, apart from positive 2R, is

    K*(b sin(theta)-a cos(theta)) + 2R sin(theta)cos(theta)/c.

On `(0,pi/2)`, divide by positive cosine. The resulting
`K*(b tan(theta)-a)+2R sin(theta)/c` is strictly increasing. It is negative at
0 and positive at atan(a/b), so bisection finds its unique minimum on this
upper branch, not an arbitrary stationary root. The test independently checks
its positive normal reaction and current-normal KKT residual.

There is also a global quadratic certificate for this particular oracle.
Let `v=R cos(theta)>0` and its per-node outward force be

    ell = K*(v-b)*R/v >= 0.

Set `L=F-ell/(2R)*(|q+|^2+|q-|^2-2R^2)`. Its common/difference Hessian
eigenvalues are `K-ell/R=K*b/v>0` and `K+2/c-ell/R>0`. Hence L is strictly
convex and globally minimized at the oracle. For every pair outside the
endpoint sphere, `F>=L>=L_oracle=F_oracle`. Capsule exterior is a subset of
that sphere exterior, and the oracle's positive-Y upper-cap points are capsule
feasible. Thus this is a global minimum certificate for this exact two-point,
zero-friction quadratic case; it is not a full-garment/global-solver claim.

The friction oracle is also independent: for sphere R=1, anchor (0,0,1/2),
previous (1,0,1/2), mu=1/2 and effective D, q=(3/5,0,4/5) has normal reaction
`(3/5)/D` and saturated tangent magnitude `(3/10)/D`. Its normal+tangent action
times D is `(3/5,0,3/10)`, taking that anchor exactly to q. The N3 test chooses
previous nodal values with the same beta-interpolated previous point and uses
the true effective reaction `sum(beta*p)/D`. The first plane visit is retained
verbatim; complementarity/force are tested after the declared 32 visits.

## Frozen execution evidence read, not rerun

Candidate001 receipts bind the source hashes above and patch SHA256
638f8eb12306c288fc6e4f1f0448671d3af990308782f041b711cb47ab89af79.
The retained package output has 100 passed / 0 failed / 0 ignored; total
supervised time 21.017514208797365 s, aggregate observed peak RSS 629,686,272 B,
target peak 82,262,113 B, with the owned process group confirmed absent.

Its unchanged 32-pass coupling output gives 6.185808821904251e-18 m position
error and 1.2371621025418404e-9 N final force residual. Friction N1/N3 gives
3.46557221e-5 m position error, about 8.66e-6/1.07e-5 N force residual and
4.00e-11/4.93e-11 J gap-times-reaction, within the declared bounds.

Baseline001's retained diagnostic shows 17,320,512.4879 N mismatch to actual
history reaction despite sewing-block residual <=6.80192125e-9 N. Its separate
best-outward-cone residual is 8,660,255.5686 N. Preserve those distinct metrics.
The baseline diagnostic itself reported one passed test: convergence was
printed rather than asserted there. Its bad quantitative outcome is not an
invented failing Rust-test exit. Candidate001 adds the final equilibrium
assertions, retaining the same physical case/oracle and iteration count.

No other blocking source defect was established. F1 must be corrected and
reviewed before SOURCE_READY. Required integrated repository checks and any
future Harbor case remain root-owned and unexecuted by this review. Passing
this manufactured case does not establish Harbor causation, general nonlinear
convergence, CCD, N4/Torso correctness, calibrated fabric, native workflow,
reference parity or roadmap completion.
