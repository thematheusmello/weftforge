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
}

struct CoupledRow {
    source: usize,
    vertices: [usize; 4],
    gradient: [Vec3; 4],
    value: f64,
}

const COUPLED_LINEAR_ITERATIONS: usize = 32;

pub(crate) fn solve_bending(
    bends: &[Bend],
    x: &mut [Vec3],
    inverse_mass: &[f64],
    dt: f64,
    correction: &mut [Vec3],
    cancelled: &impl Fn() -> bool,
) -> Result<(), SimulationError> {
    // correction is the actual stage displacement p_b, not a force or multiplier.
    // Its anchor z=x-p_b follows other stages; mass-normalized inertia is p_b/sqrt(w).
    check_cancel(cancelled)?;
    let original_cancelled = cancelled;
    let bounded_cancelled = material_cancellation(original_cancelled);
    let cancelled = &bounded_cancelled;
    let unstable = || SimulationError::Unstable("non-finite global bending solve".into());
    if !dt.is_finite() || dt <= 0. {
        return Err(unstable());
    }
    if bends.is_empty() {
        check_cancel(original_cancelled)?;
        return Ok(());
    }
    let mut root_weight = Vec::with_capacity(x.len());
    let mut diagonal = vec![[1_f64; 3]; x.len()];
    for i in 0..x.len() {
        check_cancel(cancelled)?;
        let root = inverse_mass[i].sqrt();
        if !root.is_finite() || !finite(x[i]) || !finite(correction[i]) {
            return Err(unstable());
        }
        root_weight.push(root);
    }
    let mut rows = Vec::with_capacity(bends.len());
    for (index, bend) in bends.iter().enumerate() {
        check_cancel(cancelled)?;
        let Some((angle, gradient)) = dihedral(bend.vertices, x) else {
            continue;
        };
        let difference = angle - bend.rest_angle;
        let scale = dt * bend.stiffness.sqrt();
        let value = scale * difference.sin().atan2(difference.cos());
        let gradient =
            std::array::from_fn(|i| mul(gradient[i], scale * root_weight[bend.vertices[i]]));
        // Keep the single-hinge overflow guard before normalizing the global rows.
        let denominator = 1. + gradient.into_iter().map(|g| dot(g, g)).sum::<f64>();
        if !value.is_finite() || !denominator.is_finite() || !gradient.into_iter().all(finite) {
            return Err(unstable());
        }
        for (vertex, gradient) in bend.vertices.into_iter().zip(gradient) {
            for (axis, component) in gradient.into_iter().enumerate() {
                diagonal[vertex][axis] = diagonal[vertex][axis].hypot(component);
                if !diagonal[vertex][axis].is_finite() {
                    return Err(unstable());
                }
            }
        }
        rows.push(CoupledRow {
            source: index,
            vertices: bend.vertices,
            gradient,
            value,
        });
    }
    if rows.is_empty() {
        check_cancel(original_cancelled)?;
        return Ok(());
    }
    CoupledBlock {
        rows,
        root_weight,
        diagonal,
        failure: "non-finite global bending solve",
    }
    .update(
        x,
        correction,
        cancelled,
        original_cancelled,
        |rows, x, correction, root_weight| {
            bending_energy(bends, rows, x, correction, root_weight, dt, cancelled)
        },
    )
}

struct CoupledBlock {
    rows: Vec<CoupledRow>,
    root_weight: Vec<f64>,
    diagonal: Vec<Vec3>,
    failure: &'static str,
}
impl CoupledBlock {
    fn update(
        self,
        x: &mut [Vec3],
        correction: &mut [Vec3],
        cancelled: &impl Fn() -> bool,
        original_cancelled: &impl Fn() -> bool,
        evaluate: impl Fn(&[CoupledRow], &[Vec3], &[Vec3], &[f64]) -> Result<f64, SimulationError>,
    ) -> Result<(), SimulationError> {
        let Self {
            mut rows,
            root_weight,
            diagonal,
            failure,
        } = self;
        let unstable = || SimulationError::Unstable(failure.into());
        let energy = evaluate(&rows, x, correction, &root_weight)?;
        if !energy.is_finite() {
            return Err(unstable());
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
        let mut solution = vec![[0.; 3]; x.len()];
        let mut residual = vec![[0.; 3]; x.len()];
        let mut search = vec![[0.; 3]; x.len()];
        let mut product = vec![[0.; 3]; x.len()];
        for i in 0..x.len() {
            check_cancel(cancelled)?;
            rhs[i] = rhs[i].map(|component| component / rhs_scale);
            residual[i] = rhs[i];
            search[i] = rhs[i];
        }
        let mut squared_residual = material_dot(&residual, &residual, cancelled)?;
        let initial_squared_residual = squared_residual;
        // shortcut: 32 diagonal-PCG steps, matching the bounded global material solve.
        for _ in 0..COUPLED_LINEAR_ITERATIONS {
            check_cancel(cancelled)?;
            for i in 0..x.len() {
                check_cancel(cancelled)?;
                product[i] = std::array::from_fn(|axis| {
                    search[i][axis] / diagonal[i][axis] / diagonal[i][axis]
                });
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
        let mut trial = vec![[0.; 3]; x.len()];
        let mut trial_correction = vec![[0.; 3]; x.len()];
        let mut fraction = 1.;
        // shortcut: 16 Armijo trials; reassess with independently measured block residuals.
        for _ in 0..ENERGY_BACKTRACKING_STEPS {
            check_cancel(cancelled)?;
            let mut trial_finite = true;
            for i in 0..x.len() {
                check_cancel(cancelled)?;
                trial[i] = if root_weight[i] > 0. {
                    add(x[i], mul(solution[i], fraction))
                } else {
                    x[i]
                };
                trial_correction[i] = add(correction[i], sub(trial[i], x[i]));
                trial_finite &= finite(trial[i]) && finite(trial_correction[i]);
            }
            let trial_energy = if trial_finite {
                evaluate(&rows, &trial, &trial_correction, &root_weight)?
            } else {
                f64::INFINITY
            };
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
}

fn bending_energy(
    bends: &[Bend],
    rows: &[CoupledRow],
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
    for row in rows {
        check_cancel(cancelled)?;
        let bend = &bends[row.source];
        let Some((angle, _)) = dihedral(bend.vertices, x) else {
            return Ok(f64::INFINITY);
        };
        let difference = angle - bend.rest_angle;
        let residual = dt * bend.stiffness.sqrt() * difference.sin().atan2(difference.cos());
        energy += 0.5 * residual * residual;
    }
    Ok(energy)
}

pub(crate) fn solve_sewing(
    stitches: &[Stitch],
    targets: &[f64],
    x: &mut [Vec3],
    inverse_mass: &[f64],
    dt: f64,
    correction: &mut [Vec3],
    cancelled: &impl Fn() -> bool,
) -> Result<(), SimulationError> {
    // One actual soft-sewing displacement p_s per node: z=x-p_s, not a force.
    check_cancel(cancelled)?;
    let original_cancelled = cancelled;
    let bounded_cancelled = material_cancellation(original_cancelled);
    let cancelled = &bounded_cancelled;
    let unstable = || SimulationError::Unstable("non-finite global sewing solve".into());
    if !dt.is_finite()
        || dt <= 0.
        || stitches.len() != targets.len()
        || x.len() != inverse_mass.len()
        || x.len() != correction.len()
    {
        return Err(unstable());
    }
    let mut soft_count = 0;
    let mut single_soft = 0;
    for (index, (stitch, &target)) in stitches.iter().zip(targets).enumerate() {
        check_cancel(cancelled)?;
        if !stitch.compliance.is_finite()
            || stitch.compliance < 0.
            || !target.is_finite()
            || target < 0.
        {
            return Err(unstable());
        }
        if stitch.compliance > 0. {
            soft_count += 1;
            single_soft = index;
        }
    }
    if soft_count == 0 {
        check_cancel(original_cancelled)?;
        return Ok(());
    }
    if soft_count == 1 {
        let trial = single_soft_trial(
            stitches[single_soft],
            targets[single_soft],
            x,
            inverse_mass,
            dt,
            correction,
            cancelled,
        )?;
        check_cancel(original_cancelled)?;
        if let Some((trial, trial_correction)) = trial {
            x.copy_from_slice(&trial);
            correction.copy_from_slice(&trial_correction);
        }
        return Ok(());
    }
    let mut root_weight = Vec::with_capacity(x.len());
    for i in 0..x.len() {
        check_cancel(cancelled)?;
        let root = inverse_mass[i].sqrt();
        if !root.is_finite() || !finite(x[i]) || !finite(correction[i]) {
            return Err(unstable());
        }
        root_weight.push(root);
    }
    let mut rows = Vec::new();
    for (index, (stitch, &target)) in stitches.iter().zip(targets).enumerate() {
        check_cancel(cancelled)?;
        if stitch.compliance == 0. {
            continue;
        }
        let (vertices, coefficients) = stitch_coefficients(*stitch);
        if vertices.into_iter().any(|v| v >= x.len())
            || !coefficients.into_iter().all(f64::is_finite)
        {
            return Err(unstable());
        }
        let difference = sub(stitch.a.position(x), stitch.b.position(x));
        let scale = dt / stitch.compliance.sqrt();
        let (values, directions, modes) = if target == 0. {
            (difference, [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]], 3)
        } else {
            let radius = difference[0].hypot(difference[1]).hypot(difference[2]);
            if !radius.is_finite() {
                return Err(unstable());
            }
            let direction = if radius > 0. {
                stitch_unit_direction(difference)
            } else {
                let anchor = |point: crate::WeightedPoint| {
                    add(
                        mul(
                            sub(x[point.vertices[0]], correction[point.vertices[0]]),
                            point.weights[0],
                        ),
                        mul(
                            sub(x[point.vertices[1]], correction[point.vertices[1]]),
                            point.weights[1],
                        ),
                    )
                };
                let anchor_difference = sub(anchor(stitch.a), anchor(stitch.b));
                if !finite(anchor_difference) {
                    return Err(unstable());
                }
                if anchor_difference.into_iter().any(|value| value != 0.) {
                    stitch_unit_direction(anchor_difference)
                } else {
                    [1., 0., 0.]
                }
            };
            ([radius - target, 0., 0.], [direction, [0.; 3], [0.; 3]], 1)
        };
        for mode in 0..modes {
            check_cancel(cancelled)?;
            let value = scale * values[mode];
            let gradient = std::array::from_fn(|i| {
                mul(
                    directions[mode],
                    scale * root_weight[vertices[i]] * coefficients[i],
                )
            });
            if !scale.is_finite()
                || scale <= 0.
                || !value.is_finite()
                || !gradient.into_iter().all(finite)
            {
                return Err(unstable());
            }
            rows.push(CoupledRow {
                source: index,
                vertices,
                gradient,
                value,
            });
        }
    }
    // Dual/Woodbury GN keeps every finite iterate in the weighted row-action space.
    // a=p/sqrt(W), B=J*sqrt(W): (I+B*B^T)eta=B*a-r, delta=-p+sqrt(W)*B^T*eta.
    let energy = sewing_energy(
        stitches,
        targets,
        x,
        correction,
        &root_weight,
        dt,
        cancelled,
    )?;
    if !energy.is_finite() {
        return Err(unstable());
    }
    let mut action = vec![[0.; 3]; x.len()];
    for i in 0..x.len() {
        check_cancel(cancelled)?;
        if root_weight[i] > 0. {
            action[i] = correction[i].map(|value| value / root_weight[i]);
        }
        if !finite(action[i]) {
            return Err(unstable());
        }
    }
    let mut gradient = action.clone();
    let mut diagonal = Vec::with_capacity(rows.len());
    let mut rhs = Vec::with_capacity(rows.len());
    for row in &mut rows {
        check_cancel(cancelled)?;
        let mut scale = 1_f64;
        let mut jp = 0.;
        for (vertex, g) in row.vertices.into_iter().zip(row.gradient) {
            jp += dot(g, action[vertex]);
            gradient[vertex] = add(gradient[vertex], mul(g, row.value));
            for component in g {
                scale = scale.hypot(component);
            }
        }
        let value = (jp - row.value) / scale;
        if !scale.is_finite() || !value.is_finite() {
            return Err(unstable());
        }
        diagonal.push(scale);
        rhs.push(value);
        for g in &mut row.gradient {
            *g = g.map(|component| component / scale);
        }
    }
    for &g in &gradient {
        check_cancel(cancelled)?;
        if !finite(g) {
            return Err(unstable());
        }
    }
    let dot_rows = |a: &[f64], b: &[f64]| -> Result<f64, SimulationError> {
        let mut value = 0.;
        for (&a, &b) in a.iter().zip(b) {
            check_cancel(cancelled)?;
            value += a * b;
        }
        Ok(value)
    };
    let mut rhs_scale = 0_f64;
    for &value in &rhs {
        check_cancel(cancelled)?;
        rhs_scale = rhs_scale.max(value.abs());
    }
    let mut solution = vec![0.; rows.len()];
    if rhs_scale > 0. {
        for value in &mut rhs {
            check_cancel(cancelled)?;
            *value /= rhs_scale;
        }
        let mut residual = rhs.clone();
        let mut search = residual.clone();
        let mut product = vec![0.; rows.len()];
        let mut nodal = vec![[0.; 3]; x.len()];
        let mut squared_residual = dot_rows(&residual, &residual)?;
        let initial_squared_residual = squared_residual;
        for _ in 0..COUPLED_LINEAR_ITERATIONS {
            check_cancel(cancelled)?;
            for value in &mut nodal {
                check_cancel(cancelled)?;
                *value = [0.; 3];
            }
            for (row, &value) in rows.iter().zip(&search) {
                check_cancel(cancelled)?;
                for (vertex, g) in row.vertices.into_iter().zip(row.gradient) {
                    nodal[vertex] = add(nodal[vertex], mul(g, value));
                }
            }
            for (i, row) in rows.iter().enumerate() {
                check_cancel(cancelled)?;
                product[i] = search[i] / diagonal[i] / diagonal[i]
                    + row
                        .vertices
                        .into_iter()
                        .zip(row.gradient)
                        .map(|(vertex, g)| dot(g, nodal[vertex]))
                        .sum::<f64>();
            }
            let curvature = dot_rows(&search, &product)?;
            if !curvature.is_finite() || curvature <= 0. {
                return Err(unstable());
            }
            let alpha = squared_residual / curvature;
            let mut next_squared_residual = 0.;
            for i in 0..rows.len() {
                check_cancel(cancelled)?;
                solution[i] += alpha * search[i];
                residual[i] -= alpha * product[i];
                if !solution[i].is_finite() || !residual[i].is_finite() {
                    return Err(unstable());
                }
                next_squared_residual += residual[i] * residual[i];
            }
            if !next_squared_residual.is_finite() {
                return Err(unstable());
            }
            if next_squared_residual <= 1e-24 * initial_squared_residual {
                break;
            }
            let beta = next_squared_residual / squared_residual;
            for i in 0..rows.len() {
                check_cancel(cancelled)?;
                search[i] = residual[i] + beta * search[i];
            }
            squared_residual = next_squared_residual;
        }
    }
    // A zero dual RHS still requires delta=-p: never discard the old nodal history.
    let mut direction = vec![[0.; 3]; x.len()];
    for i in 0..x.len() {
        check_cancel(cancelled)?;
        direction[i] = mul(action[i], -1.);
    }
    for (row, &value) in rows.iter().zip(&solution) {
        check_cancel(cancelled)?;
        let value = value * rhs_scale;
        for (vertex, g) in row.vertices.into_iter().zip(row.gradient) {
            direction[vertex] = add(direction[vertex], mul(g, value));
        }
    }
    // This is the true mass-normalized primal gradient dot recovered direction.
    let slope = material_dot(&gradient, &direction, cancelled)?;
    if !slope.is_finite() {
        return Err(unstable());
    }
    if slope >= 0. {
        check_cancel(original_cancelled)?;
        return Ok(());
    }
    for i in 0..x.len() {
        check_cancel(cancelled)?;
        direction[i] = mul(direction[i], root_weight[i]);
        if !finite(direction[i]) {
            return Err(unstable());
        }
    }
    let mut trial = vec![[0.; 3]; x.len()];
    let mut trial_correction = vec![[0.; 3]; x.len()];
    let mut fraction = 1.;
    for _ in 0..ENERGY_BACKTRACKING_STEPS {
        check_cancel(cancelled)?;
        let mut trial_finite = true;
        for i in 0..x.len() {
            check_cancel(cancelled)?;
            trial[i] = if root_weight[i] > 0. {
                add(x[i], mul(direction[i], fraction))
            } else {
                x[i]
            };
            trial_correction[i] = add(correction[i], sub(trial[i], x[i]));
            trial_finite &= finite(trial[i]) && finite(trial_correction[i]);
        }
        let trial_energy = if trial_finite {
            sewing_energy(
                stitches,
                targets,
                &trial,
                &trial_correction,
                &root_weight,
                dt,
                cancelled,
            )?
        } else {
            f64::INFINITY
        };
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

type SoftSewingTrial = (Vec<Vec3>, Vec<Vec3>);

fn single_soft_trial(
    stitch: Stitch,
    target: f64,
    x: &[Vec3],
    inverse_mass: &[f64],
    dt: f64,
    correction: &[Vec3],
    cancelled: &impl Fn() -> bool,
) -> Result<Option<SoftSewingTrial>, SimulationError> {
    let unstable = || SimulationError::Unstable("non-finite global sewing solve".into());
    let mut anchor = Vec::with_capacity(x.len());
    let mut root_weight = Vec::with_capacity(x.len());
    for i in 0..x.len() {
        check_cancel(cancelled)?;
        let root = inverse_mass[i].sqrt();
        if !root.is_finite() || !finite(x[i]) || !finite(correction[i]) {
            return Err(unstable());
        }
        let z = if inverse_mass[i] > 0. {
            sub(x[i], correction[i])
        } else {
            x[i]
        };
        if !finite(z) {
            return Err(unstable());
        }
        anchor.push(z);
        root_weight.push(root);
    }
    let (vertices, coefficients) = stitch_coefficients(stitch);
    if vertices.into_iter().any(|v| v >= x.len()) || !coefficients.into_iter().all(f64::is_finite) {
        return Err(unstable());
    }
    let weight = vertices
        .iter()
        .zip(coefficients)
        .map(|(&v, coefficient)| inverse_mass[v] * (coefficient * coefficient))
        .sum::<f64>();
    let alpha = stitch.compliance / (dt * dt);
    let denominator = weight + alpha;
    // Retain the local finite/denominator guard; no unrepresentable update commits.
    if denominator <= 1e-20 || !denominator.is_finite() {
        return Ok(None);
    }
    let difference = sub(stitch.a.position(&anchor), stitch.b.position(&anchor));
    let impulse = if target == 0. {
        difference.map(|value| -value / denominator)
    } else {
        let radius = difference[0].hypot(difference[1]).hypot(difference[2]);
        if !radius.is_finite() {
            return Err(unstable());
        }
        let direction = if radius > 0. {
            stitch_unit_direction(difference)
        } else {
            let current = sub(stitch.a.position(x), stitch.b.position(x));
            if !finite(current) {
                return Err(unstable());
            }
            if current.into_iter().any(|value| value != 0.) {
                stitch_unit_direction(current)
            } else {
                [1., 0., 0.]
            }
        };
        mul(direction, (target - radius) / denominator)
    };
    if !finite(impulse) {
        return Err(unstable());
    }
    let energy = sewing_energy(
        &[stitch],
        &[target],
        x,
        correction,
        &root_weight,
        dt,
        cancelled,
    )?;
    if !energy.is_finite() {
        return Err(unstable());
    }
    let mut trial = anchor;
    for (vertex, coefficient) in vertices.into_iter().zip(coefficients) {
        if inverse_mass[vertex] > 0. && coefficient != 0. {
            trial[vertex] = add(
                trial[vertex],
                mul(impulse, inverse_mass[vertex] * coefficient),
            );
        }
    }
    let mut trial_correction = Vec::with_capacity(x.len());
    for i in 0..x.len() {
        check_cancel(cancelled)?;
        let p = if inverse_mass[i] > 0. {
            add(correction[i], sub(trial[i], x[i]))
        } else {
            correction[i]
        };
        if !finite(trial[i]) || !finite(p) {
            return Err(unstable());
        }
        trial_correction.push(p);
    }
    let trial_energy = sewing_energy(
        &[stitch],
        &[target],
        &trial,
        &trial_correction,
        &root_weight,
        dt,
        cancelled,
    )?;
    if !trial_energy.is_finite() {
        return Err(unstable());
    }
    if trial_energy > energy {
        return Ok(None);
    }
    Ok(Some((trial, trial_correction)))
}

fn sewing_energy(
    stitches: &[Stitch],
    targets: &[f64],
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
    for (stitch, &target) in stitches.iter().zip(targets) {
        check_cancel(cancelled)?;
        if stitch.compliance > 0. {
            let difference = sub(stitch.a.position(x), stitch.b.position(x));
            let scale = dt / stitch.compliance.sqrt();
            if target == 0. {
                let residual = mul(difference, scale);
                energy += 0.5 * dot(residual, residual);
            } else {
                let radius = difference[0].hypot(difference[1]).hypot(difference[2]);
                let residual = scale * (radius - target);
                energy += 0.5 * residual * residual;
            }
        }
    }
    Ok(energy)
}

fn stitch_coefficients(stitch: Stitch) -> ([usize; 4], [f64; 4]) {
    let vertices = [
        stitch.a.vertices[0],
        stitch.a.vertices[1],
        stitch.b.vertices[0],
        stitch.b.vertices[1],
    ];
    let mut coefficients = [
        stitch.a.weights[0],
        stitch.a.weights[1],
        -stitch.b.weights[0],
        -stitch.b.weights[1],
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
    (vertices, coefficients)
}

fn stitch_unit_direction(vector: Vec3) -> Vec3 {
    let scale = vector[0].abs().max(vector[1].abs()).max(vector[2].abs());
    let scaled = vector.map(|value| value / scale);
    let norm = scaled[0].hypot(scaled[1]).hypot(scaled[2]);
    scaled.map(|value| value / norm)
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

pub(crate) fn solve_hard_stitch(
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

#[cfg(test)]
mod bending_energy_tests {
    use super::*;

    #[test]
    fn active_hinge_trial_degeneracy_rejects_the_whole_bending_energy() {
        let rest = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., -1., 0.]];
        let bends = [Bend::new([0, 1, 2, 3], &rest, 0.5)];
        let rows = [CoupledRow {
            source: 0,
            vertices: [0, 1, 2, 3],
            gradient: [[0.; 3]; 4],
            value: 0.,
        }];
        let correction = [[0.; 3]; 4];
        let weights = [1.; 4];
        assert!(
            bending_energy(&bends, &rows, &rest, &correction, &weights, 0.1, &|| false)
                .unwrap()
                .is_finite()
        );
        let mut trial = rest;
        trial[1] = trial[0];
        assert_eq!(
            bending_energy(&bends, &rows, &trial, &correction, &weights, 0.1, &|| false).unwrap(),
            f64::INFINITY
        );
    }
}
