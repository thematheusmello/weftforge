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
        self.analyze_contacts(x, &|| false, false)
            .expect("A non-cancelling callback cannot interrupt contact analysis")
    }

    pub(crate) fn initial_contact_errors_cancellable(
        &self,
        x: &[Vec3],
        cancelled: &impl Fn() -> bool,
    ) -> Result<(f64, usize), SimulationError> {
        self.analyze_contacts(x, cancelled, true)
    }

    fn analyze_contacts(
        &self,
        x: &[Vec3],
        cancelled: &impl Fn() -> bool,
        initial: bool,
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
        let hierarchy = if initial {
            Some(TriangleHierarchy::new(&bounds, cancelled)?)
        } else {
            None
        };
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
            let mut consider = |i: usize| {
                let tri = self.input.triangles[i];
                if tri.iter().any(|i| self.excluded[v].contains(i))
                    || !overlaps(&bounds[i], x[v], x[v])
                {
                    return;
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
            };
            if let Some(hierarchy) = &hierarchy {
                hierarchy.visit(0, x[v], cancelled, &mut consider)?;
            } else {
                for i in 0..self.input.triangles.len() {
                    if i % CANCELLATION_CHECK_INTERVAL == 0 {
                        check_cancel(cancelled)?;
                    }
                    consider(i);
                }
            }
        }
        Ok((maximum, count))
    }
}

type Bounds = [Vec3; 2];

struct TriangleHierarchy {
    indices: Vec<usize>,
    nodes: Vec<TriangleNode>,
}

struct TriangleNode {
    bounds: Bounds,
    range: std::ops::Range<usize>,
    children: Option<[usize; 2]>,
}

impl TriangleHierarchy {
    fn new(bounds: &[Bounds], cancelled: &impl Fn() -> bool) -> Result<Self, SimulationError> {
        let indices = (0..bounds.len())
            .map(|i| {
                if i % CANCELLATION_CHECK_INTERVAL == 0 {
                    check_cancel(cancelled)?;
                }
                Ok(i)
            })
            .collect::<Result<_, SimulationError>>()?;
        let mut hierarchy = Self {
            indices,
            nodes: Vec::new(),
        };
        hierarchy.build(0..bounds.len(), bounds, cancelled)?;
        Ok(hierarchy)
    }

    fn build(
        &mut self,
        range: std::ops::Range<usize>,
        bounds: &[Bounds],
        cancelled: &impl Fn() -> bool,
    ) -> Result<usize, SimulationError> {
        check_cancel(cancelled)?;
        let mut combined = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
        for (i, &triangle) in self.indices[range.clone()].iter().enumerate() {
            if i % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(cancelled)?;
            }
            for axis in 0..3 {
                combined[0][axis] = combined[0][axis].min(bounds[triangle][0][axis]);
                combined[1][axis] = combined[1][axis].max(bounds[triangle][1][axis]);
            }
        }
        let node = self.nodes.len();
        self.nodes.push(TriangleNode {
            bounds: combined,
            range: range.clone(),
            children: None,
        });
        if range.len() > 8 {
            let axis = (0..3)
                .max_by(|&a, &b| {
                    (combined[1][a] - combined[0][a]).total_cmp(&(combined[1][b] - combined[0][b]))
                })
                .unwrap();
            partition_triangles(&mut self.indices[range.clone()], bounds, axis, cancelled)?;
            let middle = range.start + range.len() / 2;
            let left = self.build(range.start..middle, bounds, cancelled)?;
            let right = self.build(middle..range.end, bounds, cancelled)?;
            self.nodes[node].children = Some([left, right]);
        }
        Ok(node)
    }

    fn visit(
        &self,
        node: usize,
        point: Vec3,
        cancelled: &impl Fn() -> bool,
        consider: &mut impl FnMut(usize),
    ) -> Result<(), SimulationError> {
        check_cancel(cancelled)?;
        let node = &self.nodes[node];
        if overlaps(&node.bounds, point, point) {
            if let Some(children) = node.children {
                for child in children {
                    self.visit(child, point, cancelled, consider)?;
                }
            } else {
                for &triangle in &self.indices[node.range.clone()] {
                    consider(triangle);
                }
            }
        }
        Ok(())
    }
}

fn partition_triangles(
    indices: &mut [usize],
    bounds: &[Bounds],
    axis: usize,
    cancelled: &impl Fn() -> bool,
) -> Result<(), SimulationError> {
    let middle = indices.len() / 2;
    let (mut start, mut end) = (0, indices.len());
    let mut work = 0;
    // shortcut: capped selection can yield coarse splits, upgrade if measured pruning regresses.
    while end - start > 1 && work < indices.len().saturating_mul(4) {
        let pivot = bounds[indices[start + (end - start) / 2]][0][axis];
        let (mut lower, mut i, mut upper) = (start, start, end);
        while i < upper {
            if work % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(cancelled)?;
            }
            work += 1;
            match bounds[indices[i]][0][axis].total_cmp(&pivot) {
                std::cmp::Ordering::Less => {
                    indices.swap(i, lower);
                    lower += 1;
                    i += 1;
                }
                std::cmp::Ordering::Equal => i += 1,
                std::cmp::Ordering::Greater => {
                    upper -= 1;
                    indices.swap(i, upper);
                }
            }
        }
        if middle < lower {
            end = lower;
        } else if middle >= upper {
            start = upper;
        } else {
            break;
        }
    }
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn separated_bounds() -> Vec<Bounds> {
        (0..4096)
            .map(|i| [[i as f64 * 3., 0., 0.], [i as f64 * 3. + 1., 1., 1.]])
            .collect()
    }

    #[test]
    fn hierarchy_is_conservative_and_prunes_separated_bounds() {
        let mut bounds = separated_bounds();
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        let mut candidates = 0;
        hierarchy
            .visit(0, [0.5; 3], &|| false, &mut |_| candidates += 1)
            .unwrap();
        assert!(
            candidates <= 8,
            "separated triangles should only visit one leaf"
        );
        bounds.extend([[[0.; 3], [1.; 3]]; 17]);
        bounds.push([[-f64::MAX; 3], [f64::MAX; 3]]);
        bounds.push([[f64::NEG_INFINITY; 3], [f64::INFINITY; 3]]);
        bounds.reverse();
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        for point in [
            [-0.; 3],
            [0.5; 3],
            [1.; 3],
            [123.5, 0.5, 0.5],
            [12286., 1., 1.],
            [f64::MAX; 3],
            [-f64::MAX; 3],
        ] {
            let expected: Vec<_> = bounds
                .iter()
                .enumerate()
                .filter_map(|(i, bounds)| overlaps(bounds, point, point).then_some(i))
                .collect();
            let mut actual = Vec::new();
            hierarchy
                .visit(0, point, &|| false, &mut |i| {
                    if overlaps(&bounds[i], point, point) {
                        actual.push(i);
                    }
                })
                .unwrap();
            actual.sort_unstable();
            assert_eq!(actual, expected);
        }
        let bounds: Vec<_> = (0..4096)
            .map(|i| {
                let radius = ((i * 4051) % 4096 + 1) as f64;
                [[-radius; 3], [radius; 3]]
            })
            .collect();
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        let mut candidates = Vec::new();
        hierarchy
            .visit(0, [0.; 3], &|| false, &mut |i| candidates.push(i))
            .unwrap();
        candidates.sort_unstable();
        assert_eq!(candidates, (0..bounds.len()).collect::<Vec<_>>());
    }

    #[test]
    fn hierarchy_build_and_partition_cancel_during_work() {
        let bounds = separated_bounds();
        let polls = Cell::new(0);
        TriangleHierarchy::new(&bounds, &|| {
            polls.set(polls.get() + 1);
            false
        })
        .unwrap();
        for stop_at in [1, 2, polls.get() / 2, polls.get()] {
            let calls = Cell::new(0);
            let result = TriangleHierarchy::new(&bounds, &|| {
                calls.set(calls.get() + 1);
                assert!(calls.get() <= stop_at);
                calls.get() == stop_at
            });
            assert!(matches!(result, Err(SimulationError::Cancelled)));
            assert_eq!(calls.get(), stop_at);
        }
        let mut indices: Vec<_> = (0..bounds.len()).collect();
        let calls = Cell::new(0);
        assert_eq!(
            partition_triangles(&mut indices, &bounds, 0, &|| {
                calls.set(calls.get() + 1);
                assert!(calls.get() <= 2);
                calls.get() == 2
            }),
            Err(SimulationError::Cancelled)
        );
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn hierarchy_query_cancels_when_all_bounds_overlap() {
        let bounds = vec![[[0.; 3], [1.; 3]]; CANCELLATION_CHECK_INTERVAL + 1];
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        let polls = Cell::new(0);
        hierarchy
            .visit(
                0,
                [0.5; 3],
                &|| {
                    polls.set(polls.get() + 1);
                    false
                },
                &mut |_| {},
            )
            .unwrap();
        for stop_at in [2, polls.get() / 2] {
            let calls = Cell::new(0);
            let mut candidates = 0;
            assert_eq!(
                hierarchy.visit(
                    0,
                    [0.5; 3],
                    &|| {
                        calls.set(calls.get() + 1);
                        assert!(calls.get() <= stop_at);
                        calls.get() == stop_at
                    },
                    &mut |_| candidates += 1,
                ),
                Err(SimulationError::Cancelled)
            );
            assert_eq!(calls.get(), stop_at);
            assert!(candidates < bounds.len());
        }
    }
}
