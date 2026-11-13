//! One document-to-solver boundary shared by desktop, CLI and verification.
use crate::{Mesh, Project, Result};
use weftforge_sim::{Cloth, ClothInput, Collider, Material, Stitch, WeightedPoint};

pub fn create_cloth(project: &Project, mesh: &Mesh) -> Result<Cloth> {
    project.validate()?;
    mesh.validate()?;
    crate::ensure(
        mesh.doc_revision == project.revision,
        "Rest mesh belongs to an obsolete document revision",
    )?;
    let triangle_materials = mesh
        .triangles
        .iter()
        .map(|triangle| {
            let piece = project
                .piece(mesh.piece_ids[triangle[0]])
                .ok_or_else(|| crate::Error("Rest mesh references an unknown piece".into()))?;
            let material = project
                .material(piece.material)
                .ok_or_else(|| crate::Error("Pattern references an unknown material".into()))?;
            Ok(Material {
                areal_density: material.areal_density,
                warp_stiffness: material.stretch_stiffness,
                weft_stiffness: material.stretch_stiffness,
                shear_stiffness: material.shear_stiffness,
                bending_stiffness: material.bending_stiffness,
                grain_angle: 0.0,
                damping: material.damping,
                friction: material.friction,
                collision_thickness: material.collision_thickness,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let weighted = |vertex: &crate::WeightedVertex| WeightedPoint {
        vertices: vertex.vertices,
        weights: [1.0 - vertex.weight, vertex.weight],
    };
    let stitches = mesh
        .stitches
        .iter()
        .map(|stitch| Stitch {
            a: weighted(&stitch.a),
            b: weighted(&stitch.b),
            compliance: 1e-8,
        })
        .collect();
    let mut colliders = vec![Collider::Ground { height: 0.0 }];
    colliders.extend(
        project
            .mannequin
            .capsules
            .iter()
            .map(|capsule| Collider::Capsule {
                a: capsule.a,
                b: capsule.b,
                radius: capsule.radius,
            }),
    );
    Cloth::new(ClothInput {
        positions: mesh.positions.clone(),
        triangles: mesh.triangles.clone(),
        material_uv: mesh.uv.clone(),
        triangle_materials,
        stitches,
        pins: vec![],
        colliders,
    })
    .map_err(|error| crate::Error(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Editor};
    #[test]
    fn rejects_stale_and_foreign_meshes_without_panicking() {
        let mut editor = Editor::default();
        editor
            .execute(Command::AddRectangle {
                name: "Panel".into(),
                origin: [0., 1.],
                width: 0.2,
                height: 0.2,
            })
            .unwrap();
        let mut mesh = editor.project.mesh().unwrap();
        assert!(create_cloth(&editor.project, &mesh).is_ok());
        mesh.doc_revision += 1;
        assert!(create_cloth(&editor.project, &mesh).is_err());
        mesh.doc_revision -= 1;
        mesh.piece_ids.fill(u64::MAX);
        assert!(create_cloth(&editor.project, &mesh).is_err());
    }
}
