# New Empty document action

Reviewed engineering increment, 2026-11-15. The native transform A3 workflow at
`699c4608ff68af132ec90b1ffb80ba354c70435f` stopped because neither File nor
command search offered New Empty. The original case remains unverified. This
contract is approved before implementing or running the new action; reference
New Project semantics remain unknown.

Expose **New Empty** in File and command search, through the same action path.
Create a default editable Project/Editor at revision 0, with no path, selection,
undo or redo. Clear document drafts, construction gestures and derived mesh/cloth
state, stop simulation and cancel old document workers so their results cannot
be accepted into the new document even when revision numbers coincide. Keep the
window and session preferences, including Snap and view settings. Do not write,
delete or modify any old primary, autosave or recovery file.

Protect unsaved work using the existing save-before-replacement convention:
refuse New Empty while the accepted document is dirty, while current inspector
commands or invalid/incomplete numeric text are pending, or while a construction
gesture is unfinished. Give a useful apply/cancel/save instruction. Refusal must
not commit that draft, clear history/Redo, lose raw text/caret, invalidate the
accepted mesh/cloth, or alter file bytes. Use the existing inspector focus and
action-blur guards across separate press/release and later frames. Defer the
replacement decision until the current inspector input pass is complete, as
existing import/export actions do. Authoring modals retain their existing input
shield. No discard option, new modal, shortcut, persistence version or dependency
is part of this increment.

Before native acceptance, verify both exposed routes with native event/AccessKit
checks, clean replacement and saved-document reopen, refusal for dirty/staged/
invalid input and unfinished gestures, raw text/caret retention across later
frames, cleared history/derived state, cancelled workers, and unchanged saved
primary/recovery bytes. Run the integrated local Rust/registry checks and build
a newly bound native executable. The original transform A/B fixtures, saved-file
grader, numeric/history expectations and call/time caps remain unchanged; use a
fresh process and case for later native acceptance. Old A/A2/A3 are not resumed,
repaired or regraded as completed workflows.
