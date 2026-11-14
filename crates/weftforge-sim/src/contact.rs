use crate::{
    CANCELLATION_CHECK_INTERVAL, Cloth, Collider, SimulationError, Vec3, check_cancel, math::*,
};

impl Cloth {
    pub(crate) fn project_colliders(&self, x: &mut [Vec3], previous: &[Vec3]) -> usize {
        let mut contacts = 0;
        for i in 0..x.len() {
            if self.inverse_masses[i] == 0. {
                continue;
            }
            for &collider in &self.input.colliders {
                if let Some((normal, depth)) = collider_contact(collider, x[i], self.thickness[i]) {
                    x[i] = add(x[i], mul(normal, depth));
                    let displacement = sub(x[i], previous[i]);
                    let tangent = sub(displacement, mul(normal, dot(displacement, normal)));
                    let distance = length(tangent);
                    if distance > 1e-14 {
                        x[i] = sub(
                            x[i],
                            mul(tangent, (self.friction[i] * depth / distance).min(1.)),
                        );
                    }
                    contacts += 1;
                }
            }
        }
        contacts
    }

    pub(crate) fn project_cloth(
        &self,
        x: &mut [Vec3],
        previous: &[Vec3],
        cancelled: &impl Fn() -> bool,
    ) -> Result<usize, SimulationError> {
        let max_thickness = self.thickness.iter().copied().fold(0., f64::max);
        let bounds = triangle_bounds(&self.input.triangles, x, previous, 2. * max_thickness);
        let mut contacts = 0;
        for v in 0..x.len() {
            if v % 32 == 0 && cancelled() {
                return Err(SimulationError::Cancelled);
            }
            for (&tri, bounds) in self.input.triangles.iter().zip(&bounds) {
                if tri.iter().any(|i| self.excluded[v].contains(i))
                    || !overlaps(bounds, x[v], previous[v])
                {
                    continue;
                }
                let gap = self.thickness[v]
                    + tri
                        .map(|i| self.thickness[i])
                        .into_iter()
                        .fold(0., f64::max);
                let Some((normal, bary, depth)) = cloth_contact(v, tri, x, previous, gap, true)
                else {
                    continue;
                };
                let weights = [
                    self.inverse_masses[v],
                    self.inverse_masses[tri[0]],
                    self.inverse_masses[tri[1]],
                    self.inverse_masses[tri[2]],
                ];
                let denominator = weights[0]
                    + weights[1] * bary[0] * bary[0]
                    + weights[2] * bary[1] * bary[1]
                    + weights[3] * bary[2] * bary[2];
                if denominator <= 1e-20 {
                    continue;
                }
                let impulse = depth / denominator;
                x[v] = add(x[v], mul(normal, weights[0] * impulse));
                for j in 0..3 {
                    x[tri[j]] = sub(x[tri[j]], mul(normal, weights[j + 1] * bary[j] * impulse));
                }
                let relative = sub(
                    sub(x[v], previous[v]),
                    sub(bary_point(x, tri, bary), bary_point(previous, tri, bary)),
                );
                let tangent = sub(relative, mul(normal, dot(relative, normal)));
                let tangent_length = length(tangent);
                if tangent_length > 1e-14 {
                    let friction = (self.friction[v]
                        + tri.map(|i| self.friction[i]).into_iter().sum::<f64>() / 3.)
                        * 0.5;
                    let amount = (friction * impulse).min(tangent_length / denominator);
                    let direction = mul(tangent, tangent_length.recip());
                    x[v] = sub(x[v], mul(direction, weights[0] * amount));
                    for j in 0..3 {
                        x[tri[j]] =
                            add(x[tri[j]], mul(direction, weights[j + 1] * bary[j] * amount));
                    }
                }
                contacts += 1;
            }
        }
        Ok(contacts)
    }

    pub(crate) fn contact_errors(&self, x: &[Vec3]) -> (f64, usize) {
        self.contact_errors_cancellable(x, &|| false)
            .expect("A non-cancelling callback cannot interrupt contact analysis")
    }

    pub(crate) fn contact_errors_cancellable(
        &self,
        x: &[Vec3],
        cancelled: &impl Fn() -> bool,
    ) -> Result<(f64, usize), SimulationError> {
        let mut max_thickness = 0_f64;
        for (i, &thickness) in self.thickness.iter().enumerate() {
            if i % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(cancelled)?;
            }
            max_thickness = max_thickness.max(thickness);
        }
        let bounds = triangle_bounds_cancellable(
            &self.input.triangles,
            x,
            x,
            2. * max_thickness,
            cancelled,
        )?;
        let mut maximum = 0_f64;
        let mut count = 0;
        for v in 0..x.len() {
            check_cancel(cancelled)?;
            for (i, &collider) in self.input.colliders.iter().enumerate() {
                if i % CANCELLATION_CHECK_INTERVAL == 0 {
                    check_cancel(cancelled)?;
                }
                if let Some((_, depth)) = collider_contact(collider, x[v], self.thickness[v]) {
                    maximum = maximum.max(depth);
                    if depth > 1e-8 {
                        count += 1;
                    }
                }
            }
            for (i, (&tri, bounds)) in self.input.triangles.iter().zip(&bounds).enumerate() {
                if i % CANCELLATION_CHECK_INTERVAL == 0 {
                    check_cancel(cancelled)?;
                }
                if tri.iter().any(|i| self.excluded[v].contains(i)) || !overlaps(bounds, x[v], x[v])
                {
                    continue;
                }
                let gap = self.thickness[v]
                    + tri
                        .map(|i| self.thickness[i])
                        .into_iter()
                        .fold(0., f64::max);
                if let Some((_, _, depth)) = cloth_contact(v, tri, x, x, gap, false) {
                    maximum = maximum.max(depth);
                    if depth > 1e-8 {
                        count += 1;
                    }
                }
            }
        }
        Ok((maximum, count))
    }
}

type Bounds = [Vec3; 2];
fn triangle_bounds(
    triangles: &[[usize; 3]],
    x: &[Vec3],
    previous: &[Vec3],
    gap: f64,
) -> Vec<Bounds> {
    triangle_bounds_cancellable(triangles, x, previous, gap, &|| false)
        .expect("A non-cancelling callback cannot interrupt bounds construction")
}
fn triangle_bounds_cancellable(
    triangles: &[[usize; 3]],
    x: &[Vec3],
    previous: &[Vec3],
    gap: f64,
    cancelled: &impl Fn() -> bool,
) -> Result<Vec<Bounds>, SimulationError> {
    triangles
        .iter()
        .enumerate()
        .map(|(i, tri)| {
            if i % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(cancelled)?;
            }
            let mut bounds = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
            for &v in tri {
                for axis in 0..3 {
                    bounds[0][axis] = bounds[0][axis].min(x[v][axis].min(previous[v][axis]) - gap);
                    bounds[1][axis] = bounds[1][axis].max(x[v][axis].max(previous[v][axis]) + gap);
                }
            }
            Ok(bounds)
        })
        .collect()
}
fn overlaps(bounds: &Bounds, a: Vec3, b: Vec3) -> bool {
    (0..3).all(|axis| {
        a[axis].max(b[axis]) >= bounds[0][axis] && a[axis].min(b[axis]) <= bounds[1][axis]
    })
}

fn cloth_contact(
    v: usize,
    tri: [usize; 3],
    x: &[Vec3],
    previous: &[Vec3],
    gap: f64,
    swept: bool,
) -> Option<(Vec3, Vec3, f64)> {
    let [a, b, c] = tri.map(|i| x[i]);
    let (point, bary) = closest_triangle(x[v], a, b, c);
    let delta = sub(x[v], point);
    let distance = length(delta);
    let face_normal = unit(cross(sub(b, a), sub(c, a)));
    if length(face_normal) < 0.5 {
        return None;
    }
    let old_normal = unit(cross(
        sub(previous[tri[1]], previous[tri[0]]),
        sub(previous[tri[2]], previous[tri[0]]),
    ));
    let old_side = dot(
        sub(previous[v], bary_point(previous, tri, bary)),
        old_normal,
    );
    let side = dot(delta, face_normal);
    // Linearized swept face crossing also catches particles whose endpoint has passed beyond thickness.
    // shortcut: this is not full continuous deforming-triangle or edge-edge collision, upgrade before thin/high-speed production scenes.
    let crossed = swept && old_side * side < -1e-18 && (distance - side.abs()).abs() < 1e-8;
    if crossed {
        let normal = mul(face_normal, if old_side >= 0. { 1. } else { -1. });
        return Some((normal, bary, gap - dot(delta, normal)));
    }
    if distance >= gap {
        return None;
    }
    let normal = if distance > 1e-12 {
        mul(delta, distance.recip())
    } else {
        mul(face_normal, if old_side < 0. { -1. } else { 1. })
    };
    Some((normal, bary, gap - distance))
}

pub(crate) fn collider_contact(collider: Collider, p: Vec3, thickness: f64) -> Option<(Vec3, f64)> {
    match collider {
        Collider::Ground { height } => {
            let depth = height + thickness - p[1];
            (depth > 0.).then_some(([0., 1., 0.], depth))
        }
        Collider::Capsule { a, b, radius } => {
            let edge = sub(b, a);
            let edge_squared = dot(edge, edge);
            let t = if edge_squared > 1e-20 {
                (dot(sub(p, a), edge) / edge_squared).clamp(0., 1.)
            } else {
                0.
            };
            let delta = sub(p, add(a, mul(edge, t)));
            let distance = length(delta);
            let depth = radius + thickness - distance;
            if depth <= 0. {
                return None;
            }
            let normal = if distance > 1e-12 {
                mul(delta, distance.recip())
            } else {
                let reference = if edge[0].abs() < edge[1].abs() {
                    [1., 0., 0.]
                } else {
                    [0., 1., 0.]
                };
                let normal = unit(cross(edge, reference));
                if length(normal) > 0.5 {
                    normal
                } else {
                    [1., 0., 0.]
                }
            };
            Some((normal, depth))
        }
        Collider::Torso { center, radii } => {
            let d = sub(p, center);
            let r = radii.map(|r| r + thickness);
            let q = (0..3).map(|i| (d[i] / r[i]).powi(2)).sum::<f64>().sqrt();
            if q >= 1. {
                return None;
            }
            // shortcut: radial inflated-ellipsoid contact approximates a constant offset, replace with exact signed distance for calibrated avatars.
            let target = if q > 1e-12 {
                mul(d, q.recip())
            } else {
                [r[0], 0., 0.]
            };
            let correction = sub(target, d);
            let depth = length(correction);
            Some((unit(correction), depth))
        }
    }
}
