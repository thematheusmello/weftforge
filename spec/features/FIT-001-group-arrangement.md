# Arrange a pattern selection

Select a pattern in the Scene, then Shift-click other patterns to add or
remove them. The Scene’s **Select multiple** checkbox also toggles membership
on each click. In the 2D Select tool either an outline or the interior of a
current rest triangle can identify its pattern. The garment view picks the
visible frontmost triangle. The Scene and both canvases highlight the
same selection. A normal selection replaces the group.

The most recently added pattern is active. Its properties and placed axes
remain visible. Removing the active pattern makes the remaining pattern with
the lowest stable ID active. The selection belongs to the open workspace.
A fresh document, including one reusing the same pattern IDs, starts with an
empty selection.

**Move pattern** accepts a drag on any selected rest piece. Each member gets
the same world-space offset from the view plane, including the current snap
or Option/Alt bypass. **Rotate pattern** uses the mean of the selected pieces’
placed bounds centers. Each individual center comes from the exact pattern
bounds, including cubic extrema. The view, world, or active pattern’s local
axis passes through this shared pivot. Offsets between the members rotate
along with their orientations. A single pattern keeps its own center.

A drag previews the placements until release. All members commit through one
transaction of existing SetPlacement commands. Undo and Redo apply to the
whole gesture. Invalid placements leave every member in its previous state.
Changing membership or losing the eligible rest state cancels a preview.
Hide and Isolate operate on the selection. Construction geometry, seams and
materials stay attached to their original stable IDs.
