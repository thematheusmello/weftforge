#[test]
fn shared_stitch_n32_matches_independent_duplicate_row_cartesian_energy() {
    let masses = [4.5714285714285506e-05_f64, 4.571428571428556e-05];
    let h = 1. / 480.;
    let compliance = 1e-8_f64;
    let target = 0.01333333333333439;
    let predictor: [Vec3; 2] = [[0., 0., 0.015], [0., 0., -0.015]];
    let inverse_mass = masses.map(|mass| mass.recip());
    let stitches = [
        Stitch {
            a: WeightedPoint::vertex(0),
            b: WeightedPoint::vertex(1),
            compliance,
        };
        2
    ];
    let mut x = predictor;
    let norm = |v: Vec3| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let total_mass = masses[0] + masses[1];
    let center = |q: [Vec3; 2]| -> Vec3 {
        std::array::from_fn(|axis| {
            (masses[0] * q[0][axis] + masses[1] * q[1][axis]) / total_mass
        })
    };
    let predictor_center = center(predictor);
    // Production adapter: two histories initialized once, both rows retained in order.
    let mut history = [[0.; 3]; 2];
    for _ in 0..32 {
        for i in 0..2 {
            constraints::solve_stitch(
                stitches[i],
                target,
                &mut x,
                &inverse_mass,
                h,
                &mut history[i],
            );
        }
        assert!(x.into_iter().flatten().all(f64::is_finite));
        assert!(history.into_iter().flatten().all(f64::is_finite));
        let center_error = norm(std::array::from_fn(|axis| {
            center(x)[axis] - predictor_center[axis]
        }));
        assert!(center_error <= 1e-14, "two-free-node center of mass: m");
    }
    // Independent Cartesian energy: both .5*k*(|q0-q1|-target)^2 terms count.
    let stiffness = compliance.recip();
    let reduced_mass = masses[0] * masses[1] / total_mass;
    let inertia = reduced_mass / (h * h);
    let initial_difference: Vec3 =
        std::array::from_fn(|axis| predictor[0][axis] - predictor[1][axis]);
    let initial_radius = norm(initial_difference);
    let radius =
        (inertia * initial_radius + 2. * stiffness * target) / (inertia + 2. * stiffness);
    let expected_difference = initial_difference.map(|value| value * radius / initial_radius);
    let expected: [Vec3; 2] = [
        std::array::from_fn(|axis| {
            predictor_center[axis] + masses[1] / total_mass * expected_difference[axis]
        }),
        std::array::from_fn(|axis| {
            predictor_center[axis] - masses[0] / total_mass * expected_difference[axis]
        }),
    ];
    let independent_residual = |q: [Vec3; 2]| -> [Vec3; 2] {
        let difference: Vec3 = std::array::from_fn(|axis| q[0][axis] - q[1][axis]);
        let distance = norm(difference);
        let elastic = difference.map(|value| 2. * stiffness * (distance - target) * value / distance);
        [
            std::array::from_fn(|axis| {
                masses[0] * (q[0][axis] - predictor[0][axis]) / (h * h) + elastic[axis]
            }),
            std::array::from_fn(|axis| {
                masses[1] * (q[1][axis] - predictor[1][axis]) / (h * h) - elastic[axis]
            }),
        ]
    };
    let objective = |q: [Vec3; 2]| {
        let inertial = (0..2)
            .map(|i| {
                masses[i]
                    * (0..3)
                        .map(|axis| (q[i][axis] - predictor[i][axis]).powi(2))
                        .sum::<f64>()
                    / (2. * h * h)
            })
            .sum::<f64>();
        let difference = std::array::from_fn(|axis| q[0][axis] - q[1][axis]);
        inertial + stiffness * (norm(difference) - target).powi(2)
    };
    let residuals = independent_residual(x);
    let oracle_residuals = independent_residual(expected);
    let position_error = (0..2)
        .map(|i| norm(std::array::from_fn(|axis| x[i][axis] - expected[i][axis])))
        .fold(0_f64, f64::max);
    let force_error = residuals.map(norm).into_iter().fold(0_f64, f64::max);
    let oracle_force_error = oracle_residuals.map(norm).into_iter().fold(0_f64, f64::max);
    println!(
        "duplicate_rows=2 sweeps=32 masses_kg={masses:?} h_s={h}\n\
         compliance_m_per_N_each={compliance} target_m={target}\n\
         predictors_m={predictor:?}\nx_m={x:?}\nexpected_m={expected:?}\n\
         histories_kg_m={history:?}\ncenter_m={:?}\nexpected_center_m={predictor_center:?}\n\
         independent_cartesian_residual_N={residuals:?}\n\
         oracle_cartesian_residual_N={oracle_residuals:?}\n\
         position_error_m={position_error}\nforce_error_N={force_error}\n\
         objective_J={}\nexpected_objective_J={}",
        center(x),
        objective(x),
        objective(expected)
    );
    assert!(oracle_force_error <= 1e-6, "independent oracle force: N");
    assert!(position_error <= 1e-8, "shared-row Cartesian position: m");
    assert!(force_error <= 1e-6, "shared-row full Cartesian stationarity: N");
}

