//! Transient accepted simulation positions. The live solver owns continuation state.
use super::{Studio, egui};

const MAX_FRAMES: usize = 600;
const POSITION_BUDGET: usize = 64 * 1024 * 1024;

pub(super) struct Frame {
    source_revision: u64,
    pub time: f64,
    pub dt: Option<f64>,
    pub positions: Box<[[f64; 3]]>,
}

pub(super) struct Capture {
    source_revision: u64,
    generation: u64,
    remaining: usize,
    frames: Vec<Frame>,
}

impl Capture {
    pub fn step(&mut self, time: f64, dt: f64, positions: &[[f64; 3]]) {
        if self.frames.len() < self.remaining {
            self.frames.push(Frame {
                source_revision: self.source_revision,
                time,
                dt: Some(dt),
                positions: positions.into(),
            });
        }
    }
}

#[derive(Default)]
pub(super) struct Timeline {
    frames: Vec<Frame>,
    revision: Option<u64>,
    limit: usize,
    recording: bool,
    generation: u64,
    selection: Option<usize>,
}

impl Timeline {
    pub fn clear(&mut self) {
        *self = Self {
            generation: self.generation.wrapping_add(1),
            ..Self::default()
        };
    }

    pub fn surface_edit(&mut self, previous: u64, current: u64) {
        if self.revision == Some(previous) {
            self.revision = Some(current);
        }
    }

    pub fn live(&mut self) {
        self.selection = None;
    }

    fn stop(&mut self) {
        self.recording = false;
        // An outstanding worker owns only the recording generation it started with.
        self.generation = self.generation.wrapping_add(1);
    }

    fn begin(&mut self, revision: u64, time: f64, positions: &[[f64; 3]]) {
        if self.revision != Some(revision) {
            self.clear();
        }
        self.revision = Some(revision);
        self.live();
        let bytes = std::mem::size_of_val(positions);
        self.limit = (POSITION_BUDGET / bytes.max(1)).min(MAX_FRAMES);
        if self.frames.len() < self.limit
            && self.frames.last().is_none_or(|frame| frame.time != time)
        {
            self.frames.push(Frame {
                source_revision: revision,
                time,
                dt: None,
                positions: positions.into(),
            });
        }
        self.recording = self.frames.len() < self.limit;
    }

    pub fn capture(&self) -> Option<Capture> {
        self.recording.then(|| Capture {
            source_revision: self.revision.expect("active recording revision"),
            generation: self.generation,
            remaining: self.limit - self.frames.len(),
            frames: Vec::new(),
        })
    }

    pub fn accept(&mut self, capture: Option<Capture>, revision: u64) {
        if let Some(mut capture) = capture
            && self.recording
            && self.generation == capture.generation
            && self.revision == Some(revision)
        {
            self.frames.append(&mut capture.frames);
            if self.frames.len() == self.limit {
                self.stop();
            }
        }
    }

    // Recording can be armed after a batch was dispatched. Its final state
    // first becomes observable when the entire batch is accepted.
    pub fn accepted_endpoint(
        &mut self,
        source_revision: u64,
        time: f64,
        dt: f64,
        positions: &[[f64; 3]],
    ) {
        if self.recording && self.frames.last().is_some_and(|last| last.time < time) {
            self.frames.push(Frame {
                source_revision,
                time,
                dt: Some(dt),
                positions: positions.into(),
            });
            if self.frames.len() == self.limit {
                self.stop();
            }
        }
    }

    pub fn selected(&self, revision: u64) -> Option<((u64, usize), &Frame)> {
        if self.revision != Some(revision) {
            return None;
        }
        let i = self.selection?;
        Some(((self.generation, i), self.frames.get(i)?))
    }

    pub fn has_frames(&self) -> bool {
        !self.frames.is_empty()
    }

    fn full(&self) -> bool {
        self.has_frames() && self.frames.len() == self.limit
    }

    fn nearest(&self, time: f64) -> usize {
        let i = self.frames.partition_point(|frame| frame.time < time);
        if i == 0 {
            return 0;
        }
        if i == self.frames.len() || time - self.frames[i - 1].time <= self.frames[i].time - time {
            i - 1
        } else {
            i
        }
    }
}

impl Studio {
    pub(super) fn record_button(&mut self, ui: &mut egui::Ui) {
        let revision = self.editor.project.revision;
        let ready = self
            .mesh
            .as_ref()
            .is_some_and(|m| m.doc_revision == revision && !m.positions.is_empty());
        let response = ui.add_enabled(
            ready && !self.timeline.full(),
            egui::Button::new(if self.timeline.recording {
                "Stop recording"
            } else {
                "Record"
            }),
        );
        self.retain_inspector_input(&response, false);
        if response.clicked() {
            if self.timeline.recording {
                self.timeline.stop();
            } else if let Some(mesh) = self
                .mesh
                .as_ref()
                .filter(|m| m.doc_revision == self.editor.project.revision)
            {
                let (time, positions) = self
                    .cloth
                    .as_ref()
                    .map_or((0., mesh.positions.as_slice()), |s| {
                        (s.simulation_time(), s.positions())
                    });
                self.timeline
                    .begin(self.editor.project.revision, time, positions);
            }
        }
    }

    fn scrub(&mut self, index: usize) {
        self.playing = false;
        self.pending_batch = None;
        if self
            .job
            .as_ref()
            .is_some_and(|j| j.kind == super::JobKind::Simulation)
        {
            self.cancel();
        }
        self.timeline.stop();
        self.timeline.selection = Some(index);
        let live_time = self
            .cloth
            .as_ref()
            .map_or(0., |state| state.simulation_time());
        self.message = format!(
            "Viewing frame {} at {:.6} s · latest solver {:.6} s",
            index + 1,
            self.timeline.frames[index].time,
            live_time
        );
        self.workspace_capture.cancel();
    }

    pub(super) fn timeline_bar(&mut self, ui: &mut egui::Ui) {
        if !self.timeline.has_frames() {
            return;
        }
        ui.horizontal(|ui| {
            let len = self.timeline.frames.len();
            let old = self.timeline.selection.unwrap_or(len - 1);
            let mut frame = old + 1;
            let slider = ui.add(egui::Slider::new(&mut frame, 1..=len).text("Frame"));
            ui.ctx()
                .accesskit_node_builder(slider.id, |node| node.set_label("Timeline frame"));
            self.retain_inspector_input(&slider, false);
            if slider.changed() || slider.clicked() {
                self.scrub(frame - 1);
            }

            let current = self.timeline.selection.unwrap_or(len - 1);
            let mut time = self.timeline.frames[current].time;
            let time_label = ui.label("Time (s)").on_hover_text(format!(
                "Simulation frame from project revision {}",
                self.timeline.frames[current].source_revision
            ));
            let response = ui
                .add(
                    egui::DragValue::new(&mut time)
                        .range(self.timeline.frames[0].time..=self.timeline.frames[len - 1].time)
                        .speed(0.01)
                        .max_decimals(6),
                )
                .labelled_by(time_label.id);
            self.retain_inspector_input(&response, false);
            ui.ctx()
                .accesskit_node_builder(response.id, |node| node.set_label("Timeline time (s)"));
            if response.changed() && self.timeline.nearest(time) != current {
                self.scrub(self.timeline.nearest(time));
            }
            let current = self.timeline.selection.unwrap_or(len - 1);
            if let Some(dt) = self.timeline.frames[current].dt {
                ui.small(format!("Δt {dt:.6} s"));
            }
            let live = ui.add_enabled(self.timeline.selection.is_some(), egui::Button::new("Live"));
            self.retain_inspector_input(&live, false);
            if live.clicked() {
                self.timeline.live();
                self.workspace_capture.cancel();
            }
            let clear = ui.button("Clear recording");
            self.retain_inspector_input(&clear, false);
            if clear.clicked() {
                self.timeline.clear();
                self.workspace_capture.cancel();
                return;
            }
            if self.timeline.full() {
                ui.label(format!("Recording limit reached · {len} frames"));
            } else {
                ui.label(if self.timeline.recording {
                    format!("Recording · {len} frames")
                } else {
                    format!("{len} recorded")
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Command, JobStatus, SimulationBatch,
        native_idle_numeric_tests::{action, button, frame},
    };
    use egui::accesskit::{Action, ActionData, Role};
    use std::{
        fs,
        sync::Arc,
        time::{Duration, Instant},
    };

    fn finish(app: &mut Studio) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while app.job.is_some() {
            assert!(Instant::now() < deadline);
            app.poll();
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn native_record_scrub_export_and_resume_preserve_the_live_pose() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        crate::studio_style(&ctx);
        let mut app = Studio::empty();
        app.session_autosave = None;
        let obj = include_bytes!("../../../fixtures/posed-obj/Bent swatch.obj");
        app.editor
            .execute(
                weftforge_core::ObjPatternImport::from_bytes_mode(
                    obj,
                    None,
                    1.,
                    weftforge_core::ObjImportMode::Posed,
                )
                .unwrap()
                .command(),
            )
            .unwrap();
        let mesh = Arc::new(app.editor.project.mesh().unwrap());
        app.mesh = Some(mesh.clone());
        app.needs_mesh = false;
        let out = frame(&ctx, &mut app, 1000., vec![]);
        frame(
            &ctx,
            &mut app,
            1000.,
            vec![action(button(&out, "Record"), Action::Click, None)],
        );
        assert!(app.timeline.recording);
        assert_eq!(app.timeline.frames.len(), 1);
        assert_eq!(app.timeline.frames[0].positions.as_ref(), mesh.positions);
        let batch = SimulationBatch {
            steps: 3,
            ..SimulationBatch::drape()
        };
        app.start_batch(&ctx, batch);
        finish(&mut app);
        assert_eq!(app.last_job.as_ref().unwrap().status, JobStatus::Accepted);
        assert_eq!(app.timeline.frames.len(), 4);
        for (i, f) in app.timeline.frames.iter().enumerate() {
            assert!((f.time - i as f64 * batch.settings.dt).abs() < 1e-12);
            assert_eq!(f.dt, (i != 0).then_some(batch.settings.dt));
        }
        frame(&ctx, &mut app, 1000., vec![]);
        assert!(app.timeline.recording && app.timeline.selection.is_none());
        let min_frame = |app: &mut Studio, events| {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1000., 650.),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
            )
        };
        min_frame(&mut app, vec![]);
        let out = min_frame(&mut app, vec![]);
        let nodes = out.platform_output.accesskit_update.unwrap().nodes;
        for label in [
            "Stop recording",
            "Timeline frame",
            "Timeline time (s)",
            "Recording · 4 frames",
            "Clear recording",
            "Live",
        ] {
            let node = nodes
                .iter()
                .find(|(_, n)| n.label() == Some(label) || n.value() == Some(label))
                .unwrap_or_else(|| panic!("Missing minimum-window control {label}"));
            let bounds = node.1.bounds().unwrap();
            assert!(
                bounds.x0 >= 0.
                    && bounds.x1 <= 1000.
                    && bounds.y0 >= 0.
                    && bounds.y1 <= 650.
                    && bounds.x1 > bounds.x0
                    && bounds.y1 > bounds.y0,
                "{label} {bounds:?}"
            );
        }
        let slider = nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Slider && n.max_numeric_value() == Some(4.))
            .unwrap()
            .0;
        min_frame(&mut app, vec![action(slider, Action::Focus, None)]);
        min_frame(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::ArrowLeft,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(app.timeline.selection, Some(2));
        frame(
            &ctx,
            &mut app,
            1000.,
            vec![action(
                slider,
                Action::SetValue,
                Some(ActionData::NumericValue(2.)),
            )],
        );
        assert_eq!(app.timeline.selection, Some(1));
        assert!(!app.timeline.recording);
        let initial = app.cloth.as_ref().unwrap().clone();
        let recorded = app.timeline.frames[1].positions.to_vec();
        assert_eq!(app.garment_surface().unwrap().1, recorded);
        assert_ne!(initial.positions(), recorded);

        let dir = std::env::temp_dir().join(format!("weftforge-timeline-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("frame.obj");
        assert!(app.export_to_path(&ctx, crate::ExportFormat::Obj, Some(&path), None));
        let readback: Vec<[f64; 3]> = fs::read_to_string(path)
            .unwrap()
            .lines()
            .filter_map(|line| line.strip_prefix("v "))
            .map(|coords| {
                let mut words = coords.split_whitespace();
                std::array::from_fn(|_| words.next().unwrap().parse().unwrap())
            })
            .collect();
        for (actual, expected) in readback.iter().zip(&recorded) {
            assert!((0..3).all(|i| (actual[i] - expected[i]).abs() < 1e-14));
        }
        assert_eq!(readback.len(), recorded.len());
        fs::remove_dir_all(dir).unwrap();

        app.timeline.begin(
            app.editor.project.revision,
            initial.simulation_time(),
            initial.positions(),
        );
        let count = app.timeline.frames.len();
        app.start_batch(&ctx, batch);
        app.cancel();
        finish(&mut app);
        assert_eq!(app.last_job.as_ref().unwrap().status, JobStatus::Cancelled);
        assert_eq!(app.timeline.frames.len(), count);
        assert!(Arc::ptr_eq(&initial, app.cloth.as_ref().unwrap()));
        app.scrub(1);
        let material = app.editor.project.pieces[0].material;
        app.execute(Command::SetFabricSurface {
            material,
            metallic: 0.2,
            roughness: 0.4,
        });
        assert_eq!(app.garment_surface().unwrap().1, recorded);
        assert!(Arc::ptr_eq(&initial, app.cloth.as_ref().unwrap()));
        app.start_batch(&ctx, SimulationBatch { steps: 1, ..batch });
        finish(&mut app);
        assert!(app.timeline.selection.is_none());
        assert!(
            (app.cloth.as_ref().unwrap().simulation_time() - 4. * batch.settings.dt).abs() < 1e-12
        );
        assert_eq!(app.timeline.frames.len(), 4);
        app.execute(Command::SetResolution { metres: 0.04 });
        assert!(!app.timeline.has_frames());
        assert!(app.garment_surface().is_none());
    }

    #[test]
    fn cancelled_obsolete_and_over_capacity_frames_cannot_enter_the_take() {
        let p = [[0.; 3]; 1];
        let mut t = Timeline::default();
        t.begin(7, 0., &p);
        let mut capture = t.capture().unwrap();
        capture.step(1., 1., &p);
        t.stop();
        t.begin(7, 0., &p);
        t.accept(Some(capture), 7);
        assert_eq!(t.frames.len(), 1);
        let mut obsolete = t.capture().unwrap();
        obsolete.step(1., 1., &p);
        t.clear();
        t.begin(8, 0., &p);
        t.accept(Some(obsolete), 8);
        assert_eq!(t.frames.len(), 1);
        let mut capture = t.capture().unwrap();
        for i in 1..=1000 {
            capture.step(i as f64, 1., &p);
        }
        t.accept(Some(capture), 8);
        assert_eq!(t.frames.len(), MAX_FRAMES);
        assert!(t.full() && !t.recording && t.capture().is_none());
        t.clear();
        t.begin(7, 0., &p);
        t.accepted_endpoint(7, 0.5, 0.1, &p);
        t.accepted_endpoint(7, 0.5, 0.1, &p);
        assert_eq!(t.frames.len(), 2);
        assert_eq!(t.frames[1].time, 0.5);
        t.clear();
        t.begin(8, 0., &vec![[0.; 3]; 100_000]);
        assert_eq!(t.limit, POSITION_BUDGET / (24 * 100_000));
    }
}
