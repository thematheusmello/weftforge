use std::{
    fs,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};
use weftforge_core::Project;
use weftforge_sim::{Cloth, StepSettings};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn write_json(file: &mut fs::File, value: &serde_json::Value) -> Result<()> {
    serde_json::to_writer(&mut *file, value)?;
    writeln!(file)?;
    file.flush()?;
    Ok(())
}
fn snapshot(
    file: &mut fs::File,
    name: &str,
    frame: usize,
    cloth: &Cloth,
    settings: StepSettings,
    elapsed: f64,
) -> Result<()> {
    write_json(
        file,
        &serde_json::json!({"case":name,"global_frame":frame,"settings_last_step":settings,"case_wall_seconds":elapsed,"phase":if frame<=60 {"assembly"} else {"zero_g_balance"},"diagnostics":cloth.diagnostics(),"positions_m":cloth.positions(),"velocities_m_per_s":cloth.velocities()}),
    )?;
    if frame.is_multiple_of(15) {
        eprintln!(
            "{name} frame={frame} t={:.6} strain={:.12} KE={:.6e} wall={elapsed:.3}",
            cloth.simulation_time(),
            cloth.diagnostics().max_stretch_error,
            cloth.diagnostics().kinetic_energy
        );
    }
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        4,
        "usage: cloth-balance-probe FIXTURE OUTPUT_DIRECTORY zero-g-balance|no-avatar"
    );
    let mut project = Project::load(&args[1])?;
    let original_project = project.clone();
    let name = args[3].as_str();
    assert!(
        name == "zero-g-balance" || name == "no-avatar",
        "unknown case"
    );
    if name == "no-avatar" {
        project.mannequin.capsules.clear();
    }
    let frames = if name == "zero-g-balance" { 180 } else { 60 };
    let mesh = project.mesh()?;
    assert_eq!(mesh, original_project.mesh()?);
    fs::create_dir(&args[2])?;
    let output = Path::new(&args[2]);
    let create = |name: &str| {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(name))
    };
    let initialization = Instant::now();
    let mut cloth = weftforge_core::simulation::create_cloth(&project, &mesh)?;
    let initialization_seconds = initialization.elapsed().as_secs_f64();
    assert_eq!(cloth.positions(), mesh.positions);
    assert!(cloth.velocities().iter().flatten().all(|q| *q == 0.));
    assert_eq!(cloth.simulation_time(), 0.);
    let stitches: Vec<_> = mesh.stitches.iter().map(|s|serde_json::json!({"seam":s.seam,"a":{"vertices":s.a.vertices,"weight":s.a.weight},"b":{"vertices":s.b.vertices,"weight":s.b.weight},"compliance_m_per_N":1e-8})).collect();
    let boundary: Vec<_> = mesh
        .boundary
        .iter()
        .map(|s| serde_json::json!({"piece":s.piece,"curve":s.curve,"t":s.t,"vertex":s.vertex}))
        .collect();
    serde_json::to_writer_pretty(
        create("rest-metadata.json")?,
        &serde_json::json!({"case":name,"original_project":original_project,"project":project,"rest_positions_m":mesh.positions,"uv_m":mesh.uv,"triangles":mesh.triangles,"piece_ids":mesh.piece_ids,"stitches":stitches,"boundary":boundary,"masses_kg":cloth.masses()}),
    )?;
    let mut trajectory = create("trajectory.jsonl")?;
    let mut timings = create("steps.jsonl")?;
    let settings = StepSettings {
        dt: 1. / 60.,
        substeps: 8,
        iterations: 32,
        seam_closure_speed: 1.,
        gravity: [0.; 3],
    };
    let start = Instant::now();
    snapshot(&mut trajectory, name, 0, &cloth, settings, 0.)?;
    let mut phase_start = Instant::now();
    let mut phase_seconds = Vec::new();
    for frame in 1..=frames {
        if frame == 61 {
            phase_seconds.push(phase_start.elapsed().as_secs_f64());
            phase_start = Instant::now();
        }
        let settings = StepSettings {
            gravity: [0.; 3],
            ..settings
        };
        let step_start = Instant::now();
        cloth.step(settings, || {
            phase_start.elapsed() >= Duration::from_secs(180)
        })?;
        write_json(
            &mut timings,
            &serde_json::json!({"global_frame":frame,"settings":settings,"measured_step_seconds":step_start.elapsed().as_secs_f64(),"diagnostics":cloth.diagnostics()}),
        )?;
        snapshot(
            &mut trajectory,
            name,
            frame,
            &cloth,
            settings,
            start.elapsed().as_secs_f64(),
        )?;
    }
    phase_seconds.push(phase_start.elapsed().as_secs_f64());
    assert!((cloth.simulation_time() - f64::from(frames as u32) / 60.).abs() < 1e-12);
    serde_json::to_writer_pretty(
        create("completed-case.json")?,
        &serde_json::json!({"completed_frames":frames,"initialization_wall_seconds":initialization_seconds,"assembly_wall_seconds":phase_seconds[0],"zero_g_balance_wall_seconds":phase_seconds.get(1),"case_wall_seconds":start.elapsed().as_secs_f64(),"final_diagnostics":cloth.diagnostics()}),
    )?;
    Ok(())
}
