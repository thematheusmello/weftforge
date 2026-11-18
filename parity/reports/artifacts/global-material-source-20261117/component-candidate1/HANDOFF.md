First frozen global material candidate: compile failure, no numerical regression.

The exact frozen source requires explicit f64 type annotation on the global
diagonal vector before it can compile (E0689 on hypot/is_finite). No test was
executed and no implementation, oracle, input, fixture, threshold or source
change followed this failure. Parent owns whether another candidate is allowed.

One Cargo command only, locked/offline actual1.94.1 tools, one job, empty Rust
flags/wrappers, own initially absent target. First environment seal additionally
pins RUSTUP_TOOLCHAIN, actual RUSTDOC and toolchain-first PATH. The exact original
RED test/patch/input/reference remain archived; source/adapter/88-file freeze,
runner/environment/argv and complete compiler output remain retained.

No full workspace, app, native, evaluator, garment, publication, commit or push
was run. All geometry/material/API/defaults/nonmaterial-stage equations remain
unchanged in source. There is no numerical GREEN or garment-cause claim.
