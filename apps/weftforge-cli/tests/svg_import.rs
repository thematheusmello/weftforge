use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use weftforge_core::{Project, recovery_path};

const ORIGINAL: &[u8] =
    include_bytes!("../../../crates/weftforge-core/tests/fixtures/original-editable-patterns.svg");

struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn cli_import_svg_edits_once_reports_scale_and_preserves_files_after_failures() {
    let directory = Directory(std::env::temp_dir().join(format!(
            "weftforge-svg-cli-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&directory.0).unwrap();
    let input = directory.0.join("source.weftforge");
    let output = directory.0.join("imported.weftforge");
    let svg = directory.0.join("original.svg");
    Project::default().save(&input).unwrap();
    Project::default().save(&output).unwrap();
    fs::write(&svg, ORIGINAL).unwrap();
    let binary = env!("CARGO_BIN_EXE_weftforge-cli");
    assert!(
        String::from_utf8(Command::new(binary).arg("--help").output().unwrap().stdout)
            .unwrap()
            .contains("import-svg INPUT OUTPUT PATTERNS.svg")
    );
    let run = |input: &std::path::Path, output: &std::path::Path, svg: &std::path::Path| {
        Command::new(binary)
            .arg("import-svg")
            .arg(input)
            .arg(output)
            .arg(svg)
            .output()
            .unwrap()
    };
    let result = run(&input, &output, &svg);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(receipt["revision"], 1);
    assert_eq!(receipt["pieces_imported"], 2);
    assert_eq!(receipt["metres_per_user_unit"], 0.002);
    assert!(
        receipt["scope"]
            .as_str()
            .unwrap()
            .contains("default 3D placement")
    );
    let imported = Project::load(&output).unwrap();
    assert_eq!(imported.pieces[0].name, "Original & curved front");
    assert_eq!(imported.points[0].position, [-0.16, 0.32]);
    assert_eq!(
        imported.curves[1].controls,
        Some([[0.08, 0.32], [0.08, 0.2]])
    );
    let old_output = fs::read(&output).unwrap();
    let old_recovery = fs::read(recovery_path(&output)).unwrap();
    let old_input = fs::read(&input).unwrap();
    for invalid in [
        String::from_utf8(ORIGINAL.to_vec())
            .unwrap()
            .replace("600mm", "600px"),
        "<svg>".into(),
        String::from_utf8(ORIGINAL.to_vec())
            .unwrap()
            .replace("</svg>", "<circle r=\"10\"/></svg>"),
    ] {
        fs::write(&svg, invalid).unwrap();
        let invalid_source = fs::read(&svg).unwrap();
        assert!(!run(&input, &output, &svg).status.success());
        assert_eq!(fs::read(&svg).unwrap(), invalid_source);
        assert_eq!(fs::read(&output).unwrap(), old_output);
        assert_eq!(fs::read(recovery_path(&output)).unwrap(), old_recovery);
        assert_eq!(fs::read(&input).unwrap(), old_input);
    }
    fs::write(&svg, ORIGINAL).unwrap();
    assert!(!run(&input, &svg, &svg).status.success());
    assert_eq!(fs::read(&svg).unwrap(), ORIGINAL);
    let blocked = directory.0.join("invalid-output.weftforge");
    fs::write(&blocked, b"preserve invalid destination").unwrap();
    assert!(!run(&input, &blocked, &svg).status.success());
    assert_eq!(fs::read(&blocked).unwrap(), b"preserve invalid destination");
    let bad_input = directory.0.join("invalid-input.weftforge");
    fs::write(&bad_input, b"invalid input").unwrap();
    assert!(!run(&bad_input, &output, &svg).status.success());
    assert_eq!(fs::read(&output).unwrap(), old_output);
    assert_eq!(fs::read(recovery_path(&output)).unwrap(), old_recovery);
    let recovery_source = directory.0.join("protected.weftforge.recovery");
    fs::write(&recovery_source, ORIGINAL).unwrap();
    assert!(
        !run(
            &input,
            &directory.0.join("protected.weftforge"),
            &recovery_source
        )
        .status
        .success()
    );
    assert_eq!(fs::read(&recovery_source).unwrap(), ORIGINAL);
    #[cfg(unix)]
    {
        let alias = directory.0.join("alias.weftforge");
        std::os::unix::fs::symlink(&svg, &alias).unwrap();
        assert!(!run(&input, &alias, &svg).status.success());
        assert_eq!(fs::read(&svg).unwrap(), ORIGINAL);
    }
    let commands = directory.0.join("commands.json");
    fs::write(
        &commands,
        serde_json::to_vec(&vec![weftforge_core::Command::ImportSvgPatterns {
            svg: String::from_utf8(ORIGINAL.to_vec()).unwrap(),
        }])
        .unwrap(),
    )
    .unwrap();
    let result = Command::new(binary)
        .arg("execute")
        .arg(&input)
        .arg(&input)
        .arg(&commands)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(Project::load(&input).unwrap(), imported);
    assert_eq!(fs::read(&svg).unwrap(), ORIGINAL);
}
