# Observed preference pages and configured values

This read-only pass visits the 11 visible preference categories in Marvelous
Designer Personal **2026.1.237 (r58920)**, executable `e7696388…c76e`. It adds 138
component observations to the [catalog](../inventory/md-2026.1.237-components.json).
Each source record is pinned by hash. Current values are observations of this
configuration, not immutable factory defaults. No preference value, account,
path, plugin or connection is changed.

Ordinary exact-process activation and the exposed Preferences Raise action make
the category list accessible. Up/Down on that focused list changes the displayed
page. Earlier CUA coordinate click/drag attempts returned `AXError.notImplemented`;
the guarded pointer dry-run refused foreground ownership and was never run.
After activation, the same Preferences window/frame changes from CG layer 0 to
layer 8. REA captures the former state but rejects selection of the latter.
The foreground pages therefore have private native AX transition records and
viewport screenshots, bound to the identified executable. A transition diff is
not a complete snapshot of every collapsed or offscreen UI element.

| Page | Selected observations |
| --- | --- |
| Graphic Options | x4; Alpha Blending; Naive Ordered; maximum 4096 × 4096 pixels, reobserved after process restart |
| View Controls | Magic Mouse; Shift axis constraint; Smart Panning, Auto Focus and focus reset checked; displayed camera shortcut defaults 0 / 0 / 3000 |
| Shortcuts | Menu Name and Shortcut Key search fields; 16 collapsed command categories; disabled Action hints for Create Pin, QWE-gizmo Pin, and Move Segment Point Only |
| User Interface | Unit System Millimeter; Decimal Places 1; Use Ligne off; topstitch thickness unit Millimeter; both toolbars Top |
| 3D | Triangle mesh; Curved arrangement; point-size value 18; fold-angle sync checked; Unified Gizmo |
| 2D | Configured default particle-distance value 20; copied-pattern numeric indexing; automatic cut offset checked |
| Other | Metadata option off; observed modular-library restrictions off; updated-name replacement selected; LiveSync checked; path/action controls present |
| Snap | Smart Guide and Snap to Pattern checked; Snap to Grid unchecked |
| Default Files | Autosave interval 10 minutes, maximum 5 files, retention 5 days, Project (ZPRJ) type; path and resource controls present |
| CLO API | Add/Delete plugin controls, empty Feature/Position table and unpopulated PLM Settings section |
| CLO-SET | Account-switch control present; identity stays private and no account action is executed |

Displayed binding selectors are recorded literally in the catalog. Their values
do not establish whether a click, drag, modifier combination or conflict works in
the canvas. The shortcut Action hints are disabled UI rows and are not treated as
verified active shortcuts. Camera shortcut defaults are not the current camera
pose. Units absent beside a control remain unspecified; Unit System alone is not
used to invent a gravity, angle or camera-distance unit.

The [official User Interface manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358169509273-User-Interface-Above-ver4-2-0)
locates the unit setting in this page. Actual Unit System and Decimal Places are
now observed, but regional numeric parsing and the selected Language setting
remain unknown. Opening Language and moving its keyboard highlight does not
establish a checked language. No language choice is activated.

One configured default Simulation Property path refers to the previous operating
system translocation and is absent at a metadata-only existence check. Its path
and any preset contents remain private; no resource contents are read. This does
not establish the settings actually loaded or the cause of the earlier startup
delay. The path is not repaired or replaced during this pass.

Font/display subfields, collapsed shortcut descendants, alternative selector
options, accepted ranges, invalid input, resets, settings persistence, actual
autosave/retention/recovery, plugin behavior and account switching remain
unexecuted or unobserved. Palette values and commercial assets are excluded.
Baseline revision 1 stays provisional and unfrozen. These observations supply
zero verified reference-parity claims.
