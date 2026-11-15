# Static capsule surface checkpoint

Runtime `0770204a54abdf5f929b6626b01d7fcfb0e13ca4` adds static triangle/capsule
surface contact, conservative uncertainty diagnostics, a CLI fitting guard and
native uncertainty display. All forty runtime/test files in `experiment/source`
match the tested commit and the integration package binding. The recorded local
gates passed formatting, 153 Rust tests, strict workspace/all-target Clippy,
xtask verification with 54 Python checks, and unsigned packaging. This packet
contains source and original redistributable authoring data; no executable or
release package is included.

Two separately authorized original Harbor cases ran once each, using unchanged
base revision20 and height revision17. Each retained its initial state plus all
180 accepted boundary states, positions, velocities, raw diagnostics/settings,
180 step records, rest metadata and OBJ exports at0/60/180. The protocol was
60 gravity-off assembly steps followed by120 gravity-on steps at dt1/60s,
8substeps/32iterations and1m/s seam closure. The independent evaluator and runner
were frozen and reviewed before authorization. These cases are closed; this
publication performs no new dynamics or timed benchmark.

| Case | Whole-case seconds | Producer peak RSS bytes | Strict result | All-state static-contact screen |
| --- | ---: | ---: | --- | --- |
| A / base | 167.26820654096082 | 8749056 | Failed | Passed,181/181 |
| B / height | 162.04855137504637 | 8585216 | Failed | Passed,181/181 |

The whole-case clock starts before first process launch and closes after the
original independent numerical analysis, exports, hashes and inspection. Later
receipt serialization/audits are integrity metadata. The producer's internal
wall time, outer process duration and whole-case duration have separate scopes.
Peak RSS is the producer process only. Numerical-output volume is an aggregate
experiment counter, including earlier cases: final recorded A19088601bytes and
B36371202bytes. The420s whole-case,256MiB producer RSS and separate256MiB
numerical-output limits passed both original operational audits. Publication
copies/redaction/replay are outside those closed clocks; their size and elapsed
time never replace the original counters.

Both cases pass independent full static triangle/capsule and supported contact
below2mm at all181 saved states, with zero unresolved oracle pairs, zero raw
conservative-policy counts and no blocked required diagnostic verification.
The largest independent physical-contact upper bounds are A4.552257086565259e-7m
and B3.7889283330261314e-7m, both at29. A's final contact interval is
[2.3070999236041168e-7,2.3071266899710174e-7]m.

A fails the unchanged15% material-axis screen at28–37, reaching
19.90919497559327% at31, and has33 exact peak triangle/axis attribution failures.
B passes all trajectory physical and completed-fit screens, peaking at
14.611755162261497% strain at30, but fails exact attribution at6,10,18. Both
strict results therefore remain failed. Full failure lists, every row and raw
diagnostic disagreement are retained in `RESULTS.json` and the original
`experiment/runs/*/analysis.json`. Initial open seams/targets at0–28 describe
arrangement, not completed-fit failure or success; exact zero targets start at29.
Final-only producer quality flags do not qualify the trajectory or delivery.

Historical original3aa A29 strain15.35589019880823%, A180 capsule penetration
9.009700546250568mm and B180 penetration10.672413745369775mm remain unchanged in
the previous reports. The native7fe Save18 trace gap and raw metadata failures
remain open. The b760 studio gallery is historical after changed runtime source;
no current native uncertainty-label observation is supplied here. No full garment
gate, reference parity, continuum proof, CCD, moving-avatar, edge-edge/layered
completeness, convergence or calibration claim follows from this packet.

## Replay and provenance

From a checkout containing this tracked packet, run:

```sh
python3 parity/reports/artifacts/capsule-surface-20261115/verify.py
```

Exit0 means the saved results reproduce, including both strict failures. It does
not mean garment acceptance. The replay starts no engine, Cargo, native UI or
driver. It verifies the complete public manifest, Git tracking, forty tested
source hashes, original closed receipts and every deterministic field of both
saved analyses using independent math. Replay elapsed time is excluded from
comparison and cannot recertify the closed execution clock.

`original-evaluator/FREEZE.json` and all24 frozen files retain their exact bytes.
They preserve private relative path bindings for audit, including historical
inputs outside this packet; the archival entry points retain those guards and
are not advertised as portable. `public-evaluator` is a separate derivative:
only three directory assignments and the rest-binding path prefix change.
The replay checks AST identity outside those assignments, exact numeric-module
bytes and complete deterministic equality against the original saved analyses.
There is no semantic, tolerance or cutoff change. Original binaries and all141
private historical binding dependencies are not distributed or publicly rerun.

`PUBLICATION-PROVENANCE.json` maps every original logical location to its actual
public file and records raw/public hashes. Personal absolute path prefixes in
gate logs, producer-handle metadata and integrity audits are replaced literally
with named placeholders, with prefix digest/length and substitution counts.
Those files are publication derivatives, never frozen originals; original
resource counters, clocks and numerical results are preserved. A private logical
path in an archived record is provenance, not a missing public artifact link.
`MANIFEST.json` binds every packet payload. The separate publication closure
records clean-checkout replay, registry/map and tracked-artifact checks.
