use crate::{Material, Vec3, old_material, new_material};
use std::time::Instant;
const NX: usize = 31;
const NY: usize = 21;
const SWEEPS: usize = 32;
const REPEATS: usize = 6;
type History = Vec<(Vec<Vec3>, Vec<Vec3>)>;
fn same_bits(a: &[Vec3], b: &[Vec3]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a,b)|
        a.iter().zip(b).all(|(a,b)| a.to_bits() == b.to_bits()))
}

fn fixture() -> (Vec<[f64; 2]>, Vec<[usize; 3]>, Vec<Vec3>, Vec<f64>, Material) {
    let material = Material {
        areal_density: 0.18, warp_stiffness: 400., weft_stiffness: 400.,
        shear_stiffness: 100., bending_stiffness: 0.003,
        grain_angle: std::f64::consts::FRAC_PI_2,
        damping: 0.2, friction: 0.4, collision_thickness: 0.003,
    };
    let mut uv = Vec::new();
    let mut predictor = Vec::new();
    for j in 0..NY {
        for i in 0..NX {
            let s = i as f64 / (NX-1) as f64;
            let t = j as f64 / (NY-1) as f64;
            let u = -0.36 + 0.72*s;
            let v = 0.96 + 0.64*t;
            uv.push([u,v]);
            predictor.push(if i == 0 { [u,v,0.] } else {
                [u + 0.06*s*t + 0.008*s*(1.-s)*(2.*t-1.),
                 v + 0.04*s*s + 0.01*t*(1.-t), 0.]
            });
        }
    }
    let mut triangles = Vec::new();
    let mut masses = vec![0.; uv.len()];
    let mut signed_area = 0.;
    for j in 0..NY-1 {
        for i in 0..NX-1 {
            let a = j*NX+i; let b = a+1; let c = a+NX; let d = c+1;
            for tri in [[a,b,d],[a,d,c]] {
                let [a,b,c] = tri.map(|v| uv[v]);
                let area = ((b[0]-a[0])*(c[1]-a[1]) - (c[0]-a[0])*(b[1]-a[1]))/2.;
                assert!(area.is_finite() && area > 0.);
                signed_area += area;
                for v in tri { masses[v] += material.areal_density*area/3.; }
                triangles.push(tri);
            }
        }
    }
    assert_eq!((uv.len(),triangles.len()),(651,1200));
    assert!((signed_area-0.4608).abs() < 1e-12);
    assert!((masses.iter().sum::<f64>()-0.082944).abs() < 1e-12);
    assert!(masses.iter().all(|m| m.is_finite() && *m > 0.));
    let weights: Vec<_> = masses.iter().enumerate().map(|(i,m)| if i%NX==0 { 0. } else { 1./m }).collect();
    assert_eq!(weights.iter().filter(|w| **w == 0.).count(),21);
    assert_eq!(weights.iter().filter(|w| **w > 0.).count(),630);
    (uv, triangles, predictor, weights, material)
}

fn control(membranes: &[old_material::Membrane], predictor: &[Vec3], weights: &[f64], started: &Instant) -> (History,u128) {
    let mut x = predictor.to_vec(); let mut p = vec![[0.;3];x.len()];
    let mut history = Vec::with_capacity(SWEEPS);
    let wall_started = Instant::now();
    for _ in 0..SWEEPS {
        old_material::solve_material(membranes,&mut x,weights,(1./60.)/8.,&mut p,&|| started.elapsed().as_secs_f64() >= 420.).unwrap();
        history.push((x.clone(),p.clone()));
    }
    (history,wall_started.elapsed().as_nanos())
}
fn observed(membranes: &[new_material::Membrane], predictor: &[Vec3], weights: &[f64], started: &Instant) -> (History,u128) {
    let mut x = predictor.to_vec(); let mut p = vec![[0.;3];x.len()];
    let mut history = Vec::with_capacity(SWEEPS);
    let wall_started = Instant::now();
    for _ in 0..SWEEPS {
        new_material::solve_material(membranes,&mut x,weights,(1./60.)/8.,&mut p,&|| started.elapsed().as_secs_f64() >= 420.).unwrap();
        history.push((x.clone(),p.clone()));
    }
    (history,wall_started.elapsed().as_nanos())
}
pub(crate) fn run() {
    let started = Instant::now();
    let (uv,triangles,predictor,weights,material) = fixture();
    let controls: Vec<_> = triangles.iter().map(|tri| old_material::Membrane::new(*tri,&uv,material)).collect();
    let membranes: Vec<_> = triangles.iter().map(|tri| new_material::Membrane::new(*tri,&uv,material)).collect();
    assert!(controls.iter().all(|m|m.valid_scale()) && membranes.iter().all(|m|m.valid_scale()));
    println!(r#"{{"kind":"fixture","vertices":651,"triangles":1200,"fixed":21,"free":630,"sweeps":32,"warmups":1,"measured_repeats":5,"control":"52099","observed":"9d4b","cancellation_predicate":"|| started.elapsed().as_secs_f64() >= 420.","per_callback_counter":false}}"#);
    for repeat in 0..REPEATS {
        let (reference,control_ns,candidate,observed_ns) = if repeat%2==0 {
            let (r,t) = control(&controls,&predictor,&weights,&started);
            let (x,u) = observed(&membranes,&predictor,&weights,&started);
            (r,t,x,u)
        } else {
            let (x,u) = observed(&membranes,&predictor,&weights,&started);
            let (r,t) = control(&controls,&predictor,&weights,&started);
            (r,t,x,u)
        };
        assert_eq!(reference.len(),SWEEPS);
        assert_eq!(candidate.len(),SWEEPS);
        for ((a,pa),(b,pb)) in reference.iter().zip(&candidate) {
            assert!(same_bits(a,b) && same_bits(pa,pb));
            assert!(b.iter().chain(pb).flatten().all(|v|v.is_finite()));
            for v in (0..NX*NY).filter(|v|v%NX==0) {
                assert!(same_bits(&b[v..v+1],&predictor[v..v+1]));
                assert_eq!(pb[v].map(f64::to_bits),[0_f64.to_bits();3]);
            }
        }
        println!(r#"{{"kind":"repeat","index":{},"warmup":{},"control_first":{},"control_wall_ns":{},"observed_wall_ns":{},"material_calls":32,"all32_position_and_p_boundaries_bit_identical":true}}"#,repeat,repeat==0,repeat%2==0,control_ns,observed_ns);
    }
}
