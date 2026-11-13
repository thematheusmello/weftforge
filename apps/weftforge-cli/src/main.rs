use std::{
    error::Error,
    path::{Path, PathBuf},
    time::Instant,
};
use weftforge_core::{Command, CurveInterval, Editor, Mesh, Placement, Project};
use weftforge_sim::{Diagnostics, StepSettings};
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("weftforge-cli: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("--help" | "-h" | "help") => println!(
            "WeftForge CLI — offline editable garment tools\n\nnew PROJECT\nsample PROJECT                         original two-panel shirt (implementation fixture)\ninspect PROJECT\nrectangle INPUT OUTPUT NAME X Y WIDTH HEIGHT\nresize INPUT OUTPUT PIECE_ID WIDTH HEIGHT\nexport-obj PROJECT OUTPUT.obj\nexport-svg PROJECT OUTPUT.svg\nsimulate PROJECT OUTPUT.obj STEPS      f64 CPU XPBD, contacts always enabled\nbenchmark PROJECT STEPS                JSON measurements for the same solver\nfit-benchmark PROJECT                  explicit 1 s zero-gravity assembly, 2 s drape\nfit PROJECT OUTPUT.obj                same protocol, exports only after quality checks pass\nrecover PROJECT OUTPUT                explicitly load last valid recovery\n\nUnits: metres, kilograms, seconds; Y-up. All edits use the desktop command engine.\nSimulation is currently a rebuildable state; projects store editable construction."
        ),
        Some("new") => {
            count(&args, 2)?;
            Project::default().save(Path::new(&args[1]))?;
        }
        Some("sample") => {
            count(&args, 2)?;
            sample()?.save(Path::new(&args[1]))?;
        }
        Some("inspect") => {
            count(&args, 2)?;
            let p = Project::load(Path::new(&args[1]))?;
            let mesh = p.mesh()?;
            let seams:Vec<_>=p.seams.iter().map(|s|serde_json::json!({"id":s.id,"name":s.name,"lengths_m":p.seam_lengths(s.id).ok()})).collect();
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"revision":p.revision,"units":"m","up":"Y","pieces":p.pieces.len(),"curves":p.curves.len(),"vertices":mesh.positions.len(),"triangles":mesh.triangles.len(),"seams":seams,"stitches":mesh.stitches.len()})
                )?
            );
        }
        Some("rectangle") => {
            count(&args, 8)?;
            let mut e = Editor::new(Project::load(Path::new(&args[1]))?)?;
            e.execute(Command::AddRectangle {
                name: args[3].clone(),
                origin: [number(&args[4])?, number(&args[5])?],
                width: number(&args[6])?,
                height: number(&args[7])?,
            })?;
            e.project.save(Path::new(&args[2]))?;
        }
        Some("resize") => {
            count(&args, 6)?;
            let mut e = Editor::new(Project::load(Path::new(&args[1]))?)?;
            e.execute(Command::ResizePiece {
                piece: args[3].parse()?,
                width: number(&args[4])?,
                height: number(&args[5])?,
            })?;
            e.project.save(Path::new(&args[2]))?;
        }
        Some("export-obj") => {
            count(&args, 3)?;
            distinct(&args[1], &args[2])?;
            Project::load(Path::new(&args[1]))?
                .mesh()?
                .export_obj(Path::new(&args[2]))?;
        }
        Some("export-svg") => {
            count(&args, 3)?;
            distinct(&args[1], &args[2])?;
            Project::load(Path::new(&args[1]))?.export_svg(Path::new(&args[2]))?;
        }
        Some("simulate" | "benchmark") => {
            let benchmark = args[0] == "benchmark";
            count(&args, if benchmark { 3 } else { 4 })?;
            let steps: usize = args[if benchmark { 2 } else { 3 }].parse()?;
            if !(1..=100000).contains(&steps) {
                return Err("steps must be between 1 and 100000".into());
            }
            let p = Project::load(Path::new(&args[1]))?;
            let mut mesh = p.mesh()?;
            let mut cloth = weftforge_core::simulation::create_cloth(&p, &mesh)?;
            let start = Instant::now();
            let mut timings = Vec::with_capacity(steps);
            for _ in 0..steps {
                timings.push(cloth.step(StepSettings::default(), || false)?.step_seconds);
            }
            let seconds = start.elapsed().as_secs_f64();
            timings.sort_by(f64::total_cmp);
            if !benchmark {
                distinct(&args[1], &args[2])?;
                mesh.positions = cloth.positions().to_vec();
                mesh.export_obj(Path::new(&args[2]))?;
            }
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"kind":"implementation_only","platform":std::env::consts::OS,"architecture":std::env::consts::ARCH,"vertices":mesh.positions.len(),"triangles":mesh.triangles.len(),"steps":steps,"settings":StepSettings::default(),"wall_seconds":seconds,"simulated_seconds":cloth.simulation_time(),"simulated_seconds_per_wall_second":cloth.simulation_time()/seconds,"median_step_seconds":timings[timings.len()/2],"p95_step_seconds":timings[(timings.len()-1)*95/100],"diagnostics":cloth.diagnostics(),"memory_bytes":null,"viewport_frame_seconds":null})
                )?
            );
        }
        Some("fit-benchmark" | "fit") => {
            let benchmark = args[0] == "fit-benchmark";
            count(&args, if benchmark { 2 } else { 3 })?;
            if !benchmark {
                distinct(&args[1], &args[2])?;
            }
            let project = Project::load(Path::new(&args[1]))?;
            let (mesh, mut report, passed) = fit(&project)?;
            if !benchmark && passed {
                mesh.export_obj(Path::new(&args[2]))?;
                report["export_written"] = serde_json::json!(true);
            }
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !passed {
                return Err(
                    "fitting quality check failed; see JSON report; prior output retained".into(),
                );
            }
        }
        Some("recover") => {
            count(&args, 3)?;
            distinct(&args[1], &args[2])?;
            Project::recover(Path::new(&args[1]))?.save(Path::new(&args[2]))?;
        }
        Some(other) => return Err(format!("unknown command {other}; run --help").into()),
    }
    Ok(())
}
fn count(args: &[String], expected: usize) -> Result<()> {
    if args.len() != expected {
        Err(format!(
            "expected {} arguments after {}; run --help",
            expected - 1,
            args[0]
        )
        .into())
    } else {
        Ok(())
    }
}
fn number(value: &str) -> Result<f64> {
    let n: f64 = value.parse()?;
    if n.is_finite() {
        Ok(n)
    } else {
        Err("numeric input must be finite".into())
    }
}
fn distinct(input: &str, output: &str) -> Result<()> {
    let absolute = |p: &str| -> std::io::Result<PathBuf> {
        let p = Path::new(p);
        if p.exists() {
            p.canonicalize()
        } else {
            Ok(std::env::current_dir()?.join(p))
        }
    };
    if absolute(input)? == absolute(output)? {
        Err("export must not replace its source project".into())
    } else {
        Ok(())
    }
}

fn fit_quality(d: &Diagnostics) -> bool {
    d.finite
        && d.seam_assembly_remaining == 0.
        && d.max_stretch_error < 0.15
        && d.max_seam_error < 0.005
        && d.max_contact_error < 0.002
}

fn fit(project: &Project) -> Result<(Mesh, serde_json::Value, bool)> {
    let mut mesh = project.mesh()?;
    let mut cloth = weftforge_core::simulation::create_cloth(project, &mesh)?;
    let start = Instant::now();
    let mut phases = Vec::new();
    let mut failure = None;
    let mut timings = Vec::new();
    for (name, steps, gravity) in [("assembly", 60, [0.; 3]), ("drape", 120, [0., -9.81, 0.])] {
        let settings = StepSettings {
            dt: 1. / 60.,
            substeps: 8,
            iterations: 32,
            seam_closure_speed: 1.,
            gravity,
        };
        let phase_start = Instant::now();
        let mut completed = 0;
        for _ in 0..steps {
            match cloth.step(settings, || false) {
                Ok(d) => {
                    timings.push(d.step_seconds);
                    completed += 1;
                }
                Err(error) => {
                    failure = Some(error.to_string());
                    break;
                }
            }
        }
        phases.push(serde_json::json!({
            "name":name,"steps_requested":steps,"steps_completed":completed,"settings":settings,
            "wall_seconds":phase_start.elapsed().as_secs_f64(),"simulated_seconds":f64::from(completed)*settings.dt,
            "diagnostics":cloth.diagnostics(),
        }));
        if failure.is_some() {
            break;
        }
    }
    let seconds = start.elapsed().as_secs_f64();
    let passed = failure.is_none() && fit_quality(cloth.diagnostics());
    timings.sort_by(f64::total_cmp);
    let median = timings.get(timings.len() / 2).copied();
    let p95 = timings
        .get(timings.len().saturating_sub(1) * 95 / 100)
        .copied();
    let report = serde_json::json!({
        "kind":"implementation_only","protocol":"explicit-zero-gravity-assembly-then-drape-v1",
        "status":if passed {"passed"} else {"failed"},"solver_error":failure,
        "criteria":{"finite":true,"seam_assembly_remaining_m":0.,"max_stretch_error_below":0.15,"max_seam_gap_below_m":0.005,"max_contact_penetration_below_m":0.002},
        "platform":std::env::consts::OS,"architecture":std::env::consts::ARCH,
        "vertices":mesh.positions.len(),"triangles":mesh.triangles.len(),"phases":phases,
        "wall_seconds":seconds,"simulated_seconds":cloth.simulation_time(),"simulated_seconds_per_wall_second":cloth.simulation_time()/seconds,
        "median_step_seconds":median,"p95_step_seconds":p95,"diagnostics":cloth.diagnostics(),
        "memory_bytes":null,"viewport_frame_seconds":null,"export_written":false,
        "limitations":["Original analytic avatar and uncalibrated fabrics","Finite fixture thresholds do not establish general garment quality or reference parity"],
    });
    mesh.positions = cloth.positions().to_vec();
    Ok((mesh, report, passed))
}

fn sample() -> Result<Project> {
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
    let mut e = Editor::new(Project::default())?;
    e.execute(Command::AddPolygon {
        name: "Shirt front".into(),
        vertices: outline.clone(),
    })?;
    e.execute(Command::AddPolygon {
        name: "Shirt back".into(),
        vertices: outline.iter().map(|p| [p[0] + 1.2, p[1]]).collect(),
    })?;
    let front = e.project.pieces[0].clone();
    let back = e.project.pieces[1].clone();
    e.execute(Command::SetPlacement {
        piece: front.id,
        placement: Placement {
            translation: [0., 0., 0.22],
            rotation: [0.; 3],
        },
    })?;
    e.execute(Command::SetPlacement {
        piece: back.id,
        placement: Placement {
            translation: [-1.2, 0., -0.22],
            rotation: [0.; 3],
        },
    })?;
    for index in [1, 2, 4, 5, 9, 10, 12, 13] {
        e.execute(Command::AddSeam {
            name: format!("Construction seam {index}"),
            a: CurveInterval::full(front.boundary[index]),
            b: CurveInterval::full(back.boundary[index]),
            correspondence: vec![[0., 0.], [1., 1.]],
        })?;
    }
    e.execute(Command::SetResolution { metres: 0.12 })?;
    Ok(e.project)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fit_quality_requires_closed_seams_and_all_declared_bounds() {
        let mut d = Diagnostics {
            finite: true,
            ..Default::default()
        };
        assert!(fit_quality(&d));
        d.seam_assembly_remaining = 0.001;
        assert!(!fit_quality(&d));
        d.seam_assembly_remaining = 0.;
        d.max_stretch_error = 0.15;
        assert!(!fit_quality(&d));
        d.max_stretch_error = 0.;
        d.max_seam_error = 0.005;
        assert!(!fit_quality(&d));
        d.max_seam_error = 0.;
        d.max_contact_error = 0.002;
        assert!(!fit_quality(&d));
        d.max_contact_error = 0.;
        d.finite = false;
        assert!(!fit_quality(&d));
    }
    #[test]
    fn shirt_is_editable_and_interval_sewn() {
        let p = sample().unwrap();
        assert_eq!(p.pieces.len(), 2);
        assert_eq!(p.seams.len(), 8);
        assert_eq!(p.curves.len(), 28);
        let mesh = p.mesh().unwrap();
        assert!(mesh.stitches.len() > 16);
        let mut e = Editor::new(p).unwrap();
        let id = e.project.pieces[0].id;
        e.execute(Command::ResizePiece {
            piece: id,
            width: 1.16,
            height: 0.6,
        })
        .unwrap();
        assert_eq!(e.project.seams.len(), 8);
        assert!(e.undo());
    }
}
