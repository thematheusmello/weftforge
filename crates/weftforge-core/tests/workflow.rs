use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use weftforge_core::{Command, CurveInterval, Editor, Mesh, Placement, Project};

static FILE_ID: AtomicU64 = AtomicU64::new(0);

fn rectangle(editor: &mut Editor, name: &str, origin: [f64; 2], width: f64, height: f64) -> u64 {
    editor
        .execute(Command::AddRectangle {
            name: name.into(),
            origin,
            width,
            height,
        })
        .unwrap();
    editor.project.pieces.last().unwrap().id
}

fn seam_scene() -> Editor {
    let mut editor = Editor::default();
    let a = rectangle(&mut editor, "Front", [0.0, 0.0], 0.7, 0.5);
    let b = rectangle(&mut editor, "Sleeve", [1.0, 0.0], 0.45, 0.3);
    let a_curve = editor.project.piece(a).unwrap().boundary[2];
    let b_curve = editor.project.piece(b).unwrap().boundary[0];
    editor
        .execute(Command::Transaction(vec![
            Command::SetCurveControls {
                curve: a_curve,
                controls: Some([[0.55, 0.7], [0.12, 0.62]]),
            },
            Command::SetCurveControls {
                curve: b_curve,
                controls: Some([[1.15, -0.1], [1.3, -0.08]]),
            },
            Command::AddSeam {
                name: "Gathered curved interval".into(),
                a: CurveInterval {
                    curve: a_curve,
                    t0: 0.15,
                    t1: 0.9,
                },
                b: CurveInterval {
                    curve: b_curve,
                    t0: 0.9,
                    t1: 0.1,
                },
                correspondence: vec![[0.0, 0.0], [0.4, 0.65], [1.0, 1.0]],
            },
        ]))
        .unwrap();
    editor
}

#[test]
fn invalid_contours_and_failed_transactions_leave_document_and_history_unchanged() {
    let mut editor = Editor::default();
    let before = editor.project.clone();
    for vertices in [
        vec![[0.0, 0.0], [1.0, 1.0], [0.0, 1.0], [1.0, 0.0]],
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        vec![[0.0, 0.0], [f64::NAN, 0.0], [0.0, 1.0]],
        vec![[0.0, 0.0], [1.0, 0.0], [0.5, 0.0]],
    ] {
        assert!(
            editor
                .execute(Command::AddPolygon {
                    name: "Invalid".into(),
                    vertices
                })
                .is_err()
        );
        assert_eq!(editor.project, before);
        assert!(!editor.can_undo());
    }
    assert!(
        editor
            .execute(Command::Transaction(vec![
                Command::AddRectangle {
                    name: "Valid first operation".into(),
                    origin: [0.0, 0.0],
                    width: 0.4,
                    height: 0.6
                },
                Command::SetResolution { metres: 0.0 },
            ]))
            .is_err()
    );
    assert_eq!(editor.project, before);
    assert!(!editor.can_undo());
}

#[test]
fn transactions_history_and_stable_ids_survive_branching() {
    let mut editor = Editor::default();
    editor
        .execute(Command::Transaction(vec![
            Command::AddRectangle {
                name: "Front".into(),
                origin: [0.0, 0.0],
                width: 0.5,
                height: 0.7,
            },
            Command::AddRectangle {
                name: "Back".into(),
                origin: [0.6, 0.0],
                width: 0.5,
                height: 0.7,
            },
        ]))
        .unwrap();
    assert_eq!(editor.project.revision, 1);
    let ids: BTreeSet<_> = editor.project.pieces.iter().map(|piece| piece.id).collect();
    assert!(editor.undo());
    assert!(editor.project.pieces.is_empty());
    assert_eq!(editor.project.revision, 2);
    assert!(editor.redo());
    assert_eq!(editor.project.pieces.len(), 2);
    assert_eq!(editor.project.revision, 3);
    assert!(editor.undo());
    let fresh = rectangle(&mut editor, "New branch", [0.0, 0.0], 0.2, 0.3);
    assert!(!ids.contains(&fresh));
    assert!(!editor.can_redo());
}

fn assert_correspondence(project: &Project, mesh: &Mesh) {
    let seam = &project.seams[0];
    let samples: BTreeMap<_, _> = mesh
        .boundary
        .iter()
        .map(|s| ((s.curve, s.vertex), s.t))
        .collect();
    let param = |curve, endpoint: weftforge_core::WeightedVertex| {
        samples[&(curve, endpoint.vertices[0])] * (1.0 - endpoint.weight)
            + samples[&(curve, endpoint.vertices[1])] * endpoint.weight
    };
    let mut previous = -1.0;
    for stitch in &mesh.stitches {
        assert_eq!(stitch.seam, seam.id);
        let a_t = param(seam.a.curve, stitch.a);
        let b_t = param(seam.b.curve, stitch.b);
        let fraction = (a_t - seam.a.t0) / (seam.a.t1 - seam.a.t0);
        let paired = if fraction <= 0.4 {
            fraction / 0.4 * 0.65
        } else {
            0.65 + (fraction - 0.4) / 0.6 * 0.35
        };
        assert!((b_t - seam.b.parameter(paired)).abs() < 1.0e-10);
        assert!(fraction > previous);
        previous = fraction;
        for (interval, endpoint, t) in [(seam.a, stitch.a, a_t), (seam.b, stitch.b, b_t)] {
            let point = project
                .curve(interval.curve)
                .unwrap()
                .evaluate(project, t)
                .unwrap();
            let linear = endpoint.position(&mesh.positions);
            assert!((point[0] - linear[0]).abs() < 0.001);
            assert!((point[1] - linear[1]).abs() < 0.001);
        }
    }
    assert!((previous - 1.0).abs() < 1.0e-10);
    assert!(
        mesh.stitches
            .iter()
            .any(|s| s.a.vertices[0] != s.a.vertices[1] || s.b.vertices[0] != s.b.vertices[1])
    );
}

#[test]
fn reversed_gathered_cubic_intervals_retain_correspondence_through_remesh_edits_and_files() {
    let mut editor = seam_scene();
    editor
        .execute(Command::SetResolution { metres: 0.09 })
        .unwrap();
    let coarse = editor.project.mesh().unwrap();
    assert_correspondence(&editor.project, &coarse);
    let authored_seam = editor.project.seams[0].clone();
    editor
        .execute(Command::SetResolution { metres: 0.035 })
        .unwrap();
    let fine = editor.project.mesh().unwrap();
    assert!(fine.positions.len() > coarse.positions.len());
    assert_eq!(editor.project.seams[0], authored_seam);
    assert_correspondence(&editor.project, &fine);
    let curve = editor.project.seams[0].a.curve;
    editor
        .execute(Command::SetCurveControls {
            curve,
            controls: Some([[0.54, 0.72], [0.14, 0.64]]),
        })
        .unwrap();
    let edited = editor.project.mesh().unwrap();
    assert_correspondence(&editor.project, &edited);
    assert!(editor.undo());
    assert_eq!(editor.project.seams[0], authored_seam);
    let reopened = Project::from_json(&editor.project.to_json().unwrap()).unwrap();
    assert_eq!(reopened, editor.project);
    assert_correspondence(&reopened, &reopened.mesh().unwrap());
    let removed = editor.project.pieces[0].id;
    editor
        .execute(Command::RemovePiece { piece: removed })
        .unwrap();
    assert!(editor.project.seams.is_empty());
    editor.project.validate().unwrap();
    assert!(editor.undo());
    assert_eq!(editor.project.seams[0], authored_seam);
}

#[test]
fn split_cubic_is_exact_and_resize_updates_real_geometry() {
    let mut editor = Editor::default();
    let piece = rectangle(&mut editor, "Curved pattern", [0.0, 0.0], 0.4, 0.6);
    let curve = editor.project.piece(piece).unwrap().boundary[2];
    editor
        .execute(Command::SetCurveControls {
            curve,
            controls: Some([[0.3, 0.8], [0.1, 0.8]]),
        })
        .unwrap();
    let original = editor.project.clone();
    editor
        .execute(Command::SplitCurve { curve, t: 0.3 })
        .unwrap();
    let boundary = &editor.project.piece(piece).unwrap().boundary;
    assert_eq!(boundary.len(), 5);
    let first = editor.project.curve(curve).unwrap();
    let second = editor.project.curve(boundary[3]).unwrap();
    for i in 0..=100 {
        let t = f64::from(i) / 100.0;
        let expected = original
            .curve(curve)
            .unwrap()
            .evaluate(&original, t)
            .unwrap();
        let actual = if t <= 0.3 {
            first.evaluate(&editor.project, t / 0.3).unwrap()
        } else {
            second.evaluate(&editor.project, (t - 0.3) / 0.7).unwrap()
        };
        for axis in 0..2 {
            assert!((expected[axis] - actual[axis]).abs() < 1.0e-12);
        }
    }
    editor
        .execute(Command::ResizePiece {
            piece,
            width: 0.6,
            height: 0.8,
        })
        .unwrap();
    let [min, max] = editor.project.piece_bounds(piece).unwrap();
    assert!((max[0] - min[0] - 0.6).abs() < 1.0e-8);
    assert!((max[1] - min[1] - 0.8).abs() < 1.0e-12);
    let mesh = editor.project.mesh().unwrap();
    assert!(!mesh.triangles.is_empty());
}

// Authored f64 geometry and sewing maps must agree within 1e-12 m/parameter.
const SPLIT_TOLERANCE: f64 = 1.0e-12;

fn paired_fraction(pairs: &[[f64; 2]], fraction: f64) -> f64 {
    let pair = pairs
        .windows(2)
        .find(|pair| fraction <= pair[1][0] + SPLIT_TOLERANCE)
        .unwrap();
    pair[0][1] + (fraction - pair[0][0]) / (pair[1][0] - pair[0][0]) * (pair[1][1] - pair[0][1])
}

fn assert_split_sewing(original: &Project, split: &Project, curve: u64, second: u64, t: f64) {
    let original_parameter = |id, parameter| {
        if id == curve {
            parameter * t
        } else if id == second {
            t + parameter * (1.0 - t)
        } else {
            parameter
        }
    };
    for old in &original.seams {
        let portions: Vec<_> = split
            .seams
            .iter()
            .filter(|seam| seam.name == old.name)
            .collect();
        assert!(!portions.is_empty());
        assert_eq!(portions[0].id, old.id);
        let mut previous = 0.0;
        for portion in portions {
            let fraction = |u| {
                (original_parameter(portion.a.curve, portion.a.parameter(u)) - old.a.t0)
                    / (old.a.t1 - old.a.t0)
            };
            assert!((fraction(0.0) - previous).abs() < SPLIT_TOLERANCE);
            previous = fraction(1.0);
            let samples = (0..=100)
                .map(|i| f64::from(i) / 100.0)
                .chain(portion.correspondence.iter().map(|pair| pair[0]));
            for u in samples {
                let v = paired_fraction(&portion.correspondence, u);
                let old_u = fraction(u);
                let old_v = paired_fraction(&old.correspondence, old_u);
                for (before, after, old_fraction, new_fraction) in
                    [(old.a, portion.a, old_u, u), (old.b, portion.b, old_v, v)]
                {
                    let expected = original
                        .curve(before.curve)
                        .unwrap()
                        .evaluate(original, before.parameter(old_fraction).clamp(0.0, 1.0))
                        .unwrap();
                    let actual = split
                        .curve(after.curve)
                        .unwrap()
                        .evaluate(split, after.parameter(new_fraction).clamp(0.0, 1.0))
                        .unwrap();
                    for axis in 0..2 {
                        assert!((expected[axis] - actual[axis]).abs() < SPLIT_TOLERANCE);
                    }
                }
            }
        }
        assert!((previous - 1.0).abs() < SPLIT_TOLERANCE);
    }
}

#[test]
fn sewn_splits_preserve_gathered_pairs_in_both_directions_at_breakpoints_and_interval_ends() {
    for straight in [false, true] {
        for source in 0..2 {
            for reverse_a in [false, true] {
                for reverse_b in [false, true] {
                    let mut scene = seam_scene();
                    let mut seam = scene.project.seams[0].clone();
                    if straight {
                        scene
                            .execute(Command::Transaction(vec![
                                Command::SetCurveControls {
                                    curve: seam.a.curve,
                                    controls: None,
                                },
                                Command::SetCurveControls {
                                    curve: seam.b.curve,
                                    controls: None,
                                },
                            ]))
                            .unwrap();
                    }
                    if reverse_a {
                        std::mem::swap(&mut seam.a.t0, &mut seam.a.t1);
                    }
                    if reverse_b {
                        std::mem::swap(&mut seam.b.t0, &mut seam.b.t1);
                    }
                    scene
                        .execute(Command::UpdateSeam { seam: seam.clone() })
                        .unwrap();
                    let interval = [seam.a, seam.b][source];
                    let breakpoint = interval.parameter(seam.correspondence[1][source]);
                    for t in [0.05, 0.1, 0.15, 0.5, 0.9, 0.95, breakpoint] {
                        let mut editor = scene.clone();
                        let original = editor.project.clone();
                        editor
                            .execute(Command::SplitCurve {
                                curve: interval.curve,
                                t,
                            })
                            .unwrap();
                        let second = editor.project.curves.last().unwrap().id;
                        assert_split_sewing(&original, &editor.project, interval.curve, second, t);
                        let crosses =
                            t > interval.t0.min(interval.t1) && t < interval.t0.max(interval.t1);
                        assert_eq!(editor.project.seams.len(), if crosses { 2 } else { 1 });
                        assert_eq!(editor.project.materials, original.materials);
                        assert_eq!(editor.project.pieces[0].name, original.pieces[0].name);
                    }
                }
            }
        }
    }
}

#[test]
fn multiple_seams_survive_split_remesh_history_branching_and_v2_save_reopen() {
    let mut editor = seam_scene();
    let curve = editor.project.seams[0].a.curve;
    let mut extra = editor.project.seams[0].clone();
    extra.a.t0 = 0.05;
    extra.a.t1 = 0.3;
    editor
        .execute(Command::AddSeam {
            name: "Second partial seam".into(),
            a: extra.a,
            b: extra.b,
            correspondence: extra.correspondence,
        })
        .unwrap();
    let original = editor.project.clone();
    editor
        .execute(Command::SplitCurve { curve, t: 0.45 })
        .unwrap();
    let split = editor.project.clone();
    let highest_id = split.seams.iter().map(|seam| seam.id).max().unwrap();
    let second = split.curves.last().unwrap().id;
    assert_split_sewing(&original, &split, curve, second, 0.45);
    assert_eq!(split.seams.len(), 3);
    assert_eq!(split.seams.last().unwrap().id, original.seams[1].id);
    let mut coarse_vertices = 0;
    for metres in [0.09, 0.035] {
        editor.execute(Command::SetResolution { metres }).unwrap();
        let mesh = editor.project.mesh().unwrap();
        mesh.validate().unwrap();
        if coarse_vertices == 0 {
            coarse_vertices = mesh.positions.len();
        } else {
            assert!(mesh.positions.len() > coarse_vertices);
        }
        for &[a, b, c] in &mesh.triangles {
            let [a, b, c] = [mesh.uv[a], mesh.uv[b], mesh.uv[c]];
            assert!((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 0.0);
        }
        let samples: BTreeMap<_, _> = mesh
            .boundary
            .iter()
            .map(|s| ((s.curve, s.vertex), s.t))
            .collect();
        for seam in &editor.project.seams {
            let stitches: Vec<_> = mesh.stitches.iter().filter(|s| s.seam == seam.id).collect();
            assert!(stitches.len() >= 2);
            for stitch in stitches {
                let parameter = |interval: CurveInterval, p: weftforge_core::WeightedVertex| {
                    let t = samples[&(interval.curve, p.vertices[0])] * (1.0 - p.weight)
                        + samples[&(interval.curve, p.vertices[1])] * p.weight;
                    (t - interval.t0) / (interval.t1 - interval.t0)
                };
                let u = parameter(seam.a, stitch.a);
                let v = parameter(seam.b, stitch.b);
                assert!((v - paired_fraction(&seam.correspondence, u)).abs() < SPLIT_TOLERANCE);
            }
        }
        assert_eq!(editor.project.seams, split.seams);
    }
    let path = std::env::temp_dir().join(format!(
        "weftforge-split-{}-{}.weftforge",
        std::process::id(),
        FILE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    editor.project.save(&path).unwrap();
    assert_eq!(Project::load(&path).unwrap(), editor.project);
    let json: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(json["version"], 2);
    fs::remove_file(path).unwrap();
    assert!(editor.undo());
    assert!(editor.undo());
    assert_eq!(editor.project.seams, split.seams);
    assert!(editor.undo());
    assert_eq!(editor.project.seams, original.seams);
    assert!(editor.redo());
    assert_eq!(editor.project.seams, split.seams);
    assert_eq!(editor.project.points, split.points);
    assert_eq!(editor.project.curves, split.curves);
    assert!(editor.undo());
    let before = editor.project.clone();
    assert!(editor.can_redo());
    for t in [0.0, 1.0, f64::NAN, 0.15 + 1.0e-12] {
        assert!(editor.execute(Command::SplitCurve { curve, t }).is_err());
        assert_eq!(editor.project, before);
        assert!(editor.can_redo());
    }
    assert!(
        editor
            .execute(Command::Transaction(vec![
                Command::SplitCurve { curve, t: 0.5 },
                Command::SetResolution { metres: 0.0 }
            ]))
            .is_err()
    );
    assert_eq!(editor.project, before);
    assert!(editor.can_redo());
    editor
        .execute(Command::SplitCurve { curve, t: 0.5 })
        .unwrap();
    assert!(!editor.can_redo());
    assert!(editor.project.curves.last().unwrap().id > highest_id);
    assert!(editor.project.seams[1].id > highest_id);
}

#[test]
fn a_seam_cannot_reference_the_same_curve_on_both_sides() {
    let mut editor = seam_scene();
    let mut seam = editor.project.seams[0].clone();
    seam.b = CurveInterval::reversed(seam.a.curve);
    let before = editor.project.clone();
    let error = editor.execute(Command::UpdateSeam { seam }).unwrap_err();
    assert!(error.to_string().contains("distinct curves"));
    assert_eq!(editor.project, before);
}

#[test]
fn rest_mesh_has_physical_scale_consistent_uvs_and_conforming_refinement() {
    let mut editor = Editor::default();
    let piece = rectangle(&mut editor, "Scale specimen", [0.0, 0.0], 0.5, 0.7);
    editor
        .execute(Command::SetPlacement {
            piece,
            placement: Placement {
                translation: [-0.25, 0.8, 0.25],
                rotation: [0.0, 0.0, 0.0],
            },
        })
        .unwrap();
    let mesh = editor.project.mesh().unwrap();
    mesh.validate().unwrap();
    let mut area = 0.0;
    let mut edges = BTreeMap::new();
    for &[a, b, c] in &mesh.triangles {
        let [a_uv, b_uv, c_uv] = [mesh.uv[a], mesh.uv[b], mesh.uv[c]];
        area += ((b_uv[0] - a_uv[0]) * (c_uv[1] - a_uv[1])
            - (b_uv[1] - a_uv[1]) * (c_uv[0] - a_uv[0]))
            * 0.5;
        for [a, b] in [[a, b], [b, c], [c, a]] {
            let key = if a < b { (a, b) } else { (b, a) };
            *edges.entry(key).or_insert(0) += 1;
            let length = ((mesh.uv[a][0] - mesh.uv[b][0]).powi(2)
                + (mesh.uv[a][1] - mesh.uv[b][1]).powi(2))
            .sqrt();
            assert!(length <= editor.project.resolution * 1.4 + 1.0e-12);
        }
    }
    assert!((area - 0.35).abs() < 1.0e-12);
    assert!(edges.values().all(|&count| count == 1 || count == 2));
    for (position, uv) in mesh.positions.iter().zip(&mesh.uv) {
        assert!((position[0] - (uv[0] - 0.25)).abs() < 1.0e-12);
        assert!((position[1] - (uv[1] + 0.8)).abs() < 1.0e-12);
        assert_eq!(position[2], 0.25);
    }
    assert!(
        mesh.normals()
            .unwrap()
            .iter()
            .all(|n| (n[2] - 1.0).abs() < 1.0e-12)
    );
}

#[test]
fn independent_obj_parser_observes_metres_y_up_uvs_normals_and_piece_groups() {
    let mut editor = Editor::default();
    let piece = rectangle(
        &mut editor,
        "Original <front> & \"piece\"",
        [0.0, 0.0],
        0.5,
        0.7,
    );
    editor
        .execute(Command::SetPlacement {
            piece,
            placement: Placement {
                translation: [0.0, 1.0, 0.25],
                rotation: [0.0, 0.0, 0.0],
            },
        })
        .unwrap();
    let mesh = editor.project.mesh().unwrap();
    let base = std::env::temp_dir().join(format!(
        "weftforge-export-{}-{}",
        std::process::id(),
        FILE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let obj = base.with_extension("obj");
    mesh.export_obj(&obj).unwrap();
    let (models, materials) = tobj::load_obj(
        &obj,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(materials.unwrap().is_empty());
    assert_eq!(models.len(), 1);
    let parsed = &models[0].mesh;
    assert_eq!(parsed.indices.len(), mesh.triangles.len() * 3);
    assert_eq!(parsed.positions.len(), mesh.positions.len() * 3);
    assert_eq!(parsed.texcoords.len(), mesh.uv.len() * 2);
    assert_eq!(parsed.normals.len(), mesh.positions.len() * 3);
    let min_y = parsed
        .positions
        .chunks_exact(3)
        .map(|p| p[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = parsed
        .positions
        .chunks_exact(3)
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max);
    assert!((min_y - 1.0).abs() < 1.0e-6);
    assert!((max_y - 1.7).abs() < 1.0e-6);
    assert!(
        parsed
            .positions
            .chunks_exact(3)
            .all(|p| (p[2] - 0.25).abs() < 1.0e-6)
    );
    let svg = base.with_extension("svg");
    editor.project.export_svg(&svg).unwrap();
    let xml = fs::read_to_string(&svg).unwrap();
    assert!(xml.contains("width=\"1000mm\""));
    assert!(xml.contains("Original &lt;front&gt; &amp; &quot;piece&quot;"));
    fs::remove_file(obj).unwrap();
    fs::remove_file(svg).unwrap();
}

#[test]
fn loaded_duplicate_ids_dangling_relationships_and_non_monotone_seams_are_rejected() {
    let mut editor = seam_scene();
    let snapshot = editor.project.clone();
    let mut seam = editor.project.seams[0].clone();
    seam.correspondence = vec![[0.0, 0.0], [0.4, 0.8], [0.6, 0.7], [1.0, 1.0]];
    assert!(editor.execute(Command::UpdateSeam { seam }).is_err());
    assert_eq!(editor.project, snapshot);
    let json = editor.project.to_json().unwrap();
    let mut document: serde_json::Value = serde_json::from_slice(&json).unwrap();
    document["project"]["points"][0]["id"] = document["project"]["points"][1]["id"].clone();
    assert!(Project::from_json(&serde_json::to_vec(&document).unwrap()).is_err());
    let mut document: serde_json::Value = serde_json::from_slice(&json).unwrap();
    document["project"]["pieces"][0]["material"] = 99999.into();
    assert!(Project::from_json(&serde_json::to_vec(&document).unwrap()).is_err());
}
