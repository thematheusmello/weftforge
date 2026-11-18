# Workspace PNG capture

WeftForge implementation contract for a bounded scene-capture increment of
MAT-003 and UI-001. Reference behavior and complete feature parity remain
unverified. This captures the rendered application client, including its 2D/3D
views and controls; it provides no garment-quality or rendering-calibration proof.

- File and command search expose **Save workspace PNG…**. The native chooser
  accepts a PNG destination. Cancel preserves pending inspector text, caret,
  authoring state, history, document revision and the previous destination.
- Capture is a view action. It does not commit unfinished authoring or use the
  simulation job slot. Only one readback request or encoding/staging worker is
  admitted at a time. Ordinary cancellation retains a worker until it retires.
- The renderer supplies the next painted frame. A matching root-viewport token,
  rendered-frame stamp and unchanged document revision are required. New/load/
  recovery, Reset, Escape and shutdown invalidate capture; late results cannot
  replace the destination. Escape remains available to existing modal handlers.
- PNG contains the returned native-size RGBA8 pixels, with straight alpha and
  a standard sRGB intent. No resizing, recreated controls, ICC/device profile,
  EXIF, text, timestamp or pixel-density metadata is added. OS chrome, desktop,
  cursor, dialogs and compositor color transformation are outside this capture.
- Admit at most 16,384 pixels per side and 16,777,216 pixels total; verify the
  returned pixel count and dimensions. Readback expires after ten seconds.
  Encoded output is limited to 80 MiB. These checks bound admitted CPU encoding;
  eframe can allocate GPU/readback pixels before event delivery, including after
  a window resize. No driver or whole-process memory sandbox is claimed.
- Encoding and file staging run in a worker. Only the UI's final validated pass
  may atomically replace the destination. Existing native-project, alias,
  symlink and nonregular-target guards remain in force. Failure, staleness and
  cancellation preserve the old destination.
- Shutdown does not wait for a worker-channel result. A late in-process result
  is dropped without commit. Existing temporary-file cleanup is best effort;
  filesystem calls may stall and process termination may leave a hidden partial
  file. This adds no power-loss or hostile-filesystem guarantee.

Verification requires the pixel/chunk/dimension oracle, request/cancel/stale/
late-result lifecycle cases, native-file preservation, File/search input and
modal Escape cases, then an actual native save with an independently decoded
PNG and unchanged editable project. Implementation-only checks do not establish
native runtime, complete MAT-003, accessibility/high-DPI or reference acceptance.
