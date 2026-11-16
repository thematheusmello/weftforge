# Physical-length Free Sewing contract

Reviewed engineering contract approved by the implementation owner before new
candidate outputs, 2026-11-15. Read against source
`699c4608ff68af132ec90b1ffb80ba354c70435f`. Implementation follows the transform
increment. No new candidate or reference runtime was used to define this contract.
Independent acceptance inputs and numerical oracles must be frozen and reviewed
before candidate execution. This contract is a requirement, not delivery evidence.

## Evidence boundary

Reviewed public inputs: `spec/features/SEW-002-paired-runtime-20261114.md`,
`SEW-002-reverse-runtime-20261114.md`, `SEW-002-runtime-20261114.md`, their three
`spec/observations/REF-SEW-002-*.json` records, and the existing ordered-path
contract. No private container or raw reference artifact is an implementation input.

On the original 200 × 300 mm two-panel setup, Free Sewing selected the curved
boundary's left endpoint, then Create Sewing Line → Length `100` mm → Tab → OK.
An interior start on the other straight boundary and a leftward second `100` mm
acceptance completed one pair with two `100.0` labels and endpoint guides. One
Undo/Redo and same-process document reopen restored it. The first side alone was
transient. Initial `8.0`/`100.0` dialog values were pointer-state observations,
not factory defaults. Exact curve controls, second-start coordinate/parameter,
continuous correspondence, ranges and cancellation were not measured. Explicit
Reverse Sewing later exchanged the endpoint guide connections without moving
the intervals or changing their labels; this is categorical endpoint evidence.
Unequal lengths/easing, M:N execution, internal sewing, remeshing and simulation
remain unqualified. This increment neither claims parity nor reduces full SEW-002.

## Existing path and smallest shared API

- `crates/weftforge-core/src/sewing.rs:608/612`: public `interval_length` and
  oriented `interval_parameter_at_length` already own bounded line/cubic metrics
  and inversion. Reuse them, their `1e-8 m` budget, precision/work errors and exact
  endpoint cases; add no integrator or sampled metric fallback.
- `document.rs:236/261`: legacy `Seam` correspondence is normalized **curve t**;
  `SewingGroup` correspondence is normalized cumulative **physical arc length**.
  Use one existing group with one interval per side and `[[0,0],[1,1]]`, never a
  legacy seam disguised as physical arc matching. Existing groups, edits,
  weighted stitches, split/join, persistence and solver derivation remain reused.
- `document.rs:1086/1116/1564`: `AddSeam`, `AddSewingGroup` and clone/validate/commit
  history already separate these semantics and retain old state on failure.

Approved interfaces:

```rust
enum SewingDirection { Forward, Reverse } // increasing/decreasing authored t
struct SewingSideByLength {
    curve: Id,
    start_t: f64,
    direction: SewingDirection,
    length_m: f64,
}
Project::interval_from_length(&self, side: SewingSideByLength) -> Result<CurveInterval>
Command::AddSewingByLength { name: String, a: SewingSideByLength, b: SewingSideByLength }
```

Types use the existing derives, snake_case serde and strict unknown-field rules;
the enum serializes as `"forward"`/`"reverse"`. The command JSON type is
`"add_sewing_by_length"`. Require an existing curve, finite `start_t ∈ [0,1]`,
and finite `length_m > 0`; direction is a closed enum, never a numeric sign.
Construct the remaining oriented interval from `start_t` toward exact `1` or
`0`, call `interval_parameter_at_length`, and return `{curve,t0:start_t,t1:end}`.
Apply the existing distinct-parameter (`abs(t1-t0) > 1e-8`), positive resolvable
length, precision and group coverage checks. A start facing out of the curve,
insufficient remaining distance, missing curve or metric failure is an error.
There is no boundary traversal, extrapolation, endpoint wrapping, length clamp
or rounding of stored t. Requested distance must fit the computed remaining
metric; equality resolves to its exact endpoint. Do not silently tolerate an
overflow because it is smaller than a canvas digit.

The new command resolves both sides against the same candidate project before
allocating an ID, then delegates to existing `AddSewingGroup`. Existing group
validation still rejects overlapping coverage, while disjoint same-curve spans
remain permitted. Different requested lengths inherit the group's existing
normalized proportional semantics; they are not reference-qualified easing.
The result stores only the ordinary group/t intervals: entered lengths choose
creation endpoints, not persistent dimension locks. Later geometry edits retain
t attachments and recompute lengths under the existing group rule. No native
format/version bump, migration, dependency, extra MCP tool or CLI subcommand.

## Native gesture and engineering picking policy

Add accessible **Free sewing** to the existing tool rail and command search;
retain **Sew edges** and **M:N sewing**. One transient, revision-stamped A/B draft
owns start/direction/length text, resolved preview and error. First-side OK only
advances to B. Second-side OK sends the single new command through the existing
inspector action path; valid staged inspector edits and creation form one
`Transaction`. Resolve against that candidate construction, not stale preview
geometry. Success clears the draft and creates one history entry. Validation
failure retains both sides, raw text/caret, staged inspector edits, accepted
document, revision, allocator, Undo/Redo and accepted mesh/solver state.

Canvas start → pointer direction/right-click → **Create Sewing Line**, with a
labelled **Length** text input and **mm** unit, Tab validation, OK and Cancel.
Provide keyboard-accessible boundary, exact Start t and Direction controls too;
pointer picking must not be the only route. Convert mm to f64 metres once at the
boundary, reject nonfinite/invalid/overflowing input, and keep unfinished text
verbatim. Seed the dialog from the current pointer interval when resolvable;
never hard-code the observed `8.0` or `100.0` as a claimed reference default.
Show A/B direction, pending/applied distinction, both derived length labels
(`100.0 mm` in the equal case) and start-to-start/end-to-end guides. Reuse the
matching-revision authored-length/group derivation; cache transient calculations
per input change, never integrate every paint frame or install stale results.

The following is explicit **WeftForge engineering behavior**, not inferred
reference snap/projection/rounding semantics. Reuse the existing 12-pixel hit
radius and 24-chord cubic boundary samples. Within the hit radius, prefer a
boundary endpoint and store exact `t=0`/`1`. Resolve coincident candidates by
selected boundary first, then existing piece/boundary order, then nearest
endpoint (start before end on an exact tie); expose the chosen boundary/t for
correction. For an interior hit, project the unsnapped f64 authoring pointer onto
each sampled chord, choose the nearest, and use `t=(i+u)/24`, with clamped chord
projection `u`. Straight boundaries use exact line projection. Do not grid-snap
or decimal-round the resulting t. A right-click on the chosen boundary selects
Forward/Reverse from its projected t relative to the start; an equal-t click
must ask for explicit direction. This is a display-chord approximation to a
cubic closest point; exact Start t is the precision route. Keep existing Select,
legacy sewing and M:N picking behavior unchanged.

Cancel, close and modal Escape abandon the whole transient Free Sewing gesture;
they preserve unrelated inspector drafts and their raw text/caret. Tool switch,
successful document replacement, history and explicit gesture reset discard it
without a seam; failed load retains the old project and draft. A revision change
invalidates the transient draft with a visible status. Modal shielding must cover
background pointer, keyboard and AccessKit actions using the current modal event
gate. First-side acceptance/cancellation must not dirty/autosave a half-pair,
clear Redo, allocate IDs or persist a draft. Cancellation semantics here are
independent requirements; reference cancellation remains unknown.

## Acceptance frozen before candidate outputs

Use a new original analytic fixture, not an approximation of the unknown reference
controls: curved boundary `C(t)=(0.2t,0.1t²) m`, controls
`(0,0),(0.2/3,0),(0.4/3,0.1/3),(0.2,0.1)`, closed by straight edges through
`(0.2,0.3),(0,0.3)`. Its bounds are 200 × 300 mm. The second original 200 × 300
mm panel has a straight boundary `B(t)=(0.4+0.2t,0) m`.

Its independent primitive is
`F(t)=0.1[t√(1+t²)+asinh(t)]`; full curved length is
`0.22955871493926383 m`. A `{start_t:0,forward,length_m:0.1}` ends at
`t≈0.4819445564563284`, point `(0.09638891129126569,0.02322705554978871) m`.
B `{start_t:0.75,reverse,length_m:0.1}` has analytic endpoint `t=0.25`. Freeze a separate
primitive-based bisection and independent derivative quadrature/inversion with
tighter error than production acceptance; neither imports production metrics nor
uses candidate output to generate expected values. Reuse the existing independent
Simpson oracle pattern in `tests/sewing_groups.rs`, including an asymmetric and
nonuniform collinear cubic counterexample to linear-t matching.

Before running candidates, freeze these checks and fail thresholds:

1. Both integral lengths and independently evaluated endpoint locations agree
   within `1e-8 m`. Check correspondence at fractions `0,1/4,1/2,3/4,1` using
   `F⁻¹(0.1f)` on A and `0.75-0.5f` on B; matching physical midpoints must not use
   half the A endpoint t. Check coarse/fine rest meshes do not redefine the answer.
2. Reverse/interior starts: A `start_t=0.8,reverse,0.1 m` ends at
   `t≈0.3702881984991444`; B `start_t=0.1,forward,0.1 m` ends at `0.6` within
   floating arithmetic tolerance. Exact start clicks and full-remaining-length
   requests retain exact endpoint t. Include both directions and no remaining room.
3. Reject invalid/NaN/infinite/out-of-range start, direction strings, zero/negative/
   enormous length, missing curve, collapsed parameter span, overlong interval,
   overlap, invalid name, exhausted metric precision/work, ID/revision overflow.
   Failure preserves document bits, allocator, history/Redo and existing output/
   recovery bytes. Exercise same-file CLI/MCP failure and existing atomic-writer
   failure paths; no disk-full behavior is inferred from the reference warning.
4. First-side OK leaves construction/history untouched; second OK adds exactly
   one group and one revision. One Undo removes both sides/guides; one Redo
   restores exact group fields/ID. Native save/new/reopen preserves the pair.
   Identical typed command via native, CLI `execute` and MCP `project_execute`
   produces equal construction and CLI/MCP output bytes; inspect lengths through
   existing shared read APIs. Preserve legacy nonlinear t-map semantics exactly.
5. Native pointer plus accessible controls reach curved/straight 100 mm creation,
   Tab/OK and two labels/guides. Test endpoint ties and interior projection against
   the declared pick policy, invalid text, cancelled A/B, Escape/close/tool switch,
   reset/history/load/stale revision, modal isolation, and inspector/draft
   text/caret retention on failure/cancel across press, release and later frames.
   Run the real native workflow separately from in-process AccessKit tests.

## Exact implementation reach after approval

| File | Callers/changes to review |
| --- | --- |
| `crates/weftforge-core/src/document.rs` | New strict side/direction types and command; `Project::apply` lowers to `AddSewingGroup`; `Editor::execute` remains the atomic owner. |
| `crates/weftforge-core/src/sewing.rs` | Thin `interval_from_length` resolver calling existing inversion; no duplicate numerical machinery. |
| `crates/weftforge-core/src/lib.rs` | Existing wildcard re-exports already expose the new document types/helper; verify no extra export is needed. |
| `crates/weftforge-core/tests/sewing_groups.rs` | Analytic/independent metric, reverse/interior, legacy, failure/history/persistence checks. Keep old fixtures unchanged. |
| `apps/weftforge/src/main.rs` | `Tool`/label, `Studio` draft/reset, `patterns` picking/drawing, toolbar/rail/search, side dialog, `apply_inspector_action`, modal gates, numeric focus/caret helpers; new in-process interaction checks beside existing sewing/transform tests. |
| `apps/weftforge-cli/src/mcp.rs` | Closed `command_schema`: strict side fields, t range, positive length, enum direction, new command variant. Existing nine-tool surface stays. |
| `apps/weftforge-cli/src/main.rs` | Existing strict deserialization and `execute_commands` transaction/save are reused; inspect/length dispatch needs no new variant. Review, do not add a redundant route. |
| `apps/weftforge-cli/tests/mcp_stdio.rs` | New schema/negative cases, serialized command count (25 → 26 nontransaction variants), native-field/CLI/MCP equivalence and failed destination retention. |
| `docs/automation.md` | New shared command/payload, units, singleton physical group semantics and errors. Native v3 file schema remains unchanged. |
| `spec/features/SEW-002-physical-length.md` | Publish the reviewed bounded contract after approval; retain the existing broad/observed contracts and unknowns. |
| `STATUS.md`, `parity/features.yaml`, `spec/inventory/weftforge-implementation-map.json`, `.md`, `verify_implementation_map.py` | Record actual source/evidence and remaining gaps. Follow the map's pinned-source/history workflow; its historical 22-command snapshot must not be relabelled as current source. |
| `parity/reports/artifacts/physical-sewing-20261115/independent/fixture.weftforge`, `CRITERIA.json`, `evaluate_physical_sewing.py` | Proposed frozen original input, thresholds and independent evaluator before outputs; track every publishable artifact/binding. No proprietary material, public personal path, or ignored-file delivery. |

After implementation, run all four required local gates (`fmt --all --check`,
workspace tests, workspace/all-target clippy with `-D warnings`, `xtask verify`),
then actual native acceptance. Before publication, run registry/map checks from
a clean tracked checkout with its own Cargo target. Keep Actions and automated
Pages deployment disabled and preserve static Pages. Source checks and this
engineering fixture cannot establish reference metric/pick/cancellation parity,
physical simulation acceptance or the full roadmap sewing gate.
