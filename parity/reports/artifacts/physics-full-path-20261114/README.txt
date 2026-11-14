Bounded full assembly-path weak-warp diagnostic

This is a local publication candidate, not delivered or registered evidence.
No repository source, fixture, production setting, registry or documentation was
changed. GitHub Actions and Pages were not enabled. All recorded jobs terminated.

Source: e342471b18bc5d636d8e869ba0d374710ea15da3, clean detached checkout.
Original fixture: fixtures/grain-shirt.weftforge
Fixture SHA-256: 82e7c6d398c7772d7e62c28196dcf11845020020d44fca9ac9113e615b9537e7
Prospective declaration: plan.json, frozen before any new solver output.
Plan SHA-256: c3e984a440fff1ce3b43c8059164221fa09392d0c368a703ac7c4a0dcfb642fb
The source-manifest hash is frozen in that plan; source-manifest.json enumerates
the exact native core/solver/manifests/lock/toolchain bytes. retained-inputs.json
enumerates the original native OBJ and prior public diagnostic inputs by their
source-relative paths and SHA-256. No historical artifact was overwritten.

Question and fixed protocol

Does refining the complete assembly and drape path change the weak-warp strain
sensitivity seen in the historical continuation-only comparison?

Each of three cases constructs a separate fresh Cloth from the same original
arrangement, zero velocities and original seam targets. All run 60 frames of
explicit zero-gravity assembly followed by 120 frames with (0,-9.81,0) gravity.
dt is 1/60 second and seam closure is 1 metre/second throughout. The cases are
8 substeps/32 iterations (control), 8/64, and 16/32, run serially. Geometry, mesh
resolution, stiffness, grain, damping, friction, thickness, seams, mannequin and
all supported contacts stay unchanged. No production fix or tuning was attempted.

The unchanged engineering screen requires finite state, zero assembly target,
maximum absolute material-axis strain below 15%, seam gap below 0.005 metre and
supported contact penetration below 0.002 metre. The separate 0.005 strain delta
and 0.001 metre nodal displacement triggers measure numerical sensitivity; they
are not fitting tolerances.

Results from independent geometry

                    At 1 second         At 3 seconds          Final screen
8/32 control        25.969213%           26.930346%            FAILED: strain
8/64 full path      26.126438%           27.168563%            FAILED: strain
16/32 full path     26.546938%           26.698742%            FAILED: strain

Before gravity, the 16/32 case differs from its same-time control by +0.00577725
strain and at most 11.0871 mm in node positions; both sensitivity triggers are
exceeded. The 8/64 case differs by +0.00157225 and 0.4943 mm at that time.
At three seconds the 8/64 and 16/32 strain deltas are +0.00238217 and -0.00231604,
below the strain sensitivity trigger, while maximum nodal differences of
2.5469 mm and 23.7484 mm both exceed the displacement trigger. Their respective
mass-weighted RMS nodal differences are 0.9343 mm and 6.3892 mm.

The control matches all 13 shared historical baseline position/velocity samples.
Its final native-original OBJ comparison has 6.938893903907228e-18 metre maximum
nodal difference, zero independently measured strain difference and identical
triangle indices. The independent oracle reconstructs deformation gradients
from UV coordinates, triangle indices and positions, rotates by authored grain,
and measures signed warp/weft strain. It calls no solver code. Hand-derived
axis, energy, unequal-mass drift, weighted-seam and capsule checks passed before
solver output. Independent mass, kinetic, sampled-stitch and analytic capsule/
ground calculations are retained too. The solver contact residual remains
separately labeled; no independent full cloth collision proof is claimed.

Historical refined continuations begin at the unchanged t=3 second baseline.
Their t=4 second iteration/substep strain deltas versus their own t=4 control
were -0.00082568/-0.00522653, with 2.1115/3.8959 mm maximum nodal differences.
These are different terminal times and paths. The new full-path t=3 values must
not be presented as a same-time convergence sequence with those old endpoints.
The new evidence demonstrates sensitivity already during assembly and persistent
path differences during drape. It does not establish that numerical error causes
all of the large soft-warp strain. Both assembly and drape were refined, so final
differences do not isolate assembly alone.

No force-balance residual, static equilibrium, general convergence, calibrated
fabric, reference execution or parity was measured. All three final fitting
cases remain failed. Small seam/contact residuals do not cure strain failure.

Retained files and immutable originals

harness/ contains the exact API driver, Cargo.toml and locked dependencies.
runs/ contains original rest metadata, 39 sampled states (every 15 frames,
including t=0/1/3 seconds) with positions AND velocities, all 540 measured step
timings/diagnostics, and completed-case phase timings. The total measured run was
110.90173 wall seconds; control/8x64/16x32 cases used 22.37305/43.80950/44.44584
wall seconds on the recorded macOS arm64 host. Timings include file/log overhead
and competing system load; they are not isolated-machine performance results.

measure.py is the independent oracle frozen in the plan. analyze.py calculates
all retained measurements without stepping the solver. analysis.json contains
every sampled result and same-time comparison. summary.json and conclusion.json
give the terminal interpretation. provenance.json records source/toolchain and
the original executed binary hash; the binary itself is omitted because compiled
bytes can contain local checkout paths. job-ledger.json and process-confirmations
retain actual completed handles, exit codes and final absence checks. Build/run
both exited 0, meaning measurement completed; no fitting pass is implied.

The experiment's original byte hashes are in original-artifact-hashes.json.
MANIFEST.json maps each local original path/hash to its publication-copy hash;
only path-bearing job/build/clippy text is redacted with <EXPERIMENT_ROOT>.
The raw states, declarations, exact API driver, measurements and derived numeric
results are unchanged. Private/local labels in original declarations describe
their creation time. Public-safe copies remain staged locally until an owner
tracks every artifact and passes registry/map checks from a clean checkout.

Read-only verification, without simulation

Use Python 3.11+ and an existing public repository checkout. Set PACKAGE to this
directory's absolute path and REPO to that checkout. Choose an unused source path.

  git -C "$REPO" worktree add --detach "$PACKAGE/source" e342471b18bc5d636d8e869ba0d374710ea15da3
  python3 "$PACKAGE/verify.py" --source "$PACKAGE/source"

The checker verifies publication hashes, frozen declaration/oracle/source/input
identities, clean detached source, all recorded calculations, native-original
identity and same-time sensitivities. It writes no files and never runs solver
steps. The source must remain clean. No network access or new installation is
required when the source and Python are already available.

Optional fresh reproduction of the exact three cases

No new simulation is performed by this packaging or verification step. A future
explicitly chosen replay needs the already-installed Rust 1.94.1 toolchain and
the pinned registry packages already cached. Stop if they are unavailable; do not
silently substitute dependencies, settings, source or thresholds. Use an exclusive
new Cargo target for this source worktree. Registry caches may be shared.

  export CARGO_BUILD_JOBS=1
  export CARGO_INCREMENTAL=0
  export CARGO_TARGET_DIR="$(mktemp -d "${TMPDIR:-/tmp}/weftforge-full-path-XXXXXX")"
  cd "$PACKAGE/source"
  cargo build --locked --release --offline --manifest-path "$PACKAGE/harness/Cargo.toml"
  python3 - "$CARGO_TARGET_DIR/release/physics-full-path-diagnostic" "$PACKAGE/source/fixtures/grain-shirt.weftforge" "$PACKAGE/runs-new" "$PACKAGE/replay.log" <<'PYRUN'
  from pathlib import Path
  import subprocess, sys
  with Path(sys.argv[4]).open("x") as log:
      proc = subprocess.Popen(sys.argv[1:4], stdout=log, stderr=subprocess.STDOUT)
      print("confirmed solver PID", proc.pid, flush=True)
      try:
          code = proc.wait(timeout=600)
      except subprocess.TimeoutExpired:
          proc.kill()
          proc.wait()
          raise
      print("solver exit", code, flush=True)
      assert code == 0
  PYRUN
  python3 "$PACKAGE/verify.py" --source "$PACKAGE/source" --runs "$PACKAGE/runs-new"

runs-new and replay.log must not exist; the driver refuses to replace outputs.
It enforces a 120 second cancellation cap per phase; the wrapper adds the original
600 second overall cap. Retain the printed PID, log and exit before considering
any retry. The checker reports rerun differences without overwriting originals.
The 1e-12 identity/consistency checks are not fitting or convergence tolerances.
Cross-platform floating point, hardware load and wall caps can affect replay;
binary byte identity is not a portable reproduction requirement.
