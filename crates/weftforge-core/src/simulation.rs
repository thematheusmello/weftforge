//! One document-to-solver boundary shared by desktop, CLI and verification.
use crate::{Mesh, Piece, Project, Result};
use weftforge_sim::{Cloth, ClothInput, Collider, Diagnostics, Material, Stitch, WeightedPoint};

/// Resolves current solver triangle attribution to editable construction.
pub fn peak_strain_piece<'a>(
    project: &'a Project,
    mesh: &Mesh,
    diagnostics: &Diagnostics,
) -> Option<&'a Piece> {
    if mesh.doc_revision != project.revision {
        return None;
    }
    let triangle = mesh.triangles.get(diagnostics.peak_strain?.triangle)?;
    project.piece(*mesh.piece_ids.get(triangle[0])?)
}

pub fn create_cloth(project: &Project, mesh: &Mesh) -> Result<Cloth> {
    create_cloth_cancellable(project, mesh, || false)
}

/// Prepares a complete solver snapshot, preserving caller-owned state on cancellation.
pub fn create_cloth_cancellable(
    project: &Project,
    mesh: &Mesh,
    cancelled: impl Fn() -> bool,
) -> Result<Cloth> {
    create_cloth_context(project, mesh, cancelled, None)
}

pub(crate) fn create_cloth_for_offset(
    project: &Project,
    mesh: &Mesh,
    work: &crate::offset::Work<'_>,
) -> Result<Cloth> {
    create_cloth_context(project, mesh, || false, Some(work))
}

fn create_cloth_context(
    project: &Project,
    mesh: &Mesh,
    cancelled: impl Fn() -> bool,
    work: Option<&crate::offset::Work<'_>>,
) -> Result<Cloth> {
    let check = || {
        if let Some(work) = work {
            work.step()?;
        }
        crate::ensure(
            !cancelled(),
            "Simulation initialization cancelled; previous state retained",
        )
    };
    check()?;
    if let Some(work) = work {
        project.validate_for_offset(work)?;
    } else {
        project.validate()?;
    }
    check()?;
    mesh.validate_context(work)?;
    check()?;
    crate::ensure(
        mesh.doc_revision == project.revision,
        "Rest mesh belongs to an obsolete document revision",
    )?;
    crate::ensure(
        mesh.sewing_groups
            .iter()
            .map(|group| group.id)
            .eq(project.sewing_groups.iter().map(|group| group.id)),
        "Rest mesh sewing previews belong to different construction",
    )?;
    let triangle_materials = mesh
        .triangles
        .iter()
        .enumerate()
        .map(|(i, triangle)| {
            if i % 256 == 0 {
                check()?;
            }
            let piece = project
                .piece(mesh.piece_ids[triangle[0]])
                .ok_or_else(|| crate::Error("Rest mesh references an unknown piece".into()))?;
            let material = project
                .material(piece.material)
                .ok_or_else(|| crate::Error("Pattern references an unknown material".into()))?;
            Ok(Material {
                areal_density: material.areal_density,
                warp_stiffness: material.warp_stiffness,
                weft_stiffness: material.weft_stiffness,
                shear_stiffness: material.shear_stiffness,
                bending_stiffness: material.bending_stiffness,
                grain_angle: piece.grain_angle,
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
        .enumerate()
        .map(|(i, stitch)| {
            if i % 256 == 0 {
                check()?;
            }
            Ok(Stitch {
                a: weighted(&stitch.a),
                b: weighted(&stitch.b),
                compliance: 1e-8,
            })
        })
        .collect::<Result<_>>()?;
    let mut colliders = vec![Collider::Ground { height: 0.0 }];
    for (i, capsule) in project.mannequin.capsules.iter().enumerate() {
        if i % 256 == 0 {
            check()?;
        }
        colliders.push(Collider::Capsule {
            a: capsule.a,
            b: capsule.b,
            radius: capsule.radius,
        });
    }
    check()?;
    let failure = std::cell::RefCell::new(None);
    let result = Cloth::new_cancellable(
        ClothInput {
            positions: mesh.positions.clone(),
            triangles: mesh.triangles.clone(),
            material_uv: mesh.uv.clone(),
            triangle_materials,
            stitches,
            pins: vec![],
            colliders,
        },
        || {
            if let Some(work) = work
                && let Err(error) = work.check().and_then(|()| work.step())
            {
                *failure.borrow_mut() = Some(error);
                return true;
            }
            cancelled()
        },
    )
    .map_err(|error| crate::Error(error.to_string()));
    if let Some(error) = failure.into_inner() {
        Err(error)
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Editor};
    #[test]
    fn strain_attribution_resolves_current_piece_and_rejects_stale_indices() {
        let mut editor = Editor::default();
        for (name, origin) in [("First", [0., 0.]), ("Second", [1., 0.])] {
            editor
                .execute(Command::AddRectangle {
                    name: name.into(),
                    origin,
                    width: 0.4,
                    height: 0.6,
                })
                .unwrap();
        }
        let mesh = editor.project.mesh().unwrap();
        let second = editor.project.pieces[1].id;
        let triangle = mesh
            .triangles
            .iter()
            .position(|t| mesh.piece_ids[t[0]] == second)
            .unwrap();
        let mut diagnostics = Diagnostics::default();
        assert!(peak_strain_piece(&editor.project, &mesh, &diagnostics).is_none());
        diagnostics.peak_strain = Some(weftforge_sim::StrainPeak {
            triangle,
            axis: weftforge_sim::MaterialAxis::Weft,
            signed_strain: -0.25,
        });
        let piece = peak_strain_piece(&editor.project, &mesh, &diagnostics).unwrap();
        assert_eq!((piece.id, piece.name.as_str()), (second, "Second"));
        diagnostics.peak_strain.as_mut().unwrap().triangle = mesh.triangles.len();
        assert!(peak_strain_piece(&editor.project, &mesh, &diagnostics).is_none());
        diagnostics.peak_strain.as_mut().unwrap().triangle = triangle;
        editor
            .execute(Command::SetResolution { metres: 0.06 })
            .unwrap();
        assert!(peak_strain_piece(&editor.project, &mesh, &diagnostics).is_none());
    }
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

    #[test]
    fn cancellable_adapter_reaches_initial_contacts_and_retains_inputs() {
        let mut editor = Editor::default();
        editor
            .execute(Command::AddRectangle {
                name: "Panel".into(),
                origin: [0., 1.],
                width: 0.2,
                height: 0.2,
            })
            .unwrap();
        let mesh = editor.project.mesh().unwrap();
        let before = editor.project.clone();
        let original_mesh = mesh.clone();
        let ordinary = create_cloth(&editor.project, &mesh).unwrap();
        let polls = std::cell::Cell::new(0);
        let complete = create_cloth_cancellable(&editor.project, &mesh, || {
            polls.set(polls.get() + 1);
            false
        })
        .unwrap();
        assert_eq!(complete.positions(), ordinary.positions());
        assert_eq!(complete.masses(), ordinary.masses());
        assert_eq!(
            complete.diagnostics().initial_intersections,
            ordinary.diagnostics().initial_intersections
        );
        assert!(polls.get() > 2);
        // The last contact poll precedes the constructor's final publication check.
        let cancel_at = polls.get() - 1;
        let calls = std::cell::Cell::new(0);
        let error = create_cloth_cancellable(&editor.project, &mesh, || {
            calls.set(calls.get() + 1);
            assert!(
                calls.get() <= cancel_at,
                "work continued after cancellation"
            );
            calls.get() == cancel_at
        })
        .unwrap_err();
        assert!(error.to_string().contains("cancelled"));
        assert_eq!(calls.get(), cancel_at);
        assert_eq!(editor.project, before);
        assert_eq!(mesh, original_mesh);
        assert_eq!(ordinary.simulation_time(), 0.);
        let mut invalid = mesh;
        invalid.positions[0][0] = f64::NAN;
        assert!(
            create_cloth_cancellable(&editor.project, &invalid, || false)
                .unwrap_err()
                .to_string()
                .contains("non-finite")
        );
    }

    #[test]
    fn original_shirt_initial_contacts_match_full_scan_at_multiple_resolutions() {
        let mut project =
            Project::from_json(include_bytes!("../../../fixtures/original-shirt.weftforge"))
                .unwrap();
        for resolution in [0.07, 0.035] {
            project.resolution = resolution;
            let mesh = project.mesh().unwrap();
            let mut cloth = create_cloth(&project, &mesh).unwrap();
            let initial = cloth.diagnostics().clone();
            // Reset uses the unchanged exhaustive live diagnostics on the same rest positions.
            cloth.reset();
            assert_eq!(cloth.positions(), mesh.positions);
            assert_eq!(
                initial.initial_intersections,
                cloth.diagnostics().initial_intersections
            );
            assert_eq!(
                initial.max_contact_error,
                cloth.diagnostics().max_contact_error
            );
        }
    }
}
