# Original core source review and failure receipt

Reviewed source: `70897ee5378cb36edb9647b4c0e5fdcfb884c54a`, parent
`3c1760028dcbf5a3759907ded137b25a0714cf4a`.
Contract SHA-256:
`9baedd713b5a8dd4f251d7f2c5d0c51b408ecdabb32813a07dae06d25dcae960`.

Original Git blob IDs (measured while worktree was clean at this commit):

- sewing.rs: `1eb10d357359a60780bc3afcfcfd930423d14fb8`
- document.rs: `4610fd78455477bf5a73e8ec1a9d4de69593b1d7`
- simulation.rs: `51bc219e01904a4fd0d149da00a84970e2bf343f`
- persistence.rs: `733472fd0ab960c66a68d6f2c2ce804c5495ea10`
- Cargo.lock: `924cf7de6429fbcecc13d1d5770f7395a20648f8`

## Must fix 1: conditioning omits global parameter representation

Original source locations: sewing.rs:474 (condition guard), :317 (line inverse
return), :337 (cubic inverse return), document.rs:190 (CurveInterval::parameter).

The map guard uses the short selected path totals to bound f64 arithmetic. A
short interval near t=1 still converts its inverse into the full curve's global
parameter, so rounding occurs on the full curve scale. A steep map can amplify
this error beyond the declared 1e-8 m absolute paired-location budget.

Public-API reproduction is src/main.rs beside this receipt. Command:

```
CARGO_TARGET_DIR=.local/chain-core-review-20261114/target cargo run --offline --manifest-path .local/chain-core-review-20261114/Cargo.toml
```

Two rectangle bottom edges: A is 10 m, B is 1 m. Select A
`[0.99999997, 0.99999999]`, B `[0, 5e-6]`. Use map
`[[0,0], [0.5,0.2], [0.50000001,0.8], [1,1]]`.

Observed original run:

```
Accepted group: lengths [1.9999999989472883e-7, 5e-6], 3 portions
tA 9.99999980000000122e-1 tB 4.00000000000000067e-6
A arc fraction 5.00000008326672685e-1
B arc fraction 8.00000000000000044e-1
paired physical error 5.01998207145255645e-7
Mesh accepted: 6 stitches, 41481 vertices
Save serialization accepted
```

The independent oracle uses exact straight-line proportionality:
`u=(At-A.t0)/(A.t1-A.t0)`, `v=(Bt-B.t0)/(B.t1-B.t0)` and evaluates the authored
piecewise-linear map itself. It does not call production metric or map helpers.
At this A point the map gives B fraction 0.6996003585709489; production pairs .8.
The 5.02e-7 m discrepancy is about 50.2 times the contract budget. An earlier
otherwise identical run with B `[0,2e-7]` was accepted and measured
2.007992828581022e-8 m, also over budget.

The smallest sufficient fix is to bound full-curve/global-t and coordinate
roundoff, amplify that floor by correspondence conditioning, and reject inputs
whose promised accuracy cannot be supported. Keep construction/history/files
unchanged on rejection. The original failure and criteria must remain visible.

## Other reviewed areas

Read the entire sewing implementation and connected construction, validation,
meshing, editor/history, persistence, solver adapter, migration and group tests.
The contract, command/caller surface and library exports were traced.

No separate confirmed bug yet in the Simpson derivative remainder, compensated
path totals/local budget allocation, reversed/partial interval transforms,
overlap checks, ID bounds, strict version dispatch, old seam map semantics,
save staging or transaction rollback. Ownership cells explicitly resolve their
endpoints through each owning interval and reject lost positive span owners.

No reference executable, native gesture, physical gathering result or public
parity claim is evaluated by this source review. Full workspace gates were not
repeated; the core author's reported 58 tests/fmt/Clippy are not independently
relabelled as review evidence. This focused failure is a source correctness
finding, not reference parity evidence.
