# Coalesced endpoint broad source gates — private handoff

Source checkpoint: `3fddeb9a72e5b4395e8e874a628086937331af2f`; tracked checkout clean.
Main source SHA256: `3ddd1a624e7dd9af0c6228a12065782cc8dadef5bddb85f31d4958458f3adf21`.
No source/style edit or tracked metadata change occurred during this gate pass.

| Gate | Terminal session | Cargo PID | Exit | Seconds |
| --- | --- | --- | --- | --- |
| workspace-tests | 89394 | 91355 | 0 | 215.454 |
| clippy | 60306 | 95040 | 0 | 66.025 |
| native-build | 12453 | 95894 | 0 | 4.985 |

397 workspace Rust tests passed. Strict all-target workspace Clippy and native
debug build each passed once. No style fix was needed. All87 actual before/after
source maps agree for each new gate, and both whole-pass manifests are exact.
Raw logs and individual gate receipts retain commands, times, hashes and exits.

Full cargo fmt --all -- --check already passed on this exact main source before
checkpoint3fd; it was not rerun. Other86 inputs stayed equal to381c and only
STATUS.md changed afterward, outside the87-input roster. format-reused.json
retains this qualification; no newly measured format-time87 map is claimed.
Its completed command had no persistent session (tool chunk96558a, exit0).

Native path: `ROOT/.local/coalesced-endpoint-drag-20261116/target/debug/weftforge`.
Native SHA256: `eccf633cbdb82dcd91952bf1123395e82a528180fc8959dab3509a3ea4c5d66d`.
Native bytes: `62642024`.
The binary is built in this worktree target and has never been launched by
this agent. CARGO_BUILD_JOBS=1; no target or binary crossed worktrees. All owned
Cargo handles are terminal. Root owns fresh D003 scenario/binding/GUI/cleanup.

NATIVE-BINDING.json binds the unlaunched native identity; SUMMARY.json binds all
new gate receipts. SOURCE87.before.json / SOURCE87.after.json are byte-identical.
Original source-only HANDOFF.md, CHECKPOINT.json, PAUSED-* and red artifacts were
not rewritten. Red6134 remains exactly the old787fcadc test snapshot; later
ownership/Escape/pending-grain variants belong to green35485/source3ddd only.

No GUI, source-map/registry/report update, publication or solver work was done.
No trace47 code is included. Source/native ancestry integration, current-source
freshness, registry checks and clean committed cargo xtask verify remain root
work. Ignored private receipts are not public delivered evidence.

Native behavior is still unverified on this new source/binary. E001/D001/D002
remain frozen with no retroactive repair/regrade. The endpoint-only fix does
not expand recognition/release behavior for Rectangle, Transform or middle-pan.
