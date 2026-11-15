# Reviewed prospective scope, frozen for A

The prospective packet is frozen; no runtime source/binary binding, GO or native execution exists. The initial proposal's
900-call aggregate was a rough estimate; the explicit table below replaces it.
One initial Save As per mutable document plus raw immutable snapshots avoids
repeated history-state dialogs. Every field focus/type/Tab still has a capture.

| Case | Full retained scope | Static maximum operations | Hard cap | Wall cap |
| --- | --- | ---: | ---: | ---: |
| A | Actual R0 save; unit square; two-click R line; invalid modal/raw/caret + staged grain; precise both endpoints in one R3; Undo R4/Redo R5; quit | 157 | 175 | 1,200 s |
| B | Fresh A R5 reopen; unchanged editor; edit R6; stale anchor + Undo/Redo; invalid modal with real old Redo; width R9/height R10; quarter-turn R11; line removal R12; restore R13; owner removal R14; restore R15 | 175 | 190 | 1,200 s |
| C | Original sewn S_line; precise coincident line; selected second identity drag; staged boundary-control + original endpoint drag in one revision; rejected boundary with retained Redo; raw owner guard and successful owner invalidation | 153 | 170 | 1,200 s |
| D | Original R_line; malformed Open with raw/caret+anchor; failed Save destination/recovery; accepted fresh save; New gesture/raw refusals; tool-change/focused Escape; New/Open/Recover reset | 233 | 240 | 1,500 s |
| E | Original loaded L/C inside-endpoint/outside-middle failures and precise corrections; six actual before/after SVG/DXF/OBJ exports; omission help | 216 | 235 | 1,800 s |

Total: 934 static maximum operations, 1,010 hard cap, five additional completion
summaries, 6,900 seconds wall cap. Counts include external launch/PID/wait/raw-copy
bookkeeping; folder-navigation and readiness allowances are conservative maxima.
These are stop limits, not a target number of actions. Each case stays separate
and serial with one owned window; only B depends on actual accepted A bytes.

Root reviewed A’s 32 ordered steps/six states and175+1/1200s cap. Only A may
receive a separate runtime grant after final metadata verification, completed
integrated gates, final source/binary/target/input binding and explicit root GO.
A's pass would be an empty-project authoring/edit/history checkpoint. B–E and all
listed native capability gaps remain required, unexecuted work.

Current draft synthetic validation: 24 original saved-state models; 42 wrong
construction/export outputs refused; eight file/provenance/missing-output/cap
counterexamples refused; byte-copy helper preserved exact primary/recovery bytes,
read-only snapshots and no-overwrite behavior. These are grader selfchecks only.

Detailed original numbers/procedures/caps are in PLAN.json; full shared protocol,
input staging, source/default authority, exact snapshot process and gaps are in
METHOD.md. EXPECTATIONS.json is the independent saved-state oracle. No actual
native process, candidate CLI/evaluator, Cargo, source edit, commit, registry,
Actions, Pages or reference action ran for this preparation.
