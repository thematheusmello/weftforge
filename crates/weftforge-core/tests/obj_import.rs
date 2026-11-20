use std::{fs, path::Path};
use weftforge_core::{Command, Editor, MAX_MTL_BYTES, MAX_OBJ_BYTES, ObjPatternImport, Project};

const SQUARE: &str = "o Sleeve\nv 100 20 80\nv 100 20 40\nv 100 60 40\nv 100 60 80\nvt 1 1\nvt 0 1\nvt 0 0\nvt 1 0\nvn 1 0 0\ns 1\nf -4/-2/1 -3/-1/1 -2/-4/1\nf 1/3 3/1 4/2\n";

fn run(
    editor: &mut Editor,
    obj: &str,
    mtl: Option<&str>,
    metres_per_unit: f64,
) -> weftforge_core::Result<()> {
    editor.execute(Command::ImportObjPatterns {
        obj: obj.into(),
        mtl: mtl.map(str::to_owned),
        metres_per_unit,
    })
}

fn fixture(file: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../parity/reports/artifacts")
        .join(file)
}

fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

fn loop_lengths(project: &Project, ids: &[u64]) -> (f64, f64) {
    let mut area = 0.;
    let mut perimeter = 0.;
    for &id in ids {
        let edge = project.curve(id).unwrap();
        let [a, b] = [edge.start, edge.end].map(|id| project.point(id).unwrap().position);
        area += (a[0] * b[1] - a[1] * b[0]) / 2.;
        perimeter += (a[0] - b[0]).hypot(a[1] - b[1]);
    }
    (area, perimeter)
}

#[test]
fn original_pocket_restores_editable_corners_and_cutout() {
    let parsed =
        ObjPatternImport::load(fixture("native-svg-cutout-20261120/Pocket edited.obj"), 1.)
            .unwrap();
    assert_eq!(
        (
            parsed.piece_count(),
            parsed.hole_count(),
            parsed.material_count()
        ),
        (1, 1, 0)
    );
    let mut editor = Editor::default();
    editor.execute(parsed.command()).unwrap();
    let p = &editor.project;
    let panel = &p.pieces[0];
    assert_eq!(
        (panel.boundary.len(), panel.holes[0].boundary.len()),
        (4, 4)
    );
    let (outer, edge) = loop_lengths(p, &panel.boundary);
    let (opening, inner) = loop_lengths(p, &panel.holes[0].boundary);
    near(outer + opening, 0.0756);
    near(edge, 1.2);
    near(inner, 0.42165525060596437);
    assert!(p.mesh().is_ok());
    let corner = p.curve(panel.holes[0].boundary[0]).unwrap().start;
    let mut moved = p.point(corner).unwrap().position;
    moved[0] += 0.002;
    editor
        .execute(Command::MovePoint {
            point: corner,
            position: moved,
        })
        .unwrap();
    let reopened = Project::from_json(&editor.project.to_json().unwrap()).unwrap();
    assert_eq!(reopened.point(corner).unwrap().position, moved);
    assert!(reopened.mesh().is_ok());
}

#[test]
fn original_harbor_keeps_each_placed_panel_and_its_color_in_one_transaction() {
    let parsed =
        ObjPatternImport::load(fixture("native-obj-fabrics-20261120/Harbor.obj"), 1.).unwrap();
    assert_eq!(
        (
            parsed.piece_count(),
            parsed.hole_count(),
            parsed.material_count()
        ),
        (2, 0, 2)
    );
    let cmd = parsed.command();
    let mut editor = Editor::default();
    editor
        .execute(serde_json::from_slice(&serde_json::to_vec(&cmd).unwrap()).unwrap())
        .unwrap();
    let p = &editor.project;
    assert_eq!(p.revision, 1);
    assert!(p.seams.is_empty());
    for (piece, (name, fabric, color, expected)) in p.pieces.iter().zip([
        (
            "piece_2",
            "fabric_1_Cotton_approximation_uncalibrated",
            [0.22, 0.55, 0.76],
            [-1.08, 0.96, 0.24],
        ),
        (
            "piece_31",
            "fabric_68_Terracotta_linen",
            [0.78, 0.25, 0.12],
            [0.36, 0.96, -0.24],
        ),
    ]) {
        assert_eq!(piece.name, name);
        let m = p.material(piece.material).unwrap();
        assert_eq!(m.name, fabric);
        assert_eq!(m.color, color);
        let (area, perimeter) = loop_lengths(p, &piece.boundary);
        near(area, 0.5923);
        near(perimeter, 4.083146221782354);
        let xyz = piece.placement.transform([expected[0], expected[1]]);
        for (x, t) in xyz.into_iter().zip([-0.36, 0.96, expected[2]]) {
            near(x, t);
        }
    }
    assert!(p.mesh().is_ok());
    let expected = p.clone();
    assert!(editor.undo());
    assert!(editor.project.pieces.is_empty());
    assert_eq!(editor.project.materials.len(), 1);
    assert!(editor.redo());
    assert_eq!(editor.project.pieces, expected.pieces);
    assert_eq!(editor.project.materials, expected.materials);
}

#[test]
fn explicit_centimetres_separate_indices_normal_splits_and_uv_seams() {
    let mut editor = Editor::default();
    run(&mut editor, SQUARE, None, 0.01).unwrap();
    let panel = &editor.project.pieces[0];
    assert_eq!(panel.name, "Sleeve");
    near(loop_lengths(&editor.project, &panel.boundary).0, 0.16);
    for (uv, xyz) in [
        ([0., 0.], [1., 0.2, 0.8]),
        ([0.4, 0.], [1., 0.2, 0.4]),
        ([0., 0.4], [1., 0.6, 0.8]),
    ] {
        for (actual, expected) in panel.placement.transform(uv).into_iter().zip(xyz) {
            near(actual, expected);
        }
    }
    // The two triangles share positions but an explicit texture seam splits their patterns.
    let split = SQUARE.replace("f 1/3 3/1 4/2", "vt 2 0\nvt 3 1\nvt 2 1\nf 1/-3 3/-2 4/-1");
    let parsed = ObjPatternImport::from_bytes(split.as_bytes(), None, 0.01).unwrap();
    assert_eq!(parsed.piece_count(), 2);
    editor.execute(parsed.command()).unwrap();
    assert_eq!(editor.project.pieces[2].name, "Sleeve 2");
    // A reflected texture chart still remeshes with the physical face pointing +X.
    let reflected = SQUARE
        .replace("vt 1 1", "vt 1 -1")
        .replace("vt 0 1", "vt 0 -1");
    let mut editor = Editor::default();
    run(&mut editor, &reflected, None, 0.01).unwrap();
    let normals = editor.project.mesh().unwrap().normals().unwrap();
    assert!(normals.iter().all(|n| n[0] > 0.999999));
}

#[test]
fn invalid_geometry_indices_and_unsupported_records_preserve_document_and_history() {
    let mut editor = Editor::default();
    run(&mut editor, SQUARE, None, 0.01).unwrap();
    assert!(editor.undo());
    let before = editor.project.clone();
    for bad in [
        SQUARE.replace("v 100 60 80", "v 95 60 80"),  // folded
        SQUARE.replace("vt 0 1", "vt 0 1.1"),         // distorted
        SQUARE.replace("v 100 60 40", "v 100 20 40"), // collapsed triangle
        SQUARE.replace("f 1/3 3/1 4/2", "f 1/3 4/2 3/1"), // same-direction shared edge
        format!("{SQUARE}f 1/3 3/1 4/2\n"),           // third incident triangle
        SQUARE.replace("-4/-2/1", "-4/-2/2"),
        SQUARE.replace("-3/-1/1", "-3//1"),
        SQUARE.replace("-3/-1/1", "-3/0/1"),
        SQUARE.replace("-3/-1/1", "-3/-5/1"),
        SQUARE.replace("vt 0 0", "vt NaN 0"),
        format!("{SQUARE}l 1 2\n"),
        format!("{SQUARE}mtllib ../private.mtl\n"),
    ] {
        assert!(run(&mut editor, &bad, None, 0.01).is_err(), "{bad}");
        assert_eq!(editor.project, before);
    }
    for scale in [0., -1., f64::NAN, f64::INFINITY] {
        assert!(run(&mut editor, SQUARE, None, scale).is_err());
    }
    assert!(editor.redo());
    assert_eq!(editor.project.pieces.len(), 1);
}

#[test]
fn only_declared_sibling_library_and_supported_materials_are_used() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Sleeve.obj");
    let obj = format!("mtllib Selected Sleeve.mtl\nusemtl Blue\n{SQUARE}");
    let mtl = "newmtl Blue\nKa 0 0 0\nKd 0.1 0.2 0.3\nKs 0 0 0\nNs 10\nd 1\nillum 2\n";
    fs::write(&path, &obj).unwrap();
    assert!(ObjPatternImport::load(&path, 0.01).is_err());
    let companion = dir.path().join("Selected Sleeve.mtl");
    fs::write(&companion, mtl).unwrap();
    let parsed = ObjPatternImport::load(&path, 0.01).unwrap();
    assert_eq!(parsed.material_library(), Some("Selected Sleeve.mtl"));
    let mut editor = Editor::default();
    for bad in [
        format!("{mtl}map_Kd sleeve.png\n"),
        format!("{mtl}newmtl Blue\n"),
        format!("{mtl}Kd 0 1 0\n"),
        mtl.replace("0.3", "1.4"),
        mtl.replace("d 1\n", "d 0.8\n"),
    ] {
        assert!(run(&mut editor, &obj, Some(&bad), 0.01).is_err());
    }
    let assigned = obj.replace("f 1/3", "usemtl Red\nf 1/3");
    assert!(
        run(
            &mut editor,
            &assigned,
            Some(&format!("{mtl}newmtl Red\nKd 1 0 0\n")),
            0.01
        )
        .is_err()
    );
    assert!(run(&mut editor, &obj, Some("newmtl Red\nKd 1 0 0"), 0.01).is_err());
    for name in [
        "../x.mtl",
        "D:\\x.mtl",
        "/tmp/x.mtl",
        "one.mtl two.mtl",
        ".hidden.mtl",
    ] {
        assert!(
            run(
                &mut editor,
                &obj.replace("Selected Sleeve.mtl", name),
                Some(mtl),
                0.01
            )
            .is_err()
        );
    }
    assert!(ObjPatternImport::from_bytes(&vec![b' '; MAX_OBJ_BYTES + 1], None, 1.).is_err());
    assert!(
        ObjPatternImport::from_bytes(obj.as_bytes(), Some(&vec![b' '; MAX_MTL_BYTES + 1]), 1.)
            .is_err()
    );
    #[cfg(unix)]
    {
        fs::remove_file(&companion).unwrap();
        fs::write(dir.path().join("outside"), mtl).unwrap();
        std::os::unix::fs::symlink(dir.path().join("outside"), &companion).unwrap();
        assert!(ObjPatternImport::load(&path, 0.01).is_err());
        fs::remove_file(&companion).unwrap();
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&companion)
                .status()
                .unwrap()
                .success()
        );
        assert!(ObjPatternImport::load(&path, 0.01).is_err());
    }
}

#[test]
fn ungrouped_fabric_charts_keep_their_scale_after_repacking_uv_coordinates() {
    use std::fmt::Write;
    let original = fixture("native-obj-fabrics-20261120/Harbor.obj");
    let mut repacked = String::new();
    for line in fs::read_to_string(&original).unwrap().lines() {
        if let Some(pair) = line.strip_prefix("vt ") {
            let p: Vec<f64> = pair
                .split_whitespace()
                .map(|v| v.parse().unwrap())
                .collect();
            writeln!(
                repacked,
                "vt {:.6} {:.6}",
                (p[0] + 2.) * 1000.,
                (p[1] - 1.) * 1000.
            )
            .unwrap();
        } else if !line.starts_with("g ") {
            writeln!(repacked, "{line}").unwrap();
        }
    }
    let library = ObjPatternImport::load(&original, 1.).unwrap();
    let mtl = fs::read(original.with_file_name(library.material_library().unwrap())).unwrap();
    let mut editor = Editor::default();
    editor
        .execute(
            ObjPatternImport::from_bytes(repacked.as_bytes(), Some(&mtl), 1.)
                .unwrap()
                .command(),
        )
        .unwrap();
    for (piece, z) in editor.project.pieces.iter().zip([0.24, -0.24]) {
        let fabric = editor.project.material(piece.material).unwrap();
        assert_eq!(piece.name, fabric.name);
        near(loop_lengths(&editor.project, &piece.boundary).0, 0.5923);
        for &id in &piece.boundary {
            let edge = editor.project.curve(id).unwrap();
            let uv = editor.project.point(edge.start).unwrap().position;
            near(piece.placement.transform(uv)[2], z);
        }
    }
}

#[test]
fn two_openings_survive_tracing_and_remeshing() {
    use std::fmt::Write;
    let mut obj = String::new();
    for y in 0..=3 {
        for x in 0..=5 {
            writeln!(obj, "v {x} {y} 0\nvt {x} {y}").unwrap();
        }
    }
    for y in 0..3 {
        for x in 0..5 {
            if y == 1 && matches!(x, 1 | 3) {
                continue;
            }
            let a = 1 + 6 * y + x;
            let (b, c, d) = (a + 1, a + 7, a + 6);
            writeln!(obj, "f {a}/{a} {b}/{b} {c}/{c}\nf {a}/{a} {c}/{c} {d}/{d}").unwrap();
        }
    }
    let mut editor = Editor::default();
    run(&mut editor, &obj, None, 0.1).unwrap();
    let p = &editor.project;
    assert_eq!(p.pieces.len(), 1);
    let panel = &p.pieces[0];
    assert_eq!(panel.holes.len(), 2);
    let areas: Vec<_> = panel
        .boundary_loops()
        .map(|ring| loop_lengths(p, ring).0)
        .collect();
    near(areas[0], 0.15);
    near(areas[1], -0.01);
    near(areas[2], -0.01);
    assert!(p.mesh().is_ok());
}
