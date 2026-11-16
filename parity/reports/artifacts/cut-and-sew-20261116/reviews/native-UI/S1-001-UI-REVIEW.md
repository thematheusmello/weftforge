# S1-001 independent native UI review

**The required visible S1 route is observed. No actual UI barrier was found.**
This supports bounded SEW-S1/S1-001 UI acceptance through the saved R4 and final
Quit, subject to root's canonical acceptance record. It does not establish S2
reopen/edit behavior, full-feature/reference parity, exact construction or physics.

I individually viewed **all 59 original JPEGs**, read every paired original AX
text and all **132 operation input/result pairs**. The inventory records all
297 capture-directory files, 264 original trace files, complete AX strings/node
lines, initial getApp AX baseline, original image/trace buffer equality and raw
hashes/modes. No image was modified, cropped, regenerated or substituted.

The binding is `80a45a09e7439792eddfc31be7000dfe9033760fad11a1baa8c00e4fe02ea807`,
closure `eca65faa200f595f0ed17b4f8356ed5d86e53b0894b9fbe2c4805489fb6d3b45`,
and summary `bf496ab69b069811a2c9816c605921107003daa644c16dd431cb39a37068f0d3`.
The frozen PLAN/README match packet freeze
`5ab3b3238d2b50a5f6fdfa58965c94a7a057b5f5cb084c46117e949d986f3b0c`.
The case remains on runtime5898/metadata385/native7809; this review did not read
product implementation or infer UI success from its source tests.

## What the original evidence shows

| Records | Actual observation and scope |
| --- | --- |
| 0001–0007 | Standalone getState, actual-CUA proof and independent twenty-helper readiness precede exact launch. PID72087 matches the owned bundle binary plus complete accepted-parent argv before getApp/capture. The initial UI shows R1, one piece #2, zero seams and PAUSED. |
| 0008–0020 | Frame all and Scene #2 selection; rail Cut piece then normal Return opens review only at R1. Fresh checkbox is unchecked. End defaults to #8; the captured menu explicitly offers and chooses Boundary3/curve#9, with Start#7. |
| 0021–0036 | Current screenshot coordinate focus, capture, select-all and literal `.375`/`.625` are followed by three pointer checkbox clicks on/off/on. Raw parameters remain. Off shows Sewing:none; on shows Cut seam#20, A#18 t0→1/B#19 t1→0, A/B360.000mm and delta+0.000mm. Both areas and the preview remain fully visible; background stays R1/one piece/zero seams/PAUSED. |
| 0037–0047 | Literal `1e-`, one Left, one invalid Apply click, then `0` with no intervening refocus produces visible `1e0-`. The modal and error remain, with accepted R1 unchanged. The no-refocus witness demonstrates caret retention across the refused click. |
| 0048–0055 | CMD+S, CMD+Z, Space and CMD+K each have an ordinary capture. No Save dialog, accepted history change, simulation start or command palette appears. Text-local Undo/Space can change the invalid raw field; the evidence does **not** say those keys leave raw text unchanged. Sewing remains checked and the modal remains open at R1. |
| 0056–0064 | Full replacement by `.375` clears the invalid content; Tab leaves a valid sewn candidate. Explicit Cancel closes review; visible R1/one piece/zero seams and “Cut cancelled; construction unchanged” remain. Cut tool stays active, which the next opener accommodates. |
| 0065–0079 | Fresh native SaveAs is observed through every chooser frame. The GoTo sheet initially recalls an old H2 folder, but its selected path is replaced by the owned S1/work path before Return. Name is set once to `cut-and-sew-r240.work.weftforge`, captured with enabled Save, then Save is clicked once. No old destination is opened/saved, overwrite prompt, filename rescue or alternate route appears. Clean UI shows R1 and atomic-save status; counted opaque retention follows. |
| 0080–0100 | The same rail/normal-Return/explicit curve9 route opens a fresh review, again unchecked with reset `.5` fields. Exact `.375`/`.625` are re-entered. Sew-on is record0099, its capture0100 shows the complete sewn preview, and the next input is sole Apply0101. There is no Space or keyboard toggle between them. |
| 0101–0107 | One pointer Apply closes review and visibly creates R2/two pieces/one seam, selects child#15, clears boundary choice and returns to Select/edit. File Save clears the dirty indicator; counted opaque R2 retention follows the clean capture. |
| 0108–0113 | Both Scene pieces #2/#15 are independently selectable, with visible widths .090/.150 and height .360. One bounded inspector scroll exposes the full R2 Sewing block: seam#20, A#18 full/B#19 reversed, A/B360.000mm, delta+0.000mm and applied revision2. Reverse/Remove are present but not used. |
| 0114–0120 | One Undo visibly gives R3/one piece/zero seams. No seam block remains. File Save clears the dirty indicator, then opaque R3 retention is counted. This is visible history behavior, not exact-bit proof. |
| 0121–0127 | One Redo visibly gives R4/two pieces/one seam, restoring seam#20 and the same direction labels. The early R4 delta line is below the inspector clip; this does not earn visibility credit. File Save and counted R4 retention follow. |
| 0128–0132 | Child#15 is selected and a second declared inspector scroll exposes the complete R4 seam/length/delta block alongside both Scene IDs. Final clean R4 remains PAUSED. Record0132 is the returned `super+q` action and the last native pair; no later capture/input was added. |

All 132 pairs are contiguous, returned and error-null. They comprise 59
observations, 35 clicks, 17 key actions, eight text insertions, one Name setValue,
four live opaque retentions, two scrolls and six startup/provenance operations.
There was no drag, simulation input, seam Reverse/Remove, rescue or rerun.

## Visibility, AX and accounting limits

Both review modals, preview summaries, error text and Apply/Cancel controls fit
within the original 1640×1032 window. Native chooser JPEGs are separate smaller
surfaces; their AX values corroborate the full folder/basename where pixels are
horizontally clipped. R2 and final R4 seam summaries are fully visible after the
two counted scrolls. The initial R4 clipped delta is explicitly excluded from
that visibility finding.

There are 36 unchanged AX returns and 23 diffs. In particular, AX does not expose
many changing cut-modal controls and reports “no change” while their pixels
change. The original JPEGs are decisive for checkbox/raw/caret/preview states;
this is an AX coverage limitation, not proof of unchanged visual state. The
coordinate route was declared and completed. No keyboard checkbox-toggle or
accessibility-completeness acceptance is claimed.

The one required documentation refresh is retained as `rewriteDocumentation`
only. Actual accounting is132 paired operations +1 refresh +1 summary =134,
within overall cap161. The continuous launch-to-summary-return upper bound is
1070.583932333s, below1800s; it is a laboratory bound, not a product speed result.
The lower marker is not claimed as the exact summary invocation time.

The fourteen retained actual-session excerpt lines are hash-linked to their
descriptor. They show the final Quit call/return followed by the single
summary-only call and returned acknowledgement, and the three actual startup
cell hashes agree with their staged identities. Root's completed canonical
history corroboration `fd5fa2b3…` and POSTFLIGHT `e7873d6b…` bind this closed case.
The separate root cleanup receipt `a953c7cb…` records zero WeftForge windows and
PID72087 absent after the case; it is not inserted into the132-operation evidence
or treated as an extra native observation inside the case.

## Raw preservation and separate saved result

Final actor manifest
`fcda8bf7a6c4b7d05024b5f406652c4b9d54b19ffaeedac3457abb5825710b0f`
binds681 self-excluded original files. I rehashed all681 entries and checked their
modes; all match, including every reviewed capture and trace. The actor guard
`c5e589eee59225dbb0058d2d81d0a7aae42833abafbf1752fb38cd5bbc2c96db`
records259 unchanged pins, source74/native/HEAD385 unchanged and no trace errors.
Those are retained guard/provenance findings, not new geometry or source tests.
Root later corroboration/grade files remain separate additions to the raw manifest.

After the complete visual review, root supplied its sole formal report,
`SAVED-GRADE-001.json` SHA
`174cb2e6540edf4934a2211ef775802d50857951cd1b3de5ac0a190527f8512c`,
with `passed_saved_files_only` and UI/reference/full-feature flags false.
Invocation receipt SHA is
`2dfbe47e6c11916ebde3a422279f21aadd6b5ba41c4eeb13ec93120a2daea224`.
I bound their hashes and read report identity/status metadata only; I did not
rerun grading or parse any S1 saved construction. The UI observations above were
completed independently of that saved result. Allocator high-water, exact IDs/
coordinates/unchanged fields/recovery bytes and analytic geometry belong to the
separate saved assessment, not to screenshot inference. Native mesh/stitches,
Cloth particle mass, simulation/drape, reopen/S2, reference and full-feature
acceptance are outside this UI review.

Only filesystem text/image reads and opaque hashing were used. No native/CUA,
product/source/evaluator/grader/Cargo/Git execution or old-case mutation occurred.
Only this private review and its inventory/hash records were written. Root owns
the final bounded S1 UI-acceptance record and any later S2 authorization.
