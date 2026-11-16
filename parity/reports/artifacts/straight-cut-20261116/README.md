# Bounded straight-cut evidence

This collection records WeftForge runtime
`19b1e88245dc627cfec8164f4924b335fe44853d`, clean metadata `982ad03`,
and native executable SHA-256
`117981304290aca8ea5f2d63bb77d81641791667bdc8ee89e5cb2b92de3c8d3e`.
It covers a straight unsewn rectangle cut, history and native saves, followed by
a fresh-process edit of the actual saved child. It establishes no reference
parity or complete cuts/holes feature.

See the [engineering report](../../WF-20261116-STRAIGHT-CUT-001.json),
[original contract](contract-history/CONTRACT.md), and exact
[runtime source snapshots](runtime-source/).

| Case | Outcome | Paired operations + summary |
| --- | --- | ---: |
| R001 | Bootstrap stopped before authoring; one saved-file assessment failed | 5 + 1 |
| R002 | Name-field focus stopped before committed authoring; one assessment failed | 12 + 1 |
| F001 | Coordinate-focus diagnostic only; no geometry or saves | 20 + 1 |
| R003 / CUT-R1 | Accepted bounded author/cut/save/Undo/Redo case | 139 + 1 |
| R2-001 / CUT-R2 | Accepted bounded restart/edit/save case, with metadata qualification | 34 + 1 |

The two accepted cases use separate continuous clocks. Their conservative
post-summary upper bounds are 1122.241026542 seconds within 1200 and 373.045164
seconds within 600. These are laboratory operation budgets, not application
performance measurements. F001's 183.527439834-second upper bound cannot establish
its 180-second cap pass; it is not a cut acceptance.

R003 retains four exact native v4 saves. Its one-shot
[saved-state assessment](native/R003/SAVED-GRADE-001.json) and separate
[root UI acceptance](native/R003/ROOT-UI-ACCEPTANCE.json) cover different barriers.
The independent [UI review](reviews/native-plan/R003-UI-REVIEW.md) inspected all
60 original images/AX captures and 139 trace pairs.

R2 opens the exact accepted R4 as a read-only primary. Its one-shot
[saved-state assessment](native/CUT-R2/R2-001/SAVED-GRADE-001.json) checks R5 against
that actual input: only the declared child-height coordinates and revision change.
The separate [UI review](reviews/native-plan/R2-001-UI-REVIEW.md) and
[qualified root acceptance](native/CUT-R2/R2-001/ROOT-UI-ACCEPTANCE.json) cover the
fresh process, one Tab, clean Save As, retained-piece inspection and exit.
The input-path autosave is native R5 output; it is distinct from the unchanged
read-only R4 launch primary.

R2's original binding retains the contradictory extra draft diagnostic
`missing_root_UI_dependency: true`. Canonical accepted dependency fields and their
actual hashed files were present before launch and independently checked. Root's
later clarification and qualified acceptance preserve the oversight; no binding
was repaired and no case was regraded. Runtime proof and exit corroboration also
retain their stated retrospective/delegated limits rather than claiming an
independent in-situ transport receipt.

The source gates retain 304 Rust tests, formatting, strict workspace Clippy and
clean metadata/xtask/build results. Separate constructor tests cover actual R240
mesh and Cloth masses at 0.005/0.07/0.5 m and L at 0.07/0.5 m. Both original and
cut L meshes hit the existing vertex cap at 0.005 m. Saved area times density is
analytic authored-data evidence; the native cases do not measure particle mass,
drape, contact or convergence. Native concave/failed-I/O cases, curved/sewn cuts,
holes, Cut & Sew and general workflows remain open.

The historical [initial index](PUBLICATION.json) and
[R2 appendix](PUBLICATION-APPENDIX-001.json) describe the original selection of
1,257 payloads. The appendix supersedes initial pending-R2 statements and clarifies
staging chronology. The [current index](PUBLICATION-CURRENT.json) defines the
effective publication bytes, superseding historical exact-JPEG wording. The
payloads, historical indices and two metadata records comprise 1,261 copied files;
this overview is an additional publication document.

Of the payloads, 1,013 preserve original bytes, 89 are explicit literal-path
derivatives, 83 omit duplicate screenshot/AX or global app-inventory result bodies,
and 72 are JPEG metadata derivatives. Original record hashes, IDs, errors and
clocks remain attributed; all 210 before/after pairs are accounted for. Saved
projects and sidecars are unchanged. The original private records remain distinct
from curated derivatives; this is an evidence archive, not a relocated executable
harness.

The [JPEG metadata record](JPEG-METADATA-REDACTION.json) binds original and public
hashes. Eight monitor serial/manufacture-date metadata bytes per image are zeroed;
the profile identifier was already zero. Encoded scan data, decoded pixel buffers
and every color-characterization tag match the originals. The native-case originals
and their receipts remain untouched. Historical receipt hashes refer to those
originals, rather than claiming the public metadata derivatives are byte-identical.

The fixed allowlists contain 72 owned-window frames. Twenty-five private
files—ten chooser images, ten chooser AX files and five global process lists—are
omitted with original hashes/reasons. No image was generated, re-encoded, cropped
or visually edited; only the documented public header identifiers were redacted.
Personal paths, account data, compiled binaries and proprietary research are not
distributed here. The application has no dependency on these laboratory scripts.
