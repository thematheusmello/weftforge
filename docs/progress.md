# WeftForge progress

Generated from [features.yaml](https://github.com/thematheusmello/weftforge/blob/main/parity/features.yaml); do not edit this report.

Reference: **MD-2026.1.237.58920-macos-arm64-20261113**, baseline revision 1, state **provisional**.

Discovery coverage: **incomplete**. 56 feature contracts inventoried; menu, dialog, shortcut, default, error and workflow discovery is unfinished. The whole product denominator remains unknown.

Implementation: 0/56 implemented; 36/56 partial; 0/56 in progress; 20/56 planned.

Verified reference parity: **0/56** inventoried contracts. Implementation-only checks do not count as reference parity.

Verification states: passed 0/56, failed 3/56, blocked 2/56, stale 0/56, not_run 51/56.

Reference observation: observed 0/56, documentation_only 5/56, unobserved 51/56, blocked 0/56.

Evidence freshness: current 19/56, stale 7/56, missing 30/56.

## Per-domain status

| Domain | Inventoried | Implemented | Partial | Verified parity |
|---|---:|---:|---:|---:|
| ANIM | 3 | 0 | 0 | 0 |
| APP | 5 | 0 | 5 | 0 |
| FILE | 7 | 0 | 4 | 0 |
| FIT | 4 | 0 | 2 | 0 |
| MAT | 3 | 0 | 2 | 0 |
| PAT | 11 | 0 | 6 | 0 |
| PERF | 1 | 0 | 0 | 0 |
| REF | 3 | 0 | 3 | 0 |
| SEW | 6 | 0 | 3 | 0 |
| SIM | 7 | 0 | 6 | 0 |
| UI | 6 | 0 | 5 | 0 |

## Contracts

| ID | Capability | Implementation | Verification | Freshness | Reference | Evidence / issue |
|---|---|---|---|---|---|---|
| UI-001 | Connected 2D and 3D workspace | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/2) |
| UI-002 | Coherent selection | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/2) |
| UI-003 | Shortcuts and command search | partial | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/2) |
| UI-004 | Transactions and history | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/2) |
| UI-005 | Cancellable background jobs | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/2) |
| UI-006 | Accessibility and high DPI | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/2) |
| PAT-001 | Editable polygons and straight boundaries | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| PAT-002 | Cubic curve authoring | partial | not_run | stale | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/3) · [issue](https://github.com/thematheusmello/weftforge/issues/9) |
| PAT-003 | Rectangles and primitives | partial | not_run | current | documentation_only | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| PAT-004 | Pattern transforms | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| PAT-005 | Split and join boundaries | partial | not_run | stale | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| PAT-006 | Cut pieces and holes | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| PAT-007 | Internal features and measurements | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| PAT-008 | Grain direction | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| PAT-009 | Seam allowances and offsets | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| PAT-010 | Darts pleats and linked symmetry | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| PAT-011 | Resolution and triangulation | partial | not_run | current | documentation_only | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| SEW-001 | Oriented interval sewing | partial | not_run | current | documentation_only | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/3) · [issue](https://github.com/thematheusmello/weftforge/issues/9) |
| SEW-002 | Free chain and internal sewing | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| SEW-003 | Easing and gathering | partial | not_run | stale | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/3) · [issue](https://github.com/thematheusmello/weftforge/issues/9) |
| SEW-004 | Seam diagnostics and editing | partial | not_run | missing | documentation_only | [issue](https://github.com/thematheusmello/weftforge/issues/3) · [issue](https://github.com/thematheusmello/weftforge/issues/9) |
| SEW-005 | Construction attachments | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| SEW-006 | Topstitching and export welding | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/3) |
| FIT-001 | Avatar import and arrangement | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/2) |
| FIT-002 | Pins grabbing and layers | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/2) |
| FIT-003 | Edit and regenerate garment | partial | failed | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-RELAXED-FIT-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-TIGHT-FIT-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/2) · [issue](https://github.com/thematheusmello/weftforge/issues/8) |
| FIT-004 | Drawing on avatar and preserving shape | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/2) |
| SIM-001 | Documented cloth solver | partial | failed | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-RELAXED-FIT-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-TIGHT-FIT-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/4) · [issue](https://github.com/thematheusmello/weftforge/issues/8) |
| SIM-002 | Physical fabric parameters | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-RELAXED-FIT-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/4) |
| SIM-003 | Avatar and ground contact | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-RELAXED-FIT-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/4) |
| SIM-004 | Self and layered contact | partial | not_run | stale | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/4) |
| SIM-005 | Simulation control and diagnostics | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-RELAXED-FIT-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/4) · [issue](https://github.com/thematheusmello/weftforge/issues/8) |
| SIM-006 | Convergence and calibration | partial | failed | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-RELAXED-FIT-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-TIGHT-FIT-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/4) · [issue](https://github.com/thematheusmello/weftforge/issues/8) |
| SIM-007 | GPU acceleration | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/4) |
| MAT-001 | Material assignment and presets | partial | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/5) |
| MAT-002 | Appearance texture and PBR | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/5) |
| MAT-003 | Lighting capture and inspection | partial | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/5) |
| ANIM-001 | Animated avatars | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/5) |
| ANIM-002 | Timeline and cache invalidation | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/5) |
| ANIM-003 | Output preparation | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/5) |
| FILE-001 | Editable native projects | partial | not_run | current | documentation_only | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/6) |
| FILE-002 | Atomic save and recovery | partial | not_run | stale | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/6) |
| FILE-003 | OBJ interchange | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/6) |
| FILE-004 | glTF and GLB interchange | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/6) |
| FILE-005 | Patterns SVG DXF and printing | partial | not_run | current | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json) · [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/6) |
| FILE-006 | Marvelous native compatibility | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/6) |
| FILE-007 | Animation and cache export | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/6) |
| APP-001 | Shared command engine and adapters | partial | not_run | stale | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/7) |
| APP-002 | Cross-platform packaging | partial | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/7) |
| APP-003 | Offline operation and licensing | partial | not_run | stale | unobserved | [run](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json) · [issue](https://github.com/thematheusmello/weftforge/issues/7) |
| APP-004 | Recovery and responsiveness | partial | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/7) |
| APP-005 | Public delivery and evidence | partial | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/7) |
| PERF-001 | Representative performance | planned | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/7) |
| REF-001 | Frozen legitimate baseline | partial | blocked | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/1) |
| REF-002 | Complete discovery inventory | partial | blocked | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/1) |
| REF-003 | Controlled comparisons | partial | not_run | missing | unobserved | [issue](https://github.com/thematheusmello/weftforge/issues/1) |

## Current gaps and next delivery

Complete identified implementation workflow: **passed**, [WF-20261113-SINGLE-CANDIDATE-001](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json), tested commit `ff3914474c8acc30d10c70a8489cae56cfa2c872`. This bounded replay does not establish reference parity or complete the remaining product gates.

Continue professional editing and sewing with relationship-preserving changes; retain tight-garment failures and investigate convergence/contact. Reference workflow acceptance remains required.

Reference access is held at user-reviewed Terms of Service and trial activation. See [STATUS.md](https://github.com/thematheusmello/weftforge/blob/main/STATUS.md), [ROADMAP.md](https://github.com/thematheusmello/weftforge/blob/main/ROADMAP.md), [workflow runs](https://github.com/thematheusmello/weftforge/actions), and [releases](https://github.com/thematheusmello/weftforge/releases).

Known solver limits: discrete vertex/triangle contact has no full edge/edge continuous contact, contact search is quadratic, and fabric approximations are uncalibrated. Cross-platform launch, animation, advanced authoring and native commercial-project compatibility remain open.

- **UI-001**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **UI-002**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **UI-003**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **UI-004**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **UI-005**: Full reference contract and runtime comparison not yet established.; Meshing and simulation are revision-tagged background jobs; solver initialization checks cancellation only before/after initial contact analysis; command validation and file IO retain synchronous portions.; User must review reference app terms and activate legitimate trial/license.
- **UI-006**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **PAT-001**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **PAT-002**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **PAT-003**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **PAT-004**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **PAT-005**: Full reference contract and runtime comparison not yet established.; Sewn-curve splitting explicitly rejected; curve joining unsupported.; User must review reference app terms and activate legitimate trial/license.
- **PAT-006**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **PAT-007**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **PAT-008**: Full reference contract and runtime comparison not yet established.; Solver supports grain but editable document has no grain parameter yet.; User must review reference app terms and activate legitimate trial/license.
- **PAT-009**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **PAT-010**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **PAT-011**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **SEW-001**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.; Public partial-sewing API uses normalized boundary length; current document intervals and default correspondence use curve parameter t. Arc-length equivalence remains unverified.
- **SEW-002**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **SEW-003**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **SEW-004**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **SEW-005**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **SEW-006**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **FIT-001**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **FIT-002**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **FIT-003**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **FIT-004**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **SIM-001**: Full reference contract and runtime comparison not yet established.; Original tight-shirt drape finishes after assembly/mesh fixes, but fitting-quality strain and seam residual fail declared criteria; preserve the scene.; User must review reference app terms and activate legitimate trial/license.
- **SIM-002**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **SIM-003**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **SIM-004**: Full reference contract and runtime comparison not yet established.; Vertex/triangle discrete contact; edge/edge and full continuous contact missing; layered shirt+jacket acceptance unverified.; User must review reference app terms and activate legitimate trial/license.
- **SIM-005**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **SIM-006**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **SIM-007**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **MAT-001**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **MAT-002**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **MAT-003**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **ANIM-001**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **ANIM-002**: Full reference contract and runtime comparison not yet established.; No timeline, cache persistence or animation implemented.; User must review reference app terms and activate legitimate trial/license.
- **ANIM-003**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **FILE-001**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **FILE-002**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **FILE-003**: Full reference contract and runtime comparison not yet established.; Current export has geometry/UV/normals/piece groups; avatar import and material assignments missing.; User must review reference app terms and activate legitimate trial/license.
- **FILE-004**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **FILE-005**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **FILE-006**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **FILE-007**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **APP-001**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **APP-002**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **APP-003**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **APP-004**: Full reference contract and runtime comparison not yet established.; Unsaved-work autosave and explicit recovery implemented/tested; crash/restart GUI and representative large-scene latency still require evidence.; User must review reference app terms and activate legitimate trial/license.
- **APP-005**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.; GitHub Project board creation rejected: authenticated token lacks project and read:project scopes. Issues work independently.
- **PERF-001**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **REF-001**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **REF-002**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.
- **REF-003**: Full reference contract and runtime comparison not yet established.; User must review reference app terms and activate legitimate trial/license.

## Latest attributable runs

- [WF-20261113-SINGLE-CANDIDATE-001](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-SINGLE-CANDIDATE-001.json): implementation_only, passed, **current**, commit `ff3914474c8acc30d10c70a8489cae56cfa2c872`, macOS 27.2 (26B123), aarch64, Apple M3 Max, Rust 1.94.1, 2026-11-13T22:03:39Z.
- [WF-20261113-TIGHT-FIT-001](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-TIGHT-FIT-001.json): implementation_only, failed, **current**, commit `ff3914474c8acc30d10c70a8489cae56cfa2c872`, macOS 27.2 (26B123), aarch64, Apple M3 Max, Rust 1.94.1, 2026-11-13T21:39:11Z.
- [WF-20261113-RELAXED-FIT-001](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-RELAXED-FIT-001.json): implementation_only, passed, **current**, commit `ff3914474c8acc30d10c70a8489cae56cfa2c872`, macOS 27.2 (26B123), aarch64, Apple M3 Max, Rust 1.94.1, 2026-11-13T21:39:11Z.
- [WF-20261113-REGRESSION-001](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-REGRESSION-001.json): implementation_only, passed, **stale**, commit `ff3914474c8acc30d10c70a8489cae56cfa2c872`, macOS 27.2 (26B123), aarch64, Apple M3 Max, Rust 1.94.1, 2026-11-13T21:39:11Z. Progress validation and its tests changed after this aggregate regression run. Original checks, source identities and artifacts are retained; scoped numerical and native workflow reports are unaffected.
- [WF-20261113-NATIVE-UI-001](https://github.com/thematheusmello/weftforge/blob/main/parity/reports/WF-20261113-NATIVE-UI-001.json): implementation_only, passed, **current**, commit `ff3914474c8acc30d10c70a8489cae56cfa2c872`, macOS 27.2 (26B123), aarch64, Apple M3 Max, Rust 1.94.1, 2026-11-13T21:39:11Z.
