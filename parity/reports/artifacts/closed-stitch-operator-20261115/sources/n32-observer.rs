// This read-only adapter keeps the frozen N32 observer identical for scalar RED and vector GREEN.
trait ObservedStitchMultiplier: Copy + Default + std::fmt::Debug {
    fn endpoint_vector(self, difference: Vec3) -> Vec3;
    fn is_scalar(self) -> bool;
}
impl ObservedStitchMultiplier for f64 {
    fn endpoint_vector(self, difference: Vec3) -> Vec3 {
        mul(difference, self / length(difference))
    }
    fn is_scalar(self) -> bool {
        true
    }
}
impl ObservedStitchMultiplier for Vec3 {
    fn endpoint_vector(self, _: Vec3) -> Vec3 {
        self
    }
    fn is_scalar(self) -> bool {
        false
    }
}

#[test]
fn closed_elastic_stitch_n32_matches_independent_quadratic_and_individual_actions() {
    let anchors = [[1., 0., 0.], [0., 1., 0.]];
    let mut x = [[0.; 3], anchors[0], anchors[1]];
    let stitches = [1, 2].map(|anchor| Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(anchor),
        compliance: 1.,
    });
    let mut lambda = [Default::default(); 2];
    let mut individual_action = [[0.; 3]; 2];
    for _ in 0..32 {
        for i in 0..2 {
            let before = x[0];
            constraints::solve_stitch(stitches[i], 0., &mut x, &[1., 0., 0.], 1., &mut lambda[i]);
            individual_action[i] = add(individual_action[i], sub(x[0], before));
        }
    }
    let q = stitches.map(|s| sub(s.a.position(&x), s.b.position(&x)));
    let lambda_vector = std::array::from_fn::<_, 2, _>(|i| lambda[i].endpoint_vector(q[i]));
    let residual = std::array::from_fn::<_, 2, _>(|i| add(q[i], lambda_vector[i]));
    let transport =
        std::array::from_fn::<_, 2, _>(|i| length(sub(individual_action[i], lambda_vector[i])));
    let energy = (dot(q[0], q[0]) + dot(q[1], q[1])) / 2.;
    let objective = dot(x[0], x[0]) / 2. + energy;
    // The gradient of Phi=3(x²+y²)/2-x-y+1 is independent of the multiplier ledger.
    let stationarity = add(x[0], add(q[0], q[1]));
    let u = 2_f64.powi(-64);
    let expected_x = [(1. - u) / 3., (1. + 2. * u) / 3., 0.];
    let expected_lambda = [
        [(2. - 2. * u) / 3., (-1. + 4. * u) / 3., 0.],
        [(-1. + u) / 3., (2. - 2. * u) / 3., 0.],
    ];
    let expected_residual = [[-u, 2. * u, 0.], [0.; 3]];
    let expected_energy = (5. - u + 5. * u * u) / 9.;
    let expected_objective = 2. / 3. + 5. * u * u / 6.;
    println!(
        "fixed_operator_N=32 h_s=1 c_m_per_N=1 free_mass_kg=1\n\
         x_m={x:?}\nlambda_private_kg_m={lambda:?}\nlambda_vector_kg_m={lambda_vector:?}\n\
         residual_m={residual:?}\nindividual_action_N={individual_action:?}\n\
         individual_transport_N={transport:?}\nenergy_J={energy:?}\n\
         objective_J={objective:?}\nstationarity_N={stationarity:?}\n\
         expected_x_m={expected_x:?}\nexpected_lambda_kg_m={expected_lambda:?}\n\
         expected_residual_m={expected_residual:?}\nexpected_energy_J={expected_energy:?}\n\
         expected_objective_J={expected_objective:?}"
    );
    if lambda[0].is_scalar() {
        assert!(transport[0] > 0.01, "legacy stitch1 RED witness: N");
    }
    assert_eq!(&x[1..], &anchors);
    assert!(length(sub(x[0], expected_x)) <= 1e-12, "position: m");
    for i in 0..2 {
        assert!(
            length(sub(lambda_vector[i], expected_lambda[i])) <= 1e-12,
            "stitch{i} multiplier: kg m"
        );
        assert!(
            length(sub(residual[i], expected_residual[i])) <= 1e-12,
            "stitch{i} constitutive residual: m"
        );
        assert!(transport[i] <= 1e-12, "stitch{i} individual transport: N");
    }
    assert!((energy - expected_energy).abs() <= 1e-12, "energy: J");
    assert!(
        (objective - expected_objective).abs() <= 1e-12,
        "incremental objective: J"
    );
    assert!(length(stationarity) <= 1e-12, "stationarity: N");
}

