use eframe::{egui, wgpu};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use weftforge_core::StagedExport;

const MAX_SIDE: usize = 16_384;
const MAX_PIXELS: usize = 16_777_216;
const MAX_PNG_BYTES: usize = 80 * 1024 * 1024;
const READBACK_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy)]
pub(super) struct RenderedFrame {
    pub number: u64,
    pub revision: u64,
}

struct Request {
    token: egui::UserData,
    path: PathBuf,
    revision: u64,
    requested_frame: u64,
    captured: Option<(RenderedFrame, [usize; 2])>,
    deadline: Instant,
}

enum State {
    Waiting(Request),
    Saving {
        request: Request,
        receiver: mpsc::Receiver<Result<StagedExport, String>>,
        cancelled: bool,
    },
}

#[derive(Default)]
pub(super) struct WorkspaceCapture {
    supported: bool,
    state: Option<State>,
}

impl WorkspaceCapture {
    pub fn new(format: Option<wgpu::TextureFormat>) -> Self {
        Self {
            supported: matches!(
                format,
                Some(wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm)
            ),
            state: None,
        }
    }

    pub fn busy(&self) -> bool {
        self.state.is_some()
    }

    pub fn start(
        &mut self,
        ctx: &egui::Context,
        revision: u64,
        project: Option<&Path>,
        path: PathBuf,
    ) -> Result<(), String> {
        if self.busy() {
            return Err("Wait for the current workspace PNG save to finish.".into());
        }
        if !self.supported || ctx.viewport_id() != egui::ViewportId::ROOT {
            return Err("Workspace capture is unavailable on this renderer.".into());
        }
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
        {
            return Err("Choose a .png destination for the workspace image.".into());
        }
        super::export_path_guard(project, &path)?;
        viewport_dimensions(ctx)?;
        let token = egui::UserData::new(());
        self.state = Some(State::Waiting(Request {
            token: token.clone(),
            path,
            revision,
            requested_frame: ctx.cumulative_frame_nr(),
            captured: None,
            deadline: Instant::now() + READBACK_TIMEOUT,
        }));
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(token));
        ctx.request_repaint();
        Ok(())
    }

    pub fn cancel(&mut self) -> bool {
        match &mut self.state {
            Some(State::Waiting(_)) => {
                self.state = None;
                true
            }
            Some(State::Saving { cancelled, .. }) => {
                let active = !*cancelled;
                *cancelled = true;
                active
            }
            None => false,
        }
    }

    pub fn receive(&mut self, ctx: &egui::Context, revision: u64) -> Option<String> {
        let Some(State::Waiting(request)) = &self.state else {
            return None;
        };
        if request.revision != revision || Instant::now() >= request.deadline {
            return None;
        }
        let mut image = None;
        ctx.input_mut(|input| {
            input.events.retain(|event| {
                if let egui::Event::Screenshot {
                    viewport_id,
                    user_data,
                    image: pixels,
                } = event
                    && *viewport_id == egui::ViewportId::ROOT
                    && *user_data == request.token
                {
                    image = Some(Arc::clone(pixels));
                    false
                } else {
                    true
                }
            });
        });
        let image = image?;
        let Some(State::Waiting(request)) = self.state.take() else {
            unreachable!("only a pending capture can receive pixels");
        };
        let Some((_, size)) = request.captured else {
            return Some("Workspace capture failed: no matching rendered frame.".into());
        };
        if let Err(error) = image_dimensions(&image) {
            return Some(error);
        }
        if image.size != size {
            return Some("Workspace capture discarded: the window size changed.".into());
        }
        let path = request.path.clone();
        let ctx = ctx.clone();
        let (sender, receiver) = mpsc::channel();
        if let Err(error) = std::thread::Builder::new().spawn(move || {
            let result = encode_png(&image)
                .and_then(|bytes| StagedExport::write(path, &bytes).map_err(|e| e.to_string()));
            let _ = sender.send(result);
            ctx.request_repaint();
        }) {
            return Some(format!("Workspace PNG worker could not start: {error}"));
        }
        self.state = Some(State::Saving {
            request,
            receiver,
            cancelled: false,
        });
        Some("Saving workspace PNG… Escape cancels.".into())
    }

    // Commit after this pass's authoring, cancellation and document lifecycle actions.
    pub fn finish_frame(
        &mut self,
        ctx: &egui::Context,
        frame: RenderedFrame,
        project: Option<&Path>,
    ) -> Option<String> {
        let state = self.state.take()?;
        let message = match state {
            State::Waiting(mut request) => {
                if request.revision != frame.revision {
                    Some("Workspace capture discarded: the document changed.".into())
                } else if Instant::now() >= request.deadline {
                    Some("Workspace capture timed out; previous file retained.".into())
                } else {
                    // eframe 0.35 applies Screenshot after painting the request frame.
                    if frame.number == request.requested_frame + 1 {
                        match viewport_dimensions(ctx) {
                            Ok(size) => request.captured = Some((frame, size)),
                            Err(error) => return Some(error),
                        }
                    }
                    self.state = Some(State::Waiting(request));
                    None
                }
            }
            State::Saving {
                request,
                receiver,
                cancelled,
            } => {
                let obsolete = request.revision != frame.revision;
                match receiver.try_recv() {
                    Ok(Err(error)) => Some(format!(
                        "Workspace PNG save failed; previous file retained: {error}"
                    )),
                    Ok(Ok(staged)) => {
                        if obsolete || cancelled {
                            drop(staged);
                            Some(if obsolete {
                                "Workspace PNG discarded: the document changed; previous file retained."
                                    .into()
                            } else {
                                "Workspace PNG cancelled; previous file retained.".into()
                            })
                        } else {
                            let result = super::export_path_guard(project, &request.path)
                                .and_then(|()| staged.commit().map_err(|e| e.to_string()));
                            Some(match result {
                                Ok(()) => {
                                    let (_, [width, height]) =
                                        request.captured.expect("admitted rendered frame");
                                    format!("Saved workspace PNG · {width} × {height} px.")
                                }
                                Err(error) => format!(
                                    "Workspace PNG save failed; previous file retained: {error}"
                                ),
                            })
                        }
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        Some("Workspace PNG worker stopped; previous file retained.".into())
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        self.state = Some(State::Saving {
                            request,
                            receiver,
                            cancelled: cancelled || obsolete,
                        });
                        None
                    }
                }
            }
        };
        if self.busy() {
            ctx.request_repaint_after(Duration::from_millis(20));
        }
        message
    }

    pub fn shutdown(&mut self) {
        // The worker never commits; a late send fails and drops its staged file.
        self.state = None;
    }
}

fn dimensions(size: [usize; 2]) -> Result<(), String> {
    if size.contains(&0)
        || size.iter().any(|side| *side > MAX_SIDE)
        || size[0]
            .checked_mul(size[1])
            .is_none_or(|pixels| pixels > MAX_PIXELS)
    {
        Err("Workspace PNG requires nonzero dimensions up to 16,384 px per side and 16,777,216 pixels total.".into())
    } else {
        Ok(())
    }
}

fn viewport_dimensions(ctx: &egui::Context) -> Result<[usize; 2], String> {
    let scale = ctx.pixels_per_point();
    let size = ctx.viewport_rect().size() * scale;
    if !scale.is_finite()
        || scale <= 0.0
        || !size.x.is_finite()
        || !size.y.is_finite()
        || size.x <= 0.0
        || size.y <= 0.0
        || size.x > MAX_SIDE as f32
        || size.y > MAX_SIDE as f32
    {
        return Err("Workspace PNG requires a valid, bounded visible viewport.".into());
    }
    let size = [size.x.round() as usize, size.y.round() as usize];
    dimensions(size)?;
    Ok(size)
}

fn image_dimensions(image: &egui::ColorImage) -> Result<(), String> {
    dimensions(image.size)?;
    if image.pixels.len() != image.size[0] * image.size[1] {
        return Err("Workspace capture returned an incomplete image.".into());
    }
    Ok(())
}

fn encode_png(image: &egui::ColorImage) -> Result<Vec<u8>, String> {
    image_dimensions(image)?;
    let mut rgba = Vec::new();
    rgba.try_reserve_exact(image.pixels.len() * 4)
        .map_err(|e| e.to_string())?;
    for pixel in &image.pixels {
        rgba.extend_from_slice(&pixel.to_srgba_unmultiplied());
    }
    let [width, height] = image
        .size
        .map(|side| u32::try_from(side).expect("bounded size"));
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(&rgba).map_err(|e| e.to_string())?;
        writer.finish().map_err(|e| e.to_string())?;
    }
    if bytes.len() > MAX_PNG_BYTES {
        return Err("Workspace PNG exceeds the 80 MiB output limit.".into());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::Cursor,
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "weftforge-workspace-png-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn image() -> egui::ColorImage {
        egui::ColorImage::new(
            [2, 2],
            vec![
                egui::Color32::RED,
                egui::Color32::GREEN,
                egui::Color32::BLUE,
                egui::Color32::from_rgba_premultiplied(16, 32, 48, 128),
            ],
        )
    }

    #[test]
    fn png_retains_native_pixels_orientation_alpha_and_only_standard_srgb_metadata() {
        let bytes = encode_png(&image()).unwrap();
        let mut reader = png::Decoder::new(Cursor::new(&bytes)).read_info().unwrap();
        assert_eq!(
            reader.info().srgb,
            Some(png::SrgbRenderingIntent::Perceptual)
        );
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let output = reader.next_frame(&mut pixels).unwrap();
        assert_eq!((output.width, output.height), (2, 2));
        assert_eq!(output.color_type, png::ColorType::Rgba);
        assert_eq!(output.bit_depth, png::BitDepth::Eight);
        assert_eq!(
            pixels,
            [
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 32, 64, 96, 128
            ]
        );
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        let mut position = 8;
        let mut chunks = Vec::new();
        while position < bytes.len() {
            let length =
                u32::from_be_bytes(bytes[position..position + 4].try_into().unwrap()) as usize;
            let kind = &bytes[position + 4..position + 8];
            assert!(matches!(kind, b"IHDR" | b"sRGB" | b"IDAT" | b"IEND"));
            if kind == b"sRGB" {
                assert_eq!(&bytes[position + 8..position + 8 + length], &[0]);
            }
            chunks.push(kind);
            position += length + 12;
        }
        assert_eq!(position, bytes.len());
        assert_eq!(chunks.first().copied(), Some(b"IHDR".as_slice()));
        assert_eq!(chunks.last().copied(), Some(b"IEND".as_slice()));
        assert_eq!(chunks.iter().filter(|kind| **kind == b"sRGB").count(), 1);
        for size in [
            [0, 2],
            [MAX_SIDE + 1, 1],
            [MAX_SIDE, MAX_SIDE],
            [usize::MAX, 2],
        ] {
            assert!(dimensions(size).is_err());
        }
        let mut malformed = image();
        let _ = malformed.pixels.pop();
        assert!(encode_png(&malformed).is_err());
        let ctx = egui::Context::default();
        let mut raw = input(vec![]);
        raw.viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(2.0);
        let _ = ctx.run_ui(raw, |_| {
            assert_eq!(viewport_dimensions(&ctx).unwrap(), [4, 4]);
        });
    }

    fn stamp(ctx: &egui::Context, revision: u64) -> RenderedFrame {
        RenderedFrame {
            number: ctx.cumulative_frame_nr(),
            revision,
        }
    }

    fn input(events: Vec<egui::Event>) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::splat(2.0),
            )),
            events,
            ..Default::default()
        }
    }

    fn advance(
        ctx: &egui::Context,
        capture: &mut WorkspaceCapture,
        revision: u64,
        events: Vec<egui::Event>,
    ) -> Option<String> {
        let mut message = None;
        let _ = ctx.run_ui(input(events), |_| {
            message = capture.receive(ctx, revision);
            message = capture
                .finish_frame(ctx, stamp(ctx, revision), None)
                .or(message.take());
        });
        message
    }

    #[test]
    fn capture_admission_and_late_commit_guards_preserve_the_previous_file() {
        for case in [
            "success",
            "wrong-token",
            "wrong-viewport",
            "timeout",
            "cancel",
            "stale",
            "resize",
            "malformed",
            "oversized",
            "late-cancel",
            "late-stale",
            "late-native",
            "worker-error",
            "shutdown",
        ] {
            let directory = Directory::new();
            let path = directory.0.join("workspace.png");
            let mut previous = b"previous image".to_vec();
            fs::write(&path, &previous).unwrap();
            let ctx = egui::Context::default();
            let mut capture = WorkspaceCapture::new(Some(wgpu::TextureFormat::Bgra8Unorm));
            let _ = ctx.run_ui(input(vec![]), |_| {
                assert!(
                    WorkspaceCapture::default()
                        .start(&ctx, 7, None, path.clone())
                        .is_err()
                );
                assert!(
                    capture
                        .start(&ctx, 7, None, path.with_extension("weftforge"))
                        .is_err()
                );
                capture.start(&ctx, 7, None, path.clone()).unwrap();
                assert!(capture.start(&ctx, 7, None, path.clone()).is_err());
                let _ = capture.finish_frame(&ctx, stamp(&ctx, 7), None);
            });
            let _ = advance(&ctx, &mut capture, 7, vec![]);
            let Some(State::Waiting(request)) = &capture.state else {
                panic!("pending {case}");
            };
            let token = request.token.clone();
            assert_eq!(
                request.captured.unwrap().0.number,
                request.requested_frame + 1
            );
            assert_eq!(request.captured.unwrap().0.revision, 7);
            let mut revision = 7;
            let mut event_image = image();
            match case {
                "cancel" => {
                    capture.cancel();
                }
                "stale" | "late-stale" => revision += 1,
                "timeout" => {
                    if let Some(State::Waiting(request)) = &mut capture.state {
                        request.deadline = Instant::now() - Duration::from_millis(1);
                    }
                }
                "resize" => event_image.size = [1, 4],
                "malformed" => {
                    event_image.pixels.pop();
                }
                "oversized" => event_image.size = [MAX_SIDE + 1, 1],
                _ => {}
            }
            let events = if matches!(
                case,
                "late-cancel" | "late-stale" | "late-native" | "worker-error" | "shutdown"
            ) {
                let Some(State::Waiting(request)) = capture.state.take() else {
                    unreachable!()
                };
                let (sender, receiver) = mpsc::channel();
                sender
                    .send(if case == "worker-error" {
                        Err("injected encoding failure".into())
                    } else {
                        Ok(StagedExport::write(&path, b"prepared image").unwrap())
                    })
                    .ok()
                    .unwrap();
                capture.state = Some(State::Saving {
                    request,
                    receiver,
                    cancelled: false,
                });
                if matches!(case, "late-cancel" | "worker-error") {
                    capture.cancel();
                }
                if case == "late-native" {
                    previous = weftforge_core::Project::default().to_json().unwrap();
                    fs::write(&path, &previous).unwrap();
                }
                if case == "shutdown" {
                    capture.shutdown();
                }
                vec![]
            } else {
                vec![egui::Event::Screenshot {
                    viewport_id: if case == "wrong-viewport" {
                        egui::ViewportId::from_hash_of("other")
                    } else {
                        egui::ViewportId::ROOT
                    },
                    user_data: if case == "wrong-token" {
                        egui::UserData::new(())
                    } else {
                        token
                    },
                    image: Arc::new(event_image),
                }]
            };
            let mut message = advance(&ctx, &mut capture, revision, events);
            if matches!(case, "wrong-token" | "wrong-viewport") {
                assert!(capture.busy());
                capture.cancel();
            }
            let deadline = Instant::now() + Duration::from_secs(5);
            while capture.busy() {
                assert!(Instant::now() < deadline, "worker did not retire: {case}");
                std::thread::yield_now();
                message = advance(&ctx, &mut capture, revision, vec![]).or(message);
            }
            if case == "success" {
                assert_eq!(fs::read(&path).unwrap(), encode_png(&image()).unwrap());
                assert_eq!(message.unwrap(), "Saved workspace PNG · 2 × 2 px.");
            } else {
                assert_eq!(fs::read(&path).unwrap(), previous, "{case}");
                if case == "worker-error" {
                    assert!(message.unwrap().contains("injected encoding failure"));
                }
            }
            assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1, "{case}");
        }
    }

    #[test]
    fn shutdown_releases_a_live_worker_without_waiting_and_drops_its_late_result() {
        let directory = Directory::new();
        let path = directory.0.join("workspace.png");
        fs::write(&path, b"previous image").unwrap();
        let ctx = egui::Context::default();
        let mut capture = WorkspaceCapture::new(Some(wgpu::TextureFormat::Rgba8Unorm));
        let _ = ctx.run_ui(input(vec![]), |_| {
            capture.start(&ctx, 7, None, path.clone()).unwrap();
            let _ = capture.finish_frame(&ctx, stamp(&ctx, 7), None);
        });
        let _ = advance(&ctx, &mut capture, 7, vec![]);
        let Some(State::Waiting(request)) = capture.state.take() else {
            unreachable!()
        };
        let (sender, receiver) = mpsc::channel();
        capture.state = Some(State::Saving {
            request,
            receiver,
            cancelled: false,
        });
        assert!(capture.cancel());
        assert!(capture.busy());
        assert!(capture.start(&ctx, 7, None, path.clone()).is_err());

        let (finished, completion) = mpsc::channel();
        let exit = std::thread::spawn(move || {
            capture.shutdown();
            assert!(!capture.busy());
            finished.send(()).unwrap();
        });
        if completion.recv_timeout(Duration::from_secs(1)).is_err() {
            // Release a regressed blocking shutdown before reporting its failure.
            drop(sender);
            exit.join().unwrap();
            panic!("shutdown waited for a live worker without a result");
        }
        exit.join().unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"previous image");

        let staged = StagedExport::write(&path, b"late image").unwrap();
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);
        let late = sender.send(Ok(staged));
        assert!(late.is_err());
        drop(late);
        assert_eq!(fs::read(&path).unwrap(), b"previous image");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
    }

    #[test]
    fn file_and_palette_capture_routes_retain_pending_text_history_and_accepted_display() {
        use super::super::{
            Studio,
            native_idle_numeric_tests::{action, button, frame},
            pattern_transform_tests::key,
            studio_style,
        };
        use egui::accesskit::Action;
        use weftforge_core::Command;

        let directory = Directory::new();
        let path = directory.0.join("workspace.png");
        for search in [false, true] {
            for pointer in [false, true] {
                for cancel in [false, true] {
                    fs::write(&path, b"previous image").unwrap();
                    let ctx = egui::Context::default();
                    ctx.enable_accesskit();
                    studio_style(&ctx);
                    let mut app = Studio::empty();
                    app.session_autosave = None;
                    app.workspace_capture =
                        WorkspaceCapture::new(Some(wgpu::TextureFormat::Rgba8Unorm));
                    app.editor
                        .execute(Command::AddRectangle {
                            name: "Capture panel".into(),
                            origin: [0., 0.],
                            width: 0.4,
                            height: 0.6,
                        })
                        .unwrap();
                    app.editor
                        .execute(Command::SetResolution { metres: 0.08 })
                        .unwrap();
                    assert!(app.editor.undo());
                    app.selected = Some(app.editor.project.pieces[0].id);
                    let mesh = Arc::new(app.editor.project.mesh().unwrap());
                    let cloth = Arc::new(
                        weftforge_core::simulation::create_cloth(&app.editor.project, &mesh)
                            .unwrap(),
                    );
                    app.mesh = Some(Arc::clone(&mesh));
                    app.cloth = Some(Arc::clone(&cloth));
                    let before = app.editor.clone();
                    let output = frame(&ctx, &mut app, 1800., vec![]);
                    let numeric = output
                        .platform_output
                        .accesskit_update
                        .as_ref()
                        .unwrap()
                        .nodes
                        .iter()
                        .find(|(_, node)| node.numeric_value() == Some(0.4))
                        .unwrap()
                        .0;
                    frame(
                        &ctx,
                        &mut app,
                        1800.,
                        vec![action(numeric, Action::Focus, None)],
                    );
                    frame(&ctx, &mut app, 1800., vec![]);
                    let output = frame(
                        &ctx,
                        &mut app,
                        1800.,
                        vec![
                            key(egui::Key::A, egui::Modifiers::COMMAND),
                            egui::Event::Text("1e-".into()),
                        ],
                    );
                    let focused = ctx.memory(|memory| memory.focused()).unwrap();
                    let caret = egui::TextEdit::load_state(&ctx, focused)
                        .unwrap()
                        .cursor
                        .char_range();
                    app.draft.as_mut().unwrap().resolution = 0.08;
                    let pending = app.draft.as_ref().unwrap().commands(&before.project);
                    let output = if search {
                        app.query = "Save workspace PNG".into();
                        frame(
                            &ctx,
                            &mut app,
                            1800.,
                            vec![key(egui::Key::K, egui::Modifiers::COMMAND)],
                        );
                        frame(&ctx, &mut app, 1800., vec![])
                    } else {
                        frame(
                            &ctx,
                            &mut app,
                            1800.,
                            vec![action(button(&output, "File"), Action::Click, None)],
                        );
                        frame(&ctx, &mut app, 1800., vec![])
                    };
                    let target = button(&output, "Save workspace PNG…");
                    ctx.data_mut(|data| {
                        data.insert_temp(
                            egui::Id::new("test-workspace-png-destination"),
                            (!cancel).then(|| path.clone()),
                        )
                    });
                    if pointer {
                        let bounds = output
                            .platform_output
                            .accesskit_update
                            .as_ref()
                            .unwrap()
                            .nodes
                            .iter()
                            .find(|(id, _)| *id == target)
                            .unwrap()
                            .1
                            .bounds()
                            .unwrap();
                        let pos = egui::pos2(
                            ((bounds.x0 + bounds.x1) * 0.5) as f32,
                            ((bounds.y0 + bounds.y1) * 0.5) as f32,
                        );
                        for pressed in [true, false] {
                            frame(
                                &ctx,
                                &mut app,
                                1800.,
                                vec![
                                    egui::Event::PointerMoved(pos),
                                    egui::Event::PointerButton {
                                        pos,
                                        button: egui::PointerButton::Primary,
                                        pressed,
                                        modifiers: egui::Modifiers::NONE,
                                    },
                                ],
                            );
                            assert_eq!(app.editor.project, before.project);
                        }
                    } else {
                        frame(
                            &ctx,
                            &mut app,
                            1800.,
                            vec![action(target, Action::Click, None)],
                        );
                    }
                    assert_eq!(app.workspace_capture.busy(), !cancel);
                    frame(&ctx, &mut app, 1800., vec![]);
                    app.workspace_capture.cancel();
                    for _ in 0..3 {
                        frame(&ctx, &mut app, 1800., vec![]);
                    }
                    assert!(!app.workspace_capture.busy());
                    assert_eq!(app.editor.project, before.project);
                    assert!(
                        !app.dirty
                            && !app.needs_mesh
                            && app.job.is_none()
                            && !app.playing
                            && app.pending_batch.is_none()
                    );
                    assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
                    assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &cloth));
                    assert_eq!(
                        app.draft.as_ref().unwrap().commands(&before.project),
                        pending
                    );
                    assert_eq!(ctx.memory(|memory| memory.focused()), Some(focused));
                    assert_eq!(
                        ctx.data_mut(|data| data.get_temp::<String>(focused)),
                        Some("1e-".into())
                    );
                    assert_eq!(
                        egui::TextEdit::load_state(&ctx, focused)
                            .unwrap()
                            .cursor
                            .char_range(),
                        caret
                    );
                    for redo in [false, true] {
                        let mut actual = app.editor.clone();
                        let mut expected = before.clone();
                        loop {
                            let actual_changed = if redo { actual.redo() } else { actual.undo() };
                            let expected_changed = if redo {
                                expected.redo()
                            } else {
                                expected.undo()
                            };
                            assert_eq!(actual_changed, expected_changed);
                            assert_eq!(actual.project, expected.project);
                            if !actual_changed {
                                break;
                            }
                        }
                    }
                    assert_eq!(fs::read(&path).unwrap(), b"previous image");
                }
            }
        }

        for overlay in ["popup", "modal"] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            studio_style(&ctx);
            let mut app = Studio::empty();
            app.session_autosave = None;
            app.workspace_capture = WorkspaceCapture::new(Some(wgpu::TextureFormat::Rgba8Unorm));
            let before = app.editor.project.clone();
            let output = frame(&ctx, &mut app, 1800., vec![]);
            app.workspace_capture
                .start(&ctx, before.revision, None, path.clone())
                .unwrap();
            let Some(State::Waiting(mut request)) = app.workspace_capture.state.take() else {
                unreachable!()
            };
            request.captured = Some((stamp(&ctx, before.revision), [1640, 1800]));
            let (sender, receiver) = mpsc::channel();
            app.workspace_capture.state = Some(State::Saving {
                request,
                receiver,
                cancelled: false,
            });
            if overlay == "popup" {
                frame(
                    &ctx,
                    &mut app,
                    1800.,
                    vec![action(button(&output, "File"), Action::Click, None)],
                );
                frame(&ctx, &mut app, 1800., vec![]);
                assert!(egui::Popup::is_any_open(&ctx));
            } else {
                app.rectangle = Some(super::super::RectangleDraft {
                    origin: [0.; 2],
                    dimensions_mm: ["400".into(), "600".into()],
                    error: None,
                });
                frame(&ctx, &mut app, 1800., vec![]);
                assert!(app.rectangle.is_some());
            }
            frame(
                &ctx,
                &mut app,
                1800.,
                vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
            );
            assert!(
                matches!(
                    app.workspace_capture.state,
                    Some(State::Saving {
                        cancelled: true,
                        ..
                    })
                ),
                "{overlay}"
            );
            assert!(app.workspace_capture.busy());
            assert!(!egui::Popup::is_any_open(&ctx));
            assert!(app.rectangle.is_none());
            assert_eq!(app.editor.project, before);
            assert!(!app.dirty);
            sender
                .send(Ok(StagedExport::write(&path, b"cancelled image").unwrap()))
                .ok()
                .unwrap();
            frame(&ctx, &mut app, 1800., vec![]);
            assert!(!app.workspace_capture.busy());
            assert_eq!(fs::read(&path).unwrap(), b"previous image");
            assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
        }
    }

    #[test]
    fn successful_same_revision_document_lifecycle_and_reset_invalidate_capture() {
        use super::super::Studio;
        let directory = Directory::new();
        let source = directory.0.join("source.weftforge");
        weftforge_core::Project::default().save(&source).unwrap();
        let path = directory.0.join("workspace.png");
        for case in ["new", "load", "failed-load", "reset", "exit"] {
            let ctx = egui::Context::default();
            let mut app = Studio::empty();
            app.session_autosave = None;
            app.workspace_capture = WorkspaceCapture::new(Some(wgpu::TextureFormat::Rgba8Unorm));
            let revision = app.editor.project.revision;
            let _ = ctx.run_ui(input(vec![]), |_| {
                app.workspace_capture
                    .start(&ctx, revision, None, path.clone())
                    .unwrap();
            });
            match case {
                "new" => app.new_empty(&ctx, None),
                "load" => app.load_document(source.clone(), false),
                "failed-load" => app.load_document(directory.0.join("missing.weftforge"), false),
                "reset" => app.reset_simulation(),
                "exit" => {
                    app.retain_on_exit().unwrap();
                }
                _ => unreachable!(),
            }
            assert_eq!(app.editor.project.revision, revision);
            assert_eq!(
                app.workspace_capture.busy(),
                case == "failed-load",
                "{case}"
            );
        }
        assert!(!path.exists());
    }
}
