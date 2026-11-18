#[test]
fn soft_sewing_dual_preserves_nonconstant_kernel_with_history_and_pins() {
    let masses = [1., 4., 0.25, 1., 4., 0.25];
    let h = 0.5_f64;
    let compliance = 0.25;
    // C_j=.5*A_j+.5*A_(j+1)-.25*B_j-.75*B_(j+1).
    // Both the constant vector and this nonconstant vector are in ker(C).
    let kernel = [0., 1., 2., -0.25, 0.75, 1.75];
    let coefficients = [
        [(0, 0.5), (1, 0.5), (3, -0.25), (4, -0.75)],
        [(1, 0.5), (2, 0.5), (4, -0.25), (5, -0.75)],
    ];
    let stitches = [0, 1].map(|j| Stitch {
        a: WeightedPoint {
            vertices: [j, j + 1],
            weights: [0.5, 0.5],
        },
        b: WeightedPoint {
            vertices: [j + 3, j + 4],
            weights: [0.25, 0.75],
        },
        compliance,
    });
    for pinned in [false, true] {
        let mut inverse_mass = masses.map(|mass: f64| mass.recip());
        if pinned {
            inverse_mass[0] = 0.;
        }
        for offset in [0., 2_f64.powi(-10)] {
            let expected: [Vec3; 6] = std::array::from_fn(|i| {
                let v = 2_f64.powi(-10) * kernel[i];
                let b = if i >= 3 { offset } else { 0. };
                [0.125 + v + 2. * b, 0.25 + 0.5 * v - b, 0.5 - v + 3. * b]
            });
            // Manufacture z=q*+W*C^T*k*h²*C*q*. Its strictly convex energy has minimum q*.
            // Offset0 also has an exactly zero dual RHS despite nonzero retained p.
            let expected_difference = [-2. * offset, offset, -3. * offset];
            let optimum_impulse = expected_difference.map(|v| -h * h * v / compliance);
            let old_impulses = [
                [1. / 64., -1. / 32., 1. / 128.],
                [-1. / 64., 1. / 16., 1. / 64.],
            ];
            let mut optimum_history = [[0.; 3]; 6];
            let mut history = [[0.; 3]; 6];
            for (j, row) in coefficients.into_iter().enumerate() {
                assert_eq!(row.into_iter().map(|(i, c)| c * kernel[i]).sum::<f64>(), 0.);
                for (i, c) in row {
                    optimum_history[i] = add(
                        optimum_history[i],
                        mul(optimum_impulse, inverse_mass[i] * c),
                    );
                    history[i] = add(history[i], mul(old_impulses[j], inverse_mass[i] * c));
                }
            }
            assert_ne!(history, [[0.; 3]; 6]);
            let anchor: [Vec3; 6] = std::array::from_fn(|i| sub(expected[i], optimum_history[i]));
            let mut x: [Vec3; 6] = std::array::from_fn(|i| add(anchor[i], history[i]));
            let initial = x;
            let initial_history = history;
            let independent_force = |q: [Vec3; 6]| {
                let mut force = std::array::from_fn::<_, 6, _>(|i| {
                    mul(sub(q[i], anchor[i]), masses[i] / (h * h))
                });
                for row in coefficients {
                    let mut d = [0.; 3];
                    for (i, c) in row {
                        d = add(d, mul(q[i], c));
                    }
                    for (i, c) in row {
                        force[i] = add(force[i], mul(d, c / compliance));
                    }
                }
                force
            };
            let oracle_force = independent_force(expected);
            for i in 0..6 {
                if inverse_mass[i] > 0. {
                    assert!(length(oracle_force[i]) <= 1e-12, "manufactured minimum: N");
                }
            }
            let mut kernel_moment = [0.; 3];
            for _ in 0..32 {
                constraints::solve_sewing(
                    &stitches,
                    &[0.; 2],
                    &mut x,
                    &inverse_mass,
                    h,
                    &mut history,
                    &|| false,
                )
                .unwrap();
                kernel_moment = [0.; 3];
                let mut total_action = [0.; 3];
                for i in 0..6 {
                    kernel_moment = add(kernel_moment, mul(history[i], masses[i] * kernel[i]));
                    total_action = add(total_action, mul(history[i], masses[i]));
                    assert!(
                        length(sub(
                            history[i],
                            add(initial_history[i], sub(x[i], initial[i]))
                        )) <= 1e-14,
                        "actual nodal history: m"
                    );
                }
                assert!(
                    length(kernel_moment) <= 1e-12,
                    "nonconstant-kernel physical action: kg m"
                );
                if !pinned {
                    assert!(length(total_action) <= 1e-12, "total internal action: kg m");
                }
                if pinned {
                    assert_eq!(x[0], initial[0]);
                    assert_eq!(history[0], [0.; 3]);
                }
                assert!(x.into_iter().flatten().all(f64::is_finite));
                assert!(history.into_iter().flatten().all(f64::is_finite));
            }
            let force = independent_force(x);
            println!(
                "dual_action_guard pinned={pinned} offset={offset} x_m={x:?} expected_m={expected:?} kernel_moment_kg_m={kernel_moment:?} force_N={force:?}"
            );
            for i in 0..6 {
                assert!(
                    length(sub(x[i], expected[i])) <= 1e-12,
                    "manufactured position: m"
                );
                if inverse_mass[i] > 0. {
                    assert!(length(force[i]) <= 1e-12, "independent force: N");
                }
            }
        }
    }
}

