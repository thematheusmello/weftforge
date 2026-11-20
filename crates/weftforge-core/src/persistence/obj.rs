use super::{StagedExport, export_path_guard};
use crate::{Error, Id, Material, Mesh, Project, Result, ensure};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

struct MaterialLibrary {
    filename: String,
    contents: String,
    by_piece: BTreeMap<Id, String>,
}

impl MaterialLibrary {
    fn from_mesh(mesh: &Mesh, project: &Project) -> Result<Self> {
        let pieces: BTreeMap<_, _> = project.pieces.iter().map(|p| (p.id, p.material)).collect();
        let fabrics: BTreeMap<_, _> = project.materials.iter().map(|m| (m.id, m)).collect();
        let mut used = BTreeMap::<Id, &Material>::new();
        let mut by_piece = BTreeMap::new();
        for &piece in &mesh.piece_ids {
            if let std::collections::btree_map::Entry::Vacant(entry) = by_piece.entry(piece) {
                let id = pieces
                    .get(&piece)
                    .ok_or_else(|| Error("OBJ mesh refers to an unknown pattern piece".into()))?;
                let fabric = fabrics[id];
                used.insert(*id, fabric);
                entry.insert(material_token(fabric));
            }
        }
        let mut contents = String::from("# WeftForge fabric colors\n");
        for fabric in used.values() {
            let [r, g, b] = fabric.color;
            writeln!(
                contents,
                "# {}\nnewmtl {}\nKd {r} {g} {b}\n",
                serde_json::to_string(&fabric.name)?,
                material_token(fabric)
            )
            .unwrap();
        }
        let filename = format!(
            "weftforge-fabrics-{:x}.mtl",
            Sha256::digest(contents.as_bytes())
        );
        Ok(Self {
            filename,
            contents,
            by_piece,
        })
    }

    fn install(&self, directory: &Path) -> Result<()> {
        let path = directory.join(&self.filename);
        let existing_matches = || -> Result<()> {
            let meta = fs::symlink_metadata(&path)?;
            ensure(
                meta.file_type().is_file() && meta.len() == self.contents.len() as u64,
                "OBJ material library already exists with different contents or type",
            )?;
            let mut bytes = Vec::new();
            File::open(&path)?
                .take(meta.len() + 1)
                .read_to_end(&mut bytes)?;
            ensure(
                bytes == self.contents.as_bytes(),
                "OBJ material library already exists with different contents",
            )
        };
        match fs::symlink_metadata(&path) {
            Ok(_) => return existing_matches(),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        let builder = tempfile::Builder::new();
        #[cfg(unix)]
        let builder = {
            let mut builder = builder;
            use std::os::unix::fs::PermissionsExt;
            builder.permissions(fs::Permissions::from_mode(0o666));
            builder
        };
        let mut staged = builder.tempfile_in(directory)?;
        staged.write_all(self.contents.as_bytes())?;
        staged.as_file().sync_all()?;
        match staged.persist_noclobber(&path) {
            Ok(_) => Ok(()),
            Err(e) if e.error.kind() == ErrorKind::AlreadyExists => existing_matches(),
            Err(e) => Err(e.error.into()),
        }
    }
}

fn material_token(fabric: &Material) -> String {
    let mut token = format!("fabric_{}", fabric.id);
    for word in fabric
        .name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
    {
        token.push('_');
        token.push_str(word);
    }
    token
}

impl Mesh {
    /// Write standalone triangle geometry with material-space UVs in metres.
    pub fn export_obj(&self, path: impl AsRef<Path>) -> Result<()> {
        export_path_guard(path.as_ref())?;
        StagedExport::write(path, self.render_obj(None)?.as_bytes())?.commit()
    }

    /// Export rest or draped triangles and their fabric colors from the same editable revision.
    pub fn export_obj_with_materials(
        &self,
        project: &Project,
        path: impl AsRef<Path>,
    ) -> Result<Option<PathBuf>> {
        let path = path.as_ref();
        export_path_guard(path)?;
        ensure(
            self.doc_revision == project.revision,
            "OBJ export requires the current project revision",
        )?;
        project.validate()?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        ensure(
            !(name.starts_with("weftforge-fabrics-") && name.ends_with(".mtl")),
            "Choose an OBJ destination outside the reserved fabric library names",
        )?;
        if self.triangles.is_empty() {
            self.export_obj(path)?;
            return Ok(None);
        }
        let library = MaterialLibrary::from_mesh(self, project)?;
        let staged = StagedExport::write(path, self.render_obj(Some(&library))?.as_bytes())?;
        let directory = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        // Publish the immutable library first so a replaced OBJ always names a complete fabric table.
        library.install(directory)?;
        staged.commit()?;
        Ok(Some(path.with_file_name(library.filename)))
    }

    fn render_obj(&self, library: Option<&MaterialLibrary>) -> Result<String> {
        let normals = self.normals()?;
        let mut obj =
            String::from("# WeftForge OBJ: metres; Y up; material UV coordinates in metres\n");
        if let Some(library) = library {
            writeln!(obj, "mtllib {}", library.filename).unwrap();
        }
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
                if let Some(library) = library {
                    writeln!(obj, "usemtl {}", library.by_piece[&piece]).unwrap();
                }
                last_piece = Some(piece);
            }
            let [a, b, c] = triangle.map(|v| v + 1);
            writeln!(obj, "f {a}/{a}/{a} {b}/{b}/{b} {c}/{c}/{c}")
                .expect("Writing a String cannot fail");
        }
        Ok(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Editor};

    #[test]
    fn independent_reader_resolves_each_fabric_and_old_exports_keep_their_colors() {
        let dir = tempfile::tempdir().unwrap();
        let mut editor = Editor::default();
        for name in ["Front", "Back", "Pocket"] {
            editor
                .execute(Command::AddRectangle {
                    name: name.into(),
                    origin: [0., 0.],
                    width: 0.25,
                    height: 0.5,
                })
                .unwrap();
        }
        editor
            .execute(Command::AddMaterial {
                material: Material {
                    name: "Linen / café #1".into(),
                    color: [0.83, 0.25, 0.12],
                    ..Material::default()
                },
            })
            .unwrap();
        let fabric = editor.project.materials[1].clone();
        let back = editor.project.pieces[1].id;
        editor
            .execute(Command::SetMaterial {
                piece: back,
                material: fabric.id,
            })
            .unwrap();
        let mesh = editor.project.mesh().unwrap();
        let first = dir.path().join("First garment.obj");
        let second = dir.path().join("Second garment.obj");
        std::thread::scope(|scope| {
            let export = |path: &Path| {
                mesh.export_obj_with_materials(&editor.project, path)
                    .unwrap()
                    .unwrap()
            };
            let second_path = &second;
            let other = scope.spawn(move || export(second_path));
            assert_eq!(export(&first), other.join().unwrap());
        });
        let (models, library) = tobj::load_obj(&first, &tobj::LoadOptions::default()).unwrap();
        let materials = library.unwrap();
        assert_eq!((models.len(), materials.len()), (3, 2));
        for model in &models {
            let piece = model.name.strip_prefix("piece_").unwrap().parse().unwrap();
            let assigned = editor
                .project
                .material(editor.project.piece(piece).unwrap().material)
                .unwrap();
            let decoded = &materials[model.mesh.material_id.unwrap()];
            assert_eq!(decoded.name, material_token(assigned));
            assert_eq!(decoded.diffuse, Some(assigned.color));
        }
        let original = fs::read(&first).unwrap();
        let old_library = mesh
            .export_obj_with_materials(&editor.project, &first)
            .unwrap()
            .unwrap();
        let old_colors = fs::read(&old_library).unwrap();
        editor
            .execute(Command::UpdateMaterial {
                material: Material {
                    color: [0., 0., 1.],
                    ..fabric
                },
            })
            .unwrap();
        assert!(
            mesh.export_obj_with_materials(&editor.project, &first)
                .is_err()
        );
        assert_eq!(fs::read(&first).unwrap(), original);
        let updated = editor.project.mesh().unwrap();
        let new_library = updated
            .export_obj_with_materials(&editor.project, &second)
            .unwrap()
            .unwrap();
        assert_ne!(old_library, new_library);
        assert_eq!(fs::read(&old_library).unwrap(), old_colors);
        assert!(
            tobj::load_obj(&second, &Default::default())
                .unwrap()
                .1
                .unwrap()
                .iter()
                .any(|m| m.diffuse == Some([0., 0., 1.]))
        );
        let alien = vec![b'x'; fs::metadata(&new_library).unwrap().len() as usize];
        fs::write(&new_library, &alien).unwrap();
        assert!(
            updated
                .export_obj_with_materials(&editor.project, &first)
                .is_err()
        );
        assert_eq!(fs::read(first).unwrap(), original);
        assert_eq!(fs::read(new_library).unwrap(), alien);
    }
}
