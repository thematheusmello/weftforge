What this change does: The final packet binds the reviewed source and unchanged numerical protocol to the actual release producer. A separate outer supervisor is intended to cover controller exit, persisted root receipts and final clock/output checks under one420-second deadline.

Must fix:

1. **The final volume check can cross the deadline without rejection** (R1, `.local/material-polling-full-case-root-20261117/supervise.py:103-106`, reviewed SHA256 b34144b5ee7c85cac600ade25e184c49a54d7e0a2c30a48d0a92c009c9304831).
   - What this is: After fsync/hash persistence of ROOT-RESULT.json, the outer supervisor samples `closed`, then traverses the output files with `total_bytes()`.
   - Problem: If `closed` is419.99s and that final traversal finishes420.01s with bytes below the cap, the terminal verdict still uses419.99s and can exit0/2. The420-second contract includes final guards.
   - Fix: Compute final volume first, then sample `closed`, gate on those two values and include the measured final volume in stdout. Correct only the new outer supervisor and recheck that small delta; preserve the old case and final packet.
   - If skipped: A late final guard can be labeled operationally accepted. This blocks numerical GO.

The nine final static binding checks in CHECKS.json passed, reusing the prior19-check source review without rerunning its checks. FREEZE SHA256 5ad83bba74240553cee9d7b7e5d05ce82ee9e748e4b084f14ab9d02fdb9f823c covers exactly61 current packet files, with62 total including itself and five exact repository bindings. All19 executable source paths are unchanged. Source9d4b629c is clean, exact input/gold remain bound and the predecessor closure is unchanged.

The actual own-checkout producer is a regular executable MachO64 arm64,1331760 bytes, SHA256 17d8d1e5b624b3edb68c6c7e0b9fb8b20f85ac3e3c38cb34b8e5ffd419b5c989. BUILD-RECEIPT SHA256102da82b8d975d4ba08b18cf20a11cf1c097ca8db95c65734928b07051f97596 matches the distinct build-only GO, preflight/compiler binding and actual binary. Cargo10435 exited0 in26.710475s, with393379840B direct-child peak RSS,2018B retained build output and no stop reason. No constructor or step is claimed by this build evidence.

Final PROTOCOL SHA2565e4cf661bcb686e9d9e75275367ceb63590e40cd15a39b412872c43130b7b769 differs from the reviewed draft only by materializing the intended binary, Python, source/compiler bindings and exact command paths. Original geometry, masses, settings, two fresh initializations,180 steps/181 states, strict trajectory screens,420s/256MiB caps and no-retry/no-prefix/no-old-case-regrade limits remain intact. The13-key ROOT-GO schema matches validate_go. No actual ROOT-GO or numerical/output/witness namespace has started.

Apart from R1, the outer source traces the required path: start before controller launch; supervise and reap the controller under its earlier deadline; reject remaining owned groups; require operational exit0/2 and matching strict-quality flag; verify controller/final receipt hashes, complete181-state independent analysis and the source/binary freeze; hash numerical outputs; fsync/hash the pending root witness; count output volume and sample time; persist ROOT-RESULT; then emit a terminal verdict whose exit distinguishes quality pass0, quality failure2 and operational failure1. Persisted pending/root-result files alone are not sufficient acceptance: retain and require the final terminal stdout plus process exit.

Verdict: Fix R1 before issuing the one exact13-key numerical GO. Recheck only the corrected new outer supervisor and unchanged final hash inputs.

Not checked: No packet/outer import, helper, synthetic execution, build/test, model/init/step/analysis, UI, network or push ran in this review. G002 closure and idle CPU scheduling are root-provided state; G003 is unlaunched.
Risk: Whole-case timing and strict trajectory quality remain unproven; metadata freshness/xtask and linker/SDK/sysroot qualification remain pending.
