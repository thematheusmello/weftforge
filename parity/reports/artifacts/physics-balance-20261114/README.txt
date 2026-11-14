Original grain-shirt: current-source diagnosis, failed quality screen

This original engineering experiment is pinned to runtime source
3eddef067c8a1f03f192c8e6822a8da2a94f9bf9 and the unchanged grain-shirt input,
SHA-256 82e7c6d398c7772d7e62c28196dcf11845020020d44fca9ac9113e615b9537e7.
Limits remain strain <15%, seam <5 mm, supported contact <2 mm, finite state
and completed assembly.

The original current control fails: peak strain 24.1185% after 1 s zero-gravity
assembly, then 32.3519% after 2 s gravity drape. A fixed extra 2 s in zero gravity
instead ends at 32.1699%. Over exactly 2.5→3.0 s (frames 150→180), peak strain
rises 30.7541→32.1699%, maximum nodal drift is 3.62048 mm and membrane+bending
energy rises 0.00383853 J. The earlier reported range 30.8088→32.1699% explicitly
samples frames 151..180, beginning 2.5167 s; raw outputs are unchanged.

Omitting only mannequin capsules in memory reduces the 1 s peak to 0.242573%,
with mesh/material/grain/seams/ground/self-contact unchanged. This is a causal
diagnostic, never garment-quality acceptance or a replacement benchmark.
Initial prestrain and incomplete closure do not explain the 1 s failure.
Avatar-loaded assembly contributes to this trajectory. Geometric impossibility,
full force balance, static equilibrium, refinement/convergence, solver defect,
calibration and reference parity remain unestablished. Conservative energy
gradients omit contact/friction reactions and per-substep impulses.

From this directory, run the standalone read-only saved-output checks:

    python3 verify.py

No simulation, build, network, GUI or Git writes occur. The check verifies artifact
and source hash closure, all 423 snapshots/420 timing records, original failures,
the exact 61-snapshot assembly prefix, target history, independent energies/forces
and supported static contact checkpoints. It checks 37 source hashes directly;
the original clean-Git observation is provenance, not a new clean-checkout claim.

Plans/drivers/analyzers/source/input/states retain original bytes. Only 15
log/ledger files contain pathname-prefix-only derivatives; PROVENANCE.json
retains raw hashes and replacement counts. The initial Clippy failure and its
logging-only correction remain visible. Recorded jobs were terminal and recorded
wrapper/child PIDs absent at the saved confirmation. No live jobs remain.
The independent original-bundle review receipt/checker are separate under
review-support/. Invoking that checker here is producer replay and does not
replace the reviewer's own receipt.

MANIFEST.json closes this bundle. source-manifest.json binds runtime source;
producer-original-artifact-inventory.json retains original private artifact hashes;
CLOSURE.json states scope/exclusions. No binaries, targets or proprietary assets
are included. This ignored candidate is not public delivery: publication requires
Git-tracked artifacts and clean-checkout registry/map checks.
