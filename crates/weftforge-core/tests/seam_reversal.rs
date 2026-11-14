use weftforge_core::{Command, CurveInterval, Editor, Mesh, Project};

fn partial_scene(reverse_a: bool, reverse_b: bool) -> Editor {
    let mut editor = Editor::default();
    for (name, origin, width, height) in [
        ("Curved front", [0., 0.], 0.7, 0.5),
        ("Straight back", [1., 0.], 0.45, 0.3),
    ] {
        editor
            .execute(Command::AddRectangle {
                name: name.into(),
                origin,
                width,
                height,
            })
            .unwrap();
    }
    let a = editor.project.pieces[0].boundary[2];
    let b = editor.project.pieces[1].boundary[0];
    editor
        .execute(Command::Transaction(vec![
            Command::SetCurveControls {
                curve: a,
                controls: Some([[0.55, 0.7], [0.12, 0.62]]),
            },
            Command::AddSeam {
                name: "Partial gathered pair".into(),
                a: CurveInterval {
                    curve: a,
                    t0: if reverse_a { 0.85 } else { 0.15 },
                    t1: if reverse_a { 0.15 } else { 0.85 },
                },
                b: CurveInterval {
                    curve: b,
                    t0: if reverse_b { 0.9 } else { 0.1 },
                    t1: if reverse_b { 0.1 } else { 0.9 },
                },
                correspondence: vec![[0., 0.], [0.25, 0.6], [1., 1.]],
            },
        ]))
        .unwrap();
    editor
}

fn parameter(mesh: &Mesh, curve: u64, endpoint: weftforge_core::WeightedVertex) -> f64 {
    let t = endpoint.vertices.map(|vertex| {
        mesh.boundary
            .iter()
            .find(|sample| sample.curve == curve && sample.vertex == vertex)
            .unwrap()
            .t
    });
    t[0] * (1. - endpoint.weight) + t[1] * endpoint.weight
}

fn assert_pairs(project: &Project, a: [f64; 2], b: [f64; 2]) {
    let mesh = project.mesh().unwrap();
    let seam = &project.seams[0];
    let mut previous = -1.;
    let mut saw_breakpoint = false;
    for stitch in &mesh.stitches {
        assert_eq!(stitch.seam, seam.id);
        let u = (parameter(&mesh, seam.a.curve, stitch.a) - a[0]) / (a[1] - a[0]);
        let v = if u <= 0.25 {
            u * 2.4
        } else {
            0.6 + (u - 0.25) * (0.4 / 0.75)
        };
        let actual_b = parameter(&mesh, seam.b.curve, stitch.b);
        assert!((actual_b - (b[0] + (b[1] - b[0]) * v)).abs() < 1e-12);
        assert!(u > previous);
        saw_breakpoint |= (u - 0.25).abs() < 1e-12;
        previous = u;
    }
    assert!(saw_breakpoint);
    assert!((previous - 1.).abs() < 1e-12);
    let first = mesh.stitches.first().unwrap();
    let last = mesh.stitches.last().unwrap();
    for (stitch, at, bt) in [(first, a[0], b[0]), (last, a[1], b[1])] {
        assert!((parameter(&mesh, seam.a.curve, stitch.a) - at).abs() < 1e-12);
        assert!((parameter(&mesh, seam.b.curve, stitch.b) - bt).abs() < 1e-12);
    }
}

#[test]
fn reversal_changes_endpoint_pairing_and_preserves_partial_nonlinear_construction() {
    for reverse_a in [false, true] {
        for reverse_b in [false, true] {
            let mut editor = partial_scene(reverse_a, reverse_b);
            let before = editor.project.clone();
            let original = &before.seams[0];
            let a = [original.a.t0, original.a.t1];
            let b = [original.b.t0, original.b.t1];
            assert_pairs(&before, a, b);
            let mut reversed = original.clone();
            reversed.b = CurveInterval {
                t0: b[1],
                t1: b[0],
                ..original.b
            };
            editor
                .execute(Command::UpdateSeam { seam: reversed })
                .unwrap();
            let after = editor.project.clone();
            assert_eq!(after.revision, before.revision + 1);
            let mut expected = before.clone();
            expected.revision = after.revision;
            expected.seams[0].b.t0 = b[1];
            expected.seams[0].b.t1 = b[0];
            assert_eq!(after, expected);
            assert_pairs(&after, a, [b[1], b[0]]);
            editor
                .execute(Command::SetResolution { metres: 0.025 })
                .unwrap();
            assert_pairs(&editor.project, a, [b[1], b[0]]);
            assert!(editor.undo());
            assert!(editor.undo());
            assert_eq!(editor.project.seams, before.seams);
            assert_pairs(&editor.project, a, b);
            assert!(editor.redo());
            assert_pairs(&editor.project, a, [b[1], b[0]]);
            let saved = editor.project.to_json().unwrap();
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&saved).unwrap()["version"],
                2
            );
            let reopened = Project::from_json(&saved).unwrap();
            assert_eq!(reopened, editor.project);
            assert_pairs(&reopened, a, [b[1], b[0]]);
            editor
                .execute(Command::UpdateSeam {
                    seam: original.clone(),
                })
                .unwrap();
            let mut restored = editor.project.clone();
            restored.revision = before.revision;
            assert_eq!(restored, before);
        }
    }
}

#[test]
fn rejected_staged_reversal_retains_document_and_redo() {
    let mut editor = partial_scene(false, true);
    editor
        .execute(Command::SetResolution { metres: 0.075 })
        .unwrap();
    assert!(editor.undo());
    let before = editor.project.clone();
    let mut reversed = before.seams[0].clone();
    std::mem::swap(&mut reversed.b.t0, &mut reversed.b.t1);
    assert!(
        editor
            .execute(Command::Transaction(vec![
                Command::SetGrain {
                    piece: before.pieces[0].id,
                    angle: 0.25,
                },
                Command::UpdateSeam { seam: reversed },
                Command::SetResolution { metres: 0. },
            ]))
            .is_err()
    );
    assert_eq!(editor.project, before);
    assert!(editor.can_redo());
    assert!(editor.redo());
    assert_eq!(editor.project.seams, before.seams);
    assert_eq!(editor.project.resolution, 0.075);
}
