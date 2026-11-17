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

    fn energy(&self, positions: [Vec3; 3]) -> f64 {
        let (w, f) = self.deformation(positions);
        [0.5 * (dot(w, w) - 1.), 0.5 * (dot(f, f) - 1.), dot(w, f)]
            .into_iter()
            .zip(self.stiffness)
            .map(|(c, k)| 0.5 * k * c * c)
            .sum()
    }

    pub fn solve(&self, x: &mut [Vec3], inverse_mass: &[f64], dt: f64, correction: &mut [Vec3; 3]) {
        let (values, gradients) = self.values_gradients(self.vertices.map(|v| x[v]));
        MaterialProjection {
            vertices: self.vertices,
            values,
            gradients,
            stiffness: self.stiffness,
        }
        .solve(x, inverse_mass, dt, correction, |q| self.energy(q));
    }
}

#[derive(Clone, Debug)]
pub(crate) enum MaterialGroup {
    Single(usize),
    Pair {
        membranes: [usize; 2],
        vertices: [usize; 4],
        local: [[usize; 3]; 2],
    },
}

impl MaterialGroup {
    pub(crate) fn pair(a: usize, b: usize, membrane: &[Membrane]) -> Self {
        let first = membrane[a].vertices;
        let second = membrane[b].vertices;
        let fourth = second
            .into_iter()
            .find(|v| !first.contains(v))
            .expect("validated adjacent distinct triangles have four union vertices");
        let vertices = [first[0], first[1], first[2], fourth];
        let local = [first, second].map(|triangle| {
            triangle.map(|v| {
                vertices
                    .iter()
                    .position(|&u| u == v)
                    .expect("material group vertex")
            })
        });
        Self::Pair {
            membranes: [a, b],
            vertices,
            local,
        }
    }

    pub(crate) fn solve(
        &self,
        membrane: &[Membrane],
        x: &mut [Vec3],
        inverse_mass: &[f64],
        dt: f64,
        correction: &mut [Vec3; 4],
    ) {
        match self {
            Self::Single(id) => {
                let mut single = [correction[0], correction[1], correction[2]];
                membrane[*id].solve(x, inverse_mass, dt, &mut single);
                correction[..3].copy_from_slice(&single);
            }
            Self::Pair {
                membranes,
                vertices,
                local,
            } => {
                let positions = vertices.map(|v| x[v]);
                let rows = std::array::from_fn::<_, 2, _>(|i| {
                    membrane[membranes[i]].values_gradients(local[i].map(|v| positions[v]))
                });
                let values = std::array::from_fn::<_, 6, _>(|i| rows[i / 3].0[i % 3]);
                let stiffness =
                    std::array::from_fn(|i| membrane[membranes[i / 3]].stiffness[i % 3]);
                let mut gradients = [[[0.; 3]; 4]; 6];
                for i in 0..6 {
                    for j in 0..3 {
                        gradients[i][local[i / 3][j]] = rows[i / 3].1[i % 3][j];
                    }
                }
                MaterialProjection {
                    vertices: *vertices,
                    values,
                    gradients,
                    stiffness,
                }
                .solve(x, inverse_mass, dt, correction, |q| {
                    (0..2)
                        .map(|i| membrane[membranes[i]].energy(local[i].map(|v| q[v])))
                        .sum()
                });
            }
        }
    }
}

struct MaterialProjection<const V: usize, const C: usize> {
    vertices: [usize; V],
    values: [f64; C],
    gradients: [[Vec3; V]; C],
    stiffness: [f64; C],
}

impl<const V: usize, const C: usize> MaterialProjection<V, C> {
    fn solve(
        self,
        x: &mut [Vec3],
        inverse_mass: &[f64],
        dt: f64,
        correction: &mut [Vec3; V],
        elastic_energy: impl Fn([Vec3; V]) -> f64,
    ) {
        let positions = self.vertices.map(|v| x[v]);
        let weights = self.vertices.map(|v| inverse_mass[v]);
        let dt_squared = dt * dt;
        let matrix = std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                (0..V)
                    .map(|v| weights[v] * dot(self.gradients[i][v], self.gradients[j][v]))
                    .sum::<f64>()
                    + if i == j {
                        self.stiffness[i].recip() / dt_squared
                    } else {
                        0.
                    }
            })
        });
        let rhs = std::array::from_fn(|i| {
            (0..V)
                .map(|v| dot(self.gradients[i][v], correction[v]))
                .sum::<f64>()
                - self.values[i]
        });
        let Some(multiplier) = solve_spd::<C>(matrix, rhs) else {
            return;
        };
        let direction: [Vec3; V] = std::array::from_fn(|v| {
            sub(
                mul(
                    (0..C).fold([0.; 3], |sum, i| {
                        add(sum, mul(self.gradients[i][v], multiplier[i]))
                    }),
                    weights[v],
                ),
                correction[v],
            )
        });
        let slope = (0..V)
            .filter(|&v| weights[v] > 0.)
            .map(|v| dot(correction[v], direction[v]) / weights[v])
            .sum::<f64>()
            + dt_squared
                * (0..C)
                    .map(|i| {
                        self.stiffness[i]
                            * self.values[i]
                            * (0..V)
                                .map(|v| dot(self.gradients[i][v], direction[v]))
                                .sum::<f64>()
                    })
                    .sum::<f64>();
        if !slope.is_finite() || slope >= 0. {
            return;
        }
        let objective = |q: [Vec3; V], p: [Vec3; V]| {
            0.5 * p
                .into_iter()
                .zip(weights)
                .filter(|(_, w)| *w > 0.)
                .map(|(p, w)| dot(p, p) / w)
                .sum::<f64>()
                + dt_squared * elastic_energy(q)
        };
        let energy = objective(positions, *correction);
        let mut fraction = 1.;
        // shortcut: cap local backtracking at 16; use a trust region if extreme compression stalls.
        for _ in 0..16 {
            let trial = std::array::from_fn(|v| add(positions[v], mul(direction[v], fraction)));
            let trial_correction =
                std::array::from_fn(|v| add(correction[v], sub(trial[v], positions[v])));
            let trial_energy = objective(trial, trial_correction);
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

// The only two sizes are three rows for a singleton and six for an adjacent pair.
fn solve_spd<const N: usize>(matrix: [[f64; N]; N], rhs: [f64; N]) -> Option<[f64; N]> {
    let scale: [f64; N] = std::array::from_fn(|i| matrix[i][i].sqrt());
    if !scale.iter().all(|s| s.is_finite() && *s > 0.) {
        return None;
    }
    let a: [[f64; N]; N] =
        std::array::from_fn(|i| std::array::from_fn(|j| matrix[i][j] / scale[i] / scale[j]));
    let b: [f64; N] = std::array::from_fn(|i| rhs[i] / scale[i]);
    let mut l = [[0.; N]; N];
    for i in 0..N {
        for j in 0..=i {
            let value = a[i][j] - (0..j).map(|k| l[i][k] * l[j][k]).sum::<f64>();
            l[i][j] = if i == j {
                value.sqrt()
            } else {
                value / l[j][j]
            };
            if !l[i][j].is_finite() || (i == j && l[i][j] <= 0.) {
                return None;
            }
        }
    }
    let mut y = [0.; N];
    for i in 0..N {
        y[i] = (b[i] - (0..i).map(|j| l[i][j] * y[j]).sum::<f64>()) / l[i][i];
    }
    let mut z = [0.; N];
    for i in (0..N).rev() {
        z[i] = (y[i] - (i + 1..N).map(|j| l[j][i] * z[j]).sum::<f64>()) / l[i][i];
    }
    let solution = std::array::from_fn(|i| z[i] / scale[i]);
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
