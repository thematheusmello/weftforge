# Editable SVG cutouts

A direct SVG path may describe a polygon and one straight-sided cutout with
two closed absolute subpaths. Both retain the drawing's physical scale and
layout. The studio's import review shows the number of patterns and cutouts.
The CLI and MCP return the count of imported holes in the import receipt.

Either subpath can come first. With `evenodd`, either winding can describe
the hole. With `nonzero`, including the default, opposite windings describe
a hole. A same-winding nested boundary under `nonzero` keeps the interior
filled. The importer rejects that compound shape. These fill rules follow
[SVG 2](https://www.w3.org/TR/SVG2/painting.html#FillRuleProperty).

The shared import command creates a piece and calls the existing hole
constructor. The constructor checks the entire inner boundary for containment,
clearance and intersections, and normalizes its winding for the editable
document. Both loops count toward the existing 4,096-span limit. One Undo
removes the complete import. Rejected input preserves the existing project
and its history, including any earlier valid path in the same import.

The original single-contour straight and cubic paths remain supported.
Additional holes, curved cutouts, disjoint compound paths and drawings with
notch marker elements remain outside this profile. No project format change
is required.
