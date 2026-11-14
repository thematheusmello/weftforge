Strain attribution: implementation visibility PASS, original garment quality FAIL

This evidence is pinned to runtime 6c45535cb312f7fed8ad872ee87b6ba5e5974e35,
under the prospective SIM-005 contract at 209a8fda72aaca90a08c330427354dd457224a6c.
It adds diagnostic visibility; original geometry, physics and acceptance remain.

The public-API regression retains exactly 181 frames with unchanged mesh,
positions, velocities and prior diagnostics, excluding added attribution and wall
time. All 180 meaningful peak locations/axes match independent reconstruction.
Native assembly displays Candidate back · Warp +24.12%; drape displays Candidate
front · Warp +32.35%. CLI fit reports IDs 31/2 at those stages, fails its unchanged
15% strain / 5 mm seam / 2 mm contact screen, and preserves the prior export.
One-step simulate/benchmark attribution is independently checked from saved OBJ.

Native fixed-17-decimal export tokens match the original control: parsed
positions/UV differ by at most 6.938893903907228e-18 m. This is not a native
velocity/private-memory serialization claim. Native seam/contact labels are
rounded; finite checks come from saved OBJ/control, not a finite-status AX label.
Original native PNG/source/OBJ and all numerical states remain unchanged.

headless/ and native/ are complete byte-identical closed input candidates,
including their original manifests. reviews/ retains the root's separate visual
receipt. CLI/binding/gate originals and six explicit pathname-only derivatives
are mapped in PROVENANCE.json with raw/published hashes and replacement counts.
Both actual executables are identified by retained original provenance: native
95fa1b7c… and CLI c7e63b52…. Binaries/Git metadata/targets are omitted. The source
snapshot's 37 hashes do not themselves reobserve an actual binary or Git state.

Recorded fmt, workspace tests, strict Clippy and packaging passed: 126 Rust tests.
Reference parity remains 0/56 coarse contracts. Force equilibrium, convergence,
calibration, general collision robustness and full SIM-005 remain unestablished.

Read-only preparation replay from any directory (no product/physics/SDK/UI/network):

    python3 -B PATH/TO/THIS/BUNDLE/verify.py --candidate-only

After root tracks every artifact in a clean publication checkout, strict replay:

    python3 -B PATH/TO/THIS/BUNDLE/verify.py

Default mode requires clean Git, complete tracked closure, current and committed
37-file source equality with 6c455, and the frozen contract. Candidate-only mode
explicitly skips that repository gate while retaining all artifact/source-snapshot
and frozen saved-output checks. Frozen headless/native verifiers and the portable
CLI checker execute; no application command or new simulation does.

All original simulation/CLI/build/check workflows are terminal. Native app PID
99058 was intentionally left open at completed Drape; replay does not touch it.
This ignored candidate and its preparation-time replay receipts are not new
publication. Root still owns tracking, clean-checkout registry/map verification
and public metadata. GitHub Actions and Pages remain disabled.
