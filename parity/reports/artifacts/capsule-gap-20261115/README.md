# Saved capsule contact counterexamples

Run `python3 -B probe.py` in this directory. The script uses only standard-library
scalar geometry and the hash-bound original project/OBJ files. It does not call
the Rust detector, run simulation or search for a favorable result.

The saved source is `3aa20b1597036e9f1dc9725b52cdc928ec1bae96`. A and B are the
unchanged Harbor and height-edit cases after the fixed 60 assembly + 120 drape
steps. The native A checkpoint later reproduced the same OBJ bytes. A later
solver change does not rewrite these historical outputs.

For each triangle, a fixed nonnegative barycentric combination gives a point on
its closed surface. Projecting that point onto the mannequin's closed capsule
axis gives a capsule penetration lower bound: radius + physical cloth offset
minus Euclidean distance. The three triangle vertices are outside that capsule,
yet the witness point penetrates by approximately 9.0097 mm (A) or 10.6724 mm (B).
Both fail the existing 2 mm screen. The witness values were independently found
before the surface-contact implementation; the 1e-10 m reproduction tolerance
comes from its reviewed static contract.

This packet checks these fixed witnesses, not the global maximum, reference
equivalence, equilibrium, swept collisions or general garment quality. Historical
zero vertex-only diagnostics remain correct within their narrower scope. The
separate A29 material-strain failure also remains; neither final-state image
nor final strain erases that transient failure.
