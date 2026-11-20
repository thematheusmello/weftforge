use super::{ImportedContour, ImportedPattern};
use crate::{Command, Error, Placement, Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
};

pub const MAX_OBJ_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_MTL_BYTES: usize = 256 * 1024;
pub const OBJ_IMPORT_SCOPE: &str = "Traces planar triangle UV islands into editable straight boundaries and cutouts with fresh IDs. Each island keeps its physical scale and rigid 3D placement. Assigned MTL names and diffuse colors use default cloth physics. Sewing, original curves, shading normals, and simulation history are unavailable. Folded or distorted panels and texture maps are unsupported.";
const MAX_PIECES: usize = 64;
const MAX_SPANS: usize = 4096;

#[derive(Debug, Clone)]
pub struct ObjPatternImport {
    obj: String,
    mtl: Option<String>,
    library: Option<String>,
    pub metres_per_unit: f64,
    pub(crate) patterns: Vec<(ImportedPattern, Placement, Option<String>)>,
    pub(crate) materials: BTreeMap<String, [f32; 3]>,
}

impl ObjPatternImport {
    pub fn piece_count(&self) -> usize {
        self.patterns.len()
    }
    pub fn hole_count(&self) -> usize {
        self.patterns.iter().map(|(p, _, _)| p.holes.len()).sum()
    }
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }
    pub fn material_library(&self) -> Option<&str> {
        self.library.as_deref()
    }

    pub fn command(&self) -> Command {
        Command::ImportObjPatterns {
            obj: self.obj.clone(),
            mtl: self.mtl.clone(),
            metres_per_unit: self.metres_per_unit,
        }
    }

    pub fn load(path: impl AsRef<Path>, metres_per_unit: f64) -> Result<Self> {
        let path = path.as_ref();
        let obj = read_file(path, MAX_OBJ_BYTES)?;
        let source = input_text(&obj, MAX_OBJ_BYTES)?;
        let library = library_name(source)?;
        let mtl = library
            .map(|name| read_file(&path.with_file_name(name), MAX_MTL_BYTES))
            .transpose()?;
        Self::from_bytes(&obj, mtl.as_deref(), metres_per_unit)
    }

    /// OBJ coordinates use the caller's physical unit. Texture coordinates may use a
    /// different uniform scale on each island. Input normals affect shading only.
    pub fn from_bytes(obj: &[u8], mtl: Option<&[u8]>, metres_per_unit: f64) -> Result<Self> {
        ensure(
            metres_per_unit.is_finite() && metres_per_unit > 0.,
            "OBJ metres per unit must be finite and positive",
        )?;
        let source = input_text(obj, MAX_OBJ_BYTES)?;
        let library = library_name(source)?;
        ensure(
            library.is_some() == mtl.is_some(),
            "OBJ requires exactly its declared MTL library",
        )?;
        let mtl = mtl
            .map(|bytes| input_text(bytes, MAX_MTL_BYTES))
            .transpose()?;
        let palette = parse_mtl(mtl.unwrap_or_default())?;
        let mesh = Drawing::parse(source, metres_per_unit, &palette)?;
        let mut imported = Self {
            obj: source.to_owned(),
            mtl: mtl.map(str::to_owned),
            library: library.map(str::to_owned),
            metres_per_unit,
            patterns: Vec::new(),
            materials: BTreeMap::new(),
        };
        let mut names = BTreeMap::new();
        let mut spans = 0;
        for ((object, group), faces) in &mesh.groups {
            for island in islands(faces)? {
                ensure(
                    imported.patterns.len() < MAX_PIECES,
                    "OBJ exceeds the 64-pattern limit",
                )?;
                let fabric = faces[island.faces[0]].material;
                ensure(
                    island.faces.iter().all(|&i| faces[i].material == fabric),
                    "OBJ UV island has multiple material assignments",
                )?;
                if let Some(token) = fabric {
                    imported.materials.insert(
                        token.to_owned(),
                        palette[token].ok_or_else(|| {
                            Error("Assigned MTL material requires a diffuse Kd color".into())
                        })?,
                    );
                }
                let base = [*group, *object, fabric.unwrap_or("OBJ pattern")]
                    .into_iter()
                    .find(|s| !s.is_empty())
                    .unwrap();
                let count = names.entry(base).or_insert(0);
                *count += 1;
                let name = if *count == 1 {
                    base.to_owned()
                } else {
                    format!("{base} {count}")
                };
                crate::document::validate_name(&name)?;
                let (pattern, placement) = mesh.trace(faces, island, name, &mut spans)?;
                imported
                    .patterns
                    .push((pattern, placement, fabric.map(str::to_owned)));
            }
        }
        ensure(
            !imported.patterns.is_empty(),
            "OBJ contains no triangle patterns",
        )?;
        Ok(imported)
    }
}

fn read_file(path: &Path, limit: usize) -> Result<Vec<u8>> {
    #[cfg(unix)]
    let file = {
        use rustix::fs::{Mode, OFlags, open};
        // A swapped symlink must not redirect a sidecar, and opening a FIFO must not wait.
        let flags = OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK;
        File::from(open(path, flags, Mode::empty()).map_err(std::io::Error::from)?)
    };
    #[cfg(windows)]
    let file = {
        use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        let file = File::options()
            .read(true)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        ensure(
            file.metadata()?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT == 0,
            "OBJ and MTL inputs must be regular files",
        )?;
        file
    };
    #[cfg(not(any(unix, windows)))]
    let file = {
        ensure(
            std::fs::symlink_metadata(path)?.is_file(),
            "OBJ and MTL inputs must be regular files",
        )?;
        File::open(path)?
    };
    ensure(
        file.metadata()?.is_file(),
        "OBJ and MTL inputs must be regular files",
    )?;
    let mut bytes = Vec::new();
    file.take((limit + 1) as u64).read_to_end(&mut bytes)?;
    ensure(bytes.len() <= limit, "OBJ or MTL exceeds its byte limit")?;
    Ok(bytes)
}

fn input_text(bytes: &[u8], limit: usize) -> Result<&str> {
    ensure(bytes.len() <= limit, "OBJ or MTL exceeds its byte limit")?;
    let text =
        std::str::from_utf8(bytes).map_err(|_| Error("OBJ and MTL require UTF-8 text".into()))?;
    ensure(
        !text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\r' | '\n' | '\t')),
        "Invalid control character in OBJ or MTL",
    )?;
    Ok(text)
}

fn records(source: &str) -> impl Iterator<Item = (&str, &str)> {
    source.lines().filter_map(|line| {
        let line = line.split('#').next()?.trim();
        if line.is_empty() {
            return None;
        }
        Some(
            line.split_once(char::is_whitespace)
                .map_or((line, ""), |(key, rest)| (key, rest.trim())),
        )
    })
}

fn library_name(source: &str) -> Result<Option<&str>> {
    let mut library = None;
    for (_, name) in records(source).filter(|(key, _)| *key == "mtllib") {
        ensure(
            library.is_none()
                && name.len() <= 255
                && name.len() > 4
                && name.to_ascii_lowercase().ends_with(".mtl")
                && !name.contains(['/', '\\', ':'])
                && !name.starts_with('.')
                && !name.chars().any(char::is_control)
                && name
                    .split_whitespace()
                    .filter(|word| word.to_ascii_lowercase().ends_with(".mtl"))
                    .count()
                    == 1,
            "OBJ supports one local MTL basename without directory components",
        )?;
        library = Some(name);
    }
    Ok(library)
}

fn numbers<const N: usize>(args: &str) -> Result<[f64; N]> {
    let mut values = args.split_whitespace();
    let mut result = [0.; N];
    for slot in &mut result {
        *slot = values
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .filter(|x| x.is_finite())
            .ok_or_else(|| Error("OBJ/MTL record requires finite numeric components".into()))?;
    }
    ensure(
        values.next().is_none(),
        "Unexpected OBJ/MTL numeric component",
    )?;
    Ok(result)
}

type Palette<'a> = BTreeMap<&'a str, Option<[f32; 3]>>;

fn parse_mtl(source: &str) -> Result<Palette<'_>> {
    let mut palette = BTreeMap::new();
    let mut current = None;
    for (key, args) in records(source) {
        if key == "newmtl" {
            crate::document::validate_name(args)?;
            ensure(
                palette.len() < 256 && palette.insert(args, None).is_none(),
                "Duplicate MTL name or more than 256 materials",
            )?;
            current = Some(args);
            continue;
        }
        let name =
            current.ok_or_else(|| Error("MTL property requires a preceding newmtl".into()))?;
        match key {
            "Kd" => {
                let color = numbers::<3>(args)?;
                ensure(
                    color.into_iter().all(|x| (0.0..=1.).contains(&x)),
                    "MTL Kd color must be in [0,1]",
                )?;
                ensure(
                    palette
                        .insert(name, Some(color.map(|x| x as f32)))
                        .flatten()
                        .is_none(),
                    "Duplicate MTL Kd color",
                )?;
            }
            "Ka" | "Ks" | "Ke" | "Tf" => {
                numbers::<3>(args)?;
            }
            "Ns" | "Ni" => {
                numbers::<1>(args)?;
            }
            "illum" => {
                ensure(
                    args.parse::<u8>().is_ok_and(|v| v <= 10),
                    "Invalid MTL illumination model",
                )?;
            }
            "d" | "Tr" => {
                ensure(
                    numbers::<1>(args)?[0] == if key == "d" { 1. } else { 0. },
                    "Transparent OBJ fabrics are unsupported",
                )?;
            }
            _ => return Err(Error(format!("Unsupported MTL record: {key}"))),
        }
    }
    Ok(palette)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Corner {
    v: usize,
    vt: usize,
}

struct Face<'a> {
    corners: [Corner; 3],
    material: Option<&'a str>,
}

#[derive(Default)]
struct Drawing<'a> {
    xyz: Vec<[f64; 3]>,
    uv: Vec<[f64; 2]>,
    groups: Vec<((&'a str, &'a str), Vec<Face<'a>>)>,
}

fn index(token: &str, count: usize) -> Result<usize> {
    let n = token
        .parse::<i64>()
        .map_err(|_| Error("Invalid OBJ index".into()))?;
    let resolved = if n < 0 { count as i64 + n } else { n - 1 };
    ensure(
        (0..count as i64).contains(&resolved),
        "OBJ index outside declared attribute table",
    )?;
    Ok(resolved as usize)
}

fn corner(token: &str, counts: [usize; 3]) -> Result<Corner> {
    let mut parts = token.split('/');
    let v = index(parts.next().unwrap(), counts[0])?;
    let vt = index(
        parts
            .next()
            .ok_or_else(|| Error("OBJ face requires texture coordinates".into()))?,
        counts[1],
    )?;
    if let Some(normal) = parts.next() {
        index(normal, counts[2])?;
    }
    ensure(parts.next().is_none(), "Unexpected OBJ face attribute")?;
    Ok(Corner { v, vt })
}

impl<'a> Drawing<'a> {
    fn parse(source: &'a str, metres_per_unit: f64, palette: &Palette<'_>) -> Result<Self> {
        let mut mesh = Self::default();
        let mut group_ids = BTreeMap::new();
        let (mut object, mut group, mut material) = ("", "", None);
        let (mut normals, mut triangles) = (0, 0);
        for (key, args) in records(source) {
            match key {
                "v" => {
                    ensure(
                        mesh.xyz.len() < crate::MAX_MESH_VERTICES,
                        "OBJ position limit exceeded",
                    )?;
                    let p = numbers::<3>(args)?.map(|x| x * metres_per_unit);
                    crate::document::validate_coordinates(&p)?;
                    mesh.xyz.push(p);
                }
                "vt" => {
                    ensure(
                        mesh.uv.len() < crate::MAX_MESH_VERTICES,
                        "OBJ texture coordinate limit exceeded",
                    )?;
                    mesh.uv.push(numbers(args)?);
                }
                "vn" => {
                    ensure(
                        normals < crate::MAX_MESH_VERTICES,
                        "OBJ normal limit exceeded",
                    )?;
                    ensure(
                        numbers::<3>(args)?.iter().any(|x| *x != 0.),
                        "Zero OBJ normal",
                    )?;
                    normals += 1;
                }
                "mtllib" => {} // The single declaration has already been resolved.
                "usemtl" => {
                    ensure(
                        palette.contains_key(args),
                        "Unknown OBJ material assignment",
                    )?;
                    material = Some(args);
                }
                "s" => {
                    ensure(
                        args == "off" || args == "on" || args.parse::<u32>().is_ok(),
                        "Invalid OBJ smoothing group",
                    )?;
                }
                "o" => {
                    crate::document::validate_name(args)?;
                    object = args;
                }
                "g" => {
                    ensure(
                        args.split_whitespace().count() <= 1,
                        "Overlapping OBJ groups are unsupported",
                    )?;
                    if !args.is_empty() {
                        crate::document::validate_name(args)?;
                    }
                    group = args;
                }
                "f" => {
                    ensure(
                        triangles < crate::MAX_MESH_TRIANGLES,
                        "OBJ triangle limit exceeded",
                    )?;
                    let mut parts = args.split_whitespace();
                    let mut corners = [Corner { v: 0, vt: 0 }; 3];
                    for c in &mut corners {
                        *c = corner(
                            parts
                                .next()
                                .ok_or_else(|| Error("OBJ face requires three corners".into()))?,
                            [mesh.xyz.len(), mesh.uv.len(), normals],
                        )?;
                    }
                    ensure(
                        parts.next().is_none(),
                        "OBJ pattern tracing requires triangle faces",
                    )?;
                    let [a, b, c] = corners.map(|c| mesh.uv[c.vt]);
                    let area = det(sub(b, a), sub(c, a));
                    ensure(area.is_finite() && area != 0., "Degenerate OBJ UV triangle")?;
                    let next = mesh.groups.len();
                    let idx = *group_ids.entry((object, group)).or_insert(next);
                    if idx == next {
                        ensure(idx < MAX_PIECES, "OBJ exceeds the 64-pattern limit")?;
                        mesh.groups.push(((object, group), Vec::new()));
                    }
                    mesh.groups[idx].1.push(Face { corners, material });
                    triangles += 1;
                }
                _ => return Err(Error(format!("Unsupported OBJ record: {key}"))),
            }
        }
        Ok(mesh)
    }
}

struct Island {
    faces: Vec<usize>,
    loops: Vec<Vec<Corner>>,
}

fn islands(faces: &[Face<'_>]) -> Result<Vec<Island>> {
    // Only an entire shared (position, texture index) edge joins two triangles.
    // Normal splits affect shading; UV seams and object/group boundaries cut cloth.
    let mut edges = BTreeMap::new();
    let mut neighbours = vec![[None; 3]; faces.len()];
    for (i, face) in faces.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (face.corners[k], face.corners[(k + 1) % 3]);
            let (j, other): (usize, usize) = *edges.entry((a.min(b), a.max(b))).or_insert((i, k));
            if j != i {
                ensure(
                    neighbours[j][other].is_none() && faces[j].corners[other] == b,
                    "Nonmanifold or inconsistently wound OBJ edge",
                )?;
                neighbours[j][other] = Some(i);
                neighbours[i][k] = Some(j);
            }
        }
    }
    let mut islands = Vec::new();
    let mut visited = vec![false; faces.len()];
    let mut owner = BTreeMap::new();
    for seed in 0..faces.len() {
        if visited[seed] {
            continue;
        }
        ensure(
            islands.len() < MAX_PIECES,
            "OBJ exceeds the 64-pattern limit",
        )?;
        let mut component = Vec::new();
        let mut outward = BTreeMap::new();
        let mut ends = BTreeSet::new();
        let mut stack = vec![seed];
        visited[seed] = true;
        while let Some(i) = stack.pop() {
            component.push(i);
            for (k, neighbour) in neighbours[i].iter().enumerate() {
                let a = faces[i].corners[k];
                ensure(
                    *owner.entry(a).or_insert(seed) == seed,
                    "OBJ UV islands touch at a nonmanifold vertex",
                )?;
                if let Some(&j) = neighbour.as_ref() {
                    if !visited[j] {
                        visited[j] = true;
                        stack.push(j);
                    }
                } else {
                    let b = faces[i].corners[(k + 1) % 3];
                    ensure(
                        outward.insert(a, b).is_none() && ends.insert(b),
                        "Branched OBJ boundary",
                    )?;
                }
            }
        }
        ensure(!outward.is_empty(), "OBJ UV island has no exterior")?;
        let mut loops = Vec::new();
        while let Some((&start, _)) = outward.first_key_value() {
            let mut ring = Vec::new();
            let mut at = start;
            loop {
                ring.push(at);
                at = outward
                    .remove(&at)
                    .ok_or_else(|| Error("Open OBJ boundary".into()))?;
                if at == start {
                    break;
                }
            }
            ensure(ring.len() >= 3, "Degenerate OBJ boundary")?;
            loops.push(ring);
        }
        islands.push(Island {
            faces: component,
            loops,
        });
    }
    Ok(islands)
}

fn sub<const N: usize>(a: [f64; N], b: [f64; N]) -> [f64; N] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot<const N: usize>(a: [f64; N], b: [f64; N]) -> f64 {
    a.into_iter().zip(b).map(|(x, y)| x * y).sum()
}
fn length<const N: usize>(p: [f64; N]) -> f64 {
    p.into_iter().fold(0., f64::hypot)
}
fn det(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

impl Drawing<'_> {
    fn trace(
        &self,
        faces: &[Face<'_>],
        island: Island,
        name: String,
        spans: &mut usize,
    ) -> Result<(ImportedPattern, Placement)> {
        let points = || island.faces.iter().flat_map(|&i| faces[i].corners);
        let p = |corner: Corner| self.xyz[corner.v];
        let t = |corner: Corner| self.uv[corner.vt];

        let [a, b, c] = faces[island.faces[0]].corners;
        let winding = det(sub(t(b), t(a)), sub(t(c), t(a))).signum();
        // Reflect a clockwise UV chart to retain the face orientation on remeshing.
        let uv = |corner| {
            let [u, v] = t(corner);
            [u, winding * v]
        };
        let a = points().next().unwrap();
        let b = points()
            .max_by(|&b, &c| length(sub(uv(b), uv(a))).total_cmp(&length(sub(uv(c), uv(a)))))
            .unwrap();
        let ab = sub(uv(b), uv(a));
        let uv_length = length(ab);
        let physical = sub(p(b), p(a));
        let extent = length(physical);
        let scale = extent / uv_length;
        ensure(
            scale.is_finite() && scale > 0.,
            "Cannot establish physical OBJ UV scale",
        )?;
        let direction = ab.map(|x| x / uv_length);
        let x = physical.map(|x| x / extent);
        // Widely spaced anchors keep the fit independent of tiny boundary triangles.
        let height = |c| det(direction, sub(uv(c), uv(a)));
        let c = points()
            .max_by(|&b, &c| height(b).abs().total_cmp(&height(c).abs()))
            .unwrap();
        let ac = sub(p(c), p(a));
        let along = dot(ac, x);
        let normal: [f64; 3] = std::array::from_fn(|i| ac[i] - along * x[i]);
        let depth = length(normal);
        ensure(
            depth.is_finite() && depth > extent * 1e-12,
            "Degenerate physical OBJ island",
        )?;
        let y = normal.map(|v| v / depth * height(c).signum());
        let u = std::array::from_fn(|i| direction[0] * x[i] - direction[1] * y[i]);
        let v = std::array::from_fn(|i| direction[1] * x[i] + direction[0] * y[i]);
        let mut placement = Placement::from_rotation_basis(u, v);
        let pattern_point = |c| uv(c).map(|t| t * scale);
        placement.translation = sub(p(a), placement.transform(pattern_point(a)));
        crate::document::validate_coordinates(&placement.translation)?;
        let tolerance = (2e-6 + 1e-6 * extent).min(1e-3 * extent);
        for &i in &island.faces {
            let corners = faces[i].corners;
            let [a, b, c] = corners.map(pattern_point);
            ensure(
                det(sub(b, a), sub(c, a)) > 0.,
                "Overlapping, folded, or degenerate OBJ UV triangles",
            )?;
            let [a, b, c] = corners.map(p);
            let ab = sub(b, a);
            let ac = sub(c, a);
            ensure(
                det([dot(ab, u), dot(ab, v)], [dot(ac, u), dot(ac, v)]) > 0.,
                "Folded or degenerate physical OBJ triangle",
            )?;
            for c in corners {
                crate::document::validate_coordinates(&pattern_point(c))?;
                ensure(
                    length(sub(placement.transform(pattern_point(c)), p(c))) <= tolerance,
                    "OBJ garment requires a planar, uniformly scaled UV-to-XYZ mapping",
                )?;
            }
        }

        let mut exterior = None;
        let mut holes = Vec::new();
        for ring in island.loops {
            let mut boundary: Vec<_> = ring.into_iter().map(pattern_point).collect();
            simplify(&mut boundary)?;
            *spans += boundary.len();
            ensure(
                *spans <= MAX_SPANS,
                "OBJ exceeds the 4096 traced-edge limit",
            )?;
            let area: f64 = boundary[1..]
                .windows(2)
                .map(|ab| det(sub(ab[0], boundary[0]), sub(ab[1], boundary[0])))
                .sum();
            ensure(area.is_finite() && area != 0., "Degenerate traced OBJ loop")?;
            if area > 0. {
                ensure(
                    exterior.replace(boundary).is_none(),
                    "OBJ island has multiple exterior rings",
                )?;
            } else {
                holes.push(boundary);
            }
        }
        let vertices = exterior.ok_or_else(|| Error("OBJ island has no exterior ring".into()))?;
        Ok((
            ImportedPattern {
                name,
                exterior: ImportedContour {
                    vertices,
                    controls: Vec::new(),
                },
                holes,
            },
            placement,
        ))
    }
}

fn simplify(ring: &mut Vec<[f64; 2]>) -> Result<()> {
    let n = ring.len();
    let mut sharpest = (f64::INFINITY, 0);
    for i in 0..n {
        let before = sub(ring[i], ring[(i + n - 1) % n]);
        let after = sub(ring[(i + 1) % n], ring[i]);
        let cosine = dot(before, after) / (length(before) * length(after));
        ensure(cosine.is_finite(), "Zero-length traced OBJ edge")?;
        if cosine < sharpest.0 {
            sharpest = (cosine, i);
        }
    }
    ring.rotate_left(sharpest.1);
    ring.push(ring[0]);
    let mut output = Vec::new();
    let mut start = 0;
    // Each skipped vertex stays within 10 nm of the same directed line.
    while start < n {
        output.push(ring[start]);
        let first = sub(ring[start + 1], ring[start]);
        let dir = first.map(|v| v / length(first));
        let mut end = start + 1;
        while end < n {
            let delta = sub(ring[end + 1], ring[start]);
            if det(dir, delta).abs() > 1e-8 || dot(dir, sub(ring[end + 1], ring[end])) <= 0. {
                break;
            }
            end += 1;
        }
        start = end;
    }
    ensure(output.len() >= 3, "Degenerate simplified OBJ ring")?;
    *ring = output;
    Ok(())
}
