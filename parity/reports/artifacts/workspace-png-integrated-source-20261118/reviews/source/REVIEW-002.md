What this change does: It removes the PNG shutdown wait while keeping the worker unable to commit a file. It also shortens the success notice to the image dimensions and removes the unused display-detail stamp fields.

Looks good. Ship.

PNG-SOURCE-001 is resolved in the reviewed source. `WorkspaceCapture::shutdown` (`apps/weftforge/src/capture.rs:L246-249`) drops its state and receiver without waiting for an in-flight worker. A queued result is dropped; a later worker result cannot be received or committed, and the failed send owns and drops its `StagedExport`. Normal cancellation still retains the busy slot until the worker retires, so ordinary UI use cannot start a second worker while the first one is pending.

The prepared oracle (`L593-639`) keeps an empty sender alive while shutdown runs. It uses a one-second observation deadline, releases the sender before joining a regressed blocking shutdown, then checks a late staged send fails and its dropped error cleans the temp while preserving the old target. This catches the original wait rather than prequeuing a completed result. These tests were read, not run.

The concise success notice is `Saved workspace PNG · W × H px.` (`L215-218`). Removing unused mesh/time/scale fields does not remove token, next-frame, revision or expected-dimension checks. Core `StagedExport` and its native-target guards are unchanged from REVIEW-001. The new early Escape observation is before receipt and does not consume existing popup/modal handlers' event; root retains the broader main.rs review.

The guarantee is removal of the worker-channel wait, not a hard duration limit for every filesystem call. Dropping a queued staged export can synchronously remove its temp. A worker that returns while the process is alive cleans its rejected late result; termination during a blocked or unfinished write can leave a hidden temp. The worker still cannot publish the destination. Backend GPU/readback allocation before screenshot delivery also remains an admitted scope limit.

The original REVIEW-001, its JSON and FREEZE-001 remain byte exact. REVIEW-002.json binds all six updated source hashes, the G base and tracked diff, including the untracked capture module by its own hash. There are no remaining must-fix findings in the assigned capture/core scope.

Verdict: Ship.

Not checked: Cargo/tests/builds, GPU/native execution, numerical behavior, post-process temp cleanup, runtime/reference parity; required root checks remain pending.
