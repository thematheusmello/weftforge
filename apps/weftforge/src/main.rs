use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use weftforge_core::{
    Command, Curve, CurveInterval, Editor, Id, Material, Mesh, Placement, Project,
};
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

struct RectangleDraft {
    origin: [f64; 2],
    dimensions_mm: [String; 2],
    error: Option<String>,
}

#[derive(Clone)]
struct InspectorDraft {
    revision: u64,
    selected: Option<Id>,
    curve: Option<Curve>,
    dimensions: Option<[f64; 2]>,
    placement: Option<Placement>,
    grain_angle: Option<f64>,
    material: Option<Material>,
    resolution: f64,
    color_pending: bool,
}
impl InspectorDraft {
    fn new(project: &Project, selected: Option<Id>, selected_curve: Option<Id>) -> Self {
        let piece = selected.and_then(|id| project.piece(id));
        Self {
            revision: project.revision,
            selected,
            curve: selected_curve.and_then(|id| project.curve(id)).cloned(),
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
        let original = Self::new(
            project,
            self.selected,
            self.curve.as_ref().map(|curve| curve.id),
        );
        let mut commands = Vec::new();
        if self.curve != original.curve
            && let Some(curve) = &self.curve
        {
            commands.push(Command::SetCurveControls {
                curve: curve.id,
                controls: curve.controls,
            });
        }
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
    selected_curve: Option<Id>,
    split_t: f64,
    tool: Tool,
    polygon: Vec<[f64; 2]>,
    pattern_canvas: Option<egui::Id>,
    drag_start: Option<[f64; 2]>,
    rectangle: Option<RectangleDraft>,
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
    discard_on_exit: bool,
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
            selected_curve: None,
            split_t: 0.5,
            tool: Tool::Select,
            polygon: vec![],
            pattern_canvas: None,
            drag_start: None,
            rectangle: None,
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
            discard_on_exit: false,
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
        if (self.selected != selected || self.selected_curve.is_some()) && self.commit_draft() {
            self.selected = selected;
            self.selected_curve = None;
        }
    }
    fn select_curve(&mut self, selected: Option<(Id, Id)>) {
        let piece = selected.map(|(piece, _)| piece);
        let curve = selected.map(|(_, curve)| curve);
        if (self.selected != piece || self.selected_curve != curve) && self.commit_draft() {
            self.selected = piece;
            self.selected_curve = curve;
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
        if self.selected_curve.is_some_and(|curve| {
            !self
                .selected
                .and_then(|id| self.editor.project.piece(id))
                .is_some_and(|piece| piece.boundary.contains(&curve))
        }) {
            self.selected_curve = None;
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
                None => {
                    weftforge_core::simulation::create_cloth_cancellable(&project, &mesh, || {
                        cancel.load(Ordering::Relaxed)
                    })
                    .map_err(|error| error.to_string())?
                }
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
    fn history(&mut self, ctx: &egui::Context, redo: bool) {
        if !self.commit_draft() {
            return;
        }
        // Restored values must not inherit the focused field's old edit text.
        if let Some(id) = ctx.memory(|memory| memory.focused()) {
            ctx.data_mut(|data| data.remove::<String>(id));
            ctx.memory_mut(|memory| memory.surrender_focus(id));
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
        egui::Window::new("Recover editable construction").enabled(self.rectangle.is_none()).open(&mut open).default_width(520.0).show(ctx,|ui| {
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
        let path = self.autosave_path();
        let Some(path) = path else {
            self.message="Autosave unavailable: the operating system data directory is not configured. Save the project explicitly.".into();
            return;
        };
        let project = self.editor.project.clone();
        let ctx = ctx.clone();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(write_autosave(&project, &path));
            ctx.request_repaint();
        });
        self.autosave_job = Some(AutosaveJob { receiver });
    }
    fn autosave_path(&self) -> Option<PathBuf> {
        self.path
            .as_ref()
            .map(|path| append_suffix(path, ".autosave"))
            .or_else(|| self.session_autosave.clone())
    }
    fn retain_on_exit(&mut self) -> Result<Option<PathBuf>, String> {
        self.cancel();
        if !self.discard_on_exit && !self.commit_draft() {
            eprintln!(
                "Pending edit rejected before shutdown recovery: {}",
                self.message
            );
        }
        if let Some(job) = self.autosave_job.take() {
            // Finish the older write before it can replace the final recovery snapshot.
            let _ = job.receiver.recv();
        }
        if self.discard_on_exit || !self.dirty {
            return Ok(None);
        }
        let fallback = self
            .session_autosave
            .as_ref()
            .and_then(|path| path.parent())
            .map(|dir| {
                static NEXT_EXIT: AtomicU64 = AtomicU64::new(0);
                dir.join(format!(
                    "Exit-{}-{}-{}.weftforge.autosave",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos(),
                    NEXT_EXIT.fetch_add(1, Ordering::Relaxed)
                ))
            });
        let mut errors = Vec::new();
        for path in self.autosave_path().into_iter().chain(fallback) {
            match write_autosave(&self.editor.project, &path) {
                Ok(()) => return Ok(Some(path)),
                Err(error) => errors.push(format!("{}: {error}", path.display())),
            }
        }
        if errors.is_empty() {
            Err("The operating system recovery directory is unavailable.".into())
        } else {
            Err(errors.join("\n"))
        }
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
                self.history(ui.ctx(), false);
            }
            if ui.button("Redo").clicked() {
                self.history(ui.ctx(), true);
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
                ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id(("piece-row",piece.id))),|ui| {
                    if ui.selectable_label(self.selected==Some(piece.id),&piece.name).clicked() { self.select(Some(piece.id)); }
                });
            }
            if let Some(piece)=self.selected.and_then(|id|self.editor.project.piece(id)).cloned() {
                ui.label("Boundary curve · select here or in 2D");
                egui::ComboBox::from_id_salt(("boundary-curve",piece.id))
                    .selected_text(self.selected_curve.map_or("Choose boundary".into(),|id|format!("Curve #{id}")))
                    .show_ui(ui,|ui| {
                        for (index,id) in piece.boundary.iter().enumerate() {
                            if ui.selectable_label(self.selected_curve==Some(*id),format!("Boundary {} · curve #{id}",index+1)).clicked() { self.select_curve(Some((piece.id,*id))); }
                        }
                    });
            }
            // Explicit scopes keep auto-ID edit buffers stable as dynamic rows appear.
            ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("creation")),|ui| {
            ui.separator(); ui.label("Pattern name"); ui.text_edit_singleline(&mut self.name);
            ui.horizontal(|ui| { ui.label("W / H m"); for value in &mut self.size { ui.add(egui::DragValue::new(value).speed(0.01).range(0.01..=5.0)); } });
            ui.horizontal(|ui| { ui.label("X / Y m"); for value in &mut self.coordinate { ui.add(egui::DragValue::new(value).speed(0.01)); } });
            if ui.button("Add rectangle").clicked() {
                self.execute(Command::AddRectangle { name:self.name.clone(),origin:self.coordinate,width:self.size[0],height:self.size[1] });
                self.select(self.editor.project.pieces.last().map(|piece|piece.id));
            }
            if ui.button("Add polygon point").clicked() { self.polygon.push(self.coordinate); self.tool=Tool::Polygon; }
            if ui.button(format!("Close pattern ({} points)",self.polygon.len())).clicked() { self.finish_polygon(); }
            });
            let mut draft=self.draft.take().filter(|draft|draft.revision==self.editor.project.revision && draft.selected==self.selected && draft.curve.as_ref().map(|curve|curve.id)==self.selected_curve)
                .unwrap_or_else(||InspectorDraft::new(&self.editor.project,self.selected,self.selected_curve));
            let mut split=None;
            if let Some(piece)=self.selected.and_then(|id|self.editor.project.piece(id)).cloned() {
                ui.separator(); ui.strong(&piece.name);
                if let Some(curve)=&mut draft.curve {
                    ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id(("curve",curve.id))),|ui| {
                    ui.strong(format!("Selected curve #{}",curve.id));
                    let start=self.editor.project.point(curve.start).expect("validated curve start").position;
                    let end=self.editor.project.point(curve.end).expect("validated curve end").position;
                    ui.label(format!("P0 #{} · {:.6} / {:.6} m",curve.start,start[0],start[1]));
                    ui.label(format!("P3 #{} · {:.6} / {:.6} m",curve.end,end[0],end[1]));
                    let mut cubic=curve.controls.is_some();
                    if ui.checkbox(&mut cubic,"Cubic Bezier").changed() {
                        curve.controls=cubic.then(||[1.0/3.0,2.0/3.0].map(|t|std::array::from_fn(|axis|start[axis]+(end[axis]-start[axis])*t)));
                        commit=true;
                    }
                    if let Some(controls)=&mut curve.controls {
                        for (index,point) in controls.iter_mut().enumerate() {
                            ui.label(format!("P{} control X / Y m",index+1));
                            ui.horizontal(|ui| { for value in point { commit|=numeric_commit(&ui.add(egui::DragValue::new(value).speed(0.001))); } });
                        }
                    }
                    ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("split")),|ui| {
                        ui.horizontal(|ui| { ui.label("Split parameter t"); ui.add(egui::DragValue::new(&mut self.split_t).speed(0.01)); });
                    });
                    ui.small("Interior curve parameter, not arc length. Sewing is partitioned without changing point pairs.");
                    if ui.button("Split selected curve").clicked() { split=Some(curve.id); }
                    ui.separator();
                    });
                }
                ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id(("piece",piece.id))),|ui| {
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
                    // RGB conversion can round unchanged values while drawing the widget.
                    let mut color=material.color;
                    if ui.color_edit_button_rgb(&mut color).changed() { material.color=color; draft.color_pending=true; }
                    commit|=draft.color_pending && ui.input(|input|input.pointer.any_released());
                }
                });
                if ui.button("Delete selected").clicked() { self.draft=Some(draft); self.execute(Command::RemovePiece { piece:piece.id }); return; }
            }
            let mut remove=None;
            ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("sewing")),|ui| {
            ui.separator(); ui.strong("Sewing"); ui.checkbox(&mut self.reversed,"Reverse paired edge"); ui.label("A interval / B interval (0–1)");
            for range in &mut self.intervals { ui.horizontal(|ui| { for value in range { ui.add(egui::DragValue::new(value).speed(0.01).range(0.0..=1.0)); } }); }
            ui.label(if self.seam_start.is_some() { "Click paired boundary in 2D." } else { "Sew edges: click two 2D boundaries." });
            for seam in &self.editor.project.seams { ui.horizontal(|ui| { ui.label(&seam.name); if ui.small_button("Remove").clicked() { remove=Some(seam.id); } }); }
            });
            ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("mesh-settings")),|ui| {
            ui.separator(); ui.label("Mesh edge length m"); commit|=numeric_commit(&ui.add(egui::DragValue::new(&mut draft.resolution).speed(0.005).range(0.005..=0.5)));
            ui.label("Iterations / substeps"); ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.settings.iterations).range(1..=50));
                ui.add(egui::DragValue::new(&mut self.settings.substeps).range(1..=16));
            });
            if let Some(cloth)=&self.cloth { let diagnostics=cloth.diagnostics(); ui.label(format!("Time {:.3} s\nAssembly remaining {:.4} m\nStretch error {:.3}\nSeam gap {:.4} m\nContact error {:.4} m\nInitial intersections {}",cloth.simulation_time(),diagnostics.seam_assembly_remaining,diagnostics.max_stretch_error,diagnostics.max_seam_error,diagnostics.max_contact_error,diagnostics.initial_intersections)); }
            });
            self.draft=Some(draft);
            if let Some(seam)=remove { self.execute(Command::RemoveSeam { seam }); }
            if let Some(curve)=split { self.execute(Command::SplitCurve { curve,t:self.split_t }); }
        });
        let opening_rectangle = self.tool == Tool::Rectangle
            && self
                .pattern_canvas
                .and_then(|id| ui.ctx().read_response(id))
                .is_some_and(|response| response.clicked());
        // Keep pending edits staged while the numeric constructor opens or remains open.
        if commit && self.rectangle.is_none() && !opening_rectangle {
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
    fn rectangle_window(&mut self, ctx: &egui::Context) {
        let Some(mut rectangle) = self.rectangle.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        let response = egui::Modal::new(egui::Id::new("create-rectangle")).show(ctx, |ui| {
            ui.heading("Create Rectangle");
            for (label, value) in ["Width (mm)", "Height (mm)"]
                .into_iter()
                .zip(&mut rectangle.dimensions_mm)
            {
                ui.horizontal(|ui| {
                    let label = ui.label(label);
                    ui.add(egui::TextEdit::singleline(value).desired_width(100.0))
                        .labelled_by(label.id);
                });
            }
            if let Some(error) = &rectangle.error {
                ui.colored_label(Color32::LIGHT_RED, error);
            }
            ui.horizontal(|ui| {
                accept = ui.button("OK").clicked();
                cancel = ui.button("Cancel").clicked();
            });
        });
        if cancel || response.should_close() {
            return;
        }
        if accept {
            let dimensions = rectangle
                .dimensions_mm
                .each_ref()
                .map(|value| value.trim().parse::<f64>());
            if let [Ok(width), Ok(height)] = dimensions {
                let mut commands = self
                    .draft
                    .as_ref()
                    .filter(|draft| draft.revision == self.editor.project.revision)
                    .map_or_else(Vec::new, |draft| draft.commands(&self.editor.project));
                // Reference input limits are unobserved; the shared command validates dimensions.
                commands.push(Command::AddRectangle {
                    name: self.name.clone(),
                    origin: rectangle.origin,
                    width: width / 1000.0,
                    height: height / 1000.0,
                });
                let command = if commands.len() == 1 {
                    commands.pop().expect("rectangle command")
                } else {
                    Command::Transaction(commands)
                };
                if self.apply_command(command) {
                    self.select(self.editor.project.pieces.last().map(|piece| piece.id));
                    return;
                }
                rectangle.error = Some(self.message.clone());
            } else {
                rectangle.error = Some("Enter numeric width and height in mm.".into());
            }
        }
        self.rectangle = Some(rectangle);
    }
    fn patterns(&mut self, ui: &mut egui::Ui) {
        ui.strong("2D PATTERNS");
        ui.small("Metres · 1 cm snap · scroll zoom · middle drag pan");
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        self.pattern_canvas = Some(response.id);
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
                        if self.selected_curve == Some(*id) {
                            Stroke::new(4., Color32::GOLD)
                        } else {
                            Stroke::new(if selected { 2.5 } else { 1.5 }, color)
                        },
                    ));
                    if self.selected_curve == Some(*id) {
                        if let Some([a, b]) = curve.controls {
                            for (point, control) in [(curve.start, a), (curve.end, b)] {
                                if let Some(point) = doc.point(point) {
                                    painter.line_segment(
                                        [
                                            self.to_screen(point.position, r),
                                            self.to_screen(control, r),
                                        ],
                                        Stroke::new(1., Color32::GOLD),
                                    );
                                }
                                painter.circle_stroke(
                                    self.to_screen(control, r),
                                    4.,
                                    Stroke::new(1.5, Color32::GOLD),
                                );
                            }
                        }
                        if let Ok(point) = curve.evaluate(&doc, self.split_t) {
                            painter.circle_filled(self.to_screen(point, r), 4., Color32::WHITE);
                        }
                    }
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
                Tool::Select => self.select_curve(nearest),
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
                Tool::Rectangle => {
                    if let Some(p) = pointer {
                        // shortcut: existing snapped lower-left anchor; reference equivalence needs observation.
                        self.rectangle = Some(RectangleDraft {
                            origin: self.to_pattern(p, r),
                            dimensions_mm: ["500".into(), "500".into()],
                            error: None,
                        });
                    }
                }
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
    fn on_exit(&mut self) {
        // macOS application Quit bypasses the cancellable viewport close request.
        if let Err(error) = self.retain_on_exit() {
            eprintln!("Shutdown recovery failed: {error}");
            rfd::MessageDialog::new()
                .set_title("Unsaved recovery failed")
                .set_description(format!("The application is exiting, but the latest edits could not be retained.\n{error}"))
                .set_level(rfd::MessageLevel::Error)
                .show();
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll();
        self.poll_autosave();
        if self.rectangle.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S))
        {
            self.save(false);
        }
        if self.rectangle.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O))
        {
            self.choose_open();
        }
        if self.rectangle.is_none()
            && !ctx.egui_wants_keyboard_input()
            && ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Z,
                )
            })
        {
            self.history(&ctx, true);
        } else if self.rectangle.is_none()
            && !ctx.egui_wants_keyboard_input()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z))
        {
            self.history(&ctx, false);
        }
        if self.rectangle.is_none()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::K))
        {
            self.search = !self.search;
        }
        if self.rectangle.is_none() && !ctx.egui_wants_keyboard_input() {
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
        // Disabled DragValue still handles AccessKit value actions in egui 0.35.
        let modal_events = self
            .rectangle
            .is_some()
            .then(|| ctx.input_mut(|input| std::mem::take(&mut input.events)));
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            if self.rectangle.is_some() {
                ui.disable();
            }
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
                        self.discard_on_exit = true;
                        self.draft = None;
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
                .enabled(self.rectangle.is_none())
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
                                "Undo" => self.history(&ctx, false),
                                "Redo" => self.history(&ctx, true),
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
        if let Some(events) = modal_events {
            ctx.input_mut(|input| input.events = events);
        }
        self.rectangle_window(&ctx);
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
fn write_autosave(project: &Project, path: &Path) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    project.save(path).map_err(|error| error.to_string())
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
    fn focused_numeric_history_restore_survives_blur_and_retains_redo() {
        use egui::accesskit::{Action, ActionRequest, NodeId, TreeId};
        fn frame(
            ctx: &egui::Context,
            app: &mut Studio,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1450., 1800.))),
                    events,
                    ..Default::default()
                },
                |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
            )
        }
        fn action(target_node: NodeId, action: Action) -> egui::Event {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action,
                target_tree: TreeId::ROOT,
                target_node,
                data: None,
            })
        }
        fn button(output: &egui::FullOutput, label: &str) -> (NodeId, Pos2) {
            let (id, node) = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.label() == Some(label) && node.supports_action(Action::Click)
                })
                .unwrap();
            let bounds = node.bounds().unwrap();
            (
                *id,
                Pos2::new(
                    ((bounds.x0 + bounds.x1) * 0.5) as f32,
                    ((bounds.y0 + bounds.y1) * 0.5) as f32,
                ),
            )
        }
        fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }
        }
        for pointer in [false, true] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = Studio::empty();
            app.editor
                .execute(Command::AddRectangle {
                    name: "Panel".into(),
                    origin: [0., 0.],
                    width: 0.4,
                    height: 0.6,
                })
                .unwrap();
            let piece = app.editor.project.pieces[0].id;
            app.editor
                .execute(Command::SetGrain {
                    piece,
                    angle: 0.125,
                })
                .unwrap();
            app.selected = Some(piece);
            let before = app.editor.project.clone();
            let output = frame(&ctx, &mut app, vec![]);
            let (grain, grain_node) = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| node.numeric_value() == Some(0.125))
                .unwrap();
            let grain = *grain;
            let display_before = grain_node.value().unwrap().to_owned();
            frame(&ctx, &mut app, vec![action(grain, Action::Focus)]);
            frame(&ctx, &mut app, vec![]);
            let output = frame(
                &ctx,
                &mut app,
                vec![
                    key(egui::Key::A, egui::Modifiers::COMMAND),
                    egui::Event::Text("0.25".into()),
                ],
            );
            assert_eq!(app.editor.project, before);
            assert_eq!(app.draft.as_ref().unwrap().grain_angle, Some(0.25));
            let (undo, pos) = button(&output, "Undo");
            if pointer {
                for pressed in [true, false] {
                    frame(
                        &ctx,
                        &mut app,
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
                }
            } else {
                frame(&ctx, &mut app, vec![action(undo, Action::Click)]);
            }
            assert_eq!(app.editor.project.piece(piece).unwrap().grain_angle, 0.125);
            assert!(
                app.editor.can_redo(),
                "redo immediately after Undo; pointer={pointer}"
            );
            let output = frame(&ctx, &mut app, vec![]);
            let grain_node = &output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(id, _)| *id == grain)
                .unwrap()
                .1;
            assert_eq!(grain_node.value(), Some(display_before.as_str()));
            let output = frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
            );
            assert_eq!(
                app.editor.project.piece(piece).unwrap().grain_angle,
                0.125,
                "after blur; pointer={pointer}"
            );
            assert!(app.editor.can_redo(), "redo after blur; pointer={pointer}");
            let mut restored = before;
            restored.revision = app.editor.project.revision;
            assert_eq!(app.editor.project, restored);
            let (redo, _) = button(&output, "Redo");
            frame(&ctx, &mut app, vec![action(redo, Action::Click)]);
            frame(&ctx, &mut app, vec![]);
            assert_eq!(app.editor.project.piece(piece).unwrap().grain_angle, 0.25);
        }
    }

    #[test]
    fn rectangle_modal_blocks_background_accesskit_actions_and_preserves_staged_edits() {
        use egui::accesskit::{Action, ActionData, ActionRequest, NodeId, TreeId};
        fn frame(
            ctx: &egui::Context,
            app: &mut Studio,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1450., 1800.))),
                    events,
                    ..Default::default()
                },
                |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
            )
        }
        fn action(target_node: NodeId, action: Action, data: Option<ActionData>) -> egui::Event {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action,
                target_tree: TreeId::ROOT,
                target_node,
                data,
            })
        }
        fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }
        }
        fn button(output: &egui::FullOutput, label: &str) -> NodeId {
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.label() == Some(label) && node.supports_action(Action::Click)
                })
                .unwrap_or_else(|| panic!("missing button {label:?}"))
                .0
        }
        fn numeric(output: &egui::FullOutput, value: f64) -> NodeId {
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| node.numeric_value() == Some(value))
                .unwrap()
                .0
        }
        for label in ["Add rectangle", "Undo"] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = Studio::empty();
            for name in ["First", "Second"] {
                app.editor
                    .execute(Command::AddRectangle {
                        name: name.into(),
                        origin: [0., 0.],
                        width: 0.4,
                        height: 0.6,
                    })
                    .unwrap();
            }
            assert!(app.editor.undo());
            app.selected = Some(app.editor.project.pieces[0].id);
            app.size = [0.47, 0.63];
            app.coordinate = [0.12, 0.34];
            let mut draft = InspectorDraft::new(&app.editor.project, app.selected, None);
            draft.grain_angle = Some(0.25);
            draft.resolution = 0.085;
            app.draft = Some(draft);
            let output = frame(&ctx, &mut app, vec![]);
            let target_button = button(&output, label);
            let numeric_targets = [0.47, 0.12, 0.25, 0.085].map(|value| numeric(&output, value));
            frame(
                &ctx,
                &mut app,
                vec![action(numeric_targets[2], Action::Focus, None)],
            );
            app.rectangle = Some(RectangleDraft {
                origin: [0., 0.],
                dimensions_mm: ["500".into(), "500".into()],
                error: None,
            });
            let output = frame(&ctx, &mut app, vec![]);
            let cancel = button(&output, "Cancel");
            let width = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, node)| node.value() == Some("500"))
                .min_by(|a, b| {
                    a.1.bounds()
                        .unwrap()
                        .y0
                        .total_cmp(&b.1.bounds().unwrap().y0)
                })
                .unwrap()
                .0;
            frame(&ctx, &mut app, vec![action(width, Action::Focus, None)]);
            for value in ["200", "300"] {
                frame(
                    &ctx,
                    &mut app,
                    vec![
                        key(egui::Key::A, egui::Modifiers::COMMAND),
                        egui::Event::Text(value.into()),
                        key(egui::Key::Tab, egui::Modifiers::NONE),
                    ],
                );
            }
            assert_eq!(
                app.rectangle.as_ref().unwrap().dimensions_mm,
                ["200", "300"]
            );
            let before = app.editor.project.clone();
            let pending = app.draft.as_ref().unwrap().commands(&before);
            let values = (app.size, app.coordinate);
            let history = (app.editor.can_undo(), app.editor.can_redo());

            frame(
                &ctx,
                &mut app,
                vec![action(target_button, Action::Click, None)],
            );
            assert_eq!(app.editor.project, before, "background {label}");
            assert_eq!(app.draft.as_ref().unwrap().commands(&before), pending);
            for target in numeric_targets {
                frame(
                    &ctx,
                    &mut app,
                    vec![
                        action(
                            target,
                            Action::SetValue,
                            Some(ActionData::NumericValue(0.95)),
                        ),
                        action(target, Action::Increment, None),
                    ],
                );
                assert_eq!((app.size, app.coordinate), values);
                assert_eq!(app.draft.as_ref().unwrap().commands(&before), pending);
                assert_eq!(app.editor.project, before);
            }
            assert_eq!((app.editor.can_undo(), app.editor.can_redo()), history);
            frame(&ctx, &mut app, vec![action(cancel, Action::Click, None)]);
            assert!(app.rectangle.is_none());
            frame(&ctx, &mut app, vec![]);
            assert_eq!(app.editor.project, before);
            assert_eq!(app.draft.as_ref().unwrap().commands(&before), pending);
            assert_eq!((app.size, app.coordinate), values);
            assert_eq!((app.editor.can_undo(), app.editor.can_redo()), history);
            assert!(!app.dirty);
        }
    }

    #[test]
    fn inspector_focus_does_not_reuse_edit_text_after_adding_or_selecting_pieces() {
        use egui::accesskit::{Action, ActionRequest, NodeId, Role, TreeId};
        fn frame(
            ctx: &egui::Context,
            app: &mut Studio,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(500., 1800.))),
                    events,
                    ..Default::default()
                },
                |ui| app.inspector(ui),
            )
        }
        fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }
        }
        fn action(output: &egui::FullOutput, text: &str, action: Action) -> egui::Event {
            let target_node = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| {
                    if action == Action::Focus {
                        node.role() == Role::TextInput && node.value() == Some(text)
                    } else {
                        node.label() == Some(text) && node.supports_action(action)
                    }
                })
                .unwrap_or_else(|| panic!("missing action {action:?} for {text:?}"))
                .0;
            request(target_node, action)
        }
        fn request(target_node: NodeId, action: Action) -> egui::Event {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action,
                target_tree: TreeId::ROOT,
                target_node,
                data: None,
            })
        }
        fn numeric_field(output: &egui::FullOutput, label: &str, index: usize) -> NodeId {
            let tree = output.platform_output.accesskit_update.as_ref().unwrap();
            let top = output
                .shapes
                .iter()
                .find_map(|clipped| match &clipped.shape {
                    egui::Shape::Text(shape) if shape.galley.text() == label => {
                        Some(shape.pos.y as f64)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing label {label:?}"));
            let mut fields: Vec<_> = tree
                .nodes
                .iter()
                .filter_map(|(id, node)| {
                    let bounds = node.bounds()?;
                    (node.numeric_value().is_some() && bounds.y0 >= top - 3.0)
                        .then_some((*id, bounds))
                })
                .collect();
            let row_top = fields
                .iter()
                .map(|(_, bounds)| bounds.y0)
                .min_by(f64::total_cmp)
                .unwrap();
            fields.retain(|(_, bounds)| bounds.y0 < row_top + 4.0);
            fields.sort_by(|a, b| a.1.x0.total_cmp(&b.1.x0));
            fields[index].0
        }
        fn replace(
            ctx: &egui::Context,
            app: &mut Studio,
            text: &str,
            tab: bool,
        ) -> egui::FullOutput {
            let output = frame(
                ctx,
                app,
                vec![
                    key(egui::Key::A, egui::Modifiers::COMMAND),
                    egui::Event::Text(text.into()),
                ],
            );
            if tab {
                frame(ctx, app, vec![key(egui::Key::Tab, egui::Modifiers::NONE)])
            } else {
                output
            }
        }
        fn focused(output: &egui::FullOutput) -> (NodeId, Option<String>, Option<f64>) {
            let tree = output.platform_output.accesskit_update.as_ref().unwrap();
            let node = tree.nodes.iter().find(|(id, _)| *id == tree.focus).unwrap();
            (
                node.0,
                node.1.value().map(str::to_owned),
                node.1.numeric_value(),
            )
        }
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = Studio::empty();
        let output = frame(&ctx, &mut app, vec![]);
        frame(
            &ctx,
            &mut app,
            vec![action(&output, "Pattern 1", Action::Focus)],
        );
        let mut output = replace(&ctx, &mut app, "Curve A", true);
        for (value, tab) in [("0.4", true), ("0.6", true), ("0", true), ("0", false)] {
            output = replace(&ctx, &mut app, value, tab);
        }
        assert_eq!(app.size, [0.4, 0.6]);
        assert_eq!(app.coordinate, [0., 0.]);
        frame(
            &ctx,
            &mut app,
            vec![action(&output, "Add rectangle", Action::Click)],
        );
        let output = frame(&ctx, &mut app, vec![]);
        let after_first_add = focused(&output);
        let first = app.editor.project.pieces[0].id;
        assert_eq!(
            app.editor.project.piece_bounds(first).unwrap(),
            [[0., 0.], [0.4, 0.6]]
        );

        frame(
            &ctx,
            &mut app,
            vec![action(&output, "Curve A", Action::Focus)],
        );
        let mut output = replace(&ctx, &mut app, "Curve B", true);
        for (value, tab) in [("0.4", true), ("0.6", true), ("1", true), ("0", false)] {
            output = replace(&ctx, &mut app, value, tab);
        }
        frame(
            &ctx,
            &mut app,
            vec![action(&output, "Add rectangle", Action::Click)],
        );
        let output = frame(&ctx, &mut app, vec![]);
        let after_second_add = focused(&output);
        let second = app.editor.project.pieces[1].id;
        assert_eq!(
            app.editor.project.piece_bounds(second).unwrap(),
            [[1., 0.], [1.4, 0.6]]
        );

        frame(
            &ctx,
            &mut app,
            vec![action(&output, "Curve A", Action::Click)],
        );
        let curve = app.editor.project.piece(first).unwrap().boundary[2];
        app.select_curve(Some((first, curve)));
        let output = frame(&ctx, &mut app, vec![]);
        frame(
            &ctx,
            &mut app,
            vec![action(&output, "Cubic Bezier", Action::Click)],
        );
        let output = frame(&ctx, &mut app, vec![]);
        frame(
            &ctx,
            &mut app,
            vec![action(&output, "Curve B", Action::Focus)],
        );
        let output = frame(&ctx, &mut app, vec![]);
        assert_eq!(app.size, [0.4, 0.6]);
        assert_eq!(after_first_add.0, after_second_add.0);
        assert_eq!(
            after_second_add.1.unwrap().parse::<f64>().unwrap(),
            after_second_add.2.unwrap()
        );

        let before_selection = app.editor.project.clone();
        let width_field = numeric_field(&output, "Size m", 0);
        frame(&ctx, &mut app, vec![request(width_field, Action::Focus)]);
        app.select(Some(first));
        let output = frame(&ctx, &mut app, vec![]);
        assert_eq!(numeric_field(&output, "Size m", 0), width_field);
        app.select_curve(Some((first, curve)));
        let output = frame(&ctx, &mut app, vec![]);
        assert_eq!(numeric_field(&output, "Size m", 0), width_field);
        assert_eq!(app.editor.project, before_selection);

        let control_field = numeric_field(&output, "P1 control X / Y m", 0);
        frame(&ctx, &mut app, vec![request(control_field, Action::Focus)]);
        frame(&ctx, &mut app, vec![]);
        let output = replace(&ctx, &mut app, "0.321", false);
        assert_eq!(
            app.draft
                .as_ref()
                .unwrap()
                .curve
                .as_ref()
                .unwrap()
                .controls
                .unwrap()[0][0],
            0.321
        );
        frame(
            &ctx,
            &mut app,
            vec![action(&output, "Curve B", Action::Click)],
        );
        assert_eq!(
            app.editor.project.curve(curve).unwrap().controls.unwrap()[0][0],
            0.321
        );
        let other_curve = app.editor.project.piece(second).unwrap().boundary[2];
        assert!(
            app.editor
                .project
                .curve(other_curve)
                .unwrap()
                .controls
                .is_none()
        );
        app.select_curve(Some((second, other_curve)));
        let output = frame(&ctx, &mut app, vec![]);
        frame(
            &ctx,
            &mut app,
            vec![action(&output, "Cubic Bezier", Action::Click)],
        );
        let output = frame(&ctx, &mut app, vec![]);
        assert_ne!(
            numeric_field(&output, "P1 control X / Y m", 0),
            control_field
        );
        let other_controls = app.editor.project.curve(other_curve).unwrap().controls;

        let mesh_field = numeric_field(&output, "Mesh edge length m", 0);
        frame(&ctx, &mut app, vec![request(mesh_field, Action::Focus)]);
        frame(&ctx, &mut app, vec![]);
        replace(&ctx, &mut app, "0.085", false);
        app.select(Some(second));
        let output = frame(&ctx, &mut app, vec![]);
        assert_eq!(numeric_field(&output, "Mesh edge length m", 0), mesh_field);
        assert_eq!(
            focused(&output),
            (mesh_field, Some("0.085".into()), Some(0.085))
        );
        app.select(Some(first));
        let output = frame(&ctx, &mut app, vec![]);
        assert_eq!(numeric_field(&output, "Mesh edge length m", 0), mesh_field);
        assert_eq!(app.editor.project.resolution, 0.085);
        assert_eq!(
            app.editor.project.curve(other_curve).unwrap().controls,
            other_controls
        );

        app.select_curve(Some((first, curve)));
        let output = frame(&ctx, &mut app, vec![]);
        let split_field = numeric_field(&output, "Split parameter t", 0);
        frame(&ctx, &mut app, vec![request(split_field, Action::Focus)]);
        frame(&ctx, &mut app, vec![]);
        let output = replace(&ctx, &mut app, "0.35", false);
        frame(
            &ctx,
            &mut app,
            vec![action(&output, "Cubic Bezier", Action::Click)],
        );
        let output = frame(&ctx, &mut app, vec![]);
        assert_eq!(numeric_field(&output, "Split parameter t", 0), split_field);
        assert_eq!(
            focused(&output),
            (split_field, Some("0.35".into()), Some(0.35))
        );
        assert_eq!(app.size, [0.4, 0.6]);
    }

    #[test]
    fn numeric_rectangle_click_commits_once_and_cancel_or_errors_preserve_drafts_and_history() {
        fn frame(
            ctx: &egui::Context,
            app: &mut Studio,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(900., 700.))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    ui.columns(2, |columns| {
                        app.inspector(&mut columns[0]);
                        app.patterns(&mut columns[1]);
                    });
                    app.rectangle_window(ui.ctx());
                },
            )
        }
        fn pointer(pos: Pos2, pressed: bool) -> egui::Event {
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }
        }
        fn click(ctx: &egui::Context, app: &mut Studio, pos: Pos2) -> egui::FullOutput {
            frame(
                ctx,
                app,
                vec![egui::Event::PointerMoved(pos), pointer(pos, true)],
            );
            frame(ctx, app, vec![pointer(pos, false)])
        }
        fn text_position(output: &egui::FullOutput, text: &str, index: usize) -> Pos2 {
            output
                .shapes
                .iter()
                .filter_map(|clipped| match &clipped.shape {
                    egui::Shape::Text(shape)
                        if shape.galley.text() == text
                            || text.parse::<f64>().ok().is_some_and(|value| {
                                shape.galley.text().parse::<f64>().ok() == Some(value)
                            }) =>
                    {
                        Some(shape.pos + shape.galley.size() * 0.5)
                    }
                    _ => None,
                })
                .nth(index)
                .unwrap_or_else(|| panic!("missing text {text:?}"))
        }
        fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }
        }
        let ctx = egui::Context::default();
        let mut app = Studio::empty();
        app.tool = Tool::Rectangle;
        let before = app.editor.project.clone();
        frame(&ctx, &mut app, vec![]);
        let anchor = Pos2::new(600., 250.);
        click(&ctx, &mut app, anchor);
        assert_eq!(app.editor.project, before);
        assert!(!app.dirty);
        assert!(!app.editor.can_undo());
        let rectangle = app.rectangle.as_ref().unwrap();
        assert_eq!(rectangle.dimensions_mm, ["500", "500"]);
        let origin = rectangle.origin;
        let output = frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, text_position(&output, "500", 0));
        frame(
            &ctx,
            &mut app,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Text("200".into()),
                key(egui::Key::Tab, egui::Modifiers::NONE),
            ],
        );
        assert_eq!(
            app.rectangle.as_ref().unwrap().dimensions_mm,
            ["200", "500"]
        );
        frame(
            &ctx,
            &mut app,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Text("300".into()),
                key(egui::Key::Tab, egui::Modifiers::NONE),
            ],
        );
        assert_eq!(
            app.rectangle.as_ref().unwrap().dimensions_mm,
            ["200", "300"]
        );
        let output = frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, text_position(&output, "OK", 0));
        assert!(app.rectangle.is_none());
        assert_eq!(app.editor.project.revision, before.revision + 1);
        assert_eq!(app.editor.project.pieces.len(), 1);
        let piece = app.editor.project.pieces[0].id;
        let [min, max] = app.editor.project.piece_bounds(piece).unwrap();
        assert_eq!(min, origin);
        assert!((max[0] - min[0] - 0.2).abs() < 1.0e-12);
        assert!((max[1] - min[1] - 0.3).abs() < 1.0e-12);
        assert_eq!(app.selected, Some(piece));
        app.history(&ctx, false);
        assert!(app.editor.project.pieces.is_empty());
        assert!(!app.editor.can_undo());
        assert!(app.editor.can_redo());

        let before = app.editor.project.clone();
        let output = frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, text_position(&output, "0.07", 0));
        frame(&ctx, &mut app, vec![]);
        frame(
            &ctx,
            &mut app,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Text("0.075".into()),
            ],
        );
        let pending_resolution = app.draft.as_ref().unwrap().resolution;
        assert_eq!(pending_resolution, 0.075);
        assert_eq!(app.editor.project, before);
        frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, anchor);
        assert_eq!(app.editor.project, before);
        for width in ["0", "invalid"] {
            app.rectangle.as_mut().unwrap().dimensions_mm[0] = width.into();
            let output = frame(&ctx, &mut app, vec![]);
            click(&ctx, &mut app, text_position(&output, "OK", 0));
            assert!(app.rectangle.as_ref().unwrap().error.is_some());
            assert_eq!(app.editor.project, before);
            assert_eq!(app.draft.as_ref().unwrap().resolution, pending_resolution);
            assert!(!app.editor.can_undo());
            assert!(app.editor.can_redo());
        }
        let output = frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, text_position(&output, "Cancel", 0));
        assert!(app.rectangle.is_none());
        assert_eq!(app.editor.project, before);
        assert_eq!(app.draft.as_ref().unwrap().resolution, pending_resolution);
        assert!(!app.editor.can_undo());
        assert!(app.editor.can_redo());

        frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, anchor);
        app.rectangle.as_mut().unwrap().dimensions_mm = ["200".into(), "300".into()];
        let output = frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, text_position(&output, "OK", 0));
        assert_eq!(app.editor.project.pieces.len(), 1);
        assert_eq!(app.editor.project.resolution, pending_resolution);
        assert!(app.draft.is_none());
        app.history(&ctx, false);
        assert!(app.editor.project.pieces.is_empty());
        assert_eq!(app.editor.project.resolution, before.resolution);
        assert!(!app.editor.can_undo());

        let ctx = egui::Context::default();
        let mut app = Studio::empty();
        app.tool = Tool::Rectangle;
        frame(&ctx, &mut app, vec![]);
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(anchor), pointer(anchor, true)],
        );
        for pos in [anchor + Vec2::splat(20.), anchor + Vec2::splat(100.)] {
            frame(&ctx, &mut app, vec![egui::Event::PointerMoved(pos)]);
        }
        frame(
            &ctx,
            &mut app,
            vec![pointer(anchor + Vec2::splat(100.), false)],
        );
        assert!(app.rectangle.is_none());
        assert_eq!(app.editor.project.pieces.len(), 1);
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
        let mut draft = InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve);
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
        let mut draft = app.draft.take().unwrap_or_else(|| {
            InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve)
        });
        draft.material.as_mut().unwrap().color = [0.7, 0.3, 0.2];
        app.draft = Some(draft);
        assert!(app.commit_draft());
        let mut expected = before;
        expected.revision += 1;
        expected.materials[0].color = [0.7, 0.3, 0.2];
        assert_eq!(app.editor.project, expected);
    }

    #[test]
    fn selected_curve_controls_and_split_use_fresh_drafts_atomic_history_and_invalidation() {
        let mut app = Studio::empty();
        app.editor
            .execute(Command::AddRectangle {
                name: "Curved panel".into(),
                origin: [0., 0.],
                width: 0.4,
                height: 0.6,
            })
            .unwrap();
        let piece = app.editor.project.pieces[0].id;
        let curve = app.editor.project.pieces[0].boundary[2];
        app.select_curve(Some((piece, curve)));
        let before = app.editor.project.clone();
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| app.inspector(ui));
        assert_eq!(app.editor.project, before);
        let mut draft = InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve);
        for height in [0.65, 0.7, 0.8] {
            draft.curve.as_mut().unwrap().controls = Some([[0.3, height], [0.1, height]]);
        }
        app.draft = Some(draft);
        assert!(app.commit_draft());
        assert_eq!(app.editor.project.revision, before.revision + 1);
        assert!(app.needs_mesh);
        assert_eq!(app.selected_curve, Some(curve));
        assert_eq!(
            app.editor.project.curve(curve).unwrap().controls,
            Some([[0.3, 0.8], [0.1, 0.8]])
        );
        let authored = app.editor.project.clone();
        app.split_t = 0.3;
        app.execute(Command::SplitCurve {
            curve,
            t: app.split_t,
        });
        assert_eq!(app.editor.project.revision, authored.revision + 1);
        assert_eq!(app.editor.project.pieces[0].boundary.len(), 5);
        assert_eq!(app.selected_curve, Some(curve));
        assert!(app.editor.undo());
        app.rebuild();
        assert_eq!(app.editor.project.curves, authored.curves);
        assert!(app.editor.undo());
        app.rebuild();
        assert_eq!(app.editor.project.curves, before.curves);
        assert!(app.editor.redo());
        app.rebuild();
        let mut draft = InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve);
        draft.curve.as_mut().unwrap().controls = Some([[f64::NAN, 0.8], [0.1, 0.8]]);
        app.draft = Some(draft);
        let before_invalid = app.editor.project.clone();
        assert!(!app.commit_draft());
        assert_eq!(app.editor.project, before_invalid);
        assert!(app.editor.can_redo());
        assert!(app.message.contains("finite"));
        let mut draft = InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve);
        draft.curve.as_mut().unwrap().controls = None;
        app.draft = Some(draft);
        assert!(app.commit_draft());
        assert_eq!(app.editor.project.curve(curve).unwrap().controls, None);
        let next = app.editor.project.pieces[0].boundary[0];
        app.select_curve(Some((piece, next)));
        let draft = InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve);
        assert_eq!(draft.curve.unwrap().id, next);
        app.select(Some(piece));
        assert_eq!(app.selected_curve, None);
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

    fn exit_test_directory(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "weftforge-exit-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn exit_retains_last_second_drafts_after_an_older_autosave_then_recovers_exactly() {
        let dir = exit_test_directory("ordered");
        let path = dir.join("Untitled.weftforge.autosave");
        let mut app = Studio::empty();
        app.session_autosave = Some(path.clone());
        app.execute(Command::AddRectangle {
            name: "Last-second panel".into(),
            origin: [0., 0.],
            width: 0.4,
            height: 0.6,
        });
        let older = app.editor.project.clone();
        let (release, wait) = mpsc::channel();
        let (sender, receiver) = mpsc::channel();
        let older_path = path.clone();
        let writer = std::thread::spawn(move || {
            wait.recv().unwrap();
            sender.send(write_autosave(&older, &older_path)).unwrap();
        });
        app.autosave_job = Some(AutosaveJob { receiver });
        let piece = app.editor.project.pieces[0].id;
        let curve = app.editor.project.pieces[0].boundary[2];
        app.execute(Command::SetGrain { piece, angle: 0.37 });
        app.select_curve(Some((piece, curve)));
        let mut draft = InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve);
        draft.curve.as_mut().unwrap().controls = Some([[0.3, 0.8], [0.1, 0.8]]);
        app.draft = Some(draft);
        let mut expected = app.editor.project.clone();
        expected
            .curves
            .iter_mut()
            .find(|item| item.id == curve)
            .unwrap()
            .controls = Some([[0.3, 0.8], [0.1, 0.8]]);
        expected.revision += 1;
        let (finished, result) = mpsc::channel();
        let shutdown = std::thread::spawn(move || {
            eframe::App::on_exit(&mut app);
            finished.send(app.editor.project).unwrap();
        });
        assert!(matches!(
            result.recv_timeout(Duration::from_millis(50)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        release.send(()).unwrap();
        assert_eq!(
            result.recv_timeout(Duration::from_secs(3)).unwrap(),
            expected
        );
        shutdown.join().unwrap();
        writer.join().unwrap();
        assert_eq!(Project::load(&path).unwrap(), expected);
        let mut reopened = Studio::empty();
        reopened.recover(path);
        assert_eq!(reopened.editor.project, expected);
        assert!(reopened.dirty);
        assert!(reopened.path.is_none());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exit_preserves_primary_clean_and_discarded_states_and_uses_unique_fallback() {
        let dir = exit_test_directory("fallback");
        let primary = dir.join("garment.weftforge");
        let sidecar = append_suffix(&primary, ".autosave");
        let mut app = Studio::empty();
        app.execute(Command::AddRectangle {
            name: "Saved panel".into(),
            origin: [0., 0.],
            width: 0.4,
            height: 0.6,
        });
        app.editor.project.save(&primary).unwrap();
        app.path = Some(primary.clone());
        app.session_autosave = Some(dir.join("Recovery/Untitled.weftforge.autosave"));
        app.dirty = false;
        let primary_bytes = fs::read(&primary).unwrap();
        assert_eq!(app.retain_on_exit().unwrap(), None);
        assert!(!sidecar.exists());
        app.selected = Some(app.editor.project.pieces[0].id);
        let mut draft = InspectorDraft::new(&app.editor.project, app.selected, None);
        draft.grain_angle = Some(0.37);
        app.draft = Some(draft);
        assert_eq!(app.retain_on_exit().unwrap(), Some(sidecar.clone()));
        assert_eq!(Project::load(&sidecar).unwrap(), app.editor.project);
        assert_eq!(app.editor.project.pieces[0].grain_angle, 0.37);
        assert_eq!(fs::read(&primary).unwrap(), primary_bytes);
        let sidecar_bytes = fs::read(&sidecar).unwrap();
        let mut draft = InspectorDraft::new(&app.editor.project, app.selected, None);
        draft.grain_angle = Some(0.8);
        app.draft = Some(draft);
        app.discard_on_exit = true;
        app.dirty = false;
        assert_eq!(app.retain_on_exit().unwrap(), None);
        assert_eq!(fs::read(&sidecar).unwrap(), sidecar_bytes);
        assert_eq!(app.editor.project.pieces[0].grain_angle, 0.37);
        app.discard_on_exit = false;
        app.draft = None;
        let piece = app.editor.project.pieces[0].id;
        app.execute(Command::SetGrain { piece, angle: 0.6 });
        fs::write(&sidecar, b"invalid prior sidecar").unwrap();
        let retained = app.retain_on_exit().unwrap().unwrap();
        assert_ne!(retained, sidecar);
        assert!(
            retained
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("Exit-")
        );
        assert!(
            retained
                .file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".autosave")
        );
        assert_eq!(Project::load(&retained).unwrap(), app.editor.project);
        let second = app.retain_on_exit().unwrap().unwrap();
        assert_ne!(second, retained);
        assert_eq!(Project::load(second).unwrap(), app.editor.project);
        assert_eq!(fs::read(&sidecar).unwrap(), b"invalid prior sidecar");
        assert_eq!(fs::read(&primary).unwrap(), primary_bytes);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exit_retains_last_valid_geometry_and_reports_when_all_recovery_paths_fail() {
        let dir = exit_test_directory("failed");
        let primary = dir.join("garment.weftforge");
        let mut app = Studio::empty();
        app.execute(Command::AddRectangle {
            name: "Valid panel".into(),
            origin: [0., 0.],
            width: 0.4,
            height: 0.6,
        });
        app.path = Some(primary.clone());
        app.editor.project.save(&primary).unwrap();
        let before = app.editor.project.clone();
        let primary_bytes = fs::read(&primary).unwrap();
        app.selected = Some(app.editor.project.pieces[0].id);
        let mut draft = InspectorDraft::new(&app.editor.project, app.selected, None);
        draft.grain_angle = Some(f64::NAN);
        app.draft = Some(draft);
        let sidecar = app.retain_on_exit().unwrap().unwrap();
        assert_eq!(app.editor.project, before);
        assert_eq!(Project::load(&sidecar).unwrap(), before);
        fs::write(&sidecar, b"preserve this invalid sidecar").unwrap();
        let blocked = dir.join("blocked-directory");
        fs::write(&blocked, b"regular file").unwrap();
        app.session_autosave = Some(blocked.join("Untitled.weftforge.autosave"));
        let error = app.retain_on_exit().unwrap_err();
        assert!(error.contains("Existing project is invalid"));
        assert!(error.contains("Exit-"));
        assert_eq!(app.editor.project, before);
        assert!(app.dirty);
        assert_eq!(
            fs::read(&sidecar).unwrap(),
            b"preserve this invalid sidecar"
        );
        assert_eq!(fs::read(&primary).unwrap(), primary_bytes);
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
