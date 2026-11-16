# Architecture and engineering decisions

The editable garment is authoritative. SI metres, kilograms, seconds and radians
are explicit at boundaries; authored geometry and CPU physics use f64. World
coordinates are right-handed Y-up. Pattern [u,v] maps to [u,v,0] before placement
XYZ Euler rotations then translation. SVG converts metres to millimetres and
flips its vertical coordinate; OBJ uses metres/Y-up.

## Present responsibility boundaries

- `weftforge-core`: stable ID document objects, authored straight/cubic boundaries and straight internal annotations,
  oriented seam intervals and ordered sewing groups, materials and
  capsule mannequin; typed commands, transaction/history; meshing, project IO,
  OBJ/SVG/DXF export and one document-to-solver adapter.
- `weftforge-sim`: GUI-independent CPU cloth, material-space strain, dihedral bend,
  area mass, weighted sewing, contact, diagnostics and transactional step failure.
- `weftforge`: native eframe/egui editor and wgpu-backed drawing. Pattern canvas,
  original capsule avatar and depth-sorted projected garment triangles are the
  first viewport implementation. It does not yet have a dedicated depth-buffered
  garment shader, PBR, textures, shadows, animation timeline or GPU simulation.
- `weftforge-cli`: offline creation/inspect/edit/mesh/simulate/export/benchmark and
  explicit recovery through the same library commands. Nine stateless local MCP
  file tools expose the shared validated engine; live-GUI control remains open.
- `xtask`, `scripts/`: integration, independently parsed output, registry/evidence
  validation, report/dashboard generation and unsigned package preparation.

These are substantive modules, not placeholders for dozens of future crates.
Separate avatar/animation/render crates when ownership or build isolation benefits.
The libraries do not depend on egui, a browser, Python, REA, or commercial SDKs.

## Authored geometry, rest mesh and state

Every point, curve, piece, material, seam and sewing group has a stable integer ID. Curves
reference authored points and optional absolute Bezier control coordinates.
Piece boundaries are ordered closed chains. Each piece also owns an explicit
`internal_curves` array of independent straight annotations backed by dedicated
Point/Curve IDs. Both endpoints are exclusive to their annotation; crossings or
coincident coordinates do not create shared topology. These lines are editable
construction with no cut, hole, fold, sewing, mass or simulation effect. Boundary
controls/split/join and every sewing route reject internal targets. Resize, rigid
transform and removal traverse both endpoints, while bounds, mesh construction
and SVG/DXF exports use the exterior only. OBJ contains no annotation primitives.

Annotation validation establishes whole-segment separation from a certified
curved-contour polyline, independently of mesh resolution: 1e-6 m clearance,
1e-7 m maximum chord deviation and a conservative coordinate-scaled f64 reserve.
The contour is built once per annotated piece; depth 20, 20,000 boundary samples
and one million project-wide subdivision/predicate steps bound work. Uncertain,
self-touching and exhausted cases fail atomically. These are WeftForge engineering
policies under the [reviewed contract](../spec/features/PAT-006-internal-segments.md),
with reference behavior and actual native acceptance recorded separately.

`CutPiece` replaces one straight polygon with two ordinary independent pieces.
It splits two distinct boundary edges, duplicates their new anchor points and
closes each child with its own chord. Material, placement and grain are retained;
there is no persistent link between the two cut edges. One command is one history
entry. Target sewing refuses the whole cut; annotations must fit wholly inside
exactly one child or the cut refuses. A separate one-million-step budget covers
authored straight-contour/chord/assignment predicates across the outer transaction,
independently of mesh resolution. The [straight-cut contract](../spec/features/PAT-006-straight-cut.md)
defines numeric margins, ownership, nine-ID allocation and current eligibility.

A seam stores two oriented [t0,t1]
intervals and a monotone normalized parameter correspondence; it never refers to
simulation vertex indices. Unequal lengths are deliberate easing.

An ordered group instead stores two lists of authored intervals and one map of
normalized cumulative arc lengths. Each interval retains its curve t endpoints;
gaps between disconnected spans add no length. Both one-sided endpoints at a
path knot survive. Metric work is bounded, with a1e-8m authored-path budget and
explicit errors for exhausted work or unsupported floating-point precision.
Native v4 saves legacy seams, groups and owned internal annotations; strict older-format migration
does not reinterpret legacy correspondence.

The rebuildable rest mesh has material coordinates, physical arranged positions,
triangles, per-vertex piece IDs, source document revision and sampled boundary
curve parameters. Sewing samples both intervals through their explicit map and
interpolates weighted positions on the current boundary mesh. Refinement retains
this authored correspondence. The simulation receives a distinct state, never
mutates the document, and is invalidated by construction edits. The current
export mesh is a copy of the rest mesh with simulated positions. UVs remain
physical pattern coordinates rather than a packed atlas; output thickness and
welding remain open work. The rest mesh also carries each group's current
lengths and paired source portions for native guides; rendering does not
integrate curve lengths every frame. These rest constraints do not establish
physical gathering quality or a converged sewn garment.

`TransformPiece` is a rigid 2D layout edit of every owned endpoint and absolute
cubic control. It rotates about an explicit authored pivot, then translates.
Placement compensation right-composes the existing XYZ rotation basis with the
inverse in-plane rotation and adjusts its translation; the continuous arranged
surface stays fixed. Grain rotates with the authored coordinates so physical
warp/weft directions remain fixed. A private basis-to-Euler helper uses a fixed
32-f64-epsilon singular threshold; pure translation bypasses that decomposition
and preserves Euler/grain bits. Other authoring commands retain their existing
semantics. No persistent layout frame or document version is introduced.

UVs continue to be authored metres. A transformed document requires fresh rest,
display and simulation revisions even when its arranged surface is unchanged;
triangulation and later solver trajectories may differ. The
[reviewed contract](../spec/features/PAT-004-piece-rigid-transform.md) fixes numeric
bounds and describes the engineering policy; reference behavior is unobserved.

Convex/concave boundaries are tessellated then triangulated and refined with
shared midpoint vertices. Cubic bounds use analytic derivative roots. Polygon
validity uses a bounded polyline approximation for curves: intersections below
the documented 0.2 mm tolerance can remain unresolved. Collinear zero-area faces
from ear clipping are discarded without weakening the strict positive-area mesh
validation. De Casteljau splits retain exact authored cubics. Every affected seam
is partitioned in its oriented correspondence coordinates and remapped to local
curve parameters; unrelated IDs and material assignments remain intact. The
original seam ID survives on its first oriented portion and later portions get
new IDs. Invalid or unrepresentably short portions fail the whole transaction.
Group split/join rewrites preserve their single identity, map, traversal order
and separate authored list entries. Removing a participating piece removes the
whole dependent group; unrelated groups survive. Quality improvement runs before
each refinement level to keep inherited slivers from multiplying, while the
existing mesh limits, authored boundary parameters and cancellation checks remain.

## Commands, transactions, revisions and jobs

`Editor::execute(Command)` clones, validates and commits a document transaction;
failure retains the previous project/history. A transaction groups related
commands into one undo entry. Undo/redo increment the revision and retain the
allocator high-water mark; removed object IDs stay reserved across saving. IDs
are not reused after an undo branch. Persistent
construction differs from selection, camera, simulation frames and rebuildable
caches. Point-drag previews do not mutate construction until commit. Inspector
numeric drafts commit once per gesture. Mesh preparation and solver stepping
run in revision-tagged workers, and explicit recovery includes untitled autosaves;
consult STATUS.md for the tested revision.

CPU simulation operates in a bounded worker batch with cancellation and an input
revision. The native worker passes the same atomic flag through
`create_cloth_cancellable` to `Cloth::new_cancellable`, then into stepping.
Initialization checks cancellation inside input/constraint construction and initial
contact bounds/vertex/candidate loops; a cancelled candidate is discarded without
replacing accepted cloth, rest mesh or construction. Ordinary constructors delegate
with a non-cancelling callback. Core project/mesh validation, allocation/copying and
final linear diagnostic aggregation still have before/after-only checks, so this
is cooperative rather than a fixed cancellation-latency bound. Results for an
obsolete document are rejected. Step failure or cancel
retains the previous complete state. Expensive meshing and solver preparation
must use the same revision/cancel discipline; command validation, file IO and
native dialogs currently retain bounded synchronous portions. No timeline cache
can yet be saved or reused. Persisted revision IDs alone are not a future cache
key: include a digest of all construction/material/avatar/solver/animation inputs.

## Dependency and toolchain decision

Rust 1.94.1 is pinned, along with direct versions and Cargo.lock. eframe/egui
0.35.0 is the newest verified release compatible with this existing toolchain
(0.36/0.37 declare Rust 1.95). Its actual local API uses `App::ui`, inspected before
integration. wgpu/Metal is rendering, not solver acceleration. See
[eframe 0.35 API](https://docs.rs/eframe/0.35.0/eframe/) and
[dependency declarations](dependencies.json). Platform frameworks, native bindings
and GPU drivers are expected; the dependency graph is not entirely Rust.

The CPU path has fixed ordering, f64 values and explicit timestep/substeps. It
promises repeatable bounded conditions on a tested platform, not universal CPU/GPU
bitwise identity. Independent reference evaluation is separate from both engines.

The [PhotoCraft architecture](https://github.com/storytold/photocraft/blob/main/docs/architecture.md)
was inspected as a responsibility-boundary and delivery benchmark; its source,
architecture size and test counts are not evidence of WeftForge parity.
