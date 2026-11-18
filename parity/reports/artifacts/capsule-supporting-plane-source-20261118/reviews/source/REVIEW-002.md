# Candidate002 narrow source disposition

**SOURCE_READY — F1 resolved; no remaining blocker found in the numeric delta.**

This supersedes only F1's open disposition in REVIEW-001.md. That report,
candidate001 and all preceding findings/results remain preserved. No product,
Cargo, arithmetic program, model, network or GUI execution, source edit, or
broad re-review was performed here; only the numeric patch, current source
sections/hashes, and retained candidate002 receipts/output were read.

Verified source bindings:

| File | Candidate001 SHA256 | Candidate002 SHA256 |
|---|---|---|
| contact.rs | f8019c9d5ad9e63e02bce8b7e67c11fe1881162aa14a460da8ca5c1dc76775b4 | 89d3d5f31dc2d6cb7ce5e207c070bb774aeb90ed36df50b9d0acb34fd2a8cac4 |
| capsule.rs | 904b16cd228126726f4dac5dc072c45148d7b78d5a08b13092aa3f518926e3da | ac833840d7568fd2deda6731325d4c33f29e1935df386bfeec7d1f206dd4e322 |

Numeric-only patch SHA256:
`4d94e3c520e2e1890515b4c86b047c2dc40da4df40c1f966b490013e601007fa`.

The correction matches the previously reviewed formula. `axis_fraction`
validates finite axis/offset differences, handles zero scales, scales both
vectors before its dot product, and selects endpoints before forming the
interior product. The interior branch keeps that product below axis_scale.
This fixes F1's valid large finite on-axis point without relying on an
overflowing dot followed by clamp.

For unusable geometry, the capsule feature rejects nonfinite fraction/depth
**before** the separated early return. Its nonfinite payload reaches existing
projection validation and returns `SimulationError::Unstable` before updating
x or its owned history. Static contact analysis now rejects nonfinite depth
instead of letting `f64::max` ignore it. N3's existing finite feature guard is
retained. The delta does not change the supporting-plane law, normalization,
pins, release, physical settings, iteration counts, or prior assertions.

The two added tests cover the exact B=1e308, fraction=3/4, distance=0 case and
both endpoint fractions; N1 reports its expected depth. They also cover invalid
inputs, overflowing subtractions and unrepresentable distance, requiring
projection failure without x/history mutation and diagnostic failure. These
are direct guards for the requested numeric correction.

The frozen candidate002 receipt/output records one package run: **102 passed,
0 failed, 0 ignored**, 20.77143937512301 s, with source/HEAD/diff unchanged.
Post-case cleanup records all 18 earlier artifacts exact and process group
79888 absent. The manufactured coupled position/force and N1/N3 friction
outputs match candidate001's retained values. This is read evidence, not a
new execution by this reviewer.

SOURCE_READY applies to this internal source increment and its declared
component scope. Current-side contact remains a local branch law: same-side
planes release; far-side clear anchors need not. Finite friction is not an
exact one-visit nonlinear solve. General convergence, full surface discovery,
N4/Torso correctness, CCD, calibration, Harbor improvement, reference parity
and roadmap completion remain unproved. Integration and required repository
gates remain root-owned.
