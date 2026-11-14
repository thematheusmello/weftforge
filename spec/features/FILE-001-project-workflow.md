# FILE-001 — Editable project persistence workflow

Reference state: **partial runtime observed**. The original rectangle
[document save/close/open run](PAT-003-runtime-20261114.md) preserves its inspected
properties and native file hash. Process restart, broader formats, atomic writes,
failed loads and recovery remain pending. Candidate baseline:
`MD-2026.1.237.58920-macos-arm64-20261113`.

The [official project manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358219651225-Project-File-ZPrj-Open-Save)
describes `.zprj` projects retaining garment/pattern/fabric/material data, avatar
and arrangement data, recorded animation when present, 3D environment settings,
and user simulation options. It documents open, save, save-as, new-project, and
selective load/add operations. Open can also use file double-click or a drop on
the 3D window. New creates an untitled project. Current macOS shortcut mapping,
unsaved-change prompts, atomic writes, embedded-resource behavior, and recovery
are not established by this documentation.

WeftForge's `.weftforge` format is an intentional native-format difference. Its
save/close/reopen workflow must preserve supported editable construction and
relationships, not just a display mesh. Native Marvelous Designer decoding is the
separate **FILE-006** contract and remains unsupported until independently tested.

Proposed scenario `REF-FILE-001-REOPEN-001`: create two named original patterns,
sew them, assign distinct materials, alter simulation settings, save, close,
reopen, then edit and undo. Compare stable authored relationships, dimensions
within 0.01 mm, material assignments, settings, and editability. Test save-as and
canceled load separately. Interrupted-save recovery belongs to FILE-002 and
requires controlled fault injection against disposable original fixtures.
