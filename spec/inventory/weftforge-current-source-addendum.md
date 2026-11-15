# WeftForge current-source addendum

This additive engineering review explicitly rechecks implementation candidates at immutable source `ad7cc8bbbfc7a1ae735a400c21dcc7a37c80a5c5`. The [historical map](weftforge-implementation-map.md) remains pinned to `3eddef067c8a1f03f192c8e6822a8da2a94f9bf9` with **949 unique IDs**, **205 packets**, **56 landmark identities** and its original **22-command** snapshot unchanged. This is source mapping, not a feature denominator, completed roadmap gate, native execution or reference parity.

[Machine-readable addendum](weftforge-current-source-addendum.json). Current feature qualification, retained failures and evidence freshness come only from [parity/features.yaml](../../parity/features.yaml) and reports at the reported revision. Source test anchors identify implementation definitions; no source/product/native/reference tests were executed by this mapping task.

## Explicit source review history

The previous review was at `b657e6bf52aca6c0c76c5c51aaf76b4e812150f7`, with 27 Rust variants, 26 schema commands, nine MCP tools, 26 candidate groups and 18 native source surfaces. Its exact JSON/Markdown remain reachable at metadata commit `d670aa7562b068bf32f5385c0f8f10858dbf29f5` (identical at `201a048c38504a2a2ea8fb59a5c96e7af338683f`); JSON SHA-256 `89d13a8471f8035a9c9a67863eb0051767085b8ef40c1d842239ba593c453af2`, Markdown SHA-256 `ab5e634e2da89582dad1fde3f182c9b7de84f181af106df0e116f199858949fc`. The checked `source_update_origin` retains that snapshot and every prior pin/group/qualification. It does not imply that the B657 review ran at this newer source.

All prior current-source anchor texts still resolve, but their hashes/lines are explicitly refreshed at ad7. The changed `docs/automation.md` is re-reviewed rather than carrying its old hash forward. PAT-006 plus architecture/file-format inputs are added. Unlisted historical groups are still historical; an anchor resolving alone does not review every old scope.

**Current Rust: 29 variants = 28 nontransaction commands + `Transaction(Vec<Command>)`.** The JSON command oneOf has exactly those 28 commands. The additions since B657 are `AddInternalSegment` and `RemoveInternalCurve`; endpoint editing uses existing `MovePoint`/`Transaction`. CLI `execute`/MCP `project_execute` accept 1–1024 nontransaction commands, reject nested transactions and wrap once. The nine MCP tools remain unchanged; no dedicated annotation CLI verb or tool is added. New Empty/view/session actions remain outside authoring. These counts do not describe features completed.

Approved PAT-006 contract: `spec/features/PAT-006-internal-segments.md` / SHA-256 `84014cffd5ec0c4fc3375b552d6238bd0b7dcda7a7ac5b10d7fc383621f69d66`. Its follow-on cut and native acceptance sections are prospective requirements, not results.

## Current native source routes

| Source surface | Controls and path | Existing engineering groups |
|---|---|---|
| File menu | New Empty; Open…; Import SVG…; Import DXF…; Save; Save as…; Recover…; Export OBJ; Export SVG; Export DXF; Export help explicitly discloses omitted annotations; native v4 retains them. `apps/weftforge/src/main.rs:1871` | `project-new`, `project-load-save`, `pattern-snapshot-print`, `obj-interchange` |
| Edit/menu-right actions and search | Undo; Redo; Commands ⌘K; Command search text/filter and fixed action rows; Internal line search action. `apps/weftforge/src/main.rs:4924` | `undo-redo`, `shortcut-preferences`, `internal-primitives` |
| View/Help menus | Frame all; Wireframe; Static original interaction help. `apps/weftforge/src/main.rs:1871` | `pattern-view-extents`, `wireframe`, `help-resources` |
| Tool rail | Select / edit; Transform pattern; Polygon; Rectangle; Internal line; Sew edges; Free sewing; M:N sewing. `apps/weftforge/src/main.rs:2027` | `selection`, `pattern-transform-tool`, `polygon`, `rectangle`, `segment-sewing`, `free-chain-sewing`, `internal-primitives` |
| Context toolbar | Off / 1 mm / 5 mm / 10 mm Snap; Frame all; Transform pattern…; Ordered A/B draft summary; Create Sewing Line; Cancel free sewing; Internal line…; Cancel internal line; Owner-first/start/end annotation state. `apps/weftforge/src/main.rs:1986` | `grid-snap`, `pattern-view-extents`, `pattern-transform-tool`, `free-chain-sewing`, `internal-primitives` |
| Document tab and status | Document name/dirty marker; Revision; Piece/seam/group/vertex counts; Worker/autosave messages; Discard and close. `apps/weftforge/src/main.rs:2067` | `workspace-panels`, `workspace-statistics`, `application-quit` |
| Properties and Scene | Piece selection rows; Selected boundary endpoints; Cubic Bezier and absolute controls; Split t / Split / Join; Applied boundary length; Size; Placement translation/rotation; Warp angle; Shared fabric mass/stiffness/bend/RGB; Delete selected; Internal annotation owner lists; Internal endpoint IDs/positions/applied length; Edit internal endpoints…; Remove internal line; Explicit Delete owner piece. `apps/weftforge/src/main.rs:2256` | `selection`, `curve-tools`, `split-boundary`, `selected-line-lengths`, `pattern-transform-tool`, `arrangement`, `grain-alignment`, `fabric-assignment`, `delete-selection`, `internal-primitives` |
| Create pattern | Pattern name; W/H m; X/Y m; Add rectangle; Add ellipse; Add polygon point; Close pattern; Internal line… (selected owner). `apps/weftforge/src/main.rs:2496` | `polygon`, `rectangle`, `ellipse`, `pattern-name-property`, `internal-primitives` |
| Legacy sewing and ordered groups | Reverse paired edge; A/B t intervals; Legacy Reverse / Remove; Applied lengths and signed mismatch; Ordered A/B span controls; Confirm A / Create group / Delete last / Cancel; Edit group / Reverse B / Remove group. `apps/weftforge/src/main.rs:2546` | `segment-sewing`, `sewing-editor`, `reverse-sewing`, `free-chain-sewing`, `sewing-length-check` |
| Rectangle modal | Width (mm); Height (mm); OK; Cancel. `apps/weftforge/src/main.rs:3494` | `rectangle`, `undo-redo` |
| Transform modal | Translation U/V (m); Angle (radians); Pivot U/V (m); Apply transform / Return; Cancel transform. `apps/weftforge/src/main.rs:3360` | `pattern-transform-tool`, `pattern-rotate`, `undo-redo` |
| Free sewing modal | Boundary; Start t; Length (mm); Forward / Reverse; Tab validation; OK / Enter; Cancel / Close. `apps/weftforge/src/main.rs:2853` | `free-sewing-length-preview`, `free-chain-sewing`, `undo-redo` |
| Structured group modal | Name; A/B ordered intervals; Reverse B order and directions; Easing knot A/B controls; Add easing knot / Remove knot; Apply group; Cancel group edit. `apps/weftforge/src/main.rs:4467` | `sewing-editor`, `free-chain-sewing` |
| Pattern import review | Piece count; Physical scale/origin/layout; Explicit omissions; Import; Cancel; Exterior-only profile; imported pieces start with empty internal arrays. `apps/weftforge/src/main.rs:1453` | `pattern-snapshot-print`, `undo-redo`, `project-load-save` |
| Recovery window | Discovered recovery choices; Refresh; Choose project…; Choose recovery file…; Recover copy; Close. `apps/weftforge/src/main.rs:1678` | `project-load-save`, `autosave-preferences` |
| 2D/3D canvases | Applied outlines/points/controls; Draft and sewing guides/labels; One-piece transform pivot/handle; 2D pan/zoom; 3D orbit/zoom/picking; Wireframe; Distinct 2D straight annotation/preview and both endpoint marks; Role-aware annotation selection/endpoint drag; sewing/Transform handles stay boundary-only. `apps/weftforge/src/main.rs:3557` | `point-pattern-edit`, `curve-tools`, `sewing-display`, `pattern-transform-tool`, `viewpoints`, `selection`, `wireframe`, `internal-line-display`, `internal-primitives` |
| Simulation transport | Play / Pause; Step; Assemble (gravity off); Drape; Reset; Cancel job; Wireframe; Stage/step progress; Task details. `apps/weftforge/src/main.rs:2105` | `simulate`, `workspace-statistics`, `wireframe` |
| Mesh/simulation/diagnostic properties | Mesh edge length m; Iterations / substeps; Assembly/strain/seam/contact/intersection diagnostics; Peak pattern and material axis. `apps/weftforge/src/main.rs:2608` | `resolution`, `simulation-properties`, `workspace-statistics`, `fit-maps` |
| Create 2D menu | Internal line; Disabled/missing-owner explanation. `apps/weftforge/src/main.rs:1913` | `internal-primitives`, `selection` |
| Internal annotation endpoint modal | Create internal line / Edit internal line; Start U (m); Start V (m); End U (m); End V (m); Apply internal line / Return; Cancel internal line; Explicit no cut/fold/sewing effect. `apps/weftforge/src/main.rs:3194` | `internal-primitives`, `point-pattern-edit`, `undo-redo` |

These are original WeftForge routes mapped to existing engineering groups, not new reference catalog IDs or aliases. Internal selection/drawing/endpoint editing uses owned construction; boundary-only topology/sewing/picking remains separate. Native v4 keeps annotations; SVG/DXF export only exterior outlines and OBJ exports cloth surfaces without annotation line primitives.

## Source-only group rechecks

### `project-new` / `PACK-project-new`

File > New Empty and the New Empty command-search row queue the same post-inspector replacement action. A clean replacement creates Editor::default at revision 0 with no file path, selection, history or derived mesh/cloth, cancels the old worker and retains session view/snap settings. Dirty accepted work, pending or invalid numeric input, unfinished construction and an active autosave cause a retained-state refusal. This native session action is distinct from CLI new/MCP project_create file creation and adds no Command variant. Unfinished internal-line creation/endpoint state also blocks clean New Empty; successful replacement clears those revision/owner/target-bound previews.

Open: Reference New Project/New Garment initial-state, discard/confirmation, option and session semantics remain unknown. Read actual native qualification only from the registry/reports at the reported revision. New Empty deliberately refuses replacement until edits/gestures are applied or cancelled and saved; no discard option or new shortcut is supplied.

Source: `apps/weftforge/src/main.rs:994` (`fn new_empty_button(`); `apps/weftforge/src/main.rs:1011` (`fn new_empty(`); `apps/weftforge/src/main.rs:11334` (`fn new_empty_file_route_replaces_clean_document_and_retains_files_and_preferences`); `apps/weftforge/src/main.rs:11339` (`fn new_empty_search_route_replaces_clean_document_and_retains_files_and_preferences`); `apps/weftforge/src/main.rs:15999` (`fn internal_line_unfinished_creation_blocks_clean_new_and_escape_abandons_without_history`).

### `workspace-panels` / `PACK-workspace-panels`

The fixed native studio shell has File/Create 2D/Edit/View/Help menus, an icon tool rail, a context toolbar, document tab/revision, equal 2D Pattern and 3D Garment canvases, Properties with a Scene list, simulation transport and a status/message bar. The existing action/focus path protects numeric drafts while menus, tool choices and command search are used; authoring modals shield background input. Source layout and focus checks are implementation candidates only. The tool rail, Create 2D menu, search, Scene/Properties owner lists and precise endpoint modal now expose the original Internal line annotation route.

Open: Docking, arbitrary panel/tab visibility, independent windows, asset/material/timeline browsers and full context/accessibility contracts remain open. Source layout assertions do not qualify an actual native window.

Source: `apps/weftforge/src/main.rs:1871` (`fn menu_bar(`); `apps/weftforge/src/main.rs:2027` (`fn tool_rail(`); `apps/weftforge/src/main.rs:2052` (`fn document_tab(`); `apps/weftforge/src/main.rs:5690` (`fn studio_workspace_keeps_creation_and_transport_visible`); `apps/weftforge/src/main.rs:5802` (`fn studio_menus_and_tool_rail_preserve_numeric_drafts_text_and_caret`); `apps/weftforge/src/main.rs:3194` (`fn internal_line_window(`); `apps/weftforge/src/main.rs:3028` (`fn selected_internal_curve(`).

### `workspace-layout` / `PACK-workspace-layout`

A source-defined fixed native layout keeps menu/context rows, paired canvases, right Properties/Scene panel, simulation transport and status row visible. This is one original workspace arrangement, with no saved layout presets or docking model.

Open: Persisted dock/window layouts, display/DPI-safe restore, export/directory handling and reset without document mutation.

Source: `apps/weftforge/src/main.rs:5690` (`fn studio_workspace_keeps_creation_and_transport_visible`).

### `appearance-preferences` / `PACK-appearance-preferences`

The native shell uses a fixed dark theme, original icon drawing, bundled licensed Inter/JetBrains Mono typography and distinct selection/status colors through studio_style. Material RGB editing remains an authoring field. No user palette/theme/typography preference interface is implemented.

Open: Original accessible line/selection/hover/status colors, typography/pixel thickness preferences and bounded user palette resource handling without copying vendor palette values.

Source: `apps/weftforge/src/main.rs:5571` (`fn studio_style(`); `apps/weftforge/src/main.rs:5056` (`fn tool_button(`).

### `help-resources` / `PACK-help-resources`

The native Help menu supplies static original interaction guidance for 2D pan/zoom/snap, whole-piece transform policy, 3D navigation, play/pause and construction confirmation/cancellation. This is discoverable local guidance, with no complete documentation browser, locale routing or community/release-link model. Help also explains owner-first internal-line creation and the omission of annotations from SVG/DXF/OBJ exports.

Open: Branded discoverable original/offline documentation and appropriate external community/release links with locale routing.

Source: `apps/weftforge/src/main.rs:1871` (`fn menu_bar(`); `apps/weftforge/src/main.rs:1950` (`Internal line: select an owner, then click start/end or enter precise endpoints.`).

### `selection` / `PACK-selection`

One selected owner piece and selected curve connect 2D, Scene/Properties and existing 3D piece picking. The selected curve can now be an exterior boundary or an owned straight annotation; separate selected_internal_curve/selected_boundary_curve helpers retain the role. Select draws/picks both open endpoints, and internal endpoint drags capture the selected annotation so coincident coordinates do not choose an unrelated earlier line. Cross-piece selection while Transform/Internal line or an internal selection has staged commands/incomplete numeric text refuses and retains text/caret/history through the house selection guard. Sewing/free-sewing/ordered groups and Transform handles continue to use boundary-only picking. This is original one-piece policy, not observed reference selection semantics.

Open: Multi-object/subobject selection, select-all/invert/sewn traversal, modifier rules and 3D manipulation gizmos remain open.

Source: `apps/weftforge/src/main.rs:1078` (`fn select(`); `apps/weftforge/src/main.rs:14629` (`fn transform_cross_piece_refusal_retains_staged_or_invalid_raw_text_caret_and_redo`); `apps/weftforge/src/main.rs:14687` (`fn transform_scene_row_selection_preserves_staged_and_raw_edits_through_real_ui_actions`); `apps/weftforge/src/main.rs:1109` (`fn select_curve(`); `apps/weftforge/src/main.rs:3035` (`fn selected_boundary_curve(`); `apps/weftforge/src/main.rs:16384` (`fn internal_line_review_coincident_endpoints_drag_only_the_selected_later_annotation`); `apps/weftforge/src/main.rs:15900` (`fn internal_line_cross_piece_selection_and_incomplete_source_text_are_retained`).

### `ellipse` / `PACK-ellipse`

AddEllipse creates four editable cubic boundary spans with stable authored points/curves from a name, origin and SI width/height. Properties > Add ellipse and command search use the same shared command, combined with pending inspector edits as one transaction. CLI ellipse and CLI/MCP command arrays expose the same construction; this bounded cubic representation is not an exact analytic conic or a new persistent ellipse primitive. Current creation initializes an empty internal ownership list under strict v4; this does not add an internal ellipse or persistent analytic conic.

Open: Reference ellipse gestures, modifiers, constructors/defaults/limits and exact semantic equivalence remain unqualified; no dedicated ellipse tool mode or internal ellipse is implemented.

Source: `crates/weftforge-core/src/document.rs:796` (`Command::AddEllipse {`); `apps/weftforge/src/main.rs:880` (`fn add_ellipse(`); `crates/weftforge-core/tests/ellipse.rs:32` (`fn ellipse_has_declared_controls_ids_bounds_tangents_and_one_persistent_transaction`); `apps/weftforge/src/main.rs:7834` (`fn ellipse_accessible_button_and_search_commit_once_and_preserve_failed_drafts`).

### `pattern-transform-tool` / `PACK-pattern-transform-tool`

TransformPiece performs one-piece rigid 2D translation/rotation about an explicit authored pivot, transforms endpoints and absolute cubic controls once, rotates grain and compensates the existing 3D placement to keep the arranged continuous surface and world warp fixed. IDs, boundary/sewing records and interval attachments remain authored; derived mesh/simulation are invalidated. Native Transform pattern rail/search routes expose a five-field SI modal plus boundary-translation/rotation-handle previews; a gesture combines pending inspector commands and the transform in one transaction. CLI execute/MCP project_execute reuse the shared command. ResizePiece retains its separate existing scaling behavior. Both owned endpoints of every straight internal annotation now participate in the same original-coordinate transform/resize traversal. Outer-only bounds still define the pivot/resize frame; annotation ownership/IDs remain, clearance is revalidated, and annotations do not become mesh/cloth constraints.

Open: Scaling/mirroring, multiselection, pivot dragging, alignment, angle/axis locks and multi-turn gestures remain open. Reference transform behavior and numerical/gesture equivalence remain unknown. Pointer translation/rotation usability and any native evidence qualification must be read from current registry/reports; source tests do not qualify that route.

Source: `crates/weftforge-core/src/document.rs:1088` (`Command::TransformPiece {`); `apps/weftforge/src/main.rs:3360` (`fn transform_window(`); `apps/weftforge/src/main.rs:3339` (`fn apply_transform(`); `crates/weftforge-core/tests/pattern_transform.rs:569` (`fn invalid_inputs_results_and_late_transactions_retain_every_history_branch`); `apps/weftforge/src/main.rs:14132` (`fn transform_pointer_translation_and_rotation_capture_original_geometry_and_snap_once`); `crates/weftforge-core/src/document.rs:1125` (`original.internal_curves.contains(&curve.id)`); `crates/weftforge-core/tests/internal_lines.rs:284` (`fn resize_rigid_transform_boundary_edits_and_owner_removal_keep_both_endpoints`).

### `pattern-rotate` / `PACK-pattern-rotate`

The one-piece TransformPiece candidate exposes an explicit radians angle and authored U/V pivot in the native modal, plus a bounds-center rotation handle. Arrangement compensation and grain rotation are deliberate WeftForge layout policy. The source relationship does not establish aliases to the inventoried Rotate/Parallel/Axis commands. The existing rigid rotation carries both annotation endpoints under the same arrangement/grain compensation; the pivot still derives from the exterior analytic bounds.

Open: Reference rotate/axis/parallel-target entry routes, selection scope, angle snapping/locks, multi-turn gestures and broader relation/pivot behavior remain open.

Source: `crates/weftforge-core/src/document.rs:1088` (`TransformPiece {`); `apps/weftforge/src/main.rs:3360` (`fn transform_window(`); `crates/weftforge-core/tests/internal_lines.rs:284` (`fn resize_rigid_transform_boundary_edits_and_owner_removal_keep_both_endpoints`).

### `grid-snap` / `PACK-grid-snap`

The native toolbar offers visible session choices Off, 1 mm, 5 mm and 10 mm. PositionSnap applies independent f64 origin-centred coordinate rounding; a gesture captures its preference and Alt/Option bypass once. Existing point/rectangle/polygon positions use that rule, while transform translation snaps displacement. Numeric dimensions, sewing picks and curve controls are not position-snapped. The visual 0.1 m guide grid is separate, and no snap field is serialized. Each click of the two-point Internal line tool captures its current position-snap/Alt rule; the precise start/end metre fields bypass snap.

Open: Reference snap origin/ties/distance/modifier/persistence and preference/menu equivalence remain unknown. This original session control supplies no adaptive, tangent or smart-snap inference.

Source: `apps/weftforge/src/main.rs:45` (`enum PositionSnap {`); `apps/weftforge/src/main.rs:2713` (`fn position_snap_control(`); `apps/weftforge/src/main.rs:6239` (`fn position_snapping_pointer_geometry_bypass_and_history_use_one_captured_rule`); `apps/weftforge/src/main.rs:6574` (`fn position_snapping_inline_choices_preserve_unfinished_numeric_text_and_caret`); `apps/weftforge/src/main.rs:15358` (`fn internal_line_search_rail_and_snapped_two_click_creation_are_atomic`).

### `pattern-smart-snap` / `PACK-pattern-smart-snap`

The later PositionSnap selector is an original fixed position-grid candidate. It does not supply curve/point/tangent/angle candidate priorities, smart guides or a reference-equivalent Smart Snap preference. Those requirements remain planned.

Open: Observed curve/point/angle/tangent snap candidates, screen-distance tolerance, priorities, guide display and modifier bypass. The matching preference checkboxes are read-only observations; their equivalence to menu controls and actual snapping/guide effects remains untested.

Source: `apps/weftforge/src/main.rs:45` (`enum PositionSnap {`).

### `pattern-view-extents` / `PACK-pattern-view-extents`

Frame all in View and the context toolbar fits all applied authored exterior outlines using analytic cubic-extrema bounds and 28 logical-pixel padding in the actual 2D canvas. It changes only center/scale, retains drafts/history/selection, disables for an empty document and handles degenerate/invalid canvas bounds. Pan/zoom remain separate view operations.

Open: Reference Zoom Extents All hidden/selected/object-scope, padding, shortcuts, high-DPI behavior and saved-camera semantics remain unknown; the original Frame all source is a candidate relationship only.

Source: `apps/weftforge/src/main.rs:2743` (`fn frame_patterns_button(`); `apps/weftforge/src/main.rs:2758` (`fn frame_patterns(`).

### `pattern-snapshot-print` / `PACK-pattern-snapshot-print`

Existing SVG and boundary-only R2000 DXF exports remain. Additive Import SVG/Import DXF File/search routes now choose a bounded source and show count, physical scale, origin/layout and omitted content before Import/Cancel. Shared ImportSvgPatterns/ImportDxfPatterns validate editable exterior lines/cubics and names with fresh IDs/default material/placement, then combine all pieces and pending inspector edits in one transaction. SVG requires explicit physical mm dimensions and uniform viewBox scale, mapping display Y to authored -Y; the strict DXF writer profile maps WCS X/Y to U/V without inversion. CLI import-svg/import-dxf, dedicated MCP file tools and command arrays reuse the same implementation. This FILE-005 engineering bucket does not invent a reference import catalog entry or equate import with printing. Strict v4 native saving retains internal annotations, while SVG/DXF remain exterior-boundary-only and OBJ emits only rest/current cloth surfaces without annotation line primitives. Native export help/messages, CLI help and MCP descriptions explicitly disclose the omission. Bounded SVG/DXF imports initialize empty internal arrays; they do not import annotation roles.

Open: General SVG/CAD compatibility, apparel conventions, internal features, sewing/material/arrangement round trip, calibrated printing, page tiling and reference semantics remain open. Native save retains the primary on failed replacement under inherited guarantees; recovery may already have advanced to the prior valid primary. Primary and recovery are not one atomic two-file transaction. Actual chooser/modal/edit/history/reopen qualification comes only from registry/reports. The annotation endpoint modal blocks background Save; ordinary native Save still commits a general inspector draft before chooser/IO. No global failed/cancelled-Save draft-atomicity result follows from this increment.

Source: `apps/weftforge/src/main.rs:1409` (`fn choose_import(`); `apps/weftforge/src/main.rs:1453` (`fn pattern_import_window(`); `crates/weftforge-core/src/persistence/svg.rs:54` (`pub fn from_bytes(bytes: &[u8]) -> Result<Self>`); `crates/weftforge-core/src/persistence/dxf_import.rs:50` (`pub fn from_bytes(bytes: &[u8]) -> Result<Self>`); `apps/weftforge/src/main.rs:12195` (`fn chooser_modal_import_commits_pending_edits_and_all_pieces_as_one_undo`); `apps/weftforge/src/main.rs:962` (`Exterior boundaries only; internal annotations stay in the native project.`); `crates/weftforge-core/src/persistence.rs:333` (`Exterior boundaries only; internal annotations are omitted.`); `crates/weftforge-core/tests/internal_lines.rs:660` (`fn exterior_exports_omit_annotations_and_failed_native_save_keeps_both_files`).

### `automation-scripts` / `PACK-automation-scripts`

Current Rust defines 29 Command variants: 28 nontransaction commands plus Transaction. The MCP command_schema oneOf exposes exactly those 28 nontransaction commands. AddInternalSegment and RemoveInternalCurve are the two additions since the preserved B657 27/26/nine-tool review; endpoint edits reuse MovePoint commands in one Transaction. Strict CLI execute/MCP project_execute arrays still accept 1–1024, reject nested transactions and commit/save once. The nine stateless MCP tools and CLI verbs are unchanged. CLI inspect now reports per-piece boundary/internal_curves ownership; project_inspect returns strict native v4 construction. Existing lengths/project_lengths can measure an annotation in unrounded authored metres without making it a sewing target. Bounded import/export profiles disclose empty imported internal arrays and omitted annotation exports. No live GUI control, script runtime, persistent undo or simulation-job API is introduced.

Open: Broader scripting/editing/reporting and observed scripting semantics remain open. The nine file tools are not reference scripting parity and do not control an active studio session.

Source: `crates/weftforge-core/src/document.rs:1549` (`pub enum Command {`); `apps/weftforge-cli/src/main.rs:365` (`fn execute_commands(`); `apps/weftforge-cli/src/mcp.rs:528` (`fn command_schema()`); `apps/weftforge-cli/src/mcp.rs:732` (`fn tools()`); `docs/automation.md:134` (`The supported shared commands are`); `apps/weftforge-cli/src/mcp.rs:617` (`"add_internal_segment"`); `apps/weftforge-cli/src/mcp.rs:626` (`"remove_internal_curve"`); `apps/weftforge-cli/src/main.rs:89` (`"internal_curves":piece.internal_curves`).

### `line-measure-display` / `PACK-line-measure-display`

Applied rest length feedback uses the existing bounded authored interval metric, independent of 3D placement, tessellation and cloth positions. Selected full boundaries and visible legacy seam pairs are cached by applied revision; group rows use matching-revision rest-mesh geometry. Native text uses three-decimal mm and signed B − A, with applied revision and draft/preview exclusion. Missing/error/rebuilding data is explicit. CLI lengths and MCP project_lengths query intervals, legacy seams or groups in unrounded metres without writing or building a mesh. Legacy curve-t correspondence remains unchanged. Selected full internal annotations also reuse the applied authored interval-length feedback. A read-only interval metric is distinct from physical sewability: every seam/group/length-based sewing target must still belong to an exterior boundary.

Open: Angle/perimeter/persistent measurements, selected-chain aggregation, current deformed 3D lengths and configurable units/precision remain open. Reference length selection/rounding/diagnostic policy is unqualified.

Source: `crates/weftforge-core/src/sewing.rs:636` (`pub fn interval_length(`); `apps/weftforge/src/main.rs:604` (`fn applied_length_heading(`); `apps/weftforge-cli/src/main.rs:31` (`fn length_report(`); `apps/weftforge-cli/src/mcp.rs:511` (`fn length_query_schema()`); `apps/weftforge/src/main.rs:16518` (`fn applied_lengths_are_formatted_cached_and_ignore_drafts_preview_and_simulation`); `crates/weftforge-core/src/document.rs:389` (`pub(crate) fn require_boundary_curve(`); `crates/weftforge-core/tests/internal_lines.rs:59` (`fn original_r_ids_analytic_length_history_and_endpoint_transaction`).

### `selected-line-lengths` / `PACK-selected-line-lengths`

Applied rest length feedback uses the existing bounded authored interval metric, independent of 3D placement, tessellation and cloth positions. Selected full boundaries and visible legacy seam pairs are cached by applied revision; group rows use matching-revision rest-mesh geometry. Native text uses three-decimal mm and signed B − A, with applied revision and draft/preview exclusion. Missing/error/rebuilding data is explicit. CLI lengths and MCP project_lengths query intervals, legacy seams or groups in unrounded metres without writing or building a mesh. Legacy curve-t correspondence remains unchanged. Selected full internal annotations also reuse the applied authored interval-length feedback. A read-only interval metric is distinct from physical sewability: every seam/group/length-based sewing target must still belong to an exterior boundary.

Open: Angle/perimeter/persistent measurements, selected-chain aggregation, current deformed 3D lengths and configurable units/precision remain open. Reference length selection/rounding/diagnostic policy is unqualified.

Source: `crates/weftforge-core/src/sewing.rs:636` (`pub fn interval_length(`); `apps/weftforge/src/main.rs:604` (`fn applied_length_heading(`); `apps/weftforge-cli/src/main.rs:31` (`fn length_report(`); `apps/weftforge-cli/src/mcp.rs:511` (`fn length_query_schema()`); `apps/weftforge/src/main.rs:16518` (`fn applied_lengths_are_formatted_cached_and_ignore_drafts_preview_and_simulation`); `crates/weftforge-core/src/document.rs:389` (`pub(crate) fn require_boundary_curve(`); `crates/weftforge-core/tests/internal_lines.rs:59` (`fn original_r_ids_analytic_length_history_and_endpoint_transaction`).

### `sewing-length-check` / `PACK-sewing-length-check`

Applied rest length feedback uses the existing bounded authored interval metric, independent of 3D placement, tessellation and cloth positions. Selected full boundaries and visible legacy seam pairs are cached by applied revision; group rows use matching-revision rest-mesh geometry. Native text uses three-decimal mm and signed B − A, with applied revision and draft/preview exclusion. Missing/error/rebuilding data is explicit. CLI lengths and MCP project_lengths query intervals, legacy seams or groups in unrounded metres without writing or building a mesh. Legacy curve-t correspondence remains unchanged. Selected full internal annotations also reuse the applied authored interval-length feedback. A read-only interval metric is distinct from physical sewability: every seam/group/length-based sewing target must still belong to an exterior boundary.

Open: Angle/perimeter/persistent measurements, selected-chain aggregation, current deformed 3D lengths and configurable units/precision remain open. Reference length selection/rounding/diagnostic policy is unqualified.

Source: `crates/weftforge-core/src/sewing.rs:636` (`pub fn interval_length(`); `apps/weftforge/src/main.rs:604` (`fn applied_length_heading(`); `apps/weftforge-cli/src/main.rs:31` (`fn length_report(`); `apps/weftforge-cli/src/mcp.rs:511` (`fn length_query_schema()`); `apps/weftforge/src/main.rs:16518` (`fn applied_lengths_are_formatted_cached_and_ignore_drafts_preview_and_simulation`); `crates/weftforge-core/src/document.rs:389` (`pub(crate) fn require_boundary_curve(`); `crates/weftforge-core/tests/internal_lines.rs:59` (`fn original_r_ids_analytic_length_history_and_endpoint_transaction`).

### `free-chain-sewing` / `PACK-free-chain-sewing`

A separate native Free sewing tool/search route owns revision-stamped transient A/B drafts. Canvas start/direction and accessible Boundary, exact Start t, Direction and Length (mm) controls feed Create Sewing Line; Tab validates, first-side OK advances without authoring, and second-side OK sends AddSewingByLength through the shared inspector transaction. Project::interval_from_length reuses the bounded metric/inversion for forward/reverse physical length on one curve. The command resolves both sides before allocation and creates one ordinary SewingGroup with one interval per side and normalized physical arc-length correspondence, rather than a legacy curve-t Seam. It rejects overflow, insufficient remaining distance, invalid/overlapping/precision-limited input without clamping. Both pending previews, raw text/caret and applied labels/guides remain distinct; Cancel/tool/load/history/revision rules retain the existing accepted construction. Legacy Sew edges and ordered M:N sewing remain separate routes. Current core role validation and native boundary-only pick/control paths explicitly reject internal annotation targets for legacy seams, ordered groups and AddSewingByLength. Straight annotation creation is not internal sewing.

Open: Reference continuous curved endpoints/correspondence, picking and initial values, cancellation/limits, unequal-length easing, M:N execution and physical sewing acceptance remain unqualified. Internal/boundary-traversing/automatic sewing and broader editing/selection remain in scope. Entered lengths choose creation endpoints; they are not persistent dimension locks. Actual native qualification must come from registry/reports, not these source anchors.

Source: `crates/weftforge-core/src/sewing.rs:610` (`pub fn interval_from_length(`); `crates/weftforge-core/src/document.rs:1232` (`Command::AddSewingByLength {`); `apps/weftforge/src/main.rs:2853` (`fn free_sewing_window(`); `apps/weftforge/src/main.rs:13141` (`fn free_sewing_failure_and_cancel_retain_raw_text_carets_redo_and_accepted_state`); `apps/weftforge/src/main.rs:13324` (`fn free_sewing_new_empty_refuses_pending_a_and_b_without_dropping_or_committing_them`); `crates/weftforge-core/tests/sewing_groups.rs:1365` (`fn physical_length_command_is_one_persistent_group_with_physical_correspondence_and_history`); `crates/weftforge-core/src/document.rs:389` (`pub(crate) fn require_boundary_curve(`); `crates/weftforge-core/src/sewing.rs:611` (`self.require_boundary_curve(side.curve)?;`).

### `free-sewing-length-preview` / `PACK-free-sewing-length-preview`

A separate native Free sewing tool/search route owns revision-stamped transient A/B drafts. Canvas start/direction and accessible Boundary, exact Start t, Direction and Length (mm) controls feed Create Sewing Line; Tab validates, first-side OK advances without authoring, and second-side OK sends AddSewingByLength through the shared inspector transaction. Project::interval_from_length reuses the bounded metric/inversion for forward/reverse physical length on one curve. The command resolves both sides before allocation and creates one ordinary SewingGroup with one interval per side and normalized physical arc-length correspondence, rather than a legacy curve-t Seam. It rejects overflow, insufficient remaining distance, invalid/overlapping/precision-limited input without clamping. Both pending previews, raw text/caret and applied labels/guides remain distinct; Cancel/tool/load/history/revision rules retain the existing accepted construction. Legacy Sew edges and ordered M:N sewing remain separate routes. Current core role validation and native boundary-only pick/control paths explicitly reject internal annotation targets for legacy seams, ordered groups and AddSewingByLength. Straight annotation creation is not internal sewing.

Open: Reference continuous curved endpoints/correspondence, picking and initial values, cancellation/limits, unequal-length easing, M:N execution and physical sewing acceptance remain unqualified. Internal/boundary-traversing/automatic sewing and broader editing/selection remain in scope. Entered lengths choose creation endpoints; they are not persistent dimension locks. Actual native qualification must come from registry/reports, not these source anchors.

Source: `crates/weftforge-core/src/sewing.rs:610` (`pub fn interval_from_length(`); `crates/weftforge-core/src/document.rs:1232` (`Command::AddSewingByLength {`); `apps/weftforge/src/main.rs:2853` (`fn free_sewing_window(`); `apps/weftforge/src/main.rs:13141` (`fn free_sewing_failure_and_cancel_retain_raw_text_carets_redo_and_accepted_state`); `apps/weftforge/src/main.rs:13324` (`fn free_sewing_new_empty_refuses_pending_a_and_b_without_dropping_or_committing_them`); `crates/weftforge-core/tests/sewing_groups.rs:1365` (`fn physical_length_command_is_one_persistent_group_with_physical_correspondence_and_history`); `crates/weftforge-core/src/document.rs:389` (`pub(crate) fn require_boundary_curve(`); `crates/weftforge-core/src/sewing.rs:611` (`self.require_boundary_curve(side.curve)?;`).

### `undo-redo` / `PACK-undo-redo`

The shared Editor transaction still owns atomic history. Later ellipse, bounded drawing imports, one-piece transforms and completed physical-length sewing combine pending inspector edits with one command into one history entry. First-side sewing and drag previews are transient. New Empty refuses dirty/pending/unfinished work and resets history only for accepted clean replacement; load/history/revision changes clear stale preview and metric state. Numeric text/caret and redo retention checks are implementation evidence only. Internal two-click creation, endpoint drag/removal and changed endpoint MovePoint arrays use the same combined inspector-action transaction. Unchanged/cancelled endpoint editing creates no history entry; stale revision/owner/target previews are discarded. General Save draft atomicity remains outside this source increment.

Open: Full observed gesture/history/coalescing semantics, native route qualification, all rejection paths and reference parity remain open; use registry/reports for the exact accepted runtime scope.

Source: `crates/weftforge-core/src/document.rs:1693` (`pub fn execute(&mut self, command: Command)`); `apps/weftforge/src/main.rs:840` (`fn apply_inspector_action(`); `apps/weftforge/src/main.rs:13324` (`fn free_sewing_new_empty_refuses_pending_a_and_b_without_dropping_or_committing_them`); `apps/weftforge/src/main.rs:3138` (`fn apply_internal_line(`); `apps/weftforge/src/main.rs:16123` (`fn internal_line_review_compound_endpoints_and_staged_grain_commit_one_revision_and_undo`); `apps/weftforge/src/main.rs:16360` (`fn internal_line_review_accepted_drag_commits_source_boundary_control_and_endpoint_once`).

### `simulate` / `PACK-simulate`

The native transport exposes Play/Pause, Step, Assemble (gravity off), Drape, Reset and Cancel job. Current workers carry process-session task IDs/input revision, count only successful simulation steps and distinguish actual worker outcome from cancellation request/obsolete input before accepting a matching-revision result. Previous accepted display state survives failed/cancelled/obsolete workers. Live cloth contact now uses conservative swept hierarchy candidates in ascending original order, with a retained linear remainder after projection moves the query; static diagnostic contact queries also use the hierarchy. The exact zero-target positive-compliance stitch branch uses vector coordinates, retaining the old hard/positive-target radial branch. These source candidates establish no garment-quality/calibration/reference acceptance. Internal annotations are authoritative construction but are excluded from mesh vertices/triangles/boundary samples, stitches, cloth mass and constraints. An accepted annotation edit still advances the document revision and invalidates derived jobs/state; an annotation-only rebuild uses a fresh revision without adding physical effects.

Open: Full physical garment protocols, equilibrium/calibration, complete deforming collision coverage, representative performance and reference simulation behavior remain open. Dense contact can return to the original scan; no general speedup follows from the hierarchy. A full worker step count is not accepted cloth or garment quality. Retained failures and actual runtime qualification remain authoritative in the registry/reports.

Source: `apps/weftforge/src/main.rs:2105` (`fn simulation_bar(`); `apps/weftforge/src/main.rs:2195` (`fn task_details(`); `apps/weftforge/src/main.rs:9908` (`fn task_progress_correlates_real_workers_and_retains_unaccepted_snapshots`); `crates/weftforge-sim/src/contact.rs:108` (`pub(crate) fn project_cloth(`); `crates/weftforge-sim/src/contact.rs:219` (`pub(crate) fn contact_errors_cancellable(`); `crates/weftforge-sim/src/constraints.rs:164` (`pub(crate) fn solve_stitch(`); `crates/weftforge-core/tests/internal_lines.rs:396` (`fn annotations_leave_every_mesh_and_physical_field_exterior_only`).

### `workspace-statistics` / `PACK-workspace-statistics`

The status bar reports applied revision, authored piece/legacy seam/group counts, accepted mesh vertices, dirty/autosave and current messages. Task details correlate process-session IDs, input revisions, successful simulation steps and terminal outcome. Inspector cloth diagnostics include assembly remaining, strain/seam/contact errors, initial intersections and uncertain capsule pairs. Current static/live hierarchy behavior is distinct from the historical initial-only BVH note; these values supply no accepted benchmark or quality result by themselves.

Open: Representative declared-hardware performance/memory/viewport/cancel/save measurements, broad overlays and reference counter semantics remain open. Mesh vertex counts are derived from the current accepted snapshot, not authoritative construction.

Source: `apps/weftforge/src/main.rs:2067` (`fn status_bar(`); `apps/weftforge/src/main.rs:2195` (`fn task_details(`); `crates/weftforge-sim/src/contact.rs:227` (`fn analyze_contacts(`).

### `fit-maps` / `PACK-fit-maps`

The inspector now attributes the maximum absolute material-axis length strain to a matching-rest-mesh pattern and warp/weft signed percentage; CLI simulation/benchmark/fitting reports expose the typed attribution. This read-only diagnostic is not a per-element colored stress/strain/pressure map, a principal/shear bound, calibration or a changed fitting acceptance limit.

Open: Per-element dimensional definitions, map shaders, legends/ranges/no-data policy and actual reference Stress/Strain/Pressure behavior remain open. Existing original garment-quality failures are not repaired by adding attribution.

Source: `apps/weftforge/src/main.rs:2617` (`if let Some(peak)=diagnostics.peak_strain`); `spec/features/SIM-005-strain-attribution.md:7` (`The existing maximum absolute material-axis length strain keeps its definition`).

### `pattern-name-property` / `PACK-pattern-name-property`

Piece names remain authored native data. Creation now accepts names through AddPolygon/AddRectangle/AddEllipse and bounded SVG/DXF imports; imported names follow their explicit profiles. The native Pattern name field configures creation and is not a selected-piece rename command. Internal annotations have stable curve IDs and owner labels, with no new authored line-name or rename field.

Open: Validated selected-pattern rename, mixed-selection behavior, command/adapter schema, one-gesture history and persistent display synchronization.

Source: `apps/weftforge/src/main.rs:2496` (`ui.strong("Create pattern")`); `crates/weftforge-core/src/document.rs:796` (`AddEllipse {`).

### `unit-display-preferences` / `PACK-unit-display-preferences`

Core/CPU remain f64 SI. Native creation/transform/placement fields retain explicit metres/radians; rectangle and Free sewing dialogs convert millimetres once, and applied length feedback formats mm while CLI/MCP metric JSON stays unrounded metres. These are fixed engineering presentation choices, with no global unit/precision/locale preference model. The internal endpoint modal labels Start U/V and End U/V in metres and does not introduce locale or unit preferences.

Open: Validated display and construction-specific unit preferences that preserve f64 SI authoring; consistent labels, rounding, parser behavior and adapters without silent physical rescaling.

Source: `apps/weftforge/src/main.rs:576` (`fn length_mm(`); `apps/weftforge/src/main.rs:2853` (`fn free_sewing_window(`); `apps/weftforge/src/main.rs:3360` (`fn transform_window(`); `apps/weftforge/src/main.rs:3194` (`fn internal_line_window(`).

### `shortcut-preferences` / `PACK-shortcut-preferences`

Fixed command search reaches New Empty, native save/open/recover, bounded SVG/DXF import and vector/mesh exports, Undo/Redo/Reset, Select/Transform/Polygon/Rectangle/Add ellipse and three distinct sewing routes. Fixed global shortcuts are modal/popup/focus guarded; this is not an editable keymap preferences implementation or proof of reference keyboard bindings. The fixed search also reaches Internal line and Create 2D supplies the owner-aware action; no reference G shortcut is inferred. Select-mode Delete/Backspace removes a selected internal annotation, not its owner.

Open: Inventory expanded command descendants and search/edit/delete/reset behavior; configurable keymap with per-tool/modal/focus scope and accessible conflict feedback.

Source: `apps/weftforge/src/main.rs:4924` (`egui::Window::new("Command search")`); `apps/weftforge/src/main.rs:4697` (`let global_shortcuts = !authoring_modal`); `apps/weftforge/src/main.rs:4147` (`if self.tool == Tool::Select`); `apps/weftforge/src/main.rs:3028` (`fn selected_internal_curve(`).

### `internal-primitives` / `PACK-internal-primitives`

Piece.internal_curves now owns independent straight annotations using ordinary stable-ID curves and two exclusive endpoint points. AddInternalSegment allocates both points/curve; RemoveInternalCurve removes only that owned annotation, and MovePoint/Transaction edit its endpoints. Whole-segment containment uses certified bounded exterior flattening, 1e-6 m clearance, 1e-7 m curve-tube reserve plus conservative f64 reserve, and a fixed one-million-work cap; uncertain/outside/degenerate/shared/cubic-internal input rejects atomically. The native Internal line rail/Create 2D/search routes provide two-click creation, correctable preview and precise start/end metre editing with role-aware selection/deletion and retained-input/modal guards. Native v4 retains ownership and endpoints. This is a straight annotation candidate, not an internal shape, cloth/topology operation or reference contract.

Open: Full open/closed internal paths, grouped shapes and primitive parameters/constraints independent from the exterior boundary; the current independent straight segment supplies none of those grouping/shape semantics. Only independent straight inside segments are source candidates. Full internal paths/groups/cubics, rectangle/ellipse shapes, boundary attachment, holes, cuts/cut-and-sew, folds and internal sewing remain absent. No actual native or reference acceptance was executed by this mapping task. Bounded containment can honestly refuse difficult geometry; no reference clearance/picking/grouping/cancellation behavior is inferred.

Source: `crates/weftforge-core/src/document.rs:178` (`pub internal_curves: Vec<Id>`); `crates/weftforge-core/src/document.rs:825` (`Command::AddInternalSegment {`); `crates/weftforge-core/src/document.rs:853` (`Command::RemoveInternalCurve {`); `crates/weftforge-core/src/geometry.rs:461` (`pub(crate) fn validate_internal_segments(`); `apps/weftforge/src/main.rs:3194` (`fn internal_line_window(`); `apps/weftforge/src/main.rs:15358` (`fn internal_line_search_rail_and_snapped_two_click_creation_are_atomic`); `crates/weftforge-core/tests/internal_lines.rs:137` (`fn original_l_c_and_clearance_are_whole_segment_and_resolution_independent`).

### `project-load-save` / `PACK-project-load-save`

Strict native v4 requires each piece internal_curves array and the project sewing_groups array; annotation ownership, straightness, containment and existing bounded validation apply before accepting input. Explicit frozen PieceV2V3/ProjectV3 and retained v1/v2 readers reject mixed schemas and add only empty internal arrays (and groups for v1/v2), preserving supported legacy IDs/order/maps/numeric fields without an authoring transaction. Saving an accepted old input writes v4; old fixtures/evidence retain their original bytes. Failed load retains the old editor; native primary/recovery staging retains its established ordering, not a two-file atomic guarantee. Ordinary Save commits a general inspector draft before chooser/IO; the annotation modal shields background Save rather than fixing that broader behavior.

Open: Reference native decoding research; metadata preservation; exact open/save option contracts and unsupported-data reporting. Show Option to Save Metadata and Save with Thumbnail preferences are configured read-only; native thumbnail/metadata contents and future-save/default/reset behavior are not implemented by current native JSON IO. Qualify exact authored f64 fields through the real native load/save/reopen path for the current runtime source. Retained runs do not automatically qualify later source changes, and round-trip decoding cannot restore values already saved after rounding. V4 is a bounded supported-model format with no downgrade writer; full future internal roles/shapes/cuts need a reviewed schema decision. Core/schema definitions do not establish current native save/reopen or universal pending-draft preservation.

Source: `crates/weftforge-core/src/persistence.rs:26` (`pub const FORMAT_VERSION: u32 = 4;`); `crates/weftforge-core/src/persistence.rs:117` (`struct PieceV2V3 {`); `crates/weftforge-core/src/persistence.rs:85` (`struct ProjectV3 {`); `crates/weftforge-core/tests/migration.rs:399` (`fn original_v3_migrates_only_empty_internal_ownership_and_keeps_f64_bits`); `crates/weftforge-core/tests/internal_lines.rs:524` (`fn v4_endpoint_bits_and_strict_annotation_input_round_trip`).

### `delete-selection` / `PACK-delete-selection`

Native piece/legacy-seam/group removal uses shared transactions. Removing a piece removes the whole ordered group if any span depends on it, reports the dependent-group count and preserves unrelated groups; Undo restores the relationship. This independent policy is not a reference deletion finding. Generic selection deletion and dependency previews remain absent. Select-mode Delete/Backspace and Remove internal line now dispatch RemoveInternalCurve for a selected annotation. The separate Delete owner piece label explicitly dispatches RemovePiece, whose traversal removes both endpoints and all internal curves with that owner. Role-aware deletion preserves unrelated construction and keeps the monotonic allocator; no generic multi-selection deletion model is added.

Open: Typed selection deletion for each object kind, dependency invalidation/cascade contract, hotkey/context and old-state preservation.

Source: `crates/weftforge-core/src/document.rs:853` (`Command::RemoveInternalCurve {`); `apps/weftforge/src/main.rs:2487` (`"Delete owner piece"`); `apps/weftforge/src/main.rs:15768` (`fn internal_line_picking_delete_and_boundary_tools_keep_roles_separate`).

### `point-pattern-edit` / `PACK-point-pattern-edit`

One authored point can drag-preview and commit once; boundary selection and numeric curve controls exist, but multi-point/segment transforms do not. Internal annotations expose both endpoint IDs/positions. Selected-line endpoint drags preview on cloned construction and commit MovePoint with valid pending inspector edits once; precise editing combines changed endpoint commands in one Transaction. Revision/owner/target-bound input, coincident-line selection and press/blur/cancel guards retain rejected or unfinished source drafts. This supplies no generalized multi-point/segment transform.

Open: Observed Edit Pattern versus Transform Point/Segment behavior, modifier selection, numeric coordinates and valid atomic neighbor handling.

Source: `apps/weftforge/src/main.rs:405` (`struct InternalLineDraft {`); `apps/weftforge/src/main.rs:3042` (`fn internal_line_edit(`); `apps/weftforge/src/main.rs:3138` (`fn apply_internal_line(`); `apps/weftforge/src/main.rs:16365` (`fn internal_line_review_rejected_drag_retains_boundary_draft_raw_caret_redo_and_outputs`).

### `curve-tools` / `PACK-curve-tools`

Absolute cubic Bezier handles are authorable numerically and displayed; tangent/curvature point/smoothing tools are absent. SetCurveControls, SplitCurve and JoinCurve are explicitly exterior-boundary-only. Internal annotations currently require controls: None; native cubic/split/join controls are hidden for an internal selection. Reusing Curve/Point identity does not implement internal cubic editing or smoothing.

Open: Reviewed handle/curve-point continuity types, gestures, smooth algorithm outcome and relationship/parameterization preservation.

Source: `crates/weftforge-core/src/document.rs:389` (`pub(crate) fn require_boundary_curve(`); `apps/weftforge/src/main.rs:2413` (`if internal {`); `crates/weftforge-core/tests/internal_lines.rs:284` (`fn resize_rigid_transform_boundary_edits_and_owner_removal_keep_both_endpoints`).

### `split-boundary` / `PACK-split-boundary`

Exact parameter-t split remaps authored seams with stable IDs; bounded JoinCurve joins the selected boundary to its consecutive oriented neighbor, including the closing pair, and retains the selected curve ID. It accepts only compatible straight/cubic inverse constructions within a placement-aware 1e-12 m control-plus-f64 bound, remaps both seam sides affinely and preserves separate seam IDs/maps. Native inspector, CLI join and MCP join_curve use the shared typed command. Native joins include pending drafts in one transaction; rejection preserves document/history/draft/current numeric input, and success clears superseded text/focus. These source routes and in-process checks do not themselves qualify native execution or reference behavior. Read the declared native join subset and its current qualification in the authoritative registry/reports; broader reference join semantics remain unknown. Ordered groups are rewritten separately: split expands each affected span in traversal order, including reversed/endpoint cases; join transforms every occurrence while retaining individual list entries, group ID/name/map/order. Current authored arc lengths/pairing are rederived within the declared group metric budget. Legacy parameter correspondence and tighter inverse-geometry tests remain distinct. The new annotation ownership list and both endpoint coordinates remain unchanged through supported boundary split/join. Internal targets are rejected explicitly, and final candidate validation rechecks retained annotation clearance after boundary edits.

Open: Reference gesture/numeric/arc-length-versus-parameter contracts, selection/snapping/failure/history semantics and independent comparison remain unknown. Bounded JoinCurve is inverse construction, not arbitrary curve fitting, corner removal, mixed straight/cubic merging or trim. Broader native invalid-split/control branches, split Undo/Redo, large-placement and coalesced-event cases require their own declared execution. Read the UI-BOUNDED-JOIN-001 native subset and its source/executable identity, independent checks and current qualification in the authoritative registry/reports; source/in-process/CLI/MCP tests do not supply reference parity. Internal splitting/joining and topological cuts remain future work; the exterior inverse-construction policy is not extended by annotation support.

Source: `crates/weftforge-core/src/document.rs:967` (`Curve has no piece boundary; internal annotations cannot be joined`); `crates/weftforge-core/tests/internal_lines.rs:284` (`fn resize_rigid_transform_boundary_edits_and_owner_removal_keep_both_endpoints`).

### `resolution` / `PACK-resolution`

One document-wide edge-length resolution rebuilds conforming rest mesh and invalidates simulation; no high/low presets or per-piece resolution/subdivision mode exists. Ordered group construction remains unchanged across remeshing; lengths/portions and weighted stitches are rebuilt for the new revision using authored arc length rather than the mesh chord metric. The final refinement schedule improves slivers before subdivision and retains an exact original fine-fixture regression; it does not define reference tessellation or change global/per-pattern scope. Annotation containment is independent of mesh resolution; internals remain excluded from the exterior-only mesher. Annotation-only edits invalidate the old revision even when freshly rebuilt mesh/topology/UV/sewing fields are identical apart from doc_revision.

Open: Runtime preset values/scope, subdivision display distinction, validation bounds and preserved rest/seam correspondence. Selected-pattern Particle Distance is explicitly measured in mm, while current SetResolution is document-wide metres. No per-pattern field/edit/remesh semantics are implemented; this control must not be conflated with Side Geometry Resolution. Preference Default Particle Distance is a separate read-only configuration value with no printed unit in that control; its effect on new versus existing patterns is untested.

Source: `crates/weftforge-core/src/geometry.rs:461` (`pub(crate) fn validate_internal_segments(`); `crates/weftforge-core/tests/internal_lines.rs:396` (`fn annotations_leave_every_mesh_and_physical_field_exterior_only`).

### `segment-sewing` / `PACK-segment-sewing`

Legacy single-curve AddSeam retains oriented intervals, normalized curve-parameter correspondence and weighted mesh stitches. Native legacy edge-pair creation remains distinct from new ordered SewingGroup arc-length pairing. Both derive revision-tagged constraints independently of the display mesh; the new representation does not reinterpret old v1/v2 nonlinear maps or establish reference correspondence. Current ownership validation requires all seam/group targets to be exterior boundaries. Native sewing controls and pick paths retain boundary-only roles even when internal annotations are selectable/measurable. Internal annotations do not become sewable, folded or physical through existing seam/group commands.

Open: Complete entry/selection/direction/endpoint contracts beyond the bounded whole-edge, paired partial and categorical reversal cases, including unequal lengths/easing, exact curved/free endpoints and numerical mismatch/correspondence diagnostics. Two separately accepted numeric 100 mm sides now establish one completed partial pair and a later Reverse Sewing changes categorical endpoint guides. These outcomes supply no exact control geometry, continuous arc-length mapping, sewn remeshing, simulation or process-restart contract.

Source: `crates/weftforge-core/src/document.rs:389` (`pub(crate) fn require_boundary_curve(`); `apps/weftforge/src/main.rs:3035` (`fn selected_boundary_curve(`); `crates/weftforge-core/tests/internal_lines.rs:396` (`fn annotations_leave_every_mesh_and_physical_field_exterior_only`).

### `sewing-editor` / `PACK-sewing-editor`

Legacy seam ID/name/curve/parameter rows retain Reverse/Remove. New ordered group rows identify every span and direction, report current revision-derived arc lengths/mismatch/paired portions, and expose Edit group, Reverse B and Remove group. The structured modal edits name, ordered curve-ID/t intervals and explicit nonlinear easing knots; accessible boundary-add controls and row comboboxes support precise paths. Shared commands combine pending inspector edits and one completed action into one transaction; rejection retains draft/input/history and Cancel commits nothing. Full canvas seam picking, general legacy interval/name/property editing, multi-selection and reference-equivalent preflight remain absent. Current ownership validation requires all seam/group targets to be exterior boundaries. Native sewing controls and pick paths retain boundary-only roles even when internal annotations are selectable/measurable. Internal annotations do not become sewable, folded or physical through existing seam/group commands.

Open: Reference direct picking, name/property/interval edit rules, mixed/multiple selection and complete preflight diagnostics remain unqualified. Bounded whole/partial/reversed retrieval and categorical guides do not establish continuous correspondence or a complete sewing editor. Run the actual native ordered draft/modal edit/reversal/removal/cancellation and editable reopen workflow separately from in-process tests, including focused edits and failed-action preservation; read its declared scope and qualification in the registry/reports.

Source: `crates/weftforge-core/src/document.rs:389` (`pub(crate) fn require_boundary_curve(`); `apps/weftforge/src/main.rs:3035` (`fn selected_boundary_curve(`); `crates/weftforge-core/tests/internal_lines.rs:396` (`fn annotations_leave_every_mesh_and_physical_field_exterior_only`).

### `reverse-sewing` / `PACK-reverse-sewing`

Legacy per-seam Reverse still swaps only side B t0/t1 through UpdateSeam, preserving its normalized parameter map and duplicate-name ID rows. Ordered group Reverse B uses UpdateSewingGroup to reverse B span order and swap every span direction while retaining identity/name/A/map; two reversals restore all group fields. Native rows/modal share transactional pending-draft and numeric-buffer guards and revision-derived portion/endpoint guides. Source/in-process tests describe implementation behavior; historical legacy reversal execution does not qualify the new group workflow, continuous reference equivalence, easing or physical twist. Current ownership validation requires all seam/group targets to be exterior boundaries. Native sewing controls and pick paths retain boundary-only roles even when internal annotations are selectable/measurable. Internal annotations do not become sewable, folded or physical through existing seam/group commands.

Open: Execute the newly declared ordered-group reversal/edit/history/reopen workflow on an identified native executable, retaining exact chosen identity, map, focused input, failure/cancel behavior and revision-derived guides. Current execution qualification belongs solely in the registry/reports; historical per-seam reversal does not qualify changed group source. Resolve reference continuous correspondence/geometry, multiple selection and alternative entry/error contracts independently. Endpoint/portion guides do not establish hidden reference orientation state, physical easing or simulated twist; broader seam properties remain unmodeled.

Source: `crates/weftforge-core/src/document.rs:389` (`pub(crate) fn require_boundary_curve(`); `apps/weftforge/src/main.rs:3035` (`fn selected_boundary_curve(`); `crates/weftforge-core/tests/internal_lines.rs:396` (`fn annotations_leave_every_mesh_and_physical_field_exterior_only`).

### `internal-line-display` / `PACK-internal-line-display`

The 2D authored canvas draws owned straight annotations in a distinct annotation style, both endpoint marks, selected identity and transient creation/endpoint previews. Scene/Properties list each internal curve under its owner and reuse applied authored length feedback. This current source relationship supplies 2D construction display/picking only; it adds no draped-cloth-following 3D internal display, fold display or visibility-preference model.

Open: Internal geometry plus separate display/picking treatment on current cloth.

Source: `apps/weftforge/src/main.rs:3741` (`for id in &piece.internal_curves {`); `apps/weftforge/src/main.rs:2391` (`Internal annotations · owned by this pattern`); `apps/weftforge/src/main.rs:15768` (`fn internal_line_picking_delete_and_boundary_tools_keep_roles_separate`).

### `cut-and-sew` / `PACK-cut-and-sew`

The bounded straight internal annotation is editable construction and a dependency candidate only; it leaves the exterior contour, cloth triangulation, mass and sewing constraints unchanged. No cut command, hole, source-to-output piece remapping, generated seam or automatic layout offset is implemented. PAT-006 describes a prospective follow-on cut separately from this candidate.

Open: Validated topological cut with new pieces/holes, stable source relationships and optional generated seams. Automatically Offset 2D Patterns when Cut is observed but unexecuted; an offset distance, direction, scope or transaction boundary is not inferred.

Source: `crates/weftforge-core/src/document.rs:1549` (`pub enum Command {`); `crates/weftforge-core/tests/internal_lines.rs:396` (`fn annotations_leave_every_mesh_and_physical_field_exterior_only`); `spec/features/PAT-006-internal-segments.md:444` (`## Follow-on cut boundary, after the first increment`).

## Boundaries retained

Only independent straight owned annotations are source candidates. Full paths/groups/cubics/shapes, holes, cuts/cut-and-sew, folds and internal sewing remain missing. The annotation is not a hole, cloth boundary, triangulation constraint, mass contribution, stitch or fold. Whole-segment containment is conservative and bounded; difficult/uncertain cases can reject. Actual native two-click/precise/endpoint/history/restart/omission-aware export acceptance remains a separate required gate on the integrated executable. No reported source-test total is used as that evidence.

Strict v4 requires explicit internal ownership and group arrays. Frozen v1/v2/v3 readers preserve supported legacy construction and add empty internals; old fixtures/evidence keep their original bytes. Unknown/mixed/unsupported saved data rejects rather than being silently stripped. A later full internal shape/cut role needs its own reviewed supported-model/migration decision.

The endpoint modal blocks background Save. Ordinary native Save still commits a general inspector draft before chooser/IO; this update supplies no global pending-draft Save atomicity claim. Primary replacement retains the established recovery order; recovery may already advance to the prior valid primary before a later replacement failure.

The historical JSON remains byte-identical with SHA-256 `9323eb803af6d07d6178fd6e808fd61f87e5da71e3b07ab4b279b3141b11c8ec`. All 949 records, 205 packets/dependencies, first three queued slices, 56 landmark identities/paths and original 22-command provenance remain unchanged. Current-only signature replacements for snap/initial_contacts remain in the additive record, leaving historical anchors untouched.

Integrate after the physical publication metadata and this runtime lineage are combined. Track every publishable artifact and run registry/map checks from a clean tracked checkout, plus all four required local Cargo gates for the integrated change. Root owns native closure, truthful status/evidence changes and publication. This proposal changes no registry status, root README, Actions configuration or static Pages content.
