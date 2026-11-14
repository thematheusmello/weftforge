use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use weftforge_core::{FORMAT_VERSION, Project};

#[test]
fn cli_authors_grain_and_orthotropy_and_preserves_output_on_invalid_values() {
    let dir = std::env::temp_dir().join(format!(
        "weftforge-cli-grain-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let input = dir.join("legacy.weftforge");
    let grain = dir.join("grain.weftforge");
    let output = dir.join("orthotropic.weftforge");
    fs::write(
        &input,
        include_bytes!("../../../fixtures/original-shirt.weftforge"),
    )
    .unwrap();
    let original = Project::load(&input).unwrap();
    let piece = original.pieces[0].id.to_string();
    let material = original.materials[0].id.to_string();
    let binary = env!("CARGO_BIN_EXE_weftforge-cli");
    let result = Command::new(binary)
        .arg("grain")
        .arg(&input)
        .arg(&grain)
        .arg(&piece)
        .arg("0.37")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result = Command::new(binary)
        .arg("stiffness")
        .arg(&grain)
        .arg(&output)
        .arg(&material)
        .args(["50", "1000"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let authored = Project::load(&output).unwrap();
    assert_eq!(authored.pieces[0].grain_angle, 0.37);
    assert_eq!(authored.materials[0].warp_stiffness, 50.);
    assert_eq!(authored.materials[0].weft_stiffness, 1000.);
    assert_eq!(
        authored.materials[0].shear_stiffness,
        original.materials[0].shear_stiffness
    );
    assert_eq!(authored.materials[0].color, original.materials[0].color);
    assert_eq!(authored.points, original.points);
    assert_eq!(authored.seams, original.seams);
    assert!(authored.sewing_groups.is_empty());
    let valid = fs::read(&output).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&valid).unwrap();
    assert_eq!(json["version"], FORMAT_VERSION);
    assert_eq!(json["project"]["sewing_groups"], serde_json::json!([]));
    let result = Command::new(binary)
        .arg("grain")
        .arg(&grain)
        .arg(&output)
        .arg(&piece)
        .arg("NaN")
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read(&output).unwrap(), valid);
    let result = Command::new(binary)
        .arg("stiffness")
        .arg(&grain)
        .arg(&output)
        .arg(&material)
        .args(["400", "0"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read(&output).unwrap(), valid);
    assert_eq!(
        fs::read(&input).unwrap(),
        include_bytes!("../../../fixtures/original-shirt.weftforge")
    );
    fs::remove_dir_all(dir).unwrap();
}
