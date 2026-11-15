# Closed A16 run review

Saved-record integrity and result consistency are verified for FINAL `9dbc231b433b539306d6229bed16bb056ab4920ece70d816bd3cb980da905397`. Strict physical quality remains failed. No discrepancy was found in the requested saved-output checks.

One exact GO binds corrected freeze `58733a8f994fee8fc866baa0e375b0b01c9032f9174ee409deba6cde747f33fe`; one ATTEMPT-001 and one base case are retained. Stage receipts agree with FINAL, with exits geometry 0, producer 0, consumer 2, comparison 0, and no stop reason. The consumer completed its analysis while returning the declared quality-failure code.

The reported outer controller exit 0 is producer/parent observed and consistent with pinned run.py plus FINAL status complete. No separate in-situ controller exit receipt was present in the reviewed packet. This provenance limit does not change the directly retained consumer exit 2 or failed-quality result.

All 181 trajectory states, 180 steps, 181 analysis rows and 181 comparison rows are present in sequence. Step settings and raw diagnostics match their states; saved rows report no issues, blocked verification or attribution disagreement. The exact rest/control gate holds: all physical metadata and 654 initialized masses match the retained A control after only two empty internal_curves arrays; rest OBJ bytes and initial positions, velocities and physical diagnostics match. The metadata has 654 vertices, 1,172 triangles and 48 stitches.

All 31 FINAL-bound output hashes match, as do 30 frozen files, 22 current source inputs, 19 retained inputs, all 38 original preparation files and two prior review files. The driver remains `1d6caa60cd3c95b860332fb2d06f1dfee919f7fd035233e4aa21fd9e948a8067`. Original preparation001 remains unchanged and unexecuted.

The whole-case clock is 285.18800870806444 seconds, below 420. Maximum stage peak RSS is 49,070,080 bytes. Numerical output is 19,011,398 bytes before FINAL and 19,018,367 bytes including it, below 268,435,456. Receipt clocks, output coverage and byte counts agree.

Saved rows confirm the new peak 0.1654700550371433 (16.5470055%) at frame 29, Harbor front/piece 2, warp, triangle 79. Saved strain failures are frames 29–30. The retained control peak is 0.19909194868993207 (19.9091949%) at frame 31, with failures 28–37; historical A/pair remain failed. Maximum saved same-frame absolute strain difference is 0.08070415883928184 (8.0704159 percentage points); maximum saved nodal difference is 0.02490758863765885 m. Both declared sensitivity triggers are true.

This supports numerical-path sensitivity for this one case. It does not establish convergence, a sufficient fix, changed production defaults, equilibrium, native/reference parity or contact completeness. Final-frame pass flags do not replace the failed trajectory result.

Skipped: every candidate/preflight/consumer/comparison/selfcheck/old-case rerun and physical regrading; no retained outputs changed.
Risk: strict quality still fails, and the outer controller exit has the provenance limit stated above.
