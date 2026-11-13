use std::{error::Error, fs, path::Path, process::Command};
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
    std::env::set_current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())?;
    match std::env::args().nth(1).as_deref() {
        Some("progress") => call("python3", &["scripts/progress.py"]),
        Some("verify") => {
            call("python3", &["scripts/progress.py", "--check"])?;
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
        Some("package") => {
            call(
                "cargo",
                &[
                    "build",
                    "--release",
                    "-p",
                    "weftforge",
                    "-p",
                    "weftforge-cli",
                ],
            )?;
            let suffix = std::env::consts::EXE_SUFFIX;
            let dest = Path::new("target/package");
            fs::create_dir_all(dest)?;
            for name in ["weftforge", "weftforge-cli"] {
                fs::copy(
                    format!("target/release/{name}{suffix}"),
                    dest.join(format!("{name}{suffix}")),
                )?;
            }
            for name in [
                "README.md",
                "LICENSE-MIT",
                "LICENSE-APACHE",
                "THIRD_PARTY.md",
            ] {
                fs::copy(name, dest.join(name))?;
            }
            if cfg!(target_os = "macos") {
                let app = dest.join("WeftForge.app/Contents");
                fs::create_dir_all(app.join("MacOS"))?;
                fs::copy("target/release/weftforge", app.join("MacOS/weftforge"))?;
                fs::write(
                    app.join("Info.plist"),
                    r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>CFBundleExecutable</key><string>weftforge</string><key>CFBundleIdentifier</key><string>org.weftforge.studio</string><key>CFBundleName</key><string>WeftForge</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleShortVersionString</key><string>0.1.0</string><key>NSHighResolutionCapable</key><true/><key>LSMinimumSystemVersion</key><string>14.0</string></dict></plist>"#,
                )?;
            }
            println!(
                "Packaged unsigned binaries at target/package; see tested-platform limitations."
            );
            Ok(())
        }
        _ => {
            println!("cargo xtask progress | verify | scenario | benchmark | package");
            Ok(())
        }
    }
}
