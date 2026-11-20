use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use weftforge_core::{Command, Editor, Project, SvgPatternImport, recovery_path};

fn annulus() -> Editor {
    let mut editor = Editor::default();
    editor
        .execute(Command::AddRectangle {
            name: "Original <R240 & hole>".into(),
            origin: [0., 0.],
            width: 0.24,
            height: 0.36,
        })
        .unwrap();
    editor
        .execute(Command::AddHole {
            piece: 2,
            vertices: vec![[0.08, 0.12], [0.08, 0.24], [0.16, 0.24], [0.16, 0.12]],
        })
        .unwrap();
    editor
}

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "weftforge-hole-io-{}-{}",
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
fn strict_v5_retains_hole_identity_and_rejects_incomplete_or_ambiguous_ownership() {
    let editor = annulus();
    let bytes = editor.project.to_json().unwrap();
    let document: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(document["version"], weftforge_core::FORMAT_VERSION);
    assert_eq!(
        document["project"]["pieces"][0]["holes"],
        json!([{"id":11,"boundary":[16,17,18,19]}])
    );
    assert_eq!(document["project"]["next_id"], 20);
    assert_eq!(Project::from_json(&bytes).unwrap(), editor.project);

    let mut invalid = Vec::new();
    for field in ["holes", "internal_curves"] {
        let mut bad = document.clone();
        bad["project"]["pieces"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        invalid.push(bad);
    }
    for field in ["id", "boundary"] {
        let mut bad = document.clone();
        bad["project"]["pieces"][0]["holes"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        invalid.push(bad);
    }
    for id in [json!(0), json!(1), json!("11"), json!(11.5), Value::Null] {
        let mut bad = document.clone();
        bad["project"]["pieces"][0]["holes"][0]["id"] = id;
        invalid.push(bad);
    }
    for boundary in [json!([]), json!([16, 17]), json!([7, 8, 9, 10])] {
        let mut bad = document.clone();
        bad["project"]["pieces"][0]["holes"][0]["boundary"] = boundary;
        invalid.push(bad);
    }
    let mut bad = document.clone();
    bad["project"]["pieces"][0]["holes"][0]["unknown"] = json!(true);
    invalid.push(bad);
    let mut bad = document.clone();
    let extra = bad["project"]["pieces"][0]["holes"][0].clone();
    bad["project"]["pieces"][0]["holes"]
        .as_array_mut()
        .unwrap()
        .push(extra);
    invalid.push(bad);
    let mut bad = document.clone();
    bad["project"]["pieces"][0]["holes"][0]["boundary"] = json!([19, 18, 17, 16]);
    for curve in &mut bad["project"]["curves"].as_array_mut().unwrap()[4..] {
        let start = curve["start"].clone();
        curve["start"] = curve["end"].clone();
        curve["end"] = start;
    }
    invalid.push(bad);
    let mut bad = document.clone();
    bad["version"] = json!(4);
    invalid.push(bad);
    for bad in invalid {
        assert!(
            Project::from_json(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{bad}"
        );
    }
    let text = String::from_utf8(bytes).unwrap();
    for bad in [
        text.replacen("\"holes\":", "\"holes\":[],\"holes\":", 1),
        text.replace("\"id\": 11,", "\"id\":11,\"id\":11,"),
        text.replace("\"id\": 11,", "\"boundary\":[16,17,18,19],\"id\":11,"),
        text.replace("\"id\": 11,", "\"id\":18446744073709551616,"),
        text.replace("0.08", "1e309"),
        text.replace("0.08", "1000.1"),
        format!("{text} null"),
    ] {
        assert_ne!(bad, text);
        assert!(Project::from_json(bad.as_bytes()).is_err());
    }
}

#[test]
fn v5_save_recovery_and_early_validation_preserve_exact_files() {
    let directory = Directory::new();
    let path = directory.0.join("annulus.weftforge");
    let mut editor = annulus();
    editor.project.save(&path).unwrap();
    let first = fs::read(&path).unwrap();
    editor
        .execute(Command::MovePoint {
            point: 12,
            position: [0.06, 0.12],
        })
        .unwrap();
    editor.project.save(&path).unwrap();
    assert_eq!(Project::load(&path).unwrap(), editor.project);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), first);
    assert_eq!(Project::recover(&path).unwrap(), annulus().project);
    let primary = fs::read(&path).unwrap();
    let mut invalid = editor.project.clone();
    invalid.pieces[0].holes[0].boundary.pop();
    assert!(invalid.save(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), primary);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), first);
    fs::write(&path, b"invalid primary").unwrap();
    assert!(editor.project.save(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"invalid primary");
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), first);
}

#[test]
fn svg_cutout_round_trips_at_physical_scale_as_one_editable_transaction() {
    let directory = Directory::new();
    let path = directory.0.join("annulus.svg");
    let mut editor = annulus();
    editor
        .execute(Command::AddInternalSegment {
            piece: 2,
            start: [0.02, 0.06],
            end: [0.22, 0.06],
        })
        .unwrap();
    editor.project.export_svg(&path).unwrap();
    let svg = fs::read_to_string(&path).unwrap();
    assert_eq!(svg.matches("<path ").count(), 1);
    assert!(svg.contains("fill-rule=\"evenodd\""));
    assert!(svg.contains("width=\"240mm\" height=\"360mm\" viewBox=\"0 -360 240 360\""));
    assert!(svg.contains("data-name=\"Original &lt;R240 &amp; hole&gt;\""));
    let data = svg
        .split(" d=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let commands = data
        .split_whitespace()
        .filter(|token| matches!(*token, "M" | "L" | "Z"))
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        ["M", "L", "L", "L", "L", "Z", "M", "L", "L", "L", "L", "Z"]
    );
    let coordinates = data
        .split_whitespace()
        .filter_map(|token| token.parse::<f64>().ok())
        .collect::<Vec<_>>();
    assert_eq!(
        coordinates,
        [
            0., 0., 240., 0., 240., -360., 0., -360., 0., 0., 80., -120., 80., -240., 160., -240.,
            160., -120., 80., -120.,
        ]
    );
    let mut imported = Editor::default();
    let empty = imported.project.clone();
    imported
        .execute(SvgPatternImport::load(&path).unwrap().command())
        .unwrap();
    let mut restored = imported.project.clone();
    assert_eq!(restored.pieces.len(), 1);
    assert_eq!(restored.pieces[0].holes.len(), 1);
    assert_eq!(restored.pieces[0].name, editor.project.pieces[0].name);
    let round_trip = directory.0.join("reexport.svg");
    restored.export_svg(&round_trip).unwrap();
    assert_eq!(fs::read_to_string(round_trip).unwrap(), svg);
    assert!(imported.undo());
    assert_eq!(imported.project.pieces, empty.pieces);
    assert_eq!(imported.project.points, empty.points);
    assert_eq!(imported.project.curves, empty.curves);
    assert!(!imported.undo());
    assert!(imported.redo());
    assert_eq!(imported.project.revision, restored.revision + 2);
    restored.revision = imported.project.revision;
    assert_eq!(imported.project, restored);
    assert_eq!(fs::read_to_string(&path).unwrap(), svg);
}

#[test]
fn holed_dxf_refuses_before_staging_and_preserves_destination_and_recovery() {
    let directory = Directory::new();
    let path = directory.0.join("retained.dxf");
    fs::write(&path, b"prior DXF sentinel").unwrap();
    fs::write(recovery_path(&path), b"prior recovery sentinel").unwrap();
    let editor = annulus();
    let before = editor.project.clone();
    for destination in [&path, &directory.0.join("absent/output.dxf")] {
        let error = editor.project.export_dxf(destination).unwrap_err();
        assert!(
            error.0.contains("DXF v1 export does not support holes"),
            "{error}"
        );
    }
    assert_eq!(editor.project, before);
    assert_eq!(fs::read(&path).unwrap(), b"prior DXF sentinel");
    assert_eq!(
        fs::read(recovery_path(&path)).unwrap(),
        b"prior recovery sentinel"
    );
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);
}

#[test]
fn obj_writes_the_actual_annular_rest_mesh_in_one_piece_group() {
    let directory = Directory::new();
    let path = directory.0.join("annulus.obj");
    let editor = annulus();
    let mesh = editor.project.mesh().unwrap();
    mesh.export_obj(&path).unwrap();
    let obj = fs::read_to_string(path).unwrap();
    let mut positions = Vec::new();
    let mut uv = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Vec::new();
    for line in obj.lines() {
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        match tokens[0] {
            "v" => positions.push(std::array::from_fn::<_, 3, _>(|i| {
                tokens[i + 1].parse::<f64>().unwrap()
            })),
            "vt" => uv.push(std::array::from_fn::<_, 2, _>(|i| {
                tokens[i + 1].parse::<f64>().unwrap()
            })),
            "vn" => normals.push(std::array::from_fn::<_, 3, _>(|i| {
                tokens[i + 1].parse::<f64>().unwrap()
            })),
            "f" => faces.push(std::array::from_fn::<_, 3, _>(|i| {
                let indices = tokens[i + 1]
                    .split('/')
                    .map(|id| id.parse::<usize>().unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(indices, [indices[0]; 3]);
                indices[0] - 1
            })),
            "#" | "g" => {}
            _ => panic!("Unexpected OBJ line: {line}"),
        }
    }
    assert!(obj.starts_with("# WeftForge OBJ: metres; Y up; material UV coordinates in metres"));
    assert_eq!(
        obj.lines()
            .filter(|line| line.starts_with("g "))
            .collect::<Vec<_>>(),
        ["g piece_2"]
    );
    assert_eq!(positions.len(), mesh.positions.len());
    assert_eq!(uv.len(), mesh.uv.len());
    for (actual, expected) in positions
        .iter()
        .flatten()
        .zip(mesh.positions.iter().flatten())
    {
        assert!((actual - expected).abs() <= 1e-12);
    }
    for (actual, expected) in uv.iter().flatten().zip(mesh.uv.iter().flatten()) {
        assert!((actual - expected).abs() <= 1e-12);
    }
    assert_eq!(faces, mesh.triangles);
    assert_eq!(normals.len(), positions.len());
    assert!(normals.iter().all(|n| *n == [0., 0., 1.]));
    let area: f64 = faces
        .iter()
        .map(|face| {
            let [a, b, c] = face.map(|i| uv[i]);
            let area = ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) * 0.5;
            assert!(area > 0.);
            area
        })
        .sum();
    assert!((area - 0.0768).abs() <= 1e-10, "{area}");
}
