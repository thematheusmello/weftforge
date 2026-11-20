use eframe::egui::{self, Color32};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread::JoinHandle,
    time::Duration,
};
use weftforge_core::ObjPatternImport;

pub(super) const IMPORT_OBJ: &str = "Import garment OBJ…";

pub(super) struct GarmentObjReview {
    path: PathBuf,
    unit: Option<f64>,
    state: Preview,
}

enum Preview {
    Pending,
    Loading(Receiver<Result<ObjPatternImport, String>>),
    Ready(ObjPatternImport),
    Failed(String),
}

impl GarmentObjReview {
    pub(super) fn new(path: PathBuf) -> Self {
        Self {
            path,
            unit: None,
            state: Preview::Pending,
        }
    }

    pub(super) fn ready(&self) -> Option<&ObjPatternImport> {
        match &self.state {
            Preview::Ready(drawing) => Some(drawing),
            _ => None,
        }
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, worker: &mut Option<JoinHandle<()>>) -> bool {
        ui.heading("Import garment OBJ");
        ui.label(self.path.file_name().unwrap_or_default().to_string_lossy());
        ui.label("Choose the length unit used by the OBJ vertices.");
        let previous = self.unit;
        ui.horizontal_wrapped(|ui| {
            for (unit, label) in [
                (0.001, "Millimetres"),
                (0.01, "Centimetres"),
                (1., "Metres"),
                (0.0254, "Inches"),
            ] {
                ui.radio_value(&mut self.unit, Some(unit), label);
            }
        });
        if previous != self.unit {
            self.state = Preview::Pending;
        }
        let Some(unit) = self.unit else { return false };
        ui.label(format!("1 OBJ unit = {} mm · Y up", unit * 1000.));
        // At most one bounded file/trace job can run even if a dialog closes or units change.
        if matches!(self.state, Preview::Pending)
            && worker.as_ref().is_none_or(JoinHandle::is_finished)
        {
            let (send, receive) = mpsc::channel();
            let path = self.path.clone();
            let ctx = ui.ctx().clone();
            self.state = match std::thread::Builder::new()
                .name("garment-obj-preview".into())
                .spawn(move || {
                    let result = ObjPatternImport::load(path, unit).map_err(|e| e.to_string());
                    let _ = send.send(result);
                    ctx.request_repaint();
                }) {
                Ok(task) => {
                    *worker = Some(task);
                    Preview::Loading(receive)
                }
                Err(error) => Preview::Failed(error.to_string()),
            };
        }
        if let Preview::Loading(receive) = &self.state {
            match receive.try_recv() {
                Ok(Ok(drawing)) => self.state = Preview::Ready(drawing),
                Ok(Err(error)) => self.state = Preview::Failed(error),
                Err(TryRecvError::Disconnected) => {
                    self.state = Preview::Failed("OBJ preview worker stopped.".into())
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        match &self.state {
            Preview::Pending | Preview::Loading(_) => {
                ui.label("Tracing editable patterns…");
                ui.ctx().request_repaint_after(Duration::from_millis(30));
            }
            Preview::Failed(error) => {
                ui.colored_label(Color32::LIGHT_RED, error);
            }
            Preview::Ready(drawing) => {
                let count = |n, label| format!("{n} {label}{}", if n == 1 { "" } else { "s" });
                ui.label(format!(
                    "{} · {} · {}",
                    count(drawing.piece_count(), "exterior contour"),
                    count(drawing.hole_count(), "cutout"),
                    count(drawing.material_count(), "assigned fabric")
                ));
            }
        }
        previous != self.unit
    }
}
