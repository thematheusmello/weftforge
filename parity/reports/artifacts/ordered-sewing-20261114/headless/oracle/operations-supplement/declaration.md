# Prospective shared-command operation supplement

Original freeze, criteria, fixtures, math and receipt interface remain unchanged.
No production implementation, executable or candidate output was read to prepare
these inputs or expectations. Public command semantics supplied before actions
fix the allocator, vector insertion, Undo/Redo and removal details below.

Use actual shared Editor commands in a persistent history context. Pin the
candidate commit, clean source tree, executable SHA-256 and local gates before
execution. Preserve actual requests, responses and invocation/tool records;
do not invoke an oracle function to produce candidate results.

The split/join input is an exact copy of original `line-identity-coarse`:
revision 73, next ID1001, group1000. Split curve16 at t=0.6. The retained left
curve is16, new split point1001 is appended to points at (1.3,1), and right
curve1002 is appended to the end of curves. Piece11 boundary becomes
`[16,1002,17,18,19]`. Allocator becomes1003, revision74. Group1000's A traversal
must be curve7 unchanged, right1002 `0.75 -> 0`, left16 `1 -> 0.5`.

Join those children using the actual join command. Parent16 retains its original
curve-vector slot; right1002 and point1001 disappear, all unaffected object order
is preserved. Revision75, allocator1003. The A list must retain three entries:
curve7 unchanged, parent16 `0.9 -> 0.6`, parent16 `0.6 -> 0.3`. Do not merge the
last two authored entries. Name, ID, B vector and map remain exact. Recovered
new parameters allow `1e-14` t rounding and split-point position `1e-12 m`; old
unaffected authoring values remain exact. Actual derived interval endpoints
must still resolve to their actual stored t values exactly.

Undo the join: revision76 with the complete split construction. While Redo
exists, try the complete group in `invalid-overlap-group.json`: it appends
curve7 `0.2 -> 0.6`, which overlaps the selected curve7 interval. The actual
shared command must reject it with an overlap error; every source bit, revision
and current derived group record must be unchanged. Redo the join successfully
at revision77. Then set resolution0.017 in one shared edit, revision78. Source
group ID/name/map/list and continuous physical pairing remain unchanged.

The separate dependent-removal input keeps the original geometry/group1000,
adds group1001 named `Independent surviving relationship`, and sets allocator
1002. Group1001 deliberately shares curve7/25 intervals with group1000, legal
across different groups. Remove piece11. Revision74, allocator1002; exclusive
curves16..19 and points12..15 disappear and the entire dependent group1000 is
removed. Preserve group1001 and every unrelated object/order/field exactly.
One Undo at revision75 restores the entire piece and both groups; one Redo at
revision76 removes them again. These are separate original input instances.

Every checkpoint supplies an actual `sewing_inspection` receipt using the
unchanged original interface. The independent evaluator checks exact source
state (with only the declared new-geometry rounding), group identity/order/map,
every derived cell and actual weighted stitch, mesh revision, original physical
locations at frozen probes and all old/new one-sided knots. Knot proximity uses
the original 1e-8 m path budget to avoid treating sub-ulp shifts of a disconnected
knot as new physical connectors. Both one-sided source endpoints remain required.
Rest chord approximation is reported separately. This is not physics evidence.

Evaluate actual captures with:

`python3 evaluate_operations.py CAPTURE_MANIFEST NEW_RESULT`

Manifest kind is `headless-chain-sewing-capture-v1`. `candidate` includes pinned
`commit` and `binary_sha256`. `checkpoints` maps all eleven names in
`expected-checkpoints.json` to relative receipt paths. `artifacts` maps every
referenced receipt/request/response/invocation path to its SHA-256. `commands`
is ordered `split`, `join`, `undo_join`, `reject_overlap`, `redo_join`, `remesh`,
`remove_piece`, `undo_removal`, `redo_removal`. Each record contains relative
`request`, `response`, `invocation`, `shared_editor_command:true`,
`observer_review:true` after actual shared-invocation review, and `rejected`.
Only `reject_overlap` is rejected and also retains its actual `error` string.

This concretely observes source/revision preservation and surviving Redo after
one rejected overlap edit. Broader deep-history/file failure preservation,
cancelled/stale/exhausted jobs, standalone native cases, reference parity,
physical quality and complete roadmap gates remain unknown. The original
criteria requiring those checks are neither removed nor marked passed.
