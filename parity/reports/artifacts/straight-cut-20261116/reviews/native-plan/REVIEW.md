# Review 001 — prospective native cut packet

**Result: two provenance blockers before execution.** This is an independent static
review, not a native run, case assessment, GO, or acceptance authority. The existing
packet and original approved-v2 authority were not changed. Root confirmed that a
fresh prospective packet will address the findings; this review covers v1 only.

Reviewed packet: `.local/straight-cut-native-plan-20261116`, FREEZE SHA256
`1deb948c67b6bfb86d6aa383a21b92145c4021607311ddab1141950815e0d2a3`.
An independent stdlib-only read/hash/accounting check found all 18 declared packet
files and 14 external dependencies equal to their pins; FREEZE makes 19 packet
files. The original MANIFEST/ORACLE/NATIVE-R pins match. No packet helper,
selfcheck, formal grade entrypoint, product, native UI, CLI, Cargo, evaluator, Git,
or remote operation was executed for this review.

## Blocker 1: final identity and actual runtime proof are not checked

`grade_saved.py:271–287` checks only several binding values and digest shapes. It
does not read or corroborate the declared clean metadata/gate receipts, the actual
source69 files, native binary/plist, actual controller CONTEXT, actual-CUA runtime
proof, or owned-PID/launch evidence. Fields in
`ROOT-BINDING.CUT-R1.template.json:5,11–24` and their R2 equivalents can remain
null without being read. `closure():181–194` also permits a closure attempt unrelated
to the binding attempt. The trace loop at `grade_saved.py:288–310` proves generic
pairing/count/time order, not that these are the bound native runtime operations.

Static reproduction recipe: in an otherwise valid prospective receipt/model
mutation, leave source_worktree/own_target_directory/bundle/native_binary_path,
OS/plist/clean-metadata/gate identity fields null; supply arbitrary 69 path keys and
64-hex digest values, a 40-hex source commit, and a 64-hex binary digest. Fill the
existing authorization/configuration/argv fields. Omit ACTOR-RUNTIME-PROOF.json and
CONTEXT.json. None of the omitted identity/proof files is opened by this function,
so its saved-only pass path remains reachable. This recipe was not executed.

Minimum correction: require concrete final binding identity and case/attempt;
read/hash its existing source69, own binary/plist and clean metadata/gate receipts;
corroborate the supplied receipt results and source/target identity. Compare actual
CONTEXT exactly to `{evidenceKind, callCap, elapsedCapS, baselineCalls, recorderPath,
recorderSha256}` derived from the bound six-key configuration and pinned recorder.
Require a small exact actual-runtime-proof receipt tied to case, attempt, binding
hash and returned proof operation. It must carry the three module pins, six-key
configuration, actual_cua_runtime=true/native_UI_calls=0 and the isolated opaque
recorder artifact's path/bytes/hash/returned record. Check those bytes and source
records. Retain/review the actual CUA tool return: a Boolean or standard Node mock
cannot establish execution in CUA. Corroborate actual launch/owned PID with the
already required bound receipts. Reject missing/null/mismatching evidence.

## Blocker 2: CUT-R2 can mix CUT-R1 acceptance runs

`dependency():197–215` checks the R4 bytes and two report hashes but ignores the
existing UI-review fields `root_binding_sha256` and
`single_saved_grade_report_sha256` (`ROOT-NATIVE-UI-REVIEW.template.json:7–8`). It
does not bind/read the accepted R1 ROOT-BINDING. Thus a saved grade from R1 run A
and a UI acceptance from run B can satisfy the checks when both have identical R4
bytes. That does not establish one accepted actual R1 attempt as R2's dependency.

Static reproduction recipe: keep the same accepted R4 hash, point the dependency's
saved-grade path/hash at A and its UI-review path/hash at B, and leave B's different
binding and single-grade hashes intact. The current function never compares those
fields. This recipe was not executed.

Minimum correction: bind/read/hash the exact accepted R1 binding and attempt.
Require `review.root_binding_sha256 == grade.root_binding_sha256 ==
sha256(accepted R1 binding)` and `review.single_saved_grade_report_sha256 ==
dep.saved_grade_report_sha256`. Require case/attempt consistency and retain the
existing exact actual-R4 byte/hash/read-only checks. This needs receipt links, not
a new native scenario or an idealized R4 fixture.

## Checks with no additional blocker found

- `grade_saved.py:54–167` independently constructs the original literal model and
  compares the complete v4 fields, IDs, order, defaults, allocator and revisions.
  The pinned helper uses f64 bits including signed zero and f32 color bits. Only
  declared new anchors get 1e-12m tolerance; endpoint copies compare exact bits.
  R4 is exact actual R2 except revision. R5 preserves actual R4 except the declared
  revision and two tolerated y edits, including exact unchanged x bits.
- `retain_saved.py:13–58` preserves opaque primary/sidecar bytes with exclusive
  target creation, fsync, read-only mode and readback. Primary/sidecar rereads and
  presence checks catch observed changes. `check_receipts():218–268` requires all
  four R1 saves, ordered save/capture/copy records, separately retained sidecar
  absence, and exact previous-primary recovery bytes. This is a multi-file copy,
  not a promised atomic filesystem snapshot under an external concurrent writer.
- The closure has the correct exact new schema/field names and CUT-R1/CUT-R2 case
  value. The formal attempt marker is exclusive-created only after closure, and
  synthetic checks do not call the formal entrypoint. Native procedure/summary
  immediacy still requires root's raw tool/image/trace review; saved-only grading
  correctly leaves native_ui_verified/reference_verified false.
- PLAN step sums independently equal 158 and 49; Start12 and SaveAs15 sums match.
  Caps remain 175+1/1200s and 65+1/600s, with refresh calls consuming reserve.
  The continuous pre-bootstrap clock, single Name setValue/no fallback, sole
  accepted Apply, actual-empty R1 and accepted-actual-R4 R2 requirements are intact.
- The pinned 393-byte observer and initial callsite use all five explicit actual
  arguments. The Node vm check correctly claims no actual CUA proof. Analytical
  area×density remains authored geometry expectation only, with solver/particle
  mass verification false. Concave and failed-I/O native coverage remain required.

No native behavior, filesystem race stress, new packet, binary, or source69 receipt
was independently executed/accepted here. The two provenance gaps must be fixed and
reviewed prospectively before root binding and a separate GO.
