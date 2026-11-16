use weftforge_core::{Command, CurveInterval, Editor, Placement};

#[test]
fn original_shirt_builds_conforming_positive_area_cloth_at_multiple_resolutions() {
    let outline = vec![
        [-0.24, 1.02],
        [0.24, 1.02],
        [0.24, 1.27],
        [0.48, 1.20],
        [0.53, 1.38],
        [0.24, 1.56],
        [0.10, 1.56],
        [0.08, 1.49],
        [-0.08, 1.49],
        [-0.10, 1.56],
        [-0.24, 1.56],
        [-0.53, 1.38],
        [-0.48, 1.20],
        [-0.24, 1.27],
    ];
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Shirt front".into(),
            vertices: outline.clone(),
        })
        .unwrap();
    editor
        .execute(Command::AddPolygon {
            name: "Shirt back".into(),
            vertices: outline.iter().map(|p| [p[0] + 1.2, p[1]]).collect(),
        })
        .unwrap();
    let front = editor.project.pieces[0].clone();
    let back = editor.project.pieces[1].clone();
    editor
        .execute(Command::SetPlacement {
            piece: front.id,
            placement: Placement {
                translation: [0.0, 0.0, 0.22],
                rotation: [0.0; 3],
            },
        })
        .unwrap();
    editor
        .execute(Command::SetPlacement {
            piece: back.id,
            placement: Placement {
                translation: [-1.2, 0.0, -0.22],
                rotation: [0.0; 3],
            },
        })
        .unwrap();
    for index in [1, 2, 4, 5, 9, 10, 12, 13] {
        editor
            .execute(Command::AddSeam {
                name: format!("Construction seam {index}"),
                a: CurveInterval::full(front.boundary[index]),
                b: CurveInterval::full(back.boundary[index]),
                correspondence: vec![[0.0, 0.0], [1.0, 1.0]],
            })
            .unwrap();
    }
    let polygon_area: f64 = outline
        .iter()
        .zip(outline.iter().cycle().skip(1))
        .take(outline.len())
        .map(|(a, b)| a[0] * b[1] - a[1] * b[0])
        .sum::<f64>()
        .abs()
        * 0.5;
    for resolution in [0.12, 0.07, 0.035] {
        editor
            .execute(Command::SetResolution { metres: resolution })
            .unwrap();
        let mesh = editor.project.mesh().unwrap();
        mesh.validate().unwrap();
        let mesh_area: f64 = mesh
            .triangles
            .iter()
            .map(|triangle| {
                let [a, b, c] = triangle.map(|v| mesh.uv[v]);
                ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() * 0.5
            })
            .sum();
        assert!((mesh_area - polygon_area * 2.0).abs() < 1.0e-10);
        let cloth = weftforge_core::simulation::create_cloth(&editor.project, &mesh).unwrap();
        assert_eq!(cloth.positions().len(), mesh.positions.len());
        assert!(cloth.masses().iter().all(|m| m.is_finite() && *m > 0.0));
    }
}
