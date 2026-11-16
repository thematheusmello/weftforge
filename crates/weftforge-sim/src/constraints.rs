use crate::{Material, Stitch, Vec3, math::*};

#[derive(Clone, Debug)]
pub(crate) struct Membrane {
    vertices: [usize; 3],
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
    fn local_energy(
        &self,
        x: [Vec3; 3],
        correction: [Vec3; 3],
        inverse_mass: [f64; 3],
        dt_squared: f64,
    ) -> f64 {
        let inertia = correction
            .into_iter()
            .zip(inverse_mass)
            .filter(|(_, weight)| *weight > 0.)
            .map(|(p, weight)| dot(p, p) / weight)
            .sum::<f64>();
        let (w, f) = self.deformation(x);
        let values = [0.5 * (dot(w, w) - 1.), 0.5 * (dot(f, f) - 1.), dot(w, f)];
        0.5 * (inertia
            + dt_squared
                * values
                    .into_iter()
                    .zip(self.stiffness)
                    .map(|(value, k)| k * value * value)
                    .sum::<f64>())
    }

    pub fn solve(&self, x: &mut [Vec3], inverse_mass: &[f64], dt: f64, correction: &mut [Vec3; 3]) {
        let positions = self.vertices.map(|v| x[v]);
        let weights = self.vertices.map(|v| inverse_mass[v]);
        let dt_squared = dt * dt;
        let (w, f) = self.deformation(positions);
        let values = [0.5 * (dot(w, w) - 1.), 0.5 * (dot(f, f) - 1.), dot(w, f)];
        let gradients = [
            self.warp.map(|q| mul(w, q)),
            self.weft.map(|q| mul(f, q)),
            std::array::from_fn(|i| add(mul(f, self.warp[i]), mul(w, self.weft[i]))),
        ];
        let matrix = std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                (0..3)
                    .map(|v| weights[v] * dot(gradients[i][v], gradients[j][v]))
                    .sum::<f64>()
                    + if i == j {
                        self.stiffness[i].recip() / dt_squared
                    } else {
                        0.
                    }
            })
        });
        let rhs = std::array::from_fn(|i| {
            (0..3)
                .map(|v| dot(gradients[i][v], correction[v]))
                .sum::<f64>()
                - values[i]
        });
        let Some(multiplier) = solve_spd3(matrix, rhs) else {
            return;
        };
        // Actual positional corrections retain the inertial anchor when nonlinear gradients rotate.
        let direction: [Vec3; 3] = std::array::from_fn(|v| {
            sub(
                mul(
                    (0..3).fold([0.; 3], |sum, i| {
                        add(sum, mul(gradients[i][v], multiplier[i]))
                    }),
                    weights[v],
                ),
                correction[v],
            )
        });
        let slope = (0..3)
            .filter(|&v| weights[v] > 0.)
            .map(|v| dot(correction[v], direction[v]) / weights[v])
            .sum::<f64>()
            + dt_squared
                * (0..3)
                    .map(|i| {
                        self.stiffness[i]
                            * values[i]
                            * (0..3)
                                .map(|v| dot(gradients[i][v], direction[v]))
                                .sum::<f64>()
                    })
                    .sum::<f64>();
        if !slope.is_finite() || slope >= 0. {
            return;
        }
        let energy = self.local_energy(positions, *correction, weights, dt_squared);
        let mut fraction = 1.;
        // shortcut: cap local backtracking at 16; use a trust region if extreme compression stalls.
        for _ in 0..16 {
            let trial = std::array::from_fn(|v| add(positions[v], mul(direction[v], fraction)));
            let trial_correction =
                std::array::from_fn(|v| add(correction[v], sub(trial[v], positions[v])));
            let trial_energy = self.local_energy(trial, trial_correction, weights, dt_squared);
            if trial_energy.is_finite() && trial_energy <= energy + 1e-4 * fraction * slope {
                for (v, position) in self.vertices.into_iter().zip(trial) {
                    x[v] = position;
                }
                *correction = trial_correction;
                return;
            }
            fraction *= 0.5;
        }
    }
}

fn solve_spd3(matrix: [[f64; 3]; 3], rhs: [f64; 3]) -> Option<[f64; 3]> {
    let scale: [f64; 3] = std::array::from_fn(|i| matrix[i][i].sqrt());
    if !scale.iter().all(|s| s.is_finite() && *s > 0.) {
        return None;
    }
    let a: [[f64; 3]; 3] =
        std::array::from_fn(|i| std::array::from_fn(|j| matrix[i][j] / scale[i] / scale[j]));
    let b: [f64; 3] = std::array::from_fn(|i| rhs[i] / scale[i]);
    let l00 = a[0][0].sqrt();
    let l10 = a[1][0] / l00;
    let l20 = a[2][0] / l00;
    let l11 = (a[1][1] - l10 * l10).sqrt();
    let l21 = (a[2][1] - l20 * l10) / l11;
    let l22 = (a[2][2] - l20 * l20 - l21 * l21).sqrt();
    if ![l00, l11, l22].iter().all(|s| s.is_finite() && *s > 0.) {
        return None;
    }
    let y0 = b[0] / l00;
    let y1 = (b[1] - l10 * y0) / l11;
    let y2 = (b[2] - l20 * y0 - l21 * y1) / l22;
    let z2 = y2 / l22;
    let z1 = (y1 - l21 * z2) / l11;
    let z0 = (y0 - l10 * z1 - l20 * z2) / l00;
    let solution = [z0 / scale[0], z1 / scale[1], z2 / scale[2]];
    solution.iter().all(|s| s.is_finite()).then_some(solution)
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
    pub fn solve(&self, x: &mut [Vec3], inverse_mass: &[f64], dt: f64, lambda: &mut f64) {
        let Some((angle, gradient)) = dihedral(self.vertices, x) else {
            return;
        };
        let difference = angle - self.rest_angle;
        let value = difference.sin().atan2(difference.cos());
        project(
            &self.vertices,
            &gradient,
            value,
            self.stiffness.recip() / (dt * dt),
            lambda,
            x,
            inverse_mass,
        );
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
