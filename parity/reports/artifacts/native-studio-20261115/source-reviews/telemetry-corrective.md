# Native task telemetry corrective review

Reviewed candidate `221fdb0658ba5a63a6f2759709d53b19cab2dddc` against
`35b14dbda586dc7c044bbeb8a1e3743f9af8dac1` by reading source only. Assumed one
local studio with one active native worker. No source files changed and no
builds, tests, native windows, or physical case actions were run for this review.

The original finding is fixed: typed worker failures survive late cancellation
and obsolete revisions, remain Failed, retain their error text, and retain the
accepted cloth and mesh. Core cancellation classification uses the operation's
own callback; the current mesh and initialization callees return immediately
when that callback reports cancellation. The idle disclosure row is now always
present, removing its asynchronous insertion/removal from viewport layout.

## Must fix

1. **A failed predecessor clears replacement work queued after cancellation**
   (`apps/weftforge/src/main.rs:838-841`).
   - What this is: the corrected failure arm always clears `playing` and
     `pending_batch`, including results that fail the existing acceptance gate.
   - Problem: a simulation can fail after `ui()`'s initial poll, then the user
     clicks Step. Step cancels that job and queues one replacement step
     (`main.rs:1384-1397`). The next poll retains the failure correctly but clears
     the replacement step. Before this correction, `!can_accept` returned before
     clearing successor work. An edit followed by a queued batch has the same
     problem when an obsolete predecessor reports a genuine error.
   - Smallest fix: always retain and display the failure, but keep the existing
     acceptance gate around clearing `playing` and `pending_batch`. Extend the
     controlled late-error test with a replacement batch retention assertion.
   - If skipped: an explicit Step or new-document batch request silently does
     not run because an already superseded worker failed.

## Existing check records reviewed

Read `check-summary.json`, `red-source.json`, and the relevant logs beneath the
candidate's `.local/native-job-telemetry-checks-20261115/`. Confirmed the recorded
late-cancel/obsolete regression is RED against unchanged production `35b14db`,
then GREEN; the initial seam-guide layout failure differs vertically by 10.5 px;
the final native suite reports 46 passed, 0 failed. Final format and strict native
Clippy records report success. Recomputed log SHA-256 values for red, green,
native-final, fmt-final, and Clippy-final match `check-summary.json`.

These are existing local implementation records. Workspace/full integration
gates and actual window usability or physical garment behavior remain
unverified here. This ignored review note is not published evidence.

Verdict: fix 1 first. No further concrete blocker found in the corrective scope.
