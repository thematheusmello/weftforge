use super::ImportedPattern;
use crate::{Command, Error, Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Read,
    path::Path,
};

pub const MAX_DXF_BYTES: usize = 1024 * 1024;
const MAX_DXF_PAIRS: usize = 131072;
const MAX_DXF_PIECES: usize = 64;
const MAX_DXF_SPANS: usize = 4096;
pub const DXF_IMPORT_SCOPE: &str = "Imported editable exterior boundaries with layer names and fresh IDs only. Original names/IDs, sewing, materials, grain, internal details, allowances, grading and simulation are not restored; new pieces use the first material and default 3D placement.";

/// Strict FILE-005-DXF-import-v1 reader for the existing boundary export profile.
#[derive(Debug, Clone)]
pub struct DxfPatternImport {
    source: String,
    pub(crate) patterns: Vec<ImportedPattern>,
}

impl DxfPatternImport {
    pub fn piece_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn command(&self) -> Command {
        Command::ImportDxfPatterns {
            dxf: self.source.clone(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        ensure(
            fs::metadata(path.as_ref())?.is_file(),
            "DXF input must be a regular file",
        )?;
        let file = File::open(path)?;
        ensure(
            file.metadata()?.is_file(),
            "DXF input must be a regular file",
        )?;
        let mut bytes = Vec::new();
        file.take((MAX_DXF_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        ensure(bytes.len() <= MAX_DXF_BYTES, "DXF exceeds the 1 MiB limit")?;
        ensure(
            bytes
                .iter()
                .all(|byte| matches!(byte, b'\t' | b'\r' | b'\n' | b' '..=b'~')),
            "DXF requires ASCII text without binary data or a BOM",
        )?;
        let source = std::str::from_utf8(bytes).map_err(|error| Error(error.to_string()))?;
        let mut reader = Drawing::new(source)?;
        reader.section("HEADER")?;
        reader.fields(&[
            (9, "$ACADVER"),
            (1, "AC1015"),
            (9, "$INSUNITS"),
            (70, "4"),
            (9, "$MEASUREMENT"),
            (70, "1"),
            (9, "$LUNITS"),
            (70, "2"),
            (9, "$HANDSEED"),
        ])?;
        let handseed = reader.hex(5)?;
        reader.expect(0, "ENDSEC")?;
        reader.section("CLASSES")?;
        reader.expect(0, "ENDSEC")?;
        reader.section("TABLES")?;
        let layers = reader.tables()?;
        reader.expect(0, "ENDSEC")?;
        reader.section("BLOCKS")?;
        for (name, space, begin, end, paper) in [
            (
                "*Model_Space",
                "model-space",
                "model-block",
                "model-end",
                false,
            ),
            (
                "*Paper_Space",
                "paper-space",
                "paper-block",
                "paper-end",
                true,
            ),
        ] {
            reader.record("BLOCK", Some(begin), Some(space))?;
            reader.expect(100, "AcDbEntity")?;
            if paper {
                reader.expect(67, "1")?;
            }
            reader.fields(&[
                (8, "0"),
                (100, "AcDbBlockBegin"),
                (2, name),
                (70, "0"),
                (10, "0"),
                (20, "0"),
                (30, "0"),
                (3, name),
                (1, ""),
            ])?;
            reader.record("ENDBLK", Some(end), Some(space))?;
            reader.expect(100, "AcDbEntity")?;
            if paper {
                reader.expect(67, "1")?;
            }
            reader.fields(&[(8, "0"), (100, "AcDbBlockEnd")])?;
        }
        reader.expect(0, "ENDSEC")?;
        reader.section("ENTITIES")?;
        let mut patterns: Vec<ImportedPattern> = layers
            .iter()
            .map(|name| ImportedPattern {
                name: (*name).to_owned(),
                exterior: Default::default(),
                holes: Vec::new(),
            })
            .collect();
        let mut endpoints: Vec<Option<[[f64; 2]; 2]>> = vec![None; layers.len()];
        let mut spans = 0;
        while reader.peek() != Some((0, "ENDSEC")) {
            spans += 1;
            ensure(spans <= MAX_DXF_SPANS, "DXF exceeds the 4096-entity limit")?;
            let kind = reader.value(0)?;
            ensure(
                matches!(kind, "LINE" | "SPLINE"),
                "DXF supports only LINE and cubic SPLINE boundary entities",
            )?;
            reader.handle(5, None)?;
            reader.reference("model-space")?;
            reader.expect(100, "AcDbEntity")?;
            let layer = reader.value(8)?;
            let index = layers
                .iter()
                .position(|name| *name == layer)
                .ok_or_else(|| Error("DXF boundary must use a declared WF_PIECE layer".into()))?;
            reader.fields(&[(67, "0"), (410, "Model"), (370, "-1")])?;
            let (points, controls) = if kind == "LINE" {
                reader.fields(&[(100, "AcDbLine"), (39, "0")])?;
                ([reader.point(10)?, reader.point(11)?], None)
            } else {
                reader.fields(&[
                    (100, "AcDbSpline"),
                    (210, "0"),
                    (220, "0"),
                    (230, "1"),
                    (70, "8"),
                    (71, "3"),
                    (72, "8"),
                    (73, "4"),
                    (74, "0"),
                ])?;
                for knot in ["0", "0", "0", "0", "1", "1", "1", "1"] {
                    reader.expect(40, knot)?;
                }
                let points = [
                    reader.point(10)?,
                    reader.point(10)?,
                    reader.point(10)?,
                    reader.point(10)?,
                ];
                (
                    [points[0], points[3]],
                    Some([metres(points[1])?, metres(points[2])?]),
                )
            };
            if let Some([_, end]) = endpoints[index] {
                ensure(
                    end == points[0],
                    "DXF piece entities must form one exactly adjacent ordered contour",
                )?;
            }
            let first = endpoints[index].map_or(points[0], |[first, _]| first);
            endpoints[index] = Some([first, points[1]]);
            patterns[index].exterior.vertices.push(metres(points[0])?);
            patterns[index].exterior.controls.push(controls);
        }
        ensure(!patterns.is_empty(), "DXF contains no pattern layers")?;
        for (pattern, ends) in patterns.iter().zip(endpoints) {
            ensure(
                pattern.exterior.vertices.len() >= 3
                    && ends.is_some_and(|[first, end]| first == end),
                "Every DXF piece layer needs one closed contour with at least three spans",
            )?;
        }
        reader.expect(0, "ENDSEC")?;
        reader.section("OBJECTS")?;
        reader.objects()?;
        reader.fields(&[(0, "ENDSEC"), (0, "EOF")])?;
        ensure(
            reader.peek().is_none(),
            "DXF contains trailing data after EOF",
        )?;
        for (handle, role) in &reader.references {
            ensure(
                reader.roles.get(role) == Some(handle),
                "DXF has an unresolved or incorrect owner/layout link",
            )?;
        }
        ensure(
            reader.handles.iter().all(|handle| *handle < handseed),
            "DXF HANDSEED must be greater than every handle",
        )?;
        Ok(Self {
            source: source.to_owned(),
            patterns,
        })
    }
}

fn metres(point: [f64; 2]) -> Result<[f64; 2]> {
    let point = point.map(|value| value / 1000.0);
    crate::document::validate_coordinates(&point)?;
    Ok(point)
}

struct Drawing<'a> {
    pairs: Vec<(i16, &'a str)>,
    offset: usize,
    handles: BTreeSet<u64>,
    roles: BTreeMap<&'static str, u64>,
    references: Vec<(u64, &'static str)>,
}

impl<'a> Drawing<'a> {
    fn new(source: &'a str) -> Result<Self> {
        let mut lines = source.lines();
        let mut pairs = Vec::new();
        let mut count = 0;
        while let Some(code) = lines.next() {
            count += 1;
            ensure(count <= MAX_DXF_PAIRS, "DXF exceeds the 131072-pair limit")?;
            let value = lines
                .next()
                .ok_or_else(|| Error("DXF has an incomplete group-code/value pair".into()))?;
            let code = code
                .trim()
                .parse::<i16>()
                .map_err(|_| Error("DXF has an invalid group code".into()))?;
            if code != 999 {
                pairs.push((code, value.trim()));
            }
        }
        Ok(Self {
            pairs,
            offset: 0,
            handles: BTreeSet::new(),
            roles: BTreeMap::new(),
            references: Vec::new(),
        })
    }

    fn peek(&self) -> Option<(i16, &'a str)> {
        self.pairs.get(self.offset).copied()
    }

    fn value(&mut self, code: i16) -> Result<&'a str> {
        let pair = self
            .peek()
            .ok_or_else(|| Error(format!("DXF ended before group {code}")))?;
        ensure(
            pair.0 == code,
            &format!(
                "Unsupported or misplaced DXF field: expected group {code}, found {}",
                pair.0
            ),
        )?;
        self.offset += 1;
        Ok(pair.1)
    }

    fn expect(&mut self, code: i16, expected: &str) -> Result<()> {
        let actual = self.value(code)?;
        let matches = match code {
            10..=59 | 140..=149 => actual
                .parse::<f64>()
                .ok()
                .is_some_and(|number| number.is_finite() && Some(number) == expected.parse().ok()),
            60..=79 | 270..=299 | 370 => actual
                .parse::<i64>()
                .ok()
                .is_some_and(|number| Some(number) == expected.parse().ok()),
            _ => actual == expected,
        };
        ensure(
            matches,
            &format!("Unsupported DXF group {code}: expected {expected}"),
        )
    }

    fn fields(&mut self, fields: &[(i16, &str)]) -> Result<()> {
        for (code, value) in fields {
            self.expect(*code, value)?;
        }
        Ok(())
    }

    fn hex(&mut self, code: i16) -> Result<u64> {
        let value = self.value(code)?;
        ensure(
            !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "DXF handles must be hexadecimal",
        )?;
        u64::from_str_radix(value, 16).map_err(|_| Error("DXF handle exceeds u64".into()))
    }

    fn handle(&mut self, code: i16, role: Option<&'static str>) -> Result<()> {
        let handle = self.hex(code)?;
        ensure(
            handle > 0 && self.handles.insert(handle),
            "DXF handles must be positive and unique",
        )?;
        if let Some(role) = role {
            self.roles.insert(role, handle);
        }
        Ok(())
    }

    fn reference(&mut self, role: &'static str) -> Result<()> {
        self.link(330, role)
    }

    fn link(&mut self, code: i16, role: &'static str) -> Result<()> {
        let handle = self.hex(code)?;
        self.references.push((handle, role));
        Ok(())
    }

    fn record(
        &mut self,
        kind: &str,
        role: Option<&'static str>,
        owner: Option<&'static str>,
    ) -> Result<()> {
        self.expect(0, kind)?;
        self.handle(if kind == "DIMSTYLE" { 105 } else { 5 }, role)?;
        if let Some(owner) = owner {
            self.reference(owner)?;
        } else {
            ensure(self.hex(330)? == 0, "DXF root/table owner must be null")?;
        }
        Ok(())
    }

    fn section(&mut self, name: &str) -> Result<()> {
        self.fields(&[(0, "SECTION"), (2, name)])
    }

    fn table(&mut self, name: &'static str, count: usize) -> Result<()> {
        self.expect(0, "TABLE")?;
        self.expect(2, name)?;
        self.handle(5, Some(name))?;
        ensure(self.hex(330)? == 0, "DXF table owner must be null")?;
        self.expect(100, "AcDbSymbolTable")?;
        self.expect(70, &count.to_string())
    }

    fn table_record(
        &mut self,
        name: &str,
        role: Option<&'static str>,
        table: &'static str,
        subclass: &str,
    ) -> Result<()> {
        self.record(name, role, Some(table))?;
        self.fields(&[(100, "AcDbSymbolTableRecord"), (100, subclass)])
    }

    fn tables(&mut self) -> Result<Vec<&'a str>> {
        self.table("VPORT", 0)?;
        self.expect(0, "ENDTAB")?;
        self.table("LTYPE", 3)?;
        for name in ["BYBLOCK", "BYLAYER", "CONTINUOUS"] {
            self.table_record("LTYPE", None, "LTYPE", "AcDbLinetypeTableRecord")?;
            self.fields(&[
                (2, name),
                (70, "0"),
                (3, ""),
                (72, "65"),
                (73, "0"),
                (40, "0"),
            ])?;
        }
        self.expect(0, "ENDTAB")?;
        self.fields(&[(0, "TABLE"), (2, "LAYER")])?;
        self.handle(5, Some("LAYER"))?;
        ensure(self.hex(330)? == 0, "DXF layer table owner must be null")?;
        self.expect(100, "AcDbSymbolTable")?;
        let count = self
            .value(70)?
            .parse::<usize>()
            .map_err(|_| Error("DXF layer count must be an integer".into()))?;
        ensure(
            (1..=MAX_DXF_PIECES + 1).contains(&count),
            "DXF exceeds the 64-piece limit or lacks layer 0",
        )?;
        let mut layers = Vec::new();
        for index in 0..count {
            self.table_record("LAYER", None, "LAYER", "AcDbLayerTableRecord")?;
            let name = self.value(2)?;
            if index == 0 {
                ensure(name == "0", "DXF must declare layer 0 first")?;
            } else {
                let suffix = name
                    .strip_prefix("WF_PIECE_")
                    .ok_or_else(|| Error("DXF supports only WF_PIECE_<id> layers".into()))?;
                let id = suffix
                    .parse::<u64>()
                    .map_err(|_| Error("DXF piece layer needs a positive u64 ID".into()))?;
                ensure(
                    id > 0 && suffix == id.to_string() && !layers.contains(&name),
                    "DXF piece layer IDs must be unique positive unpadded decimal",
                )?;
                layers.push(name);
            }
            self.fields(&[(70, "0"), (62, "7"), (6, "CONTINUOUS")])?;
        }
        self.expect(0, "ENDTAB")?;
        self.table("STYLE", 1)?;
        self.table_record(
            "STYLE",
            Some("standard-style"),
            "STYLE",
            "AcDbTextStyleTableRecord",
        )?;
        self.fields(&[
            (2, "STANDARD"),
            (70, "0"),
            (40, "0"),
            (41, "1"),
            (50, "0"),
            (71, "0"),
            (42, "2.5"),
            (3, "txt"),
            (4, ""),
        ])?;
        self.expect(0, "ENDTAB")?;
        for name in ["VIEW", "UCS"] {
            self.table(name, 0)?;
            self.expect(0, "ENDTAB")?;
        }
        self.table("APPID", 1)?;
        self.table_record("APPID", None, "APPID", "AcDbRegAppTableRecord")?;
        self.fields(&[(2, "ACAD"), (70, "0"), (0, "ENDTAB")])?;
        self.table("DIMSTYLE", 1)?;
        self.fields(&[(100, "AcDbDimStyleTable"), (71, "1")])?;
        self.table_record("DIMSTYLE", None, "DIMSTYLE", "AcDbDimStyleTableRecord")?;
        self.fields(&[(2, "STANDARD"), (70, "0")])?;
        self.link(340, "standard-style")?;
        self.expect(0, "ENDTAB")?;
        self.table("BLOCK_RECORD", 2)?;
        for (name, space, layout) in [
            ("*Model_Space", "model-space", "model-layout"),
            ("*Paper_Space", "paper-space", "paper-layout"),
        ] {
            self.table_record(
                "BLOCK_RECORD",
                Some(space),
                "BLOCK_RECORD",
                "AcDbBlockTableRecord",
            )?;
            self.expect(2, name)?;
            self.link(340, layout)?;
        }
        self.expect(0, "ENDTAB")?;
        Ok(layers)
    }

    fn objects(&mut self) -> Result<()> {
        for (role, owner) in [
            ("root-dictionary", None),
            ("group-dictionary", Some("root-dictionary")),
            ("layout-dictionary", Some("root-dictionary")),
        ] {
            self.record("DICTIONARY", Some(role), owner)?;
            self.fields(&[(100, "AcDbDictionary"), (280, "0"), (281, "1")])?;
            let entries: &[(&str, &str)] = match role {
                "root-dictionary" => &[
                    ("ACAD_GROUP", "group-dictionary"),
                    ("ACAD_LAYOUT", "layout-dictionary"),
                ],
                "layout-dictionary" => &[("Model", "model-layout"), ("Layout1", "paper-layout")],
                _ => &[],
            };
            for (name, target) in entries {
                self.expect(3, name)?;
                self.link(350, target)?;
            }
        }
        for (name, role, space, plot, order) in [
            ("Model", "model-layout", "model-space", "1024", "0"),
            ("Layout1", "paper-layout", "paper-space", "0", "1"),
        ] {
            self.record("LAYOUT", Some(role), Some("layout-dictionary"))?;
            self.fields(&[
                (100, "AcDbPlotSettings"),
                (70, plot),
                (100, "AcDbLayout"),
                (1, name),
                (70, "1"),
                (71, order),
            ])?;
            for code in [
                10, 20, 11, 21, 12, 22, 32, 14, 24, 34, 15, 25, 35, 146, 13, 23, 33,
            ] {
                self.expect(code, "0")?;
            }
            self.fields(&[
                (16, "1"),
                (26, "0"),
                (36, "0"),
                (17, "0"),
                (27, "1"),
                (37, "0"),
                (76, "0"),
            ])?;
            self.reference(space)?;
        }
        Ok(())
    }

    fn point(&mut self, code: i16) -> Result<[f64; 2]> {
        let mut point = [0.; 2];
        for (axis, value) in point.iter_mut().enumerate() {
            *value = self
                .value(code + 10 * axis as i16)?
                .parse::<f64>()
                .map_err(|_| Error("Invalid DXF coordinate".into()))?;
            ensure(value.is_finite(), "DXF coordinates must be finite")?;
        }
        self.expect(code + 20, "0")?;
        Ok(point)
    }
}
