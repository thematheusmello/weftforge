use eframe::egui::{self, Button, Color32, ComboBox, DragValue, Grid, Id, Modal};
use std::cell::Cell;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum DisplayUnit {
    Millimetres,
    Centimetres,
    Inches,
    Pixels,
}

impl DisplayUnit {
    fn label(self) -> &'static str {
        match self {
            Self::Millimetres => "mm",
            Self::Centimetres => "cm",
            Self::Inches => "in",
            Self::Pixels => "px",
        }
    }

    fn millimetres(self, options: PatternSnapshotOptions) -> Option<f64> {
        match self {
            Self::Millimetres => Some(1.),
            Self::Centimetres => Some(10.),
            Self::Inches => Some(25.4),
            Self::Pixels => options
                .pixels_per_metre()
                .ok()
                .map(|ppm| 1000. / f64::from(ppm)),
        }
    }
}

pub(super) struct PatternSnapshotDialog {
    revision: u64,
    artwork_mm: [[f64; 2]; 2],
    unit: DisplayUnit,
    origin_mm: [f64; 2],
    size_mm: [f64; 2],
    dpi: f64,
    contours: bool,
    internal_lines: bool,
    notches: bool,
    grainline: bool,
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
            unit: DisplayUnit::Millimetres,
            origin_mm,
            size_mm,
            dpi: 72.,
            contours: true,
            internal_lines: true,
            notches: true,
            grainline: false,
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

    pub fn crop_m(&self) -> [[f64; 2]; 2] {
        [
            self.origin_mm.map(|mm| mm / 1000.),
            std::array::from_fn(|axis| (self.origin_mm[axis] + self.size_mm[axis]) / 1000.),
        ]
    }

    fn options(&self) -> PatternSnapshotOptions {
        PatternSnapshotOptions {
            crop_m: self.crop_m(),
            pixels_per_inch: self.dpi,
            contours: self.contours,
            internal_lines: self.internal_lines,
            notches: self.notches,
            grainline: self.grainline,
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
        let previous_crop = self.crop_m();
        let id = Id::new("pattern-snapshot-options");
        let area = Modal::default_area(id).anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-8., 0.));
        let result = Modal::new(id)
            .area(area)
            .backdrop_color(Color32::from_black_alpha(48))
            .show(ctx, |ui| {
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
                ui.horizontal(|ui| {
                    let label = ui.label("Units");
                    ComboBox::from_id_salt("pattern-snapshot-units")
                        .selected_text(self.unit.label())
                        .show_ui(ui, |ui| {
                            for unit in [
                                DisplayUnit::Millimetres,
                                DisplayUnit::Centimetres,
                                DisplayUnit::Inches,
                                DisplayUnit::Pixels,
                            ] {
                                if ui
                                    .selectable_value(&mut self.unit, unit, unit.label())
                                    .clicked()
                                {
                                    ui.close();
                                }
                            }
                        })
                        .response
                        .labelled_by(label.id);
                });
                let unit = self.unit;
                let millimetres = unit.millimetres(self.options());
                Grid::new("pattern-snapshot-fields")
                    .num_columns(2)
                    .show(ui, |ui| {
                        let [width, height] = &mut self.size_mm;
                        let [left, bottom] = &mut self.origin_mm;
                        for (name, value) in [
                            ("Width", width),
                            ("Height", height),
                            ("Left U", left),
                            ("Bottom V", bottom),
                        ] {
                            let label = ui.label(format!("{name} {}", unit.label()));
                            if let Some(scale) = millimetres {
                                let initial = *value / scale;
                                let mut displayed = initial;
                                ui.push_id((unit, name, scale.to_bits()), |ui| {
                                    snapshot_value(ui, &mut displayed, 1. / scale)
                                        .labelled_by(label.id);
                                });
                                if displayed.to_bits() != initial.to_bits() {
                                    *value = displayed * scale;
                                }
                            } else {
                                ui.label("—");
                            }
                            ui.end_row();
                        }
                        let label = ui.label("Pixels per inch");
                        snapshot_value(ui, &mut self.dpi, 1.).labelled_by(label.id);
                        ui.end_row();
                    });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.contours, "Outlines and holes");
                    ui.checkbox(&mut self.internal_lines, "Internal lines");
                    ui.checkbox(&mut self.notches, "Notches");
                    ui.checkbox(&mut self.grainline, "Grainline");
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
        let action = result.inner.or(closed.then_some(SnapshotAction::Cancel));
        if action.is_some() || self.crop_m() != previous_crop {
            ctx.request_repaint();
        }
        action
    }
}

fn snapshot_value(ui: &mut egui::Ui, value: &mut f64, speed: f64) -> egui::Response {
    let id = ui.next_auto_id();
    let ctx = ui.ctx().clone();
    let number_formatter = ui.style().number_formatter.clone();
    let focused_before_edit = Cell::new(false);
    ui.add(
        DragValue::new(value)
            .speed(speed)
            .custom_formatter(|value, decimals| {
                focused_before_edit.set(ctx.memory(|memory| memory.has_focus(id)));
                number_formatter.format(value, decimals)
            })
            .custom_parser(|text| {
                // Live edits are already applied. Prevent rounded display text from replaying after blur.
                if !focused_before_edit.get() {
                    return None;
                }
                let normalized: String = text
                    .chars()
                    .filter(|character| !character.is_whitespace())
                    .map(|character| if character == '−' { '-' } else { character })
                    .collect();
                normalized
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite())
            }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focusing_and_leaving_density_keeps_physical_value() {
        let ctx = egui::Context::default();
        let mut dpi = 300.012101_f64;
        let original = dpi.to_bits();
        let mut field = None;
        let mut frame = |events| {
            let _ = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    field = Some(snapshot_value(ui, &mut dpi, 1.).id);
                    let _ = ui.button("Continue");
                },
            );
            field.unwrap()
        };
        let id = frame(Vec::new());
        ctx.memory_mut(|memory| memory.request_focus(id));
        frame(Vec::new());
        assert!(ctx.memory(|memory| memory.has_focus(id)));
        frame(vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        assert!(!ctx.memory(|memory| memory.has_focus(id)));
        frame(Vec::new());
        assert_eq!(dpi.to_bits(), original);
    }

    #[test]
    fn display_units_keep_crop_and_use_encoded_density() {
        let mut setup = PatternSnapshotDialog::new(4, [[-0.1, 0.05], [0.25, 0.2]]);
        setup.dpi = 300.012101;
        assert_eq!(setup.options().pixels_per_metre().unwrap(), 11812);
        let options = setup.options();
        for (unit, scale) in [
            (DisplayUnit::Millimetres, 1.),
            (DisplayUnit::Centimetres, 10.),
            (DisplayUnit::Inches, 25.4),
            (DisplayUnit::Pixels, 1000. / 11812.),
        ] {
            setup.unit = unit;
            assert_eq!(setup.options(), options);
            assert_eq!(setup.unit.millimetres(options), Some(scale));
        }
        setup.dpi = 0.;
        assert_eq!(DisplayUnit::Pixels.millimetres(setup.options()), None);
        assert_eq!(setup.crop_m(), options.crop_m);
    }
}
