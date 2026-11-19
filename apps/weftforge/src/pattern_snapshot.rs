use eframe::egui::{self, Button, ComboBox, DragValue, Grid, Id, Modal};
use weftforge_core::PatternSnapshotOptions;

const PAPER_SIZES_MM: [(&str, [f64; 2]); 7] = [
    ("A4", [210., 297.]),
    ("A3", [297., 420.]),
    ("ISO B5", [176., 250.]),
    ("ISO B4", [250., 353.]),
    ("US Letter", [215.9, 279.4]),
    ("US Legal", [215.9, 355.6]),
    ("Tabloid", [279.4, 431.8]),
];
const ARTWORK_PADDING_MM: f64 = 2.;

pub(super) struct PatternSnapshotDialog {
    revision: u64,
    artwork_mm: [[f64; 2]; 2],
    origin_mm: [f64; 2],
    size_mm: [f64; 2],
    dpi: f64,
    contours: bool,
    internal_lines: bool,
    notches: bool,
}

pub(super) enum SnapshotAction {
    Export(PatternSnapshotOptions),
    Cancel,
}

impl PatternSnapshotDialog {
    pub fn new(revision: u64, [[u0, v0], [u1, v1]]: [[f64; 2]; 2]) -> Self {
        let artwork_mm = [
            [
                u0 * 1000. - ARTWORK_PADDING_MM,
                v0 * 1000. - ARTWORK_PADDING_MM,
            ],
            [
                u1 * 1000. + ARTWORK_PADDING_MM,
                v1 * 1000. + ARTWORK_PADDING_MM,
            ],
        ];
        let origin_mm = artwork_mm[0];
        let size_mm = std::array::from_fn(|axis| artwork_mm[1][axis] - artwork_mm[0][axis]);
        let mut dialog = Self {
            revision,
            artwork_mm,
            origin_mm,
            size_mm,
            dpi: 72.,
            contours: true,
            internal_lines: true,
            notches: true,
        };
        for dpi in [300., 150.] {
            dialog.dpi = dpi;
            if dialog.options().dimensions().is_ok() {
                return dialog;
            }
        }
        dialog.dpi = 72.;
        dialog
    }

    pub fn current(&self, revision: u64) -> bool {
        self.revision == revision
    }

    fn options(&self) -> PatternSnapshotOptions {
        PatternSnapshotOptions {
            crop_m: [
                self.origin_mm.map(|mm| mm / 1000.),
                std::array::from_fn(|axis| (self.origin_mm[axis] + self.size_mm[axis]) / 1000.),
            ],
            pixels_per_inch: self.dpi,
            contours: self.contours,
            internal_lines: self.internal_lines,
            notches: self.notches,
        }
    }

    fn fit(&mut self) {
        self.origin_mm = self.artwork_mm[0];
        self.size_mm =
            std::array::from_fn(|axis| self.artwork_mm[1][axis] - self.artwork_mm[0][axis]);
    }

    fn resize_around_center(&mut self, size_mm: [f64; 2]) {
        self.origin_mm = std::array::from_fn(|axis| {
            self.origin_mm[axis] + (self.size_mm[axis] - size_mm[axis]) / 2.
        });
        self.size_mm = size_mm;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<SnapshotAction> {
        let result = Modal::new(Id::new("pattern-snapshot-options")).show(ctx, |ui| {
            ui.heading("Save 2D pattern PNG");
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("pattern-snapshot-paper")
                    .selected_text("Paper size")
                    .show_ui(ui, |ui| {
                        for (name, size) in PAPER_SIZES_MM.iter() {
                            if ui.button(*name).clicked() {
                                self.resize_around_center(*size);
                                ui.close();
                            }
                        }
                    });
                if ui.button("Fit patterns").clicked() {
                    self.fit();
                }
                if ui.button("Swap width and height").clicked() {
                    self.resize_around_center([self.size_mm[1], self.size_mm[0]]);
                }
            });
            Grid::new("pattern-snapshot-fields")
                .num_columns(2)
                .show(ui, |ui| {
                    let [width, height] = &mut self.size_mm;
                    let [left, bottom] = &mut self.origin_mm;
                    for (name, value) in [
                        ("Width mm", width),
                        ("Height mm", height),
                        ("Left U mm", left),
                        ("Bottom V mm", bottom),
                        ("Pixels per inch", &mut self.dpi),
                    ] {
                        let label = ui.label(name);
                        ui.add(DragValue::new(value).speed(1.))
                            .labelled_by(label.id);
                        ui.end_row();
                    }
                });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.contours, "Outlines and holes");
                ui.checkbox(&mut self.internal_lines, "Internal lines");
                ui.checkbox(&mut self.notches, "Notches");
            });
            let options = self.options();
            let dimensions = options.dimensions();
            match &dimensions {
                Ok([width, height]) => ui.label(format!("{width} × {height} px")),
                Err(error) => ui.label(error.to_string()),
            };
            ui.horizontal(|ui| {
                let save = ui.add_enabled(dimensions.is_ok(), Button::new("Save PNG…"));
                if save.clicked() {
                    Some(SnapshotAction::Export(options))
                } else if ui.button("Cancel").clicked() {
                    Some(SnapshotAction::Cancel)
                } else {
                    None
                }
            })
            .inner
        });
        let closed = result.should_close();
        result.inner.or(closed.then_some(SnapshotAction::Cancel))
    }
}
