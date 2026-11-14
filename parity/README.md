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
them when adding build, runtime or evaluator dependencies. Source identity alone
does not qualify reference requirements. There are currently zero published
reference comparisons and zero verified parity contracts.

## Reference comparison qualification

Before executing a comparison, commit the baseline, settings, reviewed contracts,
scenario definitions, original fixtures and criteria along with the tested
implementation/evaluator. A current executed `reference_comparison` report must
include `reference_qualification`, whether its outcome passed or failed. A
nonexecuted blocked attempt and an `implementation_only` report may omit it;
neither establishes reference parity. Stale legacy reports may also omit it.

Add an optional `contract_documents` object to each reviewed feature, mapping
repository-relative public specification paths to SHA-256 hashes of their exact
file bytes. A complete passing comparison requires at least one reviewed document
and one scenario for each covered feature and every transitive dependency. A
documentation link, menu entry, screenshot or generic inventory sentence alone is
insufficient. Partial exploratory comparisons may record qualification without
these reviewed documents, but cannot establish complete-contract parity.

The report already names `reference_baseline`; its qualification has this shape
(the placeholders below must be replaced with actual lowercase SHA-256 values):

```json
{
  "reference_qualification": {
    "baseline_revision": 1,
    "baseline_sha256": "<sha256>",
    "settings_sha256": "<sha256>",
    "contracts": {
      "PAT-001": {
        "contract_sha256": "<sha256>",
        "scenarios_sha256": "<sha256>",
        "comparison_criteria_sha256": "<sha256>"
      }
    }
  }
}
```

`contracts` contains exactly the report's `feature_ids` and all their transitive
dependencies. The hashes use SHA-256 of UTF-8 canonical JSON generated by Python
`json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False,
allow_nan=False)`, with no trailing newline:

- `baseline_sha256`: an explicit technical identity projection of
  `reference.json`: top-level `baseline_id`, `baseline_revision` and `product`;
  `installer` containing only `sha256`; `application` containing only
  `bundle_name`, `bundle_id`, `bundle_short_version`, `bundle_version`,
  `executable_name`, `executable_sha256`, `observed_ui_version`,
  `observed_ui_build` and `observed_runtime_architecture`; and `host` containing
  only `os`, `version`, `build` and `architecture`. Missing members are explicit
  nulls in the projection so provisional/exploratory comparisons remain
  attributable. Evidence, access observations, mutable status, provenance prose
  and account state are excluded. Parity requires nonblank product/bundle ID,
  observed UI version/runtime architecture and all four host fields, valid
  lowercase 64-hex installer/executable SHA-256 values, and an observed UI build
  recorded as nonblank text or a positive integer. Optional bundle names/versions
  and executable name may remain null; an absent bundle version is legitimate.
- `settings_sha256`: all `reference.json.settings` entries except the legacy
  `license_entitlement` field. That field is account state; keep account details
  outside behavior settings. Any additional declared behavior settings are
  included. Parity additionally requires `frozen: true`, `units` as a nonempty
  string or object, `locale` as nonblank text, and `preferences` and `graphics`
  as nonempty objects. No null or blank string is allowed anywhere in this
  settings projection; legitimate empty nested lists remain valid. Unknown
  technical settings remain null and the baseline remains unfrozen; do not
  invent values to satisfy the gate.
- `contract_sha256`: an object containing exactly `required_reference_behavior`,
  `contract_documents` (an empty object when absent) and `dependencies` from the
  feature. Each declared document hash must match its actual file bytes. Mutable
  implementation status, blockers and progress prose are excluded; reviewed
  contract files should contain normative behavior requirements.
- `scenarios_sha256`: the feature's complete ordered `scenarios` array, with each
  scenario extended by `definition_sha256`, the SHA-256 of its exact `path` file
  bytes. All registry scenario fields are included, including original fixture
  hashes, initial conditions and expected outcomes. Every original fixture hash
  must match its actual bytes.
- `comparison_criteria_sha256`: the complete `comparison_criteria` object,
  including units, exact comparisons, numerical tolerances/justifications and
  review status.

Use the existing evaluator helper to calculate the object from reviewed inputs:

```sh
python3 - <<'PY'
import json
import sys
sys.path.insert(0, 'scripts')
import progress
print(json.dumps(progress.reference_qualification(
    {'feature_ids': ['PAT-001']}, progress.read('parity/features.yaml'),
    progress.read('parity/reference.json')), indent=2))
PY
```

The validator recomputes every supplied qualification from the registry,
baseline and input bytes at the report's tested commit, even for stale reports.
It separately compares current executed reports with current inputs. Changing the
baseline ID/revision or technical identity, behavior settings, contract text/document, scenario metadata
or file, fixture, dependency requirements or comparison criteria requires a rerun
or `freshness: stale` plus an explicit reason. Preserve the original qualification,
tested commit, artifacts and failure diagnostics; replacing old digests with new
ones is rejected. Rerunning a dependency alone cannot qualify an older parent
comparison. Stale reference passes cannot be promoted by a current
implementation-only pass.

The generator validates schema, dependencies/cycles, evidence paths and run
identity, committed SHAs, fixture/artifact hashes and generated consistency.
Failures must retain diagnostics. Reports are generated deterministically into
docs/progress.md and docs/dashboard/index.html. GitHub Project authorization is
optional: issues and this registry remain usable without it.
