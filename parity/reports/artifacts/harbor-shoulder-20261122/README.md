# Harbor shoulder closure

In both 520 N/m Harbor documents, a 50 mm warp line starting at the
innermost shoulder stitch on seam 64 crosses the 57.5 mm limit at frame
28. The probe started each garment from rest on revision `0a4bc886`,
with eight substeps, 32 sweeps, zero gravity and a sewing speed of 1 m/s.
The original uses front/back faces 6/592. The extension uses 6/570.

| Garment | Frame | Front warp, mm | Back warp, mm | Stitch gap, mm | Minimum possible longer line at these opposite edges and gap, mm |
| --- | ---: | ---: | ---: | ---: | ---: |
| Original | 27 | 56.658 | 56.665 | 29.896 | 54.972 |
| Original | 28 | 57.778 | 57.776 | 13.393 | 56.186 |
| Original | 29 | 59.421 | 59.424 | 0.482 | 57.851 |
| Extended hem | 27 | 56.377 | 56.361 | 30.009 | 54.736 |
| Extended hem | 28 | 58.166 | 57.905 | 13.167 | 56.373 |
| Extended hem | 29 | 59.680 | 59.329 | 0.479 | 58.012 |

## Independent measurement

On each triangle the warp segment runs from the stitch vertex to a point
interpolated along the opposite edge at the same material X coordinate.
For each sampled pose the checker interpolates that point in 3D and
measures its distance from the stitch vertex. The two panels' opposite
edge points are the anchors. If their separation is D, the stitch gap is
g and the two warp lengths are A and B, the triangle inequality gives
A + B ≥ D − g. At least one line must therefore measure (D − g)/2 or
longer. Both lines attain this lower bound with an appropriate pair of
endpoints on the straight line joining the anchors in free space.

At frame 29 both lower bounds exceed 57.5 mm. Preserving the observed gap
and the 15% limit requires motion outside the two stitch vertices. At
frame 28, a rigid shift of both vertices by −8 mm in world X and −5 mm
in world Y preserves their gap vector. Both material axes of both faces
then satisfy the limit. The largest strain is 13.909% in the original
and 14.842% in the extension. A separating plane for each triangle and
each of the six avatar capsules certifies at least 0.425 and 0.452 mm of
clearance respectively, including the 3 mm material offset. These are
two-face geometric witnesses. Forces, neighboring faces, cloth contact
and the path to the witness remain outside that calculation.

## Solver stages

Each row sums the change in front warp length at every call to that
stage during frame 28, in mm. Large opposite sums occur because the
solver repeatedly moves the same vertices during its 256 sweeps.

| Garment | Prediction | Membrane | Bending | Soft sewing | Avatar and ground | Net |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Original | +1.526 | −640.869 | −6.372 | +642.741 | +4.094 | +1.121 |
| Extended hem | +1.791 | −667.703 | −4.425 | +650.787 | +21.339 | +1.789 |

Hard sewing, cloth contact and pins contribute zero direct length change
on this face in that frame. These signed changes telescope to the measured
length difference. Force and work require their own balances. A separate
branch cloned the entire frame-27 state and held the closing actuator
stationary for frame 28. Its front line shortened to 55.157 mm (original)
and 55.327 mm (extended). This comparison tests the effect of advancing
the sewing target over that frame.

## Rejected mesh controls

The earlier experiments on `210ba6bf` ran each construction through
all 181 quality states with the same documents and limits. Conditioning
the interior vertices preserved the boundary, seam samples and
connectivity. Local subdivision added resolution at the sewn-to-free
corners. The table gives the first failed frame and its global peak.

| Mesh | Original | Extended hem |
| --- | --- | --- |
| Baseline | frame 28, 15.556% | frame 28, 16.332% |
| Conditioned | frame 28, 16.693% | frame 28, 16.813% |
| Subdivided corners | frame 27, 20.874% | frame 27, 19.506% |

The original benchmark JSON, experiment patches and process identities are
in [mesh-controls](mesh-controls/). The two November 22 probes ended at
frame 29. The complete fit, resource limits and reference comparison remain
open. The next mechanical test should include the vertices on both opposite
edges and their adjacent shoulder faces in a force-balanced substep.

## Reproduction

From the repository root, run the standard-library checker on the saved
poses and compressed stage streams.

```sh
python3 parity/reports/artifacts/harbor-shoulder-20261122/replay.py
```

To generate fresh captures in a disposable checkout of the tested revision,
apply the observer and run the probe on the two exact native documents.
The observation patch assumes these meshes contain front face 6.

```sh
case_dir=parity/reports/artifacts/harbor-shoulder-20261122
git apply "$case_dir/observe.patch"
mkdir -p crates/weftforge-core/examples
cp "$case_dir/shoulder.rs" crates/weftforge-core/examples/shoulder.rs
cargo build --release --locked -p weftforge-core --example shoulder
for garment in original extended; do
    project="parity/reports/artifacts/native-hem-regeneration-20261120/full-fit/$garment/Harbor.weftforge"
    target/release/examples/shoulder "$project" > "$garment.jsonl" 2> "$garment.trace"
done
```
