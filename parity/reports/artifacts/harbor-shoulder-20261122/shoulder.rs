use serde_json::json;
use std::error::Error;
use weftforge_core::{Project, simulation::create_cloth};
use weftforge_sim::StepSettings;

fn main() -> Result<(), Box<dyn Error>> {
    let project = Project::load(std::env::args().nth(1).expect("supply a Harbor project"))?;
    let mesh = project.mesh()?;
    let front = mesh.triangles[6];
    let stitch = mesh
        .stitches
        .iter()
        .find(|s| s.a.vertices == [front[0]; 2])
        .unwrap();
    assert_eq!(stitch.b.vertices[0], stitch.b.vertices[1]);
    let vertex = stitch.b.vertices[0];
    let (face, back) = mesh
        .triangles
        .iter()
        .enumerate()
        .find(|(_, t)| {
            t[0] == vertex
                && mesh.uv[t[1]][1] == mesh.uv[t[2]][1]
                && (mesh.uv[t[1]][0] - mesh.uv[vertex][0]) * (mesh.uv[t[2]][0] - mesh.uv[vertex][0])
                    < 0.
        })
        .unwrap();
    let ids = [front, *back];
    println!(
        "{}",
        json!({
            "faces": [6, face], "vertices": ids, "seam": stitch.seam,
            "uv": ids.map(|t| t.map(|i| mesh.uv[i])),
            "grain": ids.map(|t| project.piece(mesh.piece_ids[t[0]]).unwrap().grain_angle)
        })
    );
    let mut cloth = create_cloth(&project, &mesh)?;
    let settings = StepSettings {
        dt: 1. / 60.,
        substeps: 8,
        iterations: 32,
        seam_closure_speed: 1.,
        gravity: [0.; 3],
    };
    for frame in 1..=29 {
        if frame == 28 {
            let mut held = cloth.clone();
            eprintln!("BRANCH hold");
            let diagnostics = held.step(
                StepSettings {
                    seam_closure_speed: 0.,
                    ..settings
                },
                || false,
            )?;
            println!(
                "{}",
                json!({"frame": frame, "branch":"hold", "diagnostics":diagnostics,
                "positions":ids.map(|t| t.map(|i| held.positions()[i]))})
            );
            eprintln!("BRANCH canonical");
        }
        let diagnostics = cloth.step(settings, || false)?;
        if frame >= 26 {
            println!(
                "{}",
                json!({"frame": frame, "branch":"canonical", "diagnostics":diagnostics,
                "positions":ids.map(|t| t.map(|i| cloth.positions()[i]))})
            );
        }
    }
    Ok(())
}
