# Independent A16 preparation review

Blocked as frozen. Do not issue ROOT-GO for freeze `f7c19b4295ae8ec9d4246d95eebc4f53b7306fe4f6071a67f26eb0c333cb1a84`.

The one causal question remains appropriately bounded: whether the failed Harbor A closure transient changes materially when only substeps change from 8 to 16, against retained 8x32 A after exact rest equality. No new physics defect or improvement is established.

One blocker: `preparation-a16-001/run.py:111` samples `ended` before the final numerical-byte traversals at lines 113 and 115. The completion test uses that earlier `ended`. If the final inspection crosses the 420-second deadline, `FINAL.json` can still say complete and report an elapsed time below the cap. `PROTOCOL.json` includes final numerical inspection in the clock; its receipt-serialization exclusion does not cover those traversals.

Smallest correction: obtain and retain the final numerical byte count before taking the final cap timestamp, and reuse that count for the receipt and size decision. Every numerical-byte traversal, output hash and source/input/GO inspection included by the protocol must precede that timestamp. Only receipt serialization may follow it. Retain this unexecuted freeze; correct prospectively, re-freeze and re-review before GO.

Static checks confirmed all 38 frozen files, 22 current source inputs and 19 retained inputs match. The actual binary hash is `1d6caa60cd3c95b860332fb2d06f1dfee919f7fd035233e4aa21fd9e948a8067`. Driver and consumer diffs are exactly the claimed one and three literal changes. Five numerical helpers are byte-identical; all thirteen resource-helper function ASTs match the retained controller. The nine solver/bridge bindings match the retained source binding. Expected rest metadata differs only by two empty `internal_curves` arrays, using type-sensitive f64-bit comparison.

No other scoped blocker found: full rest/project/stitch/material/boundary/setting identity, actual mass equality, strict 15%/5mm/2mm/finite/uncertainty screens, 181-state completeness, initial-state comparison, immutable control hashes, process exit checks, exclusive one-attempt paths, A-only argv, and exact root GO/freeze binding remain in place. The old two-case consumer entry points refuse. Source dependencies are read-only in the declared driver path, and its target is separate. These conclusions are source review, not runtime verification.

ROOT-GO and all execution directories were absent. No producer-owned files were changed. Only these review artifacts were written.

Skipped: Cargo, candidate/preflight/simulation/consumer execution, synthetic selfcheck replays, GUI/REA/Git and publication.
Risk: the final-clock blocker must be corrected before GO; no physical, native, contact-completeness or reference acceptance is supplied.
