This is a fresh prospective Harbor A/base + B/height experiment, not a continuation or regrading of source077. METHOD.md was written before new binding code. PROTOCOL.json fixes workload, inputs, thresholds and limits. SOURCE-BINDING.json finalizes only the authoritative commit/hash fields from root's source-pretest receipt. Numerical helpers are unchanged; REUSE-PROOF.json records all unchanged evaluator and runner function ASTs.

Current source: 2e9795a3cb92c79ad3b406e675dcbdb49c60ce67. The prior42 runtime hashes match32f9; only the reviewed constraints.rs/lib.rs/tests.rs differ. Root's integrated wrapper90568/session86752 is terminal0 for fmt/package/177 Rust tests/all-target Clippy/xtask54 Python checks. GATES-BINDING.json pins the original receipts, logs and actual native/CLI binaries. These gates do not authorize the controlled cases.

Preparation-only commands, from repository root:

```text
python3 -B .local/closed-stitch-garment-protocol-20261115/preparation-selfchecks.py --output .local/closed-stitch-garment-protocol-20261115/FRESH-preparation-selfchecks.json
python3 -B .local/closed-stitch-garment-protocol-20261115/selfcheck.py --output .local/closed-stitch-garment-protocol-20261115/FRESH-numerical-selfchecks.json
python3 -B .local/closed-stitch-garment-protocol-20261115/runner-selfchecks.py
python3 -B .local/closed-stitch-garment-protocol-20261115/bindings.py inspect
```

Root's remaining producer preparation is separate. First write preparation-authorization.json from PREPARATION-AUTHORIZATION-TEMPLATE.json, granting only the build and two mesh-only calls; it grants zero dynamics. PREPARATION-RECIPE.md gives the commands. Copy all45 exact bound files to source/; use a fresh real target/ in this directory, independent of the integration target. harness/Cargo.toml and harness/src/main.rs are exact old instrumentation. reference/previous-harness-Cargo.lock is provenance only: root must prepare and bind the actual harness/Cargo.lock against the new source before the single locked build. Never overwrite a failed build/lookup receipt. Bind the actual new driver, build log, harness bytes and geometry results in a NEW PRODUCER-BINDING.json using PRODUCER-BINDING-TEMPLATE.json. The root producer receipt must truthfully record exactly two geometry-only executions and zero dynamics.

After producer preparation and root review:

```text
python3 -B .local/closed-stitch-garment-protocol-20261115/bindings.py verify-producer
python3 -B .local/closed-stitch-garment-protocol-20261115/finalize.py
```

finalize.py writes NEW FREEZE.json only after all source/build/geometry bindings pass. Root then writes a separate execution-authorization.json from AUTHORIZATION-TEMPLATE.json, setting authorized/explicit-root fields only after reviewing exact descriptors. The template, preparation freeze and producer receipts cannot authorize dynamics. No source or preparation file may change after PREPARATION-FREEZE.json; later producer/final-freeze/authorization/run artifacts are new receipts, not protocol edits.

The following command is execution-ready but MUST NOT be invoked until root explicitly authorizes finalized exact-source cases:

```text
python3 -B .local/closed-stitch-garment-protocol-20261115/runner.py --authorization .local/closed-stitch-garment-protocol-20261115/execution-authorization.json
```

It runs exactly base then height once; actual case folders must initially be absent. The existing runner functions preserve the420 s whole-case clock, exact Darwin wait4 peakRSS256MiB, total numerical-output256MiB, exclusive writes, stop semantics and quality-failure scheduling. No retry, tuning or continuation is implemented. Analysis exit0 is strict success, exit2 is arithmetic-complete quality failure, exit1 is an operational/binding/parser/resource failure. A quality failure may advance to the one declared height case; an operational failure stops. Final integrity validation is post-clock and cannot rerun numerical analysis.

The unchanged saved-state evaluator reads every boundary state and all raw diagnostics; final producer flags never replace its measurements. It checks exact settings/time/target recurrence, strict material-axis attribution, strain, seam gaps, full static triangle/capsule occupancy, analytic vertex/ground/supported self contact, uncertainty-policy/initial count agreement, and actual OBJ0/60/180 against the same saved states. It retains all failures/transients. Initial unclosed arrangement seams are separate from completed fit, while transient physical-limit failures remain failures.

Source077 remains strictfalse for both cases (A has33 attribution failures and10 strain-limit frames; B has3 attribution failures). Earlier A29 strain15.3559% and A/B surface9.0097/10.6724mm failures remain bound. New evidence has no native UI, equilibrium, calibration, moving-avatar, self edge-edge, ellipsoid surface, full CCD, release or reference-parity acceptance.

Not executed during preparation: new producer build, product geometry preflight or any native/headless controlled garment dynamics. Risk: the changed solver may fail any unchanged screen or resource cap; no favorable result is presumed.
