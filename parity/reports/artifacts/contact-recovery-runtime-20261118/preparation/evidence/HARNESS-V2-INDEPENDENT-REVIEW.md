What this change does: V2 closes the two source-admission findings from the first
review. It binds the enumerated studio and producer compiler inputs before the
first preflight snapshot, and links selected workspace/xtask evidence to one
actual execution chain. The numerical experiment remains deliberately unarmed.

Looks good for the intended prospective source binding. No remaining concrete
must-fix was found in this V2 review. This is source readiness, not build,
numerical, garment, native or reference acceptance.

The review is bound to V2 FREEZE SHA
`82662f530ffbb1252b143c08c3b0a978484874290c8781f346ec32820667ce97`
and revision patch SHA
`5fd007c6e2575fc5b0b562e33833ad8f31cab2f6e689df847033c46625e5c3c6`.
The first review and its freeze `063bec7d…` remain unchanged.

1. **Compiler-input coverage finding closed**
   (`packet/compiler_inputs.py:31–71`; `packet/common.py:147–152`;
   unchanged `packet/source_inputs.py:35–59,65–76`).
   - The helper enumerates actual files under apps, crates, vendor, xtask and
     .cargo, plus root Cargo/toolchain/build inputs. Paths, SHA-256, byte counts
     and modes must equal the corresponding C-tested descriptor subset. New,
     removed, changed or symlinked inputs cannot merely become a new D snapshot.
   - The whole actual producer tree must equal the approved three-file map:
     Cargo.toml, Cargo.lock and src/main.rs. The same descriptor map is required
     in the hash-bound root source review. An added producer build.rs or include
     file therefore refuses admission before initial compiler binding.
   - `require_source()` performs these checks, and `create_preflight()` calls it
     before capturing local inputs; later preflight/runtime checks call it again.
     The earlier loophole for a newly added automatic build script is closed.
   - The retained new tests exercise actual synthetic file enumeration, added
     core/sim/root build scripts, automatic tests/include bytes, mode/content
     changes, removals, unapproved producer inputs and a rejected symlink. They
     were read as source and retained evidence, not rerun by this review.

2. **Mixed execution-receipt finding closed**
   (`packet/common.py:61–86,136–162`).
   - Workspace admission now checks selected RESULT SHA in TERMINAL, selected
     TERMINAL SHA in EXTERNAL, EXTERNAL metadata commit equal to actual f30c,
     and selected EXTERNAL SHA in POST. Existing completion, exit, timeout,
     cleanup and acceptance checks remain required.
   - Current-D xtask uses the same selected-file chain with `SOURCE` as its
     commit. Its selected monitor stage must be type/value-exactly present in
     the current-head result with no result errors, and must still be a successful
     `cargo xtask verify` with clean before/after and matching source hashes.
   - The byte-valid accepted constructor external/post pair from f614 can no
     longer be paired with correct f30c plan/result/terminal data. Retained tests
     cover that exact mix, each hash/commit link, rejected acceptance and missing
     or wrong-current xtask stage membership.

Twenty relevant numerical packet files remain descriptor-exact to V1: the
counter evaluator/worker/producer, both original fixtures, five geometry/math
modules, controller/resources, both outer scripts, manifest/lock, build/source
helpers, freeze helper and draft protocol. The present integer (not bool) zero
counter at REST/all181 and transient-positive trajectory failure remain intact.
Original settings, target recurrence, thresholds, 420 s / 256 MiB / 8 GB controls,
failure retention and no-continuation/no-prefix-grade policy were not changed.

Read-only checks verified all 217 V2 regular artifacts by SHA-256, bytes and mode,
the one listed inert synthetic refusal symlink outside packet, and the original
preparation/review/preservation records: 324 distinct hashed paths. The original
59-file preparation and the first review stayed exact. No source, INT, target,
active release-build file or old report was written or regraded.

The retained once-only V2 receipt reports 265 synthetic assertions in 0.226369 s
under its 60 s / 64 MiB source-check envelope. This review did not rerun it or any
prior check. Those assertions do not establish actual garment quality, Rust/API
compatibility, nonlinear recovery behavior or full-case resource fit.

Verdict: source-ready for the intended actual binding and build/GO gates. Root
must still populate exact clean-D source/xtask descriptors, the approved producer
map, actual optimized native/CLI/producer identities and final freeze/admission,
then issue a separate numerical GO. The deliberately null slots are not defects
and this review grants no runtime permission or transfer of C test execution to D.

Not checked: tests, Cargo/API compilation, current release job, model execution,
all181 numerical quality/resource fit, GUI or reference behavior.
Risk: private ignored source review is not tracked public delivery or numerical GO.
