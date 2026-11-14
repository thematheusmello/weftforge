# SEW-002 — Two numeric partial sides, history and document reopen

Controlled case `REF-SEW-002-PAIR-002` was executed in Marvelous Designer Personal
**2026.1.237 (r58920)**, executable `e7696388…c76e`. Baseline revision 1 remains
provisional and unfrozen. This records a completed original Free Sewing pair;
continuous arc-length correspondence, explicit reversal, simulation and WeftForge
comparison remain unmeasured. The [structured observation](../observations/REF-SEW-002-PAIR-002.json)
pins the retained records and these limits. The earlier
[first-side-only attempt](SEW-002-runtime-20261114.md) remains a separate case.

## Original input and declared measurement

Open the saved original two-panel curved checkpoint, SHA-256
`6d2280542cdccb84c09f46ef709c4c20e3152a01cf1fc42e775aff7b10edcffd`.
Each original rectangle was authored as 200 × 300 mm; the upper panel's bottom
boundary was curved in the earlier controlled gesture. Its full selected edge
then displayed 206.9 mm. Its exact control points and continuous arc length are
unknown. Save a new working copy before partial sewing. Keep the separate
whole-edge result, SHA-256 `43eba999…b02`, unchanged.

The target is **100 mm on each side**: begin at the upper curved boundary's left
endpoint, then choose an interior point in the right half of the other panel's
top straight boundary and extend leftward. Acceptance is the literal 100 value
with a printed mm unit in both dialogs, two 100.0 canvas labels, and a completed
selected pair. The canvas does not print units beside these labels; Millimeter
was separately observed in Preferences. No finer geometry tolerance, exact
second-start authoring coordinate or curve-parameter relationship is inferred.
Simulation stays off throughout.

Opening the original checkpoint exposed **Open Project** options: Open selected,
Add unselected, Garment and Display Settings checked. Avatar, Scene and Props,
3D State and Custom View were gray and unchecked. Reset, OK, Cancel and a close
control were present. The current options were accepted unchanged with OK;
alternative options, Reset and cancellation were not executed.

## Executed paired operation

1. Activate `Sewing > Free Sewing`. Both toolbars expose checked Free Sewing
   labels; the 2D label displays M. The shortcut and 3D canvas route are untested.
2. Click the upper curved boundary's left endpoint, called A0 here. A cyan start
   marker and zero-length preview appear.
3. Right-click near that start along the curve. **Create Sewing Line** opens with
   Length **8.0 mm** at this pointer position. Replace it with `100`, commit with
   Tab, observe Length 100, and click OK. The curve interval A0–A1 displays
   **100.0** and the preview reads `100.0 (100.0 / 0.0)`. This stage alone is not a
   completed seam.
4. Click an interior point, B0, in the right half of the other panel's top edge.
   Right-click leftward along that straight edge. A **second Create Sewing Line**
   dialog appears, initially displaying **100.0 mm**. Explicitly replace it with
   `100`, commit with Tab, then click OK.
5. A named selected seam and its properties appear, with **100.0 on both sides**.
   Both intervals have directional marks. The dashed endpoint guides connect
   A0 to B0 and A1 to the leftward B1. They cross in this fixed 2D layout. A0/A1
   and B0/B1 are reviewer endpoint names, not printed UI labels. The guides do
   not establish a physical twist, hidden orientation flag or arc-length map.

The first dialog's 8.0 value and the second dialog's 100.0 value describe this
pointer state. Neither is claimed as a factory default or a snap algorithm.
The official [Free Sewing description](https://support.marvelousdesigner.com/hc/en-us/articles/47358239854361-Free-Sewing-2D)
supports the entry route; the numeric outcomes above come from this runtime case.

## History and document persistence

One `Edit > Undo` removes both paired traces/guides and the selected-seam panel,
while both original outlines remain. One `Edit > Redo` restores the same generated
seam name, both 100.0 labels, crossed guides and current settings.

The current settings match the earlier whole-edge case: Turned Off, Fold Strength
5, Fold Angle 180, Tension Off, Set Sublayer Off, Topstitch Off, Puckering Off,
3D Seamline Plain Seam, normal-map intensity 10 and thickness 1.5 mm. Units are
not printed for Strength, Angle or Intensity in these captures. No property edit,
range, rendering effect or universal-default claim is made.

Save As a separate result, then use `File > New Project` and observe an empty
project. Reopen the saved result through Recent. Activate `Sewing > Edit Sewing`,
then `Edit > Select All`: the same named pair, property values, both 100.0 labels
and crossed guides return. This is a document close/open loop in the same
process, not a process restart.

The saved result is 101,039 bytes, SHA-256
`38a723294c8f58160da1577b29b775b63b2a0daeab16dd337e57b618fe1b63f3`.
Its hash remains unchanged after reopen, as do the original input and separate
whole-edge result. Native containers remain private and their internal format
was not inspected.

Save As displayed the observed warning about less than 10 GB of space and possible
file damage. During result reopen, the same warning remained after the first
Proceed; the second Proceed completed the open. Both warning images are retained.
Distinct modal instances versus repeated handling are not established. No Open
Project options modal was observed in this result-reopen sequence. The resource
checked by the warning is unknown.

## Input and capture qualification

The ordinary CUA canvas click returned `-10005 noWindowsAvailable`; before/after
REA content was identical. The approved, pinned public native pointer fallback
then supplied four individually guarded clicks, each reporting three events and
corroborated by fresh after-state. Three other right-click **checks** refused a
foreign AX input owner and posted no events. Foreign windows and permissions
were untouched. No ambiguous gesture was repeated.

| Stage | Retained REA evidence ID |
| --- | --- |
| First 100 mm accepted | `ev_8a8a71f5ecc06a23cb6aaf07e7124a5bf94c892dbaad5aaec1ec2c99869c550a` |
| Second interior start | `ev_1fae79dc906406d631a784f05e3b62e15f33508eeb93fd33a0ebb217ae039f9b` |
| Completed pair | `ev_4dc0363563aea0b35e5eb08a75ba7e4482f48451d8ef3c07bc7db3a0dbdbbfc7` |
| One Undo | `ev_237cc2c4a2fcc775b13301449f642aa75993b0ed5cfae84fb9938c252e260f7e` |
| One Redo | `ev_bd944b06067864732f9bf8f1d556409ca7767aa455623f76aeecd3bcec1f7542` |
| Saved result | `ev_b10b0d39f9ec174059cc5bd5bd3d392197166ce7f53b5c8b3a1c36a858062d10` |
| Empty new project | `ev_dd36190631c5f79907e99f70bb20275d431c57bfa7227c5dd759b1580cced718` |
| Reopened and reselected | `ev_ee12dda8bc7aab2f76f860f310f5116a31725c7709df9870d0516d102c63e632` |

Creation, Undo, Redo and saved captures are gap-free and untruncated. The final
reopened capture has 139 nodes, `truncated: true`, and five AX -25202 attribute
failures at unrelated path `[41,4]`. The screenshot and target seam fields are
present. A private comparison confirms equal target field sequences across
creation, Redo and reopen; it does not restore missing unrelated attributes.
Raw commercial/account UI and native files remain private.

Useful counterexamples for future comparison include a first-side preview with
no pair, a second-side dialog that is never accepted, a full-edge pair substituted
for these partial intervals, and an Undo that removes only one visible side.
Those outcomes do not satisfy this bounded case. This successful case does not
qualify invalid lengths, cancellation, reversed mapping, unequal lengths/easing,
sewn remeshing, solver closure or broad sewing parity.
