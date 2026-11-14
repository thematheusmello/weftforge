use crate::Vec3;

pub(crate) fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub(crate) fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub(crate) fn mul(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
pub(crate) fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub(crate) fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub(crate) fn length(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}
pub(crate) fn unit(a: Vec3) -> Vec3 {
    let n = length(a);
    if n > 1e-14 {
        mul(a, n.recip())
    } else {
        [0.; 3]
    }
}
pub(crate) fn finite(a: Vec3) -> bool {
    a.into_iter().all(f64::is_finite)
}
pub(crate) fn bary_point(x: &[Vec3], tri: [usize; 3], b: Vec3) -> Vec3 {
    add(
        add(mul(x[tri[0]], b[0]), mul(x[tri[1]], b[1])),
        mul(x[tri[2]], b[2]),
    )
}

// Closest-point Voronoi regions, including triangle edges and corners.
pub(crate) fn closest_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> (Vec3, Vec3) {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0. && d2 <= 0. {
        return (a, [1., 0., 0.]);
    }
    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0. && d4 <= d3 {
        return (b, [0., 1., 0.]);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0. && d1 >= 0. && d3 <= 0. {
        let v = d1 / (d1 - d3);
        return (add(a, mul(ab, v)), [1. - v, v, 0.]);
    }
    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0. && d5 <= d6 {
        return (c, [0., 0., 1.]);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0. && d2 >= 0. && d6 <= 0. {
        let w = d2 / (d2 - d6);
        return (add(a, mul(ac, w)), [1. - w, 0., w]);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0. && (d4 - d3) >= 0. && (d5 - d6) >= 0. {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return (add(b, mul(sub(c, b), w)), [0., 1. - w, w]);
    }
    let sum = va + vb + vc;
    if sum.abs() < 1e-28 {
        return (a, [1., 0., 0.]);
    }
    let v = vb / sum;
    let w = vc / sum;
    (add(a, add(mul(ab, v), mul(ac, w))), [1. - v - w, v, w])
}
