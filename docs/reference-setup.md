# Reference laboratory

Reference access is **blocked before the garment workspace**. The official macOS
Individual installer was downloaded, verified, installed and launched on
2026-11-13. The latest read-only REA observation at 23:06 UTC shows the Personal
2026.1.237 **CLO-SET sign-in** screen. No Terms modal or garment workspace is
visible in that selected main window. Current terms acceptance and account
entitlement remain unknown.

The investigator earlier observed a Terms of Service dialog and stopped before
accepting. No terms were accepted or credentials entered by the investigator.
The account holder must sign in and activate a legitimate trial or license, and
personally review any terms presented. The official website's Start Trial action
also requires sign-in; eligibility, checkout requirements and subsequent account
verification remain unobserved. Sign-in is the currently observed intervention
needed before reference garment scenarios can run.

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
| Baseline | Provisional; UI build, preferences, units, locale, graphics, and license remain unconfirmed |

A selected-window REA capture independently observed the Personal sign-in UI
showing `2026.1.237`. The full download build comes from the official listing;
it has **not** been confirmed in an About dialog. The bundle's deployment target of macOS 12.0 is
packaging metadata and does not supersede the current vendor requirements.

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

Installers, binaries, source snapshots, signed URLs, screenshots, PID/window
identities, and the private manifest remain outside the public checkout. Public
metadata is limited to redacted provenance, hashes, reviewed behavior contracts,
and qualifying redistributable reports.

## First pending reference runs

Start with numeric rectangle creation, oriented segment sewing with a deliberate
length mismatch, particle-distance editing, and save/close/reopen. Use the reviewed
contracts in `spec/features/`. Preserve the initial state and outputs before
moving to a full empty-project shirt workflow. No garment reference run has yet
executed.
