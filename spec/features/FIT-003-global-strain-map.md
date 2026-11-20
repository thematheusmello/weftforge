# Global garment strain map

**View → Strain map** or its Command search entry opens the read-only fit-map
panel. Its 3D face colors represent maximum principal stretch. A ratio of
1.0 means rest length. A ratio of 1.2 means a 20% increase in the most
extended material direction. The existing peak warp/weft diagnostic measures
a different quantity.

For each face WeftForge uses the two edges in physical rest UV metres and
their corresponding edges on the latest accepted 3D surface. The gradient
is F = Ds Dm⁻¹. The displayed ratio is the square root of the larger
eigenvalue of FᵀF, including its cross term. A pending worker leaves the
last accepted surface visible. Document revisions and mesh replacement
invalidate old measurements. An empty or unavailable surface has no peak.
Hidden faces do not contribute to the displayed visible peak.

The global legend uses an original continuous cool-to-hot palette, one
unlit color per face. The upper percentage defaults to 120 and accepts 100
through 1000. Ratios below 1 share the cool endpoint. At the upper limit
and above, the hot endpoint saturates. At a 100% upper limit, values up to
1 + 10⁻¹² are cool and larger ratios are hot. The legend and visible peak
remain in physical percentages when the range changes.

The 2D view draws original, assigned fabric colors from the current rest
triangles under the editable outlines, selection and annotations. Cutouts
remain open. An obsolete mesh or active construction preview suppresses the
old fill. Switching the 3D map or changing its scale leaves the 2D fabric,
project, accepted cloth and undo history intact.

The reviewed reference observation records the display route, accepted
global range and unchanged 2D sample colors. The reference tensor,
thresholds, interpolation, compression response, 100% endpoint response
and fabric-specific calibration remain unverified. This implementation
covers global per-face stretch. Directional fabric mode, graph-style modes,
stress and pressure remain open.
[Reviewed reference observation](../observations/REF-SIM-005-GARMENT-STRAIN-MAP-001.json).
