use serde_json::json;
use std::time::Instant;
use weftforge_core::{
    Command, CurveInterval, Editor, Mesh, Project, SewingDirection, SewingGroup,
    SewingSideByLength, WeightedVertex,
};

fn interval(curve: u64, t0: f64, t1: f64) -> CurveInterval {
    CurveInterval { curve, t0, t1 }
}

fn line_scene() -> (Editor, Vec<u64>) {
    let mut editor = Editor::default();
    let mut curves = Vec::new();
    for (index, width) in [0.1, 0.3, 0.2, 0.1, 0.3].into_iter().enumerate() {
        editor
            .execute(Command::AddRectangle {
                name: format!("Original panel {index}"),
                origin: [index as f64 * 0.5, 0.],
                width,
                height: 0.12,
            })
            .unwrap();
        curves.push(editor.project.pieces.last().unwrap().boundary[0]);
    }
    editor
        .execute(Command::AddSewingGroup {
            name: "Original disconnected 2:3".into(),
            a: curves[..2]
                .iter()
                .copied()
                .map(CurveInterval::full)
                .collect(),
            b: curves[2..]
                .iter()
                .copied()
                .map(CurveInterval::full)
                .collect(),
            correspondence: vec![[0., 0.], [1., 1.]],
        })
        .unwrap();
    (editor, curves)
}

fn parameter(mesh: &Mesh, curve: u64, endpoint: WeightedVertex) -> Option<f64> {
    let mut values = [0.; 2];
    for (index, vertex) in endpoint.vertices.into_iter().enumerate() {
        values[index] = mesh
            .boundary
            .iter()
            .find(|sample| sample.curve == curve && sample.vertex == vertex)?
            .t;
    }
    Some(values[0] * (1. - endpoint.weight) + values[1] * endpoint.weight)
}

fn point(project: &Project, interval: CurveInterval, end: bool) -> [f64; 2] {
    project
        .curve(interval.curve)
        .unwrap()
        .evaluate(project, if end { interval.t1 } else { interval.t0 })
        .unwrap()
}

fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 1.0e-8, "{a} != {b}");
}

fn close_point(a: [f64; 2], b: [f64; 2]) {
    assert!((a[0] - b[0]).hypot(a[1] - b[1]) <= 1.0e-8, "{a:?} != {b:?}");
}

#[test]
fn disconnected_two_to_three_keeps_both_one_sided_knots_and_real_constraints() {
    let (editor, curves) = line_scene();
    let original = editor.project.clone();
    let group = &original.sewing_groups[0];
    let geometry = original.sewing_group_geometry(group.id).unwrap();
    close(geometry.lengths[0], 0.4);
    close(geometry.lengths[1], 0.6);
    assert_eq!(geometry.portions.len(), 4);
    let knot = &geometry.portions[..2];
    assert_eq!(knot[0].a.curve, curves[0]);
    assert_eq!(knot[1].a.curve, curves[1]);
    assert_eq!(knot[0].a.t1, 1.);
    assert_eq!(knot[1].a.t0, 0.);
    close(knot[0].b.t1, 0.75);
    close(knot[1].b.t0, 0.75);
    assert_ne!(
        point(&original, knot[0].a, true),
        point(&original, knot[1].a, false)
    );
    for resolution in [0.09, 0.035] {
        let mut project = original.clone();
        project.resolution = resolution;
        let mesh = project.mesh().unwrap();
        assert_eq!(mesh.sewing_groups, vec![geometry.clone()]);
        assert!(mesh.stitches.iter().all(|stitch| stitch.seam == group.id));
        for (curve, expected) in [(curves[0], 1.), (curves[1], 0.)] {
            assert!(
                mesh.stitches.iter().any(|stitch| {
                    parameter(&mesh, curve, stitch.a).is_some_and(|t| (t - expected).abs() < 1e-12)
                        && parameter(&mesh, curves[2], stitch.b)
                            .is_some_and(|t| (t - 0.75).abs() < 1e-12)
                }),
                "Missing one-sided disconnected knot on curve {curve}"
            );
        }
        for stitch in &mesh.stitches {
            for endpoint in [stitch.a, stitch.b] {
                assert_eq!(
                    mesh.piece_ids[endpoint.vertices[0]],
                    mesh.piece_ids[endpoint.vertices[1]]
                );
            }
        }
        weftforge_core::simulation::create_cloth(&project, &mesh).unwrap();
    }
    assert_eq!(editor.project, original);
}

fn curved_scene(reverse: bool) -> (Editor, u64, u64) {
    let mut editor = Editor::default();
    for (index, width) in [0.4, 0.6].into_iter().enumerate() {
        editor
            .execute(Command::AddRectangle {
                name: format!("Curve panel {index}"),
                origin: [index as f64, 0.],
                width,
                height: 0.3,
            })
            .unwrap();
    }
    let a = editor.project.pieces[0].boundary[2];
    let b = editor.project.pieces[1].boundary[0];
    editor
        .execute(Command::SetCurveControls {
            curve: a,
            controls: Some([[0.38, 0.48], [0.06, 0.36]]),
        })
        .unwrap();
    let start = Instant::now();
    editor
        .execute(Command::AddSewingGroup {
            name: "Partial nonlinear original".into(),
            a: vec![interval(
                a,
                if reverse { 0.92 } else { 0.12 },
                if reverse { 0.12 } else { 0.92 },
            )],
            b: vec![interval(b, 0.9, 0.05)],
            correspondence: vec![[0., 0.], [0.3, 0.72], [1., 1.]],
        })
        .unwrap();
    eprintln!("curved group committed validation: {:?}", start.elapsed());
    (editor, a, b)
}

// Independent fixed composite Simpson integration of the original polynomial.
fn oracle_length(project: &Project, interval: CurveInterval, t: f64) -> f64 {
    let curve = project.curve(interval.curve).unwrap();
    let a = project.point(curve.start).unwrap().position;
    let b = project.point(curve.end).unwrap().position;
    let speed = |t: f64| {
        let derivative: [f64; 2] = if let Some([c, d]) = curve.controls {
            std::array::from_fn(|axis| {
                3. * ((1. - t).powi(2) * (c[axis] - a[axis])
                    + 2. * (1. - t) * t * (d[axis] - c[axis])
                    + t * t * (b[axis] - d[axis]))
            })
        } else {
            [b[0] - a[0], b[1] - a[1]]
        };
        derivative[0].hypot(derivative[1])
    };
    const STEPS: u32 = 2048;
    let step = (t - interval.t0) / f64::from(STEPS);
    let sum = speed(interval.t0)
        + speed(t)
        + (1..STEPS)
            .map(|i| speed(interval.t0 + step * f64::from(i)) * if i % 2 == 0 { 2. } else { 4. })
            .sum::<f64>();
    (sum * step / 3.).abs()
}

fn oracle_parameter(project: &Project, interval: CurveInterval, fraction: f64) -> f64 {
    let target = oracle_length(project, interval, interval.t1) * fraction;
    let mut lo = 0.;
    let mut hi = 1.;
    for _ in 0..50 {
        let mid = (lo + hi) * 0.5;
        if oracle_length(project, interval, interval.parameter(mid)) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    interval.parameter((lo + hi) * 0.5)
}

fn paired(u: f64) -> f64 {
    if u <= 0.3 {
        u * 2.4
    } else {
        0.72 + (u - 0.3) * (0.28 / 0.7)
    }
}

fn analytic_scene() -> (Editor, u64) {
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Original analytic parabola".into(),
            vertices: vec![[0., 0.], [1., 1.], [0., 1.]],
        })
        .unwrap();
    let curve = editor.project.pieces[0].boundary[0];
    editor
        .execute(Command::SetCurveControls {
            curve,
            controls: Some([[1. / 3., 0.], [2. / 3., 1. / 3.]]),
        })
        .unwrap();
    (editor, curve)
}

#[test]
fn authored_interval_lengths_match_straight_and_analytic_cubic_primitives() {
    let mut straight = Editor::default();
    straight
        .execute(Command::AddPolygon {
            name: "Original 3-4-5 edge".into(),
            vertices: vec![[0., 0.], [0.3, 0.4], [0., 0.4]],
        })
        .unwrap();
    let line = straight.project.pieces[0].boundary[0];
    for (t0, t1) in [(0., 1.), (0.2, 0.8), (0.8, 0.2)] {
        close(
            straight
                .project
                .interval_length(interval(line, t0, t1))
                .unwrap(),
            0.5 * (t1 - t0).abs(),
        );
    }
    let (mut editor, curve) = analytic_scene();
    let primitive = |t: f64| t * (1. + 4. * t * t).sqrt() / 2. + (2. * t).asinh() / 4.;
    for (t0, t1) in [(0., 1.), (0.2, 0.8), (0.8, 0.2), (0., 0.37), (0.37, 1.)] {
        close(
            editor
                .project
                .interval_length(interval(curve, t0, t1))
                .unwrap(),
            (primitive(t1) - primitive(t0)).abs(),
        );
    }
    close(
        editor
            .project
            .interval_length(interval(curve, 0., 0.37))
            .unwrap()
            + editor
                .project
                .interval_length(interval(curve, 0.37, 1.))
                .unwrap(),
        primitive(1.),
    );
    let original = editor.project.clone();
    editor
        .execute(Command::SetCurveControls {
            curve,
            controls: None,
        })
        .unwrap();
    let changed_revision = editor.project.revision;
    close(
        editor
            .project
            .interval_length(CurveInterval::full(curve))
            .unwrap(),
        2_f64.sqrt(),
    );
    assert!(editor.undo());
    assert!(editor.project.revision > changed_revision);
    close(
        editor
            .project
            .interval_length(CurveInterval::full(curve))
            .unwrap(),
        primitive(1.),
    );
    let undo_revision = editor.project.revision;
    assert!(editor.redo());
    assert!(editor.project.revision > undo_revision);
    close(
        editor
            .project
            .interval_length(CurveInterval::full(curve))
            .unwrap(),
        2_f64.sqrt(),
    );
    let mut replacement = original;
    replacement
        .curves
        .iter_mut()
        .find(|item| item.id == curve)
        .unwrap()
        .controls = None;
    close(
        replacement
            .interval_length(CurveInterval::full(curve))
            .unwrap(),
        2_f64.sqrt(),
    );
}

#[test]
fn legacy_lengths_share_accurate_raw_metrics_without_changing_correspondence() {
    let (mut editor, _, _) = curved_scene(true);
    let group = editor.project.sewing_groups[0].clone();
    editor
        .execute(Command::AddSeam {
            name: "Original raw legacy pair".into(),
            a: group.a[0],
            b: group.b[0],
            correspondence: group.correspondence.clone(),
        })
        .unwrap();
    let seam = editor.project.seams[0].clone();
    let expected = [
        oracle_length(&editor.project, seam.a, seam.a.t1),
        oracle_length(&editor.project, seam.b, seam.b.t1),
    ];
    let lengths = editor.project.seam_lengths(seam.id).unwrap();
    for (actual, expected) in lengths.into_iter().zip(expected) {
        close(actual, expected);
    }
    assert!(((lengths[1] - lengths[0]) - (expected[1] - expected[0])).abs() <= 2e-8);
    for (actual, expected) in editor
        .project
        .sewing_group_geometry(group.id)
        .unwrap()
        .lengths
        .into_iter()
        .zip(lengths)
    {
        close(actual, expected);
    }
    let mut changed_seam = seam.clone();
    changed_seam.correspondence = vec![[0., 0.], [0.4, 0.8], [1., 1.]];
    let mut changed_group = group.clone();
    changed_group.correspondence = changed_seam.correspondence.clone();
    editor
        .execute(Command::Transaction(vec![
            Command::UpdateSeam { seam: changed_seam },
            Command::UpdateSewingGroup {
                group: changed_group,
            },
        ]))
        .unwrap();
    assert_eq!(editor.project.seam_lengths(seam.id).unwrap(), lengths);
    for (actual, expected) in editor
        .project
        .sewing_group_geometry(group.id)
        .unwrap()
        .lengths
        .into_iter()
        .zip(lengths)
    {
        close(actual, expected);
    }
    assert_eq!(editor.project.seams[0].a, seam.a);
    assert_eq!(editor.project.sewing_groups[0].a, group.a);
    assert_eq!(editor.project.sewing_groups[0].b, group.b);
}

#[test]
fn length_queries_are_read_only_validate_intervals_and_ignore_display_state() {
    let (mut editor, curves) = line_scene();
    editor
        .execute(Command::SetResolution { metres: 0.04 })
        .unwrap();
    assert!(editor.undo());
    let unchanged = format!("{editor:?}");
    let bytes = editor.project.to_json().unwrap();
    let mesh = editor.project.mesh().unwrap();
    let mut cloth = weftforge_core::simulation::create_cloth(&editor.project, &mesh).unwrap();
    cloth
        .step(weftforge_sim::StepSettings::default(), || false)
        .unwrap();
    let positions = cloth.positions().to_vec();
    let time = cloth.simulation_time();
    let curve = curves[0];
    let group = editor.project.sewing_groups[0].id;
    for resolution in [0.09, 0.025] {
        let mut project = editor.project.clone();
        project.resolution = resolution;
        project.pieces[0].placement.translation = [1., 2., 3.];
        project.pieces[0].placement.rotation = [0.1, 0.2, 0.3];
        close(
            project.interval_length(CurveInterval::full(curve)).unwrap(),
            0.1,
        );
        for (actual, expected) in project
            .sewing_group_geometry(group)
            .unwrap()
            .lengths
            .into_iter()
            .zip([0.4, 0.6])
        {
            close(actual, expected);
        }
    }
    for (t0, t1) in [
        (0.2, 0.2),
        (0., 1e-8),
        (0., 1e-9),
        (-0.1, 1.),
        (0., 1.1),
        (f64::NAN, 1.),
        (0., f64::INFINITY),
    ] {
        assert!(
            editor
                .project
                .interval_length(interval(curve, t0, t1))
                .is_err()
        );
    }
    assert!(
        editor
            .project
            .interval_length(CurveInterval::full(u64::MAX))
            .is_err()
    );
    assert!(editor.project.seam_lengths(u64::MAX).is_err());
    assert!(editor.project.sewing_group_geometry(u64::MAX).is_err());
    let mut invalid = editor.project.sewing_groups[0].clone();
    invalid.a[0].t1 = invalid.a[0].t0;
    assert!(
        editor
            .execute(Command::UpdateSewingGroup { group: invalid })
            .is_err()
    );
    close(
        editor
            .project
            .interval_length(interval(curve, 0.8, 0.2))
            .unwrap(),
        0.06,
    );
    assert_eq!(format!("{editor:?}"), unchanged);
    assert_eq!(editor.project.to_json().unwrap(), bytes);
    assert!(editor.can_redo());
    assert_eq!(cloth.positions(), positions);
    assert_eq!(cloth.simulation_time(), time);
    assert_eq!(mesh, editor.project.mesh().unwrap());
}

#[test]
fn asymmetric_cubic_metric_and_nonlinear_pairs_match_independent_integrals() {
    for reverse in [false, true] {
        let (editor, a, b) = curved_scene(reverse);
        let original = editor.project.clone();
        let group = &original.sewing_groups[0];
        let start = Instant::now();
        let geometry = original.sewing_group_geometry(group.id).unwrap();
        eprintln!("curved group geometry query: {:?}", start.elapsed());
        close(
            geometry.lengths[0],
            oracle_length(&original, group.a[0], group.a[0].t1),
        );
        close(geometry.lengths[1], 0.51);
        let at = oracle_parameter(&original, group.a[0], 0.3);
        close_point(
            point(&original, geometry.portions[0].a, true),
            original.curve(a).unwrap().evaluate(&original, at).unwrap(),
        );
        close(geometry.portions[0].b.t1, group.b[0].parameter(0.72));
        for resolution in [0.09, 0.035] {
            let mut project = original.clone();
            project.resolution = resolution;
            let start = Instant::now();
            let mesh = project.mesh().unwrap();
            eprintln!(
                "curved group mesh {resolution} m: {:?}, {} stitches",
                start.elapsed(),
                mesh.stitches.len()
            );
            assert_eq!(mesh.sewing_groups, vec![geometry.clone()]);
            for stitch in &mesh.stitches {
                let at = parameter(&mesh, a, stitch.a).unwrap();
                let bt = parameter(&mesh, b, stitch.b).unwrap();
                let u = oracle_length(&original, group.a[0], at) / geometry.lengths[0];
                close(
                    (bt - group.b[0].t0) / (group.b[0].t1 - group.b[0].t0) * geometry.lengths[1],
                    paired(u) * geometry.lengths[1],
                );
            }
        }
    }
}

#[test]
fn collinear_nonuniform_cubic_uses_physical_length_in_both_directions() {
    let (mut editor, curves) = line_scene();
    let a = curves[1];
    editor
        .execute(Command::SetCurveControls {
            curve: a,
            controls: Some([[0.5, 0.], [0.51, 0.]]),
        })
        .unwrap();
    for (t0, t1) in [(0., 1.), (1., 0.), (0.12, 0.91), (0.91, 0.12)] {
        let interval = interval(a, t0, t1);
        let length = editor.project.interval_length(interval).unwrap();
        close(
            length,
            distance_points(
                point(&editor.project, interval, false),
                point(&editor.project, interval, true),
            ),
        );
        assert_eq!(
            editor
                .project
                .interval_parameter_at_length(interval, 0.)
                .unwrap(),
            t0
        );
        assert_eq!(
            editor
                .project
                .interval_parameter_at_length(interval, length)
                .unwrap(),
            t1
        );
        let t = editor
            .project
            .interval_parameter_at_length(interval, length * 0.3)
            .unwrap();
        let expected = oracle_parameter(&editor.project, interval, 0.3);
        close_point(
            editor
                .project
                .curve(a)
                .unwrap()
                .evaluate(&editor.project, t)
                .unwrap(),
            editor
                .project
                .curve(a)
                .unwrap()
                .evaluate(&editor.project, expected)
                .unwrap(),
        );
        assert!((t - interval.parameter(0.3)).abs() > 0.03);
    }
}

fn distance_points(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

#[test]
fn split_join_partial_cubics_keep_group_identity_maps_and_pair_locations() {
    for reverse in [false, true] {
        for source in 0..2 {
            let (mut editor, a, b) = curved_scene(reverse);
            let original = editor.project.clone();
            let original_group = original.sewing_groups[0].clone();
            let curve = [a, b][source];
            editor
                .execute(Command::SplitCurve { curve, t: 0.45 })
                .unwrap();
            let second = editor.project.curves.last().unwrap().id;
            let split = editor.project.clone();
            let group = &split.sewing_groups[0];
            assert_eq!(
                (group.id, &group.name, &group.correspondence),
                (
                    original_group.id,
                    &original_group.name,
                    &original_group.correspondence
                )
            );
            assert_eq!([group.a.len(), group.b.len()][source], 2);
            let source_side = [&group.a, &group.b][source];
            let reversed = [original_group.a[0], original_group.b[0]][source].t0
                > [original_group.a[0], original_group.b[0]][source].t1;
            assert_eq!(source_side[0].curve, if reversed { second } else { curve });
            for portion in split.sewing_group_geometry(group.id).unwrap().portions {
                for end in [false, true] {
                    let remap = |interval: CurveInterval| {
                        let t = if end { interval.t1 } else { interval.t0 };
                        if interval.curve == curve {
                            t * 0.45
                        } else if interval.curve == second {
                            0.45 + t * 0.55
                        } else {
                            t
                        }
                    };
                    let at = remap(portion.a);
                    let bt = remap(portion.b);
                    let u = oracle_length(&original, original_group.a[0], at)
                        / oracle_length(&original, original_group.a[0], original_group.a[0].t1);
                    let v = oracle_length(&original, original_group.b[0], bt)
                        / oracle_length(&original, original_group.b[0], original_group.b[0].t1);
                    close(v * 0.51, paired(u) * 0.51);
                }
            }
            editor.execute(Command::JoinCurve { curve }).unwrap();
            let joined = editor.project.clone();
            assert_eq!(joined.sewing_groups[0].id, original_group.id);
            assert_eq!(
                [
                    joined.sewing_groups[0].a.len(),
                    joined.sewing_groups[0].b.len()
                ][source],
                2
            );
            assert!(
                !joined.sewing_groups[0]
                    .a
                    .iter()
                    .chain(&joined.sewing_groups[0].b)
                    .any(|interval| interval.curve == second)
            );
            close(
                joined.sewing_group_geometry(group.id).unwrap().lengths[0],
                original.sewing_group_geometry(group.id).unwrap().lengths[0],
            );
            assert!(editor.undo());
            assert_eq!(editor.project.sewing_groups, split.sewing_groups);
            assert!(editor.undo());
            assert_eq!(editor.project.sewing_groups, original.sewing_groups);
            assert!(editor.redo());
            assert!(editor.redo());
            assert_eq!(
                Project::from_json(&editor.project.to_json().unwrap()).unwrap(),
                editor.project
            );
        }
    }
}

#[test]
fn split_at_partial_endpoints_and_authored_map_knots_keeps_all_positive_spans() {
    for source in 0..2 {
        let (baseline, a, b) = curved_scene(true);
        let original = baseline.project.sewing_groups[0].clone();
        let interval = [&original.a, &original.b][source][0];
        let fraction = [0.3, 0.72][source];
        let breakpoint = oracle_parameter(&baseline.project, interval, fraction);
        for t in [interval.t0, breakpoint, interval.t1] {
            let mut editor = baseline.clone();
            let before = editor.project.sewing_group_geometry(original.id).unwrap();
            editor
                .execute(Command::SplitCurve {
                    curve: [a, b][source],
                    t,
                })
                .unwrap();
            let group = &editor.project.sewing_groups[0];
            assert_eq!(group.id, original.id);
            let side = [&group.a, &group.b][source];
            assert_eq!(side.len(), if t == breakpoint { 2 } else { 1 });
            assert!(side.iter().all(|interval| interval.t0 != interval.t1));
            let after = editor.project.sewing_group_geometry(original.id).unwrap();
            for side in 0..2 {
                close(before.lengths[side], after.lengths[side]);
            }
            editor.project.mesh().unwrap();
        }
    }
}

#[test]
fn geometry_edits_keep_attachments_recompute_lengths_and_reject_old_meshes() {
    let (mut editor, curve, _) = curved_scene(false);
    let group = editor.project.sewing_groups[0].clone();
    let mesh = editor.project.mesh().unwrap();
    editor
        .execute(Command::SetCurveControls {
            curve,
            controls: Some([[0.39, 0.55], [0.03, 0.41]]),
        })
        .unwrap();
    assert_eq!(editor.project.sewing_groups, vec![group]);
    let updated = editor.project.mesh().unwrap();
    assert_ne!(mesh.doc_revision, updated.doc_revision);
    assert!((mesh.sewing_groups[0].lengths[0] - updated.sewing_groups[0].lengths[0]).abs() > 1e-4);
    assert!(weftforge_core::simulation::create_cloth(&editor.project, &mesh).is_err());
    let mut foreign = updated;
    foreign.sewing_groups.clear();
    assert!(
        weftforge_core::simulation::create_cloth(&editor.project, &foreign)
            .unwrap_err()
            .0
            .contains("different construction")
    );
}

#[test]
fn a_thousand_cubic_spans_share_one_absolute_path_budget() {
    let (mut editor, curve, _) = curved_scene(false);
    let mut group = editor.project.sewing_groups[0].clone();
    let original = group.a[0];
    let expected = oracle_length(&editor.project, original, original.t1);
    group.a = (0..1024)
        .map(|index| {
            interval(
                curve,
                original.parameter(f64::from(index) / 1024.),
                original.parameter(f64::from(index + 1) / 1024.),
            )
        })
        .collect();
    editor
        .execute(Command::UpdateSewingGroup { group })
        .unwrap();
    let geometry = editor
        .project
        .sewing_group_geometry(editor.project.sewing_groups[0].id)
        .unwrap();
    close(geometry.lengths[0], expected);
    assert!(geometry.portions.len() >= 1024);
    assert_eq!(geometry.portions[0].a.t0, original.t0);
    assert_eq!(geometry.portions.last().unwrap().a.t1, original.t1);
}

#[test]
fn group_commands_reversal_history_branches_and_dependent_removal_are_atomic() {
    let (mut editor, curves) = line_scene();
    let original = editor.project.clone();
    let group = original.sewing_groups[0].clone();
    assert!(editor.undo());
    assert!(editor.project.sewing_groups.is_empty());
    assert!(editor.redo());
    for _ in 0..2 {
        let mut group = editor.project.sewing_groups[0].clone();
        group.b.reverse();
        for interval in &mut group.b {
            std::mem::swap(&mut interval.t0, &mut interval.t1);
        }
        let revision = editor.project.revision;
        editor
            .execute(Command::UpdateSewingGroup { group })
            .unwrap();
        assert_eq!(editor.project.revision, revision + 1);
    }
    assert_eq!(editor.project.sewing_groups, original.sewing_groups);
    editor
        .execute(Command::AddSewingGroup {
            name: "Unrelated".into(),
            a: vec![CurveInterval::full(curves[3])],
            b: vec![CurveInterval::full(curves[4])],
            correspondence: vec![[0., 0.], [1., 1.]],
        })
        .unwrap();
    let before_remove = editor.project.clone();
    let unrelated = before_remove.sewing_groups[1].clone();
    editor
        .execute(Command::RemovePiece {
            piece: original.pieces[0].id,
        })
        .unwrap();
    assert_eq!(editor.project.sewing_groups, vec![unrelated.clone()]);
    assert!(editor.undo());
    assert_eq!(editor.project.sewing_groups, before_remove.sewing_groups);
    assert!(editor.can_redo());
    let before_failure = editor.project.clone();
    let mut invalid = group.clone();
    invalid.b.push(invalid.a[0]);
    assert!(
        editor
            .execute(Command::UpdateSewingGroup { group: invalid })
            .is_err()
    );
    assert_eq!(editor.project, before_failure);
    assert!(editor.can_redo());
    assert!(
        editor
            .execute(Command::Transaction(vec![
                Command::RemoveSewingGroup { group: group.id },
                Command::SetResolution { metres: 0. }
            ]))
            .is_err()
    );
    assert_eq!(editor.project, before_failure);
    assert!(editor.redo());
    editor
        .execute(Command::RemoveSewingGroup {
            group: unrelated.id,
        })
        .unwrap();
    assert!(editor.project.sewing_groups.is_empty());
    assert!(editor.undo());
    editor
        .execute(Command::AddSewingGroup {
            name: "Fresh branch".into(),
            a: unrelated.a,
            b: unrelated.b,
            correspondence: unrelated.correspondence,
        })
        .unwrap();
    assert!(editor.project.sewing_groups.last().unwrap().id > unrelated.id);
}

#[test]
fn disjoint_same_curve_coverage_survives_split_join_but_overlaps_are_rejected() {
    let (mut editor, curves) = line_scene();
    let curve = curves[1];
    editor
        .execute(Command::AddSewingGroup {
            name: "Same-curve disjoint".into(),
            a: vec![interval(curve, 0.1, 0.3), interval(curve, 0.8, 0.9)],
            b: vec![interval(curve, 0.7, 0.4)],
            correspondence: vec![[0., 0.], [1., 1.]],
        })
        .unwrap();
    let id = editor.project.sewing_groups.last().unwrap().id;
    editor
        .execute(Command::SplitCurve { curve, t: 0.5 })
        .unwrap();
    editor.execute(Command::JoinCurve { curve }).unwrap();
    assert_eq!(editor.project.sewing_groups.last().unwrap().id, id);
    editor.project.mesh().unwrap();
    let before = editor.project.clone();
    let valid = before.sewing_groups.last().unwrap().clone();
    for bad in [
        SewingGroup {
            a: vec![interval(curve, 0.1, 0.3), interval(curve, 0.2, 0.4)],
            ..valid.clone()
        },
        SewingGroup {
            b: vec![interval(curve, 0.2, 0.6)],
            ..valid.clone()
        },
        SewingGroup {
            a: Vec::new(),
            ..valid.clone()
        },
        SewingGroup {
            a: vec![valid.a[0]; 1025],
            ..valid.clone()
        },
        SewingGroup {
            correspondence: vec![[0., 0.], [0.5, 0.7], [0.7, 0.6], [1., 1.]],
            ..valid.clone()
        },
        SewingGroup {
            correspondence: vec![[0., 0.], [f64::NAN, 0.7], [1., 1.]],
            ..valid.clone()
        },
        SewingGroup {
            a: vec![interval(u64::MAX, 0., 1.)],
            ..valid.clone()
        },
    ] {
        assert!(
            editor
                .execute(Command::UpdateSewingGroup { group: bad })
                .is_err()
        );
        assert_eq!(editor.project, before);
    }
}

#[test]
fn strict_v4_and_legacy_v2_keep_authored_values_and_maps_without_reinterpretation() {
    let (mut editor, a, b) = curved_scene(false);
    editor
        .execute(Command::AddSeam {
            name: "Legacy nonlinear".into(),
            a: interval(a, 0.12345678901234566, 0.91),
            b: interval(b, 0.88, 0.019999999999999997),
            correspondence: vec![[0., 0.], [0.3, 0.7199999999999999], [1., 1.]],
        })
        .unwrap();
    let v4 = editor.project.to_json().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&v4).unwrap();
    assert_eq!(value["version"], weftforge_core::FORMAT_VERSION);
    assert_eq!(Project::from_json(&v4).unwrap(), editor.project);
    let mut v3 = value.clone();
    v3["version"] = json!(3);
    for piece in v3["project"]["pieces"].as_array_mut().unwrap() {
        piece.as_object_mut().unwrap().remove("notches");
        piece.as_object_mut().unwrap().remove("holes");
        piece.as_object_mut().unwrap().remove("internal_curves");
    }
    assert_eq!(
        Project::from_json(&serde_json::to_vec(&v3).unwrap()).unwrap(),
        editor.project
    );
    let mut v2 = v3.clone();
    v2["version"] = json!(2);
    for piece in v2["project"]["pieces"].as_array_mut().unwrap() {
        piece.as_object_mut().unwrap().remove("internal_curves");
    }
    v2["project"]
        .as_object_mut()
        .unwrap()
        .remove("sewing_groups");
    let migrated = Project::from_json(&serde_json::to_vec(&v2).unwrap()).unwrap();
    assert!(migrated.sewing_groups.is_empty());
    assert_eq!(migrated.seams, editor.project.seams);
    assert_eq!(migrated.curves, editor.project.curves);
    assert_eq!(migrated.points, editor.project.points);
    assert_eq!(migrated.revision, editor.project.revision);
    assert_eq!(
        Project::from_json(&migrated.to_json().unwrap()).unwrap(),
        migrated
    );
    for mutate in [0, 1, 2, 3, 4] {
        let mut bad = value.clone();
        match mutate {
            0 => {
                bad["project"]
                    .as_object_mut()
                    .unwrap()
                    .remove("sewing_groups");
            }
            1 => bad["project"]["sewing_groups"][0]["unknown"] = json!(true),
            2 => {
                bad["project"]["sewing_groups"][0]["id"] = bad["project"]["seams"][0]["id"].clone()
            }
            3 => bad["project"]["sewing_groups"][0]["a"][0]["t0"] = json!(null),
            _ => bad["version"] = json!(weftforge_core::POSED_FORMAT_VERSION + 1),
        }
        assert!(Project::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let text = String::from_utf8(v4).unwrap().replacen(
        "\"sewing_groups\":",
        "\"sewing_groups\":[],\"sewing_groups\":",
        1,
    );
    assert!(Project::from_json(text.as_bytes()).is_err());
}

#[test]
fn group_save_reopen_recovery_and_failed_metric_save_keep_existing_files() {
    let (editor, _) = line_scene();
    let directory = std::env::temp_dir().join(format!(
        "weftforge-group-io-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("original.weftforge");
    editor.project.save(&path).unwrap();
    let original_bytes = std::fs::read(&path).unwrap();
    assert_eq!(Project::load(&path).unwrap(), editor.project);
    let mut changed = editor.project.clone();
    changed.sewing_groups[0].name = "Edited whole group".into();
    changed.save(&path).unwrap();
    assert_eq!(Project::load(&path).unwrap(), changed);
    assert_eq!(Project::recover(&path).unwrap(), editor.project);
    assert_eq!(
        std::fs::read(weftforge_core::recovery_path(&path)).unwrap(),
        original_bytes
    );
    let valid_bytes = std::fs::read(&path).unwrap();
    changed.sewing_groups[0].correspondence = vec![[0., 0.], [1e-18, 0.5], [1., 1.]];
    assert!(changed.save(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), valid_bytes);
    assert_eq!(
        std::fs::read(weftforge_core::recovery_path(&path)).unwrap(),
        original_bytes
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn resource_precision_cancellation_and_invalid_preview_fail_without_mutation() {
    let (mut editor, _) = line_scene();
    let original = editor.project.clone();
    let mut group = original.sewing_groups[0].clone();
    group.correspondence = vec![[0., 0.], [1e-18, 0.5], [1., 1.]];
    let error = editor
        .execute(Command::UpdateSewingGroup { group })
        .unwrap_err();
    assert!(error.0.contains("precision"), "{error}");
    assert_eq!(editor.project, original);
    let mut json: serde_json::Value = serde_json::from_slice(&original.to_json().unwrap()).unwrap();
    let group = json["project"]["sewing_groups"][0].clone();
    let mut many = Vec::new();
    for index in 0..11 {
        let mut group = group.clone();
        group["id"] = json!(1000 + index);
        group["a"] = json!(vec![group["a"][0].clone(); 1024]);
        group["b"] = json!(vec![group["b"][0].clone(); 1024]);
        many.push(group);
    }
    json["project"]["sewing_groups"] = json!(many);
    json["project"]["next_id"] = json!(2000);
    assert!(
        Project::from_json(&serde_json::to_vec(&json).unwrap())
            .unwrap_err()
            .0
            .contains("reference limit")
    );
    let cancelled = std::cell::Cell::new(0);
    assert!(
        original
            .mesh_cancellable(|| {
                cancelled.set(cancelled.get() + 1);
                cancelled.get() > 8
            })
            .is_err()
    );
    assert_eq!(editor.project, original);
    let mut mesh = original.mesh().unwrap();
    mesh.sewing_groups[0].lengths[0] = f64::NAN;
    assert!(mesh.validate().is_err());
    mesh = original.mesh().unwrap();
    mesh.sewing_groups[0].portions[0].a.curve = u64::MAX;
    assert!(mesh.validate().is_err());
    mesh = original.mesh().unwrap();
    mesh.sewing_groups.push(mesh.sewing_groups[0].clone());
    assert!(mesh.validate().is_err());
}

#[test]
fn conditioned_short_spans_reject_global_parameter_rounding_before_commit_or_io() {
    let mut editor = Editor::default();
    for (name, origin, width) in [
        ("Long original", [0., 0.], 10.),
        ("Short original", [12., 0.], 1.),
    ] {
        editor
            .execute(Command::AddRectangle {
                name: name.into(),
                origin,
                width,
                height: 0.1,
            })
            .unwrap();
    }
    let a = interval(editor.project.pieces[0].boundary[0], 0.99999997, 0.99999999);
    let b = interval(editor.project.pieces[1].boundary[0], 0., 5e-6);
    let ill_conditioned = vec![[0., 0.], [0.5, 0.2], [0.50000001, 0.8], [1., 1.]];

    // Independent straight-line oracle: a rounded global t changes the steep map.
    let returned_t = a.t0 + (a.t1 - a.t0) * 0.50000001;
    let resolved_u = (returned_t - a.t0) / (a.t1 - a.t0);
    let required_v = 0.2 + (resolved_u - 0.5) / (0.50000001 - 0.5) * 0.6;
    let physical_error = (0.8 - required_v).abs() * 5e-6;
    assert!((physical_error - 5.019982071452556e-7).abs() < 1e-13);
    assert!(physical_error > 1e-8);

    let safe_map = vec![[0., 0.], [0.4, 0.2], [0.6, 0.8], [1., 1.]];
    editor
        .execute(Command::AddSewingGroup {
            name: "Representable conditioned spans".into(),
            a: vec![a],
            b: vec![b],
            correspondence: safe_map,
        })
        .unwrap();
    let geometry = editor
        .project
        .sewing_group_geometry(editor.project.sewing_groups[0].id)
        .unwrap();
    assert_eq!(geometry.portions.len(), 3);
    for portion in &geometry.portions {
        let u = (portion.a.t1 - a.t0) / (a.t1 - a.t0);
        let v = if u <= 0.4 {
            u * 0.5
        } else if u <= 0.6 {
            0.2 + (u - 0.4) * 3.
        } else {
            0.8 + (u - 0.6) * 0.5
        };
        assert!(((portion.b.t1 - b.t0) - v * (b.t1 - b.t0)).abs() <= 1e-8);
    }
    editor
        .execute(Command::SetResolution { metres: 0.5 })
        .unwrap();
    assert!(editor.undo());
    let before = editor.project.clone();
    assert!(editor.can_redo());
    for cubic in [false, true] {
        for reversed in [false, true] {
            let mut variant = Editor::new(before.clone()).unwrap();
            if cubic {
                variant
                    .execute(Command::SetCurveControls {
                        curve: a.curve,
                        controls: Some([[1., 0.], [9., 0.]]),
                    })
                    .unwrap();
            }
            if reversed {
                let mut group = variant.project.sewing_groups[0].clone();
                for interval in group.a.iter_mut().chain(&mut group.b) {
                    std::mem::swap(&mut interval.t0, &mut interval.t1);
                }
                variant
                    .execute(Command::UpdateSewingGroup { group })
                    .unwrap();
            }
            let unchanged = variant.project.clone();
            let mut group = unchanged.sewing_groups[0].clone();
            group.correspondence = ill_conditioned.clone();
            assert!(
                variant
                    .execute(Command::UpdateSewingGroup {
                        group: group.clone()
                    })
                    .unwrap_err()
                    .0
                    .contains("precision budget")
            );
            assert_eq!(variant.project, unchanged);
            let mut invalid = unchanged;
            invalid.sewing_groups[0] = group;
            assert!(
                invalid
                    .sewing_group_geometry(invalid.sewing_groups[0].id)
                    .unwrap_err()
                    .0
                    .contains("precision budget")
            );
            assert!(
                invalid
                    .to_json()
                    .unwrap_err()
                    .0
                    .contains("precision budget")
            );
        }
    }
    let mut invalid = before.clone();
    invalid.sewing_groups[0].correspondence = ill_conditioned.clone();
    for command in [
        Command::AddSewingGroup {
            name: "Unrepresentable spans".into(),
            a: vec![a],
            b: vec![b],
            correspondence: ill_conditioned.clone(),
        },
        Command::UpdateSewingGroup {
            group: invalid.sewing_groups[0].clone(),
        },
        Command::Transaction(vec![
            Command::SetResolution { metres: 0.5 },
            Command::UpdateSewingGroup {
                group: invalid.sewing_groups[0].clone(),
            },
        ]),
    ] {
        assert!(
            editor
                .execute(command)
                .unwrap_err()
                .0
                .contains("precision budget")
        );
        assert_eq!(editor.project, before);
        assert!(editor.can_redo());
    }
    assert!(invalid.validate().is_ok());
    assert!(
        invalid
            .sewing_group_geometry(invalid.sewing_groups[0].id)
            .unwrap_err()
            .0
            .contains("precision budget")
    );
    assert!(
        invalid
            .to_json()
            .unwrap_err()
            .0
            .contains("precision budget")
    );
    assert!(
        Editor::new(invalid.clone())
            .unwrap_err()
            .0
            .contains("precision budget")
    );
    invalid.resolution = 0.5;
    assert!(invalid.mesh().unwrap_err().0.contains("precision budget"));

    let directory = std::env::temp_dir().join(format!(
        "weftforge-group-precision-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("original.weftforge");
    before.save(&path).unwrap();
    before.save(&path).unwrap();
    let primary = std::fs::read(&path).unwrap();
    let recovery = std::fs::read(weftforge_core::recovery_path(&path)).unwrap();
    assert!(
        invalid
            .save(&path)
            .unwrap_err()
            .0
            .contains("precision budget")
    );
    assert_eq!(std::fs::read(&path).unwrap(), primary);
    assert_eq!(
        std::fs::read(weftforge_core::recovery_path(&path)).unwrap(),
        recovery
    );
    let mut malformed: serde_json::Value = serde_json::from_slice(&primary).unwrap();
    malformed["project"]["sewing_groups"][0]["correspondence"] = json!(ill_conditioned);
    assert!(
        Project::from_json(&serde_json::to_vec(&malformed).unwrap())
            .unwrap_err()
            .0
            .contains("precision budget")
    );
    assert_eq!(editor.project, before);
    assert!(editor.redo());
    std::fs::remove_dir_all(directory).unwrap();
}

fn physical_length_scene() -> (Editor, u64, u64) {
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Original 200 by 300 mm curved panel".into(),
            vertices: vec![[0., 0.], [0.2, 0.1], [0.2, 0.3], [0., 0.3]],
        })
        .unwrap();
    let a = editor.project.pieces[0].boundary[0];
    editor
        .execute(Command::SetCurveControls {
            curve: a,
            controls: Some([[0.2 / 3., 0.], [0.4 / 3., 0.1 / 3.]]),
        })
        .unwrap();
    editor
        .execute(Command::AddRectangle {
            name: "Original 200 by 300 mm straight panel".into(),
            origin: [0.4, 0.],
            width: 0.2,
            height: 0.3,
        })
        .unwrap();
    let b = editor.project.pieces[1].boundary[0];
    (editor, a, b)
}

fn length_side(
    curve: u64,
    start_t: f64,
    direction: SewingDirection,
    length_m: f64,
) -> SewingSideByLength {
    SewingSideByLength {
        curve,
        start_t,
        direction,
        length_m,
    }
}

fn physical_primitive(t: f64) -> f64 {
    0.1 * (t * (1. + t * t).sqrt() + t.asinh())
}

fn physical_inverse(length_m: f64) -> f64 {
    let mut lo = 0.;
    let mut hi = 1.;
    for _ in 0..60 {
        let mid = (lo + hi) * 0.5;
        if physical_primitive(mid) < length_m {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) * 0.5
}

#[test]
fn physical_length_intervals_match_primitive_and_derivative_oracles_in_both_directions() {
    use SewingDirection::{Forward, Reverse};
    let (editor, a, b) = physical_length_scene();
    let before = format!("{editor:?}");
    close(physical_primitive(1.), 0.22955871493926383);
    for (start_t, direction, expected_t) in [
        (0., Forward, 0.4819445564563284),
        (0.8, Reverse, 0.3702881984991444),
    ] {
        let resolved = editor
            .project
            .interval_from_length(length_side(a, start_t, direction, 0.1))
            .unwrap();
        assert_eq!(resolved.t0.to_bits(), start_t.to_bits());
        let target = physical_primitive(start_t) + if direction == Forward { 0.1 } else { -0.1 };
        let expected = physical_inverse(target);
        assert!((expected - expected_t).abs() < 1e-14);
        close_point(
            point(&editor.project, resolved, true),
            [0.2 * expected, 0.1 * expected * expected],
        );
        close(oracle_length(&editor.project, resolved, resolved.t1), 0.1);
        close(editor.project.interval_length(resolved).unwrap(), 0.1);
        let full = interval(a, start_t, if direction == Forward { 1. } else { 0. });
        let fraction = 0.1 / oracle_length(&editor.project, full, full.t1);
        close_point(
            point(&editor.project, resolved, true),
            editor
                .project
                .curve(a)
                .unwrap()
                .evaluate(
                    &editor.project,
                    oracle_parameter(&editor.project, full, fraction),
                )
                .unwrap(),
        );
    }
    for (start_t, direction, expected_t) in [(0.75, Reverse, 0.25), (0.1, Forward, 0.6)] {
        let resolved = editor
            .project
            .interval_from_length(length_side(b, start_t, direction, 0.1))
            .unwrap();
        assert!((resolved.t1 - expected_t).abs() < 1e-14);
        close(oracle_length(&editor.project, resolved, resolved.t1), 0.1);
    }
    for curve in [a, b] {
        for (start_t, direction, end) in [
            (0., Forward, 1.),
            (1., Reverse, 0.),
            (0.37, Forward, 1.),
            (0.63, Reverse, 0.),
        ] {
            let remaining = interval(curve, start_t, end);
            let length_m = editor.project.interval_length(remaining).unwrap();
            let resolved = editor
                .project
                .interval_from_length(length_side(curve, start_t, direction, length_m))
                .unwrap();
            assert_eq!(resolved, remaining);
            assert!(
                editor
                    .project
                    .interval_from_length(length_side(
                        curve,
                        start_t,
                        direction,
                        length_m.next_up()
                    ))
                    .is_err()
            );
        }
    }
    assert_eq!(format!("{editor:?}"), before);
}

#[test]
fn physical_length_command_is_one_persistent_group_with_physical_correspondence_and_history() {
    use SewingDirection::{Forward, Reverse};
    let (mut editor, a, b) = physical_length_scene();
    editor
        .execute(Command::AddSeam {
            name: "Original legacy nonlinear t map".into(),
            a: interval(a, 0.1, 0.8),
            b: interval(b, 0.9, 0.1),
            correspondence: vec![[0., 0.], [0.3, 0.72], [1., 1.]],
        })
        .unwrap();
    let original = editor.project.clone();
    editor
        .execute(Command::AddSewingByLength {
            name: "Original physical 100 mm pair".into(),
            a: length_side(a, 0., Forward, 0.1),
            b: length_side(b, 0.75, Reverse, 0.1),
        })
        .unwrap();
    assert_eq!(editor.project.revision, original.revision + 1);
    assert_eq!(editor.project.seams, original.seams);
    assert_eq!(editor.project.sewing_groups.len(), 1);
    let group = editor.project.sewing_groups[0].clone();
    assert_eq!(group.correspondence, vec![[0., 0.], [1., 1.]]);
    assert_eq!(group.a.len(), 1);
    assert_eq!(group.b.len(), 1);
    assert_eq!(group.a[0].t0, 0.);
    assert_eq!(group.b[0].t0, 0.75);
    let geometry = editor.project.sewing_group_geometry(group.id).unwrap();
    for length in geometry.lengths {
        close(length, 0.1);
    }
    let encoded = editor.project.to_json().unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&encoded).unwrap()["version"],
        weftforge_core::FORMAT_VERSION
    );
    assert_eq!(Project::from_json(&encoded).unwrap(), editor.project);

    let mut knots = editor.project.clone();
    knots.sewing_groups[0].correspondence =
        vec![[0., 0.], [0.25, 0.25], [0.5, 0.5], [0.75, 0.75], [1., 1.]];
    let quarters = knots.sewing_group_geometry(group.id).unwrap();
    for (index, portion) in quarters.portions.iter().enumerate() {
        for (end, fraction) in [
            (false, index as f64 * 0.25),
            (true, (index + 1) as f64 * 0.25),
        ] {
            let expected = physical_inverse(0.1 * fraction);
            close_point(
                point(&knots, portion.a, end),
                [0.2 * expected, 0.1 * expected * expected],
            );
            close_point(point(&knots, portion.b, end), [0.55 - 0.1 * fraction, 0.]);
        }
    }
    assert!((physical_inverse(0.05) - group.a[0].parameter(0.5)).abs() > 0.005);
    for resolution in [0.09, 0.025] {
        let mut project = editor.project.clone();
        project.resolution = resolution;
        let mesh = project.mesh().unwrap();
        assert_eq!(mesh.sewing_groups, vec![geometry.clone()]);
        for stitch in mesh
            .stitches
            .iter()
            .filter(|stitch| stitch.seam == group.id)
        {
            let at = parameter(&mesh, a, stitch.a).unwrap();
            let bt = parameter(&mesh, b, stitch.b).unwrap();
            close(physical_primitive(at), (0.75 - bt) * 0.2);
        }
        for stitch in mesh
            .stitches
            .iter()
            .filter(|stitch| stitch.seam == original.seams[0].id)
        {
            let at = parameter(&mesh, a, stitch.a).unwrap();
            let bt = parameter(&mesh, b, stitch.b).unwrap();
            let seam = &original.seams[0];
            close(
                (bt - seam.b.t0) / (seam.b.t1 - seam.b.t0),
                paired((at - seam.a.t0) / (seam.a.t1 - seam.a.t0)),
            );
        }
    }
    assert!(editor.undo());
    assert!(editor.project.sewing_groups.is_empty());
    assert_eq!(editor.project.seams, original.seams);
    assert!(editor.redo());
    assert_eq!(editor.project.sewing_groups, vec![group.clone()]);
    editor
        .execute(Command::SetCurveControls {
            curve: a,
            controls: None,
        })
        .unwrap();
    assert_eq!(editor.project.sewing_groups, vec![group]);
    assert!(
        (editor
            .project
            .sewing_group_geometry(geometry.id)
            .unwrap()
            .lengths[0]
            - 0.1)
            .abs()
            > 1e-5
    );
}

#[test]
fn physical_length_failures_preserve_document_allocator_and_both_history_stacks() {
    use SewingDirection::{Forward, Reverse};
    let (mut editor, a, b) = physical_length_scene();
    editor
        .execute(Command::SetResolution { metres: 0.09 })
        .unwrap();
    assert!(editor.undo());
    let before = format!("{editor:?}");
    let bytes = editor.project.to_json().unwrap();
    let valid_a = length_side(a, 0., Forward, 0.1);
    let valid_b = length_side(b, 0.75, Reverse, 0.1);
    let mut invalid = vec![
        length_side(u64::MAX, 0., Forward, 0.1),
        length_side(a, 1., Forward, 0.1),
        length_side(a, 0., Reverse, 0.1),
        length_side(a, 0., Forward, 1e-12),
        length_side(a, 0., Forward, 1.),
    ];
    for start_t in [-0.1, 1.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        invalid.push(SewingSideByLength { start_t, ..valid_a });
    }
    for length_m in [
        0.,
        -0.1,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
    ] {
        invalid.push(SewingSideByLength {
            length_m,
            ..valid_a
        });
    }
    for side in invalid {
        assert!(editor.project.interval_from_length(side).is_err());
        for (a, b) in [(side, valid_b), (valid_a, side)] {
            assert!(
                editor
                    .execute(Command::AddSewingByLength {
                        name: "Rejected pair".into(),
                        a,
                        b
                    })
                    .is_err()
            );
            assert_eq!(format!("{editor:?}"), before);
            assert_eq!(editor.project.to_json().unwrap(), bytes);
        }
    }
    for name in ["", " ", "Bad\nname", &"x".repeat(257)] {
        assert!(
            editor
                .execute(Command::AddSewingByLength {
                    name: name.into(),
                    a: valid_a,
                    b: valid_b
                })
                .is_err()
        );
        assert_eq!(format!("{editor:?}"), before);
    }
    assert!(
        editor
            .execute(Command::AddSewingByLength {
                name: "Overlap".into(),
                a: valid_a,
                b: valid_a
            })
            .unwrap_err()
            .0
            .contains("overlaps")
    );
    assert_eq!(format!("{editor:?}"), before);
    assert!(
        editor
            .execute(Command::AddSewingByLength {
                name: "Unresolvable proportion".into(),
                a: valid_a,
                b: length_side(b, 0., Forward, 1e-8)
            })
            .unwrap_err()
            .0
            .contains("precision budget")
    );
    assert_eq!(format!("{editor:?}"), before);
    assert!(
        editor
            .execute(Command::Transaction(vec![
                Command::SetCurveControls {
                    curve: a,
                    controls: None
                },
                Command::AddSewingByLength {
                    name: "Rejected second side".into(),
                    a: valid_a,
                    b: length_side(b, 1., Forward, 0.1)
                },
            ]))
            .is_err()
    );
    assert_eq!(format!("{editor:?}"), before);
    for field in ["next_id", "revision"] {
        let mut document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        document["project"][field] = json!(u64::MAX);
        let mut exhausted =
            Editor::new(Project::from_json(&serde_json::to_vec(&document).unwrap()).unwrap())
                .unwrap();
        let unchanged = format!("{exhausted:?}");
        assert!(
            exhausted
                .execute(Command::AddSewingByLength {
                    name: "Exhausted".into(),
                    a: valid_a,
                    b: valid_b
                })
                .is_err()
        );
        assert_eq!(format!("{exhausted:?}"), unchanged);
        if field == "next_id" {
            assert!(
                exhausted
                    .execute(Command::AddSewingByLength {
                        name: "Resolve before allocation".into(),
                        a: valid_a,
                        b: length_side(u64::MAX, 0., Forward, 0.1)
                    })
                    .unwrap_err()
                    .0
                    .contains("curve is missing")
            );
            assert_eq!(format!("{exhausted:?}"), unchanged);
        }
    }
    assert!(editor.redo());
}

#[test]
fn physical_length_creation_reuses_same_curve_disjoint_and_unequal_length_group_rules() {
    use SewingDirection::{Forward, Reverse};
    let (mut editor, _, b) = physical_length_scene();
    editor
        .execute(Command::AddSewingByLength {
            name: "Disjoint unequal lengths".into(),
            a: length_side(b, 0., Forward, 0.025),
            b: length_side(b, 1., Reverse, 0.05),
        })
        .unwrap();
    let group = &editor.project.sewing_groups[0];
    assert_eq!(group.a.len(), 1);
    assert_eq!(group.b.len(), 1);
    assert_eq!(group.a[0].curve, b);
    assert_eq!(group.b[0].curve, b);
    assert_eq!(group.a[0].t0, 0.);
    assert_eq!(group.b[0].t0, 1.);
    close(group.a[0].t1, 0.125);
    close(group.b[0].t1, 0.75);
    assert_eq!(group.correspondence, vec![[0., 0.], [1., 1.]]);
    let lengths = editor
        .project
        .sewing_group_geometry(group.id)
        .unwrap()
        .lengths;
    close(lengths[0], 0.025);
    close(lengths[1], 0.05);
}
