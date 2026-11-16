# Focused UI correction 001

Base supplied by root: `7eb09375b6d4172a407a3fffac01fa59a4fbc544`.
Only `apps/weftforge/src/main.rs` was edited. Both corrections are in tests;
the product implementation, tolerances and existing checks were retained.

1. The preview test expected one text shape containing all three paired-length
   rows. The actual `pair_length_feedback` splits its string at each newline and
   renders three separate labels. The corrected test requires the literal visible
   rows `A 360.000 mm`, `B 360.000 mm` and `B − A +0.000 mm`, along with the seam
   identity and A/B orientation. Every expected text shape must lie completely
   inside its clip rectangle. It does not infer lengths from a helper return.
2. The lifecycle test called `open_cut_piece` directly between egui frames after
   a canvas pointer interaction, then searched for a modal-only checkbox without
   first asserting a rendered review. It now opens through the normal Return
   event in a fresh frame and asserts that review is open before interacting.
   The test also retains the original nonmodal lifecycle cases and runs the same
   accepted/failed transitions with the sewn modal review. Failed open/save and
   New Empty refusal must retain both the correct modal state and checkbox state.

One authorized correction cycle ran with the isolated cut-and-sew worktree's own
Cargo target; no target was copied or shared with another worktree:

| Command | Exit | Result |
| --- | --- | --- |
| `cargo fmt --all` | 0 | Completed in 0.55 s |
| `cargo test -p weftforge --bin weftforge cut_piece_tests` | 0 | 14 passed, 0 failed, 110 filtered out; test bodies 2.95 s; total command 13.11 s |

`CHECKS.json` retains all 74 source hashes before and after the cycle, exact
arguments, elapsed times and stdout/stderr hashes. Every pre-edit hash matched
the supplied failed-integration pin. Only the desktop source hash changed:

- Before: `ee34de77f6f13973bae3be8309c195c1689eaca5bb29076ffaab6004372483e9`
- After: `e561485f666197368aad44c302a2216635c5ff2afdb08a6cfc4bfa77d4e4bb44`

The previous failed integration receipt and logs remain unchanged. The Cargo
target lease was returned to ROOT immediately after the focused run. This cycle
did not run the full workspace tests, Clippy, a release build, GUI/native
qualification, evaluator, Git or reference runtime. It earns no reference parity
or actual native-process evidence.
