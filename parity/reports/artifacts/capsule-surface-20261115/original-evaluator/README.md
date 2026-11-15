This private preparation package evaluates actual saved original Harbor states.
It never launches a product, a build or a garment producer. METHOD.md is the
prospective measurement contract; original evidence and source files are read-only.

Before physical execution, root must review the evaluator, freeze it and issue a
separate authorization binding the v2 plan, v2 producer binding, source/input
binding and evaluator FREEZE.json. A preparation document is refused as authority.

Commands, from repository root:

```text
python3 -B .local/capsule-garment-evaluator-20261115/selfcheck.py --output FRESH_PRIVATE_JSON
python3 -B .local/capsule-garment-evaluator-20261115/benchmark.py --output FRESH_PRIVATE_JSON
python3 -B .local/capsule-garment-evaluator-20261115/evaluator.py analyze --case base --authorization AUTH_JSON --receipt PRODUCER_RECEIPT_JSON --output FRESH_ANALYSIS_JSON
python3 -B .local/capsule-garment-evaluator-20261115/evaluator.py validate-receipt --case base --authorization AUTH_JSON --receipt FINAL_CASE_RECEIPT_JSON --output FRESH_AUDIT_JSON
```

Selfchecks are static analytic/mutation checks and the benchmark reads only old
already saved gold trajectories. Benchmark output names are fixed/exclusive;
neither command may overwrite prior output. Real analysis uses runs/base or
runs/height beneath the bound experiment; the fixed schema is documented below.

Analysis exit0 means arithmetic completed and strict delivery screens passed;
exit2 means arithmetic completed with failed quality; exit1 means structural,
binding, resource or parser failure. No code retries or changes physical settings.
Validation exit0 means bound execution receipt checks passed, exit2 means the
receipt is bound but contains failed resource/execution flags, and exit1 is refusal.

The analysis records every frame, raw diagnostics, strict independent thresholds,
exact peak attribution failures, blocked verification and failed frame lists.
`all_frame_quality_passed` includes initial unclosed arrangement rows and therefore
does not establish completed fit. `trajectory_physical_screens_passed` requires
finite/strain/contact/both uncertainty-zero screens for every frame.
`completed_fit_all_frame_quality_passed` screens every frame after target closure.
`strict_delivery_passed` additionally requires exact targets at60/180, all required
diagnostic checks, complete capture and no execution/resource/binding errors.
`quality_status` always says passed or failed; final-only quality stays separate.

Reported conservative-policy capsule counts and independent unresolved oracle
pairs are distinct. An all-pair well-conditioned zero proof may verify the former;
unproved policy geometry blocks diagnostic acceptance. Initial intersection
threshold ambiguity is a separate unknown. No unknown is relabelled penetration.

Authorization JSON requires exact integer schema1 and attempts_per_case1,
status authorized, source_commit, cases `["base","height"]`, and descriptors
`{path,sha256}` for plan, source_input_binding, producer_binding, evaluator_freeze,
and runner (the exact experiment/runner.py path and bytes).
Paths are relative to repository root or absolute within its private experiment.
Required plan is PREPARATION-PLAN-v2.json; producer is PRODUCER-BINDING-v2.json.

Producer receipt JSON requires schema1, case, attempt1, source_commit,
input_sha256, driver_sha256, evaluator_freeze_sha256, authorization_sha256,
start_monotonic_seconds, producer_end_monotonic_seconds, producer_peak_rss_bytes,
numerical_output_bytes, producer_exit_code, status and artifacts `{name:sha256}`.
Complete capture requires rest-metadata.json, trajectory.jsonl, steps.jsonl,
stage-000.obj, stage-060.obj, stage-180.obj and completed-case.json; retained logs
may also appear. Failed/stopped producer receipts can preserve/analyze an accepted
prefix only while time remains and never pass execution acceptance.

Final case receipt retains those fields/artifacts and adds final_monotonic_seconds,
whole_case_elapsed_seconds, analysis and inspection `{path,sha256}` descriptors.
Status complete means producer/arithmetic completion, with quality reported
separately; status failed retains operational failure. Whole-case420seconds and
producerRSS256MiB are separate from aggregate numerical-output256MiB across
inputs, preflight-base, preflight-height and runs, including receipts/logs/reports.
Historical receipt volume is not equated with later larger aggregate volume.

Final integrity audit receives the final case receipt and a NEW output path,
for example runner-audits/base-validation.json. It verifies the recorded numerical
clock and analysis/file bindings without rerunning numeric analysis or overwriting
analysis. It can occur after the numerical clock; new numeric analysis cannot.

Prior A29 strain15.3559% and A/B final surface9.0097/10.6724mm remain failures.
Not checked here: new garment execution, UI, workspace/Cargo gates or parity.
Risk: static contact checks do not establish convergence, full CCD or calibration.
