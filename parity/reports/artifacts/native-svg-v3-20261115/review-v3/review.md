# Native SVG v3 independent closure review

The newly declared v3 native authoring/history/persistence flow is supported,
with **two input recoveries and one failed observer assertion retained**. Both
prospectively frozen static graders passed. V2 remains strict failed; this is a
separate case with the allocator29 expectation declared before UI execution.
This review establishes no full SVG/native/physical/reference or release parity.

Final PLAN SHA256 is295687144d78469011af49cddef61550203eb6e2ddbd9b27b357c7e7212a878f;
its finalization and both checker freeze timestamps precede input000. The closed
275-file payload inventory matches
b15fab894fc456addab9530e086ad200af1257a932a8f91589f5d6ff12d02d49.
I verified228 content hashes and exact membership;47 privacy exclusions were not
reread/hashed in this closure pass. Required matching-v3 fields observed earlier
in the two quit inventories match the now-closed inventory digests. The packaged
native matchesef4e1a0f...58b76a4e and all three original inputs match PLAN.
Source45 unchanged remains root's prelaunch/postflight binding assertion; no
source inspection, builds, graders, physics or native UI were run by this review.

All75 declared inputs000–074 have ordered before/returned pairs and no recorder
failure. The recorded flow contains one SVG commit026, Undo034, Redo038, cubic
Return048, CmdQ064 and two bundle launches000/065. Additional chooser/correction
and Save As calls remain counted. No undocumented Reset, extra commit, second
quit, third launch or simulation input was found among those records.

| Checkpoint | Observed native/saved result |
|---|---|
| Cancel019 / Frame020 |rev1,one piece,pending.125 retained; Frame changes view only. Source review visibly shows2 patterns/2mm per SVG unit.|
| Import026 / saved033 |rev2,three pieces,width.125,next_id29.|
| Undo034 / saved037 |rev3,one piece,width.1,next_id29; all other baseline fields/order/f64 values exact.|
| Redo038 / saved041 |rev4,three pieces,width.125,next_id29; differs from imported only in revision.|
| Cubic048 / saved052 |curve17 first controlX.08→.09 and rev5 only; the pre-save capture06 correctly still shows an unsaved edit.|
| Rejection060 / saved063 |explanatory unsupported-transform error,rev5; retained project exact.|
| Quit/reopen/resave |settled zero-match inventory before065; fresh Empty0; Open070/resave073 atrev5 exact. Final074 is paused Rest.|

Edited, rejected and reopened files are byte-identical, SHA256
37ddcb52789271eb64bb2e58f8f7b1925bbadecab695a92edbbb6150b1082a3c.
The unchanged original geometry grader receipt isfb5fdf27538fd8b1c44c090218e8701ee612b2fa6b2d807eea74549e45216f9e;
the prospectively frozen history supplement receipt is
45cd66972dc3759b297a2dfe8e8c88598722d3191917e9898831521098f9019e.
Those receipts check actual saved geometry/export/history; native execution is
supported separately by this raw trace,45 safe main-app AX bodies and10 viewed
own-app JPEGs. No absence claim is made from provider AX clipping.

The following qualifications must accompany any publication:

- Input008 left malformed pending text `0.0.125`.010 canceled the chooser;
  011–013 selected all and corrected `.125` without Enter. Revision1 survived;
  the valid planned Cancel/Frame/Import sequence followed without a document
  reset/reload. This recovery is retained, not described as an error-free input.
-029–030 supplied an absolute filename to Save As, producing a personal colon
  basename. The accidental native file remains private;031–033 saved the correct
  imported basename with revision2 unchanged. This was an input-name recovery.
- The immediate inventory at02:59:39.177 was22ms after CmdQ returned and still
  listed v3. The observer assertion failed. `quit-confirmed.json` therefore does
  NOT prove termination. The settled02:59:53.217 inventory has zero matching v3
  apps, before the03:00:22.677 fresh launch; no additional quit input occurred.
  The later-written failure marker remains retained and
  `all_observer_assertions_passed` remains false.
- The allocator readback is explicitly timestamped02:58:20.016, after Redo save
  and before cubic Return048. A read before Undo was not executed or backdated.
  The frozen PLAN required the postcheckpoint exact checker, not the late
  followup's earlier requested read time.

The raw directory is **not safe for verbatim publication**. Exclude OS chooser/
blank AX, raw chooser JPEGs07/10, the private colon artifact/caption AX030,
.DS_Store and unrelated application inventories. Raw trace/receipt/inventory
paths also need a separate privacy-safe projection linked to immutable private
digests. Use only matching-v3 count/timestamps from quit inventories. Neither
excluded screenshot was viewed, no screenshot was edited, and JPEG embedded
metadata was not inspected.

No new product functional mismatch or missing declared input was identified.
Claims of no recoveries, all observer assertions passing, an early allocator
read, immediate quit confirmation or full parity would be unsupported.
See v3-review.json for exact comparisons, action timing, hashes and exclusions.
