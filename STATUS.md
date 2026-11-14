# WeftForge status — active development, 2026-11-14

The complete independent native garment studio and recorded one-to-one parity
remain the goal. Verified parity is **0/56** coarse contracts; discovery is
incomplete and no roadmap gate is accepted. The 949 observed interface entries
are an engineering inventory, not a feature denominator.

## Published state and cost policy

Public repository: https://github.com/thematheusmello/weftforge.
Last merged main: `16e680ad774119e1ce3609b7d6c42c317f8677ef`,
[native reversal PR31](https://github.com/thematheusmello/weftforge/pull/31), head
`94a3da64584bdb61884bd8a8052353b853fd8d12`. Its exact evidence checkpoint is
`e75786ba3e1b259503f00a25f6034427a37edb96`.

**GitHub Actions stays disabled until the full project is complete.** The user
requested this to stop development costs. Repository `enabled:false` was verified
after the last push/merge; no branch workflows ran. Keep required checks local;
do not dispatch CI or Pages at intermediate checkpoints. The Pages site remains
at main `1343b0c46b5c4f097709996785fb55d5d23ac174`, its last successful deployment
43585595609 (CI43585595600). Generated progress can still ship in the repository.
The root README stays product focused, without competitor/trial/sign-in status.

The source-only [alpha.1](https://github.com/thematheusmello/weftforge/releases/tag/v0.1.0-alpha.1)
remains at older `8419363` and retains its inspector limitation. No native binaries
have been released. Distribution notices, signing and platform launch QA remain.

## Active DXF increment

Working branch: `feat/dxf-pattern-export`. Integrated source checkpoint:
`cd151b3b89e26d55a4dcd4525790006660c4e766`. The reviewed R2000 profile was added
at `c69e377`; core `d5d0938` and atomic UI fix `409d845` were merged with their
ancestry intact. Corrected integrated fmt,98 Rust tests and strict Clippy pass.

`Project::export_dxf` writes authored exterior curves directly as ASCII R2000
LINE/cubic SPLINE geometry in millimetres, preserving UV-to-XY orientation,
ordered closure and stable piece layers. It uses existing validation, protected
export paths and staged replacement. Desktop toolbar/command search, CLI
`export-dxf` and the sixth MCP tool use that API. No runtime dependency or new
authoring command was added. Import, ASTM/AAMA interchange, print layout, sewing,
materials, grain and 3D placements remain outside this documented subset.

At source5f: fmt, **96 Rust tests**, strict workspace Clippy and standard unsigned
packaging pass. CLI SHA-256 is
`e90ac57b9bd345ad1d844b4f3f9e684c96fa58af0f795d5f7a168ddfc3fad20f`.
That desktop executable was never launched: review found an inherited GUI bug
where a pending numeric draft committed before Cancel/path/write failure. The
merged correction stages an Editor clone, defers the chooser until inspector
input is processed, and retains focus/drafts/history on failure. Its55 actual
App::ui cases and23 app tests pass. A new corrected package is building; bind it
before native launch. Changed pending OBJ construction requires Tab, mesh wait
and retry; ready rest/drape export is preserved.

The frozen independent oracle is `.local/dxf-acceptance-20261114/evaluate.py`,
SHA `6756739bebef6f0753ad0bb86eed3ed0116b8d6d1dfaecf548b368c44a959ed6`;
freeze SHA `20fc2dc605374c5ee9b4056c3716e18156266515db2a82bbc02f99288f7e6560`.
Its criteria/fixtures preceded every output inspection. Five actual headless CLI
exports pass unchanged raw-tag, ordinary ezdxf1.4.5 read, audit and geometry checks:
zero raw/consumer/audit errors or fixes, seven LINEs/one exact cubic for the mixed
case, 125x250mm rectangle, valid empty drawing, stable groups and repeated bytes.
Cubic sample error is at most5.684341886080802e-14mm. Nonzero placements and changed
resolution leave geometry unchanged. The first interpreter invocation bypassed
the venv and failed before evaluation; that attempt is retained separately.

Official Python MCP1.28.1/legacy2025-11-25 lists six tools and exports revision21
with correct mm/UV-XY/boundary-only metadata. Its bytes equal the accepted CLI
DXF (`ca94b0235ba944c0879b51b260b600ee3a10c6bd539112b213f6b59f9a719ca7`).
Source replacement, malformed JSON and future versions reject without changing
protected inputs/sentinels; server stderr is empty. This uses the pre-UI-fix CLI,
not a final/native comparison. Preserve `.local/dxf-producer-20261114/`,
`.local/dxf-sdk-20261114/` and the oracle's original inputs/expectations.

## Existing workflows and evidence

The core retains stable editable points/straight and cubic boundaries, oriented
partial/reversed/nonlinear sewing, grain and separate warp/weft stiffness, typed
transactions, undo/redo, relationship-preserving split and bounded inverse Join.
The native Sewing inspector has per-seam Reverse and endpoint guides. General
cut/trim/internal sewing, attachments and arc-length semantics remain open.

The last executed native reversal source is `9d30a71`, executable
`afcb154d5b61382d4672ddedf2c26f49b7752af878a086b913c95aa6aa371c70`, and old CLI
`ca49402e497f71a2785bd96a6ef936cb320ffef856acb682824fe66e108fa25a`.
Its nine snapshots, exact nonlinear interval proof, focused history, real process
restart/reopen, rejected revision-limit edit and OBJ/SVG subset passed. All364
unique report artifacts were tracked/hash-valid and replayed from a clean
checkout at that checkpoint. That report is now **stale** after DXF source
changes; its frozen declaration/evaluator/artifacts are unchanged. Earlier
numeric precision failures and corrected native I/O/join workflows remain in
[the reports](parity/reports), with their original identities and statuses.

Native v2 stores supported editable construction and strictly migrates v1.
Atomic saves retain prior recovery data; export guards reject native destinations,
protected aliases and nonregular files. On macOS, focused Quit recovery passed on
an earlier identified candidate. Concurrent writers, forced termination, broad
asset recovery and complete persistence remain unverified. Live-GUI MCP, modern
external clients, simulation jobs and persistent MCP undo remain open.

## Cloth quality and performance

The f64 CPU XPBD path includes orthotropic membrane strain, dihedral bending,
area-based mass, damping/friction, bounded-speed assembly and analytic
avatar/ground plus discrete self/layer contact. GPU cloth, CCD, robust layered
outfits, calibrated fabrics and general fitting remain incomplete.

[Issue16](https://github.com/thematheusmello/weftforge/issues/16) remains failed.
The new [bounded refinement report](parity/reports/WF-20261114-PHYSICS-REFINEMENT-001.json)
uses clean historical source16e and unchanged grain-shirt inputs/tolerances.
It reproduces26.9303% strain at3s; strain already reaches25.9692% during the1s
zero-gravity assembly. At4s, control8x32,8x64 and16x32 yield26.7718%,26.6893%
and26.2492%, all above15%. Refinement reduces seam error to about10nm but moves
nodes2.11/3.90mm. Equilibrium and a production fix are unproven. The exact driver,
recorded states and read-only verifier are retained; no solver/default was changed.

[Issue8](https://github.com/thematheusmello/weftforge/issues/8) retains tight-shirt
and refinement failures:0.96m circumference is below the1.131m torso. Original
fixtures and failure limits were not substituted. [Issue9](https://github.com/thematheusmello/weftforge/issues/9)
retains parameter-versus-arc-length sewing and physical correspondence gaps.

The measured BVH improvement at source461f2e2 applies only to initial static
contact diagnostics: median5.618584042→0.056252833s, same71,927 intersections and
0.00536994076963222m error. Live projection remains quadratic. Prior native Cancel
acknowledgement was890ms including CUA/AX overhead; worker-exit/phase latency is
unmeasured. See [the benchmark](docs/initial-contact-benchmark.md).

## Reference and source map

Reference Personal2026.1.237(r58920), executable `e7696388…c76e`, remains baseline
revision1/provisional. Selected units are millimetres; locale/language and wider
settings/behavior coverage still need freezing. Commercial/raw/account records
stay outside the repository. Last reference process29812 was no longer running
at the prior authoritative check; do not assume a current PID/window.

The crosswalk retains593 menu nodes+356 components,949 IDs,205 packets and56
landmarks. The DXF reanchor is merged at4dac7e1, pinned to corrected source
cd151b3 with the exact reviewed bool-to-ctx/format/focus signature change. Preserve identities, queued work and required
DXF scope. The16-child live Export-route inspection found no DXF item in that
state; it establishes neither product-wide absence nor an exclusion. Reference
rectangle, curved/partial sewing and categorical reversal contracts remain bounded
observations. No complete current reference contract qualifies as a parity pass.

## Exact next steps and job state

1. Finish corrected packaging and local verification; retain the original RED
   export case and the passing focused/integrated regressions.
2. Bind exact34-file source/package hashes to the frozen oracle before any native
   launch. Use the declared Cancel, pending-resolution success, command-search
   repeat and actual quit/relaunch/reopen case. Requalify the final CLI against
   existing headless/SDK bytes; never replace a native output with a generated one.
3. Retain the merged949-entry map, track every publishable artifact, and run
   registry/map/replay/privacy checks from a clean checkout. Publish/merge through
   the ordinary repository workflow with Actions and Pages still disabled.
4. Continue full assembly/contact diagnosis and the remaining construction,
   avatar/animation, rendering, interchange, printing and distribution gates.
   Completion is still the entire roadmap, not this DXF profile.

Root5f test97981/Clippy99078/package72251 and corrected test34987/Clippy89053
completed successfully. Corrected package session90261 is the current root job;
poll that exact handle before deciding its state. Physics39230/89137, SDK smoke,
UI-fix tests and map checks are terminal; no worker build is live.
The prior native app PID42700 was last left saved/idle at maximum revision with
no selection or simulation; inspect its current process/UI before retiring it.

Existing generated Cargo caches and the private installer depend on Expansion.
Keep compiled targets separate by worktree. Preserve source anchors7e59f2a,
9acb265,3548de4,13f1d81,8fb684e,df28375,461f2e2 and the later reviewed merge
ancestry; do not squash away source-pinned evidence. All remaining raw evidence
and job ledgers are retained in the ignored local experiment directories.
