# Fabric metallic and roughness

Select a pattern and open **Properties → Fabric**. Metallic and Roughness
belong to the assigned fabric and apply to every pattern sharing it.
Both fields accept finite, unitless values from 0 to 1. Applying an invalid
value preserves the preceding document and accepted cloth state.

An accepted appearance edit, Undo or Redo preserves the pose and solver
time. A physics or construction edit invalidates the affected results.
A mesh or simulation job can finish across appearance-only revisions.
Its history records the revision that supplied its inputs.

Projects store explicitly authored factors in a v7 material `surface`
object. A material without that object has metallic 0 and roughness 1.
The CLI `fabric-surface` and MCP `project_execute` command
`set_fabric_surface` accept the material ID and both values.

The 3D studio preview shades the finish with its world-space key light.
The 2D pattern retains its flat fabric color. The static glTF and GLB
exporters use the saved values for `metallicFactor` and `roughnessFactor`.
The editable project retains both the material assignment and parameters.
