# Native seam reversal evidence replay

This bundle contains exact copies of the preregistered native saves/exports and
all files named by the frozen artifact inventory. `README.md` and
`ui-declaration.draft.json` preserve their original prelaunch bytes.
`independent-summary.json` explains the supplied-output acceptance and its limits.

From this directory, use Python 3 with only its standard library:

```sh
python3 verify_native_reversal.py --case-dir . --output evaluation-reproduced.json
python3 verify_native_reversal.py --self-check-only --output self-check-reproduced.json
```

The first command must report `passed_for_supplied_outputs`; the second reports
`passed_synthetic_oracle_checks_only`. Generated reports are separate from native
outputs. `prepare_inputs.py` is frozen provenance from the original repository
preparation and is not needed for portable replay; the exact derived inputs and
source fixture are already present.

`preregistration-freeze.json` was recorded before candidate launch. Its SHA-256 is
`eac9898cf2b57e988baa61814fef952c18fa724eb6ca420cb96c5e310dfe8ace`.
The unchanged evaluator SHA-256 is
`ff5e33b3d1c0508091c3d0a510858c44a09115350afa4079e4f3ad1041d12c98`.
The accepted report SHA-256 is
`02045801f730180ff51ad1d99d4c18429340af134fe6d932ba32bdb23ed48cfe`.
The bundle manifest records exact-copy hashes and byte sizes. The original
historical helper is unchanged; no binary package or SDK is included here.

All nine native document checkpoints pass complete field comparisons. Undo/Tab,
restored/reopened and maximum-revision loaded/rejected pairs are byte-identical.
The final OBJ has 1,159 vertices and 2,192 positive triangles with valid UVs,
normals and metre scale. The SVG equals the original fine export and retains two
closed editable paths with one cubic. Nine deliberately wrong synthetic witnesses
are rejected; they do not count as native executions.

The nonlinear map proof concerns oriented curve parameters. Native saves cannot
prove UI event timing, focused text, endpoint-guide rendering or the visible
revision-limit error; use the separately supplied parent UI/process record.
There is no reference parity, arc-length equivalence, physical easing/twist,
fitting or simulation quality claim. Both placements remain default zero.

Publication still requires root review, tracked artifacts and registry/map checks
from a clean checkout. The bundle contains no fabricated or repaired native file.
