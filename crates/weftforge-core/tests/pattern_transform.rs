use std::{
    f64::consts::FRAC_PI_2,
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use weftforge_core::{
    Command, CurveInterval, Editor, Id, Placement, Project, simulation::create_cloth,
};

const SAMPLES: [f64; 7] = [0., 0.125, 0.25, 0.5, 0.75, 0.875, 1.];

fn fixture() -> Editor {
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Asymmetric parabola".into(),
            vertices: vec![[0., 0.], [0.75, 0.75], [-0.25, 1.25], [-0.5, 0.25]],
        })
        .unwrap();
    editor
        .execute(Command::AddRectangle {
            name: "Separate panel".into(),
            origin: [2., 0.],
            width: 0.75,
            height: 1.25,
        })
        .unwrap();
    let piece = &editor.project.pieces[0];
    let (piece, first, second) = (
        piece.id,
        piece.boundary.clone(),
        editor.project.pieces[1].boundary.clone(),
    );
    let mut material = editor.project.materials[0].clone();
    material.warp_stiffness = 50.;
    material.weft_stiffness = 1000.;
    editor
        .execute(Command::Transaction(vec![
            Command::SetCurveControls {
                curve: first[0],
                controls: Some([[0.25, 0.], [0.5, 0.25]]),
            },
            Command::SetPlacement {
                piece,
                placement: Placement {
                    translation: [0.2, 0.4, -0.3],
                    rotation: [0.31, -0.47, 0.21],
                },
            },
            Command::SetPlacement {
                piece: editor.project.pieces[1].id,
                placement: Placement {
                    translation: [-1., 0.7, 0.4],
                    rotation: [-0.12, 0.17, -0.3],
                },
            },
            Command::SetGrain { piece, angle: 0.37 },
            Command::UpdateMaterial { material },
            Command::SetResolution { metres: 0.2 },
            Command::AddSeam {
                name: "Reversed partial legacy".into(),
                a: CurveInterval {
                    curve: first[0],
                    t0: 0.875,
                    t1: 0.125,
                },
                b: CurveInterval {
                    curve: second[1],
                    t0: 0.1,
                    t1: 0.9,
                },
                correspondence: vec![[0., 0.], [0.4, 0.65], [1., 1.]],
            },
            Command::AddSewingGroup {
                name: "Ordered spans with a gap".into(),
                a: vec![
                    CurveInterval {
                        curve: first[0],
                        t0: 0.1,
                        t1: 0.4,
                    },
                    CurveInterval {
                        curve: first[0],
                        t0: 0.9,
                        t1: 0.6,
                    },
                    CurveInterval::reversed(first[2]),
                ],
                b: vec![
                    CurveInterval {
                        curve: second[0],
                        t0: 0.2,
                        t1: 0.8,
                    },
                    CurveInterval::reversed(second[2]),
                ],
                correspondence: vec![[0., 0.], [0.3, 0.55], [1., 1.]],
            },
        ]))
        .unwrap();
    Editor::new(editor.project).unwrap()
}

fn command(piece: Id, translation: [f64; 2], angle: f64, pivot: [f64; 2]) -> Command {
    Command::TransformPiece {
        piece,
        translation,
        angle,
        pivot,
    }
}

fn close<const N: usize>(actual: [f64; N], expected: [f64; N], tolerance: f64) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual:.17} != {expected:.17}; bound {tolerance}"
        );
    }
}

fn same_authored(actual: &Project, expected: &Project) {
    let mut normalized = actual.clone();
    normalized.revision = expected.revision;
    assert_eq!(&normalized, expected);
    assert_eq!(normalized.to_json().unwrap(), expected.to_json().unwrap());
}

fn next_id(project: &Project) -> serde_json::Value {
    serde_json::from_slice::<serde_json::Value>(&project.to_json().unwrap()).unwrap()["project"]
        ["next_id"]
        .clone()
}

fn warp(placement: Placement, angle: f64) -> [f64; 3] {
    Placement {
        translation: [0.; 3],
        ..placement
    }
    .transform([angle.cos(), angle.sin()])
}

fn assert_arrangement(before: &Project, after: &Project, tolerance: f64) {
    let old = &before.pieces[0];
    let new = &after.pieces[0];
    for &curve in &old.boundary {
        for t in SAMPLES {
            close(
                new.placement
                    .transform(after.curve(curve).unwrap().evaluate(after, t).unwrap()),
                old.placement
                    .transform(before.curve(curve).unwrap().evaluate(before, t).unwrap()),
                tolerance,
            );
        }
    }
    close(
        warp(new.placement, new.grain_angle),
        warp(old.placement, old.grain_angle),
        1e-12,
    );
}

fn analytic_length(t: f64) -> f64 {
    0.375 * t * (1. + 4. * t * t).sqrt() + 0.1875 * (2. * t).asinh()
}

#[test]
fn translation_uses_single_additions_and_keeps_zero_components_and_unrelated_bits() {
    for delta in [[0.5, -0.25], [0.5, 0.], [0., -0.25], [0., -0.]] {
        let mut editor = fixture();
        let before = editor.project.clone();
        let piece = before.pieces[0].id;
        editor
            .execute(command(piece, delta, 0., [1000., -1000.]))
            .unwrap();
        let after = &editor.project;
        for (old, new) in before.points.iter().zip(&after.points) {
            for (axis, &delta) in delta.iter().enumerate() {
                let expected = if old.id < before.pieces[1].id && delta != 0. {
                    old.position[axis] + delta
                } else {
                    old.position[axis]
                };
                assert_eq!(new.position[axis].to_bits(), expected.to_bits());
            }
        }
        for (old, new) in before.curves.iter().zip(&after.curves) {
            assert_eq!((old.id, old.start, old.end), (new.id, new.start, new.end));
            if let Some(controls) = old.controls {
                let actual = new.controls.unwrap();
                for (old, new) in controls.into_iter().zip(actual) {
                    for (axis, &delta) in delta.iter().enumerate() {
                        let expected = if delta == 0. {
                            old[axis]
                        } else {
                            old[axis] + delta
                        };
                        assert_eq!(new[axis].to_bits(), expected.to_bits());
                    }
                }
            }
        }
        assert_eq!(
            after.pieces[0].grain_angle.to_bits(),
            before.pieces[0].grain_angle.to_bits()
        );
        assert_eq!(
            after.pieces[0].placement.rotation.map(f64::to_bits),
            before.pieces[0].placement.rotation.map(f64::to_bits)
        );
        assert_eq!(after.pieces[1], before.pieces[1]);
        assert_eq!(after.materials, before.materials);
        assert_eq!(after.seams, before.seams);
        assert_eq!(after.sewing_groups, before.sewing_groups);
        assert_eq!(next_id(after), next_id(&before));
        assert_arrangement(&before, after, 1e-10);
        if delta == [0., 0.] {
            same_authored(after, &before);
        }
        assert_eq!(after.revision, before.revision + 1);
        assert!(editor.undo());
        same_authored(&editor.project, &before);
        assert!(!editor.can_undo());
    }
    let mut editor = fixture();
    editor.project.points[0].position = [-0., -0.];
    editor
        .execute(command(
            editor.project.pieces[0].id,
            [0., 0.],
            -0.,
            [0., 0.],
        ))
        .unwrap();
    assert_eq!(
        editor.project.points[0].position.map(f64::to_bits),
        [(-0.0_f64).to_bits(); 2]
    );
}

#[test]
fn quarter_turn_has_independent_polynomial_order_grain_and_sewing_oracles() {
    let mut editor = fixture();
    let before = editor.project.clone();
    let piece = before.pieces[0].id;
    let curve = before.pieces[0].boundary[0];
    editor
        .execute(command(piece, [0.5, -0.25], FRAC_PI_2, [0.25, 0.5]))
        .unwrap();
    let after = &editor.project;
    for (old, new) in before.points[..4].iter().zip(&after.points[..4]) {
        close(
            new.position,
            [1.25 - old.position[1], old.position[0]],
            1e-10,
        );
    }
    for (old, new) in before.curves[0]
        .controls
        .unwrap()
        .into_iter()
        .zip(after.curves[0].controls.unwrap())
    {
        close(new, [1.25 - old[1], old[0]], 1e-10);
    }
    for t in SAMPLES {
        close(
            after.curve(curve).unwrap().evaluate(after, t).unwrap(),
            [1.25 - 0.75 * t * t, 0.75 * t],
            1e-10,
        );
    }
    close(
        [
            after.pieces[0].grain_angle.cos(),
            after.pieces[0].grain_angle.sin(),
        ],
        [-0.37_f64.sin(), 0.37_f64.cos()],
        1e-12,
    );
    assert!((-std::f64::consts::PI..=std::f64::consts::PI).contains(&after.pieces[0].grain_angle));
    assert_arrangement(&before, after, 1e-10);
    assert_eq!(after.points[4..], before.points[4..]);
    assert_eq!(
        after.points.iter().map(|p| p.id).collect::<Vec<_>>(),
        before.points.iter().map(|p| p.id).collect::<Vec<_>>()
    );
    assert_eq!(
        after
            .curves
            .iter()
            .map(|c| (c.id, c.start, c.end, c.controls.is_some()))
            .collect::<Vec<_>>(),
        before
            .curves
            .iter()
            .map(|c| (c.id, c.start, c.end, c.controls.is_some()))
            .collect::<Vec<_>>()
    );
    assert_eq!(after.curves[1..], before.curves[1..]);
    assert_eq!(after.pieces[1], before.pieces[1]);
    assert_eq!(after.seams, before.seams);
    assert_eq!(after.sewing_groups, before.sewing_groups);
    assert_eq!(after.materials, before.materials);
    assert_eq!(after.mannequin, before.mannequin);
    assert_eq!(after.resolution.to_bits(), before.resolution.to_bits());
    assert_eq!(next_id(after), next_id(&before));
    assert_eq!(
        (
            after.pieces[0].id,
            &after.pieces[0].name,
            &after.pieces[0].boundary,
            after.pieces[0].material
        ),
        (
            before.pieces[0].id,
            &before.pieces[0].name,
            &before.pieces[0].boundary,
            before.pieces[0].material
        )
    );
    let interval = before.seams[0].a;
    let expected = (analytic_length(interval.t1) - analytic_length(interval.t0)).abs();
    for project in [&before, after] {
        assert!((project.interval_length(interval).unwrap() - expected).abs() <= 1e-8);
        assert!(
            (project.interval_length(CurveInterval::full(curve)).unwrap() - analytic_length(1.))
                .abs()
                <= 1e-8
        );
    }
    for (old, new) in before
        .seam_lengths(before.seams[0].id)
        .unwrap()
        .into_iter()
        .zip(after.seam_lengths(before.seams[0].id).unwrap())
    {
        assert!((old - new).abs() <= 2e-8);
    }
    for (old, new) in before
        .sewing_group_geometry(before.sewing_groups[0].id)
        .unwrap()
        .lengths
        .into_iter()
        .zip(
            after
                .sewing_group_geometry(before.sewing_groups[0].id)
                .unwrap()
                .lengths,
        )
    {
        assert!((old - new).abs() <= 2e-8);
    }
    let changed = after.clone();
    assert!(editor.undo());
    same_authored(&editor.project, &before);
    assert!(!editor.can_undo());
    assert!(editor.redo());
    same_authored(&editor.project, &changed);
    assert!(editor.project.revision > changed.revision);
}

#[test]
fn non_right_angle_inverse_and_pivot_preserve_tilted_gimbal_and_large_scale_surfaces() {
    for rotation in [
        [0., 0., 0.],
        [0.31, -0.47, 0.21],
        [0., FRAC_PI_2, 0.],
        [0., -FRAC_PI_2, 0.],
        [0.31, FRAC_PI_2 - 1e-12, 0.21],
        [0.31, -FRAC_PI_2 + 1e-12, 0.21],
    ] {
        for angle in [0.0, 0.43, FRAC_PI_2, 1e6, -1e6] {
            let mut editor = fixture();
            editor.project.pieces[0].placement.rotation = rotation;
            let before = editor.project.clone();
            let piece = before.pieces[0].id;
            let pivot = [0.25, 0.5];
            let delta = [0.5, -0.25];
            editor.execute(command(piece, delta, angle, pivot)).unwrap();
            assert_arrangement(&before, &editor.project, 1e-10);
            close(
                editor.project.pieces[0]
                    .placement
                    .transform([pivot[0] + delta[0], pivot[1] + delta[1]]),
                before.pieces[0].placement.transform(pivot),
                1e-10,
            );
            let (s, c) = angle.sin_cos();
            for (old, new) in before.points[..4].iter().zip(&editor.project.points[..4]) {
                let u = old.position[0] - pivot[0];
                let v = old.position[1] - pivot[1];
                close(
                    new.position,
                    [
                        pivot[0] + c * u - s * v + delta[0],
                        pivot[1] + s * u + c * v + delta[1],
                    ],
                    1e-10,
                );
            }
            editor
                .execute(command(
                    piece,
                    [-delta[0], -delta[1]],
                    -angle,
                    [pivot[0] + delta[0], pivot[1] + delta[1]],
                ))
                .unwrap();
            assert_arrangement(&before, &editor.project, 1e-10);
            for (old, new) in before.points.iter().zip(&editor.project.points) {
                close(new.position, old.position, 1e-10);
            }
            for (old, new) in before.curves[0]
                .controls
                .unwrap()
                .into_iter()
                .zip(editor.project.curves[0].controls.unwrap())
            {
                close(new, old, 1e-10);
            }
            close(
                [
                    editor.project.pieces[0].grain_angle.cos(),
                    editor.project.pieces[0].grain_angle.sin(),
                ],
                [0.37_f64.cos(), 0.37_f64.sin()],
                1e-12,
            );
        }
    }
    // Choose a source whose right-composed basis lands exactly/near the Euler singularity.
    for y in [
        FRAC_PI_2,
        -FRAC_PI_2,
        FRAC_PI_2 - 1e-15,
        -FRAC_PI_2 + 1e-15,
        FRAC_PI_2 - 1e-12,
        -FRAC_PI_2 + 1e-12,
    ] {
        let mut editor = fixture();
        editor.project.pieces[0].placement.rotation = [0.43, y, 0.21];
        let before = editor.project.clone();
        editor
            .execute(command(before.pieces[0].id, [0.5, -0.25], 0., [0.25, 0.5]))
            .unwrap();
        editor
            .execute(command(before.pieces[0].id, [0., 0.], 1e-16, [0.25, 0.5]))
            .unwrap();
        assert_arrangement(&before, &editor.project, 1e-10);
    }
    let mut editor = fixture();
    for point in &mut editor.project.points[..4] {
        point.position[0] += 998.;
    }
    for control in editor.project.curves[0].controls.as_mut().unwrap() {
        control[0] += 998.;
    }
    editor.project.pieces[0].placement.translation = [999., -999., 999.];
    let before = editor.project.clone();
    editor
        .execute(command(before.pieces[0].id, [0., 0.], FRAC_PI_2, [0., 0.]))
        .unwrap();
    assert_arrangement(&before, &editor.project, 1e-9);
}

#[test]
fn huge_existing_grain_is_rotated_as_a_unit_vector_without_addition_overflow() {
    let mut editor = fixture();
    editor.project.pieces[0].grain_angle = f64::MAX;
    let before = editor.project.clone();
    editor
        .execute(command(before.pieces[0].id, [0., 0.], 0.43, [0.25, 0.5]))
        .unwrap();
    let old = before.pieces[0].grain_angle;
    let actual = editor.project.pieces[0].grain_angle;
    close(
        [actual.cos(), actual.sin()],
        [
            0.43_f64.cos() * old.cos() - 0.43_f64.sin() * old.sin(),
            0.43_f64.sin() * old.cos() + 0.43_f64.cos() * old.sin(),
        ],
        1e-12,
    );
    assert!(actual.abs() <= std::f64::consts::PI);
    assert_arrangement(&before, &editor.project, 1e-10);
}

#[test]
fn mesh_rebuild_uses_new_authored_uvs_and_revision_and_native_v4_round_trips() {
    let mut editor = fixture();
    let before = editor.project.clone();
    let old_mesh = before.mesh().unwrap();
    create_cloth(&before, &old_mesh).unwrap();
    editor
        .execute(command(
            before.pieces[0].id,
            [0.5, -0.25],
            FRAC_PI_2,
            [0.25, 0.5],
        ))
        .unwrap();
    let after = &editor.project;
    assert!(create_cloth(after, &old_mesh).is_err());
    let mesh = after.mesh().unwrap();
    mesh.validate().unwrap();
    assert_eq!(mesh.doc_revision, after.revision);
    create_cloth(after, &mesh).unwrap();
    for ((&uv, &world), &piece) in mesh.uv.iter().zip(&mesh.positions).zip(&mesh.piece_ids) {
        close(
            after.piece(piece).unwrap().placement.transform(uv),
            world,
            1e-10,
        );
        let old_uv = if piece == before.pieces[0].id {
            [uv[1], 1.25 - uv[0]]
        } else {
            uv
        };
        close(
            before.piece(piece).unwrap().placement.transform(old_uv),
            world,
            1e-10,
        );
    }
    for sample in &mesh.boundary {
        let piece = after.piece(sample.piece).unwrap();
        assert!(piece.boundary.contains(&sample.curve));
        close(
            mesh.uv[sample.vertex],
            after
                .curve(sample.curve)
                .unwrap()
                .evaluate(after, sample.t)
                .unwrap(),
            1e-10,
        );
    }
    let sewing_ids: Vec<_> = after
        .seams
        .iter()
        .map(|s| s.id)
        .chain(after.sewing_groups.iter().map(|g| g.id))
        .collect();
    assert!(mesh.stitches.iter().all(|s| sewing_ids.contains(&s.seam)));
    for id in sewing_ids {
        assert!(mesh.stitches.iter().any(|s| s.seam == id));
    }
    let directory = std::env::temp_dir().join(format!(
        "weftforge-transform-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("transformed.weftforge");
    after.save(&path).unwrap();
    let reopened = Project::load(&path).unwrap();
    assert_eq!(reopened.to_json().unwrap(), after.to_json().unwrap());
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["version"], weftforge_core::FORMAT_VERSION);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn invalid_inputs_results_and_late_transactions_retain_every_history_branch() {
    let mut editor = fixture();
    let piece = editor.project.pieces[0].id;
    editor
        .execute(Command::SetResolution { metres: 0.18 })
        .unwrap();
    editor
        .execute(Command::SetGrain { piece, angle: 0.6 })
        .unwrap();
    assert!(editor.undo());
    let original = format!("{editor:?}");
    let bytes = editor.project.to_json().unwrap();
    let mut invalid = vec![
        command(0, [0., 0.], 0., [0., 0.]),
        command(u64::MAX, [0., 0.], 0., [0., 0.]),
        command(piece, [1000., 0.], 0., [0., 0.]),
        command(piece, [0., 0.], FRAC_PI_2, [1000., -1000.]),
    ];
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        1000.0001,
        -1000.0001,
    ] {
        for axis in 0..2 {
            let mut pair = [0., 0.];
            pair[axis] = value;
            invalid.push(command(piece, pair, 0., [0., 0.]));
            invalid.push(command(piece, [0., 0.], 0., pair));
        }
    }
    for angle in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        1e6 + 1.,
        -1e6 - 1.,
    ] {
        invalid.push(command(piece, [0., 0.], angle, [0., 0.]));
    }
    invalid.push(Command::Transaction(vec![
        command(piece, [0.5, -0.25], 0.43, [0.25, 0.5]),
        Command::MovePoint {
            point: 0,
            position: [0., 0.],
        },
    ]));
    invalid.push(Command::Transaction(vec![
        Command::AddRectangle {
            name: "Uncommitted allocated IDs".into(),
            origin: [5., 5.],
            width: 0.2,
            height: 0.3,
        },
        command(0, [0., 0.], 0., [0., 0.]),
    ]));
    for command in invalid {
        assert!(
            editor.execute(command.clone()).is_err(),
            "accepted {command:?}"
        );
        assert_eq!(format!("{editor:?}"), original);
        assert_eq!(editor.project.to_json().unwrap(), bytes);
    }
    assert!(editor.redo());
    assert_eq!(editor.project.pieces[0].grain_angle, 0.6);
    assert!(editor.undo());
    assert!(editor.undo());
    assert_eq!(editor.project.resolution, 0.2);
    assert!(!editor.undo());
    assert_eq!(editor.project.pieces[0].grain_angle, 0.37);
}

#[test]
fn control_only_and_compensated_placement_overflow_reject_atomically() {
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Control limit".into(),
            vertices: vec![[999., 0.], [999., 1.], [998., 1.], [998., 0.]],
        })
        .unwrap();
    let piece = editor.project.pieces[0].id;
    editor
        .execute(Command::SetCurveControls {
            curve: editor.project.pieces[0].boundary[0],
            controls: Some([[999.75, 0.25], [999.75, 0.75]]),
        })
        .unwrap();
    let before = format!("{editor:?}");
    assert!(
        editor
            .execute(command(piece, [0.5, 0.], 0., [0., 0.]))
            .is_err()
    );
    assert_eq!(format!("{editor:?}"), before);
    let mut editor = fixture();
    editor.project.pieces[0].placement = Placement {
        translation: [1000., 0., 0.],
        rotation: [0.; 3],
    };
    let before = format!("{editor:?}");
    assert!(
        editor
            .execute(command(
                editor.project.pieces[0].id,
                [-0.5, 0.],
                0.,
                [0., 0.]
            ))
            .is_err()
    );
    assert_eq!(format!("{editor:?}"), before);
}

#[test]
fn transform_json_requires_all_fields_fixed_arrays_and_original_strict_number_tokens() {
    for raw in [
        r#"{"type":"transform_piece","data":{"piece":2,"translation":[0,0],"angle":0}}"#,
        r#"{"type":"transform_piece","data":{"piece":2,"translation":[0],"angle":0,"pivot":[0,0]}}"#,
        r#"{"type":"transform_piece","data":{"piece":2,"translation":[0,0,0],"angle":0,"pivot":[0,0]}}"#,
        r#"{"type":"transform_piece","data":{"piece":2,"translation":[0,0],"angle":0,"pivot":[0]}}"#,
        r#"{"type":"transform_piece","data":{"piece":2,"translation":[0,0],"angle":1e400,"pivot":[0,0]}}"#,
        r#"{"type":"transform_piece","data":{"piece":2,"translation":[1e400,0],"angle":0,"pivot":[0,0]}}"#,
        r#"{"type":"transform_piece","data":{"piece":2,"translation":[0,0],"angle":0,"pivot":[1e400,0]}}"#,
        r#"{"type":"transform_piece","data":{"piece":2,"translation":[0,0],"angle":0,"angle":1,"pivot":[0,0]}}"#,
        r#"{"type":"transform_piece","data":{"piece":2,"translation":[0,0],"angle":0,"pivot":[0,0],"extra":0}}"#,
    ] {
        assert!(
            serde_json::from_str::<Command>(raw).is_err(),
            "accepted {raw}"
        );
    }
    let decoded: Command = serde_json::from_str(r#"{"type":"transform_piece","data":{"piece":2,"translation":[0.5,-0.25],"angle":0.43,"pivot":[0.25,0.5]}}"#).unwrap();
    assert_eq!(decoded, command(2, [0.5, -0.25], 0.43, [0.25, 0.5]));
}
