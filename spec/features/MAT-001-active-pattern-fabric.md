# Fabric assignment for an active pattern

In Properties, select **Fabric**. **Assigned fabric** shows the authored name.
Open it to choose any fabric already in the project. Rows include stable IDs
so fabrics with the same name can be distinguished. The assignment changes
only the active pattern, including when several patterns are selected.

**Copy for this pattern…** opens a name editor. **Create and assign copy**
creates a fresh material from the current fabric and reassigns the active
pattern in one undoable transaction. The copy inherits color, mass per area,
warp, weft, shear and bending stiffness, damping, friction and collision
thickness. Subsequent edits to this copy affect the patterns assigned to it.
A complete fabric property edit staged while opening the copy goes onto the
copy. Reassignment to an existing fabric requires completing the current
fabric property edit first.

**Rename fabric…** changes the name wherever the same fabric is assigned.
Escape, Cancel or closing the naming dialog discards its proposed name.
Names use the project's existing 256-byte UTF-8 limit. Invalid names and
exhausted project limits leave the project and its history intact. Undo
and Redo restore material assignments and authored properties together.

The picker works with the current project's local fabric definitions.
Regional assignment, a reusable external library and measured physical
presets remain outside this workflow.
