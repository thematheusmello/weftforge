use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::{
    collections::BTreeMap,
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
    Command, Curve, CurveInterval, DxfPatternImport, Editor, Id, Material, Mesh, Placement,
    Project, SewingGroup, SvgPatternImport,
};
use weftforge_sim::{Cloth, MaterialAxis, SimulationError, StepSettings};

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Select,
    Transform,
    Polygon,
    Rectangle,
    Sew,
    SewingPaths,
}
impl Tool {
    fn label(self) -> &'static str {
        match self {
            Self::Select => "Select / edit",
            Self::Transform => "Transform pattern",
            Self::Polygon => "Polygon",
            Self::Rectangle => "Rectangle",
            Self::Sew => "Sew edges",
            Self::SewingPaths => "M:N sewing",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum PositionSnap {
    Off,
    Mm1,
    Mm5,
    Mm10,
}
impl PositionSnap {
    fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Mm1 => "1 mm",
            Self::Mm5 => "5 mm",
            Self::Mm10 => "10 mm",
        }
    }
    fn apply(self, value: f64) -> f64 {
        let factor = match self {
            Self::Off => return value,
            Self::Mm1 => 1000.0,
            Self::Mm5 => 200.0,
            Self::Mm10 => 100.0,
        };
        (value * factor).round() / factor
    }
}
#[derive(Clone, Copy)]
enum ExportFormat {
    Obj,
    Svg,
    Dxf,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum JobKind {
    Mesh,
    Simulation,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum JobStatus {
    Accepted,
    Cancelled,
    Discarded,
    Failed,
}
impl JobStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Accepted => "Accepted",
            Self::Cancelled => "Cancelled · result discarded",
            Self::Discarded => "Discarded · obsolete input revision",
            Self::Failed => "Failed · previous state retained",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
enum JobError {
    Cancelled,
    Failed(String),
}
#[derive(Clone, Debug, PartialEq)]
struct FinishedJob {
    id: u64,
    revision: u64,
    kind: JobKind,
    progress: Option<(u64, usize)>,
    status: JobStatus,
    outcome: Result<(), JobError>,
    cancel_requested: bool,
    obsolete: bool,
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
    id: u64,
    revision: u64,
    kind: JobKind,
    batch: Option<SimulationBatch>,
    completed_steps: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Result<JobOutput, JobError>>,
}
impl Job {
    fn spawn(
        ctx: &egui::Context,
        revision: u64,
        kind: JobKind,
        work: impl FnOnce(&AtomicBool, &AtomicU64) -> Result<JobOutput, JobError> + Send + 'static,
    ) -> Self {
        static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let completed_steps = Arc::new(AtomicU64::new(0));
        let progress = completed_steps.clone();
        let ctx = ctx.clone();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let result = check_cancelled(&flag)
                .and_then(|()| work(&flag, &progress))
                .and_then(|output| check_cancelled(&flag).map(|()| output));
            let _ = sender.send(result);
            ctx.request_repaint();
        });
        Self {
            id,
            revision,
            kind,
            batch: None,
            completed_steps,
            cancel,
            receiver,
        }
    }
    fn progress(&self) -> Option<(u64, usize)> {
        self.batch
            .map(|batch| (self.completed_steps.load(Ordering::Relaxed), batch.steps))
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
struct TransformDraft {
    piece: Id,
    revision: u64,
    translation: [f64; 2],
    angle: f64,
    pivot: [f64; 2],
    focus: Option<egui::Id>,
    inspector_text: Option<(String, Option<egui::text_edit::TextEditState>)>,
    field_focus: Option<egui::Id>,
    invalid_text: [Option<String>; 5],
    epoch: u64,
    error: Option<String>,
}
impl TransformDraft {
    fn command(&self) -> Command {
        Command::TransformPiece {
            piece: self.piece,
            translation: self.translation,
            angle: self.angle,
            pivot: self.pivot,
        }
    }
    fn is_identity(&self) -> bool {
        self.translation == [0.; 2] && self.angle == 0.
    }
    fn restore_inspector_text(&self, ctx: &egui::Context) {
        if let (Some(id), Some((text, state))) = (self.focus, &self.inspector_text) {
            ctx.data_mut(|data| data.insert_temp(id, text.clone()));
            if let Some(state) = state {
                state.clone().store(ctx, id);
            }
        }
    }
    fn close_without_commit(&self, ctx: &egui::Context) {
        self.restore_inspector_text(ctx);
        finish_numeric_action(ctx, self.focus, false);
        if self.inspector_text.is_some() {
            let closed = ctx.cumulative_frame_nr();
            ctx.data_mut(|data| {
                data.insert_temp(
                    egui::Id::new("transform-cancel-input"),
                    (closed, self.clone()),
                )
            });
        }
    }
}

struct TransformGesture {
    original: Project,
    pending: Vec<Command>,
    edit: TransformDraft,
    start: [f64; 2],
    snap: PositionSnap,
    rotating: bool,
}
impl TransformGesture {
    fn update(&mut self, pointer: [f64; 2]) {
        if self.rotating {
            let a: [f64; 2] = std::array::from_fn(|axis| self.start[axis] - self.edit.pivot[axis]);
            let b: [f64; 2] = std::array::from_fn(|axis| pointer[axis] - self.edit.pivot[axis]);
            self.edit.angle = (a[0] * b[1] - a[1] * b[0]).atan2(a[0] * b[0] + a[1] * b[1]);
        } else {
            self.edit.translation =
                std::array::from_fn(|axis| self.snap.apply(pointer[axis] - self.start[axis]));
        }
    }
    fn preview(&self) -> Result<Project, String> {
        let mut editor = Editor::new(self.original.clone()).map_err(|error| error.to_string())?;
        let mut commands = self.pending.clone();
        commands.push(self.edit.command());
        editor
            .execute(Command::Transaction(commands))
            .map_err(|error| error.to_string())?;
        Ok(editor.project)
    }
}

#[derive(Default)]
struct OrderedSewingDraft {
    a: Vec<CurveInterval>,
    b: Vec<CurveInterval>,
    selecting_b: bool,
}

struct SewingGroupEdit {
    group: SewingGroup,
    focus: Option<egui::Id>,
    epoch: u64,
    error: Option<String>,
}

#[derive(Clone)]
struct InspectorDraft {
    revision: u64,
    selected: Option<Id>,
    curve: Option<Curve>,
    applied_length: Option<Result<f64, String>>,
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
            applied_length: selected_curve.map(|curve| {
                #[cfg(test)]
                authored_length_tests::record_query();
                project
                    .interval_length(CurveInterval::full(curve))
                    .map_err(|error| error.to_string())
            }),
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

#[derive(Default)]
struct LegacyLengthSnapshot {
    revision: u64,
    values: BTreeMap<Id, Result<[f64; 2], String>>,
}

fn length_mm(length: f64) -> String {
    format!("{:.3} mm", 1000. * length)
}

fn pair_lengths_mm([a, b]: [f64; 2]) -> String {
    format!(
        "A {}\nB {}\nB − A {:+.3} mm",
        length_mm(a),
        length_mm(b),
        1000. * (b - a)
    )
}

fn pair_length_feedback(ui: &mut egui::Ui, lengths: Option<Result<[f64; 2], &str>>) {
    // Keep three rows for unknown, error and measured values so lazy results do not move the scroll content.
    let text = match lengths {
        Some(Ok(lengths)) => pair_lengths_mm(lengths),
        Some(Err(error)) => format!(
            "Lengths unavailable:\n{}\n ",
            error.replace(['\r', '\n'], " ")
        ),
        None => "A — mm\nB — mm\nB − A — mm".into(),
    };
    for line in text.split('\n') {
        ui.add(egui::Label::new(egui::RichText::new(line).small()).truncate());
    }
}

fn applied_length_heading(ui: &mut egui::Ui, revision: u64) {
    ui.small(format!("Applied rest length · revision {revision}"));
    ui.small("Draft/preview edits are not included.");
}

#[derive(Clone, Copy)]
enum PatternFormat {
    Svg,
    Dxf,
}
impl PatternFormat {
    fn name(self) -> &'static str {
        match self {
            Self::Svg => "SVG",
            Self::Dxf => "DXF",
        }
    }
}
enum ImportedDrawing {
    Svg(SvgPatternImport),
    Dxf(DxfPatternImport),
}
impl ImportedDrawing {
    fn format(&self) -> PatternFormat {
        match self {
            Self::Svg(_) => PatternFormat::Svg,
            Self::Dxf(_) => PatternFormat::Dxf,
        }
    }
    fn piece_count(&self) -> usize {
        match self {
            Self::Svg(drawing) => drawing.piece_count(),
            Self::Dxf(drawing) => drawing.piece_count(),
        }
    }
    fn command(&self) -> Command {
        match self {
            Self::Svg(drawing) => drawing.command(),
            Self::Dxf(drawing) => drawing.command(),
        }
    }
    fn scope(&self) -> &'static str {
        match self {
            Self::Svg(_) => weftforge_core::SVG_IMPORT_SCOPE,
            Self::Dxf(_) => weftforge_core::DXF_IMPORT_SCOPE,
        }
    }
}
struct PatternImportDraft {
    drawing: ImportedDrawing,
    focused: Option<egui::Id>,
    error: Option<String>,
}

struct Studio {
    editor: Editor,
    mesh: Option<Arc<Mesh>>,
    cloth: Option<Arc<Cloth>>,
    job: Option<Job>,
    last_job: Option<FinishedJob>,
    needs_mesh: bool,
    pending_batch: Option<SimulationBatch>,
    last_stage: &'static str,
    draft: Option<InspectorDraft>,
    legacy_lengths: LegacyLengthSnapshot,
    playing: bool,
    selected: Option<Id>,
    selected_curve: Option<Id>,
    split_t: f64,
    tool: Tool,
    position_snap: PositionSnap,
    drag_snap: PositionSnap,
    polygon: Vec<[f64; 2]>,
    pattern_canvas: Option<egui::Id>,
    garment_canvas: Option<egui::Id>,
    drag_start: Option<[f64; 2]>,
    rectangle: Option<RectangleDraft>,
    transform: Option<TransformDraft>,
    transform_gesture: Option<TransformGesture>,
    pattern_import: Option<PatternImportDraft>,
    drag_point: Option<Id>,
    preview: Option<[f64; 2]>,
    seam_start: Option<Id>,
    sewing_draft: OrderedSewingDraft,
    sewing_edit: Option<SewingGroupEdit>,
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
        studio_style(&cc.egui_ctx);
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));
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
            last_job: None,
            needs_mesh: false,
            pending_batch: None,
            last_stage: "Rest arrangement",
            draft: None,
            legacy_lengths: LegacyLengthSnapshot::default(),
            playing: false,
            selected: None,
            selected_curve: None,
            split_t: 0.5,
            tool: Tool::Select,
            position_snap: PositionSnap::Mm10,
            drag_snap: PositionSnap::Mm10,
            polygon: vec![],
            pattern_canvas: None,
            garment_canvas: None,
            drag_start: None,
            rectangle: None,
            transform: None,
            transform_gesture: None,
            pattern_import: None,
            drag_point: None,
            preview: None,
            seam_start: None,
            sewing_draft: OrderedSewingDraft::default(),
            sewing_edit: None,
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
        let old_pieces = self.editor.project.pieces.len();
        let old_groups = self.editor.project.sewing_groups.len();
        match self.editor.execute(command) {
            Ok(()) => {
                self.dirty = true;
                self.message = "Construction updated; simulation invalidated.".into();
                if self.editor.project.pieces.len() < old_pieces
                    && self.editor.project.sewing_groups.len() < old_groups
                {
                    self.message = format!(
                        "Construction updated; {} dependent sewing groups removed; simulation invalidated.",
                        old_groups - self.editor.project.sewing_groups.len()
                    );
                }
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
    fn apply_inspector_action(
        &mut self,
        ctx: &egui::Context,
        command: Command,
        focused: Option<egui::Id>,
    ) -> bool {
        let mut commands = self
            .draft
            .as_ref()
            .filter(|draft| draft.revision == self.editor.project.revision)
            .map_or_else(Vec::new, |draft| draft.commands(&self.editor.project));
        commands.push(command);
        let command = if commands.len() == 1 {
            commands.pop().unwrap()
        } else {
            Command::Transaction(commands)
        };
        let succeeded = self.apply_command(command);
        finish_numeric_action(ctx, focused, succeeded);
        succeeded
    }

    fn reset_gestures(&mut self) {
        self.polygon.clear();
        self.seam_start = None;
        self.sewing_draft = OrderedSewingDraft::default();
        self.drag_point = None;
        self.preview = None;
        self.drag_start = None;
        self.rectangle = None;
        self.transform = None;
        self.transform_gesture = None;
    }
    fn add_ellipse(&mut self, ctx: &egui::Context, focused: Option<egui::Id>) {
        if self.apply_inspector_action(
            ctx,
            Command::AddEllipse {
                name: self.name.clone(),
                origin: self.coordinate,
                width: self.size[0],
                height: self.size[1],
            },
            focused,
        ) {
            self.select(ctx, self.editor.project.pieces.last().map(|piece| piece.id));
        }
    }

    fn set_tool(&mut self, tool: Tool) {
        if self.tool != tool {
            self.reset_gestures();
            self.tool = tool;
        }
    }

    fn confirm_sewing_side(&mut self, ctx: &egui::Context) {
        let draft = &self.sewing_draft;
        if if draft.selecting_b {
            draft.b.is_empty()
        } else {
            draft.a.is_empty()
        } {
            self.message =
                "Select at least one boundary on this side; sewing draft retained.".into();
            return;
        }
        if !draft.selecting_b {
            self.sewing_draft.selecting_b = true;
            self.message = "Side A confirmed. Select ordered boundaries for B, then Enter.".into();
            return;
        }
        let command = Command::AddSewingGroup {
            name: format!(
                "Sewing group {}",
                self.editor.project.sewing_groups.len() + 1
            ),
            a: draft.a.clone(),
            b: draft.b.clone(),
            correspondence: vec![[0., 0.], [1., 1.]],
        };
        let focused = self.inspector_focus(ctx);
        if self.apply_inspector_action(ctx, command, focused) {
            self.sewing_draft = OrderedSewingDraft::default();
        }
    }

    fn delete_sewing_span(&mut self) {
        let draft = &mut self.sewing_draft;
        if draft.selecting_b {
            if draft.b.pop().is_none() {
                draft.selecting_b = false;
                draft.a.pop();
            }
        } else {
            draft.a.pop();
        }
    }
    fn inspector_focus(&self, ctx: &egui::Context) -> Option<egui::Id> {
        let key = egui::Id::new("inspector-input-focus");
        let identity = (
            self.editor.project.revision,
            self.selected,
            self.selected_curve,
        );
        if let Some(id) = numeric_focus(ctx) {
            // Keep the edit text across a button's separate press/release frames.
            ctx.data_mut(|data| data.insert_temp(key, (identity, id)));
        }
        ctx.data_mut(|data| data.get_temp::<((u64, Option<Id>, Option<Id>), egui::Id)>(key))
            .filter(|(stamp, _)| *stamp == identity)
            .map(|(_, id)| id)
    }
    fn export_button(&self, ui: &mut egui::Ui, format: ExportFormat, label: &str) -> bool {
        let focused = self.inspector_focus(ui.ctx());
        let response = ui.button(label);
        let clicked = response.clicked();
        if response.is_pointer_button_down_on() || clicked {
            finish_numeric_action(ui.ctx(), focused, false);
            let frame = ui.ctx().cumulative_frame_nr();
            ui.ctx().data_mut(|data| {
                data.insert_temp(
                    egui::Id::new("export-request"),
                    (frame, clicked.then_some(format), focused),
                )
            });
        }
        clicked
    }
    fn import_button(&self, ui: &mut egui::Ui, format: PatternFormat, label: &str) -> bool {
        let focused = self.inspector_focus(ui.ctx());
        let response = ui.button(label);
        let clicked = response.clicked();
        if response.is_pointer_button_down_on() || clicked {
            finish_numeric_action(ui.ctx(), focused, false);
            let frame = ui.ctx().cumulative_frame_nr();
            ui.ctx().data_mut(|data| {
                data.insert_temp(
                    egui::Id::new("pattern-import-request"),
                    (frame, clicked.then_some(format), focused),
                )
            });
        }
        clicked
    }
    fn new_empty_button(&self, ui: &mut egui::Ui) -> bool {
        let focused = self.inspector_focus(ui.ctx());
        let response = ui.button("New Empty");
        let clicked = response.clicked();
        if response.is_pointer_button_down_on() || clicked {
            finish_numeric_action(ui.ctx(), focused, false);
            let frame = ui.ctx().cumulative_frame_nr();
            ui.ctx().data_mut(|data| {
                data.insert_temp(
                    egui::Id::new("new-empty-request"),
                    (frame, clicked, focused),
                )
            });
        }
        clicked
    }

    fn new_empty(&mut self, ctx: &egui::Context, focused: Option<egui::Id>) {
        let pending_text = focused
            .and_then(|id| ctx.data_mut(|data| data.get_temp::<String>(id)))
            .is_some_and(|text| {
                text.trim()
                    .parse::<f64>()
                    .ok()
                    .is_none_or(|value| !value.is_finite())
            });
        let pending_commands = self
            .draft
            .as_ref()
            .filter(|draft| draft.revision == self.editor.project.revision)
            .is_some_and(|draft| !draft.commands(&self.editor.project).is_empty());
        let gesture = !self.polygon.is_empty()
            || self.seam_start.is_some()
            || !self.sewing_draft.a.is_empty()
            || !self.sewing_draft.b.is_empty()
            || self.sewing_draft.selecting_b
            || self.drag_point.is_some()
            || self.preview.is_some()
            || self.drag_start.is_some()
            || self.rectangle.is_some()
            || self.transform.is_some()
            || self.transform_gesture.is_some()
            || self.sewing_edit.is_some()
            || self.pattern_import.is_some();
        let refusal = if pending_text || pending_commands {
            Some("Apply or cancel the current property edit, then save before creating New Empty.")
        } else if gesture {
            Some("Finish or cancel the current construction gesture before creating New Empty.")
        } else if self.dirty {
            Some("Save current edits before creating New Empty.")
        } else if self.autosave_job.is_some() {
            Some("Wait for the current autosave to finish before creating New Empty.")
        } else {
            None
        };
        if let Some(message) = refusal {
            self.message = message.into();
            finish_numeric_action(ctx, focused, false);
            return;
        }
        self.cancel();
        self.job = None;
        self.last_job = None;
        self.editor = Editor::default();
        self.path = None;
        self.dirty = false;
        self.discard_on_exit = false;
        self.selected = None;
        self.selected_curve = None;
        self.reset_gestures();
        self.sewing_edit = None;
        self.pattern_import = None;
        self.show_recovery = false;
        self.mesh = None;
        self.reset_simulation();
        self.rebuild();
        self.autosave = Instant::now();
        self.message =
            "New empty editable project · draw a pattern or enter dimensions in metres.".into();
        finish_numeric_action(ctx, focused, true);
    }

    fn select(&mut self, ctx: &egui::Context, selected: Option<Id>) -> bool {
        if self.tool == Tool::Transform {
            let focus = self.inspector_focus(ctx);
            let invalid_text = focus
                .and_then(|id| ctx.data_mut(|data| data.get_temp::<String>(id)))
                .is_some_and(|text| {
                    text.trim()
                        .parse::<f64>()
                        .ok()
                        .is_none_or(|value| !value.is_finite())
                });
            if self.selected != selected {
                // shortcut: one staged piece at a time; add cross-piece staging only with multiselection.
                if invalid_text
                    || self
                        .draft
                        .as_ref()
                        .is_some_and(|draft| !draft.commands(&self.editor.project).is_empty())
                {
                    self.message = "Apply or cancel the current property edit before selecting another pattern.".into();
                    finish_numeric_action(ctx, focus, false);
                    return false;
                }
                self.selected = selected;
                self.selected_curve = None;
                self.draft = None;
            }
            return true;
        }
        if self.selected != selected || self.selected_curve.is_some() {
            if !self.commit_draft() {
                return false;
            }
            self.selected = selected;
            self.selected_curve = None;
        }
        true
    }
    fn select_curve(&mut self, ctx: &egui::Context, selected: Option<(Id, Id)>) {
        let piece = selected.map(|(piece, _)| piece);
        let curve = selected.map(|(_, curve)| curve);
        if self.tool == Tool::Transform && self.selected != piece && !self.select(ctx, piece) {
            return;
        }
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
        self.transform = None;
        self.transform_gesture = None;
        self.playing = false;
        self.pending_batch = None;
        self.cancel();
        self.draft = None;
        self.legacy_lengths = LegacyLengthSnapshot {
            revision: self.editor.project.revision,
            values: BTreeMap::new(),
        };
        self.needs_mesh = !self.editor.project.pieces.is_empty();
        if !self.needs_mesh {
            self.mesh = None;
            self.reset_simulation();
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
            move |cancel, _| {
                let mesh =
                    cancellable_core_job(cancel, |cancelled| project.mesh_cancellable(cancelled))?;
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
        let mut job = Job::spawn(
            ctx,
            project.revision,
            JobKind::Simulation,
            move |cancel, progress| {
                let mut state = match cloth {
                    Some(state) => (*state).clone(),
                    None => cancellable_core_job(cancel, |cancelled| {
                        weftforge_core::simulation::create_cloth_cancellable(
                            &project, &mesh, cancelled,
                        )
                    })?,
                };
                check_cancelled(cancel)?;
                for _ in 0..batch.steps {
                    state
                        .step(settings, || cancel.load(Ordering::Relaxed))
                        .map_err(|error| match error {
                            SimulationError::Cancelled => JobError::Cancelled,
                            error => JobError::Failed(error.to_string()),
                        })?;
                    progress.fetch_add(1, Ordering::Relaxed);
                }
                Ok(JobOutput::Simulation(Arc::new(state)))
            },
        );
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
                Err(_) => Some(Err(JobError::Failed(
                    "Worker stopped; previous state retained.".into(),
                ))),
            });
        if let Some(result) = result {
            let job = self.job.take().expect("finished job");
            let cancel_requested = job.cancel.load(Ordering::Relaxed);
            let obsolete = job.revision != self.editor.project.revision;
            let can_accept = job.accepts(self.editor.project.revision);
            let status = match &result {
                Err(JobError::Failed(_)) => JobStatus::Failed,
                Err(JobError::Cancelled) => JobStatus::Cancelled,
                Ok(_) if !can_accept => {
                    if cancel_requested {
                        JobStatus::Cancelled
                    } else {
                        JobStatus::Discarded
                    }
                }
                Ok(_) => JobStatus::Accepted,
            };
            self.last_job = Some(FinishedJob {
                id: job.id,
                revision: job.revision,
                kind: job.kind,
                progress: job.progress(),
                status,
                outcome: result.as_ref().map(|_| ()).map_err(Clone::clone),
                cancel_requested,
                obsolete,
            });
            match result {
                Ok(JobOutput::Mesh(mesh)) if can_accept => {
                    self.mesh = Some(mesh);
                    self.cloth = None;
                    self.last_stage = "Rest arrangement";
                    self.message = "Rest mesh rebuilt from editable construction.".into();
                }
                Ok(JobOutput::Simulation(state)) if can_accept => {
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
                Err(JobError::Failed(e)) => {
                    self.message = e;
                    if can_accept {
                        self.playing = false;
                        self.pending_batch = None;
                    }
                }
                _ => {}
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
                self.selected_curve = None;
                self.reset_gestures();
                self.sewing_edit = None;
                self.mesh = None;
                self.reset_simulation();
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
    fn choose_import(
        &mut self,
        ctx: &egui::Context,
        format: PatternFormat,
        focused: Option<egui::Id>,
    ) {
        let path = choose_pattern_import_path(ctx, format);
        self.prepare_import(ctx, path.as_deref(), focused, format);
    }
    fn prepare_import(
        &mut self,
        ctx: &egui::Context,
        path: Option<&Path>,
        focused: Option<egui::Id>,
        format: PatternFormat,
    ) -> bool {
        let result = path
            .map(|path| match format {
                PatternFormat::Svg => SvgPatternImport::load(path).map(ImportedDrawing::Svg),
                PatternFormat::Dxf => DxfPatternImport::load(path).map(ImportedDrawing::Dxf),
            })
            .transpose();
        let prepared = matches!(result, Ok(Some(_)));
        self.message = match result {
            Ok(Some(drawing)) => {
                self.pattern_import = Some(PatternImportDraft {
                    drawing,
                    focused,
                    error: None,
                });
                format!("Review {} pattern scale before importing.", format.name())
            }
            Ok(None) => format!(
                "{} import cancelled; pending edits retained.",
                format.name()
            ),
            Err(error) => format!(
                "{} import failed; current document retained: {error}",
                format.name()
            ),
        };
        finish_numeric_action(ctx, focused, false);
        prepared
    }
    fn pattern_import_window(&mut self, ctx: &egui::Context) {
        let Some(mut import) = self.pattern_import.take() else {
            return;
        };
        let mut accept = false;
        let mut cancel = false;
        let response = egui::Modal::new(egui::Id::new("import-patterns")).show(ctx, |ui| {
            ui.set_width(480.);
            ui.heading(format!(
                "Import {} patterns",
                import.drawing.format().name()
            ));
            ui.label(format!(
                "Add {} editable patterns to this project.",
                import.drawing.piece_count()
            ));
            ui.label(match &import.drawing {
                ImportedDrawing::Svg(drawing) => format!(
                    "Physical scale: {} mm per SVG unit.",
                    drawing.metres_per_unit * 1000.0
                ),
                ImportedDrawing::Dxf(_) => {
                    "Physical scale: 1 mm per DXF unit ($INSUNITS=4).".into()
                }
            });
            ui.label(
                "Keep the drawing's coordinate origin and piece layout; pattern Y points upward.",
            );
            ui.label(import.drawing.scope());
            if let Some(error) = &import.error {
                ui.colored_label(Color32::LIGHT_RED, error);
            }
            ui.horizontal(|ui| {
                accept = ui.button("Import").clicked();
                cancel = ui.button("Cancel").clicked();
            });
        });
        if cancel || response.should_close() {
            finish_numeric_action(ctx, import.focused, false);
            self.message = format!(
                "{} import cancelled; pending edits retained.",
                import.drawing.format().name()
            );
            return;
        }
        if accept {
            if self.apply_inspector_action(ctx, import.drawing.command(), import.focused) {
                self.reset_gestures();
                self.select(ctx, self.editor.project.pieces.last().map(|piece| piece.id));
                self.message = format!(
                    "Imported {} editable {} patterns at physical scale. {}",
                    import.drawing.piece_count(),
                    import.drawing.format().name(),
                    import.drawing.scope()
                );
                return;
            }
            import.error = Some(self.message.clone());
        }
        self.pattern_import = Some(import);
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
    fn export(&mut self, ctx: &egui::Context, format: ExportFormat, focused: Option<egui::Id>) {
        let path = choose_export_path(ctx, format);
        self.export_to_path(ctx, format, path.as_deref(), focused);
    }
    fn export_to_path(
        &mut self,
        ctx: &egui::Context,
        format: ExportFormat,
        path: Option<&Path>,
        focused: Option<egui::Id>,
    ) -> bool {
        let result = (|| -> Result<bool, String> {
            let Some(path) = path else {
                return Ok(false);
            };
            export_path_guard(self.path.as_deref(), path)?;
            let commands = self
                .draft
                .as_ref()
                .filter(|draft| draft.revision == self.editor.project.revision)
                .map_or_else(Vec::new, |draft| draft.commands(&self.editor.project));
            let candidate = if commands.is_empty() {
                None
            } else {
                let mut editor = self.editor.clone();
                editor
                    .execute(Command::Transaction(commands))
                    .map_err(|error| error.to_string())?;
                Some(editor)
            };
            let project = candidate
                .as_ref()
                .map_or(&self.editor.project, |editor| &editor.project);
            match format {
                ExportFormat::Svg => project.export_svg(path),
                ExportFormat::Dxf => project.export_dxf(path),
                ExportFormat::Obj => {
                    if candidate.is_some() {
                        return Err("Finish pending edits with Tab, wait for the rest mesh, then retry OBJ export. Pending edits were retained.".into());
                    }
                    let mesh = self.mesh.as_ref().ok_or("Create a pattern first.")?;
                    if mesh.doc_revision != project.revision {
                        return Err("The rest mesh is rebuilding; export when it is ready.".into());
                    }
                    let mut output = (**mesh).clone();
                    if let Some(state) = &self.cloth {
                        output.positions = state.positions().to_vec();
                    }
                    output.export_obj(path)
                }
            }.map_err(|error| error.to_string())?;
            // The successful file replacement is the commit point for pending edits too.
            if let Some(editor) = candidate {
                self.editor = editor;
                self.dirty = true;
                self.rebuild();
            } else {
                self.draft = None;
            }
            Ok(true)
        })();
        let written = matches!(result, Ok(true));
        finish_numeric_action(ctx, focused, written);
        self.message = match result {
            Ok(false) => "Export cancelled; pending edits retained.".into(),
            Ok(true) => match format {
                ExportFormat::Dxf => "Exported DXF pattern boundaries in millimetres; sewing and materials stay in .weftforge.".into(),
                ExportFormat::Svg => "Exported SVG pattern boundaries in millimetres.".into(),
                ExportFormat::Obj => "Exported OBJ in metres.".into(),
            },
            Err(error) => error,
        };
        written
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
        egui::Window::new("Recover editable construction").enabled(self.rectangle.is_none() && self.transform.is_none() && self.sewing_edit.is_none() && self.pattern_import.is_none()).open(&mut open).default_width(520.0).show(ctx,|ui| {
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
    fn document_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .map_or_else(
                || "Untitled.weftforge".into(),
                |name| name.to_string_lossy().into_owned(),
            )
    }

    fn retain_inspector_input(&self, response: &egui::Response, open: bool) {
        if open {
            // Popup closes on Escape without consuming it; retain the numeric editor's text.
            response
                .ctx
                .input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        }
        let pointer = response.is_pointer_button_down_on()
            || response.clicked_by(egui::PointerButton::Primary);
        let key = response.id.with("retained-inspector-input");
        let identity = (
            self.editor.project.revision,
            self.selected,
            self.selected_curve,
        );
        let retained = response.ctx.data_mut(|data| {
            if open {
                data.get_temp::<((u64, Option<Id>, Option<Id>), egui::Id)>(key)
                    .filter(|(stamp, _)| *stamp == identity)
                    .map(|(_, id)| id)
            } else {
                data.remove::<((u64, Option<Id>, Option<Id>), egui::Id)>(key);
                None
            }
        });
        // A popup may suppress numeric focus; keyboard focus elsewhere ends that retention.
        let current = numeric_focus(&response.ctx);
        let focused = if pointer {
            self.inspector_focus(&response.ctx)
        } else if current.is_some() {
            current
        } else if response.ctx.memory(|memory| memory.focused()).is_none() {
            retained
        } else {
            None
        };
        if open {
            response.ctx.data_mut(|data| {
                if let Some(id) = focused {
                    data.insert_temp(key, (identity, id));
                } else {
                    data.remove::<((u64, Option<Id>, Option<Id>), egui::Id)>(key);
                }
            });
        }
        if pointer || ((response.clicked() || open) && focused.is_some()) {
            finish_numeric_action(&response.ctx, focused, false);
            let request = (
                response.ctx.cumulative_frame_nr(),
                self.editor.project.revision,
                self.selected,
                self.selected_curve,
                focused,
            );
            response
                .ctx
                .data_mut(|data| data.insert_temp(egui::Id::new("view-input-focus"), request));
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            let (response, menu) = egui::menu::MenuButton::new("File").ui(ui, |ui| {
                if self.new_empty_button(ui) {
                    ui.close();
                }
                if ui.button("Open…").clicked() {
                    self.choose_open();
                    ui.close();
                }
                if self.import_button(ui, PatternFormat::Svg, "Import SVG…") {
                    ui.close();
                }
                if self.import_button(ui, PatternFormat::Dxf, "Import DXF…") {
                    ui.close();
                }
                ui.separator();
                if ui.button("Save").clicked() {
                    self.save(false);
                    ui.close();
                }
                if ui.button("Save as…").clicked() {
                    self.save(true);
                    ui.close();
                }
                if ui.button("Recover…").clicked() {
                    self.refresh_recovery();
                    self.show_recovery = true;
                    ui.close();
                }
                ui.separator();
                if self.export_button(ui, ExportFormat::Obj, "Export OBJ") {
                    ui.close();
                }
                if self.export_button(ui, ExportFormat::Svg, "Export SVG") {
                    ui.close();
                }
                if self.export_button(ui, ExportFormat::Dxf, "Export DXF") {
                    ui.close();
                }
            });
            self.retain_inspector_input(&response, menu.is_some());
            let (response, menu) = egui::menu::MenuButton::new("Edit").ui(ui, |ui| {
                if ui.button("Undo").clicked() {
                    self.history(ui.ctx(), false);
                    ui.close();
                }
                if ui.button("Redo").clicked() {
                    self.history(ui.ctx(), true);
                    ui.close();
                }
                ui.separator();
                if ui.button("Commands ⌘K").clicked() {
                    self.search = true;
                    ui.close();
                }
            });
            self.retain_inspector_input(&response, menu.is_some());
            let (response, menu) = egui::menu::MenuButton::new("View").ui(ui, |ui| {
                self.frame_patterns_button(ui);
                ui.checkbox(&mut self.wireframe, "Wireframe");
            });
            self.retain_inspector_input(&response, menu.is_some());
            let (response, menu) = egui::menu::MenuButton::new("Help").ui(ui, |ui| {
                ui.label("WeftForge garment studio");
                ui.separator();
                ui.label("2D: scroll to zoom; middle drag to pan.");
                ui.label("Hold Alt / Option to bypass position snapping.");
                ui.label("Transform pattern: drag a boundary to translate or its handle to rotate.");
                ui.label("Transform keeps the arranged 3D surface and world warp fixed; placement rotation values may change.");
                ui.label("3D: drag to orbit; scroll to zoom.");
                ui.label("Space plays or pauses simulation.");
                ui.label("Enter confirms polygon or sewing; Escape cancels.");
            });
            self.retain_inspector_input(&response, menu.is_some());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let response = ui.button("Commands ⌘K");
                self.retain_inspector_input(&response, false);
                if response.clicked() {
                    self.search = true;
                }
                for (label, redo) in [("Redo", true), ("Undo", false)] {
                    let response = ui.button(label);
                    self.retain_inspector_input(&response, false);
                    if response.clicked() {
                        self.history(ui.ctx(), redo);
                    }
                }
                ui.add_space(8.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!(
                            "{}{}",
                            self.document_name(),
                            if self.dirty { " •" } else { "" }
                        ))
                        .weak(),
                    )
                    .truncate(),
                );
            });
        });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(self.tool.label());
            ui.separator();
            self.position_snap_control(ui);
            ui.separator();
            self.frame_patterns_button(ui);
            if self.tool == Tool::Transform {
                ui.separator();
                let response = ui.add_enabled(self.selected.is_some(), egui::Button::new("Transform pattern…"));
                self.retain_inspector_input(&response, false);
                if response.clicked() { self.open_transform(ui.ctx()); }
            }
            if self.tool == Tool::SewingPaths {
                ui.separator();
                ui.label(format!("Side {} · A {} / B {}", if self.sewing_draft.selecting_b { "B" } else { "A" }, self.sewing_draft.a.len(), self.sewing_draft.b.len()))
                    .on_hover_text("Select boundaries in order. Enter confirms a side; Delete removes the last span; Escape cancels.");
            }
        });
    }

    fn tool_rail(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        for tool in [
            Tool::Select,
            Tool::Transform,
            Tool::Polygon,
            Tool::Rectangle,
            Tool::Sew,
            Tool::SewingPaths,
        ] {
            if tool == Tool::Sew {
                ui.add_space(3.0);
                ui.separator();
                ui.add_space(3.0);
            }
            let response = tool_button(ui, tool, self.tool == tool);
            self.retain_inspector_input(&response, false);
            if response.clicked() {
                self.set_tool(tool);
            }
        }
    }

    fn document_tab(&self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(Color32::from_gray(56))
            .inner_margin(egui::Margin::symmetric(10, 2))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{}{}",
                        self.document_name(),
                        if self.dirty { " •" } else { "" }
                    ));
                    ui.weak(format!("rev {}", self.editor.project.revision));
                });
            });
    }
    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.dirty {
                ui.colored_label(Color32::from_rgb(217, 189, 120), "Unsaved");
                if ui.small_button("Discard and close").clicked() {
                    self.discard_on_exit = true;
                    self.draft = None;
                    self.dirty = false;
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ui.separator();
            }
            ui.small(format!(
                "rev {}  ·  {} pieces  ·  {} seams  ·  {} groups  ·  {} vertices",
                self.editor.project.revision,
                self.editor.project.pieces.len(),
                self.editor.project.seams.len(),
                self.editor.project.sewing_groups.len(),
                self.mesh.as_ref().map_or(0, |mesh| mesh.positions.len())
            ));
            ui.separator();
            if let Some(job) = &self.job {
                ui.spinner();
                ui.small(if job.cancel.load(Ordering::Relaxed) {
                    "Cancelling…"
                } else if job.kind == JobKind::Mesh {
                    "Meshing…"
                } else {
                    "Simulating…"
                });
            }
            if self.autosave_job.is_some() {
                ui.small("Autosaving…");
            }
            ui.add(egui::Label::new(egui::RichText::new(&self.message).small()).truncate())
                .on_hover_text(&self.message);
        });
    }
    fn simulation_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Simulation")
                    .small()
                    .strong()
                    .color(Color32::from_gray(160)),
            );
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
                self.message =
                    "Cancellation requested; previous accepted snapshot retained.".into();
            }
            let response = ui.checkbox(&mut self.wireframe, "Wireframe");
            self.retain_inspector_input(&response, false);
            ui.separator();
            let stage = if let Some(job) = &self.job {
                if let Some((completed, total)) = job.progress() {
                    format!(
                        "{} · {completed}/{total}",
                        job.batch.map_or(self.last_stage, |batch| batch.stage)
                    )
                } else {
                    "Building mesh…".into()
                }
            } else if self
                .last_job
                .as_ref()
                .is_some_and(|job| job.status == JobStatus::Failed)
            {
                "Task failed".into()
            } else {
                self.last_stage.into()
            };
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (response, menu) = egui::menu::MenuButton::new("Task details").ui(ui, |ui| {
                    ui.set_width(420.0);
                    self.task_details(ui);
                });
                self.retain_inspector_input(&response, menu.is_some());
                ui.add(egui::Label::new(&stage).truncate())
                    .on_hover_text(stage);
            });
        });
    }
    fn task_details(&self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let active=self.job.as_ref().and_then(|job|job.batch);
            let settings=active.map_or(self.settings,|batch|batch.settings);
            if let Some(job) = &self.job {
                if let Some((completed, total)) = job.progress() {
                    ui.label(format!(
                        "Stage: {} · {completed}/{total} steps",
                        active.map_or(self.last_stage, |batch| batch.stage),
                    ));
                } else {
                    ui.label(format!("Stage: {:?} · progress indeterminate", job.kind));
                }
            } else {
                ui.label(format!("Stage: {}", self.last_stage));
            }
            ui.label(format!("{}: dt {:.5} s · {} substeps · {} iterations · gravity {:?} m/s² · seam closure {:.2} m/s",if active.is_some() { "Active settings" } else { "Next step settings" },settings.dt,settings.substeps,settings.iterations,settings.gravity,settings.seam_closure_speed));
            if let Some(batch)=self.pending_batch { ui.colored_label(Color32::GOLD,format!("Queued: {} · {} steps",batch.stage,batch.steps)); }
        });
        ui.separator();
        if let Some(job) = &self.job {
            ui.label(format!(
                "Current: task {} · {:?} · input rev {} · {}",
                job.id,
                job.kind,
                job.revision,
                if job.cancel.load(Ordering::Relaxed) {
                    "Cancelling…"
                } else {
                    "Running"
                },
            ));
        }
        if let Some(job) = &self.last_job {
            ui.label(format!(
                "Last: task {} · {:?} · input rev {} · {}",
                job.id,
                job.kind,
                job.revision,
                job.status.label(),
            ));
            if let Some((completed, total)) = job.progress {
                ui.weak(format!("Completed steps: {completed}/{total}"));
            }
            ui.weak(format!(
                "Cancel requested: {} · obsolete input: {}",
                job.cancel_requested, job.obsolete,
            ));
            match &job.outcome {
                Ok(()) => {
                    ui.weak("Outcome: succeeded");
                }
                Err(JobError::Cancelled) => {
                    ui.weak("Outcome: cancelled");
                }
                Err(JobError::Failed(error)) => {
                    ui.label(format!("Failure: {error}"));
                }
            }
        }
    }
    fn inspector(&mut self, ui: &mut egui::Ui) {
        if self.legacy_lengths.revision != self.editor.project.revision {
            self.legacy_lengths = LegacyLengthSnapshot {
                revision: self.editor.project.revision,
                values: BTreeMap::new(),
            };
        }
        let scene_height = (ui.available_height() * 0.30).clamp(136.0, 260.0);
        egui::Panel::bottom("studio-scene")
            .exact_size(scene_height)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_gray(46))
                    .stroke(Stroke::new(1.0, Color32::from_gray(32))),
            )
            .show(ui, |ui| {
                pane_heading(ui, "Scene");
                egui::Frame::new().inner_margin(6).show(ui, |ui| {
                    ui.weak(format!(
                        "{} pattern pieces",
                        self.editor.project.pieces.len()
                    ));
                    egui::ScrollArea::vertical()
                        .id_salt("scene")
                        .show(ui, |ui| {
                            for piece in self.editor.project.pieces.clone() {
                                ui.scope_builder(
                                    egui::UiBuilder::new()
                                        .id(ui.make_persistent_id(("piece-row", piece.id))),
                                    |ui| {
                                        let selected = self.selected == Some(piece.id);
                                        let mut button = egui::Button::selectable(
                                            selected,
                                            egui::RichText::new(&piece.name)
                                                .color(Color32::from_gray(222)),
                                        )
                                        .min_size(Vec2::new(ui.available_width(), 25.0))
                                        .truncate();
                                        if selected {
                                            button = button.fill(Color32::from_gray(82)).stroke(
                                                Stroke::new(1.0, Color32::from_rgb(74, 143, 208)),
                                            );
                                        }
                                        let response = ui.add(button).on_hover_text(&piece.name);
                                        if self.tool == Tool::Transform {
                                            self.retain_inspector_input(&response, false);
                                        }
                                        if response.clicked() {
                                            self.select(ui.ctx(), Some(piece.id));
                                        }
                                    },
                                );
                            }
                        });
                });
            });
        pane_heading(ui, "Properties");
        let mut commit = false;
        let mut acting = self.search
            || self.transform.is_some()
            || self.transform_gesture.is_some()
            || (self.tool == Tool::Transform
                && ui.input(|input| input.pointer.any_down() || input.pointer.any_released())
                && self
                    .pattern_canvas
                    .into_iter()
                    .chain(self.garment_canvas)
                    .filter_map(|id| ui.ctx().read_response(id))
                    .any(|response| {
                        ui.input(|input| input.pointer.interact_pos())
                            .is_some_and(|pointer| response.rect.contains(pointer))
                    }))
            || self.tool == Tool::SewingPaths
            || self.sewing_edit.is_some()
            || self.pattern_import.is_some()
            || ui
                .ctx()
                .data_mut(|data| {
                    data.get_temp::<(u64, bool, Option<egui::Id>)>(egui::Id::new(
                        "new-empty-request",
                    ))
                })
                .is_some_and(|(frame, _, _)| frame == ui.ctx().cumulative_frame_nr())
            || ui
                .ctx()
                .data_mut(|data| {
                    data.get_temp::<(u64, Option<PatternFormat>, Option<egui::Id>)>(egui::Id::new(
                        "pattern-import-request",
                    ))
                })
                .is_some_and(|(frame, _, _)| frame == ui.ctx().cumulative_frame_nr())
            || ui
                .ctx()
                .data_mut(|data| {
                    data.get_temp::<(u64, Option<ExportFormat>, Option<egui::Id>)>(egui::Id::new(
                        "export-request",
                    ))
                })
                .is_some_and(|(frame, _, _)| frame == ui.ctx().cumulative_frame_nr());
        let focus_key = egui::Id::new("inspector-input-focus");
        let action_focus = self.inspector_focus(ui.ctx());
        egui::Frame::new().inner_margin(8).show(ui, |ui| {
        let _scroll=egui::ScrollArea::vertical().id_salt("inspector").show(ui,|ui| {
            if let Some(piece)=self.selected.and_then(|id|self.editor.project.piece(id)).cloned() {
                ui.label("Boundary curve · select here or in 2D");
                egui::ComboBox::from_id_salt(("boundary-curve",piece.id))
                    .selected_text(self.selected_curve.map_or("Choose boundary".into(),|id|format!("Curve #{id}")))
                    .show_ui(ui,|ui| {
                        for (index,id) in piece.boundary.iter().enumerate() {
                            if ui.selectable_label(self.selected_curve==Some(*id),format!("Boundary {} · curve #{id}",index+1)).clicked() { self.select_curve(ui.ctx(), Some((piece.id,*id))); }
                        }
                    });
            }
            let mut draft=self.draft.take().filter(|draft|draft.revision==self.editor.project.revision && draft.selected==self.selected && draft.curve.as_ref().map(|curve|curve.id)==self.selected_curve)
                .unwrap_or_else(||InspectorDraft::new(&self.editor.project,self.selected,self.selected_curve));
            let mut split=None;
            let mut join=None;
            let mut reverse=None;
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
                            ui.horizontal(|ui| { for value in point { commit|=numeric_commit(&ui.add(numeric_value(ui,value).speed(0.001))); } });
                        }
                    }
                    ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("split")),|ui| {
                        ui.horizontal(|ui| { ui.label("Split parameter t"); ui.add(numeric_value(ui,&mut self.split_t).speed(0.01)); });
                    });
                    ui.small("Interior curve parameter, not arc length. Sewing is partitioned without changing point pairs.");
                    if ui.button("Split selected curve").clicked() { split=Some(curve.id); }
                    let response=ui.button("Join with next boundary").on_hover_text("Preserve compatible straight or cubic geometry and sewing. The selected curve ID survives; unsafe joins are rejected.");
                    acting|=response.is_pointer_button_down_on();
                    if response.clicked() { join=Some(curve.id); }
                    ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("applied-length")),|ui| {
                        applied_length_heading(ui,draft.revision);
                        if let Some(length)=&draft.applied_length {
                            match length {
                                Ok(length)=>{ ui.small(length_mm(*length)); }
                                Err(error)=>{ ui.small(format!("Length unavailable: {error}")); }
                            }
                        }
                    });
                    ui.separator();
                    });
                }
                ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id(("piece",piece.id))),|ui| {
                if let Some(dimensions)=&mut draft.dimensions {
                    ui.horizontal(|ui| { ui.label("Size m"); for value in dimensions { commit|=numeric_commit(&ui.add(numeric_value(ui,value).speed(0.005))); } });
                }
                if let Some(placement)=&mut draft.placement {
                    ui.label("Arrange X / Y / Z m");
                    ui.horizontal(|ui| { for value in &mut placement.translation { commit|=numeric_commit(&ui.add(numeric_value(ui,value).speed(0.01))); } });
                    ui.label("Rotate X / Y / Z radians");
                    ui.horizontal(|ui| { for value in &mut placement.rotation { commit|=numeric_commit(&ui.add(numeric_value(ui,value).speed(0.02))); } });
                }
                if let Some(angle)=&mut draft.grain_angle {
                    ui.label("Warp angle radians");
                    ui.small("0 = +U (right); positive turns toward +V (up).");
                    commit|=numeric_commit(&ui.add(numeric_value(ui,angle).speed(0.02)));
                }
                if let Some(material)=&mut draft.material {
                    let count=self.editor.project.pieces.iter().filter(|piece|piece.material==material.id).count();
                    ui.label(format!("Shared fabric · {count} pieces")); ui.small("Changes affect every assigned piece. Uncalibrated approximation.");
                    ui.label("Mass/area kg/m²"); commit|=numeric_commit(&ui.add(numeric_value(ui,&mut material.areal_density).speed(0.01).range(0.001..=100.0).clamp_existing_to_range(false)));
                    ui.label("Warp stiffness N/m"); commit|=numeric_commit(&ui.add(numeric_value(ui,&mut material.warp_stiffness).speed(10.0).range(0.0..=1.0e8).clamp_existing_to_range(false)));
                    ui.label("Weft stiffness N/m"); commit|=numeric_commit(&ui.add(numeric_value(ui,&mut material.weft_stiffness).speed(10.0).range(0.0..=1.0e8).clamp_existing_to_range(false)));
                    ui.label("Bend N·m"); commit|=numeric_commit(&ui.add(numeric_value(ui,&mut material.bending_stiffness).speed(0.00001).range(0.0..=1.0e8).clamp_existing_to_range(false)));
                    // RGB conversion can round unchanged values while drawing the widget.
                    let mut color=material.color;
                    if ui.color_edit_button_rgb(&mut color).changed() { material.color=color; draft.color_pending=true; }
                    commit|=draft.color_pending && ui.input(|input|input.pointer.any_released());
                }
                });
                if ui.button("Delete selected").clicked() { self.draft=Some(draft); self.execute(Command::RemovePiece { piece:piece.id }); return; }
            }
            let mut add_rectangle = false;
            let mut add_ellipse = false;
            let mut close_polygon = false;
            // Explicit scopes keep auto-ID edit buffers stable as dynamic rows appear.
            ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("creation")),|ui| {
            ui.separator(); ui.strong("Create pattern"); ui.label("Pattern name"); ui.text_edit_singleline(&mut self.name);
            ui.horizontal(|ui| { ui.label("W / H m"); for value in &mut self.size { ui.add(numeric_value(ui,value).speed(0.01).range(0.01..=5.0)); } });
            ui.horizontal(|ui| { ui.label("X / Y m"); for value in &mut self.coordinate { ui.add(numeric_value(ui,value).speed(0.01)); } });
            if ui.button("Add rectangle").clicked() { acting=true; add_rectangle=true; }
            if ui.button("Add ellipse").clicked() { acting=true; add_ellipse=true; }
            if ui.button("Add polygon point").clicked() { self.set_tool(Tool::Polygon); self.polygon.push(self.coordinate); }
            if ui.button(format!("Close pattern ({} points)",self.polygon.len())).clicked() { acting=true; close_polygon=true; }
            });
            let mut remove=None;
            let mut group_action=None;
            let mut confirm_side=false;
            ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("sewing")),|ui| {
            ui.separator(); ui.strong("Sewing"); ui.checkbox(&mut self.reversed,"Reverse paired edge"); ui.label("A interval / B interval (0–1)");
            for range in &mut self.intervals { ui.horizontal(|ui| { for value in range { ui.add(numeric_value(ui,value).speed(0.01).range(0.0..=1.0)); } }); }
            ui.label(if self.seam_start.is_some() { "Click paired boundary in 2D." } else { "Sew edges: click two 2D boundaries." });
            ui.small("Intervals use curve parameter t, not arc length. Reverse swaps B's direction and keeps the normalized mapping.");
            for seam in &self.editor.project.seams {
                ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id(("seam",seam.id))),|ui| {
                    ui.strong(format!("{} · #{}",seam.name,seam.id));
                    ui.small(format!("A #{}: t {} → {}\nB #{}: t {} → {}",seam.a.curve,seam.a.t0,seam.a.t1,seam.b.curve,seam.b.t0,seam.b.t1));
                    ui.horizontal(|ui| {
                        let response=ui.small_button("Reverse").on_hover_text("Swap B's interval endpoints; preserve the seam identity, covered intervals and normalized correspondence.");
                        acting|=response.is_pointer_button_down_on();
                        if response.clicked() {
                            let mut reversed=seam.clone();
                            std::mem::swap(&mut reversed.b.t0,&mut reversed.b.t1);
                            reverse=Some(reversed);
                        }
                        if ui.small_button("Remove").clicked() { remove=Some(seam.id); }
                    });
                    let feedback=ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("applied-length")),|ui| {
                        applied_length_heading(ui,self.editor.project.revision);
                        pair_length_feedback(ui,self.legacy_lengths.values.get(&seam.id).map(|lengths|lengths.as_ref().copied().map_err(String::as_str)));
                    });
                    if ui.is_rect_visible(feedback.response.rect)
                        && let std::collections::btree_map::Entry::Vacant(entry)=self.legacy_lengths.values.entry(seam.id) {
                        #[cfg(test)]
                        authored_length_tests::record_query();
                        entry.insert(self.editor.project.seam_lengths(seam.id).map_err(|error|error.to_string()));
                        ui.ctx().request_repaint();
                    }
                });
            }
            });
            ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("ordered-sewing")),|ui| {
                ui.separator(); ui.strong("Ordered sewing groups");
                ui.small("Pairing: total authored arc length in metres. Span endpoints use curve parameter t.");
                if self.tool==Tool::SewingPaths {
                    let draft=&mut self.sewing_draft;
                    ui.colored_label(Color32::GOLD,format!("Select side {} · A {} spans / B {} spans",if draft.selecting_b { "B" } else { "A" },draft.a.len(),draft.b.len()));
                    ui.small("Click boundaries in traversal order; next span uses the A/B interval above and B direction setting. Enter confirms a side; Delete removes the latest span; Escape cancels.");
                    let selecting_b=draft.selecting_b;
                    let b_range=if self.reversed { [self.intervals[1][1],self.intervals[1][0]] } else { self.intervals[1] };
                    sewing_span_controls(ui,&self.editor.project,"A",&mut draft.a,self.selected_curve,(!selecting_b).then_some(self.intervals[0]));
                    sewing_span_controls(ui,&self.editor.project,"B",&mut draft.b,self.selected_curve,selecting_b.then_some(b_range));
                    ui.horizontal(|ui| {
                        confirm_side=ui.button(if selecting_b { "Create sewing group (Enter)" } else { "Confirm A (Enter)" }).clicked();
                        if ui.small_button("Delete last").clicked() { self.delete_sewing_span(); }
                    });
                    if ui.small_button("Cancel sewing (Escape)").clicked() { self.sewing_draft=OrderedSewingDraft::default(); }
                }
                let mesh=self.mesh.as_ref().filter(|mesh|mesh.doc_revision==self.editor.project.revision);
                let mesh_error=self.last_job.as_ref()
                    .filter(|job|job.kind==JobKind::Mesh && job.revision==self.editor.project.revision && self.job.is_none() && !self.needs_mesh)
                    .and_then(|job|match &job.outcome {
                        Err(JobError::Failed(error))=>Some(error.as_str()),
                        _ if job.status==JobStatus::Cancelled=>Some("Rest mesh job cancelled"),
                        _=>None,
                    });
                for group in &self.editor.project.sewing_groups {
                    ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id(("sewing-group",group.id))),|ui| {
                        ui.strong(format!("{} · group #{}",group.name,group.id));
                        ui.small(format!("A {} spans → B {} spans",group.a.len(),group.b.len()));
                        for (side,spans) in [("A",&group.a),("B",&group.b)] {
                            for (index,span) in spans.iter().enumerate() { ui.small(format!("{side}{} · #{} · t {} → {}",index+1,span.curve,span.t0,span.t1)); }
                        }
                        ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("applied-length")),|ui| {
                            applied_length_heading(ui,self.editor.project.revision);
                            if let Some(geometry)=mesh.and_then(|mesh|mesh.sewing_groups.iter().find(|geometry|geometry.id==group.id)) {
                                pair_length_feedback(ui,Some(Ok(geometry.lengths)));
                                ui.small(format!("{} paired portions",geometry.portions.len()));
                            } else {
                                pair_length_feedback(ui,mesh_error.map(Err));
                                ui.add(egui::Label::new(egui::RichText::new(if mesh_error.is_some() { "Paired portions unavailable" } else { "Lengths and guides rebuilding for this revision…" }).small()).truncate());
                            }
                        });
                        ui.horizontal(|ui| {
                            let edit=ui.small_button("Edit group");
                            acting|=edit.is_pointer_button_down_on();
                            if edit.clicked() {
                                acting=true;
                                self.sewing_edit=Some(SewingGroupEdit { group:group.clone(),focus:action_focus,epoch:ui.ctx().cumulative_frame_nr(),error:None });
                            }
                            let reverse=ui.small_button("Reverse B");
                            acting|=reverse.is_pointer_button_down_on();
                            if reverse.clicked() {
                                let mut group=group.clone();
                                reverse_sewing_side(&mut group.b);
                                group_action=Some(Command::UpdateSewingGroup { group });
                            }
                            let remove=ui.small_button("Remove group");
                            acting|=remove.is_pointer_button_down_on();
                            if remove.clicked() { group_action=Some(Command::RemoveSewingGroup { group:group.id }); }
                        });
                    });
                }
            });
            ui.scope_builder(egui::UiBuilder::new().id(ui.make_persistent_id("mesh-settings")),|ui| {
            ui.separator(); ui.label("Mesh edge length m"); commit|=numeric_commit(&ui.add(numeric_value(ui,&mut draft.resolution).speed(0.005).range(0.005..=0.5)));
            ui.label("Iterations / substeps"); ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.settings.iterations).range(1..=50));
                ui.add(egui::DragValue::new(&mut self.settings.substeps).range(1..=16));
            });
            if let Some(cloth)=&self.cloth {
                let diagnostics=cloth.diagnostics();
                ui.label(format!("Time {:.3} s\nAssembly remaining {:.4} m\nPeak material strain {:.2}%\nSeam gap {:.4} m\nContact error {:.4} m\nInitial intersections {}\nUncertain capsule pairs {}",cloth.simulation_time(),diagnostics.seam_assembly_remaining,100.*diagnostics.max_stretch_error,diagnostics.max_seam_error,diagnostics.max_contact_error,diagnostics.initial_intersections,diagnostics.capsule_surface_uncertain_pairs));
                if let Some(peak)=diagnostics.peak_strain
                    && let Some(mesh)=&self.mesh
                    && let Some(piece)=weftforge_core::simulation::peak_strain_piece(&self.editor.project,mesh,diagnostics) {
                    let axis=match peak.axis { MaterialAxis::Warp=>"Warp",MaterialAxis::Weft=>"Weft" };
                    ui.label(format!("{} · {axis} {:+.2}%",piece.name,100.*peak.signed_strain))
                        .on_hover_text("Material direction at the largest absolute strain. Positive means extension; negative means compression. This is not a principal-strain or shear bound.");
                }
            }
            });
            self.draft=Some(draft);
            if add_rectangle {
                self.execute(Command::AddRectangle { name:self.name.clone(),origin:self.coordinate,width:self.size[0],height:self.size[1] });
                self.select(ui.ctx(), self.editor.project.pieces.last().map(|piece|piece.id));
            }
            if add_ellipse { self.add_ellipse(ui.ctx(),action_focus); }
            if close_polygon { self.finish_polygon(); }
            if let Some(seam)=remove { self.execute(Command::RemoveSeam { seam }); }
            if let Some(curve)=split { self.execute(Command::SplitCurve { curve,t:self.split_t }); }
            if let Some(curve)=join { acting=true; self.apply_inspector_action(ui.ctx(),Command::JoinCurve { curve },action_focus); }
            if let Some(seam)=reverse { acting=true; self.apply_inspector_action(ui.ctx(),Command::UpdateSeam { seam },action_focus); }
            if let Some(command)=group_action { acting=true; self.apply_inspector_action(ui.ctx(),command,action_focus); }
            if confirm_side { acting=true; self.confirm_sewing_side(ui.ctx()); }
        });
        #[cfg(test)]
        ui.ctx().data_mut(|data|data.insert_temp(egui::Id::new("authored-length-scroll"),(_scroll.id,_scroll.content_size,_scroll.inner_rect,_scroll.state.offset)));
        });
        let opening_rectangle = self.tool == Tool::Rectangle
            && self
                .pattern_canvas
                .and_then(|id| ui.ctx().read_response(id))
                .is_some_and(|response| response.clicked());
        // Keep pending edits staged while the numeric constructor opens or remains open.
        if commit
            && !acting
            && self.rectangle.is_none()
            && self.transform.is_none()
            && self.transform_gesture.is_none()
            && !opening_rectangle
            && self.commit_draft()
            && let Some(id) = action_focus
        {
            // egui replays committed text on next-frame blur; rebuilt f64 bounds may differ.
            ui.ctx().data_mut(|data| data.remove::<String>(id));
        }
        if let Some((_, _, _, _, focused)) = ui
            .ctx()
            .data_mut(|data| {
                data.remove_temp::<(u64, u64, Option<Id>, Option<Id>, Option<egui::Id>)>(
                    egui::Id::new("view-input-focus"),
                )
            })
            .filter(|(frame, revision, selected, curve, _)| {
                *frame == ui.ctx().cumulative_frame_nr()
                    && *revision == self.editor.project.revision
                    && *selected == self.selected
                    && *curve == self.selected_curve
            })
        {
            // egui surrenders outside-click focus during rendering; restore before next-frame blur.
            finish_numeric_action(ui.ctx(), focused, false);
        }
        if !acting
            && !ui.input(|input| input.pointer.any_down())
            && !ui
                .ctx()
                .memory(|memory| memory.focused())
                .is_some_and(|id| {
                    ui.ctx()
                        .data_mut(|data| data.get_temp::<String>(id))
                        .is_some()
                })
        {
            ui.ctx().data_mut(|data| {
                data.remove::<((u64, Option<Id>, Option<Id>), egui::Id)>(focus_key)
            });
        }
    }
    fn to_screen(&self, p: [f64; 2], r: Rect) -> Pos2 {
        r.center()
            + Vec2::new(
                (p[0] - self.center[0]) as f32,
                -(p[1] - self.center[1]) as f32,
            ) * self.scale
    }
    fn to_pattern(&self, p: Pos2, r: Rect, snap: PositionSnap) -> [f64; 2] {
        let d = (p - r.center()) / self.scale;
        [
            snap.apply(self.center[0] + d.x as f64),
            snap.apply(self.center[1] - d.y as f64),
        ]
    }
    fn position_snap_control(&mut self, ui: &mut egui::Ui) {
        let focused = self.inspector_focus(ui.ctx());
        ui.horizontal(|ui| {
            ui.label("Snap");
            for snap in [
                PositionSnap::Off,
                PositionSnap::Mm1,
                PositionSnap::Mm5,
                PositionSnap::Mm10,
            ] {
                let response = ui.selectable_value(&mut self.position_snap, snap, egui::RichText::new(snap.label()).color(Color32::from_gray(222)))
                    .on_hover_text("Pointer positions only. Hold Alt / Option when starting a drag or clicking to bypass.");
                if response.is_pointer_button_down_on()
                    || response.clicked_by(egui::PointerButton::Primary)
                {
                    finish_numeric_action(ui.ctx(), focused, false);
                    let request = (
                        ui.ctx().cumulative_frame_nr(),
                        self.editor.project.revision,
                        self.selected,
                        self.selected_curve,
                        focused,
                    );
                    ui.ctx().data_mut(|data| {
                        data.insert_temp(egui::Id::new("view-input-focus"), request)
                    });
                }
            }
        });
    }
    fn frame_patterns_button(&self, ui: &mut egui::Ui) {
        let focused = self.inspector_focus(ui.ctx());
        let response = ui.add_enabled(
            !self.editor.project.pieces.is_empty(),
            egui::Button::new("Frame all").small(),
        );
        if response.is_pointer_button_down_on() || response.clicked() {
            finish_numeric_action(ui.ctx(), focused, false);
        }
        if response.clicked() {
            let frame = ui.ctx().cumulative_frame_nr();
            ui.ctx()
                .data_mut(|data| data.insert_temp(egui::Id::new("frame-all-patterns"), frame));
        }
    }
    fn frame_patterns(&mut self, rect: Rect) {
        let inset = rect.shrink(28.0);
        if !inset.is_finite() || inset.width() <= 0.0 || inset.height() <= 0.0 {
            return;
        }
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        for piece in &self.editor.project.pieces {
            let Ok([piece_min, piece_max]) = self.editor.project.piece_bounds(piece.id) else {
                return;
            };
            for axis in 0..2 {
                min[axis] = min[axis].min(piece_min[axis]);
                max[axis] = max[axis].max(piece_max[axis]);
            }
        }
        let center = std::array::from_fn(|axis| (min[axis] + max[axis]) * 0.5);
        let scale = ((inset.width() as f64 / (max[0] - min[0]))
            .min(inset.height() as f64 / (max[1] - min[1]))
            .min(3000.0)) as f32;
        if center.iter().all(|value| value.is_finite()) && scale.is_finite() && scale > 0.0 {
            self.center = center;
            self.scale = scale;
        }
    }
    fn transform_edit(&self, ctx: &egui::Context, piece: Id) -> Option<TransformDraft> {
        let [min, max] = self.editor.project.piece_bounds(piece).ok()?;
        let focus = self.inspector_focus(ctx);
        Some(TransformDraft {
            piece,
            revision: self.editor.project.revision,
            translation: [0.; 2],
            angle: 0.,
            pivot: std::array::from_fn(|axis| (min[axis] + max[axis]) * 0.5),
            focus,
            inspector_text: focus.and_then(|id| {
                ctx.data_mut(|data| data.get_temp::<String>(id))
                    .map(|text| (text, egui::TextEdit::load_state(ctx, id)))
            }),
            field_focus: None,
            invalid_text: std::array::from_fn(|_| None),
            epoch: ctx.cumulative_frame_nr(),
            error: None,
        })
    }
    fn open_transform(&mut self, ctx: &egui::Context) {
        let Some(edit) = self
            .selected
            .and_then(|piece| self.transform_edit(ctx, piece))
        else {
            self.message = "Select a pattern boundary to transform first.".into();
            return;
        };
        self.set_tool(Tool::Transform);
        self.transform_gesture = None;
        finish_numeric_action(ctx, edit.focus, false);
        self.transform = Some(edit);
    }
    fn discard_stale_transform(&mut self) {
        let stale = |edit: &TransformDraft| {
            self.tool != Tool::Transform
                || edit.revision != self.editor.project.revision
                || self.selected != Some(edit.piece)
                || self.editor.project.piece(edit.piece).is_none()
        };
        if self.transform.as_ref().is_some_and(stale) {
            self.transform = None;
        }
        if self
            .transform_gesture
            .as_ref()
            .is_some_and(|gesture| stale(&gesture.edit))
        {
            self.transform_gesture = None;
        }
    }
    fn apply_transform(&mut self, ctx: &egui::Context, edit: &TransformDraft) -> bool {
        if edit.revision != self.editor.project.revision {
            self.message = "Transform cancelled: construction revision changed.".into();
            return false;
        }
        let invalid_draft_text = edit
            .focus
            .and_then(|id| ctx.data_mut(|data| data.get_temp::<String>(id)))
            .is_some_and(|text| {
                text.trim()
                    .parse::<f64>()
                    .ok()
                    .is_none_or(|value| !value.is_finite())
            });
        if invalid_draft_text {
            self.message = "Correct the pending property value before transforming.".into();
            finish_numeric_action(ctx, edit.focus, false);
            return false;
        }
        self.apply_inspector_action(ctx, edit.command(), edit.focus)
    }
    fn transform_window(&mut self, ctx: &egui::Context) {
        self.discard_stale_transform();
        let Some(mut edit) = self.transform.take() else {
            return;
        };
        let prior_text = edit
            .field_focus
            .and_then(|id| ctx.data_mut(|data| data.get_temp::<String>(id)))
            .map(|text| (edit.field_focus.unwrap(), text));
        let enter = ctx.input(|input| input.key_pressed(egui::Key::Enter));
        let mut accept = false;
        let mut cancel = false;
        let mut fields = Vec::new();
        let response = egui::Modal::new(egui::Id::new(("transform-pattern", edit.piece, edit.revision, edit.epoch))).show(ctx, |ui| {
            ui.set_width(440.);
            ui.heading("Transform pattern");
            ui.label("Rotate counterclockwise around the original U/V pivot, then translate.");
            ui.small("Arranged 3D surface and world warp stay fixed. Placement rotation values may change; rest mesh and simulation rebuild.");
            let [translation_u, translation_v] = &mut edit.translation;
            let [pivot_u, pivot_v] = &mut edit.pivot;
            for (index, (label, value)) in [
                ("Translation U (m)", translation_u),
                ("Translation V (m)", translation_v),
                ("Angle (radians)", &mut edit.angle),
                ("Pivot U (m)", pivot_u),
                ("Pivot V (m)", pivot_v),
            ].into_iter().enumerate() {
                ui.horizontal(|ui| {
                    let label = ui.label(label);
                    let old_bits = value.to_bits();
                    let raw = edit.invalid_text[index].clone();
                    let mut widget = numeric_value(ui, value).speed(0.001);
                    if let Some(text) = raw.clone() { widget = widget.custom_formatter(move |_, _| text.clone()); }
                    let response = ui.add(widget).labelled_by(label.id);
                    let text = ctx.data_mut(|data| data.get_temp::<String>(response.id));
                    edit.invalid_text[index] = match text {
                        Some(text) if text.trim().parse::<f64>().ok().is_none_or(|value| !value.is_finite()) => Some(text),
                        Some(_) => None,
                        None if value.to_bits() != old_bits => None,
                        None => raw,
                    };
                    fields.push(response.id);
                });
            }
            ui.small("Default pivot: analytic bounds center. Enter another pivot above to use it.");
            if let Some(id) = numeric_focus(ctx).filter(|id| fields.contains(id)) {
                edit.field_focus = Some(id);
            }
            if let Some(error) = &edit.error { ui.colored_label(Color32::LIGHT_RED, error); }
            ui.horizontal(|ui| {
                let apply = ui.button("Apply transform");
                if apply.is_pointer_button_down_on() || apply.clicked() {
                    finish_numeric_action(ctx, edit.field_focus, false);
                }
                accept = apply.clicked();
                cancel = ui.button("Cancel transform").clicked();
            });
        });
        if cancel || response.should_close() {
            if let Some(id) = edit.field_focus {
                ctx.memory_mut(|memory| memory.surrender_focus(id));
            }
            edit.close_without_commit(ctx);
            return;
        }
        if accept || enter {
            let text = edit
                .field_focus
                .and_then(|id| {
                    ctx.data_mut(|data| data.get_temp::<String>(id))
                        .map(|text| (id, text))
                })
                .or(prior_text);
            let invalid_text = edit.invalid_text.iter().any(Option::is_some)
                || text.as_ref().is_some_and(|(_, text)| {
                    text.trim()
                        .parse::<f64>()
                        .ok()
                        .is_none_or(|value| !value.is_finite())
                });
            if invalid_text {
                self.message = "Enter numeric translation, angle and pivot values.".into();
            } else if edit.is_identity()
                && edit
                    .pivot
                    .iter()
                    .all(|value| value.is_finite() && value.abs() <= weftforge_core::MAX_COORDINATE)
            {
                clear_numeric_fields(ctx, &fields);
                edit.close_without_commit(ctx);
                return;
            } else if self.apply_transform(ctx, &edit) {
                clear_numeric_fields(ctx, &fields);
                return;
            }
            edit.error = Some(self.message.clone());
            if let Some((id, text)) = text {
                ctx.data_mut(|data| data.insert_temp(id, text));
            }
            finish_numeric_action(ctx, edit.field_focus, false);
        }
        edit.restore_inspector_text(ctx);
        self.transform = Some(edit);
    }
    fn restore_cancelled_transform_input(&self, ctx: &egui::Context) {
        let key = egui::Id::new("transform-cancel-input");
        if let Some((closed, edit)) = ctx.data_mut(|data| {
            let retained = data.get_temp::<(u64, TransformDraft)>(key);
            data.remove::<(u64, TransformDraft)>(key);
            retained
        }) && edit.revision == self.editor.project.revision
            && self.selected == Some(edit.piece)
        {
            let new_input = ctx.cumulative_frame_nr() > closed
                && ctx.input(|input| {
                    input.events.iter().any(|event| {
                        matches!(
                            event,
                            egui::Event::Key { pressed: true, .. }
                                | egui::Event::PointerButton { pressed: true, .. }
                                | egui::Event::AccessKitActionRequest(_)
                        )
                    })
                });
            if !new_input && ctx.cumulative_frame_nr() <= closed + 3 {
                // A closing modal blocks background focus for a pass, then DragValue gains focus again.
                edit.restore_inspector_text(ctx);
                finish_numeric_action(ctx, edit.focus, false);
                if ctx.cumulative_frame_nr() < closed + 3 {
                    ctx.data_mut(|data| data.insert_temp(key, (closed, edit)));
                }
            }
        }
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
                    self.select(ctx, self.editor.project.pieces.last().map(|piece| piece.id));
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
        self.discard_stale_transform();
        let gesture_paused = self.search
            || egui::Popup::is_any_open(ui.ctx())
            || ui
                .ctx()
                .data_mut(|data| {
                    data.get_temp::<(u64, bool, Option<egui::Id>)>(egui::Id::new(
                        "new-empty-request",
                    ))
                })
                .is_some_and(|(frame, _, _)| frame == ui.ctx().cumulative_frame_nr());
        pane_heading(ui, "2D Pattern").on_hover_text("Editable construction in metres. Alt / Option bypasses snap; scroll zooms; middle drag pans.");
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        self.pattern_canvas = Some(response.id);
        let r = response.rect;
        if ui
            .ctx()
            .data_mut(|data| data.remove_temp::<u64>(egui::Id::new("frame-all-patterns")))
            == Some(ui.ctx().cumulative_frame_nr())
        {
            self.frame_patterns(r);
        }
        painter.rect_filled(r, 0., Color32::from_gray(37));
        if response.hovered() {
            self.scale = (self.scale * (ui.input(|i| i.smooth_scroll_delta.y) * 0.001).exp())
                .clamp(f32::MIN_POSITIVE, 3000.);
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
                    Stroke::new(0.5, Color32::from_gray(51)),
                );
            }
        }
        let pointer = response
            .interact_pointer_pos()
            .or_else(|| ui.input(|i| i.pointer.hover_pos()));
        if !gesture_paused
            && let Some(position) =
                pointer.map(|pointer| self.to_pattern(pointer, r, PositionSnap::Off))
            && let Some(gesture) = &mut self.transform_gesture
            && ui.input(|input| {
                input.pointer.primary_down()
                    || input.pointer.button_released(egui::PointerButton::Primary)
            })
        {
            gesture.update(position);
        }
        let mut doc = self
            .transform_gesture
            .as_ref()
            .and_then(|gesture| gesture.preview().ok())
            .unwrap_or_else(|| self.editor.project.clone());
        if let (Some(id), Some(position)) = (self.drag_point, self.preview)
            && let Some(point) = doc.points.iter_mut().find(|p| p.id == id)
        {
            point.position = position;
        }
        let snap = if ui.input(|input| input.modifiers.alt) {
            PositionSnap::Off
        } else {
            self.position_snap
        };
        let mut nearest = None;
        let mut distance = 12_f32;
        for piece in &doc.pieces {
            let selected = self.selected == Some(piece.id);
            let color = if selected {
                Color32::from_rgb(92, 163, 225)
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
            let endpoint = |i: &CurveInterval, fraction| {
                doc.curve(i.curve)
                    .and_then(|c| c.evaluate(&doc, i.parameter(fraction)).ok())
                    .map(|p| self.to_screen(p, r))
            };
            for fraction in [0., 1.] {
                if let (Some(a), Some(b)) =
                    (endpoint(&seam.a, fraction), endpoint(&seam.b, fraction))
                {
                    painter.line_segment([a, b], Stroke::new(1., Color32::from_rgb(245, 175, 87)));
                }
            }
        }
        if let Some(mesh) = self
            .mesh
            .as_ref()
            .filter(|mesh| mesh.doc_revision == self.editor.project.revision)
        {
            let guide_project = if self.transform_gesture.is_some() {
                &doc
            } else {
                &self.editor.project
            };
            for geometry in &mesh.sewing_groups {
                for portion in &geometry.portions {
                    for (interval, color) in [
                        (&portion.a, Color32::from_rgb(100, 200, 255)),
                        (&portion.b, Color32::from_rgb(235, 145, 240)),
                    ] {
                        self.paint_sewing_interval(guide_project, &painter, interval, r, color);
                    }
                    for fraction in [0., 1.] {
                        let endpoint = |interval: &CurveInterval| {
                            guide_project
                                .curve(interval.curve)
                                .and_then(|curve| {
                                    curve
                                        .evaluate(guide_project, interval.parameter(fraction))
                                        .ok()
                                })
                                .map(|point| self.to_screen(point, r))
                        };
                        if let (Some(a), Some(b)) = (endpoint(&portion.a), endpoint(&portion.b)) {
                            painter.line_segment(
                                [a, b],
                                Stroke::new(1., Color32::from_rgb(125, 165, 235)),
                            );
                        }
                    }
                }
            }
        }
        if self.tool == Tool::SewingPaths {
            for (side, spans, color) in [
                ("A", &self.sewing_draft.a, Color32::from_rgb(100, 200, 255)),
                ("B", &self.sewing_draft.b, Color32::from_rgb(235, 145, 240)),
            ] {
                for (index, interval) in spans.iter().enumerate() {
                    self.paint_sewing_interval(&self.editor.project, &painter, interval, r, color);
                    if let Some(point) =
                        self.editor.project.curve(interval.curve).and_then(|curve| {
                            curve.evaluate(&self.editor.project, interval.t0).ok()
                        })
                    {
                        painter.text(
                            self.to_screen(point, r) + Vec2::new(4., 4.),
                            egui::Align2::LEFT_TOP,
                            format!("{side}{}", index + 1),
                            egui::FontId::proportional(12.),
                            color,
                        );
                    }
                }
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
                Rect::from_two_pos(
                    self.to_screen(start, r),
                    self.to_screen(self.to_pattern(pointer, r, self.drag_snap), r),
                ),
                0.,
                Stroke::new(1., Color32::GOLD),
                egui::StrokeKind::Inside,
            );
        }
        let mut handle_clicked = false;
        let transform_controls = if self.tool == Tool::Transform {
            self.selected.and_then(|piece| {
                let source = self
                    .transform_gesture
                    .as_ref()
                    .map_or(&self.editor.project, |gesture| &gesture.original);
                let [min, max] = source.piece_bounds(piece).ok()?;
                let pivot: [f64; 2] = std::array::from_fn(|axis| (min[axis] + max[axis]) * 0.5);
                let (translation, angle) = self
                    .transform_gesture
                    .as_ref()
                    .map_or(([0.; 2], 0.), |gesture| {
                        (gesture.edit.translation, gesture.edit.angle)
                    });
                let origin = self.to_screen(
                    std::array::from_fn(|axis| pivot[axis] + translation[axis]),
                    r,
                );
                let radius = (((max[1] - min[1]) * 0.5) as f32 * self.scale + 28.).max(100.);
                let handle = origin + Vec2::new(-angle.sin() as f32, -angle.cos() as f32) * radius;
                painter.line_segment([origin, handle], Stroke::new(1., Color32::GOLD));
                painter.circle_stroke(origin, 5., Stroke::new(1.5, Color32::GOLD));
                painter.line_segment(
                    [origin - Vec2::new(8., 0.), origin + Vec2::new(8., 0.)],
                    Stroke::new(1., Color32::GOLD),
                );
                painter.line_segment(
                    [origin - Vec2::new(0., 8.), origin + Vec2::new(0., 8.)],
                    Stroke::new(1., Color32::GOLD),
                );
                painter.text(
                    origin + Vec2::new(9., -9.),
                    egui::Align2::LEFT_BOTTOM,
                    "Pivot",
                    egui::FontId::proportional(11.),
                    Color32::GOLD,
                );
                let pivot_response = ui.interact(
                    Rect::from_center_size(origin, Vec2::splat(16.)),
                    response.id.with("transform-pivot"),
                    Sense::hover(),
                );
                pivot_response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Label,
                        ui.is_enabled(),
                        format!("Transform pivot U {} m, V {} m", pivot[0], pivot[1]),
                    )
                });
                let handle_rect = Rect::from_center_size(handle, Vec2::splat(24.));
                let handle_response = ui.interact(
                    handle_rect,
                    response
                        .id
                        .with(("rotation-handle", piece, self.editor.project.revision)),
                    Sense::click_and_drag(),
                );
                handle_response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Button,
                        ui.is_enabled(),
                        "Rotate pattern around bounds-center pivot",
                    )
                });
                painter.circle_filled(handle, 7., Color32::from_gray(37));
                painter.circle_stroke(
                    handle,
                    7.,
                    Stroke::new(
                        if handle_response.hovered() || handle_response.has_focus() {
                            2.5
                        } else {
                            1.5
                        },
                        Color32::GOLD,
                    ),
                );
                handle_clicked = handle_response.clicked();
                Some(handle_rect)
            })
        } else {
            None
        };
        if self.tool == Tool::Transform
            && self.transform.is_none()
            && self.transform_gesture.is_none()
            && !self.search
            && !egui::Popup::is_any_open(ui.ctx())
            && ui.is_enabled()
            && ui.rect_contains_pointer(r)
            && ui.input(|input| input.pointer.button_pressed(egui::PointerButton::Primary))
            && let Some(pointer) = pointer.filter(|pointer| r.contains(*pointer))
        {
            let rotating = transform_controls.is_some_and(|handle| handle.contains(pointer));
            let piece = if rotating {
                self.selected
            } else {
                nearest.map(|(piece, _)| piece)
            };
            if let Some(piece) = piece
                && self.select(ui.ctx(), Some(piece))
                && let Some(edit) = self.transform_edit(ui.ctx(), piece)
            {
                finish_numeric_action(ui.ctx(), edit.focus, false);
                self.transform_gesture = Some(TransformGesture {
                    original: self.editor.project.clone(),
                    pending: self
                        .draft
                        .as_ref()
                        .map_or_else(Vec::new, |draft| draft.commands(&self.editor.project)),
                    edit,
                    start: self.to_pattern(pointer, r, PositionSnap::Off),
                    snap,
                    rotating,
                });
            }
        }
        let mut transformed = false;
        if !gesture_paused
            && ui.input(|input| input.pointer.button_released(egui::PointerButton::Primary))
            && let Some(mut gesture) = self.transform_gesture.take()
            && !gesture.edit.is_identity()
        {
            transformed = true;
            if !self.apply_transform(ui.ctx(), &gesture.edit) {
                gesture.edit.error = Some(self.message.clone());
                self.transform = Some(gesture.edit);
            }
        }
        if handle_clicked && !transformed && self.transform.is_none() {
            self.open_transform(ui.ctx());
        }
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(p) = pointer
        {
            self.drag_snap = snap;
            if self.tool == Tool::Rectangle {
                self.drag_start = Some(self.to_pattern(p, r, self.drag_snap));
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
        if !gesture_paused
            && response.dragged_by(egui::PointerButton::Primary)
            && self.drag_point.is_some()
        {
            self.preview = pointer.map(|p| self.to_pattern(p, r, self.drag_snap));
        }
        if !gesture_paused && response.drag_stopped_by(egui::PointerButton::Primary) {
            if let (Some(point), Some(position)) = (self.drag_point.take(), self.preview.take()) {
                self.execute(Command::MovePoint { point, position });
            }
            if let (Some(start), Some(p)) = (self.drag_start.take(), pointer) {
                let end = self.to_pattern(p, r, self.drag_snap);
                self.execute(Command::AddRectangle {
                    name: self.name.clone(),
                    origin: [start[0].min(end[0]), start[1].min(end[1])],
                    width: (end[0] - start[0]).abs(),
                    height: (end[1] - start[1]).abs(),
                });
                self.select(ui.ctx(), self.editor.project.pieces.last().map(|p| p.id));
            }
        }
        if response.clicked() {
            match self.tool {
                Tool::Select => self.select_curve(ui.ctx(), nearest),
                Tool::Transform => {
                    if let Some((piece, _)) = nearest {
                        self.select(ui.ctx(), Some(piece));
                    }
                }
                Tool::Polygon => {
                    if let Some(p) = pointer {
                        self.polygon.push(self.to_pattern(p, r, snap));
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
                Tool::SewingPaths => {
                    if let Some((_, curve)) = nearest {
                        let draft = &mut self.sewing_draft;
                        let side = usize::from(draft.selecting_b);
                        let [t0, t1] = self.intervals[side];
                        let interval = if draft.selecting_b && self.reversed {
                            CurveInterval {
                                curve,
                                t0: t1,
                                t1: t0,
                            }
                        } else {
                            CurveInterval { curve, t0, t1 }
                        };
                        if draft.selecting_b {
                            draft.b.push(interval);
                        } else {
                            draft.a.push(interval);
                        }
                    }
                }
                Tool::Rectangle => {
                    if let Some(p) = pointer {
                        // shortcut: existing snapped lower-left anchor; reference equivalence needs observation.
                        self.rectangle = Some(RectangleDraft {
                            origin: self.to_pattern(p, r, snap),
                            dimensions_mm: ["500".into(), "500".into()],
                            error: None,
                        });
                    }
                }
            }
        }
    }
    fn paint_sewing_interval(
        &self,
        project: &Project,
        painter: &egui::Painter,
        interval: &CurveInterval,
        rect: Rect,
        color: Color32,
    ) {
        let Some(curve) = project.curve(interval.curve) else {
            return;
        };
        let points: Vec<_> = (0..=24)
            .filter_map(|index| {
                curve
                    .evaluate(project, interval.parameter(index as f64 / 24.))
                    .ok()
            })
            .map(|point| self.to_screen(point, rect))
            .collect();
        if let [.., previous, end] = points.as_slice() {
            painter.arrow(*previous, *end - *previous, Stroke::new(2., color));
        }
        painter.add(egui::Shape::line(points, Stroke::new(2., color)));
    }

    fn sewing_group_window(&mut self, ctx: &egui::Context) {
        let Some(mut edit) = self.sewing_edit.take() else {
            return;
        };
        let mut apply = false;
        let mut cancel = false;
        let response = egui::Modal::new(egui::Id::new(("edit-sewing-group", edit.group.id, edit.epoch))).show(ctx, |ui| {
            ui.set_width(620.);
            ui.heading(format!("Edit sewing group #{}", edit.group.id));
            ui.horizontal(|ui| { let label=ui.label("Name"); ui.text_edit_singleline(&mut edit.group.name).labelled_by(label.id); });
            ui.small("Pairing uses normalized total authored arc length. Each span keeps its own curve parameter endpoints and direction; gaps add no length.");
            egui::ScrollArea::vertical().id_salt("group-fields").max_height(420.).show(ui, |ui| {
                let b_fields=ui.columns(2, |columns| {
                    sewing_span_controls(&mut columns[0], &self.editor.project, "A", &mut edit.group.a, self.selected_curve, Some([0.,1.]));
                    sewing_span_controls(&mut columns[1], &self.editor.project, "B", &mut edit.group.b, self.selected_curve, Some([0.,1.]))
                });
                if ui.button("Reverse B order and directions").clicked() {
                    clear_numeric_fields(ctx,&b_fields);
                    reverse_sewing_side(&mut edit.group.b);
                }
                ui.separator();
                ui.strong("Easing map · normalized A length / B length");
                let mut remove = None;
                let mut map_fields=Vec::new();
                let last = edit.group.correspondence.len() - 1;
                for (index, pair) in edit.group.correspondence.iter_mut().enumerate() {
                    ui.push_id(("map", index), |ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("Knot {}", index + 1));
                            for (side,value) in ["A","B"].into_iter().zip(pair) {
                                let label=ui.label(side);
                                map_fields.push(ui.add(numeric_value(ui, value).speed(0.01)).labelled_by(label.id).id);
                            }
                            if index > 0 && index < last && ui.small_button("Remove knot").clicked() { remove = Some(index); }
                        });
                    });
                }
                let add=ui.button("Add easing knot").clicked();
                if remove.is_some() || add {
                    clear_numeric_fields(ctx,&map_fields);
                }
                if let Some(index) = remove { edit.group.correspondence.remove(index); }
                if add {
                    let map = &mut edit.group.correspondence;
                    let last = map.len() - 1;
                    map.insert(last, std::array::from_fn(|axis| (map[last - 1][axis] + map[last][axis]) * 0.5));
                }
            });
            if let Some(error) = &edit.error { ui.colored_label(Color32::LIGHT_RED, error); }
            ui.horizontal(|ui| {
                apply = ui.button("Apply group").clicked();
                cancel = ui.button("Cancel group edit").clicked();
            });
        });
        if cancel || response.should_close() {
            if let Some(id) = numeric_focus(ctx) {
                ctx.memory_mut(|memory| memory.surrender_focus(id));
            }
            finish_numeric_action(ctx, edit.focus, false);
            return;
        }
        if apply {
            if self.apply_inspector_action(
                ctx,
                Command::UpdateSewingGroup {
                    group: edit.group.clone(),
                },
                edit.focus,
            ) {
                return;
            }
            edit.error = Some(self.message.clone());
        }
        self.sewing_edit = Some(edit);
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
        pane_heading(ui, "3D Garment").on_hover_text("Y up · metres. Drag to orbit; scroll to zoom; click a triangle to select its pattern piece.");
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        self.garment_canvas = Some(response.id);
        let r = response.rect;
        painter.rect_filled(r, 0., Color32::from_gray(37));
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
                    Stroke::new(0.5, Color32::from_rgb(112, 161, 208))
                } else {
                    Stroke::NONE
                };
                painter.add(egui::Shape::convex_polygon(
                    points.to_vec(),
                    color,
                    Stroke::NONE,
                ));
                if stroke != Stroke::NONE {
                    for segment in [
                        [points[0], points[1]],
                        [points[1], points[2]],
                        [points[2], points[0]],
                    ] {
                        painter.line_segment(segment, stroke);
                    }
                }
                if response.clicked()
                    && response
                        .interact_pointer_pos()
                        .is_some_and(|p| point_in_triangle(p, points))
                {
                    picked = Some(id);
                }
            }
            if picked.is_some() {
                self.select(ui.ctx(), picked);
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
            Color32::from_gray(178),
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
        self.discard_stale_transform();
        let authoring_modal = self.rectangle.is_some()
            || self.transform.is_some()
            || self.sewing_edit.is_some()
            || self.pattern_import.is_some();
        let global_shortcuts = !authoring_modal && !egui::Popup::is_any_open(&ctx);
        if global_shortcuts
            && self.transform_gesture.is_some()
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            let focused = self
                .transform_gesture
                .take()
                .and_then(|gesture| gesture.edit.focus);
            finish_numeric_action(&ctx, focused, false);
        }
        if global_shortcuts
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S))
        {
            self.save(false);
        }
        if global_shortcuts
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O))
        {
            self.choose_open();
        }
        if global_shortcuts
            && !ctx.egui_wants_keyboard_input()
            && ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Z,
                )
            })
        {
            self.history(&ctx, true);
        } else if global_shortcuts
            && !ctx.egui_wants_keyboard_input()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z))
        {
            self.history(&ctx, false);
        }
        if global_shortcuts
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::K))
        {
            self.search = !self.search;
        }
        if global_shortcuts && !self.search && !ctx.egui_wants_keyboard_input() {
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
                if self.tool == Tool::SewingPaths {
                    self.confirm_sewing_side(&ctx);
                } else {
                    self.finish_polygon();
                }
            }
            if self.tool == Tool::SewingPaths
                && ctx.input(|i| {
                    i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
                })
            {
                self.delete_sewing_span();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.reset_gestures();
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
        let modal_events =
            authoring_modal.then(|| ctx.input_mut(|input| std::mem::take(&mut input.events)));
        egui::Frame::new()
            .fill(Color32::from_gray(40))
            .show(ui, |ui| {
                if authoring_modal {
                    ui.disable();
                }
                egui::Panel::top("studio-menu")
                    .exact_size(26.0)
                    .resizable(false)
                    .frame(
                        egui::Frame::new()
                            .fill(Color32::from_gray(48))
                            .inner_margin(egui::Margin::symmetric(8, 1)),
                    )
                    .show(ui, |ui| self.menu_bar(ui));
                egui::Panel::top("studio-context")
                    .exact_size(34.0)
                    .resizable(false)
                    .frame(
                        egui::Frame::new()
                            .fill(Color32::from_gray(50))
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .stroke(Stroke::new(1.0, Color32::from_gray(32))),
                    )
                    .show(ui, |ui| self.toolbar(ui));
                egui::Panel::bottom("studio-status")
                    .exact_size(24.0)
                    .resizable(false)
                    .frame(
                        egui::Frame::new()
                            .fill(Color32::from_gray(48))
                            .inner_margin(egui::Margin::symmetric(8, 0)),
                    )
                    .show(ui, |ui| self.status_bar(ui));
                egui::Panel::bottom("studio-transport")
                    .exact_size(32.0)
                    .resizable(false)
                    .frame(
                        egui::Frame::new()
                            .fill(Color32::from_gray(50))
                            .inner_margin(egui::Margin::symmetric(8, 3))
                            .stroke(Stroke::new(1.0, Color32::from_gray(32))),
                    )
                    .show(ui, |ui| self.simulation_bar(ui));
                egui::Panel::left("studio-tools")
                    .exact_size(46.0)
                    .resizable(false)
                    .frame(
                        egui::Frame::new()
                            .fill(Color32::from_gray(48))
                            .inner_margin(egui::Margin::symmetric(6, 0))
                            .stroke(Stroke::new(1.0, Color32::from_gray(32))),
                    )
                    .show(ui, |ui| self.tool_rail(ui));
                let dock_width = if ui.available_width() < 1100.0 {
                    248.0
                } else {
                    292.0
                };
                egui::Panel::right("studio-properties")
                    .exact_size(dock_width)
                    .resizable(false)
                    .frame(
                        egui::Frame::new()
                            .fill(Color32::from_gray(50))
                            .stroke(Stroke::new(1.0, Color32::from_gray(30))),
                    )
                    .show(ui, |ui| self.inspector(ui));
                egui::Panel::top("studio-document")
                    .exact_size(28.0)
                    .resizable(false)
                    .frame(egui::Frame::new().fill(Color32::from_gray(40)))
                    .show(ui, |ui| self.document_tab(ui));
                // Columns inherit the parent max rect, so bound them above the transport.
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        ui.columns(2, |cols| {
                            self.patterns(&mut cols[0]);
                            self.garment(&mut cols[1]);
                        });
                    },
                );
            });
        if self.search {
            let mut open = true;
            egui::Window::new("Command search")
                .enabled(!authoring_modal)
                .open(&mut open)
                .collapsible(false)
                .show(&ctx, |ui| {
                    ui.text_edit_singleline(&mut self.query);
                    for label in [
                        "New Empty",
                        "Save project",
                        "Open project",
                        "Import SVG…",
                        "Import DXF…",
                        "Recover project",
                        "Export OBJ",
                        "Export SVG",
                        "Export DXF",
                        "Undo",
                        "Redo",
                        "Reset simulation",
                        "Select",
                        "Transform pattern",
                        "Polygon",
                        "Rectangle",
                        "Add ellipse",
                        "Sew edges",
                        "M:N sewing",
                    ] {
                        if !label.to_lowercase().contains(&self.query.to_lowercase()) {
                            continue;
                        }
                        let clicked = match label {
                            "New Empty" => self.new_empty_button(ui),
                            "Import SVG…" => self.import_button(ui, PatternFormat::Svg, label),
                            "Import DXF…" => self.import_button(ui, PatternFormat::Dxf, label),
                            "Export OBJ" => self.export_button(ui, ExportFormat::Obj, label),
                            "Export SVG" => self.export_button(ui, ExportFormat::Svg, label),
                            "Export DXF" => self.export_button(ui, ExportFormat::Dxf, label),
                            _ => ui.button(label).clicked(),
                        };
                        if clicked {
                            match label {
                                "New Empty" => {}
                                "Save project" => self.save(false),
                                "Open project" => self.choose_open(),
                                "Import SVG…" | "Import DXF…" => {}
                                "Recover project" => {
                                    self.refresh_recovery();
                                    self.show_recovery = true;
                                }
                                "Export OBJ" | "Export SVG" | "Export DXF" => {}
                                "Undo" => self.history(&ctx, false),
                                "Redo" => self.history(&ctx, true),
                                "Reset simulation" => {
                                    self.reset_simulation();
                                }
                                "Select" => self.set_tool(Tool::Select),
                                "Transform pattern" => self.set_tool(Tool::Transform),
                                "Polygon" => self.set_tool(Tool::Polygon),
                                "Rectangle" => self.set_tool(Tool::Rectangle),
                                "Add ellipse" => {
                                    let focused = self.inspector_focus(&ctx);
                                    self.add_ellipse(&ctx, focused);
                                }
                                "Sew edges" => self.set_tool(Tool::Sew),
                                "M:N sewing" => self.set_tool(Tool::SewingPaths),
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
        self.transform_window(&ctx);
        self.sewing_group_window(&ctx);
        self.pattern_import_window(&ctx);
        if let Some((_, Some(format), focused)) = ctx
            .data_mut(|data| {
                data.remove_temp::<(u64, Option<ExportFormat>, Option<egui::Id>)>(egui::Id::new(
                    "export-request",
                ))
            })
            .filter(|(frame, _, _)| *frame == ctx.cumulative_frame_nr())
        {
            // Read numeric input before opening a blocking native chooser.
            self.export(&ctx, format, focused);
        }
        if let Some((_, Some(format), focused)) = ctx
            .data_mut(|data| {
                data.remove_temp::<(u64, Option<PatternFormat>, Option<egui::Id>)>(egui::Id::new(
                    "pattern-import-request",
                ))
            })
            .filter(|(frame, _, _)| *frame == ctx.cumulative_frame_nr())
        {
            self.choose_import(&ctx, format, focused);
        }
        if let Some((_, true, focused)) = ctx
            .data_mut(|data| {
                data.remove_temp::<(u64, bool, Option<egui::Id>)>(egui::Id::new(
                    "new-empty-request",
                ))
            })
            .filter(|(frame, _, _)| *frame == ctx.cumulative_frame_nr())
        {
            self.new_empty(&ctx, focused);
        }
        self.restore_cancelled_transform_input(&ctx);
    }
}
fn pane_heading(ui: &mut egui::Ui, title: &str) -> egui::Response {
    egui::Frame::new()
        .fill(Color32::from_gray(48))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(title)
        })
        .inner
}

fn tool_button(ui: &mut egui::Ui, tool: Tool, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            ui.is_enabled(),
            selected,
            tool.label(),
        )
    });
    let painter = ui.painter();
    if selected || response.hovered() || response.has_focus() {
        painter.rect_filled(
            rect,
            2.0,
            if selected {
                Color32::from_rgb(53, 73, 93)
            } else {
                Color32::from_gray(61)
            },
        );
        if selected || response.has_focus() {
            painter.rect_stroke(
                rect,
                2.0,
                Stroke::new(1.0, Color32::from_rgb(74, 143, 208)),
                egui::StrokeKind::Inside,
            );
        }
    }
    let c = rect.center();
    let stroke = Stroke::new(1.3, Color32::from_gray(200));
    let point = |x, y| c + Vec2::new(x, y);
    match tool {
        Tool::Select => {
            painter.add(egui::Shape::closed_line(
                vec![
                    point(-6.0, -8.0),
                    point(-5.0, 7.0),
                    point(-1.0, 3.0),
                    point(3.0, 8.0),
                    point(6.0, 6.0),
                    point(2.0, 1.0),
                    point(8.0, 0.0),
                ],
                stroke,
            ));
        }
        Tool::Transform => {
            painter.rect_stroke(
                Rect::from_center_size(c, Vec2::splat(10.)),
                0.,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.arrow(point(-9., 0.), Vec2::new(18., 0.), stroke);
            painter.arrow(point(0., 9.), Vec2::new(0., -18.), stroke);
        }
        Tool::Polygon => {
            let points = [
                point(-7.0, -4.0),
                point(2.0, -8.0),
                point(8.0, 1.0),
                point(3.0, 8.0),
                point(-7.0, 5.0),
            ];
            painter.add(egui::Shape::closed_line(points.to_vec(), stroke));
            for p in points {
                painter.rect_filled(
                    Rect::from_center_size(p, Vec2::splat(3.0)),
                    0.0,
                    stroke.color,
                );
            }
        }
        Tool::Rectangle => {
            painter.rect_stroke(
                Rect::from_center_size(c, Vec2::new(16.0, 13.0)),
                0.0,
                stroke,
                egui::StrokeKind::Inside,
            );
        }
        Tool::Sew | Tool::SewingPaths => {
            painter.line_segment([point(-7.0, -8.0), point(-7.0, 8.0)], stroke);
            painter.line_segment([point(7.0, -8.0), point(7.0, 8.0)], stroke);
            if tool == Tool::Sew {
                for y in [-5.0, 0.0, 5.0] {
                    painter.line_segment([point(-5.0, y - 2.0), point(5.0, y + 2.0)], stroke);
                }
            } else {
                for (a, b) in [(-5.0, -6.0), (-5.0, 0.0), (5.0, 6.0)] {
                    painter.line_segment([point(-5.0, a), point(5.0, b)], stroke);
                }
                for y in [-5.0, 5.0] {
                    painter.circle_filled(point(-7.0, y), 1.7, stroke.color);
                }
                for y in [-6.0, 0.0, 6.0] {
                    painter.circle_filled(point(7.0, y), 1.7, stroke.color);
                }
            }
        }
    }
    response.on_hover_text(tool.label())
}

fn numeric_focus(ctx: &egui::Context) -> Option<egui::Id> {
    ctx.memory(|memory| memory.focused())
        .filter(|&id| ctx.data_mut(|data| data.get_temp::<String>(id)).is_some())
}
fn reverse_sewing_side(spans: &mut [CurveInterval]) {
    spans.reverse();
    for interval in spans {
        std::mem::swap(&mut interval.t0, &mut interval.t1);
    }
}

fn sewing_span_controls(
    ui: &mut egui::Ui,
    project: &Project,
    side: &str,
    spans: &mut Vec<CurveInterval>,
    selected_curve: Option<Id>,
    append_range: Option<[f64; 2]>,
) -> Vec<egui::Id> {
    ui.push_id(("sewing-spans", side), |ui| {
        ui.strong(format!("Side {side} · {} spans", spans.len()));
        let mut reorder = None;
        let mut remove = None;
        let mut reverse = None;
        let mut fields = Vec::new();
        let count = spans.len();
        for (index, interval) in spans.iter_mut().enumerate() {
            ui.push_id(index, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{side}{}", index + 1));
                    egui::ComboBox::from_id_salt("curve")
                        .selected_text(format!("Curve #{}", interval.curve))
                        .width(110.)
                        .show_ui(ui, |ui| {
                            for piece in &project.pieces {
                                for (boundary, curve) in piece.boundary.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut interval.curve,
                                        *curve,
                                        format!(
                                            "{} · boundary {} · #{curve}",
                                            piece.name,
                                            boundary + 1
                                        ),
                                    );
                                }
                            }
                        });
                });
                ui.horizontal(|ui| {
                    for (label, value) in
                        [("t start", &mut interval.t0), ("t end", &mut interval.t1)]
                    {
                        let label = ui.label(label);
                        let response = ui
                            .add(numeric_value(ui, value).speed(0.01))
                            .labelled_by(label.id);
                        fields.push(response.id);
                    }
                });
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(index > 0, egui::Button::new("Up").small())
                        .clicked()
                    {
                        reorder = Some((index, index - 1));
                    }
                    if ui
                        .add_enabled(index + 1 < count, egui::Button::new("Down").small())
                        .clicked()
                    {
                        reorder = Some((index, index + 1));
                    }
                    if ui.small_button("Direction").clicked() {
                        reverse = Some(index);
                    }
                    if ui.small_button("Remove span").clicked() {
                        remove = Some(index);
                    }
                });
            });
        }
        if reorder.is_some() || reverse.is_some() || remove.is_some() {
            clear_numeric_fields(ui.ctx(), &fields);
        }
        if let Some((a, b)) = reorder {
            spans.swap(a, b);
        }
        if let Some(index) = reverse {
            let interval = &mut spans[index];
            std::mem::swap(&mut interval.t0, &mut interval.t1);
        }
        if let Some(index) = remove {
            spans.remove(index);
        }
        if let Some([t0, t1]) = append_range {
            let curve = selected_curve.or_else(|| {
                project
                    .pieces
                    .iter()
                    .find_map(|piece| piece.boundary.first().copied())
            });
            if ui
                .add_enabled(
                    curve.is_some(),
                    egui::Button::new(format!("Add boundary to {side}")),
                )
                .clicked()
            {
                spans.push(CurveInterval {
                    curve: curve.expect("enabled boundary"),
                    t0,
                    t1,
                });
            }
        }
        fields
    })
    .inner
}
fn clear_numeric_fields(ctx: &egui::Context, fields: &[egui::Id]) {
    for &id in fields {
        ctx.data_mut(|data| data.remove::<String>(id));
        ctx.memory_mut(|memory| memory.surrender_focus(id));
    }
}
fn finish_numeric_action(ctx: &egui::Context, focused: Option<egui::Id>, succeeded: bool) {
    let guard_until = ctx.cumulative_frame_nr()
        + if ctx.memory(|memory| memory.top_modal_layer().is_some()) {
            2
        } else {
            1
        };
    let current = numeric_focus(ctx);
    if succeeded {
        for id in focused.into_iter().chain(current) {
            // Numeric caches must not replay values from before the action.
            ctx.data_mut(|data| data.remove::<String>(id));
            ctx.memory_mut(|memory| memory.surrender_focus(id));
        }
    } else if let Some(id) = current.or(focused) {
        ctx.memory_mut(|memory| memory.request_focus(id));
    }
    // A closing modal blocks restored focus for one pass, then blur persists for another.
    ctx.data_mut(|data| {
        data.insert_temp(
            egui::Id::new("inspector-action-blur"),
            (guard_until, [focused, current]),
        )
    });
}
fn numeric_value<'a>(ui: &egui::Ui, value: &'a mut f64) -> egui::DragValue<'a> {
    let id = ui.next_auto_id();
    let ctx = ui.ctx().clone();
    let formatter = ui.style().number_formatter.clone();
    egui::DragValue::new(value).custom_formatter(move |value, decimals| {
        // egui reparses its edit buffer on blur even when the text was never changed.
        if ctx.memory(|memory| memory.has_focus(id)) {
            value.to_string()
        } else {
            formatter.format(value, decimals)
        }
    })
}
fn numeric_commit(response: &egui::Response) -> bool {
    // egui reports a mid-frame focus transfer's blur on the following frame too.
    let action_blur = response
        .ctx
        .data_mut(|data| {
            data.get_temp::<(u64, [Option<egui::Id>; 2])>(egui::Id::new("inspector-action-blur"))
        })
        .is_some_and(|(guard_until, ids)| {
            response.ctx.cumulative_frame_nr() <= guard_until && ids.contains(&Some(response.id))
        });
    response.drag_stopped()
        || (response.lost_focus() && !action_blur)
        || (response.changed() && !response.is_pointer_button_down_on() && !response.has_focus())
}
fn check_cancelled(cancel: &AtomicBool) -> Result<(), JobError> {
    if cancel.load(Ordering::Relaxed) {
        Err(JobError::Cancelled)
    } else {
        Ok(())
    }
}
fn cancellable_core_job<T>(
    cancel: &AtomicBool,
    work: impl FnOnce(&dyn Fn() -> bool) -> weftforge_core::Result<T>,
) -> Result<T, JobError> {
    // Core errors flatten cancellation; record only the operation's own stopping callback.
    let stopped = std::cell::Cell::new(false);
    work(&|| {
        let requested = cancel.load(Ordering::Relaxed);
        if requested {
            stopped.set(true);
        }
        requested
    })
    .map_err(|error| {
        if stopped.get() {
            JobError::Cancelled
        } else {
            JobError::Failed(error.to_string())
        }
    })
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
        return Err("Choose an OBJ, SVG or DXF output; project files are preserved.".into());
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
fn choose_pattern_import_path(_ctx: &egui::Context, format: PatternFormat) -> Option<PathBuf> {
    #[cfg(test)]
    if let Some(path) = _ctx.data_mut(|data| {
        data.remove_temp::<Option<PathBuf>>(egui::Id::new(format!(
            "test-{}-import-source",
            format.name().to_ascii_lowercase()
        )))
    }) {
        return path;
    }
    let extension = match format {
        PatternFormat::Svg => "svg",
        PatternFormat::Dxf => "dxf",
    };
    rfd::FileDialog::new()
        .add_filter(format!("{} patterns", format.name()), &[extension])
        .pick_file()
}

fn choose_export_path(_ctx: &egui::Context, format: ExportFormat) -> Option<PathBuf> {
    #[cfg(test)]
    if let Some(path) = _ctx.data_mut(|data| {
        data.remove_temp::<Option<PathBuf>>(egui::Id::new("test-export-destination"))
    }) {
        return path;
    }
    let ext = match format {
        ExportFormat::Obj => "obj",
        ExportFormat::Svg => "svg",
        ExportFormat::Dxf => "dxf",
    };
    rfd::FileDialog::new()
        .add_filter(ext, &[ext])
        .set_file_name(format!("garment.{ext}"))
        .save_file()
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
fn studio_style(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Inter".into(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "JetBrains Mono".into(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"))
            .into(),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .expect("default proportional fonts")
        .insert(0, "Inter".into());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .expect("default monospace fonts")
        .insert(0, "JetBrains Mono".into());
    ctx.set_fonts(fonts);
    ctx.set_theme(egui::ThemePreference::Dark);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = Vec2::new(6.0, 4.0);
    style.spacing.button_padding = Vec2::new(6.0, 3.0);
    style.spacing.interact_size.y = 22.0;
    style.spacing.text_edit_width = 170.0;
    style.visuals = egui::Visuals::dark();
    let visuals = &mut style.visuals;
    visuals.panel_fill = Color32::from_gray(48);
    visuals.window_fill = Color32::from_gray(50);
    visuals.extreme_bg_color = Color32::from_gray(36);
    visuals.faint_bg_color = Color32::from_gray(46);
    visuals.weak_text_color = Some(Color32::from_gray(178));
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_gray(72));
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_gray(222));
    visuals.widgets.inactive.bg_fill = Color32::from_gray(38);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(48);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_gray(74));
    visuals.widgets.hovered.bg_fill = Color32::from_gray(64);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_gray(60);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_gray(104));
    visuals.widgets.active.bg_fill = Color32::from_gray(72);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(74, 143, 208));
    visuals.selection.bg_fill = Color32::from_rgb(53, 73, 93);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_gray(222));
    visuals.hyperlink_color = Color32::from_rgb(74, 143, 208);
    visuals.window_corner_radius = egui::CornerRadius::same(3);
    visuals.menu_corner_radius = egui::CornerRadius::same(2);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::same(2);
    }
    for (kind, size) in [
        (egui::TextStyle::Heading, 14.0),
        (egui::TextStyle::Body, 13.0),
        (egui::TextStyle::Button, 13.0),
        (egui::TextStyle::Small, 12.0),
    ] {
        style
            .text_styles
            .insert(kind, egui::FontId::proportional(size));
    }
    ctx.set_style_of(egui::Theme::Dark, style);
}
fn main() -> eframe::Result {
    eframe::run_native(
        "WeftForge",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1640., 1000.])
                .with_min_inner_size([1000., 650.]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Studio::new(cc)))),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn command_search_action(output: &egui::FullOutput, label: &str) -> egui::accesskit::NodeId {
        use egui::accesskit::{Action, Role};
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        let window = nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Window && node.label() == Some("Command search"))
            .expect("command search window")
            .0;
        let mut pending = vec![window];
        while let Some(id) = pending.pop() {
            let node = &nodes
                .iter()
                .find(|(node_id, _)| *node_id == id)
                .expect("AX child in update")
                .1;
            if node.label() == Some(label) && node.supports_action(Action::Click) {
                return id;
            }
            pending.extend_from_slice(node.children());
        }
        panic!("missing command search action {label}");
    }

    #[test]
    fn studio_workspace_keeps_creation_and_transport_visible() {
        for size in [[1000.0, 650.0], [1640.0, 1000.0]] {
            for busy in [false, true] {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                studio_style(&ctx);
                let mut app = Studio::empty();
                let (_sender, receiver) = mpsc::channel();
                if busy {
                    app.dirty = true;
                    app.job = Some(Job {
                        id: 0,
                        revision: 0,
                        kind: JobKind::Simulation,
                        batch: Some(SimulationBatch::assemble()),
                        completed_steps: Arc::new(AtomicU64::new(0)),
                        cancel: Arc::new(AtomicBool::new(false)),
                        receiver,
                    });
                    app.pending_batch = Some(SimulationBatch::drape());
                }
                let frame = |app: &mut Studio| {
                    ctx.run_ui(
                        egui::RawInput {
                            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::from(size))),
                            ..Default::default()
                        },
                        |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
                    )
                };
                frame(&mut app);
                let output = frame(&mut app);
                let nodes = &output.platform_output.accesskit_update.unwrap().nodes;
                let canvases: Vec<_> = output
                    .shapes
                    .iter()
                    .filter_map(|shape| match &shape.shape {
                        egui::Shape::Rect(rect) if rect.fill == Color32::from_gray(37) => {
                            Some(rect.rect)
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(canvases.len(), 2);
                let transport = nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some("Play"))
                    .unwrap()
                    .1
                    .bounds()
                    .unwrap();
                let frame = Rect::from_min_size(Pos2::ZERO, Vec2::from(size));
                for canvas in canvases {
                    assert!(
                        canvas.width() >= 330.0 && canvas.height() >= 430.0,
                        "undersized canvas at {size:?}/busy={busy}: {canvas:?}"
                    );
                    assert!(
                        frame.contains_rect(canvas),
                        "canvas outside {size:?}/busy={busy}: {canvas:?}"
                    );
                    assert!(
                        canvas.bottom() <= transport.y0 as f32,
                        "canvas overpaints transport at {size:?}/busy={busy}: {canvas:?}, {transport:?}"
                    );
                }
                let mut labels = vec![
                    "File",
                    "Edit",
                    "View",
                    "Help",
                    "Select / edit",
                    "Polygon",
                    "Rectangle",
                    "Sew edges",
                    "M:N sewing",
                    "Properties",
                    "Scene",
                    "Undo",
                    "Add ellipse",
                    "Play",
                    "Drape",
                    "Reset",
                    app.message.as_str(),
                ];
                if busy {
                    labels.extend(["Cancel job", "Discard and close", "Unsaved"]);
                }
                for label in labels {
                    let node = &nodes
                        .iter()
                        .find(|(_, node)| {
                            node.label() == Some(label)
                                || (node.role() == egui::accesskit::Role::Label
                                    && node.value() == Some(label))
                        })
                        .unwrap_or_else(|| panic!("missing {label} at {size:?}/busy={busy}"))
                        .1;
                    let bounds = node.bounds().unwrap();
                    assert!(
                        bounds.x0 >= 0.0
                            && bounds.y0 >= 0.0
                            && bounds.x1 <= size[0] as f64
                            && bounds.y1 <= size[1] as f64,
                        "{label} outside {size:?}/busy={busy}: {bounds:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn studio_menus_and_tool_rail_preserve_numeric_drafts_text_and_caret() {
        use egui::accesskit::{Action, ActionRequest, TreeId};
        for text in ["0.450", "1e-", "NaN"] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            studio_style(&ctx);
            let mut app = Studio::empty();
            app.session_autosave = None;
            app.editor
                .execute(Command::AddRectangle {
                    name: "Working panel".into(),
                    origin: [0.0, 0.0],
                    width: 0.4,
                    height: 0.6,
                })
                .unwrap();
            app.selected = Some(app.editor.project.pieces[0].id);
            app.selected_curve = Some(app.editor.project.pieces[0].boundary[0]);
            let frame = |app: &mut Studio, events| {
                ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            Vec2::new(1640.0, 1032.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
                )
            };
            let key = |key, pressed, modifiers| egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            };
            frame(&mut app, vec![]);
            let output = frame(&mut app, vec![]);
            let numeric = output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| node.numeric_value() == Some(0.4))
                .unwrap()
                .0;
            frame(
                &mut app,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Focus,
                    target_tree: TreeId::ROOT,
                    target_node: numeric,
                    data: None,
                })],
            );
            frame(&mut app, vec![]);
            frame(
                &mut app,
                vec![
                    key(egui::Key::A, true, egui::Modifiers::COMMAND),
                    egui::Event::Text(text.into()),
                ],
            );
            let focused = numeric_focus(&ctx).unwrap();
            let caret = egui::TextEdit::load_state(&ctx, focused)
                .unwrap()
                .cursor
                .char_range();
            let editor = format!("{:?}", app.editor);
            let pending = app.draft.as_ref().unwrap().commands(&app.editor.project);
            for (label, menu) in [
                ("File", true),
                ("Edit", true),
                ("View", true),
                ("Help", true),
                ("Task details", true),
                ("Polygon", false),
            ] {
                let output = frame(&mut app, vec![]);
                let (_, node) = output
                    .platform_output
                    .accesskit_update
                    .unwrap()
                    .nodes
                    .into_iter()
                    .find(|(_, node)| {
                        node.label() == Some(label) && node.supports_action(Action::Click)
                    })
                    .unwrap();
                let bounds = node.bounds().unwrap();
                let pos = Pos2::new(
                    ((bounds.x0 + bounds.x1) * 0.5) as f32,
                    ((bounds.y0 + bounds.y1) * 0.5) as f32,
                );
                for pressed in [true, false] {
                    frame(
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
                    assert_eq!(
                        format!("{:?}", app.editor),
                        editor,
                        "{label} pointer press/release"
                    );
                }
                if menu {
                    frame(
                        &mut app,
                        vec![key(egui::Key::Escape, true, egui::Modifiers::NONE)],
                    );
                    frame(
                        &mut app,
                        vec![key(egui::Key::Escape, false, egui::Modifiers::NONE)],
                    );
                }
                for _ in 0..3 {
                    frame(&mut app, vec![]);
                }
                assert_eq!(format!("{:?}", app.editor), editor, "{label} delayed blur");
                assert_eq!(
                    app.draft.as_ref().unwrap().commands(&app.editor.project),
                    pending
                );
                assert_eq!(numeric_focus(&ctx), Some(focused), "{label} focus");
                assert_eq!(
                    ctx.data_mut(|data| data.get_temp::<String>(focused))
                        .as_deref(),
                    Some(text),
                    "{label} text"
                );
                assert_eq!(
                    egui::TextEdit::load_state(&ctx, focused)
                        .unwrap()
                        .cursor
                        .char_range(),
                    caret,
                    "{label} caret"
                );
                assert!(!app.dirty && !app.needs_mesh && app.job.is_none());
            }
            assert!(app.tool == Tool::Polygon);
        }
    }

    #[test]
    fn studio_popup_keys_preserve_unfocused_canvas_gestures() {
        for tool in [Tool::Polygon, Tool::SewingPaths] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            studio_style(&ctx);
            let mut app = Studio::empty();
            app.session_autosave = None;
            app.editor
                .execute(Command::AddRectangle {
                    name: "Existing panel".into(),
                    origin: [0.0, 0.0],
                    width: 0.4,
                    height: 0.6,
                })
                .unwrap();
            app.mesh = Some(Arc::new(app.editor.project.mesh().unwrap()));
            app.tool = tool;
            let polygon = if tool == Tool::Polygon {
                vec![[-0.3, 1.0], [-0.1, 1.4], [0.1, 1.0]]
            } else {
                vec![]
            };
            app.polygon = polygon.clone();
            let span = CurveInterval {
                curve: app.editor.project.pieces[0].boundary[0],
                t0: 0.0,
                t1: 1.0,
            };
            let spans = if tool == Tool::SewingPaths {
                vec![span]
            } else {
                vec![]
            };
            app.sewing_draft.a = spans.clone();
            let before = app.editor.clone();
            let frame = |app: &mut Studio, events| {
                ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            Vec2::new(1640.0, 1032.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
                )
            };
            let key = |key, pressed| egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            };
            frame(&mut app, vec![]);
            let output = frame(&mut app, vec![]);
            assert!(numeric_focus(&ctx).is_none());
            assert!(!ctx.egui_wants_keyboard_input());
            let (_, file) = output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| node.label() == Some("File"))
                .unwrap();
            let bounds = file.bounds().unwrap();
            let pos = Pos2::new(
                ((bounds.x0 + bounds.x1) * 0.5) as f32,
                ((bounds.y0 + bounds.y1) * 0.5) as f32,
            );
            for pressed in [true, false] {
                frame(
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
            assert!(egui::Popup::is_any_open(&ctx));
            for pressed in [
                egui::Key::Enter,
                egui::Key::Delete,
                egui::Key::Backspace,
                egui::Key::Space,
                egui::Key::Escape,
            ] {
                frame(&mut app, vec![key(pressed, true)]);
                assert_eq!(
                    format!("{:?}", app.editor),
                    format!("{before:?}"),
                    "popup {pressed:?} must not author construction"
                );
                assert_eq!(
                    app.polygon, polygon,
                    "popup {pressed:?} must retain polygon vertices"
                );
                assert_eq!(
                    app.sewing_draft.a, spans,
                    "popup {pressed:?} must retain sewing spans"
                );
                assert!(app.sewing_draft.b.is_empty() && !app.sewing_draft.selecting_b);
                assert!(
                    !app.playing && app.pending_batch.is_none() && app.job.is_none(),
                    "popup {pressed:?} must not start simulation"
                );
                frame(&mut app, vec![key(pressed, false)]);
            }
            assert!(!egui::Popup::is_any_open(&ctx));
            frame(&mut app, vec![key(egui::Key::Space, true)]);
            assert!(
                app.playing && app.job.is_some(),
                "canvas Space still starts simulation after dismissal"
            );
            app.playing = false;
            app.cancel();
            app.job = None;
            frame(
                &mut app,
                vec![key(egui::Key::Space, false), key(egui::Key::Enter, true)],
            );
            if tool == Tool::Polygon {
                let mut expected = before.clone();
                expected
                    .execute(Command::AddPolygon {
                        name: app.name.clone(),
                        vertices: polygon.clone(),
                    })
                    .unwrap();
                assert_eq!(format!("{:?}", app.editor), format!("{expected:?}"));
                assert_eq!(
                    app.editor.project.pieces.len(),
                    before.project.pieces.len() + 1
                );
                assert!(app.polygon.is_empty());
                assert!(app.editor.undo());
                assert!(expected.undo());
                assert_eq!(format!("{:?}", app.editor), format!("{expected:?}"));
            } else {
                assert!(
                    app.sewing_draft.selecting_b,
                    "canvas Enter still confirms side A after dismissal"
                );
                frame(
                    &mut app,
                    vec![key(egui::Key::Enter, false), key(egui::Key::Delete, true)],
                );
                assert!(app.sewing_draft.a.is_empty() && !app.sewing_draft.selecting_b);
            }
            app.cancel();
            app.job = None;
            app.needs_mesh = false;
            app.polygon = polygon;
            app.sewing_draft.a = spans;
            frame(
                &mut app,
                vec![
                    key(egui::Key::Enter, false),
                    key(egui::Key::Delete, false),
                    key(egui::Key::Escape, true),
                ],
            );
            assert!(
                app.polygon.is_empty() && app.sewing_draft.a.is_empty(),
                "canvas Escape still cancels gestures after dismissal"
            );
        }
    }

    #[test]
    fn studio_keyboard_menu_activation_keeps_keyboard_focus() {
        use egui::accesskit::{Action, ActionRequest, TreeId};
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        studio_style(&ctx);
        let mut app = Studio::empty();
        app.session_autosave = None;
        app.set_tool(Tool::SewingPaths);
        let before = format!("{:?}", app.editor);
        let frame = |app: &mut Studio, events| {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1640.0, 1032.0))),
                    events,
                    ..Default::default()
                },
                |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
            )
        };
        let focus = |target_node| {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Focus,
                target_tree: TreeId::ROOT,
                target_node,
                data: None,
            })
        };
        frame(&mut app, vec![]);
        let output = frame(&mut app, vec![]);
        let nodes = output.platform_output.accesskit_update.unwrap().nodes;
        let numeric = nodes
            .iter()
            .find(|(_, node)| node.numeric_value() == Some(0.07))
            .unwrap()
            .0;
        let file = nodes
            .iter()
            .find(|(_, node)| node.label() == Some("File"))
            .unwrap()
            .0;
        frame(&mut app, vec![focus(numeric)]);
        frame(&mut app, vec![]);
        assert!(numeric_focus(&ctx).is_some());
        frame(&mut app, vec![focus(file)]);
        frame(&mut app, vec![]);
        assert_eq!(
            ctx.memory(|memory| memory.focused())
                .unwrap()
                .accesskit_id(),
            file
        );
        assert!(numeric_focus(&ctx).is_none());
        frame(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(egui::Popup::is_any_open(&ctx));
        assert_eq!(
            ctx.memory(|memory| memory.focused())
                .unwrap()
                .accesskit_id(),
            file,
            "keyboard File activation must not restore cached numeric focus"
        );
        assert!(numeric_focus(&ctx).is_none());
        assert_eq!(format!("{:?}", app.editor), before);
    }

    #[test]
    fn canvas_picking() {
        let t = [Pos2::ZERO, Pos2::new(1., 0.), Pos2::new(0., 1.)];
        assert!(point_in_triangle(Pos2::new(0.2, 0.2), t));
        assert!(!point_in_triangle(Pos2::new(2., 2.), t));
        assert!(!point_in_triangle(Pos2::ZERO, [Pos2::ZERO; 3]));
        assert!(
            (segment_distance(Pos2::new(0.5, 1.), Pos2::ZERO, Pos2::new(1., 0.)) - 1.).abs() < 1e-6
        );
    }

    #[test]
    fn position_snapping_rounds_scaled_ties_away_from_zero_and_off_preserves_bits() {
        for (snap, half_step) in [
            (PositionSnap::Mm1, 0.0005),
            (PositionSnap::Mm5, 0.0025),
            (PositionSnap::Mm10, 0.005),
        ] {
            for sign in [-1.0, 1.0] {
                assert_eq!(snap.apply(sign * half_step), sign * half_step * 2.0);
                assert_eq!(snap.apply(sign * half_step * 0.9), sign * 0.0);
                assert_eq!(snap.apply(sign * half_step * 2.9), sign * half_step * 2.0);
            }
        }
        for value in [-0.0, -0.123_456_789_012_345, 0.123_456_789_012_345] {
            assert_eq!(PositionSnap::Off.apply(value).to_bits(), value.to_bits());
        }
    }

    #[test]
    fn position_snapping_pointer_geometry_bypass_and_history_use_one_captured_rule() {
        fn frame(
            ctx: &egui::Context,
            app: &mut Studio,
            modifiers: egui::Modifiers,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1200., 800.))),
                    modifiers,
                    events,
                    ..Default::default()
                },
                |ui| {
                    ui.scope(|ui| {
                        if app.rectangle.is_some() {
                            ui.disable();
                        }
                        app.toolbar(ui);
                        ui.columns(2, |columns| {
                            app.inspector(&mut columns[0]);
                            app.patterns(&mut columns[1]);
                        });
                    });
                    app.rectangle_window(ui.ctx());
                },
            )
        }
        fn button(pos: Pos2, pressed: bool, modifiers: egui::Modifiers) -> egui::Event {
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers,
            }
        }
        fn click(
            ctx: &egui::Context,
            app: &mut Studio,
            pos: Pos2,
            modifiers: egui::Modifiers,
        ) -> egui::FullOutput {
            frame(
                ctx,
                app,
                modifiers,
                vec![egui::Event::PointerMoved(pos), button(pos, true, modifiers)],
            );
            frame(ctx, app, modifiers, vec![button(pos, false, modifiers)])
        }
        fn text_position(output: &egui::FullOutput, text: &str) -> Pos2 {
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
                .next_back()
                .unwrap_or_else(|| panic!("missing text {text:?}"))
        }
        fn choose_snap(ctx: &egui::Context, app: &mut Studio, snap: PositionSnap) {
            let output = frame(ctx, app, egui::Modifiers::NONE, vec![]);
            click(
                ctx,
                app,
                text_position(&output, snap.label()),
                egui::Modifiers::NONE,
            );
            frame(ctx, app, egui::Modifiers::NONE, vec![]);
        }
        fn assert_position(actual: [f64; 2], expected: [f64; 2]) {
            for axis in 0..2 {
                assert!(
                    (actual[axis] - expected[axis]).abs() < 2.0e-8,
                    "{actual:?} != {expected:?}"
                );
            }
        }
        for (snap, bypass, start, end, moved) in [
            (
                PositionSnap::Off,
                false,
                [-0.276, -0.246],
                [-0.146, -0.384],
                [-0.157, -0.333],
            ),
            (
                PositionSnap::Mm1,
                false,
                [-0.276, -0.246],
                [-0.146, -0.384],
                [-0.157, -0.333],
            ),
            (
                PositionSnap::Mm5,
                false,
                [-0.275, -0.245],
                [-0.145, -0.385],
                [-0.155, -0.335],
            ),
            (
                PositionSnap::Mm10,
                false,
                [-0.28, -0.25],
                [-0.15, -0.38],
                [-0.16, -0.33],
            ),
            (
                PositionSnap::Mm10,
                true,
                [-0.276, -0.246],
                [-0.146, -0.384],
                [-0.157, -0.333],
            ),
        ] {
            let ctx = egui::Context::default();
            studio_style(&ctx);
            let mut app = Studio::empty();
            app.session_autosave = None;
            app.center = [-0.2, -0.3];
            app.scale = 1000.;
            assert_eq!(app.position_snap, PositionSnap::Mm10);
            if snap == PositionSnap::Off {
                let output = frame(&ctx, &mut app, egui::Modifiers::NONE, vec![]);
                click(
                    &ctx,
                    &mut app,
                    text_position(&output, "0.07"),
                    egui::Modifiers::NONE,
                );
                frame(
                    &ctx,
                    &mut app,
                    egui::Modifiers::NONE,
                    vec![
                        egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::COMMAND,
                        },
                        egui::Event::Text("0.073456789012345".into()),
                    ],
                );
                let focused = numeric_focus(&ctx).unwrap();
                let editor = format!("{:?}", app.editor);
                choose_snap(&ctx, &mut app, snap);
                for _ in 0..3 {
                    frame(&ctx, &mut app, egui::Modifiers::NONE, vec![]);
                }
                assert_eq!(format!("{:?}", app.editor), editor);
                assert_eq!(app.draft.as_ref().unwrap().resolution, 0.073456789012345);
                assert_eq!(numeric_focus(&ctx), Some(focused));
                finish_numeric_action(&ctx, Some(focused), true);
                app.draft = None;
            } else {
                choose_snap(&ctx, &mut app, snap);
            }
            assert!(!app.dirty && !app.editor.can_undo());
            app.set_tool(Tool::Rectangle);
            frame(&ctx, &mut app, egui::Modifiers::NONE, vec![]);
            let rect = ctx.read_response(app.pattern_canvas.unwrap()).unwrap().rect;
            let center = rect.center();
            let modifiers = if bypass {
                egui::Modifiers::ALT
            } else {
                egui::Modifiers::NONE
            };
            let press = center + Vec2::new(-84., -54.);
            frame(
                &ctx,
                &mut app,
                modifiers,
                vec![
                    egui::Event::PointerMoved(press),
                    button(press, true, modifiers),
                ],
            );
            frame(
                &ctx,
                &mut app,
                modifiers,
                vec![egui::Event::PointerMoved(center + Vec2::new(-76., -54.))],
            );
            assert_position(app.drag_start.unwrap(), start);
            assert_eq!(app.editor.project.revision, 0);
            // Modifier changes after drag recognition must not mix snapped and raw corners.
            let released_modifiers = if bypass {
                egui::Modifiers::NONE
            } else {
                egui::Modifiers::ALT
            };
            let finish = center + Vec2::new(54., 84.);
            frame(
                &ctx,
                &mut app,
                released_modifiers,
                vec![egui::Event::PointerMoved(finish)],
            );
            frame(
                &ctx,
                &mut app,
                released_modifiers,
                vec![button(finish, false, released_modifiers)],
            );
            assert_eq!(app.editor.project.revision, 1);
            let bounds = app
                .editor
                .project
                .piece_bounds(app.editor.project.pieces[0].id)
                .unwrap();
            assert_position(bounds[0], [start[0], end[1]]);
            assert_position(bounds[1], [end[0], start[1]]);
            let rectangle = app.editor.project.clone();
            app.history(&ctx, false);
            assert!(
                app.editor.project.pieces.is_empty()
                    && !app.editor.can_undo()
                    && app.editor.can_redo()
            );
            app.history(&ctx, true);
            assert_eq!(app.editor.project.points, rectangle.points);
            assert_eq!(app.editor.project.curves, rectangle.curves);

            app.set_tool(Tool::Select);
            frame(&ctx, &mut app, egui::Modifiers::NONE, vec![]);
            let point = app.editor.project.points[0].id;
            let press = app.to_screen(app.editor.project.point(point).unwrap().position, rect);
            let revision = app.editor.project.revision;
            frame(
                &ctx,
                &mut app,
                modifiers,
                vec![
                    egui::Event::PointerMoved(press),
                    button(press, true, modifiers),
                ],
            );
            frame(
                &ctx,
                &mut app,
                modifiers,
                vec![egui::Event::PointerMoved(press + Vec2::new(8., 0.))],
            );
            let finish = center + Vec2::new(43., 33.);
            frame(
                &ctx,
                &mut app,
                released_modifiers,
                vec![egui::Event::PointerMoved(finish)],
            );
            assert_position(app.preview.unwrap(), moved);
            assert_eq!(app.editor.project.points, rectangle.points);
            frame(
                &ctx,
                &mut app,
                released_modifiers,
                vec![button(finish, false, released_modifiers)],
            );
            assert_eq!(app.editor.project.revision, revision + 1);
            assert_position(app.editor.project.point(point).unwrap().position, moved);
            let after_move = app.editor.project.clone();
            app.history(&ctx, false);
            assert_eq!(app.editor.project.points, rectangle.points);
            app.history(&ctx, true);
            assert_eq!(app.editor.project.points, after_move.points);
            let mut history = app.editor.clone();
            assert!(history.undo() && history.undo());
            assert!(history.project.pieces.is_empty() && !history.can_undo());

            app.set_tool(Tool::Polygon);
            let revision = app.editor.project.revision;
            for (index, offset) in [
                Vec2::new(-76., -54.),
                Vec2::new(54., 84.),
                Vec2::new(43., 33.),
            ]
            .into_iter()
            .enumerate()
            {
                click(&ctx, &mut app, center + offset, modifiers);
                assert_position(app.polygon[index], [start, end, moved][index]);
                assert_eq!(app.editor.project.revision, revision);
            }
            let output = frame(&ctx, &mut app, egui::Modifiers::NONE, vec![]);
            click(
                &ctx,
                &mut app,
                text_position(&output, "Close pattern (3 points)"),
                egui::Modifiers::NONE,
            );
            assert_eq!(app.editor.project.revision, revision + 1);
            assert_eq!(app.editor.project.pieces.len(), 2);
            app.history(&ctx, false);
            assert_eq!(app.editor.project.points, after_move.points);
            assert_eq!(app.editor.project.pieces, after_move.pieces);
            assert_eq!(app.position_snap, snap);

            app.set_tool(Tool::Rectangle);
            click(&ctx, &mut app, center + Vec2::new(-76., -54.), modifiers);
            assert_position(app.rectangle.as_ref().unwrap().origin, start);
            app.rectangle.as_mut().unwrap().dimensions_mm =
                ["123.456789".into(), "234.567891".into()];
            let output = frame(&ctx, &mut app, egui::Modifiers::NONE, vec![]);
            click(
                &ctx,
                &mut app,
                text_position(&output, "OK"),
                egui::Modifiers::NONE,
            );
            let piece = app.editor.project.pieces.last().unwrap().id;
            let [min, max] = app.editor.project.piece_bounds(piece).unwrap();
            assert_position(min, start);
            assert!((max[0] - min[0] - 0.123456789).abs() < 1.0e-15);
            assert!((max[1] - min[1] - 0.234567891).abs() < 1.0e-15);
            let dir = exit_test_directory("position-snapping");
            let path = dir.join("session.weftforge");
            app.editor.project.save(&path).unwrap();
            app.open(path);
            assert_eq!(app.position_snap, snap);
            fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn position_snapping_inline_choices_preserve_unfinished_numeric_text_and_caret() {
        fn frame(
            ctx: &egui::Context,
            app: &mut Studio,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1200., 1800.))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    app.toolbar(ui);
                    ui.columns(2, |columns| {
                        app.inspector(&mut columns[0]);
                        app.patterns(&mut columns[1]);
                    });
                },
            )
        }
        fn text_position(output: &egui::FullOutput, text: &str) -> Pos2 {
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
                .next_back()
                .unwrap_or_else(|| panic!("missing text {text:?}"))
        }
        fn click(ctx: &egui::Context, app: &mut Studio, pos: Pos2) {
            for pressed in [true, false] {
                frame(
                    ctx,
                    app,
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
        for (entered, completed, resolution) in [
            ("1e-", "1e-2", 0.01),
            ("-", "-2", 0.005),
            ("0.", "0.2", 0.2),
        ] {
            let ctx = egui::Context::default();
            studio_style(&ctx);
            let mut app = Studio::empty();
            app.session_autosave = None;
            let output = frame(&ctx, &mut app, vec![]);
            click(&ctx, &mut app, text_position(&output, "0.07"));
            frame(
                &ctx,
                &mut app,
                vec![
                    key(egui::Key::A, egui::Modifiers::COMMAND),
                    egui::Event::Text(entered.into()),
                ],
            );
            let id = numeric_focus(&ctx).expect("numeric field retains focus");
            let raw_before = ctx.data_mut(|data| data.get_temp::<String>(id));
            assert_eq!(raw_before.as_deref(), Some(entered));
            let caret_before = egui::TextEdit::load_state(&ctx, id)
                .unwrap()
                .cursor
                .char_range();
            let draft_before = app.draft.as_ref().unwrap().commands(&app.editor.project);
            let editor_before = format!("{:?}", app.editor);
            for snap in [
                PositionSnap::Off,
                PositionSnap::Mm1,
                PositionSnap::Mm5,
                PositionSnap::Mm10,
            ] {
                let output = frame(&ctx, &mut app, vec![]);
                click(&ctx, &mut app, text_position(&output, snap.label()));
                for _ in 0..3 {
                    frame(&ctx, &mut app, vec![]);
                }
                assert_eq!(app.position_snap, snap);
                assert_eq!(numeric_focus(&ctx), Some(id));
                assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(id)), raw_before);
                assert_eq!(
                    egui::TextEdit::load_state(&ctx, id)
                        .unwrap()
                        .cursor
                        .char_range(),
                    caret_before
                );
                assert_eq!(
                    app.draft.as_ref().unwrap().commands(&app.editor.project),
                    draft_before
                );
                assert_eq!(format!("{:?}", app.editor), editor_before);
                assert!(!app.dirty);
            }
            frame(&ctx, &mut app, vec![egui::Event::Text("2".into())]);
            assert_eq!(
                ctx.data_mut(|data| data.get_temp::<String>(id)).as_deref(),
                Some(completed)
            );
            assert_eq!(app.draft.as_ref().unwrap().resolution, resolution);
            assert_eq!(format!("{:?}", app.editor), editor_before);
        }

        for tool in [Tool::Select, Tool::SewingPaths] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            studio_style(&ctx);
            let mut app = Studio::empty();
            app.session_autosave = None;
            app.set_tool(tool);
            let output = frame(&ctx, &mut app, vec![]);
            click(&ctx, &mut app, text_position(&output, "0.07"));
            frame(
                &ctx,
                &mut app,
                vec![
                    key(egui::Key::A, egui::Modifiers::COMMAND),
                    egui::Event::Text("1e-".into()),
                ],
            );
            let numeric = numeric_focus(&ctx).expect("prior numeric focus");
            let output = frame(&ctx, &mut app, vec![]);
            let off = output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| node.label() == Some("Off"))
                .unwrap()
                .0;
            frame(
                &ctx,
                &mut app,
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action: egui::accesskit::Action::Focus,
                        target_tree: egui::accesskit::TreeId::ROOT,
                        target_node: off,
                        data: None,
                    },
                )],
            );
            let focused = ctx.memory(|memory| memory.focused()).unwrap();
            assert_eq!(focused.accesskit_id(), off);
            assert_ne!(focused, numeric);
            let editor = format!("{:?}", app.editor);
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
            );
            assert_eq!(app.position_snap, PositionSnap::Off);
            assert_eq!(ctx.memory(|memory| memory.focused()), Some(focused));
            assert_eq!(format!("{:?}", app.editor), editor);
            assert!(!app.dirty && !app.editor.can_undo());
        }
    }

    #[test]
    fn numeric_fields_preserve_unedited_bits_and_accept_text_steps_and_drags() {
        fn frame(ctx: &egui::Context, value: &mut f64, events: Vec<egui::Event>) -> egui::Response {
            let mut response = None;
            let _ = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(500., 200.))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    response = Some(ui.add(numeric_value(ui, value).speed(0.001)));
                    let _ = ui.button("Done");
                },
            );
            response.unwrap()
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
        for original in [
            2.1333333333333333,
            std::f64::consts::FRAC_PI_2,
            -0.12345678901234566,
            1.0e-18,
            1.0e18,
            -0.0,
        ] {
            let ctx = egui::Context::default();
            let mut value = original;
            let field = frame(&ctx, &mut value, vec![]);
            ctx.memory_mut(|memory| memory.request_focus(field.id));
            frame(&ctx, &mut value, vec![]);
            let blurred = frame(
                &ctx,
                &mut value,
                vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
            );
            assert!(blurred.lost_focus());
            frame(&ctx, &mut value, vec![]);
            assert_eq!(value.to_bits(), original.to_bits());
        }
        let ctx = egui::Context::default();
        let mut value = 2.1333333333333333;
        let field = frame(&ctx, &mut value, vec![]);
        ctx.memory_mut(|memory| memory.request_focus(field.id));
        frame(&ctx, &mut value, vec![]);
        let response = frame(
            &ctx,
            &mut value,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Text("2.13333".into()),
            ],
        );
        assert!(response.changed());
        assert_eq!(
            value, 2.13333,
            "deliberately typing the rounded display must still edit"
        );
        for (step, expected) in [(egui::Key::ArrowUp, 2.134), (egui::Key::ArrowDown, 2.133)] {
            let response = frame(&ctx, &mut value, vec![key(step, egui::Modifiers::NONE)]);
            assert!(response.changed());
            assert_eq!(value, expected);
        }
        frame(
            &ctx,
            &mut value,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Text("invalid".into()),
            ],
        );
        frame(
            &ctx,
            &mut value,
            vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
        );
        assert_eq!(
            value, 2.133,
            "invalid text must preserve the last valid value"
        );
        let response = frame(&ctx, &mut value, vec![]);
        let start = response.rect.center();
        frame(
            &ctx,
            &mut value,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        let end = start + Vec2::new(20., 0.);
        let dragged = frame(&ctx, &mut value, vec![egui::Event::PointerMoved(end)]);
        assert!(dragged.dragged() && dragged.changed());
        assert!(value > 2.133);
        let released = frame(
            &ctx,
            &mut value,
            vec![egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(released.drag_stopped() && numeric_commit(&released));
    }

    #[test]
    fn tabbing_over_unedited_curve_control_preserves_saved_f64_precision() {
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
        fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }
        }
        fn control(output: &egui::FullOutput, x: f64, axis: usize) -> (NodeId, Pos2) {
            let nodes = &output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes;
            let row = nodes
                .iter()
                .find(|(_, node)| node.numeric_value() == Some(x))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            let mut fields: Vec<_> = nodes
                .iter()
                .filter(|(_, node)| {
                    node.numeric_value().is_some()
                        && node
                            .bounds()
                            .is_some_and(|bounds| (bounds.y0 - row.y0).abs() < 1.)
                })
                .collect();
            fields.sort_by(|a, b| {
                a.1.bounds()
                    .unwrap()
                    .x0
                    .total_cmp(&b.1.bounds().unwrap().x0)
            });
            let (id, node) = fields[axis];
            let bounds = node.bounds().unwrap();
            (
                *id,
                Pos2::new(
                    ((bounds.x0 + bounds.x1) * 0.5) as f32,
                    ((bounds.y0 + bounds.y1) * 0.5) as f32,
                ),
            )
        }
        fn click(ctx: &egui::Context, app: &mut Studio, pos: Pos2) {
            for pressed in [true, false] {
                frame(
                    ctx,
                    app,
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
            frame(ctx, app, vec![]);
        }
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = Studio::empty();
        app.editor
            .execute(Command::AddRectangle {
                name: "Precision panel".into(),
                origin: [2., 0.],
                width: 0.4,
                height: 0.6,
            })
            .unwrap();
        let piece = app.editor.project.pieces[0].id;
        let curve = app.editor.project.pieces[0].boundary[2];
        app.select_curve(&ctx, Some((piece, curve)));
        let output = frame(&ctx, &mut app, vec![]);
        let cubic = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Cubic Bezier"))
            .unwrap()
            .0;
        let output = frame(
            &ctx,
            &mut app,
            vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Click,
                target_tree: TreeId::ROOT,
                target_node: cubic,
                data: None,
            })],
        );
        let original = app.editor.project.curve(curve).unwrap().controls.unwrap();
        assert_eq!(
            original,
            [[2.2666666666666666, 0.6], [2.1333333333333333, 0.6]]
        );
        let (_, p1_y) = control(&output, original[0][0], 1);
        let (p2_x, _) = control(&output, original[1][0], 0);
        click(&ctx, &mut app, p1_y);
        frame(
            &ctx,
            &mut app,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Text("0.8".into()),
            ],
        );
        let output = frame(
            &ctx,
            &mut app,
            vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
        );
        assert_eq!(
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .focus,
            p2_x
        );
        let output = frame(&ctx, &mut app, vec![]);
        let (_, p2_y) = control(&output, original[1][0], 1);
        click(&ctx, &mut app, p2_y);
        frame(
            &ctx,
            &mut app,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Text("0.8".into()),
            ],
        );
        frame(
            &ctx,
            &mut app,
            vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
        );
        let dir = exit_test_directory("numeric-precision");
        let path = dir.join("precision.weftforge");
        app.path = Some(path.clone());
        frame(
            &ctx,
            &mut app,
            vec![key(egui::Key::S, egui::Modifiers::COMMAND)],
        );
        let saved = Project::load(&path).unwrap();
        fs::remove_dir_all(dir).unwrap();
        assert_eq!(
            saved.curve(curve).unwrap().controls,
            Some([[original[0][0], 0.8], [original[1][0], 0.8]])
        );
        assert_eq!(saved, app.editor.project);
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
    fn focused_export_keeps_failed_drafts_and_history_until_output_is_written() {
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
        fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }
        }
        fn button(output: &egui::FullOutput, label: &str) -> (NodeId, Pos2) {
            let palette_action = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| {
                    node.role() == egui::accesskit::Role::Window
                        && node.label() == Some("Command search")
                })
                .then(|| command_search_action(output, label));
            let (id, node) = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(id, node)| {
                    node.label() == Some(label)
                        && node.supports_action(Action::Click)
                        && palette_action.is_none_or(|expected| *id == expected)
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
        fn history(editor: &Editor) -> [Vec<Project>; 2] {
            let mut backward = editor.clone();
            let mut forward = editor.clone();
            let mut undo = Vec::new();
            let mut redo = Vec::new();
            while backward.undo() {
                undo.push(backward.project.clone());
            }
            while forward.redo() {
                redo.push(forward.project.clone());
            }
            [undo, redo]
        }
        struct Directory(PathBuf);
        impl Drop for Directory {
            fn drop(&mut self) {
                fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let directory = Directory(std::env::temp_dir().join(format!(
            "weftforge-export-ui-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        )));
        fs::create_dir(&directory.0).unwrap();
        for (format, label) in [
            (ExportFormat::Dxf, "Export DXF"),
            (ExportFormat::Svg, "Export SVG"),
            (ExportFormat::Obj, "Export OBJ"),
        ] {
            for outcome in ["cancel", "protected", "write", "invalid", "success"] {
                for input in [
                    "ax",
                    "pointer",
                    "tab",
                    "focus",
                    "text",
                    "search",
                    "search_pointer",
                ] {
                    if !matches!(format, ExportFormat::Dxf) && !matches!(input, "ax" | "pointer") {
                        continue;
                    }
                    let pointer = matches!(input, "pointer" | "search_pointer");
                    let ctx = egui::Context::default();
                    ctx.enable_accesskit();
                    let mut app = Studio::empty();
                    app.session_autosave = None;
                    app.editor
                        .execute(Command::AddRectangle {
                            name: "Export panel".into(),
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
                    app.editor
                        .execute(Command::SetGrain {
                            piece,
                            angle: 0.123,
                        })
                        .unwrap();
                    assert!(app.editor.undo());
                    app.selected = Some(piece);
                    app.search = matches!(input, "search" | "search_pointer");
                    app.query = label.into();
                    let before = app.editor.clone();
                    let original_history = history(&before);
                    let source = directory.0.join("source.weftforge");
                    app.editor.project.save(&source).unwrap();
                    let source_bytes = fs::read(&source).unwrap();
                    app.path = (outcome != "write").then(|| source.clone());
                    let mesh = Arc::new(app.editor.project.mesh().unwrap());
                    app.mesh = Some(mesh.clone());
                    let output = directory.0.join("output.export");
                    fs::write(&output, b"previous output").unwrap();
                    let destination = match outcome {
                        "cancel" => None,
                        "protected" => Some(source.clone()),
                        "write" => Some(directory.0.join("missing/output.export")),
                        _ => Some(output.clone()),
                    };
                    let update = frame(&ctx, &mut app, vec![]);
                    let numeric = update
                        .platform_output
                        .accesskit_update
                        .as_ref()
                        .unwrap()
                        .nodes
                        .iter()
                        .find(|(_, node)| node.numeric_value() == Some(0.4))
                        .unwrap()
                        .0;
                    let other = if matches!(input, "tab" | "focus") {
                        let nodes = &update
                            .platform_output
                            .accesskit_update
                            .as_ref()
                            .unwrap()
                            .nodes;
                        let bounds = nodes
                            .iter()
                            .find(|(id, _)| *id == numeric)
                            .unwrap()
                            .1
                            .bounds()
                            .unwrap();
                        let height = nodes
                            .iter()
                            .find(|(_, node)| {
                                node.numeric_value() == Some(0.6)
                                    && node.bounds().is_some_and(|b| {
                                        (b.y0 - bounds.y0).abs() < 1.0 && b.x0 > bounds.x0
                                    })
                            })
                            .unwrap()
                            .0;
                        frame(&ctx, &mut app, vec![action(height, Action::Focus)]);
                        frame(&ctx, &mut app, vec![]);
                        let id = ctx.memory(|memory| memory.focused()).unwrap();
                        let text = ctx.data_mut(|data| data.get_temp::<String>(id)).unwrap();
                        Some((height, id, text))
                    } else {
                        None
                    };
                    frame(&ctx, &mut app, vec![action(numeric, Action::Focus)]);
                    frame(&ctx, &mut app, vec![]);
                    app.draft
                        .get_or_insert_with(|| {
                            InspectorDraft::new(
                                &app.editor.project,
                                app.selected,
                                app.selected_curve,
                            )
                        })
                        .resolution = 0.08;
                    let text = if outcome == "invalid" { "0" } else { "0.450" };
                    let initial_text = if input == "text" { "0.4" } else { text };
                    let update = frame(
                        &ctx,
                        &mut app,
                        vec![
                            key(egui::Key::A, egui::Modifiers::COMMAND),
                            egui::Event::Text(initial_text.into()),
                        ],
                    );
                    let focused = ctx.memory(|memory| memory.focused()).unwrap();
                    let mut expected_draft = app.draft.as_ref().unwrap().clone();
                    expected_draft.dimensions.as_mut().unwrap()[0] =
                        if outcome == "invalid" { 0.0 } else { 0.45 };
                    let pending = expected_draft.commands(&before.project);
                    assert_eq!(app.editor.project, before.project);
                    assert_eq!(
                        ctx.data_mut(|data| data.get_temp::<String>(focused)),
                        Some(initial_text.into())
                    );
                    let update = if !app.search {
                        let (file, _) = button(&update, "File");
                        frame(&ctx, &mut app, vec![action(file, Action::Click)]);
                        frame(&ctx, &mut app, vec![])
                    } else {
                        update
                    };
                    let (export, position) = button(&update, label);
                    ctx.data_mut(|data| {
                        data.insert_temp(egui::Id::new("test-export-destination"), destination)
                    });
                    if pointer {
                        for pressed in [true, false] {
                            frame(
                                &ctx,
                                &mut app,
                                vec![
                                    egui::Event::PointerMoved(position),
                                    egui::Event::PointerButton {
                                        pos: position,
                                        button: egui::PointerButton::Primary,
                                        pressed,
                                        modifiers: egui::Modifiers::NONE,
                                    },
                                ],
                            );
                            if pressed {
                                assert_eq!(
                                    app.editor.project, before.project,
                                    "{label}/{outcome}: pointer press"
                                );
                                assert_eq!(history(&app.editor), original_history);
                            }
                        }
                    } else {
                        let mut events = match input {
                            "tab" => vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                            "focus" => vec![action(other.as_ref().unwrap().0, Action::Focus)],
                            "text" => vec![
                                key(egui::Key::A, egui::Modifiers::COMMAND),
                                egui::Event::Text(text.into()),
                            ],
                            _ => Vec::new(),
                        };
                        events.push(action(export, Action::Click));
                        frame(&ctx, &mut app, events);
                    }
                    let written = outcome == "success" && !matches!(format, ExportFormat::Obj);
                    if written {
                        let mut expected = before.clone();
                        expected.execute(Command::Transaction(pending)).unwrap();
                        assert_eq!(app.editor.project, expected.project);
                        assert_eq!(history(&app.editor), history(&expected));
                        assert!(app.dirty);
                        assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(focused)), None);
                        if let Some((_, id, _)) = &other {
                            assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(*id)), None);
                        }
                        let bytes = fs::read_to_string(&output).unwrap();
                        assert!(bytes.contains(if matches!(format, ExportFormat::Dxf) {
                            "11\n450\n"
                        } else {
                            "L 450 "
                        }));
                        frame(&ctx, &mut app, vec![]);
                        let update = frame(
                            &ctx,
                            &mut app,
                            vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                        );
                        assert_eq!(app.editor.project, expected.project);
                        let (undo, _) = button(&update, "Undo");
                        let update = frame(&ctx, &mut app, vec![action(undo, Action::Click)]);
                        assert!(expected.undo());
                        assert_eq!(app.editor.project, expected.project);
                        let (redo, _) = button(&update, "Redo");
                        frame(&ctx, &mut app, vec![action(redo, Action::Click)]);
                        assert!(expected.redo());
                        assert_eq!(app.editor.project, expected.project);
                    } else {
                        assert_eq!(
                            app.editor.project, before.project,
                            "{label}/{outcome}: failed or cancelled export"
                        );
                        assert_eq!(history(&app.editor), original_history);
                        assert!(!app.dirty);
                        assert!(!app.needs_mesh && app.job.is_none());
                        assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
                        assert_eq!(
                            app.draft.as_ref().unwrap().commands(&before.project),
                            pending
                        );
                        let retained = other.as_ref().map_or(focused, |(_, id, _)| *id);
                        let retained_text =
                            other.as_ref().map_or(text, |(_, _, value)| value.as_str());
                        assert_eq!(
                            ctx.memory(|memory| memory.focused()),
                            Some(retained),
                            "{label}/{outcome}/{input}"
                        );
                        assert_eq!(
                            ctx.data_mut(|data| data.get_temp::<String>(retained)),
                            Some(retained_text.into())
                        );
                        assert_eq!(fs::read(&output).unwrap(), b"previous output");
                        frame(&ctx, &mut app, vec![]);
                        assert_eq!(
                            app.editor.project, before.project,
                            "{label}/{outcome}: delayed blur"
                        );
                        assert_eq!(history(&app.editor), original_history);
                        assert_eq!(
                            app.draft.as_ref().unwrap().commands(&before.project),
                            pending
                        );
                        assert_eq!(ctx.memory(|memory| memory.focused()), Some(retained));
                        assert_eq!(
                            ctx.data_mut(|data| data.get_temp::<String>(retained)),
                            Some(retained_text.into())
                        );
                        if outcome != "invalid" {
                            let mut expected = before.clone();
                            expected.execute(Command::Transaction(pending)).unwrap();
                            frame(
                                &ctx,
                                &mut app,
                                vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                            );
                            assert_eq!(
                                app.editor.project, expected.project,
                                "{label}/{outcome}/{input}: intentional later Tab"
                            );
                        }
                    }
                    assert_eq!(fs::read(&source).unwrap(), source_bytes);
                    app.cancel();
                }
            }
        }
    }

    #[test]
    fn obj_export_uses_ready_rest_or_current_drape_without_changing_construction() {
        let directory = std::env::temp_dir().join(format!(
            "weftforge-export-obj-ui-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let ctx = egui::Context::default();
        let mut app = Studio::empty();
        app.editor
            .execute(Command::AddRectangle {
                name: "OBJ panel".into(),
                origin: [2., 1.],
                width: 0.4,
                height: 0.6,
            })
            .unwrap();
        let before = app.editor.project.clone();
        let mesh = Arc::new(before.mesh().unwrap());
        app.mesh = Some(mesh.clone());
        for draped in [false, true] {
            if draped {
                let mut cloth = weftforge_core::simulation::create_cloth(&before, &mesh).unwrap();
                cloth.step(Default::default(), || false).unwrap();
                assert_ne!(cloth.positions(), mesh.positions);
                app.cloth = Some(Arc::new(cloth));
            }
            let path = directory.join(if draped { "draped.obj" } else { "rest.obj" });
            app.draft = Some(InspectorDraft::new(&before, None, None));
            assert!(app.export_to_path(&ctx, ExportFormat::Obj, Some(&path), None));
            let positions: Vec<[f64; 3]> = fs::read_to_string(&path)
                .unwrap()
                .lines()
                .filter_map(|line| line.strip_prefix("v "))
                .map(|line| {
                    line.split_whitespace()
                        .map(|value| value.parse().unwrap())
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap()
                })
                .collect();
            let expected = app
                .cloth
                .as_ref()
                .map_or(mesh.positions.as_slice(), |cloth| cloth.positions());
            assert_eq!(positions.len(), expected.len());
            for (actual, expected) in positions.iter().zip(expected) {
                for axis in 0..3 {
                    assert!((actual[axis] - expected[axis]).abs() <= 1e-15);
                }
            }
            assert_eq!(app.editor.project, before);
            assert!(!app.dirty && !app.needs_mesh && app.job.is_none());
            assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
        }
        let path = directory.join("stale.obj");
        fs::write(&path, b"retained output").unwrap();
        let mut stale = (*mesh).clone();
        stale.doc_revision -= 1;
        app.mesh = Some(Arc::new(stale));
        assert!(!app.export_to_path(&ctx, ExportFormat::Obj, Some(&path), None));
        assert_eq!(fs::read(&path).unwrap(), b"retained output");
        assert_eq!(app.editor.project, before);
        fs::remove_dir_all(directory).unwrap();
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
        for label in ["Add rectangle", "Add ellipse", "Undo"] {
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
    fn ellipse_accessible_button_and_search_commit_once_and_preserve_failed_drafts() {
        use egui::accesskit::{Action, ActionRequest, Role, TreeId};
        fn frame(
            ctx: &egui::Context,
            app: &mut Studio,
            search: bool,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1450., 2000.))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    if search {
                        eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest());
                    } else {
                        app.inspector(ui);
                    }
                },
            )
        }
        fn activate(ctx: &egui::Context, app: &mut Studio, search: bool) {
            app.search = search;
            app.query = "ellipse".into();
            let output = frame(ctx, app, search, vec![]);
            let palette_action = search.then(|| command_search_action(&output, "Add ellipse"));
            let (target_node, node) = output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .into_iter()
                .find(|(id, node)| {
                    node.label() == Some("Add ellipse")
                        && node.supports_action(Action::Click)
                        && palette_action.is_none_or(|expected| *id == expected)
                })
                .expect("accessible ellipse action");
            assert_eq!(node.role(), Role::Button);
            frame(
                ctx,
                app,
                search,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Click,
                    target_tree: TreeId::ROOT,
                    target_node,
                    data: None,
                })],
            );
        }
        for search in [false, true] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = Studio::empty();
            app.editor
                .execute(Command::AddRectangle {
                    name: "Existing piece".into(),
                    origin: [1., 0.],
                    width: 0.3,
                    height: 0.4,
                })
                .unwrap();
            let selected = app.editor.project.pieces[0].id;
            app.select(&ctx, Some(selected));
            let before = app.editor.project.clone();
            let mut draft = InspectorDraft::new(&before, app.selected, app.selected_curve);
            draft.resolution = 0.09;
            app.draft = Some(draft);
            app.name = "Original native ellipse".into();
            app.coordinate = [-0.2, 0.3];
            app.size = [0.6, 0.4];
            activate(&ctx, &mut app, search);
            assert_eq!(app.editor.project.revision, before.revision + 1);
            assert_eq!(app.editor.project.resolution, 0.09);
            assert_eq!(app.editor.project.pieces.len(), 2);
            let ellipse = app.editor.project.pieces.last().unwrap();
            assert_eq!(ellipse.name, "Original native ellipse");
            assert_eq!(app.selected, Some(ellipse.id));
            let bounds = app.editor.project.piece_bounds(ellipse.id).unwrap();
            assert!((bounds[0][0] + 0.2).abs() < 1e-12 && (bounds[0][1] - 0.3).abs() < 1e-12);
            assert!((bounds[1][0] - 0.4).abs() < 1e-12 && (bounds[1][1] - 0.7).abs() < 1e-12);
            assert!(
                ellipse.boundary.iter().all(|id| app
                    .editor
                    .project
                    .curve(*id)
                    .unwrap()
                    .controls
                    .is_some())
            );
            assert!(app.dirty && app.needs_mesh);
            assert!(!app.search);
            let authored = app.editor.project.clone();
            frame(&ctx, &mut app, search, vec![]);
            assert_eq!(app.editor.project, authored);
            assert!(app.editor.undo());
            app.rebuild();
            assert_eq!(app.editor.project.points, before.points);
            assert_eq!(app.editor.project.curves, before.curves);
            assert_eq!(app.editor.project.resolution, before.resolution);
            assert!(app.editor.redo());
            app.rebuild();
            assert_eq!(app.editor.project.curves, authored.curves);
            app.name.clear();
            let before = app.editor.project.clone();
            let selection = app.selected;
            let history = (app.editor.can_undo(), app.editor.can_redo());
            let mut draft = InspectorDraft::new(&before, app.selected, app.selected_curve);
            draft.resolution = 0.11;
            let pending = draft.commands(&before);
            app.draft = Some(draft);
            activate(&ctx, &mut app, search);
            assert_eq!(app.editor.project, before);
            assert_eq!(app.selected, selection);
            assert_eq!((app.editor.can_undo(), app.editor.can_redo()), history);
            assert_eq!(app.draft.as_ref().unwrap().commands(&before), pending);
            assert!(app.message.contains("Names must be"));
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
        app.select_curve(&ctx, Some((first, curve)));
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
        app.select(&ctx, Some(first));
        let output = frame(&ctx, &mut app, vec![]);
        assert_eq!(numeric_field(&output, "Size m", 0), width_field);
        app.select_curve(&ctx, Some((first, curve)));
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
        app.select_curve(&ctx, Some((second, other_curve)));
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
        app.select(&ctx, Some(second));
        let output = frame(&ctx, &mut app, vec![]);
        assert_eq!(numeric_field(&output, "Mesh edge length m", 0), mesh_field);
        assert_eq!(
            focused(&output),
            (mesh_field, Some("0.085".into()), Some(0.085))
        );
        app.select(&ctx, Some(first));
        let output = frame(&ctx, &mut app, vec![]);
        assert_eq!(numeric_field(&output, "Mesh edge length m", 0), mesh_field);
        assert_eq!(app.editor.project.resolution, 0.085);
        assert_eq!(
            app.editor.project.curve(other_curve).unwrap().controls,
            other_controls
        );

        app.select_curve(&ctx, Some((first, curve)));
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
    fn focusing_small_and_large_piece_sizes_preserves_document_and_history() {
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
        fn field(output: &egui::FullOutput, value: f64) -> NodeId {
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
        fn focus(target_node: NodeId) -> egui::Event {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Focus,
                target_tree: TreeId::ROOT,
                target_node,
                data: None,
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
        for dimensions in [[0.005, 0.006], [6., 0.006]] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = Studio::empty();
            app.editor
                .execute(Command::AddRectangle {
                    name: "Valid dimensions".into(),
                    origin: [0., 0.],
                    width: dimensions[0],
                    height: dimensions[1],
                })
                .unwrap();
            let piece = app.editor.project.pieces[0].id;
            app.editor
                .execute(Command::SetGrain { piece, angle: 0.25 })
                .unwrap();
            assert!(app.editor.undo());
            app.selected = Some(piece);
            let before = app.editor.project.clone();
            let history = (app.editor.can_undo(), app.editor.can_redo());
            for value in dimensions {
                let output = frame(&ctx, &mut app, vec![]);
                frame(&ctx, &mut app, vec![focus(field(&output, value))]);
                frame(&ctx, &mut app, vec![]);
                frame(
                    &ctx,
                    &mut app,
                    vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                );
                frame(&ctx, &mut app, vec![]);
                assert_eq!(app.editor.project, before, "untouched size {value}");
                assert_eq!((app.editor.can_undo(), app.editor.can_redo()), history);
                assert!(
                    app.draft
                        .as_ref()
                        .is_none_or(|draft| draft.commands(&before).is_empty())
                );
            }
            let output = frame(&ctx, &mut app, vec![]);
            frame(&ctx, &mut app, vec![focus(field(&output, dimensions[0]))]);
            frame(&ctx, &mut app, vec![]);
            frame(
                &ctx,
                &mut app,
                vec![
                    key(egui::Key::A, egui::Modifiers::COMMAND),
                    egui::Event::Text("0".into()),
                ],
            );
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
            );
            assert!(
                app.message
                    .contains("Piece dimensions must be positive metres")
            );
            assert_eq!(
                app.editor.project, before,
                "invalid size must not replace the document"
            );
            assert_eq!((app.editor.can_undo(), app.editor.can_redo()), history);
            let dir = exit_test_directory("size-focus");
            let path = dir.join("valid-sizes.weftforge");
            app.path = Some(path.clone());
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::S, egui::Modifiers::COMMAND)],
            );
            let saved = Project::load(path).unwrap();
            fs::remove_dir_all(dir).unwrap();
            assert_eq!(saved, before);
        }
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
        let ctx = egui::Context::default();
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
        app.select_curve(&ctx, Some((piece, curve)));
        let before = app.editor.project.clone();
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
        app.select_curve(&ctx, Some((piece, next)));
        let draft = InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve);
        assert_eq!(draft.curve.unwrap().id, next);
        app.select(&ctx, Some(piece));
        assert_eq!(app.selected_curve, None);
    }

    #[test]
    fn selected_join_button_commits_one_revision_and_rejects_pending_edits_atomically() {
        use egui::accesskit::{Action, ActionRequest, TreeId};
        fn frame(
            ctx: &egui::Context,
            app: &mut Studio,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700., 2000.))),
                    events,
                    ..Default::default()
                },
                |ui| app.inspector(ui),
            )
        }
        fn click_join(ctx: &egui::Context, app: &mut Studio) {
            let output = frame(ctx, app, vec![]);
            let target_node = output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| {
                    node.label() == Some("Join with next boundary")
                        && node.supports_action(Action::Click)
                })
                .unwrap()
                .0;
            frame(
                ctx,
                app,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Click,
                    target_tree: TreeId::ROOT,
                    target_node,
                    data: None,
                })],
            );
        }
        let ctx = egui::Context::default();
        let mut app = Studio::empty();
        app.editor
            .execute(Command::AddRectangle {
                name: "Join panel".into(),
                origin: [0., 0.],
                width: 0.4,
                height: 0.6,
            })
            .unwrap();
        let piece = app.editor.project.pieces[0].id;
        let curve = app.editor.project.pieces[0].boundary[2];
        app.editor
            .execute(Command::SetCurveControls {
                curve,
                controls: Some([[0.3, 0.8], [0.1, 0.8]]),
            })
            .unwrap();
        app.editor
            .execute(Command::SplitCurve { curve, t: 0.4 })
            .unwrap();
        let right = app.editor.project.curves.last().unwrap().id;
        app.select_curve(&ctx, Some((piece, curve)));
        let before = app.editor.project.clone();
        let mut draft = InspectorDraft::new(&before, app.selected, app.selected_curve);
        draft.resolution = 0.09;
        app.draft = Some(draft);
        let (_sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        app.job = Some(Job {
            id: 0,
            revision: before.revision,
            kind: JobKind::Mesh,
            batch: None,
            completed_steps: Arc::new(AtomicU64::new(0)),
            cancel: cancel.clone(),
            receiver,
        });
        app.pending_batch = Some(SimulationBatch::drape());
        ctx.enable_accesskit();
        click_join(&ctx, &mut app);
        assert_eq!(app.editor.project.revision, before.revision + 1);
        assert_eq!(app.editor.project.resolution, 0.09);
        assert_eq!(app.editor.project.curves.len(), before.curves.len() - 1);
        assert_eq!(app.selected_curve, Some(curve));
        assert!(app.editor.project.curve(right).is_none());
        assert!(app.draft.is_none());
        assert!(app.dirty && app.needs_mesh);
        assert!(cancel.load(Ordering::Relaxed));
        assert!(app.pending_batch.is_none());
        let joined = app.editor.project.clone();
        assert!(app.editor.undo());
        app.rebuild();
        assert_eq!(app.editor.project.curves, before.curves);
        assert_eq!(app.editor.project.resolution, before.resolution);
        app.select_curve(&ctx, Some((piece, right)));
        let before_invalid = app.editor.project.clone();
        let mut draft = InspectorDraft::new(&before_invalid, app.selected, app.selected_curve);
        draft.resolution = 0.08;
        app.draft = Some(draft);
        app.dirty = false;
        click_join(&ctx, &mut app);
        assert_eq!(app.editor.project, before_invalid);
        assert_eq!(app.selected_curve, Some(right));
        assert!(!app.dirty);
        assert!(app.editor.can_redo());
        assert!(app.message.contains("two straight"));
        assert_eq!(app.draft.as_ref().unwrap().resolution, 0.08);
        assert!(app.editor.redo());
        app.rebuild();
        assert_eq!(app.editor.project.curves, joined.curves);
        assert_eq!(app.editor.project.resolution, joined.resolution);
        assert_eq!(app.selected_curve, None);
    }

    #[test]
    fn join_focus_keeps_failed_drafts_and_clears_successful_control_text_before_tab() {
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
        fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }
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
        #[derive(Clone, Copy, Debug)]
        enum JoinInput {
            Ax,
            Pointer,
            Tab,
            Focus,
        }
        for success in [true, false] {
            for input in [
                JoinInput::Ax,
                JoinInput::Pointer,
                JoinInput::Tab,
                JoinInput::Focus,
            ] {
                let pointer = matches!(input, JoinInput::Pointer);
                let coalesced = matches!(input, JoinInput::Tab | JoinInput::Focus);
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let mut app = Studio::empty();
                app.editor
                    .execute(Command::AddRectangle {
                        name: "Focused join panel".into(),
                        origin: [0., 0.],
                        width: 0.4,
                        height: 0.6,
                    })
                    .unwrap();
                let piece = app.editor.project.pieces[0].id;
                let left = app.editor.project.pieces[0].boundary[2];
                app.editor
                    .execute(Command::SetCurveControls {
                        curve: left,
                        controls: Some([[0.3, 0.8], [0.1, 0.8]]),
                    })
                    .unwrap();
                app.editor
                    .execute(Command::SplitCurve {
                        curve: left,
                        t: 0.4,
                    })
                    .unwrap();
                let selected = if success {
                    left
                } else {
                    app.editor.project.curves.last().unwrap().id
                };
                app.editor
                    .execute(Command::SetGrain {
                        piece,
                        angle: 0.123,
                    })
                    .unwrap();
                assert!(app.editor.undo());
                app.select_curve(&ctx, Some((piece, selected)));
                let before = app.editor.project.clone();
                let original_x = before.curve(selected).unwrap().controls.unwrap()[0][0];
                let draft = InspectorDraft::new(&before, app.selected, app.selected_curve);
                app.draft = Some(draft);
                let output = frame(&ctx, &mut app, vec![]);
                let numeric = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.numeric_value()
                            == Some(if success || coalesced {
                                original_x
                            } else {
                                before.resolution
                            })
                    })
                    .unwrap()
                    .0;
                let current_field = if coalesced {
                    let nodes = &output
                        .platform_output
                        .accesskit_update
                        .as_ref()
                        .unwrap()
                        .nodes;
                    let bounds = nodes
                        .iter()
                        .find(|(id, _)| *id == numeric)
                        .unwrap()
                        .1
                        .bounds()
                        .unwrap();
                    let y = nodes
                        .iter()
                        .filter(|(_, node)| {
                            node.numeric_value().is_some()
                                && node.bounds().is_some_and(|b| {
                                    (b.y0 - bounds.y0).abs() < 1.0 && b.x0 > bounds.x0
                                })
                        })
                        .min_by(|a, b| {
                            a.1.bounds()
                                .unwrap()
                                .x0
                                .total_cmp(&b.1.bounds().unwrap().x0)
                        })
                        .unwrap()
                        .0;
                    frame(&ctx, &mut app, vec![action(y, Action::Focus)]);
                    frame(&ctx, &mut app, vec![]);
                    Some((y, ctx.memory(|memory| memory.focused()).unwrap()))
                } else {
                    None
                };
                let current_text = current_field
                    .and_then(|(_, id)| ctx.data_mut(|data| data.get_temp::<String>(id)));
                frame(&ctx, &mut app, vec![action(numeric, Action::Focus)]);
                frame(&ctx, &mut app, vec![]);
                if success || coalesced {
                    app.draft
                        .get_or_insert_with(|| {
                            InspectorDraft::new(&before, app.selected, app.selected_curve)
                        })
                        .resolution = 0.08;
                }
                let text = if success || coalesced {
                    format!("{original_x}0")
                } else {
                    "0.080".into()
                };
                let output = frame(
                    &ctx,
                    &mut app,
                    vec![
                        key(egui::Key::A, egui::Modifiers::COMMAND),
                        egui::Event::Text(text.clone()),
                    ],
                );
                let focused = ctx.memory(|memory| memory.focused()).unwrap();
                assert_eq!(
                    ctx.data_mut(|data| data.get_temp::<String>(focused)),
                    Some(text.clone())
                );
                assert_eq!(app.editor.project, before);
                let pending = app.draft.as_ref().unwrap().commands(&before);
                let (join, pos) = button(&output, "Join with next boundary");
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
                        if pressed || !success {
                            assert_eq!(
                                app.editor.project, before,
                                "pointer pressed={pressed}, success={success}"
                            );
                            assert!(app.editor.can_redo());
                            assert_eq!(app.draft.as_ref().unwrap().commands(&before), pending);
                            assert_eq!(
                                ctx.data_mut(|data| data.get_temp::<String>(focused)),
                                Some(text.clone())
                            );
                        }
                    }
                } else {
                    let mut events = match input {
                        JoinInput::Tab => vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                        JoinInput::Focus => vec![action(current_field.unwrap().0, Action::Focus)],
                        _ => Vec::new(),
                    };
                    events.push(action(join, Action::Click));
                    frame(&ctx, &mut app, events);
                }
                if success {
                    let joined = app.editor.project.clone();
                    assert_eq!(joined.revision, before.revision + 1, "pointer={pointer}");
                    assert!(
                        (joined.curve(left).unwrap().controls.unwrap()[0][0] - 0.3).abs() <= 1e-12
                    );
                    assert_eq!(
                        ctx.data_mut(|data| data.get_temp::<String>(focused)),
                        None,
                        "pointer={pointer}"
                    );
                    if let Some((_, id)) = current_field {
                        assert_eq!(
                            ctx.data_mut(|data| data.get_temp::<String>(id)),
                            None,
                            "current numeric buffer survived coalesced Join"
                        );
                    }
                    frame(&ctx, &mut app, vec![]);
                    let output = frame(
                        &ctx,
                        &mut app,
                        vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                    );
                    assert_eq!(
                        app.editor.project, joined,
                        "Tab replayed old child controls; pointer={pointer}"
                    );
                    let (undo, _) = button(&output, "Undo");
                    frame(&ctx, &mut app, vec![action(undo, Action::Click)]);
                    let undone = app.editor.project.clone();
                    assert_eq!(undone.curves, before.curves);
                    assert_eq!(undone.resolution, before.resolution);
                    assert!(app.editor.can_redo());
                    let output = frame(
                        &ctx,
                        &mut app,
                        vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                    );
                    assert_eq!(app.editor.project, undone);
                    assert!(app.editor.can_redo());
                    let (redo, _) = button(&output, "Redo");
                    frame(&ctx, &mut app, vec![action(redo, Action::Click)]);
                    assert_eq!(app.editor.project.curves, joined.curves);
                    assert_eq!(app.editor.project.resolution, joined.resolution);
                } else {
                    assert_eq!(app.editor.project, before, "pointer={pointer}");
                    assert!(app.editor.can_redo());
                    assert_eq!(
                        app.draft.as_ref().unwrap().commands(&before),
                        pending,
                        "pointer={pointer}"
                    );
                    if let Some((_, id)) = current_field {
                        assert_eq!(
                            ctx.data_mut(|data| data.get_temp::<String>(id)),
                            current_text
                        );
                    } else {
                        assert_eq!(
                            ctx.data_mut(|data| data.get_temp::<String>(focused)),
                            Some(text),
                            "pointer={pointer}"
                        );
                    }
                    assert_eq!(
                        ctx.memory(|memory| memory.focused()),
                        Some(current_field.map_or(focused, |(_, id)| id)),
                        "pointer={pointer}"
                    );
                    frame(&ctx, &mut app, vec![]);
                    assert_eq!(app.editor.project, before, "idle after rejected {input:?}");
                    assert!(app.editor.can_redo());
                    assert_eq!(app.draft.as_ref().unwrap().commands(&before), pending);
                    if let Some((_, id)) = current_field {
                        assert_eq!(
                            ctx.data_mut(|data| data.get_temp::<String>(id)),
                            current_text
                        );
                    }
                    assert_eq!(
                        ctx.memory(|memory| memory.focused()),
                        Some(current_field.map_or(focused, |(_, id)| id))
                    );
                    frame(
                        &ctx,
                        &mut app,
                        vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                    );
                    assert_eq!(app.editor.project.revision, before.revision + 1);
                    assert_eq!(app.editor.project.resolution, 0.08);
                    assert!(!app.editor.can_redo());
                }
                app.cancel();
            }
        }
    }

    #[test]
    fn seam_reverse_button_keeps_focused_drafts_atomic_and_draws_endpoint_pairs() {
        use egui::accesskit::{Action, ActionRequest, NodeId, TreeId};
        fn frame(
            ctx: &egui::Context,
            app: &mut Studio,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1450., 2200.))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest());
                    let canvas = ctx.read_response(app.pattern_canvas.unwrap()).unwrap().rect;
                    ctx.data_mut(|data| {
                        data.insert_temp(egui::Id::new("test-current-pattern-canvas"), canvas)
                    });
                },
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
        fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }
        }
        fn buttons(output: &egui::FullOutput, label: &str) -> Vec<(NodeId, Pos2)> {
            let mut buttons: Vec<_> = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.label() == Some(label) && node.supports_action(Action::Click)
                })
                .map(|(id, node)| {
                    let b = node.bounds().unwrap();
                    (
                        *id,
                        Pos2::new(((b.x0 + b.x1) * 0.5) as f32, ((b.y0 + b.y1) * 0.5) as f32),
                    )
                })
                .collect();
            buttons.sort_by(|a, b| a.1.y.total_cmp(&b.1.y).then(a.1.x.total_cmp(&b.1.x)));
            buttons
        }
        fn guides(output: &egui::FullOutput) -> Vec<[Pos2; 2]> {
            output
                .shapes
                .iter()
                .filter_map(|shape| match shape.shape {
                    egui::Shape::LineSegment { points, stroke }
                        if stroke == Stroke::new(1., Color32::from_rgb(245, 175, 87)) =>
                    {
                        Some(points)
                    }
                    _ => None,
                })
                .collect()
        }
        for success in [true, false] {
            for input in 0..3 {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let mut app = Studio::empty();
                for (name, origin) in [("Front", [0., 0.]), ("Back", [1., 0.])] {
                    app.editor
                        .execute(Command::AddRectangle {
                            name: name.into(),
                            origin,
                            width: 0.4,
                            height: 0.6,
                        })
                        .unwrap();
                }
                let piece = app.editor.project.pieces[0].id;
                let a = app.editor.project.pieces[0].boundary[2];
                let b = app.editor.project.pieces[1].boundary[0];
                app.editor
                    .execute(Command::Transaction(vec![
                        Command::SetCurveControls {
                            curve: a,
                            controls: Some([[0.3, 0.8], [0.1, 0.8]]),
                        },
                        Command::AddSeam {
                            name: "Same seam name".into(),
                            a: CurveInterval {
                                curve: a,
                                t0: 0.15,
                                t1: 0.85,
                            },
                            b: CurveInterval {
                                curve: b,
                                t0: 0.9,
                                t1: 0.1,
                            },
                            correspondence: vec![[0., 0.], [0.4, 0.65], [1., 1.]],
                        },
                        Command::SplitCurve { curve: a, t: 0.5 },
                        Command::SetGrain {
                            piece,
                            angle: 0.125,
                        },
                    ]))
                    .unwrap();
                assert_eq!(app.editor.project.seams.len(), 2);
                assert_eq!(
                    app.editor.project.seams[0].name,
                    app.editor.project.seams[1].name
                );
                if !success {
                    let mut project = app.editor.project.clone();
                    project.revision = u64::MAX - 2;
                    app.editor = Editor::new(project).unwrap();
                }
                app.editor
                    .execute(Command::SetGrain {
                        piece,
                        angle: 0.375,
                    })
                    .unwrap();
                assert!(app.editor.undo());
                assert!(app.editor.can_redo());
                app.select_curve(&ctx, Some((piece, a)));
                let before = app.editor.project.clone();
                frame(&ctx, &mut app, vec![]); // Settle the measured footer before caching screen positions.
                let output = frame(&ctx, &mut app, vec![]);
                let original_guides = guides(&output);
                assert_eq!(original_guides.len(), 4);
                let canvas = ctx
                    .data_mut(|data| {
                        data.get_temp::<Rect>(egui::Id::new("test-current-pattern-canvas"))
                    })
                    .unwrap();
                for (seam, pair) in before.seams.iter().zip(original_guides.chunks_exact(2)) {
                    let point = |curve, t| {
                        app.to_screen(
                            before.curve(curve).unwrap().evaluate(&before, t).unwrap(),
                            canvas,
                        )
                    };
                    assert_eq!(
                        pair[0],
                        [
                            point(seam.a.curve, seam.a.t0),
                            point(seam.b.curve, seam.b.t0)
                        ]
                    );
                    assert_eq!(
                        pair[1],
                        [
                            point(seam.a.curve, seam.a.t1),
                            point(seam.b.curve, seam.b.t1)
                        ]
                    );
                }
                let reverse_ids = buttons(&output, "Reverse");
                assert_eq!(reverse_ids.len(), 2);
                assert_ne!(reverse_ids[0].0, reverse_ids[1].0);
                let grain = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(_, node)| node.numeric_value() == Some(0.125))
                    .unwrap()
                    .0;
                frame(&ctx, &mut app, vec![action(grain, Action::Focus)]);
                frame(&ctx, &mut app, vec![]);
                app.draft.as_mut().unwrap().resolution = 0.08;
                let output = frame(
                    &ctx,
                    &mut app,
                    vec![
                        key(egui::Key::A, egui::Modifiers::COMMAND),
                        egui::Event::Text("0.250".into()),
                    ],
                );
                let focused = ctx.memory(|memory| memory.focused()).unwrap();
                assert_eq!(
                    ctx.data_mut(|data| data.get_temp::<String>(focused)),
                    Some("0.250".into())
                );
                let pending = app.draft.as_ref().unwrap().commands(&before);
                assert_eq!(pending.len(), 2);
                assert_eq!(app.editor.project, before);
                let (reverse, pos) = buttons(&output, "Reverse")[1];
                let output = if input == 1 {
                    frame(
                        &ctx,
                        &mut app,
                        vec![
                            egui::Event::PointerMoved(pos),
                            egui::Event::PointerButton {
                                pos,
                                button: egui::PointerButton::Primary,
                                pressed: true,
                                modifiers: egui::Modifiers::NONE,
                            },
                        ],
                    );
                    assert_eq!(app.editor.project, before);
                    assert!(app.editor.can_redo());
                    frame(
                        &ctx,
                        &mut app,
                        vec![
                            egui::Event::PointerMoved(pos),
                            egui::Event::PointerButton {
                                pos,
                                button: egui::PointerButton::Primary,
                                pressed: false,
                                modifiers: egui::Modifiers::NONE,
                            },
                        ],
                    )
                } else {
                    let mut events = Vec::new();
                    if input == 2 {
                        let resolution = output
                            .platform_output
                            .accesskit_update
                            .as_ref()
                            .unwrap()
                            .nodes
                            .iter()
                            .find(|(_, node)| node.numeric_value() == Some(0.08))
                            .unwrap()
                            .0;
                        events.push(action(resolution, Action::Focus));
                    }
                    events.push(action(reverse, Action::Click));
                    frame(&ctx, &mut app, events)
                };
                if !success {
                    assert_eq!(app.editor.project, before);
                    assert!(app.message.contains("revision limit"));
                    assert!(app.editor.can_redo());
                    assert_eq!(app.draft.as_ref().unwrap().commands(&before), pending);
                    assert_eq!(guides(&output), original_guides);
                    let retained = ctx.memory(|memory| memory.focused()).unwrap();
                    let text = if input == 2 { "0.08" } else { "0.250" };
                    assert_eq!(
                        ctx.data_mut(|data| data.get_temp::<String>(retained)),
                        Some(text.into())
                    );
                    frame(&ctx, &mut app, vec![]);
                    assert_eq!(app.editor.project, before);
                    assert!(app.editor.can_redo());
                    assert_eq!(app.draft.as_ref().unwrap().commands(&before), pending);
                    assert_eq!(ctx.memory(|memory| memory.focused()), Some(retained));
                    assert_eq!(
                        ctx.data_mut(|data| data.get_temp::<String>(retained)),
                        Some(text.into())
                    );
                    app.cancel();
                    continue;
                }
                let reversed = app.editor.project.clone();
                assert_eq!(reversed.revision, before.revision + 1);
                assert_eq!(reversed.piece(piece).unwrap().grain_angle, 0.25);
                assert_eq!(reversed.resolution, 0.08);
                let mut expected = before.clone();
                expected.revision = reversed.revision;
                expected.pieces[0].grain_angle = 0.25;
                expected.resolution = 0.08;
                expected.seams[1].b.t0 = before.seams[1].b.t1;
                expected.seams[1].b.t1 = before.seams[1].b.t0;
                assert_eq!(reversed, expected);
                let mut expected_guides = original_guides.clone();
                expected_guides[2][1] = original_guides[3][1];
                expected_guides[3][1] = original_guides[2][1];
                assert_ne!(expected_guides, original_guides);
                assert_eq!(guides(&output), expected_guides);
                assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(focused)), None);
                frame(&ctx, &mut app, vec![]);
                let output = frame(
                    &ctx,
                    &mut app,
                    vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                );
                assert_eq!(
                    app.editor.project, reversed,
                    "numeric cache replay after Reverse input {input}"
                );
                frame(
                    &ctx,
                    &mut app,
                    vec![action(buttons(&output, "Undo")[0].0, Action::Click)],
                );
                let mut undone = app.editor.project.clone();
                undone.revision = before.revision;
                assert_eq!(undone, before);
                let output = frame(
                    &ctx,
                    &mut app,
                    vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                );
                assert!(app.editor.can_redo());
                assert_eq!(guides(&output), original_guides);
                frame(
                    &ctx,
                    &mut app,
                    vec![action(buttons(&output, "Redo")[0].0, Action::Click)],
                );
                let mut redone = app.editor.project.clone();
                redone.revision = reversed.revision;
                assert_eq!(redone, reversed);
                let dir = exit_test_directory("seam-reverse");
                let path = dir.join("reversed.weftforge");
                app.path = Some(path.clone());
                frame(
                    &ctx,
                    &mut app,
                    vec![key(egui::Key::S, egui::Modifiers::COMMAND)],
                );
                let mut reopened = Studio::empty();
                reopened.open(path);
                assert_eq!(reopened.editor.project, app.editor.project);
                assert_eq!(reopened.editor.project.seams, reversed.seams);
                fs::remove_dir_all(dir).unwrap();
                let output = frame(&ctx, &mut app, vec![]);
                frame(
                    &ctx,
                    &mut app,
                    vec![action(buttons(&output, "Reverse")[1].0, Action::Click)],
                );
                let mut expected = before.clone();
                expected.revision = app.editor.project.revision;
                expected.resolution = 0.08;
                expected.pieces[0].grain_angle = 0.25;
                assert_eq!(
                    app.editor.project, expected,
                    "two Reverse clicks must restore the construction with the committed inspector edits"
                );
                let output = frame(&ctx, &mut app, vec![]);
                assert_eq!(guides(&output), original_guides);
                let remove = buttons(&output, "Remove")[0].0;
                frame(&ctx, &mut app, vec![action(remove, Action::Click)]);
                let output = frame(&ctx, &mut app, vec![]);
                assert_eq!(buttons(&output, "Reverse")[0].0, reverse_ids[1].0);
                assert_eq!(app.editor.project.seams, vec![before.seams[1].clone()]);
                app.cancel();
                reopened.cancel();
            }
        }
    }

    fn accepted_drape_studio() -> Studio {
        let mut app = Studio::empty();
        app.session_autosave = None;
        app.editor
            .execute(Command::AddRectangle {
                name: "Stage panel".into(),
                origin: [2., 1.],
                width: 0.1,
                height: 0.1,
            })
            .unwrap();
        let mesh = Arc::new(app.editor.project.mesh().unwrap());
        let mut cloth =
            weftforge_core::simulation::create_cloth(&app.editor.project, &mesh).unwrap();
        cloth.step(Default::default(), || false).unwrap();
        app.mesh = Some(mesh);
        let (sender, receiver) = mpsc::channel();
        app.job = Some(Job {
            id: 0,
            revision: app.editor.project.revision,
            kind: JobKind::Simulation,
            batch: Some(SimulationBatch::drape()),
            completed_steps: Arc::new(AtomicU64::new(0)),
            cancel: Arc::new(AtomicBool::new(false)),
            receiver,
        });
        sender
            .send(Ok(JobOutput::Simulation(Arc::new(cloth))))
            .ok()
            .unwrap();
        app.poll();
        assert_eq!(app.last_stage, "Drape");
        assert!(app.cloth.as_ref().unwrap().simulation_time() > 0.);
        app
    }

    #[test]
    fn stage_resets_when_accepting_mesh_and_preserves_queued_work() {
        for playing in [false, true] {
            let mut app = accepted_drape_studio();
            let old_cloth = app.cloth.clone().unwrap();
            app.execute(Command::SetResolution { metres: 0.025 });
            assert!(app.needs_mesh);
            assert_eq!(app.last_stage, "Drape");
            assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &old_cloth));
            let mesh = Arc::new(app.editor.project.mesh().unwrap());
            let (sender, receiver) = mpsc::channel();
            app.job = Some(Job {
                id: 0,
                revision: app.editor.project.revision,
                kind: JobKind::Mesh,
                batch: None,
                completed_steps: Arc::new(AtomicU64::new(0)),
                cancel: Arc::new(AtomicBool::new(false)),
                receiver,
            });
            app.needs_mesh = false;
            if playing {
                app.playing = true;
            } else {
                app.queue_batch(SimulationBatch::assemble());
            }
            app.poll();
            assert_eq!(app.last_stage, "Drape");
            assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &old_cloth));
            sender.send(Ok(JobOutput::Mesh(mesh.clone()))).ok().unwrap();
            app.poll();
            assert_eq!(app.last_stage, "Rest arrangement");
            assert!(app.cloth.is_none());
            assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
            assert_eq!(app.playing, playing);
            assert_eq!(
                app.pending_batch.map(|batch| batch.stage),
                (!playing).then_some(SimulationBatch::assemble().stage)
            );
        }
    }

    #[test]
    fn stage_and_snapshot_survive_unaccepted_jobs() {
        for kind in [JobKind::Mesh, JobKind::Simulation] {
            for outcome in ["failed", "stale", "cancelled"] {
                let mut app = accepted_drape_studio();
                let mesh = app.mesh.clone().unwrap();
                let cloth = app.cloth.clone().unwrap();
                if kind == JobKind::Mesh {
                    app.execute(Command::SetResolution { metres: 0.025 });
                }
                let (sender, receiver) = mpsc::channel();
                app.job = Some(Job {
                    id: 0,
                    revision: app.editor.project.revision + u64::from(outcome == "stale"),
                    kind,
                    batch: (kind == JobKind::Simulation).then(SimulationBatch::assemble),
                    completed_steps: Arc::new(AtomicU64::new(0)),
                    cancel: Arc::new(AtomicBool::new(false)),
                    receiver,
                });
                if outcome == "cancelled" {
                    app.cancel();
                }
                app.poll();
                assert_eq!(app.last_stage, "Drape");
                assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &cloth));
                let output = if outcome == "failed" {
                    Err(JobError::Failed(
                        "Worker failed; previous state retained.".into(),
                    ))
                } else {
                    Ok(match kind {
                        JobKind::Mesh => JobOutput::Mesh(Arc::new((*mesh).clone())),
                        JobKind::Simulation => JobOutput::Simulation(Arc::new((*cloth).clone())),
                    })
                };
                sender.send(output).ok().unwrap();
                app.poll();
                assert!(app.job.is_none());
                assert_eq!(app.last_stage, "Drape", "{outcome}");
                assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh), "{outcome}");
                assert!(
                    Arc::ptr_eq(app.cloth.as_ref().unwrap(), &cloth),
                    "{outcome}"
                );
            }
        }
    }

    #[test]
    fn worker_failure_survives_late_cancel_and_revision_change() {
        let error = "Worker diagnostic must survive a later UI action.";
        let mut outcomes = Vec::new();
        for action in ["current_play", "current_queue", "cancel", "edit"] {
            let mut app = accepted_drape_studio();
            let mesh = app.mesh.clone().unwrap();
            let cloth = app.cloth.clone().unwrap();
            let (sender, receiver) = mpsc::channel();
            app.job = Some(Job {
                id: 41,
                revision: app.editor.project.revision,
                kind: JobKind::Simulation,
                batch: Some(SimulationBatch::assemble()),
                completed_steps: Arc::new(AtomicU64::new(0)),
                cancel: Arc::new(AtomicBool::new(false)),
                receiver,
            });
            sender
                .send(Err(JobError::Failed(error.into())))
                .ok()
                .unwrap();
            if action == "cancel" {
                app.cancel();
            } else if action == "edit" {
                app.editor
                    .execute(Command::SetResolution { metres: 0.025 })
                    .unwrap();
            }
            if ["current_play", "edit"].contains(&action) {
                app.playing = true;
            } else {
                app.queue_batch(SimulationBatch {
                    steps: 1,
                    settings: app.settings,
                    stage: "Manual step",
                });
            }
            app.poll();
            let job = app.last_job.as_ref().unwrap();
            assert_eq!(job.id, 41);
            assert_eq!(job.progress, Some((0, 60)));
            assert_eq!(job.outcome, Err(JobError::Failed(error.into())));
            assert_eq!(job.cancel_requested, action == "cancel");
            assert_eq!(job.obsolete, action == "edit");
            assert_eq!(app.last_stage, "Drape");
            assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
            assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &cloth));
            outcomes.push((
                action,
                job.status,
                app.message,
                app.playing,
                app.pending_batch.map(|batch| (batch.steps, batch.stage)),
            ));
        }
        assert_eq!(
            outcomes,
            vec![
                ("current_play", JobStatus::Failed, error.into(), false, None),
                (
                    "current_queue",
                    JobStatus::Failed,
                    error.into(),
                    false,
                    None
                ),
                (
                    "cancel",
                    JobStatus::Failed,
                    error.into(),
                    false,
                    Some((1, "Manual step")),
                ),
                ("edit", JobStatus::Failed, error.into(), true, None),
            ],
        );
        let cancel = AtomicBool::new(false);
        let result: Result<(), JobError> = cancellable_core_job(&cancel, |_| {
            cancel.store(true, Ordering::Relaxed);
            Err(weftforge_core::Error(error.into()))
        });
        assert_eq!(result, Err(JobError::Failed(error.into())));
    }

    #[test]
    fn stage_resets_only_on_successful_document_load() {
        let dir = exit_test_directory("stage-load");
        let valid = dir.join("valid.weftforge");
        let corrupt = dir.join("corrupt.weftforge");
        fs::write(&corrupt, b"{not valid").unwrap();
        for recovered in [false, true] {
            let mut app = accepted_drape_studio();
            let project = app.editor.project.clone();
            project.save(&valid).unwrap();
            let mesh = app.mesh.clone().unwrap();
            let cloth = app.cloth.clone().unwrap();
            app.load_document(corrupt.clone(), recovered);
            assert_eq!(app.editor.project, project);
            assert_eq!(app.last_stage, "Drape");
            assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
            assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &cloth));
            app.load_document(valid.clone(), recovered);
            assert_eq!(app.editor.project, project);
            assert_eq!(app.last_stage, "Rest arrangement");
            assert!(app.mesh.is_none() && app.cloth.is_none());
            assert!(app.needs_mesh);
            assert_eq!(app.dirty, recovered);
            assert_eq!(app.path, (!recovered).then(|| valid.clone()));
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn stage_resets_when_construction_becomes_empty() {
        let mut app = accepted_drape_studio();
        app.execute(Command::RemovePiece {
            piece: app.editor.project.pieces[0].id,
        });
        assert!(app.editor.project.pieces.is_empty());
        assert!(app.mesh.is_none() && app.cloth.is_none());
        assert!(!app.needs_mesh);
        assert_eq!(app.last_stage, "Rest arrangement");
    }

    #[test]
    fn cancelled_and_obsolete_jobs_do_not_replace_the_display_snapshot() {
        let ctx = egui::Context::default();
        let (entered_sender, entered_receiver) = mpsc::channel();
        let (release_sender, release_receiver) = mpsc::channel();
        let job = Job::spawn(&ctx, 7, JobKind::Mesh, move |_, _| {
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
        assert!(matches!(
            job.receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
            Err(JobError::Cancelled)
        ));
        let mut app = Studio::empty();
        let old = Arc::new(Mesh::default());
        app.mesh = Some(old.clone());
        let (sender, receiver) = mpsc::channel();
        app.job = Some(Job {
            id: 0,
            revision: 1,
            kind: JobKind::Mesh,
            batch: None,
            completed_steps: Arc::new(AtomicU64::new(0)),
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
    fn task_progress_correlates_real_workers_and_retains_unaccepted_snapshots() {
        fn finish(app: &mut Studio) -> FinishedJob {
            let deadline = Instant::now() + Duration::from_secs(3);
            while app.job.is_some() {
                assert!(Instant::now() < deadline, "bounded worker did not finish");
                app.poll();
                std::thread::yield_now();
            }
            app.last_job.clone().unwrap()
        }

        let ctx = egui::Context::default();
        let mut app = accepted_drape_studio();
        let mesh = app.mesh.clone().unwrap();
        let old_cloth = app.cloth.clone().unwrap();
        let revision = app.editor.project.revision;
        let batch = SimulationBatch {
            steps: 2,
            ..SimulationBatch::assemble()
        };
        let previous = app.last_job.clone();
        app.queue_batch(batch);
        assert_eq!(app.last_job, previous);
        app.start_batch(&ctx, batch);
        let job = app.job.as_ref().unwrap();
        let mut id = job.id;
        assert!(id > 0);
        assert_eq!(job.revision, revision);
        assert_eq!(job.kind, JobKind::Simulation);
        assert_eq!(job.progress().unwrap().1, batch.steps);
        assert!(job.progress().unwrap().0 <= batch.steps as u64);
        assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &old_cloth));
        let accepted = finish(&mut app);
        assert_eq!(accepted.id, id);
        assert_eq!(accepted.revision, revision);
        assert_eq!(accepted.kind, JobKind::Simulation);
        assert_eq!(accepted.progress, Some((2, 2)));
        assert_eq!(accepted.status, JobStatus::Accepted);
        let cloth = app.cloth.clone().unwrap();
        assert!(!Arc::ptr_eq(&cloth, &old_cloth));
        assert!((cloth.simulation_time() - old_cloth.simulation_time() - 2.0 / 60.0).abs() < 1e-12);

        for status in [
            JobStatus::Failed,
            JobStatus::Cancelled,
            JobStatus::Discarded,
        ] {
            let mut attempt = batch;
            if status == JobStatus::Failed {
                attempt.settings.dt = 0.0;
            }
            app.start_batch(&ctx, attempt);
            let job = app.job.as_ref().unwrap();
            assert!(job.id > id);
            id = job.id;
            assert_eq!(job.revision, revision);
            if status == JobStatus::Cancelled {
                app.cancel();
            } else if status == JobStatus::Discarded {
                // Change only the revision eligibility; let the real worker finish its batch.
                app.editor
                    .execute(Command::SetResolution { metres: 0.025 })
                    .unwrap();
            }
            let finished = finish(&mut app);
            assert_eq!(finished.id, id);
            assert_eq!(finished.revision, revision);
            assert_eq!(finished.kind, JobKind::Simulation);
            assert_eq!(finished.status, status);
            let (completed, total) = finished.progress.unwrap();
            assert_eq!(total, 2);
            match status {
                JobStatus::Failed => assert_eq!(completed, 0),
                JobStatus::Discarded => assert_eq!(completed, 2),
                JobStatus::Cancelled => assert!(completed <= 2),
                JobStatus::Accepted => unreachable!(),
            }
            assert_eq!(app.last_stage, batch.stage);
            assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &cloth));
            assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
        }

        let discarded = app.last_job.clone();
        app.queue_batch(batch);
        app.rebuild();
        assert_eq!(app.last_job, discarded);
        app.start_mesh(&ctx);
        let job = app.job.as_ref().unwrap();
        assert!(job.id > id);
        id = job.id;
        assert_eq!(job.kind, JobKind::Mesh);
        assert!(job.progress().is_none());
        assert_eq!(app.last_job, discarded);
        app.reset_simulation();
        assert_eq!(app.last_job, discarded);
        assert!(app.cloth.is_none());
        let meshed = finish(&mut app);
        assert_eq!(meshed.id, id);
        assert_eq!(meshed.revision, app.editor.project.revision);
        assert_eq!(meshed.kind, JobKind::Mesh);
        assert_eq!(meshed.status, JobStatus::Accepted);
        assert!(meshed.progress.is_none());
        app.execute(Command::RemovePiece {
            piece: app.editor.project.pieces[0].id,
        });
        assert!(app.editor.project.pieces.is_empty());
        assert_eq!(app.last_job, Some(meshed));
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
        app.select_curve(&egui::Context::default(), Some((piece, curve)));
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
    mod ordered_sewing {
        use super::*;
        use egui::accesskit::{Action, ActionData, ActionRequest, NodeId, TreeId};

        fn frame(
            ctx: &egui::Context,
            app: &mut Studio,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1450., 2400.))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest());
                    let canvas = ctx.read_response(app.pattern_canvas.unwrap()).unwrap().rect;
                    ctx.data_mut(|data| {
                        data.insert_temp(egui::Id::new("test-current-pattern-canvas"), canvas)
                    });
                },
            )
        }
        fn action(id: NodeId, action: Action) -> egui::Event {
            egui::Event::AccessKitActionRequest(ActionRequest {
                action,
                target_tree: TreeId::ROOT,
                target_node: id,
                data: None,
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
        fn buttons(output: &egui::FullOutput, label: &str) -> Vec<NodeId> {
            let mut nodes: Vec<_> = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.label() == Some(label)
                        && node.supports_action(Action::Click)
                        && !node.is_disabled()
                })
                .collect();
            nodes.sort_by(|(_, a), (_, b)| {
                let a = a.bounds().unwrap();
                let b = b.bounds().unwrap();
                a.x0.total_cmp(&b.x0).then(a.y0.total_cmp(&b.y0))
            });
            nodes.into_iter().map(|(id, _)| *id).collect()
        }
        fn click(
            ctx: &egui::Context,
            app: &mut Studio,
            output: &egui::FullOutput,
            label: &str,
        ) -> egui::FullOutput {
            let id = *buttons(output, label)
                .last()
                .unwrap_or_else(|| panic!("missing button {label}"));
            frame(ctx, app, vec![action(id, Action::Click)]);
            frame(ctx, app, vec![])
        }
        fn numeric(output: &egui::FullOutput, value: f64, index: usize) -> NodeId {
            let mut nodes: Vec<_> = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, node)| node.numeric_value() == Some(value) && !node.is_disabled())
                .collect();
            nodes.sort_by(|(_, a), (_, b)| {
                let a = a.bounds().unwrap();
                let b = b.bounds().unwrap();
                a.x0.total_cmp(&b.x0).then(a.y0.total_cmp(&b.y0))
            });
            nodes
                .get(index)
                .unwrap_or_else(|| panic!("missing numeric {value} at {index}"))
                .0
        }
        fn set_number(
            ctx: &egui::Context,
            app: &mut Studio,
            id: NodeId,
            value: f64,
        ) -> egui::FullOutput {
            frame(
                ctx,
                app,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::SetValue,
                    target_tree: TreeId::ROOT,
                    target_node: id,
                    data: Some(ActionData::NumericValue(value)),
                })],
            )
        }
        fn fixture(group: bool) -> (Studio, Vec<Id>) {
            let mut app = Studio::empty();
            for (index, (x, width)) in [
                (-1.25, 0.1),
                (-0.8, 0.3),
                (-0.25, 0.2),
                (0.35, 0.2),
                (0.95, 0.2),
            ]
            .into_iter()
            .enumerate()
            {
                app.editor
                    .execute(Command::AddRectangle {
                        name: format!("Panel {}", index + 1),
                        origin: [x, 0.3],
                        width,
                        height: 0.4,
                    })
                    .unwrap();
            }
            let curves: Vec<_> = app
                .editor
                .project
                .pieces
                .iter()
                .map(|piece| piece.boundary[0])
                .collect();
            let piece = app.editor.project.pieces[0].id;
            app.editor
                .execute(Command::SetGrain {
                    piece,
                    angle: 0.125,
                })
                .unwrap();
            if group {
                app.editor
                    .execute(Command::Transaction(vec![
                        Command::SetCurveControls {
                            curve: curves[0],
                            controls: Some([[-1.24, 0.26], [-1.18, 0.26]]),
                        },
                        Command::AddSewingGroup {
                            name: "Collective".into(),
                            a: vec![
                                CurveInterval {
                                    curve: curves[0],
                                    t0: 0.15,
                                    t1: 0.85,
                                },
                                CurveInterval {
                                    curve: curves[1],
                                    t0: 0.2,
                                    t1: 0.8,
                                },
                            ],
                            b: curves[2..]
                                .iter()
                                .map(|curve| CurveInterval {
                                    curve: *curve,
                                    t0: 0.9,
                                    t1: 0.1,
                                })
                                .collect(),
                            correspondence: vec![[0., 0.], [1., 1.]],
                        },
                    ]))
                    .unwrap();
            }
            app.editor = Editor::new(app.editor.project.clone()).unwrap();
            app.center = [0., 0.5];
            app.scale = 180.;
            (app, curves)
        }
        fn boundary_click(ctx: &egui::Context, app: &mut Studio, curve: Id) -> egui::FullOutput {
            let rect = ctx.read_response(app.pattern_canvas.unwrap()).unwrap().rect;
            let point = app
                .editor
                .project
                .curve(curve)
                .unwrap()
                .evaluate(&app.editor.project, 0.5)
                .unwrap();
            let pos = app.to_screen(point, rect);
            frame(
                ctx,
                app,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            frame(
                ctx,
                app,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            )
        }
        fn stop_jobs(app: &mut Studio) {
            app.cancel();
            app.job = None;
            app.needs_mesh = false;
        }
        fn assert_construction(actual: &Project, expected: &Project) {
            let mut actual = serde_json::to_value(actual).unwrap();
            let expected = serde_json::to_value(expected).unwrap();
            // History preserves the allocator high-water mark so deleted IDs are never reused.
            assert!(actual["next_id"].as_u64().unwrap() >= expected["next_id"].as_u64().unwrap());
            actual["revision"] = expected["revision"].clone();
            actual["next_id"] = expected["next_id"].clone();
            assert_eq!(actual, expected);
        }
        fn stage_inspector(ctx: &egui::Context, app: &mut Studio) -> (egui::FullOutput, egui::Id) {
            let output = frame(ctx, app, vec![]);
            let grain = numeric(&output, 0.125, 0);
            frame(ctx, app, vec![action(grain, Action::Focus)]);
            frame(ctx, app, vec![]);
            app.draft.as_mut().unwrap().resolution = 0.08;
            let output = frame(
                ctx,
                app,
                vec![
                    key(egui::Key::A, egui::Modifiers::COMMAND),
                    egui::Event::Text("0.250".into()),
                ],
            );
            let focused = ctx.memory(|memory| memory.focused()).unwrap();
            assert_eq!(
                ctx.data_mut(|data| data.get_temp::<String>(focused)),
                Some("0.250".into())
            );
            (output, focused)
        }

        #[test]
        fn canvas_enter_sides_delete_escape_history_and_load_are_atomic() {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let (mut app, curves) = fixture(false);
            let before = app.editor.project.clone();
            app.intervals = [[0.2, 0.8], [0.1, 0.9]];
            let output = frame(&ctx, &mut app, vec![]);
            click(&ctx, &mut app, &output, "M:N sewing");
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
            );
            assert_eq!(app.editor.project, before);
            assert!(!app.sewing_draft.selecting_b);
            boundary_click(&ctx, &mut app, curves[0]);
            boundary_click(&ctx, &mut app, curves[1]);
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Delete, egui::Modifiers::NONE)],
            );
            assert_eq!(app.sewing_draft.a.len(), 1);
            boundary_click(&ctx, &mut app, curves[1]);
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
            );
            assert!(app.sewing_draft.selecting_b);
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
            );
            assert_eq!(app.editor.project, before);
            boundary_click(&ctx, &mut app, curves[2]);
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Delete, egui::Modifiers::NONE)],
            );
            assert!(app.sewing_draft.b.is_empty() && app.sewing_draft.selecting_b);
            for curve in &curves[2..] {
                boundary_click(&ctx, &mut app, *curve);
            }
            assert_eq!(
                app.sewing_draft
                    .a
                    .iter()
                    .map(|span| span.curve)
                    .collect::<Vec<_>>(),
                curves[..2]
            );
            assert_eq!(app.editor.project, before);
            assert!(!app.editor.can_undo());
            app.draft.as_mut().unwrap().resolution = 0.08;
            let output = frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
            );
            stop_jobs(&mut app);
            let created = app.editor.project.clone();
            assert_eq!(created.revision, before.revision + 1);
            assert_eq!(created.resolution, 0.08);
            let group = &created.sewing_groups[0];
            assert_eq!(group.a.len(), 2);
            assert_eq!(group.b.len(), 3);
            assert!(group.a.iter().all(|span| [span.t0, span.t1] == [0.2, 0.8]));
            assert!(group.b.iter().all(|span| [span.t0, span.t1] == [0.9, 0.1]));
            assert!(app.sewing_draft.a.is_empty() && app.sewing_draft.b.is_empty());
            click(&ctx, &mut app, &output, "Undo");
            stop_jobs(&mut app);
            assert_construction(&app.editor.project, &before);
            assert!(!app.editor.can_undo() && app.editor.can_redo());
            boundary_click(&ctx, &mut app, curves[4]);
            let output = frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
            );
            assert!(app.sewing_draft.a.is_empty());
            assert!(app.editor.can_redo());
            assert_construction(&app.editor.project, &before);
            click(&ctx, &mut app, &output, "Redo");
            stop_jobs(&mut app);
            assert_construction(&app.editor.project, &created);
            boundary_click(&ctx, &mut app, curves[0]);
            let output = frame(&ctx, &mut app, vec![]);
            click(&ctx, &mut app, &output, "Select / edit");
            assert!(app.sewing_draft.a.is_empty());
            let output = frame(&ctx, &mut app, vec![]);
            let output = click(&ctx, &mut app, &output, "Commands ⌘K");
            click(&ctx, &mut app, &output, "M:N sewing");
            assert!(app.tool == Tool::SewingPaths);
            boundary_click(&ctx, &mut app, curves[0]);
            let dir = exit_test_directory("ordered-sewing-reopen");
            let path = dir.join("group.weftforge");
            app.editor.project.save(&path).unwrap();
            let saved: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            assert_eq!(saved["version"], weftforge_core::FORMAT_VERSION);
            app.open(dir.join("missing.weftforge"));
            assert_eq!(app.sewing_draft.a.len(), 1);
            app.open(path);
            assert!(app.sewing_draft.a.is_empty() && app.sewing_draft.b.is_empty());
            assert!(!app.sewing_draft.selecting_b);
            assert!(app.selected_curve.is_none() && app.seam_start.is_none());
            assert_construction(&app.editor.project, &created);
            frame(&ctx, &mut app, vec![]);
            stop_jobs(&mut app);
            fs::remove_dir_all(dir).unwrap();
        }

        #[test]
        fn group_controls_edit_all_fields_reverse_remove_and_preserve_one_step_history() {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let (mut app, curves) = fixture(true);
            let piece = app.editor.project.pieces[0].id;
            app.select(&ctx, Some(piece));
            let before = app.editor.project.clone();
            let (output, focused) = stage_inspector(&ctx, &mut app);
            let output = click(&ctx, &mut app, &output, "Edit group");
            assert_eq!(app.editor.project, before);
            let name = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| node.value() == Some("Collective"))
                .unwrap()
                .0;
            frame(&ctx, &mut app, vec![action(name, Action::Focus)]);
            let output = frame(
                &ctx,
                &mut app,
                vec![
                    key(egui::Key::A, egui::Modifiers::COMMAND),
                    egui::Event::Text("Edited group".into()),
                ],
            );
            let t0 = numeric(&output, 0.15, 0);
            let output = set_number(&ctx, &mut app, t0, 0.17);
            let output = click(&ctx, &mut app, &output, "Add easing knot");
            let b_knot = numeric(&output, 0.5, 1);
            let output = set_number(&ctx, &mut app, b_knot, 0.65);
            let down = buttons(&output, "Down")[0];
            let output = frame(&ctx, &mut app, vec![action(down, Action::Click)]);
            assert_eq!(
                app.sewing_edit.as_ref().unwrap().group.a[0].curve,
                before.sewing_groups[0].a[1].curve,
                "Down must reorder A immediately"
            );
            let direction = buttons(&output, "Direction")[1];
            let output = frame(&ctx, &mut app, vec![action(direction, Action::Click)]);
            let edited = app.sewing_edit.as_ref().unwrap().group.clone();
            assert_eq!(edited.name, "Edited group");
            assert_eq!(edited.a[0], before.sewing_groups[0].a[1]);
            assert_eq!([edited.a[1].t0, edited.a[1].t1], [0.85, 0.17]);
            assert_eq!(edited.correspondence, vec![[0., 0.], [0.5, 0.65], [1., 1.]]);
            assert_eq!(app.editor.project, before);
            click(&ctx, &mut app, &output, "Apply group");
            stop_jobs(&mut app);
            assert!(app.sewing_edit.is_none());
            let after = app.editor.project.clone();
            assert_eq!(after.revision, before.revision + 1);
            assert_eq!(after.sewing_groups, vec![edited.clone()]);
            assert_eq!(after.piece(piece).unwrap().grain_angle, 0.25);
            assert_eq!(after.resolution, 0.08);
            assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(focused)), None);
            let output = frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
            );
            assert_eq!(app.editor.project, after);
            let output = click(&ctx, &mut app, &output, "Undo");
            stop_jobs(&mut app);
            assert_construction(&app.editor.project, &before);
            assert!(!app.editor.can_undo() && app.editor.can_redo());
            let output = click(&ctx, &mut app, &output, "Redo");
            stop_jobs(&mut app);
            assert_construction(&app.editor.project, &after);
            let output = click(&ctx, &mut app, &output, "Reverse B");
            stop_jobs(&mut app);
            let mut reversed = edited.clone();
            reversed.b = curves[2..]
                .iter()
                .rev()
                .map(|curve| CurveInterval {
                    curve: *curve,
                    t0: 0.1,
                    t1: 0.9,
                })
                .collect();
            assert_eq!(app.editor.project.sewing_groups, vec![reversed]);
            let output = click(&ctx, &mut app, &output, "Reverse B");
            stop_jobs(&mut app);
            assert_eq!(app.editor.project.sewing_groups, vec![edited.clone()]);
            let output = click(&ctx, &mut app, &output, "Remove group");
            stop_jobs(&mut app);
            assert!(app.editor.project.sewing_groups.is_empty());
            click(&ctx, &mut app, &output, "Undo");
            stop_jobs(&mut app);
            assert_eq!(app.editor.project.sewing_groups, vec![edited]);
        }

        #[test]
        fn rejected_create_and_group_edit_cancel_keep_numeric_drafts_and_redo() {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let (mut app, curves) = fixture(true);
            let piece = app.editor.project.pieces[0].id;
            app.editor
                .execute(Command::SetGrain {
                    piece,
                    angle: 0.375,
                })
                .unwrap();
            assert!(app.editor.undo());
            app.select(&ctx, Some(piece));
            let before = app.editor.project.clone();
            let (output, focused) = stage_inspector(&ctx, &mut app);
            let output = click(&ctx, &mut app, &output, "M:N sewing");
            boundary_click(&ctx, &mut app, curves[0]);
            let output = click(&ctx, &mut app, &output, "Confirm A (Enter)");
            boundary_click(&ctx, &mut app, curves[0]);
            let output = click(&ctx, &mut app, &output, "Create sewing group (Enter)");
            assert_eq!(app.editor.project, before);
            assert!(app.editor.can_redo());
            assert!(app.message.contains("overlap"));
            assert_eq!(app.draft.as_ref().unwrap().commands(&before).len(), 2);
            assert!(app.sewing_draft.selecting_b);
            assert_eq!(app.sewing_draft.a.len(), 1);
            assert_eq!(app.sewing_draft.b.len(), 1);
            let output = click(&ctx, &mut app, &output, "Cancel sewing (Escape)");
            assert_eq!(app.editor.project, before);
            assert!(app.editor.can_redo());
            app.draft
                .as_mut()
                .unwrap()
                .material
                .as_mut()
                .unwrap()
                .areal_density = -1.;
            boundary_click(&ctx, &mut app, curves[0]);
            let output = click(&ctx, &mut app, &output, "Confirm A (Enter)");
            boundary_click(&ctx, &mut app, curves[1]);
            let output = click(&ctx, &mut app, &output, "Create sewing group (Enter)");
            assert_eq!(app.editor.project, before);
            assert!(app.editor.can_redo());
            assert!(app.message.contains("density"));
            assert_eq!(
                app.draft
                    .as_ref()
                    .unwrap()
                    .material
                    .as_ref()
                    .unwrap()
                    .areal_density,
                -1.
            );
            assert_eq!(app.sewing_draft.b[0].curve, curves[1]);
            let output = click(&ctx, &mut app, &output, "Cancel sewing (Escape)");
            app.draft
                .as_mut()
                .unwrap()
                .material
                .as_mut()
                .unwrap()
                .areal_density = before.materials[0].areal_density;
            let output = click(&ctx, &mut app, &output, "Edit group");
            let t0 = numeric(&output, 0.15, 0);
            let output = set_number(&ctx, &mut app, t0, 0.85);
            click(&ctx, &mut app, &output, "Apply group");
            assert!(app.sewing_edit.as_ref().unwrap().error.is_some());
            assert_eq!(app.editor.project, before);
            assert!(app.editor.can_redo());
            assert_eq!(app.draft.as_ref().unwrap().commands(&before).len(), 2);
            let output = frame(&ctx, &mut app, vec![]);
            assert_eq!(app.editor.project, before);
            let output = click(&ctx, &mut app, &output, "Cancel group edit");
            assert!(app.sewing_edit.is_none());
            assert_eq!(app.editor.project, before);
            assert_eq!(
                ctx.data_mut(|data| data.get_temp::<String>(focused))
                    .unwrap()
                    .parse::<f64>()
                    .unwrap(),
                0.25
            );
            assert!(app.editor.can_redo());
            let output = click(&ctx, &mut app, &output, "Edit group");
            frame(&ctx, &mut app, vec![]);
            assert_eq!(
                app.sewing_edit.as_ref().unwrap().group,
                before.sewing_groups[0]
            );
            click(&ctx, &mut app, &output, "Cancel group edit");
            stop_jobs(&mut app);
        }

        #[test]
        fn all_portion_guides_use_the_current_mesh_revision_and_authoritative_source() {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let (mut app, _) = fixture(true);
            let mesh = Arc::new(app.editor.project.mesh().unwrap());
            let portions = mesh.sewing_groups[0].portions.clone();
            assert!(portions.len() >= 4);
            app.mesh = Some(mesh.clone());
            let guides = |output: &egui::FullOutput| -> Vec<[Pos2; 2]> {
                output
                    .shapes
                    .iter()
                    .filter_map(|shape| match shape.shape {
                        egui::Shape::LineSegment { points, stroke }
                            if stroke == Stroke::new(1., Color32::from_rgb(125, 165, 235)) =>
                        {
                            Some(points)
                        }
                        _ => None,
                    })
                    .collect()
            };
            frame(&ctx, &mut app, vec![]); // Settle the measured footer before caching screen positions.
            let output = frame(&ctx, &mut app, vec![]);
            let accepted = guides(&output);
            assert_eq!(accepted.len(), portions.len() * 2);
            let canvas = ctx
                .data_mut(|data| {
                    data.get_temp::<Rect>(egui::Id::new("test-current-pattern-canvas"))
                })
                .unwrap();
            for (portion, pair) in portions.iter().zip(accepted.chunks_exact(2)) {
                for (index, t) in [0., 1.].into_iter().enumerate() {
                    let point = |span: &CurveInterval| {
                        app.to_screen(
                            app.editor
                                .project
                                .curve(span.curve)
                                .unwrap()
                                .evaluate(&app.editor.project, span.parameter(t))
                                .unwrap(),
                            canvas,
                        )
                    };
                    assert_eq!(pair[index], [point(&portion.a), point(&portion.b)]);
                }
            }
            app.drag_point = Some(app.editor.project.points[0].id);
            app.preview = Some([5., 5.]);
            assert_eq!(guides(&frame(&ctx, &mut app, vec![])), accepted);
            app.editor
                .execute(Command::SetGrain {
                    piece: app.editor.project.pieces[0].id,
                    angle: 0.4,
                })
                .unwrap();
            let output = frame(&ctx, &mut app, vec![]);
            assert!(guides(&output).is_empty());
            assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text().contains("Lengths and guides rebuilding"))));
            stop_jobs(&mut app);
        }
        fn pointer_click_label(
            ctx: &egui::Context,
            app: &mut Studio,
            output: &egui::FullOutput,
            label: &str,
        ) -> egui::FullOutput {
            let id = *buttons(output, label).last().unwrap();
            let bounds = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(node_id, _)| *node_id == id)
                .unwrap()
                .1
                .bounds()
                .unwrap();
            let pos = Pos2::new(
                ((bounds.x0 + bounds.x1) * 0.5) as f32,
                ((bounds.y0 + bounds.y1) * 0.5) as f32,
            );
            frame(
                ctx,
                app,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            frame(
                ctx,
                app,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            frame(ctx, app, vec![])
        }

        #[test]
        fn pointer_reverse_b_clears_all_reordered_row_buffers_and_preserves_pending_edits() {
            for reversals in [1, 2] {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let (mut app, curves) = fixture(true);
                let piece = app.editor.project.pieces[0].id;
                app.select(&ctx, Some(piece));
                let before = app.editor.project.clone();
                let (output, inspector_focus) = stage_inspector(&ctx, &mut app);
                let output = pointer_click_label(&ctx, &mut app, &output, "Edit group");
                let b_start = numeric(&output, 0.9, 0);
                frame(&ctx, &mut app, vec![action(b_start, Action::Focus)]);
                frame(&ctx, &mut app, vec![]);
                let output = frame(
                    &ctx,
                    &mut app,
                    vec![
                        key(egui::Key::A, egui::Modifiers::COMMAND),
                        egui::Event::Text("0.831234567890123".into()),
                    ],
                );
                let mut expected = before.sewing_groups[0].clone();
                expected.b[0].t0 = 0.831234567890123;
                if reversals == 1 {
                    expected.b = vec![
                        CurveInterval {
                            curve: curves[4],
                            t0: 0.1,
                            t1: 0.9,
                        },
                        CurveInterval {
                            curve: curves[3],
                            t0: 0.1,
                            t1: 0.9,
                        },
                        CurveInterval {
                            curve: curves[2],
                            t0: 0.1,
                            t1: 0.831234567890123,
                        },
                    ];
                }
                let mut output =
                    pointer_click_label(&ctx, &mut app, &output, "Reverse B order and directions");
                for _ in 1..reversals {
                    let _ = frame(
                        &ctx,
                        &mut app,
                        vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                    );
                    output = pointer_click_label(
                        &ctx,
                        &mut app,
                        &output,
                        "Reverse B order and directions",
                    );
                }
                let output = frame(
                    &ctx,
                    &mut app,
                    vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                );
                assert_eq!(app.sewing_edit.as_ref().unwrap().group, expected);
                assert_eq!(app.editor.project, before);
                assert_eq!(app.draft.as_ref().unwrap().commands(&before).len(), 2);
                pointer_click_label(&ctx, &mut app, &output, "Apply group");
                stop_jobs(&mut app);
                assert!(app.sewing_edit.is_none());
                assert_eq!(app.editor.project.revision, before.revision + 1);
                assert_eq!(app.editor.project.sewing_groups, vec![expected]);
                assert_eq!(app.editor.project.piece(piece).unwrap().grain_angle, 0.25);
                assert_eq!(app.editor.project.resolution, 0.08);
                assert_eq!(
                    ctx.data_mut(|data| data.get_temp::<String>(inspector_focus)),
                    None
                );
                frame(
                    &ctx,
                    &mut app,
                    vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
                );
                assert_eq!(app.editor.project.revision, before.revision + 1);
            }
        }

        fn choose_last_span_curve(
            ctx: &egui::Context,
            app: &mut Studio,
            output: &egui::FullOutput,
            from: Id,
            to: Id,
            panel: usize,
        ) -> egui::FullOutput {
            let value = format!("Curve #{from}");
            let id = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.value() == Some(value.as_str())
                        && node.supports_action(Action::Click)
                        && !node.is_disabled()
                })
                .max_by(|(_, a), (_, b)| a.bounds().unwrap().y0.total_cmp(&b.bounds().unwrap().y0))
                .unwrap()
                .0;
            frame(ctx, app, vec![action(id, Action::Click)]);
            let output = frame(ctx, app, vec![]);
            click(
                ctx,
                app,
                &output,
                &format!("Panel {panel} · boundary 1 · #{to}"),
            )
        }

        #[test]
        fn accesskit_only_two_to_three_creation_keeps_drafts_outside_document_until_one_commit() {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let (mut app, curves) = fixture(false);
            let piece = app.editor.project.pieces[0].id;
            app.select_curve(&ctx, Some((piece, curves[0])));
            app.intervals = [[0.2, 0.8], [0.1, 0.9]];
            let before = app.editor.project.clone();
            let (output, _) = stage_inspector(&ctx, &mut app);
            let output = click(&ctx, &mut app, &output, "M:N sewing");
            assert!(buttons(&output, "Add boundary to B").is_empty());
            let output = click(&ctx, &mut app, &output, "Add boundary to A");
            let output = click(&ctx, &mut app, &output, "Add boundary to A");
            let output = choose_last_span_curve(&ctx, &mut app, &output, curves[0], curves[1], 2);
            assert_eq!(
                app.sewing_draft
                    .a
                    .iter()
                    .map(|span| span.curve)
                    .collect::<Vec<_>>(),
                curves[..2]
            );
            assert_eq!(app.editor.project, before);
            assert!(!app.editor.can_undo());
            let mut output = click(&ctx, &mut app, &output, "Confirm A (Enter)");
            assert!(buttons(&output, "Add boundary to A").is_empty());
            assert!(app.sewing_draft.selecting_b);
            for (index, curve) in curves[2..].iter().enumerate() {
                output = click(&ctx, &mut app, &output, "Add boundary to B");
                output =
                    choose_last_span_curve(&ctx, &mut app, &output, curves[0], *curve, index + 3);
                assert_eq!(app.editor.project, before);
                assert!(!app.editor.can_undo());
            }
            assert_eq!(
                app.sewing_draft
                    .b
                    .iter()
                    .map(|span| span.curve)
                    .collect::<Vec<_>>(),
                curves[2..]
            );
            assert!(
                app.sewing_draft
                    .a
                    .iter()
                    .all(|span| [span.t0, span.t1] == [0.2, 0.8])
            );
            assert!(
                app.sewing_draft
                    .b
                    .iter()
                    .all(|span| [span.t0, span.t1] == [0.9, 0.1])
            );
            assert_eq!(app.draft.as_ref().unwrap().commands(&before).len(), 2);
            let output = click(&ctx, &mut app, &output, "Create sewing group (Enter)");
            stop_jobs(&mut app);
            assert_eq!(app.editor.project.revision, before.revision + 1);
            assert_eq!(app.editor.project.sewing_groups.len(), 1);
            assert_eq!(app.editor.project.sewing_groups[0].a.len(), 2);
            assert_eq!(app.editor.project.sewing_groups[0].b.len(), 3);
            assert_eq!(app.editor.project.piece(piece).unwrap().grain_angle, 0.25);
            assert_eq!(app.editor.project.resolution, 0.08);
            click(&ctx, &mut app, &output, "Undo");
            stop_jobs(&mut app);
            assert_construction(&app.editor.project, &before);
            assert!(!app.editor.can_undo() && app.editor.can_redo());
        }
    }
}

#[cfg(test)]
mod new_empty_tests {
    use super::*;
    use egui::accesskit::{Action, ActionData, ActionRequest, NodeId, TreeId};

    fn frame(ctx: &egui::Context, app: &mut Studio, events: Vec<egui::Event>) -> egui::FullOutput {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1640., 1800.))),
                events,
                ..Default::default()
            },
            |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
        )
    }

    fn action(id: NodeId, action: Action, data: Option<ActionData>) -> egui::Event {
        egui::Event::AccessKitActionRequest(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: id,
            data,
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
                node.label() == Some(label)
                    && node.supports_action(Action::Click)
                    && !node.is_disabled()
            })
            .unwrap_or_else(|| panic!("missing accessible {label:?}"));
        let bounds = node.bounds().unwrap();
        (
            *id,
            Pos2::new(
                ((bounds.x0 + bounds.x1) * 0.5) as f32,
                ((bounds.y0 + bounds.y1) * 0.5) as f32,
            ),
        )
    }

    fn route(ctx: &egui::Context, app: &mut Studio, search: bool) -> (NodeId, Pos2) {
        if search {
            app.search = true;
            app.query = "New Empty".into();
        }
        frame(ctx, app, vec![]);
        let mut output = frame(ctx, app, vec![]);
        if !search {
            frame(
                ctx,
                app,
                vec![action(button(&output, "File").0, Action::Click, None)],
            );
            output = frame(ctx, app, vec![]);
        }
        button(&output, "New Empty")
    }

    fn fixture() -> Studio {
        let mut app = Studio::empty();
        app.session_autosave = None;
        app.editor
            .execute(Command::AddRectangle {
                name: "Saved original panel".into(),
                origin: [0., 0.],
                width: 0.4,
                height: 0.6,
            })
            .unwrap();
        let piece = app.editor.project.pieces[0].id;
        app.editor
            .execute(Command::SetGrain { piece, angle: 0.37 })
            .unwrap();
        app.editor
            .execute(Command::SetResolution { metres: 0.08 })
            .unwrap();
        assert!(app.editor.undo());
        app.selected = Some(piece);
        let mesh = Arc::new(app.editor.project.mesh().unwrap());
        app.cloth = Some(Arc::new(
            weftforge_core::simulation::create_cloth(&app.editor.project, &mesh).unwrap(),
        ));
        app.mesh = Some(mesh);
        app.position_snap = PositionSnap::Mm1;
        app.center = [0.3, 0.4];
        app.scale = 233.;
        app.yaw = 0.7;
        app.pitch = -0.2;
        app.camera_scale = 350.;
        app.wireframe = true;
        app
    }

    fn clean_route(search: bool) {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = fixture();
        let directory = std::env::temp_dir().join(format!(
            "weftforge-new-empty-{}-{}-{search}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("saved.weftforge");
        app.editor.project.save(&path).unwrap();
        app.editor.project.save(&path).unwrap();
        let autosave = append_suffix(&path, ".autosave");
        fs::write(&autosave, b"existing autosave retained").unwrap();
        app.path = Some(path.clone());
        let original = app.editor.project.clone();
        let primary = fs::read(&path).unwrap();
        let recovery = fs::read(weftforge_core::recovery_path(&path)).unwrap();
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        app.job = Some(Job {
            id: 91,
            revision: 0,
            kind: JobKind::Mesh,
            batch: None,
            completed_steps: Arc::new(AtomicU64::new(0)),
            cancel: cancel.clone(),
            receiver,
        });
        app.last_job = Some(FinishedJob {
            id: 90,
            revision: original.revision,
            kind: JobKind::Mesh,
            progress: None,
            status: JobStatus::Accepted,
            outcome: Ok(()),
            cancel_requested: false,
            obsolete: false,
        });
        app.playing = true;
        app.pending_batch = Some(SimulationBatch::drape());
        let route = route(&ctx, &mut app, search);
        frame(&ctx, &mut app, vec![action(route.0, Action::Click, None)]);
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![]);
        }
        assert_eq!(app.editor.project, Project::default());
        assert!(!app.editor.can_undo() && !app.editor.can_redo());
        assert!(app.path.is_none() && app.selected.is_none() && app.selected_curve.is_none());
        assert!(
            app.mesh.is_none()
                && app.cloth.is_none()
                && app.job.is_none()
                && app.last_job.is_none()
        );
        assert!(!app.dirty && !app.needs_mesh && !app.playing && app.pending_batch.is_none());
        assert_eq!(app.last_stage, "Rest arrangement");
        assert!(cancel.load(Ordering::Relaxed));
        assert!(
            sender
                .send(Ok(JobOutput::Mesh(Arc::new(Mesh::default()))))
                .is_err()
        );
        app.poll();
        assert!(app.mesh.is_none());
        assert_eq!(app.position_snap, PositionSnap::Mm1);
        assert_eq!(app.center, [0.3, 0.4]);
        assert_eq!(
            (
                app.scale,
                app.yaw,
                app.pitch,
                app.camera_scale,
                app.wireframe
            ),
            (233., 0.7, -0.2, 350., true)
        );
        assert_eq!(fs::read(&path).unwrap(), primary);
        assert_eq!(
            fs::read(weftforge_core::recovery_path(&path)).unwrap(),
            recovery
        );
        assert_eq!(fs::read(&autosave).unwrap(), b"existing autosave retained");
        app.open(path);
        assert_eq!(app.editor.project, original);
        assert_eq!(fs::read(&autosave).unwrap(), b"existing autosave retained");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn new_empty_file_route_replaces_clean_document_and_retains_files_and_preferences() {
        clean_route(false);
    }

    #[test]
    fn new_empty_search_route_replaces_clean_document_and_retains_files_and_preferences() {
        clean_route(true);
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

    fn focus_property(ctx: &egui::Context, app: &mut Studio, text: &str) -> egui::Id {
        frame(ctx, app, vec![]);
        let output = frame(ctx, app, vec![]);
        let id = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.numeric_value() == Some(0.37))
            .unwrap()
            .0;
        frame(ctx, app, vec![action(id, Action::Focus, None)]);
        frame(ctx, app, vec![]);
        frame(
            ctx,
            app,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Text(text.into()),
                key(egui::Key::ArrowLeft, egui::Modifiers::NONE),
            ],
        );
        numeric_focus(ctx).unwrap()
    }

    #[test]
    fn new_empty_refusals_keep_numeric_text_caret_drafts_redo_and_accepted_display_across_frames() {
        for search in [false, true] {
            for pointer in [false, true] {
                for text in ["0.47", "1e-", "NaN"] {
                    let ctx = egui::Context::default();
                    ctx.enable_accesskit();
                    let mut app = fixture();
                    let focused = focus_property(&ctx, &mut app, text);
                    let raw = ctx.data_mut(|data| data.get_temp::<String>(focused));
                    let caret = egui::TextEdit::load_state(&ctx, focused)
                        .unwrap()
                        .cursor
                        .char_range();
                    let before = format!("{:?}", app.editor);
                    let pending = app.draft.as_ref().unwrap().commands(&app.editor.project);
                    if text == "1e-" {
                        assert!(pending.is_empty(), "{text}: {pending:?}");
                    }
                    let pending = format!("{pending:?}");
                    let mesh = app.mesh.clone().unwrap();
                    let cloth = app.cloth.clone().unwrap();
                    let selection = (app.selected, app.selected_curve);
                    let (id, pos) = route(&ctx, &mut app, search);
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
                            assert_eq!(
                                format!("{:?}", app.editor),
                                before,
                                "press/release {search} {text}"
                            );
                        }
                    } else {
                        frame(&ctx, &mut app, vec![action(id, Action::Click, None)]);
                    }
                    assert!(app.message.contains("Apply or cancel"), "{}", app.message);
                    for _ in 0..4 {
                        frame(&ctx, &mut app, vec![]);
                        assert_eq!(
                            format!("{:?}", app.editor),
                            before,
                            "later frame {search} {pointer} {text}"
                        );
                        assert_eq!(
                            format!(
                                "{:?}",
                                app.draft.as_ref().unwrap().commands(&app.editor.project)
                            ),
                            pending
                        );
                        assert_eq!((app.selected, app.selected_curve), selection);
                        assert_eq!(numeric_focus(&ctx), Some(focused));
                        assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(focused)), raw);
                        assert_eq!(
                            egui::TextEdit::load_state(&ctx, focused)
                                .unwrap()
                                .cursor
                                .char_range(),
                            caret
                        );
                        assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
                        assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &cloth));
                        assert!(!app.dirty && !app.needs_mesh && app.job.is_none());
                        assert!(app.editor.can_redo());
                    }
                }
            }
        }
    }

    #[test]
    fn new_empty_decision_observes_current_inspector_pass_without_committing_it() {
        for search in [false, true] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let route = route(&ctx, &mut app, search);
            let output = frame(&ctx, &mut app, vec![]);
            let field = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| node.numeric_value() == Some(0.37))
                .unwrap()
                .0;
            let before = format!("{:?}", app.editor);
            frame(
                &ctx,
                &mut app,
                vec![
                    action(
                        field,
                        Action::SetValue,
                        Some(ActionData::NumericValue(0.47)),
                    ),
                    action(route.0, Action::Click, None),
                ],
            );
            assert!(app.message.contains("Apply or cancel"));
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![]);
                assert_eq!(format!("{:?}", app.editor), before);
                assert_eq!(app.draft.as_ref().unwrap().grain_angle, Some(0.47));
                assert!(app.editor.can_redo());
            }
        }
    }

    #[test]
    fn new_empty_dirty_refusal_preserves_primary_recovery_autosave_and_redo() {
        for search in [false, true] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let directory = std::env::temp_dir().join(format!(
                "weftforge-new-empty-dirty-{}-{}-{search}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&directory).unwrap();
            let path = directory.join("saved.weftforge");
            app.editor.project.save(&path).unwrap();
            app.editor.project.save(&path).unwrap();
            let primary = fs::read(&path).unwrap();
            let recovery = fs::read(weftforge_core::recovery_path(&path)).unwrap();
            let autosave = append_suffix(&path, ".autosave");
            fs::write(&autosave, b"older autosave").unwrap();
            app.path = Some(path.clone());
            app.dirty = true;
            let before = format!("{:?}", app.editor);
            let route = route(&ctx, &mut app, search);
            frame(&ctx, &mut app, vec![action(route.0, Action::Click, None)]);
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![]);
            }
            assert_eq!(format!("{:?}", app.editor), before);
            assert!(app.message.contains("Save current edits"));
            assert_eq!(app.path.as_ref(), Some(&path));
            assert!(app.dirty && app.editor.can_redo());
            assert_eq!(fs::read(&path).unwrap(), primary);
            assert_eq!(
                fs::read(weftforge_core::recovery_path(&path)).unwrap(),
                recovery
            );
            assert_eq!(fs::read(&autosave).unwrap(), b"older autosave");
            fs::remove_dir_all(directory).unwrap();
        }
    }

    #[test]
    fn new_empty_refuses_unfinished_construction_gestures_without_resetting_them() {
        for search in [false, true] {
            for gesture in 0..7 {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let mut app = fixture();
                let curve = app.editor.project.pieces[0].boundary[0];
                match gesture {
                    0 => app.polygon = vec![[0.1, 0.2]],
                    1 => app.seam_start = Some(curve),
                    2 => app.sewing_draft.a = vec![CurveInterval::full(curve)],
                    3 => app.sewing_draft.b = vec![CurveInterval::reversed(curve)],
                    4 => app.drag_point = Some(app.editor.project.points[0].id),
                    5 => app.drag_start = Some([0.1, 0.2]),
                    _ => app.preview = Some([0.1, 0.2]),
                }
                let before = format!("{:?}", app.editor);
                let state = (
                    app.polygon.clone(),
                    app.seam_start,
                    app.sewing_draft.a.clone(),
                    app.sewing_draft.b.clone(),
                    app.drag_point,
                    app.drag_start,
                    app.preview,
                );
                let route = route(&ctx, &mut app, search);
                frame(&ctx, &mut app, vec![action(route.0, Action::Click, None)]);
                for _ in 0..3 {
                    frame(&ctx, &mut app, vec![]);
                }
                assert!(
                    app.message.contains("Finish or cancel"),
                    "gesture {gesture}: {}",
                    app.message
                );
                assert_eq!(format!("{:?}", app.editor), before);
                assert_eq!(
                    (
                        app.polygon.clone(),
                        app.seam_start,
                        app.sewing_draft.a.clone(),
                        app.sewing_draft.b.clone(),
                        app.drag_point,
                        app.drag_start,
                        app.preview
                    ),
                    state
                );
                assert!(app.editor.can_redo());
            }
        }
    }

    #[test]
    fn new_empty_authoring_modal_keeps_background_accesskit_action_shielded() {
        for search in [false, true] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let route = route(&ctx, &mut app, search);
            let before = format!("{:?}", app.editor);
            app.rectangle = Some(RectangleDraft {
                origin: [0., 0.],
                dimensions_mm: ["1e-".into(), "100".into()],
                error: None,
            });
            frame(&ctx, &mut app, vec![action(route.0, Action::Click, None)]);
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![]);
            }
            assert_eq!(format!("{:?}", app.editor), before);
            assert_eq!(
                app.rectangle.as_ref().unwrap().dimensions_mm,
                ["1e-", "100"]
            );
            assert!(app.editor.can_redo());
        }
    }

    #[test]
    fn new_empty_waits_for_existing_autosave_worker_without_dropping_it() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = fixture();
        let (sender, receiver) = mpsc::channel();
        app.autosave_job = Some(AutosaveJob { receiver });
        let before = format!("{:?}", app.editor);
        let target = route(&ctx, &mut app, false);
        frame(&ctx, &mut app, vec![action(target.0, Action::Click, None)]);
        assert!(app.message.contains("Wait for the current autosave"));
        assert_eq!(format!("{:?}", app.editor), before);
        assert!(app.autosave_job.is_some());
        sender.send(Ok(())).unwrap();
        frame(&ctx, &mut app, vec![]);
        let target = route(&ctx, &mut app, false);
        frame(&ctx, &mut app, vec![action(target.0, Action::Click, None)]);
        assert_eq!(app.editor.project, Project::default());
    }

    #[test]
    fn new_empty_refusal_keeps_transform_gesture_across_menu_search_and_pointer_release() {
        for search in [false, true] {
            for pointer in [false, true] {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let mut app = fixture();
                app.tool = Tool::Transform;
                let mut edit = app.transform_edit(&ctx, app.selected.unwrap()).unwrap();
                edit.translation = [0.1, 0.2];
                app.transform_gesture = Some(TransformGesture {
                    original: app.editor.project.clone(),
                    pending: vec![],
                    edit,
                    start: [0., 0.],
                    snap: PositionSnap::Mm1,
                    rotating: false,
                });
                let before = format!("{:?}", app.editor);
                let (id, pos) = route(&ctx, &mut app, search);
                assert!(
                    app.transform_gesture.is_some(),
                    "opening route search={search}"
                );
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
                        assert_eq!(format!("{:?}", app.editor), before);
                    }
                } else {
                    frame(&ctx, &mut app, vec![action(id, Action::Click, None)]);
                }
                for _ in 0..3 {
                    frame(&ctx, &mut app, vec![]);
                }
                assert!(app.message.contains("Finish or cancel"));
                assert_eq!(format!("{:?}", app.editor), before);
                let gesture = app.transform_gesture.as_ref().unwrap();
                assert_eq!(gesture.edit.translation, [0.1, 0.2]);
                assert_eq!(gesture.original, app.editor.project);
                assert!(app.editor.can_redo());
            }
        }
    }
}

#[cfg(test)]
mod pattern_import_tests {
    use super::*;
    use egui::accesskit::{Action, ActionData, ActionRequest, NodeId, TreeId};
    const ORIGINAL: &[u8] = include_bytes!(
        "../../../crates/weftforge-core/tests/fixtures/original-editable-patterns.svg"
    );

    const DXF_ORIGINAL: &[u8] = include_bytes!(
        "../../../crates/weftforge-core/tests/fixtures/original-editable-patterns.dxf"
    );

    fn frame(ctx: &egui::Context, app: &mut Studio, events: Vec<egui::Event>) -> egui::FullOutput {
        app.needs_mesh = false;
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600., 1800.))),
                events,
                ..Default::default()
            },
            |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
        )
    }
    fn action(id: NodeId, action: Action, data: Option<ActionData>) -> egui::Event {
        egui::Event::AccessKitActionRequest(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: id,
            data,
        })
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
                node.label() == Some(label)
                    && node.supports_action(Action::Click)
                    && !node.is_disabled()
            })
            .unwrap_or_else(|| {
                panic!(
                    "missing {label}; labels: {:?}",
                    output
                        .platform_output
                        .accesskit_update
                        .as_ref()
                        .unwrap()
                        .nodes
                        .iter()
                        .filter_map(|(_, node)| node.label())
                        .collect::<Vec<_>>()
                )
            })
            .0
    }

    fn click(ctx: &egui::Context, app: &mut Studio, label: &str) -> egui::FullOutput {
        frame(ctx, app, vec![]);
        let mut output = frame(ctx, app, vec![]);
        if matches!(label, "Import SVG…" | "Import DXF…") {
            let file = button(&output, "File");
            frame(ctx, app, vec![action(file, Action::Click, None)]);
            output = frame(ctx, app, vec![]);
        }
        let id = button(&output, label);
        frame(ctx, app, vec![action(id, Action::Click, None)]);
        frame(ctx, app, vec![])
    }
    fn fixture() -> Studio {
        let mut app = Studio::empty();
        app.session_autosave = None;
        app.editor
            .execute(Command::AddRectangle {
                name: "Existing panel".into(),
                origin: [0., 0.],
                width: 0.4,
                height: 0.6,
            })
            .unwrap();
        let piece = app.editor.project.pieces[0].id;
        app.editor
            .execute(Command::SetGrain { piece, angle: 0.25 })
            .unwrap();
        assert!(app.editor.undo());
        app.selected = Some(piece);
        let mut draft = InspectorDraft::new(&app.editor.project, app.selected, None);
        draft.grain_angle = Some(0.375);
        app.draft = Some(draft);
        app
    }
    fn source(format: PatternFormat) -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "weftforge-native-svg-{}-{}-{}.svg",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(
            &path,
            match format {
                PatternFormat::Svg => ORIGINAL,
                PatternFormat::Dxf => DXF_ORIGINAL,
            },
        )
        .unwrap();
        path
    }

    #[test]
    fn original_baseline_decimal_width_return_commits_once_and_history_is_exact() {
        fn key(key: egui::Key, modifiers: egui::Modifiers, pressed: bool) -> egui::Event {
            egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            }
        }
        fn same_geometry(actual: &Project, expected: &Project) {
            assert_eq!(actual.points, expected.points);
            assert_eq!(actual.curves, expected.curves);
            assert_eq!(actual.pieces, expected.pieces);
        }
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = Studio::empty();
        app.session_autosave = None;
        app.editor
            .execute(Command::AddRectangle {
                name: "Existing baseline".into(),
                origin: [0.5, 0.1],
                width: 0.1,
                height: 0.12,
            })
            .unwrap();
        let baseline = app.editor.project.clone();
        let piece = baseline.pieces[0].id;
        // Opening the original baseline starts at revision 1 with empty history.
        app.editor = Editor::new(baseline.clone()).unwrap();
        app.selected = Some(piece);
        let mut expected = Editor::new(baseline.clone()).unwrap();
        let mut authored = Vec::new();
        for (text, width) in [("0.11", 0.11), ("0.12", 0.12)] {
            let before = app.editor.project.clone();
            let output = frame(&ctx, &mut app, vec![]);
            let bounds = before.piece_bounds(piece).unwrap();
            let width_before = bounds[1][0] - bounds[0][0];
            let field = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| node.numeric_value() == Some(width_before))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            let pos = Pos2::new(
                ((field.x0 + field.x1) * 0.5) as f32,
                ((field.y0 + field.y1) * 0.5) as f32,
            );
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
            frame(
                &ctx,
                &mut app,
                vec![
                    key(egui::Key::A, egui::Modifiers::COMMAND, true),
                    egui::Event::Text(text.into()),
                ],
            );
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![]);
            }
            assert_eq!(
                app.editor.project, before,
                "typing {text} must stay pending"
            );
            assert_eq!(app.draft.as_ref().unwrap().dimensions.unwrap()[0], width);
            expected
                .execute(Command::ResizePiece {
                    piece,
                    width,
                    height: 0.12,
                })
                .unwrap();
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
            );
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Enter, egui::Modifiers::NONE, false)],
            );
            for idle in 0..8 {
                frame(&ctx, &mut app, vec![]);
                assert_eq!(
                    app.editor.project.revision,
                    before.revision + 1,
                    "{text} Return idle frame {idle}: one gesture must be one revision"
                );
                assert_eq!(
                    app.editor.project, expected.project,
                    "{text} Return idle frame {idle}"
                );
            }
            authored.push(app.editor.project.clone());
        }
        for (redo, target, more_history) in [
            (false, &authored[0], true),
            (false, &baseline, false),
            (true, &authored[0], true),
            (true, &authored[1], false),
        ] {
            app.history(&ctx, redo);
            let revision = app.editor.project.revision;
            same_geometry(&app.editor.project, target);
            for idle in 0..8 {
                frame(&ctx, &mut app, vec![]);
                assert_eq!(
                    app.editor.project.revision, revision,
                    "history replay at idle frame {idle}"
                );
                same_geometry(&app.editor.project, target);
                assert_eq!(
                    if redo {
                        app.editor.can_redo()
                    } else {
                        app.editor.can_undo()
                    },
                    more_history
                );
            }
        }
    }

    #[test]
    fn import_modal_controls_are_reachable_from_the_accessibility_root() {
        for format in [PatternFormat::Svg, PatternFormat::Dxf] {
            use std::collections::{HashMap, HashSet};
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            app.pattern_import = Some(PatternImportDraft {
                drawing: match format {
                    PatternFormat::Svg => {
                        ImportedDrawing::Svg(SvgPatternImport::from_bytes(ORIGINAL).unwrap())
                    }
                    PatternFormat::Dxf => {
                        ImportedDrawing::Dxf(DxfPatternImport::from_bytes(DXF_ORIGINAL).unwrap())
                    }
                },
                focused: None,
                error: None,
            });
            for _ in 0..3 {
                frame(&ctx, &mut app, vec![]);
            }
            let output = frame(&ctx, &mut app, vec![]);
            let tree = output.platform_output.accesskit_update.unwrap();
            let nodes: HashMap<_, _> = tree.nodes.iter().map(|(id, node)| (*id, node)).collect();
            let mut pending = vec![tree.tree.unwrap().root];
            let mut visible = HashSet::new();
            while let Some(id) = pending.pop() {
                let node = nodes[&id];
                if !node.is_hidden() && visible.insert(id) {
                    pending.extend(node.children());
                }
            }
            for label in [
                match format {
                    PatternFormat::Svg => "Import SVG patterns",
                    PatternFormat::Dxf => "Import DXF patterns",
                },
                match format {
                    PatternFormat::Svg => "Physical scale: 2 mm per SVG unit.",
                    PatternFormat::Dxf => "Physical scale: 1 mm per DXF unit ($INSUNITS=4).",
                },
                "Import",
                "Cancel",
            ] {
                let (id, node) = tree
                    .nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some(label) || node.value() == Some(label))
                    .unwrap_or_else(|| panic!("missing modal node {label}"));
                assert!(visible.contains(id), "modal node is unreachable: {label}");
                assert!(!node.is_disabled(), "modal node is disabled: {label}");
            }
        }
    }

    #[test]
    fn pointer_cancel_retains_focused_numeric_draft_revision_and_history() {
        for format in [PatternFormat::Svg, PatternFormat::Dxf] {
            fn pointer_click(
                ctx: &egui::Context,
                app: &mut Studio,
                output: &egui::FullOutput,
                label: &str,
            ) {
                let id = button(output, label);
                let bounds = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(node, _)| *node == id)
                    .unwrap()
                    .1
                    .bounds()
                    .unwrap();
                let pos = Pos2::new(
                    ((bounds.x0 + bounds.x1) * 0.5) as f32,
                    ((bounds.y0 + bounds.y1) * 0.5) as f32,
                );
                for pressed in [true, false] {
                    frame(
                        ctx,
                        app,
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
            }
            fn history(editor: &Editor) -> [Vec<Project>; 2] {
                std::array::from_fn(|direction| {
                    let mut editor = editor.clone();
                    let mut states = Vec::new();
                    while if direction == 0 {
                        editor.undo()
                    } else {
                        editor.redo()
                    } {
                        states.push(editor.project.clone());
                    }
                    states
                })
            }
            for review in [false, true] {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let mut app = fixture();
                let before = app.editor.clone();
                let old_history = history(&before);
                let path = source(format);
                frame(&ctx, &mut app, vec![]);
                let output = frame(&ctx, &mut app, vec![]);
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
                frame(&ctx, &mut app, vec![action(numeric, Action::Focus, None)]);
                frame(&ctx, &mut app, vec![]);
                let output = frame(
                    &ctx,
                    &mut app,
                    vec![
                        egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::COMMAND,
                        },
                        egui::Event::Text("0.125".into()),
                    ],
                );
                assert_eq!(app.editor.project, before.project);
                assert_eq!(app.draft.as_ref().unwrap().dimensions.unwrap()[0], 0.125);
                ctx.data_mut(|data| {
                    data.insert_temp(
                        egui::Id::new(format!(
                            "test-{}-import-source",
                            format.name().to_ascii_lowercase()
                        )),
                        review.then_some(path.clone()),
                    )
                });
                pointer_click(&ctx, &mut app, &output, "File");
                let output = frame(&ctx, &mut app, vec![]);
                pointer_click(
                    &ctx,
                    &mut app,
                    &output,
                    &format!("Import {}…", format.name()),
                );
                assert_eq!(app.pattern_import.is_some(), review);
                if review {
                    for _ in 0..3 {
                        frame(&ctx, &mut app, vec![]);
                    }
                    let output = frame(&ctx, &mut app, vec![]);
                    assert_eq!(app.editor.project, before.project);
                    pointer_click(&ctx, &mut app, &output, "Cancel");
                }
                assert!(app.pattern_import.is_none());
                assert_eq!(app.editor.project, before.project, "Cancel release");
                for offset in 0..3 {
                    frame(&ctx, &mut app, vec![]);
                    assert_eq!(
                        app.editor.project, before.project,
                        "post-Cancel frame {offset}"
                    );
                    assert_eq!(history(&app.editor), old_history);
                    assert_eq!(app.draft.as_ref().unwrap().dimensions.unwrap()[0], 0.125);
                }
                let mut expected = before;
                expected
                    .execute(Command::Transaction(
                        app.draft.as_ref().unwrap().commands(&expected.project),
                    ))
                    .unwrap();
                let output = frame(&ctx, &mut app, vec![]);
                let numeric = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(_, node)| node.numeric_value() == Some(0.125))
                    .unwrap()
                    .0;
                frame(&ctx, &mut app, vec![action(numeric, Action::Focus, None)]);
                frame(&ctx, &mut app, vec![]);
                frame(
                    &ctx,
                    &mut app,
                    vec![egui::Event::Key {
                        key: egui::Key::Tab,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }],
                );
                assert_eq!(app.editor.project, expected.project);
                assert_eq!(history(&app.editor), history(&expected));
                fs::remove_file(path).unwrap();
            }
        }
    }

    #[test]
    fn chooser_modal_import_commits_pending_edits_and_all_pieces_as_one_undo() {
        for format in [PatternFormat::Svg, PatternFormat::Dxf] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let before = app.editor.project.clone();
            let path = source(format);
            ctx.data_mut(|data| {
                data.insert_temp(
                    egui::Id::new(format!(
                        "test-{}-import-source",
                        format.name().to_ascii_lowercase()
                    )),
                    Some(path.clone()),
                )
            });
            let prior = frame(&ctx, &mut app, vec![]);
            let background = button(&prior, "Add ellipse");
            let numeric = prior
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| node.numeric_value() == Some(0.4))
                .unwrap()
                .0;
            click(&ctx, &mut app, &format!("Import {}…", format.name()));
            assert!(app.pattern_import.is_some());
            assert_eq!(app.editor.project, before);
            let summary = frame(
                &ctx,
                &mut app,
                vec![
                    action(background, Action::Click, None),
                    action(
                        numeric,
                        Action::SetValue,
                        Some(ActionData::NumericValue(0.9)),
                    ),
                ],
            );
            assert!(
                summary
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.value().is_some_and(|label| label.contains(
                        match format {
                            PatternFormat::Svg => "Physical scale: 2 mm per SVG unit",
                            PatternFormat::Dxf => "Physical scale: 1 mm per DXF unit",
                        }
                    )))
            );
            assert_eq!(app.editor.project, before);
            assert_eq!(app.draft.as_ref().unwrap().grain_angle, Some(0.375));
            click(&ctx, &mut app, "Import");
            assert!(app.pattern_import.is_none());
            assert_eq!(app.editor.project.revision, before.revision + 1);
            assert_eq!(app.editor.project.pieces.len(), before.pieces.len() + 2);
            assert_eq!(
                app.editor.project.pieces[1].name,
                match format {
                    PatternFormat::Svg => "Original & curved front",
                    PatternFormat::Dxf => "WF_PIECE_17",
                }
            );
            assert_eq!(
                app.editor.project.points[4].position,
                match format {
                    PatternFormat::Svg => [-0.16, 0.32],
                    PatternFormat::Dxf => [-0.25, 0.15],
                }
            );
            assert_eq!(app.editor.project.pieces[0].grain_angle, 0.375);
            assert!(app.dirty && !app.playing && app.pending_batch.is_none());
            assert!(
                app.draft.is_none()
                    || app
                        .draft
                        .as_ref()
                        .unwrap()
                        .commands(&app.editor.project)
                        .is_empty()
            );
            assert_eq!(
                fs::read(&path).unwrap(),
                match format {
                    PatternFormat::Svg => ORIGINAL,
                    PatternFormat::Dxf => DXF_ORIGINAL,
                }
            );
            app.history(&ctx, false);
            assert_eq!(app.editor.project.pieces, before.pieces);
            assert_eq!(app.editor.project.points, before.points);
            assert_eq!(app.editor.project.curves, before.curves);
            assert_eq!(app.editor.project.resolution, before.resolution);
            assert_eq!(
                app.editor.project.pieces[0].grain_angle,
                before.pieces[0].grain_angle
            );
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn cancelled_invalid_and_failed_import_keep_pending_drafts_ids_and_redo() {
        for format in [PatternFormat::Svg, PatternFormat::Dxf] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let before = app.editor.project.clone();
            let path = source(format);
            assert!(!app.prepare_import(&ctx, None, None, format));
            fs::write(&path, b"<svg>").unwrap();
            assert!(!app.prepare_import(&ctx, Some(&path), None, format));
            assert_eq!(app.editor.project, before);
            assert!(app.editor.can_redo());
            assert_eq!(app.draft.as_ref().unwrap().grain_angle, Some(0.375));
            fs::write(
                &path,
                match format {
                    PatternFormat::Svg => ORIGINAL,
                    PatternFormat::Dxf => DXF_ORIGINAL,
                },
            )
            .unwrap();
            assert!(app.prepare_import(&ctx, Some(&path), None, format));
            click(&ctx, &mut app, "Cancel");
            assert!(app.pattern_import.is_none());
            assert_eq!(app.editor.project, before);
            assert!(app.editor.can_redo());
            assert_eq!(app.draft.as_ref().unwrap().grain_angle, Some(0.375));
            app.draft
                .as_mut()
                .unwrap()
                .material
                .as_mut()
                .unwrap()
                .warp_stiffness = -1.0;
            assert!(app.prepare_import(&ctx, Some(&path), None, format));
            click(&ctx, &mut app, "Import");
            assert!(app.pattern_import.as_ref().unwrap().error.is_some());
            assert_eq!(app.editor.project, before);
            assert!(app.editor.can_redo());
            assert_eq!(
                app.draft
                    .as_ref()
                    .unwrap()
                    .material
                    .as_ref()
                    .unwrap()
                    .warp_stiffness,
                -1.0
            );
            assert_eq!(
                fs::read(&path).unwrap(),
                match format {
                    PatternFormat::Svg => ORIGINAL,
                    PatternFormat::Dxf => DXF_ORIGINAL,
                }
            );
            click(&ctx, &mut app, "Cancel");
            assert_eq!(app.editor.project, before);
            fs::remove_file(path).unwrap();
        }
    }
}

#[cfg(test)]
mod pattern_framing_tests {
    use super::*;
    use egui::accesskit::{Action, ActionRequest, TreeId};

    fn frame(
        ctx: &egui::Context,
        app: &mut Studio,
        size: [f32; 2],
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::from(size))),
                events,
                ..Default::default()
            },
            |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
        )
    }

    fn assert_outlines_fit(app: &Studio, rect: Rect) {
        let inset = rect.shrink(28.0).expand(0.001);
        assert!(app.scale.is_finite() && app.scale > 0.0);
        assert!(app.center.iter().all(|value| value.is_finite()));
        for piece in &app.editor.project.pieces {
            let [min, max] = app.editor.project.piece_bounds(piece.id).unwrap();
            for point in [min, max, [min[0], max[1]], [max[0], min[1]]] {
                assert!(inset.contains(app.to_screen(point, rect)), "{point:?}");
            }
            for curve in &piece.boundary {
                for step in 0..=64 {
                    let point = app
                        .editor
                        .project
                        .curve(*curve)
                        .unwrap()
                        .evaluate(&app.editor.project, step as f64 / 64.0)
                        .unwrap();
                    assert!(inset.contains(app.to_screen(point, rect)), "{point:?}");
                }
            }
        }
    }

    #[test]
    fn frame_all_fits_authored_outlines_without_changing_document_or_pending_edits() {
        for size in [[1000.0, 650.0], [1640.0, 1000.0]] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            studio_style(&ctx);
            let mut app = Studio::empty();
            app.session_autosave = None;
            for (origin, width, height) in [([-12.0, -9.0], 4.0, 3.0), ([-4.0, -3.0], 1.0, 1.5)] {
                app.editor
                    .execute(Command::AddRectangle {
                        name: "Original panel".into(),
                        origin,
                        width,
                        height,
                    })
                    .unwrap();
            }
            let piece = app.editor.project.pieces[0].id;
            let curve = app.editor.project.pieces[0].boundary[0];
            app.editor
                .execute(Command::SetCurveControls {
                    curve,
                    controls: Some([[-12.0, -13.0], [-8.0, -13.0]]),
                })
                .unwrap();
            // This cubic's y minimum is -12 at t=1/2, beyond every endpoint.
            assert_eq!(app.editor.project.piece_bounds(piece).unwrap()[0][1], -12.0);
            app.editor
                .execute(Command::SetGrain { piece, angle: 0.25 })
                .unwrap();
            assert!(app.editor.undo());
            app.selected = Some(piece);
            app.selected_curve = Some(curve);
            let mut draft =
                InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve);
            draft.grain_angle = Some(0.375);
            draft.curve.as_mut().unwrap().controls = Some([[-12.0, -800.0], [-8.0, -800.0]]);
            let pending = draft.commands(&app.editor.project);
            app.draft = Some(draft);
            let editor = format!("{:?}", app.editor);
            let selection = (app.selected, app.selected_curve);
            frame(&ctx, &mut app, size, vec![]);
            let output = frame(&ctx, &mut app, size, vec![]);
            let nodes = output.platform_output.accesskit_update.unwrap().nodes;
            let button = nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Frame all"))
                .unwrap();
            assert!(!button.1.is_disabled());
            let field = nodes
                .iter()
                .find(|(_, node)| node.numeric_value() == Some(-800.0))
                .unwrap();
            frame(
                &ctx,
                &mut app,
                size,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Focus,
                    target_tree: TreeId::ROOT,
                    target_node: field.0,
                    data: None,
                })],
            );
            frame(&ctx, &mut app, size, vec![]);
            app.center = [600.0, -700.0];
            app.scale = 40.0;
            let bounds = button.1.bounds().unwrap();
            let pos = Pos2::new(
                ((bounds.x0 + bounds.x1) * 0.5) as f32,
                ((bounds.y0 + bounds.y1) * 0.5) as f32,
            );
            for pressed in [true, false] {
                frame(
                    &ctx,
                    &mut app,
                    size,
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
            frame(&ctx, &mut app, size, vec![]);
            frame(&ctx, &mut app, size, vec![]);
            let rect = ctx.read_response(app.pattern_canvas.unwrap()).unwrap().rect;
            assert_outlines_fit(&app, rect);
            assert_eq!(app.center, [-7.5, -6.75]);
            assert_eq!(format!("{:?}", app.editor), editor);
            assert_eq!((app.selected, app.selected_curve), selection);
            assert_eq!(
                app.draft.as_ref().unwrap().commands(&app.editor.project),
                pending
            );
            assert!(!app.dirty && !app.needs_mesh && app.job.is_none());
            assert!(app.mesh.is_none() && app.cloth.is_none() && app.pending_batch.is_none());

            app.editor = Editor::default();
            for origin in [[-1000.0, -1000.0], [999.8, 999.8]] {
                app.editor
                    .execute(Command::AddRectangle {
                        name: "Full coordinate range".into(),
                        origin,
                        width: 0.2,
                        height: 0.2,
                    })
                    .unwrap();
            }
            app.frame_patterns(rect);
            assert_outlines_fit(&app, rect);
            assert!(app.scale < 40.0);
            let fitted = app.scale;
            let pointer = egui::Event::PointerMoved(rect.center());
            // A hovered canvas with no scroll must retain the large-range fit.
            app.draft = None;
            app.selected = None;
            app.selected_curve = None;
            frame(&ctx, &mut app, size, vec![pointer]);
            assert_eq!(app.scale, fitted);

            for point in &mut app.editor.project.points {
                point.position = [-3.0, -5.0];
            }
            app.frame_patterns(rect);
            assert_eq!(app.center, [-3.0, -5.0]);
            assert_eq!(app.scale, 3000.0);
            let view = (app.center, app.scale);
            app.frame_patterns(Rect::from_min_size(Pos2::ZERO, Vec2::splat(56.0)));
            assert_eq!((app.center, app.scale), view);
            app.editor = Editor::default();
            app.frame_patterns(rect);
            assert_eq!((app.center, app.scale), view);
            let output = frame(&ctx, &mut app, size, vec![]);
            assert!(
                output
                    .platform_output
                    .accesskit_update
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Frame all") && node.is_disabled())
            );
        }
    }
}

#[cfg(test)]
mod pattern_transform_tests {
    use super::*;
    use egui::accesskit::{Action, ActionData, ActionRequest, NodeId, TreeId};

    fn frame(ctx: &egui::Context, app: &mut Studio, events: Vec<egui::Event>) -> egui::FullOutput {
        app.needs_mesh = false;
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(3600., 1800.))),
                events,
                ..Default::default()
            },
            |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
        )
    }
    fn action(id: NodeId, action: Action, data: Option<ActionData>) -> egui::Event {
        egui::Event::AccessKitActionRequest(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: id,
            data,
        })
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
                node.label() == Some(label)
                    && node.supports_action(Action::Click)
                    && !node.is_disabled()
            })
            .unwrap_or_else(|| panic!("missing accessible {label:?}"))
            .0
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
    fn pointer(pos: Pos2, pressed: bool, alt: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers {
                alt,
                ..Default::default()
            },
        }
    }
    fn pointer_frame(
        ctx: &egui::Context,
        app: &mut Studio,
        pos: Pos2,
        pressed: Option<bool>,
        alt: bool,
    ) -> egui::FullOutput {
        app.needs_mesh = false;
        let mut events = vec![egui::Event::PointerMoved(pos)];
        if let Some(pressed) = pressed {
            events.push(pointer(pos, pressed, alt));
        }
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(3600., 1800.))),
                modifiers: egui::Modifiers {
                    alt,
                    ..Default::default()
                },
                events,
                ..Default::default()
            },
            |ui| eframe::App::ui(app, ui, &mut eframe::Frame::_new_kittest()),
        )
    }
    fn field(output: &egui::FullOutput, label: &str) -> NodeId {
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        let labels: Vec<_> = nodes
            .iter()
            .filter(|(_, node)| node.label() == Some(label) || node.value() == Some(label))
            .map(|(id, _)| *id)
            .collect();
        nodes
            .iter()
            .find(|(_, node)| {
                node.numeric_value().is_some()
                    && node.labelled_by().iter().any(|id| labels.contains(id))
            })
            .unwrap_or_else(|| panic!("missing numeric field {label:?}"))
            .0
    }

    fn set_field(ctx: &egui::Context, app: &mut Studio, label: &str, value: f64) {
        let output = frame(ctx, app, vec![]);
        frame(
            ctx,
            app,
            vec![action(
                field(&output, label),
                Action::SetValue,
                Some(ActionData::NumericValue(value)),
            )],
        );
    }
    fn fixture() -> Studio {
        let mut app = Studio::empty();
        app.session_autosave = None;
        app.editor
            .execute(Command::AddPolygon {
                name: "Asymmetric cubic panel".into(),
                vertices: vec![[0., 0.], [0.75, 0.75], [-0.25, 1.25], [-0.5, 0.25]],
            })
            .unwrap();
        let piece = app.editor.project.pieces[0].id;
        let curve = app.editor.project.pieces[0].boundary[0];
        for command in [
            Command::SetCurveControls {
                curve,
                controls: Some([[0.25, 0.], [0.5, 0.25]]),
            },
            Command::SetPlacement {
                piece,
                placement: Placement {
                    translation: [0.2, 0.3, -0.1],
                    rotation: [0.31, -0.47, 0.21],
                },
            },
            Command::SetGrain { piece, angle: 0.37 },
            Command::AddRectangle {
                name: "Separate panel".into(),
                origin: [0.85, 0.25],
                width: 0.2,
                height: 0.4,
            },
            Command::SetResolution { metres: 0.2 },
            Command::SetResolution { metres: 0.22 },
        ] {
            app.editor.execute(command).unwrap();
        }
        assert!(app.editor.undo());
        app.selected = Some(piece);
        app.selected_curve = Some(curve);
        app.tool = Tool::Transform;
        app.center = [0.125, 0.625];
        app.scale = 1000.;
        app
    }
    fn canvas_rect(ctx: &egui::Context, app: &Studio) -> Rect {
        ctx.read_response(app.pattern_canvas.unwrap()).unwrap().rect
    }
    fn garment_pick_position(output: &egui::FullOutput, app: &Studio, piece: Id) -> Pos2 {
        let rect = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) if shape.fill == Color32::from_gray(37) => {
                    Some(shape.rect)
                }
                _ => None,
            })
            .next_back()
            .unwrap();
        let mesh = app.mesh.as_ref().unwrap();
        let triangles: Vec<_> = mesh
            .triangles
            .iter()
            .map(|triangle| {
                (
                    mesh.piece_ids[triangle[0]],
                    triangle.map(|index| app.project3(mesh.positions[index], rect).0),
                )
            })
            .collect();
        triangles
            .iter()
            .filter(|(id, _)| *id == piece)
            .find_map(|(_, triangle)| {
                let position = Pos2::new(
                    triangle.iter().map(|point| point.x).sum::<f32>() / 3.,
                    triangle.iter().map(|point| point.y).sum::<f32>() / 3.,
                );
                (rect.contains(position)
                    && triangles.iter().all(|(id, triangle)| {
                        *id == piece || !point_in_triangle(position, *triangle)
                    }))
                .then_some(position)
            })
            .expect("second piece has an isolated rendered triangle to pick")
    }
    fn same_construction(actual: &Project, expected: &Project) {
        let mut actual = actual.clone();
        actual.revision = expected.revision;
        assert_eq!(&actual, expected);
    }
    fn assert_near(actual: [f64; 2], expected: [f64; 2], tolerance: f64) {
        for axis in 0..2 {
            assert!(
                (actual[axis] - expected[axis]).abs() <= tolerance,
                "{actual:?} != {expected:?}"
            );
        }
    }
    fn focus_property(ctx: &egui::Context, app: &mut Studio, text: &str) -> egui::Id {
        frame(ctx, app, vec![]);
        let output = frame(ctx, app, vec![]);
        let id = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.numeric_value() == Some(0.37))
            .unwrap()
            .0;
        frame(ctx, app, vec![action(id, Action::Focus, None)]);
        frame(ctx, app, vec![]);
        frame(
            ctx,
            app,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Text(text.into()),
                key(egui::Key::ArrowLeft, egui::Modifiers::NONE),
            ],
        );
        numeric_focus(ctx).unwrap_or_else(|| {
            panic!(
                "property focus absent for {text:?}: {:?}",
                ctx.memory(|memory| memory.focused())
            )
        })
    }
    #[test]
    fn transform_search_and_tool_names_are_real_authoring_routes() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = Studio::empty();
        app.session_autosave = None;
        app.search = true;
        app.query = "Transform".into();
        let before = format!("{:?}", app.editor);
        frame(&ctx, &mut app, vec![]);
        let output = frame(&ctx, &mut app, vec![]);
        let route = button(&output, "Transform pattern");
        frame(&ctx, &mut app, vec![action(route, Action::Click, None)]);
        assert_eq!(app.tool.label(), "Transform pattern");
        assert!(!app.search);
        assert_eq!(format!("{:?}", app.editor), before);
        let output = frame(&ctx, &mut app, vec![]);
        frame(
            &ctx,
            &mut app,
            vec![action(
                button(&output, "Select / edit"),
                Action::Click,
                None,
            )],
        );
        assert!(app.tool == Tool::Select);
        let output = frame(&ctx, &mut app, vec![]);
        frame(
            &ctx,
            &mut app,
            vec![action(
                button(&output, "Transform pattern"),
                Action::Click,
                None,
            )],
        );
        assert!(app.tool == Tool::Transform);
        assert_eq!(format!("{:?}", app.editor), before);
    }

    #[test]
    fn transform_numeric_apply_and_return_use_explicit_units_pivot_and_one_transaction() {
        for enter in [false, true] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let before = app.editor.project.clone();
            let mut draft = InspectorDraft::new(&before, app.selected, app.selected_curve);
            draft.resolution = 0.18;
            app.draft = Some(draft);
            frame(&ctx, &mut app, vec![]);
            let output = frame(&ctx, &mut app, vec![]);
            frame(
                &ctx,
                &mut app,
                vec![action(
                    button(
                        &output,
                        if enter {
                            "Rotate pattern around bounds-center pivot"
                        } else {
                            "Transform pattern…"
                        },
                    ),
                    Action::Click,
                    None,
                )],
            );
            let edit = app.transform.as_ref().unwrap();
            assert_eq!(edit.pivot, [0.125, 0.625]);
            assert_eq!(app.editor.project, before);
            for (label, value) in [
                ("Translation U (m)", 0.5),
                ("Translation V (m)", -0.25),
                ("Angle (radians)", std::f64::consts::FRAC_PI_2),
                ("Pivot U (m)", 0.25),
                ("Pivot V (m)", 0.5),
            ] {
                set_field(&ctx, &mut app, label, value);
            }
            let output = frame(&ctx, &mut app, vec![]);
            let event = if enter {
                key(egui::Key::Enter, egui::Modifiers::NONE)
            } else {
                action(button(&output, "Apply transform"), Action::Click, None)
            };
            frame(&ctx, &mut app, vec![event]);
            assert!(app.transform.is_none());
            assert_eq!(app.editor.project.revision, before.revision + 1);
            assert_eq!(app.editor.project.resolution, 0.18);
            let selected = before.piece(app.selected.unwrap()).unwrap();
            for curve in &selected.boundary {
                let original = before.curve(*curve).unwrap();
                let transformed = app.editor.project.curve(*curve).unwrap();
                for t in [0., 0.125, 0.25, 0.5, 0.75, 0.875, 1.] {
                    let p = original.evaluate(&before, t).unwrap();
                    let q = transformed.evaluate(&app.editor.project, t).unwrap();
                    assert_near(q, [1.25 - p[1], p[0]], 1e-10);
                    let old_world = selected.placement.transform(p);
                    let new_world = app
                        .editor
                        .project
                        .piece(selected.id)
                        .unwrap()
                        .placement
                        .transform(q);
                    for axis in 0..3 {
                        assert!((old_world[axis] - new_world[axis]).abs() <= 1e-10);
                    }
                }
            }
            let after = app.editor.project.clone();
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
            );
            frame(&ctx, &mut app, vec![]);
            assert_eq!(
                app.editor.project, after,
                "modal blur must not replay old text"
            );
            app.history(&ctx, false);
            same_construction(&app.editor.project, &before);
            app.history(&ctx, true);
            same_construction(&app.editor.project, &after);
            let unchanged = format!("{:?}", app.editor);
            let mut draft =
                InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve);
            draft.resolution = 0.19;
            let pending = draft.commands(&app.editor.project);
            app.draft = Some(draft);
            app.open_transform(&ctx);
            let pivot_u = app.transform.as_ref().unwrap().pivot[0];
            frame(&ctx, &mut app, vec![]);
            set_field(&ctx, &mut app, "Pivot U (m)", 1001.);
            let output = frame(&ctx, &mut app, vec![]);
            frame(
                &ctx,
                &mut app,
                vec![action(
                    button(&output, "Apply transform"),
                    Action::Click,
                    None,
                )],
            );
            assert!(
                app.transform.as_ref().unwrap().error.is_some(),
                "identity still validates its explicit pivot"
            );
            assert_eq!(format!("{:?}", app.editor), unchanged);
            set_field(&ctx, &mut app, "Pivot U (m)", pivot_u);
            let output = frame(&ctx, &mut app, vec![]);
            frame(
                &ctx,
                &mut app,
                vec![action(
                    button(&output, "Apply transform"),
                    Action::Click,
                    None,
                )],
            );
            assert!(app.transform.is_none());
            assert_eq!(
                format!("{:?}", app.editor),
                unchanged,
                "identity sends no command"
            );
            assert_eq!(
                app.draft.as_ref().unwrap().commands(&app.editor.project),
                pending
            );
        }
    }

    #[test]
    fn transform_pointer_translation_and_rotation_capture_original_geometry_and_snap_once() {
        for (snap, bypass, expected) in [
            (PositionSnap::Mm10, false, [0.12, -0.23]),
            (PositionSnap::Mm10, true, [0.1234, -0.2344]),
            (PositionSnap::Off, false, [0.1234, -0.2344]),
        ] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            app.position_snap = snap;
            let mut draft =
                InspectorDraft::new(&app.editor.project, app.selected, app.selected_curve);
            draft.resolution = 0.18;
            app.draft = Some(draft);
            frame(&ctx, &mut app, vec![]);
            frame(&ctx, &mut app, vec![]);
            let before = app.editor.project.clone();
            let rect = canvas_rect(&ctx, &app);
            let start = app.to_screen([-0.375, 0.75], rect);
            pointer_frame(&ctx, &mut app, start, Some(true), bypass);
            assert!(app.transform_gesture.is_some());
            for displacement in [Vec2::new(70., 40.), Vec2::new(123.4, 234.4)] {
                app.position_snap = PositionSnap::Mm1;
                pointer_frame(&ctx, &mut app, start + displacement, None, !bypass);
                assert_eq!(app.editor.project, before, "preview is disposable");
                assert!(app.editor.can_redo());
            }
            let preview = app.transform_gesture.as_ref().unwrap().preview().unwrap();
            let first = before.points[0].position;
            assert_near(
                preview.points[0].position,
                [first[0] + expected[0], first[1] + expected[1]],
                2e-6,
            );
            pointer_frame(
                &ctx,
                &mut app,
                start + Vec2::new(123.4, 234.4),
                Some(false),
                !bypass,
            );
            assert_eq!(app.editor.project.revision, before.revision + 1);
            assert_eq!(app.editor.project.resolution, 0.18);
            for (old, new) in before.points.iter().zip(&app.editor.project.points) {
                let owned = before
                    .piece(app.selected.unwrap())
                    .unwrap()
                    .boundary
                    .iter()
                    .any(|curve| {
                        let curve = before.curve(*curve).unwrap();
                        curve.start == old.id || curve.end == old.id
                    });
                if owned {
                    assert_near(
                        new.position,
                        [old.position[0] + expected[0], old.position[1] + expected[1]],
                        2e-6,
                    );
                } else {
                    assert_eq!(new, old);
                }
            }
            let after = app.editor.project.clone();
            app.history(&ctx, false);
            same_construction(&app.editor.project, &before);
            app.history(&ctx, true);
            same_construction(&app.editor.project, &after);
        }
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = fixture();
        frame(&ctx, &mut app, vec![]);
        let output = frame(&ctx, &mut app, vec![]);
        let handle = button(&output, "Rotate pattern around bounds-center pivot");
        let bounds = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(id, _)| *id == handle)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        let start = Pos2::new(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        );
        let pivot = app.to_screen([0.125, 0.625], canvas_rect(&ctx, &app));
        let radius = pivot.distance(start);
        assert!(radius >= 100.);
        let before = app.editor.project.clone();
        pointer_frame(&ctx, &mut app, start, Some(true), false);
        let end = pivot + Vec2::new(-radius, 0.);
        pointer_frame(
            &ctx,
            &mut app,
            pivot + Vec2::new(-radius * 0.5, -radius * 0.8660254),
            None,
            true,
        );
        pointer_frame(&ctx, &mut app, end, None, true);
        assert_eq!(app.editor.project, before);
        assert!(
            (app.transform_gesture.as_ref().unwrap().edit.angle - std::f64::consts::FRAC_PI_2)
                .abs()
                <= 2e-6
        );
        pointer_frame(&ctx, &mut app, end, Some(false), true);
        assert_eq!(app.editor.project.revision, before.revision + 1);
        for (old, new) in before.points[..4]
            .iter()
            .zip(&app.editor.project.points[..4])
        {
            assert_near(
                new.position,
                [0.75 - old.position[1], old.position[0] + 0.5],
                2e-6,
            );
        }
        let after = app.editor.project.clone();
        app.history(&ctx, false);
        same_construction(&app.editor.project, &before);
        app.history(&ctx, true);
        same_construction(&app.editor.project, &after);
    }

    #[test]
    fn transform_cross_piece_refusal_retains_staged_or_invalid_raw_text_caret_and_redo() {
        for text in ["0.47", "1e-", "NaN"] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let focused = focus_property(&ctx, &mut app, text);
            let raw = ctx.data_mut(|data| data.get_temp::<String>(focused));
            let caret = egui::TextEdit::load_state(&ctx, focused)
                .unwrap()
                .cursor
                .char_range();
            let before = format!("{:?}", app.editor);
            let selection = (app.selected, app.selected_curve);
            let pending = format!(
                "{:?}",
                app.draft.as_ref().unwrap().commands(&app.editor.project)
            );
            let target = app.to_screen([0.85, 0.45], canvas_rect(&ctx, &app));
            pointer_frame(&ctx, &mut app, target, Some(true), false);
            pointer_frame(&ctx, &mut app, target, Some(false), false);
            frame(&ctx, &mut app, vec![]);
            assert_eq!((app.selected, app.selected_curve), selection);
            assert_eq!(format!("{:?}", app.editor), before);
            assert_eq!(
                format!(
                    "{:?}",
                    app.draft.as_ref().unwrap().commands(&app.editor.project)
                ),
                pending
            );
            assert_eq!(numeric_focus(&ctx), Some(focused));
            assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(focused)), raw);
            assert_eq!(
                egui::TextEdit::load_state(&ctx, focused)
                    .unwrap()
                    .cursor
                    .char_range(),
                caret
            );
            assert!(app.editor.can_redo());
            assert!(app.transform_gesture.is_none());
            assert!(app.message.contains("Apply or cancel"));
        }
        let ctx = egui::Context::default();
        let mut app = fixture();
        frame(&ctx, &mut app, vec![]);
        frame(&ctx, &mut app, vec![]);
        let second = app.editor.project.pieces[1].id;
        let before = app.editor.project.clone();
        let target = app.to_screen([0.85, 0.45], canvas_rect(&ctx, &app));
        pointer_frame(&ctx, &mut app, target, Some(true), false);
        pointer_frame(&ctx, &mut app, target, Some(false), false);
        assert_eq!(app.selected, Some(second));
        assert_eq!(app.editor.project, before);
        assert!(app.editor.can_redo());
    }

    #[test]
    fn transform_scene_row_selection_preserves_staged_and_raw_edits_through_real_ui_actions() {
        for pointer in [false, true] {
            for text in ["0.47", "1e-", "NaN"] {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let mut app = fixture();
                let focused = focus_property(&ctx, &mut app, text);
                let raw = ctx.data_mut(|data| data.get_temp::<String>(focused));
                let caret = egui::TextEdit::load_state(&ctx, focused)
                    .unwrap()
                    .cursor
                    .char_range();
                let before = format!("{:?}", app.editor);
                let selected = (app.selected, app.selected_curve);
                let pending = format!(
                    "{:?}",
                    app.draft.as_ref().unwrap().commands(&app.editor.project)
                );
                let output = frame(&ctx, &mut app, vec![]);
                let row = button(&output, "Separate panel");
                if pointer {
                    let bounds = output
                        .platform_output
                        .accesskit_update
                        .as_ref()
                        .unwrap()
                        .nodes
                        .iter()
                        .find(|(id, _)| *id == row)
                        .unwrap()
                        .1
                        .bounds()
                        .unwrap();
                    let position = Pos2::new(
                        ((bounds.x0 + bounds.x1) * 0.5) as f32,
                        ((bounds.y0 + bounds.y1) * 0.5) as f32,
                    );
                    pointer_frame(&ctx, &mut app, position, Some(true), false);
                    assert_eq!(
                        format!("{:?}", app.editor),
                        before,
                        "Scene row press must retain {text:?}"
                    );
                    pointer_frame(&ctx, &mut app, position, Some(false), false);
                } else {
                    frame(&ctx, &mut app, vec![action(row, Action::Click, None)]);
                }
                frame(&ctx, &mut app, vec![]);
                frame(&ctx, &mut app, vec![]);
                assert_eq!(
                    format!("{:?}", app.editor),
                    before,
                    "Scene selection pointer={pointer}, text={text:?}"
                );
                assert_eq!((app.selected, app.selected_curve), selected);
                assert_eq!(
                    format!(
                        "{:?}",
                        app.draft.as_ref().unwrap().commands(&app.editor.project)
                    ),
                    pending
                );
                assert_eq!(numeric_focus(&ctx), Some(focused));
                assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(focused)), raw);
                assert_eq!(
                    egui::TextEdit::load_state(&ctx, focused)
                        .unwrap()
                        .cursor
                        .char_range(),
                    caret
                );
                assert!(app.editor.can_redo());
                assert!(app.message.contains("Apply or cancel"));
                assert!(!app.dirty);
            }
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let before = format!("{:?}", app.editor);
            frame(&ctx, &mut app, vec![]);
            let output = frame(&ctx, &mut app, vec![]);
            let row = button(&output, "Separate panel");
            frame(&ctx, &mut app, vec![action(row, Action::Click, None)]);
            assert_eq!(app.selected, Some(app.editor.project.pieces[1].id));
            assert_eq!(format!("{:?}", app.editor), before);
        }
    }

    #[test]
    fn transform_garment_pick_preserves_edits_before_press_blur_and_on_release() {
        for text in ["0.47", "1e-", "NaN"] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let mesh = Arc::new(app.editor.project.mesh().unwrap());
            app.mesh = Some(mesh.clone());
            let focused = focus_property(&ctx, &mut app, text);
            let raw = ctx.data_mut(|data| data.get_temp::<String>(focused));
            let caret = egui::TextEdit::load_state(&ctx, focused)
                .unwrap()
                .cursor
                .char_range();
            let before = format!("{:?}", app.editor);
            let selected = (app.selected, app.selected_curve);
            let pending = format!(
                "{:?}",
                app.draft.as_ref().unwrap().commands(&app.editor.project)
            );
            let output = frame(&ctx, &mut app, vec![]);
            let position = garment_pick_position(&output, &app, app.editor.project.pieces[1].id);
            pointer_frame(&ctx, &mut app, position, Some(true), false);
            assert_eq!(
                format!("{:?}", app.editor),
                before,
                "3D press must retain {text:?}"
            );
            pointer_frame(&ctx, &mut app, position, Some(false), false);
            frame(&ctx, &mut app, vec![]);
            frame(&ctx, &mut app, vec![]);
            assert_eq!(format!("{:?}", app.editor), before);
            assert_eq!((app.selected, app.selected_curve), selected);
            assert_eq!(
                format!(
                    "{:?}",
                    app.draft.as_ref().unwrap().commands(&app.editor.project)
                ),
                pending
            );
            assert_eq!(numeric_focus(&ctx), Some(focused));
            assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(focused)), raw);
            assert_eq!(
                egui::TextEdit::load_state(&ctx, focused)
                    .unwrap()
                    .cursor
                    .char_range(),
                caret
            );
            assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
            assert!(app.editor.can_redo());
            assert!(!app.dirty);
            assert!(app.message.contains("Apply or cancel"));
        }
        for tool in [Tool::Transform, Tool::Select] {
            let ctx = egui::Context::default();
            let mut app = fixture();
            app.set_tool(tool);
            app.mesh = Some(Arc::new(app.editor.project.mesh().unwrap()));
            let before = format!("{:?}", app.editor);
            frame(&ctx, &mut app, vec![]);
            let output = frame(&ctx, &mut app, vec![]);
            let second = app.editor.project.pieces[1].id;
            let position = garment_pick_position(&output, &app, second);
            pointer_frame(&ctx, &mut app, position, Some(true), false);
            pointer_frame(&ctx, &mut app, position, Some(false), false);
            assert_eq!(app.selected, Some(second));
            assert_eq!(format!("{:?}", app.editor), before);
        }
    }

    #[test]
    fn transform_numeric_failure_retains_text_caret_drafts_history_and_accepted_output() {
        for entered in ["1000", "1e-", "NaN"] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let property = focus_property(&ctx, &mut app, "0.47");
            let property_text = ctx.data_mut(|data| data.get_temp::<String>(property));
            let property_caret = egui::TextEdit::load_state(&ctx, property)
                .unwrap()
                .cursor
                .char_range();
            let before = format!("{:?}", app.editor);
            let pending = app.draft.as_ref().unwrap().commands(&app.editor.project);
            let mesh = Arc::new(app.editor.project.mesh().unwrap());
            let cloth = Arc::new(
                weftforge_core::simulation::create_cloth(&app.editor.project, &mesh).unwrap(),
            );
            app.mesh = Some(mesh.clone());
            app.cloth = Some(cloth.clone());
            let (_sender, receiver) = mpsc::channel();
            let cancel = Arc::new(AtomicBool::new(false));
            app.job = Some(Job {
                id: 91,
                revision: app.editor.project.revision,
                kind: JobKind::Simulation,
                batch: None,
                completed_steps: Arc::new(AtomicU64::new(0)),
                cancel: cancel.clone(),
                receiver,
            });
            let output = frame(&ctx, &mut app, vec![]);
            frame(
                &ctx,
                &mut app,
                vec![action(
                    button(&output, "Transform pattern…"),
                    Action::Click,
                    None,
                )],
            );
            let output = frame(&ctx, &mut app, vec![]);
            frame(
                &ctx,
                &mut app,
                vec![action(
                    field(&output, "Translation U (m)"),
                    Action::Focus,
                    None,
                )],
            );
            frame(&ctx, &mut app, vec![]);
            frame(
                &ctx,
                &mut app,
                vec![
                    key(egui::Key::A, egui::Modifiers::COMMAND),
                    egui::Event::Text(entered.into()),
                    key(egui::Key::ArrowLeft, egui::Modifiers::NONE),
                ],
            );
            let focused = numeric_focus(&ctx).unwrap();
            let raw = ctx.data_mut(|data| data.get_temp::<String>(focused));
            let caret = egui::TextEdit::load_state(&ctx, focused)
                .unwrap()
                .cursor
                .char_range();
            let output = frame(&ctx, &mut app, vec![]);
            let event = if entered == "1e-" {
                key(egui::Key::Enter, egui::Modifiers::NONE)
            } else {
                action(button(&output, "Apply transform"), Action::Click, None)
            };
            frame(&ctx, &mut app, vec![event]);
            frame(&ctx, &mut app, vec![]);
            assert!(app.transform.as_ref().unwrap().error.is_some());
            assert_eq!(format!("{:?}", app.editor), before);
            assert_eq!(
                app.draft.as_ref().unwrap().commands(&app.editor.project),
                pending
            );
            assert_eq!(numeric_focus(&ctx), Some(focused));
            assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(focused)), raw);
            assert_eq!(
                egui::TextEdit::load_state(&ctx, focused)
                    .unwrap()
                    .cursor
                    .char_range(),
                caret
            );
            assert_eq!(
                ctx.data_mut(|data| data.get_temp::<String>(property)),
                property_text
            );
            assert_eq!(
                egui::TextEdit::load_state(&ctx, property)
                    .unwrap()
                    .cursor
                    .char_range(),
                property_caret
            );
            assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
            assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &cloth));
            assert!(!cancel.load(Ordering::Relaxed));
            assert_eq!(app.job.as_ref().unwrap().id, 91);
            assert!(!app.dirty);
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
            );
            let output = frame(&ctx, &mut app, vec![]);
            assert_eq!(format!("{:?}", app.editor), before);
            if entered != "1000" {
                assert!(
                    app.transform.as_ref().unwrap().invalid_text[0].as_deref() == Some(entered)
                );
            }
            frame(
                &ctx,
                &mut app,
                vec![action(
                    button(&output, "Cancel transform"),
                    Action::Click,
                    None,
                )],
            );
            frame(&ctx, &mut app, vec![]);
            frame(&ctx, &mut app, vec![]);
            assert!(app.transform.is_none());
            assert_eq!(format!("{:?}", app.editor), before);
            assert_eq!(
                app.draft.as_ref().unwrap().commands(&app.editor.project),
                pending
            );
            assert_eq!(
                ctx.data_mut(|data| data.get_temp::<String>(property)),
                property_text
            );
            assert_eq!(
                egui::TextEdit::load_state(&ctx, property)
                    .unwrap()
                    .cursor
                    .char_range(),
                property_caret
            );
            assert!(!app.dirty);
            assert!(!cancel.load(Ordering::Relaxed));
        }
    }

    #[test]
    fn transform_modal_blocks_background_accesskit_pointer_and_shortcut_authoring() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = fixture();
        let property = focus_property(&ctx, &mut app, "0.47");
        let output = frame(&ctx, &mut app, vec![]);
        let background = ["Add rectangle", "Add ellipse", "Undo", "Select / edit"]
            .map(|label| button(&output, label));
        let numeric = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.numeric_value() == Some(0.47))
            .unwrap()
            .0;
        let before = format!("{:?}", app.editor);
        let pending = app.draft.as_ref().unwrap().commands(&app.editor.project);
        frame(
            &ctx,
            &mut app,
            vec![action(
                button(&output, "Transform pattern…"),
                Action::Click,
                None,
            )],
        );
        for target in background {
            frame(&ctx, &mut app, vec![action(target, Action::Click, None)]);
        }
        for event in [
            action(
                numeric,
                Action::SetValue,
                Some(ActionData::NumericValue(0.95)),
            ),
            action(numeric, Action::Increment, None),
            key(egui::Key::Z, egui::Modifiers::COMMAND),
            key(
                egui::Key::Z,
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            ),
            key(egui::Key::Space, egui::Modifiers::NONE),
            key(egui::Key::K, egui::Modifiers::COMMAND),
        ] {
            frame(&ctx, &mut app, vec![event]);
            assert_eq!(format!("{:?}", app.editor), before);
            assert_eq!(
                app.draft.as_ref().unwrap().commands(&app.editor.project),
                pending
            );
            assert!(app.transform.is_some());
            assert!(app.tool == Tool::Transform);
            assert!(!app.playing && !app.search);
        }
        frame(
            &ctx,
            &mut app,
            vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
        );
        frame(&ctx, &mut app, vec![]);
        frame(&ctx, &mut app, vec![]);
        assert_eq!(format!("{:?}", app.editor), before);
        assert_eq!(
            app.draft.as_ref().unwrap().commands(&app.editor.project),
            pending
        );
        assert_eq!(
            ctx.data_mut(|data| data.get_temp::<String>(property))
                .as_deref(),
            Some("0.47")
        );
        // A backdrop click may cancel the modal, but must not press the background authoring button.
        let output = frame(&ctx, &mut app, vec![]);
        let target = button(&output, "Add rectangle");
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
        let position = Pos2::new(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        );
        app.open_transform(&ctx);
        frame(&ctx, &mut app, vec![]);
        pointer_frame(&ctx, &mut app, position, Some(true), false);
        pointer_frame(&ctx, &mut app, position, Some(false), false);
        assert_eq!(format!("{:?}", app.editor), before);
        assert_eq!(
            app.draft.as_ref().unwrap().commands(&app.editor.project),
            pending
        );
        assert!(!app.dirty);
    }

    #[test]
    fn transform_cancel_revision_tool_load_and_failed_release_never_commit_stale_previews() {
        for cancel in ["escape", "tool", "revision", "history", "load"] {
            let ctx = egui::Context::default();
            let mut app = fixture();
            frame(&ctx, &mut app, vec![]);
            frame(&ctx, &mut app, vec![]);
            let before = format!("{:?}", app.editor);
            let rect = canvas_rect(&ctx, &app);
            let start = app.to_screen([-0.375, 0.75], rect);
            let end = start + Vec2::new(125., 150.);
            pointer_frame(&ctx, &mut app, start, Some(true), false);
            pointer_frame(&ctx, &mut app, end, None, false);
            assert!(app.transform_gesture.is_some());
            assert_eq!(format!("{:?}", app.editor), before);
            match cancel {
                "escape" => {
                    frame(
                        &ctx,
                        &mut app,
                        vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
                    );
                }
                "tool" => app.set_tool(Tool::Select),
                "revision" => {
                    app.editor
                        .execute(Command::SetResolution { metres: 0.19 })
                        .unwrap();
                }
                "history" => app.history(&ctx, false),
                "load" => {
                    let path = std::env::temp_dir().join(format!(
                        "weftforge-native-transform-{}.weftforge",
                        std::process::id()
                    ));
                    app.editor.project.save(&path).unwrap();
                    app.load_document(path.clone(), false);
                    fs::remove_file(path).unwrap();
                }
                _ => unreachable!(),
            }
            let after = format!("{:?}", app.editor);
            pointer_frame(&ctx, &mut app, end, Some(false), false);
            frame(&ctx, &mut app, vec![]);
            assert!(app.transform_gesture.is_none());
            assert!(app.transform.is_none());
            assert_eq!(format!("{:?}", app.editor), after);
            if cancel == "escape" || cancel == "tool" {
                assert_eq!(after, before);
            }
        }
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = fixture();
        let property = focus_property(&ctx, &mut app, "0.47");
        let raw = ctx.data_mut(|data| data.get_temp::<String>(property));
        let caret = egui::TextEdit::load_state(&ctx, property)
            .unwrap()
            .cursor
            .char_range();
        let before = app.editor.project.clone();
        let history = format!("{:?}", app.editor);
        let pending = app.draft.as_ref().unwrap().commands(&before);
        let start = app.to_screen([-0.375, 0.75], canvas_rect(&ctx, &app));
        pointer_frame(&ctx, &mut app, start, Some(true), false);
        let end = start + Vec2::new(1_001_000., 0.);
        pointer_frame(&ctx, &mut app, end, None, false);
        pointer_frame(&ctx, &mut app, end, Some(false), false);
        assert_eq!(format!("{:?}", app.editor), history);
        assert_eq!(app.draft.as_ref().unwrap().commands(&before), pending);
        let edit = app.transform.as_ref().unwrap();
        assert!(edit.error.is_some());
        assert_eq!(edit.translation, [1001., 0.]);
        assert_eq!(edit.revision, before.revision);
        assert!(app.transform_gesture.is_none());
        assert_eq!(ctx.data_mut(|data| data.get_temp::<String>(property)), raw);
        assert_eq!(
            egui::TextEdit::load_state(&ctx, property)
                .unwrap()
                .cursor
                .char_range(),
            caret
        );
        set_field(&ctx, &mut app, "Translation U (m)", 0.5);
        frame(
            &ctx,
            &mut app,
            vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        assert!(app.transform.is_none());
        assert_eq!(app.editor.project.revision, before.revision + 1);
        assert!(
            (app.editor.project.points[0].position[0] - before.points[0].position[0] - 0.5).abs()
                <= 1e-10
        );
        for raw in ["1e-", "NaN"] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = fixture();
            let property = focus_property(&ctx, &mut app, raw);
            let caret = egui::TextEdit::load_state(&ctx, property)
                .unwrap()
                .cursor
                .char_range();
            let before = format!("{:?}", app.editor);
            let pending = format!(
                "{:?}",
                app.draft.as_ref().unwrap().commands(&app.editor.project)
            );
            let start = app.to_screen([-0.375, 0.75], canvas_rect(&ctx, &app));
            pointer_frame(&ctx, &mut app, start, Some(true), false);
            pointer_frame(&ctx, &mut app, start + Vec2::new(100., 0.), None, false);
            pointer_frame(
                &ctx,
                &mut app,
                start + Vec2::new(100., 0.),
                Some(false),
                false,
            );
            assert!(app.transform.as_ref().unwrap().error.is_some());
            assert_eq!(format!("{:?}", app.editor), before);
            assert_eq!(
                format!(
                    "{:?}",
                    app.draft.as_ref().unwrap().commands(&app.editor.project)
                ),
                pending
            );
            assert_eq!(
                ctx.data_mut(|data| data.get_temp::<String>(property))
                    .as_deref(),
                Some(raw)
            );
            assert_eq!(
                egui::TextEdit::load_state(&ctx, property)
                    .unwrap()
                    .cursor
                    .char_range(),
                caret
            );
            frame(
                &ctx,
                &mut app,
                vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
            );
            frame(&ctx, &mut app, vec![]);
            frame(&ctx, &mut app, vec![]);
            assert_eq!(format!("{:?}", app.editor), before);
            assert_eq!(
                ctx.data_mut(|data| data.get_temp::<String>(property))
                    .as_deref(),
                Some(raw)
            );
            assert_eq!(
                egui::TextEdit::load_state(&ctx, property)
                    .unwrap()
                    .cursor
                    .char_range(),
                caret
            );
        }
    }
}

#[cfg(test)]
mod authored_length_tests {
    use super::*;
    use std::cell::Cell;

    thread_local! { static QUERIES: Cell<usize> = const { Cell::new(0) }; }
    pub(super) fn record_query() {
        QUERIES.set(QUERIES.get() + 1);
    }

    fn scene() -> Studio {
        let mut app = Studio::empty();
        app.session_autosave = None;
        for (index, width) in [0.3, 0.5].into_iter().enumerate() {
            app.editor
                .execute(Command::AddRectangle {
                    name: format!("Original panel {index}"),
                    origin: [index as f64, 0.],
                    width,
                    height: 0.2,
                })
                .unwrap();
        }
        let a = CurveInterval::full(app.editor.project.pieces[0].boundary[0]);
        let b = CurveInterval::reversed(app.editor.project.pieces[1].boundary[0]);
        app.editor
            .execute(Command::AddSeam {
                name: "Original legacy pair".into(),
                a,
                b,
                correspondence: vec![[0., 0.], [1., 1.]],
            })
            .unwrap();
        app.editor
            .execute(Command::AddSewingGroup {
                name: "Original ordered pair".into(),
                a: vec![a],
                b: vec![b],
                correspondence: vec![[0., 0.], [1., 1.]],
            })
            .unwrap();
        app.selected = Some(app.editor.project.pieces[0].id);
        app.selected_curve = Some(a.curve);
        app
    }

    fn frame(
        ctx: &egui::Context,
        app: &mut Studio,
        height: f32,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(520., height))),
                events,
                ..Default::default()
            },
            |ui| app.inspector(ui),
        )
    }

    fn text(output: &egui::FullOutput) -> String {
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == egui::accesskit::Role::Label)
            .filter_map(|(_, node)| node.value().or_else(|| node.label()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn applied_lengths_are_formatted_cached_and_ignore_drafts_preview_and_simulation() {
        assert_eq!(length_mm(0.0000016), "0.002 mm");
        assert_eq!(
            pair_lengths_mm([0.3, 0.5]),
            "A 300.000 mm\nB 500.000 mm\nB − A +200.000 mm"
        );
        assert_eq!(
            pair_lengths_mm([0.5, 0.3]),
            "A 500.000 mm\nB 300.000 mm\nB − A -200.000 mm"
        );
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = scene();
        let mesh = Arc::new(app.editor.project.mesh().unwrap());
        let mut simulated =
            weftforge_core::simulation::create_cloth(&app.editor.project, &mesh).unwrap();
        simulated.step(Default::default(), || false).unwrap();
        let cloth = Arc::new(simulated);
        app.mesh = Some(mesh.clone());
        app.cloth = Some(cloth.clone());
        let (sender, receiver) = mpsc::channel();
        app.job = Some(Job {
            id: 71,
            revision: app.editor.project.revision,
            kind: JobKind::Simulation,
            batch: None,
            completed_steps: Arc::new(AtomicU64::new(0)),
            cancel: Arc::new(AtomicBool::new(false)),
            receiver,
        });
        let original = format!("{:?}", app.editor);
        QUERIES.set(0);
        frame(&ctx, &mut app, 3000., vec![]);
        for _ in 0..5 {
            let output = frame(&ctx, &mut app, 3000., vec![]);
            let labels = text(&output);
            assert!(labels.contains(&format!(
                "Applied rest length · revision {}",
                app.editor.project.revision
            )));
            assert!(labels.contains("300.000 mm"));
            assert_eq!(labels.matches("B − A +200.000 mm").count(), 2);
            assert!(labels.contains("Draft/preview edits are not included."));
        }
        assert_eq!(
            QUERIES.get(),
            2,
            "one selected curve and one visible legacy seam; groups use the mesh"
        );
        app.draft.as_mut().unwrap().curve.as_mut().unwrap().controls =
            Some([[0.1, -0.4], [0.2, -0.4]]);
        app.preview = Some([4., 5.]);
        app.drag_point = Some(app.editor.project.curves[0].start);
        let mut draft_group = app.editor.project.sewing_groups[0].clone();
        draft_group.a[0].t1 = 0.5;
        app.sewing_edit = Some(SewingGroupEdit {
            group: draft_group,
            focus: None,
            epoch: 0,
            error: None,
        });
        for _ in 0..4 {
            let output = frame(&ctx, &mut app, 3000., vec![]);
            assert_eq!(text(&output).matches("B − A +200.000 mm").count(), 2);
        }
        assert_eq!(QUERIES.get(), 2);
        assert_eq!(app.draft.as_ref().unwrap().applied_length, Some(Ok(0.3)));
        app.sewing_edit = None;
        frame(&ctx, &mut app, 3000., vec![]);
        assert_eq!(QUERIES.get(), 2);
        assert_eq!(format!("{:?}", app.editor), original);
        assert!(Arc::ptr_eq(app.mesh.as_ref().unwrap(), &mesh));
        assert!(Arc::ptr_eq(app.cloth.as_ref().unwrap(), &cloth));
        assert_eq!(app.job.as_ref().unwrap().id, 71);
        assert!(!app.job.as_ref().unwrap().cancel.load(Ordering::Relaxed));
        assert!(!app.dirty && !app.needs_mesh);
        drop(sender);
    }

    #[test]
    fn feedback_state_changes_preserve_focused_numeric_text_caret_and_pending_edits() {
        use egui::accesskit::{Action, ActionRequest, TreeId};
        for input in ["0.450", "1e-", "NaN"] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut app = scene();
            let mesh = Arc::new(app.editor.project.mesh().unwrap());
            frame(&ctx, &mut app, 3000., vec![]);
            let output = frame(&ctx, &mut app, 3000., vec![]);
            let field = output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| node.numeric_value() == Some(0.3))
                .unwrap()
                .0;
            frame(
                &ctx,
                &mut app,
                3000.,
                vec![egui::Event::AccessKitActionRequest(ActionRequest {
                    action: Action::Focus,
                    target_tree: TreeId::ROOT,
                    target_node: field,
                    data: None,
                })],
            );
            frame(&ctx, &mut app, 3000., vec![]);
            frame(
                &ctx,
                &mut app,
                3000.,
                vec![
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::COMMAND,
                    },
                    egui::Event::Text(input.into()),
                ],
            );
            let focused = numeric_focus(&ctx).unwrap();
            let caret = egui::TextEdit::load_state(&ctx, focused)
                .unwrap()
                .cursor
                .char_range();
            let original = format!("{:?}", app.editor);
            let pending = app.draft.as_ref().unwrap().commands(&app.editor.project);
            app.drag_point = Some(app.editor.project.curves[0].start);
            app.preview = Some([0.1, -0.2]);
            app.sewing_edit = Some(SewingGroupEdit {
                group: app.editor.project.sewing_groups[0].clone(),
                focus: Some(focused),
                epoch: 0,
                error: None,
            });
            for matching in [true, false, true] {
                app.mesh = matching.then(|| mesh.clone());
                for _ in 0..3 {
                    frame(&ctx, &mut app, 3000., vec![]);
                }
                assert_eq!(numeric_focus(&ctx), Some(focused));
                assert_eq!(
                    ctx.data_mut(|data| data.get_temp::<String>(focused))
                        .as_deref(),
                    Some(input)
                );
                assert_eq!(
                    egui::TextEdit::load_state(&ctx, focused)
                        .unwrap()
                        .cursor
                        .char_range(),
                    caret
                );
                assert_eq!(format!("{:?}", app.editor), original);
                assert_eq!(
                    app.draft.as_ref().unwrap().commands(&app.editor.project),
                    pending
                );
                assert!(!app.dirty && !app.needs_mesh && app.job.is_none());
            }
            app.sewing_edit = None;
            for _ in 0..3 {
                frame(&ctx, &mut app, 3000., vec![]);
            }
            assert_eq!(numeric_focus(&ctx), Some(focused));
            assert_eq!(
                ctx.data_mut(|data| data.get_temp::<String>(focused))
                    .as_deref(),
                Some(input)
            );
            assert_eq!(format!("{:?}", app.editor), original);
        }
    }

    #[test]
    fn revisions_and_same_identity_document_replacement_invalidate_lengths() {
        let ctx = egui::Context::default();
        let mut app = scene();
        frame(&ctx, &mut app, 3000., vec![]);
        let original = app.draft.as_ref().unwrap().applied_length.clone();
        let curve = app.selected_curve.unwrap();
        let revision = app.editor.project.revision;
        app.execute(Command::SetCurveControls {
            curve,
            controls: Some([[0.1, -0.1], [0.2, -0.1]]),
        });
        assert_eq!(app.editor.project.revision, revision + 1);
        assert!(app.draft.is_none() && app.legacy_lengths.values.is_empty());
        frame(&ctx, &mut app, 3000., vec![]);
        let edited = app.draft.as_ref().unwrap().applied_length.clone();
        assert!(edited.as_ref().unwrap().as_ref().unwrap() > &0.3);
        assert!(app.editor.undo());
        app.rebuild();
        frame(&ctx, &mut app, 3000., vec![]);
        assert_eq!(app.draft.as_ref().unwrap().applied_length, original);
        assert!(app.editor.project.revision > revision + 1);
        assert!(app.editor.redo());
        app.rebuild();
        frame(&ctx, &mut app, 3000., vec![]);
        assert_eq!(app.draft.as_ref().unwrap().applied_length, edited);
        let revision = app.editor.project.revision;
        let selected = app.selected;
        let mut replacement = Editor::new(app.editor.project.clone()).unwrap();
        replacement
            .execute(Command::SetCurveControls {
                curve,
                controls: None,
            })
            .unwrap();
        replacement
            .execute(Command::ResizePiece {
                piece: selected.unwrap(),
                width: 0.45,
                height: 0.2,
            })
            .unwrap();
        replacement.project.revision = revision;
        let path = std::env::temp_dir().join(format!(
            "weftforge-length-replacement-{}-{}.weftforge",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        replacement.project.save(&path).unwrap();
        for recovered in [false, true] {
            app.load_document(path.clone(), recovered);
            assert_eq!(app.editor.project.revision, revision);
            assert!(app.draft.is_none() && app.legacy_lengths.values.is_empty());
            app.selected = selected;
            app.selected_curve = Some(curve);
            frame(&ctx, &mut app, 3000., vec![]);
            assert!(
                (app.draft
                    .as_ref()
                    .unwrap()
                    .applied_length
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    - 0.45)
                    .abs()
                    < 1e-12
            );
            assert!(
                (app.legacy_lengths.values[&app.editor.project.seams[0].id]
                    .as_ref()
                    .unwrap()[0]
                    - 0.45)
                    .abs()
                    < 1e-12
            );
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn scrolling_legacy_feedback_preserves_content_height_without_measuring_hidden_rows() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = scene();
        app.selected = None;
        app.selected_curve = None;
        let seam = app.editor.project.seams[0].clone();
        for index in 0..24 {
            app.editor
                .execute(Command::AddSeam {
                    name: format!("Original repeated pair {index}"),
                    a: seam.a,
                    b: seam.b,
                    correspondence: seam.correspondence.clone(),
                })
                .unwrap();
        }
        let snapshot = |ctx: &egui::Context| {
            ctx.data_mut(|data| {
                data.get_temp::<(egui::Id, Vec2, Rect, Vec2)>(egui::Id::new(
                    "authored-length-scroll",
                ))
            })
            .unwrap()
        };
        QUERIES.set(0);
        frame(&ctx, &mut app, 420., vec![]);
        let output = frame(&ctx, &mut app, 420., vec![]);
        let (id, initial_size, viewport, _) = snapshot(&ctx);
        let first = output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Label
                    && node
                        .value()
                        .or_else(|| node.label())
                        .is_some_and(|text| text.starts_with("Original legacy pair ·"))
            })
            .unwrap()
            .1
            .bounds()
            .unwrap();
        let initial_queries = QUERIES.get();
        assert!(initial_queries < app.editor.project.seams.len());
        let mut state = egui::scroll_area::State::load(&ctx, id).unwrap();
        state.offset.y = first.y0 as f32 - viewport.top();
        state.store(&ctx, id);
        frame(&ctx, &mut app, 420., vec![]);
        frame(&ctx, &mut app, 420., vec![]);
        let (_, scrolled_size, _, offset) = snapshot(&ctx);
        assert!(offset.y > 1., "the inspector must actually scroll");
        assert!(QUERIES.get() > initial_queries && QUERIES.get() < app.editor.project.seams.len());
        assert!(
            (initial_size.y - scrolled_size.y).abs() <= 0.01,
            "scrolling changed content height: {} -> {}",
            initial_size.y,
            scrolled_size.y
        );
        let queries = QUERIES.get();
        for _ in 0..3 {
            frame(&ctx, &mut app, 420., vec![]);
        }
        assert_eq!(QUERIES.get(), queries);
        let mut state = egui::scroll_area::State::load(&ctx, id).unwrap();
        state.offset.y = 0.;
        state.store(&ctx, id);
        frame(&ctx, &mut app, 420., vec![]);
        frame(&ctx, &mut app, 420., vec![]);
        assert!((snapshot(&ctx).1.y - initial_size.y).abs() <= 0.01);
        assert_eq!(QUERIES.get(), queries);
        for value in app.legacy_lengths.values.values_mut() {
            *value = Err("Original long metric error for the full hover reason. ".repeat(20));
        }
        frame(&ctx, &mut app, 420., vec![]);
        assert!(
            (snapshot(&ctx).1.y - initial_size.y).abs() <= 0.01,
            "errors must reserve the same row height"
        );
        assert_eq!(QUERIES.get(), queries);
        eprintln!(
            "legacy scroll content height: initial={} scrolled={} returned={} px; measured {queries}/{} legacy pairs",
            initial_size.y,
            scrolled_size.y,
            snapshot(&ctx).1.y,
            app.editor.project.seams.len()
        );
    }

    #[test]
    fn partially_clipped_legacy_feedback_is_measured_once() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = scene();
        app.selected = None;
        app.selected_curve = None;
        QUERIES.set(0);
        frame(&ctx, &mut app, 300., vec![]);
        let output = frame(&ctx, &mut app, 300., vec![]);
        assert!(app.legacy_lengths.values.is_empty());
        let (id, _, viewport, _) = ctx
            .data_mut(|data| {
                data.get_temp::<(egui::Id, Vec2, Rect, Vec2)>(egui::Id::new(
                    "authored-length-scroll",
                ))
            })
            .unwrap();
        let row = output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Label
                    && node.value().or_else(|| node.label()) == Some("A — mm")
            })
            .unwrap()
            .1
            .bounds()
            .unwrap();
        let mut state = egui::scroll_area::State::load(&ctx, id).unwrap();
        // The value row is at the top while the block's heading is clipped above it.
        state.offset.y = row.y0 as f32 - viewport.top();
        state.store(&ctx, id);
        for _ in 0..4 {
            frame(&ctx, &mut app, 300., vec![]);
        }
        assert!(
            app.legacy_lengths
                .values
                .contains_key(&app.editor.project.seams[0].id)
        );
        assert_eq!(QUERIES.get(), 1);
    }

    #[test]
    fn legacy_queries_are_lazy_errors_cached_and_groups_wait_for_matching_mesh_jobs() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = scene();
        app.selected = None;
        app.selected_curve = None;
        QUERIES.set(0);
        for _ in 0..3 {
            frame(&ctx, &mut app, 180., vec![]);
        }
        assert!(app.legacy_lengths.values.is_empty());
        assert_eq!(QUERIES.get(), 0);
        frame(&ctx, &mut app, 3000., vec![]);
        assert_eq!(QUERIES.get(), 1);
        let seam = app.editor.project.seams[0].id;
        app.editor.project.seams[0].a.t1 = app.editor.project.seams[0].a.t0;
        app.rebuild();
        frame(&ctx, &mut app, 3000., vec![]);
        let queries = QUERIES.get();
        for _ in 0..3 {
            let output = frame(&ctx, &mut app, 3000., vec![]);
            assert!(text(&output).contains("Lengths unavailable:"));
            assert!(app.legacy_lengths.values[&seam].is_err());
        }
        assert_eq!(QUERIES.get(), queries);
        for outcome in ["failed", "cancelled", "obsolete", "accepted"] {
            let mut app = scene();
            let current = Arc::new(app.editor.project.mesh().unwrap());
            let mut old = (*current).clone();
            old.doc_revision -= 1;
            app.mesh = Some(Arc::new(old));
            let (sender, receiver) = mpsc::channel();
            app.job = Some(Job {
                id: 72,
                revision: app.editor.project.revision + u64::from(outcome == "obsolete"),
                kind: JobKind::Mesh,
                batch: None,
                completed_steps: Arc::new(AtomicU64::new(0)),
                cancel: Arc::new(AtomicBool::new(outcome == "cancelled")),
                receiver,
            });
            app.poll();
            frame(&ctx, &mut app, 3000., vec![]);
            let pending = frame(&ctx, &mut app, 3000., vec![]);
            assert!(text(&pending).contains("Lengths and guides rebuilding for this revision…"));
            sender
                .send(if outcome == "failed" {
                    Err(JobError::Failed("Original test failure".into()))
                } else {
                    Ok(JobOutput::Mesh(current))
                })
                .ok()
                .unwrap();
            app.poll();
            let output = frame(&ctx, &mut app, 3000., vec![]);
            let labels = text(&output);
            assert_eq!(
                labels.contains("Lengths and guides rebuilding for this revision…"),
                outcome == "obsolete"
            );
            if outcome == "failed" {
                assert!(labels.contains("Original test failure"));
            }
            if outcome == "cancelled" {
                assert!(labels.contains("Rest mesh job cancelled"));
            }
            assert_eq!(
                labels.matches("B − A +200.000 mm").count(),
                if outcome == "accepted" { 2 } else { 1 }
            );
        }
    }
}
