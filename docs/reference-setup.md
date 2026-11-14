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
| Baseline | Provisional; full UI build and Graphic Options observed; units, numeric locale and simulation settings incomplete |

The workspace capture independently confirms the full build `r58920`, rather than
relying only on the download listing. The Graphic Options page currently exposes
x4 antialiasing, Alpha Blending, Naive Ordered transparency rendering, and
4096 × 4096 maximum image resolution. No preference was changed. English UI labels
are observed; selected language, numeric locale, units and simulation settings
remain incomplete. The baseline stays unfrozen at revision 1.

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
7. Before evaluating features, freeze an About/build capture and all relevant
   settings in `parity/reference.json`, then run the original controlled fixtures.

Installers, binaries, signed URLs, commercial screenshots, PID/window
identities, and the private manifest remain outside the public checkout. Public
metadata is limited to redacted provenance, hashes, reviewed behavior contracts,
and qualifying redistributable reports.

## First pending reference runs

Start with numeric rectangle creation, oriented segment sewing with a deliberate
length mismatch, particle-distance editing, and save/close/reopen. Use the reviewed
contracts in `spec/features/`. Preserve the initial state and outputs before
moving to a full empty-project shirt workflow. Rectangle tool activation and its `S` shortcut were observed; numeric creation,
editing and persistence are still pending. CUA coordinate input to this live
translocated app returned `noWindowsAvailable`; custom Preferences navigation
returned `AXError.notImplemented`. These are tooling gaps, not absent product
features. Supported menus and AX controls continue to be inspected.
