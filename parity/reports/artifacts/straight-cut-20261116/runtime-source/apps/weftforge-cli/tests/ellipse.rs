use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use weftforge_core::Project;

#[test]
fn cli_ellipse_creates_inspects_and_preserves_files_after_invalid_input() {
    let directory = std::env::temp_dir().join(format!(
        "weftforge-cli-ellipse-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    let input = directory.join("empty.weftforge");
    let output = directory.join("ellipse.weftforge");
    let binary = env!("CARGO_BIN_EXE_weftforge-cli");
    let help = Command::new(binary).arg("--help").output().unwrap();
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .contains("ellipse INPUT OUTPUT NAME X Y WIDTH HEIGHT")
    );
    assert!(
        Command::new(binary)
            .arg("new")
            .arg(&input)
            .output()
            .unwrap()
            .status
            .success()
    );
    let original_input = fs::read(&input).unwrap();
    let run = |input: &std::path::Path, values: &[&str]| {
        Command::new(binary)
            .arg("ellipse")
            .arg(input)
            .arg(&output)
            .arg("Original CLI ellipse")
            .args(values)
            .output()
            .unwrap()
    };
    let result = run(&input, &["-0.4", "0.2", "0.8", "0.6"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let saved = Project::load(&output).unwrap();
    assert_eq!(saved.revision, 1);
    assert_eq!(
        saved.piece_bounds(saved.pieces[0].id).unwrap(),
        [[-0.4, 0.2], [0.4, 0.8]]
    );
    assert!(saved.curves.iter().all(|c| c.controls.is_some()));
    let inspected = Command::new(binary)
        .arg("inspect")
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        inspected.status.success(),
        "{}",
        String::from_utf8_lossy(&inspected.stderr)
    );
    let inspected: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert_eq!(inspected["units"], "m");
    assert_eq!(
        inspected["construction"]["points"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        inspected["construction"]["curves"],
        serde_json::json!(saved.curves)
    );
    let commands = directory.join("ellipse.json");
    fs::write(&commands,r#"[{"type":"add_ellipse","data":{"name":"Original JSON ellipse","origin":[1,0],"width":0.3,"height":0.4}}]"#).unwrap();
    let result = Command::new(binary)
        .arg("execute")
        .arg(&output)
        .arg(&output)
        .arg(&commands)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let second = Project::load(&output).unwrap();
    assert_eq!((second.revision, second.pieces.len()), (2, 2));
    let previous_output = fs::read(&output).unwrap();
    let recovery = weftforge_core::recovery_path(&output);
    let previous_recovery = fs::read(&recovery).unwrap();
    for values in [
        vec!["0", "0", "0", "0.4"],
        vec!["0", "0", "-1", "0.4"],
        vec!["0", "0", "NaN", "0.4"],
        vec!["0", "0", "inf", "0.4"],
        vec!["0", "0", "1e309", "0.4"],
        vec!["999.9", "0", "0.3", "0.4"],
        vec!["0", "0", "0.3"],
        vec!["0", "0", "0.3", "0.4", "extra"],
    ] {
        assert!(
            !run(&input, &values).status.success(),
            "accepted {values:?}"
        );
        assert_eq!(fs::read(&output).unwrap(), previous_output);
        assert_eq!(fs::read(&input).unwrap(), original_input);
        assert_eq!(fs::read(&recovery).unwrap(), previous_recovery);
    }
    let malformed = directory.join("invalid.weftforge");
    fs::write(
        &malformed,
        r#"{"format":"weftforge","version":99,"project":{}}"#,
    )
    .unwrap();
    assert!(!run(&malformed, &["0", "0", "0.3", "0.4"]).status.success());
    assert_eq!(fs::read(&output).unwrap(), previous_output);
    assert_eq!(Project::load(&output).unwrap(), second);
    assert_eq!(fs::read(&recovery).unwrap(), previous_recovery);
    fs::remove_dir_all(directory).unwrap();
}
