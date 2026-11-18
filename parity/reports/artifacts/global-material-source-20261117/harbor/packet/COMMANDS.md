Run from global candidate checkout .local/global-material-repair-20261117.
Only source/hash/environment verification is authorized now:

    <HOME>/.pyenv/versions/3.13.0/bin/python3.13 -B .local/harbor-global-material-preparation-20261117/bind_source.py --verify-only

SOURCE-PREFLIGHT SHA256: f9fde5c5bd48bb7caec469c3c7011099fe5e82474ef8b7b6d61e654eccf45cf8
Build/compiler binding SHA256: 4177dbd2064e746081f3a0be1c543aeb54b7a5327387e22332d4bcb86463d0d7

Root next reviews this exact source-only packet and provides a distinct BUILD-GO.json
from the unauthorized template (exact source/preflight/compiler and closed workspace
receipt2fd970b4 binding, no constructor/step). Only after that build-only GO:

    <HOME>/.pyenv/versions/3.13.0/bin/python3.13 -B .local/harbor-global-material-preparation-20261117/build_release.py --go BUILD-GO.json --go-sha256 <EXACT_NEW_BUILD_GO_SHA256>
    <HOME>/.pyenv/versions/3.13.0/bin/python3.13 -B .local/harbor-global-material-preparation-20261117/freeze_prep.py

The helper invokes the exact Cargo argv in SOURCE-PREFLIGHT with actual Cargo/rustc/
rustdoc files, RUSTUP_TOOLCHAIN=1.94.1 and actual toolchain/bin leading PATH for every
build child. Wrappers/foreign tool choices or changed parent PATH/flags refuse. No
caller-side toolchain export is needed; retain the bound parent environment. This
same checkout's existing target is allowed; no cross-checkout compiled copy. Separate
build bounds:120s/1GiB sampled process RSS/8MiB output; build launches no producer.

Root then reviews the actual release binary, build receipt and final FREEZE. Only a
separate exact13-key execution GO may authorize the once-only case:

    <HOME>/.pyenv/versions/3.13.0/bin/python3.13 -B .local/harbor-global-material-preparation-20261117/controller.py --go ROOT-GO.json --go-sha256 <EXACT_NEW_EXECUTION_GO_SHA256>

Root starts its outer420s numerical clock before controller launch/init-only and
retains the final post-persistence terminal witness. All input/gold checks, both fresh
initializations,180 steps, independent181-state analysis, serialization, source guards
and receipts stay inside that one420s/256MiB child+output budget. All global PCG/Armijo
work is charged; no pre-stepping, continuation, retry, sweep, tuning or relaxed screen.
The init-only process takes zero steps and must match original gold before the separately
fresh run process is started. Exit0=complete strict pass;2=complete quality failure;
1=operational refusal/failure. Root witnesses clock/RSS/PIDs/output/terminal independently.
No build or execution is authorized by these prospective commands.
