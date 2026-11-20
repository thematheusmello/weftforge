mod package_notices;

use serde::Deserialize;
use std::{
    collections::BTreeSet,
    error::Error,
    fs, io,
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
    package_id: Option<String>,
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

struct BuiltExecutable {
    id: String,
    path: PathBuf,
}

struct BuildArtifacts {
    binaries: [BuiltExecutable; 2],
    libraries: BTreeSet<String>,
}

struct PackageStage {
    path: PathBuf,
    _lock: fs::File,
    keep_previous: bool,
}

fn package_directory_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => Ok(true),
        Ok(_) => Err(format!("Expected a package directory at {}", path.display()).into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

impl PackageStage {
    fn new(root: &Path) -> Result<Self> {
        let parent = root.join("target");
        fs::create_dir_all(&parent)?;
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(parent.join(".package-lock"))?;
        lock.try_lock()
            .map_err(|error| format!("Could not lock the package output: {error}"))?;
        let path = parent.join(".package-staging");
        if package_directory_exists(&path)? {
            let previous = path.join("previous");
            let destination = parent.join("package");
            if package_directory_exists(&previous)? && !package_directory_exists(&destination)? {
                fs::rename(previous, destination)?;
            }
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir(&path)?;
        Ok(Self {
            path,
            _lock: lock,
            keep_previous: false,
        })
    }

    fn publish(&mut self) -> Result<()> {
        let parent = self
            .path
            .parent()
            .ok_or("Package staging folder needs a parent")?;
        let destination = parent.join("package");
        let next = self.path.join("next");
        let previous = self.path.join("previous");
        let has_previous = package_directory_exists(&destination)?;
        if has_previous {
            fs::rename(&destination, &previous)?;
        }
        if let Err(error) = fs::rename(next, &destination) {
            if has_previous && let Err(restore) = fs::rename(&previous, &destination) {
                self.keep_previous = true;
                return Err(format!(
                    "Could not publish the package ({error}) or restore the previous package ({restore}). Previous files are in {}",
                    previous.display()
                ).into());
            }
            return Err(error.into());
        }
        Ok(())
    }
}

impl Drop for PackageStage {
    fn drop(&mut self) {
        if !self.keep_previous {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn package(root: &Path) -> Result<()> {
    let mut stage = PackageStage::new(root)?;
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
    package_build(
        root,
        &mut stage,
        &output.stdout,
        output.status.success(),
        |build| package_notices::NoticePlan::load(root, build),
    )
}

fn reported_build(stdout: &[u8], succeeded: bool) -> Result<BuildArtifacts> {
    let mut artifacts: [Option<BuiltExecutable>; 2] = [None, None];
    let mut libraries = BTreeSet::new();
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
        if !succeeded || message.reason != "compiler-artifact" {
            continue;
        }
        let Some(target) = message.target else {
            continue;
        };
        if target.kind.iter().any(|kind| {
            matches!(
                kind.as_str(),
                "lib" | "rlib" | "dylib" | "cdylib" | "staticlib"
            )
        }) {
            libraries.insert(
                message
                    .package_id
                    .ok_or("Cargo omitted a library package ID")?,
            );
        } else if target.kind.iter().any(|kind| kind == "bin")
            && let Some(slot) = BIN_NAMES.iter().position(|&name| name == target.name)
        {
            let path = message
                .executable
                .ok_or_else(|| format!("Cargo did not report an executable for {}", target.name))?;
            let id = message
                .package_id
                .ok_or("Cargo omitted an executable package ID")?;
            if artifacts[slot]
                .replace(BuiltExecutable { id, path })
                .is_some()
            {
                return Err(format!("Cargo reported duplicate {} executables", target.name).into());
            }
        }
    }
    if !succeeded {
        return Err("cargo build --release failed".into());
    }
    let [studio, cli] = artifacts;
    Ok(BuildArtifacts {
        binaries: [
            studio.ok_or("Cargo did not report the weftforge executable")?,
            cli.ok_or("Cargo did not report the weftforge-cli executable")?,
        ],
        libraries,
    })
}

fn package_build(
    root: &Path,
    stage: &mut PackageStage,
    stdout: &[u8],
    succeeded: bool,
    plan: impl FnOnce(&BuildArtifacts) -> Result<package_notices::NoticePlan>,
) -> Result<()> {
    let mut build = reported_build(stdout, succeeded)?;
    for artifact in &mut build.binaries {
        if artifact.path.is_relative() {
            artifact.path = root.join(&artifact.path);
        }
        if !artifact.path.is_file() {
            return Err(format!(
                "Cargo reported a missing executable: {}",
                artifact.path.display()
            )
            .into());
        }
    }
    let notices = plan(&build)?;
    let suffix = if notices.target.split('-').any(|part| part == "windows") {
        ".exe"
    } else {
        ""
    };
    let dest = stage.path.join("next");
    fs::create_dir(&dest)?;
    let mut packaged = Vec::new();
    for (name, artifact) in BIN_NAMES.into_iter().zip(&build.binaries) {
        let path = PathBuf::from(format!("{name}{suffix}"));
        fs::copy(&artifact.path, dest.join(&path))?;
        packaged.push(path);
    }
    let project_notices = ["LICENSE-MIT", "LICENSE-APACHE", "THIRD_PARTY.md"];
    for name in ["README.md"].into_iter().chain(project_notices) {
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
    let macos = notices.target.ends_with("-apple-darwin");
    let app = dest.join("WeftForge.app/Contents");
    if macos {
        fs::create_dir_all(app.join("MacOS"))?;
        fs::copy(dest.join(&packaged[0]), app.join("MacOS/weftforge"))?;
        packaged.push(PathBuf::from("WeftForge.app/Contents/MacOS/weftforge"));
        fs::write(
            app.join("Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>CFBundleExecutable</key><string>weftforge</string><key>CFBundleIdentifier</key><string>org.weftforge.studio</string><key>CFBundleName</key><string>WeftForge</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleShortVersionString</key><string>0.1.0</string><key>NSHighResolutionCapable</key><true/><key>LSMinimumSystemVersion</key><string>14.0</string></dict></plist>"#,
        )?;
    }
    let unresolved = notices.write(&dest, &packaged)?;
    if macos {
        let resources = app.join("Resources/licenses");
        let licenses = dest.join("licenses");
        package_notices::copy_notice_tree(&licenses.join("cargo"), &resources.join("cargo"))?;
        for name in project_notices {
            fs::copy(dest.join(name), resources.join(name))?;
        }
        for name in font_notices {
            fs::copy(licenses.join(name), resources.join(name))?;
        }
    }
    stage.publish()?;
    println!("Packaged unsigned binaries at target/package; see tested-platform limitations.");
    if unresolved != 0 {
        eprintln!(
            "Review {unresolved} unresolved Cargo notice entries in target/package/licenses/manifest.json."
        );
    }
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
        let catalog = root.join(package_notices::UPSTREAM_CATALOG);
        fs::create_dir_all(&catalog).unwrap();
        fs::write(catalog.join("index.json"), b"[]").unwrap();
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
            "reason": "compiler-artifact", "package_id": name, "target": {"name": name, "kind": ["bin"]},
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

    fn fixture_graph(root: &Path) -> serde_json::Value {
        let packages: Vec<_> = BIN_NAMES
            .iter()
            .map(|name| {
                serde_json::json!({
                    "id": name, "name": name, "version": "0.1.0", "authors": [],
                    "manifest_path": root.join("Cargo.toml")
                })
            })
            .collect();
        let nodes: Vec<_> = BIN_NAMES
            .iter()
            .map(|name| {
                serde_json::json!({
                    "id": name, "deps": []
                })
            })
            .collect();
        serde_json::json!({
            "packages": packages, "workspace_members": BIN_NAMES, "resolve": {"nodes": nodes},
        })
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
            serde_json::json!({"reason": "compiler-artifact", "package_id": "weftforge", "target": {"name": "weftforge", "kind": ["lib"]}, "executable": null}),
            artifact("weftforge", &studio),
            artifact("weftforge-cli", &cli),
            serde_json::json!({"reason": "build-finished", "success": true}),
        ]);
        let target = if cfg!(target_os = "macos") {
            "aarch64-apple-darwin"
        } else if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc"
        } else {
            "aarch64-unknown-linux-gnu"
        };
        {
            let mut stage = PackageStage::new(&root).unwrap();
            package_build(&root, &mut stage, &output, true, |build| {
                package_notices::NoticePlan::from_graph(
                    &root,
                    target.into(),
                    &serde_json::to_vec(&fixture_graph(&root)).unwrap(),
                    build,
                )
            })
            .unwrap();
        }
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
            let mut stage = PackageStage::new(&root).unwrap();
            assert!(
                package_build(&root, &mut stage, &output, succeeded, |_| {
                    panic!("failed build must stop before collecting notices")
                })
                .is_err()
            );
            assert_eq!(fs::read(&packaged).unwrap(), b"previous package");
            assert_eq!(fs::read_dir(&dest).unwrap().count(), 1);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn packages_only_target_runtime_notices_and_preserves_previous_package_on_notice_failure() {
        let root = package_fixture();
        let studio = root.join("isolated output/current-studio");
        let cli = root.join("isolated output/current-cli");
        fs::write(&studio, b"selected studio").unwrap();
        fs::write(&cli, b"selected CLI").unwrap();
        let dest = root.join("target/package");
        fs::create_dir(&dest).unwrap();
        fs::write(dest.join("weftforge"), b"previous studio").unwrap();
        let mut graph = fixture_graph(&root);
        let mut reported = vec![
            artifact("weftforge", &studio),
            artifact("weftforge-cli", &cli),
        ];
        for (name, edge, kind) in [
            ("runtime", None, Some("lib")),
            ("runtime-pinned", None, Some("lib")),
            ("unresolved", None, Some("lib")),
            ("dev-only", Some("dev"), Some("lib")),
            ("build-only", Some("build"), Some("lib")),
            ("macro", None, Some("proc-macro")),
            ("not-emitted", None, None),
        ] {
            let directory = root.join("crates").join(name);
            fs::create_dir_all(&directory).unwrap();
            let registry = matches!(name, "runtime-pinned" | "unresolved" | "not-emitted");
            if registry {
                fs::write(
                    directory.join(".cargo_vcs_info.json"),
                    b"pinned fixture revision",
                )
                .unwrap();
            } else {
                fs::write(directory.join("LICENSE-MIT"), format!("license for {name}")).unwrap();
            }
            graph["packages"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "id": name, "name": name, "version": "1.2.3",
                    "authors": ["Original author"], "license": "MIT",
                    "source": registry.then_some(package_notices::CRATES_IO),
                    "repository": "https://example.invalid/original",
                    "manifest_path": directory.join("Cargo.toml")
                }));
            graph["resolve"]["nodes"][0]["deps"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "pkg": name, "dep_kinds": [{"kind": edge}]
                }));
            graph["resolve"]["nodes"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "id": name, "deps": []
                }));
            if let Some(kind) = kind {
                reported.push(serde_json::json!({
                    "reason": "compiler-artifact", "package_id": name,
                    "target": {"name": name, "kind": [kind]},
                }));
            }
        }
        fs::write(root.join("crates/runtime/NOTICE.txt"), b"original notice").unwrap();
        graph["packages"][2]["license_file"] = serde_json::json!("license.declared");
        let full_text = b"verified fixture code license";
        let full_digest = "cc4c992b586131273edf726c61c6b55ef517ddde7dc55a7954486758fe1a8d8b";
        let declaration = b"verified fixture license declaration";
        let declaration_digest = "15e3c516a6656a94cfefdcda754316d10fe3ce358c60b86cc7c253fdb0281284";
        let absent_digest = "11".repeat(32);
        let groups: Vec<_> = [
            ("runtime-pinned", "LICENSE.txt", "code_license", full_digest, full_text.len()),
            ("unresolved", "DECLARATION.txt", "license_declaration", declaration_digest, declaration.len()),
            ("not-emitted", "STALE.txt", "code_license", &absent_digest, 1),
        ]
        .into_iter()
        .map(|(crate_name, name, kind, sha256, size_bytes)| serde_json::json!({
            "project": "fixture",
            "crates": [{"name": crate_name, "version": "1.2.3",
                "vcs_info_sha256": "60724c651fbd7eebba39e253b178661a4b438db2473d61278a1964af2b98ffd6"}],
            "documents": [{"name": name, "kind": kind, "sha256": sha256, "size_bytes": size_bytes,
                "url": format!("https://example.invalid/pinned/{name}")}],
        }))
        .collect();
        let catalog = root.join(package_notices::UPSTREAM_CATALOG);
        fs::write(
            catalog.join("index.json"),
            serde_json::to_vec(&groups).unwrap(),
        )
        .unwrap();
        let texts = catalog.join("texts");
        fs::create_dir_all(&texts).unwrap();
        let full_blob = texts.join(format!("{full_digest}.txt"));
        fs::write(texts.join(format!("{declaration_digest}.txt")), declaration).unwrap();
        let result = || {
            let mut stage = PackageStage::new(&root)?;
            package_build(&root, &mut stage, &messages(&reported), true, |build| {
                package_notices::NoticePlan::from_graph(
                    &root,
                    "aarch64-apple-darwin".into(),
                    &serde_json::to_vec(&graph)?,
                    build,
                )
            })
        };
        assert!(result().is_err());
        assert_eq!(
            fs::read(dest.join("weftforge")).unwrap(),
            b"previous studio"
        );
        assert_eq!(fs::read_dir(&dest).unwrap().count(), 1);
        fs::write(
            root.join("crates/runtime/license.declared"),
            b"declared license",
        )
        .unwrap();
        fs::write(&full_blob, b"verified fixture code licensE").unwrap();
        assert!(
            result()
                .unwrap_err()
                .to_string()
                .contains("Digest mismatch for pinned Cargo notice")
        );
        assert_eq!(
            fs::read(dest.join("weftforge")).unwrap(),
            b"previous studio"
        );
        assert_eq!(fs::read_dir(&dest).unwrap().count(), 1);
        fs::write(full_blob, full_text).unwrap();
        result().unwrap();
        assert_eq!(
            fs::read(dest.join("weftforge")).unwrap(),
            b"selected studio"
        );
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dest.join("licenses/manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["target"], "aarch64-apple-darwin");
        assert_eq!(manifest["artifacts"][0]["path"], "weftforge");
        assert_eq!(
            manifest["artifacts"][0]["sha256"],
            "ee2a2a38aaa2a2dc30a7407bedca34680d3759289417d994dc0cd721e2e61f45"
        );
        let crates = manifest["crates"].as_array().unwrap();
        assert_eq!(crates.len(), 3);
        assert_eq!(crates[0]["name"], "runtime");
        assert_eq!(crates[0]["authors"], serde_json::json!(["Original author"]));
        assert_eq!(crates[0]["files"].as_array().unwrap().len(), 3);
        assert_eq!(crates[1]["name"], "runtime-pinned");
        assert_eq!(crates[1]["unresolved"], serde_json::json!([]));
        let pinned = &crates[1]["files"][0];
        assert_eq!(
            pinned["upstream"]["url"],
            "https://example.invalid/pinned/LICENSE.txt"
        );
        assert_eq!(pinned["sha256"], full_digest);
        assert_eq!(crates[2]["name"], "unresolved");
        assert_eq!(crates[2]["unresolved"].as_array().unwrap().len(), 1);
        assert_eq!(
            crates[2]["files"][0]["upstream"]["kind"],
            "license_declaration"
        );
        assert_eq!(
            fs::read(dest.join("licenses/cargo/runtime-1.2.3/NOTICE.txt")).unwrap(),
            b"original notice"
        );
        let bundled = dest.join("WeftForge.app/Contents/Resources/licenses");
        for (path, text) in [
            ("LICENSE-MIT", "LICENSE-MIT"),
            ("LICENSE-APACHE", "LICENSE-APACHE"),
            ("THIRD_PARTY.md", "THIRD_PARTY.md"),
            ("OFL-Inter.txt", "OFL-Inter.txt"),
            ("OFL-JetBrainsMono.txt", "OFL-JetBrainsMono.txt"),
            ("cargo/runtime-1.2.3/NOTICE.txt", "original notice"),
        ] {
            assert_eq!(fs::read(bundled.join(path)).unwrap(), text.as_bytes());
        }
        let pinned_path = pinned["path"]
            .as_str()
            .unwrap()
            .strip_prefix("licenses/")
            .unwrap();
        assert_eq!(fs::read(bundled.join(pinned_path)).unwrap(), full_text);
        assert!(!bundled.join("manifest.json").exists());
        assert!(!root.join("target/.package-staging").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn retries_interrupted_package_promotion_and_serializes_builds() {
        let root = package_fixture();
        let staging = root.join("target/.package-staging");
        fs::create_dir_all(staging.join("previous")).unwrap();
        fs::create_dir(staging.join("next")).unwrap();
        fs::write(staging.join("previous/weftforge"), b"previous studio").unwrap();

        let stage = PackageStage::new(&root).unwrap();
        let previous = root.join("target/package/weftforge");
        assert_eq!(fs::read(&previous).unwrap(), b"previous studio");
        assert!(!staging.join("next").exists());
        assert!(PackageStage::new(&root).is_err());
        drop(stage);
        let retry = PackageStage::new(&root).unwrap();
        assert_eq!(fs::read(previous).unwrap(), b"previous studio");
        drop(retry);
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
