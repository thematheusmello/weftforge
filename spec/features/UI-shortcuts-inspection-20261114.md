# Read-only settings inspection and blocked shortcut expansion

Case `REF-UI-SHORTCUTS-004` observes Marvelous Designer Personal
**2026.1.237 (r58920)**, executable SHA-256
`e7696388a5a5b8ca89bec40f7ab9016096610cd20f7ab50f21348ac73723c76e`.
Baseline revision 1 remains provisional and unfrozen. The
[structured observation](../observations/REF-UI-SHORTCUTS-004.json) distinguishes
the reached page from the blocked navigation and pins the retained records.

The declared scope was to inspect the previously collapsed Sewing shortcut
category, then accessible 2D Pattern creation/edit rows, without editing or
accepting any settings. **Only Graphic Options was reached in this run.** No
shortcut descendants, effective key bindings, accepted ranges or validation
errors were observed. The earlier [eleven-page pass](UI-preferences-runtime-20261114.md)
remains a separate historical observation.

## Visible current controls

`Settings > Preferences` opens Graphic Options. These values were reobserved:

| Control | Current visible state |
| --- | --- |
| Antialiasing | Off, x2, x4, x8 and x16 radio options; x4 selected |
| Normal Blending Method | Alpha Blending |
| Transparency Rendering Method | Naive Ordered |
| Set Maximum Image Resolution for Rendering | 4096 × 4096 pixels |
| Resolution-adjacent control | Unlabeled AX button with a visible question-mark icon |
| Window actions | Save as Preferences, Reset, footer Close, separate title-bar X |

These are configured values and exposed alternatives, not immutable factory
defaults or tested edits. No antialiasing option, selector option, help action,
Save as Preferences, Reset or title-bar X was invoked. The question-mark and title
control path names are reviewer descriptors for separate observed buttons.

The [component catalog](../inventory/md-2026.1.237-components.json) remains
byte-identical at 356 entries, within the coordinated 949-record inventory. These current radio
states and separate buttons are retained as observation detail. No new inventory
ID, shortcut descendant or binding is added by this retry record.

## Actual navigation boundary

The category names are visible in the private Preferences screenshot but absent
from its AX tree. Exposed Raise and ordinary exact-process activation/AXRaise
attempts do not initially establish foreground ownership. A fresh-image CUA click
on Shortcuts returns `AXError.notImplemented`. The optional `launch_app` API is
not callable in this runtime; it performs no action.

One ordinary macOS application activation then succeeds: a separate read-only
inspection confirms the identified executable, target process frontmost, and
Preferences window onscreen. The existing pinned native pointer **check** still
refuses the target with `Selected window is not the first normal CG window at this
point`. No native pointer run is invoked; the check posts zero events by its
operation contract. The guard policy and permissions are unchanged. The reason
for this window-gate mismatch is not inferred.

The exposed footer **Close** action returns the saved original garment workspace.
No preference value or binding was edited, and no save/reset/apply/acceptance action
was taken. Global settings-file byte equality was not measured. A failed host
navigation route does not establish application input-validation behavior.

## Preservation and evidence

The investigator's saved original reversed partial-sewing fixture remains
100,554 bytes with SHA-256
`0afcfe4281526e2250bef6b6d945bdc7e48f388b8236c765623bafb17702ba8d`.
After Close, the same named partial pair is selected in Edit Sewing, both 100.0
labels and uncrossed endpoint guides remain visible, and simulation is off. No
modal, unfinished gesture, process restart, account, network or permission action
occurs in this batch. The native container stays private and is not decoded.

| Stage | Retained evidence |
| --- | --- |
| Before Preferences | `ev_22fdf5e2c6ccbd6a31f9a0f20ab38d3a26cb04001828f03da5fb8d57e7a86ac6` |
| Initial page | `CUA-SHORTCUTS004-PREFERENCES-OPEN` |
| Foreground page | `CUA-SHORTCUTS004-FOREGROUND-PREFERENCES` |
| Refused pointer check | `CHECK-SHORTCUTS004-CATEGORY` |
| After footer Close | `CUA-SHORTCUTS004-AFTER-CLOSE` |
| Final main window | `ev_43eafac4eeb88048db8360bfdedb6bffb47aba7c8db83e611cae041ee0365069` |

REA observes the actual main window before and after, with 142 nodes and no
truncation or gaps. Foreground Preferences uses retained native AX transitions and
original screenshots; no REA modal result is fabricated. CUA navigation errors
also have a clearly labeled reviewed extract where the complete raw tool response
was not exported. Raw commercial/account UI and host paths remain private.

Shortcut expansion, active bindings and collisions, alternative graphics effects,
invalid input, resets and preference persistence remain unresolved. No WeftForge
comparison, complete broad feature contract or verified parity claim follows.
