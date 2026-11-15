use serde_json::{Value, json};
use std::{fs, io::Write, path::Path, time::Instant};
use weftforge_core::{Mesh, Project};
use weftforge_sim::{Cloth, StepSettings};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn create(path: &Path) -> std::io::Result<fs::File> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
}

fn write_json(file: &mut fs::File, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *file, value)?;
    writeln!(file)?;
    file.flush()?;
    Ok(())
}

fn metadata(project: &Project, mesh: &Mesh) -> Value {
    let stitches: Vec<_> = mesh.stitches.iter().map(|s| json!({"seam":s.seam,"a":{"vertices":s.a.vertices,"weight":s.a.weight},"b":{"vertices":s.b.vertices,"weight":s.b.weight},"compliance_m_per_N":1e-8})).collect();
    let boundary: Vec<_> = mesh
        .boundary
        .iter()
        .map(|s| json!({"piece":s.piece,"curve":s.curve,"t":s.t,"vertex":s.vertex}))
        .collect();
    let triangle_materials: Vec<_> = mesh.triangles.iter().map(|t| {
        let p = project.piece(mesh.piece_ids[t[0]]).unwrap();
        let m = project.material(p.material).unwrap();
        json!({"piece_id":p.id,"material_id":m.id,"areal_density":m.areal_density,"warp_stiffness":m.warp_stiffness,"weft_stiffness":m.weft_stiffness,"shear_stiffness":m.shear_stiffness,"bending_stiffness":m.bending_stiffness,"grain_angle":p.grain_angle,"damping":m.damping,"friction":m.friction,"collision_thickness":m.collision_thickness})
    }).collect();
    json!({"project":project,"doc_revision":mesh.doc_revision,"rest_positions_m":mesh.positions,"uv_m":mesh.uv,"triangles":mesh.triangles,"piece_ids":mesh.piece_ids,"boundary":boundary,"stitches":stitches,"triangle_materials":triangle_materials,"pins":[],"ground_height_m":0.,"all_contact_enabled":true})
}

fn snapshot(cloth: &Cloth, frame: usize, settings: StepSettings, wall: f64) -> Value {
    json!({"global_frame":frame,"settings_last_step":settings,"phase":if frame<=60 {"assembly"} else {"drape"},"producer_wall_seconds":wall,"diagnostics":cloth.diagnostics(),"positions_m":cloth.positions(),"velocities_m_per_s":cloth.velocities()})
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        4,
        "usage: harbor-public-driver mesh|run INPUT OUTPUT_DIRECTORY"
    );
    assert!(args[1] == "mesh" || args[1] == "run");
    let start = Instant::now();
    let project = Project::load(&args[2])?;
    let mesh = project.mesh_cancellable(|| start.elapsed().as_secs_f64() >= 420.)?;
    let output = Path::new(&args[3]);
    fs::create_dir(output)?;
    let mut meta = metadata(&project, &mesh);
    if args[1] == "mesh" {
        serde_json::to_writer_pretty(create(&output.join("mesh-metadata.json"))?, &meta)?;
        mesh.export_obj(output.join("rest.obj"))?;
        println!(
            "geometry_only=true vertices={} triangles={} boundary={} stitches={}",
            mesh.positions.len(),
            mesh.triangles.len(),
            mesh.boundary.len(),
            mesh.stitches.len()
        );
        return Ok(());
    }
    let mut cloth = weftforge_core::simulation::create_cloth_cancellable(&project, &mesh, || {
        start.elapsed().as_secs_f64() >= 420.
    })?;
    assert_eq!(cloth.positions(), mesh.positions);
    assert!(cloth.velocities().iter().flatten().all(|v| *v == 0.));
    assert_eq!(cloth.simulation_time(), 0.);
    meta["masses_kg"] = json!(cloth.masses());
    serde_json::to_writer_pretty(create(&output.join("rest-metadata.json"))?, &meta)?;
    mesh.export_obj(output.join("stage-000.obj"))?;
    let mut states = create(&output.join("trajectory.jsonl"))?;
    let mut steps = create(&output.join("steps.jsonl"))?;
    let settings = StepSettings {
        dt: 1. / 60.,
        substeps: 8,
        iterations: 32,
        seam_closure_speed: 1.,
        gravity: [0.; 3],
    };
    write_json(
        &mut states,
        &snapshot(&cloth, 0, settings, start.elapsed().as_secs_f64()),
    )?;
    let mut last_settings = settings;
    for frame in 1..=180 {
        let settings = StepSettings {
            gravity: if frame <= 60 {
                [0.; 3]
            } else {
                [0., -9.81, 0.]
            },
            ..settings
        };
        let step_start = Instant::now();
        if let Err(error) = cloth.step(settings, || start.elapsed().as_secs_f64() >= 420.) {
            write_json(
                &mut create(&output.join("failed-step.json"))?,
                &json!({"attempted_frame":frame,"last_retained_frame":frame-1,"settings":settings,"error":error.to_string(),"producer_wall_seconds":start.elapsed().as_secs_f64(),"retained_state":snapshot(&cloth,frame-1,last_settings,start.elapsed().as_secs_f64())}),
            )?;
            return Err(error.into());
        }
        last_settings = settings;
        write_json(
            &mut steps,
            &json!({"global_frame":frame,"settings":settings,"measured_step_seconds":step_start.elapsed().as_secs_f64(),"diagnostics":cloth.diagnostics()}),
        )?;
        write_json(
            &mut states,
            &snapshot(&cloth, frame, settings, start.elapsed().as_secs_f64()),
        )?;
        if [60, 180].contains(&frame) {
            let mut display = mesh.clone();
            display.positions = cloth.positions().to_vec();
            display.export_obj(output.join(format!("stage-{frame:03}.obj")))?;
        }
        if frame.is_multiple_of(15) {
            eprintln!(
                "frame={frame} strain={} seam={} contact={} wall={:.3}",
                cloth.diagnostics().max_stretch_error,
                cloth.diagnostics().max_seam_error,
                cloth.diagnostics().max_contact_error,
                start.elapsed().as_secs_f64()
            );
        }
    }
    let d = cloth.diagnostics();
    serde_json::to_writer_pretty(
        create(&output.join("completed-case.json"))?,
        &json!({"completed_frames":180,"producer_wall_seconds":start.elapsed().as_secs_f64(),"final_diagnostics":d,"final_quality_passed":d.finite && d.seam_assembly_remaining==0. && d.max_stretch_error<0.15 && d.max_seam_error<0.005 && d.max_contact_error<0.002 && d.capsule_surface_uncertain_pairs==0,"equilibrium_claim":false,"reference_parity":"not measured"}),
    )?;
    Ok(())
}
