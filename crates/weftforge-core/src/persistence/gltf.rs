use super::{StagedExport, export_path_guard};
use crate::{Error, Id, Mesh, Project, Result, ensure};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

const FLOAT: u32 = 5126;
const UNSIGNED_INT: u32 = 5125;
const ARRAY_BUFFER: u32 = 34962;
const ELEMENT_ARRAY_BUFFER: u32 = 34963;
const BUFFER_URI_PREFIX: &str = "data:application/octet-stream;base64,";

impl Mesh {
    /// Writes a single glTF 2.0 binary of this rest or simulated surface and its fabric tint.
    pub fn export_glb(&self, project: &Project, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        export_path_guard(path)?;
        let bytes = encode_glb(self, project)?;
        StagedExport::write(path, &bytes)?.commit()
    }

    /// Writes a single glTF 2.0 JSON document with its cloth buffer embedded.
    pub fn export_gltf(&self, project: &Project, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        export_path_guard(path)?;
        let (mut scene, binary) = encode_scene(self, project)?;
        scene["buffers"][0]["uri"] = Value::String(buffer_uri(&binary));
        StagedExport::write(path, &serde_json::to_vec(&scene)?)?.commit()
    }
}

#[derive(Default)]
struct Primitive {
    vertices: Vec<usize>,
    indices: Vec<u32>,
}

#[derive(Default)]
struct Buffer {
    bytes: Vec<u8>,
    views: Vec<Value>,
    accessors: Vec<Value>,
}

impl Buffer {
    fn add(
        &mut self,
        target: u32,
        mut accessor: Value,
        components: impl Iterator<Item = [u8; 4]>,
    ) -> usize {
        let start = self.bytes.len();
        self.bytes.extend(components.flatten());
        accessor["bufferView"] = json!(self.views.len());
        self.views.push(json!({
            "buffer": 0,
            "byteOffset": start,
            "byteLength": self.bytes.len() - start,
            "target": target,
        }));
        let index = self.accessors.len();
        self.accessors.push(accessor);
        index
    }
}

fn floats<const N: usize>(points: impl Iterator<Item = [f32; N]>) -> impl Iterator<Item = [u8; 4]> {
    points.flat_map(|point| point.map(f32::to_le_bytes))
}

fn float32<const N: usize>(point: [f64; N]) -> Result<[f32; N]> {
    let converted = point.map(|value| value as f32);
    ensure(
        converted.into_iter().all(f32::is_finite),
        "glTF coordinate is outside the finite float32 range",
    )?;
    Ok(converted)
}

fn encode_scene(mesh: &Mesh, project: &Project) -> Result<(Value, Vec<u8>)> {
    ensure(
        mesh.doc_revision == project.revision,
        "glTF export requires a mesh from the current project revision",
    )?;
    ensure(
        !mesh.triangles.is_empty(),
        "Create a pattern before exporting glTF",
    )?;
    project.validate()?;
    let normals = mesh.normals()?;
    let positions: Vec<_> = mesh
        .positions
        .iter()
        .copied()
        .map(float32)
        .collect::<Result<_>>()?;
    let uv: Vec<_> = mesh
        .uv
        .iter()
        .map(|&[u, v]| float32([u, -v]))
        .collect::<Result<_>>()?;
    let normals: Vec<_> = normals.into_iter().map(float32).collect::<Result<_>>()?;
    let mut primitives = BTreeMap::<Id, Primitive>::new();
    let mut local_indices = Vec::with_capacity(positions.len());
    for (index, &piece) in mesh.piece_ids.iter().enumerate() {
        let primitive = primitives.entry(piece).or_default();
        // Mesh validation bounds the vertex count well below the glTF index limit.
        local_indices.push(primitive.vertices.len() as u32);
        primitive.vertices.push(index);
    }
    for &triangle in &mesh.triangles {
        let [a, b, c] = triangle.map(|index| positions[index].map(f64::from));
        let [ux, uy, uz]: [f64; 3] = std::array::from_fn(|axis| b[axis] - a[axis]);
        let [vx, vy, vz]: [f64; 3] = std::array::from_fn(|axis| c[axis] - a[axis]);
        ensure(
            [uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx] != [0.; 3],
            "glTF export would contain a collapsed triangle after float32 conversion",
        )?;
        primitives
            .get_mut(&mesh.piece_ids[triangle[0]])
            .expect("validated mesh piece")
            .indices
            .extend(triangle.map(|index| local_indices[index]));
    }

    let pieces: BTreeMap<_, _> = project
        .pieces
        .iter()
        .map(|piece| (piece.id, piece))
        .collect();
    let fabrics: BTreeMap<_, _> = project
        .materials
        .iter()
        .map(|fabric| (fabric.id, fabric))
        .collect();
    let mut fabric_indices = BTreeMap::new();
    let mut materials = Vec::new();
    let mut meshes = Vec::with_capacity(primitives.len());
    let mut nodes = Vec::with_capacity(primitives.len());
    let mut binary = Buffer::default();
    for (id, primitive) in primitives {
        let piece = pieces
            .get(&id)
            .ok_or_else(|| Error(format!("Missing mesh piece {id}")))?;
        let material = fabrics[&piece.material];
        let material_index = *fabric_indices.entry(material.id).or_insert_with(|| {
            let index = materials.len();
            let [r, g, b] = material.color;
            let surface = material.surface.unwrap_or_default();
            materials.push(json!({
                "name": material.name,
                "doubleSided": true,
                "pbrMetallicRoughness": {
                    "baseColorFactor": [r, g, b, 1.],
                    "metallicFactor": surface.metallic,
                    "roughnessFactor": surface.roughness,
                },
                "extras": {"weftforgeMaterialId": material.id.to_string()},
            }));
            index
        });
        let count = primitive.vertices.len();
        let piece_positions = primitive.vertices.iter().map(|&index| positions[index]);
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for position in piece_positions.clone() {
            for axis in 0..3 {
                min[axis] = min[axis].min(position[axis]);
                max[axis] = max[axis].max(position[axis]);
            }
        }
        let position = binary.add(
            ARRAY_BUFFER,
            json!({"componentType": FLOAT, "count": count, "type": "VEC3", "min": min, "max": max}),
            floats(piece_positions),
        );
        let normal = binary.add(
            ARRAY_BUFFER,
            json!({"componentType": FLOAT, "count": count, "type": "VEC3"}),
            floats(primitive.vertices.iter().map(|&index| normals[index])),
        );
        let texture = binary.add(
            ARRAY_BUFFER,
            json!({"componentType": FLOAT, "count": count, "type": "VEC2"}),
            floats(primitive.vertices.iter().map(|&index| uv[index])),
        );
        let indices = binary.add(
            ELEMENT_ARRAY_BUFFER,
            json!({"componentType": UNSIGNED_INT, "count": primitive.indices.len(), "type": "SCALAR"}),
            primitive.indices.into_iter().map(u32::to_le_bytes),
        );
        nodes.push(json!({"mesh": meshes.len(), "name": piece.name}));
        meshes.push(json!({
            "name": piece.name,
            "extras": {"weftforgePieceId": id.to_string()},
            "primitives": [{
                "attributes": {"POSITION": position, "NORMAL": normal, "TEXCOORD_0": texture},
                "indices": indices,
                "material": material_index,
                "mode": 4,
            }],
        }));
    }
    let scene = json!({
        "asset": {"version": "2.0", "generator": "WeftForge", "extras": {
            "weftforgeSourceRevision": project.revision.to_string(),
        }},
        "scene": 0,
        "scenes": [{"nodes": (0..nodes.len()).collect::<Vec<_>>()}],
        "nodes": nodes,
        "meshes": meshes,
        "materials": materials,
        "accessors": binary.accessors,
        "bufferViews": binary.views,
        "buffers": [{"byteLength": binary.bytes.len()}],
    });
    Ok((scene, binary.bytes))
}

fn encode_glb(mesh: &Mesh, project: &Project) -> Result<Vec<u8>> {
    let (scene, binary) = encode_scene(mesh, project)?;
    let mut json = serde_json::to_vec(&scene)?;
    json.resize(json.len().next_multiple_of(4), b' ');
    let length = 28 + json.len() + binary.len();
    let length32 = u32::try_from(length).map_err(|_| Error("GLB file is too large".into()))?;
    let mut output = Vec::with_capacity(length);
    output.extend_from_slice(b"glTF");
    output.extend_from_slice(&2_u32.to_le_bytes());
    output.extend_from_slice(&length32.to_le_bytes());
    for (kind, data) in [(b"JSON", json), (b"BIN\0", binary)] {
        output.extend_from_slice(&(data.len() as u32).to_le_bytes());
        output.extend_from_slice(kind);
        output.extend_from_slice(&data);
    }
    Ok(output)
}

fn buffer_uri(binary: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut uri = String::with_capacity(BUFFER_URI_PREFIX.len() + binary.len().div_ceil(3) * 4);
    uri.push_str(BUFFER_URI_PREFIX);
    for chunk in binary.chunks(3) {
        let packed = u32::from_be_bytes([
            0,
            chunk[0],
            chunk.get(1).copied().unwrap_or_default(),
            chunk.get(2).copied().unwrap_or_default(),
        ]);
        for shift in [18, 12, 6, 0].into_iter().take(chunk.len() + 1) {
            uri.push(char::from(ALPHABET[((packed >> shift) & 63) as usize]));
        }
        uri.extend(std::iter::repeat_n('=', 3 - chunk.len()));
    }
    uri
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Editor};
    use std::fs;

    fn rectangle_project() -> Project {
        let mut editor = Editor::default();
        editor
            .execute(Command::AddRectangle {
                name: "Front".into(),
                origin: [0., 0.],
                width: 0.5,
                height: 0.6,
            })
            .unwrap();
        editor.project
    }

    #[test]
    fn gltf_embeds_the_same_scene_and_buffer_as_glb() {
        let project = rectangle_project();
        let mesh = project.mesh().unwrap();
        let directory = std::env::temp_dir().join(format!("weftforge-gltf-{}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("rest.gltf");
        mesh.export_gltf(&project, &path).unwrap();
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);

        let glb = encode_glb(&mesh, &project).unwrap();
        let json_end = 20 + u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let glb_scene: Value = serde_json::from_slice(&glb[20..json_end]).unwrap();
        let binary = &glb[json_end + 8..];
        let mut gltf: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let uri = gltf["buffers"][0]
            .as_object_mut()
            .unwrap()
            .remove("uri")
            .unwrap();
        assert_eq!(gltf, glb_scene);
        assert_eq!(gltf["buffers"][0]["byteLength"], binary.len());
        let payload = uri
            .as_str()
            .unwrap()
            .strip_prefix("data:application/octet-stream;base64,")
            .unwrap();
        let symbols = payload.trim_end_matches('=');
        assert_eq!(payload.len(), binary.len().div_ceil(3) * 4);
        assert_eq!(payload.len() - symbols.len(), (3 - binary.len() % 3) % 3);

        let mut decoded = Vec::new();
        let mut word = 0_u32;
        let mut bits = 0;
        for byte in symbols.bytes() {
            let digit = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => panic!("invalid base64 digit"),
            };
            word = (word << 6) | u32::from(digit);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                decoded.push((word >> bits) as u8);
            }
        }
        assert_eq!(word & ((1 << bits) - 1), 0);
        assert_eq!(decoded, binary);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn stale_mesh_or_float32_collapse_preserves_the_previous_glb() {
        let project = &rectangle_project();
        let mut mesh = project.mesh().unwrap();
        let directory = std::env::temp_dir().join(format!("weftforge-glb-{}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("previous.glb");
        mesh.export_glb(project, &path).unwrap();
        let saved = fs::read(&path).unwrap();
        mesh.doc_revision = 0;
        assert!(mesh.export_glb(project, &path).is_err());
        assert_eq!(fs::read(&path).unwrap(), saved);
        mesh.doc_revision = project.revision;
        for position in &mut mesh.positions {
            position[0] += 1.0e9;
        }
        let error = mesh.export_glb(project, &path).unwrap_err();
        assert!(error.to_string().contains("collapsed triangle"));
        assert_eq!(fs::read(&path).unwrap(), saved);
        fs::remove_dir_all(directory).unwrap();
    }
}
