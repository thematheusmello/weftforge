# Independent final core review receipt

Final source: `9cd0a215df4a5977b1b3263061a6cb98183eb2d4`, parent
`70897ee5378cb36edb9647b4c0e5fdcfb884c54a`. Git status was clean when bound.
Reviewed contract SHA-256 remains
`9baedd713b5a8dd4f251d7f2c5d0c51b408ecdabb32813a07dae06d25dcae960`.

Final production Git blobs:

- sewing.rs: `7fb51bce5ce03ccb47d07dcc78f46e2b4f0b9f78`
- tests/sewing_groups.rs: `8ad63ee2a061e2844c603bc2229a68da4b5d5261`

Independent harness SHA-256 values:

- Original src/main.rs: `e7382664aa777a72dae7011824bbf95a8df6c0ef1aa4fe3cbf01068746c9160e`
- src/bin/ownership.rs: `0d9cb20364f395150f93370de7f3dd8a5477c46ab17c98022a0b26239fa24836`
- src/bin/precision_guard.rs: `91da2af0b38151c964c066a664f0cda5cf89924abd41b837b6daf8bcd59c4b82`

The original failed source, measurements and criteria remain in
ORIGINAL-70897ee-review.md. No original numeric input or 1e-8 m criterion was
weakened. The original binary now needs `--bin weftforge-chain-core-review`
because independent follow-up bins were added; its source remains unchanged.

## Finding 1 closed by 9cd0a21

Original 70897ee accepted a short near-end line interval with a steep map and
produced 5.01998207145255645e-7 m paired physical error. Its guard bounded
arithmetic by selected path length while the returned global t rounded at the
full curve scale.

The fix retains each interval's full control-coordinate and derivative scale,
forms a conservative global-t/evaluated-coordinate roundoff floor, sums it per
path, and applies the same physical-map conditioning to that floor. Unsupported
precision is rejected by the shared metric validation. No command or file is
committed before that check. Exact endpoint-return code is unchanged.

Independent review of the final guard found no additional confirmed issue.

## Independent checks

All runs use this review directory's isolated target tree; no shared-worktree
Cargo binaries were reused.

1. Unchanged original harness against the final production guard: Add rejects
   with `Sewing correspondence exceeds the supported absolute precision budget`.
   Its original unwrap exits 101, as expected for this unchanged counterexample.
2. precision_guard bin against committed 9cd0a21: **4 cases pass**. Line and
   uniform collinear cubic, each forward and reversed, retain the original
   near-end span and steep map. Safe identity baseline is accepted; Add,
   Update, Editor::new, geometry and to_json reject the unsupported map.
   Construction and Redo remain unchanged after command rejection.
3. ownership bin: **68 cases pass** before and after the guard. Disconnected
   2:3 paths cover both side reversals and authored map knots shifted by
   +/-8 ulps. Every selected source endpoint is exactly retained, consecutive
   portions meet, and no weighted connector joins vertices across pieces.

The core author separately reported 59 core tests, fmt, strict core Clippy and
diff-check passing on confirmed job 60145 (exit 0). These were not repeated or
relabeled as independently executed review checks.

## Scope and limits

Connected source read covered construction and shared commands, validation and
ID/overlap limits, cubic quadrature and inversion, path budgets and work limits,
one-sided cell ownership, mesh stitches/previews, split/join transforms,
editor/history transactions, strict v1/v2/v3 persistence, recovery and solver
entry points, plus their tests and command callers. Legacy f64/map handling
remains unchanged; save validation precedes staging/replacement.

No separate confirmed data-loss, migration, split/join or ownership defect was
found. The numeric guard is intentionally conservative and can reject extreme
but mathematically finite authoring configurations rather than promise an
unsupported approximation. This review does not establish a general interval
proof, hardware bitwise identity, native gesture behavior, gathering quality,
reference parity or a full roadmap gate.

Root integration/CLI/MCP/native UI work and full workspace gates remain outside
this review ownership. No production, docs, registry, branch, UI or CI file was
edited by the reviewer. These ignored local laboratory notes are not delivered
public evidence; any published qualification still requires tracked artifacts
and clean-checkout registry/map verification.

Verdict: the core source review passes at 9cd0a21; do not qualify original
70897ee as passing this precision requirement.
