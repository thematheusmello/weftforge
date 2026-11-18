Distinct v2 source-only packet is ready for root review. Candidate remains exact
`13fb43fe08c045f08a1b85007aff3c15388a1e24`, with the pinned root 407-test/fmt/strict-all-target-
Clippy / native CLI stages green. Metadata freshness/xtask remains separately pending.
Historical 20261116 case names are retained; receipts use actual November 17 UTC.

V1 failed operationally when a 0.25 s ps observation timed out and killed live Cargo;
that was no demonstrated product/source compile failure or physical-case failure.
No producer/model ran. V1's complete packet/GO/failure, original root draft and
root failure receipts remain byte/mode exact (see preservation manifests/check).

Only v2 resources.monitor changes runtime policy: ps timeout is an observation
failure. Record it, immediately re-poll the SAME child with wait4, accept its normal
terminal receipt if ended, or retry observation while it remains live. Snapshot
timeout is at most 1 s and capped to the remaining deadline; three consecutive
unavailable observations stop negatively as rss_observation_unavailable. Actual
wall/output/RSS breaches still stop. There is no child/phase retry or restart.
Best-effort group snapshots have explicit gaps; there is no continuous 20 ms RSS
claim. Direct-child terminal wait4 RSS and terminal wall/output caps remain exact.
Failures and maximum snapshot gap are retained in the receipt.

Eight tiny fake-child checks passed: transient/same PID continuation, termination
during timeout, three failures negative, wall during timeout, sampled RSS, exact
terminal RSS, terminal output and terminal wall. All fake children were reaped;
optimization guards remain explicit. No Cargo/typecheck/constructor/actual grader/
dynamics/GUI ran. Unchanged producer/consumer 11 retain the original qualification.

Physical protocol is exact: original input fc494974... / rest d966979..., four empty
v5 serialization fields only, 654 vertices/1172 triangles/48 stitches, same masses/
materials/all contact, dt 1/60, 8 × 32, 1 m/s closure, 60 zero-G + 120 gravity. Independent
init-only zero-step rest approval precedes a separately fresh 180-step producer.
Strict all 181-state 15% axis strain / 2 mm contact/finite+uncertainty and 5 mm seam after
exact-zero target use the original 8 consumer files; inherited saved-state wording
is precisely qualified as fresh-producer snapshots. One 420 s numerical whole clock
through root final witness and 256 MiB numerical child/output caps are unchanged.

SOURCE-PREFLIGHT `76b46c72fdda9a1ece636a29e52bf48fb5325f6c3565d2e2aacb3a1490711927` binds the actual source/compiler/environment/flags/
known local and locked dependency inputs; build binding is `b35c301476e23c92b376698b8a99fa38fb845ceb26fd3ba46b2eb2207a4b28f9`.
Own existing candidate target is reused without copying compiled files. Known
compiler closure excludes linker/macOS SDK/sysroot shared libraries. No actual
build GO, execution GO, binary freeze or physical result exists in v2. Next: root
reviews delta and guards, authorizes a NEW build-only GO, then separately reviews
actual binary/final freeze before any execution GO. Exact commands are retained.
