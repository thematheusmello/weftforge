# FILE-004 static glTF exports

The desktop File menu and Command search export the current cloth surface as
binary glTF 2.0. Before simulation, they use the arranged rest mesh. The CLI
`export-glb` and MCP `project_export_glb` export the rest surface. CLI
`simulate` and a successful `fit` use GLB when the output extension is `.glb`.

**Export glTF**, CLI `export-gltf` and MCP `project_export_gltf` use the same
scene and geometry as GLB. The JSON form contains one embedded Base64 buffer
with media type `application/octet-stream`. Desktop exports select the rest or
current surface. CLI and MCP export the rest mesh, with `.gltf` also available
for CLI simulation and successful fitting output.

Each authored piece becomes a named mesh and scene node. Positions use metres
and Y up, and triangle winding and material assignments follow the selected
mesh revision. Vertex normals are derived from winding. Texture coordinates
use `(U, -V)` with one UV unit per material-space metre. The stored linear
RGB becomes the base color of a thin, double-sided nonmetallic surface.
Scene extras retain the source revision and piece and material IDs.

All vertex attributes in the file use float32. The exporter rejects stale
positions and faces that collapse at that precision. It stages the complete
file before replacement and protects native project, autosave and recovery
destinations. Desktop cancellation retains the pending edit state.

The implementation check uses Harbor revision 23. Export the rest surface,
perform one manual simulation step and export the current surface. Require
zero Khronos Validator errors or warnings and the expected two named pieces,
654 vertices and 1172 triangles. Compare indices, normals, UVs, fabric tint
and metre positions with an independently parsed OBJ and the source project.
Require index equality, unit normals within `1e-6` and position error below
`1e-6` m after float32 conversion. The advanced output must contain moved
vertices and preserve the editable project bytes.

Decode the JSON glTF buffer independently and compare it to the GLB binary
chunk. The JSON scene must match after removal of the embedded buffer URI.
Validate both native outputs and retain exact bytes for the original editable
project. Both containers share the same stale-mesh and float32 checks.

glTF import, avatars, rigging, textures, animation, thickness and
welding remain open. This WeftForge implementation check leaves the reference
runtime comparison open.

[Khronos glTF 2.0 specification](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html).
