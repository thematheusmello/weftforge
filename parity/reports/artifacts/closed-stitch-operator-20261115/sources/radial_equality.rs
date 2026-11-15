#![allow(dead_code)]

use weftforge_sim::{Material, Vec3};
use math::*;
include!("stitch-types.rs");

mod math {
    include!("math.rs");
}
mod before {
    include!("base-constraints.rs");
}
mod after {
    include!("current-constraints.rs");
}

#[test]
fn unchanged_radial_paths_are_bit_identical_to_base() {
    let endpoints = [
        (WeightedPoint::vertex(0), WeightedPoint::vertex(1)),
        (
            WeightedPoint { vertices: [0, 1], weights: [0.25, 0.75] },
            WeightedPoint { vertices: [1, 2], weights: [0.5, 0.5] },
        ),
        (
            WeightedPoint { vertices: [0, 0], weights: [0.25, 0.75] },
            WeightedPoint { vertices: [0, 1], weights: [0.25, 0.75] },
        ),
    ];
    let mut projections = 0;
    for (a, b) in endpoints {
        for (target, compliance) in [
            (0., 0.),
            (f64::from_bits(1), 0.),
            (f64::from_bits(1), 0.125),
            (1., 0.),
            (1., 0.125),
        ] {
            let stitch = Stitch { a, b, compliance };
            for initial in [
                [[3., 4., 0.], [0.; 3], [1., -2., 1.]],
                [[0.; 3]; 3],
                [[1e-15, 0., 0.], [0.; 3], [0.; 3]],
            ] {
                for inverse_masses in [[0.5, 0.25, 0.], [0.; 3], [1.; 3]] {
                    for dt in [0.5, 1e-200] {
                        let mut old_x = initial;
                        let mut new_x = initial;
                        let mut old_lambda = 0.25;
                        let mut new_lambda = [0.25, 7., -8.];
                        for _ in 0..3 {
                            before::solve_stitch(stitch, target, &mut old_x, &inverse_masses, dt, &mut old_lambda);
                            after::solve_stitch(stitch, target, &mut new_x, &inverse_masses, dt, &mut new_lambda);
                            assert_eq!(
                                old_x.into_iter().flatten().map(f64::to_bits).collect::<Vec<_>>(),
                                new_x.into_iter().flatten().map(f64::to_bits).collect::<Vec<_>>(),
                            );
                            assert_eq!(old_lambda.to_bits(), new_lambda[0].to_bits());
                            assert_eq!(&new_lambda[1..], &[7., -8.]);
                            projections += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(projections, 810);
    println!("{projections} bit-identical radial projections from exact base/current sources: positive smallest/normal targets; hard/soft compliance; vertex/shared/repeated weights; free/mixed/all-pinned masses; normal/zero/tiny gaps; finite/nonfinite denominator guards; carried scalar multipliers");
}
