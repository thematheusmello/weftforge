# SEW-001 / SEW-004 — Segment sewing and diagnostics

Reference state: **documentation reviewed; runtime blocked**. Candidate baseline:
`MD-2026.1.237.58920-macos-arm64-20261113`.

The [official Segment Sewing manual](https://support.marvelousdesigner.com/hc/en-us/articles/47358199517209-Segment-Sewing-2D)
describes sewing pattern segments and internal lines/shapes. Select the first
boundary, then select the second while controlling their directional notches.
Parallel notches indicate the intended matching orientation; crossed notches
produce crossed 3D sewing and can reverse a pattern during simulation. During
second-side selection the length difference is displayed. The manual describes
a blue difference indicator, changing to red when the difference exceeds one
inch (25.4 mm). Actual-build colors, threshold rounding, units, and reversed
orientation behavior still require observation.

Independent construction contract: store oriented intervals on authored curves
with an explicit correspondence. Remeshing must not reduce a seam to vertex-index
pairing. Show pairing, direction, and mismatch before simulation. Unequal lengths
are deliberate easing/gathering inputs, not an automatic validation failure.
Those data-model requirements come from the WeftForge brief.

Proposed scenario `REF-SEW-001-ORIENTATION-001`: original 200 mm and 220 mm edges;
record each orientation and the displayed difference. Repeat around 25.4 mm,
including exactly 25.4 mm, then with curved edges and unequal tessellations.
Compare resolved boundary locations/orientation, lengths within 0.01 mm, and
measured seam closure under separately declared simulation tolerances. Error,
undo, deletion, and post-edit attachment behavior remain unknown.


The [public 2026.1 API list](https://developer.marvelousdesigner.com/list.html)
documents partial-sewing ratios normalized to boundary **length**, plus separate
orientation flags. Our current `CurveInterval` uses curve-parameter `t`. These
representations require an explicit arc-length conversion/comparison; identical
numbers do not imply identical locations on a cubic. The API is documentation
evidence only and has not been executed here.

Proposed follow-up `REF-SEW-001-ARC-LENGTH-002`: use an asymmetric original cubic,
select a known partial length in the reference, and measure the resulting start
and end locations against independently integrated arc length. Compare geometric
locations and seam correspondence, not serialization field values.
