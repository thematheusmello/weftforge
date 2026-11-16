use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use weftforge_core::{
    Command, CurveInterval, Editor, Id, MAX_OBJECTS, Material, Mesh, Placement, Project,
    recovery_path, simulation::create_cloth,
};

// Original literal inputs and answers were frozen before this implementation.
fn rectangle() -> Editor {
    Editor::new(Project::from_json(include_bytes!("fixtures/original-cut-r240.weftforge")).unwrap())
        .unwrap()
}

fn l_shape() -> Editor {
    Editor::new(Project::from_json(include_bytes!("fixtures/original-cut-l.weftforge")).unwrap())
        .unwrap()
}

fn cut() -> Command {
    Command::CutPiece {
        piece: 2,
        start_curve: 7,
        start_t: 0.375,
        end_curve: 9,
        end_t: 0.625,
    }
}

fn wire(project: &Project) -> Vec<u8> {
    serde_json::to_vec(project).unwrap()
}

fn next_id(project: &Project) -> Id {
    serde_json::to_value(project).unwrap()["next_id"]
        .as_u64()
        .unwrap()
}

fn allocator(project: Project, id: Id) -> Project {
    let mut value = serde_json::to_value(project).unwrap();
    value["next_id"] = json!(id);
    serde_json::from_value(value).unwrap()
}

fn rejected(editor: &mut Editor, command: Command) -> String {
    let before = wire(&editor.project);
    let history = (editor.can_undo(), editor.can_redo());
    let error = editor.execute(command).unwrap_err().to_string();
    assert_eq!(wire(&editor.project), before, "{error}");
    assert_eq!((editor.can_undo(), editor.can_redo()), history);
    error
}

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual:.17} != {expected:.17} within {tolerance}"
    );
}

fn stable_sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let (mut sum, mut compensation) = (0., 0.);
    for value in values {
        let corrected = value - compensation;
        let next = sum + corrected;
        compensation = (next - sum) - corrected;
        sum = next;
    }
    sum
}

fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn polygon(project: &Project, piece: Id) -> Vec<[f64; 2]> {
    project
        .piece(piece)
        .unwrap()
        .boundary
        .iter()
        .map(|id| {
            project
                .point(project.curve(*id).unwrap().start)
                .unwrap()
                .position
        })
        .collect()
}

fn area(points: &[[f64; 2]]) -> f64 {
    stable_sum((1..points.len() - 1).map(|i| cross(points[0], points[i], points[i + 1]) * 0.5))
}

fn inside(point: [f64; 2], points: &[[f64; 2]]) -> bool {
    let mut winding = 0;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        if a[1] <= point[1] && b[1] > point[1] && cross(a, b, point) > 0. {
            winding += 1;
        } else if b[1] <= point[1] && a[1] > point[1] && cross(a, b, point) < 0. {
            winding -= 1;
        }
    }
    winding != 0
}

fn proper_intersection(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2], tolerance: f64) -> bool {
    let opposite =
        |x: f64, y: f64| (x > tolerance && y < -tolerance) || (y > tolerance && x < -tolerance);
    opposite(cross(a, b, c), cross(a, b, d)) && opposite(cross(c, d, a), cross(c, d, b))
}

fn complete_walk(
    project: &Project,
    piece: Id,
    expected: &[Id],
    coordinates: &[[f64; 2]],
    tolerance: f64,
) {
    let actual = project.piece(piece).unwrap();
    assert_eq!(actual.boundary, expected);
    assert_eq!(expected.len(), coordinates.len());
    for (index, id) in expected.iter().enumerate() {
        let curve = project.curve(*id).unwrap();
        assert!(curve.controls.is_none());
        assert_eq!(
            curve.end,
            project
                .curve(expected[(index + 1) % expected.len()])
                .unwrap()
                .start
        );
        for (actual, expected) in project
            .point(curve.start)
            .unwrap()
            .position
            .into_iter()
            .zip(coordinates[index])
        {
            near(actual, expected, tolerance);
        }
    }
}

fn independent_ownership(project: &Project) {
    let (mut curves, mut points) = (BTreeSet::new(), BTreeSet::new());
    for piece in &project.pieces {
        for id in piece.boundary.iter().chain(&piece.internal_curves) {
            assert!(curves.insert(*id));
            let curve = project.curve(*id).unwrap();
            if piece.boundary.contains(id) {
                assert!(points.insert(curve.start));
            } else {
                assert!(points.insert(curve.start));
                assert!(points.insert(curve.end));
            }
        }
    }
    assert_eq!(curves.len(), project.curves.len());
    assert_eq!(points.len(), project.points.len());
}

#[test]
fn original_r240_exact_identity_history_and_independent_child_edits() {
    let mut editor = rectangle();
    let before = editor.project.clone();
    assert_eq!((before.revision, next_id(&before)), (1, 11));
    editor.execute(cut()).unwrap();
    let after = editor.project.clone();
    assert_eq!((after.revision, next_id(&after)), (2, 20));
    assert_eq!(
        after.pieces.iter().map(|p| p.id).collect::<Vec<_>>(),
        [2, 15]
    );
    assert_eq!(
        after.points.iter().map(|p| p.id).collect::<Vec<_>>(),
        [3, 4, 5, 6, 11, 13, 16, 17]
    );
    assert_eq!(
        after.curves.iter().map(|p| p.id).collect::<Vec<_>>(),
        [7, 8, 9, 10, 12, 14, 18, 19]
    );
    complete_walk(
        &after,
        2,
        &[7, 18, 14, 10],
        &[[0., 0.], [0.09, 0.], [0.09, 0.36], [0., 0.36]],
        1e-12,
    );
    complete_walk(
        &after,
        15,
        &[12, 8, 9, 19],
        &[[0.09, 0.], [0.24, 0.], [0.24, 0.36], [0.09, 0.36]],
        1e-12,
    );
    for (id, start, end) in [
        (7, 3, 11),
        (8, 4, 5),
        (9, 5, 17),
        (10, 6, 3),
        (12, 16, 4),
        (14, 13, 6),
        (18, 11, 13),
        (19, 17, 16),
    ] {
        let curve = after.curve(id).unwrap();
        assert_eq!((curve.start, curve.end, curve.controls), (start, end, None));
    }
    for (copy, original) in [(16, 11), (17, 13)] {
        assert_eq!(
            after.point(copy).unwrap().position.map(f64::to_bits),
            after.point(original).unwrap().position.map(f64::to_bits)
        );
    }
    for point in &before.points {
        assert_eq!(
            after.point(point.id).unwrap().position.map(f64::to_bits),
            point.position.map(f64::to_bits)
        );
    }
    assert_eq!(&after.curves[1], &before.curves[1]);
    assert_eq!(&after.curves[3], &before.curves[3]);
    assert_eq!(after.pieces[0].name, "Original Cut R240");
    assert_eq!(after.pieces[1].name, "Cut piece #15");
    assert_eq!(after.materials, before.materials);
    assert_eq!(after.mannequin, before.mannequin);
    independent_ownership(&after);
    assert!(editor.undo());
    assert_eq!((editor.project.revision, next_id(&editor.project)), (3, 20));
    let mut original = allocator(before.clone(), 20);
    original.revision = 3;
    assert_eq!(wire(&editor.project), wire(&original));
    assert!(editor.redo());
    let mut redone = after.clone();
    redone.revision = 4;
    assert_eq!(wire(&editor.project), wire(&redone));
    assert!(editor.undo());
    assert_eq!(editor.project.revision, 5);
    editor.execute(cut()).unwrap();
    assert_eq!((editor.project.revision, next_id(&editor.project)), (6, 29));
    assert_eq!(editor.project.pieces[1].id, 24);
    assert_eq!(editor.project.pieces[0].boundary, [7, 27, 23, 10]);
    assert_eq!(editor.project.pieces[1].boundary, [21, 8, 9, 28]);
    assert!(!editor.can_redo());

    let reopened = Project::from_json(&redone.to_json().unwrap()).unwrap();
    let mut editor = Editor::new(reopened).unwrap();
    assert!(!editor.can_undo() && !editor.can_redo());
    editor
        .execute(Command::ResizePiece {
            piece: 15,
            width: 0.15,
            height: 0.4,
        })
        .unwrap();
    assert_eq!((editor.project.revision, next_id(&editor.project)), (5, 20));
    for id in [3, 4, 6, 11, 13, 16] {
        assert_eq!(
            editor.project.point(id).unwrap().position.map(f64::to_bits),
            after.point(id).unwrap().position.map(f64::to_bits)
        );
    }
    near(editor.project.point(5).unwrap().position[1], 0.4, 1e-12);
    near(editor.project.point(17).unwrap().position[1], 0.4, 1e-12);
    for changed in [2, 15] {
        let mut editor = Editor::new(after.clone()).unwrap();
        let untouched = if changed == 2 { 15 } else { 2 };
        let untouched_piece = editor.project.piece(untouched).unwrap().clone();
        let points: Vec<_> = untouched_piece
            .boundary
            .iter()
            .map(|id| {
                editor
                    .project
                    .point(editor.project.curve(*id).unwrap().start)
                    .unwrap()
                    .clone()
            })
            .collect();
        let bounds = editor.project.piece_bounds(changed).unwrap();
        editor
            .execute(Command::ResizePiece {
                piece: changed,
                width: bounds[1][0] - bounds[0][0],
                height: 0.4,
            })
            .unwrap();
        editor
            .execute(Command::TransformPiece {
                piece: changed,
                translation: [0.03, -0.02],
                angle: 0.23,
                pivot: [0., 0.],
            })
            .unwrap();
        editor
            .execute(Command::RemovePiece { piece: changed })
            .unwrap();
        assert_eq!(editor.project.pieces, [untouched_piece]);
        for point in points {
            assert_eq!(
                editor
                    .project
                    .point(point.id)
                    .unwrap()
                    .position
                    .map(f64::to_bits),
                point.position.map(f64::to_bits)
            );
        }
        editor.project.validate().unwrap();
    }
}

fn check_mesh(project: &Project, mesh: &Mesh, expected: &[(Id, f64)], tolerance: f64) {
    mesh.validate().unwrap();
    let cloth = create_cloth(project, mesh).unwrap();
    assert_eq!(cloth.positions(), mesh.positions);
    assert_eq!(cloth.masses().len(), mesh.positions.len());
    assert_eq!(cloth.simulation_time(), 0.);
    assert!(mesh.stitches.is_empty());
    assert!(mesh.sewing_groups.is_empty());
    let polygons: BTreeMap<_, _> = project
        .pieces
        .iter()
        .map(|piece| (piece.id, polygon(project, piece.id)))
        .collect();
    let mut triangles = BTreeMap::<_, Vec<f64>>::new();
    let mut edge_owners = BTreeMap::new();
    for triangle in &mesh.triangles {
        let owner = mesh.piece_ids[triangle[0]];
        assert!(triangle.iter().all(|v| mesh.piece_ids[*v] == owner));
        let [a, b, c] = triangle.map(|v| mesh.uv[v]);
        let signed = cross(a, b, c) * 0.5;
        assert!(signed > 0.);
        triangles.entry(owner).or_default().push(signed);
        let center = [
            a[0] + ((b[0] - a[0]) + (c[0] - a[0])) / 3.,
            a[1] + ((b[1] - a[1]) + (c[1] - a[1])) / 3.,
        ];
        assert!(inside(center, &polygons[&owner]));
        for (other, points) in &polygons {
            if *other != owner {
                assert!(!inside(center, points));
            }
        }
        for [u, v] in [
            [triangle[0], triangle[1]],
            [triangle[1], triangle[2]],
            [triangle[2], triangle[0]],
        ] {
            let key = if u < v { (u, v) } else { (v, u) };
            assert_eq!(*edge_owners.entry(key).or_insert(owner), owner);
            let points = &polygons[&owner];
            for i in 0..points.len() {
                assert!(!proper_intersection(
                    mesh.uv[u],
                    mesh.uv[v],
                    points[i],
                    points[(i + 1) % points.len()],
                    tolerance
                ));
            }
        }
    }
    let mut masses = Vec::new();
    for &(id, analytic_area) in expected {
        near(area(&polygons[&id]).abs(), analytic_area, tolerance);
        near(
            stable_sum(triangles[&id].iter().copied()),
            analytic_area,
            tolerance,
        );
        let mass = stable_sum(
            cloth
                .masses()
                .iter()
                .enumerate()
                .filter(|(v, _)| mesh.piece_ids[*v] == id)
                .map(|(_, m)| *m),
        );
        near(mass, 0.18 * analytic_area, tolerance);
        masses.push(mass);
    }
    near(
        stable_sum(masses),
        0.18 * stable_sum(expected.iter().map(|(_, a)| *a)),
        tolerance,
    );
    // Every triangle, bend candidate and particle uses indices from just one owner.
    let boundary_vertices: BTreeMap<_, BTreeSet<_>> = project
        .pieces
        .iter()
        .map(|piece| {
            (
                piece.id,
                mesh.boundary
                    .iter()
                    .filter(|s| s.piece == piece.id)
                    .map(|s| s.vertex)
                    .collect(),
            )
        })
        .collect();
    assert!(boundary_vertices[&expected[0].0].is_disjoint(&boundary_vertices[&expected[1].0]));
}

#[test]
fn original_r_l_eligibility_supported_mesh_mass_and_explicit_fine_l_vertex_cap() {
    for resolution in [0.005, 0.07, 0.5] {
        for is_l in [false, true] {
            let mut editor = if is_l { l_shape() } else { rectangle() };
            editor
                .execute(Command::SetResolution { metres: resolution })
                .unwrap();
            let old_mesh = if is_l && resolution == 0.005 {
                assert_eq!(
                    editor.project.mesh().unwrap_err().to_string(),
                    "Refinement exceeds mesh vertex limit; increase resolution"
                );
                None
            } else {
                Some(editor.project.mesh().unwrap())
            };
            if is_l {
                let error = rejected(
                    &mut editor,
                    Command::CutPiece {
                        piece: 2,
                        start_curve: 11,
                        start_t: 1. / 3.,
                        end_curve: 12,
                        end_t: 2. / 3.,
                    },
                );
                assert!(
                    error.contains("midpoint must be strictly inside"),
                    "{error}"
                );
                editor
                    .execute(Command::CutPiece {
                        piece: 2,
                        start_curve: 10,
                        start_t: 0.5,
                        end_curve: 14,
                        end_t: 0.8,
                    })
                    .unwrap();
                complete_walk(
                    &editor.project,
                    2,
                    &[9, 10, 22, 18],
                    &[[0., 0.], [1., 0.], [1., 0.2], [0., 0.2]],
                    1e-12,
                );
                complete_walk(
                    &editor.project,
                    19,
                    &[16, 11, 12, 13, 14, 23],
                    &[
                        [1., 0.2],
                        [1., 0.4],
                        [0.4, 0.4],
                        [0.4, 1.],
                        [0., 1.],
                        [0., 0.2],
                    ],
                    1e-12,
                );
                assert_eq!(next_id(&editor.project), 24);
            } else {
                editor.execute(cut()).unwrap();
            }
            if let Some(old_mesh) = old_mesh {
                assert!(create_cloth(&editor.project, &old_mesh).is_err());
            }
            let expected = if is_l {
                [(2, 0.2), (19, 0.44)]
            } else {
                [(2, 0.0324), (15, 0.054)]
            };
            if is_l && resolution == 0.005 {
                // shortcut: this original fine L exceeds the existing mesher cap; add mass acceptance when refinement supports it.
                assert_eq!(
                    editor.project.mesh().unwrap_err().to_string(),
                    "Refinement exceeds mesh vertex limit; increase resolution"
                );
                for (id, expected) in expected {
                    near(area(&polygon(&editor.project, id)).abs(), expected, 1e-12);
                }
            } else {
                check_mesh(
                    &editor.project,
                    &editor.project.mesh().unwrap(),
                    &expected,
                    1e-12,
                );
            }
            independent_ownership(&editor.project);
            let total_perimeter = stable_sum(
                editor
                    .project
                    .pieces
                    .iter()
                    .flat_map(|p| &p.boundary)
                    .map(|id| {
                        editor
                            .project
                            .interval_length(CurveInterval::full(*id))
                            .unwrap()
                    }),
            );
            near(total_perimeter, if is_l { 6. } else { 1.92 }, 1e-12);
            let (first, second) = if is_l { (22, 23) } else { (18, 19) };
            let endpoints = |id| {
                let c = editor.project.curve(id).unwrap();
                [c.start, c.end].map(|p| editor.project.point(p).unwrap().position)
            };
            let [a, b] = endpoints(first);
            let [c, d] = endpoints(second);
            assert_eq!(a.map(f64::to_bits), d.map(f64::to_bits));
            assert_eq!(b.map(f64::to_bits), c.map(f64::to_bits));
            near(
                editor
                    .project
                    .interval_length(CurveInterval::full(first))
                    .unwrap(),
                if is_l { 1. } else { 0.36 },
                1e-12,
            );
        }
    }
}

#[test]
fn tilted_and_large_offset_arrangement_and_untouched_float_bits_survive() {
    for offset in [[0., 0.], [999., -999.]] {
        let mut project = rectangle().project;
        for point in &mut project.points {
            for (axis, delta) in offset.into_iter().enumerate() {
                point.position[axis] += delta;
            }
        }
        project.pieces[0].placement = Placement {
            rotation: [0.31, -0.47, 0.21],
            translation: [0.2, 1.4, -0.3],
        };
        project.pieces[0].grain_angle = 0.37;
        project.materials[0].damping = -0.0;
        let old = project.clone();
        let mut editor = Editor::new(project).unwrap();
        editor.execute(cut()).unwrap();
        for piece in &editor.project.pieces {
            assert_eq!(
                piece.placement.rotation.map(f64::to_bits),
                old.pieces[0].placement.rotation.map(f64::to_bits)
            );
            assert_eq!(
                piece.placement.translation.map(f64::to_bits),
                old.pieces[0].placement.translation.map(f64::to_bits)
            );
            assert_eq!(
                piece.grain_angle.to_bits(),
                old.pieces[0].grain_angle.to_bits()
            );
            assert_eq!(piece.material, old.pieces[0].material);
        }
        assert_eq!(
            editor.project.materials[0].damping.to_bits(),
            (-0.0_f64).to_bits()
        );
        for point in &old.points {
            assert_eq!(
                editor
                    .project
                    .point(point.id)
                    .unwrap()
                    .position
                    .map(f64::to_bits),
                point.position.map(f64::to_bits)
            );
        }
        let tolerance = if offset[0] == 0. { 1e-12 } else { 1e-9 };
        let mesh = editor.project.mesh().unwrap();
        check_mesh(
            &editor.project,
            &mesh,
            &[(2, 0.0324), (15, 0.054)],
            tolerance,
        );
        for (vertex, uv) in mesh.uv.iter().enumerate() {
            let expected = old.pieces[0].placement.transform(*uv);
            for (actual, expected) in mesh.positions[vertex].into_iter().zip(expected) {
                near(actual, expected, 1e-10);
            }
        }
        for (copy, source) in [(16, 11), (17, 13)] {
            assert_eq!(
                editor
                    .project
                    .point(copy)
                    .unwrap()
                    .position
                    .map(f64::to_bits),
                editor
                    .project
                    .point(source)
                    .unwrap()
                    .position
                    .map(f64::to_bits)
            );
        }
    }
}

#[test]
fn swapped_addresses_are_identical_and_clockwise_winding_is_preserved() {
    let mut first = rectangle();
    let mut second = rectangle();
    first.execute(cut()).unwrap();
    second
        .execute(Command::CutPiece {
            piece: 2,
            start_curve: 9,
            start_t: 0.625,
            end_curve: 7,
            end_t: 0.375,
        })
        .unwrap();
    assert_eq!(wire(&first.project), wire(&second.project));
    let mut clockwise = Editor::default();
    clockwise
        .execute(Command::AddPolygon {
            name: "Clockwise".into(),
            vertices: vec![[0., -0.0], [0., 0.36], [0.24, 0.36], [0.24, 0.]],
        })
        .unwrap();
    clockwise.project.pieces[0].grain_angle = -0.;
    clockwise.project.pieces[0].placement.translation = [-0., 0., -0.];
    clockwise.execute(cut()).unwrap();
    for piece in &clockwise.project.pieces {
        assert!(area(&polygon(&clockwise.project, piece.id)) < 0.);
        assert_eq!(piece.grain_angle.to_bits(), (-0.0_f64).to_bits());
        assert_eq!(
            piece.placement.translation.map(f64::to_bits),
            [-0.0_f64, 0., -0.].map(f64::to_bits)
        );
    }
    check_mesh(
        &clockwise.project,
        &clockwise.project.mesh().unwrap(),
        &[(2, 0.0324), (15, 0.054)],
        1e-12,
    );
}

fn annotation(start: [f64; 2], end: [f64; 2]) -> Command {
    Command::AddInternalSegment {
        piece: 2,
        start,
        end,
    }
}

#[test]
fn annotations_move_whole_in_original_order_or_refuse_the_entire_cut() {
    let mut editor = rectangle();
    editor
        .execute(Command::Transaction(vec![
            annotation([0.02, 0.1], [0.06, 0.3]),
            annotation([0.12, 0.1], [0.21, 0.3]),
        ]))
        .unwrap();
    let before = editor.project.clone();
    assert_eq!(before.pieces[0].internal_curves, [13, 16]);
    assert_eq!(next_id(&before), 17);
    editor.execute(cut()).unwrap();
    assert_eq!((editor.project.revision, next_id(&editor.project)), (3, 26));
    assert_eq!(editor.project.pieces[1].id, 21);
    assert_eq!(editor.project.pieces[0].boundary, [7, 24, 20, 10]);
    assert_eq!(editor.project.pieces[1].boundary, [18, 8, 9, 25]);
    assert_eq!(editor.project.pieces[0].internal_curves, [13]);
    assert_eq!(editor.project.pieces[1].internal_curves, [16]);
    for id in [13, 16] {
        let curve = before.curve(id).unwrap();
        assert_eq!(editor.project.curve(id).unwrap(), curve);
        for id in [curve.start, curve.end] {
            assert_eq!(
                editor.project.point(id).unwrap().position.map(f64::to_bits),
                before.point(id).unwrap().position.map(f64::to_bits)
            );
        }
    }
    independent_ownership(&editor.project);
    let mut ordered = rectangle();
    ordered
        .execute(Command::Transaction(vec![
            annotation([0.02, 0.1], [0.06, 0.3]),
            annotation([0.12, 0.1], [0.21, 0.3]),
            annotation([0.03, 0.1], [0.05, 0.2]),
            annotation([0.13, 0.1], [0.2, 0.2]),
        ]))
        .unwrap();
    ordered.execute(cut()).unwrap();
    assert_eq!(ordered.project.pieces[0].internal_curves, [13, 19]);
    assert_eq!(ordered.project.pieces[1].internal_curves, [16, 22]);
    for (start, end) in [
        ([0.05, 0.18], [0.13, 0.18]),
        ([0.09, 0.1], [0.09, 0.3]),
        ([0.0900005, 0.1], [0.0900005, 0.3]),
        ([0.0900011, 0.1], [0.0900011, 0.3]),
    ] {
        let mut editor = rectangle();
        editor.execute(annotation(start, end)).unwrap();
        assert!(rejected(&mut editor, cut()).contains("cannot be assigned unchanged"));
        assert_eq!((editor.project.revision, next_id(&editor.project)), (2, 14));
    }
}

fn partner(editor: &mut Editor, x: f64) {
    editor
        .execute(Command::AddRectangle {
            name: "Partner".into(),
            origin: [x, 0.],
            width: 0.24,
            height: 0.36,
        })
        .unwrap();
}

#[test]
fn every_target_sewing_reference_refuses_and_other_piece_relationships_stay_exact() {
    for group in [false, true] {
        let mut editor = rectangle();
        partner(&mut editor, 0.4);
        let a = CurveInterval {
            curve: if group { 8 } else { 7 },
            t0: 0.05,
            t1: 0.1,
        };
        let b = CurveInterval {
            curve: if group { 17 } else { 16 },
            t0: 0.05,
            t1: 0.1,
        };
        editor
            .execute(if group {
                Command::AddSewingGroup {
                    name: "Target remote group".into(),
                    a: vec![a],
                    b: vec![b],
                    correspondence: vec![[0., 0.], [1., 1.]],
                }
            } else {
                Command::AddSeam {
                    name: "Target remote legacy".into(),
                    a,
                    b,
                    correspondence: vec![[0., 0.], [1., 1.]],
                }
            })
            .unwrap();
        assert_eq!(next_id(&editor.project), 21);
        assert!(rejected(&mut editor, cut()).contains("sewing on any boundary"));
    }
    let mut editor = rectangle();
    partner(&mut editor, 0.4);
    partner(&mut editor, 0.8);
    editor
        .execute(Command::AddSeam {
            name: "Other reversed partial legacy".into(),
            a: CurveInterval {
                curve: 16,
                t0: 0.125,
                t1: 0.875,
            },
            b: CurveInterval {
                curve: 25,
                t0: 0.875,
                t1: 0.125,
            },
            correspondence: vec![[0., 0.], [0.25, 0.625], [1., 1.]],
        })
        .unwrap();
    editor
        .execute(Command::AddSewingGroup {
            name: "Other reversed partial group".into(),
            a: vec![CurveInterval {
                curve: 17,
                t0: 0.125,
                t1: 0.875,
            }],
            b: vec![CurveInterval {
                curve: 26,
                t0: 0.875,
                t1: 0.125,
            }],
            correspondence: vec![[0., 0.], [0.5, 0.75], [1., 1.]],
        })
        .unwrap();
    let before = editor.project.clone();
    editor.execute(cut()).unwrap();
    assert_eq!((editor.project.revision, next_id(&editor.project)), (6, 40));
    assert_eq!(editor.project.pieces[3].id, 35);
    assert_eq!(editor.project.seams, before.seams);
    assert_eq!(editor.project.sewing_groups, before.sewing_groups);
    assert_eq!(editor.project.pieces[1..3], before.pieces[1..3]);
    for point in &before.points {
        assert_eq!(
            editor
                .project
                .point(point.id)
                .unwrap()
                .position
                .map(f64::to_bits),
            point.position.map(f64::to_bits)
        );
    }
    for curve in before.curves.iter().filter(|c| c.id >= 16) {
        assert_eq!(editor.project.curve(curve.id).unwrap(), curve);
    }
    editor.project.mesh().unwrap();
}

#[test]
fn invalid_addresses_numeric_geometry_and_late_transaction_are_atomic() {
    let mut editor = rectangle();
    partner(&mut editor, 0.4);
    editor
        .execute(annotation([0.02, 0.1], [0.06, 0.3]))
        .unwrap();
    editor
        .execute(Command::SetResolution { metres: 0.08 })
        .unwrap();
    assert!(editor.undo());
    for (piece, a, t, b, u) in [
        (2, 7, 0.5, 7, 0.6),
        (2, 7, 0., 9, 0.625),
        (2, 7, 1., 9, 0.625),
        (2, 7, 1e-6, 9, 0.625),
        (2, 7, 1. - 1e-6, 9, 0.625),
        (2, 7, 4e-6, 9, 0.625),
        (2, 7, -0.1, 9, 0.625),
        (2, 7, f64::NAN, 9, 0.625),
        (2, 7, f64::INFINITY, 9, 0.625),
        (2, 7, 0.375, 9, f64::NEG_INFINITY),
        (2, 16, 0.375, 9, 0.625),
        (2, 22, 0.375, 9, 0.625),
        (2, u64::MAX, 0.5, 9, 0.625),
        (2, 0, 0.5, 9, 0.625),
        (0, 7, 0.375, 9, 0.625),
        (u64::MAX, 7, 0.375, 9, 0.625),
        (2, 7, 1. - 1e-5, 8, 1e-5),
    ] {
        rejected(
            &mut editor,
            Command::CutPiece {
                piece,
                start_curve: a,
                start_t: t,
                end_curve: b,
                end_t: u,
            },
        );
    }
    rejected(
        &mut editor,
        Command::Transaction(vec![cut(), Command::SetResolution { metres: 0. }]),
    );
    assert!(
        rejected(
            &mut editor,
            Command::CutPiece {
                piece: 2,
                start_curve: 7,
                start_t: 1. - 1e-5,
                end_curve: 8,
                end_t: 1e-5,
            },
        )
        .contains("near-zero area")
    );
    assert!(
        rejected(
            &mut editor,
            Command::CutPiece {
                piece: 2,
                start_curve: 7,
                start_t: 1. - 1.1e-6 / 0.24,
                end_curve: 8,
                end_t: 1.1e-6 / 0.36,
            },
        )
        .contains("chord is too short")
    );
    assert!(editor.can_redo());
    assert!(editor.redo());
    assert_eq!(editor.project.resolution, 0.08);
    let mut curved = rectangle();
    curved
        .execute(Command::SetCurveControls {
            curve: 7,
            controls: Some([[0.08, 0.], [0.16, 0.]]),
        })
        .unwrap();
    rejected(&mut curved, cut());
    let mut collinear = Editor::default();
    collinear
        .execute(Command::AddPolygon {
            name: "Collinear separate edges".into(),
            vertices: vec![[0., 0.], [0.12, 0.], [0.24, 0.], [0.24, 0.36], [0., 0.36]],
        })
        .unwrap();
    assert!(
        rejected(
            &mut collinear,
            Command::CutPiece {
                piece: 2,
                start_curve: 8,
                start_t: 0.5,
                end_curve: 9,
                end_t: 0.5
            }
        )
        .contains("support line")
    );
    let mut nonsimple = rectangle();
    let error = rejected(
        &mut nonsimple,
        Command::Transaction(vec![
            Command::MovePoint {
                point: 4,
                position: [-0.1, 0.2],
            },
            cut(),
        ]),
    );
    assert!(error.contains("Cut contour"), "{error}");
}

fn regular_polygon(n: usize, x: f64) -> Command {
    Command::AddPolygon {
        name: "Budget contour".into(),
        vertices: (0..n)
            .map(|i| {
                let angle = i as f64 * std::f64::consts::TAU / n as f64;
                [x + angle.cos() * 0.5, angle.sin() * 0.5]
            })
            .collect(),
    }
}

#[test]
fn id_object_revision_and_shared_outer_transaction_work_caps_are_atomic() {
    let mut limit = Editor::new(allocator(rectangle().project, u64::MAX - 8)).unwrap();
    assert!(rejected(&mut limit, cut()).contains("stable ID limit"));
    let mut last = Editor::new(allocator(rectangle().project, u64::MAX - 9)).unwrap();
    last.execute(cut()).unwrap();
    assert_eq!(next_id(&last.project), u64::MAX);
    let mut old = rectangle().project;
    old.revision = u64::MAX;
    let mut editor = Editor::new(old).unwrap();
    assert!(rejected(&mut editor, cut()).contains("revision limit"));
    let mut project = rectangle().project;
    let mut id = 11;
    while project.points.len()
        + project.curves.len()
        + project.pieces.len()
        + project.materials.len()
        < MAX_OBJECTS - 8
    {
        project.materials.push(Material {
            id,
            ..Material::default()
        });
        id += 1;
    }
    let mut editor = Editor::new(allocator(project, id)).unwrap();
    assert!(rejected(&mut editor, cut()).contains("object limit"));
    let mut project = editor.project.clone();
    project.materials.pop();
    let mut exact = Editor::new(project).unwrap();
    exact.execute(cut()).unwrap();
    assert_eq!(
        exact.project.points.len()
            + exact.project.curves.len()
            + exact.project.pieces.len()
            + exact.project.materials.len(),
        MAX_OBJECTS
    );

    let mut editor = Editor::default();
    editor.execute(regular_polygon(800, 0.)).unwrap();
    editor.execute(regular_polygon(800, 2.)).unwrap();
    let commands: Vec<_> = editor
        .project
        .pieces
        .iter()
        .map(|piece| Command::CutPiece {
            piece: piece.id,
            start_curve: piece.boundary[0],
            start_t: 0.5,
            end_curve: piece.boundary[400],
            end_t: 0.5,
        })
        .collect();
    for command in &commands {
        Editor::new(editor.project.clone())
            .unwrap()
            .execute(command.clone())
            .unwrap();
    }
    assert!(
        rejected(&mut editor, Command::Transaction(commands)).contains("work budget exhausted")
    );
    let mut oversized = Editor::default();
    oversized.execute(regular_polygon(4096, 0.)).unwrap();
    let curve = oversized.project.pieces[0].boundary[0];
    oversized
        .execute(Command::SplitCurve { curve, t: 0.5 })
        .unwrap();
    let end = oversized.project.pieces[0].boundary[2000];
    assert!(
        rejected(
            &mut oversized,
            Command::CutPiece {
                piece: 2,
                start_curve: curve,
                start_t: 0.5,
                end_curve: end,
                end_t: 0.5
            }
        )
        .contains("3 to 4096")
    );
}

#[test]
fn cut_wire_envelope_required_fields_and_original_duplicate_bytes_are_strict() {
    let valid = json!({"type":"cut_piece","data":{"piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}});
    assert_eq!(serde_json::to_value(cut()).unwrap(), valid);
    for field in ["piece", "start_curve", "start_t", "end_curve", "end_t"] {
        let mut missing = valid.clone();
        missing["data"].as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<Command>(missing).is_err());
        let raw = valid.to_string();
        let needle = format!("\"{field}\":{}", valid["data"][field]);
        assert!(
            serde_json::from_str::<Command>(&raw.replace(&needle, &format!("{needle},{needle}")))
                .is_err()
        );
    }
    for raw in [
        r#"{"type":"cut_piece","piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}"#,
        r#"{"type":"cut_piece","type":"cut_piece","data":{"piece":2,"start_curve":7,"start_t":0.375,"end_curve":9,"end_t":0.625}}"#,
        r#"{"type":"cut_piece","data":{"piece":2,"start_curve":7,"start_t":1e309,"end_curve":9,"end_t":0.625}}"#,
    ] {
        assert!(serde_json::from_str::<Command>(raw).is_err());
    }
    for (field, value) in [
        ("unknown", json!(0)),
        ("piece", json!(-1)),
        ("piece", json!(2.5)),
        ("start_t", json!(null)),
    ] {
        let mut bad = valid.clone();
        bad["data"][field] = value;
        assert!(serde_json::from_value::<Command>(bad).is_err());
    }
    let mut outer = valid;
    outer["unknown"] = json!(0);
    assert!(serde_json::from_value::<Command>(outer).is_err());
}

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "weftforge-cut-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn ordinary_v4_exports_have_both_exterior_copies_independent_obj_components_and_safe_io() {
    let directory = Directory::new();
    let mut editor = rectangle();
    editor
        .execute(Command::Transaction(vec![
            annotation([0.02, 0.1], [0.06, 0.3]),
            annotation([0.12, 0.1], [0.21, 0.3]),
        ]))
        .unwrap();
    let primary = directory.0.join("cut.weftforge");
    editor.project.save(&primary).unwrap();
    let prior = fs::read(&primary).unwrap();
    editor.execute(cut()).unwrap();
    let accepted = wire(&editor.project);
    fs::create_dir(recovery_path(&primary)).unwrap();
    assert!(editor.project.save(&primary).is_err());
    assert_eq!(fs::read(&primary).unwrap(), prior);
    assert_eq!(wire(&editor.project), accepted);
    fs::remove_dir(recovery_path(&primary)).unwrap();
    editor.project.save(&primary).unwrap();
    assert_eq!(fs::read(recovery_path(&primary)).unwrap(), prior);
    let bytes = fs::read(&primary).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["version"],
        4
    );
    let loaded = Project::load(&primary).unwrap();
    assert_eq!(wire(&loaded), accepted);
    assert!(Project::from_json(&bytes[..bytes.len() / 2]).is_err());
    assert_eq!(fs::read(&primary).unwrap(), bytes);
    let svg = directory.0.join("cut.svg");
    let dxf = directory.0.join("cut.dxf");
    let obj = directory.0.join("cut.obj");
    loaded.export_svg(&svg).unwrap();
    loaded.export_dxf(&dxf).unwrap();
    let mesh = loaded.mesh().unwrap();
    mesh.export_obj(&obj).unwrap();
    let svg = fs::read_to_string(svg).unwrap();
    let paths: Vec<_> = svg
        .lines()
        .filter(|line| line.starts_with("<path "))
        .collect();
    assert_eq!(paths.len(), 2);
    for (path, id) in paths.iter().zip([2, 21]) {
        assert!(path.contains(&format!("id=\"piece-{id}\"")));
        let data = path
            .split_once(" d=\"")
            .unwrap()
            .1
            .split('"')
            .next()
            .unwrap();
        let tokens: Vec<_> = data.split_whitespace().collect();
        assert_eq!(tokens.len(), 16);
        assert_eq!(tokens[0], "M");
        assert_eq!(tokens[15], "Z");
        for i in 1..5 {
            assert_eq!(tokens[3 * i], "L");
        }
        let actual: Vec<[f64; 2]> = tokens[..15]
            .chunks_exact(3)
            .map(|t| {
                [
                    t[1].parse::<f64>().unwrap() / 1000.,
                    -t[2].parse::<f64>().unwrap() / 1000.,
                ]
            })
            .collect();
        let expected = polygon(&loaded, id);
        for (i, p) in actual.iter().enumerate() {
            for axis in 0..2 {
                near(p[axis], expected[i % 4][axis], 1e-12);
            }
        }
    }
    let dxf = fs::read_to_string(dxf).unwrap();
    let lines: Vec<_> = dxf.lines().collect();
    let pairs: Vec<_> = lines.chunks_exact(2).map(|p| (p[0], p[1])).collect();
    let start = pairs
        .windows(2)
        .position(|p| p == [("0", "SECTION"), ("2", "ENTITIES")])
        .unwrap()
        + 2;
    let mut records = Vec::new();
    for &(code, value) in &pairs[start..] {
        if code == "0" && value == "ENDSEC" {
            break;
        }
        if code == "0" {
            assert_eq!(value, "LINE");
            records.push(BTreeMap::new());
        }
        records.last_mut().unwrap().insert(code, value);
    }
    assert_eq!(records.len(), 8);
    for (index, record) in records.iter().enumerate() {
        let id = if index < 4 { 2 } else { 21 };
        assert_eq!(record["8"], format!("WF_PIECE_{id}"));
        let expected = polygon(&loaded, id);
        for (codes, p) in [
            (["10", "20"], expected[index % 4]),
            (["11", "21"], expected[(index + 1) % 4]),
        ] {
            for axis in 0..2 {
                near(
                    record[codes[axis]].parse::<f64>().unwrap() / 1000.,
                    p[axis],
                    1e-12,
                );
            }
        }
    }
    let obj = fs::read_to_string(obj).unwrap();
    let mut owner = None;
    let mut groups = BTreeMap::<Id, BTreeSet<usize>>::new();
    let mut adjacency = BTreeMap::<usize, BTreeSet<usize>>::new();
    for line in obj.lines() {
        if let Some(group) = line.strip_prefix("g piece_") {
            owner = Some(group.parse::<Id>().unwrap());
        }
        if let Some(face) = line.strip_prefix("f ") {
            let triangle: Vec<_> = face
                .split_whitespace()
                .map(|v| v.split('/').next().unwrap().parse::<usize>().unwrap())
                .collect();
            assert_eq!(triangle.len(), 3);
            for i in 0..3 {
                groups
                    .entry(owner.unwrap())
                    .or_default()
                    .insert(triangle[i]);
                adjacency
                    .entry(triangle[i])
                    .or_default()
                    .insert(triangle[(i + 1) % 3]);
                adjacency
                    .entry(triangle[(i + 1) % 3])
                    .or_default()
                    .insert(triangle[i]);
            }
        }
        assert!(!line.starts_with("l "));
    }
    assert_eq!(groups.keys().copied().collect::<Vec<_>>(), [2, 21]);
    assert!(groups[&2].is_disjoint(&groups[&21]));
    for vertices in groups.values() {
        let mut pending = vec![*vertices.first().unwrap()];
        let mut seen = BTreeSet::new();
        while let Some(vertex) = pending.pop() {
            if seen.insert(vertex) {
                pending.extend(adjacency[&vertex].iter().copied());
            }
        }
        assert_eq!(&seen, vertices);
    }
    assert_eq!(fs::read(&primary).unwrap(), bytes);
}
