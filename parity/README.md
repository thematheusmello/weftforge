# Registry and evidence rules

`features.yaml` is JSON syntax, a valid YAML 1.2 subset, to keep development reads
deterministic. `schema.json` is JSON Schema 2020-12. Install requirements-dev.txt
and run `cargo xtask progress` / `cargo xtask verify`.

Stable IDs never disappear to improve counts. The 56 initial contracts are a
coarse, incomplete inventory. Fine-grained menus, tools, dialogs, parameters,
shortcuts, defaults, errors and whole workflows remain discovery work. Adding a
contract revises visible denominators; finishing known inventory is not proof of
complete product examination. `reference.json` has measured installation identity
but remains provisional until actual build and preferences are frozen.

Each feature separates implementation, observation, verification and freshness.
Scenarios contain original fixture hashes, exact initial conditions, actions and
expected outcomes. Reviewed specifications are in spec/features/. Run references
carry exact commit, platform, UTC timestamp and artifact hashes. Reports specify
`implementation_only` or an actual independently evaluated `reference_comparison`.
Raw licensed artifacts remain private; only reviewed redistributable evidence is
published. Never put personal paths, signed URLs, credentials or restricted code
in public reports.

Derived reference parity requires a frozen baseline, an observed complete
contract, implemented/passed/current feature state, a passing complete-contract
reference comparison, and all dependencies qualifying too. Implementation-only
passes and screenshots cannot satisfy that predicate. Missing, failed, blocked,
unobserved and stale states remain visible. Relevant source/fixture/settings
changes require rerunning evidence or marking it stale; latest tested commit is
not automatically current HEAD. Future cache/GPU checks must declare numerical
agreement separately from exact reproducibility.

The generator validates schema, dependencies/cycles, evidence paths and run
identity, committed SHAs, fixture/artifact hashes and generated consistency.
Failures must retain diagnostics. Reports are generated deterministically into
docs/progress.md and docs/dashboard/index.html. GitHub Project authorization is
optional: issues and this registry remain usable without it.
