use super::{Color32, Id, Mesh, Pos2, Project, egui};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Weak};

pub(super) fn fabric_color(rgb: [f32; 3]) -> Color32 {
    let [r, g, b] = rgb.map(|v| (255. * v).round() as u8);
    Color32::from_rgb(r, g, b)
}

pub(super) struct PatternFill {
    source: Weak<Mesh>,
    view: (Pos2, [f64; 2], f32),
    hidden: BTreeSet<Id>,
    pub surface: Arc<egui::Mesh>,
}

impl PatternFill {
    pub fn matches(
        &self,
        mesh: &Arc<Mesh>,
        view: (Pos2, [f64; 2], f32),
        hidden: &BTreeSet<Id>,
    ) -> bool {
        self.source.as_ptr() == Arc::as_ptr(mesh) && self.view == view && self.hidden == *hidden
    }
    pub fn new(
        mesh: &Arc<Mesh>,
        doc: &Project,
        view: (Pos2, [f64; 2], f32),
        hidden: &BTreeSet<Id>,
        screen: impl Fn([f64; 2]) -> Pos2,
    ) -> Self {
        let materials: BTreeMap<_, _> = doc
            .materials
            .iter()
            .map(|m| (m.id, fabric_color(m.color)))
            .collect();
        let colors: BTreeMap<_, _> = doc
            .pieces
            .iter()
            .map(|p| (p.id, materials[&p.material]))
            .collect();
        let mut surface = egui::Mesh::default();
        for (uv, id) in mesh.uv.iter().zip(&mesh.piece_ids) {
            surface.colored_vertex(screen(*uv), colors[id]);
        }
        for t in &mesh.triangles {
            if !hidden.contains(&mesh.piece_ids[t[0]]) {
                let [a, b, c] = t.map(|v| v as u32);
                surface.add_triangle(a, b, c);
            }
        }
        Self {
            source: Arc::downgrade(mesh),
            view,
            hidden: hidden.clone(),
            surface: Arc::new(surface),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use weftforge_core::{Command, Editor};

    #[test]
    fn fabric_fill_uses_current_uv_topology_and_leaves_the_cutout_clear() {
        let mut editor = Editor::default();
        editor
            .execute(Command::AddRectangle {
                name: "Pocket".into(),
                origin: [0., 0.],
                width: 0.4,
                height: 0.5,
            })
            .unwrap();
        let id = editor.project.pieces[0].id;
        editor
            .execute(Command::AddHole {
                piece: id,
                vertices: vec![[0.1, 0.1], [0.1, 0.3], [0.3, 0.3], [0.3, 0.1]],
            })
            .unwrap();
        let mesh = Arc::new(editor.project.mesh().unwrap());
        for color in [[0.01, 0.02, 0.03], [165. / 255., 180. / 255., 198. / 255.]] {
            let mut fabric = editor.project.materials[0].clone();
            fabric.color = color;
            editor
                .execute(Command::UpdateMaterial { material: fabric })
                .unwrap();
            let mesh = Arc::new(Mesh {
                doc_revision: editor.project.revision,
                ..(*mesh).clone()
            });
            let screen = |[x, y]: [f64; 2]| Pos2::new(x as f32 * 1000., y as f32 * 1000.);
            let view = (Pos2::ZERO, [0., 0.], 1000.);
            let fill = PatternFill::new(&mesh, &editor.project, view, &BTreeSet::new(), screen);
            assert!(
                fill.surface
                    .vertices
                    .iter()
                    .all(|v| v.color == fabric_color(color))
            );
            let covers = |uv| {
                fill.surface.indices.chunks_exact(3).any(|t| {
                    crate::point_in_triangle(
                        screen(uv),
                        [t[0], t[1], t[2]].map(|j| fill.surface.vertices[j as usize].pos),
                    )
                })
            };
            assert!(!covers([0.2, 0.2]) && covers([0.099, 0.2]) && !covers([0.101, 0.2]));
            assert!(fill.matches(&mesh, view, &BTreeSet::new()));
            assert!(!fill.matches(&mesh, view, &BTreeSet::from([id])));
            assert!(
                PatternFill::new(&mesh, &editor.project, view, &BTreeSet::from([id]), screen)
                    .surface
                    .indices
                    .is_empty()
            );
        }
    }
}
