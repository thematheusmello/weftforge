# WeftForge status — active development, 2026-11-14

The complete independent native garment studio remains the goal. Verified
reference parity is **0/56** coarse contracts; discovery is incomplete and no
roadmap gate is accepted. The 949 observed interface entries are an engineering
inventory, not a feature-completion denominator.

## Publication and cost policy

Public repository: https://github.com/thematheusmello/weftforge.
Last published main is `e6595ee787a30b3e8efa8502b42ce70536481d66`, merged
[PR35](https://github.com/thematheusmello/weftforge/pull/35). Its tree equals tested
final head `c8cdc3b0`; all37 runtime files match executed `6c45535c`.
Earlier [PR34](https://github.com/thematheusmello/weftforge/pull/34), main `2d65b9e`,
retains the original cloth balance failure, 132-file bundle and four
clean-publication receipts. Its tree equals tested final head `0130bf76`. Earlier
[PR33](https://github.com/thematheusmello/weftforge/pull/33), main `4a2c5568`,
delivered ordered M:N sewing and strict v3 projects with preserved ancestry.
[PR32](https://github.com/thematheusmello/weftforge/pull/32) delivered DXF.
Visibility delivery is merged. Clean final `c8cdc3b0` checks all1,686 report
artifacts and the unchanged37-file runtime. Publication audit is retained in
`.local/verification/published-e6595ee787a3.json`.

**Keep GitHub Actions and Pages disabled until the entire project is complete.**
The user requested this to stop development costs. Repository `enabled:false`
and all 54 historical runs terminal were rechecked after all three merges. PR33/34/35
final heads and merges each have zero workflow runs. Last CI/Pages runs
43585595600/43585595609 belong to main `1343b0c46b5c4f097709996785fb55d5d23ac174`.
Do not dispatch workflows at interim checkpoints. Required checks run locally;
generated progress ships in Git. The root README stays product focused.

The source-only [alpha.1](https://github.com/thematheusmello/weftforge/releases/tag/v0.1.0-alpha.1)
remains older `8419363`. No new native binaries are released. Notices, signing
and cross-platform launch/distribution acceptance remain open.

## Current material-strain increment

Working branch: `research/force-balance-audit`; published visibility integration
is `938b2670746fb97da70540912595637264b4bf1e`.
All 37 scoped runtime/test files exactly match executed candidate
`6c45535cb312f7fed8ad872ee87b6ba5e5974e35`. Prospective contract
`209a8fda72aaca90a08c330427354dd457224a6c` is
[SIM-005 strain attribution](spec/features/SIM-005-strain-attribution.md).

Read-only diagnostics now identify the solver triangle, warp/weft direction and
signed material-axis length strain. Exact ties choose the first input triangle,
then warp. The shared core maps the current mesh to its authored pattern; stale
or absent attribution cannot resolve a piece. Native labels show pattern and
signed percent. CLI simulation/benchmark and fitting phase/final reports expose
the same typed metric and pattern ID/name. Legacy diagnostics JSON remains
readable. Authoring commands, native project v3 and physical settings are unchanged.

At the exact candidate, fmt, **126 Rust tests**, strict workspace Clippy and
standard unsigned packaging pass (root gate99131, terminal0). Independent control
regression76935 is terminal0: all181 physical states, mesh, positions, velocities,
settings and prior diagnostics match the original exactly, excluding step time.
All180 meaningful peak associations agree independently within2.23e-16 strain.
The original quality failures remain failures.

The accepted native capture used PID99058 and executable
`95fa1b7cb24b8de1a6b507330c2fe43ece0efe5a14226bb28c39f922fcee28c3`.
It displays Candidate back / Warp +24.12% after actual60-frame assembly, then
Candidate front / Warp +32.35% after120-frame gravity drape. Root independently
viewed the original captures. Both150-vertex/228-triangle OBJ outputs have exact
expected topology/decimal exports; parsed position/UV error is at most6.94e-18m.
Stage authoring bytes are identical (revision28); the immutable v2 input is not
overwritten. Seam/contact labels are four-decimal display values, not raw zeros.
At capture closure the app was saved/paused at completed Drape, total3s.
It was subsequently quit for the separate empty-project workflow below.

Packaged CLI `c7e63b52d60b293357ba60c49dfed3643472ab177352f2501ebfb207f7ed1092`
passes the three-command checker2792: actual fit exits1 for quality and preserves
the prior output; simulate/benchmark exit0 with independently matching attribution.
The native47-file and independent88-file candidates are frozen. The complete
167-file bundle is committed and hash-valid, with no binary/target files.
[WF-20261114-STRAIN-ATTRIBUTION-001](parity/reports/WF-20261114-STRAIN-ATTRIBUTION-001.json)
registers bounded visibility acceptance. Root xtask7488 passes
54Python checks, registry/map and actual CLI help. Strict tracked-clean replay
passed at clean `f68a9865`: all167 files tracked, strict source/contract checks,
headless/native/CLI saved replays, registry/map and all1,681 report artifacts.
Five separate publication proof files bring this report to175 artifacts.

## Ordered sewing already delivered

Authoritative M:N groups pair cumulative authored arc lengths across partial,
reversed and disconnected spans, with nonlinear maps and stable group IDs. Gaps
add no length. Supported split/join/remesh preserve relationships; removal of a
participating piece removes its whole group. Legacy single-seam normalized-t
semantics remain. Native A/B drafts, accessible creation, structured editing,
atomic history, Reverse B and current-mesh guides are implemented. CLI
execute/inspect-sewing and six MCP file tools share the engine:22 typed variants
including Transaction,21 nontransaction MCP variants.

[WF-20261114-ORDERED-SEWING-001](parity/reports/WF-20261114-ORDERED-SEWING-001.json)
retains source3eddef acceptance:10 independent CLI cases,11 real Editor checkpoints,
13 qualified malformed-file preservation cases,6 official SDK calls and18 native
checkpoints including actual quit/relaunch/reopen. Authored portion/stitch errors
remain below1e-8m; weighted rest-chord approximation is separately qualified.
The original fine curved mesh failure is resolved at unchanged0.017m/caps:
51,906vertices/102,685triangles, maxedge0.02377649m below0.0238m. Original precision,
UI, driver and packaging failures remain retained. The850-file bundle replay and
54Python/map/xtask passed cleanly at a661ae4; final metadata/source/artifact checks
passed at081ea176 andab30318. This historical report is stale after6c455 scoped
source changes, not silently promoted to a new complete run.

## Cloth quality and remaining scope

[WF-20261114-PHYSICS-BALANCE-001](parity/reports/WF-20261114-PHYSICS-BALANCE-001.json)
retains original source3eddef control:24.1185% strain after1s assembly,32.3519%
after3s. The fixed extra zero-G branch ends at32.1699%, still evolving. Exact
frame150–180 strain rises30.7541%→32.1699%, with3.62048mm drift and0.00383853J
membrane/bending energy increase. All61 shared prefix states match. Omitting
only mannequin capsules gives0.242573% but is a causal diagnostic with quality
null, never fitting acceptance. All423 states/420 timings remain intact. Clean
checkout667d0caa passes saved replay/registry/map and1,508 then-current artifacts;
final0130bf76 checks1,512 artifacts. The historical report is stale after6c455;
its original control is independently repeated in the current visibility check.

[WF-20261114-PHYSICS-FULL-PATH-001](parity/reports/WF-20261114-PHYSICS-FULL-PATH-001.json)
retains older e342 refinement failures at3s:26.9303%,27.1686%,26.6987% for8x32,
8x64,16x32. Current150V/228T and historical158V/244T nodal arrays are not equivalent;
28 stitch mappings remain equal. No thresholds, original material or fixture
were substituted. Full force balance, equilibrium, convergence, calibration and
a solver-defect conclusion remain unproven. See [physics](docs/physics.md).

Full free/internal sewing, cut/trim, constraints, calibrated fabrics, robust
layered contact, avatars/animation, rendering/CJK fonts, imports/interchange,
physical-scale printing, native-project compatibility, broad recovery and
cross-platform distribution remain required. Native cubic precise editing,
failed IO/deep history and full metric cancellation/staleness/exhaustion remain
unverified. Keep the entire ROADMAP target.

## Reference and exact continuation

Baseline remains Personal2026.1.237(r58920), executable e7696388…c76e, provisional.
Commercial/raw/account evidence stays private. The guarded Rectangle attempt
created no constructor or geometry, so M:N reference proportions/history/files
remain unobserved. The one-click manual diagnosis question is pending. This is
input noncompletion, not absence; preserve the earlier Rectangle observation.
See [dated research notes](spec/features/SEW-002-runtime-notes-20261114.md).
Passive REA refresh at18:57:29–36Z confirms the same live process/window and
executable, Rectangle value1,95 nontruncated AX nodes and no visible constructor.
No closed pattern is visible; the isolated2Ddot is not interpreted as authored
geometry. No activation/input occurred and no manual-click outcome is inferred.
Private evidence is `ev_bc8e54db21a2003e2b5ac036a9a9fb173c20404528915a02d9288edb9a6b616a`;
reviewed facts SHA is `570833c3be8f2d3bfe595b7687c8f1e1bc67812e8f640fabf9697683d8d661a7`.

The original Harbor empty-project workflow is now active separately in
`.local/empty-shirt-authoring-20261114/`, frozen plan
`0f07ac196c8e285c1441dc3a16f0027f65c2b901b4630abfa180a216b56e1ddd`.
Same-binary fresh PID47128 starts with a verified v3 revision0 empty document.
Both14-vertex panels are created through actual numeric staging/Close pattern;
all28 frozen vertices/straight curves match independent saved-source parsing at
revision2. Checkpoint SHA is `e4828356cf37524c1106b2c8c12a6cef2dd813fd4e9f5930db9399a2a029b6f9`.
The first routed draft was rejected for a degenerate edge and left0pieces;
its correction reached the unchanged frozen design. Arrangement,grain,sewing,
save/reopen and planned height-edit history remain in progress. No simulation
is authorized for this workflow yet. It cannot replace the failed weak-grain
benchmark or qualify a complete garment gate.

1. Follow the approved single original-shirt diagnostic in
   `.local/force-balance-design-20261114/shirt-run-001/`. The private analytical
   overlay passed258 producer and321 independent checks, exact audited/ordinary
   state equality, all6 cancellation polls and invalid-settings retention.
   Independent receipt SHA is `bef9d0337b7508aea1cd542e8c81e6813180bda12da03d40664ddfea82f34074`.
   Keep base6c455 plus observational patch13f927… explicitly separate from
   production. Preregister the new driver/analyzer/binary; trace only frames
   24/27/60/180 and compare all181 states exactly with production6c455.
   Fixed caps remain420s/256MiB; no tuning,other scenes or blind extension.
   No production solver changes or tuning are authorized. Correction accounting
   alone cannot establish physical contact reactions or equilibrium.
2. Finish Harbor UI authoring/history/persistence under the frozen plan; root
   must qualify the authored state and fixed physical plan before simulation.
   The reference manual click question remains pending; do not inject further
   reference input meanwhile. All raw account/UI evidence remains private.
3. Continue native/reference construction and all remaining roadmap work.

All reported Rust builds, original three-case diagnostics, independent visibility
regression, CLI and native simulations/checks are terminal. The retained native
accepted capture application was closed; new PID47128 is authoring without a
simulation job. Unified evidence preparation
71767 and clean40401/36772/59030/47879 are terminal0. The private force-balance
agent's build/static25436 and analytical99416 are terminal0. Wrapper41625 and
children41947/41949/41950 are confirmed terminal; only the separately approved
original-shirt diagnostic is being prepared, with no execution handle reported
yet. Base37 manifest is
`c42d45e0c9bae69065a4b4f01bc5fcbc897499c0904b59b32a326c81ee23a3da`.
Reference read-only inventory36748 and REA observation64096 are terminal0. Recheck
actual handles/ledgers before resuming. Raw ledgers stay
ignored; use separate Cargo targets per worktree.
