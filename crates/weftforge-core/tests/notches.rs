use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use weftforge_core::{Command, Editor, Notch, Project, SvgPatternImport, recovery_path};

fn panel() -> Editor {
    let mut editor = Editor::default();
    editor
        .execute(Command::AddRectangle {
            name: "Original notch panel".into(),
            origin: [0.; 2],
            width: 0.24,
            height: 0.36,
        })
        .unwrap();
    editor
}

fn add(curve: u64, t: f64) -> Command {
    Command::AddNotch {
        piece: 2,
        curve,
        t,
        length: 0.012345678901234567,
    }
}

fn assert_segment(actual: [[f64; 2]; 2], expected: [[f64; 2]; 2]) {
    for (actual, expected) in actual
        .into_iter()
        .flatten()
        .zip(expected.into_iter().flatten())
    {
        assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
    }
}

fn assert_refused(editor: &mut Editor, command: Command, message: &str) {
    let before = format!("{editor:?}");
    let error = editor.execute(command).unwrap_err();
    assert!(error.to_string().contains(message), "{error}");
    assert_eq!(format!("{editor:?}"), before);
}

#[test]
fn creation_edit_removal_and_transaction_history_preserve_construction_and_mesh() {
    let mut editor = panel();
    let original = editor.project.clone();
    let mesh = original.mesh().unwrap();
    editor.execute(add(7, 0.25)).unwrap();
    let notch = editor.project.notch(11).unwrap().clone();
    assert_eq!(editor.project.points, original.points);
    assert_eq!(editor.project.curves, original.curves);
    assert_eq!(notch.curve, 7);
    let segment = notch.segment(&editor.project).unwrap();
    assert!((segment[0][0] - 0.06).abs() < 1e-12);
    assert!((segment[0][1] + notch.length * 0.5).abs() < 1e-12);
    assert!((segment[1][1] - notch.length * 0.5).abs() < 1e-12);
    let rebuilt = editor.project.mesh().unwrap();
    assert_ne!(mesh.doc_revision, rebuilt.doc_revision);
    assert!(weftforge_core::simulation::create_cloth(&editor.project, &mesh).is_err());
    assert_eq!(rebuilt.positions, mesh.positions);
    assert_eq!(rebuilt.uv, mesh.uv);
    assert_eq!(rebuilt.triangles, mesh.triangles);
    assert_eq!(rebuilt.piece_ids, mesh.piece_ids);
    let before = editor.project.clone();
    let revision = before.revision;
    editor
        .execute(Command::Transaction(vec![
            Command::SetGrain {
                piece: 2,
                angle: 0.37,
            },
            Command::UpdateNotch {
                notch: Notch {
                    t: 0.75,
                    length: 0.008,
                    ..notch.clone()
                },
            },
        ]))
        .unwrap();
    assert_eq!(editor.project.revision, revision + 1);
    assert_eq!(editor.project.notch(11).unwrap().t, 0.75);
    assert!(editor.undo());
    assert_eq!(editor.project.pieces, before.pieces);
    assert!(editor.redo());
    assert_eq!(editor.project.pieces[0].grain_angle, 0.37);
    editor.execute(Command::RemoveNotch { notch: 11 }).unwrap();
    assert!(editor.project.pieces[0].notches.is_empty());
    assert!(editor.undo());
    assert_eq!(editor.project.notch(11).unwrap().length, 0.008);
    assert!(editor.redo());
    editor.execute(add(7, 0.5)).unwrap();
    assert!(editor.project.pieces[0].notches[0].id > 11);
}

#[test]
fn invalid_values_ownership_and_unusable_tangents_refuse_without_changing_history_or_ids() {
    let mut editor = panel();
    editor.execute(add(7, 0.25)).unwrap();
    assert!(editor.undo());
    for t in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -1e-12,
        1. + 1e-12,
    ] {
        assert_refused(&mut editor, add(7, t), "Curve t");
    }
    for length in [f64::NAN, f64::INFINITY, 0., -0.01, 0.000099, 0.100001] {
        assert_refused(
            &mut editor,
            Command::AddNotch {
                piece: 2,
                curve: 7,
                t: 0.5,
                length,
            },
            "Notch length",
        );
    }
    for (piece, curve) in [(0, 7), (999, 7), (2, 0), (2, 999), (2, 3)] {
        assert_refused(
            &mut editor,
            Command::AddNotch {
                piece,
                curve,
                t: 0.5,
                length: 0.005,
            },
            "owner piece",
        );
    }
    assert!(editor.redo());
    let notch = editor.project.notch(11).unwrap().clone();
    assert_refused(
        &mut editor,
        Command::UpdateNotch {
            notch: Notch {
                id: 0,
                ..notch.clone()
            },
        },
        "Unknown notch",
    );
    assert_refused(
        &mut editor,
        Command::RemoveNotch { notch: 0 },
        "Unknown notch",
    );
    editor
        .execute(Command::AddRectangle {
            name: "Other panel".into(),
            origin: [1., 0.],
            width: 0.2,
            height: 0.3,
        })
        .unwrap();
    let other_curve = editor.project.pieces[1].boundary[0];
    assert_refused(
        &mut editor,
        Command::UpdateNotch {
            notch: Notch {
                curve: other_curve,
                ..notch
            },
        },
        "owner piece",
    );

    let mut cusp = panel();
    cusp.execute(add(7, 0.5)).unwrap();
    assert_refused(
        &mut cusp,
        Command::SetCurveControls {
            curve: 7,
            controls: Some([[0.24, 0.], [0., 0.]]),
        },
        "tangent",
    );
    cusp.execute(Command::RemoveNotch { notch: 11 }).unwrap();
    cusp.execute(Command::SetCurveControls {
        curve: 7,
        controls: Some([[0., 0.], [0.2, 0.]]),
    })
    .unwrap();
    assert_refused(&mut cusp, add(7, 0.), "tangent");
    for length in [
        weftforge_core::MIN_NOTCH_LENGTH,
        weftforge_core::MAX_NOTCH_LENGTH,
    ] {
        cusp.execute(Command::AddNotch {
            piece: 2,
            curve: 7,
            t: 1.,
            length,
        })
        .unwrap();
    }
}

#[test]
fn split_and_exact_join_preserve_straight_curved_reversed_and_endpoint_notches() {
    for curved in [false, true] {
        for reversed in [false, true] {
            let mut editor = Editor::default();
            editor
                .execute(Command::AddPolygon {
                    name: "Original oriented panel".into(),
                    vertices: if reversed {
                        vec![[0.24, 0.], [0., 0.], [0., 0.36], [0.24, 0.36]]
                    } else {
                        vec![[0., 0.], [0.24, 0.], [0.24, 0.36], [0., 0.36]]
                    },
                })
                .unwrap();
            if curved {
                let controls = if reversed {
                    [[0.18, 0.02], [0.04, 0.03]]
                } else {
                    [[0.04, 0.03], [0.18, 0.02]]
                };
                editor
                    .execute(Command::SetCurveControls {
                        curve: 7,
                        controls: Some(controls),
                    })
                    .unwrap();
            }
            editor
                .execute(Command::Transaction(
                    [0., 0.1, 0.4, 0.8, 1.].map(|t| add(7, t)).to_vec(),
                ))
                .unwrap();
            let before = editor.project.clone();
            let segments: Vec<_> = before.pieces[0]
                .notches
                .iter()
                .map(|notch| notch.segment(&before).unwrap())
                .collect();
            editor
                .execute(Command::SplitCurve { curve: 7, t: 0.4 })
                .unwrap();
            let second = editor.project.pieces[0].boundary[1];
            for ((notch, original), segment) in editor.project.pieces[0]
                .notches
                .iter()
                .zip(&before.pieces[0].notches)
                .zip(&segments)
            {
                assert_eq!(notch.id, original.id);
                assert_eq!(notch.length.to_bits(), original.length.to_bits());
                assert_eq!(notch.curve, if original.t <= 0.4 { 7 } else { second });
                assert_segment(notch.segment(&editor.project).unwrap(), *segment);
            }
            assert_eq!(editor.project.pieces[0].notches[2].t, 1.);
            let split = editor.project.clone();
            editor.execute(Command::JoinCurve { curve: 7 }).unwrap();
            for ((notch, original), segment) in editor.project.pieces[0]
                .notches
                .iter()
                .zip(&before.pieces[0].notches)
                .zip(&segments)
            {
                assert_eq!(notch.curve, 7);
                assert!((notch.t - original.t).abs() < 1e-12);
                assert_segment(notch.segment(&editor.project).unwrap(), *segment);
            }
            assert!(editor.undo());
            assert_eq!(editor.project.pieces, split.pieces);
            assert!(editor.redo());
        }
    }
}

#[test]
fn curved_notch_uses_the_analytic_tangent_and_reversal_retains_the_visible_marker() {
    let mut editor = panel();
    editor
        .execute(Command::SetCurveControls {
            curve: 7,
            controls: Some([[0.08, 0.08], [0.16, 0.08]]),
        })
        .unwrap();
    editor
        .execute(Command::AddNotch {
            piece: 2,
            curve: 7,
            t: 0.25,
            length: 0.02,
        })
        .unwrap();
    let half = 0.01 / 5_f64.sqrt();
    let expected = [
        [0.06 + half, 0.045 - 2. * half],
        [0.06 - half, 0.045 + 2. * half],
    ];
    assert_segment(
        editor
            .project
            .notch(11)
            .unwrap()
            .segment(&editor.project)
            .unwrap(),
        expected,
    );
    let mut reversed = editor.project.clone();
    reversed.pieces[0].boundary.reverse();
    for curve in &mut reversed.curves {
        std::mem::swap(&mut curve.start, &mut curve.end);
        if let Some(controls) = &mut curve.controls {
            controls.reverse();
        }
    }
    reversed.pieces[0].notches[0].t = 0.75;
    reversed.validate().unwrap();
    assert_segment(
        reversed.notch(11).unwrap().segment(&reversed).unwrap(),
        [expected[1], expected[0]],
    );
}

#[test]
fn ordinary_geometry_edits_keep_parameter_identity_and_fixed_physical_length() {
    let mut editor = panel();
    editor.execute(add(7, 0.25)).unwrap();
    let notch = editor.project.notch(11).unwrap().clone();
    for command in [
        Command::MovePoint {
            point: 4,
            position: [0.28, 0.],
        },
        Command::SetCurveControls {
            curve: 7,
            controls: Some([[0.05, 0.01], [0.2, 0.02]]),
        },
        Command::ResizePiece {
            piece: 2,
            width: 0.56,
            height: 0.72,
        },
    ] {
        editor.execute(command).unwrap();
        assert_eq!(editor.project.notch(11), Some(&notch));
        let [a, b] = notch.segment(&editor.project).unwrap();
        assert!(((a[0] - b[0]).hypot(a[1] - b[1]) - notch.length).abs() < 1e-12);
    }
    let segment = notch.segment(&editor.project).unwrap();
    editor
        .execute(Command::TransformPiece {
            piece: 2,
            translation: [1., 2.],
            angle: std::f64::consts::FRAC_PI_2,
            pivot: [0.; 2],
        })
        .unwrap();
    assert_eq!(editor.project.notch(11), Some(&notch));
    assert_segment(
        notch.segment(&editor.project).unwrap(),
        segment.map(|[u, v]| [1. - v, 2. + u]),
    );
}

#[test]
fn unsupported_topology_changes_refuse_instead_of_dropping_notch_relationships() {
    let mut editor = panel();
    editor.execute(add(7, 0.5)).unwrap();
    assert_refused(
        &mut editor,
        Command::CutPiece {
            piece: 2,
            start_curve: 7,
            start_t: 0.5,
            end_curve: 9,
            end_t: 0.5,
        },
        "notches",
    );
    assert_refused(
        &mut editor,
        Command::CutAndSew {
            piece: 2,
            start_curve: 7,
            start_t: 0.5,
            end_curve: 9,
            end_t: 0.5,
        },
        "notches",
    );
    assert_refused(
        &mut editor,
        Command::OffsetOutline {
            piece: 2,
            width: 0.01,
            outward: true,
        },
        "OffsetUnsupportedNotch",
    );
    let mut editor = Editor::new(
        Project::from_json(include_bytes!(
            "fixtures/original-notch-migration-v5.weftforge"
        ))
        .unwrap(),
    )
    .unwrap();
    editor
        .execute(Command::Transaction(vec![add(16, 0.5), add(22, 0.5)]))
        .unwrap();
    assert_refused(&mut editor, Command::RemoveHole { hole: 11 }, "notches");
    assert_refused(
        &mut editor,
        Command::RemoveInternalCurve { curve: 22 },
        "notches",
    );
    editor
        .execute(Command::Transaction(vec![
            Command::RemoveNotch { notch: 23 },
            Command::RemoveHole { hole: 11 },
            Command::RemoveNotch { notch: 24 },
            Command::RemoveInternalCurve { curve: 22 },
        ]))
        .unwrap();
    assert!(editor.project.pieces[0].holes.is_empty());
    assert!(editor.project.pieces[0].internal_curves.is_empty());
    assert!(editor.undo());
    let before = editor.project.clone();
    editor.execute(Command::RemovePiece { piece: 2 }).unwrap();
    assert!(editor.project.pieces.is_empty() && editor.project.curves.is_empty());
    assert!(editor.undo());
    assert_eq!(editor.project.pieces, before.pieces);
}

#[test]
fn strict_v6_notches_and_explicit_v5_migration_preserve_all_older_fields_and_bytes() {
    let bytes = include_bytes!("fixtures/original-notch-migration-v5.weftforge");
    let old: Value = serde_json::from_slice(bytes).unwrap();
    let project = Project::from_json(bytes).unwrap();
    let mut current: Value = serde_json::from_slice(&project.to_json().unwrap()).unwrap();
    assert_eq!(current["version"], 6);
    assert_eq!(
        current["project"]["pieces"][0]
            .as_object_mut()
            .unwrap()
            .remove("notches"),
        Some(json!([]))
    );
    assert_eq!(current["project"], old["project"]);
    let directory = Directory::new();
    let path = directory.0.join("migrated.weftforge");
    fs::write(&path, bytes).unwrap();
    let mut editor = Editor::new(project).unwrap();
    editor.execute(add(22, 0.12345678901234566)).unwrap();
    editor.project.save(&path).unwrap();
    assert_eq!(Project::load(&path).unwrap(), editor.project);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), bytes);
    let document: Value = serde_json::from_slice(&editor.project.to_json().unwrap()).unwrap();
    let mut invalid = Vec::new();
    let mut bad = document.clone();
    bad["project"]["pieces"][0]
        .as_object_mut()
        .unwrap()
        .remove("notches");
    invalid.push(bad);
    for (field, value) in [
        ("id", json!(0)),
        ("id", json!(22)),
        ("curve", json!(0)),
        ("curve", json!(7.5)),
        ("t", json!(-0.1)),
        ("t", json!(1.1)),
        ("length", json!(0)),
        ("length", json!(0.100001)),
        ("extra", json!(true)),
    ] {
        let mut bad = document.clone();
        bad["project"]["pieces"][0]["notches"][0][field] = value;
        invalid.push(bad);
    }
    for field in ["id", "curve", "t", "length"] {
        let mut bad = document.clone();
        bad["project"]["pieces"][0]["notches"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        invalid.push(bad);
    }
    let mut bad = document.clone();
    let duplicate = bad["project"]["pieces"][0]["notches"][0].clone();
    bad["project"]["pieces"][0]["notches"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    invalid.push(bad);
    let mut bad = document.clone();
    bad["version"] = json!(5);
    invalid.push(bad);
    for bad in invalid {
        assert!(
            Project::from_json(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{bad}"
        );
    }
    let text = String::from_utf8(editor.project.to_json().unwrap()).unwrap();
    for bad in [
        text.replace("\"notches\":", "\"notches\":[],\"notches\":"),
        text.replace("\"t\":", "\"t\":0.5,\"t\":"),
        text.replace("0.12345678901234566", "1e309"),
    ] {
        assert_ne!(bad, text);
        assert!(Project::from_json(bad.as_bytes()).is_err());
    }
    let primary = fs::read(&path).unwrap();
    let mut bad = editor.project.clone();
    bad.pieces[0].notches[0].length = f64::NAN;
    assert!(bad.save(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), primary);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), bytes);
}

#[test]
fn svg_keeps_physical_notch_geometry_and_bounds_while_dxf_refuses_before_replacement() {
    let mut editor = panel();
    editor
        .execute(Command::AddNotch {
            piece: 2,
            curve: 7,
            t: 0.,
            length: 0.02,
        })
        .unwrap();
    let directory = Directory::new();
    let svg = directory.0.join("pattern.svg");
    editor.project.export_svg(&svg).unwrap();
    let text = fs::read_to_string(&svg).unwrap();
    assert!(text.contains("viewBox=\"0 -360 240 370\""), "{text}");
    assert!(
        text.contains(
            "<line id=\"notch-11\" data-piece=\"2\" x1=\"0\" y1=\"10\" x2=\"0\" y2=\"-10\""
        )
    );
    assert!(
        SvgPatternImport::from_bytes(text.as_bytes())
            .unwrap_err()
            .to_string()
            .contains("Unsupported SVG element")
    );
    let dxf = directory.0.join("pattern.dxf");
    fs::write(&dxf, b"retain original destination").unwrap();
    assert!(
        editor
            .project
            .export_dxf(&dxf)
            .unwrap_err()
            .to_string()
            .contains("notches")
    );
    assert_eq!(fs::read(&dxf).unwrap(), b"retain original destination");
    let mut invalid = editor.project.clone();
    invalid.pieces[0].notches[0].t = f64::NAN;
    let before = fs::read(&svg).unwrap();
    assert!(invalid.export_svg(&svg).is_err());
    assert_eq!(fs::read(&svg).unwrap(), before);
}

#[test]
fn notch_commands_reject_incomplete_duplicate_and_unknown_fields() {
    for text in [
        r#"{"type":"add_notch","data":{"piece":2,"curve":7,"t":0.5}}"#,
        r#"{"type":"add_notch","data":{"piece":2,"piece":2,"curve":7,"t":0.5,"length":0.005}}"#,
        r#"{"type":"add_notch","data":{"piece":2,"curve":7,"t":0.5,"length":0.005,"extra":0}}"#,
        r#"{"type":"add_notch","data":{"piece":2,"curve":7,"t":null,"length":0.005}}"#,
        r#"{"type":"update_notch","data":{"notch":{"id":11,"curve":7,"t":0.5,"length":0.005,"curve":7}}}"#,
        r#"{"type":"remove_notch","data":{"notch":11,"extra":0}}"#,
    ] {
        assert!(serde_json::from_str::<Command>(text).is_err(), "{text}");
    }
}

#[test]
fn notches_share_global_object_and_stable_id_limits() {
    let editor = panel();
    let mut document: Value = serde_json::from_slice(&editor.project.to_json().unwrap()).unwrap();
    document["project"]["pieces"][0]["notches"] = json!(
        (11..=20_001)
            .map(|id| Notch {
                id,
                curve: 7,
                t: 0.5,
                length: 0.005
            })
            .collect::<Vec<_>>()
    );
    document["project"]["next_id"] = json!(20_002);
    assert!(
        Project::from_json(&serde_json::to_vec(&document).unwrap())
            .unwrap_err()
            .to_string()
            .contains("object limit")
    );
    document["project"]["pieces"][0]["notches"] =
        json!([{ "id":11,"curve":7,"t":0.5,"length":0.005 }]);
    document["project"]["next_id"] = json!(11);
    assert!(
        Project::from_json(&serde_json::to_vec(&document).unwrap())
            .unwrap_err()
            .to_string()
            .contains("next stable ID")
    );
}

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "weftforge-notches-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
