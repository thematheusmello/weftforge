# Prospective A16 final-clock correction — root review pending

Preparation001 and its blocked freeze f7c19b4295ae8ec9d4246d95eebc4f53b7306fe4f6071a67f26eb0c333cb1a84
remain unchanged and unexecuted. The original review is retained and hash-bound
in ORIGIN-BINDING.json. No root GO is present for this sibling preparation002.

The deadline correction is small: run.py obtains final_numerical_bytes before
sampling ended, then reuses that measured value in the receipt and byte-cap
check. Every included byte traversal, output hash and source/input/GO/origin
inspection finishes before ended. Only final receipt serialization/writing is
outside that timestamp. RUN.diff shows the entire code correction plus the
explicit origin/artifact/review guards; PROTOCOL.diff shows the path/clock
binding changes. Physical settings, numerical math, screens and caps are unchanged.

The synthetic test executes only the corrected closing AST block with fake
byte counts, a fake clock and an in-memory receipt writer. An inspection ending
at419.99s produces complete; the same block with an inspection ending at420.01s
produces failed/strict_screen_passed=false. Both perform exactly one byte scan
before the closing timestamp and no byte scan afterward. The old controller,
product outputs, geometry and dynamics are never executed by this test.
Selfcheck wrapper753/child926 completed exit0; the actual job/logs are retained.
The original passing16-substep rational recurrence check is reused by exact
unchanged source/helper hashes; it was not rerun.

The same driver is referenced at its **original exclusive default target**:
../preparation-a16-001/harness/target/release/harbor-public-driver
SHA2561d6caa60cd3c95b860332fb2d06f1dfee919f7fd035233e4aa21fd9e948a8067.
No target/cache was copied, no Cargo build occurred and preparation002 has no
harness/target. The driver still uses current ad7 core plus the identical solver
through its original read-only path dependencies. This is no new runtime/native
acceptance. Original build/dependency evidence is guarded as part of all38
original frozen files, rather than presented as a fresh build.

The consumer, five numerical helpers, thirteen resource-helper ASTs, signatures,
analysis/comparison adapters, source/read-only bindings, A input, expected rest
metadata and original one/three-literal diff records remain byte-identical.
GUARDS.json records checks on18 unchanged copies,22 current source files,
19 retained inputs, both original review files and the original executable.
ORIGIN-BINDING.json makes those original-file/binary guards mandatory before
and after any future authorized attempt.

PROTOCOL.json keeps the same single A16x32 experiment, immutable A input,
180steps/181states, exact physical rest preflight, strict15%/5mm/2mm and finite/
uncertainty screens,420s whole-case and256MiB resource/output caps. No B, retry,
continuation, tuning or historical regrading. Stage argv use this new working
directory and the old unchanged executable. Failed/incomplete output remains
failed; the old A/pair failure cannot be promoted.

Root must independently clear this correction and then separately hash a GO
bound to the new FREEZE.json before using its declared controller argv. The old
f7c19 freeze stays blocked. No candidate/geometry/consumer execution or real420s
case clock has started in either preparation.

Skipped: Cargo/rebuild, candidate/preflight/dynamics/product-output consumer,
workspace/GUI/reference checks and publication.
Risk: this fixes only the closing-clock defect; no physics improvement or
current garment/native/contact acceptance is established.
