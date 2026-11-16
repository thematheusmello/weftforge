What this change does: Replaces an eligible piece's exterior with ordinary editable material geometry, using one shared offset command. It prepares a new rest mesh and a real Cloth before committing, and preserves source identities where their meaning survives.

This is an independent, read-only review of the released source identified in `SOURCE-PINS-001.json`, against the reviewed contract and original recipes. Expected load is one offline native/CLI/MCP authoring request, within the existing document and mesh ceilings. No remaining numerical or atomic-commit blocker was identified in that source. This is not first-phase acceptance or PAT-009/reference parity credit.

**Must fix before first-phase acceptance**

1. **Triangulation does not meet the whole-request polling and work limits** (`crates/weftforge-core/src/geometry.rs:283`, `crates/weftforge-core/src/offset.rs:345`).
   - **What this is:** Actual material preparation calls the existing pinned `earcutr::earcut`, with cancellation checks immediately before and after it.
   - **Problem:** Its internal predicates neither poll the request token nor charge the request counter. A valid curved boundary with, for example, 512 tessellated vertices can remain inside this call for more than 256 internal geometry tasks after Cancel; `geometry_work` cannot establish the declared total internal-work ceiling.
   - **Fix:** Keep both criteria visibly UNMET, as the lead explicitly decided for this increment. Instrument or replace that kernel with bounded, cooperative work before claiming the full first-phase contract; the current source comment correctly records the limit.
   - **If we skip it:** Atomic old-state preservation still holds, but cancellation latency and the whole-request internal-work bound remain unproved. The present counter covers controlled Rust work only.

The lead retained this inherited kernel deliberately. Finding 1 blocks the corresponding contract claim, rather than identifying a newly found numerical or data-loss defect.

**Corrections checked during review**

- The final state of a flat command batch now gets serialization-size, actual mesh, target vertex-cap and real Cloth checks before history changes (`document.rs:2208`). This closes the concrete offset-then-finer-resolution failure; the regression uses a coarse-valid 2 m square followed by 0.005 m resolution (`tests/outline_offset.rs:550`). CLI/MCP batch support remains intact, and nested batches retain their existing refusal.
- Validation, certified sampling, mesh refinement/quality checks and the solver adapter now share the request token/counter. The ellipse validation test checks that subdivision can cancel and consumes the shared budget (`offset.rs:740`). Opaque triangulation remains the explicit exception above.
- Continuous checks now include separated parts of one source curve, adjacent gap/smooth curves and incident join arms. Trim sweeps enclose full parameter ranges over every signed width; cubic/cubic extra-root exclusion also spans `[0,d]`.
- The genuinely curved two-locus regression has an independently analytic symmetric root, including parameter brackets around 0.95 and 0.05 (`tests/outline_offset.rs:483`). It supplements the earlier cubic-encoded-straight case.

**Math and transaction assessment**

The interval operations widen under rounding, zero division and unbounded arithmetic; NaN products/additions propagate uncertainty. Squaring now explicitly uses one IEEE multiplication, and the exact-turn fallback protects the fused product residual against underflow. The fit certificate consistently uses local `[0,1]` derivatives, includes endpoint/restriction arithmetic, and applies the whole-cell second-derivative bound. Finite witnesses are supplementary, and an endpoint-only bad fit has a direct refusal test.

The cubic-pair solver uses a contraction mapping with enclosed derivatives to prove a unique root for every intermediate width, then bounds parameter motion and excludes other roots. Positive offset tangent factors support the monotone projection checks for self/adjacent spans. Final topology checks use the applied cubics' control hulls and bounded subdivision. Work/depth/output/object/mesh ceilings refuse without truncation; pending subdivision storage is bounded by depth-first traversal and the existing output caps.

Preparation is opaque to external callers and rechecks the complete source before commit. All fallible preparation and final batch checks precede authoritative/history mutation. Source curve IDs go to first surviving descendants; join arms receive fresh curve IDs, bevels retire the split source point, and existing Undo/Redo keep the ID high-water mark. The native Apply path consumes the prepared mesh/Cloth and clears stale products through `rebuild()`.

Swept axis-aligned boxes, strict transversality and finite arithmetic budgets may reject safe geometry. Those are intentional conservative refusals, not topology repair or proof by sampling. Holes, attached internal construction/sewing and the later persistent/per-edge/multiple-offset capability remain outside this increment; no current attachment is silently deleted.

Source changed during review while the owner completed these corrections. The released owner hashes matched the final read, including the last square-arithmetic and helper-argument/style refresh; the owner was notified. Later semantic changes require targeted review and refreshed pins. Test source was inspected, but reported compile/test observations were not rerun or treated as reference evidence.

Verdict: fix 1 before first-phase acceptance. No further numerical or atomicness blocker found in the pinned implementation.

Not checked: no Cargo, evaluator, grader or GUI execution; final integrated checks, native/output acceptance, performance timing and clean-checkout tracked publication remain separate gates. Opaque triangulation remains the material risk above.
