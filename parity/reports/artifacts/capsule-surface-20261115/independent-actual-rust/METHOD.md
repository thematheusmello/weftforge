# Prospective executable Rust extension of the 83-case static review

This new private directory extends, and never edits, the prior immutable
.local/capsule-independent-review-20261115/ report. Prior manifest SHA256 is
1041f3bcee5c3c313c72c590b455fc124bcba2ef268251ccef5e406c91ca4e8a.
Every prior artifact is rechecked before and after this task.

Question: does actual unmodified Rust capsule::closest at commit
24dba7715233a7423088772d49c32a632d901a0c satisfy the same83 input cases and
original prospective per-case tolerances as the prior independent100-digit
Decimal constrained-quadratic oracle? This fills only the source-transcription
limitation. It is not a garment, convergence, performance or parity experiment.

Before actual Rust output: reconstruct the prior case generator verbatim from
its retained source, freeze all83 full triangle/axis inputs in order, and assert
its Python transcription results exactly match every retained prior row. Copy
only the original oracle distance and original tolerance for each row; no
expected numerical value is derived from the new Rust output. Freeze expected
uncertainty flags from the earlier transcription as a separate agreement gate.

Extract math.rs and capsule.rs byte-for-byte using git show of the exact commit.
Expected SHA256: math4f344da57b5bcb7fb994bb67ae40f3909b7e23814969971dbe457c9094d1b424;
capsule67bed0887b2a0644dbab7c7003228427c0f9540e5d09a1170961f46cd14e9de7.
A tiny harness declares Vec3 and includes these unmodified module files. It
calls only closest() on the83 immutable static inputs and prints JSON lines
for distance, uncertainty, barycentric weights and axis fraction. No engine
source alteration is permitted to compile the harness. If it cannot compile,
retain the failure and stop rather than alter the engine.

Compile only that standalone harness with `rustup run 1.94.1 rustc`, edition2024,
default unoptimized code generation, using this directory alone for sources and
binary. Retain exact command, compiler verbose identity and executable hash,
harness/source hashes, compiler output and actual executable stdout/stderr.
No Cargo, dependency/workspace/release build, GUI, garment integration or
simulation stepping. The compiler is1.94.1(e408947bf2026-03-25), aarch64-apple-darwin.

Apply unchanged prior rules to actual output: finite values and valid convex
barycentric/axis domains; every returned distance <= oracle+tolerance; if exact
(uncertain=false), absolute distance error <= tolerance. Also compare uncertainty
flags against frozen prior flags and report any difference explicitly. Original
case tolerance is5e-12*max(local_scale,1e-15); copy each retained numerical value
without changing it. Any failure remains failed; no corrective engine edit,
tolerance change or retry follows. Preserve actual output before analysis.

Hash module copies and committed source again, verify prior report manifest,
and write a terminal result with all artifact/command hashes. Parent owns any
later integration gate, execution or public packaging.

Skipped: garment dynamics, actual response stepping, full workspace tests,
UI and publication. Risk:83 static primitive cases cannot establish whole-
garment behavior, convergence, continuous contact or reference parity.
