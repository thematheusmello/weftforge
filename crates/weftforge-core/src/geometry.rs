use crate::{
    Curve, CurveInterval, Error, Id, Piece, Project, Result, Seam, SewingGroupGeometry, ensure,
};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_MESH_VERTICES: usize = 100_000;
pub const MAX_MESH_TRIANGLES: usize = 200_000;
const MAX_BOUNDARY_SAMPLES: usize = 20_000;
const MAX_PENDING_GEOMETRY_TASKS: usize = 65_536;

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
    /// Authored length/portion previews belonging to `doc_revision`.
    pub sewing_groups: Vec<SewingGroupGeometry>,
}

impl Mesh {
    pub fn validate(&self) -> Result<()> {
        self.validate_context(None)
    }
    pub(crate) fn validate_context(&self, work: Option<&crate::offset::Work<'_>>) -> Result<()> {
        let n = self.positions.len();
        ensure(
            n <= MAX_MESH_VERTICES && self.triangles.len() <= MAX_MESH_TRIANGLES,
            "Mesh resource limit exceeded",
        )?;
        ensure(
            self.uv.len() == n && self.piece_ids.len() == n,
            "Mesh attribute counts differ",
        )?;
        for (position, uv) in self.positions.iter().zip(&self.uv) {
            charge(work)?;
            ensure(
                position.iter().chain(uv).all(|v| v.is_finite()),
                "Mesh contains non-finite coordinates",
            )?;
        }
        let mut used = vec![false; n];
        for triangle in &self.triangles {
            charge(work)?;
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
        for used in used {
            charge(work)?;
            ensure(used, "Mesh has vertices with no incident face")?;
        }
        for sample in &self.boundary {
            charge(work)?;
            ensure(
                sample.vertex < n
                    && self.piece_ids[sample.vertex] == sample.piece
                    && sample.t.is_finite()
                    && (0.0..=1.0).contains(&sample.t),
                "Invalid boundary association",
            )?;
        }
        for stitch in &self.stitches {
            charge(work)?;
            for endpoint in [stitch.a, stitch.b] {
                ensure(
                    endpoint.vertices.iter().all(|&v| v < n)
                        && endpoint.weight.is_finite()
                        && (0.0..=1.0).contains(&endpoint.weight),
                    "Invalid weighted seam endpoint",
                )?;
            }
        }
        let mut portion_count = 0;
        for group in &self.sewing_groups {
            charge(work)?;
            portion_count += group.portions.len();
        }
        ensure(
            self.sewing_groups.len() <= crate::MAX_OBJECTS
                && (self.sewing_groups.is_empty()
                    || self.stitches.len() <= crate::MAX_SEWING_STITCHES)
                && portion_count <= crate::MAX_SEWING_PORTIONS,
            "Sewing preview resource limit exceeded",
        )?;
        if self.sewing_groups.is_empty() {
            return Ok(());
        }
        let mut curves = BTreeSet::new();
        for sample in &self.boundary {
            charge(work)?;
            curves.insert(sample.curve);
        }
        let mut stitched = BTreeSet::new();
        for stitch in &self.stitches {
            charge(work)?;
            stitched.insert(stitch.seam);
        }
        let mut group_ids = BTreeSet::new();
        for group in &self.sewing_groups {
            charge(work)?;
            ensure(
                group.id != 0
                    && group_ids.insert(group.id)
                    && stitched.contains(&group.id)
                    && group
                        .lengths
                        .iter()
                        .all(|length| length.is_finite() && *length > 0.)
                    && !group.portions.is_empty(),
                "Invalid sewing group preview",
            )?;
            for portion in &group.portions {
                charge(work)?;
                for interval in [portion.a, portion.b] {
                    ensure(
                        curves.contains(&interval.curve)
                            && interval.t0.is_finite()
                            && interval.t1.is_finite()
                            && (0.0..=1.).contains(&interval.t0)
                            && (0.0..=1.).contains(&interval.t1)
                            && interval.t0 != interval.t1,
                        "Invalid sewing group preview portion",
                    )?;
                }
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
        self.mesh_context(&cancelled, None)
    }
    pub(crate) fn mesh_for_offset(&self, work: &crate::offset::Work<'_>) -> Result<Mesh> {
        self.mesh_context(&|| false, Some(work))
    }
    fn mesh_context(
        &self,
        cancelled: &dyn Fn() -> bool,
        work: Option<&crate::offset::Work<'_>>,
    ) -> Result<Mesh> {
        check_budget(cancelled, work)?;
        if let Some(work) = work {
            self.validate_for_offset(work)?;
        } else {
            self.validate()?;
        }
        check_budget(cancelled, work)?;
        let mut mesh = Mesh {
            doc_revision: self.revision,
            ..Mesh::default()
        };
        for piece in &self.pieces {
            check_budget(cancelled, work)?;
            let start = mesh.positions.len();
            let mut vertex_for_point = BTreeMap::new();
            let mut holes = Vec::new();
            for (loop_index, boundary) in piece.boundary_loops().enumerate() {
                if loop_index > 0 {
                    holes.push(mesh.positions.len() - start);
                }
                for id in boundary {
                    check_budget(cancelled, work)?;
                    let curve = self
                        .curve(*id)
                        .ok_or_else(|| Error("Missing boundary curve".into()))?;
                    let samples =
                        sample_curve_context(self, curve, self.resolution, cancelled, work)?;
                    ensure(
                        mesh.positions.len() - start + samples.len() - 1 <= MAX_BOUNDARY_SAMPLES,
                        "Piece boundary sample limit exceeded",
                    )?;
                    vertex_for_point.insert(curve.start, mesh.positions.len());
                    for &(t, point) in samples.iter().take(samples.len() - 1) {
                        check_budget(cancelled, work)?;
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
            }
            ensure(
                mesh.positions.len() <= MAX_MESH_VERTICES,
                "Mesh vertex limit exceeded",
            )?;
            for id in piece.boundary_loops().flatten() {
                check_budget(cancelled, work)?;
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
            let mut flat = Vec::with_capacity(2 * (mesh.uv.len() - start));
            for uv in &mesh.uv[start..] {
                check_budget(cancelled, work)?;
                flat.extend(*uv);
            }
            let indices = triangulate(&flat, &holes, cancelled, work)?;
            ensure(
                indices.len() >= 3 && indices.len() % 3 == 0,
                "Triangulation produced no valid faces",
            )?;
            let triangle_start = mesh.triangles.len();
            for triangle in indices.chunks_exact(3) {
                check_budget(cancelled, work)?;
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
            include_collinear_vertices(&mut mesh, start, triangle_start, cancelled, work)?;
        }
        refine(self, &mut mesh, cancelled, work)?;
        for seam in &self.seams {
            check_budget(cancelled, work)?;
            append_stitches(&mesh.boundary, seam, &mut mesh.stitches, || {
                check_budget(cancelled, work)
            })?;
        }
        self.append_sewing_groups(&mut mesh, cancelled, work)?;
        check_budget(cancelled, work)?;
        mesh.validate_context(work)?;
        check_budget(cancelled, work)?;
        if let Some(work) = work {
            work.check()?;
        }
        Ok(mesh)
    }

    pub fn seam_lengths(&self, id: Id) -> Result<[f64; 2]> {
        let seam = self
            .seams
            .iter()
            .find(|seam| seam.id == id)
            .ok_or_else(|| Error("Unknown seam".into()))?;
        self.require_boundary_curve(seam.a.curve)?;
        self.require_boundary_curve(seam.b.curve)?;
        Ok([self.interval_length(seam.a)?, self.interval_length(seam.b)?])
    }
}

// Interior flips improve membrane conditioning without moving authored boundary samples.
fn charge(work: Option<&crate::offset::Work<'_>>) -> Result<()> {
    work.map_or(Ok(()), |work| work.step())
}

type SweepEdge = (f64, f64, f64, f64, usize);

fn sweep_edges(
    polygon: &[[f64; 2]],
    work: Option<&crate::offset::Work<'_>>,
) -> Result<Vec<SweepEdge>> {
    let mut edges = Vec::with_capacity(polygon.len());
    for i in 0..polygon.len() {
        charge(work)?;
        let (a, b) = (polygon[i], polygon[(i + 1) % polygon.len()]);
        edges.push((
            a[0].min(b[0]),
            a[0].max(b[0]),
            a[1].min(b[1]),
            a[1].max(b[1]),
            i,
        ));
    }
    earcutr::sort_by_with_control(&mut edges, |a, b| a.0.total_cmp(&b.0), || charge(work))?;
    Ok(edges)
}
fn check_budget(
    cancelled: &dyn Fn() -> bool,
    work: Option<&crate::offset::Work<'_>>,
) -> Result<()> {
    charge(work)?;
    check_cancel(cancelled)
}

fn check_cancel(cancelled: &(impl Fn() -> bool + ?Sized)) -> Result<()> {
    ensure(
        !cancelled(),
        "Meshing cancelled; previous construction retained",
    )
}

pub(crate) fn triangulate(
    flat: &[f64],
    holes: &[usize],
    cancelled: &dyn Fn() -> bool,
    work: Option<&crate::offset::Work<'_>>,
) -> Result<Vec<usize>> {
    check_cancel(cancelled)?;
    if let Some(work) = work {
        work.check()?;
    }
    let indices = earcutr::earcut_with_control(flat, holes, 2, || check_budget(cancelled, work))
        .map_err(|error| match error {
            earcutr::ControlledError::Control(error) => error,
            earcutr::ControlledError::Triangulation(error) => {
                Error(format!("Triangulation failed: {error}"))
            }
        })?;
    check_cancel(cancelled)?;
    if let Some(work) = work {
        work.check()?;
    }
    Ok(indices)
}

fn improve_triangle_quality(
    mesh: &mut Mesh,
    cancelled: &dyn Fn() -> bool,
    work: Option<&crate::offset::Work<'_>>,
) -> Result<()> {
    for _ in 0..24 {
        check_budget(cancelled, work)?;
        let mut edges: BTreeMap<(usize, usize), Vec<(usize, usize)>> = BTreeMap::new();
        for (index, triangle) in mesh.triangles.iter().enumerate() {
            check_budget(cancelled, work)?;
            for i in 0..3 {
                check_budget(cancelled, work)?;
                let (a, b, opposite) = (triangle[i], triangle[(i + 1) % 3], triangle[(i + 2) % 3]);
                edges
                    .entry((a.min(b), a.max(b)))
                    .or_default()
                    .push((index, opposite));
            }
        }
        let mut touched = BTreeSet::new();
        for ((a, b), adjacent) in edges {
            check_budget(cancelled, work)?;
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

fn sample_curve_context(
    project: &Project,
    curve: &Curve,
    spacing: f64,
    cancelled: &dyn Fn() -> bool,
    work: Option<&crate::offset::Work<'_>>,
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
        return crate::offset::sample_cubic_certified([a, c, d, b], spacing, cancelled, work);
    } else {
        let steps = (distance(a, b) / spacing).ceil().max(1.0) as usize;
        ensure(
            steps <= MAX_BOUNDARY_SAMPLES,
            "Curve tessellation sample limit exceeded",
        )?;
        for step in 1..=steps {
            check_budget(cancelled, work)?;
            let t = step as f64 / steps as f64;
            samples.push((t, [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]));
        }
    }
    Ok(samples)
}

const INTERNAL_CLEARANCE: f64 = 1.0e-6;
const INTERNAL_FLATNESS: f64 = 1.0e-7;
const MAX_CONTAINMENT_WORK: usize = 1_000_000;

pub(crate) struct ContainmentWork<'a>(usize, Option<&'a crate::offset::Work<'a>>);

impl<'a> ContainmentWork<'a> {
    pub(crate) fn new() -> Self {
        Self(MAX_CONTAINMENT_WORK, None)
    }

    pub(crate) fn for_offset(work: &'a crate::offset::Work<'a>) -> Self {
        Self(MAX_CONTAINMENT_WORK, Some(work))
    }
    pub(crate) fn request_work(&self) -> Option<&'a crate::offset::Work<'a>> {
        self.1
    }
    pub(crate) fn charge_request(&self) -> Result<()> {
        self.1.map_or(Ok(()), |work| work.step())
    }

    fn step(&mut self) -> Result<()> {
        self.charge_request()?;
        ensure(self.0 > 0, "Internal containment work budget exhausted")?;
        self.0 -= 1;
        Ok(())
    }
}

pub(crate) fn validate_internal_segments(
    project: &Project,
    piece: &Piece,
    work: &mut ContainmentWork,
) -> Result<()> {
    let Some(first) = piece.internal_curves.first() else {
        return Ok(());
    };
    let mut validate = || -> Result<()> {
        let mut scale = 1_f64;
        for id in piece.boundary.iter().chain(&piece.internal_curves) {
            work.charge_request()?;
            let curve = project.curve(*id).unwrap();
            for point in [
                project.point(curve.start).unwrap().position,
                project.point(curve.end).unwrap().position,
            ]
            .into_iter()
            .chain(curve.controls.into_iter().flatten())
            {
                scale = point.into_iter().map(f64::abs).fold(scale, f64::max);
            }
        }
        let reserve = (256.0 * f64::EPSILON * scale).next_up();
        let clearance = (INTERNAL_CLEARANCE + INTERNAL_FLATNESS + reserve).next_up();
        let mut polygon = Vec::new();
        for id in &piece.boundary {
            work.charge_request()?;
            let curve = project.curve(*id).unwrap();
            let a = project.point(curve.start).unwrap().position;
            let b = project.point(curve.end).unwrap().position;
            if polygon.is_empty() {
                polygon.push(a);
            }
            if let Some([c, d]) = curve.controls {
                flatten_internal_contour([a, c, d, b], 0, reserve, &mut polygon, work)?;
            } else {
                work.step()?;
                ensure(
                    polygon.len() < MAX_BOUNDARY_SAMPLES,
                    "Internal containment boundary sample limit exceeded",
                )?;
                polygon.push(b);
            }
        }
        polygon.pop(); // The ordered boundary repeats its first endpoint.
        let n = polygon.len();
        let tube = (2.0 * (INTERNAL_FLATNESS + reserve)).next_up();
        let edges = sweep_edges(&polygon, work.request_work())?;
        for (position, edge) in edges.iter().enumerate() {
            work.step()?;
            let i = edge.4;
            ensure(
                distance(polygon[i], polygon[(i + 1) % n]) > reserve,
                "Internal containment contour has a degenerate edge",
            )?;
            // Adjacent edges may share only their common endpoint, never backtrack.
            ensure(
                point_segment_distance(polygon[(i + 2) % n], polygon[i], polygon[(i + 1) % n])
                    > reserve,
                "Internal containment contour backtracks or touches itself",
            )?;
            for other in edges.iter().skip(position + 1) {
                work.step()?;
                if other.0 > edge.1 + tube {
                    break;
                }
                let j = other.4;
                if (i + 1) % n == j
                    || (j + 1) % n == i
                    || other.2 > edge.3 + tube
                    || other.3 < edge.2 - tube
                {
                    continue;
                }
                ensure(
                    segment_distance(
                        polygon[i],
                        polygon[(i + 1) % n],
                        polygon[j],
                        polygon[(j + 1) % n],
                    ) > tube,
                    "Internal containment fine contour self-intersects, touches or is uncertifiable",
                )?;
            }
        }
        for id in &piece.internal_curves {
            work.charge_request()?;
            let curve = project.curve(*id).unwrap();
            let a = project.point(curve.start).unwrap().position;
            let b = project.point(curve.end).unwrap().position;
            ensure(
                internal_segment_inside(a, b, &polygon, clearance, work)
                    .map_err(|error| Error(format!("Internal curve {id}: {error}")))?,
                &format!(
                    "Internal curve {id} must stay strictly inside its owning contour with 1e-6 metre clearance"
                ),
            )?;
        }
        Ok(())
    };
    validate().map_err(|error| Error(format!("Internal curve {first} containment: {error}")))
}

fn internal_segment_inside(
    a: [f64; 2],
    b: [f64; 2],
    polygon: &[[f64; 2]],
    clearance: f64,
    work: &mut ContainmentWork,
) -> Result<bool> {
    let mut winding = 0_i32;
    for i in 0..polygon.len() {
        work.step()?;
        let (c, d) = (polygon[i], polygon[(i + 1) % polygon.len()]);
        if segment_distance(a, b, c, d) <= clearance {
            return Ok(false);
        }
        work.step()?;
        if c[1] <= a[1] && d[1] > a[1] && cross(c, d, a) > 0.0 {
            winding += 1;
        } else if d[1] <= a[1] && c[1] > a[1] && cross(c, d, a) < 0.0 {
            winding -= 1;
        }
    }
    Ok(winding != 0)
}

fn straight_polygon(
    project: &Project,
    piece: &Piece,
    work: Option<&crate::offset::Work<'_>>,
) -> Result<Vec<[f64; 2]>> {
    straight_loop(project, &piece.boundary, work)
}

fn straight_loop(
    project: &Project,
    boundary: &[Id],
    work: Option<&crate::offset::Work<'_>>,
) -> Result<Vec<[f64; 2]>> {
    ensure(
        boundary.len() >= 3,
        "Straight contour needs at least three edges",
    )?;
    let mut points = BTreeSet::new();
    let mut curves = BTreeSet::new();
    let mut polygon = Vec::with_capacity(boundary.len());
    for (index, id) in boundary.iter().enumerate() {
        charge(work)?;
        let curve = project
            .curve(*id)
            .ok_or_else(|| Error("Missing straight boundary curve".into()))?;
        let next = project
            .curve(boundary[(index + 1) % boundary.len()])
            .ok_or_else(|| Error("Missing straight boundary curve".into()))?;
        ensure(
            curve.controls.is_none()
                && curve.end == next.start
                && curves.insert(*id)
                && points.insert(curve.start),
            "An ordered, exclusively owned straight contour is required",
        )?;
        let position = project
            .point(curve.start)
            .ok_or_else(|| Error("Missing straight boundary point".into()))?
            .position;
        crate::document::validate_coordinates(&position)?;
        polygon.push(position);
    }
    Ok(polygon)
}

pub(crate) fn prepare_hole(
    project: &Project,
    piece: &Piece,
    vertices: &mut [[f64; 2]],
    work: &mut ContainmentWork,
) -> Result<()> {
    if validate_holes(project, piece, Some(vertices), work)? {
        vertices[1..].reverse();
    }
    Ok(())
}

pub(crate) fn validate_holed_piece(
    project: &Project,
    piece: &Piece,
    work: &mut ContainmentWork,
) -> Result<()> {
    validate_holes(project, piece, None, work)?;
    Ok(())
}

/// Returns whether the proposed hole needs reversing. Existing document rings
/// already have a canonical winding, including when reading a saved project.
fn validate_holes(
    project: &Project,
    piece: &Piece,
    proposed: Option<&[[f64; 2]]>,
    work: &mut ContainmentWork,
) -> Result<bool> {
    let outer = straight_polygon(project, piece, work.request_work())?;
    let existing = piece
        .holes
        .iter()
        .map(|hole| straight_loop(project, &hole.boundary, work.request_work()))
        .collect::<Result<Vec<_>>>()?;
    let holes: Vec<_> = existing.iter().map(Vec::as_slice).chain(proposed).collect();
    let mut scale = 1_f64;
    for polygon in std::iter::once(outer.as_slice()).chain(holes.iter().copied()) {
        ensure(
            (3..=4096).contains(&polygon.len()),
            "A holed piece needs 3 to 4096 straight edges in each loop",
        )?;
        for point in polygon {
            work.charge_request()?;
            crate::document::validate_coordinates(point)?;
            scale = point.iter().copied().map(f64::abs).fold(scale, f64::max);
        }
    }
    let mut annotations = Vec::new();
    for id in &piece.internal_curves {
        work.charge_request()?;
        let curve = project
            .curve(*id)
            .ok_or_else(|| Error("Missing hole owner annotation".into()))?;
        ensure(
            curve.controls.is_none(),
            "Hole owner annotations must be straight",
        )?;
        let mut endpoints = [[0.; 2]; 2];
        for (point, id) in endpoints.iter_mut().zip([curve.start, curve.end]) {
            *point = project
                .point(id)
                .ok_or_else(|| Error("Missing annotation endpoint".into()))?
                .position;
            crate::document::validate_coordinates(point)?;
            scale = point.iter().copied().map(f64::abs).fold(scale, f64::max);
        }
        annotations.push((*id, endpoints));
    }
    let reserve = (256.0 * f64::EPSILON * scale).next_up();
    let outer_area = validate_straight_loop(&outer, reserve, work)?;
    let clearance = (INTERNAL_CLEARANCE + reserve).next_up();
    let annotation_clearance = (INTERNAL_CLEARANCE + INTERNAL_FLATNESS + reserve).next_up();
    for (id, [a, b]) in &annotations {
        ensure(
            straight_segment_inside(*a, *b, &outer, annotation_clearance, reserve, work)?,
            &format!(
                "Internal curve {id} must stay strictly inside its owning contour with 1e-6 metre clearance"
            ),
        )?;
    }
    let bounds: Vec<_> = holes
        .iter()
        .map(|hole| {
            let mut min = hole[0];
            let mut max = hole[0];
            for point in *hole {
                for axis in 0..2 {
                    min[axis] = min[axis].min(point[axis]);
                    max[axis] = max[axis].max(point[axis]);
                }
            }
            (min, max)
        })
        .collect();
    let mut remaining_area = outer_area.abs();
    let mut reverse_proposed = false;
    for (index, &hole) in holes.iter().enumerate() {
        let area = validate_straight_loop(hole, reserve, work)?;
        remaining_area -= area.abs();
        let reverse = outer_area.signum() == area.signum();
        if index == existing.len() {
            reverse_proposed = reverse;
        } else {
            ensure(!reverse, "Hole must wind opposite its outer boundary")?;
        }
        for i in 0..hole.len() {
            let (a, b) = (hole[i], hole[(i + 1) % hole.len()]);
            ensure(
                straight_segment_inside(a, b, &outer, clearance, reserve, work)?,
                "Hole edges must stay wholly inside the exterior with more than 1e-6 metre clearance",
            )?;
        }
        for (prior, &other) in holes[..index].iter().enumerate() {
            work.step()?;
            let (min, max) = bounds[index];
            let (other_min, other_max) = bounds[prior];
            if (0..2).any(|axis| {
                min[axis] - other_max[axis] > clearance || other_min[axis] - max[axis] > clearance
            }) {
                continue;
            }
            for i in 0..hole.len() {
                ensure(
                    !straight_segment_inside(
                        hole[i],
                        hole[(i + 1) % hole.len()],
                        other,
                        clearance,
                        reserve,
                        work,
                    )?,
                    "Cutouts must have disjoint interiors",
                )?;
            }
            // After excluding crossings, one edge detects a prior ring fully inside this one.
            ensure(
                !straight_segment_inside(other[0], other[1], hole, clearance, reserve, work)?,
                "Cutouts must have disjoint interiors",
            )?;
        }
        for (id, [a, b]) in &annotations {
            ensure(
                !straight_segment_inside(*a, *b, hole, annotation_clearance, reserve, work)?,
                &format!("Internal curve {id} cannot enter or be surrounded by a hole"),
            )?;
        }
    }
    ensure(
        remaining_area > 1.0e-10,
        "Holes must leave positive material area",
    )?;
    Ok(reverse_proposed)
}

fn validate_straight_loop(
    polygon: &[[f64; 2]],
    reserve: f64,
    work: &mut ContainmentWork,
) -> Result<f64> {
    let n = polygon.len();
    // Relative coordinates retain small areas under large authoring translations.
    let mut area2 = 0.;
    for i in 1..n - 1 {
        work.charge_request()?;
        area2 += cross(polygon[0], polygon[i], polygon[i + 1]);
    }
    ensure(
        area2.abs() > 1.0e-10,
        "Pattern loop has zero or near-zero area",
    )?;
    let edges = sweep_edges(polygon, work.request_work())?;
    for (position, edge) in edges.iter().enumerate() {
        work.step()?;
        let i = edge.4;
        ensure(
            distance(polygon[i], polygon[(i + 1) % n]) > reserve.max(1.0e-9)
                && point_segment_distance(polygon[(i + 2) % n], polygon[i], polygon[(i + 1) % n])
                    > reserve
                && point_segment_distance(polygon[i], polygon[(i + 1) % n], polygon[(i + 2) % n])
                    > reserve,
            "Pattern loop has a degenerate, backtracking or uncertain edge",
        )?;
        for other in edges.iter().skip(position + 1) {
            work.step()?;
            if other.0 > edge.1 + reserve {
                break;
            }
            let j = other.4;
            if (i + 1) % n == j
                || (j + 1) % n == i
                || other.2 > edge.3 + reserve
                || other.3 < edge.2 - reserve
            {
                continue;
            }
            work.step()?;
            ensure(
                segment_distance(
                    polygon[i],
                    polygon[(i + 1) % n],
                    polygon[j],
                    polygon[(j + 1) % n],
                ) > reserve,
                "Pattern loop self-intersects, touches or is numerically uncertain",
            )?;
        }
    }
    Ok(area2)
}

fn straight_segment_inside(
    a: [f64; 2],
    b: [f64; 2],
    polygon: &[[f64; 2]],
    clearance: f64,
    reserve: f64,
    work: &mut ContainmentWork,
) -> Result<bool> {
    let mut winding = 0_i32;
    for i in 0..polygon.len() {
        work.step()?;
        let (c, d) = (polygon[i], polygon[(i + 1) % polygon.len()]);
        ensure(
            segment_distance(a, b, c, d) > clearance,
            "Straight domain segment crosses, touches or is too close to a boundary",
        )?;
        if (c[1] <= a[1] && d[1] > a[1]) || (d[1] <= a[1] && c[1] > a[1]) {
            work.step()?;
            let sign = cross(c, d, a);
            ensure(
                sign.abs() > reserve * distance(c, d),
                "Straight domain winding is numerically uncertain",
            )?;
            if c[1] <= a[1] && sign > 0.0 {
                winding += 1;
            } else if d[1] <= a[1] && sign < 0.0 {
                winding -= 1;
            }
        }
    }
    Ok(winding != 0)
}

pub(crate) fn validate_straight_cut(
    project: &Project,
    piece: &Piece,
    a: (usize, f64),
    b: (usize, f64),
    work: &mut ContainmentWork,
) -> Result<f64> {
    ensure(
        (3..=4096).contains(&piece.boundary.len()),
        "Cut supports 3 to 4096 authored straight boundary curves",
    )?;
    for t in [a.1, b.1] {
        ensure(
            t.is_finite() && t > 1.0e-6 && t < 1.0 - 1.0e-6,
            "Cut t must be strictly inside the curve",
        )?;
    }
    let polygon = straight_polygon(project, piece, work.request_work())?;
    let mut scale = polygon
        .iter()
        .flatten()
        .copied()
        .map(f64::abs)
        .fold(1.0, f64::max);
    for id in &piece.internal_curves {
        let curve = project
            .curve(*id)
            .ok_or_else(|| Error("Missing cut annotation".into()))?;
        ensure(curve.controls.is_none(), "Cut annotations must be straight")?;
        for id in [curve.start, curve.end] {
            let position = project
                .point(id)
                .ok_or_else(|| Error("Missing cut annotation point".into()))?
                .position;
            crate::document::validate_coordinates(&position)?;
            scale = position.into_iter().map(f64::abs).fold(scale, f64::max);
        }
    }
    let reserve = (256.0 * f64::EPSILON * scale).next_up();
    let clearance = (INTERNAL_CLEARANCE + reserve).next_up();
    let n = polygon.len();
    for i in 0..n {
        work.step()?;
        ensure(
            distance(polygon[i], polygon[(i + 1) % n]) > reserve
                && point_segment_distance(polygon[(i + 2) % n], polygon[i], polygon[(i + 1) % n])
                    > reserve,
            "Cut contour has a degenerate, backtracking or uncertain adjacent edge",
        )?;
        for j in i + 1..n {
            work.step()?;
            if (i + 1) % n == j || (j + 1) % n == i {
                continue;
            }
            work.step()?;
            ensure(
                segment_distance(
                    polygon[i],
                    polygon[(i + 1) % n],
                    polygon[j],
                    polygon[(j + 1) % n],
                ) > reserve,
                "Cut contour self-intersects, touches or is numerically uncertain",
            )?;
        }
    }
    let anchor = |(index, t)| {
        project
            .curve(piece.boundary[index])
            .unwrap()
            .evaluate(project, t)
    };
    let (start, end) = (anchor(a)?, anchor(b)?);
    ensure(
        distance(start, end) > 2.0 * clearance,
        "Cut chord is too short",
    )?;
    for point in &polygon {
        work.step()?;
        ensure(
            distance(start, *point) > clearance && distance(end, *point) > clearance,
            "Cut endpoints must stay farther than 1e-6 metres from every old vertex",
        )?;
    }
    for i in 0..n {
        work.step()?;
        let (c, d) = (polygon[i], polygon[(i + 1) % n]);
        if i == a.0 || i == b.0 {
            let far = if i == a.0 { end } else { start };
            ensure(
                cross(c, d, far).abs() / distance(c, d) > clearance,
                "Cut chord is collinear or too close to an addressed edge's support line",
            )?;
        } else {
            work.step()?;
            ensure(
                segment_distance(start, end, c, d) > clearance,
                "Cut chord crosses, touches or lies too close to another boundary",
            )?;
        }
    }
    let center = std::array::from_fn(|i| start[i] + (end[i] - start[i]) * 0.5);
    let mut winding = 0_i32;
    for i in 0..n {
        work.step()?;
        let (c, d) = (polygon[i], polygon[(i + 1) % n]);
        ensure(
            point_segment_distance(center, c, d) > reserve,
            "Cut midpoint is on an uncertain boundary",
        )?;
        if (c[1] <= center[1] && d[1] > center[1]) || (d[1] <= center[1] && c[1] > center[1]) {
            work.step()?;
            let sign = cross(c, d, center);
            ensure(
                sign.abs() > reserve * distance(c, d),
                "Cut midpoint has an uncertain winding sign",
            )?;
            if c[1] <= center[1] && sign > 0.0 {
                winding += 1;
            } else if d[1] <= center[1] && sign < 0.0 {
                winding -= 1;
            }
        }
    }
    ensure(
        winding != 0,
        "Cut chord midpoint must be strictly inside the contour",
    )?;
    Ok(reserve)
}

pub(crate) fn partition_cut_annotations(
    project: &Project,
    original: &Piece,
    retained: &Piece,
    child: &Piece,
    reserve: f64,
    work: &mut ContainmentWork,
) -> Result<(Vec<Id>, Vec<Id>)> {
    let polygons = [
        straight_polygon(project, retained, work.request_work())?,
        straight_polygon(project, child, work.request_work())?,
    ];
    let clearance = (INTERNAL_CLEARANCE + INTERNAL_FLATNESS + reserve).next_up();
    let (mut kept, mut moved) = (Vec::new(), Vec::new());
    for id in &original.internal_curves {
        let curve = project.curve(*id).unwrap();
        let [a, b] = [curve.start, curve.end].map(|id| project.point(id).unwrap().position);
        let left = internal_segment_inside(a, b, &polygons[0], clearance, work)?;
        let right = internal_segment_inside(a, b, &polygons[1], clearance, work)?;
        ensure(
            left != right,
            &format!(
                "Internal curve {id} cannot be assigned unchanged to exactly one cut child with 1e-6 metre clearance"
            ),
        )?;
        if left {
            kept.push(*id);
        } else {
            moved.push(*id);
        }
    }
    Ok((kept, moved))
}

fn flatten_internal_contour(
    p: [[f64; 2]; 4],
    depth: u32,
    reserve: f64,
    polygon: &mut Vec<[f64; 2]>,
    work: &mut ContainmentWork,
) -> Result<()> {
    work.step()?;
    ensure(
        polygon.len() < MAX_BOUNDARY_SAMPLES,
        "Internal containment boundary sample limit exceeded",
    )?;
    let flatness = p
        .into_iter()
        .map(|point| point_segment_distance(point, p[0], p[3]))
        .fold(0.0, f64::max)
        .next_up();
    let chord = [p[3][0] - p[0][0], p[3][1] - p[0][1]];
    let projection = |q: [f64; 2]| (q[0] - p[0][0]) * chord[0] + (q[1] - p[0][1]) * chord[1];
    // A flat collinear cubic can still double back; require forward control projections.
    if flatness + reserve <= INTERNAL_FLATNESS
        && projection(p[1]) >= 0.0
        && projection(p[2]) >= projection(p[1])
        && projection(p[2]) <= projection(p[3])
    {
        polygon.push(p[3]);
        return Ok(());
    }
    ensure(
        depth < 20,
        "Internal containment cubic depth limit exceeded",
    )?;
    let ab = midpoint(p[0], p[1]);
    let bc = midpoint(p[1], p[2]);
    let cd = midpoint(p[2], p[3]);
    let abc = midpoint(ab, bc);
    let bcd = midpoint(bc, cd);
    let center = midpoint(abc, bcd);
    flatten_internal_contour([p[0], ab, abc, center], depth + 1, reserve, polygon, work)?;
    flatten_internal_contour([center, bcd, cd, p[3]], depth + 1, reserve, polygon, work)
}

fn segment_distance(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> f64 {
    if segments_intersect(a, b, c, d) {
        0.0
    } else {
        point_segment_distance(a, c, d)
            .min(point_segment_distance(b, c, d))
            .min(point_segment_distance(c, a, b))
            .min(point_segment_distance(d, a, b))
    }
}

pub(crate) fn validate_contour_with_work(
    project: &Project,
    piece: &Piece,
    work: &mut ContainmentWork<'_>,
) -> Result<()> {
    // shortcut: cubic intersections use bounded polyline tolerance; use exact curve predicates before precision CAD editing.
    let mut polygon = Vec::new();
    for id in &piece.boundary {
        work.charge_request()?;
        let curve = project
            .curve(*id)
            .ok_or_else(|| Error("Missing boundary curve".into()))?;
        let samples = sample_curve_context(
            project,
            curve,
            project.resolution.min(0.02),
            &|| false,
            work.request_work(),
        )?;
        for pair in samples.windows(2) {
            work.charge_request()?;
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
    let mut area2 = 0.;
    for i in 0..n {
        work.charge_request()?;
        area2 += polygon[i][0] * polygon[(i + 1) % n][1] - polygon[(i + 1) % n][0] * polygon[i][1];
    }
    ensure(area2.abs() > 1.0e-10, "Pattern has zero or near-zero area")?;
    // Bounding-box sweep avoids testing every pair on ordinary refined contours.
    let edges = sweep_edges(&polygon, work.request_work())?;
    for (position, edge) in edges.iter().enumerate() {
        work.charge_request()?;
        for other in edges.iter().skip(position + 1) {
            work.charge_request()?;
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

fn include_collinear_vertices(
    mesh: &mut Mesh,
    start: usize,
    triangle_start: usize,
    cancelled: &dyn Fn() -> bool,
    work: Option<&crate::offset::Work<'_>>,
) -> Result<()> {
    let mut used = BTreeSet::new();
    let mut edges = BTreeSet::new();
    for triangle in &mesh.triangles[triangle_start..] {
        check_budget(cancelled, work)?;
        used.extend(triangle);
        edges.extend((0..3).map(|i| edge_key(triangle[i], triangle[(i + 1) % 3])));
    }
    let mut samples = BTreeMap::<Id, Vec<BoundarySample>>::new();
    for &sample in &mesh.boundary {
        check_budget(cancelled, work)?;
        if sample.vertex >= start {
            samples.entry(sample.curve).or_default().push(sample);
        }
    }
    let mut neighbors = BTreeMap::<usize, BTreeSet<usize>>::new();
    for samples in samples.values_mut() {
        earcutr::sort_by_with_control(
            samples,
            |a, b| a.t.total_cmp(&b.t),
            || check_budget(cancelled, work),
        )?;
        for pair in samples.windows(2) {
            check_budget(cancelled, work)?;
            neighbors
                .entry(pair[0].vertex)
                .or_default()
                .insert(pair[1].vertex);
            neighbors
                .entry(pair[1].vertex)
                .or_default()
                .insert(pair[0].vertex);
        }
    }
    for vertex in start..mesh.positions.len() {
        check_budget(cancelled, work)?;
        if used.contains(&vertex)
            && neighbors[&vertex]
                .iter()
                .all(|&other| edges.contains(&edge_key(vertex, other)))
        {
            continue;
        }
        let point = mesh.uv[vertex];
        let mut found = None;
        'triangles: for (index, triangle) in mesh.triangles.iter().enumerate().skip(triangle_start)
        {
            check_budget(cancelled, work)?;
            for edge in 0..3 {
                check_budget(cancelled, work)?;
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
        if let Some((_, a, b, _)) = found {
            // A hole bridge can use a collinear vertex while omitting it from its contour edge.
            for index in triangle_start..mesh.triangles.len() {
                check_budget(cancelled, work)?;
                let triangle = mesh.triangles[index];
                if let Some(edge) = (0..3).find(|&edge| {
                    edge_key(triangle[edge], triangle[(edge + 1) % 3]) == edge_key(a, b)
                }) {
                    let (a, b, c) = (
                        triangle[edge],
                        triangle[(edge + 1) % 3],
                        triangle[(edge + 2) % 3],
                    );
                    mesh.triangles[index] = [a, vertex, c];
                    mesh.triangles.push([vertex, b, c]);
                    edges.remove(&edge_key(a, b));
                    edges.extend([
                        edge_key(a, vertex),
                        edge_key(vertex, b),
                        edge_key(vertex, c),
                    ]);
                }
            }
            used.insert(vertex);
        }
        ensure(
            used.contains(&vertex),
            "Triangulation lost a boundary vertex",
        )?;
    }
    for (&vertex, others) in &neighbors {
        for &other in others {
            check_budget(cancelled, work)?;
            ensure(
                edges.contains(&edge_key(vertex, other)),
                "Triangulation lost an authored boundary edge",
            )?;
        }
    }
    Ok(())
}

fn refine(
    project: &Project,
    mesh: &mut Mesh,
    cancelled: &dyn Fn() -> bool,
    work: Option<&crate::offset::Work<'_>>,
) -> Result<()> {
    for _ in 0..20 {
        check_budget(cancelled, work)?;
        // Remove slivers before subdivision multiplies their long edges.
        improve_triangle_quality(mesh, cancelled, work)?;
        let mut split = BTreeMap::new();
        for &[a, b, c] in &mesh.triangles {
            check_budget(cancelled, work)?;
            for [a, b] in [[a, b], [b, c], [c, a]] {
                check_budget(cancelled, work)?;
                if distance(mesh.uv[a], mesh.uv[b]) > project.resolution * 1.4 {
                    enqueue_refinement_edge(&mut split, edge_key(a, b), mesh.piece_ids[a], work)?;
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
            check_budget(cancelled, work)?;
            samples.entry(sample.curve).or_default().push(sample);
        }
        for list in samples.values_mut() {
            earcutr::sort_by_with_control(
                list,
                |a, b| a.t.total_cmp(&b.t),
                || check_budget(cancelled, work),
            )?;
            for pair in list.windows(2) {
                check_budget(cancelled, work)?;
                boundary_edges.insert(edge_key(pair[0].vertex, pair[1].vertex), (pair[0], pair[1]));
            }
        }
        for (&(a, b), vertex) in &mut split {
            check_budget(cancelled, work)?;
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
            check_budget(cancelled, work)?;
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

fn enqueue_refinement_edge(
    split: &mut BTreeMap<(usize, usize), usize>,
    edge: (usize, usize),
    piece: Id,
    work: Option<&crate::offset::Work<'_>>,
) -> Result<()> {
    charge(work)?;
    if work.is_some() && split.len() >= MAX_PENDING_GEOMETRY_TASKS && !split.contains_key(&edge) {
        return Err(Error(format!(
            "OffsetPendingTaskBudgetExceeded: piece {piece} refinement edge {edge:?} exceeds 65536 pending geometry tasks"
        )));
    }
    split.entry(edge).or_insert(0);
    Ok(())
}

fn append_stitches(
    boundary: &[BoundarySample],
    seam: &Seam,
    stitches: &mut Vec<Stitch>,
    mut check: impl FnMut() -> Result<()>,
) -> Result<()> {
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for &sample in boundary {
        check()?;
        if sample.curve == seam.a.curve {
            a.push(sample);
        }
        if sample.curve == seam.b.curve {
            b.push(sample);
        }
    }
    earcutr::sort_by_with_control(&mut a, |a, b| a.t.total_cmp(&b.t), &mut check)?;
    earcutr::sort_by_with_control(&mut b, |a, b| a.t.total_cmp(&b.t), &mut check)?;
    ensure(a.len() >= 2 && b.len() >= 2, "Seam has no mesh boundary")?;
    let mut fractions = Vec::new();
    for pair in &seam.correspondence {
        check()?;
        fractions.push(pair[0]);
    }
    for sample in &a {
        check()?;
        let fraction = (sample.t - seam.a.t0) / (seam.a.t1 - seam.a.t0);
        if (0.0..=1.0).contains(&fraction) {
            fractions.push(fraction);
        }
    }
    for sample in &b {
        check()?;
        let fraction = (sample.t - seam.b.t0) / (seam.b.t1 - seam.b.t0);
        if (0.0..=1.0).contains(&fraction) {
            fractions.push(correspondence_with_control(
                &seam.correspondence,
                fraction,
                1,
                &mut check,
            )?);
        }
    }
    earcutr::sort_by_with_control(&mut fractions, f64::total_cmp, &mut check)?;
    let mut retained = 0;
    for i in 0..fractions.len() {
        check()?;
        let duplicate = retained > 0 && (fractions[i] - fractions[retained - 1]).abs() < 1.0e-10;
        if !duplicate {
            fractions[retained] = fractions[i];
            retained += 1;
        }
    }
    fractions.truncate(retained);
    for fraction in fractions {
        check()?;
        let paired = correspondence_with_control(&seam.correspondence, fraction, 0, &mut check)?;
        stitches.push(Stitch {
            seam: seam.id,
            a: weighted_sample(&a, seam.a, fraction, &mut check)?,
            b: weighted_sample(&b, seam.b, paired, &mut check)?,
        });
    }
    Ok(())
}

pub(crate) fn correspondence_with_control(
    pairs: &[[f64; 2]],
    fraction: f64,
    source: usize,
    mut check: impl FnMut() -> Result<()>,
) -> Result<f64> {
    for pair in pairs.windows(2) {
        check()?;
        if fraction <= pair[1][source] {
            let t = (fraction - pair[0][source]) / (pair[1][source] - pair[0][source]);
            return Ok(pair[0][1 - source] + (pair[1][1 - source] - pair[0][1 - source]) * t);
        }
    }
    Ok(1.0)
}

fn weighted_sample(
    samples: &[BoundarySample],
    interval: CurveInterval,
    fraction: f64,
    check: &mut dyn FnMut() -> Result<()>,
) -> Result<WeightedVertex> {
    let t = interval.parameter(fraction).clamp(0.0, 1.0);
    weighted_at_t(samples, t, 1.0e-10, check)
}

#[cfg(test)]
pub(crate) fn weighted_sample_at_t(samples: &[BoundarySample], t: f64) -> Result<WeightedVertex> {
    weighted_sample_at_t_with_control(samples, t, || Ok(()))
}

pub(crate) fn weighted_sample_at_t_with_control(
    samples: &[BoundarySample],
    t: f64,
    mut check: impl FnMut() -> Result<()>,
) -> Result<WeightedVertex> {
    weighted_at_t(samples, t, 0., &mut check)
}

fn weighted_at_t(
    samples: &[BoundarySample],
    t: f64,
    snap_tolerance: f64,
    check: &mut dyn FnMut() -> Result<()>,
) -> Result<WeightedVertex> {
    for sample in samples {
        check()?;
        if sample.t == t || (sample.t - t).abs() < snap_tolerance {
            return Ok(WeightedVertex {
                vertices: [sample.vertex; 2],
                weight: 0.0,
            });
        }
    }
    for pair in samples.windows(2) {
        check()?;
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
    fn refinement_refuses_new_pending_tasks_before_enqueue_without_changing_ordinary_mesh_policy() {
        let mut split: BTreeMap<_, _> = (0..MAX_PENDING_GEOMETRY_TASKS)
            .map(|i| ((i, i + 1), 0))
            .collect();
        let work = crate::offset::Work::new(&|| false);
        enqueue_refinement_edge(&mut split, (7, 8), 42, Some(&work)).unwrap();
        let edge = (MAX_PENDING_GEOMETRY_TASKS, MAX_PENDING_GEOMETRY_TASKS + 1);
        let error = enqueue_refinement_edge(&mut split, edge, 42, Some(&work)).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("OffsetPendingTaskBudgetExceeded: piece 42")
        );
        assert_eq!(split.len(), MAX_PENDING_GEOMETRY_TASKS);
        assert!(!split.contains_key(&edge));
        enqueue_refinement_edge(&mut split, edge, 42, None).unwrap();
        assert_eq!(split.len(), MAX_PENDING_GEOMETRY_TASKS + 1);
        let mut empty = BTreeMap::new();
        let cancelled = crate::offset::Work::new(&|| true);
        assert!(
            enqueue_refinement_edge(&mut empty, (0, 1), 42, Some(&cancelled))
                .unwrap_err()
                .to_string()
                .starts_with("OffsetCancelled")
        );
        assert!(empty.is_empty());
    }

    #[test]
    fn seam_candidate_scans_cancel_and_preserve_independent_correspondence() {
        let samples: Vec<_> = (0..1024)
            .map(|i| BoundarySample {
                piece: 1,
                curve: 10,
                t: i as f64 / 1023.,
                vertex: i,
            })
            .collect();
        let mut calls = 0;
        let error = weighted_sample_at_t_with_control(&samples, 0.9999, || {
            calls += 1;
            if calls == 513 {
                Err(Error("Stopped weighted candidate".into()))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(calls, 513);
        assert_eq!(error.0, "Stopped weighted candidate");
        let pairs: Vec<_> = (0..1024).map(|i| [i as f64 / 1023.; 2]).collect();
        let mut calls = 0;
        let error = correspondence_with_control(&pairs, 0.9999, 0, || {
            calls += 1;
            if calls == 513 {
                Err(Error("Stopped correspondence candidate".into()))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(calls, 513);
        assert_eq!(error.0, "Stopped correspondence candidate");

        let mut boundary = Vec::new();
        let mut positions = Vec::new();
        for (curve, times) in [(10, vec![0., 0.25, 0.5, 0.75, 1.]), (20, vec![0., 0.5, 1.])] {
            for t in times {
                boundary.push(BoundarySample {
                    piece: 1,
                    curve,
                    t,
                    vertex: positions.len(),
                });
                positions.push([t, 0., 0.]);
            }
        }
        let seam = Seam {
            id: 30,
            name: "Original controlled nonlinear seam".into(),
            a: CurveInterval::full(10),
            b: CurveInterval::full(20),
            correspondence: vec![[0., 0.], [0.5, 0.25], [1., 1.]],
        };
        let mut stitches = Vec::new();
        append_stitches(&boundary, &seam, &mut stitches, || Ok(())).unwrap();
        let expected = [
            (0., 0.),
            (0.25, 0.125),
            (0.5, 0.25),
            (2. / 3., 0.5),
            (0.75, 0.625),
            (1., 1.),
        ];
        assert_eq!(stitches.len(), expected.len());
        for (stitch, (a, b)) in stitches.iter().zip(expected) {
            assert!((stitch.a.position(&positions)[0] - a).abs() < 1e-12);
            assert!((stitch.b.position(&positions)[0] - b).abs() < 1e-12);
        }
    }

    #[test]
    fn controlled_triangulation_preserves_upstream_identity_area_and_hole_topology() {
        let mut curved = vec![0., 0., 0.4, 0.];
        for i in 1..=40 {
            let t = i as f64 / 40.;
            curved.extend([0.4 + 0.3 * t * (1. - t), 0.6 * t]);
        }
        curved.extend([0., 0.6]);
        // Index order comes from unmodified ISC earcutr 0.5.0, solely for regression identity.
        // Areas and Euler/boundary checks are independent geometric expectations.
        let fixtures = [
            (
                vec![0., 0., 4., 0., 4., 1., 1., 1., 1., 4., 0., 4.],
                vec![],
                vec![4, 5, 0, 0, 1, 2, 3, 4, 0, 0, 2, 3],
                7.,
                1e-12,
            ),
            (
                vec![
                    0., 0., 4., 0., 4., 4., 0., 4., 0.5, 0.5, 0.5, 1.5, 1.5, 1.5, 1.5, 0.5, 0.5,
                    2.5, 0.5, 3.5, 1.5, 3.5, 1.5, 2.5,
                ],
                vec![4, 8],
                vec![
                    0, 8, 9, 0, 4, 5, 7, 4, 0, 3, 0, 9, 8, 0, 5, 7, 0, 1, 2, 3, 9, 11, 8, 5, 6, 7,
                    1, 2, 9, 10, 11, 5, 6, 6, 1, 2, 2, 10, 11, 11, 6, 2,
                ],
                14.,
                1e-12,
            ),
            (
                curved,
                vec![],
                vec![
                    41, 42, 0, 0, 1, 2, 2, 3, 4, 4, 5, 6, 6, 7, 8, 8, 9, 10, 10, 11, 12, 12, 13,
                    14, 14, 15, 16, 16, 17, 18, 18, 19, 20, 20, 21, 22, 22, 23, 24, 24, 25, 26, 26,
                    27, 28, 28, 29, 30, 30, 31, 32, 32, 33, 34, 34, 35, 36, 36, 37, 38, 38, 39, 40,
                    40, 41, 0, 0, 2, 4, 4, 6, 8, 8, 10, 12, 12, 14, 16, 16, 18, 20, 20, 22, 24, 24,
                    26, 28, 28, 30, 32, 32, 34, 36, 36, 38, 40, 40, 0, 4, 4, 8, 12, 12, 16, 20, 20,
                    24, 28, 28, 32, 36, 36, 40, 4, 4, 12, 20, 20, 28, 36, 36, 4, 20,
                ],
                0.27,
                2e-5,
            ),
        ];
        for (flat, holes, expected, area, tolerance) in fixtures {
            let triangles = triangulate(&flat, &holes, &|| false, None).unwrap();
            assert_eq!(triangles, expected);
            let mut edges = BTreeMap::<_, (usize, i32)>::new();
            let mut actual_area = 0.;
            for triangle in triangles.chunks_exact(3) {
                let [a, b, c] =
                    std::array::from_fn(|i| [flat[2 * triangle[i]], flat[2 * triangle[i] + 1]]);
                let area2 = cross(a, b, c);
                assert!(area2 > 0.);
                actual_area += area2 * 0.5;
                for i in 0..3 {
                    let (a, b) = (triangle[i], triangle[(i + 1) % 3]);
                    let (count, direction) = edges.entry(edge_key(a, b)).or_default();
                    *count += 1;
                    *direction += if a < b { 1 } else { -1 };
                }
            }
            assert!((actual_area - area).abs() < tolerance);
            assert!(edges.values().all(|&(n, d)| n == 1 || (n == 2 && d == 0)));
            let vertices = flat.len() / 2;
            assert_eq!(
                vertices + triangles.len() / 3 + holes.len(),
                edges.len() + 1
            );
            let mut rings = vec![0];
            rings.extend(&holes);
            rings.push(vertices);
            let expected_boundary: BTreeSet<_> = rings
                .windows(2)
                .flat_map(|ring| {
                    (ring[0]..ring[1])
                        .map(|i| edge_key(i, if i + 1 == ring[1] { ring[0] } else { i + 1 }))
                })
                .collect();
            assert_eq!(
                edges
                    .into_iter()
                    .filter_map(|(edge, (n, _))| (n == 1).then_some(edge))
                    .collect::<BTreeSet<_>>(),
                expected_boundary
            );
        }
    }

    #[test]
    fn mesh_triangulation_adapter_cancels_internal_work_without_an_offset_counter() {
        let flat: Vec<_> = (0..192)
            .flat_map(|i| {
                let t = std::f64::consts::TAU * i as f64 / 192.;
                [t.cos(), t.sin()]
            })
            .collect();
        let polls = std::cell::Cell::new(0);
        let error = triangulate(
            &flat,
            &[],
            &|| {
                polls.set(polls.get() + 1);
                polls.get() == 513
            },
            None,
        )
        .unwrap_err();
        assert_eq!(polls.get(), 513);
        assert!(error.to_string().contains("Meshing cancelled"));
    }

    #[test]
    fn fine_curved_panels_preserve_area_topology_and_sewing_within_mesh_limits() {
        let project = Project::from_json(include_bytes!(
            "../tests/fixtures/asymmetric-fine.weftforge"
        ))
        .unwrap();
        let original = project.clone();
        assert_eq!(project.resolution, 0.017);
        let mesh = project.mesh().unwrap();
        mesh.validate().unwrap();
        let mut edges = BTreeMap::<_, (usize, i32)>::new();
        let mut areas = BTreeMap::<_, f64>::new();
        let mut faces = BTreeMap::<_, usize>::new();
        for &triangle in &mesh.triangles {
            let [a, b, c] = triangle.map(|vertex| mesh.uv[vertex]);
            let area = cross(a, b, c) * 0.5;
            assert!(area > 0.);
            *areas.entry(mesh.piece_ids[triangle[0]]).or_default() += area;
            *faces.entry(mesh.piece_ids[triangle[0]]).or_default() += 1;
            for i in 0..3 {
                let (a, b) = (triangle[i], triangle[(i + 1) % 3]);
                assert!(distance(mesh.uv[a], mesh.uv[b]) <= project.resolution * 1.4 + 1e-12);
                let (count, orientation) = edges.entry(edge_key(a, b)).or_default();
                *count += 1;
                *orientation += if a < b { 1 } else { -1 };
            }
        }
        assert!(
            edges
                .values()
                .all(|&(count, orientation)| { count == 1 || (count == 2 && orientation == 0) })
        );
        let mut expected_boundary = BTreeSet::new();
        let mut boundary_areas = BTreeMap::<_, f64>::new();
        for piece in &project.pieces {
            for id in &piece.boundary {
                let curve = project.curve(*id).unwrap();
                let mut samples: Vec<_> = mesh
                    .boundary
                    .iter()
                    .filter(|sample| sample.curve == *id)
                    .collect();
                samples.sort_by(|a, b| a.t.total_cmp(&b.t));
                assert_eq!(samples.first().unwrap().t, 0.);
                assert_eq!(samples.last().unwrap().t, 1.);
                for sample in &samples {
                    assert_eq!(sample.piece, piece.id);
                    assert_eq!(mesh.piece_ids[sample.vertex], piece.id);
                    assert!(
                        distance(
                            mesh.uv[sample.vertex],
                            curve.evaluate(&project, sample.t).unwrap()
                        ) < 1e-12
                    );
                }
                for pair in samples.windows(2) {
                    assert!(pair[0].t < pair[1].t);
                    assert!(expected_boundary.insert(edge_key(pair[0].vertex, pair[1].vertex)));
                    let [a, b] = [pair[0], pair[1]].map(|sample| mesh.uv[sample.vertex]);
                    *boundary_areas.entry(piece.id).or_default() +=
                        (a[0] * b[1] - a[1] * b[0]) * 0.5;
                }
            }
            let vertices = mesh.piece_ids.iter().filter(|&&id| id == piece.id).count();
            let piece_edges = edges
                .keys()
                .filter(|&&(a, _)| mesh.piece_ids[a] == piece.id)
                .count();
            assert_eq!(vertices + faces[&piece.id], piece_edges + 1);
            assert!((areas[&piece.id] - boundary_areas[&piece.id]).abs() < 1e-9);
        }
        assert_eq!(
            edges
                .iter()
                .filter_map(|(&edge, &(count, _))| (count == 1).then_some(edge))
                .collect::<BTreeSet<_>>(),
            expected_boundary
        );
        // Exact cubic line integrals bound the boundary tessellation's area error.
        for (piece, analytic_area) in [
            (2, 1.488565),
            (11, 0.288),
            (20, 1.11123),
            (29, 0.372),
            (38, 0.48),
        ] {
            assert!((areas[&piece] - analytic_area).abs() < 1e-5);
        }
        let group = &project.sewing_groups[0];
        assert_eq!(
            mesh.sewing_groups,
            vec![project.sewing_group_geometry(group.id).unwrap()]
        );
        assert!(!mesh.stitches.is_empty());
        for stitch in &mesh.stitches {
            assert_eq!(stitch.seam, group.id);
            for (endpoint, side) in [(stitch.a, &group.a), (stitch.b, &group.b)] {
                assert!(side.iter().any(|interval| {
                    endpoint.vertices.iter().all(|&vertex| {
                        mesh.boundary
                            .iter()
                            .any(|sample| sample.curve == interval.curve && sample.vertex == vertex)
                    })
                }));
            }
        }
        let calls = std::cell::Cell::new(0);
        assert!(
            project
                .mesh_cancellable(|| {
                    calls.set(calls.get() + 1);
                    calls.get() > 100_000
                })
                .unwrap_err()
                .0
                .contains("cancelled")
        );
        assert_eq!(project, original);
    }

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
        improve_triangle_quality(&mut mesh, &|| false, None).unwrap();
        assert_eq!(mesh.uv, boundary);
        assert!((area(&mesh) - before).abs() < 1e-12);
        assert!(
            mesh.triangles
                .iter()
                .all(|t| t.contains(&1) && t.contains(&3))
        );
        let stable = mesh.triangles.clone();
        improve_triangle_quality(&mut mesh, &|| false, None).unwrap();
        assert_eq!(mesh.triangles, stable);
    }
}
