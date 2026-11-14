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
    let mass: f64 = cloth.masses().iter().sum();
    write_json(
        file,
        &serde_json::json!({
            "case":name,"global_frame":frame,"settings_last_step":settings,"case_wall_seconds":elapsed,
            "phase":if frame <= 60 {"assembly"} else {"drape"},
            "diagnostics":cloth.diagnostics(),"positions_m":cloth.positions(),"velocities_m_per_s":cloth.velocities(),
            "total_mass_kg":mass,"rms_speed_m_per_s":(2. * cloth.diagnostics().kinetic_energy / mass).sqrt(),
            "max_speed_m_per_s":cloth.velocities().iter().map(|v|v.iter().map(|q|q*q).sum::<f64>().sqrt()).fold(0.,f64::max),
        }),
    )?;
    eprintln!(
        "{name} frame={frame} t={:.6} strain={:.12} KE={:.6e} wall={elapsed:.3}",
        cloth.simulation_time(),
        cloth.diagnostics().max_stretch_error,
        cloth.diagnostics().kinetic_energy
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "usage: diagnostic FIXTURE OUTPUT_DIRECTORY");
    let project = Project::load(&args[1])?;
    let mesh = project.mesh()?;
    fs::create_dir(&args[2])?;
    let output = Path::new(&args[2]);
    let create = |name: &str| {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(name))
    };
    let mut trajectory = create("trajectory.jsonl")?;
    let mut timings = create("steps.jsonl")?;
    let mut cases = Vec::new();
    for (name, substeps, iterations) in [
        ("control_8x32", 8, 32),
        ("iteration_8x64", 8, 64),
        ("substep_16x32", 16, 32),
    ] {
        let initialization = Instant::now();
        let mut cloth = weftforge_core::simulation::create_cloth(&project, &mesh)?;
        let initialization_seconds = initialization.elapsed().as_secs_f64();
        assert_eq!(cloth.positions(), mesh.positions);
        assert!(cloth.velocities().iter().flatten().all(|q| *q == 0.));
        assert_eq!(cloth.simulation_time(), 0.);
        if name == "control_8x32" {
            let stitches: Vec<_> = mesh.stitches.iter().map(|s| serde_json::json!({"seam":s.seam,"a":{"vertices":s.a.vertices,"weight":s.a.weight},"b":{"vertices":s.b.vertices,"weight":s.b.weight}})).collect();
            serde_json::to_writer_pretty(
                create("rest-metadata.json")?,
                &serde_json::json!({"project":project,"rest_positions_m":mesh.positions,"uv_m":mesh.uv,"triangles":mesh.triangles,"piece_ids":mesh.piece_ids,"stitches":stitches,"masses_kg":cloth.masses()}),
            )?;
        }
        let settings = StepSettings {
            dt: 1. / 60.,
            substeps,
            iterations,
            seam_closure_speed: 1.,
            gravity: [0.; 3],
        };
        let start = Instant::now();
        snapshot(&mut trajectory, name, 0, &cloth, settings, 0.)?;
        let mut phase_seconds = Vec::new();
        let mut phase_start = Instant::now();
        for frame in 1..=180 {
            if frame == 61 {
                phase_seconds.push(phase_start.elapsed().as_secs_f64());
                phase_start = Instant::now();
            }
            let settings = StepSettings {
                gravity: if frame <= 60 {
                    [0.; 3]
                } else {
                    [0., -9.81, 0.]
                },
                ..settings
            };
            let step_start = Instant::now();
            cloth.step(settings, || {
                phase_start.elapsed() >= Duration::from_secs(120)
            })?;
            let step_seconds = step_start.elapsed().as_secs_f64();
            write_json(
                &mut timings,
                &serde_json::json!({"case":name,"global_frame":frame,"phase":if frame<=60 {"assembly"} else {"drape"},"settings":settings,"measured_step_seconds":step_seconds,"diagnostics":cloth.diagnostics()}),
            )?;
            if frame % 15 == 0 {
                snapshot(
                    &mut trajectory,
                    name,
                    frame,
                    &cloth,
                    settings,
                    start.elapsed().as_secs_f64(),
                )?;
            }
        }
        phase_seconds.push(phase_start.elapsed().as_secs_f64());
        assert!((cloth.simulation_time() - 3.).abs() < 1e-12);
        cases.push(serde_json::json!({"case":name,"completed_frames":180,"initialization_wall_seconds":initialization_seconds,"assembly_wall_seconds":phase_seconds[0],"drape_wall_seconds":phase_seconds[1],"case_wall_seconds":start.elapsed().as_secs_f64(),"final_diagnostics":cloth.diagnostics()}));
    }
    serde_json::to_writer_pretty(create("completed-cases.json")?, &cases)?;
    Ok(())
}
