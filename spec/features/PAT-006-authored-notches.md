# First authored notches

WeftForge engineering scope, 2026-11-17. This source candidate adds editable
perpendicular I markers attached to existing construction curves. It is not a
complete notch feature, reference observation, native acceptance or roadmap gate.
No installed reference, private research or proprietary captures were used.

## Public documentation and independent decisions

The official CLO manuals consulted on 2026-11-17 describe creating notches on
pattern outlines for sewing alignment, moving selected notches along outlines
or internal lines, editable shape/angle/length, and explicit deletion:

- [Create Notch](https://support.clo3d.com/hc/en-us/articles/115013214287-Create-Notch)
- [Select/Move Notch](https://support.clo3d.com/hc/en-us/articles/115013214367-Select-Move-Notch)
- [Edit Notch Properties](https://support.clo3d.com/hc/en-us/articles/115013369268-Edit-Notch-Properties)
- [Delete Notch](https://support.clo3d.com/hc/en-us/articles/115013215447-Delete-Notch)

These are documented product descriptions, not executed behavior of the frozen
installed reference. They do not establish WeftForge's parameterization,
topology remapping, default size, geometric limits, persistence or transaction
rules. The decisions below are independently authored engineering policy.

## Document and numeric contract

Every piece owns a required ordered `notches` array. Each notch contains exactly
`id`, `curve`, `t` and `length`. IDs are nonzero globally unique u64s, counted
against the existing 20,000-object limit and below `next_id`. The curve must be
owned by that same piece: exterior, hole boundary or internal annotation.

`t` is a finite dimensionless straight/cubic curve parameter in `[0,1]`, not
normalized arc length or a fixed physical distance. The anchor is `curve(t)`.
The marker is a centred line segment perpendicular to the analytic first
derivative at that anchor. Reversing a curve and replacing `t` by `1-t` swaps
marker endpoints without changing its visible segment. At a corner the addressed
curve determines the tangent; no bisector or corner-direction rule is implied.

Total marker length is finite SI metres in `[0.0001,0.1]`. Native creation
defaults to `t=0.5` and `length=0.005`; its editor displays millimetres. The
derivative magnitude must exceed `1e-12` metres. A stationary first derivative,
including coincident endpoint controls, refuses rather than guessing a direction.
The anchor and both marker ends must be evaluable and within the existing finite
coordinate bound. These size/precision limits are WeftForge choices.

Notches are authoritative annotations. They create no construction points,
curves, topology, holes, folds, sewing, mass or solver constraints. Authoring
still advances the document revision and invalidates previous derived state.

Whole-document marker passes index points and curves once and index owned curves
once per notched piece. Each marker then uses indexed membership/geometry queries
and the same canonical tangent/marker calculation as single-marker evaluation.
Validation charges index construction and queries to the existing shared request
budget/cancellation poll. SVG reuses one result for bounds and output; native
drawing and framing each use one batch, with no persistent geometry cache.

## Editing and relationships

`add_notch {piece,curve,t,length}`, `update_notch {notch}` and
`remove_notch {notch}` use the existing typed command engine. Update preserves
the notch ID and owner; a different curve within the same owner is permitted.
Cross-piece reassignment rejects. Generic CLI `execute` and MCP
`project_execute` expose the same operations; inspection includes notch data.
The MCP command schema has 36 alternatives; the existing nine tools remain.

The native selected-curve inspector provides Add, Edit and Remove. Its modal
edits parameter and length with strict text parsing. Apply or Enter publishes
one transaction with pending valid property edits; failure or cancellation
retains construction, history and raw pending input. Unchanged editing retains
length bits and does not add history. Stale revision, owner, curve or tool
selection cancels/refuses. The 2D canvas draws physical-scale markers and labels
selected-curve notch IDs; framing includes marker ends. Full canvas creation,
selection and drag gestures remain open.

Point/control edits, resize and rigid transforms retain IDs and `t`, re-evaluate
the anchor/tangent and retain the physical length. Invalid attachments reject
the complete candidate. Split at `s` remaps a notch on the first portion to
`t/s` when `t<=s`, otherwise to the second curve at `(t-s)/(1-s)`. A notch exactly
at the split remains at the first curve's end. Compatible exact joins use the
existing recovered split parameter `s`: first-curve `s*t`, second-curve
`s+(1-s)*t`. Neither operation creates duplicate notches.

Cut/Cut & Sew and outline offset refuse pieces with notches. Removing an attached
hole or internal line refuses until its notches are explicitly removed. Removing
the whole owner intentionally removes all owned notch data and geometry and is
undoable. No supported edit silently drops a retained notch relationship.

## Persistence and export

Native version 6 strictly requires the notch array and fields. Frozen v1–v5
readers preserve their supported fields and add empty notch arrays without
changing IDs or authoring revision; only an explicit save writes v6. Older
versions reject the newer field, including an empty array. Atomic save/recovery
behavior and the 16 MiB limit remain in force; no downgrade writer is added.

SVG writes physical millimetre notch line elements with stable notch/owner
identifiers, inverted Y and bounds covering marker ends. The bounded SVG importer
explicitly refuses these unsupported line elements, so interchange is not a
native editing round trip. DXF refuses projects with authored notches before
staging/replacing output. OBJ is triangle mesh only and omits all annotation
markers by design. Save the native document to retain editable attachments.

## Validation boundary and remaining work

Authored tests cover numeric/ownership/tangent rejection, analytic cubic and
reversed markers, endpoint and curved split/join remapping, ordinary edits,
history, strict migration, existing-file retention, SVG physical scale/bounds,
CLI/MCP equivalence and native headless interaction. They are source tests, not
reference or live native evidence. Initial authoring occurred while compute was
reserved for a separate garment-verification job; no test/build/UI result is
claimed here.
Budget/cancellation regressions use a clean offset target and an unrelated
512-edge piece with 1,024 notches, check bounded indexed work and cancellation
during marker queries, and retain document/history and existing native/recovery/
SVG destinations on refusal. They do not establish broad performance parity.

Angle/shape/flip variants, corner-direction variants, names, seam allowances,
fixed-distance positions, grading, matching notch relationships, DXF notch
conventions and full canvas gestures remain open. The broad PAT-006 contract and
all reference-equivalence statuses remain unchanged and unverified.
