# PAT-011 — Pattern mesh resolution

Reference state: **20 to 10 mm edit, one-step history, separate save/export and target-state process reopen observed; broad contract incomplete**. Candidate baseline:
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
The [bounded runtime run](PAT-011-runtime-20261114.md) now observes a 20 to 10 mm
edit, one Undo/Redo, separate native save and post-edit OBJ export. At 10 mm,
1127 vertices and 2149 triangles preserve the declared rectangle with maximum
boundary error 0.000488 mm. Exact counts are observations of this fixture rather
than a required universal triangulation. After a normal quit, the new process
initially shows a black startup window with an AX failure. It subsequently reaches
the workspace without another restart or account action, and the saved PD10
target fields/counts are observed after reopening. The final whole-window AX
capture has one unrelated node attribute gap; this limit and the earlier startup
failures remain explicit.
