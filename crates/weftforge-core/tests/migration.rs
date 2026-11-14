use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use weftforge_core::{Command, Editor, Project, recovery_path};

const V1_FIXTURES: [&[u8]; 4] = [
    include_bytes!("../../../fixtures/original-shirt.weftforge"),
    include_bytes!("../../../fixtures/relaxed-shirt.weftforge"),
    include_bytes!("../../../fixtures/ui-authored-shirt.weftforge"),
    include_bytes!("../../../fixtures/single-candidate-shirt.weftforge"),
];
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn temp_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "weftforge-migrate-{}-{}.weftforge",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ))
}

fn assert_decimal_bits(actual: f64, decimal: &str) {
    // Keep the expected parser independent of the JSON reader under test.
    let expected: f64 = decimal.parse().unwrap();
    assert_eq!(actual.to_bits(), expected.to_bits(), "{decimal}");
}

fn assert_fixture_point_bits(project: &Project) {
    let actual = [40, 41].map(|id| project.point(id).unwrap().position[0].to_bits());
    let expected = ["1.1199999999999999", "1.0999999999999999"]
        .map(|decimal| decimal.parse::<f64>().unwrap().to_bits());
    assert_eq!(actual, expected, "Original fixture point X coordinates");
}

fn assert_disk_roundtrip(bytes: &[u8], check: impl Fn(&Project)) {
    let path = temp_path();
    fs::write(&path, bytes).unwrap();
    let project = Project::load(&path).unwrap();
    check(&project);
    check(&Project::from_json(&project.to_json().unwrap()).unwrap());
    project.save(&path).unwrap();
    check(&Project::load(&path).unwrap());
    check(&Project::recover(&path).unwrap());
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), bytes);
    fs::remove_file(&path).unwrap();
    fs::remove_file(recovery_path(&path)).unwrap();
}

#[test]
fn original_v1_point_bits_survive_load_migration_save_and_reload() {
    let prepared = std::str::from_utf8(V1_FIXTURES[0]).unwrap().replacen(
        "\"resolution\": 0.12",
        "\"resolution\": 0.015",
        1,
    );
    assert_disk_roundtrip(prepared.as_bytes(), |project| {
        assert_fixture_point_bits(project);
        assert_decimal_bits(project.resolution, "0.015");
    });
}

fn precision_document(version: u32) -> String {
    let mut text = std::str::from_utf8(V1_FIXTURES[0])
        .unwrap()
        .replace("\r\n", "\n")
        .replace(
            "\"resolution\": 0.12",
            "\"resolution\": 0.015000000000000001",
        )
        .replace(
            "\"stretch_stiffness\": 400.0",
            "\"stretch_stiffness\": 99999999.99999999",
        )
        .replace("\"damping\": 0.2", "\"damping\": 51.248178375505404")
        .replace(
            "\"translation\": [\n            0.0,",
            "\"translation\": [\n            999.9999999999999,",
        )
        .replace(
            "\"rotation\": [\n            0.0,",
            "\"rotation\": [\n            999999.9999999999,",
        );
    if version == 2 {
        text = text
            .replace("\"version\": 1", "\"version\": 2")
            .replace(
                "\"stretch_stiffness\": 99999999.99999999",
                "\"warp_stiffness\": 99999999.99999999, \"weft_stiffness\": 99999999.99999999",
            )
            .replace(
                "\"placement\":",
                "\"grain_angle\": 999999.9999999999, \"placement\":",
            );
    }
    text
}

fn assert_precision_document_bits(project: &Project, version: u32) {
    assert_decimal_bits(project.materials[0].warp_stiffness, "99999999.99999999");
    assert_decimal_bits(project.materials[0].weft_stiffness, "99999999.99999999");
    assert_decimal_bits(project.materials[0].damping, "51.248178375505404");
    assert_decimal_bits(
        project.pieces[0].placement.translation[0],
        "999.9999999999999",
    );
    assert_decimal_bits(project.pieces[0].placement.rotation[0], "999999.9999999999");
    let grain = if version == 1 {
        "0.0"
    } else {
        "999999.9999999999"
    };
    assert_decimal_bits(project.pieces[0].grain_angle, grain);
    assert_fixture_point_bits(project);
    assert_decimal_bits(project.resolution, "0.015000000000000001");
}

#[test]
fn v1_hard_decimals_and_large_authoring_values_keep_their_bits() {
    assert_disk_roundtrip(precision_document(1).as_bytes(), |project| {
        assert_precision_document_bits(project, 1);
    });
}

#[test]
fn v2_hard_decimals_and_large_authoring_values_keep_their_bits() {
    assert_disk_roundtrip(precision_document(2).as_bytes(), |project| {
        assert_precision_document_bits(project, 2);
    });
}

#[test]
fn both_schema_readers_reject_nonfinite_and_out_of_range_numbers() {
    for version in [1, 2] {
        let text = precision_document(version);
        for invalid in [
            "NaN",
            "Infinity",
            "-Infinity",
            "1e309",
            "-1e309",
            "null",
            "\"0.015\"",
            "0.5000000000000001",
        ] {
            let bad = text.replace(
                "\"resolution\": 0.015000000000000001",
                &format!("\"resolution\": {invalid}"),
            );
            assert!(
                Project::from_json(bad.as_bytes()).is_err(),
                "v{version}: {invalid}"
            );
        }
        let bad = text.replace("99999999.99999999", "100000000.00000001");
        assert!(
            Project::from_json(bad.as_bytes()).is_err(),
            "v{version}: excessive stiffness"
        );
    }
}

fn assert_lossless_v1(bytes: &[u8]) -> Project {
    let old: Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(old["version"], 1);
    let project = Project::from_json(bytes).unwrap();
    let mut current: Value = serde_json::from_slice(&project.to_json().unwrap()).unwrap();
    assert_eq!(current["version"], 2);
    for piece in current["project"]["pieces"].as_array_mut().unwrap() {
        assert_eq!(
            piece.as_object_mut().unwrap().remove("grain_angle"),
            Some(json!(0.0))
        );
    }
    for material in current["project"]["materials"].as_array_mut().unwrap() {
        let material = material.as_object_mut().unwrap();
        let warp = material.remove("warp_stiffness").unwrap();
        assert_eq!(material.remove("weft_stiffness"), Some(warp.clone()));
        material.insert("stretch_stiffness".into(), warp);
    }
    // Inverting the two declared schema changes must reproduce every v1 authored value.
    assert_eq!(current["project"], old["project"]);
    project
}

#[test]
fn real_v1_fixtures_migrate_without_authored_geometry_or_physical_data_loss() {
    for bytes in V1_FIXTURES {
        let project = assert_lossless_v1(bytes);
        let mesh = project.mesh().unwrap();
        assert_eq!(mesh.doc_revision, project.revision);
        for ((&position, &uv), &piece) in mesh.positions.iter().zip(&mesh.uv).zip(&mesh.piece_ids) {
            let expected = project.piece(piece).unwrap().placement.transform(uv);
            assert!(
                position
                    .into_iter()
                    .zip(expected)
                    .all(|(a, b)| (a - b).abs() < 1e-12)
            );
        }
        let cloth = weftforge_core::simulation::create_cloth(&project, &mesh).unwrap();
        let expected_mass: f64 = mesh
            .triangles
            .iter()
            .map(|triangle| {
                let [a, b, c] = triangle.map(|v| mesh.uv[v]);
                let area =
                    ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() * 0.5;
                let piece = project.piece(mesh.piece_ids[triangle[0]]).unwrap();
                area * project.material(piece.material).unwrap().areal_density
            })
            .sum();
        assert!((cloth.masses().iter().sum::<f64>() - expected_mass).abs() < 1e-12);
        assert!(cloth.diagnostics().max_stretch_error < 1e-10);
    }
}

#[test]
fn nondefault_v1_material_values_are_mapped_instead_of_replaced_by_defaults() {
    let mut old: Value = serde_json::from_slice(V1_FIXTURES[0]).unwrap();
    let material = &mut old["project"]["materials"][0];
    material["name"] = json!("Custom linen α");
    material["stretch_stiffness"] = json!(1573.25);
    material["areal_density"] = json!(0.31);
    material["shear_stiffness"] = json!(83.7);
    material["bending_stiffness"] = json!(0.00073);
    material["damping"] = json!(2.9);
    material["friction"] = json!(0.71);
    material["collision_thickness"] = json!(0.0041);
    material["color"] = json!([0.1, 0.7, 0.3]);
    let project = assert_lossless_v1(&serde_json::to_vec(&old).unwrap());
    assert_eq!(project.materials[0].warp_stiffness, 1573.25);
    assert_eq!(project.materials[0].weft_stiffness, 1573.25);
}

#[test]
fn v2_saves_orthotropy_and_grain_while_recovery_retains_original_v1_bytes() {
    let path = temp_path();
    fs::write(&path, V1_FIXTURES[0]).unwrap();
    let original = Project::load(&path).unwrap();
    let mut editor = Editor::new(original.clone()).unwrap();
    let piece = editor.project.pieces[0].id;
    let mut material = editor.project.materials[0].clone();
    material.warp_stiffness = 37.5;
    material.weft_stiffness = 1234.0;
    editor
        .execute(Command::Transaction(vec![
            Command::SetGrain { piece, angle: 0.37 },
            Command::UpdateMaterial { material },
        ]))
        .unwrap();
    editor.project.save(&path).unwrap();
    let saved = fs::read(&path).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&saved).unwrap()["version"],
        2
    );
    assert_eq!(Project::load(&path).unwrap(), editor.project);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), V1_FIXTURES[0]);
    assert_eq!(Project::recover(&path).unwrap(), original);
    let mut invalid = editor.project.clone();
    invalid.pieces[0].grain_angle = f64::NAN;
    assert!(invalid.save(&path).is_err());
    invalid = editor.project.clone();
    invalid.materials[0].weft_stiffness = 0.;
    assert!(invalid.save(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), saved);
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), V1_FIXTURES[0]);
    fs::write(&path, b"invalid existing primary").unwrap();
    assert!(editor.project.save(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"invalid existing primary");
    assert_eq!(fs::read(recovery_path(&path)).unwrap(), V1_FIXTURES[0]);
    fs::remove_file(&path).unwrap();
    fs::remove_file(recovery_path(&path)).unwrap();
}

#[test]
fn version_dispatch_rejects_mixed_schemas_unknown_fields_and_unsupported_versions() {
    let old: Value = serde_json::from_slice(V1_FIXTURES[0]).unwrap();
    let v2: Value = serde_json::from_slice(
        &Project::from_json(V1_FIXTURES[0])
            .unwrap()
            .to_json()
            .unwrap(),
    )
    .unwrap();
    let reject = |document: &Value| {
        assert!(Project::from_json(&serde_json::to_vec(document).unwrap()).is_err())
    };
    let mut bad = old.clone();
    bad["project"]["pieces"][0]["grain_angle"] = json!(0.0);
    reject(&bad);
    let mut bad = old.clone();
    bad["project"]["materials"][0]["warp_stiffness"] = json!(400.);
    reject(&bad);
    let mut bad = v2.clone();
    bad["project"]["materials"][0]["stretch_stiffness"] = json!(400.);
    reject(&bad);
    let mut bad = v2.clone();
    bad["project"]["pieces"][0]
        .as_object_mut()
        .unwrap()
        .remove("grain_angle");
    reject(&bad);
    let mut bad = v2.clone();
    bad["project"]["materials"][0]
        .as_object_mut()
        .unwrap()
        .remove("weft_stiffness");
    reject(&bad);
    for document in [&old, &v2] {
        let mut bad = document.clone();
        bad["unknown"] = json!(0);
        reject(&bad);
        let mut bad = document.clone();
        bad["project"]["unknown"] = json!(0);
        reject(&bad);
        for version in [0, 3, 999] {
            let mut bad = document.clone();
            bad["version"] = json!(version);
            reject(&bad);
        }
    }
    assert!(Project::from_json(b"{malformed").is_err());
    assert!(Project::from_json(&vec![b' '; 16 * 1024 * 1024 + 1]).is_err());
}

#[test]
fn duplicate_fields_are_rejected_at_envelope_project_and_material_boundaries() {
    for bytes in [
        V1_FIXTURES[0].to_vec(),
        Project::from_json(V1_FIXTURES[0])
            .unwrap()
            .to_json()
            .unwrap(),
    ] {
        let text = String::from_utf8(bytes).unwrap();
        let duplicate_format =
            text.replacen("\"format\":", "\"format\":\"weftforge\",\"format\":", 1);
        assert!(Project::from_json(duplicate_format.as_bytes()).is_err());
        let duplicate_revision = text.replacen("\"revision\":", "\"revision\":0,\"revision\":", 1);
        assert!(Project::from_json(duplicate_revision.as_bytes()).is_err());
        let duplicate_density = text.replacen(
            "\"areal_density\":",
            "\"areal_density\":0.2,\"areal_density\":",
            1,
        );
        assert!(Project::from_json(duplicate_density.as_bytes()).is_err());
    }
}
