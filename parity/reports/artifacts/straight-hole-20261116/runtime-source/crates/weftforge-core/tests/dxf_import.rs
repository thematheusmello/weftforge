use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use weftforge_core::{
    Command, DxfPatternImport, Editor, MAX_DXF_BYTES, MAX_OBJECTS, Placement, Project,
};

const ORIGINAL: &[u8] = include_bytes!("fixtures/original-editable-patterns.dxf");

fn source() -> String {
    String::from_utf8(ORIGINAL.to_vec()).unwrap()
}
fn import(dxf: impl Into<String>) -> Command {
    Command::ImportDxfPatterns { dxf: dxf.into() }
}
fn entities(change: impl FnOnce(&str) -> String) -> String {
    let original = source();
    let (before, tail) = original.split_once("2\nENTITIES\n").unwrap();
    let (body, after) = tail.split_once("0\nENDSEC\n").unwrap();
    format!("{before}2\nENTITIES\n{}0\nENDSEC\n{after}", change(body))
}
fn history(editor: &Editor) -> [Vec<Project>; 2] {
    std::array::from_fn(|direction| {
        let mut editor = editor.clone();
        let mut projects = Vec::new();
        while if direction == 0 {
            editor.undo()
        } else {
            editor.redo()
        } {
            projects.push(editor.project.clone());
        }
        projects
    })
}
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "weftforge-dxf-import-{}-{}",
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

#[test]
fn original_drawing_adds_exact_editable_si_curves_layout_and_layer_names_in_one_undo() {
    let parsed = DxfPatternImport::from_bytes(ORIGINAL).unwrap();
    assert_eq!(parsed.piece_count(), 2);
    let mut editor = Editor::default();
    editor
        .execute(Command::AddRectangle {
            name: "Existing".into(),
            origin: [0., 0.],
            width: 0.4,
            height: 0.6,
        })
        .unwrap();
    let before = editor.project.clone();
    editor.execute(parsed.command()).unwrap();
    let project = &editor.project;
    assert_eq!(project.revision, before.revision + 1);
    assert_eq!(&project.pieces[..1], before.pieces);
    assert_eq!(&project.points[..4], before.points);
    assert_eq!(&project.curves[..4], before.curves);
    assert_eq!(project.pieces[1].name, "WF_PIECE_17");
    assert_eq!(project.pieces[2].name, "WF_PIECE_4");
    assert_eq!(project.points.len(), 12);
    assert_eq!(project.curves.len(), 12);
    for (point, expected) in project.points[4..].iter().zip([
        [-0.25, 0.15],
        [-0.05, 0.15],
        [-0.05, 0.025],
        [-0.25, 0.025],
        [0.1, 0.1],
        [0.3, 0.1],
        [0.3, -0.1],
        [0.1, -0.1],
    ]) {
        assert_eq!(point.position, expected);
    }
    let curved = &project.curves[5];
    assert_eq!(curved.controls, Some([[-0.01, 0.18], [-0.005, 0.01]]));
    for t in [0.0_f64, 0.125, 0.25, 0.5, 0.75, 0.875, 1.] {
        let actual = curved.evaluate(project, t).unwrap();
        // Independently expanded cubic polynomial from the fixture's four controls.
        let expected = [
            -0.05 + 0.12 * t - 0.105 * t * t - 0.015 * t * t * t,
            0.15 + 0.09 * t - 0.6 * t * t + 0.385 * t * t * t,
        ];
        assert!((actual[0] - expected[0]).hypot(actual[1] - expected[1]) < 1e-15);
    }
    for piece in &project.pieces[1..] {
        assert_eq!(
            (piece.material, piece.grain_angle, piece.placement),
            (1, 0., Placement::default())
        );
        assert!(!before.pieces.iter().any(|old| old.id == piece.id));
        for (index, id) in piece.boundary.iter().enumerate() {
            assert_eq!(
                project.curve(*id).unwrap().end,
                project
                    .curve(piece.boundary[(index + 1) % piece.boundary.len()])
                    .unwrap()
                    .start
            );
        }
    }
    let authored = project.clone();
    let directory = Directory::new();
    let native = directory.0.join("imported.weftforge");
    project.save(&native).unwrap();
    assert_eq!(Project::load(&native).unwrap(), authored);
    assert!(editor.undo());
    assert_eq!(editor.project.points, before.points);
    assert_eq!(editor.project.curves, before.curves);
    assert_eq!(editor.project.pieces, before.pieces);
    assert!(editor.redo());
    assert_eq!(editor.project.points, authored.points);
    assert_eq!(editor.project.curves, authored.curves);
    let curve = editor.project.curves[5].id;
    editor
        .execute(Command::SetCurveControls {
            curve,
            controls: Some([[-0.02, 0.18], [-0.02, 0.01]]),
        })
        .unwrap();
    assert_eq!(
        editor.project.curve(curve).unwrap().controls,
        Some([[-0.02, 0.18], [-0.02, 0.01]])
    );
}

#[test]
fn own_writer_round_trip_is_supplemental_and_keeps_cubics_and_f64_scale() {
    let directory = Directory::new();
    let mut original = Editor::default();
    original
        .execute(Command::AddEllipse {
            name: "Name omitted by DXF".into(),
            origin: [-0.3141592653589793, 0.2718281828459045],
            width: 0.2345678901234567,
            height: 0.3456789012345678,
        })
        .unwrap();
    let piece = original.project.pieces[0].id;
    original
        .execute(Command::SetPlacement {
            piece,
            placement: Placement {
                translation: [2., 3., 4.],
                ..Placement::default()
            },
        })
        .unwrap();
    let path = directory.0.join("writer.dxf");
    original.project.export_dxf(&path).unwrap();
    let mut imported = Editor::default();
    imported
        .execute(DxfPatternImport::load(&path).unwrap().command())
        .unwrap();
    assert_eq!(imported.project.curves.len(), 4);
    assert_eq!(imported.project.pieces[0].name, format!("WF_PIECE_{piece}"));
    assert_eq!(imported.project.pieces[0].placement, Placement::default());
    for (a, b) in imported.project.points.iter().zip(&original.project.points) {
        assert_eq!(a.position, b.position.map(|x| (x * 1000.) / 1000.));
    }
    for (a, b) in imported.project.curves.iter().zip(&original.project.curves) {
        assert_eq!(
            a.controls,
            b.controls
                .map(|controls| controls.map(|point| point.map(|x| (x * 1000.) / 1000.)))
        );
    }
    let padded = source()
        .lines()
        .map(|line| format!("\t{line} \r\n"))
        .collect::<String>();
    assert_eq!(
        DxfPatternImport::from_bytes(padded.as_bytes())
            .unwrap()
            .piece_count(),
        2
    );
}

#[test]
fn malformed_unsupported_or_unsafe_input_preserves_document_and_both_history_directions() {
    let mut editor = Editor::default();
    editor.execute(import(source())).unwrap();
    let piece = editor.project.pieces[0].id;
    editor
        .execute(Command::SetGrain { piece, angle: 0.2 })
        .unwrap();
    assert!(editor.undo());
    let before = editor.project.clone();
    let before_history = history(&editor);
    let invalid = vec![
        "0\nSECTION\n2\nHEADER\n0".into(),
        source().replace("AC1015", "AC1009"),
        source().replace("$INSUNITS\n70\n4", "$INSUNITS\n70\n1"),
        source().replace("9\n$INSUNITS\n70\n4\n", ""),
        source().replace("$LUNITS", "$UCSORG"),
        source().replace("CLASSES\n0\nENDSEC", "CLASSES\n0\nCLASS\n0\nENDSEC"),
        source().replace("5\n301\n", "5\n300\n"),
        source().replace("5\nF00", "5\n300"),
        source().replace("340\n1B0", "340\n1B1"),
        source().replace("350\n1A1", "350\nFFF"),
        source().replace("WF_PIECE_17", "WF_PIECE_017"),
        source().replace("WF_PIECE_17", "WF_PIECE_0"),
        source().replace("WF_PIECE_17", "WF_PIECE_18446744073709551616"),
        source().replace("WF_PIECE_17", "WF_PIECE_4"),
        source().replace("WF_PIECE_17\n70\n0", "WF_PIECE_17\n70\n1"),
        source().replace("BLOCKS\n0\nBLOCK", "BLOCKS\n0\nLINE"),
        source().replace("3\nACAD_GROUP", "3\nUNSUPPORTED_AUTHORING"),
        source().replace("0\nEOF\n", ""),
        format!("{}0\nLINE\n", source()),
        entities(|body| body.replacen("0\nLINE", "0\nLWPOLYLINE", 1)),
        entities(|body| body.replacen("330\n181", "330\n182", 1)),
        entities(|body| body.replacen("67\n0", "67\n1", 1)),
        entities(|body| body.replacen("410\nModel", "410\nLayout1", 1)),
        entities(|body| body.replacen("8\nWF_PIECE_17", "8\n0", 1)),
        entities(|body| body.replacen("70\n8", "70\n12", 1)),
        entities(|body| body.replacen("71\n3", "71\n2", 1)),
        entities(|body| body.replacen("74\n0", "74\n1", 1)),
        entities(|body| body.replacen("40\n1", "40\n0.5", 1)),
        entities(|body| body.replacen("73\n4", "73\n4\n41\n1", 1)),
        entities(|body| body.replacen("73\n4", "73\n4\n42\n0.1", 1)),
        entities(|body| body.replacen("230\n1", "230\n-1", 1)),
        entities(|body| body.replacen("30\n0", "30\n1", 1)),
        entities(|body| body.replacen("39\n0", "39\n1", 1)),
        entities(|body| body.replacen("10\n-250", "10\n-250\n10\n-250", 1)),
        entities(|body| body.replacen("11\n-50", "11\n-49", 1)),
        entities(|body| body.replacen("10\n-250", "10\nNaN", 1)),
        entities(|body| body.replacen("10\n-250", "10\ninf", 1)),
        entities(|body| body.replacen("10\n-250", "10\n1e309", 1)),
        entities(|body| body.replace("-250", "-1000001")),
        entities(|_| String::new()),
        entities(|body| {
            body.replace("-250\n20\n150", "-50\n20\n150")
                .replace("-250\n21\n150", "-50\n21\n150")
        }),
    ];
    for (index, dxf) in invalid.into_iter().enumerate() {
        assert!(
            editor.execute(import(dxf)).is_err(),
            "accepted invalid DXF case {index}"
        );
        assert_eq!(editor.project, before, "project changed in case {index}");
        assert_eq!(
            history(&editor),
            before_history,
            "history changed in case {index}"
        );
    }
    for bytes in [
        b"AutoCAD Binary DXF\r\n\x1a\0".as_slice(),
        b"\xef\xbb\xbf0\nEOF\n",
        b"0\nEOF\n\xff",
    ] {
        assert!(DxfPatternImport::from_bytes(bytes).is_err());
    }
    assert!(DxfPatternImport::from_bytes(&vec![b' '; MAX_DXF_BYTES + 1]).is_err());
    let too_many_pairs = format!("{}{}", "999\nx\n".repeat(131073), source());
    assert!(
        DxfPatternImport::from_bytes(too_many_pairs.as_bytes())
            .unwrap_err()
            .0
            .contains("pair limit")
    );
    let too_many_layers = source().replace(
        "2\nLAYER\n5\n120\n330\n0\n100\nAcDbSymbolTable\n70\n3",
        "2\nLAYER\n5\n120\n330\n0\n100\nAcDbSymbolTable\n70\n66",
    );
    assert!(
        DxfPatternImport::from_bytes(too_many_layers.as_bytes())
            .unwrap_err()
            .0
            .contains("64-piece limit")
    );
}

#[test]
fn full_import_and_later_batch_failure_leave_old_project_and_id_allocator_unchanged() {
    let mut editor = Editor::default();
    let before = editor.project.clone();
    assert!(
        editor
            .execute(Command::Transaction(vec![
                import(source()),
                Command::MovePoint {
                    point: 0,
                    position: [0., 0.]
                }
            ]))
            .is_err()
    );
    assert_eq!(editor.project, before);
    assert!(!editor.can_undo());
    let mut oversized = Project::default();
    oversized.materials = (1..MAX_OBJECTS as u64)
        .map(|id| {
            let mut material = oversized.materials[0].clone();
            material.id = id;
            material
        })
        .collect();
    // Derive a valid allocator for the deliberately almost-full source document.
    let mut value = serde_json::to_value(&oversized).unwrap();
    value["next_id"] = serde_json::json!(MAX_OBJECTS);
    oversized = serde_json::from_value(value).unwrap();
    let mut editor = Editor::new(oversized).unwrap();
    let before = editor.project.clone();
    assert!(editor.execute(import(source())).is_err());
    assert_eq!(editor.project, before);
}

#[test]
fn entity_limit_accepts_4096_and_rejects_4097_before_authoring() {
    let drawing = |count| {
        let original = source();
        let (_, entities_tail) = original.split_once("2\nENTITIES\n").unwrap();
        let (body, _) = entities_tail.split_once("0\nENDSEC\n").unwrap();
        let back = &body[body.find("0\nLINE\n5\n304").unwrap()..];
        let count = count - 4;
        let points: Vec<_> = (0..count)
            .map(|index| {
                let angle = index as f64 * std::f64::consts::TAU / count as f64;
                [100. * angle.cos() - 250., 100. * angle.sin() + 150.]
            })
            .collect();
        entities(|_| {
            let mut output = String::new();
            for (index,start) in points.iter().enumerate() {
                let end = points[(index+1)%points.len()];
                output.push_str(&format!("0\nLINE\n5\n{:X}\n330\n181\n100\nAcDbEntity\n8\nWF_PIECE_17\n67\n0\n410\nModel\n370\n-1\n100\nAcDbLine\n39\n0\n10\n{}\n20\n{}\n30\n0\n11\n{}\n21\n{}\n31\n0\n", 0x4000+index,start[0],start[1],end[0],end[1]));
            }
            output.push_str(back);
            output
        }).replace("5\nF00", "5\nFFFFFF")
    };
    assert_eq!(
        DxfPatternImport::from_bytes(drawing(4096).as_bytes())
            .unwrap()
            .piece_count(),
        2
    );
    assert!(
        DxfPatternImport::from_bytes(drawing(4097).as_bytes())
            .unwrap_err()
            .0
            .contains("4096-entity limit")
    );
}

#[test]
fn command_wire_fields_are_strict_and_file_loading_is_bounded() {
    let command = import(source());
    let wire = serde_json::to_value(&command).unwrap();
    assert_eq!(wire["type"], "import_dxf_patterns");
    assert_eq!(serde_json::from_value::<Command>(wire).unwrap(), command);
    for invalid in [
        r#"{"type":"import_dxf_patterns","data":{"dxf":"x","ignored":1}}"#,
        r#"{"type":"import_dxf_patterns","data":{"dxf":"x","dxf":"y"}}"#,
        r#"{"type":"import_dxf_patterns","data":{}}"#,
    ] {
        assert!(serde_json::from_str::<Command>(invalid).is_err());
    }
    let directory = Directory::new();
    assert!(DxfPatternImport::load(&directory.0).is_err());
    assert!(DxfPatternImport::load(directory.0.join("missing.dxf")).is_err());
    let input = directory.0.join("original.dxf");
    fs::write(&input, ORIGINAL).unwrap();
    assert_eq!(DxfPatternImport::load(&input).unwrap().piece_count(), 2);
    assert_eq!(fs::read(&input).unwrap(), ORIGINAL);
    fs::write(&input, vec![b' '; MAX_DXF_BYTES + 1]).unwrap();
    assert!(DxfPatternImport::load(&input).is_err());
}
