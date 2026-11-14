use weftforge_core::{Command, CurveInterval, Editor};

fn main() {
    let mut checked = 0;
    for reverse_a in [false, true] {
        for reverse_b in [false, true] {
            for jitter in -8_i32..=8 {
                let mut editor = Editor::default();
                let mut curves = Vec::new();
                for (i, width) in [0.1, 0.3, 0.2, 0.1, 0.3].into_iter().enumerate() {
                    editor.execute(Command::AddRectangle {
                        name: format!("Ownership panel {i}"), origin: [i as f64 * 0.5, 0.],
                        width, height: 0.12,
                    }).unwrap();
                    curves.push(editor.project.pieces.last().unwrap().boundary[0]);
                }
                let side = |slice: &[u64], reverse: bool| -> Vec<CurveInterval> {
                    let mut intervals = slice.iter().copied().map(CurveInterval::full).collect::<Vec<_>>();
                    if reverse {
                        intervals.reverse();
                        for interval in &mut intervals { std::mem::swap(&mut interval.t0, &mut interval.t1); }
                    }
                    intervals
                };
                let x = 0.25 + f64::from(jitter) * f64::EPSILON;
                let y = 1. / 3. + f64::from(jitter) * f64::EPSILON;
                editor.execute(Command::AddSewingGroup {
                    name: "One-sided ulp knots".into(),
                    a: side(&curves[..2], reverse_a), b: side(&curves[2..], reverse_b),
                    correspondence: vec![[0., 0.], [x, y], [1., 1.]],
                }).unwrap();
                let group = &editor.project.sewing_groups[0];
                let geometry = editor.project.sewing_group_geometry(group.id).unwrap();
                for (index, intervals) in [&group.a, &group.b].into_iter().enumerate() {
                    for interval in intervals {
                        let parts = geometry.portions.iter()
                            .map(|p| if index == 0 {p.a} else {p.b})
                            .filter(|part| part.curve == interval.curve).collect::<Vec<_>>();
                        assert_eq!(parts.first().unwrap().t0, interval.t0);
                        assert_eq!(parts.last().unwrap().t1, interval.t1);
                        for pair in parts.windows(2) { assert_eq!(pair[0].t1, pair[1].t0); }
                    }
                }
                let mesh = editor.project.mesh().unwrap();
                for stitch in &mesh.stitches {
                    for endpoint in [stitch.a, stitch.b] {
                        assert_eq!(mesh.piece_ids[endpoint.vertices[0]], mesh.piece_ids[endpoint.vertices[1]]);
                    }
                }
                checked += 1;
            }
        }
    }
    println!("Passed {checked} disconnected 2:3 cases with reversed sides and +/-8-ulp map knots; every source endpoint retained, consecutive portions meet, no weighted connector spans pieces.");
}
