# Closed native-run evidence: public derivative

**This is a reviewed public derivative, not the original private run packets.** It contains selected WeftForge-owned UI evidence from three closed runs. Originals were read-only during preparation. No product experiment, replay, evaluator, grader, GUI action, or Cargo check was performed to prepare this bundle.

| Run | Retained outcome | Primary record |
| --- | --- | --- |
| OFFSET-R1-001 | Bare-executable CUA selection returned `Invalid app`; controller/setup stop at step 1, without an observed native window. | [Stop](runs/OFFSET-R1-001/RUN-STOP-001.json) |
| OFFSET-R1-002 | Owned bundle selection worked. Initial selection AX reported rev 0; the later retained window was rev 2, 0 pieces, Unsaved. `New Empty` refused before rectangle authoring. The drift remains unexplained. | [Stop](runs/OFFSET-R1-002/RUN-STOP-002.json) |
| NATIVE-IDLE-DIAG-001 | A different instrumented binary did not reproduce the drift; `New Empty` succeeded at rev 0, 0 pieces, clean. No cause or fix was established. | [Outcome](runs/NATIVE-IDLE-DIAG-001/RUN-OUTCOME.json) |

Neither stopped offset attempt created authored geometry, snapshots, exports, or a grade. The diagnostic's planned 10/25/45 second snapshots were missed; the recorded first screenshot, File action, and New action timings remain 32.729, 64.801, and 71.068 seconds. The exact timing-deviation text is retained in the outcome and [summary](SUMMARY.json). All three owned-process cleanup records are retained separately from observations.

Review the offset attempt's [initial](runs/OFFSET-R1-002/observations/001-initial.png), [File menu](runs/OFFSET-R1-002/observations/002-file-menu.png), and [refusal](runs/OFFSET-R1-002/observations/003-new-empty.png) screenshots, and the diagnostic's [idle](runs/NATIVE-IDLE-DIAG-001/001-idle.png) and [successful New Empty](runs/NATIVE-IDLE-DIAG-001/003-new-empty.png) screenshots. These five files are byte-identical selected copies and show only the own-app window. AX records are retained diffs, not synthesized full trees. The initial selection AX was described in the original records/tool conversation; no missing full-tree artifact has been invented.

The [publication manifest](PUBLICATION-MANIFEST.json) records every selected original hash and derivative hash, plus omitted files and references. Text replaces only the exact private workspace prefix with `<private-workspace>`. The two final binding derivatives additionally omit one personal executable locator outside that workspace; their recorded executable version remains unchanged. Every transform and omitted JSON field is explicit. Fields historically named `unedited` consequently contain normalized path text in this derivative. Original inventory hashes identify original private bytes, not transformed public files. Frozen binding/plan statuses remain as recorded pre-run metadata; the stop/outcome records describe execution.

App binaries/bundles, saved-file evaluators, duplicate source trees, source-only audits/gates, and external authored scenario/contract packets are omitted. Their hashes and scope remain explicit; planned output paths absent at the step-1 stops are marked not created. Source provenance supplies no native, reference, material-calibration, or physics acceptance. No proprietary artifacts, unrelated browser context, or raw restricted reference evidence is included.

This bundle is local staging only. It asserts no Git tracking, registry/map verification, publication, or delivery and changes no feature credit. Tracking every artifact and clean-checkout registry/map checks remain required before delivery.
