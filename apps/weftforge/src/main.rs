use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use weftforge_core::{Command, CurveInterval, Editor, Id, Material, Mesh, Placement, Project};
use weftforge_sim::{Cloth, StepSettings};

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Select,
    Polygon,
    Rectangle,
    Sew,
}
#[derive(Clone, Copy, PartialEq)]
enum JobKind {
    Mesh,
    Simulation,
}
enum JobOutput {
    Mesh(Arc<Mesh>),
    Simulation(Arc<Cloth>),
}
#[derive(Clone, Copy)]
struct SimulationBatch {
    steps: usize,
    settings: StepSettings,
    stage: &'static str,
}
impl SimulationBatch {
    fn assemble() -> Self {
        Self {
            steps: 60,
            settings: StepSettings {
                dt: 1.0 / 60.0,
                substeps: 8,
                iterations: 32,
                gravity: [0.0; 3],
                seam_closure_speed: 1.0,
            },
            stage: "Assembly (gravity off)",
        }
    }
    fn drape() -> Self {
        Self {
            steps: 120,
            settings: StepSettings {
                gravity: [0.0, -9.81, 0.0],
                ..Self::assemble().settings
            },
            stage: "Drape",
        }
    }
}
struct Job {
    revision: u64,
    kind: JobKind,
    batch: Option<SimulationBatch>,
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Result<JobOutput, String>>,
}
impl Job {
    fn spawn(
        ctx: &egui::Context,
        revision: u64,
        kind: JobKind,
        work: impl FnOnce(&AtomicBool) -> Result<JobOutput, String> + Send + 'static,
    ) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let ctx = ctx.clone();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let result = check_cancelled(&flag)
                .and_then(|()| work(&flag))
                .and_then(|output| check_cancelled(&flag).map(|()| output));
            let _ = sender.send(result);
            ctx.request_repaint();
        });
        Self {
            revision,
            kind,
            batch: None,
            cancel,
            receiver,
        }
    }
    fn accepts(&self, revision: u64) -> bool {
        self.revision == revision && !self.cancel.load(Ordering::Relaxed)
    }
}
struct AutosaveJob {
    receiver: mpsc::Receiver<Result<(), String>>,
}

#[derive(Clone)]
struct InspectorDraft {
    revision: u64,
    selected: Option<Id>,
    dimensions: Option<[f64; 2]>,
    placement: Option<Placement>,
    grain_angle: Option<f64>,
    material: Option<Material>,
    resolution: f64,
    color_pending: bool,
}
impl InspectorDraft {
    fn new(project: &Project, selected: Option<Id>) -> Self {
        let piece = selected.and_then(|id| project.piece(id));
        Self {
            revision: project.revision,
            selected,
            dimensions: piece
                .and_then(|piece| project.piece_bounds(piece.id).ok())
                .map(|[min, max]| std::array::from_fn(|axis| max[axis] - min[axis])),
            placement: piece.map(|piece| piece.placement),
            grain_angle: piece.map(|piece| piece.grain_angle),
            material: piece
                .and_then(|piece| project.material(piece.material))
                .cloned(),
            resolution: project.resolution,
            color_pending: false,
        }
    }
    fn commands(&self, project: &Project) -> Vec<Command> {
        let original = Self::new(project, self.selected);
        let mut commands = Vec::new();
        if let Some(piece) = self.selected {
            if self.dimensions != original.dimensions
                && let Some([width, height]) = self.dimensions
            {
                commands.push(Command::ResizePiece {
                    piece,
                    width,
                    height,
                });
            }
            if self.placement != original.placement
                && let Some(placement) = self.placement
            {
                commands.push(Command::SetPlacement { piece, placement });
            }
            if self.grain_angle != original.grain_angle
                && let Some(angle) = self.grain_angle
            {
                commands.push(Command::SetGrain { piece, angle });
            }
            if self.material != original.material
                && let Some(material) = &self.material
            {
                commands.push(Command::UpdateMaterial {
                    material: material.clone(),
                });
            }
        }
        if self.resolution != project.resolution {
            commands.push(Command::SetResolution {
                metres: self.resolution,
            });
        }
        commands
    }
}
struct Studio {
    editor: Editor,
    mesh: Option<Arc<Mesh>>,
    cloth: Option<Arc<Cloth>>,
    job: Option<Job>,
    needs_mesh: bool,
    pending_batch: Option<SimulationBatch>,
    last_stage: &'static str,
    draft: Option<InspectorDraft>,
    playing: bool,
    selected: Option<Id>,
    tool: Tool,
    polygon: Vec<[f64; 2]>,
    drag_start: Option<[f64; 2]>,
    drag_point: Option<Id>,
    preview: Option<[f64; 2]>,
    seam_start: Option<Id>,
    reversed: bool,
    intervals: [[f64; 2]; 2],
    name: String,
    size: [f64; 2],
    coordinate: [f64; 2],
    path: Option<PathBuf>,
    message: String,
    center: [f64; 2],
    scale: f32,
    yaw: f32,
    pitch: f32,
    camera_scale: f32,
    wireframe: bool,
    settings: StepSettings,
    autosave: Instant,
    session_autosave: Option<PathBuf>,
    autosave_job: Option<AutosaveJob>,
    show_recovery: bool,
    recovery_files: Vec<PathBuf>,
    dirty: bool,
    search: bool,
    query: String,
}
impl Studio {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut app = Self::empty();
        if let Some(path) = std::env::args_os().nth(1) {
            app.open(PathBuf::from(path));
        }
        app
    }
    fn empty() -> Self {
        let session_autosave = recovery_directory().map(|dir| {
            dir.join(format!(
                "Untitled-{}-{}.weftforge.autosave",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ))
        });
        let mut app = Self {
            editor: Editor::new(Project::default()).expect("valid empty project"),
            mesh: None,
            cloth: None,
            job: None,
            needs_mesh: false,
            pending_batch: None,
            last_stage: "Rest arrangement",
            draft: None,
            playing: false,
            selected: None,
            tool: Tool::Select,
            polygon: vec![],
            drag_start: None,
            drag_point: None,
            preview: None,
            seam_start: None,
            reversed: true,
            intervals: [[0., 1.]; 2],
            name: "Pattern 1".into(),
            size: [0.5, 0.6],
            coordinate: [-0.25, 1.0],
            path: None,
            message: "Empty project · draw a pattern or enter dimensions in metres.".into(),
            center: [0., 1.],
            scale: 260.,
            yaw: -0.45,
            pitch: 0.12,
            camera_scale: 270.,
            wireframe: false,
            settings: StepSettings::default(),
            autosave: Instant::now(),
            session_autosave,
            autosave_job: None,
            show_recovery: false,
            recovery_files: Vec::new(),
            dirty: false,
            search: false,
            query: String::new(),
        };
        app.refresh_recovery();
        app
    }
    fn execute(&mut self, command: Command) {
        if self.commit_draft() {
            self.apply_command(command);
        }
    }
    fn apply_command(&mut self, command: Command) -> bool {
        match self.editor.execute(command) {
            Ok(()) => {
                self.dirty = true;
                self.message = "Construction updated; simulation invalidated.".into();
                self.rebuild();
                true
            }
            Err(error) => {
                self.message = error.to_string();
                false
            }
        }
    }
    fn commit_draft(&mut self) -> bool {
        let Some(draft) = self.draft.take() else {
            return true;
        };
        if draft.revision != self.editor.project.revision {
            return true;
        }
        let commands = draft.commands(&self.editor.project);
        commands.is_empty() || self.apply_command(Command::Transaction(commands))
    }
    fn select(&mut self, selected: Option<Id>) {
        if self.selected != selected && self.commit_draft() {
            self.selected = selected;
        }
    }
    fn cancel(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Relaxed);
        }
    }
    fn reset_simulation(&mut self) {
        if self
            .job
            .as_ref()
            .is_some_and(|job| job.kind == JobKind::Simulation)
        {
            self.cancel();
        }
        self.playing = false;
        self.pending_batch = None;
        self.last_stage = "Rest arrangement";
        self.cloth = None;
    }
    fn rebuild(&mut self) {
        self.playing = false;
        self.pending_batch = None;
        self.cancel();
        self.draft = None;
        self.needs_mesh = !self.editor.project.pieces.is_empty();
        if !self.needs_mesh {
            self.mesh = None;
            self.cloth = None;
        }
        if self
            .selected
            .is_some_and(|id| self.editor.project.piece(id).is_none())
        {
            self.selected = None;
        }
        if self
            .seam_start
            .is_some_and(|id| self.editor.project.curve(id).is_none())
        {
            self.seam_start = None;
        }
    }
    fn start_mesh(&mut self, ctx: &egui::Context) {
        if self.job.is_some() || !self.needs_mesh {
            return;
        }
        let project = self.editor.project.clone();
        self.needs_mesh = false;
        self.job = Some(Job::spawn(
            ctx,
            project.revision,
            JobKind::Mesh,
            move |cancel| {
                let mesh = project
                    .mesh_cancellable(|| cancel.load(Ordering::Relaxed))
                    .map_err(|error| error.to_string())?;
                Ok(JobOutput::Mesh(Arc::new(mesh)))
            },
        ));
    }
    fn queue_batch(&mut self, batch: SimulationBatch) {
        self.playing = false;
        self.settings = batch.settings;
        self.pending_batch = Some(batch);
        self.message = format!(
            "Queued {} · {} steps · gravity {:?} m/s²",
            batch.stage, batch.steps, batch.settings.gravity
        );
    }
    fn start_batch(&mut self, ctx: &egui::Context, batch: SimulationBatch) {
        if self.job.is_some() {
            return;
        }
        let Some(mesh) = self
            .mesh
            .as_ref()
            .filter(|mesh| mesh.doc_revision == self.editor.project.revision)
            .cloned()
        else {
            if self.editor.project.pieces.is_empty() {
                self.message = "Create a pattern first.".into();
                self.playing = false;
                self.pending_batch = None;
            } else {
                self.needs_mesh = true;
            }
            return;
        };
        let project = self.editor.project.clone();
        let cloth = self.cloth.clone();
        let settings = batch.settings;
        self.pending_batch = None;
        let mut job = Job::spawn(ctx, project.revision, JobKind::Simulation, move |cancel| {
            let mut state = match cloth {
                Some(state) => (*state).clone(),
                None => weftforge_core::simulation::create_cloth(&project, &mesh)
                    .map_err(|error| error.to_string())?,
            };
            check_cancelled(cancel)?;
            for _ in 0..batch.steps {
                state
                    .step(settings, || cancel.load(Ordering::Relaxed))
                    .map_err(|e| e.to_string())?;
            }
            Ok(JobOutput::Simulation(Arc::new(state)))
        });
        job.batch = Some(batch);
        self.job = Some(job);
    }
    fn poll(&mut self) {
        let result = self
            .job
            .as_ref()
            .and_then(|job| match job.receiver.try_recv() {
                Ok(r) => Some(r),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(_) => Some(Err("Worker stopped; previous state retained.".into())),
            });
        if let Some(result) = result {
            let job = self.job.take().expect("finished job");
            if !job.accepts(self.editor.project.revision) {
                return;
            }
            match result {
                Ok(JobOutput::Mesh(mesh)) => {
                    self.mesh = Some(mesh);
                    self.cloth = None;
                    self.message = "Rest mesh rebuilt from editable construction.".into();
                }
                Ok(JobOutput::Simulation(state)) => {
                    self.last_stage = job.batch.map_or("Manual simulation", |batch| batch.stage);
                    self.message = format!(
                        "{} complete · {:.3} s · {:.1} ms/step · assembly remaining {:.4} m",
                        self.last_stage,
                        state.simulation_time(),
                        state.diagnostics().step_seconds * 1000.,
                        state.diagnostics().seam_assembly_remaining,
                    );
                    self.cloth = Some(state);
                }
                Err(e) => {
                    self.message = e;
                    self.playing = false;
                    self.pending_batch = None;
                }
            }
        }
    }
    fn open(&mut self, path: PathBuf) {
        self.load_document(path, false);
    }
    fn load_document(&mut self, path: PathBuf, recovered: bool) {
        match Project::load(&path).and_then(Editor::new) {
            Ok(editor) => {
                self.editor = editor;
                self.path = if recovered { None } else { Some(path) };
                self.dirty = recovered;
                self.selected = None;
                self.polygon.clear();
                self.mesh = None;
                self.cloth = None;
                self.rebuild();
                self.autosave = Instant::now();
                self.message = if recovered {
                    "Recovered editable construction; use Save as to keep it."
                } else {
                    "Opened editable construction."
                }
                .into();
            }
            Err(e) => self.message = format!("Open failed; current document retained: {e}"),
        }
    }
    fn choose_open(&mut self) {
        if !self.commit_draft() {
            return;
        }
        if self.dirty {
            self.message = "Save current edits before opening another project.".into();
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("WeftForge", &["weftforge"])
            .pick_file()
        {
            self.open(path);
        }
    }
    fn save(&mut self, choose: bool) {
        if !self.commit_draft() {
            return;
        }
        let path = if choose || self.path.is_none() {
            rfd::FileDialog::new()
                .add_filter("WeftForge", &["weftforge"])
                .set_file_name("garment.weftforge")
                .save_file()
        } else {
            self.path.clone()
        };
        if let Some(path) = path {
            match self.editor.project.save(&path) {
                Ok(()) => {
                    self.path = Some(path);
                    self.dirty = false;
                    self.message = "Saved editable project atomically.".into();
                    self.autosave = Instant::now();
                }
                Err(e) => self.message = e.to_string(),
            }
        }
    }
    fn export(&mut self, svg: bool) {
        if !self.commit_draft() {
            return;
        }
        let ext = if svg { "svg" } else { "obj" };
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(ext, &[ext])
            .set_file_name(format!("garment.{ext}"))
            .save_file()
        {
            if let Err(error) = export_path_guard(self.path.as_deref(), &path) {
                self.message = error;
                return;
            }
            let result = if svg {
                self.editor.project.export_svg(&path)
            } else if let Some(mesh) = &self.mesh {
                if mesh.doc_revision != self.editor.project.revision {
                    self.message = "The rest mesh is rebuilding; export when it is ready.".into();
                    return;
                }
                let mut output = (**mesh).clone();
                if let Some(state) = &self.cloth {
                    output.positions = state.positions().to_vec();
                }
                output.export_obj(&path)
            } else {
                self.message = "Create a pattern first.".into();
                return;
            };
            self.message = match result {
                Ok(()) => format!("Exported {ext}; OBJ metres, SVG millimetres."),
                Err(e) => e.to_string(),
            };
        }
    }
    fn history(&mut self, redo: bool) {
        if !self.commit_draft() {
            return;
        }
        if if redo {
            self.editor.redo()
        } else {
            self.editor.undo()
        } {
            self.dirty = true;
            self.rebuild();
        }
    }
    fn refresh_recovery(&mut self) {
        self.recovery_files.clear();
        if let Some(path) = self.path.clone() {
            self.add_project_recovery(&path);
        }
        if let Some(dir) = recovery_directory()
            && let Ok(entries) = fs::read_dir(dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().ends_with(".autosave"))
                {
                    self.recovery_files.push(path);
                }
            }
        }
        self.recovery_files.sort();
        self.recovery_files.dedup();
    }
    fn add_project_recovery(&mut self, path: &Path) {
        for path in [
            append_suffix(path, ".autosave"),
            weftforge_core::recovery_path(path),
        ] {
            if path.is_file() && !self.recovery_files.contains(&path) {
                self.recovery_files.push(path);
            }
        }
    }
    fn recover(&mut self, path: PathBuf) {
        if !self.commit_draft() {
            return;
        }
        if self.dirty {
            self.message = "Save current edits before recovering another document.".into();
            return;
        }
        self.load_document(path, true);
    }
    fn recovery_window(&mut self, ctx: &egui::Context) {
        if !self.show_recovery {
            return;
        }
        let mut open = true;
        egui::Window::new("Recover editable construction").open(&mut open).default_width(520.0).show(ctx,|ui| {
            ui.label("Recovery opens a copy. Save current edits before replacing the document.");
            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() { self.refresh_recovery(); }
                if ui.button("Choose project…").clicked() && let Some(path)=rfd::FileDialog::new().add_filter("WeftForge",&["weftforge"]).pick_file() { self.add_project_recovery(&path); }
                if ui.button("Choose recovery file…").clicked() && let Some(path)=rfd::FileDialog::new().add_filter("Recovery",&["autosave","recovery"]).pick_file() && !self.recovery_files.contains(&path) { self.recovery_files.push(path); }
            });
            if self.recovery_files.is_empty() { ui.label("No recovery files found. Untitled autosaves appear here; saved projects also retain .autosave and .recovery sidecars."); }
            egui::ScrollArea::vertical().max_height(300.0).show(ui,|ui| {
                for path in self.recovery_files.clone() {
                    ui.horizontal(|ui| {
                        ui.label(path.file_name().unwrap_or_default().to_string_lossy()).on_hover_text(path.display().to_string());
                        if ui.button("Recover copy").clicked() { self.recover(path); }
                    });
                }
            });
        });
        self.show_recovery = open;
    }
    fn start_autosave(&mut self, ctx: &egui::Context) {
        if self.autosave_job.is_some() {
            return;
        }
        self.autosave = Instant::now();
        let path = self
            .path
            .as_ref()
            .map(|path| append_suffix(path, ".autosave"))
            .or_else(|| self.session_autosave.clone());
        let Some(path) = path else {
            self.message="Autosave unavailable: the operating system data directory is not configured. Save the project explicitly.".into();
            return;
        };
        let project = self.editor.project.clone();
        let ctx = ctx.clone();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let result = (|| {
                if let Some(parent) = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                {
                    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                }
                project.save(&path).map_err(|error| error.to_string())
            })();
            let _ = sender.send(result);
            ctx.request_repaint();
        });
        self.autosave_job = Some(AutosaveJob { receiver });
    }
    fn poll_autosave(&mut self) {
        let result = self
            .autosave_job
            .as_ref()
            .and_then(|job| match job.receiver.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(_) => Some(Err("Autosave worker stopped.".into())),
            });
        if let Some(result) = result {
            self.autosave_job = None;
            if let Err(error) = result {
                self.message = format!("Autosave failed; current document retained: {error}");
            }
        }
    }
    fn finish_polygon(&mut self) {
        if self.polygon.len() < 3 {
            return;
        }
        let revision = self.editor.project.revision;
        self.execute(Command::AddPolygon {
            name: self.name.clone(),
            vertices: self.polygon.clone(),
        });
        if revision != self.editor.project.revision {
            self.polygon.clear();
            self.selected = self.editor.project.pieces.last().map(|p| p.id);
        }
    }
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.heading("WeftForge");
            ui.colored_label(Color32::from_rgb(110, 220, 190), "Open garment design.");
            ui.separator();
            if ui.button("Open…").clicked() {
                self.choose_open();
            }
            if ui.button("Save").clicked() {
                self.save(false);
            }
            if ui.button("Save as…").clicked() {
                self.save(true);
            }
            if ui.button("Recover…").clicked() {
                self.refresh_recovery();
                self.show_recovery = true;
            }
            if ui.button("Export OBJ").clicked() {
                self.export(false);
            }
            if ui.button("Export SVG").clicked() {
                self.export(true);
            }
            if ui.button("Undo").clicked() {
                self.history(false);
            }
            if ui.button("Redo").clicked() {
                self.history(true);
            }
            if ui.button("Commands ⌘K").clicked() {
                self.search = true;
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (tool, label) in [
                (Tool::Select, "Select / edit"),
                (Tool::Polygon, "Polygon"),
                (Tool::Rectangle, "Rectangle"),
                (Tool::Sew, "Sew edges"),
            ] {
                ui.selectable_value(&mut self.tool, tool, label);
            }
            ui.separator();
            if ui
                .button(if self.playing { "Pause" } else { "Play" })
                .clicked()
                && self.commit_draft()
            {
                self.playing = !self.playing;
                self.pending_batch = None;
                if !self.playing
                    && self
                        .job
                        .as_ref()
                        .is_some_and(|job| job.kind == JobKind::Simulation)
                {
                    self.cancel();
                }
            }
            if ui.button("Step").clicked() && self.commit_draft() {
                self.playing = false;
                if self
                    .job
                    .as_ref()
                    .is_some_and(|job| job.kind == JobKind::Simulation)
                {
                    self.cancel();
                }
                self.queue_batch(SimulationBatch {
                    steps: 1,
                    settings: self.settings,
                    stage: "Manual step",
                });
                ui.ctx().request_repaint();
            }
            if ui.button("Assemble (gravity off)").clicked() && self.commit_draft() {
                self.queue_batch(SimulationBatch::assemble());
                ui.ctx().request_repaint();
            }
            if ui.button("Drape").clicked() && self.commit_draft() {
                self.queue_batch(SimulationBatch::drape());
                ui.ctx().request_repaint();
            }
            if ui.button("Reset").clicked() && self.commit_draft() {
                self.reset_simulation();
            }
            if self.job.is_some() && ui.button("Cancel job").clicked() {
                self.cancel();
                self.playing = false;
                self.pending_batch = None;
                self.message = "Cancelled phase; previous accepted snapshot retained.".into();
            }
            ui.checkbox(&mut self.wireframe, "Wireframe");
            ui.label(format!(
                "{} pieces · {} seams · {} vertices",
                self.editor.project.pieces.len(),
                self.editor.project.seams.len(),
                self.mesh.as_ref().map_or(0, |m| m.positions.len())
            ));
        });
        ui.horizontal_wrapped(|ui| {
            let active=self.job.as_ref().and_then(|job|job.batch);
            let settings=active.map_or(self.settings,|batch|batch.settings);
            ui.label(format!("Stage: {}{}",active.map_or(self.last_stage,|batch|batch.stage),active.map_or(String::new(),|batch|format!(" · {} steps",batch.steps))));
            ui.label(format!("{}: dt {:.5} s · {} substeps · {} iterations · gravity {:?} m/s² · seam closure {:.2} m/s",if active.is_some() { "Active settings" } else { "Next step settings" },settings.dt,settings.substeps,settings.iterations,settings.gravity,settings.seam_closure_speed));
            if let Some(batch)=self.pending_batch { ui.colored_label(Color32::GOLD,format!("Queued: {} · {} steps",batch.stage,batch.steps)); }
        });
    }
    fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Construction");
        let mut commit = false;
        egui::ScrollArea::vertical().id_salt("inspector").show(ui,|ui| {
            for piece in self.editor.project.pieces.clone() {
                if ui.selectable_label(self.selected==Some(piece.id),&piece.name).clicked() { self.select(Some(piece.id)); }
            }
            ui.separator(); ui.label("Pattern name"); ui.text_edit_singleline(&mut self.name);
            ui.horizontal(|ui| { ui.label("W / H m"); for value in &mut self.size { ui.add(egui::DragValue::new(value).speed(0.01).range(0.01..=5.0)); } });
            ui.horizontal(|ui| { ui.label("X / Y m"); for value in &mut self.coordinate { ui.add(egui::DragValue::new(value).speed(0.01)); } });
            if ui.button("Add rectangle").clicked() {
                self.execute(Command::AddRectangle { name:self.name.clone(),origin:self.coordinate,width:self.size[0],height:self.size[1] });
                self.select(self.editor.project.pieces.last().map(|piece|piece.id));
            }
            if ui.button("Add polygon point").clicked() { self.polygon.push(self.coordinate); self.tool=Tool::Polygon; }
            if ui.button(format!("Close pattern ({} points)",self.polygon.len())).clicked() { self.finish_polygon(); }
            let mut draft=self.draft.take().filter(|draft|draft.revision==self.editor.project.revision && draft.selected==self.selected)
                .unwrap_or_else(||InspectorDraft::new(&self.editor.project,self.selected));
            if let Some(piece)=self.selected.and_then(|id|self.editor.project.piece(id)).cloned() {
                ui.separator(); ui.strong(&piece.name);
                if let Some(dimensions)=&mut draft.dimensions {
                    ui.horizontal(|ui| { ui.label("Size m"); for value in dimensions { commit|=numeric_commit(&ui.add(egui::DragValue::new(value).speed(0.005).range(0.01..=5.0).clamp_existing_to_range(false))); } });
                }
                if let Some(placement)=&mut draft.placement {
                    ui.label("Arrange X / Y / Z m");
                    ui.horizontal(|ui| { for value in &mut placement.translation { commit|=numeric_commit(&ui.add(egui::DragValue::new(value).speed(0.01))); } });
                    ui.label("Rotate X / Y / Z radians");
                    ui.horizontal(|ui| { for value in &mut placement.rotation { commit|=numeric_commit(&ui.add(egui::DragValue::new(value).speed(0.02))); } });
                }
                if let Some(angle)=&mut draft.grain_angle {
                    ui.label("Warp angle radians");
                    ui.small("0 = +U (right); positive turns toward +V (up).");
                    commit|=numeric_commit(&ui.add(egui::DragValue::new(angle).speed(0.02)));
                }
                if let Some(material)=&mut draft.material {
                    let count=self.editor.project.pieces.iter().filter(|piece|piece.material==material.id).count();
                    ui.label(format!("Shared fabric · {count} pieces")); ui.small("Changes affect every assigned piece. Uncalibrated approximation.");
                    ui.label("Mass/area kg/m²"); commit|=numeric_commit(&ui.add(egui::DragValue::new(&mut material.areal_density).speed(0.01).range(0.001..=100.0).clamp_existing_to_range(false)));
                    ui.label("Warp stiffness N/m"); commit|=numeric_commit(&ui.add(egui::DragValue::new(&mut material.warp_stiffness).speed(10.0).range(0.0..=1.0e8).clamp_existing_to_range(false)));
                    ui.label("Weft stiffness N/m"); commit|=numeric_commit(&ui.add(egui::DragValue::new(&mut material.weft_stiffness).speed(10.0).range(0.0..=1.0e8).clamp_existing_to_range(false)));
                    ui.label("Bend N·m"); commit|=numeric_commit(&ui.add(egui::DragValue::new(&mut material.bending_stiffness).speed(0.00001).range(0.0..=1.0e8).clamp_existing_to_range(false)));
                    draft.color_pending|=ui.color_edit_button_rgb(&mut material.color).changed();
                    commit|=draft.color_pending && ui.input(|input|input.pointer.any_released());
                }
                if ui.button("Delete selected").clicked() { self.draft=Some(draft); self.execute(Command::RemovePiece { piece:piece.id }); return; }
            }
            ui.separator(); ui.strong("Sewing"); ui.checkbox(&mut self.reversed,"Reverse paired edge"); ui.label("A interval / B interval (0–1)");
            for range in &mut self.intervals { ui.horizontal(|ui| { for value in range { ui.add(egui::DragValue::new(value).speed(0.01).range(0.0..=1.0)); } }); }
            ui.label(if self.seam_start.is_some() { "Click paired boundary in 2D." } else { "Sew edges: click two 2D boundaries." });
            let mut remove=None;
            for seam in &self.editor.project.seams { ui.horizontal(|ui| { ui.label(&seam.name); if ui.small_button("Remove").clicked() { remove=Some(seam.id); } }); }
            ui.separator(); ui.label("Mesh edge length m"); commit|=numeric_commit(&ui.add(egui::DragValue::new(&mut draft.resolution).speed(0.005).range(0.005..=0.5)));
            ui.label("Iterations / substeps"); ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.settings.iterations).range(1..=50));
                ui.add(egui::DragValue::new(&mut self.settings.substeps).range(1..=16));
            });
            if let Some(cloth)=&self.cloth { let diagnostics=cloth.diagnostics(); ui.label(format!("Time {:.3} s\nAssembly remaining {:.4} m\nStretch error {:.3}\nSeam gap {:.4} m\nContact error {:.4} m\nInitial intersections {}",cloth.simulation_time(),diagnostics.seam_assembly_remaining,diagnostics.max_stretch_error,diagnostics.max_seam_error,diagnostics.max_contact_error,diagnostics.initial_intersections)); }
            self.draft=Some(draft);
            if let Some(seam)=remove { self.execute(Command::RemoveSeam { seam }); }
        });
        if commit {
            self.commit_draft();
        }
    }
    fn to_screen(&self, p: [f64; 2], r: Rect) -> Pos2 {
        r.center()
            + Vec2::new(
                (p[0] - self.center[0]) as f32,
                -(p[1] - self.center[1]) as f32,
            ) * self.scale
    }
    fn to_pattern(&self, p: Pos2, r: Rect) -> [f64; 2] {
        let d = (p - r.center()) / self.scale;
        [
            snap(self.center[0] + d.x as f64),
            snap(self.center[1] - d.y as f64),
        ]
    }
    fn patterns(&mut self, ui: &mut egui::Ui) {
        ui.strong("2D PATTERNS");
        ui.small("Metres · 1 cm snap · scroll zoom · middle drag pan");
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let r = response.rect;
        painter.rect_filled(r, 3., Color32::from_rgb(28, 35, 42));
        if response.hovered() {
            self.scale = (self.scale * (ui.input(|i| i.smooth_scroll_delta.y) * 0.001).exp())
                .clamp(40., 3000.);
        }
        if response.dragged_by(egui::PointerButton::Middle) {
            let d = ui.input(|i| i.pointer.delta()) / self.scale;
            self.center[0] -= d.x as f64;
            self.center[1] += d.y as f64;
        }
        for i in -30..=30 {
            let p = i as f64 * 0.1;
            for (a, b) in [([p, -3.], [p, 4.]), ([-3., p], [3., p])] {
                painter.line_segment(
                    [self.to_screen(a, r), self.to_screen(b, r)],
                    Stroke::new(0.5, Color32::from_gray(48)),
                );
            }
        }
        let mut doc = self.editor.project.clone();
        if let (Some(id), Some(position)) = (self.drag_point, self.preview)
            && let Some(point) = doc.points.iter_mut().find(|p| p.id == id)
        {
            point.position = position;
        }
        let pointer = response
            .interact_pointer_pos()
            .or_else(|| ui.input(|i| i.pointer.hover_pos()));
        let mut nearest = None;
        let mut distance = 12_f32;
        for piece in &doc.pieces {
            let selected = self.selected == Some(piece.id);
            let color = if selected {
                Color32::from_rgb(115, 231, 202)
            } else {
                Color32::from_rgb(165, 180, 198)
            };
            for id in &piece.boundary {
                if let Some(curve) = doc.curve(*id) {
                    let samples: Vec<_> = (0..=24)
                        .filter_map(|i| curve.evaluate(&doc, i as f64 / 24.).ok())
                        .map(|p| self.to_screen(p, r))
                        .collect();
                    if let Some(p) = pointer {
                        for s in samples.windows(2) {
                            let d = segment_distance(p, s[0], s[1]);
                            if d < distance {
                                distance = d;
                                nearest = Some((piece.id, *id));
                            }
                        }
                    }
                    painter.add(egui::Shape::line(
                        samples,
                        Stroke::new(if selected { 2.5 } else { 1.5 }, color),
                    ));
                    if let Some(point) = doc.point(curve.start) {
                        painter.circle_filled(self.to_screen(point.position, r), 3.5, color);
                    }
                }
            }
            if let Ok([min, max]) = doc.piece_bounds(piece.id) {
                let center = std::array::from_fn(|axis| (min[axis] + max[axis]) * 0.5);
                let length =
                    ((max[0] - min[0]).min(max[1] - min[1]) * 0.25).min(0.1) as f32 * self.scale;
                let direction = Vec2::new(
                    piece.grain_angle.cos() as f32,
                    -piece.grain_angle.sin() as f32,
                ) * length;
                let origin = self.to_screen(center, r);
                painter.arrow(origin, direction, Stroke::new(1.5, color));
                painter.text(
                    origin + Vec2::new(0., 9.),
                    egui::Align2::CENTER_TOP,
                    "Warp",
                    egui::FontId::proportional(10.),
                    color,
                );
            }
            if let Some(curve) = piece.boundary.first().and_then(|id| doc.curve(*id))
                && let Ok(p) = curve.evaluate(&doc, 0.)
            {
                painter.text(
                    self.to_screen(p, r) + Vec2::new(7., -12.),
                    egui::Align2::LEFT_CENTER,
                    &piece.name,
                    egui::FontId::proportional(12.),
                    color,
                );
            }
        }
        for seam in &doc.seams {
            let endpoint = |i: &CurveInterval| {
                doc.curve(i.curve)
                    .and_then(|c| c.evaluate(&doc, (i.t0 + i.t1) * 0.5).ok())
                    .map(|p| self.to_screen(p, r))
            };
            if let (Some(a), Some(b)) = (endpoint(&seam.a), endpoint(&seam.b)) {
                painter.line_segment([a, b], Stroke::new(1., Color32::from_rgb(245, 175, 87)));
            }
        }
        for point in &self.polygon {
            painter.circle_filled(self.to_screen(*point, r), 3., Color32::GOLD);
        }
        if self.polygon.len() > 1 {
            painter.add(egui::Shape::line(
                self.polygon.iter().map(|p| self.to_screen(*p, r)).collect(),
                Stroke::new(2., Color32::GOLD),
            ));
        }
        if let (Some(start), Some(pointer)) = (self.drag_start, pointer)
            && self.tool == Tool::Rectangle
        {
            painter.rect_stroke(
                Rect::from_two_pos(self.to_screen(start, r), pointer),
                0.,
                Stroke::new(1., Color32::GOLD),
                egui::StrokeKind::Inside,
            );
        }
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(p) = pointer
        {
            if self.tool == Tool::Rectangle {
                self.drag_start = Some(self.to_pattern(p, r));
            }
            if self.tool == Tool::Select {
                self.drag_point = doc
                    .points
                    .iter()
                    .filter_map(|point| {
                        let d = self.to_screen(point.position, r).distance(p);
                        (d < 12.).then_some((point.id, d))
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|p| p.0);
            }
        }
        if response.dragged_by(egui::PointerButton::Primary) && self.drag_point.is_some() {
            self.preview = pointer.map(|p| self.to_pattern(p, r));
        }
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            if let (Some(point), Some(position)) = (self.drag_point.take(), self.preview.take()) {
                self.execute(Command::MovePoint { point, position });
            }
            if let (Some(start), Some(p)) = (self.drag_start.take(), pointer) {
                let end = self.to_pattern(p, r);
                self.execute(Command::AddRectangle {
                    name: self.name.clone(),
                    origin: [start[0].min(end[0]), start[1].min(end[1])],
                    width: (end[0] - start[0]).abs(),
                    height: (end[1] - start[1]).abs(),
                });
                self.select(self.editor.project.pieces.last().map(|p| p.id));
            }
        }
        if response.clicked() {
            match self.tool {
                Tool::Select => self.select(nearest.map(|p| p.0)),
                Tool::Polygon => {
                    if let Some(p) = pointer {
                        self.polygon.push(self.to_pattern(p, r));
                    }
                }
                Tool::Sew => {
                    if let Some((_, curve)) = nearest {
                        if let Some(a) = self.seam_start.take() {
                            let b = if self.reversed {
                                [self.intervals[1][1], self.intervals[1][0]]
                            } else {
                                self.intervals[1]
                            };
                            self.execute(Command::AddSeam {
                                name: format!("Seam {}", doc.seams.len() + 1),
                                a: CurveInterval {
                                    curve: a,
                                    t0: self.intervals[0][0],
                                    t1: self.intervals[0][1],
                                },
                                b: CurveInterval {
                                    curve,
                                    t0: b[0],
                                    t1: b[1],
                                },
                                correspondence: vec![[0., 0.], [1., 1.]],
                            });
                        } else {
                            self.seam_start = Some(curve);
                        }
                    }
                }
                Tool::Rectangle => {}
            }
        }
    }
    fn project3(&self, p: [f64; 3], r: Rect) -> (Pos2, f32) {
        let [x, y, z] = [p[0] as f32, p[1] as f32 - 1., p[2] as f32];
        let a = self.yaw.cos() * x + self.yaw.sin() * z;
        let z = -self.yaw.sin() * x + self.yaw.cos() * z;
        let b = self.pitch.cos() * y - self.pitch.sin() * z;
        (
            r.center() + Vec2::new(a, -b) * self.camera_scale,
            self.pitch.sin() * y + self.pitch.cos() * z,
        )
    }
    fn garment(&mut self, ui: &mut egui::Ui) {
        ui.strong("3D GARMENT");
        ui.small("Drag orbit · scroll zoom · triangle selection");
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let r = response.rect;
        painter.rect_filled(r, 3., Color32::from_rgb(21, 27, 34));
        if response.dragged() {
            let d = ui.input(|i| i.pointer.delta());
            self.yaw += d.x * 0.008;
            self.pitch = (self.pitch + d.y * 0.008).clamp(-1.4, 1.4);
        }
        if response.hovered() {
            self.camera_scale = (self.camera_scale
                * (ui.input(|i| i.smooth_scroll_delta.y) * 0.001).exp())
            .clamp(60., 1500.);
        }
        for i in -10..=10 {
            let p = i as f64 * 0.1;
            for (a, b) in [([-1., 0., p], [1., 0., p]), ([p, 0., -1.], [p, 0., 1.])] {
                painter.line_segment(
                    [self.project3(a, r).0, self.project3(b, r).0],
                    Stroke::new(0.5, Color32::from_gray(45)),
                );
            }
        }
        for capsule in &self.editor.project.mannequin.capsules {
            let a = self.project3(capsule.a, r).0;
            let b = self.project3(capsule.b, r).0;
            let radius = capsule.radius as f32 * self.camera_scale;
            let c = Color32::from_rgb(75, 85, 96);
            painter.line_segment([a, b], Stroke::new(radius * 2., c));
            painter.circle_filled(a, radius, c);
            painter.circle_filled(b, radius, c);
        }
        if let Some(mesh) = &self.mesh {
            let positions = self
                .cloth
                .as_ref()
                .map_or(mesh.positions.as_slice(), |s| s.positions());
            let mut triangles: Vec<_> = mesh
                .triangles
                .iter()
                .map(|t| {
                    let p = t.map(|i| self.project3(positions[i], r));
                    (t, p, p.iter().map(|v| v.1).sum::<f32>() / 3.)
                })
                .collect();
            triangles.sort_by(|a, b| a.2.total_cmp(&b.2));
            let mut picked = None;
            for (t, p, _) in triangles {
                let id = mesh.piece_ids[t[0]];
                let rgb = self
                    .editor
                    .project
                    .piece(id)
                    .and_then(|p| self.editor.project.material(p.material))
                    .map_or([0.3, 0.7, 0.65], |m| m.color);
                let light = triangle_light(t.map(|i| positions[i]));
                let color = Color32::from_rgb(
                    (rgb[0] * light * 255.) as u8,
                    (rgb[1] * light * 255.) as u8,
                    (rgb[2] * light * 255.) as u8,
                );
                let points = p.map(|p| p.0);
                let stroke = if self.wireframe || self.selected == Some(id) {
                    Stroke::new(0.5, Color32::from_rgb(111, 199, 181))
                } else {
                    Stroke::NONE
                };
                painter.add(egui::Shape::convex_polygon(points.to_vec(), color, stroke));
                if response.clicked()
                    && response
                        .interact_pointer_pos()
                        .is_some_and(|p| point_in_triangle(p, points))
                {
                    picked = Some(id);
                }
            }
            if picked.is_some() {
                self.select(picked);
            }
        }
        painter.text(
            r.left_top() + Vec2::splat(12.),
            egui::Align2::LEFT_TOP,
            if self
                .mesh
                .as_ref()
                .is_some_and(|mesh| mesh.doc_revision != self.editor.project.revision)
            {
                "PREVIOUS REVISION · rebuilding rest mesh"
            } else if self
                .job
                .as_ref()
                .is_some_and(|job| job.kind == JobKind::Mesh)
            {
                "REBUILDING REST MESH · Y up · m"
            } else if self.playing {
                "SIMULATING · Y up · m"
            } else {
                "PAUSED · Y up · m"
            },
            egui::FontId::monospace(12.),
            Color32::from_rgb(140, 213, 194),
        );
    }
}
impl eframe::App for Studio {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll();
        self.poll_autosave();
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            self.save(false);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O)) {
            self.choose_open();
        }
        if !ctx.egui_wants_keyboard_input()
            && ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Z,
                )
            })
        {
            self.history(true);
        } else if !ctx.egui_wants_keyboard_input()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z))
        {
            self.history(false);
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::K)) {
            self.search = !self.search;
        }
        if !ctx.egui_wants_keyboard_input() {
            if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                self.playing = !self.playing;
                self.pending_batch = None;
                if !self.playing
                    && self
                        .job
                        .as_ref()
                        .is_some_and(|job| job.kind == JobKind::Simulation)
                {
                    self.cancel();
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.finish_polygon();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.polygon.clear();
                self.seam_start = None;
                self.drag_point = None;
                self.preview = None;
                self.drag_start = None;
            }
        }
        if self.needs_mesh && self.job.is_none() {
            self.start_mesh(&ctx);
        } else if let Some(batch) = self.pending_batch
            && self.job.is_none()
        {
            self.start_batch(&ctx, batch);
        } else if self.playing && self.job.is_none() {
            self.start_batch(
                &ctx,
                SimulationBatch {
                    steps: 2,
                    settings: self.settings,
                    stage: "Manual play",
                },
            );
        }
        if self.job.is_some() {
            ctx.request_repaint_after(Duration::from_millis(20));
        }
        if self.dirty {
            ctx.request_repaint_after(Duration::from_secs(1));
            if self.autosave.elapsed() > Duration::from_secs(30) {
                self.start_autosave(&ctx);
            }
        }
        if self.dirty && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.message = "Save before closing, or explicitly discard edits.".into();
        }
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            self.toolbar(ui);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(&self.message);
                if let Some(job) = &self.job {
                    ui.spinner();
                    ui.label(if job.cancel.load(Ordering::Relaxed) {
                        "Cancelling…"
                    } else if job.kind == JobKind::Mesh {
                        "Meshing…"
                    } else {
                        "Simulating…"
                    });
                }
                if self.autosave_job.is_some() {
                    ui.label("Autosaving…");
                }
                if self.dirty {
                    ui.colored_label(Color32::GOLD, "Unsaved");
                    if ui.small_button("Discard and close").clicked() {
                        self.dirty = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });
            ui.separator();
            let h = ui.available_height().max(100.);
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(235., h),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| self.inspector(ui),
                );
                ui.separator();
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), h),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.columns(2, |cols| {
                            self.patterns(&mut cols[0]);
                            self.garment(&mut cols[1]);
                        })
                    },
                );
            });
        });
        if self.search {
            let mut open = true;
            egui::Window::new("Command search")
                .open(&mut open)
                .collapsible(false)
                .show(&ctx, |ui| {
                    ui.text_edit_singleline(&mut self.query);
                    for label in [
                        "Save project",
                        "Open project",
                        "Recover project",
                        "Export OBJ",
                        "Export SVG",
                        "Undo",
                        "Redo",
                        "Reset simulation",
                        "Select",
                        "Polygon",
                        "Rectangle",
                        "Sew edges",
                    ] {
                        if label.to_lowercase().contains(&self.query.to_lowercase())
                            && ui.button(label).clicked()
                        {
                            match label {
                                "Save project" => self.save(false),
                                "Open project" => self.choose_open(),
                                "Recover project" => {
                                    self.refresh_recovery();
                                    self.show_recovery = true;
                                }
                                "Export OBJ" => self.export(false),
                                "Export SVG" => self.export(true),
                                "Undo" => self.history(false),
                                "Redo" => self.history(true),
                                "Reset simulation" => {
                                    self.reset_simulation();
                                }
                                "Select" => self.tool = Tool::Select,
                                "Polygon" => self.tool = Tool::Polygon,
                                "Rectangle" => self.tool = Tool::Rectangle,
                                "Sew edges" => self.tool = Tool::Sew,
                                _ => {}
                            }
                            self.search = false;
                        }
                    }
                });
            self.search &= open;
        }
        self.recovery_window(&ctx);
    }
}
fn numeric_commit(response: &egui::Response) -> bool {
    response.drag_stopped()
        || response.lost_focus()
        || (response.changed() && !response.is_pointer_button_down_on() && !response.has_focus())
}
fn check_cancelled(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) {
        Err("Operation cancelled; previous snapshot retained.".into())
    } else {
        Ok(())
    }
}
fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}
fn recovery_directory() -> Option<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME")
            .map(|value| PathBuf::from(value).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| {
                std::env::var_os("HOME").map(|value| PathBuf::from(value).join(".local/share"))
            })
    };
    base.filter(|path| path.is_absolute())
        .map(|path| path.join("WeftForge/Recovery"))
}
fn export_path_guard(project: Option<&Path>, output: &Path) -> Result<(), String> {
    if output
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("weftforge"))
    {
        return Err("Choose an OBJ or SVG output; project files are preserved.".into());
    }
    let Some(project) = project else {
        return Ok(());
    };
    let canonical = |path: &Path| -> std::io::Result<PathBuf> {
        if path.exists() {
            path.canonicalize()
        } else {
            let parent = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            Ok(parent
                .canonicalize()?
                .join(path.file_name().unwrap_or_default()))
        }
    };
    if canonical(project).map_err(|error| error.to_string())?
        == canonical(output).map_err(|error| error.to_string())?
    {
        Err("Export cannot replace the current project. Choose another output file.".into())
    } else {
        Ok(())
    }
}
fn snap(v: f64) -> f64 {
    (v * 100.).round() / 100.
}
fn segment_distance(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_sq().max(1e-9)).clamp(0., 1.);
    p.distance(a + ab * t)
}
fn point_in_triangle(p: Pos2, t: [Pos2; 3]) -> bool {
    let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
    let area = cross(t[1] - t[0], t[2] - t[0]);
    if area.abs() < 1e-6 {
        return false;
    }
    let a = cross(t[1] - p, t[2] - p) / area;
    let b = cross(t[2] - p, t[0] - p) / area;
    a >= 0. && b >= 0. && a + b <= 1.
}
fn triangle_light(t: [[f64; 3]; 3]) -> f32 {
    let a: [f64; 3] = std::array::from_fn(|i| t[1][i] - t[0][i]);
    let b: [f64; 3] = std::array::from_fn(|i| t[2][i] - t[0][i]);
    let n = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let len = n.iter().map(|v| v * v).sum::<f64>().sqrt().max(1e-12);
    (0.45 + 0.55 * ((n[0] * 0.3 + n[1] * 0.5 + n[2] * 0.8) / len).abs().min(1.)) as f32
}
fn main() -> eframe::Result {
    eframe::run_native(
        "WeftForge",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1450., 900.])
                .with_min_inner_size([1000., 650.]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Studio::new(cc)))),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canvas_picking() {
        assert_eq!(snap(0.214), 0.21);
        let t = [Pos2::ZERO, Pos2::new(1., 0.), Pos2::new(0., 1.)];
        assert!(point_in_triangle(Pos2::new(0.2, 0.2), t));
        assert!(!point_in_triangle(Pos2::new(2., 2.), t));
        assert!(!point_in_triangle(Pos2::ZERO, [Pos2::ZERO; 3]));
        assert!(
            (segment_distance(Pos2::new(0.5, 1.), Pos2::ZERO, Pos2::new(1., 0.)) - 1.).abs() < 1e-6
        );
    }

    #[test]
    fn inspector_updates_commit_once_and_undo_as_one_transaction() {
        let mut app = Studio::empty();
        app.editor
            .execute(Command::AddRectangle {
                name: "Panel".into(),
                origin: [0.0, 0.0],
                width: 0.4,
                height: 0.6,
            })
            .unwrap();
        let piece = app.editor.project.pieces[0].id;
        app.selected = Some(piece);
        let mut draft = InspectorDraft::new(&app.editor.project, app.selected);
        for width in [0.41, 0.45, 0.5] {
            draft.dimensions.as_mut().unwrap()[0] = width;
        }
        draft.placement.as_mut().unwrap().translation[2] = 0.25;
        draft.material.as_mut().unwrap().areal_density = 0.3;
        draft.material.as_mut().unwrap().warp_stiffness = 50.;
        draft.material.as_mut().unwrap().weft_stiffness = 1000.;
        draft.grain_angle = Some(std::f64::consts::FRAC_PI_2);
        assert_eq!(app.editor.project.revision, 1);
        assert_eq!(app.editor.project.piece_bounds(piece).unwrap()[1][0], 0.4);
        app.draft = Some(draft);
        assert!(app.commit_draft());
        assert_eq!(app.editor.project.revision, 2);
        assert_eq!(app.editor.project.piece_bounds(piece).unwrap()[1][0], 0.5);
        assert_eq!(
            app.editor
                .project
                .piece(piece)
                .unwrap()
                .placement
                .translation[2],
            0.25
        );
        assert_eq!(
            app.editor.project.piece(piece).unwrap().grain_angle,
            std::f64::consts::FRAC_PI_2
        );
        assert_eq!(app.editor.project.materials[0].warp_stiffness, 50.);
        assert_eq!(app.editor.project.materials[0].weft_stiffness, 1000.);
        assert!(app.editor.undo());
        assert_eq!(app.editor.project.piece(piece).unwrap().grain_angle, 0.);
        assert_eq!(app.editor.project.materials[0].warp_stiffness, 400.);
        assert_eq!(app.editor.project.materials[0].weft_stiffness, 400.);
        assert_eq!(app.editor.project.piece_bounds(piece).unwrap()[1][0], 0.4);
        assert_eq!(
            app.editor
                .project
                .piece(piece)
                .unwrap()
                .placement
                .translation[2],
            0.0
        );
        assert_eq!(app.editor.project.materials[0].areal_density, 0.18);
    }

    #[test]
    fn inspecting_and_recoloring_preserves_valid_imported_dimensions_and_physics() {
        let mut app = Studio::empty();
        app.editor
            .execute(Command::AddRectangle {
                name: "Fine panel".into(),
                origin: [0., 0.],
                width: 0.005,
                height: 0.006,
            })
            .unwrap();
        let mut fabric = app.editor.project.materials[0].clone();
        fabric.areal_density = 0.002;
        fabric.warp_stiffness = 0.5;
        fabric.weft_stiffness = 200000.;
        fabric.bending_stiffness = 2.;
        app.editor
            .execute(Command::UpdateMaterial { material: fabric })
            .unwrap();
        app.selected = Some(app.editor.project.pieces[0].id);
        let before = app.editor.project.clone();
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| app.inspector(ui));
        assert_eq!(app.editor.project, before);
        let mut draft = app
            .draft
            .take()
            .unwrap_or_else(|| InspectorDraft::new(&app.editor.project, app.selected));
        draft.material.as_mut().unwrap().color = [0.7, 0.3, 0.2];
        app.draft = Some(draft);
        assert!(app.commit_draft());
        let mut expected = before;
        expected.revision += 1;
        expected.materials[0].color = [0.7, 0.3, 0.2];
        assert_eq!(app.editor.project, expected);
    }

    #[test]
    fn cancelled_and_obsolete_jobs_do_not_replace_the_display_snapshot() {
        let ctx = egui::Context::default();
        let (entered_sender, entered_receiver) = mpsc::channel();
        let (release_sender, release_receiver) = mpsc::channel();
        let job = Job::spawn(&ctx, 7, JobKind::Mesh, move |_| {
            entered_sender.send(()).unwrap();
            release_receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap();
            Ok(JobOutput::Mesh(Arc::new(Mesh::default())))
        });
        entered_receiver
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        assert!(job.accepts(7));
        assert!(!job.accepts(8));
        job.cancel.store(true, Ordering::Relaxed);
        release_sender.send(()).unwrap();
        assert!(
            matches!(job.receiver.recv_timeout(Duration::from_secs(2)).unwrap(),Err(message) if message.contains("cancelled"))
        );
        let mut app = Studio::empty();
        let old = Arc::new(Mesh::default());
        app.mesh = Some(old.clone());
        let (sender, receiver) = mpsc::channel();
        app.job = Some(Job {
            revision: 1,
            kind: JobKind::Mesh,
            batch: None,
            cancel: Arc::new(AtomicBool::new(false)),
            receiver,
        });
        sender
            .send(Ok(JobOutput::Mesh(Arc::new(Mesh {
                doc_revision: 1,
                ..Mesh::default()
            }))))
            .ok()
            .unwrap();
        app.poll();
        assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &old));
    }

    #[test]
    fn bounded_fit_phases_queue_without_rebuild_and_continue_accepted_cloth_state() {
        let ctx = egui::Context::default();
        let mut app = Studio::empty();
        app.editor
            .execute(Command::AddRectangle {
                name: "Free panel".into(),
                origin: [2.0, 1.0],
                width: 0.1,
                height: 0.1,
            })
            .unwrap();
        let mesh = Arc::new(app.editor.project.mesh().unwrap());
        let state =
            Arc::new(weftforge_core::simulation::create_cloth(&app.editor.project, &mesh).unwrap());
        app.mesh = Some(mesh.clone());
        app.cloth = Some(state.clone());
        let revision = app.editor.project.revision;
        app.queue_batch(SimulationBatch::assemble());
        let assembly = app.pending_batch.unwrap();
        assert_eq!(assembly.steps, 60);
        assert_eq!(assembly.settings.gravity, [0.0; 3]);
        assert_eq!(assembly.settings.substeps, 8);
        assert_eq!(assembly.settings.iterations, 32);
        assert_eq!(assembly.settings.seam_closure_speed, 1.0);
        assert!(!app.needs_mesh);
        assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
        app.start_batch(
            &ctx,
            SimulationBatch {
                steps: 1,
                ..assembly
            },
        );
        app.queue_batch(SimulationBatch::drape());
        assert_eq!(app.pending_batch.unwrap().steps, 120);
        assert_eq!(
            app.pending_batch.unwrap().settings.gravity,
            [0.0, -9.81, 0.0]
        );
        assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &state));
        let finish = |app: &mut Studio| {
            let deadline = Instant::now() + Duration::from_secs(3);
            while app.job.is_some() {
                assert!(
                    Instant::now() < deadline,
                    "bounded solver job did not finish"
                );
                app.poll();
                std::thread::sleep(Duration::from_millis(1));
            }
        };
        finish(&mut app);
        assert!((app.cloth.as_ref().unwrap().simulation_time() - 1.0 / 60.0).abs() < 1.0e-12);
        let drape = app.pending_batch.unwrap();
        app.start_batch(&ctx, SimulationBatch { steps: 1, ..drape });
        finish(&mut app);
        assert!((app.cloth.as_ref().unwrap().simulation_time() - 2.0 / 60.0).abs() < 1.0e-12);
        assert!(app.cloth.as_ref().unwrap().positions()[0][1] < state.positions()[0][1]);
        assert_eq!(app.editor.project.revision, revision);
        assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
        app.queue_batch(SimulationBatch::assemble());
        app.reset_simulation();
        assert!(app.pending_batch.is_none());
        assert!(app.cloth.is_none());
    }

    #[test]
    fn untitled_autosave_recovers_editable_work_and_failed_recovery_retains_it() {
        let dir = std::env::temp_dir().join(format!(
            "weftforge-ui-recovery-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = dir.join("Untitled.weftforge.autosave");
        let mut app = Studio::empty();
        app.session_autosave = Some(path.clone());
        app.editor
            .execute(Command::AddRectangle {
                name: "Unsaved panel".into(),
                origin: [0.0, 0.0],
                width: 0.2,
                height: 0.3,
            })
            .unwrap();
        app.dirty = true;
        app.start_autosave(&egui::Context::default());
        app.autosave_job
            .take()
            .unwrap()
            .receiver
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap();
        let saved = Project::load(&path).unwrap();
        assert_eq!(saved, app.editor.project);
        let mut recovered = Studio::empty();
        recovered.recover(path.clone());
        assert_eq!(recovered.editor.project, saved);
        assert!(recovered.dirty);
        assert!(recovered.path.is_none());
        let corrupt = dir.join("corrupt.recovery");
        fs::write(&corrupt, b"{not valid").unwrap();
        recovered.load_document(corrupt, true);
        assert_eq!(recovered.editor.project, saved);
        assert_eq!(Project::load(&path).unwrap(), saved);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn export_guard_preserves_source_projects_and_symlink_aliases() {
        let dir = std::env::temp_dir().join(format!(
            "weftforge-ui-export-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let source = dir.join("editable.native");
        Project::default().save(&source).unwrap();
        assert!(export_path_guard(Some(&source), &source).is_err());
        assert!(export_path_guard(None, &dir.join("other.weftforge")).is_err());
        assert!(export_path_guard(Some(&source), &dir.join("garment.obj")).is_ok());
        #[cfg(unix)]
        {
            let alias = dir.join("alias.svg");
            std::os::unix::fs::symlink(&source, &alias).unwrap();
            assert!(export_path_guard(Some(&source), &alias).is_err());
        }
        assert!(Project::load(&source).is_ok());
        fs::remove_dir_all(dir).unwrap();
    }
}
