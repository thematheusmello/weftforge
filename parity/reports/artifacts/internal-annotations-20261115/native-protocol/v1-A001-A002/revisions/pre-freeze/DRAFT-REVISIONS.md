# Prospective draft corrections

All corrections preceded any native/candidate output or final binding/freeze.
The initial PROPOSAL.md and earlier passing selfcheck reports are preserved.

- Root requested one initial native Save As followed by normal File > Save and
  exact immutable raw copies, replacing repeated Save As for each history state.
  A now also saves actual empty R0 without changing authoring/history.
- Static route review showed ordinary width/height focus/Tab can commit two
  ResizePiece actions; B explicitly expects width R9 and height R10, then R11
  transform. No unsupported same-frame grouped resize is inferred.
- Owner deletion now occurs after Undo restores the removed line and inspector
  reselects it; this actually tests deletion of an owner containing both internal
  endpoints, rather than deletion of an already annotation-free owner.
- Explicit procedure counting added unchanged editor, invalid modal with a real
  Redo branch, successful clean owner-change invalidation and focused-invalid-raw
  Escape. The original 900-call aggregate was optimistic; current maxima/caps are
  the counted values in STATIC-BUDGET.json and PROPOSAL-v2.md.

No preparation command/test has failed. Earlier draft selfcheck reports validate
only the script/model bytes current at their run; current reports are
SELF-CHECK-DRAFT-005.json, SELF-CHECK-FILES-001.json and SELF-CHECK-SNAPSHOT-001.json.
No report is a native/product acceptance result or a frozen artifact binding.
