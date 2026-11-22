//! Imported initial conditions in piece-local XYZ and authored material metres.
use crate::{BoundarySample, Error, Piece, Project, Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitialPose {
    pub uv: Vec<[f64; 2]>,
    pub positions: Vec<[f64; 3]>,
    pub triangles: Vec<[usize; 3]>,
}

pub(crate) fn twice_surface_area([a, b, c]: [[f64; 3]; 3]) -> f64 {
    let b = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
    let c = std::array::from_fn::<_, 3, _>(|i| c[i] - a[i]);
    let cross = std::array::from_fn::<_, 3, _>(|i| {
        b[(i + 1) % 3] * c[(i + 2) % 3] - b[(i + 2) % 3] * c[(i + 1) % 3]
    });
    cross.into_iter().fold(0., f64::hypot)
}

impl InitialPose {
    pub(crate) fn deserialize_present<'de, D: serde::Deserializer<'de>>(
        de: D,
    ) -> std::result::Result<Option<Self>, D::Error> {
        Self::deserialize(de).map(Some)
    }

    // Positive faces cancel pairwise on internal edges. The remaining directed
    // edges must cover precisely the authored outer ring and its holes.
    pub(crate) fn checked_boundary(
        &self,
        project: &Project,
        piece: &Piece,
        check: impl Fn() -> Result<()>,
    ) -> Result<Vec<BoundarySample>> {
        let validate = || -> Result<_> {
            let n = self.positions.len();
            ensure(
                n == self.uv.len()
                    && (3..=crate::MAX_MESH_VERTICES).contains(&n)
                    && (1..=crate::MAX_MESH_TRIANGLES).contains(&self.triangles.len()),
                "Invalid pose attribute counts",
            )?;
            let key = |point: [f64; 2]| point.map(|x| if x == 0. { 0 } else { x.to_bits() });
            let mut by_uv = BTreeMap::new();
            for (i, (&uv, &p)) in self.uv.iter().zip(&self.positions).enumerate() {
                check()?;
                crate::document::validate_coordinates(&uv)?;
                crate::document::validate_coordinates(&p)?;
                crate::document::validate_coordinates(&piece.placement.transform_3d(p))?;
                ensure(
                    by_uv.insert(key(uv), i).is_none(),
                    "Coincident material vertices",
                )?;
            }

            let mut edges = BTreeMap::new();
            let mut used = vec![false; n];
            for face in &self.triangles {
                check()?;
                ensure(face.iter().all(|&i| i < n), "Invalid pose face index")?;
                let [a, b, c] = face.map(|i| self.uv[i]);
                ensure(
                    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 1e-12,
                    "Folded or degenerate material triangle",
                )?;
                ensure(
                    twice_surface_area(face.map(|i| self.positions[i])) > 1e-12,
                    "Degenerate posed triangle",
                )?;
                for i in 0..3 {
                    let (a, b) = (face[i], face[(i + 1) % 3]);
                    used[a] = true;
                    let edge = edges.entry((a.min(b), a.max(b))).or_insert([0u8; 2]);
                    let side = usize::from(a > b);
                    ensure(edge[side] == 0, "Nonmanifold or overlapping material faces")?;
                    edge[side] = 1;
                }
            }
            ensure(used.into_iter().all(|v| v), "Unused pose vertex")?;
            let mut boundary = BTreeMap::new();
            for ((a, b), incidence) in edges {
                check()?;
                let (a, b) = match incidence {
                    [1, 0] => (a, b),
                    [0, 1] => (b, a),
                    _ => continue,
                };
                ensure(
                    boundary.insert(a, b).is_none(),
                    "Branched material boundary",
                )?;
            }
            // A positive piecewise-linear chart with a simple, correctly oriented
            // boundary has degree one inside the panel and zero outside. Check
            // the actual OBJ boundary too, before any near-collinear simplification.
            let mut spans: Vec<_> = boundary
                .iter()
                .map(|(&a, &b)| {
                    let (u, v) = (self.uv[a], self.uv[b]);
                    (
                        u[0].min(v[0]),
                        u[0].max(v[0]),
                        u[1].min(v[1]),
                        u[1].max(v[1]),
                        a,
                        b,
                    )
                })
                .collect();
            earcutr::sort_by_with_control(&mut spans, |a, b| a.0.total_cmp(&b.0), &check)?;
            let mut budget = 1_000_000;
            for (i, &(_, max_x, min_y, max_y, a, b)) in spans.iter().enumerate() {
                for &(x, _, y, yy, c, d) in &spans[i + 1..] {
                    check()?;
                    if x > max_x {
                        break;
                    }
                    ensure(budget > 0, "Pose boundary intersection work limit exceeded")?;
                    budget -= 1;
                    if y <= max_y && yy >= min_y && a != c && a != d && b != c && b != d {
                        ensure(
                            !crate::geometry::segments_intersect(
                                self.uv[a], self.uv[b], self.uv[c], self.uv[d],
                            ),
                            "Overlapping material chart boundary",
                        )?;
                    }
                }
            }
            let mut samples = Vec::new();
            for &id in piece.boundary_loops().flatten() {
                check()?;
                let curve = project
                    .curve(id)
                    .ok_or_else(|| Error("Missing curve".into()))?;
                ensure(
                    curve.controls.is_none(),
                    "Imported pose requires straight boundaries",
                )?;
                let a = project
                    .point(curve.start)
                    .ok_or_else(|| Error("Missing start".into()))?
                    .position;
                let b = project
                    .point(curve.end)
                    .ok_or_else(|| Error("Missing end".into()))?
                    .position;
                let vertex_at = |p| {
                    by_uv
                        .get(&key(p))
                        .copied()
                        .ok_or_else(|| Error("Boundary leaves imported chart".into()))
                };
                let mut at = vertex_at(a)?;
                let end = vertex_at(b)?;
                let direction = [b[0] - a[0], b[1] - a[1]];
                let len = direction[0].hypot(direction[1]);
                let mut t = 0.;
                loop {
                    check()?;
                    samples.push(BoundarySample {
                        piece: piece.id,
                        curve: id,
                        t,
                        vertex: at,
                    });
                    if at == end {
                        break;
                    }
                    at = boundary.remove(&at).ok_or_else(|| {
                        Error("Boundary topology differs from imported chart".into())
                    })?;
                    let uv = self.uv[at];
                    let delta = [uv[0] - a[0], uv[1] - a[1]];
                    let next = if at == end {
                        1.
                    } else {
                        (delta[0] * direction[0] + delta[1] * direction[1]) / (len * len)
                    };
                    ensure(
                        (delta[0] * direction[1] - delta[1] * direction[0]).abs() <= 1.01e-8 * len
                            && next > t
                            && next <= 1.,
                        "Boundary crosses or doubles back on imported chart",
                    )?;
                    t = next;
                }
            }
            ensure(
                boundary.is_empty(),
                "Extra material boundaries or overlapping chart",
            )?;
            Ok(samples)
        };
        validate().map_err(|e| Error(format!("Panel {} (#{}) initial pose: {e}. Clear imported pose before changing its boundary.",piece.name,piece.id)))
    }
}
