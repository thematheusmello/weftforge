# Architecture and engineering decisions

The editable garment is authoritative. SI metres, kilograms, seconds and radians
are explicit at boundaries; authored geometry and CPU physics use f64. World
coordinates are right-handed Y-up. Pattern [u,v] maps to [u,v,0] before placement
XYZ Euler rotations then translation. SVG converts metres to millimetres and
flips its vertical coordinate; OBJ uses metres/Y-up.

## Present responsibility boundaries

- `weftforge-core`: stable ID document objects, authored straight/cubic boundaries,
  oriented seam intervals with explicit monotone correspondence, materials and
  capsule mannequin; typed commands, transaction/history; meshing, project IO,
  OBJ/SVG export and one document-to-solver adapter.
- `weftforge-sim`: GUI-independent CPU cloth, material-space strain, dihedral bend,
  area mass, weighted sewing, contact, diagnostics and transactional step failure.
- `weftforge`: native eframe/egui editor and wgpu-backed drawing. Pattern canvas,
  original capsule avatar and depth-sorted projected garment triangles are the
  first viewport implementation. It does not yet have a dedicated depth-buffered
  garment shader, PBR, textures, shadows, animation timeline or GPU simulation.
- `weftforge-cli`: offline creation/inspect/edit/mesh/simulate/export/benchmark and
  explicit recovery through the same library commands. Local MCP remains planned.
- `xtask`, `scripts/`: integration, independently parsed output, registry/evidence
  validation, report/dashboard generation and unsigned package preparation.

These are substantive modules, not placeholders for dozens of future crates.
Separate avatar/animation/render crates when ownership or build isolation benefits.
The libraries do not depend on egui, a browser, Python, REA, or commercial SDKs.

## Authored geometry, rest mesh and state

Every point, curve, piece, material and seam has a stable integer ID. Curves
reference authored points and optional absolute Bezier control coordinates.
Piece boundaries are ordered closed chains. A seam stores two oriented [t0,t1]
intervals and a monotone normalized parameter correspondence; it never refers to
simulation vertex indices. Unequal lengths are deliberate easing.

The rebuildable rest mesh has material coordinates, physical arranged positions,
triangles, per-vertex piece IDs, source document revision and sampled boundary
curve parameters. Sewing samples both intervals through their explicit map and
interpolates weighted positions on the current boundary mesh. Refinement retains
this authored correspondence. The simulation receives a distinct state, never
mutates the document, and is invalidated by construction edits. The current
export mesh is a copy of the rest mesh with simulated positions. UVs remain
physical pattern coordinates rather than a packed atlas; output thickness and
welding remain open work.

Convex/concave boundaries are tessellated then triangulated and refined with
shared midpoint vertices. Cubic bounds use analytic derivative roots. Polygon
validity uses a bounded polyline approximation for curves: intersections below
the documented 0.2 mm tolerance can remain unresolved. Collinear zero-area faces
from ear clipping are discarded without weakening the strict positive-area mesh
validation. Sewn-curve splits are explicitly rejected pending interval rewriting;
unsupported relationships are not silently discarded.

## Commands, transactions, revisions and jobs

`Editor::execute(Command)` clones, validates and commits a document transaction;
failure retains the previous project/history. A transaction groups related
commands into one undo entry. IDs are not reused after an undo branch. Persistent
construction differs from selection, camera, simulation frames and rebuildable
caches. Point-drag previews do not mutate construction until commit. Inspector
numeric drafts commit once per gesture. Mesh preparation and solver stepping
run in revision-tagged workers, and explicit recovery includes untitled autosaves;
consult STATUS.md for the tested revision.

CPU simulation operates in a bounded worker batch with cancellation and an input
revision. Results for an obsolete document are rejected. Step failure or cancel
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
