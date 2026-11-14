# Ordered sewing core review evidence

This subset preserves the independent review of core source `70897ee` and its
precision fix `9cd0a21`. The original short-span counterexample was accepted
while producing **5.01998207145255645e-7 m** paired error against a **1e-8 m**
budget. The fixed source rejects that unsupported configuration before commit.

`harness/` contains the unchanged original Cargo files and three review binaries.
`receipts/` contains byte-identical historical notes; their relative paths refer
to the original review directory. `results/` contains literal program-output
transcripts from the retained tool results, not newly captured complete process
logs. Compiler/invocation lines are excluded. The manifest records this
provenance, source identities, frozen inputs and original-to-public SHA-256 map.
No local-path replacement was needed in the retained files.

The old binary deliberately still unwraps creation. On fixed source its exit 101
with the exact precision-budget error is the expected rejection, **not a new
failure**. `precision_guard` separately checks successful safe input, intended
rejection and construction/Redo preservation in four line/cubic/direction cases.
`ownership` checks 68 disconnected cases. A build error, unrelated panic or
unsuccessful validation binary fails replay classification.

Run these commands from this package's directory inside a repository checkout.
Use the repository's Rust toolchain, Python 3 and cached crates for the offline
builds. New detached checkouts retain the exact relative dependency in the
frozen Cargo manifest. The two compiled targets stay separate.

```sh
set -e
review_package="$PWD"
review_repo="$(git rev-parse --show-toplevel)"
review_old="$review_repo/.local/chain-core-replay-70897ee"
review_fixed="$review_repo/.local/chain-core-replay-9cd0a21"
review_old_target="$review_repo/.local/chain-core-target-70897ee"
review_fixed_target="$review_repo/.local/chain-core-target-9cd0a21"
python3 "$review_package/check_replay.py" --self-test
git -C "$review_repo" worktree add --detach "$review_old" 70897ee5378cb36edb9647b4c0e5fdcfb884c54a
git -C "$review_repo" worktree add --detach "$review_fixed" 9cd0a215df4a5977b1b3263061a6cb98183eb2d4
mkdir -p "$review_old/.local" "$review_fixed/.local"
cp -R "$review_package/harness" "$review_old/.local/chain-core-review-20261114"
cp -R "$review_package/harness" "$review_fixed/.local/chain-core-review-20261114"
test "$(git -C "$review_old" rev-parse HEAD)" = 70897ee5378cb36edb9647b4c0e5fdcfb884c54a
test "$(git -C "$review_fixed" rev-parse HEAD)" = 9cd0a215df4a5977b1b3263061a6cb98183eb2d4
git -C "$review_old" diff --exit-code
git -C "$review_fixed" diff --exit-code
```

Build first, then run the binaries directly. With `set -e`, a compile failure
stops before the checker can classify any runtime result. Each build uses the
retained lockfile and its own target directory; no binaries are supplied here.

```sh
CARGO_TARGET_DIR="$review_old_target" cargo build --locked --offline --manifest-path "$review_old/.local/chain-core-review-20261114/Cargo.toml" --bins
python3 "$review_package/check_replay.py" original "$review_old_target"
CARGO_TARGET_DIR="$review_fixed_target" cargo build --locked --offline --manifest-path "$review_fixed/.local/chain-core-review-20261114/Cargo.toml" --bins
python3 "$review_package/check_replay.py" fixed "$review_fixed_target"
```

These are source-level numeric and ownership checks. They do not establish a
general interval proof, hardware bitwise identity, native gestures, physical
gathering, reference parity or a roadmap gate. The conservative guard can reject
extreme finite configurations. Historical receipts distinguish independent
checks from the core author's separately reported checks. Packaging ran only
integrity/classifier self-checks; publication replay remains separate.

This prepared local subset is not delivered evidence until every artifact is
tracked and the registry/map checks run from a clean checkout. Those publication
checks and root CLI/MCP/native integration remain outside this source review.
