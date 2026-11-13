# WeftForge

**Open garment design. Patterns, fabric, and freedom.**

WeftForge is a free, open-source native Rust garment studio in early development.
The target is one-to-one functional parity with a recorded Marvelous Designer
baseline across interaction, authoring, complete workflows, simulation/output,
files, and operation. **Verified reference parity: zero.** The reference app is
installed but garment access is blocked at CLO-SET sign-in. Current terms
acceptance and legitimate entitlement remain unverified; the baseline is provisional.

Public repository: [thematheusmello/weftforge](https://github.com/thematheusmello/weftforge).
See [current status](STATUS.md), [generated progress](docs/progress.md), [dashboard](docs/dashboard/index.html), the
[feature registry](parity/features.yaml), and the [complete roadmap](ROADMAP.md).
Implementation coverage and verified reference parity are tracked separately.
An original empty-project shirt workflow has been exercised through the actual
macOS UI, including sewing, arrangement, simulation, save/reopen, dimension edits,
undo/redo and independently measured OBJ/SVG export. This is an implementation-only
scene result on one identified executable; reference comparison and the
remaining professional integration gates remain open.

## Current increment

The source implements a native eframe/wgpu workspace with a 2D pattern canvas,
basic 3D garment preview, piece selection, numeric properties, command search,
and undo/redo. Rectangle/polygon construction feeds the same Rust document engine
as the CLI. The core retains straight/cubic boundaries, oriented seam intervals,
explicit seam correspondence, authored warp direction per piece, separate warp/weft
physical stiffness, placement and derived meshes.

An f64 CPU cloth solver provides membrane/bending constraints, sewn boundaries,
gravity, ground/analytic mannequin contact, and self-contact. Simulation runs in
cancelable background batches; material values are uncalibrated approximations.
This is an early solver, with contact robustness and reference drape agreement
still to establish.

Version2 `.weftforge` JSON stores supported editable construction and reads old
version1 projects through a strict lossless migration. Older apps cannot open v2. OBJ mesh
and SVG pattern export exist; [format details and gaps](docs/file-formats.md)
explain what survives each path. Simulation frames/settings, imported avatars,
textures, animation, advanced construction, and Marvelous Designer native project
compatibility are not yet complete. The 3D preview is basic shaded geometry;
professional rendering and GPU cloth simulation remain on the roadmap.

## Build and run

Install the Rust toolchain selected by `rust-toolchain.toml` (1.94.1). Cargo uses
pinned dependencies and the committed lockfile. The first build downloads crates;
core creation, simulation, save, and export run offline without an account or paid
service.

```bash
cargo run --release -p weftforge
cargo run --release -p weftforge-cli -- --help
```

Open a project by passing its path to the desktop binary. Start with an empty
document to exercise authoring; the CLI's `sample` command creates an original
two-panel implementation fixture and does not replace that acceptance workflow.

```bash
cargo run --release -p weftforge-cli -- new garment.weftforge
cargo run --release -p weftforge-cli -- rectangle garment.weftforge garment.weftforge Front -0.25 1.0 0.5 0.6
cargo run --release -p weftforge-cli -- inspect garment.weftforge
cargo run --release -p weftforge-cli -- export-svg garment.weftforge patterns.svg
cargo run --release -p weftforge-cli -- export-obj garment.weftforge garment.obj
```

CLI dimensions use metres, kilograms, seconds, radians, and Y-up 3D. The desktop
currently snaps canvas gestures to 1 cm. Fine numeric editing, modifier semantics,
and full reference interaction matching remain partial.

## Verify and contribute

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask verify
```

Check [STATUS.md](STATUS.md) for checks actually run and platform evidence. The
target platforms are macOS (including Apple Silicon), Windows, and Linux;
source portability is not proof that every platform has been built or launched.
The CLI `benchmark PROJECT STEPS` command reports implementation measurements;
it does not evaluate reference equivalence.

Keep core/solver code independent of the GUI. Make bounded integrated changes
linked to stable parity IDs, retain failures and unknowns, and run relevant checks.
Read [AGENTS.md](AGENTS.md), [architecture](docs/architecture.md),
[physics](docs/physics.md), and the [research method](docs/research-method.md).
Use original or appropriately licensed assets. Keep commercial installers,
proprietary code/assets, account data, and raw restricted evidence outside this
repository.

Original WeftForge code is licensed **MIT OR Apache-2.0**. Dependency and asset
licenses retain their own terms; see the repository notices. WeftForge is an
independent project and is not affiliated with CLO Virtual Fashion.

![Actual native WeftForge on macOS, original edited shirt](docs/images/studio-single-candidate.png)
