# earcutr 0.5.0 cooperative-work fork

Source: [earcutr 0.5.0 on crates.io](https://crates.io/crates/earcutr/0.5.0),
[upstream repository](https://github.com/frewsxcv/earcutr), commit
`c1897dd01de3c3d3d43631b6d36d283969804321` (the archive's `.cargo_vcs_info.json`).
The crate archive's SHA-256 is `afc28d89ac6cb489f679fcf51aaa62b4667bb0fb73a30f1d4d23949069707c07`.
The original ISC copyright and permission notices are retained in [LICENSE](LICENSE).

SHA-256 of the original archive members before local changes:

| Member | SHA-256 |
| --- | --- |
| `Cargo.toml.orig` | `7ed577ba3ba7f57039efcccbc73349e87802aaef4abaf1cbcc1ea895e3dff470` |
| `LICENSE` | `ac0e50e523dd0ea36f077c373833f1ae6c7ad130ea5ac6ff23bfa8d3f402145f` |
| `src/lib.rs` | `a59459f2a08030e29ae8ef86560d95e58714c86908e08b9d2ae4ea7086273f25` |
| `src/legacy.rs` | `fd5120ba533484556e7a6d1a460d28084ac1f4c90f3334c71ff6f67976b933c8` |
| `src/tests.rs` | `ad7ed3cc0a50b4e5db7042fc44ea285c3e46e5c09434b929d17491693eb1af4b` |

Local changes: a fallible cooperative callback is threaded through contour
construction, ear/candidate predicates, spatial indexing and sorting, hole
bridges, filtering, repair and splits. The hole queue uses a fallible stable
merge sort so cancellation returns immediately. `earcut` retains its original
signature; `earcut_with_control` returns the callback's error without partial
triangles. The same fallible stable sorter is exposed for WeftForge's controlled
geometry passes. The manifest omits upstream benchmarks, fixture suites and their
unused development dependencies. Upstream library unit tests are retained and
adapted for callback plumbing, alongside focused cooperative-work regressions.
`src/legacy.rs` is unchanged. No upstream fixtures, examples, benchmarks or caches
are copied.

The WeftForge integration charges each hook before its candidate/task and shares
its existing request counter across pieces. Callback success never changes a
geometric predicate or its traversal order; stable hole ordering preserves equal
finite x-coordinates. These are implementation regressions, not reference parity
evidence.
