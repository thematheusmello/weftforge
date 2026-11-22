use super::{Color32, Id, Mesh, Pos2, Rect, Vec2, egui};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Weak};
use weftforge_core::simulation::maximum_principal_stretch;
use weftforge_sim::Cloth;

pub(super) const STRAIN_MAP: &str = "Strain map";
const RAMP: [[u8; 3]; 4] = [
    [45, 105, 163],
    [63, 171, 153],
    [238, 193, 92],
    [215, 77, 62],
];

pub(super) struct StrainMap {
    pub enabled: bool,
    upper_percent: u16,
    snapshot: Option<Snapshot>,
}

struct Snapshot {
    mesh: Weak<Mesh>,
    cloth: Option<Weak<Cloth>>,
    recorded: Option<(u64, usize)>,
    field: Option<Field>,
}

struct Field {
    faces: Vec<f64>,
    pieces: BTreeMap<Id, (usize, f64)>,
}

impl Field {
    fn from_surface(mesh: &Mesh, positions: &[[f64; 3]]) -> Option<Self> {
        if mesh.uv.len() != positions.len() || mesh.piece_ids.len() != positions.len() {
            return None;
        }
        let mut faces = Vec::with_capacity(mesh.triangles.len());
        let mut pieces = BTreeMap::<Id, (usize, f64)>::new();
        for &[a, b, c] in &mesh.triangles {
            let stretch = maximum_principal_stretch(
                [*mesh.uv.get(a)?, *mesh.uv.get(b)?, *mesh.uv.get(c)?],
                [positions[a], positions[b], positions[c]],
            )?;
            faces.push(stretch);
            let (count, peak) = pieces.entry(mesh.piece_ids[a]).or_default();
            *count += 1;
            *peak = peak.max(stretch);
        }
        Some(Self { faces, pieces })
    }
}

impl Default for StrainMap {
    fn default() -> Self {
        Self {
            enabled: false,
            upper_percent: 120,
            snapshot: None,
        }
    }
}

impl StrainMap {
    pub fn refresh(
        &mut self,
        revision: u64,
        mesh: Option<&Arc<Mesh>>,
        cloth: Option<&Arc<Cloth>>,
        frame: Option<((u64, usize), &super::timeline::Frame)>,
    ) {
        let Some(mesh) = mesh.filter(|m| self.enabled && m.doc_revision == revision) else {
            self.snapshot = None;
            return;
        };
        if self.snapshot.as_ref().is_some_and(|old| {
            old.mesh.as_ptr() == Arc::as_ptr(mesh)
                && old.recorded == frame.map(|(key, _)| key)
                && old.cloth.as_ref().map(Weak::as_ptr) == cloth.map(Arc::as_ptr)
        }) {
            return;
        }
        let positions = frame.map_or_else(
            || cloth.map_or(mesh.positions.as_slice(), |c| c.positions()),
            |(_, frame)| frame.positions.as_ref(),
        );
        self.snapshot = Some(Snapshot {
            mesh: Arc::downgrade(mesh),
            cloth: cloth.map(Arc::downgrade),
            recorded: frame.map(|(key, _)| key),
            field: Field::from_surface(mesh, positions),
        });
    }

    pub fn face_color(&self, face: usize) -> Option<Color32> {
        self.enabled.then(|| {
            self.snapshot
                .as_ref()
                .and_then(|s| s.field.as_ref())
                .and_then(|f| f.faces.get(face))
                .map_or(Color32::from_gray(110), |&stretch| {
                    ramp_color(display_level(stretch, self.upper_percent))
                })
        })
    }

    pub fn panel(&mut self, ui: &egui::Ui, viewport: Rect, hidden: &BTreeSet<Id>) {
        if !self.enabled {
            return;
        }
        egui::Window::new("Garment strain map")
            .open(&mut self.enabled)
            .enabled(ui.is_enabled())
            .default_pos(viewport.left_top() + Vec2::new(10., 32.))
            .default_width(218.)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.strong("Global · maximum principal stretch");
                ui.small("100% = rest length");
                ui.horizontal(|ui| {
                    let label = ui.label("Upper range");
                    ui.add(
                        egui::DragValue::new(&mut self.upper_percent)
                            .range(100..=1000)
                            .speed(1.)
                            .suffix("%"),
                    )
                    .labelled_by(label.id);
                });
                let (r, _) = ui
                    .allocate_exact_size(Vec2::new(ui.available_width(), 8.), egui::Sense::hover());
                let mut gradient = egui::Mesh::default();
                for (i, rgb) in RAMP.iter().enumerate() {
                    let x = egui::lerp(r.x_range(), i as f32 / (RAMP.len() - 1) as f32);
                    let color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                    gradient.colored_vertex(Pos2::new(x, r.top()), color);
                    gradient.colored_vertex(Pos2::new(x, r.bottom()), color);
                    if i > 0 {
                        let k = 2 * i as u32;
                        gradient.add_triangle(k - 2, k - 1, k);
                        gradient.add_triangle(k, k - 1, k + 1);
                    }
                }
                if self.upper_percent == 100 {
                    ui.painter().rect_filled(r, 0., ramp_color(0.));
                    ui.painter().rect_filled(
                        Rect::from_min_max(r.center_top(), r.right_bottom()),
                        0.,
                        ramp_color(1.),
                    );
                } else {
                    ui.painter().add(gradient);
                }
                ui.horizontal(|ui| {
                    ui.small("≤100%");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.small(if self.upper_percent == 100 {
                            ">100%".to_string()
                        } else {
                            format!("≥{}%", self.upper_percent)
                        });
                    });
                });
                match self.snapshot.as_ref().and_then(|s| s.field.as_ref()) {
                    None => {
                        ui.weak("Current surface unavailable");
                    }
                    Some(field) => {
                        let (count, peak) = field
                            .pieces
                            .iter()
                            .filter(|(id, _)| !hidden.contains(id))
                            .fold((0, 0_f64), |(n, p), (_, (count, v))| (n + count, p.max(*v)));
                        if count == 0 {
                            ui.weak("No visible faces");
                        } else {
                            ui.label(format!("Visible peak {:.2}% · {count} faces", 100. * peak));
                        }
                    }
                }
            });
    }
}

fn display_level(stretch: f64, upper: u16) -> f64 {
    if upper == 100 {
        return if stretch <= 1. + 1e-12 { 0. } else { 1. };
    }
    ((stretch - 1.) * 100. / f64::from(upper - 100)).clamp(0., 1.)
}

fn ramp_color(level: f64) -> Color32 {
    let x = level * (RAMP.len() - 1) as f64;
    let i = (x.floor() as usize).min(RAMP.len() - 2);
    let [r, g, b] = std::array::from_fn(|k| {
        let a = f64::from(RAMP[i][k]);
        let b = f64::from(RAMP[i + 1][k]);
        (a + (b - a) * (x - i as f64)).round() as u8
    });
    Color32::from_rgb(r, g, b)
}

pub(super) fn edge_color(face: Color32) -> Color32 {
    let luma =
        0.2126 * f32::from(face.r()) + 0.7152 * f32::from(face.g()) + 0.0722 * f32::from(face.b());
    if luma < 130. {
        Color32::from_gray(246)
    } else {
        Color32::from_gray(25)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Studio, inspector_click,
        native_idle_numeric_tests::{action, frame},
    };
    use egui::accesskit::{Action, ActionData};
    use weftforge_core::{Command, Editor};

    #[test]
    fn global_strain_range_and_geometry_preserve_compression_shear_visibility_and_cache_identity() {
        let mut mesh = Mesh {
            doc_revision: 17,
            uv: vec![[0., 0.], [1., 0.], [0., 1.], [0., 0.], [1., 0.], [0., 1.]],
            piece_ids: vec![2, 2, 2, 8, 8, 8],
            triangles: vec![[0, 1, 2], [3, 4, 5]],
            ..Default::default()
        };
        mesh.positions = vec![
            [0., 0., 0.],
            [0.8, 0., 0.],
            [0., 0.9, 0.],
            [1., 2., 1.],
            [2., 2., 1.],
            [2., 3., 1.],
        ];
        let mesh = Arc::new(mesh);
        let mut map = StrainMap::default();
        map.refresh(17, Some(&mesh), None, None);
        assert!(map.snapshot.is_none());
        map.enabled = true;
        map.refresh(17, Some(&mesh), None, None);
        let field = map.snapshot.as_ref().unwrap().field.as_ref().unwrap();
        assert_eq!(field.pieces.len(), 2);
        assert!((field.faces[0] - 0.9).abs() < 1e-14);
        assert!((field.faces[1] - (1. + 5_f64.sqrt()) / 2.).abs() < 1e-14);
        assert_eq!(field.pieces[&8], (1, field.faces[1]));
        let values = field.faces.as_ptr();
        for (upper, level, expected) in [
            (120, 1., 0.),
            (120, 1.10, 0.5),
            (125, 1.25, 1.),
            (1000, 5.5, 0.5),
            (100, 0.8, 0.),
            (100, 1., 0.),
            (100, 1. + 1e-13, 0.),
            (100, 1.0001, 1.),
        ] {
            map.upper_percent = upper;
            assert!((display_level(level, upper) - expected).abs() < 1e-12);
            map.refresh(17, Some(&mesh), None, None);
            assert_eq!(
                map.snapshot
                    .as_ref()
                    .unwrap()
                    .field
                    .as_ref()
                    .unwrap()
                    .faces
                    .as_ptr(),
                values
            );
        }
        assert_eq!(map.face_color(0), Some(ramp_color(0.)));
        assert_eq!(map.face_color(1), Some(ramp_color(1.)));
        map.refresh(18, Some(&mesh), None, None);
        assert!(map.snapshot.is_none());
        assert_eq!(map.face_color(0), Some(Color32::from_gray(110)));
        let invalid = Arc::new(Mesh {
            triangles: vec![[1, 2, 99]],
            ..(*mesh).clone()
        });
        map.refresh(17, Some(&invalid), None, None);
        assert!(map.snapshot.as_ref().unwrap().field.is_none());
        for c in [Color32::BLACK, Color32::from_rgb(15, 28, 43)] {
            assert_eq!(edge_color(c), Color32::from_gray(246));
        }
        for c in [Color32::WHITE, Color32::from_rgb(165, 180, 198)] {
            assert_eq!(edge_color(c), Color32::from_gray(25));
        }
    }

    #[test]
    fn global_strain_view_search_accepted_surface_and_material_invalidation_preserve_authoring() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = Studio::empty();
        app.session_autosave = None;
        let initial = frame(&ctx, &mut app, 1000., vec![]);
        frame(
            &ctx,
            &mut app,
            1000.,
            vec![inspector_click(&initial, "View")],
        );
        let menu = frame(&ctx, &mut app, 1000., vec![]);
        frame(
            &ctx,
            &mut app,
            1000.,
            vec![inspector_click(&menu, STRAIN_MAP)],
        );
        assert!(app.strain_map.enabled && app.strain_map.snapshot.is_none());
        let render = |ctx: &egui::Context, app: &mut Studio| frame(ctx, app, 1000., vec![]);
        let out = render(&ctx, &mut app);
        assert!(
            out.platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .any(|(_, n)| n.value() == Some("Current surface unavailable"))
        );
        for (name, x) in [("Face", 0.), ("Lining", 0.4)] {
            app.editor
                .execute(Command::AddRectangle {
                    name: name.into(),
                    origin: [x, 0.],
                    width: 0.2,
                    height: 0.3,
                })
                .unwrap();
        }
        app.editor = Editor::new(app.editor.project.clone()).unwrap();
        let before = app.editor.project.clone();
        let mesh = Arc::new(before.mesh().unwrap());
        app.mesh = Some(mesh.clone());
        app.needs_mesh = false;
        let first = before.pieces[0].id;
        app.selected = Some(first);
        let initial = render(&ctx, &mut app);
        let co = app
            .strain_map
            .snapshot
            .as_ref()
            .unwrap()
            .field
            .as_ref()
            .unwrap()
            .faces
            .as_ptr();
        let fill = app.pattern_fill.as_ref().unwrap().surface.clone();
        let range = initial
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .iter()
            .find(|(_, n)| n.numeric_value() == Some(120.) && n.min_numeric_value() == Some(100.))
            .unwrap()
            .0;
        frame(
            &ctx,
            &mut app,
            1000.,
            vec![action(
                range,
                Action::SetValue,
                Some(ActionData::NumericValue(125.)),
            )],
        );
        assert_eq!(app.strain_map.upper_percent, 125);
        assert_eq!(
            app.strain_map
                .snapshot
                .as_ref()
                .unwrap()
                .field
                .as_ref()
                .unwrap()
                .faces
                .as_ptr(),
            co
        );
        assert!(Arc::ptr_eq(
            &app.pattern_fill.as_ref().unwrap().surface,
            &fill
        ));
        let mut state = weftforge_core::simulation::create_cloth(&before, &mesh).unwrap();
        state.step(app.settings, || false).unwrap();
        app.cloth = Some(Arc::new(state));
        app.hidden_pieces.insert(first);
        let out = render(&ctx, &mut app);
        let accepted = app.strain_map.snapshot.as_ref().unwrap();
        assert_eq!(
            accepted.cloth.as_ref().unwrap().as_ptr(),
            Arc::as_ptr(app.cloth.as_ref().unwrap())
        );
        let visible_count = accepted.field.as_ref().unwrap().pieces[&before.pieces[1].id].0;
        assert!(
            out.platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .any(|(_, n)| n
                    .value()
                    .is_some_and(|v| v.contains(&format!("· {visible_count} faces"))))
        );
        app.open_command_search(&ctx);
        app.query = STRAIN_MAP.into();
        let out = render(&ctx, &mut app);
        let toggle = crate::tests::command_search_action(&out, STRAIN_MAP);
        frame(
            &ctx,
            &mut app,
            1000.,
            vec![action(toggle, Action::Click, None)],
        );
        render(&ctx, &mut app);
        assert!(!app.strain_map.enabled && app.strain_map.snapshot.is_none());
        assert_eq!(app.editor.project, before);
        assert!(!app.dirty && !app.editor.can_undo() && !app.editor.can_redo());
        app.strain_map.enabled = true;
        let mut material = before.materials[0].clone();
        material.areal_density *= 1.1;
        assert!(app.apply_command(Command::UpdateMaterial { material }));
        app.needs_mesh = false;
        render(&ctx, &mut app);
        assert!(app.strain_map.snapshot.is_none() && app.pattern_fill.is_none());
    }
}
