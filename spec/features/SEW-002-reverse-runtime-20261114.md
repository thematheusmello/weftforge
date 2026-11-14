# SEW-001 / SEW-002 — Explicit reversal of an original partial pair

Case `REF-SEW-002-REVERSE-003` was executed in Marvelous Designer Personal
**2026.1.237 (r58920)**, executable `e7696388…c76e`. Baseline revision 1 remains
provisional and unfrozen. It extends the [paired 100 mm case](SEW-002-paired-runtime-20261114.md)
with one explicit Reverse Sewing operation, Undo/Redo and document reopen. The
[structured record](../observations/REF-SEW-002-REVERSE-003.json) pins evidence
and limitations. This is categorical UI correspondence evidence, with no
continuous arc-length, hidden orientation flag, simulation or parity claim.

## Protected original input and declared check

Start from the separately saved original partial result, SHA-256
`38a723294c8f58160da1577b29b775b63b2a0daeab16dd337e57b618fe1b63f3`.
It contains a 100.0-labeled interval on the upper curved boundary and a
100.0-labeled interval on the other panel's top straight boundary. Keep simulation
off. In Edit Sewing, select the pair and save a new working copy before reversal.

Use reviewer names for the visible endpoints:

| Name | Landmark |
| --- | --- |
| A0 | Curved interval's left boundary endpoint |
| A1 | Curved interval's interior endpoint |
| B0 | Straight interval's right interior point, the original second-side start |
| B1 | Straight interval's left interior point, the original second-side end |

Before reversal, dashed guides connect A0–B0 and A1–B1; they cross in this layout.
The predeclared check is a categorical change in those endpoint connections while
the same visible intervals and both **100.0** labels remain. Millimeter is a
separately observed application setting; these canvas labels have no printed
unit. This run supplies no sub-display world-coordinate or continuous arc-length
tolerance. Endpoint names are reviewer descriptors, not printed UI labels.

## Observed command and history

Execute `Edit > Context Menu > Sewing > Reverse Sewing` once. The command changes
the dashed guides to **A0–B1 and A1–B0**, which do not cross in this layout. The
same visible boundary intervals and both 100.0 labels remain. The curve interval's
directional mark changes. The selected seam retains the same generated name and
current property values. No modal appears during the command.

One `Edit > Undo` restores the crossed A0–B0/A1–B1 guides. One `Edit > Redo`
restores the uncrossed A0–B1/A1–B0 guides. The pair remains selected, with the same
side labels, seam name and property values through both history commands.

Current values remain Turned Off, Fold Strength 5, Fold Angle 180, Tension Off,
Set Sublayer Off, Topstitch Off, Puckering Off, Plain Seam, normal-map intensity
10 and thickness 1.5 mm. No unit is printed for Strength, Angle or Intensity.
No property edit, physical fold effect or factory-default guarantee follows from
these unchanged values.

The route is the nested main-menu command exposed by Edit. A canvas context menu,
shortcut, toolbar reversal route and multiple-selection behavior were not tested.
Only normal exposed AX menu actions were needed for this case; the private
pointer fallback was not invoked.

## Save and document reopen

Save As another result, use `File > New Project`, and observe the empty project.
Reopen the reversed result through Recent. Activate `Sewing > Edit Sewing`, then
`Edit > Select All`. The same named seam/settings, both 100.0 labels and
**uncrossed A0–B1/A1–B0 guides** return.

The result is 100,554 bytes, SHA-256
`0afcfe4281526e2250bef6b6d945bdc7e48f388b8236c765623bafb17702ba8d`.
Its hash is unchanged after reopen. The original unsewn checkpoint, whole-edge
result, unreversed partial result and protected reversal working copy also retain
their pinned hashes. Native containers stay private; their internal format was
not inspected. This is a same-process document close/open loop, not a process
restart.

Both Save As actions displayed the retained low-space warning. During reopen,
the same warning remained after the first Proceed; the second Proceed completed
the open. Distinct modal instances versus repeated handling remain unknown. No
Open Project options modal was observed in this result-reopen sequence. The
warning's underlying resource check and consequences are not inferred.

## Retained evidence and limits

| Stage | REA evidence ID |
| --- | --- |
| Before reversal | `ev_7ac3349f5487519cf1f3a040ad62689229663199b78a6974e735980e5ddf5575` |
| Reversed | `ev_9be8bbe9f30b99d514df09a9ef5fe6dee13f98323b0ae768a232264e8de48900` |
| One Undo | `ev_3f236b3ff2b13d0ee86e6240748ad2a53d05a93c3243d29795f3cb31413cf26a` |
| One Redo | `ev_ef65ab2cdc160917b64930651487d9aac5dc35f029cbb8206addfe054385af3e` |
| Saved | `ev_cd17627a53747a24fcbfb1a1a9fa48dfcdbbbc62a8c62781f27046136ef85c52` |
| Empty new project | `ev_b7fed2b95cd76e8694666113a5a753a2e081791399f089001fbd183e86ad393c` |
| Reopened and reselected | `ev_43eafac4eeb88048db8360bfdedb6bffb47aba7c8db83e611cae041ee0365069` |

The first five selected captures each have 139 nodes, `truncated: true`, and five
AX -25202 attribute failures at unrelated path `[41,4]`. Their target seam fields
and screenshots are present. The final reopened selected capture has 142 nodes,
no truncation and no gaps. A separate private comparison found equal target
field sequences across all selected stages. It does not repair the missing
unrelated attributes. Raw commercial/account UI remains private.

A color-only selection change without a connection change would fail this case.
So would moving either interval, changing a side label, or an Undo that fails to
restore the former guides. Passing this case still does not establish exact curve
parameters, continuous arc-length pairing, unequal-length easing, physical twist,
sewn remeshing or solver behavior. Those require separately declared original
fixtures and independent measurements. Broad feature contracts and verified
parity remain incomplete.
