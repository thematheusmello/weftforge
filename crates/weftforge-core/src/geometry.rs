use crate::{Curve, CurveInterval, Error, Id, Piece, Project, Result, Seam, ensure};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_MESH_VERTICES: usize = 100_000;
pub const MAX_MESH_TRIANGLES: usize = 200_000;
const MAX_BOUNDARY_SAMPLES: usize = 20_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundarySample {
    pub piece: Id,
    pub curve: Id,
    pub t: f64,
    pub vertex: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedVertex {
    pub vertices: [usize; 2],
    /// Position = (1-weight)*vertices[0] + weight*vertices[1].
    pub weight: f64,
}

impl WeightedVertex {
    pub fn position(&self, positions: &[[f64; 3]]) -> [f64; 3] {
        std::array::from_fn(|axis| {
            positions[self.vertices[0]][axis] * (1.0 - self.weight)
                + positions[self.vertices[1]][axis] * self.weight
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stitch {
    pub seam: Id,
    pub a: WeightedVertex,
    pub b: WeightedVertex,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh {
    pub doc_revision: u64,
    pub positions: Vec<[f64; 3]>,
    /// Material-space coordinates in metres; exporters retain physical scale.
    pub uv: Vec<[f64; 2]>,
    pub triangles: Vec<[usize; 3]>,
    pub piece_ids: Vec<Id>,
    pub boundary: Vec<BoundarySample>,
    pub stitches: Vec<Stitch>,
}

impl Mesh {
    pub fn validate(&self) -> Result<()> {
        let n = self.positions.len();
        ensure(
            n <= MAX_MESH_VERTICES && self.triangles.len() <= MAX_MESH_TRIANGLES,
            "Mesh resource limit exceeded",
        )?;
        ensure(
            self.uv.len() == n && self.piece_ids.len() == n,
            "Mesh attribute counts differ",
        )?;
        ensure(
            self.positions
                .iter()
                .flatten()
                .chain(self.uv.iter().flatten())
                .all(|v| v.is_finite()),
            "Mesh contains non-finite coordinates",
        )?;
        let mut used = vec![false; n];
        for triangle in &self.triangles {
            ensure(
                triangle.iter().all(|&v| v < n),
                "Triangle index is out of range",
            )?;
            ensure(
                triangle[0] != triangle[1]
                    && triangle[1] != triangle[2]
                    && triangle[2] != triangle[0],
                "Triangle has repeated vertices",
            )?;
            let [a, b, c] = triangle.map(|v| self.uv[v]);
            if cross(a, b, c).abs() <= 1.0e-12 {
                return Err(Error(format!(
                    "Triangle {triangle:?} has degenerate material area: {a:?} {b:?} {c:?}"
                )));
            }
            ensure(
                triangle
                    .iter()
                    .all(|&v| self.piece_ids[v] == self.piece_ids[triangle[0]]),
                "A triangle spans different pattern pieces",
            )?;
            for &vertex in triangle {
                used[vertex] = true;
            }
        }
        ensure(
            used.into_iter().all(|v| v),
            "Mesh has vertices with no incident face",
        )?;
        for sample in &self.boundary {
            ensure(
                sample.vertex < n
                    && self.piece_ids[sample.vertex] == sample.piece
                    && sample.t.is_finite()
                    && (0.0..=1.0).contains(&sample.t),
                "Invalid boundary association",
            )?;
        }
        for stitch in &self.stitches {
            for endpoint in [stitch.a, stitch.b] {
                ensure(
                    endpoint.vertices.iter().all(|&v| v < n)
                        && endpoint.weight.is_finite()
                        && (0.0..=1.0).contains(&endpoint.weight),
                    "Invalid weighted seam endpoint",
                )?;
            }
        }
        Ok(())
    }

    pub fn normals(&self) -> Result<Vec<[f64; 3]>> {
        self.validate()?;
        let mut normals = vec![[0.0; 3]; self.positions.len()];
        for &[a, b, c] in &self.triangles {
            let u: [f64; 3] = std::array::from_fn(|i| self.positions[b][i] - self.positions[a][i]);
            let v: [f64; 3] = std::array::from_fn(|i| self.positions[c][i] - self.positions[a][i]);
            let normal = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            for vertex in [a, b, c] {
                for axis in 0..3 {
                    normals[vertex][axis] += normal[axis];
                }
            }
        }
        for normal in &mut normals {
            let length = normal.iter().map(|x| x * x).sum::<f64>().sqrt();
            ensure(length > 1.0e-14, "Cannot export a collapsed display mesh")?;
            for value in normal {
                *value /= length;
            }
        }
        Ok(normals)
    }
}

impl Project {
    pub fn mesh(&self) -> Result<Mesh> {
        self.mesh_cancellable(|| false)
    }

    pub fn mesh_cancellable(&self, cancelled: impl Fn() -> bool) -> Result<Mesh> {
        check_cancel(&cancelled)?;
        self.validate()?;
        check_cancel(&cancelled)?;
        let mut mesh = Mesh {
            doc_revision: self.revision,
            ..Mesh::default()
        };
        for piece in &self.pieces {
            check_cancel(&cancelled)?;
            let start = mesh.positions.len();
            let mut vertex_for_point = BTreeMap::new();
            for id in &piece.boundary {
                check_cancel(&cancelled)?;
                let curve = self
                    .curve(*id)
                    .ok_or_else(|| Error("Missing boundary curve".into()))?;
                let samples = sample_curve(self, curve, self.resolution)?;
                vertex_for_point.insert(curve.start, mesh.positions.len());
                for &(t, point) in samples.iter().take(samples.len() - 1) {
                    let vertex = mesh.positions.len();
                    mesh.positions.push(piece.placement.transform(point));
                    mesh.uv.push(point);
                    mesh.piece_ids.push(piece.id);
                    mesh.boundary.push(BoundarySample {
                        piece: piece.id,
                        curve: curve.id,
                        t,
                        vertex,
                    });
                }
            }
            ensure(
                mesh.positions.len() <= MAX_MESH_VERTICES,
                "Mesh vertex limit exceeded",
            )?;
            for id in &piece.boundary {
                check_cancel(&cancelled)?;
                let curve = self
                    .curve(*id)
                    .ok_or_else(|| Error("Missing boundary curve".into()))?;
                mesh.boundary.push(BoundarySample {
                    piece: piece.id,
                    curve: curve.id,
                    t: 1.0,
                    vertex: vertex_for_point[&curve.end],
                });
            }
            let flat: Vec<f64> = mesh.uv[start..].iter().flatten().copied().collect();
            let indices = earcutr::earcut(&flat, &[], 2)
                .map_err(|error| Error(format!("Triangulation failed: {error}")))?;
            ensure(
                indices.len() >= 3 && indices.len() % 3 == 0,
                "Triangulation produced no valid faces",
            )?;
            let triangle_start = mesh.triangles.len();
            for triangle in indices.chunks_exact(3) {
                let mut triangle = [
                    start + triangle[0],
                    start + triangle[1],
                    start + triangle[2],
                ];
                let [a, b, c] = triangle.map(|vertex| mesh.uv[vertex]);
                let area2 = cross(a, b, c);
                let scale = distance(a, b)
                    .powi(2)
                    .max(distance(b, c).powi(2))
                    .max(distance(c, a).powi(2));
                // Earcut can emit numerical-zero faces along subdivided straight boundaries.
                if area2.abs() <= f64::EPSILON * scale * 32.0 {
                    continue;
                }
                if area2 < 0.0 {
                    triangle.swap(1, 2);
                }
                mesh.triangles.push(triangle);
            }
            include_collinear_vertices(&mut mesh, start, triangle_start)?;
        }
        improve_triangle_quality(&mut mesh, &cancelled)?;
        refine(self, &mut mesh, &cancelled)?;
        improve_triangle_quality(&mut mesh, &cancelled)?;
        for seam in &self.seams {
            check_cancel(&cancelled)?;
            append_stitches(&mesh.boundary, seam, &mut mesh.stitches)?;
        }
        check_cancel(&cancelled)?;
        mesh.validate()?;
        Ok(mesh)
    }

    pub fn seam_lengths(&self, id: Id) -> Result<[f64; 2]> {
        let seam = self
            .seams
            .iter()
            .find(|seam| seam.id == id)
            .ok_or_else(|| Error("Unknown seam".into()))?;
        let mut lengths = [0.0; 2];
        for (side, interval) in [seam.a, seam.b].into_iter().enumerate() {
            let curve = self
                .curve(interval.curve)
                .ok_or_else(|| Error("Missing seam curve".into()))?;
            let mut previous = curve.evaluate(self, interval.t0)?;
            for step in 1..=128 {
                let next = curve.evaluate(self, interval.parameter(f64::from(step) / 128.0))?;
                lengths[side] += distance(previous, next);
                previous = next;
            }
        }
        Ok(lengths)
    }
}

// Interior flips improve membrane conditioning without moving authored boundary samples.
fn check_cancel(cancelled: &impl Fn() -> bool) -> Result<()> {
    ensure(
        !cancelled(),
        "Meshing cancelled; previous construction retained",
    )
}

fn improve_triangle_quality(mesh: &mut Mesh, cancelled: &impl Fn() -> bool) -> Result<()> {
    for _ in 0..24 {
        check_cancel(cancelled)?;
        let mut edges: BTreeMap<(usize, usize), Vec<(usize, usize)>> = BTreeMap::new();
        for (index, triangle) in mesh.triangles.iter().enumerate() {
            for i in 0..3 {
                let (a, b, opposite) = (triangle[i], triangle[(i + 1) % 3], triangle[(i + 2) % 3]);
                edges
                    .entry((a.min(b), a.max(b)))
                    .or_default()
                    .push((index, opposite));
            }
        }
        let mut touched = BTreeSet::new();
        for ((a, b), adjacent) in edges {
            check_cancel(cancelled)?;
            let [(first, c), (second, d)] = adjacent.as_slice() else {
                continue;
            };
            if touched.contains(first) || touched.contains(second) {
                continue;
            }
            let [pa, pb, pc, pd] = [a, b, *c, *d].map(|i| mesh.uv[i]);
            if cross(pc, pd, pa) * cross(pc, pd, pb) >= -1e-24 {
                continue;
            }
            let mut replacements = [[*c, *d, a], [*d, *c, b]];
            for triangle in &mut replacements {
                let [x, y, z] = triangle.map(|i| mesh.uv[i]);
                if cross(x, y, z) < 0.0 {
                    triangle.swap(1, 2);
                }
            }
            let quality = |triangle: [usize; 3]| {
                let [x, y, z] = triangle.map(|i| mesh.uv[i]);
                cross(x, y, z).abs()
                    / (distance(x, y).powi(2) + distance(y, z).powi(2) + distance(z, x).powi(2))
            };
            let old = quality(mesh.triangles[*first]).min(quality(mesh.triangles[*second]));
            let new = quality(replacements[0]).min(quality(replacements[1]));
            if new > old + 1e-10
                && replacements.iter().all(|t| {
                    let [x, y, z] = t.map(|i| mesh.uv[i]);
                    cross(x, y, z) > 1e-12
                })
            {
                mesh.triangles[*first] = replacements[0];
                mesh.triangles[*second] = replacements[1];
                touched.insert(*first);
                touched.insert(*second);
            }
        }
        if touched.is_empty() {
            break;
        }
    }
    Ok(())
}

pub(crate) fn sample_curve(
    project: &Project,
    curve: &Curve,
    spacing: f64,
) -> Result<Vec<(f64, [f64; 2])>> {
    let a = project
        .point(curve.start)
        .ok_or_else(|| Error("Missing curve start".into()))?
        .position;
    let b = project
        .point(curve.end)
        .ok_or_else(|| Error("Missing curve end".into()))?
        .position;
    let mut samples = vec![(0.0, a)];
    if let Some([c, d]) = curve.controls {
        flatten_cubic([a, c, d, b], [0.0, 1.0], spacing, 0, &mut samples)?;
    } else {
        let steps = (distance(a, b) / spacing).ceil().max(1.0) as usize;
        ensure(
            steps <= MAX_BOUNDARY_SAMPLES,
            "Curve tessellation sample limit exceeded",
        )?;
        for step in 1..=steps {
            let t = step as f64 / steps as f64;
            samples.push((t, [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]));
        }
    }
    Ok(samples)
}

fn flatten_cubic(
    p: [[f64; 2]; 4],
    range: [f64; 2],
    spacing: f64,
    depth: u32,
    samples: &mut Vec<(f64, [f64; 2])>,
) -> Result<()> {
    ensure(
        samples.len() < MAX_BOUNDARY_SAMPLES,
        "Cubic tessellation sample limit exceeded",
    )?;
    let flatness =
        point_segment_distance(p[1], p[0], p[3]).max(point_segment_distance(p[2], p[0], p[3]));
    if distance(p[0], p[3]) <= spacing && flatness <= spacing * 0.01 {
        samples.push((range[1], p[3]));
        return Ok(());
    }
    ensure(
        depth < 20,
        "Cubic could not be tessellated within supported tolerance",
    )?;
    let ab = midpoint(p[0], p[1]);
    let bc = midpoint(p[1], p[2]);
    let cd = midpoint(p[2], p[3]);
    let abc = midpoint(ab, bc);
    let bcd = midpoint(bc, cd);
    let center = midpoint(abc, bcd);
    let middle = (range[0] + range[1]) * 0.5;
    flatten_cubic(
        [p[0], ab, abc, center],
        [range[0], middle],
        spacing,
        depth + 1,
        samples,
    )?;
    flatten_cubic(
        [center, bcd, cd, p[3]],
        [middle, range[1]],
        spacing,
        depth + 1,
        samples,
    )
}

pub(crate) fn validate_contour(project: &Project, piece: &Piece) -> Result<()> {
    // shortcut: cubic intersections use bounded polyline tolerance; use exact curve predicates before precision CAD editing.
    let mut polygon = Vec::new();
    for id in &piece.boundary {
        let curve = project
            .curve(*id)
            .ok_or_else(|| Error("Missing boundary curve".into()))?;
        let samples = sample_curve(project, curve, project.resolution.min(0.02))?;
        for pair in samples.windows(2) {
            ensure(
                distance(pair[0].1, pair[1].1) > 1.0e-9,
                "Boundary has a degenerate edge",
            )?;
        }
        polygon.extend(
            samples
                .into_iter()
                .rev()
                .skip(1)
                .rev()
                .map(|(_, point)| point),
        );
        ensure(
            polygon.len() <= MAX_BOUNDARY_SAMPLES,
            "Piece boundary sample limit exceeded",
        )?;
    }
    let n = polygon.len();
    let area2: f64 = (0..n)
        .map(|i| polygon[i][0] * polygon[(i + 1) % n][1] - polygon[(i + 1) % n][0] * polygon[i][1])
        .sum();
    ensure(area2.abs() > 1.0e-10, "Pattern has zero or near-zero area")?;
    // Bounding-box sweep avoids testing every pair on ordinary refined contours.
    let mut edges: Vec<_> = (0..n)
        .map(|i| {
            let a = polygon[i];
            let b = polygon[(i + 1) % n];
            (
                a[0].min(b[0]),
                a[0].max(b[0]),
                a[1].min(b[1]),
                a[1].max(b[1]),
                i,
            )
        })
        .collect();
    edges.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (position, edge) in edges.iter().enumerate() {
        for other in edges.iter().skip(position + 1) {
            if other.0 > edge.1 + 1.0e-12 {
                break;
            }
            if other.2 > edge.3 + 1.0e-12 || other.3 < edge.2 - 1.0e-12 {
                continue;
            }
            let (i, j) = (edge.4, other.4);
            if i == j || (i + 1) % n == j || (j + 1) % n == i {
                continue;
            }
            ensure(
                !segments_intersect(
                    polygon[i],
                    polygon[(i + 1) % n],
                    polygon[j],
                    polygon[(j + 1) % n],
                ),
                "Pattern boundary self-intersects or touches itself",
            )?;
        }
    }
    Ok(())
}

fn include_collinear_vertices(mesh: &mut Mesh, start: usize, triangle_start: usize) -> Result<()> {
    let mut used: BTreeSet<usize> = mesh.triangles[triangle_start..]
        .iter()
        .flatten()
        .copied()
        .collect();
    for vertex in start..mesh.positions.len() {
        if used.contains(&vertex) {
            continue;
        }
        let point = mesh.uv[vertex];
        let mut found = None;
        'triangles: for (index, triangle) in mesh.triangles.iter().enumerate().skip(triangle_start)
        {
            for edge in 0..3 {
                let (a, b) = (triangle[edge], triangle[(edge + 1) % 3]);
                if point_segment_distance(point, mesh.uv[a], mesh.uv[b]) < 1.0e-10
                    && distance(point, mesh.uv[a]) > 1.0e-9
                    && distance(point, mesh.uv[b]) > 1.0e-9
                {
                    found = Some((index, a, b, triangle[(edge + 2) % 3]));
                    break 'triangles;
                }
            }
        }
        let (index, a, b, c) =
            found.ok_or_else(|| Error("Triangulation lost a boundary vertex".into()))?;
        mesh.triangles[index] = [a, vertex, c];
        mesh.triangles.push([vertex, b, c]);
        used.insert(vertex);
    }
    Ok(())
}

fn refine(project: &Project, mesh: &mut Mesh, cancelled: &impl Fn() -> bool) -> Result<()> {
    for _ in 0..20 {
        check_cancel(cancelled)?;
        let mut split = BTreeMap::new();
        for &[a, b, c] in &mesh.triangles {
            check_cancel(cancelled)?;
            for [a, b] in [[a, b], [b, c], [c, a]] {
                if distance(mesh.uv[a], mesh.uv[b]) > project.resolution * 1.4 {
                    split.entry(edge_key(a, b)).or_insert(0);
                }
            }
        }
        if split.is_empty() {
            return Ok(());
        }
        ensure(
            mesh.positions.len() + split.len() <= MAX_MESH_VERTICES,
            "Refinement exceeds mesh vertex limit; increase resolution",
        )?;
        let mut boundary_edges = BTreeMap::new();
        let mut samples: BTreeMap<Id, Vec<BoundarySample>> = BTreeMap::new();
        for &sample in &mesh.boundary {
            samples.entry(sample.curve).or_default().push(sample);
        }
        for list in samples.values_mut() {
            list.sort_by(|a, b| a.t.total_cmp(&b.t));
            for pair in list.windows(2) {
                boundary_edges.insert(edge_key(pair[0].vertex, pair[1].vertex), (pair[0], pair[1]));
            }
        }
        for (&(a, b), vertex) in &mut split {
            *vertex = mesh.positions.len();
            let piece = mesh.piece_ids[a];
            let uv = if let Some((a, b)) = boundary_edges.get(&(a, b)) {
                let t = (a.t + b.t) * 0.5;
                let curve = project
                    .curve(a.curve)
                    .ok_or_else(|| Error("Missing boundary curve".into()))?;
                mesh.boundary.push(BoundarySample {
                    piece,
                    curve: curve.id,
                    t,
                    vertex: *vertex,
                });
                curve.evaluate(project, t)?
            } else {
                midpoint(mesh.uv[a], mesh.uv[b])
            };
            let placement = project
                .piece(piece)
                .ok_or_else(|| Error("Missing mesh piece".into()))?
                .placement;
            mesh.positions.push(placement.transform(uv));
            mesh.uv.push(uv);
            mesh.piece_ids.push(piece);
        }
        let mut triangles = Vec::with_capacity(mesh.triangles.len() * 2);
        for &[a, b, c] in &mesh.triangles {
            check_cancel(cancelled)?;
            let ab = split.get(&edge_key(a, b)).copied();
            let bc = split.get(&edge_key(b, c)).copied();
            let ca = split.get(&edge_key(c, a)).copied();
            match (ab, bc, ca) {
                (None, None, None) => triangles.push([a, b, c]),
                (Some(m), None, None) => triangles.extend([[a, m, c], [m, b, c]]),
                (None, Some(m), None) => triangles.extend([[b, m, a], [m, c, a]]),
                (None, None, Some(m)) => triangles.extend([[c, m, b], [m, a, b]]),
                (Some(m), Some(n), None) => triangles.extend([[b, n, m], [a, m, c], [m, n, c]]),
                (None, Some(m), Some(n)) => triangles.extend([[c, n, m], [b, m, a], [m, n, a]]),
                (Some(m), None, Some(n)) => triangles.extend([[a, m, n], [c, n, b], [n, m, b]]),
                (Some(m), Some(n), Some(p)) => {
                    triangles.extend([[a, m, p], [m, b, n], [p, n, c], [m, n, p]])
                }
            }
        }
        ensure(
            triangles.len() <= MAX_MESH_TRIANGLES,
            "Refinement exceeds mesh triangle limit; increase resolution",
        )?;
        mesh.triangles = triangles;
    }
    Err(Error("Refinement did not converge within 20 levels".into()))
}

fn append_stitches(
    boundary: &[BoundarySample],
    seam: &Seam,
    stitches: &mut Vec<Stitch>,
) -> Result<()> {
    let mut a: Vec<_> = boundary
        .iter()
        .copied()
        .filter(|sample| sample.curve == seam.a.curve)
        .collect();
    let mut b: Vec<_> = boundary
        .iter()
        .copied()
        .filter(|sample| sample.curve == seam.b.curve)
        .collect();
    a.sort_by(|a, b| a.t.total_cmp(&b.t));
    b.sort_by(|a, b| a.t.total_cmp(&b.t));
    ensure(a.len() >= 2 && b.len() >= 2, "Seam has no mesh boundary")?;
    let mut fractions: Vec<f64> = seam.correspondence.iter().map(|pair| pair[0]).collect();
    for sample in &a {
        let fraction = (sample.t - seam.a.t0) / (seam.a.t1 - seam.a.t0);
        if (0.0..=1.0).contains(&fraction) {
            fractions.push(fraction);
        }
    }
    for sample in &b {
        let fraction = (sample.t - seam.b.t0) / (seam.b.t1 - seam.b.t0);
        if (0.0..=1.0).contains(&fraction) {
            fractions.push(correspondence(&seam.correspondence, fraction, 1));
        }
    }
    fractions.sort_by(f64::total_cmp);
    fractions.dedup_by(|a, b| (*a - *b).abs() < 1.0e-10);
    for fraction in fractions {
        let paired = correspondence(&seam.correspondence, fraction, 0);
        stitches.push(Stitch {
            seam: seam.id,
            a: weighted_sample(&a, seam.a, fraction)?,
            b: weighted_sample(&b, seam.b, paired)?,
        });
    }
    Ok(())
}

fn correspondence(pairs: &[[f64; 2]], fraction: f64, source: usize) -> f64 {
    for pair in pairs.windows(2) {
        if fraction <= pair[1][source] {
            let t = (fraction - pair[0][source]) / (pair[1][source] - pair[0][source]);
            return pair[0][1 - source] + (pair[1][1 - source] - pair[0][1 - source]) * t;
        }
    }
    1.0
}

fn weighted_sample(
    samples: &[BoundarySample],
    interval: CurveInterval,
    fraction: f64,
) -> Result<WeightedVertex> {
    let t = interval.parameter(fraction).clamp(0.0, 1.0);
    for sample in samples {
        if (sample.t - t).abs() < 1.0e-10 {
            return Ok(WeightedVertex {
                vertices: [sample.vertex; 2],
                weight: 0.0,
            });
        }
    }
    for pair in samples.windows(2) {
        if pair[0].t <= t && t <= pair[1].t {
            return Ok(WeightedVertex {
                vertices: [pair[0].vertex, pair[1].vertex],
                weight: (t - pair[0].t) / (pair[1].t - pair[0].t),
            });
        }
    }
    Err(Error("Seam interval is outside mesh boundary".into()))
}

fn edge_key(a: usize, b: usize) -> (usize, usize) {
    if a < b { (a, b) } else { (b, a) }
}
fn midpoint(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
}
fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}
fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
fn point_segment_distance(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let length2 = (b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2);
    if length2 <= 1.0e-30 {
        return distance(p, a);
    }
    let t =
        (((p[0] - a[0]) * (b[0] - a[0]) + (p[1] - a[1]) * (b[1] - a[1])) / length2).clamp(0.0, 1.0);
    distance(p, [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t])
}
fn segments_intersect(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let [ab_c, ab_d, cd_a, cd_b] = [
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    ];
    if ((ab_c > 0.0 && ab_d < 0.0) || (ab_c < 0.0 && ab_d > 0.0))
        && ((cd_a > 0.0 && cd_b < 0.0) || (cd_a < 0.0 && cd_b > 0.0))
    {
        return true;
    }
    point_segment_distance(a, c, d) < 1.0e-10
        || point_segment_distance(b, c, d) < 1.0e-10
        || point_segment_distance(c, a, b) < 1.0e-10
        || point_segment_distance(d, a, b) < 1.0e-10
}

#[cfg(test)]
mod quality_tests {
    use super::*;
    #[test]
    fn cancellation_leaves_authoring_unchanged() {
        let project = Project::default();
        let before = project.clone();
        assert!(
            project
                .mesh_cancellable(|| true)
                .unwrap_err()
                .to_string()
                .contains("cancelled")
        );
        assert_eq!(project, before);
    }
    #[test]
    fn interior_flip_preserves_boundary_and_area_while_improving_sliver() {
        let mut mesh = Mesh {
            uv: vec![[0., 0.], [1., 0.], [1., 1.], [0., 0.1]],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            ..Mesh::default()
        };
        let boundary = mesh.uv.clone();
        let area = |mesh: &Mesh| {
            mesh.triangles
                .iter()
                .map(|t| {
                    let [a, b, c] = t.map(|i| mesh.uv[i]);
                    cross(a, b, c) / 2.
                })
                .sum::<f64>()
        };
        let before = area(&mesh);
        improve_triangle_quality(&mut mesh, &|| false).unwrap();
        assert_eq!(mesh.uv, boundary);
        assert!((area(&mesh) - before).abs() < 1e-12);
        assert!(
            mesh.triangles
                .iter()
                .all(|t| t.contains(&1) && t.contains(&3))
        );
        let stable = mesh.triangles.clone();
        improve_triangle_quality(&mut mesh, &|| false).unwrap();
        assert_eq!(mesh.triangles, stable);
    }
}
