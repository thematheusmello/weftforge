# Closed soft stitch operator records

This small packet retains historical implementation-only records for source
`c39401c173eb81111858e6b5ebfdfb165b27c365`, based on `c17d303e099836f022fe8f8222bc61b1cb33c3cd` and reusing the independently authored
`35afac604a93174fcdc2a5538799432aa6d69286` stitch update. It contains no garment-quality, physical-convergence,
calibration, reference-parity or integrated-release acceptance. The prospective
`spec/features/SIM-001-closed-stitch-coordinates.md` remains unchanged in Git.

## What was observed

- The final original solver run reported **51 passing tests**. Solver/all-target
  Clippy and final formatting also passed. These are bounded package checks;
  workspace gates/package are outside this packet.
- The exact generic N32 observer against the base scalar solver failed at the
  position assertion, with individual transport residuals
  `[0.3262304845504921, 0.09239588081082441]` N and stationarity
  `[0.1849692798866941, 0.3565047390259405, 0]` N. The expected RED run remains
  failed; its exit is 101.
- The original seven-test GREEN log records N32 transport residuals
  `[2.220446049250313e-16, 2.7755575615628914e-16]` N under unchanged 1e-12
  tolerances. That log predates a local variable rename and rustfmt; its production
  constraint and N32 observer bytes match final source. The later 51-test log
  covers final test/source edits.
- The independently authored direct harness compared **810 radial projections**
  bit-for-bit, preserving scalar lambda[0] and unused vector components. It covers
  270 fixed combinations over vertex/shared/repeated weights, hard and soft
  compliance with smallest-positive/normal targets, hard zero targets,
  free/mixed/all-pinned inverse masses, normal/zero/tiny gaps and finite/nonfinite
  denominator guards; each carries its scalar multiplier through three calls.
  It does not exhaust every possible floating-point input.
- The original source-scope checker and independent read-only review confirm the
  exact source boundary: remove only the vector dispatch and normalize its private
  lambda type/index to recover the complete base constraints file; normalize only
  seam allocation to recover base lib.rs. Contact, canonical strain, membrane,
  bending, cancellation/acceptance, integration and quality limits remain unchanged.
  The reviewer inspected these sources/logs and did **not** rerun the suites or
  harness.

No solver, harness, native or garment run was performed while preparing this
packet. Preparation checked input hashes, exact Git extraction and reversible
privacy transformations only. There is no new live replay certification.

## Records and privacy

`provenance.json` binds retained originals and public files by SHA-256 and byte
count. Safe files are unchanged byte copies. Other files are publication
derivatives made only by the enumerated literal own-path substitutions. Public
mapping entries give the inverse token, raw-literal hash/length and per-file
counts. The literal private prefixes and originals remain private, immutable,
outside this packet. Inverting with the owner's private mapping reproduced every
original byte and hash during preparation. No executable, Cargo target, runtime
binary, credential or account data is included.

The original formatting failure and first E0624 harness compilation failure are
retained. The first overbroad source-audit failure survives only in the original
summary: its raw log was empty. The baseline 42-pass run likewise has a terminal
summary, not a standalone raw log. Empty final-format output alone is not an exit
receipt; the original terminal summary records exit 0.

`manifest.json` inventories every packet file except itself. Its hashes establish
packet integrity, not independent execution. Source bindings use exact Git object
contents and deliberately omit two redundant full constraint snapshots.

## Reproduction inputs and assumptions

Keep the packet outside disposable worktrees. The repository must contain all
three exact commit objects above. Use fresh real targets per worktree; do not copy
or symlink compiled caches. Registry downloads may be shared. The recipes below
are instructions only and were **not executed during preparation**.

The old comparator is **c17d303e099836f022fe8f8222bc61b1cb33c3cd**, not the earlier 35af
point-contact branch. `sources/radial_equality.rs` imports only Material/Vec3 from
the built c394 library; both compared solve_stitch/project functions are exact
source includes. `sources/math.rs` is byte-identical at base and c394. The frozen
WeightedPoint/Stitch extraction is identical to c394/base except omission of Serde
derive metadata, letting the private position method remain accessible inside the
harness crate. Whole constraint modules compile, but only solve_stitch is called.

Set `$packet` to this packet's absolute directory. From a repository containing
the required Git objects, create disposable checkouts and an outside scratch
directory (these commands do not reuse any compiled target):

```sh
scratch="$(mktemp -d)"
git worktree add --detach "$scratch/candidate" c39401c173eb81111858e6b5ebfdfb165b27c365
git worktree add --detach "$scratch/base" c17d303e099836f022fe8f8222bc61b1cb33c3cd
cd "$scratch/candidate"
```

From that clean exact-c394 checkout, run the original checker as:

```sh
python3 "$packet/sources/source_scope.py"
```

It requires base/candidate Git objects and exactly the five-file c394 delta with
no untracked files. It intentionally does not certify a later integrated checkout
or an additional tracked evidence packet. To repeat the final solver checks:

```sh
export CARGO_TARGET_DIR="$scratch/candidate-target"
cargo fmt --all --check
cargo test -p weftforge-sim
cargo test -p weftforge-sim closed_elastic_stitch -- --nocapture
cargo clippy -p weftforge-sim --all-targets -- -D warnings
```

For the direct radial comparison, copy the small frozen harness sources to the
outside scratch directory, then extract the exact constraint Git objects:

```sh
cp "$packet/sources/radial_equality.rs" "$packet/sources/math.rs" "$packet/sources/stitch-types.rs" "$scratch/"
git show c17d303e099836f022fe8f8222bc61b1cb33c3cd:crates/weftforge-sim/src/constraints.rs > "$scratch/base-constraints.rs"
git show c39401c173eb81111858e6b5ebfdfb165b27c365:crates/weftforge-sim/src/constraints.rs > "$scratch/current-constraints.rs"
```

Verify both extracted hashes against `provenance.json`. Locate the one
`libweftforge_sim-*.rlib` in the fresh candidate target's `debug/deps`, then compile
and run the unchanged harness (replace `$rlib` with that exact path):

```sh
rustc --edition=2024 --test "$scratch/radial_equality.rs" \
  --extern "weftforge_sim=$rlib" \
  -L "dependency=$CARGO_TARGET_DIR/debug/deps" -o "$scratch/radial_equality"
"$scratch/radial_equality" --nocapture
```

To reproduce the RED observation, switch to the separate disposable base checkout
and set a distinct fresh target:

```sh
cd "$scratch/base"
export CARGO_TARGET_DIR="$scratch/base-target"
```

Insert the exact `sources/n32-observer.rs` fragment immediately
before `#[test]` for
`analytic_dihedral_gradient_matches_independent_finite_difference` in base
`crates/weftforge-sim/src/tests.rs`; make no solver edits. Run:

```sh
cargo test -p weftforge-sim \
  closed_elastic_stitch_n32_matches_independent_quadratic_and_individual_actions \
  -- --nocapture
```

Expected result is the retained position-assertion failure, not a pass. The same
unchanged observer exists in c394 for GREEN. Historical clocks, thread IDs and
compiler output ordering are not replay equality criteria. Future executed
reproduction must record its actual source/toolchain/commands, exits and results
separately; these instructions do not certify it.
