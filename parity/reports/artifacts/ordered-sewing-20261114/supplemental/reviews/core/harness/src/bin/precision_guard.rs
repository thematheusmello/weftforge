use weftforge_core::{Command, CurveInterval, Editor};

fn main() {
    let mut checked = 0;
    for cubic in [false, true] {
        for reverse in [false, true] {
            let mut editor = Editor::default();
            for (name, origin, width) in [("A", [0., 0.], 10.), ("B", [20., 0.], 1.)] {
                editor.execute(Command::AddRectangle { name: name.into(), origin, width, height: 0.1 }).unwrap();
            }
            let curve = editor.project.pieces[0].boundary[0];
            if cubic {
                editor.execute(Command::SetCurveControls {
                    curve, controls: Some([[10. / 3., 0.], [20. / 3., 0.]]),
                }).unwrap();
            }
            let a = CurveInterval {
                curve,
                t0: if reverse {0.99999999} else {0.99999997},
                t1: if reverse {0.99999997} else {0.99999999},
            };
            let b = CurveInterval { curve: editor.project.pieces[1].boundary[0], t0: 0., t1: 5e-6 };
            editor.execute(Command::AddSewingGroup {
                name: "Representable baseline".into(), a: vec![a], b: vec![b],
                correspondence: vec![[0., 0.], [1., 1.]],
            }).unwrap();
            editor.execute(Command::SetResolution { metres: 0.2 }).unwrap();
            assert!(editor.undo());
            assert!(editor.can_redo());
            let before = editor.project.clone();
            let map = vec![[0., 0.], [0.5, 0.2], [0.50000001, 0.8], [1., 1.]];
            let add = Command::AddSewingGroup {
                name: "Unrepresentable original case".into(), a: vec![a], b: vec![b],
                correspondence: map.clone(),
            };
            let mut invalid = before.clone();
            invalid.sewing_groups[0].correspondence = map;
            for command in [add, Command::UpdateSewingGroup { group: invalid.sewing_groups[0].clone() }] {
                assert!(editor.execute(command).unwrap_err().0.contains("precision budget"));
                assert_eq!(editor.project, before);
                assert!(editor.can_redo());
            }
            assert!(invalid.sewing_group_geometry(invalid.sewing_groups[0].id).unwrap_err().0.contains("precision budget"));
            assert!(invalid.to_json().unwrap_err().0.contains("precision budget"));
            assert!(Editor::new(invalid).unwrap_err().0.contains("precision budget"));
            assert!(editor.redo());
            checked += 1;
        }
    }
    println!("Passed {checked} precision-guard cases: line/uniform cubic, forward/reversed tiny end span. Safe baseline accepted; original ill-conditioned map rejected by Add/Update/Editor::new/geometry/to_json; construction and Redo retained.");
}
