use super::{ImportedContour, ImportedPattern};
use crate::{Command, Error, Result, ensure};
use quick_xml::{
    Reader, XmlVersion,
    events::{BytesStart, Event},
};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Read,
    path::Path,
};

pub const MAX_SVG_BYTES: usize = 1024 * 1024;
const MAX_SVG_PIECES: usize = 64;
const MAX_SVG_SPANS: usize = 4096;
pub const SVG_IMPORT_SCOPE: &str = "Imported editable exterior boundaries, names and disjoint straight-sided cutouts. Appearance, descriptions, sewing, grain, internal details, allowances, grading and simulation are omitted; new pieces use the first material and default 3D placement.";

/// The bounded FILE-005-SVG-import-v1 drawing and its physical scale.
#[derive(Debug, Clone)]
pub struct SvgPatternImport {
    pub metres_per_unit: f64,
    source: String,
    pub(crate) patterns: Vec<ImportedPattern>,
}

impl SvgPatternImport {
    pub fn piece_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn hole_count(&self) -> usize {
        self.patterns
            .iter()
            .map(|pattern| pattern.holes.len())
            .sum()
    }

    pub fn command(&self) -> Command {
        Command::ImportSvgPatterns {
            svg: self.source.clone(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        ensure(
            fs::metadata(path.as_ref())?.is_file(),
            "SVG input must be a regular file",
        )?;
        let file = File::open(path)?;
        ensure(
            file.metadata()?.is_file(),
            "SVG input must be a regular file",
        )?;
        let mut bytes = Vec::new();
        file.take((MAX_SVG_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        ensure(bytes.len() <= MAX_SVG_BYTES, "SVG exceeds the 1 MiB limit")?;
        let source = std::str::from_utf8(bytes).map_err(xml_error)?;
        validate_xml_chars(source)?;
        let mut reader = Reader::from_str(source);
        reader.config_mut().check_comments = true;
        reader.config_mut().expand_empty_elements = true;
        let mut stack: Vec<String> = Vec::new();
        let mut scale = None;
        let mut closed = false;
        let mut declared = false;
        let mut patterns = Vec::new();
        let mut total_spans = 0;
        loop {
            match reader.read_event().map_err(xml_error)? {
                Event::Start(element) => {
                    ensure(stack.len() < 3, "SVG exceeds the supported XML depth of 3")?;
                    let name = std::str::from_utf8(element.name().as_ref()).map_err(xml_error)?.to_owned();
                    let attributes = attributes(&element)?;
                    match (stack.last().map(String::as_str), name.as_str()) {
                        (None, "svg") => {
                            ensure(scale.is_none(), "SVG must contain exactly one root")?;
                            scale = Some(root_scale(&attributes)?);
                        }
                        (Some("svg"), "path") => {
                            ensure(patterns.len() < MAX_SVG_PIECES, "SVG exceeds the 64-piece limit")?;
                            check_attributes(&attributes, &["id", "data-name", "d", "fill", "fill-rule", "stroke", "stroke-width"])?;
                            for (key, value) in &attributes {
                                if matches!(key.as_str(), "fill" | "stroke") {
                                    let color = value.strip_prefix('#').is_some_and(|hex| {
                                        matches!(hex.len(), 3 | 4 | 6 | 8) && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
                                    });
                                    ensure(color || matches!(value.as_str(), "none" | "black" | "white"), "SVG supports only literal black, white, none or hexadecimal fill/stroke; appearance is omitted")?;
                                } else if key == "stroke-width" {
                                    ensure(single_number(value)? >= 0.0, "SVG stroke-width must be nonnegative and unitless; appearance is omitted")?;
                                } else if key == "fill-rule" {
                                    ensure(matches!(value.as_str(), "evenodd" | "nonzero"), "SVG fill-rule must be evenodd or nonzero")?;
                                }
                            }
                            let name = attributes.get("data-name").or_else(|| attributes.get("id")).cloned().unwrap_or_else(|| format!("SVG pattern {}", patterns.len() + 1));
                            crate::document::validate_name(&name)?;
                            let evenodd = attributes.get("fill-rule").is_some_and(|rule| rule == "evenodd");
                            let pattern = parse_path(name, required(&attributes, "d")?, scale.expect("root scale"), evenodd)?;
                            total_spans += pattern.exterior.vertices.len() + pattern.holes.iter().map(Vec::len).sum::<usize>();
                            ensure(total_spans <= MAX_SVG_SPANS, "SVG exceeds the 4096-span limit")?;
                            patterns.push(pattern);
                        }
                        (Some("svg" | "path"), "title" | "desc") => check_attributes(&attributes, &[])?,
                        _ => return Err(Error(format!("Unsupported SVG element or nesting: {name}; import requires direct closed path boundaries"))),
                    }
                    stack.push(name);
                }
                Event::End(element) => {
                    let name = std::str::from_utf8(element.name().as_ref()).map_err(xml_error)?.to_owned();
                    ensure(stack.pop().as_deref() == Some(&name), "SVG has an unmatched closing element")?;
                    if name == "svg" { closed = true; }
                }
                Event::Text(text) => {
                    let text = text.xml10_content().map_err(xml_error)?;
                    ensure(!text.contains("]]>") , "SVG contains invalid XML text")?;
                    ensure(descriptive(&stack) || text.bytes().all(whitespace), "SVG text is supported only inside title or desc")?;
                }
                Event::CData(text) => {
                    ensure(descriptive(&stack), "SVG CDATA is supported only inside title or desc")?;
                    validate_xml_chars(&text.xml10_content().map_err(xml_error)?)?;
                }
                Event::GeneralRef(reference) => {
                    ensure(descriptive(&stack), "SVG references are supported only in descriptive text or attributes")?;
                    let reference = format!("&{};", std::str::from_utf8(&reference).map_err(xml_error)?);
                    validate_xml_chars(&quick_xml::escape::unescape(&reference).map_err(xml_error)?)?;
                }
                Event::Decl(declaration) => {
                    ensure(!declared && scale.is_none() && source.trim_start_matches('\u{feff}').starts_with("<?xml"), "SVG XML declaration must appear once at the start")?;
                    ensure(declaration.version().map_err(xml_error)?.as_ref() == b"1.0", "SVG requires XML 1.0")?;
                    ensure(!declaration.contains(&b'&'), "SVG XML declaration cannot contain character/entity references")?;
                    let declaration = std::str::from_utf8(&declaration).map_err(xml_error)?;
                    let declaration = BytesStart::from_content(declaration, 3);
                    let values = attributes(&declaration)?;
                    check_attributes(&values, &["version", "encoding", "standalone"])?;
                    let mut previous = 0;
                    for attribute in declaration.attributes() {
                        let position = match attribute.map_err(xml_error)?.key.as_ref() { b"version" => 1, b"encoding" => 2, _ => 3 };
                        ensure(position > previous, "SVG XML declaration attributes are out of order")?;
                        previous = position;
                    }
                    ensure(values.get("encoding").is_none_or(|value| value.eq_ignore_ascii_case("UTF-8")), "SVG encoding must be UTF-8")?;
                    ensure(values.get("standalone").is_none_or(|value| matches!(value.as_str(), "yes" | "no")), "Invalid SVG XML standalone declaration")?;
                    declared = true;
                }
                Event::Comment(_) => {}
                Event::DocType(_) | Event::PI(_) => return Err(Error("SVG DTDs, entities and processing instructions are unsupported; no resources are loaded".into())),
                Event::Eof => break,
                Event::Empty(_) => unreachable!("empty elements are expanded"),
            }
        }
        ensure(closed && stack.is_empty(), "SVG document is incomplete")?;
        ensure(
            !patterns.is_empty(),
            "SVG contains no supported pattern paths",
        )?;
        Ok(Self {
            metres_per_unit: scale.expect("closed root"),
            source: source.to_owned(),
            patterns,
        })
    }
}

fn xml_error(error: impl std::fmt::Display) -> Error {
    Error(format!("Invalid SVG XML: {error}"))
}

fn validate_xml_chars(value: &str) -> Result<()> {
    ensure(value.chars().all(|ch| matches!(ch, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')), "SVG contains invalid XML characters")
}

fn descriptive(stack: &[String]) -> bool {
    stack
        .last()
        .is_some_and(|name| matches!(name.as_str(), "title" | "desc"))
}

fn attributes(element: &BytesStart<'_>) -> Result<BTreeMap<String, String>> {
    element
        .attributes()
        .enumerate()
        .map(|(index, attribute)| {
            ensure(
                index < 8,
                "SVG element exceeds the supported attribute limit",
            )?;
            let attribute = attribute.map_err(xml_error)?;
            ensure(
                !attribute.value.contains(&b'<'),
                "SVG XML attributes must escape less-than characters",
            )?;
            let key = std::str::from_utf8(attribute.key.as_ref())
                .map_err(xml_error)?
                .to_owned();
            let value = attribute
                .normalized_value(XmlVersion::Explicit1_0)
                .map_err(xml_error)?
                .into_owned();
            validate_xml_chars(&value)?;
            Ok((key, value))
        })
        .collect()
}

fn required<'a>(attributes: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str> {
    attributes
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| Error(format!("SVG requires {name}")))
}

fn check_attributes(attributes: &BTreeMap<String, String>, allowed: &[&str]) -> Result<()> {
    for name in attributes.keys() {
        ensure(
            allowed.contains(&name.as_str()),
            &format!(
                "Unsupported SVG attribute: {name}; transforms, CSS and external resources are not imported"
            ),
        )?;
    }
    Ok(())
}

fn root_scale(attributes: &BTreeMap<String, String>) -> Result<f64> {
    check_attributes(
        attributes,
        &["xmlns", "width", "height", "viewBox", "version"],
    )?;
    ensure(
        required(attributes, "xmlns")? == "http://www.w3.org/2000/svg",
        "SVG requires the standard default SVG namespace",
    )?;
    ensure(
        attributes
            .get("version")
            .is_none_or(|value| matches!(value.as_str(), "1.1" | "2.0")),
        "Unsupported SVG version",
    )?;
    let millimetres = |name| -> Result<f64> {
        let value = required(attributes, name)?.trim();
        let number = value.strip_suffix("mm").ok_or_else(|| {
            Error(format!(
                "SVG {name} must declare physical mm units; no DPI is inferred"
            ))
        })?;
        ensure(
            !number
                .as_bytes()
                .last()
                .is_some_and(|&byte| whitespace(byte)),
            "SVG mm units must immediately follow their number",
        )?;
        let number = single_number(number)?;
        ensure(number > 0.0, "SVG physical dimensions must be positive")?;
        Ok(number)
    };
    let width = millimetres("width")?;
    let height = millimetres("height")?;
    let mut numbers = Numbers::new(required(attributes, "viewBox")?);
    let mut view_box = [0.0; 4];
    for (index, value) in view_box.iter_mut().enumerate() {
        if index > 0 {
            let start = numbers.offset;
            numbers.skip_whitespace();
            ensure(
                numbers.offset > start || numbers.peek() == Some(b','),
                "SVG viewBox numbers need whitespace or comma separators",
            )?;
        }
        *value = numbers.number()?;
    }
    numbers.finish()?;
    ensure(
        view_box[2] > 0.0 && view_box[3] > 0.0,
        "SVG viewBox dimensions must be positive",
    )?;
    let [x, y] = [width / view_box[2], height / view_box[3]];
    ensure(
        x.is_finite()
            && y.is_finite()
            && x > 0.0
            && y > 0.0
            && (x - y).abs() <= x.abs().max(y.abs()) * 1e-12,
        "SVG width/height and viewBox must define a finite uniform physical scale",
    )?;
    let scale = x / 1000.0;
    ensure(
        scale.is_finite() && scale > 0.0,
        "SVG physical scale is outside supported precision",
    )?;
    Ok(scale)
}

fn parse_path(name: String, data: &str, scale: f64, evenodd: bool) -> Result<ImportedPattern> {
    let mut numbers = Numbers::new(data);
    let first = contour(&mut numbers, scale)?;
    let mut total = first.vertices.len();
    let mut rings = vec![first];
    numbers.skip_whitespace();
    while numbers.peek() == Some(b'M') {
        let ring = contour(&mut numbers, scale)?;
        total += ring.vertices.len();
        ensure(total <= MAX_SVG_SPANS, "SVG exceeds the 4096-span limit")?;
        rings.push(ring);
        numbers.skip_whitespace();
    }
    numbers.finish()?;
    // A single containing exterior has the largest absolute area, regardless
    // of SVG subpath order. The document checks containment and separation.
    let outer_index = rings
        .iter()
        .enumerate()
        .map(|(index, ring)| (index, signed_double_area(&ring.vertices).abs()))
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("the first contour was parsed")
        .0;
    let exterior = rings.remove(outer_index);
    if !rings.is_empty() {
        ensure(
            std::iter::once(&exterior)
                .chain(&rings)
                .flat_map(|r| &r.controls)
                .all(Option::is_none),
            "SVG cutouts require straight sides on both the exterior and the hole",
        )?;
        if !evenodd {
            let winding = signed_double_area(&exterior.vertices).signum();
            ensure(
                rings
                    .iter()
                    .all(|ring| signed_double_area(&ring.vertices).signum() != winding),
                "A nonzero SVG cutout needs oppositely wound contours",
            )?;
        }
    }
    Ok(ImportedPattern {
        name,
        exterior,
        holes: rings.into_iter().map(|ring| ring.vertices).collect(),
    })
}

fn signed_double_area(vertices: &[[f64; 2]]) -> f64 {
    let [x, y] = vertices[0];
    vertices[1..]
        .windows(2)
        .map(|pair| {
            let [ax, ay] = pair[0];
            let [bx, by] = pair[1];
            (ax - x) * (by - y) - (ay - y) * (bx - x)
        })
        .sum()
}

fn contour(numbers: &mut Numbers<'_>, scale: f64) -> Result<ImportedContour> {
    ensure(
        numbers.command() == Some(b'M'),
        "SVG path must begin with one absolute M moveto",
    )?;
    let mut vertices = vec![numbers.point(scale)?];
    let mut controls = Vec::new();
    let mut command = b'L';
    loop {
        if let Some(next) = numbers.command() {
            if next == b'Z' {
                if vertices.last() == vertices.first() {
                    vertices.pop();
                } else {
                    controls.push(None);
                }
                ensure(
                    (3..=MAX_SVG_SPANS).contains(&vertices.len()),
                    "SVG boundary needs 3 to 4096 spans",
                )?;
                ensure(
                    controls.len() == vertices.len(),
                    "SVG path has an invalid closed boundary",
                )?;
                return Ok(ImportedContour { vertices, controls });
            }
            ensure(
                matches!(next, b'L' | b'C'),
                "SVG contours require absolute M/L/C/Z commands",
            )?;
            command = next;
        }
        let control = if command == b'C' {
            Some([numbers.point(scale)?, numbers.point(scale)?])
        } else {
            None
        };
        let point = numbers.point(scale)?;
        ensure(
            vertices.last() != Some(&point),
            "SVG boundary has a repeated or degenerate endpoint",
        )?;
        vertices.push(point);
        controls.push(control);
        ensure(
            controls.len() <= MAX_SVG_SPANS,
            "SVG exceeds the 4096-span limit",
        )?;
    }
}

fn whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

struct Numbers<'a> {
    source: &'a str,
    offset: usize,
    comma_allowed: bool,
}

impl<'a> Numbers<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: 0,
            comma_allowed: false,
        }
    }
    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(whitespace) {
            self.offset += 1;
        }
    }
    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset).copied()
    }
    fn command(&mut self) -> Option<u8> {
        self.skip_whitespace();
        let command = self.peek().filter(u8::is_ascii_alphabetic)?;
        self.offset += 1;
        self.comma_allowed = false;
        Some(command)
    }
    fn number(&mut self) -> Result<f64> {
        self.skip_whitespace();
        if self.comma_allowed && self.peek() == Some(b',') {
            self.offset += 1;
            self.skip_whitespace();
        }
        let start = self.offset;
        if matches!(self.peek(), Some(b'+' | b'-')) {
            self.offset += 1;
        }
        let digits = self.offset;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.offset += 1;
        }
        let mut has_digits = self.offset > digits;
        if self.peek() == Some(b'.') {
            self.offset += 1;
            let digits = self.offset;
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.offset += 1;
            }
            has_digits |= self.offset > digits;
        }
        ensure(
            has_digits,
            "SVG path requires finite numbers and an explicit final Z",
        )?;
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.offset += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.offset += 1;
            }
            let digits = self.offset;
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.offset += 1;
            }
            ensure(self.offset > digits, "SVG number has an invalid exponent")?;
        }
        let value: f64 = self.source[start..self.offset].parse().map_err(xml_error)?;
        ensure(value.is_finite(), "SVG numbers must be finite")?;
        self.comma_allowed = true;
        Ok(value)
    }
    fn point(&mut self, scale: f64) -> Result<[f64; 2]> {
        let point = [self.number()? * scale, -self.number()? * scale];
        crate::document::validate_coordinates(&point)?;
        Ok(point)
    }
    fn finish(&mut self) -> Result<()> {
        self.skip_whitespace();
        ensure(
            self.offset == self.source.len(),
            "SVG contains trailing or unsupported path/number data",
        )
    }
}

fn single_number(value: &str) -> Result<f64> {
    let mut numbers = Numbers::new(value);
    let number = numbers.number()?;
    numbers.finish()?;
    Ok(number)
}
