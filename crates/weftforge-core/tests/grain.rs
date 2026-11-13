use std::f64::consts::FRAC_PI_2;
use weftforge_core::{Command, Editor, Mannequin, Project, simulation::create_cloth};
use weftforge_sim::StepSettings;

fn panel() -> Editor {
    let mut editor = Editor::new(Project::default()).unwrap();
    editor
        .execute(Command::AddRectangle {
            name: "Loaded material patch".into(),
            origin: [0., 0.003],
            width: 0.2,
            height: 0.2,
        })
        .unwrap();
    editor
        .execute(Command::SetResolution { metres: 0.5 })
        .unwrap();
    editor
        .execute(Command::SetMannequin {
            mannequin: Mannequin { capsules: vec![] },
        })
        .unwrap();
    editor
}

#[test]
fn grain_and_orthotropy_are_one_transaction_with_history_and_stale_mesh_rejection() {
    let mut editor = panel();
    let piece = editor.project.pieces[0].id;
    let rest = editor.project.mesh().unwrap();
    let old = editor.project.clone();
    let mut fabric = editor.project.materials[0].clone();
    fabric.warp_stiffness = 50.;
    fabric.weft_stiffness = 1000.;
    editor
        .execute(Command::Transaction(vec![
            Command::SetGrain {
                piece,
                angle: FRAC_PI_2,
            },
            Command::UpdateMaterial { material: fabric },
        ]))
        .unwrap();
    assert_eq!(editor.project.revision, old.revision + 1);
    assert_eq!(editor.project.pieces[0].grain_angle, FRAC_PI_2);
    assert_eq!(editor.project.materials[0].warp_stiffness, 50.);
    assert!(create_cloth(&editor.project, &rest).is_err());
    let changed = editor.project.clone();
    assert!(editor.undo());
    assert_eq!(editor.project.pieces, old.pieces);
    assert_eq!(editor.project.materials, old.materials);
    assert!(editor.redo());
    assert_eq!(editor.project.pieces, changed.pieces);
    assert_eq!(editor.project.materials, changed.materials);
    assert_eq!(editor.project.points, old.points);
    assert_eq!(editor.project.curves, old.curves);
}

#[test]
fn invalid_grain_or_directional_stiffness_retains_document_and_history() {
    let mut editor = panel();
    let piece = editor.project.pieces[0].id;
    let before = editor.project.clone();
    for angle in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(editor.execute(Command::SetGrain { piece, angle }).is_err());
        assert_eq!(editor.project, before);
    }
    for (warp, weft) in [
        (0., 400.),
        (400., -1.),
        (f64::NAN, 400.),
        (400., f64::INFINITY),
    ] {
        let mut material = before.materials[0].clone();
        material.warp_stiffness = warp;
        material.weft_stiffness = weft;
        assert!(
            editor
                .execute(Command::UpdateMaterial { material })
                .is_err()
        );
        assert_eq!(editor.project, before);
    }
    assert!(editor.undo());
    assert_eq!(editor.project.mannequin, Mannequin::default());
}

fn loaded_positions(project: &Project) -> Vec<[f64; 3]> {
    let mesh = project.mesh().unwrap();
    let mut cloth = create_cloth(project, &mesh).unwrap();
    for _ in 0..120 {
        cloth
            .step(
                StepSettings {
                    dt: 1. / 120.,
                    substeps: 2,
                    iterations: 24,
                    ..Default::default()
                },
                || false,
            )
            .unwrap();
    }
    assert!(cloth.diagnostics().finite);
    assert!(cloth.diagnostics().max_contact_error < 1e-6);
    cloth.positions().to_vec()
}

#[test]
fn authored_grain_rotates_real_cpu_response_while_color_preserves_physics() {
    let mut editor = panel();
    let mut material = editor.project.materials[0].clone();
    material.areal_density = 0.2;
    material.warp_stiffness = 50.;
    material.weft_stiffness = 1000.;
    material.damping = 8.;
    editor
        .execute(Command::UpdateMaterial { material })
        .unwrap();
    let stiff = loaded_positions(&editor.project);
    let piece = editor.project.pieces[0].id;
    editor
        .execute(Command::SetGrain {
            piece,
            angle: FRAC_PI_2,
        })
        .unwrap();
    let rotated = loaded_positions(&editor.project);
    let top_height = |positions: &[[f64; 3]]| {
        positions
            .iter()
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max)
    };
    let difference = top_height(&stiff) - top_height(&rotated);
    eprintln!(
        "ground-loaded patch: stiff top={}m rotated soft top={}m delta={}m",
        top_height(&stiff),
        top_height(&rotated),
        difference
    );
    assert!(
        difference > 0.0003,
        "rotating the weak warp axis into the load must increase compression: delta={difference} m"
    );
    let mut material = editor.project.materials[0].clone();
    material.color = [0.8, 0.2, 0.1];
    editor
        .execute(Command::UpdateMaterial { material })
        .unwrap();
    assert_eq!(loaded_positions(&editor.project), rotated);
}
