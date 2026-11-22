# Native minimum-size garment timeline

The Debian 13.7 arm64 package was built offline from `1497cb3e` with Rust
1.94.1. Four original 1000 × 650 PNG captures show the active recorder, the
last frame, the result of one Left key, and the saved physical edit. Sway
reported one focused WeftForge window with the same rectangle for each capture.
The studio exited normally and the dedicated VM was stopped.

The package audit read all 443 staged files. Its manifest covers 243 emitted
Cargo crates. The release binaries, notices and manifest are identified by their
SHA-256 digests in the retained ledger. The full build input list, native steps,
captured window geometry and cleanup observations accompany the run.

From this repository, replay the capture and saved-project checks with
`python3 parity/reports/artifacts/debian-wayland-minimum-timeline-20261122/check_minimum.py`.
Adding `--ocr` reads the recorder, selected frame, latest solver time and saved
status from the original screenshots using Tesseract. The screenshots also
retain the native controls for direct inspection.

`MACOS-COMPANION.json` records a separate Apple Silicon smoke run on the same
source. It opened Harbor 670, recorded one step, and reproduced the bent
swatch's fourth-step export byte for byte. Both macOS package copies contained
298 Cargo notice files for 177 crates, with five unresolved attribution entries.
