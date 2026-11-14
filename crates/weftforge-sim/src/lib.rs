//! Deterministic f64 CPU cloth in SI units. The authored document is owned by the caller.
mod constraints;
mod contact;
mod math;

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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Diagnostics {
    pub simulation_time: f64,
    pub step_seconds: f64,
    /// Maximum absolute principal material-axis length change, dimensionless.
    pub max_stretch_error: f64,
    pub max_seam_error: f64,
    /// Largest temporary stitch target distance while sewing assembles, metres.
    pub seam_assembly_remaining: f64,
    pub max_contact_error: f64,
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
    pub fn new_cancellable(
        input: ClothInput,
        cancelled: impl Fn() -> bool,
    ) -> Result<Self, SimulationError> {
        check_cancel(&cancelled)?;
        validate(&input, &cancelled)?;
        let n = input.positions.len();
        let mut masses = vec![0.; n];
        let mut thickness = vec![0_f64; n];
        let mut damping = vec![0.; n];
        let mut friction = vec![0.; n];
        let mut membrane = Vec::with_capacity(input.triangles.len());
        let mut excluded = vec![BTreeSet::new(); n];
        let mut edge_faces: BTreeMap<[usize; 2], Vec<(usize, usize)>> = BTreeMap::new();
        for (ti, (&tri, &material)) in input
            .triangles
            .iter()
            .zip(&input.triangle_materials)
            .enumerate()
        {
            if ti % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(&cancelled)?;
            }
            let constraint = Membrane::new(tri, &input.material_uv, material);
            if !constraint.valid_scale() {
                return Err(invalid("non-finite derived membrane coefficients"));
            }
            let mass = constraint.area * material.areal_density / 3.;
            for &v in &tri {
                masses[v] += mass;
                thickness[v] = thickness[v].max(material.collision_thickness);
                damping[v] += mass * material.damping;
                friction[v] += mass * material.friction;
                excluded[v].extend(tri);
            }
            for j in 0..3 {
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
        for (i, (edge, faces)) in edge_faces.into_iter().enumerate() {
            if i % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(&cancelled)?;
            }
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
        for (i, stitch) in input.stitches.iter().enumerate() {
            if i % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(&cancelled)?;
            }
            // Sewing neighborhoods are one surface after closure, so thickness must not oppose the seam.
            for a in stitch.a.vertices {
                for b in stitch.b.vertices {
                    excluded[a].insert(b);
                    excluded[b].insert(a);
                }
            }
        }
        let mut inverse_masses = Vec::with_capacity(n);
        for i in 0..n {
            if i % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(&cancelled)?;
            }
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
        for (i, pin) in input.pins.iter().enumerate() {
            if i % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(&cancelled)?;
            }
            inverse_masses[pin.vertex] = 0.;
        }
        let seam_remaining = input
            .stitches
            .iter()
            .enumerate()
            .map(|(i, s)| {
                if i % CANCELLATION_CHECK_INTERVAL == 0 {
                    check_cancel(&cancelled)?;
                }
                Ok(length(sub(
                    s.a.position(&input.positions),
                    s.b.position(&input.positions),
                )))
            })
            .collect::<Result<_, SimulationError>>()?;
        let mut cloth = Self {
            positions: input.positions.clone(),
            velocities: vec![[0.; 3]; n],
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
        let (contact_error, intersections) =
            cloth.contact_errors_cancellable(&cloth.positions, &cancelled)?;
        cloth.diagnostics =
            cloth.measure_errors_with_contact(&cloth.positions, &cloth.velocities, contact_error);
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
        self.diagnostics.initial_intersections = self.contact_errors(&self.positions).1;
        self.update_errors();
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
            let mut membrane_lambda = vec![[0.; 3]; self.membrane.len()];
            let mut bend_lambda = vec![0.; self.bends.len()];
            let mut seam_lambda = vec![0.; self.input.stitches.len()];
            for _ in 0..settings.iterations {
                if cancelled() {
                    return Err(SimulationError::Cancelled);
                }
                for (m, lambda) in self.membrane.iter().zip(&mut membrane_lambda) {
                    m.solve(&mut x, &self.inverse_masses, dt, lambda);
                }
                for (b, lambda) in self.bends.iter().zip(&mut bend_lambda) {
                    b.solve(&mut x, &self.inverse_masses, dt, lambda);
                }
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
                contacts += self.project_colliders(&mut x, &previous);
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
        let mut diagnostics = self.measure_errors(&x, &velocities);
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
    fn update_errors(&mut self) {
        self.diagnostics = self.measure_errors(&self.positions, &self.velocities);
    }
    fn measure_errors(&self, positions: &[Vec3], velocities: &[Vec3]) -> Diagnostics {
        self.measure_errors_with_contact(positions, velocities, self.contact_errors(positions).0)
    }
    fn measure_errors_with_contact(
        &self,
        positions: &[Vec3],
        velocities: &[Vec3],
        contact_error: f64,
    ) -> Diagnostics {
        let mut diagnostics = self.diagnostics.clone();
        diagnostics.seam_assembly_remaining =
            self.seam_remaining.iter().copied().fold(0., f64::max);
        diagnostics.max_stretch_error = self
            .membrane
            .iter()
            .map(|m| m.stretch_error(positions))
            .fold(0., f64::max);
        diagnostics.max_seam_error = self
            .input
            .stitches
            .iter()
            .map(|s| length(sub(s.a.position(positions), s.b.position(positions))))
            .fold(0., f64::max);
        diagnostics.max_contact_error = contact_error;
        diagnostics.kinetic_energy = self
            .masses
            .iter()
            .zip(velocities)
            .map(|(m, v)| 0.5 * m * dot(*v, *v))
            .sum();
        diagnostics.finite = positions.iter().chain(velocities).copied().all(finite)
            && [
                diagnostics.max_stretch_error,
                diagnostics.max_seam_error,
                diagnostics.max_contact_error,
                diagnostics.kinetic_energy,
            ]
            .into_iter()
            .all(f64::is_finite);
        diagnostics
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
    for (i, (&position, uv)) in input.positions.iter().zip(&input.material_uv).enumerate() {
        if i % CANCELLATION_CHECK_INTERVAL == 0 {
            check_cancel(cancelled)?;
        }
        if !finite(position) || !uv.iter().all(|v| v.is_finite()) {
            return Err(invalid("non-finite rest/material coordinates"));
        }
    }
    let mut unique_triangles = BTreeSet::new();
    for (i, (tri, mat)) in input
        .triangles
        .iter()
        .zip(&input.triangle_materials)
        .enumerate()
    {
        if i % CANCELLATION_CHECK_INTERVAL == 0 {
            check_cancel(cancelled)?;
        }
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
    for (i, stitch) in input.stitches.iter().enumerate() {
        if i % CANCELLATION_CHECK_INTERVAL == 0 {
            check_cancel(cancelled)?;
        }
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
    for (i, pin) in input.pins.iter().enumerate() {
        if i % CANCELLATION_CHECK_INTERVAL == 0 {
            check_cancel(cancelled)?;
        }
        if pin.vertex >= n || !finite(pin.target) || !pinned.insert(pin.vertex) {
            return Err(invalid("invalid or duplicate pin"));
        }
    }
    for (i, collider) in input.colliders.iter().enumerate() {
        if i % CANCELLATION_CHECK_INTERVAL == 0 {
            check_cancel(cancelled)?;
        }
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
