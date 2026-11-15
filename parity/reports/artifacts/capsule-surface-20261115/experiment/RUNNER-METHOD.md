# Capsule runner preparation method

This private preparation adds only `runner.py`, this method, `runner-selfchecks.py`
and new stub selfcheck evidence. It does not run the garment driver, Cargo, native
UI, Git, or any physics. The runner has no build, tuning, retry, resume, or third
case mode. Source, inputs, driver, harness, prior artifacts and public files remain
read-only. Root must review the runner and frozen evaluator, then separately write
an execution authorization before the real runner can launch anything.

## Binding and scope

The only actual invocation is `python3 -B runner.py --authorization PATH` (an
absolute authorization path, or a path relative to this experiment); cases
are fixed in order `base`, `height`, one exclusive attempt marker each. A base
operational/structural/refusal/budget/producer failure stops the invocation and
leaves height unattempted; the authorization cannot resume or retry. Base quality
failure after all 180 steps and structurally complete independent arithmetic
(analysis exit 2) is retained and height still runs. Existing markers, output directories, logs,
receipts or reports cause refusal; missing records are never fabricated. The
driver itself must create the initially absent `runs/CASE` directory.

Before consuming an attempt, verify a real private experiment directory with no
symlinks, Darwin `wait4`, plan SHA
`d76f668d21d2837b41574b2d403f7ed6ed24f873265e56cbcbb5e666280034e6`,
source/input binding SHA
`d313af247e05b1e331c6982d3201079a13a9bf1890fb9ad02cb60f1d4944e055`,
producer binding SHA
`68ca7dcd05f4a443fe71f76fbd0accef9c60249befec4e1638c9065e77534c9a`,
the bound 40 source files, two inputs, harness/build descriptors, preflight files,
and driver SHA
`7f01a3355385af0ed8df2accf2c2a6fbf9cebc2af71f391f259ed44a9db48f34`.
The fixed settings remain 180 steps, assembly 60 plus drape 120, 1/60 s,
8 substeps, 32 iterations and the plan's gravity/closure values.

Authorization is a strict JSON object: schema integer 1, status `authorized`,
source commit `0770204a54abdf5f929b6626b01d7fcfb0e13ca4`, cases exactly
`["base", "height"]`, attempts_per_case integer 1, and
`explicit_root_authorization: true`. Required top-level descriptors `plan`,
`source_input_binding`, `producer_binding`, `evaluator_freeze`, and `runner` each
contain `path` relative to the repository and `sha256`. Paths must identify the
expected files. The evaluator freeze must have schema 1, status
`frozen_preparation_no_garment_execution`, and a nonempty `files` map of its frozen
method/source/evidence hashes, source commit, no-garment-execution flag, and exact
plan/source-input/producer/readonly-input binding descriptors. Every frozen file
is checked; unfrozen Python sources/caches in the evaluator directory are refused.
Authorization and all
bindings are checked again after producer and numerical inspection; a changed or
moved file fails the case. No Git command supplies acceptance evidence.

## Process and clock ownership

The shared Python `time.monotonic()` origin is recorded immediately before actual
producer `Popen`. The 420-second case budget includes producer launch/execution,
log copies, all evaluator arithmetic/exports, hash reading and inspection. Every
child starts a fresh process group. A monitor owns `os.wait4(pid, WNOHANG)` and
`wait4(pid, 0)` exactly once; it never calls `Popen.poll()` or `Popen.wait()`.
It sets `Popen.returncode` from that wait status to prevent destructor reaping.

The producer's final `ru_maxrss` is interpreted in Darwin bytes and must be at
most 268435456. Sampled `/bin/ps` RSS in KiB provides an early stop; it cannot
replace exact `wait4` peak RSS. Wall expiry, sampled RSS, output budget, monitor
error or interruption kills the entire fresh group with SIGKILL, then reaps it.
Producer exit alone is insufficient: exact peak, whole-clock and byte caps are
checked again. Evaluator arithmetic is monitored under the same deadline and
volume budget. No arithmetic starts or continues after a closed/expired clock.

Actual stdout/stderr begin in new `runs/.producer-logs/CASE.*` files, outside the
initially absent case folder. After the producer, their actual bytes are copied
exclusively into the case directory when it exists. Originals and copied logs are
both counted and preserved. If the clock/output budget has already closed, the
original logs remain retained and missing case log copies stay visibly missing.
Partial driver files, stderr and failure receipts are retained after any failure;
the runner does not invent completion or state files.

## Numerical volume and receipts

The independent aggregate cap is 268435456 bytes across **every regular file** in
`inputs`, `preflight-base`, `preflight-height`, and `runs`. This includes original
and copied logs, attempt markers, producer/final receipts, independent analysis
rows/exports and inspection. Source, binary, compile inputs, authorization and
freeze files live outside these roots and are not numerical outputs. Posthoc
integrity-only logs/reports in `runner-audits` are also outside the numerical
roots; they do no arithmetic or numerical export and add no acceptance evidence.
Nested
symlinks or nonregular entries fail closed. Hardlinked files are counted per path.

During a child, the stop budget leaves 65536 bytes for failure receipts and the
known pending copied log bytes. This is a conservative early stop, not an increased
cap. Monitoring is sampled; a child can overshoot between samples. Overshoot is
retained and explicitly fails exact final acceptance. No deletion disguises it.

JSON is encoded deterministically. Each receipt's `numerical_output_bytes` is a
fixed point of the existing aggregate plus that receipt's own final encoded size.
The producer receipt records the historical volume at producer observation; the
final receipt records the final aggregate including itself. No self-hash appears
inside its own receipt. Other artifact descriptors use repository-relative paths
and SHA-256. A final inventory excludes only the inspection and final receipt
which do not yet exist, and states those pending paths explicitly.

Producer receipt schema 1 records case/attempt/source/input/driver/freeze/auth
hashes, monotonic start/end, exact peak RSS, exit code, stop reason, actual artifact
hashes, and status `producer_complete`, `producer_failed` or `stopped`. Missing
driver output stays missing. Failed attempts also get a final `failed` receipt.

For a producer eligible for analysis, invoke the frozen independent evaluator:

```
python3 -B evaluator.py analyze --case CASE --authorization PATH \
  --receipt runs/CASE/producer-receipt.json --output runs/CASE/analysis.json
```

Exit 0 means arithmetic complete and quality passed; exit 2 means arithmetic
complete with quality failure; exit 1 means structural/refusal/cap failure. The
runner preserves the independent quality flags without deriving a new threshold.
It writes inspection and final receipts only after analysis, export, hash and
binding checks within the case clock. Final status `complete` means all caps,
bindings and structural arithmetic completed; it does not imply quality passed.
All individual flags and failures remain visible. Exit 2 for the runner means
both cases completed arithmetic but at least one failed quality; exit 1 means an
operational/structural stop, with no continuation under the same authorization.

The logical numerical clock closes after all numerical analysis, exports, hashes,
inspection and post-bindings. Final receipt serialization/self-size accounting
is integrity metadata after that close. Immediately after writing it, observe
actual elapsed time and aggregate output bytes again. Any late write or output
excess is preserved in a separate failed integrity report and stops scheduling;
the case receives no within-budget credit, even if its logical receipt had status
`complete`.

After that close, `validate-receipt` receives the final receipt and writes a fresh
`runner-audits/CASE-validation.json`, with its logs alongside. This is a read-only
integrity audit with a separate 60-second process timeout; it cannot change the
analysis, add numerical results, reopen the case clock or turn a failed case into
accepted numerical evidence. The runner also verifies the original analysis and
final receipt bytes stayed unchanged. Integrity failures fail the invocation.

## Stub-only checks

`runner-selfchecks.py` imports monitor/helpers without calling the runner main.
It creates a new private temporary selfcheck directory, launches only short
Python children, and retains a new report. Check successful and failed exits,
exact `wait4` peak accounting, wall stop and whole-group kill, sampled RSS stop,
aggregate output stop, non-overwrite/no-symlink refusal, strict missing/invalid
authorization refusal, receipt self-size arithmetic and the quality-fail versus
operational-stop scheduling distinction. Resource-stop checks use small declared
test limits, not real experiment thresholds. No real driver command is passed to
the monitor. Preparation evidence is not garment acceptance.
