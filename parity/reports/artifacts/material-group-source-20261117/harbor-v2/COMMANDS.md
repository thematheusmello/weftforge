Run from candidate checkout `.local/coupled-edge-regression-20261116`.
Only read/hash/static verification is authorized now:

```sh
<HOME>/.pyenv/versions/3.13.0/bin/python3.13 -B .local/harbor-material-groups-preparation-20261116-v2/bind_source.py --verify-only
```

SOURCE-PREFLIGHT SHA256: `76b46c72fdda9a1ece636a29e52bf48fb5325f6c3565d2e2aacb3a1490711927`.
Build binding SHA256: `b35c301476e23c92b376698b8a99fa38fb845ceb26fd3ba46b2eb2207a4b28f9`.
Root must review the complete v2 delta and supply a NEW BUILD-GO.json from the
unauthorized template. The v1 GO does not authorize v2. After that build-only GO:

```sh
<HOME>/.pyenv/versions/3.13.0/bin/python3.13 -B .local/harbor-material-groups-preparation-20261116-v2/build_release.py --go BUILD-GO.json --go-sha256 <EXACT_NEW_BUILD_GO_SHA256>
<HOME>/.pyenv/versions/3.13.0/bin/python3.13 -B .local/harbor-material-groups-preparation-20261116-v2/freeze_prep.py
```

The helper pins the actual Cargo/rustc executable hashes, offline locked argv,
flags, wrapper-free child environment, local/compiler config inventory and 16
locked dependency source trees. It uses this same candidate's existing `target`;
no compiled cache or binary is copied. Separate build bounds remain 120 s / 1 GiB
sampled per-process RSS/8 MiB output. It launches no producer/model.

After root separately reviews the actual binary, build receipt and final FREEZE,
a distinct execution GO is required. Only then, once:

```sh
<HOME>/.pyenv/versions/3.13.0/bin/python3.13 -B .local/harbor-material-groups-preparation-20261116-v2/controller.py --go ROOT-GO.json --go-sha256 <EXACT_NEW_EXECUTION_GO_SHA256>
```

The numerical clock starts before input copying and init-only; 420 s includes
both fresh initializations, metadata comparison, 180 steps, independent analysis,
receipt persistence and root external final witness. Numerical child/output caps
remain 256 MiB. Exit 0 = complete strict pass; 2 = complete strict quality failure;
1 = refusal/operational failure. No phase/child retry, sweep, warm start,
continuation or prefix regrade. Root independently retains terminal/PID/RSS/
volume/clock/hash witnesses. No build or execution is authorized by this file.
