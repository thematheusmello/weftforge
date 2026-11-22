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
        mode: weftforge_core::ObjImportMode::Flat,
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

fn posed(path: &Path) -> Editor {
    let parsed =
        ObjPatternImport::load_mode(path, 1., weftforge_core::ObjImportMode::Posed).unwrap();
    let mut editor = Editor::default();
    editor.execute(parsed.command()).unwrap();
    editor
}

fn swatch(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/posed-obj")
        .join(name)
}

// Independent OBJ readback. Keep both winding and connectivity while allowing
// vertex/face renumbering. Coordinate bins are one tenth of the 1 nm budget.
fn oriented_faces(obj: &str) -> Vec<[[i64; 3]; 3]> {
    let mut points = Vec::new();
    let mut faces = Vec::new();
    for line in obj.lines() {
        let mut fields = line.split_whitespace();
        match fields.next() {
            Some("v") => points.push(std::array::from_fn(|_| {
                (fields.next().unwrap().parse::<f64>().unwrap() * 1e10).round() as i64
            })),
            Some("f") => {
                let mut triangle: [[i64; 3]; 3] = std::array::from_fn(|_| {
                    let i = fields
                        .next()
                        .unwrap()
                        .split('/')
                        .next()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    points[i - 1]
                });
                let start = (0..3).min_by_key(|&i| triangle[i]).unwrap();
                triangle.rotate_left(start);
                faces.push(triangle);
            }
            _ => {}
        }
    }
    faces.sort_unstable();
    faces
}

#[test]
fn original_posed_surfaces_reopen_and_export_with_source_winding_and_flat_dimensions() {
    let harbor = fixture("native-selected-obj-20261120/Harbor draped.obj");
    let cases = [
        (swatch("Bent swatch.obj"), vec![[0.240, 0.360, 1.200]]),
        (
            swatch("Extended bent swatch.obj"),
            vec![[0.24592681838322228, 0.3688902275748334, 1.2296340919161113]],
        ),
        (
            harbor,
            vec![
                [1.3257072537835786, 0.6427671533496141, 4.100800427633242],
                [1.3257047986551815, 0.6427659629843306, 4.100792833202759],
            ],
        ),
    ];
    for (source, dimensions) in cases {
        assert!(ObjPatternImport::load(&source, 1.).is_err());
        let mut editor = posed(&source);
        let project = editor.project.clone();
        assert!(editor.undo());
        assert!(editor.project.pieces.is_empty());
        assert!(editor.redo());
        assert_eq!(editor.project.pieces, project.pieces);
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("posed.weftforge");
        project.save(&file).unwrap();
        let reopened = Project::load(&file).unwrap();
        assert_eq!(project, reopened);
        assert_eq!(reopened.format_version(), 8);
        assert_eq!(reopened.pieces.len(), dimensions.len());
        for (i, (piece, [w, h, perimeter])) in reopened.pieces.iter().zip(dimensions).enumerate() {
            let [a, b] = reopened.piece_bounds(piece.id).unwrap();
            near(b[0] - a[0], w);
            near(b[1] - a[1], h);
            near(loop_lengths(&reopened, &piece.boundary).1, perimeter);
            if reopened.pieces.len() == 2 {
                let color = reopened.material(piece.material).unwrap().color;
                for (actual, expected) in color
                    .into_iter()
                    .zip([[0.22, 0.55, 0.76], [0.78, 0.25, 0.12]][i])
                {
                    assert!((actual - expected).abs() <= 1e-6);
                }
            }
        }
        let mesh = reopened.mesh().unwrap();
        let output = dir.path().join("direct.obj");
        let mtl = mesh
            .export_obj_with_materials(&reopened, &output)
            .unwrap()
            .unwrap();
        if reopened.pieces.len() == 2 {
            let material = fs::read_to_string(mtl).unwrap();
            assert!(
                material.contains("Kd 0.22 0.55 0.76") && material.contains("Kd 0.78 0.25 0.12")
            );
        }
        assert_eq!(
            oriented_faces(&fs::read_to_string(source).unwrap()),
            oriented_faces(&fs::read_to_string(output).unwrap())
        );
        let cloth = weftforge_core::simulation::create_cloth(&reopened, &mesh).unwrap();
        assert_eq!(cloth.positions(), mesh.positions);
    }
}

#[test]
fn posed_chart_edits_keep_material_rest_independent_of_start_and_undo() {
    let mut editor = posed(&swatch("Extended bent swatch.obj"));
    let id = editor.project.pieces[0].id;
    let mesh = editor.project.mesh().unwrap();
    let mut cloth = weftforge_core::simulation::create_cloth(&editor.project, &mesh).unwrap();
    near(
        cloth.diagnostics().max_stretch_error,
        1.1 / 1.05_f64.sqrt() - 1.,
    );
    cloth.step(Default::default(), || false).unwrap();
    assert_ne!(cloth.positions(), mesh.positions);
    cloth.reset();
    assert_eq!(cloth.positions(), mesh.positions);
    editor
        .execute(Command::ResizePiece {
            piece: id,
            width: 0.48,
            height: 0.72,
        })
        .unwrap();
    let sized = editor.project.mesh().unwrap();
    assert_eq!(sized.positions, mesh.positions);
    assert_ne!(sized.uv, mesh.uv);
    let rotation = weftforge_core::Placement {
        translation: [0.3, 0.6, -0.1],
        rotation: [0.3, -0.4, 0.2],
    };
    editor
        .execute(Command::SetPlacement {
            piece: id,
            placement: rotation,
        })
        .unwrap();
    editor
        .execute(Command::TransformPiece {
            piece: id,
            translation: [0.2, -0.3],
            angle: 0.3,
            pivot: [0.1, 0.2],
        })
        .unwrap();
    let arranged = editor.project.mesh().unwrap();
    for (&p, actual) in mesh.positions.iter().zip(arranged.positions) {
        for (actual, expected) in actual.into_iter().zip(rotation.transform_3d(p)) {
            near(actual, expected);
        }
    }
    let p = &editor.project;
    let point = p.curve(p.pieces[0].boundary[0]).unwrap().start;
    let [x, y] = p.point(point).unwrap().position;
    let command = Command::MovePoint {
        point,
        position: [x + 0.002, y],
    };
    let original = p.clone();
    let error = editor.execute(command.clone()).unwrap_err();
    assert!(error.0.contains("Folded swatch") && error.0.contains("Clear imported pose"));
    assert_eq!(original, editor.project);
    editor
        .execute(Command::Transaction(vec![
            Command::ClearPiecePose { piece: id },
            command,
        ]))
        .unwrap();
    assert!(editor.project.pieces[0].initial_pose.is_none());
    assert!(editor.undo());
    assert_eq!(original.pieces, editor.project.pieces);
    assert_eq!(original.points, editor.project.points);
}

#[test]
fn persisted_pose_requires_valid_chart_and_v8_and_preserves_failed_save() {
    let mut editor = posed(&swatch("Bent swatch.obj"));
    let id = editor.project.pieces[0].material;
    editor
        .execute(Command::SetFabricSurface {
            material: id,
            metallic: 0.15,
            roughness: 0.4,
        })
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let primary = dir.path().join("mixed.weftforge");
    editor.project.save(&primary).unwrap();
    let saved = fs::read(&primary).unwrap();
    assert_eq!(Project::load(&primary).unwrap(), editor.project);
    let good: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    for version in [6, 7] {
        let mut wrong = good.clone();
        wrong["version"] = version.into();
        assert!(Project::from_json(&serde_json::to_vec(&wrong).unwrap()).is_err());
    }
    for triangles in [
        serde_json::json!([[100, 1, 2]]),
        serde_json::json!([[0, 1, 2], [0, 1, 2]]),
        serde_json::json!([[2, 1, 0]]),
    ] {
        let mut wrong = good.clone();
        wrong["project"]["pieces"][0]["initial_pose"]["triangles"] = triangles;
        assert!(
            Project::from_json(&serde_json::to_vec(&wrong).unwrap())
                .unwrap_err()
                .0
                .contains("Folded swatch")
        );
    }
    editor.project.pieces[0].initial_pose.as_mut().unwrap().uv[0] = [f64::NAN, 0.];
    assert!(editor.project.save(&primary).is_err());
    assert_eq!(fs::read(primary).unwrap(), saved);
}

#[test]
fn posed_cutout_has_sewable_boundary_ownership_on_input_topology() {
    let source = fixture("native-svg-cutout-20261120/Pocket edited.obj");
    let editor = posed(&source);
    let p = &editor.project;
    let mesh = p.mesh().unwrap();
    let panel = &p.pieces[0];
    assert_eq!(panel.holes.len(), 1);
    for &curve in panel.boundary_loops().flatten() {
        assert!(
            mesh.boundary
                .iter()
                .any(|s| s.piece == panel.id && s.curve == curve && s.t == 0.)
        );
        assert!(
            mesh.boundary
                .iter()
                .any(|s| s.piece == panel.id && s.curve == curve && s.t == 1.)
        );
    }
}

#[test]
fn a_bad_second_posed_chart_reports_its_panel_and_rolls_back_import() {
    let source = fs::read_to_string(swatch("Bent swatch.obj")).unwrap();
    let bad = format!("{source}\ng Bad_panel\nf 1/1 2/2 5/5\nf 1/1 5/5 4/3\n");
    let mut editor = posed(&swatch("Bent swatch.obj"));
    assert!(editor.undo());
    let before = editor.project.clone();
    let error = editor
        .execute(Command::ImportObjPatterns {
            obj: bad,
            mtl: None,
            metres_per_unit: 1.,
            mode: weftforge_core::ObjImportMode::Posed,
        })
        .unwrap_err();
    assert!(error.0.contains("Bad_panel"), "{error}");
    assert_eq!(before, editor.project);
    assert!(editor.redo());
}
