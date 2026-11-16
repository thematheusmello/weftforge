use std::{
    cell::Cell,
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use weftforge_core::{
    Command, CurveInterval, Editor, Placement, Project, simulation::create_cloth,
};

fn polygon(vertices: &[[f64; 2]]) -> Editor {
    let mut e = Editor::default();
    e.execute(Command::AddPolygon {
        name: "Original offset fixture".into(),
        vertices: vertices.to_vec(),
    })
    .unwrap();
    let mut material = e.project.materials[0].clone();
    material.areal_density = 0.2;
    e.execute(Command::UpdateMaterial { material }).unwrap();
    e
}
fn rectangle() -> Editor {
    polygon(&[[0., 0.], [0.4, 0.], [0.4, 0.6], [0., 0.6]])
}
fn curved() -> Editor {
    let mut e = rectangle();
    let curve = e.project.pieces[0].boundary[1];
    e.execute(Command::SetCurveControls {
        curve,
        controls: Some([[0.5, 0.2], [0.5, 0.4]]),
    })
    .unwrap();
    e
}
fn curved_l() -> Editor {
    let mut e = polygon(&[
        [0., 0.],
        [0.4, 0.],
        [0.4, 0.2],
        [0.2, 0.2],
        [0.2, 0.4],
        [0., 0.4],
    ]);
    let curve = e.project.pieces[0].boundary[2];
    e.execute(Command::SetCurveControls {
        curve,
        controls: Some([[0.35, 0.18], [0.25, 0.18]]),
    })
    .unwrap();
    e
}
fn close(a: f64, b: f64, t: f64) {
    assert!((a - b).abs() <= t, "{a:.17} != {b:.17}; tolerance {t}");
}
fn point(a: [f64; 2], b: [f64; 2], t: f64) {
    close(a[0], b[0], t);
    close(a[1], b[1], t);
}
fn total_rest(project: &Project) -> (f64, f64) {
    let mesh = project.mesh().unwrap();
    let cloth = create_cloth(project, &mesh).unwrap();
    let area = mesh
        .triangles
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| mesh.uv[i]);
            ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() / 2.
        })
        .sum::<f64>();
    (area, cloth.masses().iter().sum())
}
fn counter(p: &Project) -> u64 {
    let v: serde_json::Value = serde_json::from_slice(&p.to_json().unwrap()).unwrap();
    v["project"]["next_id"].as_u64().unwrap()
}
fn same_authored(a: &Project, b: &Project) {
    assert_eq!(a.points, b.points);
    assert_eq!(a.curves, b.curves);
    assert_eq!(a.pieces, b.pieces);
    assert_eq!(a.materials, b.materials);
    assert_eq!(a.seams, b.seams);
    assert_eq!(a.sewing_groups, b.sewing_groups);
    assert_eq!(a.resolution, b.resolution);
    assert_eq!(a.mannequin, b.mannequin);
}

#[test]
fn rectangle_changes_real_material_and_preserves_identity_arrangement_and_one_history_action() {
    let mut e = rectangle();
    let piece = e.project.pieces[0].id;
    e.execute(Command::SetPlacement {
        piece,
        placement: Placement {
            translation: [0.2, 1.1, -0.4],
            rotation: [0.3, 0.2, -0.1],
        },
    })
    .unwrap();
    e.execute(Command::SetGrain { piece, angle: 0.37 }).unwrap();
    let original = e.project.clone();
    let prepared = e
        .project
        .prepare_outline_offset(piece, 0.01, true, || false)
        .unwrap();
    assert_eq!(e.project, original);
    close(prepared.before().authored_area, 0.24, 1e-12);
    close(prepared.before().rest_area, 0.24, 1e-12);
    close(prepared.before().cloth_mass, 0.048, 2e-10);
    point(prepared.after().bounds[0], [-0.01, -0.01], 1e-9);
    point(prepared.after().bounds[1], [0.41, 0.61], 1e-9);
    close(prepared.after().authored_area, 0.2604, 1e-9);
    close(prepared.after().rest_area, 0.2604, 1e-9);
    close(prepared.after().cloth_mass, 0.05208, 2e-10);
    assert!(prepared.certificate().maximum_error <= prepared.certificate().tolerance);
    let (mesh, cloth) = e.commit_outline_offset(prepared).unwrap();
    assert_eq!(mesh.doc_revision, e.project.revision);
    close(cloth.masses().iter().sum(), 0.05208, 2e-10);
    assert_eq!(e.project.revision, original.revision + 1);
    assert_eq!(counter(&e.project), counter(&original));
    assert_eq!(e.project.pieces[0].boundary, original.pieces[0].boundary);
    assert_eq!(
        e.project.points.iter().map(|p| p.id).collect::<Vec<_>>(),
        original.points.iter().map(|p| p.id).collect::<Vec<_>>()
    );
    let mut expected_piece = original.pieces[0].clone();
    expected_piece.boundary = e.project.pieces[0].boundary.clone();
    assert_eq!(e.project.pieces[0], expected_piece);
    let applied = e.project.clone();
    assert!(e.undo());
    same_authored(&e.project, &original);
    assert_eq!(e.project.revision, applied.revision + 1);
    assert!(e.redo());
    same_authored(&e.project, &applied);
    assert_eq!(e.project.revision, applied.revision + 2);
    let (a, m) = total_rest(&e.project);
    close(a, 0.2604, 1e-9);
    close(m, 0.05208, 2e-10);
}

#[test]
fn straight_concave_trim_bevel_and_reversed_winding_match_independent_coordinates_and_masses() {
    let fixtures = vec![
        (
            vec![
                [0., 0.],
                [0.4, 0.],
                [0.4, 0.2],
                [0.2, 0.2],
                [0.2, 0.4],
                [0., 0.4],
            ],
            vec![
                [-0.01, -0.01],
                [0.41, -0.01],
                [0.41, 0.21],
                [0.21, 0.21],
                [0.21, 0.41],
                [-0.01, 0.41],
            ],
            0.1364,
        ),
        (
            vec![[0., 0.], [0.4, 0.], [0.2, 0.02]],
            vec![
                [-0.000995037190209989, 0.009950371902099893],
                [0., -0.01],
                [0.4, -0.01],
                [0.40099503719021, 0.009950371902099893],
                [0.2, 0.030049875621120888],
            ],
            0.012039900620350458,
        ),
    ];
    for (vertices, expected, area) in fixtures {
        for reverse in [false, true] {
            let mut input = vertices.clone();
            if reverse {
                input.reverse();
            }
            let mut e = polygon(&input);
            let piece = e.project.pieces[0].id;
            let old = e.project.clone();
            e.execute(Command::OffsetOutline {
                piece,
                width: 0.01,
                outward: true,
            })
            .unwrap();
            let actual: Vec<_> = e.project.pieces[0]
                .boundary
                .iter()
                .map(|id| {
                    e.project
                        .point(e.project.curve(*id).unwrap().start)
                        .unwrap()
                        .position
                })
                .collect();
            assert_eq!(actual.len(), expected.len());
            for p in &expected {
                assert!(
                    actual
                        .iter()
                        .any(|a| (a[0] - p[0]).hypot(a[1] - p[1]) < 1e-9),
                    "missing {p:?}: {actual:?}"
                );
            }
            let (a, m) = total_rest(&e.project);
            close(a, area, 1e-9);
            close(m, area * 0.2, 2e-10);
            for id in &old.pieces[0].boundary {
                assert!(e.project.curve(*id).is_some());
            }
            if vertices.len() == 3 {
                for p in &old.points {
                    if p.position[1] == 0. {
                        assert!(e.project.point(p.id).is_none());
                    } else {
                        assert!(e.project.point(p.id).is_some());
                    }
                }
                assert_eq!(
                    e.project
                        .points
                        .iter()
                        .filter(|p| old.point(p.id).is_none())
                        .count(),
                    4
                );
            }
        }
    }
}

fn normal_cubic(t: f64, l: bool) -> [f64; 2] {
    let (x, y, dx, dy) = if l {
        (
            0.4 - 0.15 * t - 0.15 * t * t + 0.1 * t * t * t,
            0.2 - 0.06 * t + 0.06 * t * t,
            -0.15 - 0.3 * t + 0.3 * t * t,
            -0.06 + 0.12 * t,
        )
    } else {
        (0.4 + 0.3 * t * (1. - t), 0.6 * t, 0.3 - 0.6 * t, 0.6)
    };
    let speed = dx.hypot(dy);
    [x + 0.01 * dy / speed, y - 0.01 * dx / speed]
}

#[test]
fn real_cubic_fit_and_curved_concave_trim_have_between_sample_certificates_actual_mesh_and_mass() {
    for l in [false, true] {
        let mut e = if l { curved_l() } else { curved() };
        let piece = e.project.pieces[0].id;
        let original = e.project.clone();
        let source = original.pieces[0].boundary[if l { 2 } else { 1 }];
        let prepared = e
            .project
            .prepare_outline_offset(piece, 0.01, true, || false)
            .unwrap();
        let target = if l {
            0.13437458628915014
        } else {
            0.2906113444752574
        };
        close(
            prepared.before().authored_area,
            if l { 0.1179 } else { 0.27 },
            1e-12,
        );
        close(prepared.after().authored_area, target, 5e-5);
        close(prepared.after().rest_area, target, 5e-5);
        close(prepared.after().cloth_mass, 0.2 * target, 1e-5);
        close(
            prepared.after().cloth_mass,
            0.2 * prepared.after().rest_area,
            2e-10,
        );
        let cert = prepared.certificate();
        assert!(cert.maximum_error <= cert.tolerance && cert.tolerance <= 1e-5);
        assert!(cert.geometry_work <= 16_777_216);
        let spans: Vec<_> = cert
            .spans
            .iter()
            .filter(|s| s.source_curve == source)
            .collect();
        assert!(!spans.is_empty());
        assert_eq!(spans[0].output_curve, source);
        for (i, span) in spans.iter().enumerate() {
            assert!(span.source_interval[1] > span.source_interval[0]);
            assert!(span.error_bound <= cert.tolerance);
            if i > 0 {
                close(spans[i - 1].source_interval[1], span.source_interval[0], 0.);
            }
            let c = prepared.project().curve(span.output_curve).unwrap();
            assert!(c.controls.is_some());
            // Independent analytic witnesses supplement, and do not replace, the continuous certificate.
            for j in 0..=64 {
                let f = j as f64 / 64.;
                let t = span.source_interval[0]
                    + f * (span.source_interval[1] - span.source_interval[0]);
                point(
                    c.evaluate(prepared.project(), f).unwrap(),
                    normal_cubic(t, l),
                    1e-5,
                );
            }
        }
        if l {
            let last = spans.last().unwrap();
            close(last.source_interval[1], 0.9564693930114259, 2e-5);
            let c = prepared.project().curve(last.output_curve).unwrap();
            point(
                c.evaluate(prepared.project(), 1.).unwrap(),
                [0.21, 0.20697791651287822],
                2e-5,
            );
        }
        println!(
            "{} authored={} actual_rest={} real_Cloth_mass={} max_bound={} work={} fitted_cubics={}",
            if l { "curved-L" } else { "curved-side" },
            prepared.after().authored_area,
            prepared.after().rest_area,
            prepared.after().cloth_mass,
            cert.maximum_error,
            cert.geometry_work,
            spans.len()
        );
        e.commit_outline_offset(prepared).unwrap();
        let applied = e.project.clone();
        let reopened = Project::from_json(&e.project.to_json().unwrap()).unwrap();
        assert_eq!(reopened, applied);
        let (a, m) = total_rest(&reopened);
        close(a, target, 5e-5);
        close(m, 0.2 * target, 1e-5);
        let curve = e.project.curve(source).unwrap();
        let mut controls = curve.controls.unwrap();
        controls[0][0] += 0.001;
        e.execute(Command::SetCurveControls {
            curve: source,
            controls: Some(controls),
        })
        .unwrap();
        assert!(e.undo());
        same_authored(&e.project, &applied);
        assert!(e.undo());
        same_authored(&e.project, &original);
        assert!(e.redo());
        same_authored(&e.project, &applied);
    }
}

#[test]
fn split_curve_has_no_artificial_corner_and_keeps_first_descendant_ids() {
    let mut e = curved();
    let source = e.project.pieces[0].boundary[1];
    e.execute(Command::SplitCurve {
        curve: source,
        t: 0.5,
    })
    .unwrap();
    let original = e.project.clone();
    let piece = original.pieces[0].id;
    let prepared = e
        .project
        .prepare_outline_offset(piece, 0.01, true, || false)
        .unwrap();
    close(prepared.after().authored_area, 0.2906113444752574, 5e-5);
    for id in &original.pieces[0].boundary {
        let s = prepared
            .certificate()
            .spans
            .iter()
            .find(|s| s.source_curve == *id)
            .unwrap();
        assert_eq!(s.output_curve, *id);
    }
    let split_point = original.curve(source).unwrap().end;
    point(
        prepared.project().point(split_point).unwrap().position,
        [0.485, 0.3],
        1e-5,
    );
    let generated = prepared
        .project()
        .points
        .iter()
        .filter(|p| original.point(p.id).is_none())
        .map(|p| p.id)
        .collect::<Vec<_>>();
    assert!(!generated.is_empty());
    let high = counter(prepared.project());
    e.commit_outline_offset(prepared).unwrap();
    assert!(e.undo());
    assert!(counter(&e.project) >= high);
    e.execute(Command::AddRectangle {
        name: "New branch".into(),
        origin: [1., 0.],
        width: 0.1,
        height: 0.1,
    })
    .unwrap();
    assert!(!e.can_redo());
    assert!(
        e.project
            .points
            .iter()
            .filter(|p| original.point(p.id).is_none())
            .all(|p| p.id >= high)
    );
}

#[test]
fn inward_curved_trim_and_reversed_authored_winding_preserve_material_relative_side() {
    let mut reversed = polygon(&[[0., 0.6], [0.4, 0.6], [0.4, 0.], [0., 0.]]);
    let curve = reversed.project.pieces[0].boundary[1];
    reversed
        .execute(Command::SetCurveControls {
            curve,
            controls: Some([[0.5, 0.4], [0.5, 0.2]]),
        })
        .unwrap();
    for e in [curved(), reversed] {
        let piece = e.project.pieces[0].id;
        let prepared = e
            .project
            .prepare_outline_offset(piece, 0.01, true, || false)
            .unwrap();
        close(prepared.after().authored_area, 0.2906113444752574, 5e-5);
        point(prepared.after().bounds[0], [-0.01, -0.01], 2e-5);
        point(prepared.after().bounds[1], [0.485, 0.61], 2e-5);
        let prepared = e
            .project
            .prepare_outline_offset(piece, 0.01, false, || false)
            .unwrap();
        assert!(prepared.after().authored_area < 0.27);
        point(prepared.after().bounds[0], [0.01, 0.01], 2e-5);
        point(prepared.after().bounds[1], [0.465, 0.59], 2e-5);
        close(
            prepared.after().cloth_mass,
            prepared.after().rest_area * 0.2,
            2e-10,
        );
    }
}

#[test]
fn adjacent_cubic_pair_trim_certifies_the_root_and_continuation_across_widths() {
    let mut e = curved_l();
    let piece = e.project.pieces[0].id;
    let outgoing = e.project.pieces[0].boundary[3];
    e.execute(Command::SetCurveControls {
        curve: outgoing,
        controls: Some([[0.2, 0.26666666666666666], [0.2, 0.3333333333333333]]),
    })
    .unwrap();
    let prepared = e
        .project
        .prepare_outline_offset(piece, 0.01, true, || false)
        .unwrap();
    close(prepared.after().authored_area, 0.13437458628915014, 5e-5);
    close(prepared.after().rest_area, 0.13437458628915014, 5e-5);
    close(prepared.after().cloth_mass, 0.02687491725783003, 1e-5);
    let span = prepared
        .certificate()
        .spans
        .iter()
        .find(|s| s.source_curve == outgoing)
        .unwrap();
    assert!(span.source_interval[0] > 0. && span.source_interval[0] < 1.);
    assert_eq!(span.output_curve, outgoing);
    assert!(
        prepared
            .project()
            .curve(outgoing)
            .unwrap()
            .controls
            .is_some()
    );
}

#[test]
fn two_genuinely_curved_loci_trim_at_an_independently_analytic_symmetric_root() {
    // C_in(h)=(h,-k*h²), h decreases to zero; C_out(h)=(-k*h²,h), h increases.
    let mut e = polygon(&[
        [0., 0.],
        [0.4, 0.],
        [0.4, 0.196],
        [0.2, 0.2],
        [0.196, 0.4],
        [0., 0.4],
    ]);
    let piece = e.project.pieces[0].id;
    let incoming = e.project.pieces[0].boundary[2];
    let outgoing = e.project.pieces[0].boundary[3];
    e.execute(Command::Transaction(vec![
        Command::SetCurveControls {
            curve: incoming,
            controls: Some([
                [0.3333333333333333, 0.19866666666666666],
                [0.26666666666666666, 0.2],
            ]),
        },
        Command::SetCurveControls {
            curve: outgoing,
            controls: Some([
                [0.2, 0.26666666666666666],
                [0.19866666666666666, 0.3333333333333333],
            ]),
        },
    ]))
    .unwrap();
    let h: f64 = 0.01;
    let k: f64 = 0.1;
    let speed = (1. + 4. * k * k * h * h).sqrt();
    let width = (h + k * h * h) * speed / (1. - 2. * k * h);
    let expected = 0.2 + h + width * 2. * k * h / speed;
    let prepared = e
        .project
        .prepare_outline_offset(piece, width, true, || false)
        .unwrap();
    close(
        prepared.before().authored_area,
        0.12 - 2. * 0.1 * 0.2_f64.powi(3) / 3.,
        1e-12,
    );
    let join = prepared
        .certificate()
        .joins
        .iter()
        .find(|j| j.incoming_curve == incoming && j.outgoing_curve == outgoing)
        .unwrap();
    let parameters = join.trim_parameters.unwrap();
    assert!(parameters[0][0] <= 0.95 && parameters[0][1] >= 0.95);
    assert!(parameters[1][0] <= 0.05 && parameters[1][1] >= 0.05);
    let root = prepared
        .project()
        .point(join.source_vertex)
        .unwrap()
        .position;
    point(root, [expected, expected], 1e-5);
    close(
        prepared.after().cloth_mass,
        0.2 * prepared.after().rest_area,
        2e-10,
    );
}

#[test]
fn final_transaction_mesh_preflight_catches_later_resolution_and_nested_batches_atomically() {
    let mut e = Editor::default();
    e.execute(Command::Transaction(vec![
        Command::SetResolution { metres: 0.5 },
        Command::AddRectangle {
            name: "Coarse-valid panel".into(),
            origin: [0., 0.],
            width: 2.,
            height: 2.,
        },
    ]))
    .unwrap();
    let piece = e.project.pieces[0].id;
    let old = e.project.clone();
    let old_mesh = old.mesh().unwrap();
    let old_cloth = create_cloth(&old, &old_mesh).unwrap();
    let error = e
        .execute(Command::Transaction(vec![
            Command::OffsetOutline {
                piece,
                width: 0.01,
                outward: true,
            },
            Command::SetResolution { metres: 0.005 },
        ]))
        .unwrap_err();
    assert!(error.to_string().contains("limit"), "{error}");
    assert_eq!(e.project, old);
    assert_eq!(old_cloth.positions(), old_mesh.positions);
    e.execute(Command::Transaction(vec![Command::Transaction(vec![
        Command::OffsetOutline {
            piece,
            width: 0.01,
            outward: true,
        },
    ])]))
    .unwrap_err();
    assert_eq!(e.project, old);
    e.execute(Command::Transaction(vec![
        Command::OffsetOutline {
            piece,
            width: 0.01,
            outward: true,
        },
        Command::SetResolution { metres: 0.25 },
    ]))
    .unwrap();
    assert_eq!(e.project.revision, old.revision + 1);
    close(total_rest(&e.project).0, 2.02 * 2.02, 1e-9);
}

#[test]
fn request_and_global_object_caps_are_independent_and_refuse_without_truncation() {
    let vertices: Vec<_> = (0..4096)
        .map(|i| {
            let t = std::f64::consts::TAU * i as f64 / 4096.;
            [0.5 * t.cos(), 0.5 * t.sin()]
        })
        .collect();
    let mut e = polygon(&vertices);
    let piece = e.project.pieces[0].id;
    let polls = Cell::new(0);
    let old = e.project.clone();
    let error = e
        .project
        .prepare_outline_offset(piece, 0.01, true, || {
            polls.set(polls.get() + 1);
            polls.get() == 4
        })
        .unwrap_err();
    assert!(error.to_string().contains("OffsetCancelled"), "{error}");
    assert_eq!(e.project, old);
    e.execute(Command::SplitCurve {
        curve: e.project.pieces[0].boundary[0],
        t: 0.5,
    })
    .unwrap();
    let old = e.project.clone();
    let error = e
        .project
        .prepare_outline_offset(piece, 0.01, true, || false)
        .unwrap_err();
    assert!(error.to_string().contains("OffsetInputBudgetExceeded"));
    assert_eq!(e.project, old);
    let e = curved();
    let mut p = e.project.clone();
    let mut next = counter(&p);
    while p.materials.len() + p.points.len() + p.curves.len() + p.pieces.len()
        < weftforge_core::MAX_OBJECTS
    {
        let mut material = p.materials[0].clone();
        material.id = next;
        next += 1;
        p.materials.push(material);
    }
    let mut value = serde_json::to_value(p).unwrap();
    value["next_id"] = serde_json::json!(next);
    let p: Project = serde_json::from_value(value).unwrap();
    p.validate().unwrap();
    let old = p.clone();
    let error = p
        .prepare_outline_offset(p.pieces[0].id, 0.01, true, || false)
        .unwrap_err();
    assert!(
        error.to_string().contains("OffsetOutputBudgetExceeded"),
        "{error}"
    );
    assert_eq!(p, old);
}

#[test]
fn invalid_collapsed_past_reappearance_and_nonadjacent_contacts_refuse_atomically() {
    let mut e = rectangle();
    e.execute(Command::SetResolution { metres: 0.06 }).unwrap();
    assert!(e.undo());
    let before = e.project.clone();
    let history = (e.can_undo(), e.can_redo());
    for (width, outward) in [
        (0., true),
        (-0.01, true),
        (f64::NAN, true),
        (f64::INFINITY, true),
        (0.2, false),
        (0.21, false),
        (0.31, false),
    ] {
        let error = e
            .execute(Command::OffsetOutline {
                piece: before.pieces[0].id,
                width,
                outward,
            })
            .unwrap_err();
        assert!(error.to_string().starts_with("Offset"), "{error}");
        assert_eq!(e.project, before);
        assert_eq!((e.can_undo(), e.can_redo()), history);
    }
    e.execute(Command::OffsetOutline {
        piece: 99999,
        width: 0.01,
        outward: true,
    })
    .unwrap_err();
    assert_eq!(e.project, before);
    let mut u = polygon(&[
        [0., 0.],
        [0.4, 0.],
        [0.4, 0.4],
        [0.3, 0.4],
        [0.3, 0.1],
        [0.1, 0.1],
        [0.1, 0.4],
        [0., 0.4],
    ]);
    let old = u.project.clone();
    let error = u
        .execute(Command::OffsetOutline {
            piece: old.pieces[0].id,
            width: 0.11,
            outward: true,
        })
        .unwrap_err();
    assert!(error.to_string().contains("Offset"));
    assert_eq!(u.project, old);
}

#[test]
fn relationship_blockers_preserve_holes_internal_lines_seams_groups_and_unrelated_construction() {
    for kind in 0..4 {
        let mut e = rectangle();
        let piece = e.project.pieces[0].id;
        match kind {
            0 => e
                .execute(Command::AddHole {
                    piece,
                    vertices: vec![[0.1, 0.1], [0.1, 0.2], [0.2, 0.2], [0.2, 0.1]],
                })
                .unwrap(),
            1 => e
                .execute(Command::AddInternalSegment {
                    piece,
                    start: [0.1, 0.1],
                    end: [0.2, 0.2],
                })
                .unwrap(),
            _ => {
                e.execute(Command::AddRectangle {
                    name: "Unrelated".into(),
                    origin: [1., 0.],
                    width: 0.4,
                    height: 0.6,
                })
                .unwrap();
                let a = CurveInterval::full(e.project.pieces[0].boundary[1]);
                let b = CurveInterval::full(e.project.pieces[1].boundary[3]);
                if kind == 2 {
                    e.execute(Command::AddSeam {
                        name: "Original seam".into(),
                        a,
                        b,
                        correspondence: vec![[0., 0.], [1., 1.]],
                    })
                    .unwrap();
                } else {
                    e.execute(Command::AddSewingGroup {
                        name: "Original group".into(),
                        a: vec![a],
                        b: vec![b],
                        correspondence: vec![[0., 0.], [1., 1.]],
                    })
                    .unwrap();
                }
            }
        }
        let old = e.project.clone();
        let error = e
            .execute(Command::OffsetOutline {
                piece,
                width: 0.01,
                outward: true,
            })
            .unwrap_err();
        assert!(error.to_string().contains("OffsetUnsupported"), "{error}");
        assert_eq!(e.project, old);
    }
    let mut e = rectangle();
    e.execute(Command::AddRectangle {
        name: "Unrelated".into(),
        origin: [1., 0.],
        width: 0.4,
        height: 0.6,
    })
    .unwrap();
    let old = e.project.clone();
    let piece = old.pieces[0].id;
    e.execute(Command::OffsetOutline {
        piece,
        width: 0.01,
        outward: true,
    })
    .unwrap();
    assert_eq!(e.project.pieces[1], old.pieces[1]);
    for id in &old.pieces[1].boundary {
        assert_eq!(e.project.curve(*id), old.curve(*id));
        for p in [old.curve(*id).unwrap().start, old.curve(*id).unwrap().end] {
            assert_eq!(e.project.point(p), old.point(p));
        }
    }
}

#[test]
fn cancellation_stale_revision_same_revision_tampering_and_mesh_failure_preserve_old_state() {
    let mut e = curved();
    let piece = e.project.pieces[0].id;
    let old = e.project.clone();
    let old_mesh = old.mesh().unwrap();
    let old_cloth = create_cloth(&old, &old_mesh).unwrap();
    let history = (e.can_undo(), e.can_redo());
    let error = e
        .project
        .prepare_outline_offset(piece, 0.01, true, || true)
        .unwrap_err();
    assert!(error.to_string().contains("OffsetCancelled"));
    let polls = Cell::new(0);
    let error = e
        .project
        .prepare_outline_offset(piece, 0.01, true, || {
            polls.set(polls.get() + 1);
            polls.get() == 4
        })
        .unwrap_err();
    assert!(error.to_string().contains("cancelled") || error.to_string().contains("Cancelled"));
    assert_eq!(polls.get(), 4);
    assert_eq!(e.project, old);
    assert_eq!((e.can_undo(), e.can_redo()), history);
    let prepared = e
        .project
        .prepare_outline_offset(piece, 0.01, true, || false)
        .unwrap();
    e.execute(Command::SetResolution { metres: 0.06 }).unwrap();
    let changed = e.project.clone();
    let error = e.commit_outline_offset(prepared).unwrap_err();
    assert!(error.to_string().contains("Stale"));
    assert_eq!(e.project, changed);
    let prepared = e
        .project
        .prepare_outline_offset(piece, 0.01, true, || false)
        .unwrap();
    e.project.pieces[0].name.push('!');
    let changed = e.project.clone();
    e.commit_outline_offset(prepared).unwrap_err();
    assert_eq!(e.project, changed);
    assert_eq!(old_cloth.positions(), old_mesh.positions);
    close(
        old_cloth.masses().iter().sum(),
        0.2 * total_rest(&old).0,
        2e-10,
    );
    let e = rectangle();
    let old = e.project.clone();
    let error = e
        .project
        .prepare_outline_offset(old.pieces[0].id, 100., true, || false)
        .unwrap_err();
    assert!(error.to_string().contains("limit"), "{error}");
    assert_eq!(e.project, old);
}

#[test]
fn native_v5_and_physical_svg_cubic_dxf_and_real_rest_obj_follow_applied_construction() {
    let directory = std::env::temp_dir().join(format!(
        "weftforge-outline-offset-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    for curved_fixture in [false, true] {
        let mut e = if curved_fixture {
            curved()
        } else {
            rectangle()
        };
        let piece = e.project.pieces[0].id;
        e.execute(Command::OffsetOutline {
            piece,
            width: 0.01,
            outward: true,
        })
        .unwrap();
        let native = directory.join("fixture.weftforge");
        e.project.save(&native).unwrap();
        let reopened = Project::load(&native).unwrap();
        assert_eq!(e.project, reopened);
        assert_eq!(weftforge_core::FORMAT_VERSION, 5);
        let svg = directory.join("fixture.svg");
        let dxf = directory.join("fixture.dxf");
        let obj = directory.join("fixture.obj");
        reopened.export_svg(&svg).unwrap();
        reopened.export_dxf(&dxf).unwrap();
        let mesh = reopened.mesh().unwrap();
        mesh.export_obj(&obj).unwrap();
        let svg = fs::read_to_string(svg).unwrap();
        let dxf = fs::read_to_string(dxf).unwrap();
        let obj_text = fs::read_to_string(&obj).unwrap();
        assert!(obj_text.contains("metres; Y up"));
        if curved_fixture {
            assert!(svg.contains("C "));
            assert!(dxf.contains("SPLINE"));
        } else {
            assert!(
                svg.contains("width=\"420") && svg.contains("height=\"620"),
                "{svg}"
            );
            let (models, _) = tobj::load_obj(
                &obj,
                &tobj::LoadOptions {
                    single_index: true,
                    ..Default::default()
                },
            )
            .unwrap();
            let positions: &Vec<f32> = &models[0].mesh.positions;
            let minx = positions
                .chunks_exact(3)
                .map(|p| p[0])
                .fold(f32::INFINITY, f32::min);
            let maxx = positions
                .chunks_exact(3)
                .map(|p| p[0])
                .fold(f32::NEG_INFINITY, f32::max);
            close(f64::from(maxx - minx), 0.42, 1e-7);
        }
        assert!(dxf.contains("$INSUNITS\n70\n4\n"));
    }
    fs::remove_dir_all(directory).unwrap();
}
