use std::{fs, io::Write, path::Path, time::{Duration, Instant}};
use weftforge_core::Project;
use weftforge_sim::{Cloth, StepSettings};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn snapshot(file: &mut fs::File, name: &str, frame: usize, cloth: &Cloth, settings: StepSettings, elapsed: f64) -> Result<()> {
    let mass: f64 = cloth.masses().iter().sum();
    let value = serde_json::json!({
        "phase":name,"frame":frame,"settings":settings,"wall_seconds":elapsed,
        "diagnostics":cloth.diagnostics(),"positions_m":cloth.positions(),"velocities_m_per_s":cloth.velocities(),
        "total_mass_kg":mass,"rms_speed_m_per_s":(2. * cloth.diagnostics().kinetic_energy / mass).sqrt(),
        "max_speed_m_per_s":cloth.velocities().iter().map(|v|v.iter().map(|q|q*q).sum::<f64>().sqrt()).fold(0.,f64::max),
        "quality_passed":cloth.diagnostics().finite && cloth.diagnostics().seam_assembly_remaining == 0. && cloth.diagnostics().max_stretch_error < 0.15 && cloth.diagnostics().max_seam_error < 0.005 && cloth.diagnostics().max_contact_error < 0.002,
    });
    serde_json::to_writer(&mut *file, &value)?;
    writeln!(file)?;
    file.flush()?;
    eprintln!("{name} frame={frame} t={:.6} strain={:.12} KE={:.6e} wall={elapsed:.3}",cloth.simulation_time(),cloth.diagnostics().max_stretch_error,cloth.diagnostics().kinetic_energy);
    Ok(())
}

fn phase(file: &mut fs::File, name: &str, cloth: &mut Cloth, frames: usize, settings: StepSettings, cap_s: u64) -> Result<()> {
    let start = Instant::now();
    snapshot(file,name,0,cloth,settings,0.)?;
    for frame in 1..=frames {
        cloth.step(settings,||start.elapsed() >= Duration::from_secs(cap_s))?;
        if frame % 15 == 0 || frame == frames {
            snapshot(file,name,frame,cloth,settings,start.elapsed().as_secs_f64())?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(),3,"usage: diagnostic FIXTURE OUTPUT_DIRECTORY");
    let project=Project::load(&args[1])?;
    let mesh=project.mesh()?;
    let mut cloth=weftforge_core::simulation::create_cloth(&project,&mesh)?;
    fs::create_dir(&args[2])?;
    let output=Path::new(&args[2]);
    let mut meta=fs::OpenOptions::new().write(true).create_new(true).open(output.join("rest-metadata.json"))?;
    serde_json::to_writer_pretty(&mut meta,&serde_json::json!({"project":project,"rest_positions_m":mesh.positions,"uv_m":mesh.uv,"triangles":mesh.triangles,"piece_ids":mesh.piece_ids,"masses_kg":cloth.masses()}))?;
    let mut file=fs::OpenOptions::new().write(true).create_new(true).open(output.join("trajectory.jsonl"))?;
    let settings=StepSettings {dt:1./60.,substeps:8,iterations:32,seam_closure_speed:1.,gravity:[0.;3]};
    phase(&mut file,"baseline_assembly_8x32",&mut cloth,60,settings,120)?;
    let settings=StepSettings {gravity:[0.,-9.81,0.],..settings};
    phase(&mut file,"baseline_drape_8x32",&mut cloth,120,settings,120)?;
    assert!((cloth.diagnostics().max_stretch_error-0.26930346146570994).abs()<1e-12,"baseline differs from retained report");
    assert!((cloth.simulation_time()-3.).abs()<1e-12);
    let base_positions=cloth.positions().to_vec();
    for (name,substeps,iterations) in [("continuation_8x32",8,32),("iteration_8x64",8,64),("substep_16x32",16,32)] {
        let mut branch=cloth.clone();
        phase(&mut file,name,&mut branch,60,StepSettings{substeps,iterations,..settings},90)?;
        assert_eq!(cloth.positions(),base_positions,"branch mutated the original clone");
        assert!((branch.simulation_time()-4.).abs()<1e-12);
    }
    Ok(())
}
