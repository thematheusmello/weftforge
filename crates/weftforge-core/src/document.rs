use crate::{Error, Id, Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_OBJECTS: usize = 20_000;
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Piece {
    pub id: Id,
    pub name: String,
    /// Closed, ordered chain. Curves are traversed start to end.
    pub boundary: Vec<Id>,
    pub placement: Placement,
    pub material: Id,
    /// Warp axis in material UV: radians counterclockwise from +U toward +V.
    pub grain_angle: f64,
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
        ensure(
            self.points.len()
                + self.curves.len()
                + self.pieces.len()
                + self.materials.len()
                + self.seams.len()
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
            .chain(self.materials.iter().map(|x| x.id))
            .chain(self.seams.iter().map(|x| x.id))
        {
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
            validate_coordinates(&point.position)?;
        }
        for curve in &self.curves {
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
            for (index, id) in piece.boundary.iter().enumerate() {
                ensure(
                    used_curves.insert(*id),
                    "Boundary curves cannot be shared between pieces",
                )?;
                let curve = self
                    .curve(*id)
                    .ok_or_else(|| Error("Missing boundary curve".into()))?;
                let next = self
                    .curve(piece.boundary[(index + 1) % piece.boundary.len()])
                    .ok_or_else(|| Error("Missing boundary curve".into()))?;
                ensure(
                    curve.end == next.start,
                    "Piece boundary is not a closed ordered chain",
                )?;
                local_points.insert(curve.start);
            }
            for id in local_points {
                ensure(
                    used_points.insert(id),
                    "Points cannot be shared between pieces",
                )?;
            }
            crate::geometry::validate_contour(self, piece)?;
        }
        ensure(
            used_curves.len() == self.curves.len() && used_points.len() == self.points.len(),
            "Orphan construction geometry is unsupported",
        )?;
        for seam in &self.seams {
            validate_name(&seam.name)?;
            for interval in [seam.a, seam.b] {
                ensure(
                    self.curve(interval.curve).is_some(),
                    "Seam curve is missing",
                )?;
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

    fn allocate_id(&mut self) -> Result<Id> {
        let id = self.next_id;
        self.next_id = id
            .checked_add(1)
            .ok_or_else(|| Error("Stable ID limit reached".into()))?;
        Ok(id)
    }

    fn apply(&mut self, command: Command) -> Result<()> {
        match command {
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
                self.apply(Command::AddPolygon {
                    name,
                    vertices: vec![
                        [x, y],
                        [x + width, y],
                        [x + width, y + height],
                        [x, y + height],
                    ],
                })?;
            }
            Command::MovePoint { point, position } => {
                self.points
                    .iter_mut()
                    .find(|item| item.id == point)
                    .ok_or_else(|| Error("Unknown point".into()))?
                    .position = position;
            }
            Command::SetCurveControls { curve, controls } => {
                self.curves
                    .iter_mut()
                    .find(|item| item.id == curve)
                    .ok_or_else(|| Error("Unknown curve".into()))?
                    .controls = controls;
            }
            Command::SplitCurve { curve, t } => {
                ensure(
                    t.is_finite() && t > 1.0e-6 && t < 1.0 - 1.0e-6,
                    "Split t must be strictly inside the curve",
                )?;
                ensure(
                    !self
                        .seams
                        .iter()
                        .any(|seam| seam.a.curve == curve || seam.b.curve == curve),
                    "Splitting a sewn curve is unsupported; remove its seams first",
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
                    .find(|piece| piece.boundary.contains(&curve))
                    .ok_or_else(|| Error("Curve has no piece".into()))?;
                let index = piece
                    .boundary
                    .iter()
                    .position(|&id| id == curve)
                    .ok_or_else(|| Error("Curve has no piece".into()))?;
                piece.boundary.insert(index + 1, second);
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
                let boundary = self
                    .piece(piece)
                    .ok_or_else(|| Error("Unknown piece".into()))?
                    .boundary
                    .clone();
                let mut point_ids = BTreeSet::new();
                for curve in self
                    .curves
                    .iter_mut()
                    .filter(|curve| boundary.contains(&curve.id))
                {
                    point_ids.insert(curve.start);
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
            Command::RemovePiece { piece } => {
                let boundary = self
                    .piece(piece)
                    .ok_or_else(|| Error("Unknown piece".into()))?
                    .boundary
                    .clone();
                let points: BTreeSet<Id> = self
                    .curves
                    .iter()
                    .filter(|curve| boundary.contains(&curve.id))
                    .map(|curve| curve.start)
                    .collect();
                self.pieces.retain(|item| item.id != piece);
                self.curves.retain(|item| !boundary.contains(&item.id));
                self.points.retain(|item| !points.contains(&item.id));
                self.seams.retain(|item| {
                    !boundary.contains(&item.a.curve) && !boundary.contains(&item.b.curve)
                });
            }
            Command::SetResolution { metres } => self.resolution = metres,
            Command::SetMannequin { mannequin } => self.mannequin = mannequin,
            Command::Transaction(commands) => {
                ensure(commands.len() <= 1024, "Transaction command limit exceeded")?;
                for command in commands {
                    ensure(
                        !matches!(command, Command::Transaction(_)),
                        "Nested transactions are unsupported",
                    )?;
                    self.apply(command)?;
                }
            }
        }
        Ok(())
    }
}

fn validate_name(name: &str) -> Result<()> {
    ensure(
        !name.trim().is_empty() && name.len() <= 256 && !name.chars().any(char::is_control),
        "Names must be nonempty, at most 256 bytes, with no control characters",
    )
}

fn validate_coordinates<const N: usize>(point: &[f64; N]) -> Result<()> {
    ensure(
        point
            .iter()
            .all(|x| x.is_finite() && x.abs() <= MAX_COORDINATE),
        "Coordinates must be finite and within 1000 metres",
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
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
    ResizePiece {
        piece: Id,
        width: f64,
        height: f64,
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
        Ok(Self {
            revision: project.revision,
            project,
            undo: Vec::new(),
            redo: Vec::new(),
        })
    }

    pub fn execute(&mut self, command: Command) -> Result<()> {
        let mut next = self.project.clone();
        next.apply(command)?;
        next.validate()?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| Error("Document revision limit reached".into()))?;
        next.revision = revision;
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
