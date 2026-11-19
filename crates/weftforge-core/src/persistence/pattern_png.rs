use super::{StagedExport, export_path_guard};
use crate::{Error, Project, Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;
use tiny_skia::{IntSize, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

const METRES_PER_INCH: f64 = 0.0254;
const MAX_SIDE: u32 = 16_384;
const MAX_PIXELS: u64 = 32_000_000;
const MAX_PNG_BYTES: usize = 80 * 1024 * 1024;
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

    fn grid(self) -> Result<([u32; 2], u32)> {
        let [min, max] = self.crop_m;
        ensure(
            min.into_iter().chain(max).all(f64::is_finite)
                && (0..2).all(|axis| min[axis] < max[axis]),
            "Pattern PNG crop needs finite, increasing X and Y bounds",
        )?;
        let ppm = self.pixels_per_metre()?;
        let size = std::array::from_fn(|axis| ((max[axis] - min[axis]) * f64::from(ppm)).round());
        ensure(
            size.iter()
                .all(|side| (1.0..=f64::from(MAX_SIDE)).contains(side)),
            "Pattern PNG width and height must be between 1 and 16384 pixels",
        )?;
        let size = size.map(|side| side as u32);
        ensure(
            u64::from(size[0]) * u64::from(size[1]) <= MAX_PIXELS,
            "Pattern PNG exceeds the 32 megapixel limit",
        )?;
        Ok((size, ppm))
    }
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
        let pixmap = rasterize(self, options, size, ppm)?;
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
}

struct Strokes {
    origin_m: [f64; 2],
    size: [u32; 2],
    ppm: f64,
    tiles: u64,
    work: u64,
    path: PathBuilder,
}

impl Strokes {
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
        ensure(
            points
                .iter()
                .flatten()
                .all(|value| value.abs() <= MAX_PROJECTED_COORDINATE),
            "Pattern PNG crop and density exceed raster coordinate precision",
        )?;
        // Bound pixel visits by control polygon length, with overhead for tiny-skia subdivisions.
        let length: u64 = points
            .windows(2)
            .map(|pair| {
                ((pair[0][0] - pair[1][0]).abs() + (pair[0][1] - pair[1][1]).abs()).ceil() as u64
            })
            .sum();
        let overhead = if controls.is_some() { 4096 } else { 8 };
        self.work += (length + overhead) * self.tiles;
        ensure(
            self.work <= MAX_STROKE_WORK,
            "Pattern PNG exceeds the raster stroke work limit",
        )?;
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
    ppm: u32,
) -> Result<Pixmap> {
    let [[left, _], [_, top]] = options.crop_m;
    let mut strokes = Strokes {
        origin_m: [left, top],
        size: [width, height],
        ppm: f64::from(ppm),
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
    let bytes = width as usize * height as usize * 4;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(bytes)
        .map_err(|error| Error(format!("Could not allocate pattern PNG pixels: {error}")))?;
    pixels.resize(bytes, 255);
    let size =
        IntSize::from_wh(width, height).ok_or_else(|| Error("Invalid raster size".into()))?;
    let mut pixmap =
        Pixmap::from_vec(pixels, size).ok_or_else(|| Error("Invalid pattern PNG buffer".into()))?;
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

#[derive(Default)]
struct PngBuffer(Vec<u8>);

impl Write for PngBuffer {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let length = self.0.len() + data.len();
        if length > MAX_PNG_BYTES {
            return Err(io::Error::other(
                "Pattern PNG exceeds the 80 MiB file limit",
            ));
        }
        if length > self.0.capacity() {
            let capacity = length.max(self.0.capacity() * 2).min(MAX_PNG_BYTES);
            self.0
                .try_reserve_exact(capacity - self.0.len())
                .map_err(io::Error::other)?;
        }
        self.0.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn encode_png(pixmap: &Pixmap, ppm: u32) -> Result<Vec<u8>> {
    let png_error = |error| Error(format!("Could not encode pattern PNG: {error}"));
    let mut bytes = PngBuffer::default();
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
    Ok(bytes.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use std::fs;
    use std::io::Cursor;
    use std::sync::atomic::Ordering;

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
