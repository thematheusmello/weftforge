use crate::Vec3;

pub(crate) fn signed_strains(x: [Vec3; 3], uv: [[f64; 2]; 3], grain: f64) -> [f64; 2] {
    let a = [uv[1][0] - uv[0][0], uv[1][1] - uv[0][1]];
    let b = [uv[2][0] - uv[0][0], uv[2][1] - uv[0][1]];
    let det = a[0] * b[1] - a[1] * b[0];
    let first: Vec3 = std::array::from_fn(|i| x[1][i] - x[0][i]);
    let second: Vec3 = std::array::from_fn(|i| x[2][i] - x[0][i]);
    // Form current F columns before rotating: cached rotated rest weights round differently.
    let u: Vec3 = std::array::from_fn(|i| (first[i] * b[1] - second[i] * a[1]) / det);
    let v: Vec3 = std::array::from_fn(|i| (second[i] * a[0] - first[i] * b[0]) / det);
    let (sin, cos) = grain.sin_cos();
    let warp: Vec3 = std::array::from_fn(|i| cos * u[i] + sin * v[i]);
    let weft: Vec3 = std::array::from_fn(|i| -sin * u[i] + cos * v[i]);
    [warp, weft].map(|axis| compensated_sum(axis.map(|q| q * q)).sqrt() - 1.)
}

fn compensated_sum(terms: Vec3) -> f64 {
    let mut sum = 0.;
    let mut correction = 0.;
    // Neumaier compensation sums the three rounded squares without a plain left fold.
    for value in terms {
        let next = sum + value;
        if !next.is_finite() {
            return next;
        }
        correction += if sum.abs() >= value.abs() {
            (sum - next) + value
        } else {
            (value - next) + sum
        };
        sum = next;
    }
    sum + correction
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_squares_are_compensated_and_nonfinite_sums_stay_nonfinite() {
        let small = 2_f64.powi(-53);
        assert_eq!(1. + small + small, 1.);
        assert_eq!(compensated_sum([1., small, small]), 1. + 2_f64.powi(-52));
        assert!(compensated_sum([f64::MAX, f64::MAX, 0.]).is_infinite());
        assert!(compensated_sum([f64::NAN, 0., 0.]).is_nan());
    }
}
