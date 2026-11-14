# Original control strain attribution acceptance

This is a bounded read-only visibility regression for source `6c45535cb312f7fed8ad872ee87b6ba5e5974e35`, under the prospective SIM-005 contract at `209a8fda72aaca90a08c330427354dd457224a6c`. It is separate from the earlier three-case cloth diagnosis and does not extend that plan's case budget.

Exactly one new original-shirt control ran: 60 zero-gravity assembly frames followed by 120 gravity frames, dt 1/60 s, 8 substeps, 32 iterations, sewing closure 1 m/s. The original fixture, physical parameters and all original ground/mannequin/self/interpanel contacts stayed unchanged. No fresh no-avatar or extra zero-G case ran.

The old control driver source and Cargo lock are byte-identical. Only the two dependency pathnames in its Cargo manifest change from `../source/` to `../runtime-source/`. The plan froze the clean detached source's 37 file hashes, fixture, unchanged driver, original control outputs, independent checker and previously frozen peak expectations before any new simulation output. `binary-binding.json` subsequently bound the actual offline release binary before the run. Build and strict focused harness Clippy passed; the run finished exit 0 in 19.75 seconds under its 420-second cap. The source remained clean and all six recorded build/Clippy/run launcher and child PIDs were absent after completion.

The independent saved-output replay passed all 181 retained frames:

- Mesh, project, masses, sampled stitches and boundary associations are exact.
- Positions, velocities, settings and every prior diagnostic are exact after removing only the added `peak_strain` field and measured step duration. No wall times were compared across runs.
- All 180 physically nonzero peak triangle, axis and authored-piece associations agree with independent material reconstruction. The maximum signed-strain discrepancy is `2.220446049250313e-16`, below the prospectively declared `1e-12` arithmetic tolerance.
- Frame 0 remains roundoff-only. Its location happens to agree as well, although the contract does not assign physical meaning to a residual below `1e-12`.
- The original assembly and final quality failures remain failures under the unchanged 15% strain, 5 mm seam and 2 mm supported-contact screens.

| Frame | Solver triangle | Material axis | Signed strain | Authored piece |
| --- | --- | --- | --- | --- |
| 0 | 16 | weft | -6.661338147750939e-16 | 2, Candidate front |
| 24 | 31 | warp | +0.1637577026526289 | 2, Candidate front |
| 60 | 186 | warp | +0.24118490382884605 | 31, Candidate back |
| 180 | 31 | warp | +0.32351913672966237 | 2, Candidate front |

`review-support/replay_review.py` and `expected-peaks-preregistered.json` are unchanged copies of the earlier independent reviewer artifacts. The original expected file also contains old zero-G/no-avatar expectations; their presence does not represent new runs. The new `replay_acceptance.py` imports only that independent checker for material reconstruction, never product diagnostic code. `baseline-control/` holds the original source-3ed control outputs; `control/` holds the single source-6c regression. `acceptance-receipt.json` is the original reviewer's local acceptance receipt, including the observed source and actual retained binary verification.

Run `python3 -B verify.py` in the public bundle for complete artifact/source hash closure and saved-output replay. It launches no simulator. `runtime-source/` is the exact 37-file source snapshot, not a complete build checkout. The complete source commit and pre-run actual binary hashes remain recorded; compiled binaries and Git worktree files are deliberately absent from this public bundle. The public replay explicitly reports that it cannot reobserve the omitted original checkout or binary.

`PROVENANCE.json` identifies every path-only derivative of private logs/job ledgers, retaining original and published hashes. Measurements, saved states, registrations, source files, driver, checker, binary binding and original acceptance receipt are unchanged. Runtime-specific absolute path text in public log derivatives is replaced by `<acceptance-root>`; no original personal path is published.

This accepts original-control attribution visibility only. It does not establish fabric calibration, convergence, force equilibrium, full collision robustness, reference parity, native UI/CLI adapter acceptance or complete SIM-005 coverage. Root-owned evidence covers product adapters and full workspace checks separately. This ignored candidate still requires all artifacts to be tracked and registry/map verification from a clean checkout before publication.
