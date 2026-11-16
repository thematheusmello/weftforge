use crate::{Error, Id, Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_OBJECTS: usize = 20_000;
pub const MAX_SEWING_SPANS: usize = 20_000;
pub const MIN_RESOLUTION: f64 = 0.005;
pub const MAX_RESOLUTION: f64 = 0.5;
pub const MAX_COORDINATE: f64 = 1_000.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub id: Id,
    pub position: [f64; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Curve {
    pub id: Id,
    pub start: Id,
    pub end: Id,
    /// Absolute cubic Bezier control points; `None` is a straight segment.
    pub controls: Option<[[f64; 2]; 2]>,
}

impl Curve {
    pub fn bounds(&self, project: &Project) -> Result<[[f64; 2]; 2]> {
        let start = self.evaluate(project, 0.0)?;
        let end = self.evaluate(project, 1.0)?;
        let mut min = std::array::from_fn(|axis| start[axis].min(end[axis]));
        let mut max = std::array::from_fn(|axis| start[axis].max(end[axis]));
        if let Some([c, d]) = self.controls {
            for axis in 0..2 {
                let a = 3.0 * (-start[axis] + 3.0 * c[axis] - 3.0 * d[axis] + end[axis]);
                let b = 6.0 * (start[axis] - 2.0 * c[axis] + d[axis]);
                let c = 3.0 * (c[axis] - start[axis]);
                let mut roots = [f64::NAN; 2];
                if a.abs() < 1.0e-14 {
                    if b.abs() > 1.0e-14 {
                        roots[0] = -c / b;
                    }
                } else {
                    let discriminant = b * b - 4.0 * a * c;
                    if discriminant >= 0.0 {
                        let q = -0.5 * (b + discriminant.sqrt().copysign(b));
                        if q.abs() > 1.0e-30 {
                            roots = [q / a, c / q];
                        } else {
                            roots[0] = -b / (2.0 * a);
                        }
                    }
                }
                for t in roots {
                    if t > 0.0 && t < 1.0 {
                        let point = self.evaluate(project, t)?;
                        min[axis] = min[axis].min(point[axis]);
                        max[axis] = max[axis].max(point[axis]);
                    }
                }
            }
        }
        Ok([min, max])
    }

    pub fn evaluate(&self, project: &Project, t: f64) -> Result<[f64; 2]> {
        ensure(
            t.is_finite() && (0.0..=1.0).contains(&t),
            "Curve t must be in [0, 1]",
        )?;
        let a = project
            .point(self.start)
            .ok_or_else(|| Error("Missing curve start".into()))?
            .position;
        let b = project
            .point(self.end)
            .ok_or_else(|| Error("Missing curve end".into()))?
            .position;
        Ok(match self.controls {
            None => [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t],
            Some([c, d]) => {
                let s = 1.0 - t;
                std::array::from_fn(|axis| {
                    s.powi(3) * a[axis]
                        + 3.0 * s * s * t * c[axis]
                        + 3.0 * s * t * t * d[axis]
                        + t.powi(3) * b[axis]
                })
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub translation: [f64; 3],
    /// XYZ Euler rotations in radians, applied in that order.
    pub rotation: [f64; 3],
}

impl Placement {
    pub fn transform(&self, point: [f64; 2]) -> [f64; 3] {
        let [sx, sy, sz] = self.rotation.map(f64::sin);
        let [cx, cy, cz] = self.rotation.map(f64::cos);
        let [x, y, z] = [point[0], point[1] * cx, point[1] * sx];
        let [x, y, z] = [x * cy + z * sy, y, -x * sy + z * cy];
        let [x, y, z] = [x * cz - y * sz, x * sz + y * cz, z];
        [
            x + self.translation[0],
            y + self.translation[1],
            z + self.translation[2],
        ]
    }

    fn compensate_pattern_transform(
        self,
        translation: [f64; 2],
        angle: f64,
        pivot: [f64; 2],
    ) -> Self {
        let basis = Self {
            translation: [0.; 3],
            ..self
        };
        if angle == 0.0 {
            let delta = basis.transform(translation);
            return Self {
                translation: std::array::from_fn(|axis| {
                    if delta[axis] == 0.0 {
                        self.translation[axis]
                    } else {
                        self.translation[axis] - delta[axis]
                    }
                }),
                ..self
            };
        }
        let old_u = basis.transform([1., 0.]);
        let old_v = basis.transform([0., 1.]);
        let (sin, cos) = angle.sin_cos();
        let u: [f64; 3] = std::array::from_fn(|i| cos * old_u[i] - sin * old_v[i]);
        let v: [f64; 3] = std::array::from_fn(|i| sin * old_u[i] + cos * old_v[i]);
        let w: [f64; 3] = std::array::from_fn(|i| {
            u[(i + 1) % 3] * v[(i + 2) % 3] - u[(i + 2) % 3] * v[(i + 1) % 3]
        });
        // Reconstruct the perpendicular column so near-singular Euler phases share rounding.
        let v: [f64; 3] = std::array::from_fn(|i| {
            w[(i + 1) % 3] * u[(i + 2) % 3] - w[(i + 2) % 3] * u[(i + 1) % 3]
        });
        let cos_y = u[0].hypot(u[1]);
        // The fixed singular threshold is part of the reviewed transform contract.
        let (x, z) = if cos_y > 32.0 * f64::EPSILON {
            (v[2].atan2(w[2]), u[1].atan2(u[0]))
        } else {
            (0.0, (-v[0]).atan2(v[1]))
        };
        let mut result = Self {
            rotation: [x, (-u[2]).atan2(cos_y), z],
            translation: [0.; 3],
        };
        let old_pivot = self.transform(pivot);
        let new_pivot = result.transform(std::array::from_fn(|i| pivot[i] + translation[i]));
        result.translation = std::array::from_fn(|i| old_pivot[i] - new_pivot[i]);
        result
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hole {
    pub id: Id,
    /// Closed, ordered straight chain, wound opposite its owner's exterior.
    pub boundary: Vec<Id>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Piece {
    pub id: Id,
    pub name: String,
    /// Closed, ordered chain. Curves are traversed start to end.
    pub boundary: Vec<Id>,
    pub holes: Vec<Hole>,
    /// Independent straight annotations, each with two exclusive endpoint IDs.
    pub internal_curves: Vec<Id>,
    pub placement: Placement,
    pub material: Id,
    /// Warp axis in material UV: radians counterclockwise from +U toward +V.
    pub grain_angle: f64,
}

impl Piece {
    pub fn boundary_loops(&self) -> impl Iterator<Item = &[Id]> {
        std::iter::once(self.boundary.as_slice())
            .chain(self.holes.iter().map(|hole| hole.boundary.as_slice()))
    }

    pub fn construction_curves(&self) -> impl Iterator<Item = &Id> {
        self.boundary_loops().flatten().chain(&self.internal_curves)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub id: Id,
    pub name: String,
    pub areal_density: f64,
    pub warp_stiffness: f64,
    pub weft_stiffness: f64,
    pub shear_stiffness: f64,
    pub bending_stiffness: f64,
    pub damping: f64,
    pub friction: f64,
    pub collision_thickness: f64,
    pub color: [f32; 3],
}

impl Default for Material {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Cotton approximation (uncalibrated)".into(),
            areal_density: 0.18,
            warp_stiffness: 400.0,
            weft_stiffness: 400.0,
            shear_stiffness: 100.0,
            bending_stiffness: 0.003,
            damping: 0.2,
            friction: 0.4,
            collision_thickness: 0.003,
            color: [0.22, 0.55, 0.76],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurveInterval {
    pub curve: Id,
    pub t0: f64,
    pub t1: f64,
}

impl CurveInterval {
    pub fn full(curve: Id) -> Self {
        Self {
            curve,
            t0: 0.0,
            t1: 1.0,
        }
    }

    pub fn reversed(curve: Id) -> Self {
        Self {
            curve,
            t0: 1.0,
            t1: 0.0,
        }
    }

    pub fn parameter(&self, fraction: f64) -> f64 {
        self.t0 + (self.t1 - self.t0) * fraction
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SewingDirection {
    Forward,
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SewingSideByLength {
    pub curve: Id,
    pub start_t: f64,
    pub direction: SewingDirection,
    pub length_m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seam {
    pub id: Id,
    pub name: String,
    pub a: CurveInterval,
    pub b: CurveInterval,
    /// Strictly increasing normalized interval-parameter pairs, including
    /// [0,0] and [1,1]. Unequal lengths represent deliberate easing.
    pub correspondence: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SewingGroup {
    pub id: Id,
    pub name: String,
    /// Ordered, oriented coverage; disconnected intervals contribute no gap length.
    pub a: Vec<CurveInterval>,
    pub b: Vec<CurveInterval>,
    /// Strictly increasing normalized cumulative arc-length pairs.
    pub correspondence: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capsule {
    pub a: [f64; 3],
    pub b: [f64; 3],
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mannequin {
    pub capsules: Vec<Capsule>,
}

impl Default for Mannequin {
    fn default() -> Self {
        // Original analytic mannequin; dimensions are design approximations.
        Self {
            capsules: vec![
                Capsule {
                    a: [0.0, 0.90, 0.0],
                    b: [0.0, 1.40, 0.0],
                    radius: 0.18,
                },
                Capsule {
                    a: [0.0, 1.53, 0.0],
                    b: [0.0, 1.65, 0.0],
                    radius: 0.105,
                },
                Capsule {
                    a: [-0.22, 1.40, 0.0],
                    b: [-0.55, 1.22, 0.0],
                    radius: 0.065,
                },
                Capsule {
                    a: [0.22, 1.40, 0.0],
                    b: [0.55, 1.22, 0.0],
                    radius: 0.065,
                },
                Capsule {
                    a: [-0.09, 0.88, 0.0],
                    b: [-0.09, 0.18, 0.0],
                    radius: 0.085,
                },
                Capsule {
                    a: [0.09, 0.88, 0.0],
                    b: [0.09, 0.18, 0.0],
                    radius: 0.085,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub revision: u64,
    pub points: Vec<Point>,
    pub curves: Vec<Curve>,
    pub pieces: Vec<Piece>,
    pub materials: Vec<Material>,
    pub seams: Vec<Seam>,
    pub sewing_groups: Vec<SewingGroup>,
    pub mannequin: Mannequin,
    pub resolution: f64,
    pub(crate) next_id: Id,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            revision: 0,
            points: Vec::new(),
            curves: Vec::new(),
            pieces: Vec::new(),
            materials: vec![Material::default()],
            seams: Vec::new(),
            sewing_groups: Vec::new(),
            mannequin: Mannequin::default(),
            resolution: 0.07,
            next_id: 2,
        }
    }
}

impl Project {
    pub fn point(&self, id: Id) -> Option<&Point> {
        self.points.iter().find(|item| item.id == id)
    }
    pub fn curve(&self, id: Id) -> Option<&Curve> {
        self.curves.iter().find(|item| item.id == id)
    }
    pub fn piece(&self, id: Id) -> Option<&Piece> {
        self.pieces.iter().find(|item| item.id == id)
    }
    pub fn material(&self, id: Id) -> Option<&Material> {
        self.materials.iter().find(|item| item.id == id)
    }

    pub(crate) fn require_boundary_curve(&self, id: Id) -> Result<()> {
        ensure(self.curve(id).is_some(), "Authored curve is missing")?;
        ensure(
            self.pieces.iter().any(|piece| piece.boundary.contains(&id)),
            &format!(
                "Curve {id} is not an outer boundary; internal annotations cannot be used here"
            ),
        )
    }

    pub fn piece_bounds(&self, id: Id) -> Result<[[f64; 2]; 2]> {
        let piece = self
            .piece(id)
            .ok_or_else(|| Error("Unknown piece".into()))?;
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        for curve in &piece.boundary {
            let curve = self
                .curve(*curve)
                .ok_or_else(|| Error("Missing boundary curve".into()))?;
            let [curve_min, curve_max] = curve.bounds(self)?;
            for axis in 0..2 {
                min[axis] = min[axis].min(curve_min[axis]);
                max[axis] = max[axis].max(curve_max[axis]);
            }
        }
        Ok([min, max])
    }

    pub fn validate(&self) -> Result<()> {
        self.validate_with_work(&mut crate::geometry::ContainmentWork::new())
    }

    pub(crate) fn validate_for_offset(&self, work: &crate::offset::Work<'_>) -> Result<()> {
        self.validate_with_work(&mut crate::geometry::ContainmentWork::for_offset(work))
    }

    fn validate_with_work(
        &self,
        containment_work: &mut crate::geometry::ContainmentWork,
    ) -> Result<()> {
        ensure(
            self.points.len()
                + self.curves.len()
                + self.pieces.len()
                + self.materials.len()
                + self.seams.len()
                + self.sewing_groups.len()
                + self
                    .pieces
                    .iter()
                    .map(|piece| piece.holes.len())
                    .sum::<usize>()
                <= MAX_OBJECTS,
            "Project object limit exceeded",
        )?;
        ensure(
            self.resolution.is_finite()
                && (MIN_RESOLUTION..=MAX_RESOLUTION).contains(&self.resolution),
            "Resolution must be between 0.005 and 0.5 metres",
        )?;
        let mut ids = BTreeSet::new();
        for id in self
            .points
            .iter()
            .map(|x| x.id)
            .chain(self.curves.iter().map(|x| x.id))
            .chain(self.pieces.iter().map(|x| x.id))
            .chain(
                self.pieces
                    .iter()
                    .flat_map(|piece| piece.holes.iter().map(|hole| hole.id)),
            )
            .chain(self.materials.iter().map(|x| x.id))
            .chain(self.seams.iter().map(|x| x.id))
            .chain(self.sewing_groups.iter().map(|x| x.id))
        {
            containment_work.charge_request()?;
            ensure(
                id != 0 && ids.insert(id),
                "Object IDs must be nonzero and globally unique",
            )?;
        }
        ensure(
            self.next_id > ids.last().copied().unwrap_or(0),
            "Invalid next stable ID",
        )?;
        for point in &self.points {
            containment_work.charge_request()?;
            validate_coordinates(&point.position)?;
        }
        for curve in &self.curves {
            containment_work.charge_request()?;
            ensure(
                curve.start != curve.end
                    && self.point(curve.start).is_some()
                    && self.point(curve.end).is_some(),
                "Curve endpoints are missing or identical",
            )?;
            if let Some(controls) = &curve.controls {
                for control in controls {
                    validate_coordinates(control)?;
                }
            }
        }
        for material in &self.materials {
            containment_work.charge_request()?;
            validate_name(&material.name)?;
            ensure(
                material.areal_density.is_finite()
                    && (0.001..=100.0).contains(&material.areal_density),
                "Areal density must be in [0.001, 100] kg/m²",
            )?;
            for value in [
                material.warp_stiffness,
                material.weft_stiffness,
                material.shear_stiffness,
            ] {
                ensure(
                    value.is_finite() && value > 0.0 && value <= 1.0e8,
                    "Membrane stiffness must be positive and finite",
                )?;
            }
            ensure(
                material.bending_stiffness.is_finite()
                    && (0.0..=1.0e8).contains(&material.bending_stiffness),
                "Invalid bending stiffness",
            )?;
            ensure(
                material.damping.is_finite() && (0.0..=100.0).contains(&material.damping),
                "Invalid fabric damping",
            )?;
            ensure(
                material.friction.is_finite() && (0.0..=10.0).contains(&material.friction),
                "Invalid fabric friction",
            )?;
            ensure(
                material.collision_thickness.is_finite()
                    && (0.0001..=0.1).contains(&material.collision_thickness),
                "Invalid collision thickness",
            )?;
            ensure(
                material
                    .color
                    .iter()
                    .all(|x| x.is_finite() && (0.0..=1.0).contains(x)),
                "Invalid material color",
            )?;
        }
        let mut used_curves = BTreeSet::new();
        let mut used_points = BTreeSet::new();
        for piece in &self.pieces {
            containment_work.charge_request()?;
            validate_name(&piece.name)?;
            ensure(
                piece.grain_angle.is_finite(),
                "Grain angle must be finite radians",
            )?;
            ensure(
                piece.boundary.len() >= 3,
                "A boundary needs at least three curves",
            )?;
            ensure(
                piece.holes.len() <= 1,
                "Only one straight hole per piece is supported",
            )?;
            ensure(
                self.material(piece.material).is_some(),
                "Piece material is missing",
            )?;
            validate_coordinates(&piece.placement.translation)?;
            ensure(
                piece
                    .placement
                    .rotation
                    .iter()
                    .all(|x| x.is_finite() && x.abs() <= 1.0e6),
                "Invalid placement rotation",
            )?;
            let mut local_points = BTreeSet::new();
            for boundary in piece.boundary_loops() {
                ensure(
                    boundary.len() >= 3,
                    "A boundary needs at least three curves",
                )?;
                for (index, id) in boundary.iter().enumerate() {
                    containment_work.charge_request()?;
                    ensure(
                        used_curves.insert(*id),
                        "Curves must have exactly one boundary or internal owner",
                    )?;
                    let curve = self
                        .curve(*id)
                        .ok_or_else(|| Error("Missing boundary curve".into()))?;
                    let next = self
                        .curve(boundary[(index + 1) % boundary.len()])
                        .ok_or_else(|| Error("Missing boundary curve".into()))?;
                    ensure(
                        curve.end == next.start,
                        "Piece boundary is not a closed ordered chain",
                    )?;
                    ensure(
                        local_points.insert(curve.start),
                        "Boundary points cannot be shared between loops or repeated",
                    )?;
                }
            }
            for id in &piece.internal_curves {
                containment_work.charge_request()?;
                ensure(
                    used_curves.insert(*id),
                    "Curves must have exactly one boundary or internal owner",
                )?;
                let curve = self
                    .curve(*id)
                    .ok_or_else(|| Error(format!("Missing internal curve {id}")))?;
                ensure(
                    curve.controls.is_none(),
                    &format!("Internal curve {id} must be straight"),
                )?;
                for endpoint in [curve.start, curve.end] {
                    ensure(
                        local_points.insert(endpoint),
                        &format!("Internal curve {id} endpoints must be exclusive"),
                    )?;
                }
                let [a, b] = [curve.start, curve.end].map(|id| self.point(id).unwrap().position);
                ensure(
                    (a[0] - b[0]).hypot(a[1] - b[1]) > 1.0e-9,
                    &format!("Internal curve {id} must exceed 1e-9 metres in length"),
                )?;
            }
            for id in local_points {
                ensure(
                    used_points.insert(id),
                    "Points cannot be shared between pieces",
                )?;
            }
            if piece.holes.is_empty() {
                crate::geometry::validate_contour_with_work(self, piece, containment_work)?;
                crate::geometry::validate_internal_segments(self, piece, containment_work)?;
            } else {
                crate::geometry::validate_holed_piece(self, piece, containment_work)?;
            }
        }
        ensure(
            used_curves.len() == self.curves.len() && used_points.len() == self.points.len(),
            "Orphan construction geometry is unsupported",
        )?;
        for seam in &self.seams {
            containment_work.charge_request()?;
            validate_name(&seam.name)?;
            for interval in [seam.a, seam.b] {
                self.require_boundary_curve(interval.curve)?;
                ensure(
                    interval.t0.is_finite()
                        && interval.t1.is_finite()
                        && (0.0..=1.0).contains(&interval.t0)
                        && (0.0..=1.0).contains(&interval.t1)
                        && (interval.t1 - interval.t0).abs() > 1.0e-8,
                    "Seam intervals must have distinct endpoints in [0,1]",
                )?;
            }
            ensure(
                seam.a.curve != seam.b.curve,
                "A seam must connect distinct curves",
            )?;
            ensure(
                (2..=1024).contains(&seam.correspondence.len())
                    && seam.correspondence.first() == Some(&[0.0, 0.0])
                    && seam.correspondence.last() == Some(&[1.0, 1.0]),
                "Seam correspondence must include [0,0] and [1,1]",
            )?;
            for pair in seam.correspondence.windows(2) {
                ensure(
                    pair[1]
                        .iter()
                        .all(|x| x.is_finite() && (0.0..=1.0).contains(x))
                        && pair[0][0] < pair[1][0]
                        && pair[0][1] < pair[1][1],
                    "Seam correspondence must be strictly increasing",
                )?;
            }
        }
        ensure(
            self.sewing_groups
                .iter()
                .map(|group| group.a.len() + group.b.len())
                .sum::<usize>()
                <= MAX_SEWING_SPANS,
            "Sewing group interval reference limit exceeded",
        )?;
        for group in &self.sewing_groups {
            containment_work.charge_request()?;
            validate_name(&group.name)?;
            let mut coverage = std::collections::BTreeMap::<Id, Vec<[f64; 2]>>::new();
            for side in [&group.a, &group.b] {
                ensure(
                    (1..=1024).contains(&side.len()),
                    "A sewing group side needs 1 to 1024 intervals",
                )?;
                for interval in side {
                    self.require_boundary_curve(interval.curve)?;
                    ensure(
                        interval.t0.is_finite()
                            && interval.t1.is_finite()
                            && (0.0..=1.0).contains(&interval.t0)
                            && (0.0..=1.0).contains(&interval.t1)
                            && (interval.t1 - interval.t0).abs() > 1.0e-8,
                        "Sewing intervals must have distinct endpoints in [0,1]",
                    )?;
                    coverage
                        .entry(interval.curve)
                        .or_default()
                        .push([interval.t0.min(interval.t1), interval.t0.max(interval.t1)]);
                }
            }
            for intervals in coverage.values_mut() {
                intervals.sort_by(|a, b| a[0].total_cmp(&b[0]));
                ensure(
                    intervals.windows(2).all(|pair| pair[0][1] <= pair[1][0]),
                    "Sewing group coverage overlaps on the same curve",
                )?;
            }
            ensure(
                (2..=1024).contains(&group.correspondence.len())
                    && group.correspondence.first() == Some(&[0., 0.])
                    && group.correspondence.last() == Some(&[1., 1.]),
                "Sewing group correspondence must include [0,0] and [1,1]",
            )?;
            for pair in group.correspondence.windows(2) {
                ensure(
                    pair[1]
                        .iter()
                        .all(|x| x.is_finite() && (0.0..=1.0).contains(x))
                        && pair[0][0] < pair[1][0]
                        && pair[0][1] < pair[1][1],
                    "Sewing group correspondence must be strictly increasing",
                )?;
            }
        }
        ensure(
            self.mannequin.capsules.len() <= 128,
            "Too many mannequin capsules",
        )?;
        for capsule in &self.mannequin.capsules {
            validate_coordinates(&capsule.a)?;
            validate_coordinates(&capsule.b)?;
            ensure(
                capsule.radius.is_finite() && (0.001..=10.0).contains(&capsule.radius),
                "Invalid mannequin radius",
            )?;
        }
        Ok(())
    }

    pub(crate) fn allocate_id(&mut self) -> Result<Id> {
        let id = self.next_id;
        self.next_id = id
            .checked_add(1)
            .ok_or_else(|| Error("Stable ID limit reached".into()))?;
        Ok(id)
    }

    fn import_patterns(
        &mut self,
        patterns: Vec<crate::persistence::ImportedPattern>,
        cut_work: &mut crate::geometry::ContainmentWork,
    ) -> Result<()> {
        for pattern in patterns {
            let first_curve = self.curves.len();
            self.apply(
                Command::AddPolygon {
                    name: pattern.name,
                    vertices: pattern.vertices,
                },
                cut_work,
            )?;
            for (curve, controls) in self.curves[first_curve..].iter_mut().zip(pattern.controls) {
                curve.controls = controls;
            }
        }
        Ok(())
    }

    fn cut_piece(
        &mut self,
        piece: Id,
        start: (Id, f64),
        end: (Id, f64),
        reserve_seam: bool,
        cut_work: &mut crate::geometry::ContainmentWork,
    ) -> Result<[Id; 2]> {
        let (start_curve, start_t) = start;
        let (end_curve, end_t) = end;
        ensure(
            piece != 0 && start_curve != 0 && end_curve != 0,
            "Cut piece and curve IDs must be nonzero",
        )?;
        let piece_index = self
            .pieces
            .iter()
            .position(|item| item.id == piece)
            .ok_or_else(|| Error("Unknown cut piece".into()))?;
        let original = self.pieces[piece_index].clone();
        ensure(
            original.holes.is_empty(),
            "Cutting a piece with a hole is unsupported",
        )?;
        let boundary_index = |curve| {
            original
                .boundary
                .iter()
                .position(|&id| id == curve)
                .ok_or_else(|| Error("Cut addresses must be this piece's outer boundaries".into()))
        };
        let mut a = (boundary_index(start_curve)?, start_curve, start_t);
        let mut b = (boundary_index(end_curve)?, end_curve, end_t);
        ensure(a.0 != b.0, "Cut needs two distinct boundary curves")?;
        if a.0 > b.0 {
            std::mem::swap(&mut a, &mut b);
        }
        let owned: BTreeSet<_> = original.boundary.iter().copied().collect();
        ensure(
            !self
                .seams
                .iter()
                .any(|seam| owned.contains(&seam.a.curve) || owned.contains(&seam.b.curve))
                && !self.sewing_groups.iter().any(|group| {
                    group
                        .a
                        .iter()
                        .chain(&group.b)
                        .any(|interval| owned.contains(&interval.curve))
                }),
            "Cut cannot remap sewing on any boundary of the target piece",
        )?;
        let additions = if reserve_seam { 10 } else { 9 };
        ensure(
            self.points.len()
                + self.curves.len()
                + self.pieces.len()
                + self.materials.len()
                + self.seams.len()
                + self.sewing_groups.len()
                + self
                    .pieces
                    .iter()
                    .map(|piece| piece.holes.len())
                    .sum::<usize>()
                <= MAX_OBJECTS - additions,
            "Cut exceeds the project object limit",
        )?;
        let first_id = self.next_id;
        ensure(
            first_id.checked_add(additions as u64).is_some(),
            "Cut exceeds the stable ID limit",
        )?;
        let reserve = crate::geometry::validate_straight_cut(
            self,
            &original,
            (a.0, a.2),
            (b.0, b.2),
            cut_work,
        )?;
        self.apply(Command::SplitCurve { curve: a.1, t: a.2 }, cut_work)?;
        self.apply(Command::SplitCurve { curve: b.1, t: b.2 }, cut_work)?;
        let (anchor_a, tail_a, anchor_b, tail_b) =
            (first_id, first_id + 1, first_id + 2, first_id + 3);
        let new_piece = self.allocate_id()?;
        let copy_a = self.allocate_id()?;
        let copy_b = self.allocate_id()?;
        let retained_chord = self.allocate_id()?;
        let new_chord = self.allocate_id()?;
        for (id, anchor) in [(copy_a, anchor_a), (copy_b, anchor_b)] {
            self.points.push(Point {
                id,
                position: self.point(anchor).unwrap().position,
            });
        }
        self.curves
            .iter_mut()
            .find(|curve| curve.id == tail_a)
            .unwrap()
            .start = copy_a;
        self.curves
            .iter_mut()
            .find(|curve| curve.id == b.1)
            .unwrap()
            .end = copy_b;
        self.curves.extend([
            Curve {
                id: retained_chord,
                start: anchor_a,
                end: anchor_b,
                controls: None,
            },
            Curve {
                id: new_chord,
                start: copy_b,
                end: copy_a,
                controls: None,
            },
        ]);
        let mut retained = original.clone();
        retained.boundary = original.boundary[..=a.0].to_vec();
        retained.boundary.extend([retained_chord, tail_b]);
        retained
            .boundary
            .extend_from_slice(&original.boundary[b.0 + 1..]);
        let mut child = Piece {
            id: new_piece,
            name: format!("Cut piece #{new_piece}"),
            boundary: vec![tail_a],
            internal_curves: Vec::new(),
            ..original.clone()
        };
        child
            .boundary
            .extend_from_slice(&original.boundary[a.0 + 1..=b.0]);
        child.boundary.push(new_chord);
        (retained.internal_curves, child.internal_curves) =
            crate::geometry::partition_cut_annotations(
                self, &original, &retained, &child, reserve, cut_work,
            )?;
        self.pieces[piece_index] = retained;
        self.pieces.push(child);
        Ok([retained_chord, new_chord])
    }

    fn apply(
        &mut self,
        command: Command,
        cut_work: &mut crate::geometry::ContainmentWork,
    ) -> Result<()> {
        match command {
            Command::ImportSvgPatterns { svg } => {
                let imported = crate::SvgPatternImport::from_bytes(svg.as_bytes())?;
                self.import_patterns(imported.patterns, cut_work)?;
            }
            Command::ImportDxfPatterns { dxf } => {
                let imported = crate::DxfPatternImport::from_bytes(dxf.as_bytes())?;
                self.import_patterns(imported.patterns, cut_work)?;
            }
            Command::AddPolygon { name, vertices } => {
                ensure(
                    (3..=4096).contains(&vertices.len()),
                    "Polygon needs 3 to 4096 vertices",
                )?;
                validate_name(&name)?;
                for vertex in &vertices {
                    validate_coordinates(vertex)?;
                }
                let material = self
                    .materials
                    .first()
                    .ok_or_else(|| Error("A material is required".into()))?
                    .id;
                let id = self.allocate_id()?;
                let mut point_ids = Vec::new();
                for position in vertices {
                    let id = self.allocate_id()?;
                    self.points.push(Point { id, position });
                    point_ids.push(id);
                }
                let mut boundary = Vec::new();
                for index in 0..point_ids.len() {
                    let id = self.allocate_id()?;
                    self.curves.push(Curve {
                        id,
                        start: point_ids[index],
                        end: point_ids[(index + 1) % point_ids.len()],
                        controls: None,
                    });
                    boundary.push(id);
                }
                self.pieces.push(Piece {
                    id,
                    name,
                    boundary,
                    holes: Vec::new(),
                    internal_curves: Vec::new(),
                    placement: Placement::default(),
                    material,
                    grain_angle: 0.,
                });
            }
            Command::AddRectangle {
                name,
                origin: [x, y],
                width,
                height,
            } => {
                ensure(
                    width.is_finite() && height.is_finite() && width > 1.0e-6 && height > 1.0e-6,
                    "Rectangle dimensions must be positive metres",
                )?;
                self.apply(
                    Command::AddPolygon {
                        name,
                        vertices: vec![
                            [x, y],
                            [x + width, y],
                            [x + width, y + height],
                            [x, y + height],
                        ],
                    },
                    cut_work,
                )?;
            }
            Command::AddEllipse {
                name,
                origin: [x, y],
                width,
                height,
            } => {
                ensure(
                    width.is_finite() && height.is_finite() && width > 1.0e-6 && height > 1.0e-6,
                    "Ellipse dimensions must be positive metres",
                )?;
                let [rx, ry] = [width * 0.5, height * 0.5];
                let [cx, cy, right, top] = [x + rx, y + ry, x + width, y + height];
                let first_curve = self.curves.len();
                self.apply(
                    Command::AddPolygon {
                        name,
                        vertices: vec![[cx, y], [right, cy], [cx, top], [x, cy]],
                    },
                    cut_work,
                )?;
                // shortcut: four cubics approximate a conic; use a conic relation for exact ellipse editing.
                let k = 4.0 * (std::f64::consts::SQRT_2 - 1.0) / 3.0;
                let controls = [
                    [[cx + k * rx, y], [right, cy - k * ry]],
                    [[right, cy + k * ry], [cx + k * rx, top]],
                    [[cx - k * rx, top], [x, cy + k * ry]],
                    [[x, cy - k * ry], [cx - k * rx, y]],
                ];
                for (curve, controls) in self.curves[first_curve..].iter_mut().zip(controls) {
                    curve.controls = Some(controls);
                }
            }
            Command::AddHole {
                piece,
                mut vertices,
            } => {
                let owner = self
                    .piece(piece)
                    .ok_or_else(|| Error("Unknown piece".into()))?;
                ensure(
                    owner.holes.is_empty(),
                    "Only one straight hole per piece is supported",
                )?;
                ensure(
                    (3..=4096).contains(&vertices.len()),
                    "Hole needs 3 to 4096 vertices",
                )?;
                let additional = 1 + 2 * vertices.len();
                let objects = self.points.len()
                    + self.curves.len()
                    + self.pieces.len()
                    + self.materials.len()
                    + self.seams.len()
                    + self.sewing_groups.len()
                    + self
                        .pieces
                        .iter()
                        .map(|piece| piece.holes.len())
                        .sum::<usize>();
                ensure(
                    objects <= MAX_OBJECTS - additional,
                    "Hole exceeds the project object limit",
                )?;
                ensure(
                    self.next_id.checked_add(additional as u64).is_some(),
                    "Hole exceeds the stable ID limit",
                )?;
                crate::geometry::prepare_hole(self, owner, &mut vertices, cut_work)?;
                let id = self.allocate_id()?;
                let mut points = Vec::with_capacity(vertices.len());
                for position in vertices {
                    let id = self.allocate_id()?;
                    self.points.push(Point { id, position });
                    points.push(id);
                }
                let mut boundary = Vec::with_capacity(points.len());
                for index in 0..points.len() {
                    let id = self.allocate_id()?;
                    self.curves.push(Curve {
                        id,
                        start: points[index],
                        end: points[(index + 1) % points.len()],
                        controls: None,
                    });
                    boundary.push(id);
                }
                self.pieces
                    .iter_mut()
                    .find(|owner| owner.id == piece)
                    .unwrap()
                    .holes
                    .push(Hole { id, boundary });
            }
            Command::RemoveHole { hole } => {
                let owner = self
                    .pieces
                    .iter_mut()
                    .find(|piece| piece.holes.iter().any(|item| item.id == hole))
                    .ok_or_else(|| Error("Unknown hole".into()))?;
                let index = owner.holes.iter().position(|item| item.id == hole).unwrap();
                let boundary = owner.holes.remove(index).boundary;
                let points: BTreeSet<_> = self
                    .curves
                    .iter()
                    .filter(|curve| boundary.contains(&curve.id))
                    .flat_map(|curve| [curve.start, curve.end])
                    .collect();
                self.curves.retain(|curve| !boundary.contains(&curve.id));
                self.points.retain(|point| !points.contains(&point.id));
            }
            Command::AddInternalSegment { piece, start, end } => {
                ensure(self.piece(piece).is_some(), "Unknown piece")?;
                validate_coordinates(&start)?;
                validate_coordinates(&end)?;
                let start_id = self.allocate_id()?;
                let end_id = self.allocate_id()?;
                let curve = self.allocate_id()?;
                self.points.push(Point {
                    id: start_id,
                    position: start,
                });
                self.points.push(Point {
                    id: end_id,
                    position: end,
                });
                self.curves.push(Curve {
                    id: curve,
                    start: start_id,
                    end: end_id,
                    controls: None,
                });
                self.pieces
                    .iter_mut()
                    .find(|item| item.id == piece)
                    .unwrap()
                    .internal_curves
                    .push(curve);
            }
            Command::RemoveInternalCurve { curve } => {
                let owner = self
                    .pieces
                    .iter_mut()
                    .find(|piece| piece.internal_curves.contains(&curve))
                    .ok_or_else(|| Error(format!("Curve {curve} is not an internal annotation")))?;
                owner.internal_curves.retain(|&id| id != curve);
                let original = self
                    .curve(curve)
                    .ok_or_else(|| Error(format!("Missing internal curve {curve}")))?;
                let endpoints = [original.start, original.end];
                self.curves.retain(|item| item.id != curve);
                self.points.retain(|item| !endpoints.contains(&item.id));
            }
            Command::MovePoint { point, position } => {
                self.points
                    .iter_mut()
                    .find(|item| item.id == point)
                    .ok_or_else(|| Error("Unknown point".into()))?
                    .position = position;
            }
            Command::SetCurveControls { curve, controls } => {
                self.require_boundary_curve(curve)?;
                ensure(
                    !self
                        .pieces
                        .iter()
                        .any(|piece| piece.boundary.contains(&curve) && !piece.holes.is_empty()),
                    "Curve controls are unsupported while the piece has a hole",
                )?;
                self.curves
                    .iter_mut()
                    .find(|item| item.id == curve)
                    .ok_or_else(|| Error("Unknown curve".into()))?
                    .controls = controls;
            }
            Command::SplitCurve { curve, t } => {
                ensure(
                    self.pieces.iter().any(|piece| {
                        piece
                            .boundary_loops()
                            .any(|boundary| boundary.contains(&curve))
                    }),
                    "Curve has no piece boundary; internal annotations cannot be split",
                )?;
                ensure(
                    t.is_finite() && t > 1.0e-6 && t < 1.0 - 1.0e-6,
                    "Split t must be strictly inside the curve",
                )?;
                let original = self
                    .curve(curve)
                    .ok_or_else(|| Error("Unknown curve".into()))?
                    .clone();
                let a = self
                    .point(original.start)
                    .ok_or_else(|| Error("Missing curve start".into()))?
                    .position;
                let b = self
                    .point(original.end)
                    .ok_or_else(|| Error("Missing curve end".into()))?
                    .position;
                let position = original.evaluate(self, t)?;
                let point = self.allocate_id()?;
                let second = self.allocate_id()?;
                let (left_controls, right_controls) = if let Some([c, d]) = original.controls {
                    let lerp = |a: [f64; 2], b: [f64; 2]| {
                        std::array::from_fn(|axis| a[axis] + (b[axis] - a[axis]) * t)
                    };
                    let ac = lerp(a, c);
                    let cd = lerp(c, d);
                    let db = lerp(d, b);
                    (Some([ac, lerp(ac, cd)]), Some([lerp(cd, db), db]))
                } else {
                    (None, None)
                };
                self.points.push(Point {
                    id: point,
                    position,
                });
                let first = self
                    .curves
                    .iter_mut()
                    .find(|item| item.id == curve)
                    .ok_or_else(|| Error("Unknown curve".into()))?;
                first.end = point;
                first.controls = left_controls;
                self.curves.push(Curve {
                    id: second,
                    start: point,
                    end: original.end,
                    controls: right_controls,
                });
                let piece = self
                    .pieces
                    .iter_mut()
                    .find(|piece| {
                        piece
                            .boundary_loops()
                            .any(|boundary| boundary.contains(&curve))
                    })
                    .ok_or_else(|| Error("Curve has no piece".into()))?;
                let boundary = if piece.boundary.contains(&curve) {
                    &mut piece.boundary
                } else {
                    &mut piece
                        .holes
                        .iter_mut()
                        .find(|hole| hole.boundary.contains(&curve))
                        .unwrap()
                        .boundary
                };
                let index = boundary
                    .iter()
                    .position(|&id| id == curve)
                    .ok_or_else(|| Error("Curve has no piece".into()))?;
                boundary.insert(index + 1, second);
                for seam in std::mem::take(&mut self.seams) {
                    let portions = split_seam(seam, curve, second, t);
                    for (index, mut portion) in portions.into_iter().enumerate() {
                        if index > 0 {
                            portion.id = self.allocate_id()?;
                        }
                        self.seams.push(portion);
                    }
                }
                for group in &mut self.sewing_groups {
                    for side in [&mut group.a, &mut group.b] {
                        *side = side
                            .iter()
                            .flat_map(|&interval| split_group_interval(interval, curve, second, t))
                            .collect();
                    }
                }
            }
            Command::CutPiece {
                piece,
                start_curve,
                start_t,
                end_curve,
                end_t,
            } => {
                self.cut_piece(
                    piece,
                    (start_curve, start_t),
                    (end_curve, end_t),
                    false,
                    cut_work,
                )?;
            }
            Command::CutAndSew {
                piece,
                start_curve,
                start_t,
                end_curve,
                end_t,
            } => {
                let [a, b] = self.cut_piece(
                    piece,
                    (start_curve, start_t),
                    (end_curve, end_t),
                    true,
                    cut_work,
                )?;
                self.apply(
                    Command::AddSeam {
                        name: format!("Cut seam #{}", self.next_id),
                        a: CurveInterval::full(a),
                        b: CurveInterval::reversed(b),
                        correspondence: vec![[0., 0.], [1., 1.]],
                    },
                    cut_work,
                )?;
            }
            Command::JoinCurve { curve } => {
                let piece_index = self
                    .pieces
                    .iter()
                    .position(|piece| {
                        piece
                            .boundary_loops()
                            .any(|boundary| boundary.contains(&curve))
                    })
                    .ok_or_else(|| {
                        Error(
                            "Curve has no piece boundary; internal annotations cannot be joined"
                                .into(),
                        )
                    })?;
                let boundary = self.pieces[piece_index]
                    .boundary_loops()
                    .find(|boundary| boundary.contains(&curve))
                    .unwrap();
                ensure(
                    boundary.len() > 3,
                    "Join must retain at least three boundary curves",
                )?;
                let index = boundary.iter().position(|&id| id == curve).unwrap();
                let second = boundary[(index + 1) % boundary.len()];
                let left = self
                    .curve(curve)
                    .ok_or_else(|| Error("Unknown curve".into()))?
                    .clone();
                let right = self
                    .curve(second)
                    .ok_or_else(|| Error("Missing next curve".into()))?
                    .clone();
                ensure(
                    left.end == right.start,
                    "Join needs consecutive oriented curves",
                )?;
                ensure(
                    self.pieces
                        .iter()
                        .flat_map(|piece| piece.boundary_loops().flatten())
                        .filter(|&&id| id == curve || id == second)
                        .count()
                        == 2
                        && !self.curves.iter().any(|other| {
                            other.id != curve
                                && other.id != second
                                && (other.start == left.end || other.end == left.end)
                        }),
                    "Join would remove referenced construction geometry",
                )?;
                let (controls, t) =
                    joined_geometry(self, &left, &right, self.pieces[piece_index].placement)?;
                for seam in &mut self.seams {
                    for interval in [&mut seam.a, &mut seam.b] {
                        if interval.curve == curve {
                            interval.t0 *= t;
                            interval.t1 *= t;
                        } else if interval.curve == second {
                            interval.curve = curve;
                            interval.t0 = t + (1.0 - t) * interval.t0;
                            interval.t1 = t + (1.0 - t) * interval.t1;
                        }
                    }
                }
                for group in &mut self.sewing_groups {
                    for interval in group.a.iter_mut().chain(&mut group.b) {
                        if interval.curve == curve {
                            interval.t0 *= t;
                            interval.t1 *= t;
                        } else if interval.curve == second {
                            interval.curve = curve;
                            interval.t0 = t + (1.0 - t) * interval.t0;
                            interval.t1 = t + (1.0 - t) * interval.t1;
                        }
                    }
                }
                let joined = self
                    .curves
                    .iter_mut()
                    .find(|item| item.id == curve)
                    .unwrap();
                joined.end = right.end;
                joined.controls = controls;
                let owner = &mut self.pieces[piece_index];
                let boundary = if owner.boundary.contains(&curve) {
                    &mut owner.boundary
                } else {
                    &mut owner
                        .holes
                        .iter_mut()
                        .find(|hole| hole.boundary.contains(&curve))
                        .unwrap()
                        .boundary
                };
                boundary.retain(|&id| id != second);
                self.curves.retain(|item| item.id != second);
                self.points.retain(|item| item.id != left.end);
            }
            Command::OffsetOutline {
                piece,
                width,
                outward,
            } => {
                *self = self
                    .prepare_outline_offset(piece, width, outward, || false)?
                    .candidate;
            }
            Command::ResizePiece {
                piece,
                width,
                height,
            } => {
                ensure(
                    width.is_finite() && height.is_finite() && width > 1.0e-6 && height > 1.0e-6,
                    "Piece dimensions must be positive metres",
                )?;
                let [min, max] = self.piece_bounds(piece)?;
                let scale = [width / (max[0] - min[0]), height / (max[1] - min[1])];
                let owner = self
                    .piece(piece)
                    .ok_or_else(|| Error("Unknown piece".into()))?;
                let owned: BTreeSet<_> = owner.construction_curves().copied().collect();
                let mut point_ids = BTreeSet::new();
                for curve in self
                    .curves
                    .iter_mut()
                    .filter(|curve| owned.contains(&curve.id))
                {
                    point_ids.extend([curve.start, curve.end]);
                    if let Some(controls) = &mut curve.controls {
                        for control in controls {
                            for axis in 0..2 {
                                control[axis] =
                                    min[axis] + (control[axis] - min[axis]) * scale[axis];
                            }
                        }
                    }
                }
                for point in self
                    .points
                    .iter_mut()
                    .filter(|point| point_ids.contains(&point.id))
                {
                    for axis in 0..2 {
                        point.position[axis] =
                            min[axis] + (point.position[axis] - min[axis]) * scale[axis];
                    }
                }
            }
            Command::TransformPiece {
                piece,
                translation,
                angle,
                pivot,
            } => {
                let original = self
                    .piece(piece)
                    .ok_or_else(|| Error("Unknown piece".into()))?
                    .clone();
                validate_coordinates(&translation)?;
                validate_coordinates(&pivot)?;
                ensure(
                    angle.is_finite() && angle.abs() <= 1.0e6,
                    "Transform angle must be finite and within 1000000 radians",
                )?;
                let (sin, cos) = angle.sin_cos();
                let transform = |point: [f64; 2]| {
                    if angle == 0.0 {
                        std::array::from_fn(|i| {
                            if translation[i] == 0.0 {
                                point[i]
                            } else {
                                point[i] + translation[i]
                            }
                        })
                    } else {
                        let [u, v] = std::array::from_fn(|i| point[i] - pivot[i]);
                        [
                            pivot[0] + cos * u - sin * v + translation[0],
                            pivot[1] + sin * u + cos * v + translation[1],
                        ]
                    }
                };
                let mut point_ids = BTreeSet::new();
                for curve in self
                    .curves
                    .iter_mut()
                    .filter(|curve| original.construction_curves().any(|&id| id == curve.id))
                {
                    point_ids.extend([curve.start, curve.end]);
                    if let Some(controls) = &mut curve.controls {
                        *controls = controls.map(transform);
                    }
                }
                for point in self
                    .points
                    .iter_mut()
                    .filter(|point| point_ids.contains(&point.id))
                {
                    point.position = transform(point.position);
                }
                let transformed = self
                    .pieces
                    .iter_mut()
                    .find(|item| item.id == piece)
                    .unwrap();
                transformed.placement =
                    original
                        .placement
                        .compensate_pattern_transform(translation, angle, pivot);
                if angle != 0.0 {
                    let (grain_sin, grain_cos) = original.grain_angle.sin_cos();
                    transformed.grain_angle = (sin * grain_cos + cos * grain_sin)
                        .atan2(cos * grain_cos - sin * grain_sin);
                }
            }
            Command::SetPlacement { piece, placement } => {
                self.pieces
                    .iter_mut()
                    .find(|item| item.id == piece)
                    .ok_or_else(|| Error("Unknown piece".into()))?
                    .placement = placement;
            }
            Command::SetGrain { piece, angle } => {
                self.pieces
                    .iter_mut()
                    .find(|item| item.id == piece)
                    .ok_or_else(|| Error("Unknown piece".into()))?
                    .grain_angle = angle;
            }
            Command::SetMaterial { piece, material } => {
                self.pieces
                    .iter_mut()
                    .find(|item| item.id == piece)
                    .ok_or_else(|| Error("Unknown piece".into()))?
                    .material = material;
            }
            Command::AddMaterial { mut material } => {
                material.id = self.allocate_id()?;
                self.materials.push(material);
            }
            Command::UpdateMaterial { material } => {
                let target = self
                    .materials
                    .iter_mut()
                    .find(|item| item.id == material.id)
                    .ok_or_else(|| Error("Unknown material".into()))?;
                *target = material;
            }
            Command::AddSeam {
                name,
                a,
                b,
                correspondence,
            } => {
                let id = self.allocate_id()?;
                self.seams.push(Seam {
                    id,
                    name,
                    a,
                    b,
                    correspondence,
                });
            }
            Command::UpdateSeam { seam } => {
                let target = self
                    .seams
                    .iter_mut()
                    .find(|item| item.id == seam.id)
                    .ok_or_else(|| Error("Unknown seam".into()))?;
                *target = seam;
            }
            Command::RemoveSeam { seam } => {
                ensure(
                    self.seams.iter().any(|item| item.id == seam),
                    "Unknown seam",
                )?;
                self.seams.retain(|item| item.id != seam);
            }
            Command::AddSewingGroup {
                name,
                a,
                b,
                correspondence,
            } => {
                let id = self.allocate_id()?;
                self.sewing_groups.push(SewingGroup {
                    id,
                    name,
                    a,
                    b,
                    correspondence,
                });
            }
            Command::AddSewingByLength { name, a, b } => {
                let a = self.interval_from_length(a)?;
                let b = self.interval_from_length(b)?;
                self.apply(
                    Command::AddSewingGroup {
                        name,
                        a: vec![a],
                        b: vec![b],
                        correspondence: vec![[0., 0.], [1., 1.]],
                    },
                    cut_work,
                )?;
            }
            Command::UpdateSewingGroup { group } => {
                let target = self
                    .sewing_groups
                    .iter_mut()
                    .find(|item| item.id == group.id)
                    .ok_or_else(|| Error("Unknown sewing group".into()))?;
                *target = group;
            }
            Command::RemoveSewingGroup { group } => {
                ensure(
                    self.sewing_groups.iter().any(|item| item.id == group),
                    "Unknown sewing group",
                )?;
                self.sewing_groups.retain(|item| item.id != group);
            }
            Command::RemovePiece { piece } => {
                let owner = self
                    .piece(piece)
                    .ok_or_else(|| Error("Unknown piece".into()))?;
                let owned: BTreeSet<_> = owner.construction_curves().copied().collect();
                let points: BTreeSet<Id> = self
                    .curves
                    .iter()
                    .filter(|curve| owned.contains(&curve.id))
                    .flat_map(|curve| [curve.start, curve.end])
                    .collect();
                self.pieces.retain(|item| item.id != piece);
                self.curves.retain(|item| !owned.contains(&item.id));
                self.points.retain(|item| !points.contains(&item.id));
                self.seams.retain(|item| {
                    !owned.contains(&item.a.curve) && !owned.contains(&item.b.curve)
                });
                self.sewing_groups.retain(|group| {
                    !group
                        .a
                        .iter()
                        .chain(&group.b)
                        .any(|interval| owned.contains(&interval.curve))
                });
            }
            Command::SetResolution { metres } => self.resolution = metres,
            Command::SetMannequin { mannequin } => self.mannequin = mannequin,
            Command::Transaction(commands) => {
                ensure(
                    (1..=1024).contains(&commands.len()),
                    "Transaction needs 1 to 1024 commands",
                )?;
                for command in commands {
                    ensure(
                        !matches!(command, Command::Transaction(_)),
                        "Nested transactions are unsupported",
                    )?;
                    self.apply(command, cut_work)?;
                }
            }
        }
        Ok(())
    }
}

fn split_group_interval(
    interval: CurveInterval,
    curve: Id,
    second: Id,
    t: f64,
) -> Vec<CurveInterval> {
    if interval.curve != curve {
        return vec![interval];
    }
    let remap = |t0: f64, t1: f64| {
        if t0.max(t1) <= t {
            CurveInterval {
                curve,
                t0: t0 / t,
                t1: t1 / t,
            }
        } else {
            CurveInterval {
                curve: second,
                t0: (t0 - t) / (1. - t),
                t1: (t1 - t) / (1. - t),
            }
        }
    };
    if interval.t0.min(interval.t1) < t && t < interval.t0.max(interval.t1) {
        vec![remap(interval.t0, t), remap(t, interval.t1)]
    } else {
        vec![remap(interval.t0, interval.t1)]
    }
}

fn joined_geometry(
    project: &Project,
    left: &Curve,
    right: &Curve,
    placement: Placement,
) -> Result<(Option<[[f64; 2]; 2]>, f64)> {
    const TOLERANCE: f64 = 1.0e-12;
    let a = project
        .point(left.start)
        .ok_or_else(|| Error("Missing curve start".into()))?
        .position;
    let m = project
        .point(left.end)
        .ok_or_else(|| Error("Missing join point".into()))?
        .position;
    let b = project
        .point(right.end)
        .ok_or_else(|| Error("Missing curve end".into()))?
        .position;
    let sub = |a: [f64; 2], b: [f64; 2]| std::array::from_fn(|axis| a[axis] - b[axis]);
    let length = |v: [f64; 2]| v[0].hypot(v[1]);
    let rotation_scale = placement
        .rotation
        .into_iter()
        .map(|angle| angle.sin().hypot(angle.cos()).max(1.0).next_up())
        .fold(1.0, |scale, norm| (scale * norm).next_up());
    let [x, y, z] = placement.translation;
    let translation_scale = x.hypot(y).next_up().hypot(z).next_up();
    // Include the unchanged placement's rotation and translation arithmetic in the physical-point error budget.
    let roundoff = |points: &[[f64; 2]]| {
        let scale = points
            .iter()
            .map(|&point| length(point).next_up())
            .fold(0.0, f64::max);
        256.0 * f64::EPSILON * ((rotation_scale * scale).next_up() + translation_scale).next_up()
    };
    let interior = |t: f64| t.is_finite() && t > 1.0e-6 && t < 1.0 - 1.0e-6;
    match (left.controls, right.controls) {
        (None, None) => {
            let lengths = [length(sub(m, a)), length(sub(b, m))];
            let t = lengths[0] / (lengths[0] + lengths[1]);
            let join = std::array::from_fn(|axis| a[axis] + (b[axis] - a[axis]) * t);
            let error = (rotation_scale * length(sub(join, m))).next_up();
            ensure(
                interior(t) && (error + roundoff(&[a, m, b])).next_up() <= TOLERANCE,
                "Join needs nondegenerate collinear straight curves with 1e-6 < t < 1-1e-6 and error including f64 roundoff <= 1e-12 m",
            )?;
            Ok((None, t))
        }
        (Some([c, d]), Some([e, f])) => {
            // Endpoint derivatives scale by t^k and (1-t)^k; higher orders recover stationary joins.
            let derivatives = [
                [sub(m, d), sub(e, m)],
                [
                    std::array::from_fn(|i| m[i] - 2.0 * d[i] + c[i]),
                    std::array::from_fn(|i| f[i] - 2.0 * e[i] + m[i]),
                ],
                [
                    std::array::from_fn(|i| m[i] - 3.0 * d[i] + 3.0 * c[i] - a[i]),
                    std::array::from_fn(|i| b[i] - 3.0 * f[i] + 3.0 * e[i] - m[i]),
                ],
            ];
            let mut accepted: Option<([[f64; 2]; 2], f64)> = None;
            for (order, derivative) in derivatives.into_iter().enumerate() {
                let norms = derivative.map(length);
                if norms.iter().any(|&norm| norm <= TOLERANCE) {
                    continue;
                }
                let weights = norms.map(|norm| norm.powf(1.0 / (order + 1) as f64));
                let t = weights[0] / (weights[0] + weights[1]);
                if !interior(t) {
                    continue;
                }
                let controls = [
                    std::array::from_fn(|i| a[i] + (c[i] - a[i]) / t),
                    std::array::from_fn(|i| b[i] + (f[i] - b[i]) / (1.0 - t)),
                ];
                let lerp =
                    |a: [f64; 2], b: [f64; 2]| std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t);
                let ac = lerp(a, controls[0]);
                let cd = lerp(controls[0], controls[1]);
                let db = lerp(controls[1], b);
                let left_control = lerp(ac, cd);
                let right_control = lerp(cd, db);
                let join = lerp(left_control, right_control);
                let budget = roundoff(&[a, c, d, m, e, f, b, controls[0], controls[1]]);
                // Bernstein weights are nonnegative and sum to one: this bounds every curve point.
                if [[a, c, d, m], [m, e, f, b]]
                    .into_iter()
                    .zip([[a, ac, left_control, join], [join, right_control, db, b]])
                    .any(|(actual, candidate)| {
                        actual.into_iter().zip(candidate).any(|(a, b)| {
                            let error = (rotation_scale * length(sub(a, b))).next_up();
                            let bound = (error + budget).next_up();
                            !bound.is_finite() || bound > TOLERANCE
                        })
                    })
                {
                    continue;
                }
                if let Some((_, previous_t)) = accepted {
                    ensure(
                        (t - previous_t).abs() <= TOLERANCE,
                        "Join parameter is ambiguous within the geometric tolerance",
                    )?;
                } else {
                    accepted = Some((controls, t));
                }
            }
            accepted
                .map(|(controls, t)| (Some(controls), t))
                .ok_or_else(|| {
                    Error(
                        "Join needs compatible nondegenerate cubic children with 1e-6 < t < 1-1e-6 and error <= 1e-12 m".into(),
                    )
                })
        }
        _ => Err(Error(
            "Join needs two straight curves or two compatible cubic children".into(),
        )),
    }
}

fn split_seam(seam: Seam, curve: Id, second: Id, t: f64) -> Vec<Seam> {
    let source = if seam.a.curve == curve {
        0
    } else if seam.b.curve == curve {
        1
    } else {
        return vec![seam];
    };
    let intervals = [seam.a, seam.b];
    let interval = intervals[source];
    let fraction = (t - interval.t0) / (interval.t1 - interval.t0);
    let mut cuts = vec![[0.0, 0.0]];
    if fraction > 0.0 && fraction < 1.0 {
        let mut cut = [0.0; 2];
        cut[source] = fraction;
        cut[1 - source] = crate::geometry::correspondence(&seam.correspondence, fraction, source);
        cuts.push(cut);
    }
    cuts.push([1.0, 1.0]);
    cuts.windows(2)
        .map(|bounds| {
            let [start, end] = [bounds[0], bounds[1]];
            let mut portion = seam.clone();
            portion.correspondence = vec![[0.0, 0.0]];
            for pair in &seam.correspondence {
                // An interpolated cut can round onto a breakpoint on either axis.
                if (0..2).all(|axis| pair[axis] > start[axis] && pair[axis] < end[axis]) {
                    portion.correspondence.push(std::array::from_fn(|axis| {
                        (pair[axis] - start[axis]) / (end[axis] - start[axis])
                    }));
                }
            }
            portion.correspondence.push([1.0, 1.0]);
            let remap = |axis: usize| {
                let interval = intervals[axis];
                let parameter = |fraction| {
                    if fraction == 0.0 {
                        interval.t0
                    } else if fraction == 1.0 {
                        interval.t1
                    } else if axis == source {
                        // This boundary was constructed at the exact authored split t.
                        t
                    } else {
                        interval.parameter(fraction)
                    }
                };
                let [mut t0, mut t1] = [parameter(start[axis]), parameter(end[axis])];
                let mut id = interval.curve;
                if axis == source {
                    if (t0 + t1) * 0.5 > t {
                        id = second;
                        t0 = (t0 - t) / (1.0 - t);
                        t1 = (t1 - t) / (1.0 - t);
                    } else {
                        t0 /= t;
                        t1 /= t;
                    }
                }
                CurveInterval { curve: id, t0, t1 }
            };
            portion.a = remap(0);
            portion.b = remap(1);
            portion
        })
        .collect()
}

pub(crate) fn validate_name(name: &str) -> Result<()> {
    ensure(
        !name.trim().is_empty() && name.len() <= 256 && !name.chars().any(char::is_control),
        "Names must be nonempty, at most 256 bytes, with no control characters",
    )
}

pub(crate) fn validate_coordinates<const N: usize>(point: &[f64; N]) -> Result<()> {
    ensure(
        point
            .iter()
            .all(|x| x.is_finite() && x.abs() <= MAX_COORDINATE),
        "Coordinates must be finite and within 1000 metres",
    )
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Command {
    ImportSvgPatterns {
        svg: String,
    },
    ImportDxfPatterns {
        dxf: String,
    },
    AddPolygon {
        name: String,
        vertices: Vec<[f64; 2]>,
    },
    AddRectangle {
        name: String,
        origin: [f64; 2],
        width: f64,
        height: f64,
    },
    AddEllipse {
        name: String,
        origin: [f64; 2],
        width: f64,
        height: f64,
    },
    AddInternalSegment {
        piece: Id,
        start: [f64; 2],
        end: [f64; 2],
    },
    AddHole {
        piece: Id,
        vertices: Vec<[f64; 2]>,
    },
    RemoveHole {
        hole: Id,
    },
    RemoveInternalCurve {
        curve: Id,
    },
    MovePoint {
        point: Id,
        position: [f64; 2],
    },
    SetCurveControls {
        curve: Id,
        controls: Option<[[f64; 2]; 2]>,
    },
    SplitCurve {
        curve: Id,
        t: f64,
    },
    CutPiece {
        piece: Id,
        start_curve: Id,
        start_t: f64,
        end_curve: Id,
        end_t: f64,
    },
    CutAndSew {
        piece: Id,
        start_curve: Id,
        start_t: f64,
        end_curve: Id,
        end_t: f64,
    },
    JoinCurve {
        curve: Id,
    },
    /// Replaces one whole exterior with editable material geometry; unsupported attachments refuse.
    OffsetOutline {
        piece: Id,
        width: f64,
        outward: bool,
    },
    ResizePiece {
        piece: Id,
        width: f64,
        height: f64,
    },
    /// Rigid 2D layout edit; compensates placement and rotates grain to retain world arrangement.
    TransformPiece {
        piece: Id,
        translation: [f64; 2],
        angle: f64,
        pivot: [f64; 2],
    },
    SetPlacement {
        piece: Id,
        placement: Placement,
    },
    SetGrain {
        piece: Id,
        /// Warp direction in material-space radians, counterclockwise from +U.
        angle: f64,
    },
    SetMaterial {
        piece: Id,
        material: Id,
    },
    AddMaterial {
        material: Material,
    },
    UpdateMaterial {
        material: Material,
    },
    AddSeam {
        name: String,
        a: CurveInterval,
        b: CurveInterval,
        correspondence: Vec<[f64; 2]>,
    },
    UpdateSeam {
        seam: Seam,
    },
    RemoveSeam {
        seam: Id,
    },
    AddSewingGroup {
        name: String,
        a: Vec<CurveInterval>,
        b: Vec<CurveInterval>,
        correspondence: Vec<[f64; 2]>,
    },
    AddSewingByLength {
        name: String,
        a: SewingSideByLength,
        b: SewingSideByLength,
    },
    UpdateSewingGroup {
        group: SewingGroup,
    },
    RemoveSewingGroup {
        group: Id,
    },
    RemovePiece {
        piece: Id,
    },
    SetResolution {
        metres: f64,
    },
    SetMannequin {
        mannequin: Mannequin,
    },
    Transaction(Vec<Command>),
}

#[derive(Debug, Clone)]
pub struct Editor {
    pub project: Project,
    undo: Vec<Project>,
    redo: Vec<Project>,
    revision: u64,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new(Project::default()).expect("Default project is valid")
    }
}

impl Editor {
    pub fn new(project: Project) -> Result<Self> {
        project.validate()?;
        project.validate_sewing_metrics(None)?;
        Ok(Self {
            revision: project.revision,
            project,
            undo: Vec::new(),
            redo: Vec::new(),
        })
    }

    pub fn execute(&mut self, command: Command) -> Result<()> {
        if let Command::Transaction(commands) = &command {
            ensure(
                !commands
                    .iter()
                    .any(|command| matches!(command, Command::Transaction(_))),
                "Nested transactions are unsupported",
            )?;
        }
        if let Command::OffsetOutline {
            piece,
            width,
            outward,
        } = command
        {
            let prepared = self
                .project
                .prepare_outline_offset(piece, width, outward, || false)?;
            self.commit_outline_offset(prepared)?;
            return Ok(());
        }
        let offset_pieces: BTreeSet<_> = match &command {
            Command::Transaction(commands) => commands
                .iter()
                .filter_map(|command| {
                    if let Command::OffsetOutline { piece, .. } = command {
                        Some(*piece)
                    } else {
                        None
                    }
                })
                .collect(),
            _ => BTreeSet::new(),
        };
        let mut next = self.project.clone();
        let mut work = crate::geometry::ContainmentWork::new();
        next.apply(command, &mut work)?;
        next.validate_with_work(&mut work)?;
        if !next.sewing_groups.is_empty() {
            use std::collections::BTreeMap;
            let old_points: BTreeMap<_, _> = self
                .project
                .points
                .iter()
                .map(|point| (point.id, point))
                .collect();
            let changed_points: BTreeSet<_> = next
                .points
                .iter()
                .filter(|point| old_points.get(&point.id).copied() != Some(point))
                .map(|point| point.id)
                .collect();
            let old_curves: BTreeMap<_, _> = self
                .project
                .curves
                .iter()
                .map(|curve| (curve.id, curve))
                .collect();
            let changed_curves: BTreeSet<_> = next
                .curves
                .iter()
                .filter(|curve| {
                    old_curves.get(&curve.id).copied() != Some(curve)
                        || changed_points.contains(&curve.start)
                        || changed_points.contains(&curve.end)
                })
                .map(|curve| curve.id)
                .collect();
            let old_groups: BTreeMap<_, _> = self
                .project
                .sewing_groups
                .iter()
                .map(|group| (group.id, group))
                .collect();
            let affected: BTreeSet<_> = next
                .sewing_groups
                .iter()
                .filter(|group| {
                    old_groups.get(&group.id).copied() != Some(group)
                        || group
                            .a
                            .iter()
                            .chain(&group.b)
                            .any(|interval| changed_curves.contains(&interval.curve))
                })
                .map(|group| group.id)
                .collect();
            next.validate_sewing_metrics(Some(&affected))?;
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| Error("Document revision limit reached".into()))?;
        next.revision = revision;
        if !offset_pieces.is_empty() {
            // Later members can change meshing or physical settings after an offset was prepared.
            crate::offset::check_serialized_size(&next)?;
            let mesh = next.mesh()?;
            for piece in offset_pieces {
                ensure(
                    mesh.piece_ids.iter().filter(|id| **id == piece).count() <= 65_536,
                    &format!(
                        "OffsetDerivedVertexBudgetExceeded: final transaction piece {piece} exceeds 65536 vertices"
                    ),
                )?;
            }
            crate::simulation::create_cloth(&next, &mesh)?;
        }
        self.revision = revision;
        self.undo.push(std::mem::replace(&mut self.project, next));
        if self.undo.len() > 128 {
            self.undo.remove(0);
        }
        self.redo.clear();
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Publishes an already validated rest/solver candidate as one history action, without remeshing.
    pub fn commit_outline_offset(
        &mut self,
        prepared: crate::PreparedOutlineOffset,
    ) -> Result<(crate::Mesh, weftforge_sim::Cloth)> {
        crate::offset::recheck(&prepared, &self.project)?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| Error("Document revision limit reached".into()))?;
        ensure(
            prepared.candidate.revision == revision,
            "OffsetStalePreparation: revision changed",
        )?;
        self.revision = revision;
        self.undo
            .push(std::mem::replace(&mut self.project, prepared.candidate));
        if self.undo.len() > 128 {
            self.undo.remove(0);
        }
        self.redo.clear();
        Ok((prepared.mesh, prepared.cloth))
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        let Some(mut previous) = self.undo.pop() else {
            return false;
        };
        let Some(revision) = self.revision.checked_add(1) else {
            self.undo.push(previous);
            return false;
        };
        previous.revision = revision;
        self.revision = revision;
        // Never reuse an ID created on an abandoned history branch.
        previous.next_id = previous.next_id.max(self.project.next_id);
        self.redo
            .push(std::mem::replace(&mut self.project, previous));
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(mut next) = self.redo.pop() else {
            return false;
        };
        let Some(revision) = self.revision.checked_add(1) else {
            self.redo.push(next);
            return false;
        };
        next.revision = revision;
        self.revision = revision;
        next.next_id = next.next_id.max(self.project.next_id);
        self.undo.push(std::mem::replace(&mut self.project, next));
        true
    }
}
