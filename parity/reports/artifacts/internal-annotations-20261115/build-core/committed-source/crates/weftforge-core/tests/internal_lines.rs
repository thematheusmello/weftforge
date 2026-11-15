use serde_json::json;
use weftforge_core::{
    Command, Curve, CurveInterval, Editor, Placement, Point, Project, SewingDirection,
    SewingSideByLength, simulation::create_cloth,
};

fn square() -> Editor {
    polygon(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]])
}

fn polygon(vertices: Vec<[f64; 2]>) -> Editor {
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Original contour".into(),
            vertices,
        })
        .unwrap();
    editor
}

fn add(start: [f64; 2], end: [f64; 2]) -> Command {
    Command::AddInternalSegment {
        piece: 2,
        start,
        end,
    }
}

fn baseline() -> Editor {
    let mut editor = square();
    editor.execute(add([0.125, 0.25], [0.875, 0.75])).unwrap();
    editor
}

fn next_id(project: &Project) -> u64 {
    serde_json::to_value(project).unwrap()["next_id"]
        .as_u64()
        .unwrap()
}

fn endpoints(project: &Project, curve: u64) -> [[f64; 2]; 2] {
    let curve = project.curve(curve).unwrap();
    [curve.start, curve.end].map(|id| project.point(id).unwrap().position)
}

fn rejected(editor: &mut Editor, command: Command) -> String {
    let before = editor.project.clone();
    let undo = editor.can_undo();
    let redo = editor.can_redo();
    let error = editor.execute(command).unwrap_err().to_string();
    assert_eq!(editor.project, before);
    assert_eq!(editor.can_undo(), undo);
    assert_eq!(editor.can_redo(), redo);
    error
}

#[test]
fn original_r_ids_analytic_length_history_and_endpoint_transaction() {
    let mut editor = baseline();
    assert_eq!(editor.project.pieces[0].boundary, [7, 8, 9, 10]);
    assert_eq!(editor.project.pieces[0].internal_curves, [13]);
    assert_eq!(editor.project.curve(13).unwrap().start, 11);
    assert_eq!(editor.project.curve(13).unwrap().end, 12);
    assert_eq!(next_id(&editor.project), 14);
    assert_eq!(editor.project.revision, 2);
    assert!(
        (editor
            .project
            .interval_length(CurveInterval::full(13))
            .unwrap()
            - 13_f64.sqrt() / 4.)
            .abs()
            < 1e-12
    );
    // The first intermediate state is degenerate; the final two-endpoint edit is valid.
    rejected(
        &mut editor,
        Command::MovePoint {
            point: 11,
            position: [0.875, 0.75],
        },
    );
    editor
        .execute(Command::Transaction(vec![
            Command::MovePoint {
                point: 11,
                position: [0.875, 0.75],
            },
            Command::MovePoint {
                point: 12,
                position: [0.125, 0.25],
            },
        ]))
        .unwrap();
    assert_eq!(editor.project.revision, 3);
    assert_eq!(
        endpoints(&editor.project, 13),
        [[0.875, 0.75], [0.125, 0.25]]
    );
    assert!(editor.undo());
    assert_eq!(
        endpoints(&editor.project, 13),
        [[0.125, 0.25], [0.875, 0.75]]
    );
    assert_eq!(editor.project.revision, 4);
    rejected(
        &mut editor,
        Command::Transaction(vec![
            add([0.2, 0.2], [0.7, 0.3]),
            Command::MovePoint {
                point: 12,
                position: [2., 2.],
            },
        ]),
    );
    assert!(editor.can_redo());
    assert!(editor.redo());
    editor
        .execute(Command::RemoveInternalCurve { curve: 13 })
        .unwrap();
    assert_eq!(editor.project.points.len(), 4);
    assert_eq!(editor.project.curves.len(), 4);
    assert_eq!(next_id(&editor.project), 14);
    assert!(editor.undo());
    assert_eq!(editor.project.pieces[0].internal_curves, [13]);
    assert!(editor.redo());
    editor.execute(add([0.2, 0.2], [0.7, 0.3])).unwrap();
    assert_eq!(editor.project.pieces[0].internal_curves, [16]);
    assert!(editor.undo());
    editor.execute(add([0.3, 0.2], [0.6, 0.3])).unwrap();
    assert_eq!(editor.project.pieces[0].internal_curves, [19]);
    assert!(!editor.can_redo());
}

#[test]
fn original_l_c_and_clearance_are_whole_segment_and_resolution_independent() {
    for resolution in [0.005, 0.07, 0.5] {
        let mut l = polygon(vec![
            [0., 0.],
            [1., 0.],
            [1., 0.4],
            [0.4, 0.4],
            [0.4, 1.],
            [0., 1.],
        ]);
        l.execute(Command::SetResolution { metres: resolution })
            .unwrap();
        assert!(rejected(&mut l, add([0.2, 0.8], [0.8, 0.2])).contains("Internal curve"));
        l.execute(add([0.2, 0.8], [0.2, 0.2])).unwrap();
        let mut c = square();
        c.execute(Command::SetCurveControls {
            curve: 7,
            controls: Some([[1. / 3., 2. / 3.], [2. / 3., 2. / 3.]]),
        })
        .unwrap();
        c.execute(Command::SetResolution { metres: resolution })
            .unwrap();
        // q(t)=(t, 2t(1-t)); q(1/2).y=1/2, while q(0.1).y=q(0.9).y=0.18.
        rejected(&mut c, add([0.1, 0.2], [0.9, 0.2]));
        c.execute(add([0.1, 0.75], [0.9, 0.75])).unwrap();
        let mut r = square();
        r.execute(Command::SetResolution { metres: resolution })
            .unwrap();
        for (a, b) in [
            ([0., 0.25], [0.75, 0.5]),
            ([0.25, 0.], [0.75, 0.]),
            ([0.25, 0.5e-6], [0.75, 0.5e-6]),
            ([0.25, 1.05e-6], [0.75, 1.05e-6]),
            ([0.25, 1.1e-6], [0.75, 1.1e-6]),
            ([-0.2, 0.25], [-0.1, 0.75]),
        ] {
            rejected(&mut r, add(a, b));
        }
        r.execute(add([0.25, 2e-6], [0.75, 2e-6])).unwrap();
    }
}

#[test]
fn crossing_and_coincident_annotations_stay_independent_and_exclusive() {
    let mut editor = baseline();
    editor.execute(add([0.125, 0.75], [0.875, 0.25])).unwrap();
    editor.execute(add([0.125, 0.25], [0.5, 0.5])).unwrap();
    assert_eq!(editor.project.pieces[0].internal_curves, [13, 16, 19]);
    assert_eq!(
        endpoints(&editor.project, 19)[0],
        endpoints(&editor.project, 13)[0]
    );
    assert_ne!(
        editor.project.curve(19).unwrap().start,
        editor.project.curve(13).unwrap().start
    );
    let others = [16, 19].map(|id| endpoints(&editor.project, id));
    editor
        .execute(Command::MovePoint {
            point: 11,
            position: [0.2, 0.3],
        })
        .unwrap();
    editor
        .execute(Command::RemoveInternalCurve { curve: 13 })
        .unwrap();
    assert_eq!([16, 19].map(|id| endpoints(&editor.project, id)), others);
    assert_eq!(editor.project.pieces[0].internal_curves, [16, 19]);
    for (a, b) in [([0.2, 0.2], [0.2, 0.2]), ([0.2, 0.2], [0.2 + 0.5e-9, 0.2])] {
        rejected(&mut editor, add(a, b));
    }
    let original = editor.project;
    for mutation in 0..10 {
        let mut bad = original.clone();
        match mutation {
            0 => bad.pieces[0].internal_curves.push(16),
            1 => bad.pieces[0].internal_curves.push(7),
            2 => {
                bad.curves
                    .iter_mut()
                    .find(|curve| curve.id == 16)
                    .unwrap()
                    .start = 3
            }
            3 => {
                bad.curves
                    .iter_mut()
                    .find(|curve| curve.id == 19)
                    .unwrap()
                    .start = bad.curve(16).unwrap().start
            }
            4 => {
                bad.curves
                    .iter_mut()
                    .find(|curve| curve.id == 16)
                    .unwrap()
                    .controls = Some([[0.2, 0.2], [0.4, 0.4]])
            }
            5 => bad.pieces[0].internal_curves.clear(),
            6 => {
                bad.curves
                    .iter_mut()
                    .find(|curve| curve.id == 16)
                    .unwrap()
                    .end = bad.curve(16).unwrap().start
            }
            7 => bad.pieces[0].internal_curves.push(u64::MAX),
            8 => bad.points.retain(|point| point.id != 14),
            _ => bad.points[0].id = 0,
        }
        assert!(bad.validate().is_err(), "mutation {mutation}");
    }
    let mut cross_piece = Editor::new(original).unwrap();
    cross_piece
        .execute(Command::AddRectangle {
            name: "Other".into(),
            origin: [2., 0.],
            width: 1.,
            height: 1.,
        })
        .unwrap();
    cross_piece.project.pieces[1].internal_curves.push(16);
    assert!(cross_piece.project.validate().is_err());
}

#[test]
fn resize_rigid_transform_boundary_edits_and_owner_removal_keep_both_endpoints() {
    let mut editor = baseline();
    editor
        .execute(Command::ResizePiece {
            piece: 2,
            width: 2.,
            height: 0.5,
        })
        .unwrap();
    assert_eq!(
        endpoints(&editor.project, 13).map(|p| p.map(f64::to_bits)),
        [[0.25_f64, 0.125], [1.75, 0.375]].map(|p| p.map(f64::to_bits))
    );
    let placement = Placement {
        translation: [0.31, -0.47, 0.21],
        rotation: [0.37, -0.42, 0.61],
    };
    editor
        .execute(Command::SetPlacement {
            piece: 2,
            placement,
        })
        .unwrap();
    let before = endpoints(&editor.project, 13);
    let world = before.map(|p| placement.transform(p));
    let (translation, angle, pivot) = ([0.19, -0.13], 0.43_f64, [0.4, 0.2]);
    editor
        .execute(Command::TransformPiece {
            piece: 2,
            translation,
            angle,
            pivot,
        })
        .unwrap();
    for ((actual, expected_world), old) in endpoints(&editor.project, 13)
        .into_iter()
        .zip(world)
        .zip(before)
    {
        let expected = [
            pivot[0] + angle.cos() * (old[0] - pivot[0]) - angle.sin() * (old[1] - pivot[1])
                + translation[0],
            pivot[1]
                + angle.sin() * (old[0] - pivot[0])
                + angle.cos() * (old[1] - pivot[1])
                + translation[1],
        ];
        for axis in 0..2 {
            assert!((actual[axis] - expected[axis]).abs() < 1e-10);
        }
        let actual_world = editor.project.pieces[0].placement.transform(actual);
        for axis in 0..3 {
            assert!((actual_world[axis] - expected_world[axis]).abs() < 1e-10);
        }
    }
    editor
        .execute(Command::TransformPiece {
            piece: 2,
            translation: translation.map(|x| -x),
            angle: -angle,
            pivot: [pivot[0] + translation[0], pivot[1] + translation[1]],
        })
        .unwrap();
    for (actual, expected) in endpoints(&editor.project, 13).into_iter().zip(before) {
        for axis in 0..2 {
            assert!((actual[axis] - expected[axis]).abs() < 1e-10);
        }
    }
    let retained = endpoints(&editor.project, 13);
    editor
        .execute(Command::SplitCurve { curve: 7, t: 0.4 })
        .unwrap();
    editor.execute(Command::JoinCurve { curve: 7 }).unwrap();
    assert_eq!(endpoints(&editor.project, 13), retained);
    assert_eq!(editor.project.pieces[0].internal_curves, [13]);
    for command in [
        Command::SplitCurve { curve: 13, t: 0.5 },
        Command::JoinCurve { curve: 13 },
        Command::SetCurveControls {
            curve: 13,
            controls: None,
        },
        Command::RemoveInternalCurve { curve: 7 },
    ] {
        assert!(rejected(&mut editor, command).contains("internal"));
    }
    editor.execute(Command::RemovePiece { piece: 2 }).unwrap();
    assert!(
        editor.project.pieces.is_empty()
            && editor.project.curves.is_empty()
            && editor.project.points.is_empty()
    );
    assert!(editor.undo());
    assert_eq!(endpoints(&editor.project, 13), retained);
    let mut r = baseline();
    rejected(
        &mut r,
        Command::SetCurveControls {
            curve: 7,
            controls: Some([[1. / 3., 1.], [2. / 3., 1.]]),
        },
    );
    rejected(
        &mut r,
        Command::MovePoint {
            point: 3,
            position: [0.9, 0.9],
        },
    );
}

#[test]
fn annotations_leave_every_mesh_and_physical_field_exterior_only() {
    let mut editor = square();
    editor
        .execute(Command::AddRectangle {
            name: "Second".into(),
            origin: [2., 0.],
            width: 1.,
            height: 1.,
        })
        .unwrap();
    editor
        .execute(Command::Transaction(vec![
            Command::AddSeam {
                name: "Reversed partial legacy".into(),
                a: CurveInterval {
                    curve: 7,
                    t0: 0.2,
                    t1: 0.8,
                },
                b: CurveInterval {
                    curve: 16,
                    t0: 0.9,
                    t1: 0.1,
                },
                correspondence: vec![[0., 0.], [0.3, 0.6], [1., 1.]],
            },
            Command::AddSewingGroup {
                name: "Ordered group".into(),
                a: vec![
                    CurveInterval {
                        curve: 8,
                        t0: 0.,
                        t1: 0.5,
                    },
                    CurveInterval {
                        curve: 8,
                        t0: 0.5,
                        t1: 1.,
                    },
                ],
                b: vec![
                    CurveInterval {
                        curve: 17,
                        t0: 1.,
                        t1: 0.5,
                    },
                    CurveInterval {
                        curve: 17,
                        t0: 0.5,
                        t1: 0.,
                    },
                ],
                correspondence: vec![[0., 0.], [1., 1.]],
            },
        ]))
        .unwrap();
    let mut mesh = editor.project.mesh().unwrap();
    let masses = create_cloth(&editor.project, &mesh)
        .unwrap()
        .masses()
        .to_vec();
    let seams = editor.project.seams.clone();
    let groups = editor.project.sewing_groups.clone();
    let curve = next_id(&editor.project) + 2;
    for command in [
        add([0.125, 0.25], [0.875, 0.75]),
        Command::MovePoint {
            point: curve - 1,
            position: [0.75, 0.7],
        },
        Command::RemoveInternalCurve { curve },
    ] {
        editor.execute(command).unwrap();
        assert!(create_cloth(&editor.project, &mesh).is_err());
        let next = editor.project.mesh().unwrap();
        mesh.doc_revision = next.doc_revision;
        assert_eq!(mesh, next);
        assert_eq!(
            create_cloth(&editor.project, &next).unwrap().masses(),
            masses
        );
        assert_eq!(editor.project.seams, seams);
        assert_eq!(editor.project.sewing_groups, groups);
        mesh = next;
    }
    editor.execute(add([0.2, 0.2], [0.7, 0.7])).unwrap();
    let internal = editor.project.pieces[0].internal_curves[0];
    for command in [
        Command::AddSeam {
            name: "Invalid".into(),
            a: CurveInterval::full(internal),
            b: CurveInterval::full(7),
            correspondence: vec![[0., 0.], [1., 1.]],
        },
        Command::AddSewingGroup {
            name: "Invalid".into(),
            a: vec![CurveInterval::full(internal)],
            b: vec![CurveInterval::full(7)],
            correspondence: vec![[0., 0.], [1., 1.]],
        },
        Command::AddSewingByLength {
            name: "Invalid".into(),
            a: SewingSideByLength {
                curve: internal,
                start_t: 0.,
                direction: SewingDirection::Forward,
                length_m: 0.1,
            },
            b: SewingSideByLength {
                curve: 7,
                start_t: 0.,
                direction: SewingDirection::Forward,
                length_m: 0.1,
            },
        },
    ] {
        assert!(rejected(&mut editor, command).contains("outer boundary"));
    }
    let mut bad = editor.project.clone();
    bad.seams[0].a.curve = internal;
    assert!(bad.to_json().is_err());
    let mut bad = editor.project.clone();
    bad.sewing_groups[0].a[0].curve = internal;
    assert!(bad.to_json().is_err());
    assert!(bad.sewing_group_geometry(bad.sewing_groups[0].id).is_err());
}

#[test]
fn v4_endpoint_bits_and_strict_annotation_input_round_trip() {
    let mut editor = polygon(vec![[-1., -1.], [1., -1.], [1., 1.], [-1., 1.]]);
    let a = [
        "-0.0".parse::<f64>().unwrap(),
        "0.12345678901234566".parse().unwrap(),
    ];
    let b = [
        "0.7199999999999999".parse::<f64>().unwrap(),
        "-0.09999999999999999".parse().unwrap(),
    ];
    editor.execute(add(a, b)).unwrap();
    let bytes = editor.project.to_json().unwrap();
    let reopened = Project::from_json(&bytes).unwrap();
    assert_eq!(
        endpoints(&reopened, 13).map(|p| p.map(f64::to_bits)),
        [a, b].map(|p| p.map(f64::to_bits))
    );
    let text = String::from_utf8(bytes.clone()).unwrap();
    for text in [
        text.replacen(
            "\"internal_curves\":",
            "\"internal_curves\":[],\"internal_curves\":",
            1,
        ),
        text.replacen("-0.0", "1e309", 1),
    ] {
        assert!(Project::from_json(text.as_bytes()).is_err());
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for mutation in 0..7 {
        let mut bad = value.clone();
        match mutation {
            0 => bad["project"]["pieces"][0]
                .as_object_mut()
                .unwrap()
                .remove("internal_curves")
                .map(|_| ())
                .unwrap(),
            1 => bad["project"]["curves"][4]["controls"] = json!([[0.1, 0.1], [0.2, 0.2]]),
            2 => bad["project"]["pieces"][0]["internal_curves"] = json!([13, 13]),
            3 => bad["project"]["points"][4]["position"] = json!([1000.1, 0.2]),
            4 => bad["project"]["points"][4]["id"] = json!(3),
            5 => bad["project"]["pieces"][0]["unknown"] = json!(true),
            _ => bad["project"]
                .as_object_mut()
                .unwrap()
                .remove("sewing_groups")
                .map(|_| ())
                .unwrap(),
        }
        assert!(Project::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for raw in [
        r#"{"type":"add_internal_segment","data":{"piece":2,"start":[0.2,0.2],"end":[0.7,0.7],"extra":0}}"#,
        r#"{"type":"add_internal_segment","data":{"piece":2,"piece":2,"start":[0.2,0.2],"end":[0.7,0.7]}}"#,
        r#"{"type":"add_internal_segment","data":{"piece":2,"start":[0.2],"end":[0.7,0.7]}}"#,
        r#"{"type":"remove_internal_curve","data":{"curve":13,"curve":13}}"#,
    ] {
        assert!(serde_json::from_str::<Command>(raw).is_err());
    }
}

#[test]
fn actual_project_validation_exhausts_fixed_containment_budget_atomically() {
    // 600 exterior edges x 900 independent lines need over 1e6 edge/winding predicates.
    let mut vertices = Vec::new();
    for side in 0..4 {
        for i in 0..150 {
            let t = i as f64 / 150.;
            vertices.push(match side {
                0 => [t, 0.],
                1 => [1., t],
                2 => [1. - t, 1.],
                _ => [0., 1. - t],
            });
        }
    }
    let mut editor = polygon(vertices);
    let before = editor.project.clone();
    let commands = (0..900).map(|_| add([0.25, 0.25], [0.75, 0.75])).collect();
    let error = rejected(&mut editor, Command::Transaction(commands));
    assert!(
        error.contains("containment work budget exhausted"),
        "{error}"
    );
    assert_eq!(editor.project, before);
    let mut limited = baseline().project;
    let mut unused_id = next_id(&limited);
    while limited.points.len()
        + limited.curves.len()
        + limited.pieces.len()
        + limited.materials.len()
        <= 20_000
    {
        let id = unused_id;
        unused_id += 1;
        limited.points.push(Point {
            id,
            position: [0.2, 0.2],
        });
    }
    assert!(
        limited
            .validate()
            .unwrap_err()
            .to_string()
            .contains("object limit")
    );
}

#[test]
fn standalone_owned_internal_curve_requires_both_actual_endpoint_objects() {
    let mut project = square().project;
    project.curves.push(Curve {
        id: 13,
        start: 11,
        end: 12,
        controls: None,
    });
    project.pieces[0].internal_curves.push(13);
    project.points.push(Point {
        id: 11,
        position: [0.2, 0.2],
    });
    let mut value = serde_json::to_value(&project).unwrap();
    value["next_id"] = json!(14);
    project = serde_json::from_value(value).unwrap();
    assert!(project.validate().is_err());
    project.points.push(Point {
        id: 12,
        position: [0.7, 0.7],
    });
    project.validate().unwrap();
}

#[test]
fn exterior_exports_omit_annotations_and_failed_native_save_keeps_both_files() {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };
    let directory = std::env::temp_dir().join(format!(
        "weftforge-internal-io-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    let native = directory.join("construction.weftforge");
    let before = square();
    before.project.save(&native).unwrap();
    let old_bytes = fs::read(&native).unwrap();
    let mut after = baseline();
    let mut exterior = before.project.clone();
    exterior.revision = after.project.revision;
    for (label, project) in [("before", &exterior), ("after", &after.project)] {
        project
            .export_svg(directory.join(format!("{label}.svg")))
            .unwrap();
        project
            .export_dxf(directory.join(format!("{label}.dxf")))
            .unwrap();
        project
            .mesh()
            .unwrap()
            .export_obj(directory.join(format!("{label}.obj")))
            .unwrap();
    }
    for extension in ["svg", "dxf"] {
        assert_eq!(
            fs::read(directory.join(format!("before.{extension}"))).unwrap(),
            fs::read(directory.join(format!("after.{extension}"))).unwrap()
        );
    }
    let geometry = |label: &str| {
        fs::read_to_string(directory.join(format!("{label}.obj")))
            .unwrap()
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(geometry("before"), geometry("after"));
    after.project.save(&native).unwrap();
    assert_eq!(
        fs::read(weftforge_core::recovery_path(&native)).unwrap(),
        old_bytes
    );
    assert_eq!(Project::load(&native).unwrap(), after.project);
    let primary = fs::read(&native).unwrap();
    let recovery = fs::read(weftforge_core::recovery_path(&native)).unwrap();
    after
        .project
        .points
        .iter_mut()
        .find(|point| point.id == 11)
        .unwrap()
        .position = [0., 0.];
    assert!(after.project.save(&native).is_err());
    assert_eq!(fs::read(&native).unwrap(), primary);
    assert_eq!(
        fs::read(weftforge_core::recovery_path(&native)).unwrap(),
        recovery
    );
    fs::remove_dir_all(directory).unwrap();
}
