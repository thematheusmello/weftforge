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
    constraints::solve_stitch(stitch, 0., &mut x, &[1.; 4], 0.01, &mut 0.);
    assert!(length(sub(stitch.a.position(&x), stitch.b.position(&x))) < 1e-12);
    assert!(length(sub(x.iter().copied().fold([0.; 3], add), before)) < 1e-12);
    // A repeated endpoint must be treated as one degree of freedom.
    let s = Stitch {
        a: WeightedPoint::vertex(0),
        b: WeightedPoint::vertex(2),
        compliance: 0.,
    };
    constraints::solve_stitch(s, 0., &mut x, &[1.; 4], 0.01, &mut 0.);
    assert!(length(sub(x[0], x[2])) < 1e-12);
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
fn cancellation_and_failures_preserve_old_state_and_allow_recovery() {
    let mut cloth = Cloth::new(square()).unwrap();
    let before = cloth.positions().to_vec();
    let counter = std::cell::Cell::new(0);
    let error = cloth
        .step(settings(), || {
            counter.set(counter.get() + 1);
            counter.get() > 4
        })
        .unwrap_err();
    assert_eq!(error, SimulationError::Cancelled);
    assert_eq!(cloth.positions(), before);
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
    cloth.step(settings(), || false).unwrap();
    assert!(cloth.diagnostics().finite);
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
fn bending_reduces_fold_angle_without_translating_center_of_mass() {
    let rest = vec![[0., 0., 0.], [1., 0., 0.], [0.5, 1., 0.], [0.5, -1., 0.]];
    let bend = constraints::Bend::new([0, 1, 2, 3], &rest, 0.1);
    let mut x = rest;
    x[2][2] = 0.4;
    let before_angle = constraints::dihedral([0, 1, 2, 3], &x).unwrap().0.abs();
    let before_center = x.iter().copied().fold([0.; 3], add);
    let mut lambda = 0.;
    for _ in 0..8 {
        bend.solve(&mut x, &[1.; 4], 0.1, &mut lambda);
    }
    let after_angle = constraints::dihedral([0, 1, 2, 3], &x).unwrap().0.abs();
    assert!(after_angle < before_angle);
    assert!(length(sub(x.iter().copied().fold([0.; 3], add), before_center)) < 1e-12);
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
