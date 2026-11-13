# PAT-008 — authored grain and orthotropic membrane stiffness

This contract describes WeftForge's bounded implementation. It is not an observed
Marvelous Designer runtime contract or a parity comparison.

The public [stretch/weft/warp/shear manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358420149017-FABRIC-PHYSICAL-PROPERTIES-Adjust-Stretch-Weft-Warp-Shear)
describes distinct directional resistance controls. The public
[2D Snapshot manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358119414937-2D-Snapshot-2025-0)
includes grainline display among snapshot options. Those documentation leads do
not establish reference grain-angle defaults, exact angle/editing conventions,
physical parameter conversion, solver response or persistence semantics. Reference
execution remains blocked at legitimate terms/sign-in/activation.

## Independent authoring behavior

- A pattern piece owns `grain_angle`, f64 radians counterclockwise from material
  +U toward +V. The angle identifies its warp axis; weft is perpendicular. Zero
  points right in the V-up pattern canvas. Placement rotates the arranged cloth
  without changing that material-space angle.
- Each material separately owns positive finite warp/weft membrane stiffnesses in
  N/m. Shear, flexural rigidity, mass/area, damping, friction and collision offset
  remain distinct. Display color does not change physical values.
- `SetGrain` and `UpdateMaterial` use the shared validated command engine and can
  participate in one undoable transaction. They advance document revision and
  invalidate prior solver/rest correspondence. Invalid commands retain the old
  project and history. A material edit affects all pieces assigned to it; grain
  remains specific to each piece.
- The native inspector keeps a draft until numeric commit/focus loss, draws the
  actual committed warp arrow in 2D and exposes separate stiffness controls.
  Inspecting or recoloring a valid imported project must not silently clamp its
  physical values or pattern dimensions.
- CLI `grain INPUT OUTPUT PIECE_ID RADIANS` and
  `stiffness INPUT OUTPUT MATERIAL_ID WARP WEFT` call those same commands.
- The CPU bridge passes authored angle and both stiffnesses to the existing solver.
  The bounded ground-loaded anisotropic patch must respond differently after a
  quarter-turn, while a color-only change preserves its solved positions.

## Native compatibility

Version 2 writes the angle and both stiffnesses explicitly. Strict v1 migration
copies `stretch_stiffness` to both axes and sets each piece's angle to zero,
matching the old solver inputs. Every other authored field, ID and revision is
retained. Existing v1 fixture files remain unchanged. Unknown/duplicate fields,
mixed version schemas, unsupported versions and invalid values are rejected;
16 MiB loading limits, atomic save/recovery and failed-operation preservation stay
in force. Old readers cannot read v2. No downgrade writer is provided.

## Acceptance boundary

Implementation tests cover strict migration, history, rollback, command-line
editing, numerical bridge response and color isolation. Native interaction,
reopen/export inspection and evidence tied to an identified executable must be
performed separately. Fabric values are uncalibrated approximations. No test in
this increment establishes reference parity or repairs retained garment/contact
and convergence failures.
