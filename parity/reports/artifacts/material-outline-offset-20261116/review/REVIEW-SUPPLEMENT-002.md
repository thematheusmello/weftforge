What this change does: Updates one existing MCP discovery assertion to expect 33 shared command variants after adding `offset_outline`.

Targeted review only: `apps/weftforge-cli/tests/mcp_stdio.rs:729` changes the expected value from `32` to `33`. Replacing that single numeric token with `32` reconstructs the exact previously pinned SHA-256, `f8fa6ffe6f1b664810aec4d1b68ba82e6888f752cfce2475325f2e8354aa9f63`. This confirms there is no other lexical change in this test file.

The updated test-file SHA-256 is `8c20f4e7bfd7b0a2ef60538a43c662d4363288b66ba05abd1c5528b15fde3811`; `SOURCE-PINS-002.json` updates that pin and inherits the other reviewed pins unchanged. The original 001 review and pins remain immutable.

Verdict: Ship this test-expectation correction. The numerical and atomicness assessment in `REVIEW-001.md` remains unchanged; the opaque earcut cancellation/internal-work criteria remain UNMET before first-phase acceptance.

Not checked: no other source, Cargo, native, evaluator or Git operations. This supplement does not establish a passing workspace test run.
