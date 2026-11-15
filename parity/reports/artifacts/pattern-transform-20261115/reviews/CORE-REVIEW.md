# Core and automation source review

What this change does: `TransformPiece` changes a single piece's authored 2D
layout, including absolute cubic controls. It compensates 3D placement and rotates
grain so the continuous arranged surface and world material directions stay fixed.

No concrete correctness or validation findings in the reviewed source scope.

Reviewed immutable commit `02bcd8c833376d2ba7c007c7dcd2516337d8a02f` against
`be8a031b5f8342d0857f3bed34f52dd165fa86da`, using `git show`/`git diff`.
The frozen contract SHA-256 was verified as
`afffdf4e68ca5afb7f0038d2568576feb66824f873a8aaa659f60bfc072bf557`.

Checked:

- `document.rs:986-1052`: endpoint ownership follows validated closed boundaries;
  the start-ID set covers every endpoint once. Absolute controls move with their
  curve. The command does not allocate IDs or alter ordered construction/sewing
  records, other pieces, materials, resolution, or mannequin fields.
- `document.rs:996-1025`: unknown pieces, finite bounded input, and angle range are
  checked. Translation uses one componentwise addition and preserves zero axes;
  identity and negative-zero angle take that same path. Grain and placement Euler
  values retain their bits for pure translation.
- `document.rs:117-166`: the basis composition implements `A * Rz(-angle)`;
  translation enforces the pivot identity. The cross-product reconstruction makes
  the near-singular Euler phases coherent. The fixed 32-epsilon branch remains
  unchanged. The ordinary and singular extraction formulas have the correct
  signs for the existing XYZ convention.
- A separate source-level numerical analysis checked `A' * R2(angle) = A` on
  both in-plane basis vectors for 40,000 deterministic cases, including large
  supported angles and exact/near-gimbal inputs. The maximum component error was
  approximately `1.37e-14`. This was a math investigation using Python's binary64
  trigonometry, not execution of compiled Rust or the frozen independent checker.
- `document.rs:1043-1050`: grain rotates as a unit vector and canonicalizes with
  `atan2`; it does not add to an arbitrary large finite stored angle. Existing
  mesh UV and solver material consumers use the resulting authored coordinates
  and grain consistently.
- `document.rs:390-648,1565-1631`: clone/apply/validate happens before any project,
  revision, undo, redo, or allocator replacement. Result endpoints, controls,
  placement, grain, contour, and affected ordered sewing metrics are validated.
  Late transaction failures leave the existing editor untouched.
- `apps/weftforge-cli/src/main.rs:139-155,365-379` and `mcp.rs:384-387,435-444`:
  both routes deserialize original bytes into the shared command and save only
  after the single transaction succeeds. The transform schema requires all four
  fields, fixed two-element arrays, a nonzero ID, bounded numbers, and no extra
  object fields.
- The eight core test functions cover the analytic quarter-turn polynomial and
  reversed partial lengths, a non-right-angle inverse, tilted and singular
  placements, large coordinates/grain, bit preservation, relationship/material
  preservation, stale mesh rejection, v3 persistence, history/redo preservation,
  control-only overflow, compensated-placement overflow, and strict JSON tokens.
  The added stdio test checks schema discovery, CLI/MCP equivalence, malformed
  original requests, late failure, and destination/recovery byte preservation.
- `docs/architecture.md`, `docs/automation.md`, and `docs/file-formats.md` match
  the command, units, transform order, placement/grain policy, revision behavior,
  and unchanged v3 fields. They retain the reference-behavior unknown.

Verdict: no source blocker found for this core/automation increment.

Not checked: mutable native `main.rs`, compiled candidate behavior, Cargo gates,
the independent 93-case checker, CUA/native/reference execution, publication,
registry/map verification, or Actions. Reported test/Clippy passes were not rerun.
The existing sewing-metric validator covers ordered groups; legacy seams retain
their structural validation and compute lengths on demand. This review establishes
no reference parity and does not authorize publishing evidence from ignored files.
