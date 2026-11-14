Bounded weak-warp garment refinement analysis

Historical source: 16e680ad774119e1ce3609b7d6c42c317f8677ef.
Original experiment declaration: 2026-11-14T11:53:26.408271+00:00.
This is implementation-only analysis. Every final case is FAILED under the
unchanged 15% material-axis strain screen. The 0.005 strain and 1 mm displacement
values are separate sensitivity triggers, never fitting acceptance limits.
Equilibrium, full convergence, a solver defect/fix and reference parity remain
unestablished. summary.json lists all five cases and their scope.

Package contents

harness/ contains the exact Cargo.toml, Cargo.lock and main.rs used in the run.
plan.json is the pre-run declaration with its exact local target prefix redacted.
runs/ holds the original API output: rest metadata and all sampled trajectories.
analysis.json and conclusion.json preserve the actual derived results unchanged.
provenance.json identifies the historical source, input and executed binary.
retained-artifact-hashes.json identifies the 15 existing public artifacts; their
raw reports, OBJ and images are deliberately not copied into this package.
job-ledger.json retains completed build/run handles, with exact local path
prefixes replaced by named placeholders. Both jobs exited 0; that means the
measurement completed, not that any fitting case passed. No job remains live.
MANIFEST.json maps original and published SHA-256 values. It also distinguishes
original executable/source hashes from hashes of redacted publication copies.
The original private/published=false labels in retained JSON describe its creation
time. The enclosing summary labels this historical publication subset.

Read-only verification of the retained evidence

Use Python 3.11+ and an existing checkout of the public WeftForge repository.
Set PACKAGE to this directory's absolute path and REPO to that checkout's path.
Run the following from a shell; choose an unused source path.

  git -C "$REPO" worktree add --detach "$PACKAGE/source" 16e680ad774119e1ce3609b7d6c42c317f8677ef
  python3 "$PACKAGE/verify.py" --source "$PACKAGE/source"

verify.py checks publication hashes, original source/artifact identities and all
recorded strain/drift calculations without stepping the solver or writing files.
The source worktree must remain clean. Do not substitute a later DXF-era source.

Optional reproduction of the declared simulation

This package preparation did not rerun the experiment. The following commands
are portable reproduction instructions for a future explicitly chosen run.
Use Rust 1.94.1, pinned by the detached source's rust-toolchain.toml. Keep CPU jobs
serial and use a new, exclusive Cargo target directory for this source worktree.
Do not share compiled targets with any other checkout. Registry caches may be
shared. Existing dependency versions/checksums are pinned in harness/Cargo.lock.

  export CARGO_BUILD_JOBS=1
  export CARGO_INCREMENTAL=0
  export CARGO_TARGET_DIR="$(mktemp -d "${TMPDIR:-/tmp}/weftforge-physics-XXXXXX")"
  cd "$PACKAGE/source"
  cargo build --locked --release --offline --manifest-path "$PACKAGE/harness/Cargo.toml"

If registry archives are absent, fetch the exact lock once with cargo fetch
--locked --manifest-path "$PACKAGE/harness/Cargo.toml", then repeat the offline
build. A registry fetch is not a new simulation or an alteration of the lock.
The OUTPUT_DIRECTORY below must not exist; the unchanged harness refuses to
replace an earlier output directory. Retain stdout/stderr and the process handle.

  python3 - "$CARGO_TARGET_DIR/release/physics-refinement-diagnostic" "$PACKAGE/source/fixtures/grain-shirt.weftforge" "$PACKAGE/runs-new" <<'PYRUN'
  import subprocess, sys
  subprocess.run(sys.argv[1:], check=True, timeout=600)
  PYRUN
  python3 "$PACKAGE/verify.py" --source "$PACKAGE/source" --runs "$PACKAGE/runs-new"

The exact harness contains a 120 s cancellation cap for each baseline phase and a
90 s cap for each continuation. The wrapper adds the original 600 s overall cap.
After timeout, inspect the retained handle/output; do not restart a live job.
No fixture, material, construction, tolerance or solver code is changed. Contacts
remain enabled throughout all runs. The three continuations clone the same t=3 s
positions AND velocities; none restarts assembly at the refined settings.

Reproducibility limits

The retained run was serial, optimized Rust on the original macOS arm64 host.
Binary bytes depend on build paths/toolchain/platform and are not a portable
reproduction requirement. Wall timings depend on hardware and competing load.
The harness asserts the original baseline max strain within 1e-12; cross-platform
floating-point differences may fail that identity check and must stay visible.
The checker uses 1e-12 only for recorded/calculated numeric consistency, not for
fitting acceptance. Rerun differences are reported, not used to overwrite the
retained results or weaken 15%. Slower hosts may hit the unchanged wall caps.

The stored native OBJ comparison was 6.938893903907228e-18m max nodal difference
with identical triangle indices; public source/artifacts remain required to
recheck it. Supported contact residual is not an independent full CCD/edge-edge
collision result. No static force-balance residual, full assembly-path refinement,
mesh convergence, calibration, reference execution or broader garment success
was measured. All GitHub Actions remained disabled; no production edits belong
to this experiment.
