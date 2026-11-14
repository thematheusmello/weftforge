# Native ordered sewing independent follow-up

The unchanged review harnesses fail on original commit `65c1e22fbff6ba528921629e4926caed0be397b0` (UI `ea4dce3fd56927db455bf375b354715d9f78e2b3`) and pass on fix `ed0dd37dc06789d531db5091917c7d64afeaa4d4`. `provenance.json` binds the source and artifact SHA-256 values. The original failure log retains the committed wrong-span comparison.

P1 originally replayed typed first-B endpoint text onto the old last B span after pointer-click whole-side reversal. The frozen harness still requires the modal to close, exactly one revision to commit, and the complete committed group to match the intended reversal: curve #43 stays 0.1 -> 0.9. P2 originally exposed no accessible action to append the inspector-selected boundary to side A; the same action-presence assertion now passes.

The fix clears all affected B numeric edit buffers before permuting the side. Accessible creation now exposes the active side's append action and preserves the canvas interval and B-direction settings. Connected command transactions, document authority, history and revision-filtered preview contracts remain in use. The fix's new product tests additionally cover one/two pointer reversals with pending InspectorDraft and a complete AccessKit-only 2:3 creation followed by one Undo; those tests were reviewed but are not substituted for these frozen review harnesses.

To rerun, use a disposable checkout at either listed commit and a separate Cargo target. The two `.rs` snippets must be inserted together immediately before the final two closing braces of `apps/weftforge/src/main.rs`, inside the existing `tests::ordered_sewing` module, with this wrapper:

```rust
mod frozen_independent_review {
    use super::*;
    // Exact contents of reverse-b-repro.rs, then accessibility-repro.rs.
}
```

The wrapper prevents a name collision with the fix's new pointer helper. The identical wrapper was also rerun on the original source and reproduced both failures; neither frozen snippet changed. Run `cargo test -p weftforge frozen_independent_review -- --nocapture` with `CARGO_TARGET_DIR` set to that checkout's separate target directory.

This is an in-process product UI review. No native app or reference runtime was launched. It does not establish reference parity, physical gathering quality, roadmap acceptance, or clean-checkout registry publication. Root owns native acceptance and workspace gates. Original failed logs and frozen criteria remain retained.
