# Reference laboratory

Reference garment workspace access is **available**. The account holder handled
sign-in and trial/license activation. A fresh read-only REA capture on
2026-11-14 at 00:31 UTC observed `Untitled_MD.zprj`, both 2D/3D toolbars, Object
Browser, Property Editor, and the exact label `Version:  2026.1.237 (r58920)`.
The running executable matches the retained installer identity. Account greeting,
commercial UI screenshots and raw AX records remain private.

The older Terms and sign-in observations are preserved as history, not current
access blockers. No terms, credentials or account actions were performed by the
investigator. Workspace access does not establish license type, expiry, complete
workflow behavior or reference parity.

## Recorded identity

| Item | Observed value |
| --- | --- |
| Download listing | Individual Mac, 2026.1.237.58920, dated 2026-10-14 |
| Installer filename | `MarvelousDesigner_Personal_2026_1_237_Installer.dmg` |
| Installer SHA-256 | `3eedb7529e3e22a9e6d1953f8201c839dbc7217948454758f9b04552bcea3931` |
| App bundle | `MarvelousDesigner.app` |
| Bundle identifier | `com.clovirtualfashion.MarvelousDesigner_PER` |
| Bundle short version | `2026.1.237`; no `CFBundleVersion` value |
| Executable name | `MarvelousDesigner_PER` |
| Executable SHA-256 | `e7696388a5a5b8ca89bec40f7ab9016096610cd20f7ab50f21348ac73723c76e` |
| Executable architecture | Universal x86_64 and arm64 |
| Signing | CLO Virtual Fashion Inc., team `LM3T6Q78ZL` |
| Gatekeeper assessment | Accepted, Notarized Developer ID |
| Host | macOS 27.2, build 26B123, arm64 |
| Baseline | Provisional; build, Unit System and configured preferences observed; numeric locale and other qualifying conditions incomplete |

The workspace capture independently confirms the full build `r58920`, rather than
relying only on the download listing. The Graphic Options page currently exposes
x4 antialiasing, Alpha Blending, Naive Ordered transparency rendering, and
4096 × 4096 maximum image resolution. The
[read-only preference pass](../spec/features/UI-preferences-runtime-20261114.md)
also observes Unit System Millimeter, Decimal Places 1, Magic Mouse controls,
construction/snap defaults and autosave configuration. No preference was changed.
English UI labels are observed; selected language and regional numeric parsing
remain unknown. A displayed default simulation-property path is absent at a
metadata check; settings actually loaded are not inferred. The baseline stays
unfrozen at revision 1.

The [observed menu catalog](../spec/inventory/md-2026.1.237-main-menus.json) records
17 main menus and 572 nonroot AX entries. These are presence observations, with
505 leaf-like nodes that can include disabled headings or empty submenus. They
are not 505 complete behavior contracts or a whole-product denominator.

The [official requirements](https://www.marvelousdesigner.com/manuals/articles/47358219834649)
list Apple Silicon and macOS 14 minimum, with 15/26 recommended. This host exceeds
the stated minimum but is not one of those explicitly recommended OS versions.
Linux requires an Enterprise Team Linux license. The [trial page](https://www.marvelousdesigner.com/products/trial)
advertises 14 days. Account eligibility was not established.

## Reproduce the installation

1. Inspect the actual host and installed applications. Preserve existing apps.
2. Open the [official download page](https://www.marvelousdesigner.com/products/download).
   Select the installer matching the legitimate license and host; do not synthesize
   a CDN URL. Download links may contain temporary signatures.
3. Move only that download into an operator-controlled research directory outside
   this repository. Record its SHA-256, source page, time, size, and selected build.
4. Mount the DMG read-only. Discover the actual bundle and executable from its
   directory and `Info.plist`; record signing and Gatekeeper results.
5. Copy to a new application destination without overwriting an existing copy.
   Launch using normal OS controls. Complete any required terms/account steps as
   the legitimate account holder; never bypass activation or Gatekeeper.
6. Record the actual running executable identity. macOS may launch a quarantined
   copy through App Translocation; do not assume the installed path equals the
   process path.
7. Record the actual About/build and relevant settings in `parity/reference.json`.
   Keep exploratory cases provisional while settings are incomplete. Freeze the
   baseline before qualifying a reference comparison.

Installers, binaries, signed URLs, commercial screenshots, PID/window
identities, and the private manifest remain outside the public checkout. Public
metadata is limited to redacted provenance, hashes, reviewed behavior contracts,
and qualifying redistributable reports.

## Controlled observations and next runs

The [original numeric rectangle run](../spec/features/PAT-003-runtime-20261114.md)
created 200 × 300 mm through the actual UI, tested one-step Undo/Redo, saved,
closed the document through New Project, reopened it and independently measured
its original OBJ export. Maximum dimension error was 0.000122 mm against the
predeclared 0.01 mm tolerance. That initial document lifecycle does not complete
the broad rectangle feature or establish a WeftForge comparison.

The [bounded resolution run](../spec/features/PAT-011-runtime-20261114.md) changes
the same original rectangle from 20 to 10 mm, checks one Undo/Redo, saves a
separate native file, exports and independently checks its boundary, then observes
its target fields/counts after normal process quit/relaunch and reopening.
Maximum boundary error is 0.000488 mm under the same tolerance. Startup initially
shows a black window and AX failure, later reaches the workspace without another
restart or account action, and the final target-state capture has one unrelated
AX node attribute gap. These limits remain explicit.

The available CUA coordinate route failed for the translocated target. A private,
pinned native pointer driver using public CoreGraphics and Accessibility APIs
then passed exact process/window/frame/foreground/input-owner checks and opened
the actual constructor. The earlier driver failures remain retained history;
canvas input is now available under the investigator's exclusive UI lease.
No permissions, license, terms or account action was performed by that driver.

Separately selected REA modal-window capture still rejected fresh constructor
window IDs. Modal values therefore have native CUA evidence, while untruncated
REA workspace captures cover before/created/history/saved/reopened states.

Next: numeric resize, constructor edge cases and modifiers, particle-distance
limits/cancellation and sewn constraints, then controlled curved and reversed
partial sewing. Continue all menu/flyout/context/panel/dialog,
shortcut, default, range, error and end-to-end discovery. The baseline remains
provisional and verified reference parity remains zero.
