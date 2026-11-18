Only tests.rs changes from47da5bf1 toff955cf2:30 inserted lines in the existing
shared_soft_sewing_cancellation_noop_and_invalid_inputs_are_atomic test.

For the first existing weighted spring alone, both target0 and target.5 have a
fresh successful baseline. Each must move positions and nodal history, retain
both pinned positions and pinned histories exactly, and keep all values finite.
Each observes at least entry and final successful pre-copy callbacks. The test
then cancels at EVERY observed original callback, including the last, and requires
Cancelled, the exact callback count, and exact original x/history retention.
There is no production change, numerical tolerance change, new test or framework.
Removing the saved insertion recovers the entire prior tests.rs exactly; all
other90 product hashes/modes match47da. Prior physical guards/oracles/32 and
main bounds are untouched. Prior DESIGN and bend/hard extraction proofs still
apply byte-exactly; this delta supplies their missing singleton callback coverage.

The old47da preparation seal remains byte/mode exact. The prior tests.rs bytes
are retained beside the new insertion and diff, as well as recoverable from Git.
No Cargo, formatter, constructor, solver/test/model, analyzer or GUI has executed.
Root and bounded independent re-review plus distinct first-GREEN GO are pending.
