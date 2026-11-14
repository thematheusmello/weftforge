# Supplemental native ExportDXF Cancel with pending field and Redo

This is a separate, prospective check of native failure preservation. It does not change original FILE-005-v1 freeze `20fc2dc605374c5ee9b4056c3716e18156266515db2a82bbc02f99288f7e6560`, evaluator `6756739bebef6f0753ad0bb86eed3ed0116b8d6d1dfaecf548b368c44a959ed6`, primary fixtures or completed primary outputs. Same candidate binding `ded5f27d1cb5df0dbb3c007f2dbdf5936dc1f2d6babbac256eda61d1cdbd2bf4`, source cd151b3b89e26d55a4dcd4525790006660c4e766 and native executable `4cdc758b057975b47fe81306d4fec2c8f598e8ea5e5d83601c6eab2baa0d199b` apply.

The primary Cancel had no pending draft. This case deliberately creates both a pending numeric draft and a Redo target, then checks the saved consequence. All supplemental names are new; the eight named primary snapshots/exports/sentinel remain immutable. Autosave/recovery scratch files are excluded from primary immutability assertions because they belong to active editing, not the named evidence snapshots.

Before starting this case, freeze this declaration, the small evaluator, independent expected documents, observation template, pretest and primary file hashes. Expected documents are derived only from the original frozen `input/native-expected.weftforge` revision-21 fixture, preserving every field except stated revision/resolution. No subsequent native output is used to generate expectations. No production source, Cargo, native app or server is invoked by this oracle.

Sequence, with one invocation at each stated Undo/Redo step:

1. Fresh native Open of `native/reopened.weftforge` (revision 21, resolution 0.045) resets history. Set resolution to 0.05 and Tab once to commit revision 22. Undo once, yielding revision 23/resolution 0.045 with an available Redo target of 0.05. Wait for the stable 907-vertex 0.045 rest mesh.
2. Native Save As `native/cancel-draft-baseline.weftforge`. Every saved JSON field must equal `expected-baseline.weftforge`, revision 23/resolution 0.045. Save As must preserve the Redo target.
3. Focus the resolution field, atomically paste 0.055 without Tab, invoke toolbar ExportDXF, select the same existing `native/native-cancel-sentinel.dxf` name and Cancel without Save/Replace.
4. Before any further action, observe and capture the still-pending field text 0.055, no Unsaved marker and unchanged 907 vertices. This is required because saved documents alone cannot prove an uncommitted field was retained. Preserve a readable image of the full relevant UI and a separate action log.
5. Restore the field text to 0.045 without Tab; invoke exactly one Redo. Native Save As `native/cancel-draft-redone.weftforge`. Every saved JSON field must equal `expected-redone.weftforge`: revision 24/resolution 0.05, all other fields exactly the original source values.

The original precommit-on-Cancel behavior would commit 0.055 at revision 24 and clear Redo; restoring 0.045 and invoking Redo would leave revision 25/resolution 0.045. This explicit negative witness must fail. Wrong revisions, lost Redo, a committed draft, unrelated geometry changes or discarded pending text also fail pretests. The saved r24/.05 outcome proves the intended Redo target survived across Cancel and the baseline Save As; the captured field/marker/count observation independently covers visible pending-state preservation.

After actions, copy `observation-template.json` to `native/cancel-draft-observations.json` and truthfully fill observed values. Do not edit the template or report expected values if they were not observed. Add `post_cancel_image` and `action_log` relative paths under the acceptance directory and SHA-256 hashes of their original files; no absolute paths or parent traversal. Those two evidence paths may be selected from the actual capture after observation; their content and interpretation remain reviewable. The evaluator checks the declared values, evidence bytes/hashes, saved source JSON and all eight primary hashes. Its `accepted` result still requires separate independent visual review of the referenced image for the visible assertions; producer JSON alone is not visual evidence. Preserve any failure as a failure, with no criterion adjustment or export repair.

Run, from repository root:

```
python3 .local/dxf-acceptance-20261114/supplement-cancel-draft/evaluate_cancel.py --self-test --report .local/dxf-acceptance-20261114/supplement-cancel-draft/pretest.json
python3 .local/dxf-acceptance-20261114/supplement-cancel-draft/evaluate_cancel.py --report .local/dxf-acceptance-20261114/reports/cancel-draft.json
```

This supplement covers this specific native Cancel interaction only. It adds no DXF geometry, reference parity, import, printing, other consumer or general history claim. No native write-failure/overwrite-warning case is implied. Ignored local evidence remains preparation until selected artifacts are tracked and registry/map checks run from a clean checkout.
