use super::*;

fn square() -> ClothInput {
    ClothInput {
        positions: vec![[0., 2., 0.], [1., 2., 0.], [1., 2., 1.], [0., 2., 1.]],
        material_uv: vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]],
        triangles: vec![[0, 1, 2], [0, 2, 3]],
        triangle_materials: vec![
            Material {
                damping: 0.,
                ..Default::default()
            };
            2
        ],
        ..Default::default()
    }
}
fn settings() -> StepSettings {
    StepSettings {
        dt: 0.01,
        substeps: 1,
        iterations: 12,
        gravity: [0.; 3],
        seam_closure_speed: 0.5,
    }
}

#[test]
fn area_mass_rest_and_reset_are_si_consistent() {
    let mut cloth = Cloth::new(square()).unwrap();
    let rest_peak = Some(StrainPeak {
        triangle: 0,
        axis: MaterialAxis::Warp,
        signed_strain: 0.,
    });
    assert_eq!(cloth.diagnostics().peak_strain, rest_peak);
    assert_eq!(Diagnostics::default().peak_strain, None);
    assert!((cloth.masses().iter().sum::<f64>() - 0.2).abs() < 1e-14);
    let rest = cloth.positions().to_vec();
    for _ in 0..20 {
        cloth.step(settings(), || false).unwrap();
    }
    assert!(
        cloth
            .positions()
            .iter()
            .zip(&rest)
            .all(|(&a, &b)| length(sub(a, b)) < 1e-12)
    );
    cloth
        .step(
            StepSettings {
                gravity: [0., -9.81, 0.],
                ..settings()
            },
            || false,
        )
        .unwrap();
    assert!(cloth.positions()[0][1] < rest[0][1]);
    cloth.reset();
    assert_eq!(cloth.positions(), rest);
    assert_eq!(cloth.simulation_time(), 0.);
    assert!(cloth.velocities().iter().all(|v| *v == [0.; 3]));
    assert_eq!(cloth.diagnostics().peak_strain, rest_peak);
}

#[test]
fn freefall_matches_symplectic_euler_and_refines_toward_analytic_solution() {
    let run = |substeps| {
        let mut cloth = Cloth::new(square()).unwrap();
        cloth
            .step(
                StepSettings {
                    dt: 0.1,
                    substeps,
                    iterations: 2,
                    gravity: [0., -9.81, 0.],
                    ..Default::default()
                },
                || false,
            )
            .unwrap();
        cloth.positions()[0][1]
    };
    let coarse = run(10);
    let fine = run(40);
    let analytic = 2. - 0.5 * 9.81 * 0.1_f64.powi(2);
    let symplectic = 2. - 9.81 * 0.01_f64.powi(2) * 55.;
    assert!((coarse - symplectic).abs() < 1e-11);
    assert!((fine - analytic).abs() < 0.0013);
    assert!((fine - analytic).abs() < 0.51 * (coarse - analytic).abs());
}

#[test]
fn membrane_affine_strain_follows_material_axes_under_arrangement_and_winding() {
    let uv = [[2., 3.], [2.4, 3.1], [1.8, 3.5]];
    for [u_scale, shear, v_scale] in [[1.2, 0.25, 0.7], [1.5, 0.25, 0.85]] {
        // A 90-degree rigid arrangement of F = [[u_scale, shear], [0, v_scale], [0, 0]].
        let x: Vec<_> = uv
            .iter()
            .map(|[u, v]| [10. - v_scale * v, 2. + u_scale * u + shear * v, -0.4])
            .collect();
        for grain_angle in [0., 0.37, std::f64::consts::FRAC_PI_2] {
            let (sin, cos) = grain_angle.sin_cos();
            let warp = (u_scale * cos + shear * sin).hypot(v_scale * sin) - 1.;
            let weft = (-u_scale * sin + shear * cos).hypot(v_scale * cos) - 1.;
            let (axis, signed_strain) = if weft.abs() > warp.abs() {
                (MaterialAxis::Weft, weft)
            } else {
                (MaterialAxis::Warp, warp)
            };
            for tri in [[0, 1, 2], [0, 2, 1]] {
                let material = Material {
                    grain_angle,
                    ..Default::default()
                };
                let membrane = constraints::Membrane::new(tri, &uv, material);
                assert!((membrane.area - 0.11).abs() < 1e-14);
                let actual =
                    strain::signed_strains(tri.map(|i| x[i]), tri.map(|i| uv[i]), grain_angle);
                assert!((actual[0] - warp).abs() < 1e-13);
                assert!((actual[1] - weft).abs() < 1e-13);
                let cloth = Cloth::new(ClothInput {
                    positions: x.clone(),
                    material_uv: uv.to_vec(),
                    triangles: vec![tri],
                    triangle_materials: vec![material],
                    ..Default::default()
                })
                .unwrap();
                let diagnostics = cloth.diagnostics();
                let peak = diagnostics.peak_strain.unwrap();
                assert_eq!(peak.triangle, 0);
                assert_eq!(peak.axis, axis);
                assert!((peak.signed_strain - signed_strain).abs() < 1e-13);
                assert_eq!(diagnostics.max_stretch_error, peak.signed_strain.abs());
            }
        }
    }
}

#[test]
fn peak_strain_uses_input_triangle_order_and_exact_ties_choose_warp() {
    let mut input = ClothInput {
        positions: vec![
            [10., 2., 0.],
            [10.5, 2., 0.],
            [10., 2., 1.5],
            [0., 2., 0.],
            [1.5, 2., 0.],
            [0., 2., 0.5],
        ],
        material_uv: [[0., 0.], [1., 0.], [0., 1.]].repeat(2),
        triangles: vec![[3, 4, 5], [0, 1, 2]],
        triangle_materials: vec![Material::default(); 2],
        ..Default::default()
    };
    let check = |input: &ClothInput, triangle, axis, signed_strain| {
        let cloth = Cloth::new(input.clone()).unwrap();
        assert_eq!(
            cloth.diagnostics().peak_strain,
            Some(StrainPeak {
                triangle,
                axis,
                signed_strain,
            })
        );
        assert_eq!(cloth.diagnostics().max_stretch_error, signed_strain.abs());
    };
    check(&input, 0, MaterialAxis::Warp, 0.5);
    input.triangles.reverse();
    check(&input, 0, MaterialAxis::Warp, -0.5);
    input.triangles.reverse();
    input.positions[2][2] = 1.75;
    check(&input, 1, MaterialAxis::Weft, 0.75);
    input.triangles.reverse();
    check(&input, 0, MaterialAxis::Weft, 0.75);
}

#[test]
fn shared_edge_peak_preserves_exact_transforms_and_distinguishes_near_ties() {
    let original = [[0., 0., 0.], [1.5, 0., 0.], [0., 1., 0.], [1.5, 1., 0.]];
    for grain_angle in [0., std::f64::consts::FRAC_PI_2] {
        let axis = if grain_angle == 0. {
            MaterialAxis::Warp
        } else {
            MaterialAxis::Weft
        };
        let mut input = ClothInput {
            material_uv: vec![[0., 0.], [1., 0.], [0., 1.], [1., 1.]],
            triangles: vec![[0, 1, 2], [1, 3, 2]],
            triangle_materials: vec![
                Material {
                    grain_angle,
                    ..Default::default()
                };
                2
            ],
            ..Default::default()
        };
        let check = |input: &ClothInput, triangle, signed_strain| {
            let diagnostics = Cloth::new(input.clone()).unwrap().diagnostics().clone();
            assert_eq!(
                diagnostics.peak_strain,
                Some(StrainPeak {
                    triangle,
                    axis,
                    signed_strain,
                })
            );
            assert_eq!(diagnostics.max_stretch_error, signed_strain.abs());
        };
        for positions in [
            original,
            original.map(|[x, y, z]| [x + 8., y - 4., z + 2.]),
            original.map(|[x, y, z]| [z, x, y]),
            original.map(|[x, y, z]| [-x, y, z]),
        ] {
            input.positions = positions.to_vec();
            check(&input, 0, 0.5);
        }
        input.positions = original.to_vec();
        input.positions[3][0] += 2_f64.powi(-51);
        check(&input, 1, 0.5 + 2_f64.powi(-51));
        input.positions[3][0] = 1.625;
        check(&input, 1, 0.625);
    }
}

#[test]
fn nonfinite_strain_after_a_finite_candidate_invalidates_diagnostics() {
    let cloth = Cloth::new(square()).unwrap();
    let mut positions = cloth.positions().to_vec();
    for position in &mut positions {
        position[0] = f64::MAX;
    }
    positions[3][0] = -f64::MAX;
    let diagnostics = cloth.measure_errors_with_contact(
        &positions,
        cloth.velocities(),
        contact::ContactErrors::default(),
    );
    assert!(diagnostics.max_stretch_error.is_finite());
    assert!(diagnostics.peak_strain.unwrap().signed_strain.is_finite());
    assert!(!diagnostics.finite);
}

#[test]
fn shared_edge_material_reaches_independent_incremental_energy_equilibrium() {
    let rest = [
        [-0.064, 0., 0.],
        [0., -0.004, 0.],
        [0., 0.004, 0.],
        [0.064, 0., 0.],
    ];
    let material = Material {
        areal_density: 0.18,
        warp_stiffness: 400.,
        weft_stiffness: 400.,
        shear_stiffness: 100.,
        grain_angle: std::f64::consts::FRAC_PI_2,
        bending_stiffness: 0.,
        damping: 0.,
        friction: 0.,
        collision_thickness: 0.003,
    };
    let mut cloth = Cloth::new(ClothInput {
        positions: rest.to_vec(),
        material_uv: rest.map(|v| [v[0], v[1]]).to_vec(),
        triangles: vec![[0, 1, 2], [1, 3, 2]],
        triangle_materials: vec![material; 2],
        pins: vec![
            Pin {
                vertex: 0,
                target: rest[0],
            },
            Pin {
                vertex: 3,
                target: rest[3],
            },
        ],
        ..Default::default()
    })
    .unwrap();
    let h = (1. / 60.) / 8.;
    let speed = 0.0005 / h;
    cloth.velocities[1] = [0., -speed, 0.];
    cloth.velocities[2] = [0., speed, 0.];
    let predicted = [
        add(rest[1], mul(cloth.velocities[1], h)),
        add(rest[2], mul(cloth.velocities[2], h)),
    ];
    let area = 0.064 * 0.008 / 2.;
    let mass = 2. * 0.18 * area / 3.;
    for v in [1, 2] {
        assert!((cloth.masses()[v] - mass).abs() < 1e-18);
    }
    assert!(cloth.bends.is_empty() && cloth.input.stitches.is_empty());
    assert!(cloth.input.colliders.is_empty());
    cloth
        .step(
            StepSettings {
                dt: h,
                substeps: 1,
                iterations: 32,
                gravity: [0.; 3],
                seam_closure_speed: 0.,
            },
            || false,
        )
        .unwrap();

    // Independent 100-digit Newton root of the six-DOF physical energy;
    // the symmetric quarter-turn reduction also gives an independent cubic.
    let expected = [
        [6.729906241347043e-23, -0.004000552234905839, 0.],
        [-6.729906241347043e-23, 0.004000552234905839, 0.],
    ];
    let x = [cloth.positions()[1], cloth.positions()[2]];
    let position_error = (0..2)
        .map(|i| length(sub(x[i], expected[i])))
        .fold(0., f64::max);

    // Direct columns for this independent two-face geometry, with one inertia
    // per free particle. No cached constraint gradients or multipliers enter.
    let center = mul(add(x[0], x[1]), 0.5);
    let v = mul(sub(x[1], x[0]), 1. / 0.008);
    let (sin, cos) = material.grain_angle.sin_cos();
    let mut gradient: [Vec3; 2] =
        std::array::from_fn(|i| mul(sub(x[i], predicted[i]), mass / (h * h)));
    for sign in [1., -1.] {
        let u = add([1., 0., 0.], mul(center, sign / 0.064));
        let w = add(mul(u, cos), mul(v, sin));
        let f = add(mul(u, -sin), mul(v, cos));
        let cw = (dot(w, w) - 1.) / 2.;
        let cf = (dot(f, f) - 1.) / 2.;
        let cs = dot(w, f);
        let gu = add(
            add(
                mul(w, 400. * area * cw * cos),
                mul(f, -400. * area * cf * sin),
            ),
            mul(sub(mul(f, cos), mul(w, sin)), 100. * area * cs),
        );
        let gv = add(
            add(
                mul(w, 400. * area * cw * sin),
                mul(f, 400. * area * cf * cos),
            ),
            mul(add(mul(f, sin), mul(w, cos)), 100. * area * cs),
        );
        for (i, entry) in gradient.iter_mut().enumerate() {
            *entry = add(
                *entry,
                add(mul(gu, sign / 0.128), mul(gv, (2. * i as f64 - 1.) / 0.008)),
            );
        }
    }
    let stationarity = gradient.into_iter().map(length).fold(0., f64::max);
    println!(
        "shared-edge positions={x:?} reference={expected:?} position_error={position_error:e}m stationarity={stationarity:e}N"
    );
    assert!(position_error < 1e-8 && stationarity < 1e-6);
    assert_eq!(cloth.positions()[0], rest[0]);
    assert_eq!(cloth.positions()[3], rest[3]);
}

#[test]
fn coupled_membrane_reaches_independent_incremental_energy_equilibrium() {
    let uv = [[0., 0.], [0.01, 0.], [0.003, 0.07]];
    let material = Material {
        areal_density: 0.18,
        warp_stiffness: 400.,
        weft_stiffness: 400.,
        shear_stiffness: 100.,
        grain_angle: std::f64::consts::FRAC_PI_2,
        ..Default::default()
    };
    let membrane = constraints::Membrane::new([0, 1, 2], &uv, material);
    let h = (1. / 60.) / 8.;
    let area = 0.01 * 0.07 / 2.;
    let mass = 0.18 * area / 3.;
    let predicted = [0.012, 0.002, 0.];
    let mut x = [[0.; 3], predicted, [0.003, 0.07, 0.]];
    let mut state = [[0.; 3]; 3];
    for _ in 0..32 {
        constraints::solve_material(
            std::slice::from_ref(&membrane),
            &mut x,
            &[0., 1. / mass, 0.],
            h,
            &mut state,
            &|| false,
        )
        .unwrap();
    }

    // Independent dense Newton root of inertia plus the direct deformation-gradient energy.
    let expected = [0.010007132086944948, 0.000027306978065735514, 0.];
    let position_error = length(sub(x[1], expected));
    let u = mul(x[1], 100.);
    let v = mul(sub([0.003, 0.07, 0.], mul(x[1], 0.3)), 1. / 0.07);
    let cu = 0.5 * (dot(u, u) - 1.);
    let cv = 0.5 * (dot(v, v) - 1.);
    let cs = dot(u, v);
    let elastic_gradient = add(
        mul(
            add(mul(u, 100. * cu), mul(v, -0.3 / 0.07 * cv)),
            400. * area,
        ),
        mul(add(mul(v, 100.), mul(u, -0.3 / 0.07)), 100. * area * cs),
    );
    let stationarity = length(add(
        mul(sub(x[1], predicted), mass / (h * h)),
        elastic_gradient,
    ));
    println!(
        "independent energy position error={position_error:e}m stationarity={stationarity:e}N"
    );
    assert!(position_error < 1e-8 && stationarity < 1e-6);
    assert_eq!(x[0], [0.; 3]);
    assert_eq!(x[2], [0.003, 0.07, 0.]);
}

#[test]
fn membrane_projection_matches_analytic_uniaxial_and_shear_updates() {
    let uv = [[0., 0.], [1., 0.], [0., 1.]];
    for (grain_angle, expected_x) in [
        (0., 1.151470588235294),
        (std::f64::consts::FRAC_PI_2, 1.0390243902439025),
    ] {
        let membrane = constraints::Membrane::new(
            [0, 1, 2],
            &uv,
            Material {
                warp_stiffness: 50.,
                weft_stiffness: 1000.,
                grain_angle,
                ..Default::default()
            },
        );
        let mut x = [[0., 0., 0.], [1.2, 0., 0.], [0., 1., 0.]];
        // Only x[1] moves: C=0.22, dC/dx=1.2, alpha=4 or 0.2 for area=0.5 m².
        constraints::solve_material(
            &[membrane],
            &mut x,
            &[0., 1., 0.],
            0.1,
            &mut [[0.; 3]; 3],
            &|| false,
        )
        .unwrap();
        assert!((x[1][0] - expected_x).abs() < 1e-14);
        assert_eq!(x[0], [0.; 3]);
        assert_eq!(x[2], [0., 1., 0.]);
    }
    let membrane = constraints::Membrane::new(
        [0, 1, 2],
        &uv,
        Material {
            shear_stiffness: 10.,
            ..Default::default()
        },
    );
    let height = 0.91_f64.sqrt();
    let mut x = [[0., 0., 0.], [1., 0., 0.], [0.3, height, 0.]];
    // Exact rational solve of the coupled Gram matrix with rows [33/16,3/10,8/5],
    // [3/10,25/12,8/5], [8/5,8/5,48/5] and RHS [0,0,-3/10].
    constraints::solve_material(
        &[membrane],
        &mut x,
        &[1.; 3],
        0.2,
        &mut [[0.; 3]; 3],
        &|| false,
    )
    .unwrap();
    let [warp, weft, shear] = [1284. / 46963., 1269. / 46963., -60579. / 1502816.];
    let u = [1., 0., 0.];
    let v = [0.3, height, 0.];
    let expected = [
        mul(add(mul(u, warp + shear), mul(v, weft + shear)), -1.),
        add(u, add(mul(u, warp), mul(v, shear))),
        add(v, add(mul(v, weft), mul(u, shear))),
    ];
    for (actual, expected) in x.into_iter().zip(expected) {
        assert!(length(sub(actual, expected)) < 1e-14);
    }
}

#[test]
fn weighted_stitch_closes_unequal_samples_and_preserves_momentum() {
    let stitch = Stitch {
        a: WeightedPoint {
            vertices: [0, 1],
            weights: [0.2, 0.8],
        },
        b: WeightedPoint {
            vertices: [2, 3],
            weights: [0.7, 0.3],
        },
        compliance: 0.,
    };
    let mut x = vec![[0., 0., 0.], [1., 0., 0.], [2., 0.3, 0.], [3., 0.3, 0.]];
    let before = x.iter().copied().fold([0.; 3], add);
    constraints::solve_stitch(stitch, 0., &mut x, &[1.; 4], 0.01, &mut [0.; 3]);
    assert!(length(sub(stitch.a.position(&x), stitch.b.position(&x))) < 1e-12);
    assert!(length(sub(x.iter().copied().fold([0.; 3], add), before)) < 1e-12);
    // A repeated endpoint must be treated as one degree of freedom.
    let s = Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(2),
        compliance: 0.,
    };
    constraints::solve_stitch(s, 0., &mut x, &[1.; 4], 0.01, &mut [0.; 3]);
    assert!(length(sub(x[0], x[2])) < 1e-12);
}

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

#[test]
fn closed_elastic_stitch_preserves_isolated_energy_and_units() {
    let stitch = Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(1),
        compliance: 0.125,
    };
    let predictor = [1., 2., 0.];
    let mut x = [predictor, [0.; 3]];
    let mut lambda = [0.; 3];
    constraints::solve_stitch(stitch, 0., &mut x, &[0.5, 0.], 0.5, &mut lambda);
    assert_eq!(lambda, [-1., -2., 0.]);
    assert_eq!(x, [[0.5, 1., 0.], [0.; 3]]);
    let q = sub(stitch.a.position(&x), stitch.b.position(&x));
    let force = [-4., -8., 0.];
    assert_eq!(mul(q, -1. / stitch.compliance), force);
    assert_eq!(mul(lambda, 1. / 0.5_f64.powi(2)), force);
    assert_eq!(mul(sub(x[0], predictor), 2. / 0.5_f64.powi(2)), force);
    assert_eq!(dot(q, q) / (2. * stitch.compliance), 5.);
}

#[test]
fn closed_elastic_stitch_two_free_masses_preserve_reactions_under_rigid_transforms() {
    let stitch = Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(1),
        compliance: 0.0625,
    };
    for (predictor, initial_lambda, expected_x, expected_lambda, expected_energy) in [
        (
            [[1., 2., 0.], [0.; 3]],
            [0.; 3],
            [[0.5, 1., 0.], [0.25, 0.5, 0.]],
            [-1., -2., 0.],
            2.5,
        ),
        (
            [[0.; 3]; 2],
            [2., -4., 1.],
            [[-0.25, 0.5, -0.125], [0.125, -0.25, 0.0625]],
            [1.5, -3., 0.75],
            5.90625,
        ),
    ] {
        // For masses 2/4 kg and h=0.5 s, alpha=1/4 kg^-1 and D=1 kg^-1.
        for (angle, translation) in [(0_f64, [0.; 3]), (0., [8., -4., 2.]), (0.37, [8., -4., 2.])] {
            let axis = unit([1., 2., 3.]);
            let (sin, cos) = angle.sin_cos();
            let rotate = |v| {
                add(
                    add(mul(v, cos), mul(cross(axis, v), sin)),
                    mul(axis, dot(axis, v) * (1. - cos)),
                )
            };
            let transform = |v| add(rotate(v), translation);
            let before = predictor.map(transform);
            let mut x = before;
            let initial_lambda = rotate(initial_lambda);
            let mut lambda = initial_lambda;
            constraints::solve_stitch(stitch, 0., &mut x, &[0.5, 0.25], 0.5, &mut lambda);
            for (actual, expected) in x.into_iter().zip(expected_x.map(transform)) {
                assert!(length(sub(actual, expected)) <= 1e-12, "position: m");
            }
            assert!(length(sub(lambda, rotate(expected_lambda))) <= 1e-12);
            let action = mul(sub(lambda, initial_lambda), 4.);
            assert!(length(sub(mul(sub(x[0], before[0]), 8.), action)) <= 1e-12);
            assert!(length(add(mul(sub(x[1], before[1]), 16.), action)) <= 1e-12);
            let difference = sub(x[0], x[1]);
            assert!(length(add(mul(difference, 16.), mul(lambda, 4.))) <= 1e-12);
            assert!(
                (dot(difference, difference) / (2. * stitch.compliance) - expected_energy).abs()
                    <= 1e-12,
                "energy: J"
            );
            let mass_moment = |x: [Vec3; 2]| add(mul(x[0], 2.), mul(x[1], 4.));
            assert!(length(sub(mass_moment(x), mass_moment(before))) <= 1e-12);
        }
    }
}

#[test]
fn closed_elastic_stitch_merges_repeated_and_cross_side_vertices() {
    let stitch = Stitch {
        a: WeightedPoint {
            vertices: [0, 0],
            weights: [0.25, 0.75],
        },
        b: WeightedPoint {
            vertices: [0, 1],
            weights: [0.25, 0.75],
        },
        compliance: 1.,
    };
    let mut x = [[0.; 3], [1., 0., 0.]];
    let mut lambda = [0.; 3];
    constraints::solve_stitch(stitch, 0., &mut x, &[1., 0.], 1., &mut lambda);
    assert!(length(sub(lambda, [12. / 25., 0., 0.])) <= 1e-14);
    assert!(length(sub(x[0], [9. / 25., 0., 0.])) <= 1e-14);
    assert_eq!(x[1], [1., 0., 0.]);
    assert!(
        length(add(
            sub(stitch.a.position(&x), stitch.b.position(&x)),
            lambda
        )) <= 1e-14
    );
}

#[test]
fn closed_elastic_stitch_keeps_accepted_weights_and_endpoint_arithmetic() {
    let beta = 0.75 + 1e-11;
    let stitch = Stitch {
        a: WeightedPoint {
            vertices: [0, 1],
            weights: [0.25, beta],
        },
        b: WeightedPoint {
            vertices: [0, 2],
            weights: [0.25, 0.75],
        },
        compliance: 1.,
    };
    let mut input = square();
    input.stitches.push(stitch);
    validate(&input, &|| false).unwrap();
    let mut x = [[1e8, -2e8, 3e8]; 3];
    let before = x;
    let q = sub(
        add(mul(x[0], 0.25), mul(x[1], beta)),
        add(mul(x[0], 0.25), mul(x[2], 0.75)),
    );
    assert!(length(q) > 0.001);
    let expected_lambda = q.map(|value| -value / (beta * beta + 1.));
    let expected_free = add(before[1], mul(expected_lambda, beta));
    let mut lambda = [0.; 3];
    constraints::solve_stitch(stitch, 0., &mut x, &[0., 1., 0.], 1., &mut lambda);
    assert_eq!(lambda, expected_lambda);
    assert_eq!(x, [before[0], expected_free, before[2]]);
    assert_eq!(stitch.a.weights, [0.25, beta]);
}

#[test]
fn closed_elastic_stitch_handles_zero_gap_pins_and_denominator_guard() {
    let stitch = Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(1),
        compliance: 1.,
    };
    let mut x = [[0.; 3]; 2];
    let mut lambda = [0.; 3];
    constraints::solve_stitch(stitch, 0., &mut x, &[1., 0.], 1., &mut lambda);
    assert_eq!(x, [[0.; 3]; 2]);
    assert_eq!(lambda, [0.; 3]);
    assert_eq!(dot(x[0], x[0]) / (2. * stitch.compliance), 0.);
    lambda = [2., -4., 1.];
    constraints::solve_stitch(stitch, 0., &mut x, &[1., 0.], 1., &mut lambda);
    assert_eq!(lambda, [1., -2., 0.5]);
    assert_eq!(x, [[-1., 2., -0.5], [0.; 3]]);
    assert_eq!(add(sub(x[0], x[1]), lambda), [0.; 3]);

    let stitch = Stitch {
        compliance: 0.125,
        ..stitch
    };
    let pinned = [[1., 2., 3.], [0.; 3]];
    x = pinned;
    lambda = [0.; 3];
    constraints::solve_stitch(stitch, 0., &mut x, &[0.; 2], 0.5, &mut lambda);
    assert_eq!(x, pinned);
    assert_eq!(lambda, [-2., -4., -6.]);
    assert_eq!(mul(lambda, 4.), [-8., -16., -24.]);
    assert_eq!(mul(sub(x[0], x[1]), -8.), [-8., -16., -24.]);

    for (compliance, dt) in [(1e-21, 1.), (1., 1e-200)] {
        let stitch = Stitch {
            compliance,
            ..stitch
        };
        let before_lambda = lambda;
        constraints::solve_stitch(stitch, 0., &mut x, &[0.; 2], dt, &mut lambda);
        assert_eq!(x, pinned);
        assert_eq!(lambda, before_lambda);
    }
}

#[test]
fn positive_targets_and_hard_stitches_keep_radial_updates() {
    // Reciprocal normalization rounds the 3-4-5 direction's squared length above one.
    let r = 1.0_f64 / 5.;
    let d = 0.5 * ((3. * r).powi(2) + (4. * r).powi(2));
    let hard_expected = 0.25 - 5. / d;
    assert!((hard_expected + 9.75).abs() <= 1e-14);
    for (target, compliance, expected_x, expected_lambda) in [
        (1., 0.125, [1.7625, 2.35, 0.], -3.875),
        (f64::from_bits(1), 0.125, [1.4625, 1.95, 0.], -4.875),
        (0., 0., [0.; 3], hard_expected),
    ] {
        let stitch = Stitch {
            a: WeightedPoint::vertex(0),
            b: WeightedPoint::vertex(1),
            compliance,
        };
        let mut x = [[3., 4., 0.], [0.; 3]];
        let mut lambda = [0.25, 7., -8.];
        constraints::solve_stitch(stitch, target, &mut x, &[0.5, 0.], 0.5, &mut lambda);
        assert!(length(sub(x[0], expected_x)) <= 1e-14);
        assert_eq!(x[1], [0.; 3]);
        assert_eq!(lambda, [expected_lambda, 7., -8.]);
    }
    let hard = Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(1),
        compliance: 0.,
    };
    let pinned = [[1., 0., 0.], [0.; 3]];
    let mut x = pinned;
    let mut lambda = [2., 3., 4.];
    constraints::solve_stitch(hard, 0., &mut x, &[0.; 2], 1., &mut lambda);
    assert_eq!(x, pinned);
    assert_eq!(lambda, [2., 3., 4.]);
    x = [[0.; 3]; 2];
    constraints::solve_stitch(hard, 1., &mut x, &[1.; 2], 1., &mut lambda);
    assert_eq!(x, [[0.; 3]; 2]);
    assert_eq!(lambda, [2., 3., 4.]);
}

#[test]
fn positive_target_stitch_n32_matches_independent_coupled_radial_energy() {
    let mass = 0.01_f64;
    let h = 0.1_f64;
    let predictor = [0., 1., 0.];
    let anchors = [[0.; 3], [1., 0., 0.]];
    let mut x = [predictor, anchors[0], anchors[1]];
    let inverse_mass = [mass.recip(), 0., 0.];
    let stitches = [1, 2].map(|anchor| Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(anchor),
        compliance: 2.,
    });
    let targets = [0.5, 0.];
    let mut lambda = [[0.; 3]; 2];
    let mut applied_action = [[0.; 3]; 2];
    let mut last_update = [[0.; 3]; 2];
    let norm = |v: Vec3| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    for _ in 0..32 {
        for i in 0..2 {
            assert!(norm(x[0]) > 1e-6, "fixture stays away from radial singularity");
            let before = x[0];
            constraints::solve_stitch(
                stitches[i],
                targets[i],
                &mut x,
                &inverse_mass,
                h,
                &mut lambda[i],
            );
            for (axis, &value) in before.iter().enumerate() {
                last_update[i][axis] = x[0][axis] - value;
                applied_action[i][axis] += mass * last_update[i][axis] / (h * h);
            }
            assert_eq!([x[1], x[2]], anchors, "pins: m");
            assert!(x.into_iter().flatten().all(f64::is_finite));
            assert!(lambda.into_iter().flatten().all(f64::is_finite));
            assert!(applied_action.into_iter().flatten().all(f64::is_finite));
        }
    }
    // Direct differentiation of Phi=.5|q-p|²+.25(|q|-.5)²+.25|q-a|².
    let independent_forces = |q: Vec3| {
        let radius = norm(q);
        [
            q.map(|value| -0.5 * (radius - 0.5) * value / radius),
            std::array::from_fn(|axis| -0.5 * (q[axis] - anchors[1][axis])),
        ]
    };
    let objective = |q: Vec3| {
        let predictor_distance_squared = (0..3)
            .map(|axis| (q[axis] - predictor[axis]).powi(2))
            .sum::<f64>();
        let anchor_distance_squared = (0..3)
            .map(|axis| (q[axis] - anchors[1][axis]).powi(2))
            .sum::<f64>();
        0.5 * predictor_distance_squared
            + 0.25 * (norm(q) - 0.5).powi(2)
            + 0.25 * anchor_distance_squared
    };
    let root_five = 5_f64.sqrt();
    let expected = [
        0.25 + 1. / (8. * root_five),
        0.5 + 1. / (4. * root_five),
        0.,
    ];
    let forces = independent_forces(x[0]);
    let expected_forces = independent_forces(expected);
    let stationarity = std::array::from_fn(|axis| {
        mass * (x[0][axis] - predictor[axis]) / (h * h)
            - forces[0][axis]
            - forces[1][axis]
    });
    let oracle_stationarity = std::array::from_fn(|axis| {
        expected[axis] - predictor[axis] - expected_forces[0][axis] - expected_forces[1][axis]
    });
    let action_error = std::array::from_fn::<_, 2, _>(|i| {
        norm(std::array::from_fn(|axis| applied_action[i][axis] - forces[i][axis]))
    });
    let radius = norm(x[0]);
    let history_force = [
        x[0].map(|value| lambda[0][0] * value / (radius * h * h)),
        lambda[1].map(|value| value / (h * h)),
    ];
    let position_error = norm(std::array::from_fn(|axis| x[0][axis] - expected[axis]));
    let direction = x[0].map(|value| value / radius);
    println!(
        "positive_then_zero_N=32 mass_kg={mass} h_s={h} compliance_m_per_N=2\n\
         predictor_m={predictor:?}\nx_m={x:?}\nexpected_m={expected:?}\n\
         initial_radial_direction=[0,1,0]\nfinal_radial_direction={direction:?}\n\
         lambda_private_kg_m={lambda:?}\ncurrent_history_force_N={history_force:?}\n\
         independent_elastic_force_N={forces:?}\napplied_action_N={applied_action:?}\n\
         action_error_N={action_error:?}\nlast_stage_update_m={last_update:?}\n\
         stationarity_N={stationarity:?}\n\
         position_error_m={position_error}\nobjective_J={}\nexpected_objective_J={}",
        objective(x[0]),
        objective(expected)
    );
    assert!(norm(oracle_stationarity) <= 1e-14, "independent oracle: N");
    assert!(position_error <= 1e-8, "coupled radial position: m");
    assert!(norm(stationarity) <= 1e-6, "independent stationarity: N");
    for (i, error) in action_error.into_iter().enumerate() {
        assert!(error <= 1e-6, "stitch{i} independent action balance: N");
    }
}

#[test]
fn elastic_stitch_substeps_reset_multipliers_when_target_reaches_zero() {
    let mut input = square();
    input.stitches.push(Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(1),
        compliance: 1.,
    });
    let mut combined = Cloth::new(input).unwrap();
    let mut split = combined.clone();
    let settings = StepSettings {
        dt: 0.2,
        substeps: 2,
        seam_closure_speed: 5.,
        ..settings()
    };
    combined.step(settings, || false).unwrap();
    let settings = StepSettings {
        dt: 0.1,
        substeps: 1,
        ..settings
    };
    split.step(settings, || false).unwrap();
    assert_eq!(split.seam_remaining, [0.5]);
    split.step(settings, || false).unwrap();
    assert_eq!(combined.seam_remaining, [0.]);
    assert_eq!(combined.seam_remaining, split.seam_remaining);
    assert_eq!(combined.positions, split.positions);
    assert_eq!(combined.velocities, split.velocities);
    assert_eq!(combined.simulation_time(), split.simulation_time());
}

#[test]
fn closed_elastic_stitch_cancellation_and_failure_retain_whole_call_state() {
    let mut input = square();
    input.stitches.push(Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(1),
        compliance: 1.,
    });
    let mut cloth = Cloth::new(input).unwrap();
    let before = cloth.clone();
    let diagnostics = |d: &Diagnostics| {
        (
            d.simulation_time,
            d.step_seconds,
            d.max_stretch_error,
            d.peak_strain,
            d.max_seam_error,
            d.seam_assembly_remaining,
            d.max_contact_error,
            d.kinetic_energy,
            d.contacts,
            d.initial_intersections,
            d.capsule_surface_uncertain_pairs,
            d.finite,
        )
    };
    let assert_retained = |cloth: &Cloth| {
        assert_eq!(cloth.positions, before.positions);
        assert_eq!(cloth.velocities, before.velocities);
        assert_eq!(cloth.seam_remaining, before.seam_remaining);
        assert_eq!(
            diagnostics(&cloth.diagnostics),
            diagnostics(&before.diagnostics)
        );
    };
    let settings = StepSettings {
        dt: 0.1,
        substeps: 2,
        seam_closure_speed: 100.,
        ..settings()
    };
    let calls = std::cell::Cell::new(0);
    assert_eq!(
        cloth
            .step(settings, || {
                calls.set(calls.get() + 1);
                calls.get() == 4
            })
            .unwrap_err(),
        SimulationError::Cancelled
    );
    assert_eq!(calls.get(), 4);
    assert_retained(&cloth);
    assert!(matches!(
        cloth.step(
            StepSettings {
                gravity: [0., -1e100, 0.],
                ..settings
            },
            || false
        ),
        Err(SimulationError::Unstable(_))
    ));
    assert_retained(&cloth);
    cloth.step(settings, || false).unwrap();
    assert_eq!(cloth.seam_remaining, [0.]);
    assert!(cloth.diagnostics.finite);
}

#[test]
fn analytic_dihedral_gradient_matches_independent_finite_difference() {
    let x = vec![
        [0., 0.1, 0.],
        [1., 0., 0.1],
        [0.3, 0.9, 0.4],
        [0.8, -0.7, 0.2],
    ];
    let (_, gradient) = constraints::dihedral([0, 1, 2, 3], &x).unwrap();
    let epsilon = 1e-6;
    for vertex in 0..4 {
        for axis in 0..3 {
            let mut plus = x.clone();
            let mut minus = x.clone();
            plus[vertex][axis] += epsilon;
            minus[vertex][axis] -= epsilon;
            let a = constraints::dihedral([0, 1, 2, 3], &plus).unwrap().0;
            let b = constraints::dihedral([0, 1, 2, 3], &minus).unwrap().0;
            assert!(
                ((a - b) / (2. * epsilon) - gradient[vertex][axis]).abs() < 1e-7,
                "v{vertex} axis{axis}: numeric={} analytic={}",
                (a - b) / (2. * epsilon),
                gradient[vertex][axis]
            );
        }
    }
}

#[test]
fn ground_torso_and_capsule_project_to_exterior() {
    for collider in [
        Collider::Ground { height: 2. },
        Collider::Torso {
            center: [0.5, 2., 0.5],
            radii: [2., 1., 2.],
        },
        Collider::Capsule {
            a: [0.5, 1., 0.5],
            b: [0.5, 3., 0.5],
            radius: 1.,
        },
    ] {
        let mut input = square();
        input.colliders.push(collider);
        let mut cloth = Cloth::new(input).unwrap();
        assert!(cloth.diagnostics().initial_intersections > 0);
        let result = cloth
            .step(
                StepSettings {
                    iterations: 30,
                    ..settings()
                },
                || false,
            )
            .unwrap();
        assert!(
            result.max_contact_error < 1e-6,
            "{collider:?} residual {}",
            result.max_contact_error
        );
        assert!(result.contacts > 0);
    }
}

fn layers(pin_bottom: bool) -> ClothInput {
    let mut input = square();
    for p in &mut input.positions {
        p[1] = 1.;
    }
    let mut upper = input.positions.clone();
    for p in &mut upper {
        p[1] += 0.001;
    }
    input.positions.extend(upper);
    input.material_uv.extend(input.material_uv.clone());
    input.triangles.extend([[4, 5, 6], [4, 6, 7]]);
    input.triangle_materials = vec![
        Material {
            collision_thickness: 0.01,
            damping: 0.,
            ..Default::default()
        };
        4
    ];
    if pin_bottom {
        input.pins = (0..4)
            .map(|vertex| Pin {
                vertex,
                target: input.positions[vertex],
            })
            .collect();
    }
    input
}

#[test]
fn layers_contact_and_transfer_motion_to_the_other_layer() {
    let mut fixed = Cloth::new(layers(true)).unwrap();
    let result = fixed
        .step(
            StepSettings {
                iterations: 30,
                ..settings()
            },
            || false,
        )
        .unwrap();
    assert!(result.contacts > 0);
    assert!(
        result.max_contact_error < 1e-6,
        "residual {}",
        result.max_contact_error
    );
    assert!(fixed.positions()[4][1] >= 1.02 - 1e-6);
    let input = layers(false);
    let initial_top = input.positions[4][1];
    let initial_bottom = input.positions[0][1];
    let mut cloth = Cloth::new(input).unwrap();
    cloth
        .step(
            StepSettings {
                iterations: 30,
                ..settings()
            },
            || false,
        )
        .unwrap();
    assert!(cloth.positions()[0][1] < initial_bottom - 0.001);
    assert!(cloth.positions()[4][1] > initial_top + 0.001);
}

#[test]
fn cancellation_after_measurement_preserves_peak_strain() {
    let mut input = square();
    for p in &mut input.positions {
        p[0] *= 1.5;
    }
    let mut cloth = Cloth::new(input).unwrap();
    let before = cloth.diagnostics().clone();
    let mut complete = cloth.clone();
    let total = std::cell::Cell::new(0);
    complete
        .step(settings(), || {
            total.set(total.get() + 1);
            false
        })
        .unwrap();
    assert_ne!(complete.diagnostics().peak_strain, before.peak_strain);
    let calls = std::cell::Cell::new(0);
    assert_eq!(
        cloth
            .step(settings(), || {
                calls.set(calls.get() + 1);
                calls.get() == total.get()
            })
            .unwrap_err(),
        SimulationError::Cancelled
    );
    assert_eq!(calls.get(), total.get());
    assert_eq!(cloth.diagnostics().peak_strain, before.peak_strain);
    assert_eq!(
        cloth.diagnostics().max_stretch_error,
        before.max_stretch_error
    );
    assert_eq!(cloth.positions(), cloth.input.positions);
    assert!(cloth.velocities().iter().all(|v| *v == [0.; 3]));
    assert_eq!(cloth.simulation_time(), 0.);
}

#[test]
fn cancellation_and_failures_preserve_old_state_and_allow_recovery() {
    let mut input = square();
    for position in square().positions {
        input.positions.push(add(position, [3., 0., 0.]));
    }
    input.material_uv.extend(square().material_uv);
    input.triangles.extend([[4, 5, 6], [4, 6, 7]]);
    input.triangle_materials.extend(square().triangle_materials);
    for position in &mut input.positions {
        position[0] *= 1.5;
    }
    let mut cloth = Cloth::new(input).unwrap();
    assert_eq!(cloth.membrane.len(), 4);
    let before = cloth.positions().to_vec();
    let before_peak = cloth.diagnostics().peak_strain;
    let before_state = format!("{cloth:?}");
    let stage_calls = std::cell::Cell::new(0);
    let mut projected = before.clone();
    constraints::solve_material(
        &cloth.membrane,
        &mut projected,
        &cloth.inverse_masses,
        settings().dt,
        &mut vec![[0.; 3]; before.len()],
        &|| {
            stage_calls.set(stage_calls.get() + 1);
            false
        },
    )
    .unwrap();
    assert_ne!(projected, before);
    let stop_at = 3 + stage_calls.get() + 1;
    let counter = std::cell::Cell::new(0);
    let error = cloth
        .step(settings(), || {
            counter.set(counter.get() + 1);
            // The first material sweep has already changed the staged positions.
            counter.get() == stop_at
        })
        .unwrap_err();
    assert_eq!(error, SimulationError::Cancelled);
    assert_eq!(counter.get(), stop_at);
    assert_eq!(format!("{cloth:?}"), before_state);
    assert_eq!(cloth.positions(), before);
    assert_eq!(cloth.diagnostics().peak_strain, before_peak);
    assert_eq!(cloth.simulation_time(), 0.);
    assert!(
        cloth
            .step(
                StepSettings {
                    dt: f64::NAN,
                    ..settings()
                },
                || false
            )
            .is_err()
    );
    assert_eq!(cloth.positions(), before);
    assert_eq!(cloth.diagnostics().peak_strain, before_peak);
    assert_eq!(format!("{cloth:?}"), before_state);
    assert!(matches!(
        cloth.step(
            StepSettings {
                gravity: [0., -1e100, 0.],
                ..settings()
            },
            || false
        ),
        Err(SimulationError::Unstable(_))
    ));
    assert_eq!(cloth.positions(), before);
    assert_eq!(cloth.diagnostics().peak_strain, before_peak);
    assert_eq!(format!("{cloth:?}"), before_state);
    cloth.step(settings(), || false).unwrap();
    assert!(cloth.diagnostics().finite);
    assert_ne!(cloth.positions(), before);
}

#[test]
fn malformed_external_geometry_material_and_constraints_are_rejected() {
    let mut input = square();
    input.triangles[0][0] = usize::MAX;
    assert!(Cloth::new(input).is_err());
    let mut input = square();
    input.material_uv[0] = [f64::NAN, 0.];
    assert!(Cloth::new(input).is_err());
    let mut input = square();
    input.triangle_materials[0].areal_density = -1.;
    assert!(Cloth::new(input).is_err());
    let mut input = square();
    input.stitches.push(Stitch {
        a: WeightedPoint {
            vertices: [0, 1],
            weights: [-0.1, 1.1],
        },
        b: WeightedPoint::vertex(2),
        compliance: 0.,
    });
    assert!(Cloth::new(input).is_err());
    let mut input = square();
    input.pins = vec![
        Pin {
            vertex: 0,
            target: [0.; 3]
        };
        2
    ];
    assert!(Cloth::new(input).is_err());
}

fn hanging_grid(n: usize, stiffness: f64) -> ClothInput {
    let mut input = ClothInput::default();
    for j in 0..=n {
        for i in 0..=n {
            let u = i as f64 / n as f64;
            let v = j as f64 / n as f64;
            input.positions.push([u, 1. + v, 0.]);
            input.material_uv.push([u, v]);
            if j == n {
                input.pins.push(Pin {
                    vertex: j * (n + 1) + i,
                    target: [u, 1. + v, 0.],
                });
            }
        }
    }
    for j in 0..n {
        for i in 0..n {
            let v = j * (n + 1) + i;
            input
                .triangles
                .extend([[v, v + 1, v + n + 2], [v, v + n + 2, v + n + 1]]);
        }
    }
    input.triangle_materials = vec![
        Material {
            warp_stiffness: stiffness,
            weft_stiffness: stiffness,
            shear_stiffness: stiffness,
            bending_stiffness: 0.,
            damping: 8.,
            ..Default::default()
        };
        input.triangles.len()
    ];
    input
}
fn settle(n: usize, stiffness: f64, substeps: u32) -> (f64, f64) {
    let mut cloth = Cloth::new(hanging_grid(n, stiffness)).unwrap();
    for _ in 0..240 {
        cloth
            .step(
                StepSettings {
                    dt: 1. / 120.,
                    substeps,
                    iterations: 16,
                    gravity: [0., -9.81, 0.],
                    ..Default::default()
                },
                || false,
            )
            .unwrap();
    }
    let bottom = cloth.positions()[..=n].iter().map(|p| p[1]).sum::<f64>() / (n + 1) as f64;
    (1. - bottom, cloth.masses().iter().sum())
}

#[test]
fn physical_stiffness_changes_extension_and_refinement_preserves_mass_and_sag() {
    let (coarse, mass_a) = settle(2, 100., 1);
    let (fine, mass_b) = settle(4, 100., 2);
    let (stiff, _) = settle(2, 1000., 1);
    let analytic_extension = 0.2 * 9.81 / (2. * 100.);
    eprintln!(
        "hanging SI patch: coarse_extension={coarse:.9}m fine_extension={fine:.9}m stiff_extension={stiff:.9}m mass={mass_a:.9}kg analytic={analytic_extension:.9}m"
    );
    assert!((mass_a - 0.2).abs() < 1e-12 && (mass_b - 0.2).abs() < 1e-12);
    assert!(
        coarse > 0.004 && fine > 0.004,
        "coarse={coarse} fine={fine}"
    );
    assert!((coarse - fine).abs() < 0.003, "coarse={coarse} fine={fine}");
    assert!(
        (fine - analytic_extension).abs() < 0.004,
        "fine={fine} linear continuum={analytic_extension}"
    );
    assert!(stiff < coarse * 0.3, "stiff={stiff} soft={coarse}");
}

#[test]
fn bending_reaches_independent_pinned_hinge_incremental_energy_equilibrium() {
    let rest = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., -1., 0.]];
    let bend = constraints::Bend::new([0, 1, 2, 3], &rest, 0.5);
    let (sin, cos) = 1_f64.sin_cos();
    let predicted = [0., cos, sin];
    let mut x = rest;
    x[2] = predicted;
    let mut correction = [[0.; 3]; 4];
    for _ in 0..32 {
        bend.solve(&mut x, &[0., 0., 100., 0.], 0.1, &mut correction, &|| false)
            .unwrap();
    }

    // Independent polar minimization: m/h²=1, k=0.5, r=cos(theta-1).
    // Its angular equation is strictly increasing for 0<=theta<=1.
    let (mut low, mut high) = (0_f64, 1_f64);
    for _ in 0..64 {
        let theta = (low + high) * 0.5;
        if 0.5 * theta + 0.5 * (2. * (theta - 1.)).sin() < 0. {
            low = theta;
        } else {
            high = theta;
        }
    }
    let theta = (low + high) * 0.5;
    let radius = (theta - 1.).cos();
    let expected = [0., radius * theta.cos(), radius * theta.sin()];
    let [_, y, z] = x[2];
    let angle = z.atan2(y);
    let radius_squared = y * y + z * z;
    let gradient = [
        0.,
        y - predicted[1] - 0.5 * angle * z / radius_squared,
        z - predicted[2] + 0.5 * angle * y / radius_squared,
    ];
    let position_error = length(sub(x[2], expected));
    let stationarity = length(gradient);
    println!(
        "pinned hinge q={:?} reference={expected:?} position_error={position_error:e}m stationarity={stationarity:e}N",
        x[2]
    );
    assert!(position_error < 1e-8 && stationarity < 1e-6);
    for i in [0, 1, 3] {
        assert_eq!(x[i], rest[i]);
    }
}

#[test]
fn bending_reduces_fold_angle_without_translating_center_of_mass() {
    let rest = vec![[0., 0., 0.], [1., 0., 0.], [0.5, 1., 0.], [0.5, -1., 0.]];
    let bend = constraints::Bend::new([0, 1, 2, 3], &rest, 0.1);
    let mut x = rest;
    x[2][2] = 0.4;
    let before_angle = constraints::dihedral([0, 1, 2, 3], &x).unwrap().0.abs();
    let before_center = x.iter().copied().fold([0.; 3], add);
    let mut correction = [[0.; 3]; 4];
    for _ in 0..8 {
        bend.solve(&mut x, &[1.; 4], 0.1, &mut correction, &|| false)
            .unwrap();
    }
    let after_angle = constraints::dihedral([0, 1, 2, 3], &x).unwrap().0.abs();
    assert!(after_angle < before_angle);
    assert!(length(sub(x.iter().copied().fold([0.; 3], add), before_center)) < 1e-12);
}

#[test]
fn bending_cancellation_retains_positions_and_displacement_history() {
    let rest = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., -1., 0.]];
    let bend = constraints::Bend::new([0, 1, 2, 3], &rest, 0.5);
    let mut x = rest;
    x[2] = [0., 1_f64.cos(), 1_f64.sin()];
    let predictor = x;
    let mut correction = [[0.; 3]; 4];
    bend.solve(&mut x, &[0., 0., 100., 0.], 0.1, &mut correction, &|| false)
        .unwrap();
    assert_ne!(x, predictor);
    for i in 0..4 {
        assert_eq!(correction[i], sub(x[i], predictor[i]));
    }
    let before = x;
    let history = correction;
    let calls = std::cell::Cell::new(0);
    bend.solve(&mut x, &[0., 0., 100., 0.], 0.1, &mut correction, &|| {
        calls.set(calls.get() + 1);
        false
    })
    .unwrap();
    assert_ne!(x, before);
    for stop_at in 1..=calls.get() {
        let mut x = before;
        let mut correction = history;
        let polls = std::cell::Cell::new(0);
        assert_eq!(
            bend.solve(&mut x, &[0., 0., 100., 0.], 0.1, &mut correction, &|| {
                polls.set(polls.get() + 1);
                polls.get() == stop_at
            }),
            Err(SimulationError::Cancelled)
        );
        assert_eq!(polls.get(), stop_at);
        assert_eq!(x, before);
        assert_eq!(correction, history);
    }
    let extreme = constraints::Bend::new([0, 1, 2, 3], &rest, 1e308);
    let mut x = before;
    let mut correction = history;
    assert!(matches!(
        extreme.solve(&mut x, &[0., 0., 100., 0.], 1., &mut correction, &|| false),
        Err(SimulationError::Unstable(_))
    ));
    assert_eq!(x, before);
    assert_eq!(correction, history);
}

#[test]
fn bending_displacement_history_resets_between_substeps() {
    let mut input = square();
    for material in &mut input.triangle_materials {
        material.bending_stiffness = 0.5;
    }
    input.pins = [0, 1, 3]
        .map(|vertex| Pin {
            vertex,
            target: input.positions[vertex],
        })
        .to_vec();
    let mut combined = Cloth::new(input).unwrap();
    combined.velocities[2][1] = 4.;
    let mut split = combined.clone();
    let combined_settings = StepSettings {
        dt: 0.02,
        substeps: 2,
        iterations: 4,
        ..settings()
    };
    combined.step(combined_settings, || false).unwrap();
    let split_settings = StepSettings {
        dt: 0.01,
        substeps: 1,
        ..combined_settings
    };
    split.step(split_settings, || false).unwrap();
    split.step(split_settings, || false).unwrap();
    assert_eq!(combined.positions(), split.positions());
    assert_eq!(combined.velocities(), split.velocities());
    assert_eq!(combined.simulation_time(), split.simulation_time());
}

#[test]
fn swept_layer_crossing_is_resolved_and_ground_friction_resists_slide() {
    let mut input = layers(true);
    for p in &mut input.positions[4..] {
        p[1] = 1.1;
    }
    let mut cloth = Cloth::new(input).unwrap();
    cloth
        .step(
            StepSettings {
                dt: 0.02,
                substeps: 1,
                iterations: 16,
                gravity: [0., -1000., 0.],
                ..Default::default()
            },
            || false,
        )
        .unwrap();
    assert!(cloth.positions()[4..].iter().all(|p| p[1] >= 1.02 - 1e-6));
    let slide = |friction| {
        let mut input = square();
        for p in &mut input.positions {
            p[1] = 0.002;
        }
        for m in &mut input.triangle_materials {
            m.friction = friction;
        }
        input.colliders.push(Collider::Ground { height: 0. });
        let mut cloth = Cloth::new(input).unwrap();
        for _ in 0..40 {
            cloth
                .step(
                    StepSettings {
                        gravity: [1., -9.81, 0.],
                        ..settings()
                    },
                    || false,
                )
                .unwrap();
        }
        cloth.positions()[0][0]
    };
    let frictionless = slide(0.);
    let rough = slide(0.5);
    assert!(frictionless > 0.07);
    assert!(rough.abs() < 1e-8, "rough slide={rough}");
}

#[test]
fn grain_rotation_changes_the_physical_hanging_response() {
    let response = |angle| {
        let mut input = hanging_grid(2, 100.);
        for material in &mut input.triangle_materials {
            material.warp_stiffness = 50.;
            material.weft_stiffness = 1000.;
            material.grain_angle = angle;
        }
        let mut cloth = Cloth::new(input).unwrap();
        for _ in 0..240 {
            cloth
                .step(
                    StepSettings {
                        dt: 1. / 120.,
                        substeps: 1,
                        iterations: 16,
                        gravity: [0., -9.81, 0.],
                        ..Default::default()
                    },
                    || false,
                )
                .unwrap();
        }
        1. - cloth.positions()[..3].iter().map(|p| p[1]).sum::<f64>() / 3.
    };
    let stiff = response(0.);
    let rotated = response(std::f64::consts::FRAC_PI_2);
    assert!(rotated > 5. * stiff, "rotated={rotated} original={stiff}");
}

#[test]
fn seam_assembly_is_bounded_monotone_cancellable_and_resettable() {
    let mut input = layers(false);
    for p in &mut input.positions[4..] {
        p[1] = 1.4;
    }
    input.stitches = (0..4)
        .map(|i| Stitch {
            a: WeightedPoint::vertex(i),
            b: WeightedPoint::vertex(i + 4),
            compliance: 0.,
        })
        .collect();
    let mut cloth = Cloth::new(input).unwrap();
    assert!((cloth.diagnostics().seam_assembly_remaining - 0.4).abs() < 1e-12);
    let settings = StepSettings {
        dt: 0.1,
        substeps: 4,
        iterations: 16,
        gravity: [0.; 3],
        seam_closure_speed: 0.5,
    };
    cloth.step(settings, || false).unwrap();
    assert!((cloth.diagnostics().seam_assembly_remaining - 0.35).abs() < 1e-12);
    assert!((cloth.diagnostics().max_seam_error - 0.35).abs() < 1e-5);
    assert!(matches!(
        cloth.step(settings, || true),
        Err(SimulationError::Cancelled)
    ));
    assert!((cloth.diagnostics().seam_assembly_remaining - 0.35).abs() < 1e-12);
    cloth
        .step(
            StepSettings {
                seam_closure_speed: 0.,
                ..settings
            },
            || false,
        )
        .unwrap();
    assert!((cloth.diagnostics().seam_assembly_remaining - 0.35).abs() < 1e-12);
    for _ in 0..8 {
        cloth.step(settings, || false).unwrap();
    }
    assert_eq!(cloth.diagnostics().seam_assembly_remaining, 0.);
    assert!(cloth.diagnostics().max_seam_error < 1e-5);
    cloth.reset();
    assert!((cloth.diagnostics().seam_assembly_remaining - 0.4).abs() < 1e-12);
}

#[test]
fn finite_diagnostics_do_not_claim_satisfied_contact() {
    let mut input = square();
    input.pins = (0..4)
        .map(|vertex| Pin {
            vertex,
            target: input.positions[vertex],
        })
        .collect();
    input.colliders.push(Collider::Ground { height: 3. });
    let mut cloth = Cloth::new(input).unwrap();
    let result = cloth.step(settings(), || false).unwrap();
    assert!(result.finite);
    assert_eq!(result.initial_intersections, 4);
    assert!(result.max_contact_error > 1.);
    assert_eq!(result.contacts, 0);
}

#[test]
fn constructor_cancels_inside_initial_contact_query_without_publishing_state() {
    use std::cell::Cell;
    // A fully overlapping hierarchy still has to poll during expensive candidate work.
    let patches = CANCELLATION_CHECK_INTERVAL + 1;
    let mut input = ClothInput::default();
    for _ in 0..patches {
        let offset = input.positions.len();
        input
            .positions
            .extend([[0., 1., 0.], [1., 1., 0.], [0., 1., 1.]]);
        input.material_uv.extend([[0., 0.], [1., 0.], [0., 1.]]);
        input.triangles.push([offset, offset + 1, offset + 2]);
        input.triangle_materials.push(Material::default());
    }
    let total = Cell::new(0_usize);
    let ordinary = Cloth::new(input.clone()).unwrap();
    let complete = Cloth::new_cancellable(input.clone(), || {
        total.set(total.get() + 1);
        false
    })
    .unwrap();
    assert_eq!(complete.positions(), ordinary.positions());
    assert_eq!(complete.velocities(), ordinary.velocities());
    assert_eq!(complete.masses(), ordinary.masses());
    assert_eq!(
        complete.diagnostics().initial_intersections,
        3 * patches * (patches - 1)
    );
    assert_eq!(
        complete.diagnostics().initial_intersections,
        ordinary.diagnostics().initial_intersections
    );
    assert_eq!(
        complete.diagnostics().max_contact_error,
        2. * Material::default().collision_thickness
    );
    assert_eq!(
        complete.diagnostics().max_contact_error,
        ordinary.diagnostics().max_contact_error
    );
    assert_eq!(
        complete.diagnostics().max_stretch_error,
        ordinary.diagnostics().max_stretch_error
    );
    assert_eq!(complete.simulation_time(), 0.);
    assert!(complete.diagnostics().finite);

    let analysis = Cell::new(0_usize);
    let analysis_errors = complete
        .contact_errors_cancellable(complete.positions(), &|| {
            analysis.set(analysis.get() + 1);
            false
        })
        .unwrap();
    assert_eq!(
        analysis_errors.intersections,
        complete.diagnostics().initial_intersections
    );
    assert!(
        analysis.get() > 2 * input.positions.len(),
        "candidate-loop polling must occur inside each vertex, not only outside it"
    );
    // Cancel in the last vertex's query, before the final constructor publication check.
    let stop_at = total.get() - 2;
    assert!(stop_at > total.get() - analysis.get());
    let calls = Cell::new(0);
    let cancelled = Cell::new(false);
    let mut accepted = Cloth::new(square()).unwrap();
    let previous = accepted.positions().to_vec();
    match Cloth::new_cancellable(input, || {
        assert!(
            !cancelled.get(),
            "initialization continued polling after cancellation"
        );
        calls.set(calls.get() + 1);
        cancelled.set(calls.get() == stop_at);
        cancelled.get()
    }) {
        Err(SimulationError::Cancelled) => {}
        Err(error) => panic!("expected cancellation inside initial contacts, got {error}"),
        Ok(next) => {
            accepted = next;
            panic!(
                "cancelled initialization published {} vertices",
                accepted.positions().len()
            );
        }
    }
    assert!(cancelled.get());
    assert_eq!(calls.get(), stop_at);
    assert_eq!(accepted.positions(), previous);
    assert_eq!(accepted.simulation_time(), 0.);
}

#[test]
fn initial_hierarchy_matches_brute_force_for_contacts_thickness_and_sewing_filters() {
    let compare = |input: ClothInput| {
        let cloth = Cloth::new(input.clone()).unwrap();
        // Keep an exhaustive diagnostic oracle independent of the hierarchy.
        let expected = cloth.exhaustive_contact_errors(&input.positions);
        assert_eq!(cloth.diagnostics().max_contact_error, expected.maximum);
        assert_eq!(
            cloth.diagnostics().initial_intersections,
            expected.intersections
        );
        assert_eq!(
            cloth.diagnostics().capsule_surface_uncertain_pairs,
            expected.uncertain_capsule_pairs
        );
        assert_eq!(cloth.positions(), input.positions);
        assert_eq!(cloth.simulation_time(), 0.);
        assert!(cloth.diagnostics().finite);
        expected.intersections
    };
    assert_eq!(compare(square()), 0); // Adjacent triangles are excluded.
    for separation in [0., 0.001, 0.004, 0.02, 0.02000001, 1.] {
        for thickness in [1e-9, 0.002, 0.01] {
            let mut input = layers(false);
            for p in &mut input.positions[4..] {
                p[1] = 1. + separation;
            }
            for material in &mut input.triangle_materials {
                material.collision_thickness = thickness;
            }
            compare(input.clone());
            for (i, material) in input.triangle_materials.iter_mut().enumerate() {
                material.collision_thickness = thickness * (i + 1) as f64;
            }
            compare(input.clone());
            input.stitches.push(Stitch {
                a: WeightedPoint {
                    vertices: [0, 1],
                    weights: [0.25, 0.75],
                },
                b: WeightedPoint {
                    vertices: [4, 5],
                    weights: [0.75, 0.25],
                },
                compliance: 0.,
            });
            compare(input);
        }
    }
    let mut crossing = layers(false);
    crossing.positions[4..].copy_from_slice(&[
        [0.5, 0.5, 0.5],
        [0.5, 1., 0.5],
        [0.5, 1.5, 1.5],
        [0.5, 1.5, 0.],
    ]);
    assert!(compare(crossing) > 0);
    let mut colliders = layers(false);
    colliders.colliders = vec![
        Collider::Ground { height: 1.1 },
        Collider::Torso {
            center: [0.5, 1., 0.5],
            radii: [1.; 3],
        },
        Collider::Capsule {
            a: [0.5, 0., 0.5],
            b: [0.5, 2., 0.5],
            radius: 1.,
        },
    ];
    assert!(compare(colliders) > 0);
    for n in [2, 5, 12] {
        let patch = hanging_grid(n, 800.);
        for layers in [1, 2, 3] {
            let mut input = ClothInput::default();
            for layer in 0..layers {
                let offset = input.positions.len();
                input.positions.extend(
                    patch
                        .positions
                        .iter()
                        .map(|p| [p[0], p[1], p[2] + layer as f64 * 0.001]),
                );
                input.material_uv.extend_from_slice(&patch.material_uv);
                input
                    .triangles
                    .extend(patch.triangles.iter().map(|tri| tri.map(|v| v + offset)));
                input
                    .triangle_materials
                    .extend(patch.triangle_materials.iter().map(|m| Material {
                        collision_thickness: 0.002 * (layer + 1) as f64,
                        ..*m
                    }));
            }
            let expected = compare(input.clone());
            input.triangles.reverse();
            input.triangle_materials.reverse();
            assert_eq!(compare(input), expected);
        }
    }
}

#[test]
fn cancellable_constructor_keeps_invalid_input_checks_and_honors_early_cancel() {
    let mut invalid = square();
    invalid.triangles[0][0] = usize::MAX;
    assert!(matches!(
        Cloth::new_cancellable(invalid, || false),
        Err(SimulationError::InvalidInput(_))
    ));
    let calls = std::cell::Cell::new(0);
    assert!(matches!(
        Cloth::new_cancellable(square(), || {
            calls.set(calls.get() + 1);
            true
        }),
        Err(SimulationError::Cancelled)
    ));
    assert_eq!(calls.get(), 1);
}

fn capsule_triangle_input(
    triangle: [Vec3; 3],
    thickness: f64,
    colliders: Vec<Collider>,
) -> ClothInput {
    ClothInput {
        positions: triangle.to_vec(),
        material_uv: triangle.map(|p| [p[0], p[1]]).to_vec(),
        triangles: vec![[0, 1, 2]],
        triangle_materials: vec![Material {
            damping: 0.,
            friction: 0.,
            bending_stiffness: 0.,
            collision_thickness: thickness,
            ..Default::default()
        }],
        colliders,
        ..Default::default()
    }
}

fn edge_capsule_input() -> ClothInput {
    capsule_triangle_input(
        [[-1., 0., 0.1], [1., 0., 0.1], [0., 2., 0.1]],
        0.05,
        vec![Collider::Capsule {
            a: [0., -1., 0.],
            b: [0., 1., 0.],
            radius: 0.2,
        }],
    )
}

#[test]
fn capsule_surface_projection_uses_barycentric_inverse_masses_and_existing_friction() {
    let input = edge_capsule_input();
    for &p in &input.positions {
        assert!(contact::collider_contact(input.colliders[0], p, 0.05).is_none());
    }
    for friction in [0., 0.5] {
        let mut cloth = Cloth::new(input.clone()).unwrap();
        cloth.inverse_masses = vec![1., 3., 0.];
        cloth.friction.fill(friction);
        let mut x = input.positions.clone();
        let previous: Vec<_> = x.iter().map(|&p| sub(p, [0.1, 0., 0.])).collect();
        assert_eq!(
            cloth
                .project_colliders(&mut x, &previous, &|| false)
                .unwrap(),
            1
        );
        // beta=(1/2,1/2,0), sum(w*beta²)=1, depth=.15 m.
        for (j, weight) in [1., 3., 0.].into_iter().enumerate() {
            assert!((x[j][2] - input.positions[j][2] - weight * 0.5 * 0.15).abs() < 1e-14);
            assert!(
                (x[j][0] - input.positions[j][0] + weight * 0.5 * friction * 0.15).abs() < 1e-14
            );
            assert_eq!(x[j][1], input.positions[j][1]);
        }
        assert_eq!(x[2], input.positions[2]);
    }
}

#[test]
fn capsule_surface_fixed_nodes_report_conflict_and_reset_restores_full_occupancy() {
    let mut input = edge_capsule_input();
    input.pins = input
        .positions
        .iter()
        .enumerate()
        .map(|(vertex, &target)| Pin { vertex, target })
        .collect();
    let rest = input.positions.clone();
    let mut cloth = Cloth::new(input).unwrap();
    assert_eq!(cloth.diagnostics().initial_intersections, 1);
    assert!((cloth.diagnostics().max_contact_error - 0.15).abs() < 1e-14);
    let result = cloth.step(settings(), || false).unwrap();
    assert!(result.finite);
    assert_eq!(result.contacts, 0);
    assert!((result.max_contact_error - 0.15).abs() < 1e-14);
    assert_eq!(cloth.positions(), rest);
    cloth.reset();
    assert_eq!(cloth.diagnostics().initial_intersections, 1);
    assert!((cloth.diagnostics().max_contact_error - 0.15).abs() < 1e-14);
    assert_eq!(cloth.diagnostics().capsule_surface_uncertain_pairs, 0);
}

#[test]
fn final_capsule_surface_residual_rechecks_contacts_undone_by_a_later_capsule() {
    let triangle = [[-1., -1., 0.], [1., -1., 0.], [0., 2., 0.]];
    let input = capsule_triangle_input(
        triangle,
        0.01,
        [-0.2, 0.2]
            .map(|z| Collider::Capsule {
                a: [0., 0., z],
                b: [0., 0., z],
                radius: 0.25,
            })
            .to_vec(),
    );
    let mut cloth = Cloth::new(input).unwrap();
    assert_eq!(cloth.diagnostics().initial_intersections, 2);
    assert!((cloth.diagnostics().max_contact_error - 0.06).abs() < 1e-14);
    let result = cloth
        .step(
            StepSettings {
                iterations: 1,
                ..settings()
            },
            || false,
        )
        .unwrap();
    assert!(result.finite);
    assert_eq!(result.contacts, 2);
    // Equal beta and masses translate the whole face: +.06 m then -.12 m.
    for p in cloth.positions() {
        assert!((p[2] + 0.06).abs() < 1e-14);
    }
    assert!((result.max_contact_error - 0.12).abs() < 1e-14);
    cloth.reset();
    assert_eq!(cloth.positions(), triangle);
    assert!((cloth.diagnostics().max_contact_error - 0.06).abs() < 1e-14);
    assert_eq!(cloth.diagnostics().initial_intersections, 2);
    assert_eq!(cloth.simulation_time(), 0.);
}

#[test]
fn capsule_axis_face_crossing_has_finite_deterministic_recovery_and_honest_residual() {
    let triangle = [[-2., -2., 0.], [2., -2., 0.], [0., 2., 0.]];
    let a = [0., 0., -1.];
    let b = [0., 0., 1.];
    let input = capsule_triangle_input(
        triangle,
        0.05,
        vec![Collider::Capsule { a, b, radius: 0.2 }],
    );
    for &p in &input.positions {
        assert!(contact::collider_contact(input.colliders[0], p, 0.05).is_none());
    }
    let mut cloth = Cloth::new(input).unwrap();
    assert_eq!(cloth.diagnostics().initial_intersections, 1);
    assert!((cloth.diagnostics().max_contact_error - 0.25).abs() < 1e-14);
    let mut repeat = cloth.clone();
    let settings = StepSettings {
        iterations: 2,
        ..settings()
    };
    let result = cloth.step(settings, || false).unwrap();
    repeat.step(settings, || false).unwrap();
    assert!(result.finite);
    assert!(result.contacts > 0);
    assert_eq!(cloth.positions(), repeat.positions());
    assert_ne!(cloth.positions(), triangle);
    let current = cloth.positions();
    let feature = capsule::closest([current[0], current[1], current[2]], a, b);
    assert!(result.max_contact_error >= (0.25 - feature.distance).max(0.) - 1e-14);
    assert!(cloth.positions().iter().copied().all(finite));
}

#[test]
fn conservative_skinny_capsule_pairs_remain_visible_and_exact_collapsed_faces_are_not_dropped() {
    let triangle = [[-1., 0., 0.], [1., 0., 0.], [0., 1e-8, 0.]];
    let mut input = capsule_triangle_input(
        triangle,
        1e-4,
        vec![Collider::Capsule {
            a: [0., 0.5e-8, 1e-3],
            b: [0., 0.5e-8, 1e-3],
            radius: 2e-3,
        }],
    );
    input.pins = triangle
        .into_iter()
        .enumerate()
        .map(|(vertex, target)| Pin { vertex, target })
        .collect();
    let mut cloth = Cloth::new(input).unwrap();
    assert_eq!(cloth.diagnostics().capsule_surface_uncertain_pairs, 1);
    assert!(cloth.diagnostics().max_contact_error >= 0.0011);
    let result = cloth.step(settings(), || false).unwrap();
    assert!(result.finite);
    assert_eq!(result.capsule_surface_uncertain_pairs, 1);
    assert!(result.max_contact_error >= 0.0011);
    cloth.reset();
    assert_eq!(cloth.diagnostics().capsule_surface_uncertain_pairs, 1);

    let mut input = edge_capsule_input();
    input.pins = input
        .positions
        .iter()
        .enumerate()
        .map(|(vertex, &target)| Pin {
            vertex,
            target: if vertex == 2 { [0., 0., 0.1] } else { target },
        })
        .collect();
    let mut collapsed = Cloth::new(input).unwrap();
    let result = collapsed.step(settings(), || false).unwrap();
    assert!(result.finite);
    assert_eq!(result.capsule_surface_uncertain_pairs, 0);
    assert!((result.max_contact_error - 0.15).abs() < 1e-14);
    assert_eq!(result.contacts, 0);
}

#[test]
fn capsule_projection_and_final_surface_analysis_cancellation_preserve_the_accepted_step() {
    use std::cell::Cell;
    let mut input = edge_capsule_input();
    let second: Vec<_> = input
        .positions
        .iter()
        .map(|&p| add(p, [5., 0., 0.]))
        .collect();
    input.positions.extend(second);
    input.material_uv.extend([[4., 0.], [6., 0.], [5., 2.]]);
    input.triangles.push([3, 4, 5]);
    input.triangle_materials.push(input.triangle_materials[0]);
    input.stitches.push(Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(3),
        compliance: 1e-8,
    });
    let mut accepted = Cloth::new(input).unwrap();
    accepted.step(settings(), || false).unwrap();
    let mut complete = accepted.clone();
    let total = Cell::new(0);
    complete
        .step(settings(), || {
            total.set(total.get() + 1);
            false
        })
        .unwrap();
    let collider_calls = Cell::new(0);
    let previous = accepted.input.positions.clone();
    let mut candidate = previous.clone();
    accepted
        .project_colliders(&mut candidate, &previous, &|| {
            collider_calls.set(collider_calls.get() + 1);
            false
        })
        .unwrap();
    assert_ne!(candidate, previous);
    let calls = Cell::new(0);
    let mut partial = previous.clone();
    assert!(matches!(
        accepted.project_colliders(&mut partial, &previous, &|| {
            calls.set(calls.get() + 1);
            calls.get() == collider_calls.get()
        }),
        Err(SimulationError::Cancelled)
    ));
    assert_ne!(partial, previous);
    let old_positions = accepted.positions().to_vec();
    let old_velocities = accepted.velocities().to_vec();
    let old_targets = accepted.seam_remaining.clone();
    let old_diagnostics = format!("{:?}", accepted.diagnostics());
    // Cover every actual callback, including projected contacts and final surface analysis.
    for stop_at in 1..=total.get() {
        let calls = Cell::new(0);
        let error = accepted
            .step(settings(), || {
                calls.set(calls.get() + 1);
                calls.get() == stop_at
            })
            .unwrap_err();
        assert_eq!(error, SimulationError::Cancelled);
        assert_eq!(calls.get(), stop_at);
        assert_eq!(accepted.positions(), old_positions);
        assert_eq!(accepted.velocities(), old_velocities);
        assert_eq!(accepted.seam_remaining, old_targets);
        assert_eq!(format!("{:?}", accepted.diagnostics()), old_diagnostics);
    }
    accepted.step(settings(), || false).unwrap();
    assert_eq!(accepted.positions(), complete.positions());
    assert_eq!(accepted.velocities(), complete.velocities());
    assert_eq!(accepted.seam_remaining, complete.seam_remaining);
}

#[test]
fn material_stage_retains_each_input_face_once_in_order() {
    let mut input = square();
    input.positions.push([-0.3, 2., 0.1]);
    input.material_uv.push([-0.3, 0.1]);
    input.triangles.push([0, 3, 4]);
    input.triangle_materials.push(Material::default());
    let cloth = Cloth::new(input).unwrap();
    assert_eq!(
        cloth
            .membrane
            .iter()
            .map(|m| m.vertices)
            .collect::<Vec<_>>(),
        vec![[0, 1, 2], [0, 2, 3], [0, 3, 4]],
    );
    assert_eq!(
        format!("{:?}", cloth.membrane),
        format!("{:?}", cloth.clone().membrane)
    );
}

#[test]
fn global_material_zero_rhs_pins_and_cancellation_preserve_stage_state() {
    let cloth = Cloth::new(square()).unwrap();
    let before = cloth.positions().to_vec();
    let mut x = before.clone();
    let mut correction = vec![[0.; 3]; x.len()];
    constraints::solve_material(
        &cloth.membrane,
        &mut x,
        &cloth.inverse_masses,
        settings().dt,
        &mut correction,
        &|| false,
    )
    .unwrap();
    assert_eq!(x, before);
    assert_eq!(correction, vec![[0.; 3]; x.len()]);

    x.iter_mut().for_each(|p| p[0] *= 1.5);
    let strained = x.clone();
    constraints::solve_material(
        &cloth.membrane,
        &mut x,
        &[0.; 4],
        settings().dt,
        &mut correction,
        &|| false,
    )
    .unwrap();
    assert_eq!(x, strained);
    assert_eq!(correction, vec![[0.; 3]; x.len()]);

    let calls = std::cell::Cell::new(0);
    constraints::solve_material(
        &cloth.membrane,
        &mut x,
        &cloth.inverse_masses,
        settings().dt,
        &mut correction,
        &|| {
            calls.set(calls.get() + 1);
            false
        },
    )
    .unwrap();
    assert_ne!(x, strained);
    for i in 0..x.len() {
        assert_eq!(correction[i], sub(x[i], strained[i]));
    }
    // Cancel at every actual callback, including entry and the final pre-copy check.
    for stop_at in 1..=calls.get() {
        let mut candidate = strained.clone();
        let mut p = vec![[0.; 3]; x.len()];
        let polls = std::cell::Cell::new(0);
        let error = constraints::solve_material(
            &cloth.membrane,
            &mut candidate,
            &cloth.inverse_masses,
            settings().dt,
            &mut p,
            &|| {
                polls.set(polls.get() + 1);
                polls.get() == stop_at
            },
        )
        .unwrap_err();
        assert_eq!(error, SimulationError::Cancelled);
        assert_eq!(polls.get(), stop_at);
        assert_eq!(candidate, strained);
        assert_eq!(p, vec![[0.; 3]; x.len()]);
    }
}

#[test]
fn material_cancellation_checks_entry_and_each_noop_return() {
    let cloth = Cloth::new(square()).unwrap();
    let rest = cloth.positions().to_vec();
    let mut strained = rest.clone();
    strained.iter_mut().for_each(|p| p[0] *= 1.5);
    for (before, weights) in [
        (rest, cloth.inverse_masses.as_slice()),
        (strained, &[0.; 4]),
    ] {
        let calls = std::cell::Cell::new(0);
        let mut x = before.clone();
        let mut p = vec![[0.; 3]; x.len()];
        constraints::solve_material(
            &cloth.membrane,
            &mut x,
            weights,
            settings().dt,
            &mut p,
            &|| {
                calls.set(calls.get() + 1);
                false
            },
        )
        .unwrap();
        assert_eq!(calls.get(), 2);
        assert_eq!(x, before);
        assert_eq!(p, vec![[0.; 3]; x.len()]);
        for stop_at in 1..=calls.get() {
            let polls = std::cell::Cell::new(0);
            assert_eq!(
                constraints::solve_material(
                    &cloth.membrane,
                    &mut x,
                    weights,
                    settings().dt,
                    &mut p,
                    &|| {
                        polls.set(polls.get() + 1);
                        polls.get() == stop_at
                    },
                ),
                Err(SimulationError::Cancelled)
            );
            assert_eq!(polls.get(), stop_at);
            assert_eq!(x, before);
            assert_eq!(p, vec![[0.; 3]; x.len()]);
        }
    }
}

#[test]
fn material_cancellation_at_every_cloth_callback_rolls_back_all_persistent_state() {
    let mut input = square();
    input.positions.iter_mut().for_each(|p| p[0] *= 1.5);
    input.pins.push(Pin {
        vertex: 0,
        target: input.positions[0],
    });
    input.stitches.push(Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(1),
        compliance: 0.01,
    });
    input.colliders.push(Collider::Ground { height: 0. });
    let settings = StepSettings {
        iterations: 2,
        gravity: [0., -9.81, 0.],
        ..settings()
    };
    let mut accepted = Cloth::new(input).unwrap();
    accepted.step(settings, || false).unwrap();
    let before = format!("{accepted:?}");
    let mut complete = accepted.clone();
    let calls = std::cell::Cell::new(0);
    complete
        .step(settings, || {
            calls.set(calls.get() + 1);
            false
        })
        .unwrap();
    assert_ne!(complete.positions, accepted.positions);
    assert!(complete.seam_remaining[0] < accepted.seam_remaining[0]);
    for stop_at in 1..=calls.get() {
        let mut candidate = accepted.clone();
        let polls = std::cell::Cell::new(0);
        assert_eq!(
            candidate
                .step(settings, || {
                    polls.set(polls.get() + 1);
                    polls.get() == stop_at
                })
                .unwrap_err(),
            SimulationError::Cancelled
        );
        assert_eq!(polls.get(), stop_at);
        assert_eq!(format!("{candidate:?}"), before);
        assert_eq!(candidate.positions, accepted.positions);
        assert_eq!(candidate.velocities, accepted.velocities);
        assert_eq!(candidate.seam_remaining, accepted.seam_remaining);
        assert_eq!(candidate.simulation_time(), accepted.simulation_time());
    }
}

#[test]
fn grouped_mixed_grain_materials_reach_direct_energy_balance_with_pins() {
    fn objective(x: &[Vec3], predicted: &[Vec3], input: &ClothInput, mass: &[f64], h: f64) -> f64 {
        let inertia = [1, 2]
            .into_iter()
            .map(|i| {
                let d = sub(x[i], predicted[i]);
                0.5 * mass[i] * dot(d, d) / (h * h)
            })
            .sum::<f64>();
        inertia
            + input
                .triangles
                .iter()
                .zip(&input.triangle_materials)
                .map(|(t, m)| {
                    let [a, b, c] = t.map(|i| input.material_uv[i]);
                    let du1 = b[0] - a[0];
                    let du2 = c[0] - a[0];
                    let dv1 = b[1] - a[1];
                    let dv2 = c[1] - a[1];
                    let det = du1 * dv2 - du2 * dv1;
                    let e1 = sub(x[t[1]], x[t[0]]);
                    let e2 = sub(x[t[2]], x[t[0]]);
                    let u = mul(sub(mul(e1, dv2), mul(e2, dv1)), det.recip());
                    let v = mul(sub(mul(e2, du1), mul(e1, du2)), det.recip());
                    let (s, c) = m.grain_angle.sin_cos();
                    let w = add(mul(u, c), mul(v, s));
                    let f = add(mul(u, -s), mul(v, c));
                    let cw = (dot(w, w) - 1.) / 2.;
                    let cf = (dot(f, f) - 1.) / 2.;
                    let cs = dot(w, f);
                    det.abs() / 4.
                        * (m.warp_stiffness * cw * cw
                            + m.weft_stiffness * cf * cf
                            + m.shear_stiffness * cs * cs)
                })
                .sum::<f64>()
    }
    let rest = vec![
        [-0.064, 0., -0.],
        [0., -0.004, 0.],
        [0., 0.004, 0.],
        [0.064, 0., 0.],
    ];
    let input = ClothInput {
        positions: rest.clone(),
        material_uv: rest.iter().map(|v| [v[0], v[1]]).collect(),
        triangles: vec![[0, 1, 2], [1, 3, 2]],
        triangle_materials: vec![
            Material {
                areal_density: 0.18,
                warp_stiffness: 300.,
                weft_stiffness: 700.,
                shear_stiffness: 55.,
                grain_angle: 0.31,
                bending_stiffness: 0.,
                damping: 0.,
                ..Default::default()
            },
            Material {
                areal_density: 0.25,
                warp_stiffness: 950.,
                weft_stiffness: 120.,
                shear_stiffness: 140.,
                grain_angle: -0.47,
                bending_stiffness: 0.,
                damping: 0.,
                ..Default::default()
            },
        ],
        pins: vec![
            Pin {
                vertex: 0,
                target: rest[0],
            },
            Pin {
                vertex: 3,
                target: rest[3],
            },
        ],
        ..Default::default()
    };
    let h = (1. / 60.) / 8.;
    let mut cloth = Cloth::new(input.clone()).unwrap();
    assert_eq!(cloth.membrane.len(), 2);
    cloth.velocities[1] = [0.0001 / h, -0.00015 / h, 0.00003 / h];
    cloth.velocities[2] = [-0.00008 / h, 0.00012 / h, -0.00002 / h];
    let predicted: Vec<_> = rest
        .iter()
        .zip(&cloth.velocities)
        .map(|(&p, &v)| add(p, mul(v, h)))
        .collect();
    let area = 0.064 * 0.008 / 2.;
    let mass = [
        0.18 * area / 3.,
        (0.18 + 0.25) * area / 3.,
        (0.18 + 0.25) * area / 3.,
        0.25 * area / 3.,
    ];
    for (actual, expected) in cloth.masses().iter().zip(mass) {
        assert!((actual - expected).abs() < 1e-18);
    }
    let initial = objective(&predicted, &predicted, &input, &mass, h);
    cloth
        .step(
            StepSettings {
                dt: h,
                substeps: 1,
                iterations: 32,
                gravity: [0.; 3],
                seam_closure_speed: 0.,
            },
            || false,
        )
        .unwrap();
    assert!(objective(cloth.positions(), &predicted, &input, &mass, h) < initial);
    let epsilon = 1e-8;
    let mut maximum = 0_f64;
    for node in [1, 2] {
        let mut gradient = [0.; 3];
        for axis in 0..3 {
            let mut plus = cloth.positions().to_vec();
            let mut minus = plus.clone();
            plus[node][axis] += epsilon;
            minus[node][axis] -= epsilon;
            gradient[axis] = (objective(&plus, &predicted, &input, &mass, h)
                - objective(&minus, &predicted, &input, &mass, h))
                / (2. * epsilon);
        }
        maximum = maximum.max(length(gradient));
    }
    println!("mixed-grain pair direct energy force residual={maximum:e}N");
    assert!(maximum < 1e-6);
    for pin in &input.pins {
        assert_eq!(
            cloth.positions()[pin.vertex].map(f64::to_bits),
            pin.target.map(f64::to_bits)
        );
    }
}

#[test]
#[allow(clippy::approx_constant)] // Preserve the independently frozen material literal.
fn material_network_reaches_independent_incremental_energy_equilibrium() {
    fn objective(
        x: &[Vec3],
        input: &ClothInput,
        predictor: &[Vec3],
        masses: [f64; 2],
        h: f64,
    ) -> f64 {
        let inertia = [5, 6]
            .into_iter()
            .zip(masses)
            .map(|(node, mass)| {
                let delta = sub(x[node], predictor[node]);
                0.5 * mass * dot(delta, delta) / (h * h)
            })
            .sum::<f64>();
        inertia
            + input
                .triangles
                .iter()
                .zip(&input.triangle_materials)
                .map(|(t, m)| {
                    let [a, b, c] = t.map(|i| input.material_uv[i]);
                    let du1 = b[0] - a[0];
                    let du2 = c[0] - a[0];
                    let dv1 = b[1] - a[1];
                    let dv2 = c[1] - a[1];
                    let det = du1 * dv2 - du2 * dv1;
                    let e1 = sub(x[t[1]], x[t[0]]);
                    let e2 = sub(x[t[2]], x[t[0]]);
                    let u = mul(sub(mul(e1, dv2), mul(e2, dv1)), det.recip());
                    let v = mul(sub(mul(e2, du1), mul(e1, du2)), det.recip());
                    let (s, c) = m.grain_angle.sin_cos();
                    let w = add(mul(u, c), mul(v, s));
                    let f = add(mul(u, -s), mul(v, c));
                    let cw = (dot(w, w) - 1.) / 2.;
                    let cf = (dot(f, f) - 1.) / 2.;
                    let cs = dot(w, f);
                    det.abs() / 4.
                        * (m.warp_stiffness * cw * cw
                            + m.weft_stiffness * cf * cf
                            + m.shear_stiffness * cs * cs)
                })
                .sum::<f64>()
    }
    // Complete actual groups around global free nodes500/501 at saved frame28.
    // Global nodes:39,307,308,498,499,500,501,561; faces:214,215,216,221,286,288,414,416.
    let rest = vec![
        [-0.033333333333333326, 1.5, 0.24],
        [-0.0331818181818182, 1.365, 0.24],
        [-0.016818181818181843, 1.365, 0.24],
        [-0.00840909090909081, 1.4325, 0.24],
        [-0.041439393939394, 1.4325, 0.24],
        [-0.03325757575757571, 1.4325, 0.24],
        [-0.02507575757575764, 1.4325, 0.24],
        [-0.024924242424242404, 1.2974999999999999, 0.24],
    ];
    let uv = vec![
        [-0.7533333333333333, 1.5],
        [-0.7531818181818182, 1.365],
        [-0.7368181818181818, 1.365],
        [-0.7284090909090908, 1.4325],
        [-0.761439393939394, 1.4325],
        [-0.7532575757575757, 1.4325],
        [-0.7450757575757576, 1.4325],
        [-0.7449242424242424, 1.2974999999999999],
    ];
    let predictor = vec![
        [
            -0.033498148833116595,
            1.5168706938759002,
            0.14566104866625693,
        ],
        [-0.0334210793269415, 1.3870810608749282, 0.18200059013621286],
        [-0.01683930195603287, 1.387278279312309, 0.18465908192500208],
        [-0.008383278964555426, 1.45621973330344, 0.17850616015501183],
        [
            -0.043347835925128614,
            1.4559258593972446,
            0.17374173905854726,
        ],
        [-0.0350218534159662, 1.4560676877118206, 0.17520016483975107],
        [
            -0.025356882744323836,
            1.455932023624931,
            0.17679416384316252,
        ],
        [-0.02469902892629936, 1.31825428209096, 0.1827706602096836],
    ];
    let triangles = vec![
        [0, 5, 6],
        [2, 6, 1],
        [6, 5, 1],
        [1, 7, 2],
        [0, 4, 5],
        [4, 1, 5],
        [0, 6, 3],
        [3, 6, 2],
    ];
    let material = Material {
        areal_density: 0.18,
        warp_stiffness: 400.0,
        weft_stiffness: 400.0,
        shear_stiffness: 100.0,
        bending_stiffness: 0.003,
        grain_angle: 1.5707963267948966,
        damping: 0.2,
        friction: 0.4,
        collision_thickness: 0.003,
    };
    let input = ClothInput {
        positions: rest,
        material_uv: uv,
        triangles,
        triangle_materials: vec![material; 8],
        pins: (0..8)
            .filter(|v| ![5, 6].contains(v))
            .map(|vertex| Pin {
                vertex,
                target: predictor[vertex],
            })
            .collect(),
        ..Default::default()
    };
    let cloth = Cloth::new(input.clone()).unwrap();
    let expected_mass: [f64; 2] = [6.627272727272724e-05, 0.00013377272727272752];
    for (node, mass) in [5, 6].into_iter().zip(expected_mass) {
        assert_eq!(cloth.masses()[node].to_bits(), mass.to_bits());
    }
    // The global stage retains every original face in order; it has no pair partition.
    assert_eq!(
        cloth
            .membrane
            .iter()
            .map(|m| m.vertices)
            .collect::<Vec<_>>(),
        input.triangles,
    );
    for node in 0..8 {
        assert_eq!(cloth.inverse_masses[node] == 0., ![5, 6].contains(&node));
    }
    let h: f64 = (1. / 60.) / 8.;
    let mut x = predictor.clone();
    let mut corrections = vec![[0.; 3]; x.len()];
    // One isolated material substep: no prediction, damping, bending, sewing or contact stage.
    cloth.apply_pins(&mut x);
    for _ in 0..32 {
        constraints::solve_material(
            &cloth.membrane,
            &mut x,
            &cloth.inverse_masses,
            h,
            &mut corrections,
            &|| false,
        )
        .unwrap();
        cloth.apply_pins(&mut x);
    }
    // Independent100-digit stationary reference, locally positive Hessian; no global uniqueness claim.
    let expected: [Vec3; 2] = [
        [
            -0.03468793525197834,
            1.4559016208026938,
            0.17491839356990843,
        ],
        [
            -0.02602759678104083,
            1.4559576701123809,
            0.17611347479474232,
        ],
    ];
    let position_error = [5, 6]
        .into_iter()
        .zip(expected)
        .map(|(node, target)| length(sub(x[node], target)))
        .fold(0., f64::max);
    let epsilon = 1e-7;
    let mut force_residual = 0_f64;
    for node in [5, 6] {
        let mut gradient = [0.; 3];
        for (axis, entry) in gradient.iter_mut().enumerate() {
            let energy_at = |offset| {
                let mut q = x.clone();
                q[node][axis] += offset;
                objective(&q, &input, &predictor, expected_mass, h)
            };
            *entry = (energy_at(-2. * epsilon) - 8. * energy_at(-epsilon)
                + 8. * energy_at(epsilon)
                - energy_at(2. * epsilon))
                / (12. * epsilon);
        }
        force_residual = force_residual.max(length(gradient));
    }
    println!(
        "material-network q={:?} reference={expected:?} position_error={position_error:e}m force_residual={force_residual:e}N",
        [x[5], x[6]]
    );
    assert!(position_error < 1e-8 && force_residual < 1e-6);
    for pin in &input.pins {
        assert_eq!(
            x[pin.vertex].map(f64::to_bits),
            pin.target.map(f64::to_bits)
        );
    }
}
