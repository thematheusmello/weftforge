use crate::{
    Curve, Error, Id, Mannequin, Material, Mesh, Piece, Placement, Point, Project, Result, Seam,
    ensure,
};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

mod dxf;

pub const FORMAT_VERSION: u32 = 2;
pub const MAX_PROJECT_BYTES: u64 = 16 * 1024 * 1024;
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document<P = Project> {
    format: String,
    version: u32,
    project: P,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectV1 {
    revision: u64,
    points: Vec<Point>,
    curves: Vec<Curve>,
    pieces: Vec<PieceV1>,
    materials: Vec<MaterialV1>,
    seams: Vec<Seam>,
    mannequin: Mannequin,
    resolution: f64,
    next_id: Id,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PieceV1 {
    id: Id,
    name: String,
    boundary: Vec<Id>,
    placement: Placement,
    material: Id,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialV1 {
    id: Id,
    name: String,
    areal_density: f64,
    stretch_stiffness: f64,
    shear_stiffness: f64,
    bending_stiffness: f64,
    damping: f64,
    friction: f64,
    collision_thickness: f64,
    color: [f32; 3],
}

impl From<ProjectV1> for Project {
    fn from(old: ProjectV1) -> Self {
        Self {
            revision: old.revision,
            points: old.points,
            curves: old.curves,
            pieces: old
                .pieces
                .into_iter()
                .map(|piece| Piece {
                    id: piece.id,
                    name: piece.name,
                    boundary: piece.boundary,
                    placement: piece.placement,
                    material: piece.material,
                    grain_angle: 0.0,
                })
                .collect(),
            materials: old
                .materials
                .into_iter()
                .map(|material| Material {
                    id: material.id,
                    name: material.name,
                    areal_density: material.areal_density,
                    warp_stiffness: material.stretch_stiffness,
                    weft_stiffness: material.stretch_stiffness,
                    shear_stiffness: material.shear_stiffness,
                    bending_stiffness: material.bending_stiffness,
                    damping: material.damping,
                    friction: material.friction,
                    collision_thickness: material.collision_thickness,
                    color: material.color,
                })
                .collect(),
            seams: old.seams,
            mannequin: old.mannequin,
            resolution: old.resolution,
            next_id: old.next_id,
        }
    }
}

pub fn recovery_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".recovery");
    path.with_file_name(name)
}

impl Project {
    pub fn to_json(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(&Document {
            format: "weftforge".into(),
            version: FORMAT_VERSION,
            project: self.clone(),
        })?;
        ensure(
            bytes.len() as u64 <= MAX_PROJECT_BYTES,
            "Project exceeds the 16 MiB file limit",
        )?;
        Ok(bytes)
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        ensure(
            bytes.len() as u64 <= MAX_PROJECT_BYTES,
            "Project exceeds the 16 MiB file limit",
        )?;
        let header: Document<serde::de::IgnoredAny> = serde_json::from_slice(bytes)?;
        ensure(header.format == "weftforge", "Not a WeftForge project")?;
        let project = match header.version {
            1 => serde_json::from_slice::<Document<ProjectV1>>(bytes)?
                .project
                .into(),
            FORMAT_VERSION => serde_json::from_slice::<Document>(bytes)?.project,
            _ => {
                return Err(Error(
                    "Unsupported WeftForge project version; no data was loaded".into(),
                ));
            }
        };
        project.validate()?;
        Ok(project)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_json(&read_bounded(path.as_ref())?)
    }

    /// Explicitly loads the prior successfully saved document, never silently
    /// substitutes a backup for a corrupt or unsupported primary file.
    pub fn recover(path: impl AsRef<Path>) -> Result<Self> {
        Self::load(recovery_path(path))
    }

    /// Writes in the target directory, fsyncs before replacement, and retains
    /// the previous valid document as `<filename>.recovery`.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        self.save_impl(path.as_ref(), false)
    }

    fn save_impl(&self, path: &Path, interrupt_before_replace: bool) -> Result<()> {
        let bytes = self.to_json()?;
        ensure(path.file_name().is_some(), "Save path needs a filename")?;
        let exists = match fs::symlink_metadata(path) {
            Ok(metadata) => {
                ensure(
                    metadata.file_type().is_file(),
                    "Refusing to replace a non-regular project file",
                )?;
                true
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        };
        let staged = StagedFile::write(path, &bytes)?;
        if exists {
            let previous = read_bounded(path)?;
            Self::from_json(&previous).map_err(|error| {
                Error(format!(
                    "Existing project is invalid and was preserved; save to a new path: {error}"
                ))
            })?;
            StagedFile::write(&recovery_path(path), &previous)?.replace(&recovery_path(path))?;
        }
        if interrupt_before_replace {
            return Err(Error(
                "Injected interruption before atomic project replacement".into(),
            ));
        }
        staged.replace(path)
    }

    pub fn export_svg(&self, path: impl AsRef<Path>) -> Result<()> {
        export_path_guard(path.as_ref())?;
        self.validate()?;
        let mut min = [0.0_f64; 2];
        let mut max = [1.0_f64; 2];
        for point in &self.points {
            for axis in 0..2 {
                min[axis] = min[axis].min(point.position[axis]);
                max[axis] = max[axis].max(point.position[axis]);
            }
        }
        for curve in &self.curves {
            if let Some(controls) = curve.controls {
                for point in controls {
                    for axis in 0..2 {
                        min[axis] = min[axis].min(point[axis]);
                        max[axis] = max[axis].max(point[axis]);
                    }
                }
            }
        }
        let width = (max[0] - min[0]) * 1000.0;
        let height = (max[1] - min[1]) * 1000.0;
        let mut svg = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}mm\" height=\"{height}mm\" viewBox=\"{} {} {width} {height}\">\n<title>WeftForge editable pattern export</title>\n<desc>Scale: one SVG user unit is one millimetre. Pattern V points upward; SVG Y is inverted.</desc>\n",
            min[0] * 1000.0,
            -max[1] * 1000.0
        );
        for piece in &self.pieces {
            let mut data = String::new();
            for (index, id) in piece.boundary.iter().enumerate() {
                let curve = self
                    .curve(*id)
                    .ok_or_else(|| Error("Missing boundary curve".into()))?;
                let a = self
                    .point(curve.start)
                    .ok_or_else(|| Error("Missing point".into()))?
                    .position;
                let b = self
                    .point(curve.end)
                    .ok_or_else(|| Error("Missing point".into()))?
                    .position;
                if index == 0 {
                    write!(data, "M {} {} ", a[0] * 1000.0, -a[1] * 1000.0)
                        .expect("Writing a String cannot fail");
                }
                if let Some([c, d]) = curve.controls {
                    write!(
                        data,
                        "C {} {} {} {} {} {} ",
                        c[0] * 1000.0,
                        -c[1] * 1000.0,
                        d[0] * 1000.0,
                        -d[1] * 1000.0,
                        b[0] * 1000.0,
                        -b[1] * 1000.0
                    )
                    .expect("Writing a String cannot fail");
                } else {
                    write!(data, "L {} {} ", b[0] * 1000.0, -b[1] * 1000.0)
                        .expect("Writing a String cannot fail");
                }
            }
            writeln!(svg,"<path id=\"piece-{}\" data-name=\"{}\" d=\"{}Z\" fill=\"none\" stroke=\"black\" stroke-width=\"0.3\"/> ",piece.id,xml_escape(&piece.name),data).expect("Writing a String cannot fail");
        }
        svg.push_str("</svg>\n");
        StagedFile::write(path.as_ref(), svg.as_bytes())?.replace(path.as_ref())
    }

    /// Exports authored pattern boundaries in millimetres, using the FILE-005-v1
    /// ASCII R2000 profile. Sewing, materials, placement and simulation are omitted.
    pub fn export_dxf(&self, path: impl AsRef<Path>) -> Result<()> {
        export_path_guard(path.as_ref())?;
        self.validate()?;
        let dxf = dxf::render(self)?;
        StagedFile::write(path.as_ref(), dxf.as_bytes())?.replace(path.as_ref())
    }
}

impl Mesh {
    pub fn export_obj(&self, path: impl AsRef<Path>) -> Result<()> {
        export_path_guard(path.as_ref())?;
        self.validate()?;
        let normals = self.normals()?;
        let mut obj =
            String::from("# WeftForge OBJ: metres; Y up; material UV coordinates in metres\n");
        for [x, y, z] in &self.positions {
            writeln!(obj, "v {x:.17} {y:.17} {z:.17}").expect("Writing a String cannot fail");
        }
        for [u, v] in &self.uv {
            writeln!(obj, "vt {u:.17} {v:.17}").expect("Writing a String cannot fail");
        }
        for [x, y, z] in normals {
            writeln!(obj, "vn {x:.17} {y:.17} {z:.17}").expect("Writing a String cannot fail");
        }
        let mut last_piece = None;
        for triangle in &self.triangles {
            let piece = self.piece_ids[triangle[0]];
            if last_piece != Some(piece) {
                writeln!(obj, "g piece_{piece}").expect("Writing a String cannot fail");
                last_piece = Some(piece);
            }
            let [a, b, c] = triangle.map(|v| v + 1);
            writeln!(obj, "f {a}/{a}/{a} {b}/{b}/{b} {c}/{c}/{c}")
                .expect("Writing a String cannot fail");
        }
        StagedFile::write(path.as_ref(), obj.as_bytes())?.replace(path.as_ref())
    }
}

fn export_path_guard(path: &Path) -> Result<()> {
    let native_name = |path: &Path| {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        let name = name.strip_suffix(".recovery").unwrap_or(&name);
        let name = name.strip_suffix(".autosave").unwrap_or(name);
        name.ends_with(".weftforge")
    };
    ensure(
        !native_name(path),
        "Exports must not replace native project, autosave, or recovery files",
    )?;
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
        Ok(_) => {}
    }
    ensure(
        !native_name(&path.canonicalize()?),
        "Exports must not replace native project, autosave, or recovery files",
    )?;
    let metadata = fs::metadata(path)?;
    ensure(metadata.is_file(), "Export output must be a regular file")?;
    if metadata.len() <= MAX_PROJECT_BYTES {
        ensure(
            Project::from_json(&read_bounded(path)?).is_err(),
            "Exports must not replace an existing native project",
        )?;
    }
    Ok(())
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn read_bounded(path: &Path) -> Result<Vec<u8>> {
    ensure(
        fs::metadata(path)?.is_file(),
        "Project input must be a regular file",
    )?;
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_PROJECT_BYTES + 1)
        .read_to_end(&mut bytes)?;
    ensure(
        bytes.len() as u64 <= MAX_PROJECT_BYTES,
        "Project exceeds the 16 MiB file limit",
    )?;
    Ok(bytes)
}

struct StagedFile {
    path: PathBuf,
}

impl StagedFile {
    fn write(target: &Path, bytes: &[u8]) -> Result<Self> {
        let name = target
            .file_name()
            .ok_or_else(|| Error("Output path needs a filename".into()))?;
        let mut temp_name = OsString::from(".");
        temp_name.push(name);
        temp_name.push(format!(
            ".{}-{}.tmp",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let path = target.with_file_name(temp_name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let staged = Self { path };
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(staged)
    }

    fn replace(self, target: &Path) -> Result<()> {
        match fs::symlink_metadata(target) {
            Ok(metadata) => ensure(
                metadata.file_type().is_file(),
                "Refusing to replace a non-regular project or recovery file",
            )?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        fs::rename(&self.path, target)?;
        // The rename is the commit point; directory fsync support varies by OS.
        if let Some(parent) = target.parent().filter(|p| !p.as_os_str().is_empty())
            && let Ok(directory) = File::open(parent)
        {
            let _ = directory.sync_all();
        }
        Ok(())
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Editor};

    #[test]
    fn saves_refuse_nonregular_destinations_without_changing_targets_or_recovery() {
        let directory = std::env::temp_dir().join(format!(
            "weftforge-save-destinations-{}-{}",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let primary = directory.join("primary.weftforge");
        let mut project = Project::default();
        project.save(&primary).unwrap();
        let original = fs::read(&primary).unwrap();
        project.resolution = 0.04;
        let folder = directory.join("folder.weftforge");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("retained"), b"folder contents").unwrap();
        fs::write(recovery_path(&folder), b"prior folder recovery").unwrap();
        assert!(project.save(&folder).is_err());
        assert_eq!(
            fs::read(folder.join("retained")).unwrap(),
            b"folder contents"
        );
        assert_eq!(
            fs::read(recovery_path(&folder)).unwrap(),
            b"prior folder recovery"
        );
        #[cfg(unix)]
        for (name, target) in [
            ("valid.weftforge", primary.clone()),
            ("dangling.weftforge", directory.join("absent")),
        ] {
            let output = directory.join(name);
            std::os::unix::fs::symlink(&target, &output).unwrap();
            fs::write(recovery_path(&output), b"prior symlink recovery").unwrap();
            assert!(project.save(&output).is_err(), "{name}");
            assert!(
                fs::symlink_metadata(&output)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
            assert_eq!(fs::read_link(&output).unwrap(), target);
            assert_eq!(fs::read(&primary).unwrap(), original);
            assert_eq!(
                fs::read(recovery_path(&output)).unwrap(),
                b"prior symlink recovery"
            );
        }
        assert!(!directory.join("absent").exists());
        let recovery = recovery_path(&primary);
        #[cfg(unix)]
        for target in [primary.clone(), directory.join("absent-recovery")] {
            std::os::unix::fs::symlink(&target, &recovery).unwrap();
            assert!(project.save(&primary).is_err());
            assert_eq!(fs::read_link(&recovery).unwrap(), target);
            assert_eq!(fs::read(&primary).unwrap(), original);
            fs::remove_file(&recovery).unwrap();
        }
        fs::create_dir(&recovery).unwrap();
        fs::write(recovery.join("retained"), b"recovery folder contents").unwrap();
        assert!(project.save(&primary).is_err());
        assert_eq!(fs::read(&primary).unwrap(), original);
        assert_eq!(
            fs::read(recovery.join("retained")).unwrap(),
            b"recovery folder contents"
        );
        fs::remove_dir_all(&recovery).unwrap();
        project.save(&primary).unwrap();
        assert_eq!(Project::load(&primary).unwrap(), project);
        assert_eq!(fs::read(recovery_path(&primary)).unwrap(), original);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn exports_preserve_native_projects_and_aliases_at_any_extension() {
        let directory = std::env::temp_dir().join(format!(
            "weftforge-export-guard-{}-{}",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let mut editor = Editor::default();
        editor
            .execute(Command::AddRectangle {
                name: "Front".into(),
                origin: [0., 0.],
                width: 0.3,
                height: 0.4,
            })
            .unwrap();
        let mesh = editor.project.mesh().unwrap();
        let native = Project::default().to_json().unwrap();
        for name in [
            "other.weftforge",
            "draft.WEFTFORGE.AUTOSAVE",
            "old.weftforge.recovery",
            "old.weftforge.autosave.recovery",
            "project.obj",
            "project.svg",
            "project.dxf",
            "unusual.native",
        ] {
            let path = directory.join(name);
            fs::write(&path, &native).unwrap();
            assert!(editor.project.export_svg(&path).is_err(), "{name}");
            assert!(editor.project.export_dxf(&path).is_err(), "{name}");
            assert!(mesh.export_obj(&path).is_err(), "{name}");
            assert_eq!(fs::read(&path).unwrap(), native);
        }
        let protected = directory.join("ordinary.WeFtFoRgE");
        fs::write(&protected, b"invalid but native-named contents").unwrap();
        assert!(editor.project.export_svg(&protected).is_err());
        assert!(editor.project.export_dxf(&protected).is_err());
        assert!(mesh.export_obj(&protected).is_err());
        assert_eq!(
            fs::read(&protected).unwrap(),
            b"invalid but native-named contents"
        );
        let hardlink = directory.join("hardlink.svg");
        fs::hard_link(directory.join("unusual.native"), &hardlink).unwrap();
        assert!(editor.project.export_svg(&hardlink).is_err());
        assert!(editor.project.export_dxf(&hardlink).is_err());
        assert!(mesh.export_obj(&hardlink).is_err());
        assert_eq!(fs::read(&hardlink).unwrap(), native);
        let folder = directory.join("folder.svg");
        fs::create_dir(&folder).unwrap();
        assert!(editor.project.export_svg(&folder).is_err());
        assert!(editor.project.export_dxf(&folder).is_err());
        assert!(mesh.export_obj(&folder).is_err());
        assert!(folder.is_dir());
        #[cfg(unix)]
        for (target, alias) in [
            ("unusual.native", "alias.obj"),
            ("ordinary.WeFtFoRgE", "named-alias.svg"),
        ] {
            let path = directory.join(alias);
            std::os::unix::fs::symlink(directory.join(target), &path).unwrap();
            let previous = fs::read(&path).unwrap();
            assert!(editor.project.export_svg(&path).is_err());
            assert!(editor.project.export_dxf(&path).is_err());
            assert!(mesh.export_obj(&path).is_err());
            assert_eq!(fs::read(&path).unwrap(), previous);
            assert!(
                fs::symlink_metadata(&path)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
        }
        #[cfg(unix)]
        {
            let broken = directory.join("broken.svg");
            std::os::unix::fs::symlink(directory.join("absent"), &broken).unwrap();
            assert!(editor.project.export_svg(&broken).is_err());
            assert!(editor.project.export_dxf(&broken).is_err());
            assert!(mesh.export_obj(&broken).is_err());
            assert!(
                fs::symlink_metadata(&broken)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
        }
        let svg = directory.join("patterns.svg");
        let obj = directory.join("rest.obj");
        let dxf = directory.join("patterns.dxf");
        fs::write(&svg, b"old SVG").unwrap();
        fs::write(&obj, b"old OBJ").unwrap();
        fs::write(&dxf, b"old DXF").unwrap();
        editor.project.export_svg(&svg).unwrap();
        mesh.export_obj(&obj).unwrap();
        editor.project.export_dxf(&dxf).unwrap();
        assert!(fs::read_to_string(dxf).unwrap().contains("AC1015"));
        assert!(fs::read_to_string(svg).unwrap().contains("<svg"));
        assert!(
            fs::read_to_string(obj)
                .unwrap()
                .starts_with("# WeftForge OBJ")
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn interrupted_save_and_invalid_inputs_preserve_the_previous_document() {
        let path = std::env::temp_dir().join(format!(
            "weftforge-save-{}-{}.weftforge",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let mut editor = Editor::default();
        editor.project.save(&path).unwrap();
        let original = fs::read(&path).unwrap();
        editor
            .execute(Command::AddRectangle {
                name: "Front".into(),
                origin: [0.0, 0.0],
                width: 0.5,
                height: 0.7,
            })
            .unwrap();
        assert!(editor.project.save_impl(&path, true).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        assert!(Project::recover(&path).unwrap().pieces.is_empty());
        editor.project.save(&path).unwrap();
        assert_eq!(Project::load(&path).unwrap(), editor.project);
        assert!(Project::recover(&path).unwrap().pieces.is_empty());
        let valid = fs::read(&path).unwrap();
        editor.project.resolution = f64::NAN;
        assert!(editor.project.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), valid);
        let mut json: serde_json::Value = serde_json::from_slice(&valid).unwrap();
        json["version"] = 99.into();
        assert!(Project::from_json(&serde_json::to_vec(&json).unwrap()).is_err());
        assert!(Project::from_json(b"{malformed").is_err());
        fs::write(&path, b"corrupt primary").unwrap();
        assert!(Project::default().save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"corrupt primary");
        fs::remove_file(&path).unwrap();
        fs::remove_file(recovery_path(&path)).unwrap();
    }
}
