# Attribution and dependency scope

Original WeftForge source is MIT OR Apache-2.0. The original analytic mannequin
and original-shirt fixture are authored for this project under the same terms.

See [dependencies.json](docs/dependencies.json) for all Cargo.lock package license
declarations, including build-time and platform-specific packages. The unsigned
packager records collected runtime Cargo notices and unresolved entries in
`licenses/manifest.json`. The offline notice catalog supplies upstream texts
when the emitted crate's version and revision match. The manifest records
their pinned upstream sources. Complete distribution review covers the final
emitted libraries and platform runtimes.

Notable components: egui/eframe/wgpu (MIT OR Apache-2.0), serde/serde_json
(MIT OR Apache-2.0), earcutr (ISC), rfd (MIT), and tobj (MIT, test-only).
Pinned versions and full declared expressions are in the manifest and inventory.
Pattern PNG export uses png (MIT OR Apache-2.0), tiny-skia and tiny-skia-path
(BSD-3-Clause), arrayref (BSD-2-Clause), and strict-num (MIT). The rasterizer
operates on the CPU without a windowing or graphics driver dependency.
Pattern JPEG export uses the grayscale encoder in image 0.25.10
(MIT OR Apache-2.0). Its JPEG feature resolves zune-core and zune-jpeg
(MIT OR Apache-2.0 OR Zlib) for decoding. The dependency inventory lists
all three packages and their declared licenses.
The ISC-licensed earcutr 0.5.0 source is vendored with cooperative cancellation
hooks. Its original license, source provenance and checksums are retained in
[`vendor/earcutr`](vendor/earcutr/UPSTREAM.md).
The UI embeds Inter Regular and SemiBold (The Inter Project Authors) and
JetBrains Mono Regular (The JetBrains Mono Project Authors), under the SIL Open
Font License 1.1. The original license texts are in
`apps/weftforge/assets/fonts/OFL-Inter.txt` and `OFL-JetBrainsMono.txt`; packaging
includes them in `licenses/` and the macOS application's resources. The exact
upstream source and checksums are recorded beside the font files. The
packager also collects the supplied epaint fallback font notices. The complete
distribution audit includes the font and code licenses of the emitted packages.

The runtime is native Rust application code, not an entirely Rust dependency
or operating-system stack. macOS uses Objective-C/AppKit, CoreFoundation,
CoreGraphics, Metal, Accessibility and platform drivers via Rust bindings;
Windows uses Win32/D3D and platform drivers; Linux uses windowing, GPU, D-Bus
and portal libraries. No browser, Electron, JavaScript frontend, Python service,
Blender subprocess or proprietary SDK is required to run the product.

Development and research tools include REA, Node, Python/jsonschema and
independent export checkers. Cargo tests use tobj as an independent OBJ consumer.
The packager uses sha2 for its SHA-256 manifest.

Engineering reference: [PhotoCraft](https://github.com/storytold/photocraft) and its
[architecture](https://github.com/storytold/photocraft/blob/main/docs/architecture.md)
inform responsibility boundaries and evidence discipline. Application code is
independently authored; the bundled upstream fonts are credited above.
[ArtCraft](https://getartcraft.com/apps) provides product context. Cloth equations
and citations are documented in [physics.md](docs/physics.md).
