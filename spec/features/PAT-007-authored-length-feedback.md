# PAT-007 / SEW-004 — Authored boundary and sewing length feedback

Status: root approved API direction and tolerances before outputs; no
implementation, build or test run. Inspected the
UI worktree at `330a5b4` / `e121cde`; the reported later contrast-only `7ab2107`
does not change the length interfaces below. Root reports the final UI visual
accepted; this is the handoff contract for a fresh implementer.

## Scope and evidence boundary

Deliver selected authored/rest boundary length and raw legacy seam / ordered
sewing-group side lengths with signed **B − A** mismatch. Values describe the
editable pattern in metres, independent of resolution, placement, display
sampling, current cloth positions and simulation time. Native presentation is
millimetres; machine output remains unrounded SI f64.

This is an engineering increment under `PAT-007` and `SEW-004`, not reference
parity. The supplied observations establish only presence of Display > 2D
Information > Show Line Length/Angle (`MDUI-D9617DB38195`) and Sewing > Check
Sewing Length (`MDUI-A55CEF572E5E`). Numerical, rounding, selection and angle
semantics remain unobserved. No warning threshold, percentage mismatch, angle,
perimeter, internal feature or measurement persistence is added.

## Existing source and complete caller trace

- `crates/weftforge-core/src/sewing.rs:608`: `Project::interval_length` already
  uses the bounded authored `IntervalMetric`. `SEWING_METRIC_TOLERANCE` is
  `1e-8 m`; the internal integration budget is that divided by 256. Retain its
  finite/positive length, work/depth and interval validation; add no integrator.
  Its only existing direct caller is `tests/sewing_groups.rs:262`.
- `crates/weftforge-core/src/geometry.rs:292`: `Project::seam_lengths` currently
  sums 128 chords. Its **only production caller** is CLI `main.rs:45`, the
  `inspect` branch. No direct tests call it. Other repository mentions are source
  landmarks, not runtime calls. Replace the sampling loop with
  `Ok([self.interval_length(seam.a)?, self.interval_length(seam.b)?])` after the
  existing unknown-seam lookup. The public signature stays unchanged; curved
  CLI `inspect` values deliberately become more accurate.
- Legacy pairing remains in `geometry.rs:648` `append_stitches` and `:692`
  `correspondence`; solver initialization consumes `Mesh::stitches` in
  `simulation.rs`, not `seam_lengths`. Do not change those paths or legacy
  curve-t correspondence. Existing seam-reversal tests exercise that distinction.
- `sewing.rs:625` `Project::sewing_group_geometry(id)` already returns raw
  `lengths: [f64; 2]`. Reuse it for a stateless group query; it also derives
  portions, which is acceptable once per explicit query. Production mesh build
  uses `append_sewing_groups`, not this public wrapper. Existing direct callers:
  geometry unit test `geometry.rs:873`; integration tests `sewing_groups.rs`
  at 75, 213, 338, 377–378, 404, 416, 471, 799, 856, 902; `ellipse.rs:287`.
- Native `Studio::inspector` (`main.rs:1700`) has `selected` piece and
  `selected_curve` IDs, no selected seam. Legacy rows have only direction/edit
  controls; group rows already show lengths and B − A using
  `mesh.sewing_groups` only when `mesh.doc_revision == project.revision`.
- Native mesh jobs (`start_mesh`, `poll`) retain the old display snapshot but
  reject cancelled/obsolete results. `rebuild`, successful load and history
  restoration invalidate derived work. `InspectorDraft`, numeric edit text,
  group-edit drafts and point-drag `preview` are distinct from applied geometry.
  Undo preserves IDs but advances the revision; an ID-only metric cache is wrong.

## Smallest API and adapter decision

Keep the current core APIs. Add one **adapter-private** `LengthQuery` in CLI
`main.rs`, deriving Serialize/Deserialize with the existing closed `type` / `data`
serde convention. CLI and MCP call one `length_report(&Project, LengthQuery)`
function in that file. Native calls the same core metric APIs directly; it does
not depend on the CLI. No new core report hierarchy or authoring command.

Exactly three requests:

```json
{"type":"interval","data":{"curve":7,"t0":0.2,"t1":0.8}}
{"type":"seam","data":{"id":23}}
{"type":"sewing_group","data":{"id":42}}
```

Full selected curves use an interval `[0,1]`; a separate curve variant is
unnecessary. A reversed interval keeps its supplied endpoints in the echo and
returns the same positive length. Seam/group side order is always authored A/B.

CLI forms, with exact arity and the existing finite-number parser:

```text
lengths PROJECT interval CURVE_ID T0 T1
lengths PROJECT seam SEAM_ID
lengths PROJECT sewing-group GROUP_ID
```

MCP tool: `project_lengths`, arguments `{"path":"original.weftforge","query":…}`.
It loads with `Project::load`, performs the query, and uses the existing result
envelope. Advertise `readOnlyHint:true`, `destructiveHint:false`,
`openWorldHint:false`. Input schema is a closed object with required path/query;
query uses closed `oneOf` type/data variants, positive integer IDs and finite
JSON numbers in `[0,1]`. Runtime core validation is authoritative.

Exact success payload shapes (numbers below are schematic):

```json
{"source_revision":17,"length_unit":"m","query":{"type":"interval","data":{"curve":7,"t0":0.2,"t1":0.8}},"length_m":0.9}
{"source_revision":17,"length_unit":"m","query":{"type":"seam","data":{"id":23}},"lengths_m":[0.9,1.1],"mismatch_b_minus_a_m":0.2}
```

The group payload has the same pair fields and echoes `sewing_group`. Pair
mismatch is computed directly as `lengths_m[1] - lengths_m[0]`; no absolute value,
ratio, cutoff or classification. CLI JSON, MCP `structuredContent` and MCP text
JSON must be equal for the same saved revision/request. No rounding in JSON.

Unknown IDs, nonfinite/out-of-range/equal endpoints, or
`abs(t1-t0) <= 1e-8` return an explicit error under existing metric rules. Retain
all precision/work errors; no chord fallback, null, zero or previous value
masquerading as a new success. Reject unknown/duplicate/missing/null query
fields using original request bytes, preserving MCP's current parsing and line
limits. No `Editor`, save, recovery write, rest-mesh build, history change or
persistent cache is created by either query path. Existing `inspect` should
propagate a metric failure instead of hiding it through its present `.ok()`.

## Native presentation and cache decision

1. Add a plain read-only `Applied rest length · revision R` label for the
   selected full boundary curve. Arbitrary oriented partial intervals are
   exposed by the explicit query and by authored seam/group sides; no new native
   interval-editing controls are needed for this increment. This does not deliver
   a full native measurement tool.
2. Legacy seam rows gain A/B raw lengths and signed B − A; group rows retain
   existing revision-gated geometry and use the same mm presentation. Display
   `1000 * metres` with three decimal places and explicit `mm`; mismatch includes
   a sign. This is a chosen engineering formatter, not observed reference
   rounding. Display rounding never becomes a mismatch threshold.
3. Prefer storing the committed full-curve metric (including an error) in the
   existing `InspectorDraft` when it is constructed from `Project`. The
   inspector already retains it by revision/piece/curve and `rebuild` clears it;
   successful load/recover calls `rebuild` even for matching revision/IDs. Keep
   this derived member out of `commands` comparisons. Its baseline construction
   may also happen on explicit commit/actions, but idle repaint must not repeat
   integration. Use only a small local transient revision snapshot for legacy
   seam results, including errors; compute each visible legacy pair at most once per
   applied revision. Populate lazily; do not integrate every seam in a large
   document before its rows are visible. Reuse current `Mesh::sewing_groups` for groups. Do not call
   `sewing_group_geometry` or repeat arc integrals during idle repaint.
   Invalidate on `rebuild` and every successful new/open/recover replacement,
   including a new document with the same IDs/revision. Keep this local, without
   a generic cache framework, persistent fields, new worker type or changes to
   `Mesh`.
4. Numeric text/drag values and point-drag previews do not feed these applied
   metrics. Labels always identify their applied source revision and disclose
   `Draft/preview edits are not included` while applicable (or as a permanent
   concise qualifier). Group edit/create drafts likewise do not replace
   committed group values. A mesh for another revision shows rebuilding, never
   current-looking old numbers. Errors show unavailable with the reason.
5. Feedback must not commit or cancel drafts, move the caret, steal focus, call
   `execute`, change selection, dirty the project, start/cancel jobs or invalidate
   solver state. Put dynamic feedback in its own existing-style egui ID scope so
   changes between pending/success/error do not reassign numeric edit IDs.

## Acceptance frozen before execution

Absolute tolerances chosen now: each length **1e-8 m**, independently checked
B − A **2e-8 m** (two side errors). Query parity between adapters is exact parsed
JSON equality; mismatch arithmetic is exactly B − A of the returned f64 values.
Native formatted output is exact expected three-decimal mm text; formatter
rounding is separate from the numerical tolerance. No tolerance increase after
results. These are implementation acceptance criteria only.

- **Straight:** original valid closed CCW polygon with a 0.3/0.4 m edge gives
  0.5 m. Full/partial/reversed intervals agree with the analytic endpoint metric.
- **Asymmetric analytic cubic:** create a CCW original polygon
  `[(0,0),(1,1),(0,1)]`, then set its first curve controls to
  `[(1/3,0),(2/3,1/3)]`. Its first boundary is `(t,t²)` and the other two lines
  close above it; the fixture is valid and CCW. Independent primitive
  `F(t)=t*sqrt(1+4*t²)/2 + asinh(2*t)/4` gives full length
  `sqrt(5)/2 + asinh(2)/4`. Check `[.2,.8]`, `[.8,.2]`, and additivity at `.37`
  against F, without calling the production metric to establish expected values.
- **Existing asymmetric cubic:** extend `tests/sewing_groups.rs` beside its
  existing independent 2048-step composite Simpson oracle. Exercise accurate
  legacy side lengths, one reversed partial side and an unequal pair/group.
  Keep original interval/list order; gaps add no length; correspondence does not
  alter raw side lengths. Do not invent another numerical integrator.
- **Resolution/placement/simulation isolation:** coarse/fine resolution and 3D
  placement changes give the same authored metric. Queries do not change project
  bytes/IDs/revision/history/Redo, existing mesh/cloth snapshots or job handles.
  Existing legacy reversal/stitch correspondence tests still pass unchanged.
- **Retention and invalid input:** accept valid reversed intervals. Reject
  unknown curve/seam/group IDs, equal or tiny-invalid endpoints, out-of-range
  values, NaN/Inf (CLI/core), malformed JSON,
  unknown/duplicate fields and invalid project input. Verify retained input,
  recovery and unrelated destination bytes; a successful retry still works.
  A failed authoring interval/group edit retains document, revision, draft and
  Redo under existing command rules. Query failures never rewrite files.
- **Revision correctness:** measure, edit controls/points with the same IDs,
  measure changed length at a new revision, then Undo and Redo. Check restored
  numbers at monotonically newer revisions. Replace the document with another
  having the same IDs/revision and different geometry; no prior cached result.
- **Native:** inspect selected curve, legacy pair and group; check mm conversion
  and sign. Leave valid, incomplete and invalid numeric text focused, run idle
  frames, drag a point without release, open/cancel a group draft, and verify
  applied labels/draft disclosure and unchanged text/caret/history. Commit one
  gesture, then Undo/Redo. Test pending, cancelled, failed and obsolete mesh
  states; only a matching revision supplies group numbers. Prove repeated idle
  frames reuse metric results rather than repeatedly invoking integration.
- **Adapters:** for each request variant and both MCP protocol profiles, compare
  CLI output, MCP structured/text payload, and the core metric at the same saved
  revision. Confirm the new tool's strict schema and annotations. Update the two
  explicit `tools.len()==8` assertions to nine; authoring command variants and
  protocol behavior remain unchanged.

## Exact implementation reach and later delivery

- `crates/weftforge-core/src/geometry.rs`: shared legacy metric correction.
- `crates/weftforge-core/tests/sewing_groups.rs`: analytic/independent oracle,
  interval errors, pair lengths and read-only/revision regressions. Existing
  `tests/seam_reversal.rs` remains the pairing regression; extend only if needed.
- `apps/weftforge/src/main.rs`: native feedback/cache/draft qualifier and focused
  egui tests beside the existing inspector/curve/group/job tests.
- `apps/weftforge-cli/src/main.rs`: typed query, shared report, CLI parser/help,
  and explicit errors in existing inspect. `src/mcp.rs`: closed args/schema,
  allowlist/call branch and tool discovery, reusing that report.
- `apps/weftforge-cli/tests/mcp_stdio.rs`: adapter equality, lifecycle/schema,
  preservation and tool-count checks (currently at lines 306 and 953). Extend
  existing CLI test infrastructure; no dependency or fixture framework needed.
- `docs/automation.md`: CLI/query examples, SI output, new ninth tool and limits.
  A compact engineering contract/scenario, `STATUS.md`, and `parity/features.yaml`
  must record partial scope, current candidate, exact checks, unknown reference
  behavior and stale prior evidence. Update source-map relationships only with
  an explicit new pin/review; never silently rewrite the old source-pinned claim.
  Generate/check registry views through existing tools if registry changes need
  them. README needs no laboratory details or unrelated rewrite.

Implement the bounded change, run fmt/test/clippy/xtask
locally in an isolated Cargo target, then identify the candidate for independent
native acceptance. Before publishing any evidence, track every artifact and run
registry/map checks from a clean checkout. Root owns reviewed publication. Keep Actions and automated Pages deployments
disabled until the complete project is done. This contract grants no reference-parity status.

Skipped: new measurement framework, angles, mismatch thresholds, numerical
fallbacks, persistent measurements and proprietary/reference investigation.
Risk: more accurate legacy reports change curved `inspect.lengths_m` values;
rare metric limits can now produce an honest error where the old chord estimate
returned a number. Runtime and native acceptance remain unexecuted.
