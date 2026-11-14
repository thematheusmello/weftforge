# Native seam reversal acceptance

This private implementation-only case preregisters `UI-NATIVE-SEAM-REVERSAL-001`.
The evaluator reads the two public sewing contracts and original public native
join evidence, plus candidate identity metadata. It imports no Rust or new
implementation code and does not operate the GUI or run Cargo.

`fixture-provenance.json` records exact original hashes and the two deliberate
input changes. `original-inputs/fine.weftforge`, `fine.svg` and the original UI
declaration retain their source bytes. Both input fixtures are declared external
inputs; they are not new native outputs.

The work reaches the fixtures, the independent expected snapshot model, whole
document/history/rejection checks, exact no-change byte comparisons, the original
OBJ checker, SVG geometry comparison and the declaration. No production callers,
tool schemas, exports or persistence code are changed by this evaluator task.

Run from this directory using only Python's standard library:

```sh
python3 verify_native_reversal.py --self-check-only --output self-check-reproduced.json
python3 verify_native_reversal.py --case-dir . --output evaluation.json
```

Absent native files produce `incomplete_missing_native_outputs` and exit 2.
Self-check success is explicitly synthetic and never substitutes for native
execution. Rejected witnesses cover the wrong side, map mutation, an extra field,
an extra revision, stale numeric replay, allocator drift, revision-limit mutation,
nonfinite OBJ data and SVG drift. Temporary synthetic export text is deleted.

The parent supplies nine genuine saves (`before-reverse`, `after-reverse`,
`undo`, `undo-tab`, `redo`, `restored`, `reopened`, `max-loaded`, `max-rejected`,
each `.weftforge`) and `restored.obj` / `restored.svg`. The frozen declaration
contains the exact action order, identities, required filenames and criteria.
The input-to-native-save comparisons permit lexical JSON normalization; Undo to
Tab, restored to reopened and max-loaded to max-rejected require exact bytes.

The native candidate is source `9d30a71bbc2b0e051d9eaddc42b5f3b7dab08bde`.
`candidate-pretest.json` identifies its scoped source hashes and standard unsigned
package without importing or executing its code. Freeze the declaration and
evaluator before launching that candidate; record the freeze digest outside the
bundle. Do not tune a frozen oracle to observed native output.

Saved fields prove the specified oriented-parameter pairing and exact authoring
changes. Parent screenshots/process observations must establish focus, input
order, seam-row identity, visible endpoint-guide rendering and the revision-limit
error. No reference parity, arc-length equivalence, physical easing/twist, fitting,
simulation quality, nonzero placement or broader product claim is made.

This directory is private preparation. Publication requires tracked artifacts,
privacy review and registry/map checks from a clean checkout.
