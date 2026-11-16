# Cut & Sew actor route review 001

Status: **prospective route changes required before freeze; no native execution or GO**.
Reviewed the draft packet at `REVISION-002.json` against the sealed application's
own UI source and the approved engineering contract. This review writes only this
new receipt and its hash inventory. No GUI/CUA, case clock, controller constructor,
application, evaluator, Cargo, Git or case staging was invoked. Source tests below
were read, not run. The findings do not establish native behavior, reference
parity, simulation, mesh or physics acceptance.

## Scope and identities

- Draft: `.local/cut-and-sew-native-plan-20261116`, revision 002, not frozen.
  PLAN SHA256 `9ab5a3f4ff302ee1d6313a3ec1d7ad799ee8d679c422b6ea3189bb08e78161d4`;
  revision receipt `06e461c16842dedfa70eeeee2d13ddb77326dd14e2accf6f54fd1c4d16240f09`;
  inventory 002 `430c378e72b461bd36b09cc9eb72cfed41b83f9a7cb919b9d8f5ece94a8783ce`.
- Sealed source: `.local/straight-cut-and-sew-20261116`, runtime
  `5898f1a1274f8632d96b5fd0898f8253dee2f4db`, clean metadata
  `3858848afb3764014a5d305a0e5dfb5661ea6d86`, 74 source inputs. These commits and
  full gate results are root-supplied identities, not new Git/build verification.
  The UI file read for this review is `apps/weftforge/src/main.rs`, SHA256
  `e561485f666197368aad44c302a2216635c5ff2afdb08a6cfc4bfa77d4e4bb44`, matching
  the packet source inventory.
- Bound native identity remains root-supplied
  `7809e41948e19d0055da45e10a3b55225173df48e43465fab1b48effd7efa1c8`,
  62,245,032 bytes. Runtime gates receipt is `34db1fa96460c6468cb4c6ea93da198dcdac6907e9016dd2ff5dc039ab945599`;
  clean xtask/build receipt is `fd8e69195e15c996e668107a5a5f0012c99346fecb549347e01b06d184d77f5a`.
  No substitution from the separate SVG-fix worktree is authorized.
- Requirement sources: approved `spec/features/SEW-001-cut-and-sew.md`, SHA256
  `9420f9a88ea184868aba5543f0d356181fcd8373dadfa4632376cc4a5a5f8941`, and
  the original private proposal `.local/cut-and-sew-contract-20261116/CONTRACT.md`,
  SHA256 `57a121c85f599cc41931dc32993e57f29541c1ea161f6286f391db2ecd4a90f3`.
- S1 must copy only actual accepted H1-002 `snapshots/parent-r1.weftforge`, SHA256
  `525855265074aed8ff1af10297c353973cc294a3d83fca818161ce702ba7310f`, read-only,
  with that attempt's root binding, sole grade and UI acceptance. This review did
  not parse or generate a native project. S2 remains blocked on actual accepted
  same-attempt S1 `redo-r4`, not an expected state or a different attempt.
- H2-001's sole failed SVG canvas assessment remains failed and unaccepted. It
  supplies no S1/S2 dependency. All historical cases and packets remain untouched.

All source line references below refer to the sealed `main.rs` hash above.

## Changes needed before freeze

1. Resolve both symbolic eight-operation Cut openers to the explicit normal
   Return route below. Fresh drafts default to curves 7 and 8, not 7 and 9.
   The End boundary combo must explicitly select curve 9.
2. **Move the four shortcut shields into the first, invalid-field review.**
   Running Space immediately after clicking the checked `Sew cut edges` checkbox
   can toggle sewing off. The source explicitly suppresses Apply for a focused
   checkbox, and its keyboard test toggles Enter on / Space off (3613–3614,
   16438–16467). The root approved this no-extra-operation ordering during review:
   raw `1e-` → Left → refused Apply → no-refocus `0` witness `1e0-` → four shields
   while the invalid text field remains focused → full correction → Cancel.
   The second review is Sew-on → one Apply, with no Space in between. A Space
   may change raw text; claim only modal/stopped/no accepted-state change during
   shielding, then replace the whole field with the known correction.
3. Replace vague seam-row actions with Properties → `Sewing` inspection and the
   seam-scoped `Remove` button. There is no selectable Scene seam20 row in this
   source. Do not confuse `Reverse paired edge`, `Reverse`, `Remove group`,
   `Delete selected` or `Delete owner piece` with removing seam20.
4. Replace S2's four-operation final inspection with eight operations: select2,
   open its boundary combo, select chord18, then reselect15, each input followed
   by capture. Revised S2 static upper bound is 72, leaving reserve3 under75.
5. Spell out one fresh Save As and three ordinary Saves/four opaque copies in
   S1, and one fresh Save As plus one Save/two opaque copies in S2. The OS chooser
   basename and folder must come from current observations; native source seeds
   `garment.weftforge`, not the intended test filename (1718–1745).

These are procedure/label/count changes only. They do not change the model,
tolerances, IDs, expected saved states, case caps or sole-grade rule.

## Source-backed controls and focus behavior

| Purpose | Exact control / behavior | Source |
| --- | --- | --- |
| Owner selection | Scene contains selectable `{piece.name} · #{piece.id}` rows; selecting a piece clears an existing curve selection under the normal selection path | 2509–2583; 1266–1298 |
| Cut tool | Rail button `Cut piece`; menu alternative `Create 2D` → `Cut piece` only enters the tool | 39; 2122–2135; 2253–2285 |
| Precise opener, not a runtime fallback | Toolbar `Cut piece…` calls `open_cut_piece` | 2231–2238 |
| Normal Return opener | With Cut piece active, no pending numeric edit, no popup/modal/search and no keyboard-input editor, normal Return calls `open_cut_piece`; opening consumes Return before showing the new modal | 3502–3547; 5439–5447; 5514–5535 |
| Exact boundary address | `Start boundary`, `End boundary`; selected text `Curve #<id>`; options `Boundary <ordinal> · curve #<id>` | 3597–3606 |
| Raw decimal fields | `Start t`, `End t`, raw single-line TextEdit, not a numeric ID editor; command parser trims text and parses finite f64 | 499–537; 3607–3612 |
| Fresh draft defaults | Start is selected exterior or first exterior; End is first different exterior; t values both `0.5`; `sew_cut_edges: false` | 3511–3531 |
| Candidate preview | Key includes curves, raw strings and checkbox; computes an Editor from a cloned project | 538–559 |
| Checkbox | `Sew cut edges`; focused or clicked checkbox clears modal `enter`; pointer/keyboard interaction must leave the review open | 3613–3614; read-only test 16438–16467 |
| Preview labels | `Retained: …`, `New: …`, `Sewing: none` or `Sewing: Cut seam #20 · #20`; direction text `A #18: t 0 → 1 · B #19: t 1 → 0` | 3623–3639 |
| Length presentation | `A 360.000 mm`, `B 360.000 mm`, `B − A +0.000 mm` for the contracted straight candidate; ordinary UI is in mm, not `.36 m` text | 755–780; 3631–3636 |
| Refuse/correct/cancel/apply | `Apply cut`, `Cancel cut`; failure re-requests the last t-field focus; cancel keeps construction; success selects new child, clears curve and changes to Select | 3646–3707; 3549–3561 |
| Applied seam | Properties `Sewing`, label `Cut seam #20 · #20`, separate A/B interval rows, `Reverse`, `Remove`, `Applied rest length · revision …` | 2806–2835 |
| Ordinary lengths may initially be pending | `A — mm`, `B — mm`, `B − A — mm`; visible seam feedback populates cache and requests repaint | 768–780; 2825–2834 |
| S2 removal | Only seam20's `Remove` invokes `Command::RemoveSeam { seam }` | 2823; 2933 |
| Child height | Second numeric value of selected child15's `Size m`, not creation `W / H m` or placement; draft dimensions are bounds `[width,height]` | 671–674; 710–720; 2751–2754 |
| Numeric commit | One Tab/lost-focus commits the changed inspector draft; don't add a second commit gesture | 2950–2959; 6104–6134 |
| Retained anchors | After choosing retained2 chord18, inspector shows `P0 #11 · … m` and `P3 #13 · … m` | 2638–2645; 2688–2699 |
| History | Literal top-bar `Undo` / `Redo` buttons call history; no reliance on text-field shortcut focus | 2186–2193; 1816–1843 |
| Save | `File` → `Save as…` or `Save`; commit pending inspector draft then native FileDialog / project save | 2082–2106; 1718–1745 |
| Quit | Real macOS application Quit is required; native `on_exit` has separate recovery handling | 5418–5428 |

Source tests `cut_opening_return_without_draft_only_opens_review` and
`cut_opening_return_after_first_anchor_only_opens_review` read at16638–16732
exercise ordinary key input after tool activation; they are not actual native
focus evidence. The prospective actor must observe the intended control/focus
and state at every required capture. If Return fails to open the review, or any
focus/route differs, stop; do not substitute the precise button or a mouse pick.
Likewise, a later Return outside checkbox focus can Apply; use only the single
declared `Apply cut` acceptance below.

## Declared S1 route and recount

Use current AX labels/indices for exposed buttons and combo choices. Every raw
t-field focus is ONE coordinate click derived from the current raw screenshot,
then capture before select-all/type. No old pixels, titlebar offset, numeric AX
focus fallback, retry, drag, hidden state or expected-answer calculation.

`field5` = focus/capture2 + select-all1 + literal type/capture2.
`input2` = one input plus capture. Each capture uses the immutable 393-byte
observer with five explicit actual arguments. Standalone observations below are
real counted captures, not inspections of hidden construction.

| Step | Literal prospective sequence | Operations |
| --- | --- | ---: |
| 1 | Start13: getState0001, actual canonical FS proof0002, independent cross-cell readiness0003, external open0004, exact own PID0005, getApp0006, initial capture0007, at most3 startup wait/capture pairs | 13 |
| 2 | `Frame all`/capture; Scene owner2/capture | 4 |
| 3 | Rail `Cut piece`/capture; normal Return/capture, review only; End boundary combo/capture; `Boundary 3 · curve #9`/capture. Observe Start `Curve #7`, owner2/R1 | 8 |
| 4 | One additional declared capture: fresh unchecked `Sew cut edges`, `Sewing: none` | 1 |
| 5 | Start t `0.375` field5; End t `0.625` field5 | 10 |
| 6 | THREE pointer clicks on the currently observed `Sew cut edges` checkbox, with capture/inspection after each: on → off → on. No keyboard-toggle input is part of this native case; Enter/Space checkbox semantics above remain source-only | 6 |
| 7 | Start t `1e-` field5; Left/capture; refused `Apply cut`/capture; no-refocus type `0`/capture; retain raw `1e0-` witness and unchanged accepted R1 | 11 |
| 8 | While that invalid text field is still focused: CMD+S/capture, CMD+Z/capture, Space/capture, CMD+K/capture. Modal stays open, accepted state R1, simulation stopped, no search/save/history action. Do not claim raw text unchanged by Space | 8 |
| 9 | Full Start t `0.375` field5 clears prior raw content; Tab/capture; inspect corrected candidate, not an accepted revision | 7 |
| 10 | `Cancel cut`/capture: still accepted R1; future saved grade assesses next11/no accepted allocations | 2 |
| 11 | Fresh File→`Save as…`: `cut-and-sew-r240.work.weftforge` in owned work; clean native Save capture then opaque `parent-r1` copy | 15 |
| 12 | Repeat step3 exactly, including rail activation and normal Return; fresh review, End curve9 chosen explicitly | 8 |
| 13 | One additional capture: fresh checkbox reset off / `Sewing: none` | 1 |
| 14 | Exact Start t `0.375` and End t `0.625`, field5 each | 10 |
| 15 | Pointer click `Sew cut edges` on/capture: seam20, A18 full, B19 reversed, 360.000mm each. No Space follows | 2 |
| 16 | ONE later pointer `Apply cut`/capture: accepted R2, child15 selected, curve cleared, Select; future saved grade assesses next21 | 2 |
| 17 | File→Save/captures, opaque `sewn-r2` copy | 5 |
| 18 | Scene retained2/capture, Scene child15/capture. Inspect seam20 in the Properties Sewing section using allocated scroll/capture pairs, not a seam-selection click | 4 |
| 19 | Top-bar `Undo`/capture R3; File→Save/captures; opaque `undo-r3` copy | 7 |
| 20 | Top-bar `Redo`/capture R4; File→Save/captures; opaque `redo-r4` copy | 7 |
| 21 | Scene child15/capture after history's selection policy; inspect stable final R4 and existing seam block using allocated captures | 2 |
| 22 | At most4 Properties scroll/capture pairs, distributed where required for size/sewing visibility; current screenshot targeting, no extra Frame all | 8 |
| 23 | At most3 further wait/capture pairs, only for declared readiness, including applied length placeholders | 6 |
| 24 | ONE real Quit, last native operation; VERY NEXT tool is one completion_summary only, no inventory or final capture | 1 |
| 25 | ONE mandatory documentation refresh, separately counted, no fake controller pair or clock reset | 1 |
| Total | Paired upper bound148 + unpaired documentation refresh1 | **149** |

S1 operation cap remains160, reserve11, one separate summary, overall161,
continuous1800s. These are laboratory bounds, not product performance claims.
The root approved the step8 relocation, not an unobserved focus workaround.
The declared checkbox route is three pointer clicks; native keyboard-toggle
behavior remains unclaimed. No keyboard alternative may be improvised at runtime.

## Declared S2 route and recount

Start only after root accepts the exact same S1 attempt's actual saved R4 and
binds its sole grade/UI report; real restart/reopen uses that read-only file as
the sole native argv input. Loaded documents reset selection (1567–1590), so
select rows from current observations, never inherited state.

| Step | Literal prospective sequence | Operations |
| --- | --- | ---: |
| 1 | Start13, same proof/readiness/full-argv PID conditions, actual accepted S1 R4 only | 13 |
| 2 | Scene retained2/capture, Scene child15/capture; two declared ordinary captures of visible Properties Sewing block (one can expose pending feedback and the next its repaint). Use allocated scroll/readiness pairs to reach it; inspect exact seam name/ID, A18 0→1, B19 1→0, both 360.000mm, Δ+0.000mm | 6 |
| 3 | ONE current seam20-scoped `Remove`/capture: R5, both pieces remain; no Reverse/group/delete action | 2 |
| 4 | Fresh File→Save as… `cut-and-sew-r240-reopened.work.weftforge`; clean native capture then opaque `removed-r5` copy | 15 |
| 5 | Scene child15/capture, even if already selected; verify correct child context | 2 |
| 6 | Second numeric `Size m` field: current coordinate focus, select-all, type `0.40`, capture (field5); ONE Tab/capture commits R6 | 7 |
| 7 | File→Save/captures; opaque `child-height-r6` copy | 5 |
| 8 | Scene retained2/capture; `Boundary curve · select here or in 2D` combo/capture; `Boundary 2 · curve #18`/capture showing P0#11/P3#13; Scene child15/capture restores independent child context | **8** |
| 9 | At most4 Properties scroll/capture pairs, distributed for Sewing, child Size and retained chord inspector | 8 |
| 10 | At most2 further wait/capture pairs for readiness, not added retries | 4 |
| 11 | ONE real Quit last native operation; VERY NEXT tool single completion_summary only | 1 |
| 12 | ONE mandatory documentation refresh, same continuous clock | 1 |
| Total | Paired upper bound71 + unpaired documentation refresh1 | **72** |

S2 cap remains75, reserve3, summary1, overall76, continuous900s. Draft68 was
arithmetically correct for its symbolic rows but undercounted the declared
anchor inspection by4. UI point labels use six decimal places; they do not prove
IEEE-bit preservation. Exact unchanged retained2 and changed child15 fields,
ownership, revisions and allocator are assessed only by the closed-case sole
saved grade. No source-only guarantee is made that four scroll pairs reveal all
sections on an eventual native window: if bounded visibility/readiness is not
achieved, stop under the unchanged cap, without recalibration/rescue.

## Save, provenance and terminal accounting that must remain intact

- Save As15 = File/capture2 + `Save as…`/capture2 + at most one owned-folder
  Go To Folder sequence/captures6 + ONE observed Name setValue/capture2 + ONE
  enabled Save/capture2 + ONE paired opaque FS retention1. No filename keyboard
  fallback or overwrite/recovery rescue. Unexpected chooser layout is a stop.
- Save5 = File/capture2 + Save/capture2 + ONE paired opaque FS retention1.
  Only after a clean native Save capture, retain the declared primary and
  sidecars/absence with the packet retainer; no live construction parse/grade,
  file repair, fixture construction or alternate copy. S1 labels are exactly
  parent-r1/sewn-r2/undo-r3/redo-r4, S2 removed-r5/child-height-r6.
- The accepted native input, fresh owned bundle/default target binary, all74
  source hashes and integrated receipts remain bound. All20 helpers must be
  initialized with program-top-level `var`, never hidden in try/IIFE; retain
  canonical `native_cua_runtime` actual FSproof0002, the independent actual
  next-cell0003 closed-schema/helper/context/proof/opaque/status flags before
  any launch, and all six controller config keys including recorderPath.
- External0004 `/usr/bin/open` uses bound own bundle plus sole input argv.
  External0005 records BOTH `expected_binary` and `expected_command` equal to
  the entire binary+argv joined command, and accepts exactly one positive live
  matching PID. Binary-only PID matching is insufficient. No getApp before that.
- Continuous clock begins before first standalone getState and never restarts.
  Paired count plus unpaired documentation refresh stays within the operation
  cap; summary is one additional call. Every further required refresh consumes
  remaining reserve, not an invented native pair. Startup/further readiness and
  inspector scrolls are bounded declared allowances, not unlimited retries.
- First ambiguity/error/cap barrier: the VERY NEXT tool call is exactly one CUA
  completion_summary, with no further observer, input or Quit. Success likewise
  has real Quit last then immediate summary only. Root verifies asynchronous
  exit separately after raw retention; no in-case inventory after Quit.
- Retain immutable original raw evidence, honest tool-history/summary bounds
  and guards before root's sole closed-case grade and independent native review.
  This source route review neither freezes the packet nor grants a case lease.

Disposition: **request a prospective draft route/count revision before freeze**,
then independent verification and root final binding/GO. No implementation
change is requested by this review, and no old attempt is repaired or regraded.
