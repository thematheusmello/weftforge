# Native bounded-join output evidence

These original native WeftForge saves and exports record UI-BOUNDED-JOIN-001.
The frozen independent evaluator passed all supplied-output checks at the
preregistered 1e-12 m and normalized 1e-12 limits. See `evaluation.json` and
`independent-summary.json` for measured bounds, identities and limitations.

From this directory, reproduce the checks using Python's standard library:

```sh
python3 original-evaluators/verify_native_join.py --case-dir . --output evaluation-reproduced.json
python3 original-evaluators/verify_native_join.py --self-check-only --output self-check-reproduced.json
```

The two evaluator files and historical baseline retain their original bytes.
The original split SVG validator assumes two cubic children, so the join evaluator
uses its unchanged SVG parser and OBJ checker with a join-aware SVG check.
Exact rational power restrictions and Bernstein control bounds cover every
curve parameter and every interval of the stored sewing correspondence.

All eleven native snapshots, both resolutions of OBJ/SVG exports and the frozen
declaration are exact copies. Focus-only, later Tab and rejected corner-join
snapshots preserve their respective accepted document bytes. Undo/Redo,
resolution changes and reopen preserve the declared authored fields and IDs.
`corner-join-error.png` is the exact reviewed app screenshot; EXIF holds image
dimensions only. `inputs-preserved.json` records the native input hash inventory.

The reports inside `original-evaluators` named self-check, synthetic-check or
missing-input-check test the evaluator and remain separate from native outputs.
No reference parity, cloth fitting, simulation quality, large-translation native
coverage or coalesced-event native coverage is claimed. Parent observations
provide GUI/process attribution; the saved files alone cannot establish it.
