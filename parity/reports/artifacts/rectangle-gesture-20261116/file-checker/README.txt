Independent native rectangle file checker

Only the 1,977-byte empty native v5 seed and a parent-supplied pre-output
serialization/case contract were inspected. No product source, tests,
formal graders, actual case work/output files, app, Cargo, or UI were read/run.

Scope: checker.py, targets.json, seed-expected.json, synthetic self-check receipt,
freeze manifest, and the one later actual-run receipt in this isolated namespace.
No repository callers, exports, configuration, input files, or case files change.

Run synthetic checks before freezing:
  python3 .local/rectangle-file-checker-20261116/checker.py --selfcheck

Wait for explicit parent GO and all five output SHA-256 hashes. Write that supplied
path-to-hash map to an authorization file in this namespace, then run exactly once:
  python3 .local/rectangle-file-checker-20261116/checker.py --run-once <root-relative-authorization-file>

The runner exclusively creates run-started.json before reading outputs; it cannot
be rerun in this namespace. It checks the frozen artifact hashes/modes, original
seed, declared native outputs, and exact work/R4 bytes, and retains before/after
hash, byte-size and mode snapshots. Failed inputs are never repaired or regraded.

The creation origin and 3D physical default placement are unspecified. Placement
must have finite f64 three-vectors and stay exact across redo/R4. Rectangle geometry
uses the ordered 3→4→5→6→3 loop, axis alignment, 10 mm lattice, and absolute 1e-12
metre/1e-12 square-metre tolerances. Area is independently calculated from translated
analytic polygon coordinates. Unchanged authoring values retain exact types and f64
bits, including signed zero. Shape/loop/project extras are rejected.

Before GO, the R4 mechanism changed from numeric entry to a 10 mm leftward point
drag after Frame all because the numeric UI fields are absent. This does not change
file targets. All actual outputs were reported absent by the parent during freeze
preparation; the checker did not inspect output directories to establish this.

Limits: file-only evidence cannot establish UI actions, drag behavior, lifecycle,
native reopen, reference parity, physics, 3D surface, or full garment acceptance.
No novel generic schema framework, dependencies, product changes, or Cargo checks.
