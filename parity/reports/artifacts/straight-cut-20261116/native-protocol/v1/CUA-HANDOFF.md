# Future actual CUA runtime handoff — no current authority

Keep the original pinned controller/recorder/observer immutable. The observer path
and SHA are in both binding templates; it is 393 bytes and accepts
`observe(controller, app, label, mark, output)`. It is not a closure over `cutApp`.

After root has completed binding, frozen it and issued the separate GO:

1. Start the filesystem UTC+monotonic clock and pair the first external operation
   before any case-relevant CUA entry. A fresh CUA entry invocation contains only
   `await cua.getState();`, exactly once. Retain that documentation/inventory result
   and the paired external receipt. It does not select/control an app.
2. In one paired/countable FS-only proof operation **in the actual CUA runtime**,
   import `node:module`, use its `createRequire` with a root-bound absolute actor
   filename, require fs/path/crypto/process and the pinned modules. Verify all
   bound source69/artifact/controller/recorder/observer/input hashes before launch.
   Create the native controller with the exact complete constructor:
   `createController(cutBinding.controller_configuration)`. All six keys are
   explicit: root, recorderPath, evidenceKind, callCap, elapsedCapS, baselineCalls.
   Its root already contains the original clock and trace; do not call startClock
   again. `baselineCalls` is zero because external/FS operations are recorded too.
3. In that same actual-runtime no-UI proof, use the immutable recorder to record
   and retain one opaque synthetic byte artifact under an isolated offline-proof
   subdirectory, verify exact bytes/hash and returned record, and retain an actual
   `ACTOR-RUNTIME-PROOF.json` receipt. It explicitly records actual_cua_runtime true,
   createRequire/fs-writable/recorder proof, all three module hashes, six-key config,
   and native_UI_calls zero. The subdirectory's mock records are not native trace
   operations. Standard Node vm checks in this packet do not satisfy this step.
4. Set `cutObserve = cutRequire(cutBinding.observer).observe`; initialize a
   synchronous `cutMark` before the first observation. It writes conditional
   conservative filesystem lower markers using only already initialized
   fs/root/process/controller objects. It must not read a future cutApp variable.
   The output argument is the actual current `nodeRepl`. No one-argument observer
   wrapper or variable-dependent future closure is allowed.
5. Externally launch the exact root-bound fresh own bundle using the bound
   `/usr/bin/open` argv, and retain/count exact own-PID evidence. CUT-R1 has no
   project input. CUT-R2's only project argv is the immutable accepted actual R4
   copy; never stage the oracle model as that input. No app control occurs until
   the exact externally launched owned PID is live. Then the two-operation
   `CUA-INITIAL-CAPTURE.template.js` callsite applies with actual bound objects.
6. Every later capture is exactly
   `await cutObserve(cutController, cutApp, 'literal-planned-label', cutMark, nodeRepl)`.
   Resolve its literal label before execution. There is no placeholder free label
   variable in a runtime call. Inputs are recorded separately and followed by the
   original required ordinary captures; no extra observer route is introduced.

The clock covers all six bootstrap/launch operations and bounded waits; Start12
is unchanged. Pair every external/FS operation in the same recorder trace; source
records are contiguous and returned. Any extra needed FS/bootstrap operation or
mandatory documentation refresh consumes reserve. The root review checks actual
source69 input inventory and own binary instead of trusting template values.

Success: one real Quit is the final native operation. Failure: stop immediately
without another input, observation or Quit. In either case the very next tool
call is CUA completion_summary only, exactly once. Do not call a clock tool first.
After it returns, separately retain the actual upper return receipt and exact
call/refresh counts, close input with the exact new schema, and perform the one
formal saved-data grade. Conditional stored markers are bounds, never claimed
as exact summary invocation times or extra attempted summary calls. Root raw UI
review and later cleanup are separate from the frozen actual case.

No launcher, controller, recorder, actual runtime proof, owned target or GO is
created by this document. Failure before a case clock/launch is recorded without
inventing a native pair, elapsed value or cap-pass claim.
