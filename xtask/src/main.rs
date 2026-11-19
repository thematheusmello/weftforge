use serde::Deserialize;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn call(program: &str, args: &[&str]) -> Result<()> {
    if Command::new(program).args(args).status()?.success() {
        Ok(())
    } else {
        Err(format!("{program} {} failed", args.join(" ")).into())
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("xtask: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let current = std::env::current_dir()?;
    std::env::set_current_dir(workspace_root(&current)?)?;
    match std::env::args().nth(1).as_deref() {
        Some("progress") => call("python3", &["scripts/progress.py"]),
        Some("verify") => {
            call("python3", &["scripts/progress.py", "--check"])?;
            call("python3", &["spec/inventory/verify_implementation_map.py"])?;
            call(
                "python3",
                &[
                    "-m",
                    "unittest",
                    "discover",
                    "-s",
                    "scripts",
                    "-p",
                    "test_*.py",
                ],
            )?;
            call(
                "python3",
                &[
                    "-m",
                    "unittest",
                    "discover",
                    "-s",
                    "spec/evaluators",
                    "-p",
                    "test_*.py",
                ],
            )?;
            call("cargo", &["run", "-p", "weftforge-cli", "--", "--help"])?;
            Ok(())
        }
        Some("scenario") => {
            fs::create_dir_all("target/scenarios")?;
            let project = "target/scenarios/shirt.weftforge";
            call(
                "cargo",
                &["run", "-p", "weftforge-cli", "--", "sample", project],
            )?;
            call(
                "cargo",
                &[
                    "run",
                    "-p",
                    "weftforge-cli",
                    "--",
                    "export-obj",
                    project,
                    "target/scenarios/shirt.obj",
                ],
            )?;
            call(
                "cargo",
                &[
                    "run",
                    "-p",
                    "weftforge-cli",
                    "--",
                    "export-svg",
                    project,
                    "target/scenarios/shirt.svg",
                ],
            )?;
            call(
                "python3",
                &[
                    "scripts/check_exports.py",
                    "target/scenarios/shirt.obj",
                    "target/scenarios/shirt.svg",
                ],
            )
        }
        Some("benchmark") => call(
            "cargo",
            &[
                "run",
                "--release",
                "-p",
                "weftforge-cli",
                "--",
                "benchmark",
                "fixtures/original-shirt.weftforge",
                "120",
            ],
        ),
        Some("package") => package(Path::new(".")),
        _ => {
            println!("cargo xtask progress | verify | scenario | benchmark | package");
            Ok(())
        }
    }
}

const BIN_NAMES: [&str; 2] = ["weftforge", "weftforge-cli"];

#[derive(Deserialize)]
struct BuildMessage {
    reason: String,
    target: Option<BuildTarget>,
    executable: Option<PathBuf>,
    message: Option<BuildDiagnostic>,
}
#[derive(Deserialize)]
struct BuildTarget {
    name: String,
    kind: Vec<String>,
}
#[derive(Deserialize)]
struct BuildDiagnostic {
    rendered: Option<String>,
}

fn package(root: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--locked",
            "--message-format=json-render-diagnostics",
            "-p",
            "weftforge",
            "-p",
            "weftforge-cli",
            "--bin",
            "weftforge",
            "--bin",
            "weftforge-cli",
        ])
        .current_dir(root)
        .stderr(Stdio::inherit())
        .output()?;
    package_build(root, &output.stdout, output.status.success())
}

fn reported_executables(stdout: &[u8], succeeded: bool) -> Result<[PathBuf; 2]> {
    let mut artifacts: [Option<PathBuf>; 2] = [None, None];
    for (index, line) in stdout.split(|&b| b == b'\n').enumerate() {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let message: BuildMessage = serde_json::from_slice(line)
            .map_err(|e| format!("invalid Cargo build message on line {}: {e}", index + 1))?;
        if let Some(BuildDiagnostic {
            rendered: Some(rendered),
        }) = message.message
        {
            eprint!("{rendered}");
        }
        if succeeded
            && message.reason == "compiler-artifact"
            && let Some(target) = message.target
            && target.kind.iter().any(|kind| kind == "bin")
            && let Some(slot) = BIN_NAMES.iter().position(|&name| name == target.name)
        {
            let path = message
                .executable
                .ok_or_else(|| format!("Cargo did not report an executable for {}", target.name))?;
            if artifacts[slot].replace(path).is_some() {
                return Err(format!("Cargo reported duplicate {} executables", target.name).into());
            }
        }
    }
    if !succeeded {
        return Err("cargo build --release failed".into());
    }
    let [studio, cli] = artifacts;
    Ok([
        studio.ok_or("Cargo did not report the weftforge executable")?,
        cli.ok_or("Cargo did not report the weftforge-cli executable")?,
    ])
}

fn package_build(root: &Path, stdout: &[u8], succeeded: bool) -> Result<()> {
    let artifacts = reported_executables(stdout, succeeded)?.map(|path| {
        if path.is_absolute() {
            path
        } else {
            root.join(path)
        }
    });
    for path in &artifacts {
        if !path.is_file() {
            return Err(format!("Cargo reported a missing executable: {}", path.display()).into());
        }
    }
    let suffix = std::env::consts::EXE_SUFFIX;
    let dest = root.join("target/package");
    fs::create_dir_all(&dest)?;
    for (name, path) in BIN_NAMES.into_iter().zip(&artifacts) {
        fs::copy(path, dest.join(format!("{name}{suffix}")))?;
    }
    for name in [
        "README.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "THIRD_PARTY.md",
    ] {
        fs::copy(root.join(name), dest.join(name))?;
    }
    let font_notices = ["OFL-Inter.txt", "OFL-JetBrainsMono.txt"];
    fs::create_dir_all(dest.join("licenses"))?;
    for name in font_notices {
        fs::copy(
            root.join(format!("apps/weftforge/assets/fonts/{name}")),
            dest.join("licenses").join(name),
        )?;
    }
    if cfg!(target_os = "macos") {
        let app = dest.join("WeftForge.app/Contents");
        fs::create_dir_all(app.join("MacOS"))?;
        fs::copy(&artifacts[0], app.join("MacOS/weftforge"))?;
        fs::create_dir_all(app.join("Resources/licenses"))?;
        for name in font_notices {
            fs::copy(
                root.join(format!("apps/weftforge/assets/fonts/{name}")),
                app.join("Resources/licenses").join(name),
            )?;
        }
        fs::write(
            app.join("Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>CFBundleExecutable</key><string>weftforge</string><key>CFBundleIdentifier</key><string>org.weftforge.studio</string><key>CFBundleName</key><string>WeftForge</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleShortVersionString</key><string>0.1.0</string><key>NSHighResolutionCapable</key><true/><key>LSMinimumSystemVersion</key><string>14.0</string></dict></plist>"#,
        )?;
    }
    println!("Packaged unsigned binaries at target/package; see tested-platform limitations.");
    Ok(())
}

fn workspace_root(start: &Path) -> Result<&Path> {
    start
        .ancestors()
        .find(|root| {
            root.join("Cargo.toml").is_file()
                && root.join("parity/features.yaml").is_file()
                && root.join("apps/weftforge/Cargo.toml").is_file()
        })
        .ok_or_else(|| "Run xtask from within a WeftForge checkout".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn package_fixture() -> PathBuf {
        static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "weftforge-xtask-package-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir_all(root.join("target/release")).unwrap();
        fs::create_dir_all(root.join("isolated output")).unwrap();
        fs::create_dir_all(root.join("apps/weftforge/assets/fonts")).unwrap();
        for name in [
            "README.md",
            "LICENSE-MIT",
            "LICENSE-APACHE",
            "THIRD_PARTY.md",
        ] {
            fs::write(root.join(name), name).unwrap();
        }
        for name in ["OFL-Inter.txt", "OFL-JetBrainsMono.txt"] {
            fs::write(root.join("apps/weftforge/assets/fonts").join(name), name).unwrap();
        }
        root
    }

    fn artifact(name: &str, path: &Path) -> serde_json::Value {
        serde_json::json!({
            "reason": "compiler-artifact", "target": {"name": name, "kind": ["bin"]},
            "executable": path, "fresh": true
        })
    }

    fn messages(values: &[serde_json::Value]) -> Vec<u8> {
        values
            .iter()
            .map(|v| format!("{v}\n"))
            .collect::<String>()
            .into_bytes()
    }

    #[test]
    fn packages_reported_executables_even_when_default_target_contains_stale_binaries() {
        let root = package_fixture();
        let studio = root.join("isolated output/current-studio");
        let cli = root.join("isolated output/current-cli");
        fs::write(&studio, b"selected studio").unwrap();
        fs::write(&cli, b"selected CLI").unwrap();
        for name in ["weftforge", "weftforge-cli"] {
            fs::write(
                root.join(format!(
                    "target/release/{name}{}",
                    std::env::consts::EXE_SUFFIX
                )),
                b"stale default target",
            )
            .unwrap();
        }
        let output = messages(&[
            serde_json::json!({"reason": "compiler-artifact", "target": {"name": "weftforge", "kind": ["lib"]}, "executable": null}),
            artifact("weftforge", &studio),
            artifact("weftforge-cli", &cli),
            serde_json::json!({"reason": "build-finished", "success": true}),
        ]);
        package_build(&root, &output, true).unwrap();
        let dest = root.join("target/package");
        assert_eq!(
            fs::read(dest.join(format!("weftforge{}", std::env::consts::EXE_SUFFIX))).unwrap(),
            b"selected studio"
        );
        assert_eq!(
            fs::read(dest.join(format!("weftforge-cli{}", std::env::consts::EXE_SUFFIX))).unwrap(),
            b"selected CLI"
        );
        assert_eq!(fs::read(dest.join("README.md")).unwrap(), b"README.md");
        assert_eq!(
            fs::read(dest.join("licenses/OFL-Inter.txt")).unwrap(),
            b"OFL-Inter.txt"
        );
        if cfg!(target_os = "macos") {
            assert_eq!(
                fs::read(dest.join("WeftForge.app/Contents/MacOS/weftforge")).unwrap(),
                b"selected studio"
            );
            assert_eq!(
                fs::read(
                    dest.join("WeftForge.app/Contents/Resources/licenses/OFL-JetBrainsMono.txt")
                )
                .unwrap(),
                b"OFL-JetBrainsMono.txt"
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_or_incomplete_build_reports_do_not_copy_any_files() {
        let root = package_fixture();
        let studio = root.join("isolated output/current-studio");
        let cli = root.join("isolated output/current-cli");
        fs::write(&studio, b"new studio").unwrap();
        fs::write(&cli, b"new CLI").unwrap();
        let dest = root.join("target/package");
        fs::create_dir_all(&dest).unwrap();
        let packaged = dest.join(format!("weftforge{}", std::env::consts::EXE_SUFFIX));
        fs::write(&packaged, b"previous package").unwrap();
        let a = artifact("weftforge", &studio);
        let b = artifact("weftforge-cli", &cli);
        let missing_file = artifact("weftforge-cli", &root.join("missing executable"));
        for (output, succeeded) in [
            (messages(&[a.clone(), b.clone()]), false),
            (messages(std::slice::from_ref(&a)), true),
            (messages(&[a.clone(), a.clone(), b.clone()]), true),
            (messages(&[a.clone(), b.clone(), b]), true),
            (messages(&[a, missing_file]), true),
        ] {
            assert!(package_build(&root, &output, succeeded).is_err());
            assert_eq!(fs::read(&packaged).unwrap(), b"previous package");
            assert_eq!(fs::read_dir(&dest).unwrap().count(), 1);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selects_the_nearest_runtime_checkout_and_rejects_outside_directories() {
        let temp = std::env::temp_dir().join(format!(
            "weftforge-xtask-root-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let outer = temp.join("outer");
        let inner = outer.join(".local/worktrees/inner");
        for root in [&outer, &inner] {
            fs::create_dir_all(root.join("parity")).unwrap();
            fs::create_dir_all(root.join("apps/weftforge/src")).unwrap();
            for marker in [
                "Cargo.toml",
                "parity/features.yaml",
                "apps/weftforge/Cargo.toml",
            ] {
                fs::write(root.join(marker), b"test checkout marker").unwrap();
            }
        }
        assert_eq!(workspace_root(&outer).unwrap(), outer);
        assert_eq!(
            workspace_root(&inner.join("apps/weftforge/src")).unwrap(),
            inner
        );
        assert!(workspace_root(&temp).is_err());
        fs::remove_dir_all(temp).unwrap();
    }
}
