# Prospective native supplement

This supplement concretizes the original frozen standalone native case without
changing `freeze.json`, `criteria.md`, `interface.json` or any original artifact.
No native candidate was launched or candidate output inspected while preparing
this supplement. The original freeze is
`d84f38184dded9a574ace4e30c6a3860029d1b4fa9ae98e0e50d52bf73ac1ed0`.

`input-no-groups-v3.weftforge` derives only from the original frozen
`fixtures/line-identity-coarse.weftforge`: remove its `sewing_groups`, retaining
every other field and bit, including revision 73 and allocator 1001. The first
completed native gesture must therefore allocate group 1001, revision 74 and
allocator 1002. The documented public native default name is `Sewing group 1`.

Before any candidate action, pin the exact candidate commit, clean source tree,
executable SHA-256, local gate results, UTC time and launch command separately.
The candidate pin is supplied later by the root; it cannot change this input,
action sequence, expected checkpoints or evaluator. Both native launches must
use that pinned executable and commit. Record confirmed process handles.

Use a standalone native process and native save, quit and relaunch. Actual
tool-event transcripts, screenshots and accessibility captures must support
each checkpoint. Producer assertions, headless shared commands and in-process
UI tests cannot replace these native observations. `expected-checkpoints.json`
contains the complete exact native sources and stable action identifiers.

1. Load the original no-group input. With M:N sewing active, press Enter with A
   empty. Select curve 7 with next-click A controls `0.125 -> 0.375`, then curve
   16 with `0.9 -> 0.3`; press Delete and visibly verify only the last selection
   disappears. Press Escape. Repeat those two ordered A selections, Enter, then
   Enter with B empty; no partial group may enter construction.
2. Exercise tool switching with a pending draft, then loading the original input
   with a pending draft. Both must explicitly reset the draft without committing.
   Record every declared baseline checkpoint. All keep revision 73 / allocator
   1001 and the exact original no-group construction.
3. In M:N sewing, set the precise global next-click A endpoints for each click:
   curve 7 `0.125 -> 0.375`, curve 16 `0.9 -> 0.3`, in that order. Press Enter.
   Record `confirmed_a_pending_b`: source remains the baseline while the visible
   ordered A draft and directions persist. Set next-click B endpoints and select
   curve 25 `0.25 -> 0.75`, curve 34 `0.75 -> 0.25`, curve 43 `0.125 -> 0.375`,
   in that order; Enter commits exactly one group.
4. One native Undo must remove the complete newly created group at revision 75,
   while allocator stays 1002. One native Redo restores it at revision 76.
5. Open Edit group. Enter exact name `Native ordered acceptance edited`, change
   A span 0 to `0.15 -> 0.4`, preserve all other span entries/order/directions,
   and enter map `[[0,0],[0.2,0.12],[0.55,0.72],[1,1]]`. Commit the complete edit
   once: revision 77. One Undo at revision 78 restores every created group field,
   proving the name, endpoint and map edits are one transaction. Redo restores
   the complete edited group at revision 79.
6. Invoke the actual group-row Reverse B action twice, saving source checkpoints
   at revisions 80 and 81. The first reverses B list order and swaps each pair of
   t endpoints; the second restores every edited group field exactly. The map
   must remain unchanged. This must exercise the native UI action.
7. Review actual visible group identity, directions, lengths/mismatch, every
   disconnected guide and the declared arc-length metric. Native-save at
   revision 81, quit natively, relaunch the same pinned binary and reopen that
   saved project. Reopened source must be identical, revision 81 / allocator1002.

Evaluate after actual captures exist:

`python3 evaluate_native.py CAPTURE_MANIFEST NEW_RESULT`

The manifest uses kind `native-chain-sewing-capture-v1`, a `candidate` object with
40-character `commit` and 64-character `binary_sha256`, `standalone_native:true`,
and `checkpoints` keyed exactly as the expected JSON. Each checkpoint contains
relative artifact paths `source`, `tool_transcript`, `screenshot`, `accessibility`,
the exact `observed_ui_action` identifier and `observer_review:true` after actual
independent review. `artifacts` maps every referenced checkpoint path to SHA-256.
`processes` records distinct confirmed `handle`, `launched_utc`, `executable`,
`commit`, `binary_sha256`, relative `launch_record` and `observed_native_quit`.
The first launch must have an observed native quit. Two explicit review flags
record visible pending/direction and visible identity/metric/guide review:
`visible_pending_and_directions_reviewed` and
`visible_group_identity_lengths_mismatch_metric_guides_reviewed`.

This is a declared concrete line case, not a substitute for the original native
cubic/precise-edit case, failed native IO preservation, cancellation/exhaustion,
reference runtime equivalence, physical gathering quality or a roadmap gate.
Those remain unknown until separately exercised and reviewed.
