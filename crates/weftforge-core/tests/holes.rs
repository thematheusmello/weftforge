use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use weftforge_core::{
    Command, Curve, CurveInterval, Editor, Hole, Id, Material, Mesh, Placement, Point, Project,
    SewingDirection, SewingSideByLength, simulation::create_cloth,
};

// Literal inputs and analytic answers are from the independently reviewed R240 contract.
const HOLE: [[f64; 2]; 4] = [[0.08, 0.12], [0.08, 0.24], [0.16, 0.24], [0.16, 0.12]];

fn rectangle() -> Editor {
    let mut editor = Editor::default();
    editor
        .execute(Command::AddRectangle {
            name: "R240".into(),
            origin: [0., 0.],
            width: 0.24,
            height: 0.36,
        })
        .unwrap();
    editor
}

fn add_hole(vertices: impl Into<Vec<[f64; 2]>>) -> Command {
    Command::AddHole {
        piece: 2,
        vertices: vertices.into(),
    }
}

fn annulus() -> Editor {
    let mut editor = rectangle();
    editor.execute(add_hole(HOLE)).unwrap();
    editor
}

fn next_id(project: &Project) -> Id {
    serde_json::to_value(project).unwrap()["next_id"]
        .as_u64()
        .unwrap()
}

fn with_next_id(project: Project, id: Id) -> Project {
    let mut value = serde_json::to_value(project).unwrap();
    value["next_id"] = json!(id);
    serde_json::from_value(value).unwrap()
}

fn rejected(editor: &mut Editor, command: Command) -> String {
    let before = editor.project.clone();
    let history = (editor.can_undo(), editor.can_redo());
    let result = editor.execute(command.clone());
    assert!(result.is_err(), "Unexpected success: {command:?}");
    let error = result.unwrap_err().to_string();
    assert_eq!(editor.project, before, "{error}");
    assert_eq!((editor.can_undo(), editor.can_redo()), history);
    error
}

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual:.17} != {expected:.17}"
    );
}

fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn area(points: &[[f64; 2]]) -> f64 {
    (1..points.len().saturating_sub(1))
        .map(|i| cross(points[0], points[i], points[i + 1]) * 0.5)
        .sum()
}

fn coordinates(project: &Project, boundary: &[Id]) -> Vec<[f64; 2]> {
    boundary
        .iter()
        .map(|id| {
            project
                .point(project.curve(*id).unwrap().start)
                .unwrap()
                .position
        })
        .collect()
}

// Independent half-plane clipping checks entire faces, including overlap invisible to centroid tests.
fn intersection(mut subject: Vec<[f64; 2]>, clip: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let direction = area(clip).signum();
    for i in 0..clip.len() {
        if subject.is_empty() {
            break;
        }
        let (a, b) = (clip[i], clip[(i + 1) % clip.len()]);
        let input = std::mem::take(&mut subject);
        let mut previous = *input.last().unwrap();
        let mut previous_distance = direction * cross(a, b, previous);
        for current in input {
            let current_distance = direction * cross(a, b, current);
            if (current_distance >= 0.) != (previous_distance >= 0.) {
                let t = previous_distance / (previous_distance - current_distance);
                subject.push(std::array::from_fn(|axis| {
                    previous[axis] + t * (current[axis] - previous[axis])
                }));
            }
            if current_distance >= 0. {
                subject.push(current);
            }
            previous = current;
            previous_distance = current_distance;
        }
    }
    subject
}

fn components(adjacency: &BTreeMap<usize, BTreeSet<usize>>) -> usize {
    let mut remaining: BTreeSet<_> = adjacency.keys().copied().collect();
    let mut count = 0;
    while let Some(&first) = remaining.first() {
        count += 1;
        let mut stack = vec![first];
        while let Some(vertex) = stack.pop() {
            if remaining.remove(&vertex) {
                stack.extend(&adjacency[&vertex]);
            }
        }
    }
    count
}

fn inspect_annulus(project: &Project, mesh: &Mesh, hole: &[[f64; 2]], expected_area: f64) {
    mesh.validate().unwrap();
    assert_eq!(mesh.doc_revision, project.revision);
    assert!(mesh.stitches.is_empty());
    let mut edges = BTreeMap::<(usize, usize), usize>::new();
    let mut adjacency = BTreeMap::<usize, BTreeSet<usize>>::new();
    let mut boxes = Vec::new();
    let mut total_area = 0.;
    for (index, triangle) in mesh.triangles.iter().enumerate() {
        let points = triangle.map(|vertex| mesh.uv[vertex]);
        let signed_area = cross(points[0], points[1], points[2]) * 0.5;
        assert!(signed_area > 0.);
        total_area += signed_area;
        for (&vertex, point) in triangle.iter().zip(points) {
            assert_eq!(mesh.piece_ids[vertex], 2);
            assert!(point[0] >= -1e-12 && point[0] <= 0.24 + 1e-12);
            assert!(point[1] >= -1e-12 && point[1] <= 0.36 + 1e-12);
            for (actual, expected) in mesh.positions[vertex]
                .into_iter()
                .zip([point[0], point[1], 0.])
            {
                near(actual, expected, 1e-12);
            }
        }
        near(area(&intersection(points.to_vec(), hole)).abs(), 0., 1e-14);
        for i in 0..3 {
            let (a, b) = (triangle[i], triangle[(i + 1) % 3]);
            *edges.entry((a.min(b), a.max(b))).or_default() += 1;
            adjacency.entry(a).or_default().insert(b);
            adjacency.entry(b).or_default().insert(a);
        }
        boxes.push((
            points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min),
            points
                .iter()
                .map(|p| p[0])
                .fold(f64::NEG_INFINITY, f64::max),
            points.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min),
            points
                .iter()
                .map(|p| p[1])
                .fold(f64::NEG_INFINITY, f64::max),
            index,
        ));
    }
    near(total_area, expected_area, 1e-10);
    assert_eq!(components(&adjacency), 1);
    assert_eq!(
        mesh.uv.len() as isize - edges.len() as isize + mesh.triangles.len() as isize,
        0
    );
    assert!(edges.values().all(|count| *count == 1 || *count == 2));
    boxes.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (index, a) in boxes.iter().enumerate() {
        for b in boxes.iter().skip(index + 1) {
            if b.0 >= a.1 {
                break;
            }
            if b.2 >= a.3 || b.3 <= a.2 {
                continue;
            }
            let first = mesh.triangles[a.4].map(|i| mesh.uv[i]);
            let second = mesh.triangles[b.4].map(|i| mesh.uv[i]);
            near(
                area(&intersection(first.to_vec(), &second)).abs(),
                0.,
                1e-14,
            );
        }
    }
    let mut boundary_adjacency = BTreeMap::<usize, BTreeSet<usize>>::new();
    let actual: BTreeSet<_> = edges
        .iter()
        .filter_map(|(&edge, &count)| (count == 1).then_some(edge))
        .collect();
    for &(a, b) in &actual {
        boundary_adjacency.entry(a).or_default().insert(b);
        boundary_adjacency.entry(b).or_default().insert(a);
    }
    assert!(
        boundary_adjacency
            .values()
            .all(|neighbors| neighbors.len() == 2),
        "Boundary branches: {:?}",
        boundary_adjacency
            .iter()
            .filter(|(_, neighbors)| neighbors.len() != 2)
            .map(|(vertex, neighbors)| (
                *vertex,
                mesh.uv[*vertex],
                neighbors
                    .iter()
                    .map(|other| (*other, mesh.uv[*other]))
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(components(&boundary_adjacency), 2);
    let mut associated = BTreeSet::new();
    for boundary in project.pieces[0].boundary_loops() {
        for &id in boundary {
            let curve = project.curve(id).unwrap();
            let mut samples: Vec<_> = mesh
                .boundary
                .iter()
                .filter(|sample| sample.curve == id)
                .collect();
            samples.sort_by(|a, b| a.t.total_cmp(&b.t));
            assert_eq!(samples.first().unwrap().t, 0.);
            assert_eq!(samples.last().unwrap().t, 1.);
            for sample in &samples {
                assert_eq!(sample.piece, 2);
                for (actual, expected) in mesh.uv[sample.vertex]
                    .into_iter()
                    .zip(curve.evaluate(project, sample.t).unwrap())
                {
                    near(actual, expected, 1e-12);
                }
            }
            for pair in samples.windows(2) {
                assert!(pair[0].t < pair[1].t);
                let [a, b] = [pair[0].vertex, pair[1].vertex];
                assert!(associated.insert((a.min(b), a.max(b))));
            }
        }
    }
    assert_eq!(
        associated, actual,
        "An earcut bridge must never become a material boundary"
    );
    let mut perimeters = [0., 0.];
    for &(a, b) in &actual {
        let outer = mesh.uv[a][0].abs() < 1e-12
            || (mesh.uv[a][0] - 0.24).abs() < 1e-12
            || mesh.uv[a][1].abs() < 1e-12
            || (mesh.uv[a][1] - 0.36).abs() < 1e-12;
        perimeters[usize::from(!outer)] +=
            (mesh.uv[a][0] - mesh.uv[b][0]).hypot(mesh.uv[a][1] - mesh.uv[b][1]);
    }
    near(perimeters[0], 1.2, 1e-12);
    let hole_perimeter: f64 = (0..hole.len())
        .map(|i| {
            let (a, b) = (hole[i], hole[(i + 1) % hole.len()]);
            (a[0] - b[0]).hypot(a[1] - b[1])
        })
        .sum();
    near(perimeters[1], hole_perimeter, 1e-12);
}

#[test]
fn actual_annular_mesh_and_cloth_match_analytic_domain_topology_area_and_mass() {
    for resolution in [0.005, 0.07, 0.5] {
        for (edited, expected_area, expected_mass) in
            [(false, 0.0768, 0.013824), (true, 0.0756, 0.013608)]
        {
            let mut editor = rectangle();
            editor
                .execute(Command::SetResolution { metres: resolution })
                .unwrap();
            let old_mesh = editor.project.mesh().unwrap();
            editor.execute(add_hole(HOLE)).unwrap();
            let mut hole = HOLE;
            if edited {
                editor
                    .execute(Command::MovePoint {
                        point: 12,
                        position: [0.06, 0.12],
                    })
                    .unwrap();
                hole[0] = [0.06, 0.12];
            }
            assert!(
                create_cloth(&editor.project, &old_mesh)
                    .unwrap_err()
                    .to_string()
                    .contains("obsolete")
            );
            let mesh = editor.project.mesh().unwrap();
            inspect_annulus(&editor.project, &mesh, &hole, expected_area);
            let mut cloth = create_cloth(&editor.project, &mesh).unwrap();
            near(cloth.masses().iter().sum(), expected_mass, 1e-10);
            assert_eq!(cloth.positions(), mesh.positions);
            assert!(
                cloth
                    .masses()
                    .iter()
                    .all(|mass| mass.is_finite() && *mass > 0.)
            );
            cloth.reset();
            assert_eq!(cloth.positions(), mesh.positions);
            assert!(
                cloth
                    .velocities()
                    .iter()
                    .flatten()
                    .all(|value| *value == 0.)
            );
            near(cloth.masses().iter().sum(), expected_mass, 1e-10);
        }
    }
}

#[test]
fn exact_stable_identity_edit_history_split_join_remove_and_abandoned_branch() {
    let mut editor = rectangle();
    let old = editor.project.clone();
    assert_eq!((old.revision, next_id(&old)), (1, 11));
    assert!(old.pieces[0].holes.is_empty());
    editor.execute(add_hole(HOLE)).unwrap();
    let accepted = editor.project.clone();
    assert_eq!((accepted.revision, next_id(&accepted)), (2, 20));
    assert_eq!(
        accepted.pieces[0].holes,
        [Hole {
            id: 11,
            boundary: vec![16, 17, 18, 19]
        }]
    );
    assert_eq!(accepted.points[..4], old.points);
    assert_eq!(accepted.curves[..4], old.curves);
    for (index, &id) in [16, 17, 18, 19].iter().enumerate() {
        let curve = accepted.curve(id).unwrap();
        assert_eq!(
            (curve.start, curve.end),
            (12 + index as u64, 12 + (index as u64 + 1) % 4)
        );
    }
    editor
        .execute(Command::MovePoint {
            point: 12,
            position: [0.06, 0.12],
        })
        .unwrap();
    assert_eq!((editor.project.revision, next_id(&editor.project)), (3, 20));
    assert!(editor.undo());
    assert_eq!(editor.project.revision, 4);
    assert_eq!(editor.project.point(12).unwrap().position, HOLE[0]);
    assert!(editor.redo());
    assert_eq!(editor.project.revision, 5);
    assert_eq!(editor.project.point(12).unwrap().position, [0.06, 0.12]);
    let mut split = Editor::new(accepted.clone()).unwrap();
    split
        .execute(Command::SplitCurve { curve: 16, t: 0.5 })
        .unwrap();
    assert_eq!(split.project.point(20).unwrap().position, [0.08, 0.18]);
    assert_eq!(
        split.project.pieces[0].holes[0].boundary,
        [16, 21, 17, 18, 19]
    );
    split.execute(Command::JoinCurve { curve: 16 }).unwrap();
    assert_eq!(split.project.pieces[0].holes, accepted.pieces[0].holes);
    assert_eq!(split.project.points, accepted.points);
    assert_eq!(split.project.curves, accepted.curves);
    assert_eq!(next_id(&split.project), 22);
    let mut remove = Editor::new(accepted).unwrap();
    remove.execute(Command::RemoveHole { hole: 11 }).unwrap();
    assert_eq!(remove.project.points, old.points);
    assert_eq!(remove.project.curves, old.curves);
    assert_eq!(next_id(&remove.project), 20);
    assert!(remove.undo());
    assert_eq!(remove.project.pieces[0].holes[0].id, 11);
    let mut branch = annulus();
    assert!(branch.undo());
    assert_eq!((branch.project.revision, next_id(&branch.project)), (3, 20));
    branch.execute(add_hole(HOLE)).unwrap();
    assert_eq!((branch.project.revision, next_id(&branch.project)), (4, 29));
    assert_eq!(
        branch.project.pieces[0].holes,
        [Hole {
            id: 20,
            boundary: vec![25, 26, 27, 28]
        }]
    );
}

#[test]
fn last_first_join_preserves_addressable_hole_and_removes_only_common_geometry() {
    let mut editor = rectangle();
    editor
        .execute(add_hole(vec![
            [0.12, 0.12],
            [0.08, 0.12],
            [0.08, 0.24],
            [0.16, 0.24],
            [0.16, 0.12],
        ]))
        .unwrap();
    editor.execute(Command::JoinCurve { curve: 21 }).unwrap();
    assert_eq!(
        editor.project.pieces[0].holes,
        [Hole {
            id: 11,
            boundary: vec![18, 19, 20, 21]
        }]
    );
    assert!(editor.project.point(12).is_none());
    assert!(editor.project.curve(17).is_none());
    assert_eq!(next_id(&editor.project), 22);
    near(
        area(&coordinates(
            &editor.project,
            &editor.project.pieces[0].holes[0].boundary,
        ))
        .abs(),
        0.0096,
        1e-12,
    );
    editor.execute(Command::RemoveHole { hole: 11 }).unwrap();
    assert_eq!(editor.project.points.len(), 4);
    assert_eq!(editor.project.curves.len(), 4);
    assert_eq!(next_id(&editor.project), 22);
}

#[test]
fn resize_transform_placement_and_piece_removal_include_both_loops_and_annotations() {
    let mut editor = annulus();
    editor
        .execute(Command::AddInternalSegment {
            piece: 2,
            start: [0.02, 0.06],
            end: [0.22, 0.06],
        })
        .unwrap();
    let original = editor.project.clone();
    editor
        .execute(Command::ResizePiece {
            piece: 2,
            width: 0.48,
            height: 0.72,
        })
        .unwrap();
    for point in &original.points {
        for (actual, expected) in editor
            .project
            .point(point.id)
            .unwrap()
            .position
            .into_iter()
            .zip(point.position.map(|v| 2. * v))
        {
            near(actual, expected, 1e-12);
        }
    }
    assert_eq!(editor.project.pieces[0], original.pieces[0]);
    let mesh = editor.project.mesh().unwrap();
    near(
        mesh.triangles
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|v| mesh.uv[v]);
                cross(a, b, c) * 0.5
            })
            .sum(),
        0.3072,
        1e-10,
    );
    near(
        create_cloth(&editor.project, &mesh)
            .unwrap()
            .masses()
            .iter()
            .sum(),
        0.055296,
        1e-10,
    );

    let mut editor = Editor::new(original).unwrap();
    editor
        .execute(Command::SetPlacement {
            piece: 2,
            placement: Placement {
                translation: [0.4, 1., 0.2],
                rotation: [0.; 3],
            },
        })
        .unwrap();
    let placed = editor.project.clone();
    editor
        .execute(Command::TransformPiece {
            piece: 2,
            translation: [0.03, -0.02],
            angle: std::f64::consts::FRAC_PI_2,
            pivot: [0.12, 0.18],
        })
        .unwrap();
    for point in &placed.points {
        let uv = editor.project.point(point.id).unwrap().position;
        for (actual, expected) in uv
            .into_iter()
            .zip([0.33 - point.position[1], point.position[0] + 0.04])
        {
            near(actual, expected, 1e-12);
        }
        for (actual, expected) in editor.project.pieces[0]
            .placement
            .transform(uv)
            .into_iter()
            .zip([point.position[0] + 0.4, point.position[1] + 1., 0.2])
        {
            near(actual, expected, 1e-12);
        }
    }
    near(
        editor.project.pieces[0].grain_angle,
        std::f64::consts::FRAC_PI_2,
        1e-12,
    );
    assert_eq!(editor.project.pieces[0].holes, placed.pieces[0].holes);
    assert_eq!(
        editor.project.pieces[0].internal_curves,
        placed.pieces[0].internal_curves
    );
    assert_eq!(next_id(&editor.project), next_id(&placed));
    let mesh = editor.project.mesh().unwrap();
    near(
        create_cloth(&editor.project, &mesh)
            .unwrap()
            .masses()
            .iter()
            .sum(),
        0.013824,
        1e-10,
    );
    editor.execute(Command::RemovePiece { piece: 2 }).unwrap();
    assert!(
        editor.project.pieces.is_empty()
            && editor.project.points.is_empty()
            && editor.project.curves.is_empty()
    );
    assert_eq!(next_id(&editor.project), next_id(&placed));
}

#[test]
fn invalid_commands_are_atomic_and_do_not_consume_ids_or_abandon_redo() {
    let mut empty = rectangle();
    for vertices in [
        vec![[0.1, 0.1]; 2],
        vec![[0.1, 0.1]; 4097],
        vec![[f64::NAN, 0.1], [0.15, 0.1], [0.1, 0.2]],
        vec![[1000.1, 0.1], [0.15, 0.1], [0.1, 0.2]],
        vec![[0.1, 0.1], [0.2, 0.1], [0.3, 0.1]],
        vec![[0.1, 0.1], [0.2, 0.1], [0.1, 0.1], [0.1, 0.2]],
        vec![[0.06, 0.1], [0.18, 0.24], [0.06, 0.24], [0.18, 0.1]],
        vec![[0.2, 0.12], [0.2, 0.24], [0.28, 0.24], [0.28, 0.12]],
        vec![[0., 0.12], [0.08, 0.24], [0.16, 0.12]],
        vec![[1e-6, 0.12], [1e-6, 0.24], [0.08, 0.24], [0.08, 0.12]],
        vec![[0.1, 0.1], [0.1 + 0.5e-9, 0.1], [0.15, 0.2]],
        vec![[0.10, 0.10], [0.100001, 0.10], [0.10, 0.100001]],
        vec![
            [0.08, 0.12],
            [0.16, 0.12],
            [0.06, 0.12],
            [0.06, 0.24],
            [0.08, 0.24],
        ],
    ] {
        rejected(&mut empty, add_hole(vertices));
    }
    rejected(
        &mut empty,
        Command::AddHole {
            piece: 999,
            vertices: HOLE.to_vec(),
        },
    );
    rejected(
        &mut empty,
        Command::Transaction(vec![add_hole(HOLE), Command::SetResolution { metres: 0. }]),
    );
    rejected(&mut empty, Command::Transaction(Vec::new()));
    rejected(
        &mut empty,
        Command::Transaction(vec![Command::Transaction(vec![add_hole(HOLE)])]),
    );
    rejected(&mut empty, Command::Transaction(vec![add_hole(HOLE); 1025]));
    assert_eq!(next_id(&empty.project), 11);
    let mut editor = annulus();
    for command in [
        add_hole(HOLE),
        Command::RemoveHole { hole: 16 },
        Command::MovePoint {
            point: 12,
            position: [-0.02, 0.12],
        },
        Command::MovePoint {
            point: 3,
            position: [0.15, 0.2],
        },
        Command::SetCurveControls {
            curve: 16,
            controls: None,
        },
        Command::SetCurveControls {
            curve: 7,
            controls: Some([[0.08, 0.], [0.16, 0.]]),
        },
        Command::SetCurveControls {
            curve: 7,
            controls: None,
        },
        Command::ResizePiece {
            piece: 2,
            width: 0.000002,
            height: 0.36,
        },
        Command::SplitCurve { curve: 16, t: 1. },
        Command::SplitCurve { curve: 16, t: 1e-7 },
        Command::JoinCurve { curve: 16 },
        Command::CutPiece {
            piece: 2,
            start_curve: 7,
            start_t: 0.5,
            end_curve: 9,
            end_t: 0.5,
        },
        Command::Transaction(vec![
            Command::RemoveHole { hole: 11 },
            add_hole(HOLE),
            Command::SetResolution { metres: 0. },
        ]),
    ] {
        rejected(&mut editor, command);
    }
    editor
        .execute(Command::MovePoint {
            point: 12,
            position: [0.06, 0.12],
        })
        .unwrap();
    assert!(editor.undo());
    rejected(&mut editor, add_hole(HOLE));
    assert!(editor.can_redo());
}

#[test]
fn winding_large_translation_concavity_and_authored_eligibility_are_resolution_independent() {
    for resolution in [0.005, 0.07, 0.5] {
        for clockwise in [false, true] {
            let mut editor = Editor::default();
            let mut outer = vec![
                [999., 999.],
                [999.24, 999.],
                [999.24, 999.36],
                [999., 999.36],
            ];
            if clockwise {
                outer[1..].reverse();
            }
            editor
                .execute(Command::AddPolygon {
                    name: "Translated".into(),
                    vertices: outer,
                })
                .unwrap();
            editor
                .execute(Command::SetResolution { metres: resolution })
                .unwrap();
            let mut vertices: Vec<_> = HOLE.map(|p| [p[0] + 999., p[1] + 999.]).into();
            vertices[1..].reverse();
            let first = vertices[0];
            editor.execute(add_hole(vertices)).unwrap();
            assert_eq!(editor.project.point(12).unwrap().position, first);
            let piece = &editor.project.pieces[0];
            assert_eq!(
                area(&coordinates(&editor.project, &piece.boundary)).signum(),
                if clockwise { -1. } else { 1. }
            );
            assert_eq!(
                area(&coordinates(&editor.project, &piece.holes[0].boundary)).signum(),
                if clockwise { 1. } else { -1. }
            );
            editor.project.validate().unwrap();
        }
        let mut concave = Editor::default();
        concave
            .execute(Command::AddPolygon {
                name: "C".into(),
                vertices: vec![
                    [0., 0.],
                    [1., 0.],
                    [1., 0.3],
                    [0.3, 0.3],
                    [0.3, 0.7],
                    [1., 0.7],
                    [1., 1.],
                    [0., 1.],
                ],
            })
            .unwrap();
        concave
            .execute(Command::SetResolution { metres: resolution })
            .unwrap();
        // All hole vertices are inside the C; the right edge crosses its exterior notch.
        rejected(
            &mut concave,
            add_hole(vec![[0.1, 0.1], [0.1, 0.9], [0.8, 0.9], [0.8, 0.1]]),
        );
        concave
            .execute(add_hole(vec![
                [0.05, 0.1],
                [0.05, 0.9],
                [0.2, 0.9],
                [0.2, 0.1],
            ]))
            .unwrap();
    }
    let mut editor = rectangle();
    editor
        .execute(Command::SetCurveControls {
            curve: 7,
            controls: Some([[0.08, 0.], [0.16, 0.]]),
        })
        .unwrap();
    rejected(&mut editor, add_hole(HOLE));
}

#[test]
fn annotations_remain_exact_and_their_whole_segments_exclude_the_closed_hole() {
    let mut editor = rectangle();
    editor
        .execute(Command::AddInternalSegment {
            piece: 2,
            start: [0.02, 0.06],
            end: [0.22, 0.06],
        })
        .unwrap();
    let before = editor.project.clone();
    editor.execute(add_hole(HOLE)).unwrap();
    assert_eq!(
        editor.project.pieces[0].internal_curves,
        before.pieces[0].internal_curves
    );
    assert_eq!(editor.project.points[..before.points.len()], before.points);
    assert_eq!(editor.project.curves[..before.curves.len()], before.curves);
    for (start, end) in [
        ([0.10, 0.16], [0.14, 0.20]),
        ([0.06, 0.18], [0.18, 0.18]),
        ([0.08, 0.14], [0.08, 0.22]),
        ([0.08 - 0.5e-6, 0.14], [0.08 - 0.5e-6, 0.22]),
    ] {
        let mut empty = rectangle();
        empty
            .execute(Command::AddInternalSegment {
                piece: 2,
                start,
                end,
            })
            .unwrap();
        rejected(&mut empty, add_hole(HOLE));
        rejected(
            &mut editor,
            Command::AddInternalSegment {
                piece: 2,
                start,
                end,
            },
        );
    }
    let curve = before.curves.last().unwrap();
    rejected(
        &mut editor,
        Command::MovePoint {
            point: curve.start,
            position: [0.12, 0.18],
        },
    );
    let hole_point = editor
        .project
        .curve(editor.project.pieces[0].holes[0].boundary[0])
        .unwrap()
        .start;
    rejected(
        &mut editor,
        Command::MovePoint {
            point: hole_point,
            position: [0.02, 0.05],
        },
    );
    editor
        .execute(Command::AddInternalSegment {
            piece: 2,
            start: [0.02, 0.28],
            end: [0.22, 0.28],
        })
        .unwrap();
}

#[test]
fn ownership_winding_and_orphans_are_rejected_at_the_document_boundary() {
    let project = annulus().project;
    for mutation in 0..10 {
        let mut invalid = project.clone();
        match mutation {
            0 => invalid.pieces[0].holes[0].id = 2,
            1 => {
                let duplicate = invalid.pieces[0].holes[0].clone();
                invalid.pieces[0].holes.push(duplicate);
            }
            2 => invalid.pieces[0].holes[0].boundary.truncate(2),
            3 => invalid.pieces[0].holes[0].boundary[0] = 7,
            4 => {
                invalid
                    .curves
                    .iter_mut()
                    .find(|c| c.id == 16)
                    .unwrap()
                    .start = 3;
                invalid.curves.iter_mut().find(|c| c.id == 19).unwrap().end = 3;
            }
            5 => {
                invalid
                    .curves
                    .iter_mut()
                    .find(|c| c.id == 16)
                    .unwrap()
                    .controls = Some([[0.08, 0.16], [0.08, 0.20]])
            }
            6 => {
                invalid.points.push(Point {
                    id: 20,
                    position: [0.1, 0.1],
                });
                invalid = with_next_id(invalid, 21);
            }
            7 => {
                invalid.curves.push(Curve {
                    id: 20,
                    start: 12,
                    end: 14,
                    controls: None,
                });
                invalid = with_next_id(invalid, 21);
            }
            8 => {
                for (id, position) in [12, 13, 14, 15]
                    .into_iter()
                    .zip([HOLE[0], HOLE[3], HOLE[2], HOLE[1]])
                {
                    invalid
                        .points
                        .iter_mut()
                        .find(|p| p.id == id)
                        .unwrap()
                        .position = position;
                }
            }
            _ => invalid = with_next_id(invalid, 19),
        }
        assert!(invalid.validate().is_err(), "mutation {mutation}");
        assert!(Editor::new(invalid).is_err(), "mutation {mutation}");
    }
}

#[test]
fn outer_sewing_remains_exact_and_every_hole_sewing_route_refuses() {
    let mut editor = rectangle();
    editor
        .execute(Command::AddRectangle {
            name: "Other".into(),
            origin: [0.4, 0.],
            width: 0.24,
            height: 0.36,
        })
        .unwrap();
    let other = editor.project.pieces[1].boundary[0];
    editor
        .execute(Command::AddSeam {
            name: "Legacy outer".into(),
            a: CurveInterval::full(7),
            b: CurveInterval::full(other),
            correspondence: vec![[0., 0.], [0.4, 0.6], [1., 1.]],
        })
        .unwrap();
    editor
        .execute(Command::AddSewingGroup {
            name: "Ordered outer".into(),
            a: vec![CurveInterval::full(7)],
            b: vec![CurveInterval::full(other)],
            correspondence: vec![[0., 0.], [1., 1.]],
        })
        .unwrap();
    let seams = editor.project.seams.clone();
    let groups = editor.project.sewing_groups.clone();
    let lengths = editor.project.seam_lengths(seams[0].id).unwrap();
    editor.execute(add_hole(HOLE)).unwrap();
    let hole = editor.project.pieces[0].holes[0].clone();
    let curve = hole.boundary[0];
    for command in [
        Command::AddSeam {
            name: "Invalid".into(),
            a: CurveInterval::full(curve),
            b: CurveInterval::full(other),
            correspondence: vec![[0., 0.], [1., 1.]],
        },
        Command::AddSewingGroup {
            name: "Invalid".into(),
            a: vec![CurveInterval::full(curve)],
            b: vec![CurveInterval::full(other)],
            correspondence: vec![[0., 0.], [1., 1.]],
        },
        Command::AddSewingByLength {
            name: "Invalid".into(),
            a: SewingSideByLength {
                curve,
                start_t: 0.,
                direction: SewingDirection::Forward,
                length_m: 0.02,
            },
            b: SewingSideByLength {
                curve: other,
                start_t: 0.,
                direction: SewingDirection::Forward,
                length_m: 0.02,
            },
        },
    ] {
        rejected(&mut editor, command);
    }
    let mut seam = seams[0].clone();
    seam.a.curve = curve;
    rejected(&mut editor, Command::UpdateSeam { seam: seam.clone() });
    let mut group = groups[0].clone();
    group.a[0].curve = curve;
    rejected(&mut editor, Command::UpdateSewingGroup { group });
    let mut invalid = editor.project.clone();
    invalid.seams[0] = seam;
    assert!(invalid.validate().is_err());
    assert!(invalid.seam_lengths(seams[0].id).is_err());
    assert!(
        editor
            .project
            .interval_from_length(SewingSideByLength {
                curve,
                start_t: 0.,
                direction: SewingDirection::Forward,
                length_m: 0.02
            })
            .is_err()
    );
    let hole_point = editor.project.curve(curve).unwrap().start;
    editor
        .execute(Command::MovePoint {
            point: hole_point,
            position: [0.06, 0.12],
        })
        .unwrap();
    editor
        .execute(Command::SplitCurve { curve, t: 0.5 })
        .unwrap();
    editor.execute(Command::JoinCurve { curve }).unwrap();
    assert_eq!(editor.project.seams, seams);
    assert_eq!(editor.project.sewing_groups, groups);
    assert_eq!(editor.project.seam_lengths(seams[0].id).unwrap(), lengths);
    editor
        .execute(Command::RemoveHole { hole: hole.id })
        .unwrap();
    assert_eq!(editor.project.seams, seams);
    assert_eq!(editor.project.sewing_groups, groups);
    assert_eq!(editor.project.seam_lengths(seams[0].id).unwrap(), lengths);
}

#[test]
fn stable_id_and_total_object_limits_include_holes_and_refuse_before_mutation() {
    let mut exhausted = Editor::new(with_next_id(rectangle().project, u64::MAX - 8)).unwrap();
    assert!(rejected(&mut exhausted, add_hole(HOLE)).contains("stable ID"));
    let mut final_ids = Editor::new(with_next_id(rectangle().project, u64::MAX - 9)).unwrap();
    final_ids.execute(add_hole(HOLE)).unwrap();
    assert_eq!(final_ids.project.pieces[0].holes[0].id, u64::MAX - 9);
    assert_eq!(next_id(&final_ids.project), u64::MAX);
    final_ids
        .execute(Command::RemoveHole { hole: u64::MAX - 9 })
        .unwrap();
    assert_eq!(next_id(&final_ids.project), u64::MAX);
    rejected(&mut final_ids, add_hole(HOLE));
    let mut limited = rectangle().project;
    let mut id = next_id(&limited);
    while limited.points.len()
        + limited.curves.len()
        + limited.pieces.len()
        + limited.materials.len()
        < 20_000 - 8
    {
        limited.materials.push(Material {
            id,
            ..Material::default()
        });
        id += 1;
    }
    let mut limited = Editor::new(with_next_id(limited, id)).unwrap();
    assert!(rejected(&mut limited, add_hole(HOLE)).contains("object limit"));
    let mut holed = annulus().project;
    let mut id = next_id(&holed);
    while holed.points.len() + holed.curves.len() + holed.pieces.len() + holed.materials.len()
        < 20_000
    {
        holed.materials.push(Material {
            id,
            ..Material::default()
        });
        id += 1;
    }
    assert!(
        with_next_id(holed, id)
            .validate()
            .unwrap_err()
            .to_string()
            .contains("object limit")
    );
}

#[test]
fn containment_budget_is_shared_across_hole_operations_in_a_transaction() {
    let mut outer = Vec::new();
    for side in 0..4 {
        for step in 0..150 {
            let t = step as f64 / 15.;
            outer.push(match side {
                0 => [t, 0.],
                1 => [10., t],
                2 => [10. - t, 10.],
                _ => [0., 10. - t],
            });
        }
    }
    let mut editor = Editor::default();
    editor
        .execute(Command::AddPolygon {
            name: "Work budget".into(),
            vertices: outer,
        })
        .unwrap();
    let vertices = vec![[4., 4.], [4., 6.], [6., 6.], [6., 4.]];
    let mut check = editor.clone();
    check.execute(add_hole(vertices.clone())).unwrap();
    let first = next_id(&editor.project);
    let commands: Vec<_> = (0..64)
        .flat_map(|i| {
            [
                add_hole(vertices.clone()),
                Command::RemoveHole {
                    hole: first + 9 * i,
                },
            ]
        })
        .collect();
    assert!(
        rejected(&mut editor, Command::Transaction(commands))
            .contains("containment work budget exhausted")
    );
}

#[test]
fn eligible_large_annulus_reports_bounded_mesh_refusal_without_losing_authored_hole() {
    let mut editor = Editor::default();
    editor
        .execute(Command::AddRectangle {
            name: "Bounded 96 m2 annulus".into(),
            origin: [0., 0.],
            width: 10.,
            height: 10.,
        })
        .unwrap();
    editor
        .execute(add_hole(vec![[4., 4.], [4., 6.], [6., 6.], [6., 4.]]))
        .unwrap();
    editor
        .execute(Command::SetResolution { metres: 0.005 })
        .unwrap();
    let accepted = editor.project.clone();
    let error = editor.project.mesh().unwrap_err().to_string();
    assert!(
        error.contains("limit") || error.contains("20 levels"),
        "{error}"
    );
    assert_eq!(editor.project, accepted);
    assert_eq!(editor.project.pieces[0].holes[0].id, 11);
    editor.project.validate().unwrap();
    let bytes = editor.project.to_json().unwrap();
    assert_eq!(Project::from_json(&bytes).unwrap(), accepted);
}

#[test]
fn disjoint_holes_keep_their_boundaries_mesh_area_and_history() {
    let mut editor = annulus();
    let first = editor.project.pieces[0].holes[0].clone();
    let second = [[0.015, 0.02], [0.045, 0.02], [0.045, 0.06], [0.015, 0.06]];
    editor.execute(add_hole(second)).unwrap();
    let ids = editor.project.pieces[0].holes.clone();
    assert_eq!(ids[0], first);
    assert_ne!(ids[0].id, ids[1].id);
    let restored = Project::from_json(&editor.project.to_json().unwrap()).unwrap();
    assert_eq!(restored.pieces[0].holes, ids);
    let mesh = restored.mesh().unwrap();
    let mut total = 0.;
    for triangle in &mesh.triangles {
        let face = triangle.map(|i| mesh.uv[i]);
        total += area(&face).abs();
        for ring in [HOLE, second] {
            let mut clip = ring;
            if area(&clip) < 0. {
                clip.reverse();
            }
            near(area(&intersection(face.to_vec(), &clip)).abs(), 0., 1e-14);
        }
    }
    near(total, 0.24 * 0.36 - 0.08 * 0.12 - 0.03 * 0.04, 1e-12);
    for invalid in [
        [[0.02, 0.03], [0.03, 0.03], [0.03, 0.04], [0.02, 0.04]],
        [[0.01, 0.01], [0.06, 0.01], [0.06, 0.07], [0.01, 0.07]],
        [[0.04, 0.05], [0.07, 0.05], [0.07, 0.08], [0.04, 0.08]],
        [
            [0.0450005, 0.03],
            [0.06, 0.03],
            [0.06, 0.04],
            [0.0450005, 0.04],
        ],
    ] {
        rejected(&mut editor, add_hole(invalid));
    }
    editor
        .execute(Command::RemoveHole { hole: first.id })
        .unwrap();
    assert_eq!(editor.project.pieces[0].holes, vec![ids[1].clone()]);
    assert!(editor.undo());
    assert_eq!(editor.project.pieces[0].holes, ids);
}
