What this change does: It captures the next root workspace frame through eframe, encodes its native pixels as an sRGB PNG, and stages the file before the UI commits it. The core wrapper keeps byte exports independent of GUI dependencies and preserves the previous destination until that commit.

**Must fix**

1. **A stalled PNG worker can block application shutdown indefinitely** (`apps/weftforge/src/capture.rs:L261-264`)
   - **What this is:** The exit hook waits for the PNG worker's result. The worker can be encoding or inside the destination filesystem's write/sync operation (`L153-156`); cancellation only marks its eventual result for discard.
   - **Problem:** Choose a PNG on a mounted filesystem which stops answering during staging, then cancel or close the app. `Saving` has no deadline and `receiver.recv()` never returns until that filesystem call finishes, so app shutdown can wait indefinitely. The existing shutdown case queues a completed result before calling `shutdown` (`L542-570`) and cannot catch this case.
   - **Fix:** Bound the shutdown wait or disconnect the result receiver without waiting. Keep the worker unable to commit, retain normal single-worker retirement, and ensure a later staged result is dropped. Add a deterministic empty-live-channel and late-result oracle; do not depend on an actual slow filesystem.
   - **If we skip it:** Closing the studio can hang even after the user cancels capture. The old PNG is preserved, but exit is unbounded.

Checked scope:

- Actual cached eframe/egui-wgpu 0.35 source agrees with the requested-frame-plus-one stamp: commands are applied after the request frame paint; the next paint copies pixels; a later frame delivers the event. The same Arc-backed `UserData` identity is returned. The root viewport/token guards are exact, and final-pass metadata replaces earlier pass metadata for the same frame number.
- The backend maps BGRA/RGBA surface bytes into premultiplied `Color32` values without reversing rows. Unpremultiplication for PNG, RGBA/eight-bit encoding and the sRGB chunk agree with that representation. The prepared 2x2 decode/chunk oracle tests channel order, orientation, alpha and metadata independently of a raw encoded-byte comparison.
- Nonzero dimensions, checked pixel count, side limits and exact vector length are enforced before CPU encoding. Output length is checked after encoding. These are admission/output limits, not a strict bound on all backend GPU/readback or allocator peak memory: eframe creates its screenshot allocation before delivering the event, and a resize after command submission can exceed the module's pixel cap before rejection. Root and readiness explicitly accept this boundary; no renderer rewrite is requested.
- The worker returns a `StagedExport`; it cannot publish the destination. UI commit follows the current frame's cancellation and revision checks and rechecks the current project path. The core guard runs before staging and before commit; staged replacement rechecks the target's regular-file status. The existing target is preserved for invalid native names/content, symlinks, failed staging, cancellation, stale revisions and rejected commits. Staged-file drop removes a temporary result after rejected commit or channel failure.
- The wrapper reuses existing temporary-file creation (`create_new`), write/sync, rename and drop cleanup. No GUI dependency enters the core. Prepared tests cover old-target preservation, drop cleanup, missing staging file, a destination becoming a native project while pending, successful replacement, and existing native-name/content/link/directory protection. The asynchronous shutdown gap above is the remaining finding.

Assumed load: one user and one admitted PNG worker in the native studio, within the declared CPU-image dimensions. No claim is made that an arbitrary filesystem operation has a bounded duration.

Verdict: fix 1 first.

Not checked: Cargo/test/build execution, a real GPU screenshot, native lifecycle behavior, numerical model behavior, runtime or reference parity; parent separately reviews broader main.rs integration.
