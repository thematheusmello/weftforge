# Independent ordered sewing oracle — preregistration

This bounded acceptance oracle was designed before inspecting the candidate's
source, executable, derived output or screenshots. The reviewed behavior input is
`spec/features/SEW-002-ordered-paths.md`, SHA-256
`9baedd713b5a8dd4f251d7f2c5d0c51b408ecdabb32813a07dae06d25dcae960`.
The only historical schema examples read were the public original fixtures
`fixtures/original-shirt.weftforge` and
`fixtures/sewn-curve/before-split.weftforge`. Their geometry is not an oracle input.
All new geometry, maps and values were authored independently here.

The exact rational line witness has A lengths 0.1 and 0.3 m and B lengths 0.2,
0.3 and 0.1 m. Identity mapping pairs A's first cumulative knot at 1/4 with B
length 0.15 m. That A knot has two distinct physical positions, approximately
(0.15, 0) and (1.45, 1), which must both survive. Stored binary64 values are used
in the independent numerical answer; their sub-ulp decimal differences do not
change the 1e-8 m authored budget. The 2:3 spans are physically disconnected on
five original pieces. No connector contributes any length.

The second line map is explicitly nonlinear. Partial and reversed intervals
occur on both sides. The collinear cubic has very nonuniform control spacing;
its interior map knots force arc-length inversion even if no mesh is inspected.
The asymmetric case has different curved cubics and a collinear cubic on 2:3
paths. The oracle integrates analytic Bezier derivatives in Decimal precision,
using bounded adaptive Gauss 12/24 quadrature. A separate adaptive Simpson
implementation cross-checks every full cubic length, and collinear totals are
also checked analytically. Inversion bisects oriented authored length. This is
independent numerical evidence for these original curves, not a universal
interval-proof claim. `expectations.json` retains high-precision lengths, owner
indices, cell fractions, parameters and endpoint positions.

`evaluate.py` consumes only the agreed public receipt in `interface.json`.
It derives owners from curve identity and portion interiors; it does not require
producer-supplied fractions, sample grids or owner labels. Additional portion
subdivision is legal. Omitted one-sided endpoints, reordered spans, reversed
direction, nonlinear-map mistakes, incorrect length totals, stale revisions and
f64 changes fail. Mesh constraints must use the actual group ID. Mesh boundary
metadata and weights are checked independently against authored arc fractions.
The maximum authored-to-weighted-rest chord deviation is reported separately;
it is not an authoring tolerance or a physics acceptance result.

Each coarse/fine pair changes only resolution and revision. Group fields and
authored geometry must be unchanged; independent expected metrics and pair
locations remain the same within the metric budget. Refinement may change
stitch count and rest chord approximation. A finite mesh, a source test or a
small simulation cannot establish the sewing contract or reference parity.

Legacy v1/v2 inputs are original and intentionally carry long f64 values,
signed zero, nontrivial revision/allocator, partial/reversed intervals and a
nonlinear normalized parameter map. The native v3 source must preserve every
legacy field/bit/order, except the explicitly required v1 stiffness/grain
migration and empty new group vector. Actual legacy mesh pairing is separately
checked in parameter space. Serialization alone does not establish semantics.

The malformed input set requires rejection for missing v3 groups, duplicate
JSON fields, unknown group fields, unsupported version, empty or oversized
sides, unknown curves, zero spans, within-side overlap, cross-side overlap,
nonmonotone maps, duplicate IDs and the 20,000-reference limit. The total limit
witness uses 20 valid-sized groups, each with 1024 touching, nonoverlapping A
intervals and one B interval: 20,500 references, below the object-count bound.
Failed load/save/commands must preserve the old project, revision, history/Redo
and existing destination bytes; process error alone cannot prove preservation.

Still prospective until actual state/operation captures exist:

1. Add a group in one history step, edit the complete group in one step,
   reverse B by reversing list order and each interval's endpoints, then reverse
   again. Both reversals must restore every group field exactly; map unchanged.
2. Split the reversed A interval on curve 16 at t=0.6. The right child must
   precede the left: its interval is 0.75 -> 0, then the left interval 1 -> 0.5.
   IDs come from the actual public command result; validate recovered geometry
   and globally unique ownership independently. Group ID/name/map survive.
   Splitting at an exact selected endpoint creates no zero interval.
3. Join those children using the recovered parameter transform. Retain two
   entries on the recovered parent: 0.9 -> 0.6, then 0.6 -> 0.3. Do not silently
   merge the authored boundary. Continuous pair locations remain within 1e-8 m.
4. Remove any participating piece. Remove the entire dependent group, retain an
   unrelated group exactly, and restore both piece and group with one Undo.
   Different groups may share intervals; touching or disjoint coverage of one
   curve in the same group is legal. Positive overlap remains illegal.
5. Reject invalid edits while Redo exists, preserving construction, revision,
   Undo/Redo depths and recoverable files. Force cancelled, stale and exhausted
   metric/mesh jobs through a bounded public job/test configuration, requiring
   a useful error and no partial result, source mutation or approximate fallback.

No production API for these prospective operations was invented. Their
invariants are frozen here before later operation/output inspection. A later
adapter/evaluator supplement must preserve this file and its hash and state its
own freeze time and inputs.

# Standalone native scenario, declared before UI API availability

Use one standalone native process bound to the candidate commit and executable
hash. Record launch/process handles, actual UI automation events, visible states
and native saves; do not substitute an in-process UI unit test or generic author
receipt. Load the original no-group v3 fixture. Activate ordered segment sewing,
select the two A intervals in authored order, press Enter, select the three B
intervals in authored order, then press Enter. The draft must stay outside the
document; the completed gesture must create exactly one group/Undo step.

Before committing, independently exercise Delete-last, Escape, Enter on an
empty side, tool switching and document load/reset. Each cancellation leaves
construction and history unchanged. After creation, inspect visible identity,
directions, both disconnected guides, side lengths/mismatch and explicit arc
metric; inspect the precise partial/cubic/nonlinear editing controls using the
original cubic input. Edit the whole group, reverse twice, and exercise one-step
Undo/Redo. Save natively, close and reopen in the standalone process; group
fields, all span endpoints, ID/name/map/order and geometry must be identical.
Invalid save/load must preserve the previous file and active project.

This native case is UNKNOWN until the UI exists and the actual actions and saved
states are inspected. No screenshot, producer statement or headless receipt can
mark it passed. Reference behavior remains unknown beyond the reviewed public
contract; no physical gathering or accepted roadmap gate is claimed.
