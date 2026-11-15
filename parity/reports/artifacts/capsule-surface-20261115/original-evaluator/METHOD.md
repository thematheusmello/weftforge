# Prospective independent Harbor capsule garment evaluator

Written before evaluator code on 2026-11-15. This is preparation only. It neither
executes nor authorizes garment dynamics, engine/CLI programs, UI, Cargo builds,
Git/CI mutations, parameter changes, retries or a third case. Work and new outputs
are confined to this private directory. Prior evaluators/evidence remain read-only.

Prospective resource correction received before freeze: PREPARATION-PLAN-v2.json
SHA256 d76f668d21d2837b41574b2d403f7ed6ed24f873265e56cbcbb5e666280034e6
supersedes the original plan without changing physical settings/thresholds/inputs.
It preserves the prior separate numerical-output256MiB stop, across inputs,
preflight-base, preflight-height and runs, in addition to producerRSS256MiB.
Use PRODUCER-BINDING-v2.json SHA256
68ca7dcd05f4a443fe71f76fbd0accef9c60249befec4e1638c9065e77534c9a.
Original plan/binding bytes remain immutable and bound; no garment ran before
this correction. These v2 descriptors are required by execution authorization.

## Frozen scope and trust boundary

Bind PREPARATION-PLAN.json SHA256
98d575ccfb04afad9ddf72f9ca215fea7161f58ed0de9da1bc44f80e0bebcc12 and its
SOURCE-INPUT-BINDING.json; verify all forty frozen source hashes without reading
candidate solver implementation for answers. Bind source commit
0770204a54abdf5f929b6626b01d7fcfb0e13ca4. Inputs are exact base SHA256
fc494974955d440aacbfde49c332a50218bd3a9354d06f4fbbf0596e5da8eda7 revision20 and
height SHA256 725fc896bc7f020682e2bef3a471a3e1a9048dcab50a7733afad3597722a7ec5
revision17. Before real analysis a separate root execution authorization must bind
this evaluator freeze, exact driver, preflight rest geometry and all inputs.
Preparation is never accepted as execution authorization.

Reject duplicate JSON keys, nonfinite numbers (including exponent overflow), bad
shapes/indices, unexpected keys for fixed records, symlink artifacts and paths
escaping the experiment. Reports are exclusive-created and never overwrite a
receipt. Preserve partial output and errors. Source/input/authorization/driver/
state/OBJ hashes are recorded and rechecked after analysis. Every new file stays
private; no public evidence or parity claim follows from these checks.

## Independent checks and original mathematical definitions

Use only original authoring data, actual saved rest metadata, actual saved state,
and old verified independent measurement routines. No product diagnostic is an
answer. Reuse a byte-identical private copy of the established independent
metrics.py for material axes, seam interpolation, mass reconstruction, supported
static self-contact and OBJ reconstruction. Freeze its exact provenance/hash.

Validate exact authored project/revision, SI/Y-up coordinates, known rest mesh
identity against already saved old3aa rest metadata (same authoring and mesher),
triangle/piece/material attribution, UVs, boundary samples, interpolated stitches,
masses from triangle UV area and areal density, no pins, ground0 and contacts on.
Any exact mismatch remains a failure; tolerances do not turn it into acceptance.
The exact old metadata binding will be recorded before the freeze.

For every accepted state0..180, including failure prefixes, require finite actual
positions and velocities with unchanged mesh cardinality; retain raw diagnostics.
Verify actual setting records for all steps: dt=1/60, 8substeps,32iterations,
closure speed1m/s, gravity0 at steps1..60 and [0,-9.81,0] at61..180. Initial phase
is assembly. Time uses repeated actual dt addition rather than frame/dt division.
Target recurrence uses the maximum initial interpolated stitch distance, then
max(0,target-speed*(dt/substeps)) once per substep. Retain exact recurrence values,
zero completion frame and closed target at60/180. No initial arrangement may be
called completed fit. Settings/time/target discrepancies fail and remain visible.

For each triangle form F=[x1-x0,x2-x0]*inverse([uv1-uv0,uv2-uv0]). Rotate rest axes
by the authored grain angle; signed strain is |F axis|-1. Record signed extension,
compression, maximum absolute warp/weft strain and exact triangle/piece/vertices/
axis/stiffness attribution. Compare diagnostics numerically and check attribution
independently; any diagnostic disagreement remains explicit and fails the
diagnostic agreement screen. Never replace the actual independent maximum by the
product's nominated triangle. Seam gap is the maximum distance between the two
barycentrically interpolated points of every retained stitch; record the worst
stitch and seam, initial open arrangement separately, and closed-fit rows.

Vertex analytic contact uses distance to each capsule axis minus radius and each
vertex's physical material collision_thickness, and ground is offset-Y. Ground
coverage is complete for a linear triangle. Ellipsoid contact, if present, remains
explicitly vertex-only (this bounded Harbor fixture is expected capsule-only).
Supported static self-contact is current particle/triangle distance with the
existing one-ring/direct-stitch exclusions, physical offsets and material/piece
attribution. Keep degenerate skipped faces and the supported-model limitation
visible. Compare conservative spatial pruning with exhaustive checks in tests.
No edge-edge self-contact or swept/CCD completeness follows from this model.

For EVERY triangle T and capsule axis segment S measure
 d=min(beta in simplex,s in[0,1]) |sum beta_i*x_i-(a+s*(b-a))|.
Physical triangle offset is max(collision_thickness of its incident vertices),
never broadphase padding. Penetration=max(0,radius+offset-d). Retain the distance
interval, worst triangle/capsule/witness and all uncertain pair counts. Conservative
AABB separation may prove zero occupancy. It never removes a pair that could
penetrate; exhaustive/pruned equality is checked on analytic and saved old states.
The f64 geometric candidate is certified by an independent convex dual lower
bound: for unit n, min_i(n dot x_i)-max(n dot a,n dot b) <= d; any actual feasible
pair gives an upper bound. Outward numerical error is retained. If this certificate
is unresolved, invoke independent high-precision constrained-quadratic active-set
enumeration over beta1,beta2,s, preserving face-interior occupancy even for skinny
triangles. Singular/contradictory/uncertified cases stay UNKNOWN, with conservative
penetration bounds and no false zero certification. The frozen old high-precision
oracle and actual83-case results are independent static calibration sources only;
never import transcribed solver code or execute those binaries.
An unbounded f64 roundoff envelope (huge common coordinate with tiny relative
geometry) invokes the same high-precision fallback or stays explicit UNKNOWN;
it never emits a nonfinite certified distance interval.

Root's pre-freeze review distinguishes the reported conservative-policy pair
count from the independent oracle's unresolved-certificate count. Both must be
zero, but they are never equated. A sufficient expected-zero policy proof scans
ALL triangle/capsule pairs, including AABB-separated pairs. In normalized pair
coordinates it bounds component error by4096*epsilon*max(1,world_scale/pair_scale),
bounds longest edge upward by8 such envelopes and area-normal length downward
by32*error*(1+longest)^2, and requires the lower area minus the documented
1e-7*upper_longest^2 to exceed64 envelopes. Normal/edge lengths must additionally
exceed1e-125, far above the f64 squared-underflow regime. This is a sufficient
zero-count proof, not a transcription of the solver or a general counter oracle.
An unproved pair blocks required diagnostic verification; it does not become a
measured penetration failure. A nonzero reported count independently fails its
zero-uncertainty physical screen. Unknown oracle pairs remain separately visible.

Initial intersection counts include vertex/ground, supported self-contact and
full static capsule pairs with depth>1e-8. Capsule intervals give lower/upper pair
counts; any threshold ambiguity is UNKNOWN and blocks diagnostic verification,
with mismatches asserted only when the reported count is outside the interval.
The typed nonnegative initial count must remain unchanged at every later frame.
If the all-pair zero-policy proof fails, the producer's conservative surface
counter may exceed the independent true-intersection range. In that regime the
range is retained as physical geometry, product initial-count agreement is
UNVERIFIED/blocked, and a count outside that range is not labelled a definite
mismatch. Only a proved exact ordinary policy supports the range comparison.
IDs, counts, frame/schema/attempt numbers retain exact integer types; booleans
and float spellings cannot gain integer identity through numeric comparison.

Strict screens: finite; material-axis absolute strain<0.15; sampled seam gap<0.005m;
physical contact residual<0.002m; completed assembly target exactly0; uncertain
capsule pairs exactly0. Boundary equality fails. All rows and exact failed frame
lists remain. Full trajectory and completed-fit screens are separate. Old A29
strain0.1535589019880823, A180 surface0.009009700546250568m and B180 surface
0.010672413745369775m remain historical failures even if new rows improve.

At0/60/180 compare the actual OBJ against the SAME saved actual state, not an old
answer geometry. Require exact v/vt17-decimal tokens, triangle/winding/piece groups,
valid v/vt/vn indices, SI/Y-up positions, independently area-weighted normals and
the existing decimal rounding/normal bounds. Comments are recorded separately.
Duplicate groups, nonfinite values and unexpected record types fail.

## Execution receipts and bounds

A later whole-case receipt must bind one attempt and its case/source/driver/input/
evaluator/authorization, capture the first-launch monotonic start and producer end,
actual producer exit/status/peak RSS (<=268435456 bytes), and final analysis/export/
hash/inspection completion. The420second cap includes independent analysis and
all inspection. A missing/unverifiable clock, exceeded cap/RSS, skipped frame,
producer error or failed export remains a case failure; never continue after cap.
The evaluator checks elapsed time during its own work, preserves a partial report
on cap and never restarts a case. Root is responsible for launching and enforcing
the producer process; this evaluator only validates receipts and saved outputs.
Authorization additionally requires the exact hashed experiment/runner.py
descriptor; a moved wrapper or changed wrapper bytes are refused.
The aggregate numerical-output256MiB cap is checked separately during analysis
and before exclusive report writes. A later integrity-only receipt audit may
read already closed evidence outside the numerical clock; it cannot run new
numeric analysis or revise the analysis/recorded completion clock.

## Static selfchecks and freeze

Analytic tests cover material rotation/compression, seam interpolation, weighted
masses, strict boundaries, duplicate/nonfinite parsing, settings/time/target
recurrence, exact diagnostic attribution failures, OBJ mutations and output
nonoverwrite. Contact tests include face-interior/crossing/parallel/endpoint,
zero-length capsule axes, exact line/point triangles, skinny triangles (1e-5 to
1e-12 altitude), rotations/scales, deliberately unresolved certificates, and retained
old A/B witnesses. Cross-check new capsule results against the existing100-digit
active-set oracle and the83 saved original static cases. Tests may inspect/benchmark
already saved old3aa frames only, never generate new cloth dynamics. Forecast
analysis wall cost before real authorization, retaining wall cap unchanged.

Freeze method, code, copied numeric routines, source/input/preflight bindings and
selfcheck hashes for root source review. Preparation status must explicitly say no
new garment execution and no final physical acceptance/parity established.
