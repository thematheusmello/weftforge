# Native seam reversal — bounded implementation contract

The Sewing inspector exposes **Reverse** on each seam row, identified by seam
name and ID plus both curve IDs and oriented parameters. Widget identities use
the seam ID, including rows with duplicate names produced by splitting a sewn
curve. This implements the categorical endpoint-pairing change described by the
[reviewed reversal case](SEW-002-reverse-runtime-20261114.md), within WeftForge's
[oriented-interval model](SEW-001-oriented-segment-sewing.md).

Reverse clones that seam, exchanges side B's `t0` and `t1`, and submits the existing
typed `Command::UpdateSeam`. It preserves the seam ID, name, array order, side A,
covered curve intervals and normalized correspondence. Other construction and
material data are unchanged by reversal. Two reversals restore the exact
construction, apart from document revision. Intervals are curve parameters,
not physical or normalized arc lengths.

Both endpoint guides appear in the 2D canvas. For the same covered intervals,
the guides change from A0–B0/A1–B1 to A0–B1/A1–B0. Interior nonlinear
correspondence remains stored and used by remeshing; the two guides visualize
only endpoint pairing.

Pending inspector edits and Reverse commit together as one undoable transaction.
A failed transaction retains the document, redo history, staged values and
focused edit text. Successful actions clear numeric edit caches to prevent old
text from replaying after history commands. Reversed intervals persist through
the existing v2 project format. The CLI's local MCP `project_execute` schema
already accepts `update_seam` with the full seam in an atomic command array;
no new public command or persistence field is needed.

The focused regressions use original curved/straight partial intervals, a
nonlinear normalized map, both initial orientations, remeshing, two reversals,
history, v2 serialization and rejected staged transactions. A full headless
`App::ui` regression exercises AccessKit clicks, pointer clicks and coalesced
focus, valid pending numeric edits, a rejected revision-limit transaction,
endpoint drawing, Undo/Redo, save/open and split-generated duplicate seam rows.

These are implementation checks. Desktop input acceptance, reference parity,
continuous arc-length pairing, unequal-length physical easing and simulated
twist are unverified. The reference's additional sewing properties are not
modeled by this bounded change.
