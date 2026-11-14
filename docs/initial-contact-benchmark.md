# Initial-contact initialization benchmark

For the original-shirt fixture at 0.015 m resolution, the public `create_cloth`
constructor's median elapsed time fell from **5.618584042 s to 0.056252833 s**
in this paired run: an observed **99.88×** ratio. Every trial reported exactly
**71,927 initial intersections** and **0.00536994076963222 m** maximum contact
error. The mesh had 15,459 vertices and 30,488 triangles.

This is implementation-only constructor timing, including initial static contact
diagnostics. The constructor also validates the project/mesh, prepares solver
input, and builds the complete cloth state. Fixture parsing, rest-mesh generation,
compilation, printing, and subsequent cloth destruction are outside its timer.
`initial_intersections` includes the existing analytic-collider and cloth-contact
diagnostic counts.

| Trial | Before, seconds | After, seconds |
|---|---:|---:|
| 1 | 5.598073500 | 0.057026500 |
| 2 | 5.618584042 | 0.055006042 |
| 3 | 5.624450458 | 0.056252833 |
| Median | 5.618584042 | 0.056252833 |

## Retained evidence

The [public artifact bundle](../parity/reports/artifacts/initial-contact-20261114/benchmark.json)
records source, producer, fixture, lockfile, compiler and measured executable
hashes, UTC timestamps, environment, exact counts, durations and derived medians.
Its [before stdout](../parity/reports/artifacts/initial-contact-20261114/before.run.stdout.log)
and [after stdout](../parity/reports/artifacts/initial-contact-20261114/after.run.stdout.log)
are unchanged raw process output, free of private filesystem paths. Empty raw
stderr files are also retained. The JSON is a public derived summary with private
paths omitted; it is labelled accordingly. Build logs and binaries remain private.
The earlier dirty-checkout timings are separate and are excluded from this bundle.

The fresh run used exact committed sources:

| Version | Commit |
|---|---|
| Before | `68bdb9fb774177f074f69edc1b2965336670cc9f` |
| After | `461f2e208b11aaa630c8a028e94d6a7d5ab80b56` |

Both checkouts kept their tracked sources unchanged. Their separately built
executables ran serially after both release builds had finished. The after
commit's complete `weftforge-core` and `weftforge-sim` trees match integrated
commit `9b2a8fc7ff895f6eada9ac8f39ac865e99eafe50`.

The measurement ran on Darwin arm64, macOS 27.2.0, with Rust/Cargo 1.94.1,
LLVM 21.1.8 and the checked-in release profile with thin LTO. There were three
trials per version, with no warmup or randomized execution order. Competing
Cargo/compiler/benchmark processes were absent at the pre-phase checks; large
mesh/simulation work was deferred. Tiny GUI actions and normal system activity
were permitted. These observations do not provide a confidence interval or
controlled background-load isolation.

## Reproduce from exact sources

Use a POSIX shell and the two exact commit objects above. Keep the retained
[producer](../parity/reports/artifacts/initial-contact-20261114/initial_contact_timing.rs)
unchanged. It must be copied to its original example path for its relative
`include_bytes!` fixture path to resolve. Require the exact commits; do not
substitute a moving branch head.

Verify the bundle with `sha256sum -c SHA256SUMS`, or macOS
`shasum -a 256 -c SHA256SUMS`, from its artifact directory. The included original
fixture must have SHA-256
`34bbc74a5ff78265fd503c2b586db859a56b9e985295f399673342aa304cb060`.
Both exact checkouts' `Cargo.lock` must have SHA-256
`6dd461ec03d344bc37acb7ff49a52cfabf4d0f60f723e86541bf8c979c7b29ba`.

Replace `bench_root` below with a **new** directory on a volume with room for two
release targets. Each checkout gets a fresh separate target; never share or copy
compiled targets between worktrees. Registry downloads may be shared. Run the
commands as one script with no concurrent Cargo, benchmark or heavy simulation
job. On Windows, use a POSIX shell and the `.exe` executable suffix.

```sh
set -eu
repo_root="$(git rev-parse --show-toplevel)"
artifact_root="$repo_root/parity/reports/artifacts/initial-contact-20261114"
bench_root="/path/to/new/benchmark-directory"
before_commit=68bdb9fb774177f074f69edc1b2965336670cc9f
after_commit=461f2e208b11aaa630c8a028e94d6a7d5ab80b56

git cat-file -e "${before_commit}^{commit}"
git cat-file -e "${after_commit}^{commit}"
mkdir "$bench_root"
git worktree add --detach "$bench_root/before" "$before_commit"
git worktree add --detach "$bench_root/after" "$after_commit"
mkdir -p "$bench_root/targets/before" "$bench_root/targets/after" \
  "$bench_root/before/crates/weftforge-core/examples" \
  "$bench_root/after/crates/weftforge-core/examples"
cp "$artifact_root/initial_contact_timing.rs" \
  "$bench_root/before/crates/weftforge-core/examples/initial_contact_timing.rs"
cp "$artifact_root/initial_contact_timing.rs" \
  "$bench_root/after/crates/weftforge-core/examples/initial_contact_timing.rs"
cmp "$artifact_root/original-shirt.weftforge" \
  "$bench_root/before/fixtures/original-shirt.weftforge"
cmp "$artifact_root/original-shirt.weftforge" \
  "$bench_root/after/fixtures/original-shirt.weftforge"
cmp "$bench_root/before/Cargo.lock" "$bench_root/after/Cargo.lock"

(
  cd "$bench_root/before"
  CARGO_TARGET_DIR="$bench_root/targets/before" \
    cargo build --release --locked -p weftforge-core --example initial_contact_timing \
    > "$bench_root/before.build.stdout.log" 2> "$bench_root/before.build.stderr.log"
)
(
  cd "$bench_root/after"
  CARGO_TARGET_DIR="$bench_root/targets/after" \
    cargo build --release --locked -p weftforge-core --example initial_contact_timing \
    > "$bench_root/after.build.stdout.log" 2> "$bench_root/after.build.stderr.log"
)

"$bench_root/targets/before/release/examples/initial_contact_timing" \
  > "$bench_root/before.run.stdout.log" 2> "$bench_root/before.run.stderr.log"
"$bench_root/targets/after/release/examples/initial_contact_timing" \
  > "$bench_root/after.run.stdout.log" 2> "$bench_root/after.run.stderr.log"
```

Save compiler versions, executable hashes and UTC timestamps with any new run.
Each executable already performs three constructor trials. Compare counts and
maximum errors exactly, then compute each median from the three printed
durations. Elapsed times and rebuilt executable hashes can differ across machines,
checkout paths and toolchains; the recorded binary hashes identify the measured
executables rather than prescribe a portable build hash.

## Limits

The acceleration applies to immutable initial contact/intersection analysis.
Live cloth collision projection and live step/reset contact diagnostics still
scan triangles quadratically. Fully overlapping initial bounds retain worst-case
quadratic candidate work, and capped partition work can yield coarser pruning on
adversarial input order.

This benchmark does not establish native responsiveness, step throughput, cloth
fitting, full simulation quality, or reference parity.
