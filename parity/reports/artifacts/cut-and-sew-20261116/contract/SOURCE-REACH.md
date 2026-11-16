# Inspected source pin and future reach

Read 2026-11-16; no source changes or execution of application, Cargo, evaluator,
project CLI, Git, GUI or REA runtime. Reads/searches/hashes used filesystem tools.
Only these three private proposal artifacts were written. No job or window was
started by this task.

- Runtime source recorded by STATUS.md and the current-source addendum:
  `a021a2581c8e1dd000e535a417dcd964c3835884`.
- Metadata ref read directly from the existing worktree metadata:
  `52163adb86b9f48fdd06f042e4ddf615797c3947`, branch
  `feat/straight-pattern-holes`.
- Sealed source tree: `.local/straight-holes-20261116`. Its source, ref, target
  and existing artifacts were not changed. The parent-supplied 73-input runtime
  seal and native build qualification were not re-executed or inferred from this
  source review. Hashes below identify the actual files read, not a new build.
- Existing published cut/hole evidence, current H1/H2 native qualification and
  registry freshness remain the parent's work; this proposal changes none of them.

## Actual source/API reach for future implementation

Paths are repository-relative at the pin above; line numbers identify current
anchors, not future output. "Review/test" means no production change is expected
unless the approved implementation exposes a concrete defect.

| Surface and current anchor | Smallest required reach |
| --- | --- |
| `crates/weftforge-core/src/document.rs:1133` | Extract inline CutPiece body into one private cut helper returning the two chord IDs. Preserve canonicalization, two private SplitCurve calls, identity/allocation/order, independent ownership and whole-annotation partition. New command adds one legacy seam in the same candidate; ten-object/ID preflight includes it. Existing CutPiece remains nine. |
| `crates/weftforge-core/src/document.rs:1928` | Add five-field CutAndSew enum alternative; same strict serde envelope. No new field on CutPiece, Piece, Hole, Seam or Project. |
| `crates/weftforge-core/src/document.rs:241`, `:287`, `:1515`, `:1538`, `:1588` | Reuse CurveInterval full/reversed and legacy Seam/AddSeam; deterministic Cut seam #id name. Verify RemoveSeam deletes only the constraint. Review existing RemovePiece dependency cleanup. |
| `crates/weftforge-core/src/document.rs:442`, `:2036`, `:2113`, `:2130` | Include the extra seam in preflight/final object/ID validation; retain one shared work budget, clone/validate/commit, revision overflow, atomic late failures and Undo/Redo high-water semantics. |
| `crates/weftforge-core/src/geometry.rs:857`, `:988` | Reuse straight-cut eligibility and annotation partition unchanged. No new tolerance, topology model, special mesher or looser cap. Holed owner and target relationship refusals remain in document/picker paths. |
| `crates/weftforge-core/src/geometry.rs:292`, `:302`, `:1325` | Review/test existing legacy seam_lengths and append_stitches full/reversed behavior on the new independent chords. Verify actual per-piece triangles/boundary ownership, weighted pairs, endpoints/coverage and no welded mesh indices. |
| `crates/weftforge-core/src/sewing.rs:7` | Review/test unchanged authored interval lengths and unrelated legacy/group data. Do not route the new seam through ordered-group arc-length/easing semantics or change those algorithms. |
| `crates/weftforge-core/src/simulation.rs:18`, `:77`; `crates/weftforge-sim/src/lib.rs:55`, `:77`, `:290`, `:376`, `:382` | Review/test actual fresh-revision adapter weights/stitches/compliance and initial per-owner masses. Minimum evidence API: add read-only Cloth::stitches() -> &[Stitch] beside existing getters, because retained Cloth input is private. No solver algorithm change or stepping required. |
| `crates/weftforge-core/src/lib.rs:12` | Existing wildcard document exports expose the new variant; existing sim Stitch type is already public. Review only; no export wrapper or dependency. |
| `apps/weftforge-cli/src/main.rs:365`; `apps/weftforge-cli/src/mcp.rs:528`, `:666`, `:770` | Generic execute/project_execute already wrap shared commands once. Add closed cut_and_sew alternative with all five required fields and existing bounds; retain strict raw-byte validation, 1 MiB/1–1024 limits, nine tools and both protocol profiles. No verb/tool dispatch or inspect schema addition. |
| `apps/weftforge/src/main.rs:420`, `:425`, `:498`, `:516`, `:3494`, `:3559`, `:3623` | Add one transient sew option to CutDraft/CutPreview key/command selection; default false, clone-only preview and ordinary seam summary. Preserve typed boundary selectors, raw text/caret, stale guards, cancel/Return/apply/input shielding, selected child and one history entry. |
| `apps/weftforge/src/main.rs:3485`, `:3543`, `:5050`, `:5408`, `:6299` | Review/test existing entry/property guard, owner/revision/tool/history/file lifecycle, picking and modal shield. Reuse existing Cut piece entry points; no additional tool or picker. Keep exterior-only and holed-owner refusal. |
| `apps/weftforge/src/main.rs:2823`, `:2916`; `crates/weftforge-core/src/persistence.rs:26`, `:130`, `:278`, `:342` | Reuse ordinary seam inspection/removal and strict v5 save/reopen. Test failed IO retains accepted state/output, same-path handling and old migrations; no format bump/default field. Existing SVG/DXF/OBJ remain geometry exports, not native seam round trips. |
| `crates/weftforge-core/tests/cut_piece.rs:170`, `:322`, `:416`, `:624`, `:673`, `:739`, `:833`, `:961`, `:1046`, `:1104` | Focused CutAndSew tests beside existing original cut tests or one small sibling file. Verify analytic IDs/ownership/direction/attributes/area/mass/stitches, normal cut exact regression, annotation partition, refusals, extra-seam caps, history/high-water, RemoveSeam/RemovePiece and late failure. Preserve original fixtures and fine-L cap qualification. |
| `crates/weftforge-core/tests/holes.rs:641`; `crates/weftforge-core/tests/hole_io.rs` | Add CutAndSew holed-target refusal alongside CutPiece and retain v5/migration/IO tests. Do not expand hole semantics. |
| `apps/weftforge-cli/tests/mcp_stdio.rs:157`, `:195`, `:525`, `:662`, `:887`, `:2324`, `:2510` | Test strict new variant, CLI/both MCP profiles, actual saved seam, same-path/separate output and refusal bytes. Update all six existing schema-count assertions from31 to32; tools stay9. |
| `apps/weftforge/src/main.rs:16148` | Extend existing cut egui tests for checkbox off/on/no accepted ID use, cache invalidation, raw invalid input/caret, input shield/opening Return, Cancel/stale state, one Apply/Undo/Redo and native save/reopen. Actual native acceptance remains a separate future observation. |
| `docs/automation.md:134`, `:167`; `docs/architecture.md:69`; `docs/file-formats.md`; feature specifications | Document new command/ten-ID result, ordinary seam orientations/removal and unchanged v5. Add only the approved bounded engineering contract; preserve historical contracts/oracles and failures. |
| `spec/inventory/weftforge-current-source-addendum.json` / `.md`; `parity/features.yaml`; `STATUS.md`; existing registry/map verification | Future approved source pin/anchors/counts become33 Rust /32 schema /9 tools. Record actual evidence and exact next handles, preserve unknown/failed/blocked/stale outcomes and no reference credit. Historical implementation map remains unchanged. Publish only tracked artifacts with clean-checkout checks. |
| `.github/workflows/ci.yml`, `.github/workflows/pages.yml`, root README, static Pages | No change. Actions/automated deployment remain disabled and product branding/static site remain intact. |

## Actual SHA-256 reads

| Path | SHA-256 |
| --- | --- |
| `AGENTS.md` | `b9bdf4c9b881d0ab6c084a5314c45f4f55ff0bb17d19fafe357caa353cd7e78e` |
| `crates/weftforge-core/src/document.rs` | `4ce5ea5d3919e4fa180d723558ad30975ce2158f68772bc3c151e86857bb05f0` |
| `crates/weftforge-core/src/geometry.rs` | `d4ba435fe2fbcd907c97a01b16be2b8bcfa934a1e3cbb4882fc75ab43f380105` |
| `crates/weftforge-core/src/sewing.rs` | `b68442ce95e3207bcfb34745e2c653eddaea1ce65fa8feda56321bbcc9d415ed` |
| `crates/weftforge-core/src/simulation.rs` | `73ee0a5d1c6d5b58c70a9ccac74898e12e4ffb51bfe2e3f2002b339912715f29` |
| `crates/weftforge-core/src/persistence.rs` | `42c3d163bf347af1dcf3bb007ac15c7ba3e4836c829b18904592cb96b6cd9812` |
| `crates/weftforge-core/src/lib.rs` | `157990183f20487c3b1494a837914e81fb98e3e4fb35a1f0dedc0542a1ec6661` |
| `crates/weftforge-sim/src/lib.rs` | `5bfc996e6bf1067ac261358fdd043815adf304a5d8d512a0e6429c913e90fa76` |
| `apps/weftforge/src/main.rs` | `71b2ff79a315a5de5347ce36a4a07a13a3e9faca6c46e77086338bb24b8a2d48` |
| `apps/weftforge-cli/src/main.rs` | `7049615180a793ad1557a2bbd3a27797b4b06aac060a2cb0620bd40436e64e0b` |
| `apps/weftforge-cli/src/mcp.rs` | `34af8f549de7eac7f025089fddb8c1d798fbd1e65b85d5fce74de473ba72c7cb` |
| `crates/weftforge-core/tests/cut_piece.rs` | `c5b673f0a69a43086c3a6d5b7aa58e4b68bcf061637f57c4731a83af4879f018` |
| `crates/weftforge-core/tests/holes.rs` | `eaf21904ea94858632e6d11caddda63c0bf220845788633481c3bd114a355594` |
| `crates/weftforge-core/tests/hole_io.rs` | `dfe604ad08e660e9feecb88d99153eee78bb1cadcbb8307350ad9983956025a2` |
| `apps/weftforge-cli/tests/mcp_stdio.rs` | `23f3d0d1ee01d86dfb426e2a89b514a399ba53b51159dedbf932b18ce93753f4` |
| `spec/features/PAT-006-straight-cut.md` | `da88a248482dd45dd702b57e33dab13c38b885c222042863e973bf809c886b0c` |
| `spec/features/PAT-006-straight-hole.md` | `634028923f25811aa364d85ce13947db242812d6523635c13ce5a715277b1d2f` |
| `crates/weftforge-core/tests/fixtures/original-cut-r240.weftforge` | `afcd39b8e65caa76a4fe7e556d10d58a0117c5af6954859cfa6ad35d50bb1d50` |
| `crates/weftforge-core/tests/fixtures/original-cut-l.weftforge` | `a8ddc0c3b9e625e6bc77d72ec382cc333dcd4225e87a6c91261636af28828e7d` |
| `docs/automation.md` | `a32c398cd5bfa9c5f8073f9015e883712e3b62c2152023cd9d01797794613528` |
| `docs/architecture.md` | `ce2709de29fc64a5de302870b4b6d47fbc66b331aae699687a629953ca895e88` |
| `docs/file-formats.md` | `880888c4da9bead588f0d77b3e82f0d845f947387211e493059bf28d4bba4b36` |
| `spec/inventory/weftforge-current-source-addendum.json` | `12031db93d5e64b25e9f4eea2e7c57c01fc18b351e7184b9efc3c80d469c1cb2` |
| `spec/inventory/weftforge-current-source-addendum.md` | `0c9d8f005c1c949079c4449465e5b0d8275cd7197fd730fec14c1d32bf16c2c5` |

These are source pins for a proposal, not published delivery evidence. No required
Rust gate, clean-checkout registry check, native observation, actual mesh/Cloth
measurement or reference qualification ran during this task.
