use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use weftforge_core::{Command, CurveInterval, Editor, MAX_OBJECTS, Material, Placement, Project};

fn ellipse(origin: [f64; 2], width: f64, height: f64) -> Command {
    Command::AddEllipse {
        name: "Original ellipse".into(),
        origin,
        width,
        height,
    }
}

fn close(actual: [f64; 2], expected: [f64; 2]) {
    assert!(
        (actual[0] - expected[0]).hypot(actual[1] - expected[1]) <= 1e-12,
        "{actual:?} != {expected:?}"
    );
}

// Independent power-basis evaluation, without Curve::evaluate's Bernstein form.
fn polynomial(p: [[f64; 2]; 4], t: f64) -> [f64; 2] {
    std::array::from_fn(|i| {
        let [a, b, c, d] = p.map(|p| p[i]);
        a + t * (3.0 * (b - a) + t * (3.0 * (a - 2.0 * b + c) + t * (d - a + 3.0 * (b - c))))
    })
}

#[test]
fn ellipse_has_declared_controls_ids_bounds_tangents_and_one_persistent_transaction() {
    let mut editor = Editor::default();
    editor.execute(ellipse([-0.4, 0.2], 0.8, 0.6)).unwrap();
    let project = &editor.project;
    assert_eq!(project.revision, 1);
    assert_eq!(
        project.points.iter().map(|p| p.id).collect::<Vec<_>>(),
        [3, 4, 5, 6]
    );
    let piece = &project.pieces[0];
    assert_eq!(piece.id, 2);
    assert_eq!(piece.boundary, [7, 8, 9, 10]);
    assert_eq!(
        (piece.material, piece.grain_angle, piece.placement),
        (1, 0., Placement::default())
    );
    assert_eq!(
        project.piece_bounds(piece.id).unwrap(),
        [[-0.4, 0.2], [0.4, 0.8]]
    );
    for (actual, expected) in
        project
            .points
            .iter()
            .zip([[0., 0.2], [0.4, 0.5], [0., 0.8], [-0.4, 0.5]])
    {
        close(actual.position, expected);
    }
    let expected = [
        [[0.22091389993231744, 0.2], [0.4, 0.33431457505076194]],
        [[0.4, 0.6656854249492381], [0.22091389993231744, 0.8]],
        [[-0.22091389993231744, 0.8], [-0.4, 0.6656854249492381]],
        [[-0.4, 0.33431457505076194], [-0.22091389993231744, 0.2]],
    ];
    for (i, curve) in project.curves.iter().enumerate() {
        let controls = curve.controls.unwrap();
        for (actual, expected) in controls.into_iter().zip(expected[i]) {
            close(actual, expected);
        }
        let next = &project.curves[(i + 1) % 4];
        assert_eq!(curve.end, next.start);
        let endpoint = project.point(curve.end).unwrap().position;
        close(
            std::array::from_fn(|i| 3.0 * (endpoint[i] - controls[1][i])),
            std::array::from_fn(|i| 3.0 * (next.controls.unwrap()[0][i] - endpoint[i])),
        );
    }
    let authored = project.clone();
    assert!(editor.undo());
    assert!(
        editor.project.pieces.is_empty()
            && editor.project.points.is_empty()
            && editor.project.curves.is_empty()
    );
    assert!(!editor.can_undo());
    assert!(editor.redo());
    assert_eq!(editor.project.points, authored.points);
    assert_eq!(editor.project.curves, authored.curves);
    assert_eq!(editor.project.pieces, authored.pieces);
    let directory = std::env::temp_dir().join(format!(
        "weftforge-ellipse-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("original.weftforge");
    editor.project.save(&path).unwrap();
    assert_eq!(Project::load(&path).unwrap(), editor.project);
    let file: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(file["version"], weftforge_core::FORMAT_VERSION);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn ellipse_radial_error_obeys_predeclared_independent_polynomial_bound() {
    let max_error = (1.0 + (34.0 - 24.0 * 2.0_f64.sqrt()) / 108.0).sqrt() - 1.0;
    assert!(max_error < 0.001);
    let mut sampled_max: f64 = 0.0;
    for (origin, width, height) in [
        ([-0.4, 0.2], 0.8, 0.6),
        ([0., 0.], 0.6, 0.6),
        ([5., -4.], 0.02, 0.8),
    ] {
        let mut editor = Editor::default();
        editor.execute(ellipse(origin, width, height)).unwrap();
        let project = &editor.project;
        let bounds = project.piece_bounds(project.pieces[0].id).unwrap();
        close(bounds[0], origin);
        close(bounds[1], [origin[0] + width, origin[1] + height]);
        let radii = [width / 2., height / 2.];
        let center = [origin[0] + radii[0], origin[1] + radii[1]];
        for curve in &project.curves {
            let [c, d] = curve.controls.unwrap();
            let p = [
                project.point(curve.start).unwrap().position,
                c,
                d,
                project.point(curve.end).unwrap().position,
            ]
            .map(|p| std::array::from_fn(|i| (p[i] - center[i]) / radii[i]));
            let extrema = [(3. - 3.0_f64.sqrt()) / 6., 0.5, (3. + 3.0_f64.sqrt()) / 6.];
            for t in (0..=2000).map(|i| f64::from(i) / 2000.).chain(extrema) {
                let [x, y] = polynomial(p, t);
                let expected_squared_error = (34. - 24. * 2.0_f64.sqrt())
                    * t.powi(2)
                    * (1. - t).powi(2)
                    * (2. * t - 1.).powi(2);
                assert!((x * x + y * y - 1. - expected_squared_error).abs() < 1e-12);
                let error = (x.hypot(y) - 1.).abs();
                sampled_max = sampled_max.max(error);
                assert!(error <= max_error + 1e-12 && error <= 0.001, "{error}");
            }
        }
    }
    println!(
        "Independent normalized radial error: sampled {sampled_max:.16}; analytic maximum {max_error:.16}; declared limit 0.001"
    );
}

#[test]
fn ellipse_rejects_bad_dimensions_coordinates_names_ids_and_resource_counts_atomically() {
    let mut editor = Editor::default();
    editor.execute(ellipse([0., 0.], 0.3, 0.4)).unwrap();
    editor
        .execute(Command::SetResolution { metres: 0.08 })
        .unwrap();
    assert!(editor.undo());
    let original = editor.project.clone();
    let history = (editor.can_undo(), editor.can_redo());
    let mut invalid: Vec<_> = [
        0.,
        -1.,
        1e-7,
        1e-6,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
    ]
    .into_iter()
    .flat_map(|n| [ellipse([0., 0.], n, 0.4), ellipse([0., 0.], 0.3, n)])
    .collect();
    invalid.extend([
        ellipse([0., 0.], 2e-6, 2e-6),
        ellipse([f64::NAN, 0.], 0.3, 0.4),
        ellipse([0., f64::INFINITY], 0.3, 0.4),
        ellipse([999.9, 0.], 0.3, 0.4),
        ellipse([f64::MAX, 0.], f64::MAX, 0.4),
        Command::AddEllipse {
            name: "".into(),
            origin: [0., 0.],
            width: 0.3,
            height: 0.4,
        },
    ]);
    for command in invalid {
        assert!(
            editor.execute(command.clone()).is_err(),
            "accepted {command:?}"
        );
        assert_eq!(editor.project, original);
        assert_eq!((editor.can_undo(), editor.can_redo()), history);
    }
    for raw in [
        r#"{"type":"add_ellipse","data":{"name":"x","origin":[0,0],"width":0.3}}"#,
        r#"{"type":"add_ellipse","data":{"name":"x","origin":[0,0],"width":0.3,"height":0.4,"extra":1}}"#,
        r#"{"type":"add_ellipse","data":{"name":"x","origin":[0],"width":0.3,"height":0.4}}"#,
        r#"{"type":"add_ellipse","data":{"name":"x","origin":[0,0],"width":1e400,"height":0.4}}"#,
        r#"{"type":"add_ellipse","data":{"name":"x","origin":[0,0],"width":0.3,"width":0.4,"height":0.4}}"#,
    ] {
        assert!(
            serde_json::from_str::<Command>(raw).is_err(),
            "accepted {raw}"
        );
    }
    for next_id in [u64::MAX, u64::MAX - 4, u64::MAX - 8] {
        let mut value = serde_json::to_value(Project::default()).unwrap();
        value["next_id"] = next_id.into();
        let mut editor = Editor::new(serde_json::from_value(value).unwrap()).unwrap();
        let before = editor.project.clone();
        assert!(editor.execute(ellipse([0., 0.], 0.3, 0.4)).is_err());
        assert_eq!(editor.project, before);
        assert!(!editor.can_undo() && !editor.can_redo());
    }
    let mut project = Project::default();
    project.materials = (1..=MAX_OBJECTS as u64)
        .map(|id| Material {
            id,
            ..Material::default()
        })
        .collect();
    let mut value = serde_json::to_value(project).unwrap();
    value["next_id"] = (MAX_OBJECTS as u64 + 1).into();
    let mut editor = Editor::new(serde_json::from_value(value).unwrap()).unwrap();
    let before = editor.project.clone();
    assert!(
        editor
            .execute(ellipse([0., 0.], 0.3, 0.4))
            .unwrap_err()
            .0
            .contains("object limit")
    );
    assert_eq!(editor.project, before);
    assert!(!editor.can_undo() && !editor.can_redo());
}

#[test]
fn ellipse_boundaries_keep_authored_ids_and_sewing_under_remeshing() {
    let mut editor = Editor::default();
    for x in [0., 1.] {
        editor.execute(ellipse([x, 0.], 0.3, 0.4)).unwrap();
    }
    let boundaries: Vec<_> = editor
        .project
        .pieces
        .iter()
        .map(|p| p.boundary.clone())
        .collect();
    editor
        .execute(Command::AddSeam {
            name: "Original partial cubic seam".into(),
            a: CurveInterval {
                curve: boundaries[0][0],
                t0: 0.2,
                t1: 0.85,
            },
            b: CurveInterval {
                curve: boundaries[1][0],
                t0: 0.2,
                t1: 0.85,
            },
            correspondence: vec![[0., 0.], [1., 1.]],
        })
        .unwrap();
    editor
        .execute(Command::AddSewingGroup {
            name: "Original two-quarter group".into(),
            a: boundaries[0][1..3]
                .iter()
                .copied()
                .map(CurveInterval::full)
                .collect(),
            b: boundaries[1][1..3]
                .iter()
                .copied()
                .map(CurveInterval::full)
                .collect(),
            correspondence: vec![[0., 0.], [1., 1.]],
        })
        .unwrap();
    let original = editor.project.clone();
    let geometry = original
        .sewing_group_geometry(original.sewing_groups[0].id)
        .unwrap();
    let mut counts = Vec::new();
    for metres in [0.08, 0.025] {
        editor.execute(Command::SetResolution { metres }).unwrap();
        let project = &editor.project;
        assert_eq!(project.points, original.points);
        assert_eq!(project.curves, original.curves);
        assert_eq!(project.pieces, original.pieces);
        assert_eq!(project.seams, original.seams);
        assert_eq!(project.sewing_groups, original.sewing_groups);
        let mesh = project.mesh().unwrap();
        mesh.validate().unwrap();
        assert_eq!(mesh.doc_revision, project.revision);
        assert_eq!(mesh.sewing_groups, std::slice::from_ref(&geometry));
        for sample in &mesh.boundary {
            close(
                mesh.uv[sample.vertex],
                project
                    .curve(sample.curve)
                    .unwrap()
                    .evaluate(project, sample.t)
                    .unwrap(),
            );
            assert!(
                project
                    .piece(sample.piece)
                    .unwrap()
                    .boundary
                    .contains(&sample.curve)
            );
        }
        for seam in [original.seams[0].id, original.sewing_groups[0].id] {
            let stitches: Vec<_> = mesh.stitches.iter().filter(|s| s.seam == seam).collect();
            assert!(stitches.len() > 2);
            for stitch in stitches {
                let a = stitch.a.position(&mesh.positions);
                let b = stitch.b.position(&mesh.positions);
                assert!((b[0] - a[0] - 1.).hypot(b[1] - a[1]).hypot(b[2] - a[2]) < 1e-10);
            }
        }
        counts.push(mesh.positions.len());
    }
    assert!(counts[1] > counts[0]);
}
