# Research method

WeftForge is independently authored. This investigation uses public documentation,
normal observable UI, and installation identity/header classification. No
proprietary source, disassembly, pseudocode, presets, avatars, or garment samples
were used as implementation inputs. Reviewed contracts are in `spec/features/`;
raw research stays in a separate operator-controlled directory.

## Tool identity and readiness

REA CLI **6.9.0**, skill **62**, and existing Node **26.12.1** were used on macOS
27.2 arm64. The default shell's Node 22.11.0 and Homebrew Node 26.5.0 were too old
for that REA release. Selecting the existing compatible runtime fixed CLI startup;
no Node installation or global runtime change was required.

No REA tools were connected initially. Following the [REA installation workflow](https://github.com/morluto/rea/blob/main/docs/getting-started/installation.md),
the investigator ran the pinned package from a trusted tooling directory outside
the target, inspected `setup --client codex --dry-run --json`, then applied that
same Codex-only scope. Setup appended the REA registration, retained the old
configuration byte-for-byte and wrote its backup. The already aligned skill was
not replaced. `doctor --client codex --json` then reported healthy registration;
that does not establish a live MCP connection in an already running client.

The registered launcher uses the selected Node and pinned package cache entry.
Keep those available or repair the registration through another reviewed setup
plan. Restart/reconnect the client to load the new MCP registration. CLI work can
continue without that restart.

`doctor --provider ghidra --json` reported no configured Ghidra installation.
Setup did not install an analysis engine. An existing Hopper launcher was detected
but not invoked. Neither engine is needed for the bounded installation scan.
REA and its prerequisites are laboratory tooling, never WeftForge runtime
requirements.

## Evidence types and limits

| Type | What it can establish | What it cannot establish |
| --- | --- | --- |
| Official manual | Documented behavior at the cited page/date | Current-build execution, hidden defaults, completeness |
| Hash/signature/plist | Identity, provenance, declared packaging metadata | Licensed access or authoring correctness |
| REA installation scan | Bounded file/header observations | Loaded code, missing features, executed workflows |
| Selected native UI observation | The selected process/window state at capture time | Other windows or actions not recorded |
| Controlled reference run | Outcomes for its fixture/actions/settings | Whole-product parity or wider numerical validity |
| WeftForge-only check | Internal behavior against an independent expectation | Reference equivalence |

The installed-app scan examined 7,427 entries and read 28,608 bytes of selected
headers; 106 child symlinks were not followed. Its directory digest covers
examined metadata/read ranges, not every byte. It reported no candidate markers.
That result does not imply the application has no native components or features.
Complete raw Evidence and its limitations were retained privately. A separate
selected-window REA observation captured the Personal sign-in screen, 12 AX nodes
without truncation, and its matching arm64 executable hash. An initial attempt to
select the modal-layer window returned a retained `window-mismatch` failure; the
subsequent run selected the discovered main window. Those historical captures performed no authoring actions and did not observe
the garment workspace. After account-holder activation, the 2026-11-14 workspace
capture independently confirmed the full `2026.1.237 (r58920)` build and garment
workspace. Raw account/commercial UI remains private. Native menu presence and
selected Graphic Options are now observed; complete authoring workflows remain
pending.

## Bounded investigation loop

1. Select a stable feature ID and one uncertainty. Define the observation that
   would resolve it, the stopping point, and the expected failure signal.
2. Reuse matching evidence by target/build/input identity. Make a minimal original
   fixture with explicit SI conversions, source hash, initial settings, and seed.
3. Record exact reference actions and outcomes, retaining errors and incomplete
   observations. Declare numerical metrics/tolerances before comparison.
4. Produce a reviewed behavior contract. Separate observed facts, manual claims,
   supported inferences, intentional differences, contradictions, and unknowns.
5. Give implementers that contract and permitted fixtures, excluding restricted
   artifacts. Implementers author code independently; no direct translation of
   recovered code is part of this method.
6. Run the equivalent WeftForge scenario at a recorded commit. Use an independent
   evaluator and external parser/consumer for export checks. Retain discrepancies.
7. Update reference, implementation, and verification states separately. Changed
   inputs, relevant code, baseline, or settings invalidate affected evidence.

For native UI work, discover the actual running PID and selected window ID first.
Inspect the installed command's help/schema. `observe-native-ui` captures that
existing window; `capture-native-ui-scenario` supports declared Accessibility
steps. They neither activate a license nor cover every canvas gesture. Use a
native driver or documented human steps for unsupported gestures, with consistent
before/after evidence. macOS permissions and helper failures remain explicit.
The Linux desktop-capture workflow is not a macOS recorder.

## Retention and publication

Every raw record has a private location, timestamp, producer/version, SHA-256,
source identity, settings, and limitations in the research manifest. REA summary
records link to complete retained Evidence IDs. A CLI process's retained state is
not durable by itself: save the complete bundle during the run. No active binary
analysis session was opened in this investigation.

Publish concise redacted findings and source links; do not publish temporary
signed URLs, credentials, account data, commercial assets, binaries, or local
personal paths. Preserve unknown, blocked, failed, partial, and stale outcomes.
A manual inventory, classifier, screenshot, implementation test, or issue closure
cannot by itself qualify a verified-parity claim.
