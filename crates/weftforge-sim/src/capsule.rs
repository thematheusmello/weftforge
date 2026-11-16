use crate::{SimulationError, Vec3, check_cancel, math::*};

fn norm(v: Vec3) -> f64 {
    v[0].hypot(v[1]).hypot(v[2])
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Closest {
    pub bary: Vec3,
    pub axis_fraction: f64,
    pub distance: f64,
    pub uncertain: bool,
}

// The minimum is on a closed edge, an axis endpoint's face projection, or a face crossing.
// shortcut: static occupancy misses between-step crossings, upgrade with continuous capsule contact.
pub(crate) fn closest(triangle: [Vec3; 3], a: Vec3, b: Vec3) -> Closest {
    closest_cancellable(triangle, a, b, &|| false)
        .expect("A non-cancelling callback cannot interrupt capsule contact")
}

pub(crate) fn closest_cancellable(
    triangle: [Vec3; 3],
    a: Vec3,
    b: Vec3,
    cancelled: &impl Fn() -> bool,
) -> Result<Closest, SimulationError> {
    check_cancel(cancelled)?;
    let origin = triangle[0];
    let relative = [
        sub(triangle[1], origin),
        sub(triangle[2], origin),
        sub(a, origin),
        sub(b, origin),
    ];
    let scale = relative
        .into_iter()
        .flatten()
        .map(f64::abs)
        .fold(0., f64::max);
    if !scale.is_finite() {
        return Ok(Closest {
            bary: [1., 0., 0.],
            axis_fraction: 0.,
            distance: 0.,
            uncertain: true,
        });
    }
    if scale == 0. {
        return Ok(Closest {
            bary: [1., 0., 0.],
            axis_fraction: 0.,
            distance: 0.,
            uncertain: false,
        });
    }
    let [u, v, a, b] = relative.map(|p| p.map(|x| x / scale));
    let triangle = [[0.; 3], u, v];
    let axis = sub(b, a);
    let mut best = Closest {
        bary: [1., 0., 0.],
        axis_fraction: axis_fraction(triangle[0], a, b),
        distance: f64::INFINITY,
        uncertain: false,
    };
    let consider = |best: &mut Closest, bary: Vec3, fraction: f64| {
        let point = add(mul(u, bary[1]), mul(v, bary[2]));
        let distance = norm(sub(point, add(a, mul(axis, fraction))));
        if distance < best.distance {
            *best = Closest {
                bary,
                axis_fraction: fraction,
                distance,
                uncertain: false,
            };
        }
    };
    for [i, j] in [[0, 1], [1, 2], [2, 0]] {
        check_cancel(cancelled)?;
        let p = triangle[i];
        let q = triangle[j];
        let edge = sub(q, p);
        let mut offer = |t: f64, s: f64| {
            let mut bary = [0.; 3];
            bary[i] = 1. - t;
            bary[j] = t;
            consider(&mut best, bary, s);
        };
        check_cancel(cancelled)?;
        offer(0., axis_fraction(p, a, b));
        check_cancel(cancelled)?;
        offer(1., axis_fraction(q, a, b));
        check_cancel(cancelled)?;
        offer(axis_fraction(a, p, q), 0.);
        check_cancel(cancelled)?;
        offer(axis_fraction(b, p, q), 1.);
        check_cancel(cancelled)?;
        let normal = cross(edge, axis);
        let normal_length = norm(normal);
        if normal_length > 0. {
            let normal = normal.map(|x| x / normal_length);
            let delta = sub(p, a);
            let t = dot(cross(axis, delta), normal) / normal_length;
            let s = dot(cross(edge, delta), normal) / normal_length;
            if (0. ..=1.).contains(&t) && (0. ..=1.).contains(&s) {
                offer(t, s);
            }
        }
    }
    check_cancel(cancelled)?;
    let normal = cross(u, v);
    let normal_length = norm(normal);
    let normal_squared = dot(normal, normal);
    let longest = [u, v, sub(v, u)].map(norm).into_iter().fold(0., f64::max);
    let longest_squared = longest * longest;
    let underflow =
        (longest > 0. && longest_squared == 0.) || (normal_length > 0. && normal_squared == 0.);
    if underflow {
        best.distance = 0.;
        best.uncertain = true;
        return Ok(best);
    }
    if normal_length <= 1e-7 * longest_squared {
        // Every face point is within this altitude of the longest closed edge.
        let rounding = 4.
            * f64::EPSILON
            * norm([
                (u[1] * v[2]).abs() + (u[2] * v[1]).abs(),
                (u[2] * v[0]).abs() + (u[0] * v[2]).abs(),
                (u[0] * v[1]).abs() + (u[1] * v[0]).abs(),
            ]);
        let altitude = if longest > 0. {
            (normal_length + rounding) / longest
        } else {
            0.
        };
        best.distance = (best.distance - altitude).max(0.) * scale;
        best.uncertain = normal_length > 0. || rounding > 0.;
        return Ok(best);
    }
    let barycentric = |p: Vec3| {
        let v_weight = dot(cross(p, v), normal) / normal_squared;
        let w_weight = dot(cross(u, p), normal) / normal_squared;
        let bary = [1. - v_weight - w_weight, v_weight, w_weight];
        bary.into_iter()
            .all(|weight| (0. ..=1.).contains(&weight))
            .then_some(bary)
    };
    for (p, fraction) in [(a, 0.), (b, 1.)] {
        check_cancel(cancelled)?;
        if let Some(bary) = barycentric(p) {
            consider(&mut best, bary, fraction);
        }
    }
    check_cancel(cancelled)?;
    let start = dot(a, normal);
    let change = dot(axis, normal);
    if change != 0. {
        let fraction = -start / change;
        if (0. ..=1.).contains(&fraction)
            && let Some(bary) = barycentric(add(a, mul(axis, fraction)))
        {
            consider(&mut best, bary, fraction);
        }
    }
    best.distance *= scale;
    Ok(best)
}

pub(crate) fn axis_fraction(p: Vec3, a: Vec3, b: Vec3) -> f64 {
    let axis = sub(b, a);
    let squared = dot(axis, axis);
    if squared > 0. {
        (dot(sub(p, a), axis) / squared).clamp(0., 1.)
    } else {
        0.
    }
}

fn normalized(v: Vec3) -> Option<Vec3> {
    let scale = v.into_iter().map(f64::abs).fold(0., f64::max);
    if scale == 0. || !scale.is_finite() {
        return None;
    }
    let v = v.map(|x| x / scale);
    Some(mul(v, length(v).recip()))
}

pub(crate) fn normal(
    closest: Closest,
    triangle: [Vec3; 3],
    previous: [Vec3; 3],
    a: Vec3,
    b: Vec3,
) -> Vec3 {
    let point = |triangle: [Vec3; 3]| {
        add(
            add(
                mul(triangle[0], closest.bary[0]),
                mul(triangle[1], closest.bary[1]),
            ),
            mul(triangle[2], closest.bary[2]),
        )
    };
    let axis = sub(b, a);
    let axis_point = add(a, mul(axis, closest.axis_fraction));
    let scale = [
        sub(triangle[1], triangle[0]),
        sub(triangle[2], triangle[0]),
        axis,
    ]
    .into_iter()
    .flatten()
    .map(f64::abs)
    .fold(0., f64::max);
    let tolerance = 64. * f64::EPSILON * scale;
    if closest.distance > tolerance
        && let Some(normal) = normalized(sub(point(triangle), axis_point))
    {
        return normal;
    }
    let axis_unit = normalized(axis);
    let radial = |direction: Vec3| {
        let direction = normalized(direction)?;
        if closest.axis_fraction > 0.
            && closest.axis_fraction < 1.
            && let Some(axis) = axis_unit
        {
            let radial = sub(direction, mul(axis, dot(direction, axis)));
            if length(radial) <= 64. * f64::EPSILON {
                return None;
            }
            return normalized(radial);
        }
        Some(direction)
    };
    let old_point = point(previous);
    let old_axis_point = add(a, mul(axis, axis_fraction(old_point, a, b)));
    let old_delta = sub(old_point, old_axis_point);
    let old_crossing = self::closest(previous, a, b).distance <= tolerance;
    if !old_crossing
        && norm(old_delta) > tolerance
        && let Some(normal) = radial(old_delta)
    {
        return normal;
    }
    // An already crossed face has no previous side; keep its recovery direction fixed for the substep.
    let face = if old_crossing { previous } else { triangle };
    if let Some(normal) = radial(cross(sub(face[1], face[0]), sub(face[2], face[0]))) {
        return normal;
    }
    if let Some(axis) = axis_unit {
        let coordinate = (0..3)
            .min_by(|&i, &j| axis[i].abs().total_cmp(&axis[j].abs()))
            .unwrap();
        let mut basis = [0.; 3];
        basis[coordinate] = 1.;
        return normalized(cross(axis, basis)).unwrap();
    }
    [1., 0., 0.]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn face() -> [Vec3; 3] {
        [[-2., -2., 0.], [2., -2., 0.], [0., 2., 0.]]
    }

    #[test]
    fn controlled_capsule_features_preserve_results_and_abort_at_each_task() {
        for triangle in [face(), [[-1., 0., 0.], [0.; 3], [1., 0., 0.]], [[0.; 3]; 3]] {
            let (a, b) = ([0., 0., -1.], [0., 0., 1.]);
            let calls = Cell::new(0);
            let complete = closest_cancellable(triangle, a, b, &|| {
                calls.set(calls.get() + 1);
                false
            })
            .unwrap();
            let ordinary = closest(triangle, a, b);
            assert_eq!(format!("{complete:?}"), format!("{ordinary:?}"));
            assert_eq!(complete.distance, 0.);
            assert_eq!(calls.get(), if triangle == face() { 23 } else { 20 });
            for stop_at in 1..=calls.get() {
                let polls = Cell::new(0);
                let result = closest_cancellable(triangle, a, b, &|| {
                    polls.set(polls.get() + 1);
                    assert!(polls.get() <= stop_at);
                    polls.get() == stop_at
                });
                assert!(matches!(result, Err(SimulationError::Cancelled)));
                assert_eq!(polls.get(), stop_at);
            }
        }
    }

    fn check(triangle: [Vec3; 3], a: Vec3, b: Vec3, expected: f64) -> Closest {
        let result = closest(triangle, a, b);
        assert!(
            (result.distance - expected).abs() < 1e-12,
            "{result:?}, expected {expected}"
        );
        assert!(result.bary.into_iter().all(|w| (0. ..=1.).contains(&w)));
        assert!((result.bary.into_iter().sum::<f64>() - 1.).abs() < 1e-14);
        assert!((0. ..=1.).contains(&result.axis_fraction));
        assert!(!result.uncertain);
        result
    }

    #[test]
    fn exact_closest_features_include_caps_faces_edges_and_vertices() {
        let first_cap = check(face(), [0., 0., 0.5], [0., 0., 2.], 0.5);
        assert_eq!(first_cap.axis_fraction, 0.);
        assert_eq!(first_cap.bary, [0.25, 0.25, 0.5]);
        assert_eq!(
            check(face(), [0., 0., 2.], [0., 0., 0.5], 0.5).axis_fraction,
            1.
        );
        let crossing = check(face(), [0., 0., -1.], [0., 0., 1.], 0.);
        assert_eq!(crossing.bary, [0.25, 0.25, 0.5]);
        assert_eq!(crossing.axis_fraction, 0.5);
        let edge = [[-1., 0., 0.], [1., 0., 0.], [0., -2., 0.]];
        let closest = check(edge, [0., 1., -1.], [0., 1., 1.], 1.);
        assert_eq!(closest.bary, [0.5, 0.5, 0.]);
        assert_eq!(closest.axis_fraction, 0.5);
        let vertex = check(
            [[0.; 3], [1., 0., 0.], [0., 1., 0.]],
            [2., 0., 1.],
            [2., 1., 1.],
            2_f64.sqrt(),
        );
        assert_eq!(vertex.bary, [0., 1., 0.]);
        check(face(), [0., 0., 0.25], [0., 0., 0.25], 0.25);
    }

    #[test]
    fn parallel_and_coplanar_segments_reach_face_between_outside_endpoints() {
        for height in [0., 0.1] {
            let a = [-4., 0., height];
            let b = [4., 0., height];
            check(face(), a, b, height);
        }
        let triangle = [[-1., 0., 0.1], [1., 0., 0.1], [0., 2., 0.1]];
        let a = [0., -1., 0.];
        let b = [0., 1., 0.];
        for vertex in triangle {
            let fraction = axis_fraction(vertex, a, b);
            assert!(length(sub(vertex, add(a, mul(sub(b, a), fraction)))) > 0.25);
        }
        check(triangle, a, b, 0.1);
    }

    #[test]
    fn tiny_and_degenerate_triangles_keep_closed_surface_coverage() {
        let tiny = face().map(|v| mul(v, 1e-18));
        let result = closest(tiny, [0., 0., 0.5e-18], [0., 0., 2e-18]);
        assert!((result.distance - 0.5e-18).abs() < 1e-32);
        assert!(!result.uncertain);
        let underflow = closest(face().map(|v| mul(v, 1e-170)), [0., 0., -1.], [0., 0., 1.]);
        assert!(underflow.uncertain);
        assert_eq!(underflow.distance, 0.);
        let line = [[-1., 0., 0.], [0.; 3], [1., 0., 0.]];
        check(line, [0., 1., 0.], [0., 1., 0.], 1.);
        check(line, [0., -1., 0.], [0., 1., 0.], 0.);
        check([[0.; 3]; 3], [0., 1., 0.], [0., 2., 0.], 1.);
        let skinny = [[-1., 0., 0.], [1., 0., 0.], [0., 1e-10, 0.]];
        let on_face = closest(skinny, [0., 0.5e-10, 0.], [0., 0.5e-10, 0.]);
        assert!(on_face.uncertain);
        assert_eq!(on_face.distance, 0.);
        let above = closest(skinny, [0., 0.5e-10, 1e-6], [0., 0.5e-10, 1e-6]);
        assert!(above.uncertain);
        assert!(above.distance <= 1e-6 && above.distance >= 1e-6 - 1e-10);
    }

    #[test]
    fn distance_and_barycentric_gradients_survive_rigid_transforms() {
        let triangle = [[-1., 0., 0.], [1., 0., 0.], [0., -2., 0.]];
        let a = [0., 1., -1.];
        let b = [0., 1., 1.];
        let base = check(triangle, a, b, 1.);
        let transform = |p: Vec3| [4. + p[2], -5. + p[0], 6. + p[1]];
        let rotated = check(triangle.map(transform), transform(a), transform(b), 1.);
        assert_eq!(rotated.bary, base.bary);
        assert_eq!(rotated.axis_fraction, base.axis_fraction);
        for (triangle, a, b) in [(triangle, a, b), (face(), [0., 0., 0.5], [0., 0., 2.])] {
            let feature = closest(triangle, a, b);
            let direction = normal(feature, triangle, triangle, a, b);
            for vertex in 0..3 {
                for axis in 0..3 {
                    let mut plus = triangle;
                    let mut minus = triangle;
                    plus[vertex][axis] += 1e-6;
                    minus[vertex][axis] -= 1e-6;
                    let numeric =
                        (closest(plus, a, b).distance - closest(minus, a, b).distance) / 2e-6;
                    assert!(
                        (numeric - feature.bary[vertex] * direction[axis]).abs() < 1e-8,
                        "vertex {vertex} axis {axis}: {numeric} vs {}",
                        feature.bary[vertex] * direction[axis]
                    );
                }
            }
        }
    }

    #[test]
    fn zero_distance_fallback_is_deterministic_finite_and_radial() {
        let a = [0., 0., -1.];
        let b = [0., 0., 1.];
        let feature = closest(face(), a, b);
        let direction = normal(feature, face(), face(), a, b);
        assert!(finite(direction));
        assert!((length(direction) - 1.).abs() < 1e-14);
        assert!(dot(direction, sub(b, a)).abs() < 1e-14);
        assert_eq!(direction, normal(feature, face(), face(), a, b));
        let previous = face().map(|p| add(p, [3., 0., 0.]));
        assert_eq!(normal(feature, face(), previous, a, b), [1., 0., 0.]);
        for triangle in [face(), [[0.; 3]; 3], [[-1., 0., 0.], [0.; 3], [1., 0., 0.]]] {
            for (a, b) in [(a, b), ([0.; 3], [0.; 3])] {
                let feature = closest(triangle, a, b);
                let direction = normal(feature, triangle, triangle, a, b);
                assert!(finite(direction));
                assert!((length(direction) - 1.).abs() < 1e-14);
            }
        }
    }

    #[test]
    fn saved_original_and_harbor_surface_witnesses_are_detected() {
        let a = [-0.22, 1.4, 0.];
        let b = [-0.55, 1.22, 0.];
        // Original grain triangle 140, Harbor A1164 and B678; static tolerance preceded implementation.
        for (triangle, expected_gap) in [
            (
                [
                    [
                        -0.24686485358149718,
                        1.3093502323883635,
                        0.01316285089646587,
                    ],
                    [
                        -0.22305433617792467,
                        1.4554418244325111,
                        -0.06610868372179386,
                    ],
                    [
                        -0.2536165879127128,
                        1.4433189136647577,
                        -0.06266991697418287,
                    ],
                ],
                -0.043686905228137965,
            ),
            (
                [
                    [
                        -0.36499002170987355,
                        1.3380459289852806,
                        -0.06631599466997012,
                    ],
                    [
                        -0.33784653405030557,
                        1.2791794623640007,
                        -0.04647787572455338,
                    ],
                    [
                        -0.31275320781459137,
                        1.297351447760658,
                        -0.050442452917365724,
                    ],
                ],
                -0.00900970054625062,
            ),
            (
                [
                    [
                        -0.24087639890490373,
                        1.3980895801621886,
                        -0.06751146614747697,
                    ],
                    [
                        -0.22294241368554168,
                        1.332577821894836,
                        -0.03592648619927106,
                    ],
                    [
                        -0.1808996461827904,
                        1.3956218661164295,
                        -0.07290234504561213,
                    ],
                ],
                -0.010672413745369747,
            ),
        ] {
            let result = closest(triangle, a, b);
            assert!(!result.uncertain);
            assert!(
                (result.distance - 0.068 - expected_gap).abs() < 1e-10,
                "{result:?}"
            );
        }
        for (triangle, a, b, centroid_gap) in [
            (
                [
                    [
                        -0.24686485358149718,
                        1.3093502323883635,
                        0.01316285089646587,
                    ],
                    [
                        -0.1689640238811713,
                        1.4444630228367277,
                        -0.054743993372153195,
                    ],
                    [
                        -0.22305433617792467,
                        1.4554418244325111,
                        -0.06610868372179386,
                    ],
                ],
                a,
                b,
                -0.03128991384122659,
            ),
            (
                [
                    [
                        -0.31275320781459137,
                        1.297351447760658,
                        -0.050442452917365724,
                    ],
                    [
                        -0.3014205529833128,
                        1.3070485058919536,
                        -0.05302813349857248,
                    ],
                    [-0.343130035279275, 1.3582161672441617, -0.06596114022068185],
                ],
                a,
                b,
                -0.007385040743506673,
            ),
            (
                [
                    [0.5678495709713409, 1.2809037088121504, 0.027964953791314275],
                    [0.5840722581551911, 1.2367960219204888, 0.0564001318425458],
                    [
                        0.588729169927744,
                        1.2758515025027506,
                        -0.0021751052141720188,
                    ],
                ],
                [0.22, 1.4, 0.],
                [0.55, 1.22, 0.],
                -0.007622733850101965,
            ),
        ] {
            let centroid = mul(add(add(triangle[0], triangle[1]), triangle[2]), 1. / 3.);
            let axis_point = add(a, mul(sub(b, a), axis_fraction(centroid, a, b)));
            assert!((length(sub(centroid, axis_point)) - 0.068 - centroid_gap).abs() < 1e-10);
            assert!(closest(triangle, a, b).distance - 0.068 <= centroid_gap + 1e-10);
        }
    }
}
