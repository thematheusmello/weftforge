use crate::{Material, SimulationError, Stitch, Vec3, check_cancel, math::*};
use std::cell::Cell;

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
const ENERGY_BACKTRACKING_STEPS: usize = 16;

fn material_cancellation(cancelled: &impl Fn() -> bool) -> impl Fn() -> bool + '_ {
    let remaining = Cell::new(63_u8);
    move || {
        let left = remaining.get();
        if left == 0 {
            remaining.set(63);
            cancelled()
        } else {
            remaining.set(left - 1);
            false
        }
    }
}

pub(crate) fn solve_material(
    membrane: &[Membrane],
    x: &mut [Vec3],
    inverse_mass: &[f64],
    dt: f64,
    correction: &mut [Vec3],
    cancelled: &impl Fn() -> bool,
) -> Result<(), SimulationError> {
    check_cancel(cancelled)?;
    let original_cancelled = cancelled;
    let material_cancelled = material_cancellation(original_cancelled);
    let cancelled = &material_cancelled;
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
        check_cancel(original_cancelled)?;
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
        check_cancel(original_cancelled)?;
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
    for _ in 0..ENERGY_BACKTRACKING_STEPS {
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
            check_cancel(original_cancelled)?;
            x.copy_from_slice(&trial);
            correction.copy_from_slice(&trial_correction);
            return Ok(());
        }
        fraction *= 0.5;
    }
    check_cancel(original_cancelled)?;
    Ok(())
}

#[cfg(test)]
mod material_cancellation_tests {
    use super::*;

    #[test]
    fn material_cancellation_polls_after_at_most_63_skipped_checkpoints() {
        let checkpoint = Cell::new(0);
        let polls = Cell::new(0);
        let previous = Cell::new(0);
        let callback = || {
            assert_eq!(checkpoint.get() - previous.get(), 64);
            previous.set(checkpoint.get());
            polls.set(polls.get() + 1);
            false
        };
        let cancelled = material_cancellation(&callback);
        for site in 1..=64 * 5 + 63 {
            checkpoint.set(site);
            assert!(!cancelled());
        }
        assert_eq!(polls.get(), 5);
    }
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

#[derive(Clone, Debug)]
pub(crate) struct Bend {
    vertices: [usize; 4],
    rest_angle: f64,
    stiffness: f64,
}
impl Bend {
    pub fn new(vertices: [usize; 4], x: &[Vec3], rigidity: f64) -> Self {
        let e = sub(x[vertices[1]], x[vertices[0]]);
        let l = length(e);
        let n1 = cross(e, sub(x[vertices[2]], x[vertices[0]]));
        let n2 = cross(sub(x[vertices[3]], x[vertices[0]]), e);
        let h = (length(n1) + length(n2)) / (2. * l);
        Self {
            vertices,
            rest_angle: dihedral(vertices, x).map_or(0., |(a, _)| a),
            stiffness: rigidity * l / h,
        }
    }
    pub fn valid_scale(&self) -> bool {
        self.rest_angle.is_finite() && self.stiffness.is_finite() && self.stiffness > 0.
    }
    pub fn solve(
        &self,
        x: &mut [Vec3],
        inverse_mass: &[f64],
        dt: f64,
        correction: &mut [Vec3; 4],
        cancelled: &impl Fn() -> bool,
    ) -> Result<(), SimulationError> {
        check_cancel(cancelled)?;
        let Some((angle, gradient)) = dihedral(self.vertices, x) else {
            return Ok(());
        };
        let unstable = || SimulationError::Unstable("non-finite bending energy solve".into());
        let difference = angle - self.rest_angle;
        let value = difference.sin().atan2(difference.cos());
        let scale = dt * self.stiffness.sqrt();
        let root_weight = self.vertices.map(|v| inverse_mass[v].sqrt());
        let inertia: [Vec3; 4] = std::array::from_fn(|i| {
            if root_weight[i] > 0. {
                mul(correction[i], root_weight[i].recip())
            } else {
                [0.; 3]
            }
        });
        let row: [Vec3; 4] = std::array::from_fn(|i| mul(gradient[i], scale * root_weight[i]));
        let residual = scale * value;
        let denominator = 1. + row.into_iter().map(|g| dot(g, g)).sum::<f64>();
        let multiplier = (row
            .into_iter()
            .zip(inertia)
            .map(|(g, p)| dot(g, p))
            .sum::<f64>()
            - residual)
            / denominator;
        // Retain actual displacements: a rotating angle gradient cannot reconstruct inertia from lambda.
        let direction: [Vec3; 4] =
            std::array::from_fn(|i| sub(mul(row[i], multiplier), inertia[i]));
        let slope = (0..4)
            .map(|i| dot(add(inertia[i], mul(row[i], residual)), direction[i]))
            .sum::<f64>();
        let energy =
            0.5 * (inertia.into_iter().map(|p| dot(p, p)).sum::<f64>() + residual * residual);
        if !denominator.is_finite() || !slope.is_finite() || !energy.is_finite() {
            return Err(unstable());
        }
        if slope >= 0. {
            check_cancel(cancelled)?;
            return Ok(());
        }
        let original = self.vertices.map(|v| x[v]);
        let mut fraction = 1.;
        // shortcut: 16 Armijo trials; a trust region is needed if a highly folded hinge stalls.
        for _ in 0..ENERGY_BACKTRACKING_STEPS {
            check_cancel(cancelled)?;
            let trial: [Vec3; 4] = std::array::from_fn(|i| {
                add(original[i], mul(direction[i], fraction * root_weight[i]))
            });
            let trial_correction: [Vec3; 4] =
                std::array::from_fn(|i| add(correction[i], sub(trial[i], original[i])));
            let Some((angle, _)) = dihedral([0, 1, 2, 3], &trial) else {
                fraction *= 0.5;
                continue;
            };
            let difference = angle - self.rest_angle;
            let residual = scale * difference.sin().atan2(difference.cos());
            let trial_energy = 0.5
                * ((0..4)
                    .filter(|&i| root_weight[i] > 0.)
                    .map(|i| {
                        let p = mul(trial_correction[i], root_weight[i].recip());
                        dot(p, p)
                    })
                    .sum::<f64>()
                    + residual * residual);
            if trial_energy.is_finite() && trial_energy <= energy + 1e-4 * fraction * slope {
                check_cancel(cancelled)?;
                for i in 0..4 {
                    x[self.vertices[i]] = trial[i];
                }
                *correction = trial_correction;
                return Ok(());
            }
            fraction *= 0.5;
        }
        check_cancel(cancelled)?;
        Ok(())
    }
}

pub(crate) fn dihedral(vertices: [usize; 4], x: &[Vec3]) -> Option<(f64, [Vec3; 4])> {
    let [a, b, c, d] = vertices.map(|v| x[v]);
    let e = sub(b, a);
    let l = length(e);
    let n1 = cross(e, sub(c, a));
    let n2 = cross(sub(d, a), e);
    let a1 = length(n1);
    let a2 = length(n2);
    if l < 1e-12 || a1 < 1e-14 || a2 < 1e-14 {
        return None;
    }
    let n1 = mul(n1, a1.recip());
    let n2 = mul(n2, a2.recip());
    let angle = dot(cross(n1, n2), mul(e, l.recip())).atan2(dot(n1, n2));
    let gc = mul(n1, -l / a1);
    let gd = mul(n2, -l / a2);
    let tc = dot(sub(c, a), e) / (l * l);
    let td = dot(sub(d, a), e) / (l * l);
    let ga = add(mul(gc, tc - 1.), mul(gd, td - 1.));
    let gb = add(mul(gc, -tc), mul(gd, -td));
    Some((angle, [ga, gb, gc, gd]))
}

fn project<const N: usize>(
    vertices: &[usize; N],
    gradient: &[Vec3; N],
    value: f64,
    alpha: f64,
    lambda: &mut f64,
    x: &mut [Vec3],
    inverse_mass: &[f64],
) {
    let denominator: f64 = vertices
        .iter()
        .zip(gradient)
        .map(|(&v, &g)| inverse_mass[v] * dot(g, g))
        .sum::<f64>()
        + alpha;
    if denominator <= 1e-20 || !denominator.is_finite() {
        return;
    }
    let delta = (-value - alpha * *lambda) / denominator;
    *lambda += delta;
    for (&v, &g) in vertices.iter().zip(gradient) {
        x[v] = add(x[v], mul(g, inverse_mass[v] * delta));
    }
}

pub(crate) fn solve_stitch(
    s: Stitch,
    target_distance: f64,
    x: &mut [Vec3],
    inverse_mass: &[f64],
    dt: f64,
    lambda: &mut Vec3,
) {
    let vertices = [
        s.a.vertices[0],
        s.a.vertices[1],
        s.b.vertices[0],
        s.b.vertices[1],
    ];
    let mut coefficients = [
        s.a.weights[0],
        s.a.weights[1],
        -s.b.weights[0],
        -s.b.weights[1],
    ];
    for i in 0..4 {
        for j in 0..i {
            if vertices[i] == vertices[j] {
                coefficients[j] += coefficients[i];
                coefficients[i] = 0.;
                break;
            }
        }
    }
    let difference = sub(s.a.position(x), s.b.position(x));
    if target_distance == 0. && s.compliance > 0. {
        let alpha = s.compliance / (dt * dt);
        let denominator = vertices
            .iter()
            .zip(coefficients)
            .map(|(&v, weight)| inverse_mass[v] * (weight * weight))
            .sum::<f64>()
            + alpha;
        if denominator <= 1e-20 || !denominator.is_finite() {
            return;
        }
        let delta = std::array::from_fn(|i| (-difference[i] - alpha * lambda[i]) / denominator);
        *lambda = add(*lambda, delta);
        for (&v, weight) in vertices.iter().zip(coefficients) {
            x[v] = add(x[v], mul(delta, inverse_mass[v] * weight));
        }
        return;
    }
    if s.compliance > 0. {
        let alpha = s.compliance / (dt * dt);
        let weight = vertices
            .iter()
            .zip(coefficients)
            .map(|(&v, coefficient)| inverse_mass[v] * (coefficient * coefficient))
            .sum::<f64>();
        let denominator = weight + alpha;
        if !target_distance.is_finite()
            || target_distance <= 0.
            || weight < 0.
            || denominator <= 1e-20
            || !denominator.is_finite()
        {
            return;
        }
        let anchor = sub(difference, mul(*lambda, weight));
        let radius = anchor[0].hypot(anchor[1]).hypot(anchor[2]);
        if !radius.is_finite() {
            return;
        }
        let unit_direction = |vector: Vec3| {
            let scale = vector[0].abs().max(vector[1].abs()).max(vector[2].abs());
            let scaled = vector.map(|value| value / scale);
            let norm = scaled[0].hypot(scaled[1]).hypot(scaled[2]);
            scaled.map(|value| value / norm)
        };
        let direction = if radius > 0. {
            unit_direction(anchor)
        } else {
            if weight == 0. {
                return;
            }
            let distance = difference[0].hypot(difference[1]).hypot(difference[2]);
            if !distance.is_finite() {
                return;
            }
            // At a zero anchor every direction minimizes the radial objective.
            if distance > 0. {
                unit_direction(difference)
            } else {
                [1., 0., 0.]
            }
        };
        let next = mul(direction, (target_distance - radius) / denominator);
        let delta = sub(next, *lambda);
        let updates = std::array::from_fn::<_, 4, _>(|i| {
            mul(delta, inverse_mass[vertices[i]] * coefficients[i])
        });
        if !finite(next)
            || !finite(delta)
            || vertices
                .iter()
                .zip(updates)
                .any(|(&v, update)| !finite(update) || !finite(add(x[v], update)))
        {
            return;
        }
        *lambda = next;
        for (&v, update) in vertices.iter().zip(updates) {
            x[v] = add(x[v], update);
        }
        return;
    }
    let distance = length(difference);
    if distance < 1e-14 {
        return;
    }
    let direction = mul(difference, distance.recip());
    let gradient = coefficients.map(|weight| mul(direction, weight));
    project(
        &vertices,
        &gradient,
        distance - target_distance,
        s.compliance / (dt * dt),
        &mut lambda[0],
        x,
        inverse_mass,
    );
}
