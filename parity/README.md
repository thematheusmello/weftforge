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

Registry schema version 2 gives every retained report its own `freshness` and
`source_scope`. A scope lists exact repository files or directories relevant to
that run, including its implementation dependencies and independent evaluators.
Directory scopes detect added and deleted source files. Every original entry in
`source_hashes` must still match the report's tested commit, even for stale reports
and files outside the current scope. Artifacts and original fixture hashes remain
checked and retained. Only reports declared `current` compare their scoped inputs
with the working tree; changed inputs require a rerun or explicit stale status and
a `freshness_reason`. An old baseline may remain in a stale report. A current
report must match the registry's current baseline.

A feature marked current needs at least one current source-qualified run. A
passing feature cannot hide any current failed scene, including by dropping its
run reference. Historical stale failures do not block a new current pass. A stale
reference pass cannot establish parity through a newer implementation-only run:
the qualifying comparison itself must be current, source-qualified, reference
executed and complete, and every dependency must qualify at the same baseline.

The initial migration appends these review fields to all five reports without
changing their IDs, tested commits, original source hashes, artifact hashes,
fixtures, outcomes or measurements. Native workflow and numerical scopes include
their Rust/runtime and independent evaluator inputs; editing progress generation
alone does not stale those runs. The aggregate regression report includes the
progress checks and is now stale. Its seven otherwise-uncovered feature freshness
flags are stale until those checks are rerun; no implementation or parity count
was increased. Do not narrow a scope to omit relevant changed inputs.

Scopes are reviewed input declarations, not compiler dependency tracing; review
them when adding build, runtime or evaluator dependencies. Before promoting the
first reference comparison, explicitly qualify the baseline revision/settings
digest and reviewed contract/scenario/comparison-criteria digests. Matching a
baseline ID and source alone cannot establish that those requirements stayed
unchanged. That promotion gate remains future work; there are currently zero
reference comparisons and zero verified parity contracts.

The generator validates schema, dependencies/cycles, evidence paths and run
identity, committed SHAs, fixture/artifact hashes and generated consistency.
Failures must retain diagnostics. Reports are generated deterministically into
docs/progress.md and docs/dashboard/index.html. GitHub Project authorization is
optional: issues and this registry remain usable without it.
