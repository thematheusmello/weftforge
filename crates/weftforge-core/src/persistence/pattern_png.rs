use super::{StagedExport, export_path_guard};
use crate::{Error, Project, Result, ensure};
use image::ExtendedColorType;
use image::codecs::jpeg::{JpegEncoder, PixelDensity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;
use tiny_skia::{IntSize, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

const METRES_PER_INCH: f64 = 0.0254;
const MAX_SIDE: u32 = 16_384;
const MAX_PIXELS: u64 = 32_000_000;
const MAX_RASTER_BYTES: usize = 80 * 1024 * 1024;
const MAX_STROKE_WORK: u64 = 64_000_000;
const SKIA_TILE_SIDE: u32 = 8191;
const MAX_PROJECTED_COORDINATE: f64 = 1_048_576.0;
const CLIP_MARGIN: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PatternSnapshotOptions {
    /// Minimum and maximum XY in material space, in metres.
    pub crop_m: [[f64; 2]; 2],
    pub pixels_per_inch: f64,
    pub contours: bool,
    pub internal_lines: bool,
    pub notches: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub grainline: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl PatternSnapshotOptions {
    /// Rounded pixel width and height. Raster coordinates use the declared PNG density.
    pub fn dimensions(self) -> Result<[u32; 2]> {
        self.grid().map(|(dimensions, _)| dimensions)
    }

    /// Square-pixel density stored in the PNG pHYs chunk.
    pub fn pixels_per_metre(self) -> Result<u32> {
        let ppm = (self.pixels_per_inch / METRES_PER_INCH).round();
        ensure(
            (1.0..=f64::from(u32::MAX)).contains(&ppm),
            "Pattern PNG density must round to between 1 and 4294967295 pixels per metre",
        )?;
        Ok(ppm as u32)
    }

    /// Whole square-pixel density recorded in the JPEG JFIF header.
    pub fn jpeg_pixels_per_inch(self) -> Result<u16> {
        let dpi = self.pixels_per_inch.round();
        ensure(
            (1.0..=f64::from(u16::MAX)).contains(&dpi),
            "Pattern JPEG density must round to between 1 and 65535 pixels per inch",
        )?;
        Ok(dpi as u16)
    }

    /// Rounded pixel width and height at the JPEG's stored whole PPI.
    pub fn jpeg_dimensions(self) -> Result<[u32; 2]> {
        self.jpeg_grid().map(|(dimensions, _, _)| dimensions)
    }

    fn crop_extent(self, format_name: &str) -> Result<[f64; 2]> {
        let [min, max] = self.crop_m;
        if !min.into_iter().chain(max).all(f64::is_finite)
            || (0..2).any(|axis| min[axis] >= max[axis])
        {
            return Err(Error(format!(
                "Pattern {format_name} crop needs finite, increasing X and Y bounds"
            )));
        }
        Ok(std::array::from_fn(|axis| max[axis] - min[axis]))
    }

    fn grid(self) -> Result<([u32; 2], u32)> {
        let extent = self.crop_extent("PNG")?;
        let ppm = self.pixels_per_metre()?;
        Ok((raster_dimensions(extent, f64::from(ppm), "PNG")?, ppm))
    }

    fn jpeg_grid(self) -> Result<([u32; 2], u16, f64)> {
        let extent = self.crop_extent("JPEG")?;
        let dpi = self.jpeg_pixels_per_inch()?;
        let ppm = f64::from(dpi) / METRES_PER_INCH;
        Ok((raster_dimensions(extent, ppm, "JPEG")?, dpi, ppm))
    }
}

fn raster_dimensions(extent: [f64; 2], ppm: f64, format_name: &str) -> Result<[u32; 2]> {
    let size = extent.map(|side| (side * ppm).round());
    if !size
        .iter()
        .all(|side| (1.0..=f64::from(MAX_SIDE)).contains(side))
    {
        return Err(Error(format!(
            "Pattern {format_name} width and height must be between 1 and 16384 pixels"
        )));
    }
    let size = size.map(|side| side as u32);
    if u64::from(size[0]) * u64::from(size[1]) > MAX_PIXELS {
        return Err(Error(format!(
            "Pattern {format_name} exceeds the 32 megapixel limit"
        )));
    }
    Ok(size)
}

impl Project {
    /// Bounds of authored pattern art, including the endpoints of notch marks.
    pub fn pattern_snapshot_bounds(&self) -> Result<[[f64; 2]; 2]> {
        self.validate()?;
        ensure(
            !self.pieces.is_empty(),
            "Create a pattern to set snapshot bounds",
        )?;
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        let mut include = |point: [f64; 2]| {
            for axis in 0..2 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        };
        for piece in &self.pieces {
            for point in self.piece_bounds(piece.id)? {
                include(point);
            }
        }
        for point in self.notch_segments()?.into_values().flatten() {
            include(point);
        }
        Ok([min, max])
    }

    /// Prepares a physical-scale raster of editable 2D pattern art for commit.
    pub fn stage_pattern_png(
        &self,
        path: impl AsRef<Path>,
        options: PatternSnapshotOptions,
    ) -> Result<StagedExport> {
        let path = path.as_ref();
        let (size, ppm) = options.grid()?;
        export_path_guard(path)?;
        self.validate()?;
        let pixmap = rasterize(self, options, size, f64::from(ppm), "PNG")?;
        StagedExport::write(path, &encode_png(&pixmap, ppm)?)
    }

    /// Exports a physical-scale PNG without changing the editable project.
    pub fn export_pattern_png(
        &self,
        path: impl AsRef<Path>,
        options: PatternSnapshotOptions,
    ) -> Result<()> {
        self.stage_pattern_png(path, options)?.commit()
    }

    /// Prepares a physical-scale grayscale JPEG for commit.
    pub fn stage_pattern_jpeg(
        &self,
        path: impl AsRef<Path>,
        options: PatternSnapshotOptions,
    ) -> Result<StagedExport> {
        let path = path.as_ref();
        let (size, dpi, ppm) = options.jpeg_grid()?;
        export_path_guard(path)?;
        self.validate()?;
        let pixmap = rasterize(self, options, size, ppm, "JPEG")?;
        StagedExport::write(path, &encode_jpeg(pixmap, dpi)?)
    }

    /// Exports a physical-scale JPEG without changing the editable project.
    pub fn export_pattern_jpeg(
        &self,
        path: impl AsRef<Path>,
        options: PatternSnapshotOptions,
    ) -> Result<()> {
        self.stage_pattern_jpeg(path, options)?.commit()
    }
}

struct Strokes {
    format_name: &'static str,
    origin_m: [f64; 2],
    size: [u32; 2],
    ppm: f64,
    tiles: u64,
    work: u64,
    path: PathBuilder,
}

impl Strokes {
    fn add_grainline(
        &mut self,
        [[left, bottom], [right, top]]: [[f64; 2]; 2],
        angle: f64,
    ) -> Result<()> {
        let center = [(left + right) * 0.5, (bottom + top) * 0.5];
        let length = ((right - left).min(top - bottom) * 0.25).min(0.1);
        let (sin, cos) = angle.sin_cos();
        let tip = [center[0] + length * cos, center[1] + length * sin];
        self.add([center, tip], None)?;
        // Match the studio's arrowhead in material space before the raster Y flip.
        let (wing_sin, wing_cos) = (std::f64::consts::TAU / 10.).sin_cos();
        let back = length * 0.25 * wing_cos;
        let side = length * 0.25 * wing_sin;
        for offset in [-side, side] {
            let end = [
                tip[0] - back * cos - offset * sin,
                tip[1] - back * sin + offset * cos,
            ];
            self.add([tip, end], None)?;
        }
        Ok(())
    }

    fn add(&mut self, [a, b]: [[f64; 2]; 2], controls: Option<[[f64; 2]; 2]>) -> Result<()> {
        let [c, d] = controls.unwrap_or([a, b]);
        let points = [a, c, d, b].map(|[x, y]| {
            [
                (x - self.origin_m[0]) * self.ppm,
                (self.origin_m[1] - y) * self.ppm,
            ]
        });
        if (0..2).any(|axis| {
            points.iter().all(|p| p[axis] < -CLIP_MARGIN)
                || points
                    .iter()
                    .all(|p| p[axis] > f64::from(self.size[axis]) + CLIP_MARGIN)
        }) {
            return Ok(());
        }
        // Bound the f64-to-f32 conversion error to 1/16 px, including off-canvas controls.
        if !points
            .iter()
            .flatten()
            .all(|value| value.abs() <= MAX_PROJECTED_COORDINATE)
        {
            return Err(Error(format!(
                "Pattern {} crop and density exceed raster coordinate precision",
                self.format_name
            )));
        }
        // Bound pixel visits by control polygon length, with overhead for tiny-skia subdivisions.
        let length: u64 = points
            .windows(2)
            .map(|pair| {
                ((pair[0][0] - pair[1][0]).abs() + (pair[0][1] - pair[1][1]).abs()).ceil() as u64
            })
            .sum();
        let overhead = if controls.is_some() { 4096 } else { 8 };
        self.work += (length + overhead) * self.tiles;
        if self.work > MAX_STROKE_WORK {
            return Err(Error(format!(
                "Pattern {} exceeds the raster stroke work limit",
                self.format_name
            )));
        }
        let [[x0, y0], [x1, y1], [x2, y2], [x3, y3]] =
            points.map(|point| point.map(|value| value as f32));
        self.path.move_to(x0, y0);
        if controls.is_some() {
            self.path.cubic_to(x1, y1, x2, y2, x3, y3);
        } else {
            self.path.line_to(x3, y3);
        }
        Ok(())
    }
}

fn rasterize(
    project: &Project,
    options: PatternSnapshotOptions,
    [width, height]: [u32; 2],
    ppm: f64,
    format_name: &'static str,
) -> Result<Pixmap> {
    let [[left, _], [_, top]] = options.crop_m;
    let mut strokes = Strokes {
        format_name,
        origin_m: [left, top],
        size: [width, height],
        ppm,
        tiles: u64::from(width.div_ceil(SKIA_TILE_SIDE) * height.div_ceil(SKIA_TILE_SIDE)),
        work: 0,
        path: PathBuilder::new(),
    };
    if options.contours || options.internal_lines {
        let points: BTreeMap<_, _> = project
            .points
            .iter()
            .map(|point| (point.id, point.position))
            .collect();
        let curves: BTreeMap<_, _> = project
            .curves
            .iter()
            .map(|curve| (curve.id, curve))
            .collect();
        for piece in &project.pieces {
            let contour = piece
                .boundary_loops()
                .flatten()
                .filter(|_| options.contours);
            let internal = piece
                .internal_curves
                .iter()
                .filter(|_| options.internal_lines);
            for id in contour.chain(internal) {
                let edge = curves[id];
                strokes.add([points[&edge.start], points[&edge.end]], edge.controls)?;
            }
        }
    }
    if options.notches {
        for segment in project.notch_segments()?.into_values() {
            strokes.add(segment, None)?;
        }
    }
    if options.grainline {
        for piece in &project.pieces {
            strokes.add_grainline(project.piece_bounds(piece.id)?, piece.grain_angle)?;
        }
    }
    let bytes = width as usize * height as usize * 4;
    let mut pixels = Vec::new();
    pixels.try_reserve_exact(bytes).map_err(|error| {
        Error(format!(
            "Could not allocate pattern {format_name} pixels: {error}"
        ))
    })?;
    pixels.resize(bytes, 255);
    let size =
        IntSize::from_wh(width, height).ok_or_else(|| Error("Invalid raster size".into()))?;
    let mut pixmap = Pixmap::from_vec(pixels, size)
        .ok_or_else(|| Error(format!("Invalid pattern {format_name} buffer")))?;
    if let Some(path) = strokes.path.finish() {
        let stroke = Stroke {
            width: 1.0,
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        pixmap.stroke_path(
            &path,
            &Paint::default(),
            &stroke,
            Transform::identity(),
            None,
        );
    }
    Ok(pixmap)
}

struct RasterBuffer {
    format_name: &'static str,
    bytes: Vec<u8>,
}

impl RasterBuffer {
    fn new(format_name: &'static str) -> Self {
        Self {
            format_name,
            bytes: Vec::new(),
        }
    }
}

impl Write for RasterBuffer {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let length = self.bytes.len() + data.len();
        if length > MAX_RASTER_BYTES {
            return Err(io::Error::other(format!(
                "Pattern {} exceeds the 80 MiB file limit",
                self.format_name
            )));
        }
        if length > self.bytes.capacity() {
            let capacity = length.max(self.bytes.capacity() * 2).min(MAX_RASTER_BYTES);
            self.bytes
                .try_reserve_exact(capacity - self.bytes.len())
                .map_err(io::Error::other)?;
        }
        self.bytes.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn encode_png(pixmap: &Pixmap, ppm: u32) -> Result<Vec<u8>> {
    let png_error = |error| Error(format!("Could not encode pattern PNG: {error}"));
    let mut bytes = RasterBuffer::new("PNG");
    let mut encoder = png::Encoder::new(&mut bytes, pixmap.width(), pixmap.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    encoder.set_pixel_dims(Some(png::PixelDimensions {
        xppu: ppm,
        yppu: ppm,
        unit: png::Unit::Meter,
    }));
    let mut writer = encoder.write_header().map_err(png_error)?;
    let mut stream = writer.stream_writer().map_err(png_error)?;
    // The pixmap and ink are opaque, so premultiplied RGBA is already straight RGBA.
    stream.write_all(pixmap.data())?;
    stream.finish().map_err(png_error)?;
    writer.finish().map_err(png_error)?;
    Ok(bytes.bytes)
}

fn encode_jpeg(pixmap: Pixmap, dpi: u16) -> Result<Vec<u8>> {
    let (width, height) = (pixmap.width(), pixmap.height());
    // The opaque raster is neutral grayscale, so compact one color channel in place.
    let mut luma = pixmap.take();
    let samples = luma.len() / 4;
    for (destination, source) in (0..luma.len()).step_by(4).enumerate() {
        luma[destination] = luma[source];
    }
    luma.truncate(samples);
    let mut bytes = RasterBuffer::new("JPEG");
    {
        let mut encoder = JpegEncoder::new_with_quality(&mut bytes, 100);
        encoder.set_pixel_density(PixelDensity::dpi(dpi));
        encoder
            .encode(&luma, width, height, ExtendedColorType::L8)
            .map_err(|error| Error(format!("Could not encode pattern JPEG: {error}")))?;
    }
    Ok(bytes.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use std::fs;
    use std::io::Cursor;
    use std::sync::atomic::Ordering;

    #[test]
    fn pattern_jpeg_uses_the_declared_jfif_grid_and_stages_before_replacing() {
        let mut editor = crate::Editor::default();
        editor
            .execute(Command::AddRectangle {
                name: "Original JPEG sample".into(),
                origin: [-0.15, 0.03],
                width: 0.2,
                height: 0.1,
            })
            .unwrap();
        let project = editor.project;
        let source = project.to_json().unwrap();
        let options = PatternSnapshotOptions {
            crop_m: [[-1., 0.], [1., 0.2]],
            pixels_per_inch: 72.,
            contours: true,
            internal_lines: false,
            notches: false,
            grainline: false,
        };
        // A two-metre side spans 5669.291... pixels at 72 PPI and 5670 on the PNG grid.
        assert_eq!(options.dimensions().unwrap(), [5670, 567]);
        assert_eq!(options.jpeg_dimensions().unwrap(), [5669, 567]);
        let fractional = PatternSnapshotOptions {
            pixels_per_inch: 72.4,
            ..options
        };
        assert_eq!(fractional.jpeg_pixels_per_inch().unwrap(), 72);
        assert_eq!(fractional.jpeg_dimensions().unwrap(), [5669, 567]);

        let dir = std::env::temp_dir().join(format!(
            "weftforge-pattern-jpeg-{}-{}",
            std::process::id(),
            super::super::TEMP_ID.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&dir).unwrap();
        let output = dir.join("pattern.jpg");
        fs::write(&output, b"previous JPEG").unwrap();
        for pixels_per_inch in [0.49, 65_535.5, f64::NAN] {
            let invalid = PatternSnapshotOptions {
                pixels_per_inch,
                ..options
            };
            assert!(project.export_pattern_jpeg(&output, invalid).is_err());
            assert_eq!(fs::read(&output).unwrap(), b"previous JPEG");
        }
        let staged = project.stage_pattern_jpeg(&output, options).unwrap();
        assert_eq!(fs::read(&output).unwrap(), b"previous JPEG");
        staged.commit().unwrap();

        let jpeg = fs::read(&output).unwrap();
        assert!(jpeg.starts_with(&[0xff, 0xd8]) && jpeg.ends_with(&[0xff, 0xd9]));
        let be16 =
            |bytes: &[u8], pos: usize| u16::from_be_bytes(bytes[pos..pos + 2].try_into().unwrap());
        let segment = |at: usize| {
            assert_eq!(jpeg[at], 0xff);
            let end = at + 2 + usize::from(be16(&jpeg, at + 2));
            (jpeg[at + 1], &jpeg[at + 4..end], end)
        };
        let (marker, jfif, mut at) = segment(2);
        assert_eq!(marker, 0xe0);
        assert_eq!(&jfif[..8], b"JFIF\0\x01\x02\x01");
        assert_eq!([be16(jfif, 8), be16(jfif, 10)], [72, 72]);
        let frame = loop {
            let (marker, data, next) = segment(at);
            if marker == 0xc0 {
                break data;
            }
            assert_ne!(marker, 0xda, "SOF0 must precede the scan");
            at = next;
        };
        assert_eq!([frame[0], frame[5]], [8, 1], "8-bit grayscale");
        assert_eq!([be16(frame, 3), be16(frame, 1)], [5669, 567]);
        assert_eq!(project.to_json().unwrap(), source);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn pattern_png_grainline_rotates_with_authored_angle_and_is_opt_in() {
        let mut editor = crate::Editor::default();
        editor
            .execute(Command::AddRectangle {
                name: "Original grain sample".into(),
                origin: [-0.03, -0.02],
                width: 0.12,
                height: 0.08,
            })
            .unwrap();
        let legacy = br#"{"crop_m":[[-0.05,-0.04],[0.12,0.095]],"pixels_per_inch":25.4,"contours":false,"internal_lines":false,"notches":false}"#;
        let omitted: PatternSnapshotOptions = serde_json::from_slice(legacy).unwrap();
        assert_eq!(serde_json::to_vec(&omitted).unwrap(), legacy);
        let mut enabled_options = serde_json::to_value(omitted).unwrap();
        enabled_options["grainline"] = serde_json::Value::Bool(true);
        let enabled: PatternSnapshotOptions = serde_json::from_value(enabled_options).unwrap();
        assert!(enabled.grainline);
        let draw = |project: &Project, options: PatternSnapshotOptions| {
            let (size, ppm) = options.grid().unwrap();
            assert_eq!((size, ppm), ([170, 135], 1000));
            rasterize(project, options, size, f64::from(ppm), "PNG").unwrap()
        };
        let ink = |image: &Pixmap, x, y| image.pixel(x, y).unwrap().red() < 255;
        let blank = draw(&editor.project, omitted);
        assert!(blank.data().iter().all(|sample| *sample == 255));
        let unrotated = draw(&editor.project, enabled);
        assert!(ink(&unrotated, 90, 75));
        assert!(!ink(&unrotated, 80, 65));

        editor
            .execute(Command::SetGrain {
                piece: editor.project.pieces[0].id,
                angle: std::f64::consts::FRAC_PI_2,
            })
            .unwrap();
        let rotated = draw(&editor.project, enabled);
        assert!(
            ink(&rotated, 80, 65),
            "rotated shaft points up from the bounds centre"
        );
        assert!(
            ink(&rotated, 77, 58) && ink(&rotated, 82, 58),
            "both arrowhead wings"
        );
        assert!(!ink(&rotated, 90, 75) && !ink(&rotated, 80, 85));
        assert_eq!(
            encode_png(&draw(&editor.project, omitted), 1000).unwrap(),
            encode_png(&blank, 1000).unwrap()
        );
    }

    #[test]
    fn pattern_png_uses_declared_scale_and_crops_y_up_without_replacing_on_failure() {
        let mut editor = crate::Editor::default();
        editor
            .execute(Command::AddRectangle {
                name: "Original snapshot panel".into(),
                origin: [1.5, 2.0],
                width: 2.8,
                height: 1.0,
            })
            .unwrap();
        let piece = &editor.project.pieces[0];
        let id = piece.id;
        let right = piece.boundary[1];
        editor
            .execute(Command::Transaction(vec![
                Command::AddHole {
                    piece: id,
                    vertices: vec![[2.35, 2.25], [2.35, 2.55], [2.65, 2.55], [2.65, 2.25]],
                },
                Command::AddInternalSegment {
                    piece: id,
                    start: [2.7, 2.8],
                    end: [3.2, 2.8],
                },
                Command::AddNotch {
                    piece: id,
                    curve: right,
                    t: 0.6,
                    length: 0.095,
                },
            ]))
            .unwrap();
        let project = editor.project;
        let original = project.to_json().unwrap();
        let [min, max] = project.pattern_snapshot_bounds().unwrap();
        assert_eq!(min, [1.5, 2.0]);
        assert!((max[0] - 4.3475).abs() < 1e-12);
        assert_eq!(max[1], 3.0);

        let dir = std::env::temp_dir().join(format!(
            "weftforge-pattern-png-{}-{}",
            std::process::id(),
            super::super::TEMP_ID.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&dir).unwrap();
        let output = dir.join("pattern.png");
        fs::write(&output, b"previous snapshot").unwrap();
        let options = PatternSnapshotOptions {
            crop_m: [[2.0, 1.611], [4.5245, 3.641]],
            pixels_per_inch: 20.49 * 0.0254,
            contours: true,
            internal_lines: true,
            notches: true,
            grainline: false,
        };
        for invalid in [
            PatternSnapshotOptions {
                crop_m: [[0.0, 0.0], [400.0, 400.0]],
                ..options
            },
            PatternSnapshotOptions {
                crop_m: [[f64::NAN, 0.0], [1.0, 1.0]],
                ..options
            },
            PatternSnapshotOptions {
                pixels_per_inch: 0.001,
                ..options
            },
        ] {
            assert!(project.export_pattern_png(&output, invalid).is_err());
            assert_eq!(fs::read(&output).unwrap(), b"previous snapshot");
        }
        assert_eq!(options.dimensions().unwrap(), [50, 41]);
        let stage = project.stage_pattern_png(&output, options).unwrap();
        assert_eq!(fs::read(&output).unwrap(), b"previous snapshot");
        stage.commit().unwrap();

        let read = || {
            let data = fs::read(&output).unwrap();
            let mut reader = png::Decoder::new(Cursor::new(data)).read_info().unwrap();
            let density = reader.info().pixel_dims.unwrap();
            assert_eq!(
                (density.xppu, density.yppu, density.unit),
                (20, 20, png::Unit::Meter)
            );
            let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
            let frame = reader.next_frame(&mut pixels).unwrap();
            assert_eq!((frame.width, frame.height), (50, 41));
            assert_eq!(frame.color_type, png::ColorType::Rgba);
            move |x: usize, y: usize| pixels[4 * (50 * y + x)]
        };
        let pixel = read();
        assert!(
            (1..255).contains(&pixel(1, 12)),
            "cropped top contour has AA coverage"
        );
        assert_eq!(
            pixel(1, 8),
            255,
            "material Y increases toward the image top"
        );
        assert!(
            pixel(45, 28) < 255,
            "right edge is at declared 46 px, not 47.1 px"
        );
        assert!(pixel(10, 21) < 255, "hole boundary");
        assert!(pixel(17, 16) < 255, "internal straight line");
        assert!(pixel(46, 20) < 255, "notch on the right boundary");

        project
            .export_pattern_png(
                &output,
                PatternSnapshotOptions {
                    contours: false,
                    ..options
                },
            )
            .unwrap();
        let pixel = read();
        assert_eq!(pixel(1, 12), 255);
        assert_eq!(pixel(10, 21), 255);
        assert!(pixel(17, 16) < 255);
        assert!(pixel(46, 20) < 255);
        assert_eq!(project.to_json().unwrap(), original);
        fs::remove_dir_all(&dir).unwrap();
    }
}
