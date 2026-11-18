use crate::{Material, SimulationError, Stitch, Vec3, check_cancel, math::*};

#[derive(Clone, Debug)]
pub(crate) struct Membrane {
    pub(crate) vertices: [usize; 3],
    warp: [f64; 3],
    weft: [f64; 3],
    stiffness: [f64; 3],
    pub area: f64,
}
impl Membrane {
    pub fn new(vertices: [usize; 3], uv: &[[f64; 2]], material: Material) -> Self {
        let a = uv[vertices[0]];
        let b = uv[vertices[1]];
        let c = uv[vertices[2]];
        let du1 = b[0] - a[0];
        let dv1 = b[1] - a[1];
        let du2 = c[0] - a[0];
        let dv2 = c[1] - a[1];
        let det = du1 * dv2 - du2 * dv1;
        let u = [(dv1 - dv2) / det, dv2 / det, -dv1 / det];
        let v = [(du2 - du1) / det, -du2 / det, du1 / det];
        let (sin, cos) = material.grain_angle.sin_cos();
        let area = det.abs() * 0.5;
        Self {
            vertices,
            warp: std::array::from_fn(|i| cos * u[i] + sin * v[i]),
            weft: std::array::from_fn(|i| -sin * u[i] + cos * v[i]),
            stiffness: [
                material.warp_stiffness * area,
                material.weft_stiffness * area,
                material.shear_stiffness * area,
            ],
            area,
        }
    }
    pub fn valid_scale(&self) -> bool {
        self.area.is_finite()
            && self.area > 0.
            && self.stiffness.iter().all(|s| s.is_finite() && *s > 0.)
            && self.warp.iter().chain(&self.weft).all(|q| q.is_finite())
    }
    fn deformation(&self, x: [Vec3; 3]) -> (Vec3, Vec3) {
        // Relative positions avoid cancellation when the garment is translated far from the origin.
        let b = sub(x[1], x[0]);
        let c = sub(x[2], x[0]);
        (
            add(mul(b, self.warp[1]), mul(c, self.warp[2])),
            add(mul(b, self.weft[1]), mul(c, self.weft[2])),
        )
    }
    fn values_gradients(&self, positions: [Vec3; 3]) -> ([f64; 3], [[Vec3; 3]; 3]) {
        let (w, f) = self.deformation(positions);
        (
            [0.5 * (dot(w, w) - 1.), 0.5 * (dot(f, f) - 1.), dot(w, f)],
            [
                self.warp.map(|q| mul(w, q)),
                self.weft.map(|q| mul(f, q)),
                std::array::from_fn(|i| add(mul(f, self.warp[i]), mul(w, self.weft[i]))),
            ],
        )
    }

    fn energy(&self, positions: [Vec3; 3], dt: f64) -> f64 {
        let (w, f) = self.deformation(positions);
        [0.5 * (dot(w, w) - 1.), 0.5 * (dot(f, f) - 1.), dot(w, f)]
            .into_iter()
            .zip(self.stiffness)
            .map(|(c, k)| {
                let residual = dt * k.sqrt() * c;
                0.5 * residual * residual
            })
            .sum()
    }
}

struct MaterialRow {
    vertices: [usize; 3],
    gradient: [Vec3; 3],
    value: f64,
}

const MATERIAL_LINEAR_ITERATIONS: usize = 32;
const MATERIAL_BACKTRACKING_STEPS: usize = 16;

pub(crate) fn solve_material(
    membrane: &[Membrane],
    x: &mut [Vec3],
    inverse_mass: &[f64],
    dt: f64,
    correction: &mut [Vec3],
    cancelled: &impl Fn() -> bool,
) -> Result<(), SimulationError> {
    let unstable = || SimulationError::Unstable("non-finite global material solve".into());
    let mut root_weight = Vec::with_capacity(x.len());
    let mut diagonal: Vec<Vec3> = vec![[1.; 3]; x.len()];
    for &weight in inverse_mass {
        check_cancel(cancelled)?;
        let root = weight.sqrt();
        if !root.is_finite() {
            return Err(unstable());
        }
        root_weight.push(root);
    }
    let mut rows = Vec::new();
    for triangle in membrane {
        let (values, gradients) = triangle.values_gradients(triangle.vertices.map(|v| x[v]));
        for mode in 0..3 {
            check_cancel(cancelled)?;
            let scale = dt * triangle.stiffness[mode].sqrt();
            let value = scale * values[mode];
            let gradient = std::array::from_fn(|i| {
                mul(
                    gradients[mode][i],
                    scale * root_weight[triangle.vertices[i]],
                )
            });
            if !value.is_finite() || !gradient.into_iter().all(finite) {
                return Err(unstable());
            }
            for (vertex, gradient) in triangle.vertices.into_iter().zip(gradient) {
                for (axis, component) in gradient.into_iter().enumerate() {
                    diagonal[vertex][axis] = diagonal[vertex][axis].hypot(component);
                    if !diagonal[vertex][axis].is_finite() {
                        return Err(unstable());
                    }
                }
            }
            rows.push(MaterialRow {
                vertices: triangle.vertices,
                gradient,
                value,
            });
        }
    }
    let mut rhs = vec![[0.; 3]; x.len()];
    for i in 0..x.len() {
        check_cancel(cancelled)?;
        if root_weight[i] > 0. {
            rhs[i] = std::array::from_fn(|axis| {
                -correction[i][axis] / root_weight[i] / diagonal[i][axis]
            });
        }
    }
    for row in &mut rows {
        check_cancel(cancelled)?;
        for (vertex, gradient) in row.vertices.into_iter().zip(&mut row.gradient) {
            for (axis, component) in gradient.iter_mut().enumerate() {
                *component /= diagonal[vertex][axis];
            }
            rhs[vertex] = sub(rhs[vertex], mul(*gradient, row.value));
        }
    }
    let energy = material_energy(membrane, x, correction, &root_weight, dt, cancelled)?;
    if !energy.is_finite() {
        return Err(unstable());
    }
    let mut rhs_scale = 0_f64;
    for value in &rhs {
        check_cancel(cancelled)?;
        if !finite(*value) {
            return Err(unstable());
        }
        for component in value {
            rhs_scale = rhs_scale.max(component.abs());
        }
    }
    if rhs_scale == 0. {
        return Ok(());
    }
    for value in &mut rhs {
        check_cancel(cancelled)?;
        for component in value {
            *component /= rhs_scale;
        }
    }
    let mut solution = vec![[0.; 3]; x.len()];
    let mut residual = rhs.clone();
    let mut search = residual.clone();
    let mut product = vec![[0.; 3]; x.len()];
    let mut squared_residual = material_dot(&residual, &residual, cancelled)?;
    let initial_squared_residual = squared_residual;
    // shortcut: at most 32 diagonal-PCG steps; reassess this cap with large-mesh residual evidence.
    for _ in 0..MATERIAL_LINEAR_ITERATIONS {
        check_cancel(cancelled)?;
        for i in 0..x.len() {
            check_cancel(cancelled)?;
            product[i] =
                std::array::from_fn(|axis| search[i][axis] / diagonal[i][axis] / diagonal[i][axis]);
        }
        for row in &rows {
            check_cancel(cancelled)?;
            let value = row
                .vertices
                .into_iter()
                .zip(row.gradient)
                .map(|(v, g)| dot(g, search[v]))
                .sum::<f64>();
            for (vertex, gradient) in row.vertices.into_iter().zip(row.gradient) {
                product[vertex] = add(product[vertex], mul(gradient, value));
            }
        }
        let curvature = material_dot(&search, &product, cancelled)?;
        if !curvature.is_finite() || curvature <= 0. {
            return Err(unstable());
        }
        let alpha = squared_residual / curvature;
        let mut next_squared_residual = 0.;
        for i in 0..x.len() {
            check_cancel(cancelled)?;
            solution[i] = add(solution[i], mul(search[i], alpha));
            residual[i] = sub(residual[i], mul(product[i], alpha));
            if !finite(solution[i]) || !finite(residual[i]) {
                return Err(unstable());
            }
            next_squared_residual += dot(residual[i], residual[i]);
        }
        if !next_squared_residual.is_finite() {
            return Err(unstable());
        }
        if next_squared_residual <= 1e-24 * initial_squared_residual {
            break;
        }
        let beta = next_squared_residual / squared_residual;
        for i in 0..x.len() {
            check_cancel(cancelled)?;
            search[i] = add(residual[i], mul(search[i], beta));
        }
        squared_residual = next_squared_residual;
    }
    let slope = -material_dot(&rhs, &solution, cancelled)? * rhs_scale * rhs_scale;
    if !slope.is_finite() {
        return Err(unstable());
    }
    if slope >= 0. {
        return Ok(());
    }
    for i in 0..x.len() {
        check_cancel(cancelled)?;
        for axis in 0..3 {
            solution[i][axis] *= rhs_scale * (root_weight[i] / diagonal[i][axis]);
        }
        if !finite(solution[i]) {
            return Err(unstable());
        }
    }
    let mut trial = x.to_vec();
    let mut trial_correction = correction.to_vec();
    let mut fraction = 1.;
    // shortcut: cap Armijo backtracking at 16; use a trust region if extreme compression stalls.
    for _ in 0..MATERIAL_BACKTRACKING_STEPS {
        check_cancel(cancelled)?;
        for i in 0..x.len() {
            check_cancel(cancelled)?;
            if root_weight[i] > 0. {
                trial[i] = add(x[i], mul(solution[i], fraction));
                trial_correction[i] = add(correction[i], sub(trial[i], x[i]));
            }
        }
        let trial_energy = material_energy(
            membrane,
            &trial,
            &trial_correction,
            &root_weight,
            dt,
            cancelled,
        )?;
        if trial_energy.is_finite() && trial_energy <= energy + 1e-4 * fraction * slope {
            x.copy_from_slice(&trial);
            correction.copy_from_slice(&trial_correction);
            return Ok(());
        }
        fraction *= 0.5;
    }
    Ok(())
}

fn material_dot(
    a: &[Vec3],
    b: &[Vec3],
    cancelled: &impl Fn() -> bool,
) -> Result<f64, SimulationError> {
    let mut result = 0.;
    for (&a, &b) in a.iter().zip(b) {
        check_cancel(cancelled)?;
        result += dot(a, b);
    }
    Ok(result)
}

fn material_energy(
    membrane: &[Membrane],
    x: &[Vec3],
    correction: &[Vec3],
    root_weight: &[f64],
    dt: f64,
    cancelled: &impl Fn() -> bool,
) -> Result<f64, SimulationError> {
    let mut energy = 0.;
    for (&p, &weight) in correction.iter().zip(root_weight) {
        check_cancel(cancelled)?;
        if weight > 0. {
            let inertia = p.map(|component| component / weight);
            energy += 0.5 * dot(inertia, inertia);
        }
    }
    for triangle in membrane {
        check_cancel(cancelled)?;
        energy += triangle.energy(triangle.vertices.map(|v| x[v]), dt);
    }
    Ok(energy)
}
