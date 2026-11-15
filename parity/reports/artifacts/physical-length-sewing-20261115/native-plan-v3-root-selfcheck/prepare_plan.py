#!/usr/bin/env python3
"""Freeze native acceptance preparation; never invoke product, Cargo or CUA."""
from pathlib import Path
import copy
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
LAB = REPO / ".local/physical-sewing-independent-20261115"
UI = REPO / ".local/physical-sewing-20261115"
CONTROLLER_SHA = "9debdc6b1d42b49dce16c8b1227df0621b579532ca7c5fd76c3a8a5db537790b"
RECORDER_SHA = "45707351f12007967c3803a7551a262607fd68ff2939a2793f7c4c86921329fc"
PINS = {
    "fixture.weftforge": "239009c6ae23da35d4e6fd12e53dfc07df12b721f9eb841a841f9cc30ec39831",
    "CRITERIA.json": "d748d52d5aee5bee2d57c02e81edd137c8e50ab5c9bea9f69afb1c27c31d2938",
    "evaluate_physical_sewing.py": "ac252d08d820f7aedac771502486a54ec2dc4242017911588d5e39fcaa38a384",
}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def dump(name, value):
    (ROOT / name).write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def main():
    for name, expected in PINS.items():
        raw = (LAB / name).read_bytes()
        assert sha(raw) == expected
        (ROOT / name).write_bytes(raw)
    sources = []
    paths = [
        ".local/physical-sewing-20261115/spec/features/SEW-002-physical-length.md",
        ".local/pattern-transform-20261115/spec/features/APP-001-new-empty.md",
        ".local/physical-sewing-20261115/apps/weftforge/src/main.rs",
        ".local/physical-sewing-20261115/crates/weftforge-core/src/document.rs",
        ".local/native-transform-controller-20261115/controller.cjs",
        ".local/native-length-readouts-v2-20261115/recorder.cjs",
        ".local/transform-publication-stage-20261115/native-transform-plan-v2-20261115/A3-INPUT-OBSERVATION-PROTOCOL.json",
        ".local/physical-sewing-independent-20261115/FREEZE.json",
        ".local/physical-sewing-independent-20261115/run-002-26f88e0-manifest.json",
        ".local/physical-sewing-independent-20261115/run-002-26f88e0/RESULT.json",
    ]
    for path in paths:
        raw = (REPO / path).read_bytes()
        sources.append(dict(path_relative_to_repo=path, sha256=sha(raw), bytes=len(raw)))
    assert sources[0]["sha256"] == "4016013d9dda6bf4f910e52224586925314c156b9489d3c6d4a0599411fa18b2"
    assert sources[1]["sha256"] == "ce6de41684f98772c69bf8e0117e22f5f2cdf6b15952e4394f6756dd0077deb7"
    assert sources[4]["sha256"] == CONTROLLER_SHA
    assert sources[5]["sha256"] == RECORDER_SHA
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=UI, text=True).strip()
    dirty = subprocess.check_output(["git", "status", "--short"], cwd=UI, text=True).splitlines()
    dump("SOURCE-INPUTS.json", dict(
        status="preparation_snapshot_not_native_source_or_binary_binding", sources=sources,
        copied_oracle_files=[dict(path=k, sha256=v) for k, v in PINS.items()],
        ui_source_worktree_relative_to_repo=".local/physical-sewing-20261115",
        ui_source_head_at_preparation=head, ui_source_status_at_preparation=dirty,
        ui_policy_snapshot="Free sewing; Create Sewing Line; Boundary; Start t; Length/mm; Forward (increasing t); Reverse (decreasing t); OK/Cancel/Close. Name = Free sewing {sewing_groups.len()+1}, therefore Free sewing 1 on the original empty-group fixture.",
        new_empty="APP-001 contract is an input. New Empty is being implemented separately; final merged native source, labels and all gates must be bound by root before launch.",
        prior_outputs_disclosure="Preparer previously evaluated headless candidate 26f88e0: 50/50 bounded observations. Native numerical expectations copy the earlier pre-candidate frozen oracle, not those outputs. Headless success supplies no actual-native workflow evidence.",
        excluded="No product/Cargo/CUA execution for this preparation, reference application, private reference container, vendor raw material, account data, binary or product source edit.",
    ))
    original = json.loads((ROOT / "fixture.weftforge").read_text())
    criteria = json.loads((ROOT / "CRITERIA.json").read_text())
    initial = copy.deepcopy(next(c for c in criteria["positive_cases"] if c["id"] == "initial_asymmetric"))
    initial["command"]["data"]["name"] = "Free sewing 1"
    expected = dict(
        status="prospective_parsed_authored_expectations_not_candidate_outputs",
        fixture_sha256=PINS["fixture.weftforge"], original_revision=41, original_next_id=20,
        native_command=initial["command"], independent_intervals_and_locations=initial["expected"],
        snapshots={
            "created.weftforge": {"revision": 42, "next_id": 21, "group_count": 1, "group_id": 20},
            "undo.weftforge": {"revision": 43, "next_id": 21, "group_count": 0, "note": "Allocator high-water survives Undo; require 21, never original 20."},
            "redo.weftforge": {"revision": 44, "next_id": 21, "group_count": 1, "group_id": 20},
            "reopened.weftforge": {"revision": 44, "next_id": 21, "group_count": 1, "group_id": 20},
        },
        new_empty="UI-only checkpoint: fresh Project/Editor R0, no path/selection/history/groups/seams/pieces; saved-output grader does not claim to observe this step.",
        unchanged="Envelope/version; every original point/curve/piece/material/mannequin/resolution/legacy-seam field and order. Native group is exactly Free sewing 1, one interval per side and [[0,0],[1,1]]. Entered lengths are not persisted locks.",
        history="Undo is original parsed construction with only revision43 and next_id21 changed. Redo is created parsed construction with only revision44 changed. Reopened is exactly Redo parsed construction. Native Redo/reopened saved primary bytes must be identical under the same serializer.",
        tolerances={"length_and_evaluated_location_m": 1e-8, "straight_endpoint_t": 2e-14, "oracle_crosscheck_m": 2e-12},
        primitive="F(t)=0.1*(t*sqrt(1+t*t)+asinh(t)); C(t)=(0.2t,0.1t^2). Separate frozen derivative quadrature/inverse remains unchanged.",
        guides={"start_to_start_m": [[0.0, 0.0], [0.55, 0.0]], "end_to_end_m": [initial["expected"]["a"]["samples"][-1]["point_m"], [0.45, 0.0]]},
        parsed_value_limit="Parsed equality does not distinguish signed-zero bits or JSON numeric spelling. Byte hashes prove primary preservation; no stronger authored bit guarantee is inferred.",
    )
    assert original["project"]["revision"] == 41 and original["project"]["next_id"] == 20
    dump("EXPECTED.json", expected)
    plan = dict(
        schema_version=1, case_id="NATIVE-PHYSICAL-SEWING-001", status="prospective_preparation_only_pending_root_review_and_binding",
        product_or_CUA_executed_for_preparation=False, source_snapshot_head=head,
        claim="One bounded actual-native original authored engineering journey; no reference parity or full sewing/New Empty acceptance.",
        root_binding_required={
            "file": "ROOT-BINDING.json supplied separately; do not edit frozen plan", "root_approved": None,
            "reviewed_freeze_sha256": None, "source_worktree": None, "source_commit": None,
            "scoped_source_hashes": None, "native_binary_or_owned_app_bundle": None, "native_binary_sha256": None,
            "worktree_specific_cargo_target": None, "four_required_local_gate_receipts": None,
            "merged_new_empty_contract_and_runtime_source": None, "fresh_case_directory": None,
            "input_copy_sha256_and_mode": None,
        },
        caps={"all_native_calls_including_OS_launch_bootstrap_observations_and_quit": 219,
              "completion_summary_tool_calls_reserved": 1, "all_calls_total": 220,
              "elapsed_seconds_from_immediately_before_OS_launch_through_quit": 1200,
              "policy": "No widening after outputs. Before every call use persisted FS trace count and launch clock; reserve final observation/quit/summary within total. Unexpected error/ambiguity/cap => stop, next CUA completion_summary only, no extra quit."},
        static_budget=[
            {"stage": "OS launch, documented app bootstrap, initial capture, Frame all, Scene selection, Free sewing", "calls": 10},
            {"stage": "No-op raw inspector sentinel, one invalid Length refusal/Cancel, caret insertion and no-op Tab", "calls": 36},
            {"stage": "Canonical A endpoint/right-click/Length100/Tab/OK with screenshot barriers", "calls": 13},
            {"stage": "Canonical B interior/leftward right-click/exactStartt0.75/Length100/Tab/OK barriers", "calls": 20},
            {"stage": "Four native Save As dialogs with observed Go To Folder + filename typing barriers", "calls": 72},
            {"stage": "One Undo and one Redo using Edit menu and captures", "calls": 8},
            {"stage": "File New Empty R0, one no-op Undo and Redo at R0 and captures", "calls": 12},
            {"stage": "Same-process File Open of redo primary, observed path/dialog typing barriers", "calls": 20},
            {"stage": "Planned successful quit and immediate completion_summary", "calls": 2},
            {"stage": "Reserved observation-only calls: at most8 derived-mesh/pending-label captures, additional documented dialog/frame observations; no repair/retry", "calls": 27},
        ],
        budget_justification="193 planned worst-case API/tool calls plus27 fixed observation reserve. Four Save As dialogs need at most18 calls each via the frozen observed-folder/name flow. Numeric focus/type/Tab screenshots are already included. At roughly4s/call plus200s review, 1200s is a meaningful stop cap. New Empty is observed rather than adding an unasked fifth Save As.",
        controller={"path_relative_to_repo": ".local/native-transform-controller-20261115/controller.cjs", "sha256": CONTROLLER_SHA,
                    "recorder_path_relative_to_repo": ".local/native-length-readouts-v2-20261115/recorder.cjs", "recorder_sha256": RECORDER_SHA,
                    "configuration": {"evidenceKind": "native", "callCap": 219, "elapsedCapS": 1200, "baselineCalls": 0},
                    "FS_state": "Before launch prove external createRequire/fs imports, a writable fresh case directory, pinned controller/recorder hashes and an offline before/returned+raw-artifact receipt. Prove the same FS/module bindings in the actual actor runtime before authoring; never assume FS/counter/app globals from another cell/session. Persist external OS-launch/bootstrap records into the same trace.",
                    "bootstrap": "Respect first-CUA-call documentation rule: when an initialized actor permits it, prove actor imports before launch; otherwise externally pair/count OS launch and the sole exact known already-launched app getApp entry point, then immediately import/prove actor FS/controller without another app call before authoring. Missing imports stop; never combine initialization API with extra calls. Preparation itself authorizes no CUA/product call.",
                    "usage": "Pass the current explicit app/target to every controller.call/capture; capture raw AX+JPEG/PNG with emit:false, retain original bytes/receipts and emit real images only at top level. No invented focus role, mock image or hidden target call.",
                    "shutdown": "Success only: one documented Cmd+Q/expected termination call is final app observer, then next CUA call only completion_summary exactly once. No AX read on a closed window. Failure: next CUA completion_summary only; no quit or other native call. Account PID/source/binary/sidecars outside CUA afterward."},
        observation_protocol=[
            "Use only current documented native APIs and observed controls/coordinates. No frozen screen pixels or guessed AX selectors.",
            "After initial click/doubleclick into every typed field, capture and inspect the visible field/caret/selection before Cmd+A/type. Available AX may corroborate but missing focusedAXTextField is not a hard gate.",
            "After every typeText, capture and inspect its raw text at the intended field before any Tab, Return, button, focus change or next typeText. This includes path/name fields. Never batch type+Tab without the screenshot barrier.",
            "After each Tab used for validation/focus movement, capture and inspect retained value/error and next focus before further input. Use Tab, not Enter, between sewing fields; Enter may accept a side.",
            "Screenshots must clearly show A/B stage, relevant boundary/t/direction/Length text and revision/status. Main egui modal AX may omit fields; record that limit without inventing a focused-role witness.",
            "At most8 reserved captures may wait for an explicitly visible pending mesh/readout to become the matching-revision applied labels/guides. No authoring input is repeated. Other unexpected/missing/ambiguous state stops unverified.",
        ],
        file_protocol={
            "input": "Actor-owned byte-identical readonly fixture copy, mode0444; writable parent for sidecars. Original preparation fixture never edited.",
            "outputs": ["created.weftforge", "undo.weftforge", "redo.weftforge", "reopened.weftforge"],
            "save_as_flow": ["Observed File -> Save as…", "Capture native save dialog", "Documented Go To Folder shortcut; capture field", "Cmd+A/type bound actor-owned output directory; capture exact path; Return; capture directory", "Click observed filename field; capture focus; Cmd+A/type declared basename; capture exact name", "Click observed Save once; capture saved native path/revision"],
            "open_flow": "Observed File -> Open…; same documented observed Go To Folder/directory and filename selection flow, typing only the already planned redo primary path; capture before Open and reopened R44 afterward. No Recent-list assumptions.",
            "sidecars": "At initial, post-refusal/cancel, A-only, each Save As, New Empty, reopened and exit checkpoints inventory only actor-owned primary/autosave/recovery files with size/SHA/parsed revision/origin. Do not require absence of normal authoring sidecars. A-only sidecars, if present, must contain unchanged R41/allocator20/groups0 and no draft.",
            "retention": "All four saved primaries must remain byte-identical to their own first saved checkpoint through New Empty/reopen/exit; retain before/after hashes. Readonly input and original fixture hashes never change. Unknown/nonparseable sidecars remain visible and prevent a preservation claim.",
        },
        steps=[
            {"id": "01-launch-original", "actions": "Fresh bound process via externally paired/counted /usr/bin/open -n owned app bundle --args readonly input. Select exact already-running app; verify owned PID/kernel executable/window. Capture R41,2pieces,0seams,0groups,stopped simulation. Frame all once. Select Scene Original analytic panel and Free sewing using exposed controls.", "require": "Original input SHA239009c6…39831; no accepted edits or dirty indicator. Source/new binary/contract binding is approved beforehand."},
            {"id": "02-sentinel-and-invalid-cancel", "actions": "In the selected panel's Size m width, stage raw text '0.2e0' without Tab, then Left twice to put the caret at '0.2|e0'. Capture. Open Create Sewing Line by toolbar, choose Forward, leave boundary #6 / Start t 0, type Length '1e-', capture, Tab/capture, OK once/capture. Require useful error, raw '1e-' and side A modal retained, R41 / 0 groups, no accepted edit. Cancel once/capture. Width must restore '0.2e0' and its caret. Planned caret witness: type '0' without select-all; capture exact '0.20e0', then Tab/capture. Both strings equal original 0.2 numerically, so R41 / no dirty / no geometry or history entry. This planned no-op sentinel must not add a command to canonical sewing; it is never a fix for unexpected output.", "require": "One unfinished Length case only. No negative-length second case. Refusal preserves modal input; Cancel abandons gesture and preserves the unrelated raw inspector draft/caret. If any target/text/caret-witness result differs, stop; do not correct it."},
            {"id": "03-canonical-A", "actions": "In the observed framed 2D canvas click exact curve #6 start C(0)=(0,0); capture marker/source boundary/t. Right-click forward near C(0.25)=(0.05,0.00625); capture Create Sewing Line. Require Boundary Curve #6, exact Start t 0 and Forward selected. Replace Length with '100', capture, Tab/capture, OK once/capture.", "require": "A pending 100.0 mm, side B prompt, R41, 0 groups, unchanged dirty/input/allocator. Do not hardcode or claim the seeded dialog length. No manual Start t correction is allowed for A."},
            {"id": "04-canonical-B", "actions": "Click straight curve #15 interior near B(0.75)=(0.55,0); capture. Right-click leftward near B(0.5)=(0.5,0); capture modal and Reverse direction. Planned precision route regardless of pointer parameter: replace Start t with '0.75', capture, Tab/capture; replace Length with '100', capture, Tab/capture. Require Curve #15, Start t 0.75, Reverse and 100 mm before OK once.", "require": "Exactly one group 'Free sewing 1', ID20, R42, allocator21. Matching-revision 'A applied · 100.0 mm' and 'B applied · 100.0 mm' and guides A0->B0/A1->B1. No output-driven parameter repair or duplicate OK."},
            {"id": "05-save-history", "actions": "Native Save As created.weftforge R42. Edit Undo once/capture: both sides/guides disappear together, R43/0groups; native Save As undo.weftforge. Edit Redo once/capture: same pair/name/labels/guides return, R44/1group; native Save As redo.weftforge.", "require": "Undo keeps allocator high-water21 while removing the group. Redo restores the created group fields/ID, not a new ID. Saving does not add revisions."},
            {"id": "06-new-empty-reopen", "actions": "After clean redo save invoke File -> New Empty once/capture. Require Untitled/no path, R0,0pieces/seams/groups and empty canvases. One Edit Undo and one Edit Redo at R0 are planned no-ops; capture both. Reopen saved redo.weftforge using File Open in this same process; capture R44/same pair/labels/guides. Native Save As reopened.weftforge; capture final saved path/R44. No empty-file Save As is requested.", "require": "R0 no-op history must not restore prior document or dirty the new one. New Empty/reopen cannot change any prior saved primary. Same-process persistence only; transient tool prompts alone do not imply a draft."},
            {"id": "07-success-quit-grade", "actions": "Only after all required UI observations and saved-file inputs exist, quit once through documented Cmd+Q and observe its expected close result; next CUA call only completion_summary. Outside CUA collect final hashes/sidecars/PID, run frozen saved-file grader once into fresh report and bind full trace/artifact manifest.", "require": "Saved-file pass alone is passed_saved_output_only. Native pass additionally requires root review of actual trace/captures, all stages,220call/1200s limits and exact final source/binary/input hashes."},
        ],
        saved_file_grader="python3 grade_saved.py grade --input ACTOR_READONLY_INPUT --created CREATED --undo UNDO --redo REDO --reopened REOPENED --out FRESH_REPORT.json",
        coverage_limits=[
            "Saved files cannot prove physical pointer action, invalid/modal text/caret retention, first-side transience, dirty/history/cache state, labels/guides or actual New Empty. Actual trace/screenshots are mandatory and remain unverified during preparation.",
            "A pointer endpoint and B planned exactStartt precision route do not qualify arbitrary closest-point projection or all endpoint ties, inaccessible AX modal fields, snap semantics or reference correspondence.",
            "No existing nonempty Redo branch during refusal, B cancellation, Escape/close/tool switch/stale revision, failed load/save, both New Empty routes or all unsaved-work refusals in this one case; separate source/event gates remain.",
            "No simulation/remeshing/solver acceptance, process restart, unequal easing, full sewing, reference parity or publication. Headless candidate success is separate evidence.",
            "Local ignored preparation is not delivery. Root owns later Git tracking and clean-checkout registry/map verification; no Actions/Pages change.",
        ],
    )
    assert sum(s["calls"] for s in plan["static_budget"]) == 220
    dump("PLAN.json", plan)


if __name__ == "__main__":
    main()
