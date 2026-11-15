# Independent authored-length acceptance — frozen before candidate execution

Prepared only from root `AGENTS.md`, Ponytail, the reviewed contract, public
native-v3 types/validation, shared command declarations, and documented CLI/MCP
request/envelope formats. No production length metric, implementation oracle,
candidate result, reference application, Cargo build, or UI was consulted.

Reviewed contract SHA-256:
`4f79150ed28854b8a16760af07b1558808dcca2080bc1ee8e114d5b2682f851c`.

## Scope and fixed criteria

This checks CLI `lengths` and MCP `project_lengths` against original analytic
geometry. It establishes implementation acceptance only. Reference observations
establish control presence; numerical and selection parity remain unknown.

`criteria.json` freezes requests and expected payloads. Each length has an
absolute tolerance of **1e-8 m**; independently expected signed B − A has an
absolute tolerance of **2e-8 m**. The reported mismatch must also equal the exact
Python/f64 subtraction of the returned B and A numbers. Units, revision, query
echo, payload keys, and CLI/MCP structured/text parsed JSON equality are exact.
Every numerical result must be finite. Tolerances cannot change after results.

Scope reaches this plan, one original fixture, the frozen criteria, and a Python
standard-library grader/runner. It changes no product source, caller, test,
fixture, configuration, export, workflow, or site. Product files are protected
from writes by the query checks. Candidate execution is deferred until root
reviews these artifacts and supplies a binary and source identity.

## Original fixture and analytic oracle

`fixture.weftforge` is native-v3 JSON with revision **137**, next ID **33**,
material **1**, parabola piece **11**, straight piece **21**, seam **31**, and
group **32**. All object IDs are nonzero and globally unique. There are no
external assets or mannequin capsules. The placements are deliberately nonzero.

Parabola points 12/13/14 are `(0,0),(1,1),(0,1)`, with closed boundary 15/16/17.
Curve 15's absolute cubic controls are `(1/3,0),(2/3,1/3)`, so its analytic form
is `P(t)=(t,t²)`. Its boundary encloses positive area `2/3 m²` and is CCW.
The independent primitive is
`F(t)=t*sqrt(1+4*t²)/2+asinh(2*t)/4`; interval length is `abs(F(t1)-F(t0))`.
Full, partial `.2→.8`, reversed `.8→.2`, and additive spans split at `.37`
are frozen. No numerical integrator or production metric supplies expectations.

Straight points 22/23/24 are `(2,0),(2.3,.4),(2,.4)`, with closed boundary
25/26/27 and positive area `.06 m²`. Curve 25 is a 3-4-5 edge of `.5 m`;
curves 26 and 27 are `.3 m` and `.4 m`. Its full, partial, and reversed cases
use `.5*abs(t1-t0)`.

Seam 31 connects reversed curve-15 `.8→.2` to curve-25 `.1→.9` on the other
piece. Its nonidentity map contains `(.23,.71)`. A is longer than B, so its
mismatch must be negative.

Group 32 keeps the authored order: A has curve-15 `.94→.66`, then `.05→.37`;
B has full curve 25, then full curve 27 on the other piece. Its nonidentity map
contains `(.29,.63)`. The omitted spans and spatial gaps add no length, and map
weights do not alter raw sums. B is longer than A, so this mismatch is positive.

## Decisive execution checks

1. Run the ten frozen success queries through CLI and both documented MCP
   profiles (current `2026-07-28`, legacy `2025-11-25`). Grade every payload
   independently and compare exact parsed CLI/structured/text JSON. The two
   analytic partial spans cover additivity at `.37`; reversal equality is exact.
2. Discover `project_lengths`; check its read-only/non-destructive/closed-world
   annotations, closed root/variant/data schema, all three variants, positive
   integer IDs, and endpoint number bounds `[0,1]`.
3. Reject unknown curve/seam/group, equal/tiny intervals, both out-of-range
   directions, CLI NaN/Inf and arity errors. MCP additionally rejects original
   raw requests containing unknown, duplicate, missing, or null query/data
   fields, an unknown variant, and malformed/nonfinite JSON. No parse through
   a generic JSON value erases input duplicates before they reach the server.
4. Put malformed primary input beside a valid recovery, then query it: failure
   must be explicit and must not silently read recovery. After every MCP error
   sequence, a valid retry in the same process must return the frozen result.
5. Before and after every CLI process and MCP batch, compare all staged file
   bytes and the directory file set: primary, valid recovery, malformed input,
   its valid recovery, and unrelated destination/recovery sentinels. Successes
   and failures must preserve everything and create no sidecars or caches.
6. Derive one temporary isolation file from the original (no second frozen
   fixture): keep IDs/revision/geometry, change resolution to `.006`, then `.49`,
   and change both 3D placements. CLI checks all ten cases for both variants;
   both MCP profiles check all ten for the last variant. All expected payloads
   stay identical. The runner itself creates these temporary variants before
   taking each preservation snapshot.

The run captures stdout/stderr, requests, responses, process return codes, file
hashes, candidate binary SHA-256, and root-supplied source identity. A fresh run
directory is mandatory. Failures stop acceptance and remain visible in its JSON
report. No tolerance widening, production-derived expected values, or automatic
fixture repair is permitted. If fixture validation fails, report it for root
review; do not adapt it to make a candidate pass.

## Frozen artifacts and execution

`SHA256SUMS` freezes this plan, `fixture.weftforge`, `criteria.json`, and
`grade_lengths.py`. The grader verifies those bytes before self-check or run.

```text
python3 grade_lengths.py self-check
python3 grade_lengths.py run --candidate ROOT_SUPPLIED_BINARY --source ROOT_SUPPLIED_COMMIT --run-id NAME
```

Only the first command has been authorized/executed during preparation. Its
self-check accepts frozen payloads and rejects known bad length, sign, units,
revision, nonfinite value, and adapter divergence. It independently rechecks
analytic frozen values and the fixture IDs/CCW construction.

These are local review artifacts, not published delivery evidence. Root owns
tracking, clean-checkout registry/map verification, and publication. The runner
does not invoke Cargo, inspect source, control UI/reference software, open a PR,
push, enable Actions, or alter static Pages.

Skipped: native display/focus/cache/history/Undo/Redo, in-memory mesh/cloth/job
state, authoring edits, `inspect` integration, and direct core tests; root's
separate integrated/native acceptance must cover them. Risk: black-box file
checks cannot prove absence of transient computation or establish reference
parity; a binary/source association is recorded from root, not inferred.
