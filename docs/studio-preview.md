# Native studio preview

These are unedited 1640 × 1032 captures of the running native Rust application on
macOS. The current images use source `21b9f5e76a77ca21357783240cef7506b3887fbc`.
PhotoCraft informed the charcoal workspace and teal accents; the application,
documents and graphics are original WeftForge work.

## Current workspace and Snap choices

![Current studio with inline Snap choices and connected construction/rest views](../parity/reports/artifacts/native-studio-20261115/preview/captures/studio-current-hero.jpg)

This separate read-only preview loads the existing original Harbor project at
revision 20: two pieces, eight seams and 654 rest vertices. Snap remains 10 mm.
It makes no numeric input, construction edit, Save, history, Snap change or physics
action. It shows the current workspace, not completed garment or snapping quality.
The session-only Off / 1 / 5 / 10 mm preference and Alt/Option bypass are implemented;
The preference applies to pointer positions while numeric edits stay exact.

## Current task details

![Current transport showing the accepted rest-mesh task](../parity/reports/artifacts/native-studio-20261115/preview/captures/studio-task-details.jpg)

Task details shows Mesh task 1, input revision 20, Accepted, worker outcome
succeeded, cancel=false and obsolete=false. The expanded panel keeps the job
identity and outcome beside the transport while both construction and rest views
remain visible.

Native numeric/snapping and task-lifecycle acceptance remains unverified; see the
[scoped verification report](../parity/reports/WF-20261115-NATIVE-STUDIO-001.json).

## Historical DXF workflow (76be)

![Earlier source 76be studio in Harbor rest arrangement](../parity/reports/artifacts/dxf-studio-20261115/preview/Harbor-current-studio-rest-hero.jpg)

These captures retain their original source 76be results and limits. That report
is stale on the current native source; this publication reruns none of its cases.

The source 76be native workflow adds two original DXF patterns to an existing piece
after review of count, 1 mm per DXF unit, retained origin/layout, layer names and
construction-only scope. The strict R2000 profile accepts ordered closed `LINE`
and clamped cubic `SPLINE` boundaries; it is not general CAD or apparel import.
The [format profile](file-formats.md) lists the accepted structure and omissions.

The observed run retained a pending numeric draft through chooser Cancel,
review Cancel and command-search Escape. One Import combined that draft and
both pieces, followed by two Undo and two Redo actions. An imported cubic stayed
editable: P1 X changed from −0.01 to 0 m, with its other controls retained.
Rejected input left the edited construction intact. Actual quit, fresh Empty
launch, reopen, resave and SVG/DXF exports followed. Independent saved-file and
export grading passed, including exact edited/rejected/reopened native bytes
and an ezdxf 1.4.5 audit with zero errors or fixes. See the
[DXF studio packet](../parity/reports/artifacts/dxf-studio-20261115/README.md)
for the native captures, trace review and separate headless checks.

This supports the bounded observed workflow. The old Redo branch after Cancel
was not invoked before the new import, and the modal did not explicitly name
printing. Background autosaves retained temporary edits even though all original
input primaries remained unchanged. The complete frozen native plan, full
persistence behavior, printing and reference parity remain unestablished.

### DXF review, editing and restoration

![DXF review before import, showing count, millimetre scale and construction scope](../parity/reports/artifacts/dxf-studio-20261115/native-v2/captures/04-original-dxf-review-before-import.jpg)

The review shows the bounded construction import. Its text did not explicitly
name printing; the full disclosure criterion remains unestablished.

![Accepted DXF patterns remain editable beside the original piece](../parity/reports/artifacts/dxf-studio-20261115/native-v2/captures/11-imported-three-pieces-revision5.jpg)

One accepted import produces revision 5 and three editable pieces. The pending
width and both added patterns belong to the same transaction.

![Imported cubic after the single P1 X edit](../parity/reports/artifacts/dxf-studio-20261115/native-v2/captures/17-edited-cubic-controls-revision10.jpg)

Saved revision 10 changes P1 X from −0.01 to 0 m while retaining the other controls.
This establishes construction editing, not simulated garment quality.

![Already-saved revision 10 restored in paused Rest after the temporary-document phase](../parity/reports/artifacts/dxf-studio-20261115/native-v2/captures/29-final-reopened10-paused-cubic-visible.jpg)

This view follows the third launch used to restore saved revision 10 after the
separate temporary document was explicitly discarded. Phase A quit/fresh-open/
resave is separately recorded; this restoration is not a retry. The source 76be Harbor Rest preview belongs to that earlier checkpoint.

## Historical SVG workflow (32f9)

The following captures belong to source
`32f9b3481c46e28a3b3acc49e6cf3908c7001b28`; they retain that build's results
and limitations.

![Earlier studio with connected 2D construction and 3D rest arrangement](../parity/reports/artifacts/native-svg-20261115/native-v2/captures/09-current-studio-harbor-rest.jpg)

![Native import review with explicit physical scale and scope](../parity/reports/artifacts/native-svg-20261115/native-v2/captures/01-scale-review.jpg)

Import reviews the piece count, physical scale, retained origin and omitted
properties before adding anything. The bounded profile accepts closed absolute
straight/cubic paths with explicit millimetres and uniform viewBox scale. It
imports exterior boundaries and names; the [format profile](file-formats.md)
describes the supported subset.

![Imported cubic selected with editable control handles and numeric properties](../parity/reports/artifacts/native-svg-20261115/native-v2/captures/05-edited-cubic-framed.jpg)

The selected imported cubic exposes its original control points. This actual
native edit moves its first control X from 0.08 to 0.09 m. **Frame all** keeps the
separated patterns visible using their authored curve bounds.

![Reopened native project retains the imported pattern construction](../parity/reports/artifacts/native-svg-20261115/native-v2/captures/08-reopened-framed.jpg)

The separate SVG workflow exercised cancellation with a pending numeric edit, atomic
import, one-step Undo/Redo, control editing, invalid-input retention, SVG export
and an actual quit/relaunch/reopen/resave. Independent checks verify geometry,
physical scale, stable IDs and native persistence. The retained earlier v1/v2
strict failures remain separate from the prospectively declared v3 verification;
see the [evidence packet](../parity/reports/artifacts/native-svg-20261115/README.md)
and [current status](../STATUS.md).

## Historical workspace capture (b760)

The following four captures retain their original tested source
`b760b990b142bb46b2481434fead9e7e0178ae0f` and evidence limitations.

![Editable shirt patterns beside the simulated garment](../parity/reports/artifacts/studio-preview-20261114/captures/current/workspace.jpg)

The document bar provides open, save, recovery, exports, history and command
search. Pattern tools sit above the two viewports. The inspector exposes
placement, grain, fabric and sewing properties; the transport and diagnostics
stay below the viewports. The pictured Harbor fixture has two editable pieces,
eight seams and 654 simulation vertices after assembly and draping.

## Historical editable ellipses and cubic controls (b760)

![An ellipse with its selected cubic boundary and control handles](../parity/reports/artifacts/studio-preview-20261114/captures/current/ellipse.jpg)

Numeric ellipse creation uses four editable cubic Bézier boundaries. Selected
curves expose their controls and split parameter. The same validated operation
is available through command search, the CLI and the local MCP adapter. This is
a cubic approximation, with no persistent exact-conic constraint after editing.
See the [ellipse contract](../spec/features/PAT-003-original-ellipse-20261114.md).

## Historical ordered sewing and easing (b760)

![Ordered sewing spans in the native inspector](../parity/reports/artifacts/studio-preview-20261114/captures/current/sewing.jpg)

Ordered M:N groups pair authored boundary spans by cumulative arc length.
The inspector supports partial and reversed spans, ordering and explicit
correspondence instead of pairing mesh vertex indices.

![The four-knot easing map dialog](../parity/reports/artifacts/studio-preview-20261114/captures/current/easing.jpg)

Easing-map knots control correspondence along the two sides. This capture shows
the actual editor dialog on an original five-piece sample, then cancelled
without changing the saved construction.

## Historical preview evidence and remaining work

The earlier gallery establishes the observed b760 layout and displayed authoring
tools at that source. The [retained preview receipt](../parity/reports/artifacts/studio-preview-20261114/PREVIEW-RECEIPT.json)
binds the images to the observed executable and records earlier failed layout
attempts and an undeclared play event. The saved examples are available in the
[document directory](../parity/reports/artifacts/studio-preview-20261114/documents).

The preview loaded existing sample documents. The separate empty-project garment
acceptance remains a different workflow. A zero displayed contact value does
not establish surface clearance: independent checks found triangle/capsule
penetration in the Harbor results. Cloth convergence, full contact coverage,
calibrated fabrics, production rendering, avatar import, animation, advanced
construction and cross-platform launch remain incomplete. See the
[complete roadmap](../ROADMAP.md) and [current status](../STATUS.md).

The original preview checks used shared compiled-cache links. Their raw results
are retained with that qualification. Fresh isolated formatting, workspace tests,
strict Clippy, xtask verification and packaging all passed, reproducing the
observed native executable byte-for-byte. The
[build qualification](../parity/reports/artifacts/studio-preview-20261114/BUILD-ISOLATION-QUALIFICATION.json)
records the finding and correction. GitHub Actions and Pages remain disabled
during development.
