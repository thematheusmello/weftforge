use super::{StagedExport, export_path_guard};
use crate::{Id, Piece, Project, Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Display, Write as _};
use std::path::Path;

const MM: f64 = 72.0 / 25.4;
const INSET: [f64; 2] = [10.0 * MM, 20.0 * MM];
const OVERLAP: f64 = 10.0 * MM;
const INK_MARGIN_M: f64 = 0.001;
const MAX_PAGES: usize = 4096;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PrintPaper {
    A4,
    Letter,
}

impl PrintPaper {
    fn size(self) -> [f64; 2] {
        match self {
            Self::A4 => [210.0, 297.0],
            Self::Letter => [215.9, 279.4],
        }
        .map(|mm| mm * MM)
    }
}

impl Project {
    /// Exports editable 2D boundaries and marks at 1:1 scale on overlapping sheets.
    pub fn export_pattern_pdf(&self, path: impl AsRef<Path>, paper: PrintPaper) -> Result<()> {
        let path = path.as_ref();
        export_path_guard(path)?;
        StagedExport::write(path, &render(self, paper)?)?.commit()
    }
}

fn render(project: &Project, paper: PrintPaper) -> Result<Vec<u8>> {
    project.validate()?;
    ensure(
        !project.pieces.is_empty(),
        "Create a pattern before printing",
    )?;
    let notches = project.notch_segments()?;
    let size = paper.size();
    let area = std::array::from_fn(|axis| size[axis] - 2.0 * INSET[axis]);
    let step = area.map(|dimension| dimension - OVERLAP);
    let mut layouts = Vec::new();
    let mut page_count = 1;
    for piece in &project.pieces {
        let [mut min, mut max] = project.piece_bounds(piece.id)?;
        for point in piece.notches.iter().flat_map(|notch| notches[&notch.id]) {
            for axis in 0..2 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
        let extent =
            std::array::from_fn(|axis| (max[axis] - min[axis] + 2.0 * INK_MARGIN_M) * 1000.0 * MM);
        let columns_rows =
            std::array::from_fn(|axis| tile_count(extent[axis], area[axis], step[axis]));
        let [columns, rows] = columns_rows;
        page_count += columns * rows;
        ensure(page_count <= MAX_PAGES, "Pattern PDF exceeds 4096 pages")?;
        let origin = [min[0] - INK_MARGIN_M, max[1] + INK_MARGIN_M];
        layouts.push((piece, origin, extent, columns_rows));
    }

    let mut pdf = Pdf::new();
    let mut pages = Vec::with_capacity(page_count);
    pages.push(pdf.page(size, &calibration(size), None));
    let mut bookmarks = Vec::new();
    for (piece, origin, [width, height], [columns, rows]) in layouts {
        let art = paths(project, piece, origin, &notches);
        let form = pdf.add_stream(
            &format!(
                "/Type /XObject /Subtype /Form /BBox [0 {:.6} {width:.6} 0] /Resources << >>",
                -height
            ),
            &art,
        );
        let first_page = pages.len();
        for row in 0..rows {
            for column in 0..columns {
                let clip_x = INSET[0];
                let clip_y = INSET[1];
                let top = size[1] - clip_y;
                let x = clip_x - column as f64 * step[0];
                let y = top + row as f64 * step[1];
                let [art_width, art_height] = area;
                let mut sheet = format!(
                    "q {clip_x:.6} {clip_y:.6} {art_width:.6} {art_height:.6} re W n 1 0 0 1 {x:.6} {y:.6} cm /Art Do Q\n"
                );
                let title = format!(
                    "Pattern #{}   Row {}/{}   Column {}/{}",
                    piece.id,
                    row + 1,
                    rows,
                    column + 1,
                    columns
                );
                label(&mut sheet, [clip_x, size[1] - 10.0 * MM], &title);
                label(
                    &mut sheet,
                    [clip_x, 10.0 * MM],
                    "Actual size (100%)   |   Align matching crosses across the 10 mm overlap",
                );
                registration(&mut sheet, size, [column, row], [columns, rows]);
                pages.push(pdf.page(size, &sheet, Some(form)));
            }
        }
        bookmarks.push((piece.name.as_str(), pages[first_page]));
    }
    pdf.finish(&pages, &bookmarks)
}

fn tile_count(extent: f64, area: f64, step: f64) -> usize {
    // Round away floating-point noise below the precision used for PDF coordinates.
    1 + ((extent - area - 1e-7).max(0.0) / step).ceil() as usize
}

fn paths(
    project: &Project,
    piece: &Piece,
    origin: [f64; 2],
    notches: &BTreeMap<Id, [[f64; 2]; 2]>,
) -> String {
    let local =
        |point: [f64; 2]| std::array::from_fn(|axis| (point[axis] - origin[axis]) * 1000.0 * MM);
    let mut out = format!("{:.6} w 1 j 1 J\n", 0.25 * MM);
    let point = |id| project.point(id).expect("Validated point").position;
    let curve = |id| project.curve(id).expect("Validated curve");
    for boundary in piece.boundary_loops() {
        coordinates(&mut out, local(point(curve(boundary[0]).start)), "m");
        for &id in boundary {
            let edge = curve(id);
            let [x, y] = local(point(edge.end));
            if let Some([a, b]) = edge.controls {
                let [ax, ay] = local(a);
                let [bx, by] = local(b);
                writeln!(out, "{ax:.6} {ay:.6} {bx:.6} {by:.6} {x:.6} {y:.6} c").unwrap();
            } else {
                coordinates(&mut out, [x, y], "l");
            }
        }
        out.push_str("h S\n");
    }
    for notch in &piece.notches {
        let [a, b] = notches[&notch.id].map(local);
        coordinates(&mut out, a, "m");
        coordinates(&mut out, b, "l S");
    }
    out.push_str("[4 3] 0 d\n");
    for &id in &piece.internal_curves {
        let edge = curve(id);
        coordinates(&mut out, local(point(edge.start)), "m");
        coordinates(&mut out, local(point(edge.end)), "l S");
    }
    out
}

fn calibration([width, height]: [f64; 2]) -> String {
    let edge = 100.0 * MM;
    let x = (width - edge) / 2.0;
    let y = (height - edge) / 2.0;
    let mut content = format!("0.5 w {x:.6} {y:.6} {edge:.6} {edge:.6} re S\n");
    label(&mut content, [x, y + edge + 5.0 * MM], "100 mm x 100 mm");
    let mut y = height - INSET[1];
    for instruction in [
        "WeftForge pattern printing",
        "Print this page at Actual size (100%). Turn off Fit to page.",
        "Measure the square. Each side should be 100 mm.",
        "Print the pattern sheets using the same paper and print settings.",
        "Align matching crosses across overlapping rows and columns.",
    ] {
        label(&mut content, [INSET[0], y], instruction);
        y -= 6.0 * MM;
    }
    content
}

fn registration(
    content: &mut String,
    size: [f64; 2],
    [col, row]: [usize; 2],
    [cols, rows]: [usize; 2],
) {
    let [left, bottom] = INSET;
    let right = size[0] - left;
    let top = size[1] - bottom;
    let offset = OVERLAP / 2.0;
    let gutter = 3.0 * MM;
    let arm = 1.5 * MM;
    content.push_str("0.5 w\n");
    let mut cross = |x, y| {
        coordinates(content, [x - arm, y], "m");
        coordinates(content, [x + arm, y], "l");
        coordinates(content, [x, y - arm], "m");
        coordinates(content, [x, y + arm], "l S");
    };
    for (exists, x) in [(col > 0, left + offset), (col + 1 < cols, right - offset)] {
        if exists {
            cross(x, top + gutter);
            cross(x, bottom - gutter);
        }
    }
    for (exists, y) in [(row > 0, top - offset), (row + 1 < rows, bottom + offset)] {
        if exists {
            cross(left - gutter, y);
            cross(right + gutter, y);
        }
    }
}

fn coordinates(out: &mut String, [x, y]: [f64; 2], operation: &str) {
    writeln!(out, "{x:.6} {y:.6} {operation}").unwrap();
}

fn label(out: &mut String, [x, y]: [f64; 2], text: &str) {
    write!(out, "BT /F1 9 Tf {x:.6} {y:.6} Td <").unwrap();
    for byte in text.bytes() {
        write!(out, "{byte:02X}").unwrap();
    }
    out.push_str("> Tj ET\n");
}

struct Pdf {
    body: String,
    offsets: Vec<usize>,
}

impl Pdf {
    fn new() -> Self {
        Self {
            body: "%PDF-1.7\n".into(),
            offsets: vec![0; 5],
        }
    }

    fn set(&mut self, id: usize, value: impl Display) {
        self.offsets[id] = self.body.len();
        writeln!(self.body, "{id} 0 obj\n{value}\nendobj").unwrap();
    }

    fn add(&mut self, value: impl Display) -> usize {
        let id = self.offsets.len();
        self.offsets.push(0);
        self.set(id, value);
        id
    }

    fn add_stream(&mut self, entries: &str, content: &str) -> usize {
        self.add(format_args!(
            "<< {entries} /Length {} >>\nstream\n{content}endstream",
            content.len()
        ))
    }

    fn page(&mut self, [width, height]: [f64; 2], content: &str, form: Option<usize>) -> usize {
        let stream = self.add_stream("", content);
        let xobject = form.map_or_else(String::new, |id| format!("/XObject << /Art {id} 0 R >>"));
        self.add(format_args!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width:.6} {height:.6}] /Contents {stream} 0 R /Resources << /Font << /F1 3 0 R >> {xobject} >> >>"
        ))
    }

    fn finish(mut self, pages: &[usize], bookmarks: &[(&str, usize)]) -> Result<Vec<u8>> {
        let first = self.offsets.len();
        let last = first + bookmarks.len() - 1;
        for (name, page) in bookmarks {
            let id = self.offsets.len();
            let mut entry = String::from("<< /Title <FEFF");
            for unit in name.encode_utf16() {
                write!(entry, "{unit:04X}").unwrap();
            }
            write!(entry, "> /Parent 4 0 R /Dest [{page} 0 R /Fit]").unwrap();
            if id > first {
                write!(entry, " /Prev {} 0 R", id - 1).unwrap();
            }
            if id < last {
                write!(entry, " /Next {} 0 R", id + 1).unwrap();
            }
            entry.push_str(" >>");
            self.add(entry);
        }
        self.set(1, "<< /Type /Catalog /Pages 2 0 R /Outlines 4 0 R /ViewerPreferences << /PrintScaling /None >> >>");
        let mut tree = format!("<< /Type /Pages /Count {} /Kids [", pages.len());
        for id in pages {
            write!(tree, "{id} 0 R ").unwrap();
        }
        tree.push_str("] >>");
        self.set(2, tree);
        self.set(3, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>");
        self.set(
            4,
            format_args!(
                "<< /Type /Outlines /First {first} 0 R /Last {last} 0 R /Count {} >>",
                bookmarks.len()
            ),
        );
        let xref = self.body.len();
        ensure(
            (xref as u64) < 10_000_000_000,
            "Pattern PDF exceeds its file size limit",
        )?;
        let count = self.offsets.len();
        writeln!(self.body, "xref\n0 {count}\n0000000000 65535 f ").unwrap();
        for offset in self.offsets.into_iter().skip(1) {
            writeln!(self.body, "{offset:010} 00000 n ").unwrap();
        }
        writeln!(
            self.body,
            "trailer\n<< /Size {count} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF"
        )
        .unwrap();
        Ok(self.body.into_bytes())
    }
}
