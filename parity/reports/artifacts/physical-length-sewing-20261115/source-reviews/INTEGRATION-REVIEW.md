What this change does: It integrates physical-length Free sewing and its retention fixes with guarded New Empty and the final Transform pointer/adoption fixes. New Empty additionally treats any pending Free sewing draft as an unfinished construction gesture.

Independent read-only source review of immutable `b657e6bf52aca6c0c76c5c51aaf76b4e812150f7`, including merge `a9a2f741a8266df8d9b224f322cb7e0892337e13` with reviewed parents `c4be919391efb8c39e035bb7c40e74ea52c592f4` and `dd5a16826dafe0d4f32c251f7c351197ad0936af`. Verified `apps/weftforge/src/main.rs` SHA-256 `fece6170af560fb3b56e430f4e0bdef1ca23cb70c73a0f5741f0f1288e091f9e` and both unchanged contracts: APP-001 New Empty `ce6de41684f98772c69bf8e0117e22f5f2cdf6b15952e4394f6756dd0077deb7`; SEW-002 physical length `4016013d9dda6bf4f910e52224586925314c156b9489d3c6d4a0599411fa18b2`.

Looks good. Ship this scoped source integration.

Checked the actual merge resolutions and adjacent callers:

- `patterns` retains Free sewing stale-revision rejection, cached candidate/metric refresh and the canvas input-retention call, alongside Transform search/popup/deferred-New pause handling. It keeps the unsnapped f64 Free sewing pointer calculation and Transform pointer-owner detach/update/release checks. Preview selection still prefers the active Transform preview, then the Free sewing candidate.
- The Transform start still requires a pressed pointer that remains down, so same-pass selection clicks leave no identity drag. Numeric adoption still rejects stale state, captures fresh inspector metadata and copies only retained translation/angle/pivot. The Pivot label remains above/right. Shared generic retained-input restoration preserves the reviewed Transform behavior.
- Free sewing nonmodal cancel still captures live inspector input; modal cancellation still uses its opening snapshot. First-side and final-side transaction behavior, failed-load retention guards, successful replacement/reset clearing and the earlier raw-input/caret regression tests remain present. Core resolver and CLI/MCP source are unchanged by this integration.
- Deferred New is processed after inspector/modal input and before generic cancelled-inspector restoration. `new_empty` now refuses `free_sewing.is_some()` (`apps/weftforge/src/main.rs:983`) without committing or dropping the gesture; the existing dirty/staged/raw/worker guards and fresh-editor reset remain intact. Both Free side modals are included in the authoring event shield.
- The two added integration tests exercise pending A start/B pending/B start through File/search and pointer/AccessKit routes, checking side inputs, revision, history/Redo and accepted mesh/cloth over later frames. They also check stale background New actions against both side modals. The final one-line helper change observes the initial search layout before deriving an action target; it changes test setup only.

No concrete blocker found within this bounded integration review. Earlier findings and their timing qualifications remain in the preserved original/fix reports; this approval does not replace their history.

Verdict: Ship the scoped integration source.

Not checked: No Cargo commands, native app/CUA, publishing, registry/map verification, OS accessibility or reference comparison were performed by this reviewer. Exact-commit Free 13/New 9/Transform 12/full-native 89 and Clippy passes are producer reports, not rerun or counted here. Root's full integrated gates and fresh native acceptance remain independent work; source approval is not a reference-parity or native-OS-acceptance claim.
