# Prospective native transform acceptance

This packet fixes two separate native cases before any candidate/native output was read. It derives expectations from the original R37 fixture, the reviewed PAT-004 contract and the native selection clarification. It reuses the SHA-pinned, unchanged independent evaluator's matrix/curve/bit checks. Reference behavior remains unobserved. Exact final native source/binary bindings and launch authorization are pending root review; supply them in a separate `ROOT-BINDING.json` without editing the frozen `PLAN.json`.

A has a 900-second/90-paired-call cap: staged raw `1e-` width and Scene-other-piece refusal; original width `1.25` no-op; changed modal cancellation; finite U `1000` rejection without lost modal drafts or R37 change; explicit quarter transform with Tab between fields and one final Apply R38; native Save As; Undo/save R39; Redo/save R40; New Empty then reopen/save R40 in that same process. The fixed transform is `(u,v) -> (1.25-v,u)`. Undo must recover all original authored bits except revision; Redo and reopen must preserve all quarter fields exactly. Screenshots and paired trace establish the intermediate native behavior; saved files establish the actual authored result.

B uses a fresh process, successful A Redo R40 and a 600-second/60-paired-call cap. Enable Snap 10 mm and perform one boundary drag with exact snapped displacement `(0.10,0.05)` to R41, then one visible positive approximately 45-degree handle drag around the displayed analytic bounds center `(0.725,0.175)` to R42. Fit only the proper rotation angle from saved endpoints 3 and 4, require 43–47 degrees, and use that angle to check every remaining endpoint/control/sample, the fixed pivot, zero extra translation, world surface and grain. Scale, shear, reflection and unrelated changes fail. This is a coarse real-device gesture check; the strict 1000 px/m, >=100 px-radius `2e-6` mapping proof remains native event-test coverage.

Before each launch, the actor must import the known recorder using `createRequire` from `node:module`, pin its SHA and prove an offline before/returned pair plus a raw-artifact SHA receipt. Do not assume `require` or `process` globals in CUA. Respect CUA's mandatory first-invocation API rule, pairing bootstrap externally if needed; unsupported actor-runtime imports stop the case. Record every native API call including bootstrap/launch/snapshots/waits/dialogs/quit. Preserve raw screenshot bytes in their real format; do not crop, annotate or re-encode them. Use observed targets, not guessed selectors/coordinates. No explicit OS `focusedAXTextField` gate is required.

Keep the original fixture primary and actor-owned read-only input primaries byte-identical; keep their directories writable. Record actor-owned autosave/recovery sidecars with SHA/size/origin before and after saved states. Sidecar creation during authoring is allowed. B's read-only primary must be byte-identical to A's successful Redo. Preserve source/binary/input hashes through each case. Any unexpected error, ambiguity or cap stops that case unverified; there are no retries, repaired fixtures or weaker criteria.

Run the offline bad-output check before root binding or candidate output:

```sh
python3 .local/native-transform-plan-20261115/grade_saved.py selfcheck
```

After actual native actions, grade only their own saved files:

```sh
python3 .local/native-transform-plan-20261115/grade_saved.py grade-a A_READONLY_INPUT QUARTER UNDO REDO REOPENED
python3 .local/native-transform-plan-20261115/grade_saved.py grade-b A_REDO B_READONLY_INPUT TRANSLATED ROTATED
```

`passed_saved_output_only` is deliberately narrower than a native pass. The root must also review the predeclared trace/captures, caps, unchanged primary hashes, sidecar inventory and exact binary/source bindings. No process-restart persistence, physics/export, reference parity, normal unstaged cross-piece selection or nonempty Redo retention during refusal is claimed. The last two are separate native event-test coverage. Existing unrelated closed cases remain unchanged.

Skipped: CUA, app launch, Cargo, source edits, reference control and publication. Risk: UI feasibility and recorder loading in the actual actor runtime are unverified until the authorized bounded case; this ignored local packet is not delivery.
