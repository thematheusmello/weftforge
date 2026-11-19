use super::{BuildArtifacts, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Deserialize)]
struct Workspace {
    target_directory: PathBuf,
}

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<CargoPackage>,
    workspace_members: BTreeSet<String>,
    resolve: Resolve,
}

#[derive(Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    repository: Option<String>,
    authors: Vec<String>,
    manifest_path: PathBuf,
    license: Option<String>,
    license_file: Option<PathBuf>,
}

#[derive(Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}

#[derive(Deserialize)]
struct Node {
    id: String,
    deps: Vec<Edge>,
}

#[derive(Deserialize)]
struct Edge {
    pkg: String,
    dep_kinds: Vec<DepKind>,
}

#[derive(Deserialize)]
struct DepKind {
    kind: Option<String>,
}

pub(super) struct NoticePlan {
    pub(super) target: String,
    workspace: PathBuf,
    packages: Vec<CargoPackage>,
}

fn metadata(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--locked"])
        .args(args)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "Could not resolve the Cargo packaging graph: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(output.stdout)
}

fn rustc_host(root: &Path) -> Result<String> {
    let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg("-vV")
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err("Could not identify rustc's host target".into());
    }
    let version = String::from_utf8(output.stdout)?;
    version
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_owned)
        .ok_or_else(|| "rustc did not report its host target".into())
}

fn artifact_target(path: &Path, cargo_target: &Path, host: &str) -> Result<String> {
    let path = path.canonicalize()?;
    let relative = path.strip_prefix(cargo_target)?;
    let parts: Vec<_> = relative.iter().collect();
    match parts.as_slice() {
        [profile, _] if *profile == std::ffi::OsStr::new("release") => Ok(host.to_owned()),
        [target, profile, _] if *profile == std::ffi::OsStr::new("release") => target
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| "The Cargo artifact target is not UTF-8".into()),
        _ => Err("Cargo did not report a release executable in its target directory".into()),
    }
}

impl NoticePlan {
    pub(super) fn load(root: &Path, build: &BuildArtifacts) -> Result<Self> {
        let workspace: Workspace = serde_json::from_slice(&metadata(root, &["--no-deps"])?)?;
        let directory = workspace.target_directory.canonicalize()?;
        let host = rustc_host(root)?;
        let [studio, cli] = &build.binaries;
        let target = artifact_target(&studio.path, &directory, &host)?;
        if target != artifact_target(&cli.path, &directory, &host)? {
            return Err("The packaged binaries must use the same Cargo target".into());
        }
        let graph = metadata(root, &["--filter-platform", &target])?;
        Self::from_graph(root, target, &graph, build)
    }

    pub(super) fn from_graph(
        root: &Path,
        target: String,
        graph: &[u8],
        build: &BuildArtifacts,
    ) -> Result<Self> {
        let metadata: Metadata = serde_json::from_slice(graph)?;
        let nodes: BTreeMap<_, _> = metadata
            .resolve
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();
        let mut pending: Vec<_> = build.binaries.iter().map(|bin| bin.id.as_str()).collect();
        let mut visited = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !visited.insert(id) {
                continue;
            }
            let node = nodes
                .get(id)
                .ok_or("Cargo's metadata omits a built runtime package")?;
            for edge in &node.deps {
                if edge.dep_kinds.iter().any(|kind| kind.kind.is_none())
                    && build.libraries.contains(&edge.pkg)
                {
                    pending.push(&edge.pkg);
                }
            }
        }
        let mut packages = Vec::new();
        for package in metadata.packages {
            if visited.remove(package.id.as_str())
                && !metadata.workspace_members.contains(&package.id)
            {
                packages.push(package);
            }
        }
        if !visited.is_empty() {
            return Err("Cargo's metadata omits a resolved runtime package".into());
        }
        packages.sort_by(|a, b| {
            (&a.name, &a.version, &a.source).cmp(&(&b.name, &b.version, &b.source))
        });
        Ok(Self {
            target,
            workspace: root.canonicalize()?,
            packages,
        })
    }

    pub(super) fn write(&self, package: &Path, artifacts: &[PathBuf]) -> Result<usize> {
        let directory = package.join("licenses/cargo");
        fs::create_dir(&directory)?;
        let mut crates = Vec::new();
        for dependency in &self.packages {
            crates.push(crate_notices(&self.workspace, package, dependency)?);
        }
        let unresolved = crates
            .iter()
            .filter(|notice| !notice.unresolved.is_empty())
            .count();
        let manifest = Manifest {
            version: 1,
            target: &self.target,
            scope: "Target-filtered Cargo normal dependencies reported by the packaged release build",
            limitations: [
                "Cargo dependencies only. System runtimes, shared libraries, drivers and OS notices require a separate audit.",
                "Collected text and declared license expressions do not resolve all redistribution obligations.",
            ],
            artifacts: artifacts
                .iter()
                .map(|path| file_record(package, path))
                .collect::<Result<_>>()?,
            project_notices: [
                "LICENSE-APACHE",
                "LICENSE-MIT",
                "THIRD_PARTY.md",
                "licenses/OFL-Inter.txt",
                "licenses/OFL-JetBrainsMono.txt",
            ]
            .into_iter()
            .map(|path| file_record(package, Path::new(path)))
            .collect::<Result<_>>()?,
            crates,
        };
        let mut file = File::create(package.join("licenses/manifest.json"))?;
        serde_json::to_writer_pretty(&mut file, &manifest)?;
        Ok(unresolved)
    }
}

#[derive(Serialize)]
struct Manifest<'a> {
    version: u32,
    target: &'a str,
    scope: &'a str,
    limitations: [&'a str; 2],
    artifacts: Vec<FileRecord>,
    project_notices: Vec<FileRecord>,
    crates: Vec<CrateNotice<'a>>,
}

#[derive(Serialize)]
struct CrateNotice<'a> {
    name: &'a str,
    version: &'a str,
    source: String,
    authors: &'a [String],
    repository: Option<&'a str>,
    declared_license: Option<&'a str>,
    files: Vec<FileRecord>,
    unresolved: Vec<&'static str>,
}

#[derive(Serialize)]
struct FileRecord {
    path: String,
    size_bytes: u64,
    sha256: String,
}

fn portable_path(path: &Path) -> Result<String> {
    path.iter()
        .map(|part| part.to_str().ok_or("A packaged path is not UTF-8"))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map(|parts| parts.join("/"))
        .map_err(Into::into)
}

fn file_record(base: &Path, relative: &Path) -> Result<FileRecord> {
    let mut file = File::open(base.join(relative))?;
    let mut hasher = Sha256::new();
    let mut bytes = [0; 32 * 1024];
    let mut size_bytes = 0;
    loop {
        let length = file.read(&mut bytes)?;
        if length == 0 {
            break;
        }
        size_bytes += length as u64;
        hasher.update(&bytes[..length]);
    }
    Ok(FileRecord {
        path: portable_path(relative)?,
        size_bytes,
        sha256: format!("{:x}", hasher.finalize()),
    })
}

fn notice_kind(name: &str) -> Option<bool> {
    let name = name.to_ascii_uppercase();
    match name.split(['.', '-', '_']).next()? {
        "LICENSE" | "LICENCE" | "COPYING" | "UNLICENSE" => Some(true),
        "NOTICE" | "NOTICES" | "COPYRIGHT" | "AUTHORS" => Some(false),
        _ => None,
    }
}

fn notice_paths(root: &Path, license_file: Option<&Path>) -> Result<BTreeMap<PathBuf, bool>> {
    let mut paths = BTreeMap::new();
    for item in fs::read_dir(root)? {
        let item = item?;
        let name = item.file_name();
        let kind = item.file_type()?;
        let label = name.to_string_lossy();
        if kind.is_dir()
            && (label.eq_ignore_ascii_case("licenses") || label.eq_ignore_ascii_case("notices"))
        {
            for child in fs::read_dir(item.path())? {
                let child = child?;
                if child.file_type()?.is_file() {
                    paths.insert(
                        Path::new(&name).join(child.file_name()),
                        label.eq_ignore_ascii_case("licenses"),
                    );
                }
            }
        } else if (kind.is_file() || kind.is_symlink())
            && let Some(is_license) = notice_kind(&label)
        {
            paths.insert(PathBuf::from(name), is_license);
        }
    }
    if let Some(declared) = license_file {
        let declared = root.join(declared).canonicalize()?;
        paths.insert(declared.strip_prefix(root)?.to_path_buf(), true);
    }
    Ok(paths)
}

fn crate_notices<'a>(
    workspace: &Path,
    package: &Path,
    dependency: &'a CargoPackage,
) -> Result<CrateNotice<'a>> {
    let root = dependency
        .manifest_path
        .parent()
        .ok_or("Cargo package has no manifest directory")?
        .canonicalize()?;
    let origin = match &dependency.source {
        Some(source) if source.starts_with("registry+https://") => source.clone(),
        Some(_) => "Cargo package from an alternate source".to_owned(),
        None => match root.strip_prefix(workspace) {
            Ok(relative) => portable_path(relative)?,
            Err(_) => "Cargo path dependency outside the checkout".to_owned(),
        },
    };
    let mut paths = notice_paths(&root, dependency.license_file.as_deref())?;
    if dependency.name == "epaint_default_fonts" && dependency.version == "0.35.0" {
        for text in [
            "fonts/Hack-Regular.txt",
            "fonts/OFL.txt",
            "fonts/UFL.txt",
            "fonts/emoji-icon-font-mit-license.txt",
        ] {
            paths.insert(PathBuf::from(text), false);
        }
    }
    let mut files = Vec::new();
    let mut has_license_text = false;
    if !paths.is_empty() {
        let folder = format!("{}-{}", dependency.name, dependency.version);
        if Path::new(&folder)
            .file_name()
            .and_then(|value| value.to_str())
            != Some(folder.as_str())
        {
            return Err("Invalid Cargo package name or version".into());
        }
        let destination = Path::new("licenses/cargo").join(folder);
        fs::create_dir(package.join(&destination))?;
        for (relative, is_license) in paths {
            let source = root.join(&relative).canonicalize()?;
            if !source.starts_with(&root) || !source.is_file() {
                return Err(format!(
                    "Notice for {} must be a file inside its package",
                    dependency.name
                )
                .into());
            }
            let path = destination.join(relative);
            let output = package.join(&path);
            fs::create_dir_all(output.parent().expect("packaged notice parent"))?;
            fs::copy(source, output)?;
            let record = file_record(package, &path)?;
            has_license_text |= is_license && record.size_bytes != 0;
            files.push(record);
        }
    }
    let mut unresolved = Vec::new();
    if !has_license_text {
        unresolved.push("No upstream license text was found in the package's notice files.");
    }
    if dependency.license.as_deref().is_none_or(str::is_empty) && dependency.license_file.is_none()
    {
        unresolved.push("Cargo declares neither a license expression nor a license file.");
    }
    Ok(CrateNotice {
        name: &dependency.name,
        version: &dependency.version,
        source: origin,
        authors: &dependency.authors,
        repository: dependency
            .repository
            .as_deref()
            .filter(|value| value.starts_with("https://")),
        declared_license: dependency.license.as_deref(),
        files,
        unresolved,
    })
}

pub(super) fn copy_notice_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for item in fs::read_dir(source)? {
        let item = item?;
        let target = destination.join(item.file_name());
        if item.file_type()?.is_dir() {
            copy_notice_tree(&item.path(), &target)?;
        } else {
            fs::copy(item.path(), target)?;
        }
    }
    Ok(())
}
