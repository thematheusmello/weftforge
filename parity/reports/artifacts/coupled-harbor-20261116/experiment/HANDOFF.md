# Harbor A8 preparation: no execution authorization

This is a new preparation namespace. Combined source remains clean at
96d3baee4543c7edbdb804df6090697a9f65da09. The frozen proposal, both older physics
packets, original cases and public metadata are unchanged. No Harbor project has
been initialized, no Cloth::step has been called by this preparation, and no
root GO exists. No GUI, old-frame analysis or remote operation occurred.

## Corrected completeness and pre-step gate

The missing independently authored static_geometry.py is copied exactly from
the original public evaluator, SHA256
`c4583bbda9250cfa32cdc305c1b21563c26ca46ddb75032f5c2ae4d128299b34`.
The other independent numerical modules remain exact copies. ORIGINS.json and
PREPARATION-DIFF.json retain provenance. The old proposal's analyze.py entry
point is removed only from this new namespace; incompatible old evaluator
receipt/main entry points are never invoked.

The harness adds two non-dynamics modes and a mandatory run gate:

- `prepare INPUT OUTPUT` constructs the current mesh/cloth/masses and exports
  initialized metadata/state, then exits with exactly zero step calls.
- `check INPUT OUTPUT APPROVED_METADATA` tests the same metadata guard as run,
  then exits with zero step calls. It was used only for new synthetic fixtures.
- `run INPUT OUTPUT APPROVED_METADATA` constructs a fresh cloth and compares its
  normalized round-trip JSON bytes with the independently approved metadata
  before any step or trajectory creation. This preserves numeric types and
  signed zero; Value numeric equality is not used.

The actual prospective case has one init-only initialization and one fresh actual
run initialization, both charged to the same 420-second clock. The independent
gate compares current initialized metadata against the original d966... gold,
not merely one current initialization against another. Only holes=[] and
internal_curves=[] on pieces 2/31 are added to the old project representation;
physical/rest/topology/UV/stitch/material/mass fields remain bit/type-exact.

## Builds and identities

All release builds used this combined worktree's own target, jobs1, an explicit
cwd/manifest/target and offline locked dependencies. No binary/cache was copied.

- First build: session 69213 terminal 0; lock child 63903 exit 0, build child 64009
  exit 0,29.87s. It produced bd7fb2a3... and is superseded solely by adding safe
  validation-only `check` mode. Original log, receipt and producer-built-v1.rs
  remain unchanged.
- Final build: session 7200 terminal 0, child 71066 exit 0,3.50s. Raw log is
  checks/build-v2.log and exact argv/environment is in BUILD-V2-RECEIPT.json.
- Producer source SHA256:
  `87b15646929d7ed76b03970fe84626176eb6cef72207f5f75a247ff649907a0b`.
- Private Cargo.lock SHA256:
  `6acc2df8d38cf70047cbe22ca2875a38a0fd204a63452edb2d42b4ce14c1ec89`.
- Final release binary SHA256:
  `d9d430b819d4ac0787ec2faebfbd3f54b27d1c2096d005ac4104c69f27691c41`,
  1,330,880 bytes, target/release/harbor-coupled-producer.
- Combined SOURCE88 manifest SHA256:
  `1c7dfd33e2de90b40a69b3655dd8f73f1196ad369f1da0ab82896d2b7e8bb1d0`.

Both builds verified all 88 actual source files and original input/rest hashes
before/after compilation. The final producer was launched only on new synthetic
fixtures in prepare/check modes, never run mode or Harbor input.

## Seven synthetic-only checks

checks/SELF-CHECKS.json and selfcheck-2.log retain all successful checks:

1. Exact CPython 3.13.0/resolved binary, original builtin-sum probe, no -O,
   bytecode disabled, Darwin and wait4 byte-RSS context; wrong contexts refuse.
2. Full transitive local import closure, independent analytic strain/energy/mass/
   self-contact/OBJ helper checks and new analytic capsule distances.
3. Empty-field compatibility, JSON duplicate/nonfinite refusal, numeric-type and
   signed-zero distinctions, initialized zero-time/zero-velocity/zero-step gate.
4. Actual release prepare/check on a newly authored analytic triangle with known
   0.1kg mass. Valid approval succeeds; signed-zero and numeric-type substitutions
   refuse before output/steps. No trajectory or step file exists.
5. Real child/grandchild process-group timeout and output-cap termination, exact
   owned wait4/reap, and counting every declared numerical directory.
6. Actual receipt persistence/hashing followed by the terminal clock; a delayed
   receipt crossing the deadline refuses and creates no positive verdict.
7. The real controller command without root GO refuses before any case directory
   is created.

The first synthetic fixture requested invalid 1.0 m resolution; existing input
validation correctly refused before simulation. That failure's source/log/input
and owned PID 72548 exit 1 remain under checks/synthetic and
SELF-CHECK-FAILURE-1.json. The fresh synthetic-2 fixture uses supported 0.5 m
resolution with the same analytic mass and tolerances. No failed old case was
repaired or regraded, and no numerical screen was relaxed.

## One-run controller and caps

Use the exact resolved interpreter and command recorded in PROTOCOL.json:

`python -B controller.py --go ROOT-GO.json --go-sha256 <exact-root-GO-SHA256>`

The root GO must exactly bind FREEZE.json, experiment/source, cases=[base], one
init-only initialization, one actual-run initialization, one actual dynamics
run, zero init-only step calls, zero retries and zero continuations. No sample
authorized GO file is created. validate_go in common.py is the exact schema.

Source88, actual prepared files, source HEAD/clean state, original input/rest,
actual release binary, private lock/manifest/build receipts, evaluator/controller
code and resolved interpreter/arithmetic are rechecked against the freeze.
Producer receipts bind the independent original-rest approval, actual output
coverage/hashes, exact child peak RSS, source/input/binary and elapsed clock.
The independent evaluator supplies strict quality separately from operational
completion; no producer final-only flag determines the trajectory grade.

The clock starts before input copying and both initializations. Each phase is
in its own supervised process group; inner analysis/serialization and final
source/artifact guards/report persistence remain under that same absolute
deadline. The cap is 420 s; all numerical bytes under inputs/expected/preflight-base/
runs/runtime count toward256 MiB, including the full analysis.json. Small receipt
headroom does not exempt a large analysis. All phase children receive256 MiB RSS
limits: this is prospectively stronger than the proposal's producer-only cap.

Finalization is supervised through receipt fsync/hash and child exit. The
supervisor fsyncs/hashes its own small pending receipt, then samples a final
external clock/volume check. Only its subsequent terminal stdout JSON and exit
can establish completion. A late negative appendix preserves failure and cannot
create a late success. There is no receipt-writing retry/recursion.

Qualification: persistence of that final stdout witness is externally captured.
Root's actual outer execution must retain stdout/terminal status and perform the
final wall/hash/source check against the SAME deadline before accepting the
bounded outcome. The pending supervisor receipt alone is not a successful
terminal verdict. Exit0 means operational completion plus independent strict
quality pass; exit2 means completed-but-quality-failed; exit 1 means operational
failure/refusal. Failed outputs remain; any existing case path prevents retry.

Original 8 substeps/32 iterations, dt 1/60, sewing 1 m/s, 60 assembly+120 gravity frames,
all contacts, original geometry/material and strict 15%/5mm/2mm screens remain.
No native, complete-garment, convergence, calibration or reference acceptance is
conferred by this preparation or any future bounded numerical pass.

FREEZE.json seals these preparations after synthetic checks. Its hash is returned
separately to root to avoid a self-referential receipt. All artifacts are private
ignored review inputs, not publication. No owned process remains active.

First seal qualification: d12762b474bac6d4260e8a1aa82928a273e09cfc51111ec2deed6e78060710e8
was rejected by the post-command audit because its redirected checks/seal.log
was still open at freeze time. Only that log mismatched. The exact first freeze
and completed log remain in checks/FREEZE-ATTEMPT1.json and checks/seal.log,
with FIRST-SEAL-ERROR.json. No code, build, synthetic check or case was rerun.
The replacement metadata seal captures stdout outside the packet and requires
a separate successful post-command actual-file audit before root review.
The fresh-seal invocation also correctly refused exclusive creation of the
existing first-attempt PROTOCOL.json before producing a new freeze. That exact
protocol is now archived as checks/PROTOCOL-ATTEMPT1.json, with the refusal
record in SECOND-SEAL-REFUSAL.json. Both metadata artifacts of the first failed
seal are retained; the next seal starts with neither current file present.
