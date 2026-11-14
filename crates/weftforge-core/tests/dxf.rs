use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use weftforge_core::{
    Command, Editor, Placement, Project, recovery_path, simulation::create_cloth,
};

type Tags = [(u16, String)];
static FILE_ID: AtomicU64 = AtomicU64::new(0);

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "weftforge-dxf-{}-{}",
            std::process::id(),
            FILE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn read_tags(path: &Path) -> Vec<(u16, String)> {
    let text = fs::read_to_string(path).unwrap();
    assert!(text.is_ascii());
    assert!(!text.contains('\r'));
    assert!(text.ends_with('\n'));
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(lines.len() % 2, 0);
    lines
        .chunks_exact(2)
        .map(|pair| (pair[0].parse().unwrap(), pair[1].to_owned()))
        .collect()
}

fn section<'a>(tags: &'a Tags, name: &str) -> &'a Tags {
    let start = tags
        .windows(2)
        .position(|pair| pair[0] == (0, "SECTION".into()) && pair[1] == (2, name.into()))
        .unwrap()
        + 2;
    let length = tags[start..]
        .iter()
        .position(|tag| *tag == (0, "ENDSEC".into()))
        .unwrap();
    &tags[start..start + length]
}

fn records(tags: &Tags) -> Vec<&Tags> {
    let mut starts: Vec<_> = tags
        .iter()
        .enumerate()
        .filter_map(|(index, tag)| (tag.0 == 0).then_some(index))
        .collect();
    starts.push(tags.len());
    starts
        .windows(2)
        .map(|pair| &tags[pair[0]..pair[1]])
        .collect()
}

fn values(tags: &Tags, code: u16) -> Vec<&str> {
    tags.iter()
        .filter_map(|tag| (tag.0 == code).then_some(tag.1.as_str()))
        .collect()
}

fn value(tags: &Tags, code: u16) -> &str {
    let found = values(tags, code);
    assert_eq!(found.len(), 1, "code {code}: {tags:?}");
    found[0]
}

fn point(tags: &Tags, code: u16) -> [f64; 3] {
    [code, code + 10, code + 20].map(|code| value(tags, code).parse().unwrap())
}

fn mixed_scene() -> Editor {
    let mut editor = Editor::default();
    for (name, origin) in [
        ("Frente /<>\\:*?\"| ç", [-0.4000000000000123, -0.2]),
        ("後ろ + overlap", [-0.1, 0.1]),
    ] {
        editor
            .execute(Command::AddRectangle {
                name: name.into(),
                origin,
                width: 0.5,
                height: 0.7,
            })
            .unwrap();
    }
    let curve = editor.project.pieces[0].boundary[0];
    editor
        .execute(Command::SetCurveControls {
            curve,
            controls: Some([
                [-0.31234567890123456, -0.2578912345678912],
                [-0.07123456789012346, -0.22345678901234567],
            ]),
        })
        .unwrap();
    editor
}

#[test]
fn exact_authored_boundaries_and_piece_order_survive_ascii_mm_export() {
    let directory = Directory::new();
    let mut editor = mixed_scene();
    // Storage order is not stable identity or the boundary's authored order.
    editor.project.pieces.reverse();
    editor.project.curves.reverse();
    editor.project.points.reverse();
    let original = editor.project.clone();
    let a = directory.path("a.dxf");
    let b = directory.path("different-name.dxf");
    editor.project.export_dxf(&a).unwrap();
    editor.project.export_dxf(&b).unwrap();
    assert_eq!(fs::read(&a).unwrap(), fs::read(&b).unwrap());
    assert_eq!(editor.project, original);
    let text = fs::read_to_string(&a).unwrap();
    for name in original.pieces.iter().map(|piece| &piece.name) {
        assert!(!text.contains(name));
    }
    assert!(text.contains("WeftForge FILE-005-v1: pattern boundaries in millimetres"));
    assert!(text.contains(&format!("Source document revision: {}", original.revision)));
    let tags = read_tags(&a);
    let entities = records(section(&tags, "ENTITIES"));
    assert_eq!(entities.len(), 8);
    assert_eq!(
        entities.iter().filter(|e| value(e, 0) == "SPLINE").count(),
        1
    );
    let mut pieces: Vec<_> = original.pieces.iter().collect();
    pieces.sort_unstable_by_key(|piece| piece.id);
    let mut index = 0;
    for piece in pieces {
        let mut ends = Vec::new();
        let mut starts = Vec::new();
        for id in &piece.boundary {
            let curve = original.curve(*id).unwrap();
            let expected = |id| {
                let [u, v] = original.point(id).unwrap().position;
                [u * 1000.0, v * 1000.0, 0.0]
            };
            let record = entities[index];
            index += 1;
            assert_eq!(value(record, 8), format!("WF_PIECE_{}", piece.id));
            assert_eq!(value(record, 330), "11");
            assert_eq!(value(record, 67), "0");
            assert_eq!(value(record, 410), "Model");
            assert_eq!(value(record, 370), "-1");
            assert!(values(record, 62).is_empty());
            assert!(values(record, 6).is_empty());
            let (start, end) = if let Some([a, b]) = curve.controls {
                assert_eq!(value(record, 0), "SPLINE");
                assert_eq!(values(record, 100), ["AcDbEntity", "AcDbSpline"]);
                for (code, expected) in [(70, "8"), (71, "3"), (72, "8"), (73, "4"), (74, "0")] {
                    assert_eq!(value(record, code), expected);
                }
                assert_eq!(values(record, 40), ["0", "0", "0", "0", "1", "1", "1", "1"]);
                assert_eq!(point(record, 210), [0., 0., 1.]);
                for code in [41, 11, 12, 13] {
                    assert!(values(record, code).is_empty());
                }
                let controls: Vec<[f64; 3]> = values(record, 10)
                    .into_iter()
                    .zip(values(record, 20))
                    .zip(values(record, 30))
                    .map(|((x, y), z)| [x, y, z].map(|v| v.parse().unwrap()))
                    .collect();
                assert_eq!(
                    controls,
                    [
                        expected(curve.start),
                        [a[0] * 1000., a[1] * 1000., 0.],
                        [b[0] * 1000., b[1] * 1000., 0.],
                        expected(curve.end),
                    ]
                );
                (controls[0], controls[3])
            } else {
                assert_eq!(value(record, 0), "LINE");
                assert_eq!(values(record, 100), ["AcDbEntity", "AcDbLine"]);
                assert_eq!(value(record, 39), "0");
                (point(record, 10), point(record, 11))
            };
            assert_eq!(start, expected(curve.start));
            assert_eq!(end, expected(curve.end));
            starts.push(start);
            ends.push(end);
        }
        for index in 0..ends.len() {
            assert_eq!(ends[index], starts[(index + 1) % starts.len()]);
        }
    }
}

#[test]
fn r2000_infrastructure_has_resolved_unique_handles_counts_and_layout_links() {
    let directory = Directory::new();
    let editor = mixed_scene();
    let path = directory.path("structure.dxf");
    editor.project.export_dxf(&path).unwrap();
    let tags = read_tags(&path);
    let sections: Vec<_> = tags
        .windows(2)
        .filter(|pair| pair[0] == (0, "SECTION".into()))
        .map(|pair| pair[1].1.as_str())
        .collect();
    assert_eq!(
        sections,
        [
            "HEADER", "CLASSES", "TABLES", "BLOCKS", "ENTITIES", "OBJECTS"
        ]
    );
    assert_eq!(tags.last().unwrap(), &(0, "EOF".into()));
    assert!(section(&tags, "CLASSES").is_empty());
    let header = section(&tags, "HEADER");
    for (name, code, expected) in [
        ("$ACADVER", 1, "AC1015"),
        ("$INSUNITS", 70, "4"),
        ("$MEASUREMENT", 70, "1"),
        ("$LUNITS", 70, "2"),
    ] {
        let index = header
            .iter()
            .position(|tag| *tag == (9, name.into()))
            .unwrap();
        assert_eq!(header[index + 1], (code, expected.into()));
    }
    let mut handles = BTreeSet::new();
    let all: Vec<_> = ["TABLES", "BLOCKS", "ENTITIES", "OBJECTS"]
        .into_iter()
        .flat_map(|name| records(section(&tags, name)))
        .collect();
    for record in &all {
        if value(record, 0) == "ENDTAB" {
            continue;
        }
        let code = if value(record, 0) == "DIMSTYLE" {
            105
        } else {
            5
        };
        let handle = u64::from_str_radix(value(record, code), 16).unwrap();
        assert!(handle > 0 && handles.insert(handle), "{record:?}");
        assert!(values(record, if code == 5 { 105 } else { 5 }).is_empty());
    }
    let seed = u64::from_str_radix(value(header, 5), 16).unwrap();
    assert!(seed > *handles.last().unwrap());
    for record in &all {
        for code in [330, 340, 350, 360] {
            for reference in values(record, code) {
                let reference = u64::from_str_radix(reference, 16).unwrap();
                assert!(reference == 0 || handles.contains(&reference), "{record:?}");
            }
        }
    }
    let tables = records(section(&tags, "TABLES"));
    let names: Vec<_> = tables
        .iter()
        .filter(|r| value(r, 0) == "TABLE")
        .map(|r| value(r, 2))
        .collect();
    assert_eq!(
        names,
        [
            "VPORT",
            "LTYPE",
            "LAYER",
            "STYLE",
            "VIEW",
            "UCS",
            "APPID",
            "DIMSTYLE",
            "BLOCK_RECORD"
        ]
    );
    let mut table = None;
    let mut count = 0;
    for record in &tables {
        match value(record, 0) {
            "TABLE" => {
                table = Some(*record);
                count = 0;
                assert!(values(record, 100).contains(&"AcDbSymbolTable"));
            }
            "ENDTAB" => assert_eq!(value(table.unwrap(), 70).parse::<usize>().unwrap(), count),
            kind => {
                let owner = table.unwrap();
                assert_eq!(kind, value(owner, 2));
                assert_eq!(value(record, 330), value(owner, 5));
                assert!(values(record, 100).contains(&"AcDbSymbolTableRecord"));
                count += 1;
            }
        }
    }
    let layers: Vec<_> = tables.iter().filter(|r| value(r, 0) == "LAYER").collect();
    let mut expected = vec!["0".to_owned()];
    expected.extend(
        editor
            .project
            .pieces
            .iter()
            .map(|p| format!("WF_PIECE_{}", p.id)),
    );
    assert_eq!(
        layers.iter().map(|r| value(r, 2)).collect::<Vec<_>>(),
        expected
    );
    for layer in layers {
        assert_eq!(value(layer, 70), "0");
        assert_eq!(value(layer, 62), "7");
        assert_eq!(value(layer, 6), "CONTINUOUS");
        assert!(values(layer, 100).contains(&"AcDbLayerTableRecord"));
    }
    let objects = records(section(&tags, "OBJECTS"));
    assert_eq!(objects.len(), 5);
    assert_eq!(values(objects[0], 3), ["ACAD_GROUP", "ACAD_LAYOUT"]);
    assert_eq!(
        values(objects[0], 350),
        [value(objects[1], 5), value(objects[2], 5)]
    );
    assert_eq!(values(objects[2], 3), ["Model", "Layout1"]);
    assert_eq!(
        values(objects[2], 350),
        [value(objects[3], 5), value(objects[4], 5)]
    );
    let spaces: Vec<_> = tables
        .iter()
        .filter(|r| value(r, 0) == "BLOCK_RECORD")
        .collect();
    let blocks = records(section(&tags, "BLOCKS"));
    assert_eq!(blocks.len(), 4);
    for (index, space) in spaces.iter().enumerate() {
        for code in [70, 280, 281] {
            assert!(values(space, code).is_empty());
        }
        let layout = objects[index + 3];
        assert_eq!(value(space, 340), value(layout, 5));
        assert_eq!(values(layout, 330), [value(objects[2], 5), value(space, 5)]);
        assert_eq!(values(layout, 100), ["AcDbPlotSettings", "AcDbLayout"]);
        assert!(values(layout, 331).is_empty());
        let block = blocks[index * 2];
        let end = blocks[index * 2 + 1];
        assert_eq!(value(block, 0), "BLOCK");
        assert_eq!(value(end, 0), "ENDBLK");
        assert_eq!(value(block, 2), value(space, 2));
        assert_eq!(value(block, 330), value(space, 5));
        assert_eq!(value(end, 330), value(space, 5));
    }
}

#[test]
fn known_dimensions_and_geometry_ignore_placement_resolution_and_simulation() {
    let directory = Directory::new();
    let mut editor = Editor::default();
    editor
        .execute(Command::AddRectangle {
            name: "500 by 700 millimetres".into(),
            origin: [0., 0.],
            width: 0.5,
            height: 0.7,
        })
        .unwrap();
    let path = directory.path("rectangle.dxf");
    editor.project.export_dxf(&path).unwrap();
    let before = read_tags(&path);
    let entities = records(section(&before, "ENTITIES"));
    assert_eq!(entities.len(), 4);
    assert_eq!(point(entities[0], 10), [0., 0., 0.]);
    assert_eq!(point(entities[0], 11), [500., 0., 0.]);
    assert_eq!(point(entities[1], 11), [500., 700., 0.]);
    assert_eq!(point(entities[3], 11), [0., 0., 0.]);
    editor
        .execute(Command::Transaction(vec![
            Command::SetPlacement {
                piece: editor.project.pieces[0].id,
                placement: Placement {
                    translation: [0.7, 1.2, -0.3],
                    rotation: [0.4, -0.3, 0.2],
                },
            },
            Command::SetResolution { metres: 0.2 },
        ]))
        .unwrap();
    let mesh = editor.project.mesh().unwrap();
    let mut cloth = create_cloth(&editor.project, &mesh).unwrap();
    let positions = cloth.positions().to_vec();
    cloth.step(Default::default(), || false).unwrap();
    assert_ne!(cloth.positions(), positions);
    let snapshot = editor.project.clone();
    editor.project.export_dxf(&path).unwrap();
    assert_eq!(editor.project, snapshot);
    let after = read_tags(&path);
    assert_eq!(section(&before, "ENTITIES"), section(&after, "ENTITIES"));
}

#[test]
fn empty_project_is_valid_and_failed_exports_preserve_project_history_and_files() {
    let directory = Directory::new();
    let empty = directory.path("empty.dxf");
    Project::default().export_dxf(&empty).unwrap();
    let tags = read_tags(&empty);
    assert!(section(&tags, "ENTITIES").is_empty());
    let mut editor = mixed_scene();
    let source = directory.path("project.weftforge");
    editor.project.save(&source).unwrap();
    let source_bytes = fs::read(&source).unwrap();
    let output = directory.path("existing.dxf");
    fs::write(&output, b"prior destination").unwrap();
    let recovery = recovery_path(&output);
    let autosave = output.with_extension("dxf.autosave");
    fs::write(&recovery, b"prior recovery").unwrap();
    fs::write(&autosave, b"prior autosave").unwrap();
    let snapshot = editor.project.clone();
    let mut expected_history = editor.clone();
    let mut invalid = snapshot.clone();
    invalid.points[0].position[0] = f64::INFINITY;
    assert!(invalid.export_dxf(&output).is_err());
    invalid = snapshot.clone();
    invalid.pieces[0].boundary.reverse();
    assert!(invalid.export_dxf(&output).is_err());
    invalid = snapshot.clone();
    invalid.curves[0].controls = Some([[f64::NAN, 0.], [0., 0.]]);
    assert!(invalid.export_dxf(&output).is_err());
    assert!(
        editor
            .project
            .export_dxf(directory.path("absent/output.dxf"))
            .is_err()
    );
    assert!(editor.project.export_dxf(&directory.0).is_err());
    assert!(editor.project.export_dxf(&source).is_err());
    #[cfg(unix)]
    {
        let alias = directory.path("nonregular.dxf");
        std::os::unix::fs::symlink(&output, &alias).unwrap();
        assert!(editor.project.export_dxf(&alias).is_err());
        assert_eq!(fs::read_link(alias).unwrap(), output);
    }
    assert_eq!(fs::read(&output).unwrap(), b"prior destination");
    assert_eq!(fs::read(&recovery).unwrap(), b"prior recovery");
    assert_eq!(fs::read(&autosave).unwrap(), b"prior autosave");
    assert_eq!(fs::read(&source).unwrap(), source_bytes);
    assert_eq!(editor.project, snapshot);
    assert_eq!(editor.undo(), expected_history.undo());
    assert_eq!(editor.project, expected_history.project);
    assert_eq!(editor.redo(), expected_history.redo());
    assert_eq!(editor.project, expected_history.project);
    assert!(fs::read_dir(&directory.0).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")
    }));
}
