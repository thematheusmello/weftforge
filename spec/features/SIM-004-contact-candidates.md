# SIM-004 — Conservative live contact candidates

Prospective implementation-only contract. The frozen Harbor inputs at source
6c455 require roughly 2.5 seconds per frame: live cloth projection visits every
triangle for every vertex on every iteration. The 420-second attempt remains
incomplete. Its resolution, settings, contacts and deadline must not change.

Reuse the existing triangle hierarchy to prune the same swept AABB predicates.
Keep `cloth_contact`, exclusions, thickness, normal/friction equations and
ascending triangle-index order unchanged. Bounds remain the same per-pass bounds
as the existing method; this does not add continuous collision completeness.

A contact correction can move the queried vertex into a later triangle's bounds.
After that happens, finish the remaining triangle indices with the original
linear scan. This preserves sequential behavior and bounds the worst case by the
old scan plus one hierarchy query. A single immutable candidate list is not
valid after such movement. Static diagnostic queries can use the same hierarchy
because they do not modify positions.

Acceptance, declared before candidate execution:

- Swept hierarchy candidates, after exact AABB filtering, equal exhaustive
  candidates on translated, crossing and inclusive-boundary cases.
- A point at y=.004 between pinned planes at y=0 and y=.0116, with cloth offset
  .003 on both sides, must visit the later plane after the first correction moves
  it to y=.006. Its final y is .0056 and both contacts occur. Preserve all other
  particles and the existing contact law.
- Existing supported contact/cloth tests, cancellation and whole-call rollback
  pass. Cancellation polling may become more frequent; callback counts are not
  an API promise.
- The original production control's 181 states and all retained Harbor prefixes
  agree exactly, excluding wall time. Compare the same authored geometry, mass,
  constraints, contacts and settings; do not substitute an easier scene.
- Repeat the unchanged Harbor A/B inputs once on the qualified new source,
  retaining the same 420-second/256-MiB limits and strict physical quality screen.
  Report observed timing separately from correctness; no speedup or completion
  is assumed before measurement.

Dense contact may fall back to the old scan. Edge-edge collision, full deforming
CCD, equilibrium, calibration, layered garment quality and reference parity
remain separate requirements.
