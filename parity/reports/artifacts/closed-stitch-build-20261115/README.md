# Closed-stitch integrated build checks

These original local records bind runtime `2e9795a3cb92c79ad3b406e675dcbdb49c60ce67`. All five commands passed: formatting, unsigned native packaging, 177 workspace Rust tests, strict workspace/all-target Clippy, and xtask verification with 54 Python checks. The exact 45 runtime/test inputs and native/CLI SHA-256 values are retained. The target was a real empty directory created in the isolated worktree; no compiled target was shared. All jobs are terminal.

The archived wrapper and logs are execution records, not a portable entrypoint or a newly executed replay. Commands can be repeated from a disposable checkout of the tested commit with its pinned toolchain and a fresh real Cargo target. Registry downloads may be shared. No executable, archive, Cargo target or installer is distributed here.

`provenance.json` distinguishes exact copies from literal own-repository path substitutions. Their original bytes and private inverse mapping remain with the owner; reversing each substitution reproduced the original hash. Substitutions change only path text, with no numerical, command or result changes. The manifest inventories every file except itself.

These are implementation checks only. They do not establish new native interaction, garment quality, convergence, calibration, cross-platform launch, release readiness or reference parity. Original failures remain separately recorded. GitHub Actions and automated Pages deployments remained disabled.
