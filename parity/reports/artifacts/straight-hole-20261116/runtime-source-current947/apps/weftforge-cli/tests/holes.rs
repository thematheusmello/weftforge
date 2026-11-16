use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Process, Output},
    sync::atomic::{AtomicU64, Ordering},
};
use weftforge_core::{Command, Editor, Project, recovery_path};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "weftforge-cli-holes-{}-{}",
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

fn run(verb: &str, paths: &[&Path]) -> Output {
    Process::new(env!("CARGO_BIN_EXE_weftforge-cli"))
        .arg(verb)
        .args(paths)
        .output()
        .unwrap()
}

#[test]
fn execute_inspect_export_and_remove_share_stable_hole_identity_and_safe_io() {
    let directory = Directory::new();
    let project = directory.0.join("annulus.weftforge");
    let commands = directory.0.join("commands.json");
    let original =
        include_bytes!("../../../crates/weftforge-core/tests/fixtures/original-cut-r240.weftforge");
    fs::write(&project, original).unwrap();
    let add = json!({"type":"add_hole","data":{"piece":2,"vertices":[[0.08,0.12],[0.08,0.24],[0.16,0.24],[0.16,0.12]]}});
    fs::write(&commands, json!([add]).to_string()).unwrap();
    let result = run("execute", &[&project, &project, &commands]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut expected = Editor::new(Project::from_json(original).unwrap()).unwrap();
    expected
        .execute(serde_json::from_value(add.clone()).unwrap())
        .unwrap();
    assert_eq!(Project::load(&project).unwrap(), expected.project);
    assert_eq!(fs::read(recovery_path(&project)).unwrap(), original);
    let inspection = run("inspect", &[&project]);
    assert!(
        inspection.status.success(),
        "{}",
        String::from_utf8_lossy(&inspection.stderr)
    );
    let inspected: Value = serde_json::from_slice(&inspection.stdout).unwrap();
    assert_eq!(
        inspected["patterns"][0]["holes"],
        json!([{"id":11,"boundary":[16,17,18,19]}])
    );
    assert_eq!(inspected["patterns"][0]["boundary"], json!([7, 8, 9, 10]));
    assert_eq!(inspected["construction"]["curves"][4]["start"], 12);

    let svg = directory.0.join("annulus.svg");
    let obj = directory.0.join("annulus.obj");
    let dxf = directory.0.join("retained.dxf");
    assert!(run("export-svg", &[&project, &svg]).status.success());
    assert!(run("export-obj", &[&project, &obj]).status.success());
    let exported = fs::read_to_string(&svg).unwrap();
    assert_eq!(exported.matches("<path ").count(), 1);
    assert!(exported.contains("fill-rule=\"evenodd\""));
    assert_eq!(
        exported
            .split(" d=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .matches('Z')
            .count(),
        2
    );
    assert!(fs::read_to_string(&obj).unwrap().contains("g piece_2\n"));
    fs::write(&dxf, b"DXF sentinel").unwrap();
    fs::write(recovery_path(&dxf), b"DXF recovery sentinel").unwrap();
    let refused = run("export-dxf", &[&project, &dxf]);
    assert!(!refused.status.success());
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("DXF v1 export does not support holes")
    );
    assert_eq!(fs::read(&dxf).unwrap(), b"DXF sentinel");
    assert_eq!(
        fs::read(recovery_path(&dxf)).unwrap(),
        b"DXF recovery sentinel"
    );
    let primary = fs::read(&project).unwrap();
    let recovery = fs::read(recovery_path(&project)).unwrap();
    let import = run("import-svg", &[&project, &project, &svg]);
    assert!(!import.status.success());
    assert!(
        String::from_utf8_lossy(&import.stderr)
            .contains("compound paths with holes are unsupported")
    );
    for raw in [
        r#"[{"type":"add_hole","data":{"piece":2}}]"#.to_owned(),
        r#"[{"type":"remove_hole","data":{}}]"#.to_owned(),
        r#"[{"type":"remove_hole","data":{"hole":11,"hole":11}}]"#.to_owned(),
        r#"[{"type":"remove_hole","data":{"hole":11,"unknown":true}}]"#.to_owned(),
        r#"[{"type":"remove_hole","data":{"hole":"11"}}]"#.to_owned(),
        r#"[{"type":"remove_hole","data":{"hole":18446744073709551616}}]"#.to_owned(),
        r#"[{"type":"remove_hole","data":{"hole":0}}]"#.to_owned(),
        r#"[{"type":"remove_hole","data":{"hole":11.5}}]"#.to_owned(),
        r#"[{"type":"remove_hole","data":{"hole":11}},{"type":"move_point","data":{"point":3,"position":[1000.1,0]}}]"#.to_owned(),
        format!("{} null", json!([add])),
        " ".repeat(1024 * 1024 + 1),
    ] {
        fs::write(&commands, raw).unwrap();
        let result = run("execute", &[&project, &project, &commands]);
        assert!(!result.status.success());
        assert_eq!(fs::read(&project).unwrap(), primary);
        assert_eq!(fs::read(recovery_path(&project)).unwrap(), recovery);
    }
    fs::write(
        &commands,
        json!([{"type":"remove_hole","data":{"hole":11}}]).to_string(),
    )
    .unwrap();
    assert!(
        run("execute", &[&project, &project, &commands])
            .status
            .success()
    );
    expected.execute(Command::RemoveHole { hole: 11 }).unwrap();
    assert_eq!(Project::load(&project).unwrap(), expected.project);
    assert_eq!(fs::read(recovery_path(&project)).unwrap(), primary);
}
