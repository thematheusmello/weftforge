# Independent straight-cut implementation review — 2026-11-15

Outcome: no remaining blocking finding in the source identified by `SOURCE-SHA256.txt`. One concrete native review-gate defect was found, reported before acceptance, and corrected during this review. This is a static engineering review, not product execution, native acceptance, reference evidence, or permission to publish.

Reviewed the uncommitted `feat/straight-topology-cut` candidate in `.local/straight-cut-20261115`, HEAD/base `bbada3f7f5228a9479a76c0c300d3aa716e57d77`, under AGENTS.md and Ponytail FULL. The reviewed specification SHA256 is `da88a248482dd45dd702b57e33dab13c38b885c222042863e973bf809c886b0c`; ORACLE SHA256 is `13846d635a1432c3e5bfd77eb66802cc633bd29a3661c940972783dc2f5dfea2`; original v2 MANIFEST SHA256 is `1144d1e955cd77bd3e8690364555e7f0e9fec1a921b7ee410966a45fd0edd942`. Both candidate R240/L fixture bytes match that manifest exactly; the new fixtures are still untracked at review time.

## Finding R1 — opening Return also applied the newly opened modal — corrected

Before correction, with an owner selected and the Cut piece tool active, pressing Return without a draft entered `open_cut_piece` from the global key handler, then reached `cut_piece_window` in the same frame. The modal observed the same unconsumed Return and applied its valid default adjacent-edge cut. No intervening review was possible. Opening an existing first-anchor draft also immediately attempted Apply, producing an error before any review. This violated the explicit review/Apply boundary.

Final `apps/weftforge/src/main.rs:3182–3183` consumes the opening Return inside `open_cut_piece`, covering fresh and existing nonmodal drafts. The later modal Return handling at lines 3224/3279 remains. Regressions at lines 16038–16139 cover both entries, preserve the full editor/selection/mesh/cloth/job/dirty state on opening, then require one separate Return to execute exactly one shared cut and one Undo to restore it. Final native source SHA256: `2e8a3f7003ab5ee56c38042162bcdfc325d30428510b878b0d0bf66053e68343`. The correction and both regressions were independently read; no further native blocker was found.

## Reviewed reach

- Core: CutPiece enum/apply/callers; canonical original-boundary order; nine-ID allocation and preflight; shared private SplitCurve reuse; independent point/curve ownership; global object order and exact copied metadata; target-wide seam/group refusal; whole annotation partition and strict clearance; one outer shared work budget; authored simplicity/chord/midpoint certificate; existing validation/history/allocator and I/O boundaries.
- Tests and derived state: all new cut tests and literal fixtures; exact R/L walks and IDs; swapped/clockwise/offset/placement cases; initial per-piece UV triangle area and cloth mass checks; no shared particles or automatic cut sewing; stale-mesh rejection; independent child resize/transform/remove; v4/export/recovery checks. Relevant unchanged meshing, sewing, solver adapter, persistence, CLI transaction and export paths were read for integration risks.
- Adapters: one closed cut schema alternative on the existing tools, strict original-byte command deserialization, required fields/IDs/finite parameters, batch limits and transaction rollback, same-path primary/recovery preservation. No new CLI verb/tool or format field was introduced.
- Native: rail/menu/search/precise routes, typed boundary IDs, cut-local selected-owner raw projection, 12-pixel vertex/multiple-hit refusal, preview clone isolation, retained inspector text/caret guards, modal validation/cancellation/background shielding, accepted selection/history behavior, stale tool/owner/revision/load/recovery handling, failed-I/O retention, reopened child edit and bounded annotation summaries.
- Documentation: architecture, automation and formats agree with the reviewed engineering subset. STATUS remains explicitly in progress; it must be finalized by the root after the separate gates. No parity/registry promotion was reviewed or granted.

## Evidence limits

This reviewer ran only read-only source/Git/hash checks and `git diff --check`; no Cargo, product/CLI, GUI, evaluator/grader, remote or Git-write operation was performed. No production source was edited. Native owner reports RED handle 90972 for both opening-Return regressions, GREEN 35460 after correction, 116/116 native package tests and strict package Clippy; these are attributed reports, not independently rerun checks. Root retains the original failed handles and decides acceptance after integrated gates.

The named original L fixture at 0.005 m hits the existing mesher vertex cap both before and after cutting; eligibility/walk/analytic area remain checked there. This remains unsupported fine-L mesh/mass evidence, not a newly found defect or a full acceptance pass. Actual native cut cases, reference observation, integrated workspace gates and clean-checkout tracked-artifact registry/map publication checks remain outside this review.

Skipped new execution and broader geometry/solver work. Residual risk is conservative numeric/work refusal and the already retained fine-L meshing limit; static review does not establish native or reference parity.
