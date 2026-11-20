# Fabric assignments in OBJ exports

The studio exports the current rest or draped mesh. The CLI and MCP rest
export use the same writer. Simulation and accepted fit exports also carry
the fabric assignments. Positions use metres and Y up. Each piece retains
its group, material-space UVs and vertex normals.

For a nonempty surface, the OBJ names a sibling MTL file. Each piece group
selects its fabric by a token containing the stable material ID and the ASCII
words of its name. The library includes the full authored name as a comment
and the diffuse RGB channels as `Kd`. Pieces sharing a fabric select the same
definition. Unused fabrics are omitted. The MCP result gives the companion
path. An empty geometry export has a null companion path.

The SHA-256 of the MTL bytes determines its filename. Exporters can reuse a
complete matching library. A fabric edit can publish a new library while
older OBJs still refer to the earlier colors. A conflicting file or symlink
at the library path refuses export and preserves the previous OBJ. The OBJ
is replaced only after the complete library is available. A failed OBJ
replacement can leave a complete reusable library beside the previous file.

This profile covers static diffuse colors. Textures, seam welds, editable
patterns, animation and fabric mechanics require other interchange work.
