# Cancellable request geometry — ready for root integration

Branch `feat/cancellable-triangulation`, worktree
`.local/cancellable-triangulation-20261116`, base
`89e6d53b9ab78b8c6642ebe2eae07e9f56f42da4`. This agent changed no index/commits,
launched/replayed no native case, and used only this worktree's ignored `target/`
with `CARGO_BUILD_JOBS=1`. No compiled artifact was copied across worktrees.

## Final checks — all pass, exact same 87 source inputs

| Check | Exit | Wall seconds |
| --- | ---: | ---: |
| New pending-refinement guard | 0 | 0.282 |
| cargo fmt --all --check | 0 | 0.795 |
| cargo test --workspace (385 tests) | 0 | 317.880 |
| cargo clippy --workspace --all-targets -- -D warnings | 0 | 61.240 |
| cargo build -p weftforge -p weftforge-cli | 0 | 14.708 |
| Final standalone earcutr library tests (30 tests) | 0 | 1.229 |

`CHECKS.json` retains full stdout/stderr, exact argv/environment, exit status,
UTC/wall timings, and all 87 before/after hashes per command. Every command and
all commands together use unchanged source. Canonical source-map SHA-256:
`cfd5019723e7639854790b40dbf028250e0922c261046a32794a8f40e6142dc1`.
Native debug binary SHA-256:
`c05bac974a4f8b40e2c5c46e4f857d5e00c270410f3a817f83439dbbb1f5d71e`.
CLI debug binary SHA-256:
`a7f23bd8876a04456e0d668e11eabefe54f510757664d0180cc17e03c51e058c`.
No Cargo process remains. Final gate sessions: 11517/6268/70912/10226, all closed.

## Owned implementation

- Minimal exact-version ISC earcutr fork: original notices, archive/member hashes
  and upstream commit in UPSTREAM.md; unchanged legacy source, original 27 tests
  retained. Ordinary earcut API preserved. Fallible hooks cover area/insertion,
  linked traversal/sort, both ear scans, holes/bridges, filtering/repair/splits.
  Callback failures retain exact caller errors and never return partial triangles.
- One fallible stable merge sort serves fork and controlled geometry passes.
  Validation sweeps, collinear recovery, refinement, legacy seam correspondence,
  weighted scans and dedup are controlled. New split tasks refuse before enqueue
  at 65,536 for offset context; duplicates and ordinary mesh policy stay intact.
- Immutable Cell-backed Work supports a single typed Transaction context through
  internal prepare_outline_offset_with_work. Source/output tasks, measurements,
  controlled bounds, target counts and final joins stay on that context, with
  cancellation before publication. Core Cloth adapter charges each record.
- Exact global exhaustion tests span holes/later pieces, validation+real sewing,
  repeated real preparation, nonlinear inverse and bounds. The actual public
  1,024-offset Transaction uses the fixed .4×.6/.02/1e-6 fixture, hits the global
  limit, and preserves byte-exact project plus full undo/redo behavior. Other
  tests cover >256-operation cancellation, independent areas/Euler/boundaries,
  real Cloth masses, unrelated holed/curved pieces and atomic batch refusal.
- Root-owned document/sewing and solver-agent changes are part of the successful
  final workspace gates. No command/schema/public authoring API changed.
- Dependency inventory source-only change is Cargo metadata's null for local
  earcutr; ISC remains, with no private path in public files.

Generated standalone `vendor/earcutr/Cargo.lock` was deliberately removed after
its final test (cleanup recorded separately in CHECKS.json). Intended vendor
source is exactly six files. No upstream unused fixtures/docs/caches were copied.
Earlier fixture-assumption failures were corrected to existing density and
hole/curve authoring rules; intermediate E0599 session63325 raced incomplete
constructor source and was superseded by passing combined checks. No finished
source failure remains. Complete earlier tool outputs/session handles are in the
conversation; final gate streams are retained in CHECKS.json.

Root owns clean committed xtask/map/registry checks, STATUS/metadata publication
and a fresh native successor. No reference parity, full PAT-009 acceptance or
hard cancellation-latency claim is made by these implementation checks.

## Owned final source hashes

| Path | SHA-256 |
| --- | --- |
| `Cargo.toml` | `032b32e80547618a72d39e42c0204611ff37908b8a6bf28615b3e592f79fa499` |
| `Cargo.lock` | `eb8a60650b473c79dda2e58a56f68b1abe9ca9a68c15115c9c8754c561732f3a` |
| `THIRD_PARTY.md` | `86d65f258938c55ed36d3a621222ef5124e348f5519d01eab8f61db540a4876e` |
| `docs/dependencies.json` | `31b44a3989e374510638df611ec403bfdd55bac31095bb9ce2ed66a685c07e2a` |
| `crates/weftforge-core/src/geometry.rs` | `3738282136928c459eaf524036c61e20d4c8b3cf2e04cd46ac812faf4f823c41` |
| `crates/weftforge-core/src/offset.rs` | `761881af22d784d0354ac25e6cda98971cb459bf78fd1d635eee21dc3e758203` |
| `crates/weftforge-core/src/simulation.rs` | `24e12aaf323f34abe745dc9c42c9823acbd6e2a1832d6827a5a6397112525707` |
| `crates/weftforge-core/tests/outline_offset.rs` | `9e5711fe91d8cb1b42818f7db241e94186110585cea8839eb312f01e6a0b3f69` |
| `vendor/earcutr/Cargo.toml` | `906d72d033764b39570416ef92906a90d4c49e85b9c35ab0138cf7a0aafe9c76` |
| `vendor/earcutr/LICENSE` | `ac0e50e523dd0ea36f077c373833f1ae6c7ad130ea5ac6ff23bfa8d3f402145f` |
| `vendor/earcutr/UPSTREAM.md` | `e686aeaa8758581d2c0659b1ed5e87dce4fe6608a8605c29ba90459495818944` |
| `vendor/earcutr/src/lib.rs` | `29992d5bd3d0717df6b4db9e60a3d280c276b67c3ddb87fa5de9173c1b3578d3` |
| `vendor/earcutr/src/legacy.rs` | `fd5120ba533484556e7a6d1a460d28084ac1f4c90f3334c71ff6f67976b933c8` |
| `vendor/earcutr/src/tests.rs` | `2e54cb4cceb9c073ae911a5909d2d9b430c6bf4ce53000332254233b21a544e9` |
