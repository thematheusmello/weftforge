use serde_json::{Value, json};
use weftforge_core::{Command, Editor, FabricSurface, Project};

#[test]
fn legacy_fabric_payloads_and_files_retain_separate_appearance_and_history() {
    let mut editor = Editor::new(
        Project::from_json(include_bytes!("../../../fixtures/original-shirt.weftforge")).unwrap(),
    )
    .unwrap();
    let material = editor.project.materials[0].clone();
    assert_eq!(
        material.surface.unwrap_or_default(),
        FabricSurface {
            metallic: 0.,
            roughness: 1.
        }
    );
    let plain: Value = serde_json::from_slice(&editor.project.to_json().unwrap()).unwrap();
    assert_eq!(plain["version"], 6);
    assert!(plain["project"]["materials"][0].get("surface").is_none());
    let finish = FabricSurface {
        metallic: 0.65,
        roughness: 0.125,
    };
    editor
        .execute(Command::SetFabricSurface {
            material: material.id,
            metallic: finish.metallic,
            roughness: finish.roughness,
        })
        .unwrap();
    assert!(editor.last_change_appearance_only());
    let finished = editor.project.clone();
    assert_eq!(finished.materials[0].surface, Some(finish));
    assert_eq!(finished.materials[0].color, material.color);
    let mut authored: Value = serde_json::from_slice(&finished.to_json().unwrap()).unwrap();
    assert_eq!(authored["version"], 7);
    assert_eq!(
        Project::from_json(&finished.to_json().unwrap()).unwrap(),
        finished
    );
    authored["version"] = json!(6);
    assert!(Project::from_json(&serde_json::to_vec(&authored).unwrap()).is_err());
    let mut legacy_update = plain["project"]["materials"][0].clone();
    legacy_update["friction"] = json!(0.38);
    let command =
        serde_json::from_value(json!({"type":"update_material","data":{"material":legacy_update}}))
            .unwrap();
    editor.execute(command).unwrap();
    assert!(!editor.last_change_appearance_only());
    assert_eq!(editor.project.materials[0].surface, Some(finish));
    assert!(editor.undo() && !editor.last_change_appearance_only());
    assert!(editor.undo() && editor.last_change_appearance_only());
    assert_eq!(editor.project.materials[0], material);
    assert!(editor.redo() && editor.last_change_appearance_only());
    assert_eq!(editor.project.materials, finished.materials);

    for (metallic, roughness) in [
        (-0.01, 0.),
        (1.001, 1.),
        (0., -0.01),
        (0., 1.1),
        (f64::NAN, 0.),
        (0., f64::INFINITY),
    ] {
        let before = editor.project.clone();
        assert!(
            editor
                .execute(Command::SetFabricSurface {
                    material: material.id,
                    metallic,
                    roughness
                })
                .is_err()
        );
        assert_eq!(editor.project, before);
        assert!(editor.can_redo());
    }
    authored["version"] = json!(7);
    for bad in [
        Value::Null,
        json!({"metallic": 0.1}),
        json!({"metallic": 0.1,"roughness": 2.}),
        json!({"metallic":0.,"roughness":1.,"other":0}),
    ] {
        authored["project"]["materials"][0]["surface"] = bad;
        assert!(Project::from_json(&serde_json::to_vec(&authored).unwrap()).is_err());
    }
}
