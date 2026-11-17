# Two-face material grouping: final uncommitted handoff

The reviewed independent shared-edge component changes from RED on unchanged production to GREEN on the first material-group candidate, with its original physical reference and 1e-8 m / 1e-6 N bounds fixed. Only the authorized constraints.rs/lib.rs/tests.rs/docs/physics.md change, based on exact ae8dd6ebe6da760501f507248a11b069a02cb453 in the sparse `feat/coupled-edge-regression` worktree. No commit, GUI/core/schema/API/Cargo change or old garment execution occurred.

## Evidence and actual executions

- Original immutable RED packet: sibling `.local/edge-proof`; FREEZE e2104078f7b99ffc6664eeba230f52490d85ae18da139a042a6104875cd37d76. Session 99626 / Cargo 83305 exit 101 after 12.327603 s: position error 4.2662743467e-7 m, independently confirmed force residual 2.7341292831e-3 N. Exactly one selected test, no retry. Source code, input, reference, result, log and proof remain exact.
- First-candidate same frozen regression: CANDIDATE-1-INPUTS.json SHA 63a93ca5c273540686fca02dd8aefe8d7381b2d08316ab49922ec5bc6d9ae663; session 69323 / Cargo 89003 exit 0 after 5.970246 s. Position error 5.6838907534e-18 m; direct physical force residual 1.0747913122e-14 N. Original test source 04f53392… is retained by that source map, separate from later tests/docs.
- Stabilization 1: fmt 4194 exit 0; strict sim Clippy 4252 exit 101 for a needless-range-loop warning in the new regression oracle. The only fix was test-only iter_mut/enumerate, preserving floating-point operation order, expected positions and bounds. Raw failed log/map remain; no full sim run started in that attempt.
- Stabilization 2 / session 57546: sim fmt 5022 exit 0; strict sim/all-target Clippy 5080 exit 0; full sim 5084 exit 0 (9.527781s including compilation). Actual raw summaries are 60 unit tests passed, 0 failed/ignored/filtered, plus 0 doc tests. Runtime test section 2.68 s is not a performance benchmark.
- All owned jobs are terminal. No workspace/native gate, garment producer, formal grader, old observer or reference-app run occurred. No target/cache/binary was copied. A later build replaces the unit-test executable at its usual own-target path; original RED binary hash is historical, not a claim that the old executable remains there. Source/input/log identities retain the reproducible RED.

## Equations and scope

Each internal material edge is ordered by its UV rest length (hypot, avoiding squared-length overflow); endpoint IDs resolve ties. Greedy matching assigns a triangle at most once. Matched groups and singleton leftovers are visited by smallest triangle ID. Every original material energy belongs exactly once; anisotropy, grain and per-face coefficients remain separate. Adjacent valid distinct manifold faces give four union nodes. No garment indices or quality settings enter matching.

For a group, p is its actual accumulated union-node displacement within the current substep, z=x-p, and E_g is the sum of its one or two unchanged material energies. The local objective is Phi_g(q)=(q-z)^T M(q-z)/(2h²)+E_g(q), using each union node's physical lumped mass once and fixed pinned coordinates. Its 3 / 6 current rows use:

```text
A = J W J^T + diag(1/(k h²))
A eta = J p - C
d = -p + W J^T eta
```

One diagonally scaled SPD Cholesky solve and the original at-most 16 Armijo fractions (coefficient 1e-4) update each group per declared outer iteration. A shared small const-sized implementation is instantiated only for 3 rows / 3 nodes and 6 rows / 4 nodes; no general solver framework/dependency/control was added. Accepted p uses the actual rounded position displacement. Invalid factors/non-descent/exhausted trials preserve that projection. Candidate group histories stay local to Cloth::step; every group polls cancellation. Constructor pairing candidate and group emission also poll, without an opaque sort.

Summing group corrections retains the material displacement identity while material energies appear once. Finite inter-group residuals can still remain. The existing scalar bend, seam, collider/cloth-contact and pin algorithms and order remain unchanged; the source Bend-and-below tail is byte-exact. This is not a full soft-stage replacement or complete coupled-contact equilibrium proof.

## Meaningful final checks

The final full sim run includes shortest-edge versus longer-edge pairing, equal-length stable ties, exact once-only face coverage and union mappings, mixed per-face density/anisotropic stiffness/grain against a directly evaluated independent incremental-energy gradient, bit-exact pins, and cancellation after the first strained pair before the next group with full state rollback/recovery. Existing constructor work/abort checks now include pairing work. The original shared-edge and earlier isolated material physical-energy regressions pass. No oracle value, force/position bound or garment screen was changed.

No production logic changed after the first focused GREEN: constraints.rs is byte-exact; lib.rs differs only in cfg(test) constructor polling assertions. Later tests added coverage/paired rollback and the one mechanical iterator lint correction; docs were updated afterward. The first-GREEN map is never presented as including these later additions.

## Cost and remaining risks

Matching is an extra temporary BTreeMap pass with O(E log E) construction and O(T) partner/group storage. Six-row factors cost roughly 4 times the cubic factor work of two three-row factors; dense Gram construction also grows, while paired energy work still visits two faces. This is an operation-count implication, not a measured slowdown or speedup. Correction arrays use 12 f64 per group: fully paired regions 12 per two faces versus 18 before, singletons 12 versus 9. Immutable union/mapping cache adds memory. One group plus its bounded trials can finish before cancellation is polled again; no hard native latency promise follows.

Pairing is a deterministic heuristic, not a convergence guarantee across meshes/materials. Existing nonconvex compression/degeneracy and finite line-search refusal remain. Full inter-group/bend/sew/friction/contact balance, fabric calibration and original Harbor 15% / 5 mm / 2 mm strict quality remain unproved. The component demonstrates the addressed coupling class; it does not grant native, garment, reference, full-feature or roadmap acceptance. Root owns final independent source review, commit and broader integration/prospective garment protocol.

Skipped: commit, broad workspace/native gates, new garment/observer execution, metadata/publication and any tolerance/count/default/material/mesh/contact relaxation. All retained failure histories stay visible.
