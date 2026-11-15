# WeftForge current-source addendum

This additive engineering review describes implementation candidates at immutable source `b657e6bf52aca6c0c76c5c51aaf76b4e812150f7`. The [historical map](weftforge-implementation-map.md) remains pinned to `3eddef067c8a1f03f192c8e6822a8da2a94f9bf9` with **949 unique IDs**, **205 packets** and **56 landmark identities** unchanged. Its historical 22-command count is retained. This is neither a completed-feature denominator nor reference parity.

[Machine-readable addendum](weftforge-current-source-addendum.json). Current feature qualification, unknowns, failures and evidence freshness come only from [parity/features.yaml](../../parity/features.yaml) and the reports at the revision being reported. Source test anchors below identify implementation checks; this addendum did not run product or native acceptance.

## Source and command boundaries

- `af79f596f247cc48e709b818bb70a30d1b716580`: public native studio/ellipse/import/length ancestor.
- `dd5a16826dafe0d4f32c251f7c351197ad0936af`: immutable transform/New Empty candidate; native evidence remains separate.
- `b657e6bf52aca6c0c76c5c51aaf76b4e812150f7`: immutable physical-length Free Sewing/retention candidate; no native acceptance follows from this source review.

**Current Rust: 27 variants = 26 nontransaction commands + `Transaction(Vec<Command>)`.** The JSON schema advertises exactly 26 nontransaction commands. CLI `execute` and MCP `project_execute` accept 1–1024, reject nested transactions and wrap the array once. The nine MCP tools include bounded SVG/DXF imports and `project_lengths`. Transform and physical-length sewing use the existing command-array routes; New Empty and view/session controls add no command. These API counts are separate from the historical 22 variants.

## Current native routes

| Source surface | Controls and path | Existing engineering groups |
|---|---|---|
| File menu | New Empty; Open…; Import SVG…; Import DXF…; Save; Save as…; Recover…; Export OBJ; Export SVG; Export DXF. `apps/weftforge/src/main.rs:1772` | `project-new`, `project-load-save`, `pattern-snapshot-print`, `obj-interchange` |
| Edit/menu-right actions and search | Undo; Redo; Commands ⌘K; Command search text/filter and fixed action rows. `apps/weftforge/src/main.rs:4296` | `undo-redo`, `shortcut-preferences` |
| View/Help menus | Frame all; Wireframe; Static original interaction help. `apps/weftforge/src/main.rs:1772` | `pattern-view-extents`, `wireframe`, `help-resources` |
| Tool rail | Select / edit; Transform pattern; Polygon; Rectangle; Sew edges; Free sewing; M:N sewing. `apps/weftforge/src/main.rs:1907` | `selection`, `pattern-transform-tool`, `polygon`, `rectangle`, `segment-sewing`, `free-chain-sewing` |
| Context toolbar | Off / 1 mm / 5 mm / 10 mm Snap; Frame all; Transform pattern…; Ordered A/B draft summary; Create Sewing Line; Cancel free sewing. `apps/weftforge/src/main.rs:1876` | `grid-snap`, `pattern-view-extents`, `pattern-transform-tool`, `free-chain-sewing` |
| Document tab and status | Document name/dirty marker; Revision; Piece/seam/group/vertex counts; Worker/autosave messages; Discard and close. `apps/weftforge/src/main.rs:1946` | `workspace-panels`, `workspace-statistics`, `application-quit` |
| Properties and Scene | Piece selection rows; Selected boundary endpoints; Cubic Bezier and absolute controls; Split t / Split / Join; Applied boundary length; Size; Placement translation/rotation; Warp angle; Shared fabric mass/stiffness/bend/RGB; Delete selected. `apps/weftforge/src/main.rs:2135` | `selection`, `curve-tools`, `split-boundary`, `selected-line-lengths`, `pattern-transform-tool`, `arrangement`, `grain-alignment`, `fabric-assignment`, `delete-selection` |
| Create pattern | Pattern name; W/H m; X/Y m; Add rectangle; Add ellipse; Add polygon point; Close pattern. `apps/weftforge/src/main.rs:2330` | `polygon`, `rectangle`, `ellipse`, `pattern-name-property` |
| Legacy sewing and ordered groups | Reverse paired edge; A/B t intervals; Legacy Reverse / Remove; Applied lengths and signed mismatch; Ordered A/B span controls; Confirm A / Create group / Delete last / Cancel; Edit group / Reverse B / Remove group. `apps/weftforge/src/main.rs:2375` | `segment-sewing`, `sewing-editor`, `reverse-sewing`, `free-chain-sewing`, `sewing-length-check` |
| Rectangle modal | Width (mm); Height (mm); OK; Cancel. `apps/weftforge/src/main.rs:3064` | `rectangle`, `undo-redo` |
| Transform modal | Translation U/V (m); Angle (radians); Pivot U/V (m); Apply transform / Return; Cancel transform. `apps/weftforge/src/main.rs:2930` | `pattern-transform-tool`, `pattern-rotate`, `undo-redo` |
| Free sewing modal | Boundary; Start t; Length (mm); Forward / Reverse; Tab validation; OK / Enter; Cancel / Close. `apps/weftforge/src/main.rs:2677` | `free-sewing-length-preview`, `free-chain-sewing`, `undo-redo` |
| Structured group modal | Name; A/B ordered intervals; Reverse B order and directions; Easing knot A/B controls; Add easing knot / Remove knot; Apply group; Cancel group edit. `apps/weftforge/src/main.rs:3872` | `sewing-editor`, `free-chain-sewing` |
| Pattern import review | Piece count; Physical scale/origin/layout; Explicit omissions; Import; Cancel. `apps/weftforge/src/main.rs:1359` | `pattern-snapshot-print`, `undo-redo` |
| Recovery window | Discovered recovery choices; Refresh; Choose project…; Choose recovery file…; Recover copy; Close. `apps/weftforge/src/main.rs:1579` | `project-load-save`, `autosave-preferences` |
| 2D/3D canvases | Applied outlines/points/controls; Draft and sewing guides/labels; One-piece transform pivot/handle; 2D pan/zoom; 3D orbit/zoom/picking; Wireframe. `apps/weftforge/src/main.rs:3127` | `point-pattern-edit`, `curve-tools`, `sewing-display`, `pattern-transform-tool`, `viewpoints`, `selection`, `wireframe` |
| Simulation transport | Play / Pause; Step; Assemble (gravity off); Drape; Reset; Cancel job; Wireframe; Stage/step progress; Task details. `apps/weftforge/src/main.rs:1984` | `simulate`, `workspace-statistics`, `wireframe` |
| Mesh/simulation/diagnostic properties | Mesh edge length m; Iterations / substeps; Assembly/strain/seam/contact/intersection diagnostics; Peak pattern and material axis. `apps/weftforge/src/main.rs:2437` | `resolution`, `simulation-properties`, `workspace-statistics`, `fit-maps` |

The rows index source controls and existing group IDs. They add no catalog IDs and establish no reference aliases. Native authoring flows reach shared commands, one transaction/history boundary and validated native persistence; view/session and read-only metric routes remain outside authoring. The JSON records the exact current field proposals and pinned source/test anchors for every rechecked group.

## Source-only group rechecks

### `project-new` / `PACK-project-new`

File > New Empty and the New Empty command-search row queue the same post-inspector replacement action. A clean replacement creates Editor::default at revision 0 with no file path, selection, history or derived mesh/cloth, cancels the old worker and retains session view/snap settings. Dirty accepted work, pending or invalid numeric input, unfinished construction and an active autosave cause a retained-state refusal. This native session action is distinct from CLI new/MCP project_create file creation and adds no Command variant.

Open: Reference New Project/New Garment initial-state, discard/confirmation, option and session semantics remain unknown. Read actual native qualification only from the registry/reports at the reported revision. New Empty deliberately refuses replacement until edits/gestures are applied or cancelled and saved; no discard option or new shortcut is supplied.

Source: `apps/weftforge/src/main.rs:941` (`fn new_empty_button(`); `apps/weftforge/src/main.rs:958` (`fn new_empty(`); `apps/weftforge/src/main.rs:10690` (`fn new_empty_file_route_replaces_clean_document_and_retains_files_and_preferences`); `apps/weftforge/src/main.rs:10695` (`fn new_empty_search_route_replaces_clean_document_and_retains_files_and_preferences`).

### `workspace-panels` / `PACK-workspace-panels`

The fixed native studio shell has File/Edit/View/Help menus, an icon tool rail, a context toolbar, document tab/revision, equal 2D Pattern and 3D Garment canvases, Properties with a Scene list, simulation transport and a status/message bar. The existing action/focus path protects numeric drafts while menus, tool choices and command search are used; authoring modals shield background input. Source layout and focus checks are implementation candidates only.

Open: Docking, arbitrary panel/tab visibility, independent windows, asset/material/timeline browsers and full context/accessibility contracts remain open. Source layout assertions do not qualify an actual native window.

Source: `apps/weftforge/src/main.rs:1772` (`fn menu_bar(`); `apps/weftforge/src/main.rs:1907` (`fn tool_rail(`); `apps/weftforge/src/main.rs:1931` (`fn document_tab(`); `apps/weftforge/src/main.rs:5046` (`fn studio_workspace_keeps_creation_and_transport_visible`); `apps/weftforge/src/main.rs:5158` (`fn studio_menus_and_tool_rail_preserve_numeric_drafts_text_and_caret`).

### `workspace-layout` / `PACK-workspace-layout`

A source-defined fixed native layout keeps menu/context rows, paired canvases, right Properties/Scene panel, simulation transport and status row visible. This is one original workspace arrangement, with no saved layout presets or docking model.

Open: Persisted dock/window layouts, display/DPI-safe restore, export/directory handling and reset without document mutation.

Source: `apps/weftforge/src/main.rs:5046` (`fn studio_workspace_keeps_creation_and_transport_visible`).

### `appearance-preferences` / `PACK-appearance-preferences`

The native shell uses a fixed dark theme, original icon drawing, bundled licensed Inter/JetBrains Mono typography and distinct selection/status colors through studio_style. Material RGB editing remains an authoring field. No user palette/theme/typography preference interface is implemented.

Open: Original accessible line/selection/hover/status colors, typography/pixel thickness preferences and bounded user palette resource handling without copying vendor palette values.

Source: `apps/weftforge/src/main.rs:4927` (`fn studio_style(`); `apps/weftforge/src/main.rs:4425` (`fn tool_button(`).

### `help-resources` / `PACK-help-resources`

The native Help menu supplies static original interaction guidance for 2D pan/zoom/snap, whole-piece transform policy, 3D navigation, play/pause and construction confirmation/cancellation. This is discoverable local guidance, with no complete documentation browser, locale routing or community/release-link model.

Open: Branded discoverable original/offline documentation and appropriate external community/release links with locale routing.

Source: `apps/weftforge/src/main.rs:1772` (`fn menu_bar(`).

### `selection` / `PACK-selection`

A piece and boundary selection connect 2D, 3D picking, Scene rows and Properties. Transform mode refuses a cross-piece selection while the current inspector has staged commands or incomplete/nonfinite numeric text; it retains selection, text/caret and history. Normal clean selection still proceeds. This is an original one-piece rule, not observed reference selection semantics.

Open: Multi-object/subobject selection, select-all/invert/sewn traversal, modifier rules and 3D manipulation gizmos remain open.

Source: `apps/weftforge/src/main.rs:1024` (`fn select(`); `apps/weftforge/src/main.rs:13981` (`fn transform_cross_piece_refusal_retains_staged_or_invalid_raw_text_caret_and_redo`); `apps/weftforge/src/main.rs:14039` (`fn transform_scene_row_selection_preserves_staged_and_raw_edits_through_real_ui_actions`).

### `ellipse` / `PACK-ellipse`

AddEllipse creates four editable cubic boundary spans with stable authored points/curves from a name, origin and SI width/height. Properties > Add ellipse and command search use the same shared command, combined with pending inspector edits as one transaction. CLI ellipse and CLI/MCP command arrays expose the same construction; this bounded cubic representation is not an exact analytic conic or a new persistent ellipse primitive.

Open: Reference ellipse gestures, modifiers, constructors/defaults/limits and exact semantic equivalence remain unqualified; no dedicated ellipse tool mode or internal ellipse is implemented.

Source: `crates/weftforge-core/src/document.rs:763` (`Command::AddEllipse {`); `apps/weftforge/src/main.rs:830` (`fn add_ellipse(`); `crates/weftforge-core/tests/ellipse.rs:32` (`fn ellipse_has_declared_controls_ids_bounds_tangents_and_one_persistent_transaction`); `apps/weftforge/src/main.rs:7190` (`fn ellipse_accessible_button_and_search_commit_once_and_preserve_failed_drafts`).

### `pattern-transform-tool` / `PACK-pattern-transform-tool`

TransformPiece performs one-piece rigid 2D translation/rotation about an explicit authored pivot, transforms endpoints and absolute cubic controls once, rotates grain and compensates the existing 3D placement to keep the arranged continuous surface and world warp fixed. IDs, boundary/sewing records and interval attachments remain authored; derived mesh/simulation are invalidated. Native Transform pattern rail/search routes expose a five-field SI modal plus boundary-translation/rotation-handle previews; a gesture combines pending inspector commands and the transform in one transaction. CLI execute/MCP project_execute reuse the shared command. ResizePiece retains its separate existing scaling behavior.

Open: Scaling/mirroring, multiselection, pivot dragging, alignment, angle/axis locks and multi-turn gestures remain open. Reference transform behavior and numerical/gesture equivalence remain unknown. Pointer translation/rotation usability and any native evidence qualification must be read from current registry/reports; source tests do not qualify that route.

Source: `crates/weftforge-core/src/document.rs:1002` (`Command::TransformPiece {`); `apps/weftforge/src/main.rs:2930` (`fn transform_window(`); `apps/weftforge/src/main.rs:2909` (`fn apply_transform(`); `crates/weftforge-core/tests/pattern_transform.rs:569` (`fn invalid_inputs_results_and_late_transactions_retain_every_history_branch`); `apps/weftforge/src/main.rs:13484` (`fn transform_pointer_translation_and_rotation_capture_original_geometry_and_snap_once`).

### `pattern-rotate` / `PACK-pattern-rotate`

The one-piece TransformPiece candidate exposes an explicit radians angle and authored U/V pivot in the native modal, plus a bounds-center rotation handle. Arrangement compensation and grain rotation are deliberate WeftForge layout policy. The source relationship does not establish aliases to the inventoried Rotate/Parallel/Axis commands.

Open: Reference rotate/axis/parallel-target entry routes, selection scope, angle snapping/locks, multi-turn gestures and broader relation/pivot behavior remain open.

Source: `crates/weftforge-core/src/document.rs:1002` (`TransformPiece {`); `apps/weftforge/src/main.rs:2930` (`fn transform_window(`).

### `grid-snap` / `PACK-grid-snap`

The native toolbar offers visible session choices Off, 1 mm, 5 mm and 10 mm. PositionSnap applies independent f64 origin-centred coordinate rounding; a gesture captures its preference and Alt/Option bypass once. Existing point/rectangle/polygon positions use that rule, while transform translation snaps displacement. Numeric dimensions, sewing picks and curve controls are not position-snapped. The visual 0.1 m guide grid is separate, and no snap field is serialized.

Open: Reference snap origin/ties/distance/modifier/persistence and preference/menu equivalence remain unknown. This original session control supplies no adaptive, tangent or smart-snap inference.

Source: `apps/weftforge/src/main.rs:43` (`enum PositionSnap {`); `apps/weftforge/src/main.rs:2537` (`fn position_snap_control(`); `apps/weftforge/src/main.rs:5595` (`fn position_snapping_pointer_geometry_bypass_and_history_use_one_captured_rule`); `apps/weftforge/src/main.rs:5930` (`fn position_snapping_inline_choices_preserve_unfinished_numeric_text_and_caret`).

### `pattern-smart-snap` / `PACK-pattern-smart-snap`

The later PositionSnap selector is an original fixed position-grid candidate. It does not supply curve/point/tangent/angle candidate priorities, smart guides or a reference-equivalent Smart Snap preference. Those requirements remain planned.

Open: Observed curve/point/angle/tangent snap candidates, screen-distance tolerance, priorities, guide display and modifier bypass. The matching preference checkboxes are read-only observations; their equivalence to menu controls and actual snapping/guide effects remains untested.

Source: `apps/weftforge/src/main.rs:43` (`enum PositionSnap {`).

### `pattern-view-extents` / `PACK-pattern-view-extents`

Frame all in View and the context toolbar fits all applied authored exterior outlines using analytic cubic-extrema bounds and 28 logical-pixel padding in the actual 2D canvas. It changes only center/scale, retains drafts/history/selection, disables for an empty document and handles degenerate/invalid canvas bounds. Pan/zoom remain separate view operations.

Open: Reference Zoom Extents All hidden/selected/object-scope, padding, shortcuts, high-DPI behavior and saved-camera semantics remain unknown; the original Frame all source is a candidate relationship only.

Source: `apps/weftforge/src/main.rs:2567` (`fn frame_patterns_button(`); `apps/weftforge/src/main.rs:2582` (`fn frame_patterns(`).

### `pattern-snapshot-print` / `PACK-pattern-snapshot-print`

Existing SVG and boundary-only R2000 DXF exports remain. Additive Import SVG/Import DXF File/search routes now choose a bounded source and show count, physical scale, origin/layout and omitted content before Import/Cancel. Shared ImportSvgPatterns/ImportDxfPatterns validate editable exterior lines/cubics and names with fresh IDs/default material/placement, then combine all pieces and pending inspector edits in one transaction. SVG requires explicit physical mm dimensions and uniform viewBox scale, mapping display Y to authored -Y; the strict DXF writer profile maps WCS X/Y to U/V without inversion. CLI import-svg/import-dxf, dedicated MCP file tools and command arrays reuse the same implementation. This FILE-005 engineering bucket does not invent a reference import catalog entry or equate import with printing.

Open: General SVG/CAD compatibility, apparel conventions, internal features, sewing/material/arrangement round trip, calibrated printing, page tiling and reference semantics remain open. Native save retains the primary on failed replacement under inherited guarantees; recovery may already have advanced to the prior valid primary. Primary and recovery are not one atomic two-file transaction. Actual chooser/modal/edit/history/reopen qualification comes only from registry/reports.

Source: `apps/weftforge/src/main.rs:1315` (`fn choose_import(`); `apps/weftforge/src/main.rs:1359` (`fn pattern_import_window(`); `crates/weftforge-core/src/persistence/svg.rs:54` (`pub fn from_bytes(bytes: &[u8]) -> Result<Self>`); `crates/weftforge-core/src/persistence/dxf_import.rs:50` (`pub fn from_bytes(bytes: &[u8]) -> Result<Self>`); `apps/weftforge/src/main.rs:11551` (`fn chooser_modal_import_commits_pending_edits_and_all_pieces_as_one_undo`).

### `automation-scripts` / `PACK-automation-scripts`

The current Rust enum has 27 variants: 26 nontransaction authoring commands plus Transaction. The MCP command_schema oneOf contains the 26 nontransaction commands; CLI execute/MCP project_execute accept strict original-byte arrays of 1–1024, reject nested Transaction, and commit/save through one shared Transaction. The five additions after the historical snapshot are ImportSvgPatterns, ImportDxfPatterns, AddEllipse, TransformPiece and AddSewingByLength. Nine stateless MCP tools include the two bounded import tools and read-only project_lengths; no tool was added specifically for TransformPiece or physical-length sewing. CLI lengths and project_lengths return unrounded SI authored metrics and source revision. No live GUI control, script runtime, persistent undo or simulation-job API is provided.

Open: Broader scripting/editing/reporting and observed scripting semantics remain open. The nine file tools are not reference scripting parity and do not control an active studio session.

Source: `crates/weftforge-core/src/document.rs:1460` (`pub enum Command {`); `apps/weftforge-cli/src/main.rs:365` (`fn execute_commands(`); `apps/weftforge-cli/src/mcp.rs:528` (`fn command_schema()`); `apps/weftforge-cli/src/mcp.rs:716` (`fn tools()`); `docs/automation.md:134` (`The supported shared commands are`).

### `line-measure-display` / `PACK-line-measure-display`

Applied rest length feedback uses the existing bounded authored interval metric, independent of 3D placement, tessellation and cloth positions. Selected full boundaries and visible legacy seam pairs are cached by applied revision; group rows use matching-revision rest-mesh geometry. Native text uses three-decimal mm and signed B − A, with applied revision and draft/preview exclusion. Missing/error/rebuilding data is explicit. CLI lengths and MCP project_lengths query intervals, legacy seams or groups in unrounded metres without writing or building a mesh. Legacy curve-t correspondence remains unchanged.

Open: Angle/perimeter/persistent measurements, selected-chain aggregation, current deformed 3D lengths and configurable units/precision remain open. Reference length selection/rounding/diagnostic policy is unqualified.

Source: `crates/weftforge-core/src/sewing.rs:634` (`pub fn interval_length(`); `apps/weftforge/src/main.rs:566` (`fn applied_length_heading(`); `apps/weftforge-cli/src/main.rs:31` (`fn length_report(`); `apps/weftforge-cli/src/mcp.rs:511` (`fn length_query_schema()`); `apps/weftforge/src/main.rs:14699` (`fn applied_lengths_are_formatted_cached_and_ignore_drafts_preview_and_simulation`).

### `selected-line-lengths` / `PACK-selected-line-lengths`

Applied rest length feedback uses the existing bounded authored interval metric, independent of 3D placement, tessellation and cloth positions. Selected full boundaries and visible legacy seam pairs are cached by applied revision; group rows use matching-revision rest-mesh geometry. Native text uses three-decimal mm and signed B − A, with applied revision and draft/preview exclusion. Missing/error/rebuilding data is explicit. CLI lengths and MCP project_lengths query intervals, legacy seams or groups in unrounded metres without writing or building a mesh. Legacy curve-t correspondence remains unchanged.

Open: Angle/perimeter/persistent measurements, selected-chain aggregation, current deformed 3D lengths and configurable units/precision remain open. Reference length selection/rounding/diagnostic policy is unqualified.

Source: `crates/weftforge-core/src/sewing.rs:634` (`pub fn interval_length(`); `apps/weftforge/src/main.rs:566` (`fn applied_length_heading(`); `apps/weftforge-cli/src/main.rs:31` (`fn length_report(`); `apps/weftforge-cli/src/mcp.rs:511` (`fn length_query_schema()`); `apps/weftforge/src/main.rs:14699` (`fn applied_lengths_are_formatted_cached_and_ignore_drafts_preview_and_simulation`).

### `sewing-length-check` / `PACK-sewing-length-check`

Applied rest length feedback uses the existing bounded authored interval metric, independent of 3D placement, tessellation and cloth positions. Selected full boundaries and visible legacy seam pairs are cached by applied revision; group rows use matching-revision rest-mesh geometry. Native text uses three-decimal mm and signed B − A, with applied revision and draft/preview exclusion. Missing/error/rebuilding data is explicit. CLI lengths and MCP project_lengths query intervals, legacy seams or groups in unrounded metres without writing or building a mesh. Legacy curve-t correspondence remains unchanged.

Open: Angle/perimeter/persistent measurements, selected-chain aggregation, current deformed 3D lengths and configurable units/precision remain open. Reference length selection/rounding/diagnostic policy is unqualified.

Source: `crates/weftforge-core/src/sewing.rs:634` (`pub fn interval_length(`); `apps/weftforge/src/main.rs:566` (`fn applied_length_heading(`); `apps/weftforge-cli/src/main.rs:31` (`fn length_report(`); `apps/weftforge-cli/src/mcp.rs:511` (`fn length_query_schema()`); `apps/weftforge/src/main.rs:14699` (`fn applied_lengths_are_formatted_cached_and_ignore_drafts_preview_and_simulation`).

### `free-chain-sewing` / `PACK-free-chain-sewing`

A separate native Free sewing tool/search route owns revision-stamped transient A/B drafts. Canvas start/direction and accessible Boundary, exact Start t, Direction and Length (mm) controls feed Create Sewing Line; Tab validates, first-side OK advances without authoring, and second-side OK sends AddSewingByLength through the shared inspector transaction. Project::interval_from_length reuses the bounded metric/inversion for forward/reverse physical length on one curve. The command resolves both sides before allocation and creates one ordinary SewingGroup with one interval per side and normalized physical arc-length correspondence, rather than a legacy curve-t Seam. It rejects overflow, insufficient remaining distance, invalid/overlapping/precision-limited input without clamping. Both pending previews, raw text/caret and applied labels/guides remain distinct; Cancel/tool/load/history/revision rules retain the existing accepted construction. Legacy Sew edges and ordered M:N sewing remain separate routes.

Open: Reference continuous curved endpoints/correspondence, picking and initial values, cancellation/limits, unequal-length easing, M:N execution and physical sewing acceptance remain unqualified. Internal/boundary-traversing/automatic sewing and broader editing/selection remain in scope. Entered lengths choose creation endpoints; they are not persistent dimension locks. Actual native qualification must come from registry/reports, not these source anchors.

Source: `crates/weftforge-core/src/sewing.rs:609` (`pub fn interval_from_length(`); `crates/weftforge-core/src/document.rs:1147` (`Command::AddSewingByLength {`); `apps/weftforge/src/main.rs:2677` (`fn free_sewing_window(`); `apps/weftforge/src/main.rs:12497` (`fn free_sewing_failure_and_cancel_retain_raw_text_carets_redo_and_accepted_state`); `apps/weftforge/src/main.rs:12680` (`fn free_sewing_new_empty_refuses_pending_a_and_b_without_dropping_or_committing_them`); `crates/weftforge-core/tests/sewing_groups.rs:1353` (`fn physical_length_command_is_one_persistent_group_with_physical_correspondence_and_history`).

### `free-sewing-length-preview` / `PACK-free-sewing-length-preview`

A separate native Free sewing tool/search route owns revision-stamped transient A/B drafts. Canvas start/direction and accessible Boundary, exact Start t, Direction and Length (mm) controls feed Create Sewing Line; Tab validates, first-side OK advances without authoring, and second-side OK sends AddSewingByLength through the shared inspector transaction. Project::interval_from_length reuses the bounded metric/inversion for forward/reverse physical length on one curve. The command resolves both sides before allocation and creates one ordinary SewingGroup with one interval per side and normalized physical arc-length correspondence, rather than a legacy curve-t Seam. It rejects overflow, insufficient remaining distance, invalid/overlapping/precision-limited input without clamping. Both pending previews, raw text/caret and applied labels/guides remain distinct; Cancel/tool/load/history/revision rules retain the existing accepted construction. Legacy Sew edges and ordered M:N sewing remain separate routes.

Open: Reference continuous curved endpoints/correspondence, picking and initial values, cancellation/limits, unequal-length easing, M:N execution and physical sewing acceptance remain unqualified. Internal/boundary-traversing/automatic sewing and broader editing/selection remain in scope. Entered lengths choose creation endpoints; they are not persistent dimension locks. Actual native qualification must come from registry/reports, not these source anchors.

Source: `crates/weftforge-core/src/sewing.rs:609` (`pub fn interval_from_length(`); `crates/weftforge-core/src/document.rs:1147` (`Command::AddSewingByLength {`); `apps/weftforge/src/main.rs:2677` (`fn free_sewing_window(`); `apps/weftforge/src/main.rs:12497` (`fn free_sewing_failure_and_cancel_retain_raw_text_carets_redo_and_accepted_state`); `apps/weftforge/src/main.rs:12680` (`fn free_sewing_new_empty_refuses_pending_a_and_b_without_dropping_or_committing_them`); `crates/weftforge-core/tests/sewing_groups.rs:1353` (`fn physical_length_command_is_one_persistent_group_with_physical_correspondence_and_history`).

### `undo-redo` / `PACK-undo-redo`

The shared Editor transaction still owns atomic history. Later ellipse, bounded drawing imports, one-piece transforms and completed physical-length sewing combine pending inspector edits with one command into one history entry. First-side sewing and drag previews are transient. New Empty refuses dirty/pending/unfinished work and resets history only for accepted clean replacement; load/history/revision changes clear stale preview and metric state. Numeric text/caret and redo retention checks are implementation evidence only.

Open: Full observed gesture/history/coalescing semantics, native route qualification, all rejection paths and reference parity remain open; use registry/reports for the exact accepted runtime scope.

Source: `crates/weftforge-core/src/document.rs:1596` (`pub fn execute(&mut self, command: Command)`); `apps/weftforge/src/main.rs:796` (`fn apply_inspector_action(`); `apps/weftforge/src/main.rs:12680` (`fn free_sewing_new_empty_refuses_pending_a_and_b_without_dropping_or_committing_them`).

### `simulate` / `PACK-simulate`

The native transport exposes Play/Pause, Step, Assemble (gravity off), Drape, Reset and Cancel job. Current workers carry process-session task IDs/input revision, count only successful simulation steps and distinguish actual worker outcome from cancellation request/obsolete input before accepting a matching-revision result. Previous accepted display state survives failed/cancelled/obsolete workers. Live cloth contact now uses conservative swept hierarchy candidates in ascending original order, with a retained linear remainder after projection moves the query; static diagnostic contact queries also use the hierarchy. The exact zero-target positive-compliance stitch branch uses vector coordinates, retaining the old hard/positive-target radial branch. These source candidates establish no garment-quality/calibration/reference acceptance.

Open: Full physical garment protocols, equilibrium/calibration, complete deforming collision coverage, representative performance and reference simulation behavior remain open. Dense contact can return to the original scan; no general speedup follows from the hierarchy. A full worker step count is not accepted cloth or garment quality. Retained failures and actual runtime qualification remain authoritative in the registry/reports.

Source: `apps/weftforge/src/main.rs:1984` (`fn simulation_bar(`); `apps/weftforge/src/main.rs:2074` (`fn task_details(`); `apps/weftforge/src/main.rs:9264` (`fn task_progress_correlates_real_workers_and_retains_unaccepted_snapshots`); `crates/weftforge-sim/src/contact.rs:108` (`pub(crate) fn project_cloth(`); `crates/weftforge-sim/src/contact.rs:219` (`pub(crate) fn contact_errors_cancellable(`); `crates/weftforge-sim/src/constraints.rs:164` (`pub(crate) fn solve_stitch(`).

### `workspace-statistics` / `PACK-workspace-statistics`

The status bar reports applied revision, authored piece/legacy seam/group counts, accepted mesh vertices, dirty/autosave and current messages. Task details correlate process-session IDs, input revisions, successful simulation steps and terminal outcome. Inspector cloth diagnostics include assembly remaining, strain/seam/contact errors, initial intersections and uncertain capsule pairs. Current static/live hierarchy behavior is distinct from the historical initial-only BVH note; these values supply no accepted benchmark or quality result by themselves.

Open: Representative declared-hardware performance/memory/viewport/cancel/save measurements, broad overlays and reference counter semantics remain open. Mesh vertex counts are derived from the current accepted snapshot, not authoritative construction.

Source: `apps/weftforge/src/main.rs:1946` (`fn status_bar(`); `apps/weftforge/src/main.rs:2074` (`fn task_details(`); `crates/weftforge-sim/src/contact.rs:227` (`fn analyze_contacts(`).

### `fit-maps` / `PACK-fit-maps`

The inspector now attributes the maximum absolute material-axis length strain to a matching-rest-mesh pattern and warp/weft signed percentage; CLI simulation/benchmark/fitting reports expose the typed attribution. This read-only diagnostic is not a per-element colored stress/strain/pressure map, a principal/shear bound, calibration or a changed fitting acceptance limit.

Open: Per-element dimensional definitions, map shaders, legends/ranges/no-data policy and actual reference Stress/Strain/Pressure behavior remain open. Existing original garment-quality failures are not repaired by adding attribution.

Source: `apps/weftforge/src/main.rs:2446` (`if let Some(peak)=diagnostics.peak_strain`); `spec/features/SIM-005-strain-attribution.md:7` (`The existing maximum absolute material-axis length strain keeps its definition`).

### `pattern-name-property` / `PACK-pattern-name-property`

Piece names remain authored native data. Creation now accepts names through AddPolygon/AddRectangle/AddEllipse and bounded SVG/DXF imports; imported names follow their explicit profiles. The native Pattern name field configures creation and is not a selected-piece rename command.

Open: Validated selected-pattern rename, mixed-selection behavior, command/adapter schema, one-gesture history and persistent display synchronization.

Source: `apps/weftforge/src/main.rs:2330` (`ui.strong("Create pattern")`); `crates/weftforge-core/src/document.rs:763` (`AddEllipse {`).

### `unit-display-preferences` / `PACK-unit-display-preferences`

Core/CPU remain f64 SI. Native creation/transform/placement fields retain explicit metres/radians; rectangle and Free sewing dialogs convert millimetres once, and applied length feedback formats mm while CLI/MCP metric JSON stays unrounded metres. These are fixed engineering presentation choices, with no global unit/precision/locale preference model.

Open: Validated display and construction-specific unit preferences that preserve f64 SI authoring; consistent labels, rounding, parser behavior and adapters without silent physical rescaling.

Source: `apps/weftforge/src/main.rs:538` (`fn length_mm(`); `apps/weftforge/src/main.rs:2677` (`fn free_sewing_window(`); `apps/weftforge/src/main.rs:2930` (`fn transform_window(`).

### `shortcut-preferences` / `PACK-shortcut-preferences`

Fixed command search reaches New Empty, native save/open/recover, bounded SVG/DXF import and vector/mesh exports, Undo/Redo/Reset, Select/Transform/Polygon/Rectangle/Add ellipse and three distinct sewing routes. Fixed global shortcuts are modal/popup/focus guarded; this is not an editable keymap preferences implementation or proof of reference keyboard bindings.

Open: Inventory expanded command descendants and search/edit/delete/reset behavior; configurable keymap with per-tool/modal/focus scope and accessible conflict feedback.

Source: `apps/weftforge/src/main.rs:4296` (`egui::Window::new("Command search")`); `apps/weftforge/src/main.rs:4100` (`let global_shortcuts = !authoring_modal`).

## Historical preservation and integration

The historical JSON SHA-256 is `9323eb803af6d07d6178fd6e808fd61f87e5da71e3b07ab4b279b3141b11c8ec` at `af79f596f247cc48e709b818bb70a30d1b716580` and is byte-identical at the two later pins. Retain its provenance, 949 entry records, all 205 packets/dependencies, first three queued slices, 56 landmark identities/paths and 22-command snapshot. The later source needs current-only anchor replacements for `snap` (`PositionSnap::apply`) and `initial_contacts` (`contact_errors_cancellable`); the historical anchors stay untouched. Other landmark anchors resolve at the later pin, without asserting that all historical scopes were re-reviewed.

This addendum reuses the existing map’s group/source-recheck vocabulary and verifier. A full repin is unnecessary for this bounded update and would imply broader re-review. When a later source is integrated, explicitly re-review affected rows and repin this addendum; do not change the historical snapshot or carry a native result to changed code.

Internal lines/primitives and internal sewing remain planned. An approved future contract alone supplies no implementation candidate. General scaling/mirroring/multiselection, reference snap/measurement/cancellation semantics, full CAD/garment interchange, physical garment quality and calibration remain open.

Before publication, root must close or explicitly retain the separate native attempt, decide the exact integrated source/evidence revision, track every artifact, run registry/map checks from a clean tracked checkout, and run all four required local Cargo gates for the integrated change. No registry field, evidence qualification, root README, Actions configuration or static Pages content is changed by this proposal.
