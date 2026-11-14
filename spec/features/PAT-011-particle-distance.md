# PAT-011 — Pattern mesh resolution

Reference state: **documentation reviewed; workspace accessible; 20 mm baseline measured; resolution edit pending**. Candidate baseline:
`MD-2026.1.237.58920-macos-arm64-20261113`.

The [official Particle Distance manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358268602777-Particle-Distance-Setting)
defines particle distance as average point spacing in a garment pattern, affecting
mesh size, simulation speed, and garment quality. It describes selecting patterns,
entering the property under simulation properties, and pressing Enter to apply.
The documented range is 0.8–700.0 mm. It suggests 20 mm while constructing/fitting
and below 5 mm for completed garments; these are recommendations, not measured
defaults or a calibrated material model.

Independent requirement: resolution changes rebuild a derived mesh while
preserving authored boundaries, seam references, materials, and supported
constraints, or produce an explicit invalidation report. They must invalidate
incompatible simulation caches. These relationship rules come from the brief.

Proposed scenario `REF-PAT-011-RESOLUTION-001`: use the same original 200 × 300 mm
rectangle at 20 mm, 10 mm, and 5 mm. Record vertex/face counts, boundary error,
property values, and post-sewing behavior. Verify that author dimensions remain
within 0.01 mm; mesh counts need measurement before any exact expectation is set.
Probe 0.8, 700, adjacent out-of-range values, NaN-like input, undo/redo, and canceled
background work. Clamping, rejection, remeshing algorithm, and defaults remain
unknown.

The [original rectangle baseline](PAT-003-runtime-20261114.md) now supplies one
measured 20 mm case: 290 vertices, 525 triangles and a single 53-edge boundary loop.
The [independent boundary evaluator](../evaluators/original_obj_boundary.py)
checks fixed physical dimensions, boundary positions, connected incidence,
projected winding and area for that unsimulated axis-aligned fixture. A mesh with
an interior hole can preserve its bounds, so the physical-size check alone is
insufficient for this comparison. These measurements are the baseline of one
run; they do not require every mesh at 20 mm to have those exact counts.
No particle-distance edit, Undo/Redo of that edit, or post-edit export has yet
been observed.
