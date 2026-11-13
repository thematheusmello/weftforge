# Contributing

Read AGENTS.md, STATUS.md, ROADMAP.md and the feature contract before changing
behavior. Use a feature branch and link stable parity IDs and issues in the PR.
Keep implementation, reference observation and verification states separate.
Reference investigations use original fixtures and reviewed public behavior
contracts; restricted binary analysis and commercial assets stay outside Git.

Install the pinned Rust toolchain and Python 3.11+ for development tooling:

```sh
python3 -m pip install -r requirements-dev.txt
cargo run -p weftforge
cargo run -p weftforge-cli -- --help
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask scenario
cargo xtask verify
```

`cargo xtask progress` regenerates the report/dashboard. Keep evidence tied to
its tested commit, fixture hashes, settings and platform. Recheck affected
contracts or mark evidence stale after changes. Never replace a failed scene
with a smaller passing fixture while claiming the original scene passed.

Do not execute untrusted fork code in a reference workstation with account
credentials. Public CI only receives redistributable fixtures and source.
Contributor submissions are licensed MIT OR Apache-2.0 unless explicitly agreed.
