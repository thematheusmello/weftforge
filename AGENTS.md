# WeftForge

Build the complete native Rust garment studio described by ROADMAP.md. Keep work
bounded to integrated increments while preserving the full parity target.

- Editable construction is authoritative. Keep rest meshes, simulation state,
  display meshes and caches distinct and traceable to document revisions.
- SI metres, kilograms and seconds; f64 authoring and CPU simulation; Y-up 3D.
- Core and solver must stay independent of GUI dependencies.
- Never claim reference parity from implementation-only tests. Keep unknown,
  blocked, failed and stale evidence visible in parity/features.yaml.
- Keep the root README focused on WeftForge; put reference laboratory details
  in research and evidence documents.
- No proprietary installers, assets, binary extracts, credentials, or personal
  filesystem paths in public files. Research lives outside this repository.
- Use shared typed commands for authoring; commit gestures as one transaction.
  Keep command changes in sync with CLI/MCP adapters, tool schemas and tests.
- Validate external input and preserve the old project on any failed load/save.
- Run cargo fmt --all --check, cargo test --workspace, cargo clippy --workspace
  --all-targets -- -D warnings, and cargo xtask verify for an integrated change.
- Keep GitHub Actions disabled during development at the user's request. Run
  checks locally; re-enable Actions and Pages only once the full project is done.
- Maintain STATUS.md with evidence, exact next steps and confirmed job handles.
- Before publishing evidence, require every artifact to be Git-tracked and run
  registry/map checks from a clean checkout; local ignored files are not delivery.
- Keep compiled Cargo targets separate across worktrees; cached binaries can embed
  checkout paths. Registry downloads may be shared.
- Independent geometry, simulation, UI and verification agents may work in
  parallel with explicit file ownership and agreed interfaces. Reference
  investigators deliver reviewed public behavior contracts, never proprietary
  pseudocode or assembly, to fresh-context implementers.
