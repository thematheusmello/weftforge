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
    fn deformation(&self, x: &[Vec3]) -> (Vec3, Vec3) {
        // Relative positions avoid cancellation when the garment is translated far from the origin.
        let b = sub(x[self.vertices[1]], x[self.vertices[0]]);
        let c = sub(x[self.vertices[2]], x[self.vertices[0]]);
        (
            add(mul(b, self.warp[1]), mul(c, self.warp[2])),
            add(mul(b, self.weft[1]), mul(c, self.weft[2])),
        )
    }
    pub fn stretch_error(&self, x: &[Vec3]) -> f64 {
        let (w, f) = self.deformation(x);
        (length(w) - 1.).abs().max((length(f) - 1.).abs())
    }
    pub fn solve(&self, x: &mut [Vec3], inverse_mass: &[f64], dt: f64, lambda: &mut [f64; 3]) {
        for (kind, lambda) in lambda.iter_mut().enumerate() {
            let (w, f) = self.deformation(x);
            let (value, gradient) = match kind {
                0 => (0.5 * (dot(w, w) - 1.), self.warp.map(|q| mul(w, q))),
                1 => (0.5 * (dot(f, f) - 1.), self.weft.map(|q| mul(f, q))),
                _ => (
                    dot(w, f),
                    std::array::from_fn(|i| add(mul(f, self.warp[i]), mul(w, self.weft[i]))),
                ),
            };
            project(
                &self.vertices,
                &gradient,
                value,
                self.stiffness[kind].recip() / (dt * dt),
                lambda,
                x,
                inverse_mass,
            );
        }
    }
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
    lambda: &mut f64,
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
        lambda,
        x,
        inverse_mass,
    );
}
