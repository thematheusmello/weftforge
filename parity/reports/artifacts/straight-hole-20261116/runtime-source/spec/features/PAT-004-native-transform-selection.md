# Native transform selection clarification — 2026-11-15

The selection rule was approved before its guard and regression tests were implemented. This supplements the frozen
PAT-004 contract afffdf4e68ca5afb7f0038d2568576feb66824f873a8aaa659f60bfc072bf557;
its command semantics and numerical tolerances are unchanged.

In Transform mode, a click targeting another piece must not implicitly commit
or discard a staged inspector edit attached to the current piece, including
incomplete/nonfinite raw text whose numeric model has not changed. Refuse that
selection change, retain the current selection/draft/text/caret/Redo, and explain
that the edit must be applied or cancelled first. Normal cross-piece selection
continues when no staged edit exists. Same-piece numeric and drag transforms
still combine pending edits and the transform into one atomic transaction.

No broader cross-piece staging model is added in this increment. This is an
explicit native scope limit, not an observed reference rule. Test refusal and
normal selection separately.
