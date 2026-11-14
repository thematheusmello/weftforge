# Frozen independent FILE-005-v1 acceptance

This acceptance oracle was prepared from reviewed spec commit c69e377 and the original public native source document, before inspecting any candidate DXF or production writer code. It invokes no WeftForge code, Cargo or UI. It uses ordinary ezdxf 1.4.5 readfile, followed by an audit that must report zero errors and zero fixes; it never recovers, saves or rewrites a candidate. Verification dependency only: the private environment's PyPI distribution/license/source and full resolved version list are in reports/consumer.json and reports/pip-install.json.

Input generation, stable identities, exact operations and hashes are in prepare_fixtures.py and input/manifest.json. The original source bytes remain unchanged in input/original-restored.weftforge. All expected geometry comes directly from JSON point positions, boundary-ordered curve IDs and cubic controls, using binary64 multiplication by 1000. No production-generated expected export is used. Native expected revision 21 is independently derived before launch from revision-20 mixed plus the declared resolution-only delta 0.04 to 0.045.

Required acceptance is conjunctive:

- Original ASCII bytes contain LF pairs, final newline, exact ordered six sections and EOF; empty CLASSES; required R2000/version/units/display headers and valid HANDSEED; scope/revision/units/mapping comments without names/paths/timestamps.
- Raw infrastructure includes all nine symbol tables and the profile's standard records, exact counts, subclasses, positive unique handles, resolving non-null references and correct table owners. DIMSTYLE has handle 105. Two empty named space blocks link through block records to the root/group/layout dictionaries and Model/Layout1 layout objects. R2007 block fields, XDATA, group objects, proxy/classes, per-piece blocks and extraneous entity types fail.
- Every expected authored boundary entity appears exactly once in stable ascending piece-ID order, authored boundary order, on declared WF_PIECE_<id> layers after layer 0. Layer visibility/thaw/lock/color/linetype match the contract. Every entity explicitly declares model ownership/flag/layout/subclass/lineweight and zero-Z geometry.
- LINE endpoints and SPLINE controls recover exactly the JSON-derived scaled binary64 numbers, with signed zero equal. SPLINE flags 8, degree 3, count fields 8/4/0, ordered knots 00001111, normal (0,0,1) and omitted weights/fit/tangents are mandatory. Adjacency/closure is exact; no extra closure edge, approximation, packing, placement or Y inversion is accepted. Rectangle source measures exactly 125 by 250 mm from numeric coordinates.
- Ordinary ezdxf model-space entities, layers, units, layouts, controls and knots agree with raw and source data. Independent B-spline evaluation agrees with direct authored Bezier evaluation at t=0,.125,.25,.5,.75,.875,1 within 1e-8 mm per coordinate. Exact control/knots comparisons remain required. The original DXF file bytes must remain unchanged after read/audit.
- Repeated same-source destinations must be byte-identical. Headless mixed and placement/resolution-only variant must export identical bytes because both have revision 20 and identical authored geometry. Empty valid source must pass with zero model-space entities and full required infrastructure.

The pretest uses an independent, in-memory hand-authored raw-tag calibration and an in-memory ezdxf-created geometry calibration, never a candidate. It checks positive geometry/structure and meaningful negative witnesses; reports/pretest.json records all failures and their rejection. The source generator, evaluator, manifest, consumer metadata, pretest and this criterion are hashed in freeze.json before any output reading. Later source/package/candidate hashes may be bound separately; acceptance criteria and evaluator may not be changed to accommodate output. Preserve every failure instead of repairing an export or relaxing this oracle. If a criterion defect is found, report it explicitly and treat replacement checking as a distinct revision.

## Frozen native scenario

1. Launch the identified candidate, open input/mixed.weftforge, save native/source-before.weftforge; JSON must equal the independent revision-20 fixture.
2. Open Export DXF and Cancel. Keep existing native/native-cancel-sentinel.dxf unchanged. Save native/source-after-cancel.weftforge; byte-identical to before.
3. Focus mesh resolution 0.04, atomically paste 0.045 without Tab, click toolbar Export DXF, save native/native-mixed.dxf. Pending field edit commits exactly once. Save native/source-after.weftforge; JSON must equal input/native-expected.weftforge, differing from initial source only in resolution and revision 21.
4. With no edits, use Command Search Export DXF to native/native-mixed-repeat.dxf; bytes match first. Save native/source-after-repeat.weftforge; bytes match after.
5. Quit/relaunch the same identified candidate, open source-after, save native/reopened.weftforge; bytes match after. No overwrite-warning case is promised.

Sentinel bytes are exactly `FILE-005 cancellation sentinel; preserve these original bytes.` followed by LF. The oracle compares all saved JSON fields, byte equality across cancel/repeat/reopen snapshots, repeated DXF bytes, and sentinel identity. Screenshots/action observations/launch provenance are separate evidence supplied by the native investigator. The oracle's saved-output comparisons cannot by themselves establish which UI actions occurred or inspect hidden history.

Commands, from repository root:

```
.local/dxf-acceptance-20261114/venv/bin/python .local/dxf-acceptance-20261114/evaluate.py self-test --report .local/dxf-acceptance-20261114/reports/pretest.json
.local/dxf-acceptance-20261114/venv/bin/python .local/dxf-acceptance-20261114/evaluate.py check --source .local/dxf-acceptance-20261114/input/native-expected.weftforge --dxf .local/dxf-acceptance-20261114/native/native-mixed.dxf --report .local/dxf-acceptance-20261114/reports/native-mixed.json
.local/dxf-acceptance-20261114/venv/bin/python .local/dxf-acceptance-20261114/evaluate.py native-snapshots --directory .local/dxf-acceptance-20261114/native --report .local/dxf-acceptance-20261114/reports/native-snapshots.json
```

Equivalent check commands bind each headless fixture directly to its matching original .dxf and report. Byte-comparison commands/reporting do not alter evaluator thresholds. Any native/headless file that is absent, failed or incomplete remains visible as such.

This proves only the bounded FILE-005-v1 writer/consumer profile, when outputs pass. It establishes no reference parity, DXF import, apparel interchange, garment round trip, printing or other CAD consumer behavior. Local ignored files are preparation, not published delivery; all selected evidence must be tracked and registry/map checks must run in a clean checkout before publication. GitHub Actions remain disabled until the whole project is done.

Primary source references reviewed for structure, auditing, layout links and units: the reviewed spec, https://ezdxf.readthedocs.io/en/stable/dxfinternals/filestructure.html, https://ezdxf.readthedocs.io/en/stable/drawing/drawing.html, https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-433D25BF-655D-4697-834E-C666EDFD956D.htm, https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-40B92C63-26F0-485B-A9C2-B349099B26D0.htm and the version-pinned PyPI metadata. No proprietary data used.
