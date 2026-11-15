use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use weftforge_core::{Command as Edit, CurveInterval, Editor, Project};

#[test]
fn cli_edits_splits_and_joins_sewn_curves_and_preserves_outputs_on_failure() {
    let dir = std::env::temp_dir().join(format!(
        "weftforge-cli-curves-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let mut editor = Editor::default();
    for (name, x) in [("Front", 0.), ("Back", 1.)] {
        editor
            .execute(Edit::AddRectangle {
                name: name.into(),
                origin: [x, 0.],
                width: 0.4,
                height: 0.6,
            })
            .unwrap();
    }
    let a = editor.project.pieces[0].boundary[2];
    let b = editor.project.pieces[1].boundary[2];
    editor
        .execute(Edit::AddSeam {
            name: "Shoulder".into(),
            a: CurveInterval::full(a),
            b: CurveInterval::reversed(b),
            correspondence: vec![[0., 0.], [0.4, 0.65], [1., 1.]],
        })
        .unwrap();
    let input = dir.join("original.weftforge");
    let output = dir.join("edited.weftforge");
    editor.project.save(&input).unwrap();
    let binary = env!("CARGO_BIN_EXE_weftforge-cli");
    let curve = a.to_string();
    let run = |command: &str, input: &std::path::Path, values: &[&str]| {
        Command::new(binary)
            .arg(command)
            .arg(input)
            .arg(&output)
            .arg(&curve)
            .args(values)
            .output()
            .unwrap()
    };
    let result = run("curve-controls", &input, &["0.3", "0.8", "0.1", "0.8"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let controls = Project::load(&output).unwrap();
    assert_eq!(
        controls.curve(a).unwrap().controls,
        Some([[0.3, 0.8], [0.1, 0.8]])
    );
    assert_eq!(controls.seams, editor.project.seams);
    let cubic = dir.join("cubic.weftforge");
    fs::copy(&output, &cubic).unwrap();
    let result = run("split", &cubic, &["0.4"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let split = Project::load(&output).unwrap();
    assert_eq!(split.curves.len(), controls.curves.len() + 1);
    assert_eq!(split.seams.len(), 2);
    assert_eq!(split.seams[0].id, controls.seams[0].id);
    assert_eq!(split.revision, controls.revision + 1);
    let split_path = dir.join("split.weftforge");
    fs::copy(&output, &split_path).unwrap();
    let result = run("join", &split_path, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let joined = Project::load(&output).unwrap();
    assert_eq!(joined.curves.len(), controls.curves.len());
    assert_eq!(joined.points, controls.points);
    assert_eq!(joined.pieces, controls.pieces);
    assert_eq!(joined.revision, split.revision + 1);
    assert_eq!(joined.seams.len(), split.seams.len());
    for (before, after) in controls
        .curve(a)
        .unwrap()
        .controls
        .unwrap()
        .into_iter()
        .flatten()
        .zip(
            joined
                .curve(a)
                .unwrap()
                .controls
                .unwrap()
                .into_iter()
                .flatten(),
        )
    {
        assert!((before - after).abs() < 1e-12);
    }
    for (before, after) in split.seams.iter().zip(&joined.seams) {
        assert_eq!(
            (before.id, &before.correspondence),
            (after.id, &after.correspondence)
        );
        assert_eq!(after.a.curve, a);
    }
    let valid = fs::read(&output).unwrap();
    for (command, values) in [
        ("split", vec!["0"]),
        ("split", vec!["NaN"]),
        ("join", vec![]),
        ("join", vec!["unexpected"]),
        ("curve-controls", vec!["NaN", "0", "0", "0"]),
        ("curve-controls", vec!["0.3", "-1", "0.1", "-1"]),
        ("curve-controls", vec!["straight", "unexpected"]),
    ] {
        let result = run(command, &cubic, &values);
        assert!(!result.status.success());
        assert_eq!(fs::read(&output).unwrap(), valid);
    }
    let result = run("curve-controls", &cubic, &["straight"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let straight = Project::load(&output).unwrap();
    assert_eq!(straight.curve(a).unwrap().controls, None);
    assert_eq!(straight.seams, controls.seams);
    let result = Command::new(binary)
        .arg("inspect")
        .arg(&output)
        .output()
        .unwrap();
    assert!(result.status.success());
    let inspection: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(inspection["construction"]["curves"][2]["id"], a);
    let result = Command::new(binary).arg("--help").output().unwrap();
    let help = String::from_utf8(result.stdout).unwrap();
    assert!(help.contains("curve-controls INPUT OUTPUT CURVE_ID straight"));
    assert!(help.contains("split INPUT OUTPUT CURVE_ID T"));
    assert!(help.contains("join INPUT OUTPUT CURVE_ID"));
    assert!(help.contains("export-dxf PROJECT OUTPUT.dxf"));
    assert_eq!(Project::load(&input).unwrap(), editor.project);
    fs::remove_dir_all(dir).unwrap();
}
