# WeftForge implementation map

This source-pinned engineering map assigns every entry in the reviewed menu and component catalogs. It preserves the complete target and prepares bounded implementation handoffs. It does not establish feature completion or reference parity.

Source: `9acb265411e80d01152a5345a7119e2ed8bb2194`. Reference build: `2026.1.237 (r58920)`. [Machine-readable map](weftforge-implementation-map.json).

Coverage: **755 unique IDs** = 589 menu nodes + 166 components; 498 action candidates, 166 components, 86 containers, 3 provisional headings, 2 excluded vendor administration entries. The menu input contains 17 roots, 572 nonroot entries and 505 leaf-like nodes. **These counts are not a feature denominator.**

Current implementation, reference observation, verification and evidence freshness come only from [parity/features.yaml](../../parity/features.yaml). Join each group’s feature IDs to that registry at the revision being reported. The map copies no feature status fields. Candidate source relationships describe only the pinned code and must be re-reviewed after relevant source/catalog changes.

## Scope and source interpretation

The native code has 18 Command variants, including Transaction, and four tool modes: Select, Polygon, Rectangle and Sew. Existing related paths include native versioned projects/recovery, editable polygons/cubics, exact split/seam remapping, piece resizing/placement, grain/physical fabric editing, interval sewing, triangle meshing, a cancellable CPU solver, shared selection and bounded CLI/MCP file authoring. These are candidates for comparison, not established semantic matches.

The 193 engineering groups have 53 source-backed partial candidates, 120 planned missing mappings and 20 unresolved mappings. Groups combine candidate aliases and engineering variants; they do not collapse differing format directions, selection scopes, parameter semantics or entry paths. Every retained path remains in the JSON. Similar names do not establish behavior equivalence.

Only AI Studio Activation and Sign Out are excluded vendor activation/session administration entries. AI Studio creation remains a discovery lead. EveryWear, GameWear/Explore, AI pose, auto avatar/motion conversion, pattern drafting, libraries and AI assistance remain in scope with local-substitute leads or explicit unresolved semantics. Resolve destination-only storefront administration from runtime evidence before excluding any more entries.

The disabled Pattern, Arrangement and Layouts leaves are provisional headings; selected-state observation must resolve whether they ever become actions. Recent and Plug-in are empty containers with unknown dynamic children. Neither heading/container counts nor these unresolved leaves establish a completion percentage.

## Expanded property/control review

The 105 added IDs preserve all earlier 650 entry mappings, all 184 packets and the first three bounded slices unchanged. The [reviewed surface notes](../features/UI-property-surfaces-20261114.md) cover 24 Simulation Properties controls, 31 fields on one selected original pattern, eight rectangle-constructor controls, 29 OBJ export options and 13 Fit controls. Most controls were only read; the [bounded rectangle run](../features/PAT-003-runtime-20261114.md) records keyboard width/height entry, one OK, history, document reopen and one unchanged-option export. These observations supply no completed broad feature or WeftForge parity claim.

The source pin contains current candidate code **before the separate numeric-dialog and widget stable-ID implementation is integrated**. Document stable IDs already exist; the new dialog work is not claimed here. Preserve the catalog commit as an ancestor when integrating this map so its source/catalog links remain resolvable.

Most new controls join existing groups: global simulation properties, grain alignment/reset, solidify, particle-distance resolution, seam taping, UVs, topology, fabric assignment, statistics, rectangle, OBJ and fit maps. Nine appended packets separate selected name/line-length properties, elastic, schematic-line presentation, physical layers, directional shrinkage, additional collision thickness, pressure and rendering extrusion/side geometry. Their precise source gaps, runtime questions and fixtures are in JSON; no existing variant is removed.

- Particle Distance is explicitly mm and differs from Side Geometry Resolution. Additional physical collision thickness differs from rendering extrusion. Directional shrinkage differs from material stiffness.
- Global solver values and selected grain/pressure fields with no printed unit remain unknown. CG labels do not identify an algorithm; this host’s disabled CPU count does not define a product constant. Ground children remain collapsed/unobserved.
- Constructor pre-OK dimensions differ from initial dialog values. Link, replication, Cancel and invalid-input limits remain untested; the observed width/height and one OK do not complete those controls.
- The executed OBJ route and visible title are Export despite the AX Import title. Alternate grouping/weld/thickness/UV/axis/texture/preset/metadata options remain untested. Exported units come from the observed unit control, never inferred by fitting dimensions.
- Fit Properties shows Stress and a visible unopened Pressure tab. The unsuccessful Strain selection does not establish a Strain tab; the independent Strain Map menu entry stays in scope. Displayed kPa ranges and colors are neither material calibration nor accepted edit limits.

## First three executable parts

Each starts only after a reviewed public runtime contract and an original fixture answer the listed uncertainty. Existing implementation-only specifications remain useful regressions; they do not substitute for a reference run.

### 1. rectangle

One native rectangle numeric/drag creation contract only; preserve the existing AddRectangle path across GUI/CLI/MCP.

Contract prerequisite: Reviewed rectangle runtime contract: gestures, anchor/dimensions/units/defaults/ranges, numeric accept/cancel, error and one-undo semantics.

Acceptance: Run the same original empty-to-rectangle actions in both real applications; exact authored size/anchor and cancelled/no-history state match; edit/save/reopen/undo and adapters pass, with retained independent comparison evidence.

- **authoring**: `crates/weftforge-core/src/document.rs`, `crates/weftforge-core/tests/workflow.rs`
- **desktop**: `apps/weftforge/src/main.rs`
- **automation**: `apps/weftforge-cli/src/main.rs`, `apps/weftforge-cli/src/mcp.rs`, `apps/weftforge-cli/tests/mcp_stdio.rs`

### 2. point-pattern-edit

One selected authored point drag with preview/commit/cancel and connected piece selection; do not bundle multi-selection or whole-pattern transforms.

Contract prerequisite: Reviewed point selection/drag contract: pick tolerance, snap/modifier/numeric behavior, preview, cancel and gesture boundary.

Acceptance: On an original curved piece, compare exact point/control/selection state and one undo unit; invalid/cancelled moves retain the old document, remesh invalidates old simulation and saved construction remains editable.

- **authoring**: `crates/weftforge-core/src/document.rs`, `crates/weftforge-core/tests/workflow.rs`
- **desktop**: `apps/weftforge/src/main.rs`
- **automation**: `apps/weftforge-cli/src/mcp.rs`, `apps/weftforge-cli/tests/mcp_stdio.rs`

### 3. segment-sewing

One oriented straight/curved edge-interval pair, including reversed and deliberate unequal-length fixture plus visible lengths/direction; leave chains/M:N/auto sewing for their own packets.

Contract prerequisite: Reviewed segment-sewing runtime contract with entry paths, pick sequence, interval parameter semantics, orientation, mismatch/error policy and retained editable correspondence.

Acceptance: Independent authored correspondence/length checks survive reversed orientation, unequal tessellation, split/remesh/undo/reopen; native diagnostics and actual simulated seam closure meet predeclared criteria without using WeftForge as the reference answer.

- **authoring**: `crates/weftforge-core/src/document.rs`, `crates/weftforge-core/tests/workflow.rs`
- **geometry**: `crates/weftforge-core/src/geometry.rs`
- **solver**: `crates/weftforge-core/src/simulation.rs`
- **desktop**: `apps/weftforge/src/main.rs`
- **automation**: `apps/weftforge-cli/src/mcp.rs`, `apps/weftforge-cli/tests/mcp_stdio.rs`

These first slices deliberately leave multi-selection, chain/M:N/auto sewing and the rest of their group variants in the queue. Finishing a slice does not finish its coarse feature contract.

## Ownership and acceptance

The JSON gives each packet explicit dependencies and a linked acceptance profile with owner lanes and decisive checks; linked groups supply runtime questions, original fixture recipes and source candidates. Shared packet rules define contract inputs and integrated checks once. Queue order respects prerequisite packets; dependencies mean qualifying contract/interface slices, not completion of an entire coarse feature. Agents must not concurrently edit a file owned by the same lane. The central desktop, document and adapter files have one active owner each.

An investigator owns the exclusive reference UI lease and supplies reviewed public contracts. Fresh implementation agents receive original fixtures and those contracts, with no proprietary code/account/raw UI records. Geometry, authoring, simulation, persistence, desktop and automation owners integrate behind typed command and revision interfaces. The verification owner independently checks outcomes and updates the authoritative registry, scenarios/reports, STATUS and generated progress at integration.

Each packet is bounded to one reviewed operation or entry variant on one original fixture. A separate state machine, format direction or object type becomes a separate delivery slice before edits. The packet’s linked group keeps every remaining entry visible. New authoring uses shared typed commands, one gesture transaction and matching GUI/CLI/MCP schemas/adapters/tests; core/solver stay GUI-independent. Loads, saves, failed jobs and cancellation preserve the last accepted project/state.

Every integrated change runs:

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask verify
```

Checks from our implementation cannot supply the reference answer. Compare exact authored state or predeclared dimensioned metrics using independent fixtures/evaluators; keep unknown, blocked, failed, partial and stale evidence visible.

## Ordered handoff queue

| Order | Packet / behavior group | Authoritative feature links | Dependency packets | Owner lanes |
|---:|---|---|---|---|
| 1 | `PACK-rectangle` — Draw or numerically create rectangle pattern | PAT-003, PAT-001, UI-004, APP-001 | — | authoring, desktop, automation, verification |
| 2 | `PACK-point-pattern-edit` — Edit point/segment/pattern selection and geometry | PAT-001, PAT-002, PAT-004, UI-002, UI-004 | `PACK-rectangle` | authoring, desktop, automation, verification |
| 3 | `PACK-segment-sewing` — Sew one oriented boundary interval to another | SEW-001, SEW-003, SEW-004, UI-004 | `PACK-point-pattern-edit` | authoring, geometry, solver, desktop, automation, verification |
| 4 | `PACK-undo-redo` — Undo/redo authoring transactions and inspect history | UI-004, APP-001 | `PACK-rectangle` | authoring, desktop, automation, verification |
| 5 | `PACK-delete-selection` — Delete the selected authored objects | UI-002, UI-004, PAT-001, SEW-004 | `PACK-rectangle` | authoring, desktop, automation, verification |
| 6 | `PACK-selection` — Piece selection, select-all/sewn/invert and connected view selection | UI-002, UI-003 | `PACK-rectangle` | authoring, desktop, automation, verification |
| 7 | `PACK-polygon` — Draw editable polygon pattern | PAT-001, UI-004 | `PACK-rectangle` | authoring, desktop, automation, verification |
| 8 | `PACK-reverse-sewing` — Reverse an existing seam | SEW-001, SEW-004, UI-004 | `PACK-point-pattern-edit` | authoring, geometry, solver, desktop, automation, verification |
| 9 | `PACK-sewing-editor` — Select/edit/remove sewing | SEW-004, SEW-001, UI-002, UI-004 | `PACK-point-pattern-edit` | authoring, geometry, solver, desktop, automation, verification |
| 10 | `PACK-free-chain-sewing` — Free or M:N segment/free sewing | SEW-002, SEW-003, SEW-004 | `PACK-point-pattern-edit` | authoring, geometry, solver, desktop, automation, verification |
| 11 | `PACK-auto-sewing` — Automatically propose/apply sewing | SEW-002, SEW-004 | `PACK-point-pattern-edit` | authoring, geometry, solver, desktop, automation, verification |
| 12 | `PACK-sewing-length-check` — Check sewing lengths | SEW-004, SEW-003 | `PACK-point-pattern-edit` | authoring, geometry, solver, desktop, automation, verification |
| 13 | `PACK-sewing-display` — Display/hide sewing, lengths, seamlines and threads | SEW-004, SEW-006, MAT-003 | `PACK-point-pattern-edit` | authoring, geometry, solver, desktop, automation, verification |
| 14 | `PACK-point-conversion` — Convert curve/segment points with or without maintaining curve | PAT-002, PAT-005, UI-004 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 15 | `PACK-curve-tools` — Edit curvature/curve points or smooth curves | PAT-002, UI-004 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 16 | `PACK-split-boundary` — Add point or split line | PAT-005, PAT-002, SEW-001, UI-004 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 17 | `PACK-ellipse` — Create ellipse primitive | PAT-003, PAT-002 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 18 | `PACK-spiral` — Create spiral primitive | PAT-003, PAT-002 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 19 | `PACK-project-new` — New project | FILE-001, FILE-002, UI-004 | `PACK-rectangle` | files, authoring, desktop, automation, verification |
| 20 | `PACK-garment-new` — New garment inside a project | FILE-001, FIT-002 | `PACK-rectangle` | files, authoring, desktop, automation, verification |
| 21 | `PACK-project-load-save` — Open/save complete editable projects and metadata variants | FILE-001, FILE-002, FILE-006 | `PACK-rectangle` | files, authoring, desktop, automation, verification |
| 22 | `PACK-garment-packages` — Load, add and save garment packages, including selection and metadata | FILE-001, FILE-006, FIT-002 | `PACK-rectangle` | files, authoring, desktop, automation, verification |
| 23 | `PACK-project-merge` — Add project to an existing document | FILE-001, FILE-006, APP-001 | `PACK-rectangle` | files, authoring, desktop, automation, verification |
| 24 | `PACK-garment-information` — Garment information | FILE-001, UI-001, PERF-001 | `PACK-rectangle` | files, authoring, desktop, automation, verification |
| 25 | `PACK-file-history` — File history | FILE-001, UI-004 | `PACK-rectangle` | files, authoring, desktop, automation, verification |
| 26 | `PACK-application-quit` — Quit while retaining recoverable authoring | FILE-002, APP-004, APP-002 | `PACK-rectangle` | files, authoring, desktop, automation, verification |
| 27 | `PACK-change-length` — Change a selected boundary length | PAT-004, PAT-002, PAT-007 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 28 | `PACK-pattern-groups` — Group/ungroup pattern selections | UI-002, PAT-004, FILE-001 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 29 | `PACK-pattern-flip` — Flip selected patterns together or each independently | PAT-004, PAT-008, SEW-004 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 30 | `PACK-pattern-order` — Change 2D stacking order | UI-001, UI-002, FILE-001 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 31 | `PACK-pattern-rotate` — Rotate by fixed angle/axis or parallel to a target | PAT-004, UI-004 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 32 | `PACK-pattern-align-distribute` — Align or distribute pattern selections | PAT-004, UI-002 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 33 | `PACK-match-up` — Match start/center/end or measurements | PAT-004, PAT-007, SEW-004 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 34 | `PACK-pattern-transform-tool` — Transform complete patterns | PAT-004, UI-004 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 35 | `PACK-pattern-smart-snap` — Smart guides and snap to pattern | PAT-004, UI-003, PAT-007 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 36 | `PACK-grid-snap` — Snap to grid | PAT-004, UI-003 | `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 37 | `PACK-offsets` — Offset outline or create internal offset | PAT-009, PAT-007 | `PACK-split-boundary`, `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 38 | `PACK-internal-distribution` — Distribute internal lines between segments | PAT-007, PAT-010 | `PACK-split-boundary`, `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 39 | `PACK-cut-and-sew` — Cut pattern or cut-and-sew construction | PAT-006, SEW-002, UI-004 | `PACK-split-boundary`, `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 40 | `PACK-extend-trim` — Extend/trim against outline/internal line, optionally add point | PAT-005, PAT-007 | `PACK-split-boundary`, `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 41 | `PACK-trace` — Trace source contours into pattern/internal shape | PAT-001, PAT-007, PAT-006 | `PACK-split-boundary`, `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 42 | `PACK-garment-measurements` — Create/edit linear or circumference garment measures and display | PAT-007, FIT-003, FILE-001 | `PACK-split-boundary`, `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 43 | `PACK-internal-primitives` — Create internal polygon/line, rectangle or ellipse | PAT-007, PAT-006 | `PACK-split-boundary`, `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 44 | `PACK-internal-line-display` — Display internal authored lines | PAT-007, MAT-003 | `PACK-split-boundary`, `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 45 | `PACK-line-measure-display` — Display line lengths and angles | PAT-007, MAT-003 | `PACK-split-boundary`, `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 46 | `PACK-rulers-guidelines` — Display/use rulers and guidelines | PAT-007, UI-003 | `PACK-split-boundary`, `PACK-point-pattern-edit` | authoring, geometry, desktop, automation, verification |
| 47 | `PACK-clipboard` — Copy/paste selection | UI-002, UI-004, PAT-010, FILE-001 | `PACK-split-boundary`, `PACK-segment-sewing` | authoring, geometry, desktop, automation, verification |
| 48 | `PACK-unfold` — Unfold and unfold with symmetric sewing edits | PAT-010, SEW-002 | `PACK-split-boundary`, `PACK-segment-sewing` | authoring, geometry, desktop, automation, verification |
| 49 | `PACK-mirror-paste` — Mirror paste copied construction | PAT-004, PAT-010, UI-004 | `PACK-split-boundary`, `PACK-segment-sewing` | authoring, geometry, desktop, automation, verification |
| 50 | `PACK-linked-symmetry` — Clone/link/instance/symmetric patterns and remove links | PAT-010, SEW-002, FILE-001 | `PACK-split-boundary`, `PACK-segment-sewing` | authoring, geometry, desktop, automation, verification |
| 51 | `PACK-clone-construction` — Clone as pattern or internal shape, with optional sewing | PAT-010, PAT-007, SEW-002 | `PACK-split-boundary`, `PACK-segment-sewing` | authoring, geometry, desktop, automation, verification |
| 52 | `PACK-slash-spread` — Slash and spread construction | PAT-010, PAT-006, PAT-004 | `PACK-split-boundary`, `PACK-segment-sewing` | authoring, geometry, desktop, automation, verification |
| 53 | `PACK-darts` — Create dart | PAT-010, SEW-002, PAT-007 | `PACK-split-boundary`, `PACK-segment-sewing` | authoring, geometry, desktop, automation, verification |
| 54 | `PACK-pleats` — Create/fold/sew pleats | PAT-010, SEW-002, SEW-005 | `PACK-split-boundary`, `PACK-segment-sewing` | authoring, geometry, desktop, automation, verification |
| 55 | `PACK-linked-line-display` — Display symmetric/instance editing links | PAT-010, UI-002 | `PACK-split-boundary`, `PACK-segment-sewing` | authoring, geometry, desktop, automation, verification |
| 56 | `PACK-obj-interchange` — OBJ import/add/to-garment/export/selected export | FILE-003, FIT-001, ANIM-003 | `PACK-point-pattern-edit` | authoring, geometry, files, desktop, automation, verification |
| 57 | `PACK-strengthen` — Strengthen pattern | FIT-002, SIM-002 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, solver, desktop, automation, verification |
| 58 | `PACK-solidify` — Solidify pattern | FIT-002, SIM-002, FIT-004 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, solver, desktop, automation, verification |
| 59 | `PACK-fabric-assignment` — Assign/reset/create fabrics and browse fabric objects | MAT-001, SIM-002, FILE-001 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, solver, desktop, automation, verification |
| 60 | `PACK-fabric-angle` — Reset fabric angle | PAT-008, MAT-002, REF-002 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, solver, desktop, automation, verification |
| 61 | `PACK-grain-alignment` — Align grain to X/Y/selected direction and show grainlines | PAT-008, SIM-002 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, solver, desktop, automation, verification |
| 62 | `PACK-simulate` — Play/pause simulation from menu, toolbar or shortcut | SIM-005, UI-005, FIT-003, APP-004 | `PACK-segment-sewing` | geometry, solver, desktop, automation, verification |
| 63 | `PACK-resolution` — High/low/custom garment resolution | PAT-011, FIT-003, ANIM-003 | `PACK-segment-sewing` | geometry, solver, desktop, automation, verification |
| 64 | `PACK-wind-controller` — Display/edit wind controller | SIM-001, SIM-005, MAT-003, REF-002 | `PACK-segment-sewing` | geometry, solver, desktop, automation, verification |
| 65 | `PACK-simulation-properties` — Edit simulation properties | SIM-005, SIM-001, SIM-002, SIM-004, SIM-007 | `PACK-segment-sewing` | geometry, solver, desktop, automation, verification |
| 66 | `PACK-pattern-activation` — Archive or deactivate pattern and sewing participation | FIT-002, SIM-005, SEW-004, FILE-001 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 67 | `PACK-freeze` — Freeze pattern motion | FIT-002, SIM-005 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 68 | `PACK-superimpose-layer-clone` — Superimpose and layer-clone pieces | FIT-002, PAT-010, SIM-004 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 69 | `PACK-authored-pins` — Select, remove, attach or detach pins | FIT-002, FILE-001, SIM-005 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 70 | `PACK-mesh-selection` — Select cloth mesh by brush, box or lasso | UI-002, FIT-002 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 71 | `PACK-pin-display` — Display authored pins | FIT-002, MAT-003 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 72 | `PACK-pattern-layer-display` — Display pattern layer values | FIT-002, UI-001 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 73 | `PACK-arrangement` — Reset, re-drape, mirror, reverse, flatten or wrap arrangement | FIT-001, FIT-003, PAT-004, UI-004 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 74 | `PACK-roll-up` — Roll up horizontal or vertical arrangement | SEW-005, FIT-001 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 75 | `PACK-stylelines` — Draw/edit/move/scale stylelines | FIT-004, PAT-002, PAT-007 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 76 | `PACK-surface-drawing` — 3D pen/pencil on garment or avatar, including line editing and display | FIT-004, PAT-002, PAT-007 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 77 | `PACK-flatten` — Flatten 3D drawn/garment regions | FIT-004, PAT-001, PAT-002 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 78 | `PACK-auto-fitting` — Auto fit garment | FIT-003, FIT-001, PAT-004, APP-003 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 79 | `PACK-garment-fitting-suit` — Display garment fitting suit | FIT-003, FIT-004, REF-002 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 80 | `PACK-gizmos` — Screen/local/world gizmos and gizmo scale | PAT-004, FIT-001, UI-003 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 81 | `PACK-sculpt` — Sculpt garment | FIT-004, ANIM-003, SIM-005 | `PACK-point-pattern-edit`, `PACK-segment-sewing` | authoring, geometry, solver, desktop, automation, verification |
| 82 | `PACK-trim-assets` — Load/save trim assets and browse trim objects | SEW-005, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 83 | `PACK-garment-object-conversion` — Convert garment to trim or avatar accessory | SEW-005, FIT-001, ANIM-003 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 84 | `PACK-material-context-unknowns` — Selection-specific material duplicate/reset/layer/flip actions | MAT-001, SEW-005, SEW-006, REF-002 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 85 | `PACK-steam` — Apply/remove steam, selected or all scope | SEW-005, SIM-002, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 86 | `PACK-press` — Press garment | FIT-004, SEW-005, SIM-002 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 87 | `PACK-fold-arrangement` — Fold arrangement | SEW-005, FIT-001, PAT-007 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 88 | `PACK-ripping` — Edit or create seamline rips | SEW-004, SEW-005, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 89 | `PACK-lacing` — Create/edit lacing | SEW-005, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 90 | `PACK-tacks` — Create/edit tacks, including avatar tacks | SEW-005, FIT-002, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 91 | `PACK-buttons` — Create/move/fasten/register buttons and buttonholes | SEW-005, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 92 | `PACK-zippers` — Create/edit zippers and register OBJ assets | SEW-005, FILE-001, FILE-003 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 93 | `PACK-piping` — Create/edit piping | SEW-005, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 94 | `PACK-binding` — Create/select binding | SEW-005, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 95 | `PACK-topstitch` — Create/edit/assign segment, free or seamline topstitch | SEW-006, PAT-007, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 96 | `PACK-puckering` — Create/edit segment, free or seamline puckering | SEW-006, SIM-002, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 97 | `PACK-seam-taping` — Apply seam taping | SEW-005, SIM-002, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 98 | `PACK-trim-display` — Display buttons, piping, bond/skive, trims, puckering and status colors | SEW-005, SEW-006, MAT-003 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 99 | `PACK-avatar-assets` — Load, add and save avatar assets | FIT-001, FILE-001, ANIM-001 | `PACK-obj-interchange` | authoring, geometry, solver, files, desktop, automation, verification |
| 100 | `PACK-accessory-assets` — Open/register avatar accessories | FIT-001, SEW-005, FILE-001 | `PACK-obj-interchange` | authoring, geometry, solver, files, desktop, automation, verification |
| 101 | `PACK-scene-assets` — Open/add/save/delete scene and prop objects | FIT-001, FILE-001, MAT-003 | `PACK-obj-interchange` | authoring, geometry, solver, files, desktop, automation, verification |
| 102 | `PACK-avatar-lifecycle` — Delete/deactivate avatar, accessory or ground placement | FIT-001, SIM-003, FILE-001 | `PACK-obj-interchange` | authoring, geometry, solver, files, desktop, automation, verification |
| 103 | `PACK-avatar-measurements` — Create/edit/attach avatar tape, linear, height and circumference measures | FIT-001, PAT-007, FILE-001 | `PACK-obj-interchange` | authoring, geometry, solver, files, desktop, automation, verification |
| 104 | `PACK-avatar-editor` — Edit avatar body/shape | FIT-001, ANIM-001, APP-003 | `PACK-obj-interchange` | authoring, geometry, solver, files, desktop, automation, verification |
| 105 | `PACK-arrangement-aid-display` — Display avatar arrangement points or bounding volumes | FIT-001, MAT-003 | `PACK-obj-interchange` | authoring, geometry, solver, files, desktop, automation, verification |
| 106 | `PACK-avatar-fitting-suit` — Display avatar fitting suit | FIT-001, FIT-003, REF-002 | `PACK-obj-interchange` | authoring, geometry, solver, files, desktop, automation, verification |
| 107 | `PACK-pose-assets` — Open/save body or hand poses and register pose/motion | ANIM-001, FILE-001 | `PACK-avatar-assets` | authoring, solver, files, desktop, automation, verification |
| 108 | `PACK-joint-motion-assets` — Open/save joint motion assets | ANIM-001, FILE-007 | `PACK-avatar-assets` | authoring, solver, files, desktop, automation, verification |
| 109 | `PACK-bone-json` — Export bone information JSON | FILE-007, ANIM-001 | `PACK-avatar-assets` | authoring, solver, files, desktop, automation, verification |
| 110 | `PACK-symmetric-weights` — Copy symmetric weights | ANIM-001, FIT-004, REF-002 | `PACK-avatar-assets` | authoring, solver, files, desktop, automation, verification |
| 111 | `PACK-avatar-joints` — Reset avatar joints | ANIM-001, FIT-001 | `PACK-avatar-assets` | authoring, solver, files, desktop, automation, verification |
| 112 | `PACK-avatar-joint-display` — Display avatar joints | ANIM-001, MAT-003 | `PACK-avatar-assets` | authoring, solver, files, desktop, automation, verification |
| 113 | `PACK-animation-interchange` — Generic animation import/export | FILE-007, ANIM-001, ANIM-002 | `PACK-joint-motion-assets`, `PACK-simulate` | authoring, solver, files, desktop, automation, verification |
| 114 | `PACK-obj-sequence` — Export OBJ animation sequence | FILE-007, ANIM-002, ANIM-003 | `PACK-joint-motion-assets`, `PACK-simulate` | authoring, solver, files, desktop, automation, verification |
| 115 | `PACK-motion-playback` — Play avatar motion | ANIM-001, ANIM-002, SIM-003 | `PACK-joint-motion-assets`, `PACK-simulate` | authoring, solver, files, desktop, automation, verification |
| 116 | `PACK-animation-editor` — Animation timeline/editor | ANIM-002, ANIM-001, FILE-001 | `PACK-joint-motion-assets`, `PACK-simulate` | authoring, solver, files, desktop, automation, verification |
| 117 | `PACK-mesh-topology-conversion` — Quadrangulate or triangulate pattern output | PAT-011, ANIM-003 | `PACK-split-boundary`, `PACK-obj-interchange` | authoring, geometry, files, desktop, automation, verification |
| 118 | `PACK-flip-normals` — Flip garment normals | ANIM-003, FILE-003 | `PACK-split-boundary`, `PACK-obj-interchange` | authoring, geometry, files, desktop, automation, verification |
| 119 | `PACK-retopology` — Create/edit/patch topology | ANIM-003, PAT-011 | `PACK-split-boundary`, `PACK-obj-interchange` | authoring, geometry, files, desktop, automation, verification |
| 120 | `PACK-uv-editor` — UV editor | ANIM-003, MAT-002 | `PACK-split-boundary`, `PACK-obj-interchange` | authoring, geometry, files, desktop, automation, verification |
| 121 | `PACK-surface-texture-export` — Save surface textures | MAT-002, FILE-001, ANIM-003 | `PACK-point-pattern-edit` | authoring, files, desktop, automation, verification |
| 122 | `PACK-textures` — Edit/refresh/delete/flip 2D or 3D textures | MAT-002, FILE-001 | `PACK-point-pattern-edit` | authoring, files, desktop, automation, verification |
| 123 | `PACK-graphics` — Create/transform 2D or 3D graphics and browse graphic objects | MAT-002, FILE-001 | `PACK-point-pattern-edit` | authoring, files, desktop, automation, verification |
| 124 | `PACK-pattern-render-style` — 2D front/back texture, monochrome, translucency, transparency or random color | MAT-002, MAT-003, UI-002 | `PACK-point-pattern-edit` | authoring, files, desktop, automation, verification |
| 125 | `PACK-backgrounds` — Format 3D background or add reference image | MAT-003, PAT-007, FILE-001 | `PACK-point-pattern-edit` | authoring, files, desktop, automation, verification |
| 126 | `PACK-gltf-interchange` — glTF/GLB import and export | FILE-004, ANIM-001, MAT-002 | `PACK-obj-interchange`, `PACK-textures` | authoring, geometry, files, solver, desktop, automation, verification |
| 127 | `PACK-collada-interchange` — OpenCOLLADA import and add | FIT-001, ANIM-001, FILE-007 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 128 | `PACK-fbx-interchange` — FBX import/add/export | FIT-001, ANIM-001, FILE-007 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 129 | `PACK-alembic-interchange` — Alembic import/add/export | FILE-007, ANIM-001, ANIM-003 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 130 | `PACK-usd-interchange` — USD import/add/export and layer editor | FILE-007, MAT-001, ANIM-001 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 131 | `PACK-dna-interchange` — DNA import and add | FIT-001, ANIM-001, REF-002 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 132 | `PACK-vrm-interchange` — VRM import and export | FIT-001, ANIM-001, FILE-007 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 133 | `PACK-mc-cache` — Maya Cache MC import/export | FILE-007, ANIM-002 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 134 | `PACK-mcx-cache` — Maya Cache MCX import/export | FILE-007, ANIM-002 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 135 | `PACK-pc2-cache` — Point Cache 2 import/export | FILE-007, ANIM-002 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 136 | `PACK-mdd-standard-cache` — MDD Standard import/export | FILE-007, ANIM-002 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 137 | `PACK-mdd-dcc-cache` — MDD Maya/3ds Max import/export | FILE-007, ANIM-002 | `PACK-obj-interchange`, `PACK-joint-motion-assets` | authoring, geometry, files, solver, desktop, automation, verification |
| 138 | `PACK-pattern-snapshot-print` — Physical-scale 2D snapshot/printing | FILE-005 | `PACK-rectangle` | geometry, files, desktop, automation, verification |
| 139 | `PACK-camera-assets` — Load/save projection and camera transform presets | MAT-003, FILE-001 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 140 | `PACK-viewport-snapshot` — Capture the 3D viewport | MAT-003 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 141 | `PACK-wireframe` — Inspect garment or 2D mesh wireframe | MAT-003, ANIM-003 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 142 | `PACK-video-capture` — Animation video capture | MAT-003, ANIM-002, FILE-007 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 143 | `PACK-quality-render` — Quality viewport render | MAT-003, MAT-002 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 144 | `PACK-schematic-render` — Schematic render | MAT-003, SEW-006 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 145 | `PACK-scene-light-display` — Display shadows, lights or cameras | MAT-003 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 146 | `PACK-avatar-render-style` — Avatar textured/monochrome/translucent styles | MAT-002, MAT-003, FIT-001 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 147 | `PACK-garment-render-style` — Garment thickness/texture/mono/translucent/random styles | MAT-002, MAT-003, ANIM-003 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 148 | `PACK-fit-maps` — Stress, strain or pressure maps and fit properties | MAT-003, SIM-005, FIT-003 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 149 | `PACK-camera-properties` — Edit main camera properties | MAT-003, FILE-001 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 150 | `PACK-graphic-preferences` — Antialiasing, blending, transparency and image render limits | MAT-003, MAT-002, UI-006 | `PACK-textures`, `PACK-simulate` | geometry, solver, desktop, files, verification |
| 151 | `PACK-visibility-isolation` — Hide/show/isolate garments or avatars | UI-002, FIT-002, FIT-001 | `PACK-selection` | desktop, authoring, files, verification |
| 152 | `PACK-locks` — Lock outline, internal line, guideline or remesh-pattern editing | UI-002, PAT-007, ANIM-003, FILE-001 | `PACK-selection` | desktop, authoring, files, verification |
| 153 | `PACK-viewpoints` — 3D camera viewpoints, focus and extents | UI-001, MAT-003 | `PACK-selection` | desktop, authoring, files, verification |
| 154 | `PACK-pattern-name-display` — Display pattern names in 2D/3D | UI-001, PAT-007 | `PACK-selection` | desktop, authoring, files, verification |
| 155 | `PACK-pattern-view-extents` — Frame all 2D patterns | UI-001 | `PACK-selection` | desktop, authoring, files, verification |
| 156 | `PACK-picking-point-display` — Display picking point | UI-002, UI-003 | `PACK-selection` | desktop, authoring, files, verification |
| 157 | `PACK-grid-display` — Display 2D/3D grids | UI-001, UI-003 | `PACK-selection` | desktop, authoring, files, verification |
| 158 | `PACK-grid-properties` — Grid distance/zoom adjustment, arrow increments and 3D brightness | UI-001, UI-003, PAT-004 | `PACK-selection` | desktop, authoring, files, verification |
| 159 | `PACK-workspace-panels` — Connected 2D/3D workspace, scene hierarchy and property panels | UI-001, UI-002, UI-006 | `PACK-selection` | desktop, authoring, files, verification |
| 160 | `PACK-workspace-layout` — Apply/reset/export/browse workspace layouts | UI-001, UI-006, APP-002 | `PACK-selection` | desktop, authoring, files, verification |
| 161 | `PACK-toolset-assets` — Load/save tool sets and customize tool groups/order | UI-003, FILE-001 | `PACK-workspace-panels` | desktop, files, automation, verification |
| 162 | `PACK-preference-assets` — Open/save preferences and the preferences dialog | UI-003, UI-006, APP-003 | `PACK-workspace-panels` | desktop, files, automation, verification |
| 163 | `PACK-languages` — Application language choices | UI-006, APP-002 | `PACK-workspace-panels` | desktop, files, automation, verification |
| 164 | `PACK-help-resources` — Manual languages, tutorials, FAQ, Q&A, contact, release notes and news | UI-003, APP-002, APP-005 | `PACK-workspace-panels` | desktop, files, automation, verification |
| 165 | `PACK-share-packages` — Share project, garment or avatar content | FILE-001, APP-003, REF-002 | `PACK-avatar-assets`, `PACK-point-pattern-edit` | authoring, geometry, solver, files, desktop, automation, verification |
| 166 | `PACK-pattern-drafter` — Pattern drafter | PAT-003, PAT-010, APP-003, REF-002 | `PACK-avatar-assets`, `PACK-point-pattern-edit` | authoring, geometry, solver, files, desktop, automation, verification |
| 167 | `PACK-motion-conversion` — Auto convert an avatar to motion | ANIM-001, APP-003, REF-002 | `PACK-avatar-assets`, `PACK-point-pattern-edit` | authoring, geometry, solver, files, desktop, automation, verification |
| 168 | `PACK-avatar-conversion` — Auto convert geometry to avatar | FIT-001, ANIM-001, APP-003, REF-002 | `PACK-avatar-assets`, `PACK-point-pattern-edit` | authoring, geometry, solver, files, desktop, automation, verification |
| 169 | `PACK-ai-pose` — AI pose generation | ANIM-001, APP-003, REF-002 | `PACK-avatar-assets`, `PACK-point-pattern-edit` | authoring, geometry, solver, files, desktop, automation, verification |
| 170 | `PACK-everywear` — EveryWear creation/preparation workflow | ANIM-003, FILE-007, APP-003, REF-002 | `PACK-avatar-assets`, `PACK-point-pattern-edit` | authoring, geometry, solver, files, desktop, automation, verification |
| 171 | `PACK-hosted-content-discovery` — Explore EveryWear or GameWear content | MAT-001, ANIM-003, APP-003, REF-002 | `PACK-avatar-assets`, `PACK-point-pattern-edit` | authoring, geometry, solver, files, desktop, automation, verification |
| 172 | `PACK-local-libraries` — Asset and modular libraries | MAT-001, PAT-010, SEW-005, APP-003, FILE-001 | `PACK-avatar-assets`, `PACK-point-pattern-edit` | authoring, geometry, solver, files, desktop, automation, verification |
| 173 | `PACK-ai-help` — AI help/chat assistant | UI-003, APP-003, REF-002 | `PACK-avatar-assets`, `PACK-point-pattern-edit` | authoring, geometry, solver, files, desktop, automation, verification |
| 174 | `PACK-automation-scripts` — Script/Python import, editor and API reference | APP-001, APP-003, UI-003 | `PACK-segment-sewing`, `PACK-obj-interchange` | authoring, files, desktop, automation, verification |
| 175 | `PACK-plugin-manager` — Manage local plug-ins | APP-001, APP-003 | `PACK-segment-sewing`, `PACK-obj-interchange` | authoring, files, desktop, automation, verification |
| 176 | `PACK-livesync` — LiveSync integration discovery | APP-001, FILE-007, ANIM-003, REF-002 | `PACK-segment-sewing`, `PACK-obj-interchange` | authoring, files, desktop, automation, verification |
| 177 | `PACK-goz` — GoZ mesh-transfer integration | FILE-003, APP-001, ANIM-003, REF-002 | `PACK-segment-sewing`, `PACK-obj-interchange` | authoring, files, desktop, automation, verification |
| 178 | `PACK-section-labels` — Selection-dependent context/layout section labels | UI-001, REF-002 | — | reference, verification |
| 179 | `PACK-vendor-session-administration` — Vendor activation/session administration only | APP-003, REF-002 | — | reference, verification |
| 180 | `PACK-catalog-containers` — Menu containers and currently empty dynamic submenus | UI-001, UI-003, REF-002 | — | reference, verification |
| 181 | `PACK-workspace-statistics` — Display statistics, viewport size or autosave countdown | PERF-001, UI-001, FILE-002 | `PACK-simulate`, `PACK-resolution` | solver, geometry, desktop, files, automation, verification |
| 182 | `PACK-application-about` — About application | APP-002, APP-003 | `PACK-application-quit`, `PACK-workspace-panels` | desktop, files, verification |
| 183 | `PACK-os-services` — Native OS Services and diagnostic menu routes | APP-002, UI-006, REF-002 | `PACK-application-quit`, `PACK-workspace-panels` | desktop, files, verification |
| 184 | `PACK-application-window-lifecycle` — Hide app/others or show all windows | APP-002, UI-001, UI-006 | `PACK-application-quit`, `PACK-workspace-panels` | desktop, files, verification |
| 185 | `PACK-pattern-name-property` — Selected pattern name property | PAT-007, PAT-001, UI-004, FILE-001 | `PACK-point-pattern-edit` | authoring, geometry, solver, files, desktop, automation, verification |
| 186 | `PACK-selected-line-lengths` — Selected-line 2D and 3D length properties | PAT-007, FIT-003, UI-002 | `PACK-point-pattern-edit`, `PACK-simulate` | authoring, geometry, solver, files, desktop, automation, verification |
| 187 | `PACK-edge-elastic` — Selected-line elastic construction | SEW-005, SIM-002, PAT-007, FILE-001 | `PACK-segment-sewing`, `PACK-internal-primitives` | authoring, geometry, solver, files, desktop, automation, verification |
| 188 | `PACK-schematic-line-properties` — Selected schematic-line visibility, face and Z-offset | MAT-003, PAT-007, FILE-001 | `PACK-point-pattern-edit`, `PACK-schematic-render` | authoring, geometry, solver, files, desktop, automation, verification |
| 189 | `PACK-pattern-layer-property` — Selected pattern physical layer property | FIT-002, SIM-004, SIM-005, FILE-001 | `PACK-superimpose-layer-clone`, `PACK-simulate` | authoring, geometry, solver, files, desktop, automation, verification |
| 190 | `PACK-directional-shrinkage` — Selected pattern warp/weft simulation shrinkage | SIM-002, PAT-008, FIT-003, FILE-001 | `PACK-fabric-assignment`, `PACK-grain-alignment` | authoring, geometry, solver, files, desktop, automation, verification |
| 191 | `PACK-pattern-collision-thickness` — Selected pattern additional collision thickness | SIM-002, SIM-003, SIM-004, FIT-003, FILE-001 | `PACK-simulate`, `PACK-fabric-assignment` | authoring, geometry, solver, files, desktop, automation, verification |
| 192 | `PACK-pattern-pressure` — Selected pattern pressure property | SIM-002, SIM-005, FIT-003, FILE-001 | `PACK-simulate` | authoring, geometry, solver, files, desktop, automation, verification |
| 193 | `PACK-pattern-render-geometry` — Selected rendering thickness, extrusion and side geometry | ANIM-003, MAT-003, MAT-002, FILE-003, FILE-001 | `PACK-retopology`, `PACK-obj-interchange` | authoring, geometry, solver, files, desktop, automation, verification |

## Crosswalk review and remaining discovery

Each JSON group identifies the actual source landmarks and shared commands, source-backed candidate behavior, missing functionality, exact runtime questions, original fixture recipes and its bounded packet. All 755 JSON entries retain exact catalog IDs and paths and have one explicit action/component/container/heading/excluded classification. Source landmarks name actual Rust anchors; proposed future behavior is described as missing rather than given imaginary implementation locations.

The catalogs combine an empty-workspace menu snapshot with component presence/read-only values and selected original-pattern/constructor/OBJ/Fit observations. Actual canvas context menus, unlabeled icons, toolbar flyouts, selected-state properties, accepted ranges/defaults/errors, populated dynamic menus, native format semantics and complete workflows still require observation. Required discovery leads in JSON retain AI Studio creation, native editable compatibility, uncataloged SVG/DXF/printing, Bond/Skive creation, complete physical/GPU/performance contracts and cross-platform/offline release gates.

The existing 56 coarse feature contracts remain authoritative and discovery is incomplete. Specialized production formats or newly observed behaviors may need reviewed new feature contracts; this mapping does not quietly redefine those 56 contracts or turn 755 menu/component records into a new denominator.

## Reproducible self-check

```sh
python3 spec/inventory/verify_implementation_map.py
```

The dependency-free check verifies exact 755-ID coverage and uniqueness, preservation of the previous 650 entries and packets, catalog/path/classification fidelity, pinned input hashes, valid feature/source/fixture/owner links, packet/group/dependency integrity and an acyclic dependency graph. It rejects copied status fields. It validates mapping structure, not reference behavior. Full Rust gates are for integrated source changes; this artifact-only mapping does not change the application.

## Input pins

| Catalog | Records | SHA-256 |
|---|---:|---|
| [md-2026.1.237-main-menus.json](md-2026.1.237-main-menus.json) | 589 | `0d0ebb92cac1d64a717249dc9f10fbb4ac7256b447442d99c3eb88a5314dece9` |
| [md-2026.1.237-components.json](md-2026.1.237-components.json) | 166 | `ca8c6f222757f47cc96daeb2e2244374e9b59d636cd2e5fe122ec22875cf11ac` |

Pinned registry SHA-256: `cb800867209e5df8fa1dbee58d39582a29bf8164f2baa716f920c44d8030f0a7`. The hash records the mapping review input; current feature status is always read from the authoritative registry, including later revisions. The retained catalog commit is the source pin and an ancestor of this map update; integration must preserve that history.

Previous reviewed map: `3fe36c5faccec7489ee54f97a70fcda382f57767`, SHA-256 `44899611b7687a4dd33edd107ef47f3d2924b70781207129e6f45a761caf5db5`. Its 650 entry records, 184 packets and first three bounded slices are preserved exactly; the checker rejects regressions to those records as well as missing or duplicate new IDs.
