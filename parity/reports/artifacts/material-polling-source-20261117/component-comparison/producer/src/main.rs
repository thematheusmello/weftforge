pub use weftforge_sim::{Material, SimulationError, Stitch, Vec3};

fn check_cancel(cancelled: &impl Fn() -> bool) -> Result<(), SimulationError> {
    if cancelled() {
        Err(SimulationError::Cancelled)
    } else {
        Ok(())
    }
}
#[allow(dead_code)]
#[path = "../../../../crates/weftforge-sim/src/math.rs"]
mod math;
#[allow(dead_code, unused_imports)]
#[path = "../../old-material.rs"]
mod old_material;
#[allow(dead_code, unused_imports)]
#[path = "../../new-material.rs"]
mod new_material;
#[path = "../../probe.rs"]
mod probe;
fn main() { probe::run(); }
