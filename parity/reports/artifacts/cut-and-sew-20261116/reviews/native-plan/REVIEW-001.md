# Native Cut & Sew draft — independent review 001

What this change does: Revision 002 proposes two native cases on sealed runtime
5898 / source 74 and clean metadata 385. S1 uses only the accepted H1-002 parent and
retains one sewn cut with Undo/Redo; S2 reopens only accepted S1 R4, removes the
seam and edits the independent child. Saved grading, raw UI acceptance and source
test results remain separate.

This review covers **revision 002**, inventory SHA-256
`430c378e72b461bd36b09cc9eb72cfed41b83f9a7cb919b9d8f5ece94a8783ce`.
Its file/dependency hashes matched when read. The author began a prospective
revision 003 after receiving these findings; those in-progress edits are outside
this receipt. Locations below refer to the inventoried revision-002 bytes, not
whatever a later draft places at the same path. This is not a freeze or GO.

## Must fix before freeze

1. **S1 Space turns sewing off before the intended Apply** (`PLAN.json`, S1
   second-review shield row; actor review section “Changes needed before freeze”).
   - **What this is:** The draft enables `Sew cut edges`, then sends four modal
     shortcuts including Space, then applies the cut.
   - **Problem:** The checkbox retains focus. The sealed UI handles Space as a
     checkbox toggle, so it becomes unchecked and Apply selects ordinary
     CutPiece. That produces no seam and next ID 20, contradicting required seam 20 /
     next ID 21. The separate source audit confirms this with source/test anchors.
   - **Fix:** Apply root's prospective no-extra-operation ordering: put those
     eight shortcut/capture operations in the first review after the invalid
     `1e0-` witness and before full field correction. Claim modal/stopped/R1
     preservation, not raw text preservation under Space. The second review is
     Sew-on then one Apply, without Space between. Finalize the audit's literal
     normal-Return opener, explicit End curve 9 and three pointer checkbox clicks;
     no runtime alternate opener or keyboard-toggle claim.
   - **If skipped:** The prescribed successful route authors an unsewn cut and
     consumes the sole case/grade opportunity on the wrong result.

2. **S2 undercounts its required retained-anchor inspection** (`PLAN.json`, S2
   final inspection row and declared 68; actor review “Declared S2 route”).
   - **What this is:** The draft budgets four operations to inspect retained piece 2
     anchors 11/13 and child 15 after the height edit.
   - **Problem:** There is no selectable seam row, and retained anchors are shown
     after selecting retained piece 2's chord 18. The source-backed route needs four
     input/capture pairs: select 2, open boundary combo, select Boundary 2 / curve 18,
     reselect 15. The literal inspection therefore needs eight operations.
   - **Fix:** Use the actor audit's exact Properties→Sewing seam-scoped `Remove`
     route and second selected-child `Size m` field. Replace final inspection 4
     with 8 and record S2 total 72, paired 71, reserve 3, unchanged cap 75 plus one
     summary / overall 76 and 900 s. S1 remains 149 / cap 160. Keep current-observation
     Save As names/folders, bounded scrolling/readiness and stop-on-ambiguity.
   - **If skipped:** The declared procedure/count record does not describe the
     actual proposed evidence route. Six-decimal UI labels also cannot replace
     the saved grader's bit-preservation check.

3. **S2 does not bind its S1 dependency to the same metadata and packet**
   (`grade_saved.py:276`, particularly revision-002 lines 315–318).
   - **What this is:** The dependency checker follows the prior binding, sole
     grade, root UI review, closure, postflight and actual accepted R4 bytes.
   - **Problem:** Its S2 branch compares prior source commit and binary but never
     compares prior clean metadata, source 74 inventory/count or packet freeze.
     An accepted, fully hash-linked S1 from another metadata/packet binding with
     the same 5898 runtime/native bytes can satisfy the dependency predicates.
     This violates the declared single sealed 5898 / 385 / source 74 dependency.
   - **Fix:** In S2 require prior/current clean metadata385, identical named
     source 74 inventory/count, and prior binding plus prior grade freeze equal
     to the current bound freeze. Keep the deliberate historical H1 exception
     only in S1. Add a positive fully linked dependency control and refusal
     controls that relink hashes after changing each prior identity.
   - **If skipped:** S2 can silently cross metadata or protocol generations even
     though all local hashes and the saved geometry look valid.

4. **An incomplete freeze leaves consumed inputs unbound**
   (`grade_saved.py:365`, revision-002 `verify_freeze`).
   - **What this is:** The function verifies hashes for whichever packet files
     and external dependencies a supplied freeze lists.
   - **Problem:** It never requires coverage. Empty dictionaries or a freeze
     omitting `EXPECTED-STATES.json`, `PLAN.json` or `SOURCE-74.json` satisfy its
     loops, although grading consumes those bytes. No inherited routine adds
     this coverage check. An omitted/changed model can cause an irreversible
     false grade despite a matching freeze-file hash.
   - **Fix:** Require the named grading/protocol/model/authority inputs and all
     declared PROVENANCE dependencies with their declared digests before the
     hash loops. Add synthetic omission/digest refusal controls. This needs no
     new framework, old-packet edit or actual case run.
   - **If skipped:** The final binding can refer to a syntactically accepted
     freeze that does not freeze the answers or other required inputs.

## Checked and retained

The actual clean receipt SHA-256
`fd8e69195e15c996e668107a5a5f0012c99346fecb549347e01b06d184d77f5a`
matches the draft's receipt model: `checked_clean_commit` is 3858848, runtime is
5898f1a, all 74 source hashes match runtime receipt 34db1fa…, both xtask and
debug-build exits are 0, before/after clean and source-unchanged flags are true,
and `fresh_owned_default_target_no_cache_copy` is true. The four clean log hashes,
all 74 sealed source bytes and the 62,245,032-byte native SHA 7809e419… matched by
read-only hashing. No Git/build or gate was rerun.

The assessment literals agree with the approved construction: pieces 2/15,
independent chord 18:11→13 and chord 19:17→16, seam 20 A-full/B-reversed identity map,
next ID 21, .0324/.0540 m² and analytic density products. Undo 3 retains next ID 21; Redo 4
compares actual R2 bits; Remove 5 changes only revision/seams; child-height 6
changes only point 5/17 V to .40 within 1e-12. The parent model matches the frozen
Hole V2 assessment defaults/name, while the actual native input and its three
acceptance provenance files were only hashed as opaque bytes. No expected
fixture is authorized as a native input.

The grader uses closed exact document comparison, unchanged f64/signed-zero and
f32-color bits, narrow new-anchor/height tolerance, actual earlier retained
states for Redo/removal, directed walks/disjoint ownership and independent saved
interval interpolation. The forward-B negative checks endpoint mismatch even
when lengths and midpoint agree. Its measurements are explicitly analytic saved
area×density and intervals, not actual Mesh/Cloth, particle mass or physics.

The raw retainer reuses the pinned opaque exclusive-copy path, fsync/readback and
read-only snapshots/sidecars. The generic receipt checker enforces ordered
Save→capture→FS-copy pairs, exact declared labels/revisions, no first-save
recovery, later recovery equal to previous primary, and final primary/sidecars
equal to final retained bytes. It does not parse or grade live construction.

The final-binding code rejects required nulls, false grants and extra nested
fields. The draft templates remain honestly false/null and FREEZE is absent.
The bootstrap/readiness code requires source 0002 canonical actual-CUA proof,
independent next-cell 0003 with all 20 helper types/flags, exact six-key controller
configuration, context/proof/opaque/status checks, launch 0004, full binary-plus-
input-argv own-PID 0005, then getApp 0006 and capture 0007. The saved grader enforces
continuous trace chronology, separately counted documentation refresh, bounded
summary return and an exclusive one-attempt marker after closed scope/binding.
Root still must corroborate real tool history, immediate summary/actual Quit and
raw UI independently; saved-file success explicitly does not verify native UI.

The 33 synthetic refusal names match the read selfcheck source and retained
SELF-CHECK-002 receipt. Positive anchor-roundoff/copy/Redo and f32-equivalent
controls, opaque-copy and readiness controls are represented. They were not
rerun in this review. The three new Python files parse; no module was imported
or grader/selfcheck invoked.

The separate actor audit is now **complete, with changes required**, rather than
pending: `.local/cut-and-sew-actor-route-review-20261116/REVIEW-001.md`, SHA-256
`40244dd2932cf99aa2e535ce22ece5a5aa83ba979fc40c227af89f2972071722`;
its hash record is `08be6cde15adfe21590b3d872e84bd61fa258d846d6a2b5bfabf2b9afa75e4e2`.
It supplies prospective source-backed labels/counts, not a native execution.
Root has approved its shield ordering; revision 002 does not yet contain those
changes. The author's prospective revision 003 must be separately inventoried
and checked; this receipt supplies no advance approval of it.

Verdict: fix 1–4 before freeze. The draft remains NO GO. No actual native case,
formal grade, clock/controller, source edit, Cargo, Git, GUI or old-case replay
was performed. Only this private review and input/hash records were written.

Not checked: final actor cells, completed case bindings/freeze, actual native
focus/visibility/timing or reference behavior. Hole H2's failed SVG bounds case
stays failed and separate; these cases have no exports and cannot silently bind
the later SVG-fix source/binary.
