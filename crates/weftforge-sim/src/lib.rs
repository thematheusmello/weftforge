//! Deterministic f64 CPU cloth in SI units. The authored document is owned by the caller.
mod capsule;
mod constraints;
mod contact;
mod math;
mod strain;

use constraints::{Bend, Membrane};
use math::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::time::Instant;

pub type Vec3 = [f64; 3];

const CANCELLATION_CHECK_INTERVAL: usize = 256;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Material {
    /// kg/m²; each triangle distributes its material-space mass equally to its nodes.
    pub areal_density: f64,
    /// Membrane stiffnesses in N/m; these are approximate, uncalibrated parameters.
    pub warp_stiffness: f64,
    pub weft_stiffness: f64,
    pub shear_stiffness: f64,
    /// Flexural rigidity in N m.
    pub bending_stiffness: f64,
    /// Radians counterclockwise from the material-space U axis.
    pub grain_angle: f64,
    /// Exponential velocity damping rate, s⁻¹.
    pub damping: f64,
    pub friction: f64,
    /// Offset from the cloth midsurface, metres; unrelated to rendered thickness.
    pub collision_thickness: f64,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            areal_density: 0.2,
            warp_stiffness: 800.,
            weft_stiffness: 600.,
            shear_stiffness: 120.,
            bending_stiffness: 0.0002,
            grain_angle: 0.,
            damping: 1.5,
            friction: 0.35,
            collision_thickness: 0.002,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WeightedPoint {
    pub vertices: [usize; 2],
    pub weights: [f64; 2],
}

impl WeightedPoint {
    pub fn vertex(vertex: usize) -> Self {
        Self {
            vertices: [vertex; 2],
            weights: [1., 0.],
        }
    }
    fn position(self, x: &[Vec3]) -> Vec3 {
        add(
            mul(x[self.vertices[0]], self.weights[0]),
            mul(x[self.vertices[1]], self.weights[1]),
        )
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Stitch {
    pub a: WeightedPoint,
    pub b: WeightedPoint,
    /// m/N; zero creates a hard coincidence constraint.
    pub compliance: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Pin {
    pub vertex: usize,
    pub target: Vec3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Collider {
    Ground {
        height: f64,
    },
    /// Original analytic ellipsoid. Radii are in metres along world X/Y/Z.
    Torso {
        center: Vec3,
        radii: Vec3,
    },
    Capsule {
        a: Vec3,
        b: Vec3,
        radius: f64,
    },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClothInput {
    pub positions: Vec<Vec3>,
    pub triangles: Vec<[usize; 3]>,
    /// Pattern/material coordinates in metres, separate from arranged 3D positions.
    pub material_uv: Vec<[f64; 2]>,
    /// Exactly one physical material per triangle.
    pub triangle_materials: Vec<Material>,
    pub stitches: Vec<Stitch>,
    pub pins: Vec<Pin>,
    pub colliders: Vec<Collider>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct StepSettings {
    /// Duration of the complete call in seconds, subdivided into `substeps`.
    pub dt: f64,
    pub substeps: u32,
    pub iterations: u32,
    pub gravity: Vec3,
    /// Sewing assembly actuator speed in m/s; zero keeps the current target separation.
    pub seam_closure_speed: f64,
}
impl Default for StepSettings {
    fn default() -> Self {
        Self {
            dt: 1. / 60.,
            substeps: 4,
            iterations: 8,
            gravity: [0., -9.81, 0.],
            seam_closure_speed: 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialAxis {
    Warp,
    Weft,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrainPeak {
    /// Index into the original `ClothInput::triangles`, in input order.
    pub triangle: usize,
    pub axis: MaterialAxis,
    /// Material-axis length minus one: positive extension, negative compression.
    pub signed_strain: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Diagnostics {
    pub simulation_time: f64,
    pub step_seconds: f64,
    /// Maximum absolute warp/weft length change, dimensionless; not a principal-strain bound.
    pub max_stretch_error: f64,
    /// Largest absolute strain; ties choose the first input triangle, then warp.
    #[serde(default)]
    pub peak_strain: Option<StrainPeak>,
    pub max_seam_error: f64,
    /// Largest temporary stitch target distance while sewing assembles, metres.
    pub seam_assembly_remaining: f64,
    /// Current maximum supported penetration, including full static triangle/capsule occupancy.
    pub max_contact_error: f64,
    /// Current triangle/capsule pairs evaluated with a conservative uncertain-geometry distance.
    #[serde(default)]
    pub capsule_surface_uncertain_pairs: usize,
    pub kinetic_energy: f64,
    /// Number of contacts projected during this call, including repeated iterations.
    pub contacts: usize,
    pub initial_intersections: usize,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimulationError {
    InvalidInput(String),
    InvalidSettings(String),
    Cancelled,
    Unstable(String),
}
impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(s) => write!(f, "invalid cloth input: {s}"),
            Self::InvalidSettings(s) => write!(f, "invalid simulation settings: {s}"),
            Self::Cancelled => f.write_str("simulation cancelled; previous state retained"),
            Self::Unstable(s) => write!(f, "simulation unstable; previous state retained: {s}"),
        }
    }
}
impl std::error::Error for SimulationError {}

#[derive(Clone, Debug)]
pub struct Cloth {
    input: ClothInput,
    positions: Vec<Vec3>,
    velocities: Vec<Vec3>,
    masses: Vec<f64>,
    inverse_masses: Vec<f64>,
    thickness: Vec<f64>,
    damping: Vec<f64>,
    friction: Vec<f64>,
    membrane: Vec<Membrane>,
    bends: Vec<Bend>,
    excluded: Vec<BTreeSet<usize>>,
    seam_remaining: Vec<f64>,
    diagnostics: Diagnostics,
}

impl Cloth {
    pub fn new(input: ClothInput) -> Result<Self, SimulationError> {
        Self::new_cancellable(input, || false)
    }

    /// Builds a complete state or returns cancellation; no partially built cloth is published.
    /// Calls the control callback before each geometry record, task or candidate predicate.
    pub fn new_cancellable(
        input: ClothInput,
        cancelled: impl Fn() -> bool,
    ) -> Result<Self, SimulationError> {
        check_cancel(&cancelled)?;
        validate(&input, &cancelled)?;
        let n = input.positions.len();
        let mut positions = Vec::with_capacity(n);
        let mut velocities = Vec::with_capacity(n);
        let mut masses = Vec::with_capacity(n);
        let mut thickness = Vec::<f64>::with_capacity(n);
        let mut damping = Vec::with_capacity(n);
        let mut friction = Vec::with_capacity(n);
        let mut membrane = Vec::with_capacity(input.triangles.len());
        let mut excluded = Vec::with_capacity(n);
        for &position in &input.positions {
            check_cancel(&cancelled)?;
            positions.push(position);
            velocities.push([0.; 3]);
            masses.push(0.);
            thickness.push(0.);
            damping.push(0.);
            friction.push(0.);
            excluded.push(BTreeSet::new());
        }
        let mut edge_faces: BTreeMap<[usize; 2], Vec<(usize, usize)>> = BTreeMap::new();
        for (ti, (&tri, &material)) in input
            .triangles
            .iter()
            .zip(&input.triangle_materials)
            .enumerate()
        {
            check_cancel(&cancelled)?;
            let constraint = Membrane::new(tri, &input.material_uv, material);
            if !constraint.valid_scale() {
                return Err(invalid("non-finite derived membrane coefficients"));
            }
            let mass = constraint.area * material.areal_density / 3.;
            for &v in &tri {
                check_cancel(&cancelled)?;
                masses[v] += mass;
                thickness[v] = thickness[v].max(material.collision_thickness);
                damping[v] += mass * material.damping;
                friction[v] += mass * material.friction;
                for neighbor in tri {
                    check_cancel(&cancelled)?;
                    excluded[v].insert(neighbor);
                }
            }
            for j in 0..3 {
                check_cancel(&cancelled)?;
                let mut edge = [tri[j], tri[(j + 1) % 3]];
                edge.sort_unstable();
                edge_faces
                    .entry(edge)
                    .or_default()
                    .push((ti, tri[(j + 2) % 3]));
            }
            membrane.push(constraint);
        }
        let mut bends = Vec::new();
        for (edge, faces) in edge_faces {
            check_cancel(&cancelled)?;
            if faces.len() > 2 {
                return Err(invalid("non-manifold triangle edge"));
            }
            if faces.len() == 2 {
                let (a, p) = faces[0];
                let (b, q) = faces[1];
                let rigidity = input.triangle_materials[a].bending_stiffness * 0.5
                    + input.triangle_materials[b].bending_stiffness * 0.5;
                if rigidity > 0. {
                    let bend = Bend::new([edge[0], edge[1], p, q], &input.positions, rigidity);
                    if !bend.valid_scale() {
                        return Err(invalid("non-finite derived bending coefficient"));
                    }
                    bends.push(bend);
                }
            }
        }
        for stitch in &input.stitches {
            check_cancel(&cancelled)?;
            // Sewing neighborhoods are one surface after closure, so thickness must not oppose the seam.
            for a in stitch.a.vertices {
                for b in stitch.b.vertices {
                    check_cancel(&cancelled)?;
                    excluded[a].insert(b);
                    excluded[b].insert(a);
                }
            }
        }
        let mut inverse_masses = Vec::with_capacity(n);
        for i in 0..n {
            check_cancel(&cancelled)?;
            if !masses[i].is_finite() || masses[i] <= 1e-16 {
                return Err(invalid(
                    "every vertex must have positive incident material area and mass",
                ));
            }
            damping[i] /= masses[i];
            friction[i] /= masses[i];
            if !damping[i].is_finite()
                || !friction[i].is_finite()
                || !(2. * thickness[i]).is_finite()
            {
                return Err(invalid(
                    "non-finite derived damping, friction or contact offset",
                ));
            }
            inverse_masses.push(masses[i].recip());
        }
        for pin in &input.pins {
            check_cancel(&cancelled)?;
            inverse_masses[pin.vertex] = 0.;
        }
        let seam_remaining = input
            .stitches
            .iter()
            .map(|s| {
                check_cancel(&cancelled)?;
                Ok(length(sub(
                    s.a.position(&input.positions),
                    s.b.position(&input.positions),
                )))
            })
            .collect::<Result<_, SimulationError>>()?;
        let mut cloth = Self {
            positions,
            velocities,
            masses,
            inverse_masses,
            thickness,
            damping,
            friction,
            membrane,
            bends,
            excluded,
            seam_remaining,
            input,
            diagnostics: Diagnostics {
                finite: true,
                ..Default::default()
            },
        };
        let contact_error = cloth.contact_errors_cancellable(&cloth.positions, &cancelled)?;
        let intersections = contact_error.intersections;
        cloth.diagnostics = cloth.measure_errors_with_contact_cancellable(
            &cloth.positions,
            &cloth.velocities,
            contact_error,
            &cancelled,
        )?;
        cloth.diagnostics.initial_intersections = intersections;
        if !cloth.diagnostics.finite {
            return Err(invalid(
                "non-finite derived mass, strain or contact geometry",
            ));
        }
        check_cancel(&cancelled)?;
        Ok(cloth)
    }

    pub fn positions(&self) -> &[Vec3] {
        &self.positions
    }
    pub fn velocities(&self) -> &[Vec3] {
        &self.velocities
    }
    pub fn masses(&self) -> &[f64] {
        &self.masses
    }
    pub fn stitches(&self) -> &[Stitch] {
        &self.input.stitches
    }
    pub fn simulation_time(&self) -> f64 {
        self.diagnostics.simulation_time
    }
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    pub fn reset(&mut self) {
        self.positions.clone_from(&self.input.positions);
        self.velocities.fill([0.; 3]);
        self.seam_remaining = self
            .input
            .stitches
            .iter()
            .map(|s| {
                length(sub(
                    s.a.position(&self.positions),
                    s.b.position(&self.positions),
                ))
            })
            .collect();
        self.diagnostics = Diagnostics {
            finite: true,
            ..Default::default()
        };
        let contact_error = self.contact_errors(&self.positions);
        self.diagnostics.initial_intersections = contact_error.intersections;
        self.diagnostics =
            self.measure_errors_with_contact(&self.positions, &self.velocities, contact_error);
    }

    /// Transactional whole-call stepping: cancellation, invalid settings and numerical failures
    /// retain the previous positions, velocities, time and diagnostics.
    pub fn step(
        &mut self,
        settings: StepSettings,
        cancelled: impl Fn() -> bool,
    ) -> Result<Diagnostics, SimulationError> {
        if !settings.dt.is_finite()
            || settings.dt <= 0.
            || settings.dt > 1.
            || settings.substeps == 0
            || settings.substeps > 1000
            || settings.iterations == 0
            || settings.iterations > 1000
            || !finite(settings.gravity)
            || !settings.seam_closure_speed.is_finite()
            || settings.seam_closure_speed < 0.
        {
            return Err(SimulationError::InvalidSettings(
                "require 0 < dt ≤ 1 s, 1..=1000 substeps/iterations, finite gravity and nonnegative finite sewing speed".into(),
            ));
        }
        if cancelled() {
            return Err(SimulationError::Cancelled);
        }
        let start = Instant::now();
        let mut x = self.positions.clone();
        let mut velocities = self.velocities.clone();
        let mut seam_remaining = self.seam_remaining.clone();
        let dt = settings.dt / f64::from(settings.substeps);
        let mut contacts = 0;
        for _ in 0..settings.substeps {
            if cancelled() {
                return Err(SimulationError::Cancelled);
            }
            let previous = x.clone();
            for i in 0..x.len() {
                if self.inverse_masses[i] > 0. {
                    velocities[i] = mul(
                        add(velocities[i], mul(settings.gravity, dt)),
                        (-self.damping[i] * dt).exp(),
                    );
                    x[i] = add(x[i], mul(velocities[i], dt));
                }
            }
            self.apply_pins(&mut x);
            for remaining in &mut seam_remaining {
                *remaining -= (settings.seam_closure_speed * dt).min(*remaining);
            }
            let mut material_corrections = vec![[0.; 3]; x.len()];
            let mut bend_corrections = vec![[0.; 3]; x.len()];
            let mut seam_lambda = vec![[0.; 3]; self.input.stitches.len()];
            for _ in 0..settings.iterations {
                if cancelled() {
                    return Err(SimulationError::Cancelled);
                }
                constraints::solve_material(
                    &self.membrane,
                    &mut x,
                    &self.inverse_masses,
                    dt,
                    &mut material_corrections,
                    &cancelled,
                )?;
                constraints::solve_bending(
                    &self.bends,
                    &mut x,
                    &self.inverse_masses,
                    dt,
                    &mut bend_corrections,
                    &cancelled,
                )?;
                for ((s, &remaining), lambda) in self
                    .input
                    .stitches
                    .iter()
                    .zip(&seam_remaining)
                    .zip(&mut seam_lambda)
                {
                    constraints::solve_stitch(
                        *s,
                        remaining,
                        &mut x,
                        &self.inverse_masses,
                        dt,
                        lambda,
                    );
                }
                contacts += self.project_colliders(&mut x, &previous, &cancelled)?;
                contacts += self.project_cloth(&mut x, &previous, &cancelled)?;
                self.apply_pins(&mut x);
                if !x.iter().copied().all(finite) {
                    return Err(SimulationError::Unstable("non-finite position".into()));
                }
            }
            for i in 0..x.len() {
                velocities[i] = if self.inverse_masses[i] > 0. {
                    mul(sub(x[i], previous[i]), dt.recip())
                } else {
                    [0.; 3]
                };
                if !finite(velocities[i]) || length(sub(x[i], previous[i])) > 100. {
                    return Err(SimulationError::Unstable(
                        "non-finite velocity or displacement exceeds 100 m in one substep".into(),
                    ));
                }
            }
        }
        if cancelled() {
            return Err(SimulationError::Cancelled);
        }
        let contact_error = self.contact_errors_cancellable(&x, &cancelled)?;
        let mut diagnostics = self.measure_errors_with_contact(&x, &velocities, contact_error);
        if !diagnostics.finite || diagnostics.max_stretch_error > 5. {
            return Err(SimulationError::Unstable(format!(
                "finite={}, material-axis extension={:.1}% (limit 500%), seam gap={:.6}m, contact penetration={:.6}m",
                diagnostics.finite,
                100. * diagnostics.max_stretch_error,
                diagnostics.max_seam_error,
                diagnostics.max_contact_error
            )));
        }
        if cancelled() {
            return Err(SimulationError::Cancelled);
        }
        diagnostics.simulation_time += settings.dt;
        diagnostics.contacts = contacts;
        diagnostics.seam_assembly_remaining = seam_remaining.iter().copied().fold(0., f64::max);
        diagnostics.step_seconds = start.elapsed().as_secs_f64();
        self.positions = x;
        self.velocities = velocities;
        self.seam_remaining = seam_remaining;
        self.diagnostics = diagnostics;
        Ok(self.diagnostics.clone())
    }

    fn apply_pins(&self, x: &mut [Vec3]) {
        for pin in &self.input.pins {
            x[pin.vertex] = pin.target;
        }
    }
    fn measure_errors_with_contact(
        &self,
        positions: &[Vec3],
        velocities: &[Vec3],
        contact_error: contact::ContactErrors,
    ) -> Diagnostics {
        self.measure_errors_with_contact_cancellable(positions, velocities, contact_error, &|| {
            false
        })
        .expect("A non-cancelling callback cannot interrupt diagnostics")
    }

    fn measure_errors_with_contact_cancellable(
        &self,
        positions: &[Vec3],
        velocities: &[Vec3],
        contact_error: contact::ContactErrors,
        cancelled: &impl Fn() -> bool,
    ) -> Result<Diagnostics, SimulationError> {
        let mut diagnostics = self.diagnostics.clone();
        diagnostics.seam_assembly_remaining = 0.;
        for &remaining in &self.seam_remaining {
            check_cancel(cancelled)?;
            diagnostics.seam_assembly_remaining =
                diagnostics.seam_assembly_remaining.max(remaining);
        }
        diagnostics.max_stretch_error = 0.;
        diagnostics.peak_strain = None;
        let mut finite_strains = true;
        for (triangle, &vertices) in self.input.triangles.iter().enumerate() {
            check_cancel(cancelled)?;
            let [warp, weft] = strain::signed_strains(
                vertices.map(|v| positions[v]),
                vertices.map(|v| self.input.material_uv[v]),
                self.input.triangle_materials[triangle].grain_angle,
            );
            finite_strains &= warp.is_finite() && weft.is_finite();
            for (axis, signed_strain) in [(MaterialAxis::Warp, warp), (MaterialAxis::Weft, weft)] {
                if diagnostics
                    .peak_strain
                    .is_none_or(|peak| signed_strain.abs() > peak.signed_strain.abs())
                {
                    diagnostics.max_stretch_error = signed_strain.abs();
                    diagnostics.peak_strain = Some(StrainPeak {
                        triangle,
                        axis,
                        signed_strain,
                    });
                }
            }
        }
        diagnostics.max_seam_error = 0.;
        for s in &self.input.stitches {
            check_cancel(cancelled)?;
            diagnostics.max_seam_error = diagnostics.max_seam_error.max(length(sub(
                s.a.position(positions),
                s.b.position(positions),
            )));
        }
        diagnostics.max_contact_error = contact_error.maximum;
        diagnostics.capsule_surface_uncertain_pairs = contact_error.uncertain_capsule_pairs;
        diagnostics.kinetic_energy =
            self.masses
                .iter()
                .zip(velocities)
                .try_fold(0., |energy, (m, v)| {
                    check_cancel(cancelled)?;
                    Ok::<_, SimulationError>(energy + 0.5 * m * dot(*v, *v))
                })?;
        let mut finite_state = true;
        if finite_strains {
            for &position in positions.iter().chain(velocities) {
                check_cancel(cancelled)?;
                if !finite(position) {
                    finite_state = false;
                    break;
                }
            }
        }
        diagnostics.finite = finite_strains
            && finite_state
            && diagnostics
                .peak_strain
                .is_none_or(|peak| peak.signed_strain.is_finite())
            && [
                diagnostics.max_stretch_error,
                diagnostics.max_seam_error,
                diagnostics.max_contact_error,
                diagnostics.kinetic_energy,
            ]
            .into_iter()
            .all(f64::is_finite);
        Ok(diagnostics)
    }
}

fn invalid(message: &str) -> SimulationError {
    SimulationError::InvalidInput(message.into())
}
fn check_cancel(cancelled: &impl Fn() -> bool) -> Result<(), SimulationError> {
    if cancelled() {
        Err(SimulationError::Cancelled)
    } else {
        Ok(())
    }
}
fn validate(input: &ClothInput, cancelled: &impl Fn() -> bool) -> Result<(), SimulationError> {
    let n = input.positions.len();
    if n < 3
        || input.triangles.is_empty()
        || input.material_uv.len() != n
        || input.triangle_materials.len() != input.triangles.len()
    {
        return Err(invalid(
            "require vertices, material coordinates and one material per triangle",
        ));
    }
    for (&position, uv) in input.positions.iter().zip(&input.material_uv) {
        check_cancel(cancelled)?;
        if !finite(position) || !uv.iter().all(|v| v.is_finite()) {
            return Err(invalid("non-finite rest/material coordinates"));
        }
    }
    let mut unique_triangles = BTreeSet::new();
    for (tri, mat) in input.triangles.iter().zip(&input.triangle_materials) {
        check_cancel(cancelled)?;
        if tri.iter().any(|&i| i >= n) || tri[0] == tri[1] || tri[1] == tri[2] || tri[2] == tri[0] {
            return Err(invalid("triangle contains invalid/repeated vertex index"));
        }
        let mut sorted = *tri;
        sorted.sort_unstable();
        if !unique_triangles.insert(sorted) {
            return Err(invalid("duplicate triangle"));
        }
        let a = input.material_uv[tri[0]];
        let b = input.material_uv[tri[1]];
        let c = input.material_uv[tri[2]];
        let area2 = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        let rest_area2 = length(cross(
            sub(input.positions[tri[1]], input.positions[tri[0]]),
            sub(input.positions[tri[2]], input.positions[tri[0]]),
        ));
        if !area2.is_finite()
            || area2.abs() < 1e-12
            || !rest_area2.is_finite()
            || rest_area2 < 1e-12
        {
            return Err(invalid("degenerate rest or material-space triangle"));
        }
        if ![
            mat.areal_density,
            mat.warp_stiffness,
            mat.weft_stiffness,
            mat.shear_stiffness,
            mat.bending_stiffness,
            mat.grain_angle,
            mat.damping,
            mat.friction,
            mat.collision_thickness,
        ]
        .into_iter()
        .all(f64::is_finite)
            || mat.areal_density <= 0.
            || mat.warp_stiffness <= 0.
            || mat.weft_stiffness <= 0.
            || mat.shear_stiffness <= 0.
            || mat.bending_stiffness < 0.
            || mat.damping < 0.
            || mat.friction < 0.
            || mat.collision_thickness <= 0.
        {
            return Err(invalid(
                "require finite positive density/membrane stiffness/thickness and nonnegative bending/damping/friction",
            ));
        }
    }
    for stitch in &input.stitches {
        check_cancel(cancelled)?;
        if !stitch.compliance.is_finite() || stitch.compliance < 0. {
            return Err(invalid("invalid stitch compliance"));
        }
        for p in [stitch.a, stitch.b] {
            if p.vertices.into_iter().any(|v| v >= n)
                || p.weights
                    .into_iter()
                    .any(|w| !w.is_finite() || !(0. ..=1.).contains(&w))
                || (p.weights[0] + p.weights[1] - 1.).abs() > 1e-10
            {
                return Err(invalid(
                    "weighted seam point requires valid indices and convex weights summing to one",
                ));
            }
        }
    }
    let mut pinned = BTreeSet::new();
    for pin in &input.pins {
        check_cancel(cancelled)?;
        if pin.vertex >= n || !finite(pin.target) || !pinned.insert(pin.vertex) {
            return Err(invalid("invalid or duplicate pin"));
        }
    }
    for collider in &input.colliders {
        check_cancel(cancelled)?;
        let valid = match *collider {
            Collider::Ground { height } => height.is_finite(),
            Collider::Torso { center, radii } => {
                finite(center) && radii.into_iter().all(|v| v.is_finite() && v > 0.)
            }
            Collider::Capsule { a, b, radius } => {
                finite(a) && finite(b) && radius.is_finite() && radius > 0.
            }
        };
        if !valid {
            return Err(invalid("invalid analytic collider"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod constructor_control_tests {
    use super::*;
    use std::cell::Cell;

    fn squares(patches: usize) -> ClothInput {
        let mut input = ClothInput::default();
        for patch in 0..patches {
            let offset = input.positions.len();
            let x = 3. * patch as f64;
            input
                .positions
                .extend([[x, 1., 0.], [x + 1., 1., 0.], [x + 1., 1., 1.], [x, 1., 1.]]);
            input
                .material_uv
                .extend([[0., 0.], [1., 0.], [1., 1.], [0., 1.]]);
            input.triangles.extend([
                [offset, offset + 1, offset + 2],
                [offset, offset + 2, offset + 3],
            ]);
            input.triangle_materials.extend([Material::default(); 2]);
            input.stitches.push(Stitch {
                a: WeightedPoint::vertex(offset),
                b: WeightedPoint::vertex(offset + 1),
                compliance: 0.,
            });
            input.pins.push(Pin {
                vertex: offset,
                target: input.positions[offset],
            });
        }
        input.colliders = vec![
            Collider::Ground { height: 0. },
            Collider::Capsule {
                a: [-1e6, 0., 0.],
                b: [-1e6, 1., 0.],
                radius: 0.1,
            },
        ];
        input
    }

    #[test]
    fn validation_charges_each_record_and_stops_exactly_at_caller_abort() {
        let input = squares(CANCELLATION_CHECK_INTERVAL + 1);
        let stages = [
            input.positions.len(),
            input.triangles.len(),
            input.stitches.len(),
            input.pins.len(),
            input.colliders.len(),
        ];
        let calls = Cell::new(0);
        validate(&input, &|| {
            calls.set(calls.get() + 1);
            false
        })
        .unwrap();
        assert_eq!(calls.get(), stages.iter().sum::<usize>());
        let mut start = 0;
        for stage in stages {
            let stop_at = start + stage.min(CANCELLATION_CHECK_INTERVAL + 1);
            let calls = Cell::new(0);
            assert_eq!(
                validate(&input, &|| {
                    calls.set(calls.get() + 1);
                    assert!(calls.get() <= stop_at);
                    calls.get() == stop_at
                }),
                Err(SimulationError::Cancelled)
            );
            assert_eq!(calls.get(), stop_at);
            start += stage;
        }
    }

    #[test]
    fn diagnostics_charges_each_scan_and_preserves_numeric_results() {
        let cloth = Cloth::new(squares(CANCELLATION_CHECK_INTERVAL + 1)).unwrap();
        let velocities: Vec<Vec3> = (0..cloth.positions.len())
            .map(|i| [i as f64 / 7., (i % 5) as f64 / 3., -(i as f64) / 13.])
            .collect();
        let stages = [
            cloth.seam_remaining.len(),
            cloth.input.triangles.len(),
            cloth.input.stitches.len(),
            cloth.masses.len(),
            cloth.positions.len(),
            velocities.len(),
        ];
        let calls = Cell::new(0);
        let diagnostics = cloth
            .measure_errors_with_contact_cancellable(
                &cloth.positions,
                &velocities,
                contact::ContactErrors::default(),
                &|| {
                    calls.set(calls.get() + 1);
                    false
                },
            )
            .unwrap();
        assert_eq!(calls.get(), stages.iter().sum::<usize>());
        let energy: f64 = cloth
            .masses
            .iter()
            .zip(&velocities)
            .map(|(m, v)| 0.5 * m * dot(*v, *v))
            .sum();
        assert_eq!(diagnostics.kinetic_energy.to_bits(), energy.to_bits());
        assert_eq!(diagnostics.max_stretch_error, 0.);
        assert_eq!(diagnostics.max_seam_error, 1.);
        assert_eq!(diagnostics.seam_assembly_remaining, 1.);
        assert!(diagnostics.finite);
        let mut start = 0;
        for stage in stages {
            let stop_at = start + CANCELLATION_CHECK_INTERVAL + 1;
            let calls = Cell::new(0);
            let result = cloth.measure_errors_with_contact_cancellable(
                &cloth.positions,
                &velocities,
                contact::ContactErrors::default(),
                &|| {
                    calls.set(calls.get() + 1);
                    assert!(calls.get() <= stop_at);
                    calls.get() == stop_at
                },
            );
            assert!(matches!(result, Err(SimulationError::Cancelled)));
            assert_eq!(calls.get(), stop_at);
            start += stage;
        }
    }

    #[test]
    fn constructor_charges_initialization_and_never_publishes_after_abort() {
        let patches = CANCELLATION_CHECK_INTERVAL + 1;
        let input = squares(patches);
        let total = Cell::new(0);
        let complete = Cloth::new_cancellable(input.clone(), || {
            total.set(total.get() + 1);
            false
        })
        .unwrap();
        let ordinary = Cloth::new(input.clone()).unwrap();
        assert_eq!(format!("{complete:?}"), format!("{ordinary:?}"));
        assert_eq!(complete.positions(), input.positions);
        assert_eq!(complete.membrane.len(), 2 * patches);
        assert_eq!(complete.bends.len(), patches);
        assert_eq!(complete.diagnostics.initial_intersections, 0);
        assert_eq!(complete.diagnostics.max_contact_error, 0.);
        assert!((complete.masses.iter().sum::<f64>() - 0.2 * patches as f64).abs() < 1e-12);
        let contacts = Cell::new(0);
        let errors = complete
            .contact_errors_cancellable(&complete.positions, &|| {
                contacts.set(contacts.get() + 1);
                false
            })
            .unwrap();
        let diagnostics = Cell::new(0);
        complete
            .measure_errors_with_contact_cancellable(
                &complete.positions,
                &complete.velocities,
                errors,
                &|| {
                    diagnostics.set(diagnostics.get() + 1);
                    false
                },
            )
            .unwrap();
        let validation = input.positions.len()
            + input.triangles.len()
            + input.stitches.len()
            + input.pins.len()
            + input.colliders.len();
        let initialization = 2 * input.positions.len()
            + 16 * input.triangles.len()
            + 5 * patches
            + 6 * input.stitches.len()
            + input.pins.len();
        let contact_start = 1 + validation + initialization;
        assert_eq!(
            total.get(),
            contact_start + contacts.get() + diagnostics.get() + 1
        );
        let mut accepted = Cloth::new(squares(1)).unwrap();
        let previous = format!("{accepted:?}");
        for stop_at in [
            1,
            CANCELLATION_CHECK_INTERVAL + 1,
            1 + validation + CANCELLATION_CHECK_INTERVAL + 1,
            // First inverse-mass poll, after all five edges per square.
            1 + validation + input.positions.len() + 16 * input.triangles.len() + 5 * patches + 1,
            contact_start,
            contact_start + CANCELLATION_CHECK_INTERVAL + 1,
            contact_start + contacts.get() / 2,
            contact_start + contacts.get() + CANCELLATION_CHECK_INTERVAL + 1,
            total.get(),
        ] {
            let calls = Cell::new(0);
            let result = Cloth::new_cancellable(input.clone(), || {
                calls.set(calls.get() + 1);
                assert!(
                    calls.get() <= stop_at,
                    "continued work after abort at {stop_at}"
                );
                calls.get() == stop_at
            })
            .map(|next| accepted = next);
            assert_eq!(result, Err(SimulationError::Cancelled));
            assert_eq!(calls.get(), stop_at);
            assert_eq!(format!("{accepted:?}"), previous);
        }
    }
}
