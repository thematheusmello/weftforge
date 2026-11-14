use weftforge_core::{Command, CurveInterval, Editor};

fn paired(x: f64) -> f64 {
    let pairs = [[0., 0.], [0.5, 0.2], [0.50000001, 0.8], [1., 1.]];
    for p in pairs.windows(2) {
        if x <= p[1][0] {
            return p[0][1] + (p[1][1] - p[0][1]) * (x - p[0][0]) / (p[1][0] - p[0][0]);
        }
    }
    1.
}

fn main() {
    let mut editor = Editor::default();
    for (name, origin, width) in [("A", [0., 0.], 10.), ("B", [20., 0.], 1.)] {
        editor.execute(Command::AddRectangle { name: name.into(), origin, width, height: 0.1 }).unwrap();
    }
    let a = CurveInterval { curve: editor.project.pieces[0].boundary[0], t0: 0.99999997, t1: 0.99999999 };
    let b = CurveInterval { curve: editor.project.pieces[1].boundary[0], t0: 0., t1: 5e-6 };
    editor.execute(Command::AddSewingGroup {
        name: "Conditioned tiny end interval".into(), a: vec![a], b: vec![b],
        correspondence: vec![[0., 0.], [0.5, 0.2], [0.50000001, 0.8], [1., 1.]],
    }).unwrap();
    let group = &editor.project.sewing_groups[0];
    let geometry = editor.project.sewing_group_geometry(group.id).unwrap();
    println!("Accepted group: lengths {:?}, {} portions", geometry.lengths, geometry.portions.len());
    for portion in &geometry.portions {
        for (at, bt) in [(portion.a.t0, portion.b.t0), (portion.a.t1, portion.b.t1)] {
            let u = (at - a.t0) / (a.t1 - a.t0);
            let v = (bt - b.t0) / (b.t1 - b.t0);
            let error_m = (v - paired(u)).abs() * geometry.lengths[1];
            println!("tA {at:.17e} tB {bt:.17e}, A arc fraction {u:.17e}, B arc fraction {v:.17e}, paired physical error {error_m:.17e}");
        }
    }
    let mesh = editor.project.mesh().unwrap();
    println!("Mesh accepted: {} stitches, {} vertices", mesh.stitches.len(), mesh.positions.len());
    editor.project.to_json().unwrap();
    println!("Save serialization accepted");
}
