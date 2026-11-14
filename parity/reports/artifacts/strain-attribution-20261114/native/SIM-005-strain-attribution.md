# SIM-005 — Material strain attribution

Implementation contract, declared before executing the new candidate. This is
an original diagnostic requirement, not an observed reference contract or a
change to fitting acceptance. The original shirt failures remain required data.

The existing maximum absolute material-axis length strain keeps its definition
and value. A read-only `peak_strain` additionally identifies the current solver
triangle, warp or weft direction, and signed length strain (positive extension,
negative compression). Exact ties choose the first input triangle, then warp.
Triangle indices belong to the current rest mesh and are not authored IDs.
Default or empty diagnostics may have no attribution. Old diagnostics JSON
without the added field must remain readable.

The desktop inspector shows the current pattern name, material direction and
signed percentage. CLI simulation/benchmark and both fitting phases/final
report expose the same typed measurement and resolved pattern ID/name. A stale
mesh revision or absent triangle cannot resolve a piece. No authoring command,
project format, solver parameter, contact rule, or acceptance limit changes.

Acceptance:

- Analytic affine extension/compression with rotated grain identifies the
  correct axis and signed value within 1e-12 absolute strain. Exact constructed
  ties retain deterministic input order. Cancellation and reset preserve the
  existing state guarantees.
- The unchanged original shirt/control, at 8 substeps and 32 iterations with
  60 zero-gravity assembly frames followed by 120 gravity frames, has identical
  mesh, positions, velocities and prior diagnostics at every retained frame,
  except measured wall time. Attribution agrees with independently reconstructed
  material gradients within 1e-12 strain; nonzero decisive peak indices and
  directions agree. Roundoff-only rest strains below 1e-12 do not establish a
  physically meaningful peak location.
- Independent expected peaks are frozen before observing new executable output.
  The earlier saved-output checker is independent of the new diagnostic code.
- The actual native executable displays the appropriate original shirt pattern
  and signed direction after assembly and drape; the CLI retains its failed
  quality outcome and prior export when the unchanged 15% / 5 mm / 2 mm limits
  fail. New fields cannot promote a failed garment to acceptance.

Complete force balance, principal strain/shear bounds, fabric calibration,
convergence, reference parity and full SIM-005 coverage remain separate work.
