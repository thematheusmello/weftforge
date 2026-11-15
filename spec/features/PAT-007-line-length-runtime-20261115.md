# PAT-007 — Boundary length display observation

Case `REF-UI-LENGTH-OBSERVATION-20261115` executed on Marvelous Designer Personal
2026.1.237 (r58920), executable `e7696388…c76e`, baseline revision 1 provisional.
The [structured record](../observations/REF-UI-LENGTH-OBSERVATION-20261115.json)
retains evidence identities and limits. This observation postdates the frozen
[engineering contract](PAT-007-authored-length-feedback.md); its implementation
acceptance criteria are unchanged. This is not a paired implementation comparison.

A fresh pinned process opened a new copy of the original curved, unsewn two-panel
checkpoint, SHA256 `6d2280542cdccb84c09f46ef709c4c20e3152a01cf1fc42e775aff7b10edcffd`.
The earlier [sewing observation](SEW-002-paired-runtime-20261114.md) records its
200 × 300 mm rectangle provenance. Exact controls/geometry were not measured here.

With both outlines unselected in Transform Pattern mode, invoking **Display >
2D Information > Show Line Length / Angle** added per-boundary canvas labels:
curved 206.9 mm, straight 200.0 mm and 300.0 mm. The actual labels printed literal
`mm` with a space and one fractional digit, including `.0` on the straight values.
Thus this display includes unselected boundaries.

Switching to Edit Pattern, then **Edit > Select All**, highlighted both panels.
The canvas labels remained. Separately, Property Editor’s **Selected Line**
section showed **2D Line Length: 2006.9mm** and **3D Line Length: 2006.9mm**.
The general aggregation rule is unknown. A second display-toggle invocation
removed the canvas labels while selection highlights and both aggregate
readouts remained. The toggle and selected-line readout are separate in this case.

| State | REA evidence |
|---|---|
| Checkpoint opened without labels | `ev_04a1d0c17ba672c0900af8f942303abe86cbe44922a3c6492cdf3945ce4c65bb` |
| Labels on unselected boundaries | `ev_57de3af4efecdd8b136fe2910a4a302e59b4f3338a4c0a9e38fe25f76b834d7b` |
| Select All and aggregate readouts | `ev_65487e6acfebeee02c5bdb92d66872843bb085095934bcb36016193e5148a76d` |
| Labels restored off, aggregate retained | `ev_9894e6fc06190c2a0e0bad85feb463df91d9e3b9ae1c1113c6908c9eb8323f06` |

These four captures are untruncated and gap-free. One intended single-edge
coordinate click returned `-10005:noWindowsAvailable`; no cause is inferred and
no retry or pointer fallback ran. Fresh before/after REA raw/PNG bytes and evidence
ID were identical. This establishes no single-edge selection result.

The copied fixture remained unchanged, simulation stayed off and no authoring,
Save, history or export ran. The final own reference process retains Select All
with the display toggle restored off. Raw commercial/account UI, native files
and process/path inventories remain private. Root reviewed the investigator’s
handoff, manifest and snapshot hashes; full raw image/trace auditing is not claimed.

Unknown: exact arc length/algorithm, controls, rounding versus truncation and tie
rules, angles/point labels despite the menu name, individual edge selection,
alternative units, other settings, general aggregation and restart persistence.
One fractional digit describes these displayed values, not a geometric tolerance.

The current WeftForge increment chooses three-decimal millimetres in a selected
boundary inspector and raw seam/group readouts. The reference canvas toggle,
Select All aggregate and angle behavior remain separate pending work; this
engineering precision/scope choice is not claimed as equivalent display behavior.
