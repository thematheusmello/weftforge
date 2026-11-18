Working directory: the reviewed candidate checkout
`.local/coupled-edge-regression-20261116` relative to the root workspace.
Use the exact Python executable pinned in SOURCE-PREFLIGHT.json.

Read/hash/environment verification only (allowed now):

```sh
python3 -B .local/harbor-material-groups-preparation-20261116/bind_source.py --verify-only
```

Root next reviews SOURCE-PREFLIGHT and writes BUILD-GO.json in the packet with:
status=authorized_build_only, explicit_root_review_and_go=true, experiment,
source_commit, source_preflight_sha256, build_binding_sha256,
full_workspace_gates_passed=true, workspace_gate_receipt={path,sha256},
Cloth_initializations=0 and Cloth_steps=0. The receipt is the closed root four-stage
workspace result, already retained at evidence/WORKSPACE-GATES-RESULT.json.
The exact Cargo path/argv/compiler/env/source closure are in the build binding.
The existing own target is permitted; no target, binary or cache may be copied.
Only after root build GO:

```sh
python3 -B .local/harbor-material-groups-preparation-20261116/build_release.py --go BUILD-GO.json --go-sha256 <EXACT_BUILD_GO_SHA256>
python3 -B .local/harbor-material-groups-preparation-20261116/freeze_prep.py
```

Build-only work launches no producer/model. Its separate bounds are 120 s,
1 GiB sampled per process and 8 MiB log. Numerical bounds remain 420 s and 256 MiB,
not the earlier diagnostic's smaller limits. Root must review the actual binary
and final FREEZE before creating ROOT-GO.json with status=authorized,
explicit_root_review_and_go=true, experiment, freeze_sha256, source_commit,
build_binding_sha256, cases=[base], init_only_initializations=1,
actual_run_initializations=1, actual_dynamics_runs=1, init_only_step_calls=0,
retries=0 and continuations=0. Then, once only:

```sh
python3 -B .local/harbor-material-groups-preparation-20261116/controller.py --go ROOT-GO.json --go-sha256 <EXACT_EXECUTION_GO_SHA256>
```

Controller starts the one 420 s numerical clock before input copying/init-only.
Independent analysis, every receipt and root external final guards must fit that
same deadline/output budget. Exit0 means complete strict trajectory quality pass;
exit2 means complete quality failure; exit1 means refusal/operational failure.
Root must independently witness terminal exit, clock, volume and receipt hashes.
The four workspace stages do not certify metadata freshness/xtask or garment/
reference acceptance. No actual build or case execution is authorized now.
