use serde_json::json;
use std::time::Instant;
use weftforge_core::{Command, CurveInterval, Editor, Mesh, Project, SewingGroup, WeightedVertex};

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
fn strict_v3_and_legacy_v2_keep_authored_values_and_maps_without_reinterpretation() {
    let (mut editor, a, b) = curved_scene(false);
    editor
        .execute(Command::AddSeam {
            name: "Legacy nonlinear".into(),
            a: interval(a, 0.12345678901234566, 0.91),
            b: interval(b, 0.88, 0.019999999999999997),
            correspondence: vec![[0., 0.], [0.3, 0.7199999999999999], [1., 1.]],
        })
        .unwrap();
    let v3 = editor.project.to_json().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&v3).unwrap();
    assert_eq!(value["version"], 3);
    assert_eq!(Project::from_json(&v3).unwrap(), editor.project);
    let mut v2 = value.clone();
    v2["version"] = json!(2);
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
            _ => bad["version"] = json!(4),
        }
        assert!(Project::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let text = String::from_utf8(v3).unwrap().replacen(
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
