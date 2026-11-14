use weftforge_core::{Command, CurveInterval, Editor, Project};

const TOLERANCE: f64 = 1.0e-12;
const PARENT: [[f64; 2]; 4] = [[0.7, 0.5], [0.58, 0.78], [0.11, 0.63], [0.0, 0.5]];

// Independent power-basis restriction, rather than the command's de Casteljau reconstruction.
fn restriction(p: [[f64; 2]; 4], lo: f64, hi: f64) -> [[f64; 2]; 4] {
    let coefficients: [[f64; 2]; 4] = [
        p[0],
        std::array::from_fn(|i| 3.0 * (p[1][i] - p[0][i])),
        std::array::from_fn(|i| 3.0 * (p[0][i] - 2.0 * p[1][i] + p[2][i])),
        std::array::from_fn(|i| p[3][i] - p[0][i] + 3.0 * (p[1][i] - p[2][i])),
    ];
    let value = |t: f64| {
        std::array::from_fn(|i| {
            coefficients[0][i]
                + t * (coefficients[1][i] + t * (coefficients[2][i] + t * coefficients[3][i]))
        })
    };
    let start = value(lo);
    let h = hi - lo;
    [
        start,
        std::array::from_fn(|i| {
            start[i]
                + h * (coefficients[1][i]
                    + 2.0 * lo * coefficients[2][i]
                    + 3.0 * lo * lo * coefficients[3][i])
                    / 3.0
        }),
        std::array::from_fn(|i| {
            start[i]
                + 2.0
                    * h
                    * (coefficients[1][i]
                        + 2.0 * lo * coefficients[2][i]
                        + 3.0 * lo * lo * coefficients[3][i])
                    / 3.0
                + h * h * (coefficients[2][i] + 3.0 * lo * coefficients[3][i]) / 3.0
        }),
        value(hi),
    ]
}

fn children(parent: [[f64; 2]; 4], t: f64, straight: bool) -> (Editor, u64, u64) {
    let left = restriction(parent, 0.0, t);
    let right = restriction(parent, t, 1.0);
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Independently restricted panel".into(),
            vertices: vec![
                [0.0, 0.0],
                [parent[0][0], 0.0],
                parent[0],
                left[3],
                parent[3],
            ],
        })
        .unwrap();
    let [a, b] = [
        editor.project.pieces[0].boundary[2],
        editor.project.pieces[0].boundary[3],
    ];
    if !straight {
        editor
            .execute(Command::Transaction(vec![
                Command::SetCurveControls {
                    curve: a,
                    controls: Some([left[1], left[2]]),
                },
                Command::SetCurveControls {
                    curve: b,
                    controls: Some([right[1], right[2]]),
                },
            ]))
            .unwrap();
    }
    (editor, a, b)
}

fn polygon(project: &Project, curve: u64) -> [[f64; 2]; 4] {
    let curve = project.curve(curve).unwrap();
    let a = project.point(curve.start).unwrap().position;
    let b = project.point(curve.end).unwrap().position;
    let [c, d] = curve.controls.unwrap();
    [a, c, d, b]
}

fn close(a: [f64; 2], b: [f64; 2]) {
    assert!(
        (a[0] - b[0]).hypot(a[1] - b[1]) <= TOLERANCE,
        "{a:?} != {b:?}"
    );
}

fn assert_same_pairs(before: &Project, after: &Project) {
    assert_eq!(before.seams.len(), after.seams.len());
    for (old, new) in before.seams.iter().zip(&after.seams) {
        assert_eq!(
            (old.id, &old.name, &old.correspondence),
            (new.id, &new.name, &new.correspondence)
        );
        for u in (0..=100)
            .map(|i| f64::from(i) / 100.0)
            .chain(old.correspondence.iter().map(|pair| pair[0]))
        {
            let pair = old
                .correspondence
                .windows(2)
                .find(|pair| u <= pair[1][0])
                .unwrap();
            let v = pair[0][1]
                + (u - pair[0][0]) / (pair[1][0] - pair[0][0]) * (pair[1][1] - pair[0][1]);
            for (a, b, fraction) in [(old.a, new.a, u), (old.b, new.b, v)] {
                close(
                    before
                        .curve(a.curve)
                        .unwrap()
                        .evaluate(before, a.parameter(fraction))
                        .unwrap(),
                    after
                        .curve(b.curve)
                        .unwrap()
                        .evaluate(after, b.parameter(fraction))
                        .unwrap(),
                );
            }
        }
    }
}

#[test]
fn join_recovers_analytic_cubic_children_including_stationary_derivatives_and_wrap() {
    for parent in [
        PARENT,
        [[0.7, 0.5], [0.0, 0.5], [0.7, 0.5], [0.0, 0.5]],
        [[0.7, 0.5], [0.0, 0.7], [0.7, 0.7], [0.0, 0.5]],
    ] {
        for t in [0.04, 0.3, 0.5, 0.91] {
            let (mut editor, left, right) = children(parent, t, false);
            editor.project.pieces[0].boundary.rotate_left(3);
            editor.project.validate().unwrap();
            assert_eq!(*editor.project.pieces[0].boundary.last().unwrap(), left);
            assert_eq!(editor.project.pieces[0].boundary[0], right);
            let before = editor.project.clone();
            editor.execute(Command::JoinCurve { curve: left }).unwrap();
            assert_eq!(editor.project.revision, before.revision + 1);
            let reconstructed = polygon(&editor.project, left);
            for (actual, expected) in reconstructed.into_iter().zip(parent) {
                close(actual, expected);
            }
            // The control polygon of the difference bounds the full curve, including unsampled points.
            for (id, lo, hi) in [(left, 0.0, t), (right, t, 1.0)] {
                for (a, b) in
                    polygon(&before, id)
                        .into_iter()
                        .zip(restriction(reconstructed, lo, hi))
                {
                    close(a, b);
                }
            }
            assert!(
                editor
                    .project
                    .point(before.curve(left).unwrap().end)
                    .is_none()
            );
            assert!(editor.project.curve(right).is_none());
            assert_eq!(editor.project.materials, before.materials);
            for curve in before
                .curves
                .iter()
                .filter(|curve| curve.id != left && curve.id != right)
            {
                assert_eq!(editor.project.curve(curve.id), Some(curve));
            }
            assert!(editor.undo());
            assert_eq!(editor.project.curves, before.curves);
            assert_eq!(editor.project.points, before.points);
            assert!(editor.redo());
            assert_eq!(polygon(&editor.project, left), reconstructed);
            editor
                .execute(Command::SplitCurve { curve: left, t })
                .unwrap();
            assert!(editor.project.curves.last().unwrap().id > right);
            assert!(editor.project.points.last().unwrap().id > before.curve(left).unwrap().end);
        }
    }
}

#[test]
fn join_preserves_both_children_sewn_pairs_in_all_orientations_and_v2_remesh() {
    for straight in [false, true] {
        for source in 0..2 {
            for reverse_a in [false, true] {
                for reverse_b in [false, true] {
                    let parent = if straight {
                        [
                            [0.7, 0.5],
                            [0.7 * 2.0 / 3.0, 0.5],
                            [0.7 / 3.0, 0.5],
                            [0.0, 0.5],
                        ]
                    } else {
                        PARENT
                    };
                    let (mut editor, left, right) = children(parent, 0.4, straight);
                    editor
                        .execute(Command::AddRectangle {
                            name: "Paired panel".into(),
                            origin: [1.0, 0.0],
                            width: 0.5,
                            height: 0.5,
                        })
                        .unwrap();
                    let other = editor.project.pieces[1].boundary[2];
                    for (index, curve) in [left, right].into_iter().enumerate() {
                        let mut intervals = [
                            CurveInterval {
                                curve,
                                t0: 0.1,
                                t1: 0.95,
                            },
                            CurveInterval {
                                curve: other,
                                t0: 0.05,
                                t1: 0.9,
                            },
                        ];
                        if source == 1 {
                            intervals.swap(0, 1);
                        }
                        for (interval, reverse) in intervals.iter_mut().zip([reverse_a, reverse_b])
                        {
                            if reverse {
                                std::mem::swap(&mut interval.t0, &mut interval.t1);
                            }
                        }
                        editor
                            .execute(Command::AddSeam {
                                name: format!("Child {index}"),
                                a: intervals[0],
                                b: intervals[1],
                                correspondence: vec![
                                    [0.0, 0.0],
                                    [0.3, 0.6],
                                    [0.8, 0.9],
                                    [1.0, 1.0],
                                ],
                            })
                            .unwrap();
                    }
                    let before = editor.project.clone();
                    editor.execute(Command::JoinCurve { curve: left }).unwrap();
                    assert_same_pairs(&before, &editor.project);
                    assert_eq!(editor.project.pieces[1], before.pieces[1]);
                    assert_eq!(editor.project.materials, before.materials);
                    if !straight && source == 0 && !reverse_a && !reverse_b {
                        let joined = editor.project.clone();
                        let mut vertices = 0;
                        for metres in [0.09, 0.035] {
                            editor.execute(Command::SetResolution { metres }).unwrap();
                            let mesh = editor.project.mesh().unwrap();
                            mesh.validate().unwrap();
                            assert!(mesh.positions.len() > vertices);
                            vertices = mesh.positions.len();
                            assert_eq!(editor.project.seams, joined.seams);
                        }
                        let json = editor.project.to_json().unwrap();
                        assert_eq!(
                            serde_json::from_slice::<serde_json::Value>(&json).unwrap()["version"],
                            2
                        );
                        let reopened = Project::from_json(&json).unwrap();
                        assert_eq!(reopened, editor.project);
                        assert_same_pairs(&before, &reopened);
                    }
                }
            }
        }
    }
}

fn rejects_with_redo(mut editor: Editor, curve: u64, message: &str) {
    editor
        .execute(Command::SetResolution { metres: 0.08 })
        .unwrap();
    assert!(editor.undo());
    let before = editor.project.clone();
    let error = editor.execute(Command::JoinCurve { curve }).unwrap_err();
    assert!(error.to_string().contains(message), "{error}");
    assert_eq!(editor.project, before);
    assert!(editor.can_redo());
    assert!(editor.redo());
    assert_eq!(editor.project.resolution, 0.08);
}

#[test]
fn incompatible_midpoint_invisible_cubics_mixed_controls_and_corners_reject_atomically() {
    let (mut editor, left, _) = children(PARENT, 0.4, false);
    let original_midpoint = editor
        .project
        .curve(left)
        .unwrap()
        .evaluate(&editor.project, 0.5)
        .unwrap();
    let mut controls = editor.project.curve(left).unwrap().controls.unwrap();
    controls[0][1] += 1.0e-8;
    controls[1][1] -= 1.0e-8;
    editor
        .execute(Command::SetCurveControls {
            curve: left,
            controls: Some(controls),
        })
        .unwrap();
    close(
        original_midpoint,
        editor
            .project
            .curve(left)
            .unwrap()
            .evaluate(&editor.project, 0.5)
            .unwrap(),
    );
    rejects_with_redo(editor, left, "compatible");
    let (mut editor, left, _) = children(PARENT, 0.4, false);
    editor
        .execute(Command::SetCurveControls {
            curve: left,
            controls: None,
        })
        .unwrap();
    rejects_with_redo(editor, left, "two straight");
    let (editor, _, right) = children(PARENT, 0.4, true);
    rejects_with_redo(editor, right, "collinear");
    let (editor, left, _) = children(PARENT, 0.4, false);
    rejects_with_redo(editor, left + 10000, "no piece");
}

#[test]
fn join_rejects_self_seams_tiny_remapped_intervals_and_failing_transactions() {
    let (mut editor, left, right) = children(PARENT, 0.4, false);
    editor
        .execute(Command::AddSeam {
            name: "Adjacent self sewing".into(),
            a: CurveInterval::full(left),
            b: CurveInterval::reversed(right),
            correspondence: vec![[0.0, 0.0], [1.0, 1.0]],
        })
        .unwrap();
    rejects_with_redo(editor, left, "distinct curves");
    let (mut editor, left, _) = children(PARENT, 0.4, false);
    let other = editor.project.pieces[0].boundary[0];
    editor
        .execute(Command::AddSeam {
            name: "Too small after compression".into(),
            a: CurveInterval {
                curve: left,
                t0: 0.2,
                t1: 0.2 + 2.0e-8,
            },
            b: CurveInterval::full(other),
            correspondence: vec![[0.0, 0.0], [1.0, 1.0]],
        })
        .unwrap();
    rejects_with_redo(editor, left, "distinct endpoints");
    let (mut editor, left, _) = children(PARENT, 0.4, false);
    let before = editor.project.clone();
    assert!(
        editor
            .execute(Command::Transaction(vec![
                Command::JoinCurve { curve: left },
                Command::SetResolution { metres: 0.0 }
            ]))
            .is_err()
    );
    assert_eq!(editor.project, before);
    editor
        .execute(Command::AddPolygon {
            name: "Triangle".into(),
            vertices: vec![[2.0, 0.0], [2.5, 0.0], [2.0, 0.5]],
        })
        .unwrap();
    let curve = editor.project.pieces.last().unwrap().boundary[0];
    rejects_with_redo(editor, curve, "at least three");
}

#[test]
fn join_rejects_numerically_ambiguous_or_extreme_parameter_reconstruction() {
    let small = [
        [0.7, 0.5],
        [0.6999983, 0.500004],
        [0.6999915, 0.500002],
        [0.69999, 0.5],
    ];
    let (mut editor, left, _) = children(small, 0.4, false);
    let mut controls = editor.project.curve(left).unwrap().controls.unwrap();
    controls[0][1] += 1.0e-14;
    editor
        .execute(Command::SetCurveControls {
            curve: left,
            controls: Some(controls),
        })
        .unwrap();
    rejects_with_redo(editor, left, "ambiguous");
    let (editor, left, _) = children(PARENT, 5.0e-7, false);
    rejects_with_redo(editor, left, "1e-6 < t");
}

#[test]
fn join_rejects_a_full_domain_roundoff_error_above_the_unchanged_tolerance() {
    let [a, m, b] = [
        [913.1126541019662, 0.5],
        [968.7824350386107, 0.500000000001],
        [977.3265791058585, 0.5],
    ];
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Floating-point boundary witness".into(),
            vertices: vec![a, m, b, [b[0], 0.8], [a[0], 0.8]],
        })
        .unwrap();
    let curve = editor.project.pieces[0].boundary[0];
    let right = editor.project.pieces[0].boundary[1];
    let mut candidate = editor.project.curve(curve).unwrap().clone();
    candidate.end = editor.project.curve(right).unwrap().end;
    let length = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
    let t = length(a, m) / (length(a, m) + length(m, b));
    assert!(length(candidate.evaluate(&editor.project, t).unwrap(), m) <= TOLERANCE);
    let old = editor
        .project
        .curve(curve)
        .unwrap()
        .evaluate(&editor.project, 0.9999)
        .unwrap();
    let joined = candidate.evaluate(&editor.project, t * 0.9999).unwrap();
    assert!(length(old, joined) > TOLERANCE, "{old:?} versus {joined:?}");
    rejects_with_redo(editor, curve, "1e-12");
}
