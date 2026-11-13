# Reference discovery coverage

This is a reviewed public-documentation discovery map, **not a completed menu or
feature inventory**. Reviewed on 2026-11-13 against the provisional
`MD-2026.1.237.58920-macos-arm64-20261113` baseline. The installed reference reached
terms/sign-in; no garment workspace or authoring scenario was executed.

The 56 current registry contracts are broad groups, not a count of all commercial
features. Documentation leads are mapped for **56/56** groups below. Completed reference-workflow
contracts remain **0/56**; the total number of actual reference tools,
parameters and workflows is still unknown. Installation/sign-in evidence is
recorded separately in `parity/reference.json` and cannot qualify a garment contract.

## Registry discovery map

Each row is **unobserved** at runtime. A link identifies an official documentation
lead inspected through its page or section index; it does not establish every
listed behavior, complete current-build coverage, or an implementation. Project
obligations without a vendor equivalent are identified explicitly. Follow-up
questions are our audit plan, not claims of vendor guarantees.

| ID | Registry group | Runtime state | Documentation leads | Required follow-up |
| --- | --- | --- | --- | --- |
| UI-001 | Connected 2D and 3D workspace | unobserved | [Window](https://support.marvelousdesigner.com/hc/en-us/sections/51986372762905-Window) | Panel/layout defaults and all workspace modes. |
| UI-002 | Coherent selection | unobserved | [2D editing](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern), [3D garment editing](https://support.marvelousdesigner.com/hc/en-us/sections/51986665714457-Edit-3D-Garment) | Cross-view selection types, add/remove/range behavior. |
| UI-003 | Shortcuts and command search | unobserved | [Shortcuts](https://support.marvelousdesigner.com/hc/en-us/articles/47358120094361-Shortcuts-Hotkeys-2025-0), [Menus](https://support.marvelousdesigner.com/hc/en-us/articles/47358240361625-Menu) | Actual macOS mappings, command search, context actions. |
| UI-004 | Transactions and history | unobserved | [History](https://support.marvelousdesigner.com/hc/en-us/articles/47358269290393-History-Window) | Gesture boundaries versus 3D-state snapshots; history invalidation. |
| UI-005 | Cancellable background jobs | unobserved | [Simulation](https://support.marvelousdesigner.com/hc/en-us/articles/47358232630937-Simulation), [Simulation properties](https://support.marvelousdesigner.com/hc/en-us/articles/47358125463321-Simulation-Properties) | Responsiveness, cancellation latency and stale-job behavior. |
| UI-006 | Accessibility and high DPI | unobserved | [UI settings](https://support.marvelousdesigner.com/hc/en-us/articles/47358169509273-User-Interface-Above-ver4-2-0), [Requirements](https://www.marvelousdesigner.com/manuals/articles/47358219834649) | Display scaling, keyboard access and AX coverage beyond login. |
| PAT-001 | Editable polygons and straight boundaries | unobserved | [Pattern creation](https://support.marvelousdesigner.com/hc/en-us/sections/51986608298009-2D-Pattern-Shape) | Polygon closure, dimensions and invalid-contour behavior. |
| PAT-002 | Cubic curve authoring | unobserved | [2D editing page 2](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern?page=2) | Bezier representation, curvature and editing semantics. |
| PAT-003 | Rectangles and primitives | unobserved | [Rectangle](https://support.marvelousdesigner.com/hc/en-us/articles/47358203451673-Rectangle), [Pattern creation](https://support.marvelousdesigner.com/hc/en-us/sections/51986608298009-2D-Pattern-Shape) | Creation modifiers, numeric dialog, defaults and bounds. |
| PAT-004 | Pattern transforms | unobserved | [2D editing](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern) | Move/rotate/scale/mirror anchors and numeric precision. |
| PAT-005 | Split and join boundaries | unobserved | [2D editing](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern), [2D editing page 2](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern?page=2) | Split/merge relationship preservation and unsupported cases. |
| PAT-006 | Cut pieces and holes | unobserved | [2D editing](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern), [2D editing page 2](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern?page=2) | Hole winding, cut/sew and topology limits. |
| PAT-007 | Internal features and measurements | unobserved | [Pattern creation](https://support.marvelousdesigner.com/hc/en-us/sections/51986608298009-2D-Pattern-Shape), [2D Snapshot](https://support.marvelousdesigner.com/hc/en-us/articles/47358119414937-2D-Snapshot-2025-0) | Internal geometry, notches and measurement attachments. |
| PAT-008 | Grain direction | unobserved | [2D Snapshot](https://support.marvelousdesigner.com/hc/en-us/articles/47358119414937-2D-Snapshot-2025-0), [Fabric](https://support.marvelousdesigner.com/hc/en-us/sections/51986752872217-Fabric) | Grainline display versus physical orientation and editing. |
| PAT-009 | Seam allowances and offsets | unobserved | [2D editing](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern) | Offsets are indexed; seam-allowance scope is unresolved. |
| PAT-010 | Darts pleats and linked symmetry | unobserved | [Pattern creation](https://support.marvelousdesigner.com/hc/en-us/sections/51986608298009-2D-Pattern-Shape), [2D editing page 2](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern?page=2), [2026.1 features](https://support.marvelousdesigner.com/hc/en-us/articles/59743730927129-Marvelous-Designer-2026-1-New-Feature-List) | Darts/pleats, linked symmetry, parameter-preserving edits. |
| PAT-011 | Resolution and triangulation | unobserved | [Particle distance](https://support.marvelousdesigner.com/hc/en-us/articles/47358268602777-Particle-Distance-Setting) | Resolution bounds, meshing, regeneration and attachment survival. |
| SEW-001 | Oriented interval sewing | unobserved | [Segment sewing](https://support.marvelousdesigner.com/hc/en-us/articles/47358199517209-Segment-Sewing-2D), [API list](https://developer.marvelousdesigner.com/list.html) | Direction and arc-length versus curve-parameter intervals. |
| SEW-002 | Free chain and internal sewing | unobserved | [Sewing](https://support.marvelousdesigner.com/hc/en-us/sections/51986755223705-Sewing) | Free/internal and M:N chains; selection ordering. |
| SEW-003 | Easing and gathering | unobserved | [Segment sewing](https://support.marvelousdesigner.com/hc/en-us/articles/47358199517209-Segment-Sewing-2D), [Garment properties](https://support.marvelousdesigner.com/hc/en-us/sections/51986704279065-Garment-Properties) | Unequal-length ease/gathering correspondence and resulting drape. |
| SEW-004 | Seam diagnostics and editing | unobserved | [Sewing](https://support.marvelousdesigner.com/hc/en-us/sections/51986755223705-Sewing) | Mismatch display, edit/delete/reverse and invalidation behavior. |
| SEW-005 | Construction attachments | unobserved | [Zipper style](https://support.marvelousdesigner.com/hc/en-us/articles/48920706512665-ZIPPER-Zipper-Style), [Garment properties](https://support.marvelousdesigner.com/hc/en-us/sections/51986704279065-Garment-Properties), [Manual index](https://support.marvelousdesigner.com/hc/en-us/categories/51985515993625-Manual) | Buttons, elastic, folds, trims, tack/lacing and attachments. |
| SEW-006 | Topstitching and export welding | unobserved | [Topstitch](https://support.marvelousdesigner.com/hc/en-us/sections/51986890767129-Topstitch), [Proximity welding](https://support.marvelousdesigner.com/hc/en-us/articles/47358323609369-Merge-Vertex-by-Proximity-at-Export) | Visible stitches versus physical sewing and optional export welding. |
| FIT-001 | Avatar import and arrangement | unobserved | [Avatar](https://support.marvelousdesigner.com/hc/en-us/sections/51986513612313-Avatar), [Arrangement](https://support.marvelousdesigner.com/hc/en-us/sections/51986702397337-Arrangement) | Import scale/axes, arrangement aids and reset behavior. |
| FIT-002 | Pins grabbing and layers | unobserved | [Pins](https://support.marvelousdesigner.com/hc/en-us/articles/47358146159769-Pins), [Garment properties](https://support.marvelousdesigner.com/hc/en-us/sections/51986704279065-Garment-Properties) | Pin/cloth grab scopes, hide/isolate and interacting layers. |
| FIT-003 | Edit and regenerate garment | unobserved | [2D editing](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern), [3D garment editing](https://support.marvelousdesigner.com/hc/en-us/sections/51986665714457-Edit-3D-Garment) | Defined regeneration after pattern, seam, fabric and avatar changes. |
| FIT-004 | Drawing on avatar and preserving shape | unobserved | [3D Pencil](https://support.marvelousdesigner.com/hc/en-us/articles/55654873005081-3D-Pencil-Avatar), [3D garment editing](https://support.marvelousdesigner.com/hc/en-us/sections/51986665714457-Edit-3D-Garment) | Drawing-to-pattern conversion and shape-preserving edits. |
| SIM-001 | Documented cloth solver | unobserved | [Simulation properties](https://support.marvelousdesigner.com/hc/en-us/articles/47358125463321-Simulation-Properties) | Observable solver controls; public equations are not established. |
| SIM-002 | Physical fabric parameters | unobserved | [Fabric](https://support.marvelousdesigner.com/hc/en-us/sections/51986752872217-Fabric), [Fabric page 2](https://support.marvelousdesigner.com/hc/en-us/sections/51986752872217-Fabric?page=2) | Units, physical parameter ranges and grain dependence. |
| SIM-003 | Avatar and ground contact | unobserved | [Avatar](https://support.marvelousdesigner.com/hc/en-us/sections/51986513612313-Avatar), [3D environment](https://support.marvelousdesigner.com/hc/en-us/sections/51986433234585-3D-Window-Environment) | Avatar/ground contact thickness, friction and penetration metrics. |
| SIM-004 | Self and layered contact | unobserved | [Garment properties](https://support.marvelousdesigner.com/hc/en-us/sections/51986704279065-Garment-Properties), [Simulation properties](https://support.marvelousdesigner.com/hc/en-us/articles/47358125463321-Simulation-Properties) | Self-contact, garment layering and initial-intersection handling. |
| SIM-005 | Simulation control and diagnostics | unobserved | [Simulation](https://support.marvelousdesigner.com/hc/en-us/articles/47358232630937-Simulation), [Simulation properties](https://support.marvelousdesigner.com/hc/en-us/articles/47358125463321-Simulation-Properties) | Play/pause/reset/step, diagnostics and cancellation. |
| SIM-006 | Convergence and calibration | unobserved | [Particle distance](https://support.marvelousdesigner.com/hc/en-us/articles/47358268602777-Particle-Distance-Setting), [Simulation properties](https://support.marvelousdesigner.com/hc/en-us/articles/47358125463321-Simulation-Properties) | Refinement/timestep convergence and independently measured calibration. |
| SIM-007 | GPU acceleration | unobserved | [2026.1 features](https://support.marvelousdesigner.com/hc/en-us/articles/59743730927129-Marvelous-Designer-2026-1-New-Feature-List), [Simulation](https://support.marvelousdesigner.com/hc/en-us/articles/47358232630937-Simulation) | macOS GPU support/backend and CPU/GPU agreement. |
| MAT-001 | Material assignment and presets | unobserved | [Fabric](https://support.marvelousdesigner.com/hc/en-us/sections/51986752872217-Fabric) | Assignment scopes, physical presets and measured provenance. |
| MAT-002 | Appearance texture and PBR | unobserved | [Material](https://support.marvelousdesigner.com/hc/en-us/sections/51986821785753-Material), [Fabric](https://support.marvelousdesigner.com/hc/en-us/sections/51986752872217-Fabric) | PBR/texture controls independent of physical fabric properties. |
| MAT-003 | Lighting capture and inspection | unobserved | [3D environment](https://support.marvelousdesigner.com/hc/en-us/sections/51986433234585-3D-Window-Environment), [UV Editor](https://support.marvelousdesigner.com/hc/en-us/articles/47358198817689--Mode-UV-EDITOR) | Lighting/capture, strain/contact inspection and UV diagnostics. |
| ANIM-001 | Animated avatars | unobserved | [Avatar](https://support.marvelousdesigner.com/hc/en-us/sections/51986513612313-Avatar), [Blend shapes](https://support.marvelousdesigner.com/hc/en-us/articles/55654243482649-Blend-Shape-Avatar) | Skeleton, motion, blend-shape and collision update behavior. |
| ANIM-002 | Timeline and cache invalidation | unobserved | [Animation](https://support.marvelousdesigner.com/hc/en-us/sections/51986994136857-Animation) | Timeline/frame ranges, recording, scrubbing and cache invalidation. |
| ANIM-003 | Output preparation | unobserved | [Retopology](https://support.marvelousdesigner.com/hc/en-us/articles/47358214227353-Retopology-ver-12-Above), [UV Editor](https://support.marvelousdesigner.com/hc/en-us/articles/47358198817689--Mode-UV-EDITOR) | Retopology, mesh density, normals, UV and thickness preparation. |
| FILE-001 | Editable native projects | unobserved | [Project open/save](https://support.marvelousdesigner.com/hc/en-us/articles/47358219651225-Project-File-ZPrj-Open-Save) | Editable reopen, object selection and settings preservation. |
| FILE-002 | Atomic save and recovery | unobserved | [Default files/autosave](https://support.marvelousdesigner.com/hc/en-us/articles/47358260608793-Set-Default-Files-Above-ver-4-2-0), [Project open/save](https://support.marvelousdesigner.com/hc/en-us/articles/47358219651225-Project-File-ZPrj-Open-Save) | Autosave options versus interrupted-write/recovery guarantees. |
| FILE-003 | OBJ interchange | unobserved | [Import/export](https://support.marvelousdesigner.com/hc/en-us/sections/51986503167513-Import-Export), [Proximity welding](https://support.marvelousdesigner.com/hc/en-us/articles/47358323609369-Merge-Vertex-by-Proximity-at-Export) | OBJ objects/materials/units/axes/normals/UV/thickness options. |
| FILE-004 | glTF and GLB interchange | unobserved | [glTF/GLB](https://support.marvelousdesigner.com/hc/en-us/articles/55686813557913-glTG-2-0-glTF-GLB-File-ver-2026-0) | glTF/GLB static/animated subsets and metadata preservation. |
| FILE-005 | Patterns SVG DXF and printing | unobserved | [2D Snapshot](https://support.marvelousdesigner.com/hc/en-us/articles/47358119414937-2D-Snapshot-2025-0), [API list](https://developer.marvelousdesigner.com/list.html) | 1:1 raster snapshot; SVG/DXF/PDF/physical-print scope unresolved. |
| FILE-006 | Marvelous native compatibility | unobserved | [Native file types](https://support.marvelousdesigner.com/hc/en-us/articles/47358169252633-Marvelous-Designer-File-Format), [API list](https://developer.marvelousdesigner.com/list.html) | Native schemas, CPX coverage, loss reports and independent consumers. |
| FILE-007 | Animation and cache export | unobserved | [Cache export](https://support.marvelousdesigner.com/hc/en-us/articles/47358355178393-Cache-Animation-Export), [Import/export](https://support.marvelousdesigner.com/hc/en-us/sections/51986503167513-Import-Export) | Cache formats, frame timing and axis/unit conversion. |
| APP-001 | Shared command engine and adapters | unobserved | [Python scripting](https://support.marvelousdesigner.com/hc/en-us/articles/47358277921177-Python-Script), [API documentation](https://developer.marvelousdesigner.com/index.html) | Reference scripting surface; Rust shared commands/local MCP are project requirements. |
| APP-002 | Cross-platform packaging | unobserved | [Requirements](https://www.marvelousdesigner.com/manuals/articles/47358219834649) | Our three-platform build/launch/package evidence is a project requirement. |
| APP-003 | Offline operation and licensing | unobserved | [Requirements](https://www.marvelousdesigner.com/manuals/articles/47358219834649), [Trial](https://www.marvelousdesigner.com/products/trial) | Vendor access requires network/licensing; our offline/no-account operation is intentional. |
| APP-004 | Recovery and responsiveness | unobserved | [Default files/autosave](https://support.marvelousdesigner.com/hc/en-us/articles/47358260608793-Set-Default-Files-Above-ver-4-2-0), [Simulation](https://support.marvelousdesigner.com/hc/en-us/articles/47358232630937-Simulation) | Crash recovery, dirty states and responsiveness under real work. |
| APP-005 | Public delivery and evidence | unobserved | [REA coverage](https://github.com/morluto/rea/blob/main/docs/reconstruction-coverage.md) | Public delivery/evidence is a WeftForge requirement, with no claimed vendor equivalent. |
| PERF-001 | Representative performance | unobserved | [3D environment](https://support.marvelousdesigner.com/hc/en-us/sections/51986433234585-3D-Window-Environment), [Simulation properties](https://support.marvelousdesigner.com/hc/en-us/articles/47358125463321-Simulation-Properties) | Representative garment/hardware measurements; no vendor benchmark inferred. |
| REF-001 | Frozen legitimate baseline | unobserved | [Downloads](https://www.marvelousdesigner.com/products/download), [Requirements](https://www.marvelousdesigner.com/manuals/articles/47358219834649) | Installation identity observed separately; full garment baseline not frozen. |
| REF-002 | Complete discovery inventory | unobserved | [Manual index](https://support.marvelousdesigner.com/hc/en-us/categories/51985515993625-Manual), [2026.1 features](https://support.marvelousdesigner.com/hc/en-us/articles/59743730927129-Marvelous-Designer-2026-1-New-Feature-List) | Complete UI/default/error/workflow discovery; actual total remains unknown. |
| REF-003 | Controlled comparisons | unobserved | [REA native observation](https://github.com/morluto/rea/blob/main/docs/guides/native/native-investigation.md), [REA coverage](https://github.com/morluto/rea/blob/main/docs/reconstruction-coverage.md) | Independent controlled comparisons; no authoring replay has run. |

## 2026.1 advanced discovery obligations

The [official 2026.1 feature list](https://support.marvelousdesigner.com/hc/en-us/articles/59743730927129-Marvelous-Designer-2026-1-New-Feature-List)
adds the following obligations. All remain **unobserved** in our running baseline;
feature-list wording is not a tested contract.

| Advertised area | Relevant contracts | Specific public lead / unresolved scope |
| --- | --- | --- |
| Lacing edits | SEW-005 | [Edit 3D Garment section](https://support.marvelousdesigner.com/hc/en-us/sections/51986665714457-Edit-3D-Garment); distinguish paths, attachments and physical response |
| 3D Pencil symmetry side lock | FIT-004, PAT-010 | [3D Pencil](https://support.marvelousdesigner.com/hc/en-us/articles/55654873005081-3D-Pencil-Avatar); conversion, lock and symmetry rules |
| Variable fold angle | SEW-005, PAT-007 | [Variable Fold Angle](https://support.marvelousdesigner.com/hc/en-us/articles/59783425428121-Internal-Line-Variable-Fold-Angle); spatial parameterization |
| Half-symmetry synchronization | PAT-010 | [Edit 2D Pattern](https://support.marvelousdesigner.com/hc/en-us/sections/51986645976473-Edit-2D-Pattern); linked edit and seam effects |
| Zipper placement/end extension | SEW-005 | [Zipper Style](https://support.marvelousdesigner.com/hc/en-us/articles/48920706512665-ZIPPER-Zipper-Style); component dimensions, physical/visual roles |
| Brush pinching | FIT-002 | [Brush Pinching](https://support.marvelousdesigner.com/hc/en-us/articles/59781111736345-Brush-Pinching); selection/strength/falloff |
| Pinching simulation override | FIT-002, SIM-005 | [Override](https://support.marvelousdesigner.com/hc/en-us/articles/59782132043289-Pinching-Simulation-Override); temporary parameters and rest-shape updates |
| Seamline ripping beta | SEW-005, SIM-005 | [Seamline Rip](https://support.marvelousdesigner.com/hc/en-us/articles/59816167297433-Seamline-Rip); force threshold, damage, restore and fray are distinct |
| macOS GPU simulation | SIM-007 | [Release](https://www.marvelousdesigner.com/products/release?tab=feature&v=2026.1); backend, device limits and numerical agreement unknown |
| Collision/render direction alignment | SIM-003, MAT-003 | [Collision Thickness](https://support.marvelousdesigner.com/hc/en-us/articles/47358430412441-FABRIC-PHYSICAL-PROPERTIES-Adjust-Collision-Thickness); physical/display thickness stay separate |
| Imported garment OBJ UV retention | FILE-003, ANIM-003 | [Import/Export](https://support.marvelousdesigner.com/hc/en-us/sections/51986503167513-Import-Export); import/remesh preservation and limits |
| Graphics lock | MAT-002 | Feature-list lead; locking selection/transforms versus persistence still unaudited |
| Blend-shape animation | ANIM-001/002 | [Blend Shape Avatar](https://support.marvelousdesigner.com/hc/en-us/articles/55654243482649-Blend-Shape-Avatar); keyframes and collision updates |
| Camera import/export | MAT-003, FILE-007 | [Viewport Camera](https://support.marvelousdesigner.com/hc/en-us/articles/59790915241241-Viewport-Camera); transform/projection/time data |
| Custom-view display states | MAT-003 | [Custom View](https://support.marvelousdesigner.com/hc/en-us/articles/47358348786969-Custom-View); camera versus visibility state |
| Toolbar customization/global search | UI-003 | [Customize Toolbar](https://support.marvelousdesigner.com/hc/en-us/articles/59782795610393-Customize-Toolbar); platform differences and saved settings |
| Timed autosave | FILE-002, APP-004 | [Default Files](https://support.marvelousdesigner.com/hc/en-us/articles/47358260608793-Set-Default-Files-Above-ver-4-2-0); timing, retention/count, failure/recovery |

Older documentation can describe a different build. GPU limitations stated in
2024.2 articles cannot override newer advertised features without observation.
The API documentation also corrects earlier CLO-only/no-op entries: do not import
grading, colorways, V-Ray or REST assumptions into the MD inventory merely from
old shared API names. See the [official API changelog](https://developer.marvelousdesigner.com/changelog.html).

## Menu, context and preference audit gaps

The [Menu manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358240361625-Menu)
identifies separate 2D/3D background and object context menus, including geometry,
sewing, topology, textures and topstitch contexts. None has yet been enumerated in
the running baseline. Enumerate every command with enabled/disabled states,
selection prerequisites, shortcuts, dialogs, defaults, ranges, undo/redo,
cancellation, errors and resulting editable relationships. Repeat with empty,
single-object, multi-object, simulated and invalid states; include hierarchy,
material, avatar/joint, pin, attachment, timeline and output-specific contexts.

The [UI settings manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358169509273-User-Interface-Above-ver4-2-0)
describes measurement units, grouped tools and display scaling. Record actual
baseline values instead of inheriting manual assumptions. Audit unit-change
restart behavior, locale/decimal input, DPI/font scaling, keyboard focus,
Accessibility exposure, numeric precision, hidden grouped tools, viewport camera,
lighting, graphics and selection conventions. Appearance snapshots cannot
establish physical simulation or contact correctness.

## Native files, API and physical printing

The [native file table](https://support.marvelousdesigner.com/hc/en-us/articles/47358169252633-Marvelous-Designer-File-Format)
distinguishes `.zprj` projects, `.zpac` garments and separate avatar, pose, motion,
fabric, physical-property, arrangement, camera and simulation-property formats.
It describes content roles, not a complete binary schema. Encrypted avatar
limitations are not permission to bypass protection. Native decoding/editability
and data-loss reports remain FILE-006 work, separate from mesh interchange.

The [public API list](https://developer.marvelousdesigner.com/list.html) documents
binary `.cpx` pattern exchange through `ExportPatternCPX`/`ImportPatternCPX`.
Its legacy JSON import/export APIs are deprecated in 2026.1. The conversion API
replaces the current scene: any future probe must use a disposable original
scene. No CPX schema, reference export or native compatibility has been verified.
The same API documents partial sewing ratios in normalized **boundary length**,
with separate directions and success status zero. WeftForge's current intervals
use curve parameter `t`; exact correspondence requires an explicit arc-length
comparison, not an equivalence claim based on field names.

The [2D Snapshot manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358119414937-2D-Snapshot-2025-0)
describes 1:1 JPEG/PNG pattern output with physical units, paper sizes, ppi,
annotations and cancellation. Test a printed ruler/rectangle through an independent
image reader and physical printer configuration before claiming scale accuracy.
[Print on Fabric](https://support.marvelousdesigner.com/hc/en-us/articles/49132239080857-Print-on-Fabric)
is an appearance/image-placement feature, not physical pattern printing. The
bounded API-list searches for `DXF` and `PDF` found no text; that does not establish
product absence. SVG/DXF subset and verifiable printing remain WeftForge target
requirements regardless, with baseline support to resolve separately.

## Hosted creation features require offline substitutes

[AI Studio activation](https://support.marvelousdesigner.com/hc/en-us/articles/49694260744089-AI-Studio-Plug-In-Activation)
is an excluded vendor-account administration operation; the creation features it
controls remain in scope. These public behavior leads are all unobserved, and no
local substitute is implemented:

| Creation workflow | Official lead | Required independent offline behavior |
| --- | --- | --- |
| Texture generation | [AI Texture](https://support.marvelousdesigner.com/hc/en-us/articles/47358351055897--Beta-AI-Texture-Generator-Ver-2024-1) | Generate candidates, preview/history and apply as appearance without changing fabric physics |
| Graphic generation | [AI Graphic](https://support.marvelousdesigner.com/hc/en-us/articles/47358161486361--Beta-AI-Graphic-Generator-Ver-2024-1) | Generate/select/apply graphics, including supported transparent backgrounds |
| Avatar pose generation | [AI Pose](https://support.marvelousdesigner.com/hc/en-us/articles/48920289920537-AI-Pose-Generator-Beta) | Text/image-to-editable pose workflow with validated skeleton/scale and persisted result |
| Parametric/AI pattern drafting | [Pattern Drafter](https://support.marvelousdesigner.com/hc/en-us/articles/48920347302297-Pattern-Drafter-Beta) | Measurement/style or sketch-driven editable construction; retain parameter edits and supported avatar matching |
| Image generation/edit, virtual try-on and flat lay | [AI Image](https://support.marvelousdesigner.com/hc/en-us/articles/51581344460185-AI-Image-Generator) | Original-input image workflows with locally available, appropriately licensed models and explicit output limitations |

Determine input/output contracts, accepted assets, semantic/numerical comparison
criteria, model/asset licenses, compute/storage needs, cancellation and offline
availability before implementation. A paid hosted proxy, pre-generated visual,
or control with no creation behavior cannot close these contracts. Drafting,
pose, material and image concerns span existing PAT/FIT/MAT/ANIM/APP groups;
retain distinct scenario IDs as the registry is refined rather than hiding them
inside a generic implemented group.

## Remaining discovery boundary

Other indexed areas still need full article/runtime audits: trims/buttons,
piping/binding, tacks, lacing, puckering, graphics, modular blocks, UV baking,
retopology, avatar fitting/rigging, scripting and all output modes. Some section
indexes are paginated; Edit 2D Pattern and Fabric page 2 were checked, while other
additional pages remain unaudited. The feature list, manual index, public API and
this 56-row map are complementary discovery evidence, never proof that the entire
product has been examined or that a workflow passed.
