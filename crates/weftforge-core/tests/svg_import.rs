use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use weftforge_core::{
    Command, Editor, MAX_OBJECTS, MAX_SVG_BYTES, Placement, Project, SvgPatternImport,
};

const ORIGINAL: &[u8] = include_bytes!("fixtures/original-editable-patterns.svg");

fn drawing(data: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="100mm" height="100mm" viewBox="0 0 100 100"><path d="{data}"/></svg>"#
    )
}

fn import(svg: impl Into<String>) -> Command {
    Command::ImportSvgPatterns { svg: svg.into() }
}

fn close(actual: [f64; 2], expected: [f64; 2]) {
    assert!(
        (actual[0] - expected[0]).hypot(actual[1] - expected[1]) <= 1e-12,
        "{actual:?} != {expected:?}"
    );
}

fn assert_svg_canvas(svg: &str, expected: [f64; 4]) {
    let root = svg.lines().find(|line| line.starts_with("<svg ")).unwrap();
    let attribute = |name| {
        root.split_once(&format!("{name}=\""))
            .unwrap()
            .1
            .split_once('"')
            .unwrap()
            .0
    };
    let view_box = attribute("viewBox")
        .split_whitespace()
        .map(|number| number.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(view_box.len(), 4);
    for (actual, expected) in view_box.into_iter().zip(expected).chain(
        ["width", "height"]
            .into_iter()
            .zip([expected[2], expected[3]])
            .map(|(name, expected)| {
                (
                    attribute(name)
                        .strip_suffix("mm")
                        .unwrap()
                        .parse::<f64>()
                        .unwrap(),
                    expected,
                )
            }),
    ) {
        assert!(
            actual.is_finite() && (actual - expected).abs() <= 1e-9,
            "canvas {actual} != {expected}"
        );
    }
}

fn history(editor: &Editor) -> [Vec<Project>; 2] {
    let mut backward = editor.clone();
    let mut forward = editor.clone();
    let mut undo = Vec::new();
    let mut redo = Vec::new();
    while backward.undo() {
        undo.push(backward.project.clone());
    }
    while forward.redo() {
        redo.push(forward.project.clone());
    }
    [undo, redo]
}

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "weftforge-svg-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
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
fn original_svg_preserves_physical_geometry_names_layout_and_one_editable_transaction() {
    let parsed = SvgPatternImport::from_bytes(ORIGINAL).unwrap();
    assert_eq!(parsed.piece_count(), 2);
    assert_eq!(parsed.metres_per_unit, 0.002);
    let mut editor = Editor::default();
    editor
        .execute(Command::AddRectangle {
            name: "Existing pattern".into(),
            origin: [-0.5, 0.],
            width: 0.2,
            height: 0.3,
        })
        .unwrap();
    let before = editor.project.clone();
    editor.execute(parsed.command()).unwrap();
    let project = &editor.project;
    assert_eq!(project.revision, before.revision + 1);
    assert_eq!(&project.pieces[..1], before.pieces);
    assert_eq!(&project.points[..4], before.points);
    assert_eq!(&project.curves[..4], before.curves);
    assert_eq!(project.pieces[1].name, "Original & curved front");
    assert_eq!(project.pieces[2].name, "back-panel");
    assert_eq!(project.points.len(), 12);
    assert_eq!(project.curves.len(), 12);
    let expected = [
        [-0.16, 0.32],
        [0.04, 0.32],
        [0.04, 0.2],
        [-0.16, 0.2],
        [0.12, 0.3],
        [0.32, 0.3],
        [0.32, 0.1],
        [0.12, 0.1],
    ];
    for (point, expected) in project.points[4..].iter().zip(expected) {
        close(point.position, expected);
    }
    let curved = &project.curves[5];
    assert_eq!(curved.controls, Some([[0.08, 0.32], [0.08, 0.2]]));
    assert!(
        project
            .curves
            .iter()
            .filter(|curve| curve.id != curved.id)
            .all(|curve| curve.controls.is_none())
    );
    for t in [0.0_f64, 0.125, 0.5, 0.875, 1.0] {
        // Independently specified power-basis polynomial for the original SVG C.
        close(
            curved.evaluate(project, t).unwrap(),
            [
                0.04 + 0.12 * t - 0.12 * t * t,
                0.32 - 0.36 * t * t + 0.24 * t * t * t,
            ],
        );
    }
    for piece in &project.pieces[1..] {
        assert_eq!(
            (piece.material, piece.grain_angle, piece.placement),
            (1, 0., Placement::default())
        );
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
    assert!(
        serde_json::to_value(&editor.project).unwrap()["next_id"]
            .as_u64()
            .unwrap()
            > serde_json::to_value(&before).unwrap()["next_id"]
                .as_u64()
                .unwrap()
    );
    assert!(editor.redo());
    assert_eq!(editor.project.points, authored.points);
    assert_eq!(editor.project.curves, authored.curves);
    assert_eq!(editor.project.pieces, authored.pieces);
    let changed = editor.project.curves[5].id;
    editor
        .execute(Command::SetCurveControls {
            curve: changed,
            controls: Some([[0.06, 0.32], [0.06, 0.2]]),
        })
        .unwrap();
    assert_eq!(
        editor.project.curve(changed).unwrap().controls,
        Some([[0.06, 0.32], [0.06, 0.2]])
    );
}

#[test]
fn own_export_retains_explicit_closing_cubics_and_absolute_origin_without_flattening() {
    let mut original = Editor::default();
    original
        .execute(Command::Transaction(vec![
            Command::AddEllipse {
                name: "Original <ellipse & curved>".into(),
                origin: [-0.37, 0.21],
                width: 0.8,
                height: 0.6,
            },
            Command::AddRectangle {
                name: "Other panel".into(),
                origin: [0.7, -0.2],
                width: 0.4,
                height: 0.3,
            },
        ]))
        .unwrap();
    let directory = Directory::new();
    let svg = directory.0.join("export.svg");
    original.project.export_svg(&svg).unwrap();
    let mut imported = Editor::default();
    imported
        .execute(SvgPatternImport::load(&svg).unwrap().command())
        .unwrap();
    assert_eq!(imported.project.pieces.len(), 2);
    assert_eq!(imported.project.curves.len(), 8);
    assert_eq!(imported.project.points.len(), 8);
    for (a, b) in imported.project.points.iter().zip(&original.project.points) {
        close(a.position, b.position);
    }
    for (a, b) in imported.project.curves.iter().zip(&original.project.curves) {
        assert_eq!(a.controls.is_some(), b.controls.is_some());
        if let (Some(a), Some(b)) = (a.controls, b.controls) {
            for (a, b) in a.into_iter().zip(b) {
                close(a, b);
            }
        }
    }
    for (a, b) in imported.project.pieces.iter().zip(&original.project.pieces) {
        assert_eq!(a.name, b.name);
    }
}

#[test]
fn export_canvas_preserves_negative_large_offset_and_multiple_piece_layouts() {
    let directory = Directory::new();
    for (name, rectangles, expected) in [
        (
            "negative",
            vec![([-2., -3.], 0.24, 0.36)],
            [-2000., 2640., 240., 360.],
        ),
        (
            "large-offset",
            vec![([999., 999.], 0.24, 0.36)],
            [999000., -999360., 240., 360.],
        ),
        (
            "multiple-pieces",
            vec![([-2., -3.], 0.24, 0.36), ([2., 3.], 0.4, 0.5)],
            [-2000., -3500., 4400., 6500.],
        ),
    ] {
        let mut editor = Editor::default();
        for (origin, width, height) in rectangles {
            editor
                .execute(Command::AddRectangle {
                    name: name.into(),
                    origin,
                    width,
                    height,
                })
                .unwrap();
        }
        let path = directory.0.join(format!("{name}.svg"));
        editor.project.export_svg(&path).unwrap();
        assert_svg_canvas(&fs::read_to_string(&path).unwrap(), expected);
        let mut imported = Editor::default();
        imported
            .execute(SvgPatternImport::load(&path).unwrap().command())
            .unwrap();
        assert_eq!(imported.project.points.len(), editor.project.points.len());
        for (actual, expected) in imported.project.points.iter().zip(&editor.project.points) {
            close(actual.position, expected.position);
        }
        assert_eq!(imported.project.pieces.len(), editor.project.pieces.len());
    }
}

#[test]
fn export_canvas_includes_true_cubic_extrema_without_control_hull_padding() {
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Cubic bulge".into(),
            vertices: vec![[0., 0.], [1., 0.], [1., -1.], [0., -1.]],
        })
        .unwrap();
    editor
        .execute(Command::SetCurveControls {
            curve: editor.project.pieces[0].boundary[0],
            controls: Some([[0., 1.], [1., 1.]]),
        })
        .unwrap();
    let directory = Directory::new();
    let path = directory.0.join("cubic.svg");
    editor.project.export_svg(&path).unwrap();
    let svg = fs::read_to_string(&path).unwrap();
    // V(t) = 3t(1-t): the true maximum is .75, between endpoints0 and controls1.
    assert_svg_canvas(&svg, [0., -750., 1000., 1750.]);
    assert!(svg.contains("M 0 -0 C 0 -1000 1000 -1000 1000 -0 L 1000 1000"));
}

#[test]
fn empty_export_keeps_the_one_metre_blank_canvas() {
    let directory = Directory::new();
    let path = directory.0.join("empty.svg");
    Editor::default().project.export_svg(&path).unwrap();
    let svg = fs::read_to_string(&path).unwrap();
    assert_svg_canvas(&svg, [0., -1000., 1000., 1000.]);
    assert!(!svg.contains("<path "));
}

#[test]
fn rejected_svg_preserves_project_stable_ids_and_undo_redo_history() {
    let valid = drawing("M0 0L100 0 100 100 0 100Z");
    let mut editor = Editor::default();
    editor.execute(import(valid.clone())).unwrap();
    editor
        .execute(Command::SetResolution { metres: 0.05 })
        .unwrap();
    assert!(editor.undo());
    let before = editor.project.to_json().unwrap();
    let old_history = history(&editor);
    let mut invalid = vec![
        valid.replace("100mm", "100"),
        valid.replace("100mm", "100px"),
        valid.replace("100mm", "10cm"),
        valid.replace("100mm", "100%"),
        valid.replacen("100mm", "200mm", 1),
        valid.replacen("100mm", "0mm", 1),
        valid.replacen("100mm", "-1mm", 1),
        valid.replace("viewBox=\"0 0 100 100\"", "viewBox=\"0 0 -100 100\""),
        valid.replace("viewBox=\"0 0 100 100\"", "viewBox=\"0 0 0 100\""),
        valid.replace("viewBox=\"0 0 100 100\"", "viewBox=\"0 0 100 100 5\""),
        valid.replace("viewBox=\"0 0 100 100\"", "viewBox=\"0 0 100 100,\""),
        valid
            .replacen("100mm", "110mm", 1)
            .replacen("100mm", "10mm", 1)
            .replace("viewBox=\"0 0 100 100\"", "viewBox=\"0 0 1.1.1\""),
        valid.replace("http://www.w3.org/2000/svg", "http://invalid.example/svg"),
        valid.replace("xmlns=\"http://www.w3.org/2000/svg\"", ""),
        valid.replace("<path", "<path transform=\"translate(10)\""),
        valid.replace("<path", "<path style=\"display:none\""),
        valid.replace("<path", "<path href=\"http://invalid.example/asset\""),
        valid.replace("<path", "<path fill=\"url(http://invalid.example/asset)\""),
        valid.replace("<path", "<path d=\"M0 0Z\""),
        valid.replace("<path", "<path data-name=\"&unknown;\""),
        valid.replace(
            "</svg>",
            "<rect x=\"0\" y=\"0\" width=\"10\" height=\"10\"/></svg>",
        ),
        valid.replace("</svg>", "<g><path d=\"M0 0L1 0 1 1Z\"/></g></svg>"),
        valid.replace(
            "</svg>",
            "<title><desc><title>Too deep</title></desc></title></svg>",
        ),
        valid.replace("<path", "<path xmlns=\"http://invalid.example/svg\""),
        valid.replace("<path", "<path data-name=\"&#1;\""),
        valid.replace("<path", "<path data-name=\"bad<name\""),
        valid.replace("<path", "<path data-name=\"\""),
        valid.replace("</svg>", ""),
        format!("{valid}{valid}"),
        format!("{valid}trailing"),
        format!("<!DOCTYPE svg [<!ENTITY data 'anything'>]>{valid}"),
        format!("<?xml-stylesheet href=\"http://invalid.example/css\"?>{valid}"),
        format!("<?xml version=\"1.1\"?>{valid}"),
        format!("<?xml version=\"1.0\" encoding=\"UTF-16\"?>{valid}"),
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\" encoding=\"UTF-8\"?>{valid}"),
        format!("<?xml version=\"1.0\" standalone=\"yes\" encoding=\"UTF-8\"?>{valid}"),
        format!("<?xml version=\"1.0\" encoding=\"UTF&#45;8\"?>{valid}"),
        valid.replace("<path", "<path data-name=\"A\" data-name=\"B\""),
        valid.replace(
            "<path",
            &format!(
                "<path {}",
                (0..64).map(|i| format!("a{i}=\"x\" ")).collect::<String>()
            ),
        ),
    ];
    for data in [
        "",
        "M0 0L100 0 100 100 0 100",
        "m0 0L100 0 100 100Z",
        "M0 0H100V100Z",
        "M0 0Q50 10 100 0L100 100Z",
        "M0 0A10 10 0 0 1 100 0L100 100Z",
        "M0 0L100 0 100 100Z M10 10L20 10 20 20Z",
        "M0 0L0 0 100 100Z",
        "M0 0L100 100 0 100 100 0Z",
        "M0 0L100 0 200 0Z",
        "M0 0L1e999 0 100 100Z",
        "M0 0L1e 0 100 100Z",
        "M0 0L100,,0 100 100Z",
        "M0 0L100 0 100 100,Z",
        "M0 0L100 0 100 100ZL0 0",
        "M0 0L1000001 0 0 100Z",
    ] {
        invalid.push(drawing(data));
    }
    for svg in invalid {
        assert!(
            editor.execute(import(svg.clone())).is_err(),
            "accepted {svg}"
        );
        assert_eq!(editor.project.to_json().unwrap(), before);
        assert_eq!(history(&editor), old_history);
    }
    assert!(SvgPatternImport::from_bytes(&[0xff]).is_err());
    assert!(SvgPatternImport::from_bytes(valid.replace("<path", "\0<path").as_bytes()).is_err());
    assert!(
        editor
            .execute(Command::Transaction(vec![
                import(valid),
                Command::MovePoint {
                    point: u64::MAX,
                    position: [0., 0.]
                }
            ]))
            .is_err()
    );
    assert_eq!(editor.project.to_json().unwrap(), before);
    assert_eq!(history(&editor), old_history);
}

#[test]
fn parser_resource_limits_and_native_capacity_revision_id_limits_are_atomic() {
    assert!(SvgPatternImport::from_bytes(&vec![b' '; MAX_SVG_BYTES + 1]).is_err());
    let path = r#"<path d="M0 0L100 0 100 100 0 100Z"/>"#;
    let many = |count| {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100mm" height="100mm" viewBox="0 0 100 100">{}</svg>"#,
            path.repeat(count)
        )
    };
    assert_eq!(
        SvgPatternImport::from_bytes(many(64).as_bytes())
            .unwrap()
            .piece_count(),
        64
    );
    assert!(SvgPatternImport::from_bytes(many(65).as_bytes()).is_err());
    let spans = |count| {
        format!(
            "M0 0{}Z",
            (1..count).map(|i| format!("L{i} 0")).collect::<String>()
        )
    };
    assert!(SvgPatternImport::from_bytes(drawing(&spans(4096)).as_bytes()).is_ok());
    assert!(SvgPatternImport::from_bytes(drawing(&spans(4097)).as_bytes()).is_err());
    let combined =
        drawing(&spans(2049)).replace("</svg>", &format!("<path d=\"{}\"/></svg>", spans(2049)));
    assert!(SvgPatternImport::from_bytes(combined.as_bytes()).is_err());
    for kind in ["ids", "revision", "objects"] {
        let mut project = Project::default();
        match kind {
            "ids" => {}
            "revision" => project.revision = u64::MAX,
            _ => {
                project.materials = (1..=MAX_OBJECTS)
                    .map(|id| {
                        let mut m = project.materials[0].clone();
                        m.id = id as u64;
                        m
                    })
                    .collect();
            }
        }
        let mut value = serde_json::to_value(project).unwrap();
        if kind == "ids" {
            value["next_id"] = serde_json::json!(u64::MAX);
        }
        if kind == "objects" {
            value["next_id"] = serde_json::json!(MAX_OBJECTS + 1);
        }
        let project = serde_json::from_value(value).unwrap();
        let mut editor = Editor::new(project).unwrap();
        let before = editor.project.clone();
        assert!(
            editor
                .execute(import(drawing("M0 0L100 0 100 100 0 100Z")))
                .is_err(),
            "{kind}"
        );
        assert_eq!(editor.project, before);
        assert!(!editor.can_undo() && !editor.can_redo());
    }
}

#[test]
fn shared_command_serde_is_strict_and_preserves_original_svg_bytes() {
    let command = import(String::from_utf8(ORIGINAL.to_vec()).unwrap());
    let value = serde_json::to_value(&command).unwrap();
    assert_eq!(value["type"], "import_svg_patterns");
    assert_eq!(serde_json::from_value::<Command>(value).unwrap(), command);
    for json in [
        r#"{"type":"import_svg_patterns","data":{"svg":"x","ignored":1}}"#,
        r#"{"type":"import_svg_patterns","data":{"svg":"x","svg":"y"}}"#,
        r#"{"type":"import_svg_patterns","data":{}}"#,
    ] {
        assert!(serde_json::from_str::<Command>(json).is_err());
    }
}

#[test]
fn cutout_import_respects_svg_winding_subpath_order_and_atomic_rejection() {
    let outer = "M0 0L100 0 100 100 0 100Z";
    let inner = "M20 20L60 20 60 60 20 60Z";
    let reversed = "M20 20L20 60 60 60 60 20Z";
    let with_rule = |paths: &str, rule: &str| {
        drawing(paths).replace("<path ", &format!("<path fill-rule=\"{rule}\" "))
    };
    for data in [
        with_rule(&format!("{outer}{inner}"), "evenodd"),
        with_rule(&format!("{inner}{outer}"), "evenodd"),
        with_rule(&format!("{reversed}{outer}"), "nonzero"),
        drawing(&format!("{outer}{reversed}")),
    ] {
        let mut editor = Editor::default();
        editor.execute(import(data)).unwrap();
        let piece = &editor.project.pieces[0];
        assert_eq!(piece.boundary.len(), 4);
        assert_eq!(piece.holes.len(), 1);
        assert_eq!(piece.holes[0].boundary.len(), 4);
        assert_eq!(
            editor
                .project
                .point(editor.project.curve(piece.boundary[0]).unwrap().start)
                .unwrap()
                .position,
            [0., 0.]
        );
        assert_eq!(
            editor
                .project
                .point(
                    editor
                        .project
                        .curve(piece.holes[0].boundary[0])
                        .unwrap()
                        .start
                )
                .unwrap()
                .position,
            [0.02, -0.02]
        );
    }

    let mut editor = Editor::default();
    editor.execute(import(drawing(outer))).unwrap();
    editor.execute(import(drawing(inner))).unwrap();
    assert!(editor.undo());
    let before = editor.project.clone();
    let before_history = history(&editor);
    for data in [
        drawing(&format!("{outer}{inner}")),
        with_rule(&format!("{outer}{inner}{reversed}"), "evenodd"),
        with_rule(&format!("{outer}M0 20L50 20 50 60 0 60Z"), "evenodd"),
        with_rule(&format!("{outer}M120 20L160 20 160 60 120 60Z"), "evenodd"),
        with_rule(&format!("{outer}M20 20L60 20 130 60 20 60Z"), "evenodd"),
        with_rule(
            &format!("{outer}M20 20C22 21 50 24 60 20L60 60 20 60Z"),
            "evenodd",
        ),
        with_rule(
            &format!("M0 0C20 0 80 0 100 0L100 100 0 100Z{inner}"),
            "evenodd",
        ),
    ] {
        let two_pieces = data.replace("<path ", &format!("<path d=\"{outer}\"/><path "));
        assert!(editor.execute(import(two_pieces)).is_err());
        assert_eq!(editor.project, before);
        assert_eq!(history(&editor), before_history);
    }
}
