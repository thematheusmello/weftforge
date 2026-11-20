# Import a flat OBJ garment as editable patterns

Choose **Import garment OBJ…** in File or Command search, select the OBJ and
choose the length unit used for its 3D vertices. The studio offers millimetres,
centimetres, metres and inches. The review shows the number of panels, cutouts
and imported fabrics before you commit. Changing units runs the preview again.
Cancel keeps your pending edits. Import adds the reviewed garment to the open
project in one undoable action.

Each continuous triangle island becomes a pattern with straight boundary
segments and editable cutouts. The vertices in 3D determine physical scale and
rigid placement. The importer uses texture-coordinate connectivity to distinguish
islands. It traces in material space, so either orientation of a flat panel can
be recovered. OBJ groups or object names provide pattern names. A sibling MTL
can supply fabric names and diffuse colors. New fabrics start with the default
cloth properties. The OBJ carries no sewing relationships or original spline
controls to restore.

For example, an OBJ with millimetre vertex coordinates can be added to an
existing project and saved as a new editable document.

```sh
weftforge-cli import-obj existing.weftforge edited.weftforge pocket.obj 0.001
```

The MCP file tool `project_import_obj` accepts `input`, `output`, `obj` and
`metres_per_unit`. An `import_obj_patterns` authoring command carries OBJ text,
optional MTL text and the same positive scale, including inside a shared
transaction. Invalid imports preserve the current project and recovery file.

## Supported geometry and limits

Use triangulated panels with texture coordinates and a consistent winding.
Each island must admit a uniformly scaled, rigid UV-to-3D mapping. The importer
rejects folded or stretched charts, nonmanifold edges, conflicting material
assignments within one island and texture maps. One local material-library
filename may contain spaces. File reads are bounded to 8 MiB of OBJ and
256 KiB of MTL. An import can add up to 64 patterns and 4,096 traced boundary
segments.

Draped-garment reconstruction, avatar meshes, fitted curves, sewing, texture
transfer and animated OBJ workflows remain open. See [surface export and
material handling](FILE-003-obj-fabrics.md) for the other direction.
