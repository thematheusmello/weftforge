use std::time::Instant;
use weftforge_core::{Project, simulation::create_cloth};

fn main() {
    let mut project =
        Project::from_json(include_bytes!("../../../fixtures/original-shirt.weftforge")).unwrap();
    project.resolution = 0.015;
    let mesh = project.mesh().unwrap();
    println!(
        "original-shirt resolution={} vertices={} triangles={}",
        project.resolution,
        mesh.positions.len(),
        mesh.triangles.len()
    );
    for run in 1..=3 {
        let started = Instant::now();
        let cloth = create_cloth(&project, &mesh).unwrap();
        println!(
            "run={run} seconds={:.9} initial_intersections={} max_contact_error={:.17}",
            started.elapsed().as_secs_f64(),
            cloth.diagnostics().initial_intersections,
            cloth.diagnostics().max_contact_error
        );
    }
}
