use crate::{
    BoundarySample, CurveInterval, Error, Id, Mesh, Project, Result, SewingGroup, Stitch, ensure,
};
use std::collections::{BTreeMap, BTreeSet};

/// Absolute authored-path length/location budget, independent of mesh resolution.
pub const SEWING_METRIC_TOLERANCE: f64 = 1.0e-8;
pub const MAX_SEWING_PORTIONS: usize = 200_000;
pub const MAX_SEWING_STITCHES: usize = 1_000_000;
const METRIC_BUDGET: f64 = SEWING_METRIC_TOLERANCE / 256.;
const MAX_METRIC_WORK: usize = 4_000_000;
const MAX_METRIC_DEPTH: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SewingPortion {
    pub a: CurveInterval,
    pub b: CurveInterval,
}

/// Rebuildable authored geometry for the owning mesh's document revision.
#[derive(Debug, Clone, PartialEq)]
pub struct SewingGroupGeometry {
    pub id: Id,
    pub lengths: [f64; 2],
    /// One source-owned interval per side per correspondence/span cell.
    pub portions: Vec<SewingPortion>,
}

struct Work<'a> {
    cancelled: &'a dyn Fn() -> bool,
    used: usize,
}

impl<'a> Work<'a> {
    fn new(cancelled: &'a dyn Fn() -> bool) -> Self {
        Self { cancelled, used: 0 }
    }

    fn step(&mut self) -> Result<()> {
        self.used += 1;
        ensure(
            self.used <= MAX_METRIC_WORK,
            "Sewing metric/meshing work budget exhausted",
        )?;
        if self.used == 1 || self.used.is_multiple_of(128) {
            ensure(
                !(self.cancelled)(),
                "Sewing metric/meshing cancelled; previous construction retained",
            )?;
        }
        Ok(())
    }
}

type Cubic = [[f64; 2]; 4];

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (b[0] - a[0]).hypot(b[1] - a[1])
}

fn lerp(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    std::array::from_fn(|axis| a[axis] + (b[axis] - a[axis]) * t)
}

fn split(p: Cubic, t: f64) -> (Cubic, Cubic) {
    let a = lerp(p[0], p[1], t);
    let b = lerp(p[1], p[2], t);
    let c = lerp(p[2], p[3], t);
    let d = lerp(a, b, t);
    let e = lerp(b, c, t);
    let f = lerp(d, e, t);
    ([p[0], a, d, f], [f, e, c, p[3]])
}

fn restricted(mut p: Cubic, lo: f64, hi: f64) -> Cubic {
    if hi < 1. {
        p = split(p, hi).0;
    }
    if lo > 0. {
        p = split(p, lo / hi).1;
    }
    p
}

fn polygon_length(p: Cubic) -> f64 {
    p.windows(2).map(|pair| distance(pair[0], pair[1])).sum()
}

// Simpson's fourth-derivative remainder is bounded using the restricted cubic's
// derivative control hull; near stationary points the geometric length bounds win.
fn simpson(p: Cubic) -> (f64, f64) {
    let v: [[f64; 2]; 3] =
        std::array::from_fn(|i| std::array::from_fn(|a| 3. * (p[i + 1][a] - p[i][a])));
    let norm = |p: [f64; 2]| p[0].hypot(p[1]);
    let midpoint = std::array::from_fn(|a| (v[0][a] + 2. * v[1][a] + v[2][a]) * 0.25);
    let estimate = (norm(v[0]) + 4. * norm(midpoint) + norm(v[2])) / 6.;
    let min: [f64; 2] = std::array::from_fn(|axis| {
        let lo = v.iter().map(|p| p[axis]).fold(f64::INFINITY, f64::min);
        let hi = v.iter().map(|p| p[axis]).fold(f64::NEG_INFINITY, f64::max);
        lo.max(0.).max(-hi)
    });
    let lower = norm(min);
    if lower == 0. {
        return (estimate, f64::INFINITY);
    }
    let velocity = v.into_iter().map(norm).fold(0., f64::max);
    let a: [[f64; 2]; 2] =
        std::array::from_fn(|i| std::array::from_fn(|axis| 2. * (v[i + 1][axis] - v[i][axis])));
    let acceleration = a.into_iter().map(norm).fold(0., f64::max);
    let jerk = norm(std::array::from_fn(|axis| {
        2. * (v[2][axis] - 2. * v[1][axis] + v[0][axis])
    }));
    let q1 = 2. * velocity * acceleration;
    let q2 = 2. * (acceleration * acceleration + velocity * jerk);
    let q3 = 6. * acceleration * jerk;
    let q4 = 6. * jerk * jerk;
    let fourth = q4 / (2. * lower)
        + (q1 * q3 + 0.75 * q2 * q2) / lower.powi(3)
        + 2.25 * q1 * q1 * q2 / lower.powi(5)
        + 0.9375 * q1.powi(4) / lower.powi(7);
    (estimate, fourth / 2880.)
}

struct Leaf {
    p: Cubic,
    t: [f64; 2],
    length: f64,
    start: f64,
}

fn integrate(
    p: Cubic,
    t: [f64; 2],
    budget: f64,
    depth: u32,
    work: &mut Work<'_>,
    leaves: &mut Vec<Leaf>,
) -> Result<()> {
    work.step()?;
    let origin = p[0];
    let p = p.map(|point| std::array::from_fn(|axis| point[axis] - origin[axis]));
    let chord = distance(p[0], p[3]);
    let polygon = polygon_length(p);
    ensure(polygon.is_finite(), "Sewing length is nonfinite")?;
    let rounding = 64. * f64::EPSILON * polygon;
    let geometric = (polygon - chord).max(0.) * 0.5;
    let (estimate, remainder) = simpson(p);
    let length = if geometric + rounding <= budget {
        Some(chord + (polygon - chord) * 0.5)
    } else if remainder + rounding <= budget {
        Some(estimate.clamp(chord, polygon.max(chord)))
    } else {
        None
    };
    if let Some(length) = length {
        leaves.push(Leaf {
            p,
            t,
            length,
            start: 0.,
        });
        return Ok(());
    }
    ensure(
        depth < MAX_METRIC_DEPTH && budget > 0.,
        "Sewing cubic length cannot resolve the supported precision budget",
    )?;
    let middle = t[0] + (t[1] - t[0]) * 0.5;
    ensure(
        t[0] < middle && middle < t[1],
        "Sewing cubic parameter precision collapsed",
    )?;
    let (left, right) = split(p, 0.5);
    integrate(left, [t[0], middle], budget * 0.5, depth + 1, work, leaves)?;
    integrate(right, [middle, t[1]], budget * 0.5, depth + 1, work, leaves)
}

fn add_length(total: &mut f64, correction: &mut f64, length: f64) -> f64 {
    let increment = length - *correction;
    let next = *total + increment;
    *correction = (next - *total) - increment;
    *total = next;
    next
}

struct IntervalMetric {
    interval: CurveInterval,
    leaves: Vec<Leaf>,
    length: f64,
    budget: f64,
}

impl IntervalMetric {
    fn new(
        project: &Project,
        interval: CurveInterval,
        budget: f64,
        work: &mut Work<'_>,
    ) -> Result<Self> {
        work.step()?;
        ensure(
            interval.t0.is_finite()
                && interval.t1.is_finite()
                && (0.0..=1.).contains(&interval.t0)
                && (0.0..=1.).contains(&interval.t1)
                && (interval.t1 - interval.t0).abs() > 1.0e-8,
            "Sewing intervals must have distinct endpoints in [0,1]",
        )?;
        let curve = project
            .curve(interval.curve)
            .ok_or_else(|| Error("Sewing curve is missing".into()))?;
        let start = project
            .point(curve.start)
            .ok_or_else(|| Error("Missing curve start".into()))?
            .position;
        let end = project
            .point(curve.end)
            .ok_or_else(|| Error("Missing curve end".into()))?
            .position;
        let mut leaves = Vec::new();
        let length = if let Some([c, d]) = curve.controls {
            // Translation does not affect length and local coordinates avoid cancellation.
            let p = [start, c, d, end].map(|p| std::array::from_fn(|axis| p[axis] - start[axis]));
            let lo = interval.t0.min(interval.t1);
            let hi = interval.t0.max(interval.t1);
            integrate(
                restricted(p, lo, hi),
                [lo, hi],
                budget,
                0,
                work,
                &mut leaves,
            )?;
            let mut total = 0.;
            let mut correction = 0.;
            for leaf in &mut leaves {
                leaf.start = total;
                add_length(&mut total, &mut correction, leaf.length);
            }
            total
        } else {
            distance(start, end) * (interval.t1 - interval.t0).abs()
        };
        ensure(
            length.is_finite() && length > 0.,
            "Sewing interval has no resolvable positive finite length",
        )?;
        ensure(
            256. * f64::EPSILON * length < SEWING_METRIC_TOLERANCE,
            "Sewing interval exceeds the supported absolute precision budget",
        )?;
        Ok(Self {
            interval,
            leaves,
            length,
            budget,
        })
    }

    fn partial_leaf(&self, leaf: &Leaf, u: f64, work: &mut Work<'_>) -> Result<f64> {
        if u == 0. {
            return Ok(0.);
        }
        if u == 1. {
            return Ok(leaf.length);
        }
        let mut leaves = Vec::new();
        integrate(
            split(leaf.p, u).0,
            [0., 1.],
            self.budget,
            0,
            work,
            &mut leaves,
        )?;
        Ok(leaves.iter().map(|leaf| leaf.length).sum())
    }

    fn length_at(&self, t: f64, work: &mut Work<'_>) -> Result<f64> {
        if t == self.interval.t0 {
            return Ok(0.);
        }
        if t == self.interval.t1 {
            return Ok(self.length);
        }
        if self.leaves.is_empty() {
            return Ok(
                (t - self.interval.t0) / (self.interval.t1 - self.interval.t0) * self.length,
            );
        }
        let index = self
            .leaves
            .partition_point(|leaf| leaf.t[1] < t)
            .min(self.leaves.len() - 1);
        let leaf = &self.leaves[index];
        let u = ((t - leaf.t[0]) / (leaf.t[1] - leaf.t[0])).clamp(0., 1.);
        let length = leaf.start + self.partial_leaf(leaf, u, work)?;
        Ok(if self.interval.t1 > self.interval.t0 {
            length
        } else {
            self.length - length
        })
    }

    fn parameter(&self, length: f64, precision: f64, work: &mut Work<'_>) -> Result<f64> {
        if length == 0. {
            return Ok(self.interval.t0);
        }
        if length == self.length {
            return Ok(self.interval.t1);
        }
        ensure(
            length.is_finite() && (0.0..=self.length).contains(&length),
            "Sewing distance must be inside the interval length",
        )?;
        if self.leaves.is_empty() {
            return Ok(self.interval.parameter(length / self.length));
        }
        let target = if self.interval.t1 > self.interval.t0 {
            length
        } else {
            self.length - length
        };
        let index = self
            .leaves
            .partition_point(|leaf| leaf.start + leaf.length < target)
            .min(self.leaves.len() - 1);
        let leaf = &self.leaves[index];
        let target = (target - leaf.start).clamp(0., leaf.length);
        let mut lo = 0.;
        let mut hi = 1.;
        for _ in 0..64 {
            work.step()?;
            let u = lo + (hi - lo) * 0.5;
            let actual = self.partial_leaf(leaf, u, work)?;
            if (actual - target).abs() <= precision {
                return Ok(leaf.t[0] + (leaf.t[1] - leaf.t[0]) * u);
            }
            ensure(
                lo < u && u < hi,
                "Sewing inverse length parameter precision collapsed",
            )?;
            if actual < target {
                lo = u;
            } else {
                hi = u;
            }
        }
        Err(Error(
            "Sewing inverse length did not converge within 64 iterations".into(),
        ))
    }
}

struct PathMetric {
    intervals: Vec<IntervalMetric>,
    starts: Vec<f64>,
    knots: Vec<f64>,
    total: f64,
}

impl PathMetric {
    fn new(
        project: &Project,
        intervals: &[CurveInterval],
        budget: f64,
        work: &mut Work<'_>,
    ) -> Result<Self> {
        ensure(
            (1..=1024).contains(&intervals.len()),
            "A sewing group side needs 1 to 1024 intervals",
        )?;
        let intervals = intervals
            .iter()
            .map(|&interval| {
                IntervalMetric::new(project, interval, budget / intervals.len() as f64, work)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut total = 0.;
        let mut correction = 0.;
        let mut starts = Vec::new();
        for interval in &intervals {
            starts.push(total);
            let previous = total;
            add_length(&mut total, &mut correction, interval.length);
            ensure(
                total > previous,
                "Positive sewing span collapsed in the cumulative length precision",
            )?;
        }
        let mut knots: Vec<_> = starts.iter().map(|start| start / total).collect();
        knots.push(1.);
        ensure(
            knots.windows(2).all(|pair| pair[0] < pair[1]),
            "Positive sewing span collapsed in normalized path precision",
        )?;
        Ok(Self {
            intervals,
            starts,
            knots,
            total,
        })
    }

    fn owner(&self, fraction: f64) -> usize {
        self.knots[1..]
            .partition_point(|&knot| knot <= fraction)
            .min(self.intervals.len() - 1)
    }

    fn parameter(
        &self,
        owner: usize,
        fraction: f64,
        precision: f64,
        work: &mut Work<'_>,
    ) -> Result<f64> {
        let interval = &self.intervals[owner];
        if (fraction - self.knots[owner]).abs() * self.total <= precision {
            return Ok(interval.interval.t0);
        }
        if (fraction - self.knots[owner + 1]).abs() * self.total <= precision {
            return Ok(interval.interval.t1);
        }
        let length = fraction * self.total - self.starts[owner];
        ensure(
            length >= -precision && length <= interval.length + precision,
            "Sewing correspondence escaped its source interval",
        )?;
        interval.parameter(length.clamp(0., interval.length), precision, work)
    }

    fn fraction(&self, owner: usize, t: f64, work: &mut Work<'_>) -> Result<f64> {
        Ok((self.starts[owner] + self.intervals[owner].length_at(t, work)?) / self.total)
    }
}

struct Cell {
    domain: [f64; 2],
    owners: [usize; 2],
    portion: SewingPortion,
}

struct GroupMetric {
    paths: [PathMetric; 2],
    precision: f64,
}

impl GroupMetric {
    fn new(project: &Project, group: &SewingGroup, work: &mut Work<'_>) -> Result<Self> {
        ensure(
            (2..=1024).contains(&group.correspondence.len())
                && group.correspondence.first() == Some(&[0., 0.])
                && group.correspondence.last() == Some(&[1., 1.])
                && group.correspondence.windows(2).all(|pair| {
                    pair[1]
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.).contains(v))
                        && pair[0][0] < pair[1][0]
                        && pair[0][1] < pair[1][1]
                }),
            "Invalid sewing group correspondence",
        )?;
        let mut paths = [
            PathMetric::new(project, &group.a, METRIC_BUDGET, work)?,
            PathMetric::new(project, &group.b, METRIC_BUDGET, work)?,
        ];
        let mut condition: f64 = 1.;
        for pair in group.correspondence.windows(2) {
            let ratio = (pair[1][1] - pair[0][1]) / (pair[1][0] - pair[0][0]) * paths[1].total
                / paths[0].total;
            condition = condition.max(ratio).max(1. / ratio);
        }
        ensure(
            condition.is_finite()
                && 256. * f64::EPSILON * paths[0].total.max(paths[1].total) * condition
                    < SEWING_METRIC_TOLERANCE / 16.,
            "Sewing correspondence exceeds the supported absolute precision budget",
        )?;
        let precision = METRIC_BUDGET / condition;
        if condition > 1. {
            paths = [
                PathMetric::new(project, &group.a, precision, work)?,
                PathMetric::new(project, &group.b, precision, work)?,
            ];
        }
        Ok(Self { paths, precision })
    }

    fn cells(&self, group: &SewingGroup, work: &mut Work<'_>) -> Result<Vec<Cell>> {
        let mut cuts = self.paths[0].knots.clone();
        cuts.extend(
            self.paths[1].knots.iter().map(|&fraction| {
                crate::geometry::correspondence(&group.correspondence, fraction, 1)
            }),
        );
        cuts.extend(group.correspondence.iter().map(|pair| pair[0]));
        cuts.sort_by(f64::total_cmp);
        // Algebraically equal A/B knots can differ by an ulp after split/remap.
        // Owners still resolve both exact source endpoints at the shared cut.
        let merge_distance = self.precision / self.paths[0].total.max(self.paths[1].total);
        cuts.dedup_by(|a, b| {
            if (*a - *b).abs() <= merge_distance {
                if *a == 1. {
                    *b = 1.;
                }
                true
            } else {
                false
            }
        });
        let mut cells = Vec::new();
        for bounds in cuts.windows(2) {
            work.step()?;
            let middle = bounds[0] + (bounds[1] - bounds[0]) * 0.5;
            ensure(
                bounds[0] < middle && middle < bounds[1],
                "Sewing correspondence cell precision collapsed",
            )?;
            let owners = [
                self.paths[0].owner(middle),
                self.paths[1].owner(crate::geometry::correspondence(
                    &group.correspondence,
                    middle,
                    0,
                )),
            ];
            let mut intervals = [
                self.paths[0].intervals[owners[0]].interval,
                self.paths[1].intervals[owners[1]].interval,
            ];
            for side in 0..2 {
                let fractions = if side == 0 {
                    [bounds[0], bounds[1]]
                } else {
                    [
                        crate::geometry::correspondence(&group.correspondence, bounds[0], 0),
                        crate::geometry::correspondence(&group.correspondence, bounds[1], 0),
                    ]
                };
                intervals[side].t0 =
                    self.paths[side].parameter(owners[side], fractions[0], self.precision, work)?;
                intervals[side].t1 =
                    self.paths[side].parameter(owners[side], fractions[1], self.precision, work)?;
                ensure(
                    intervals[side].t0 != intervals[side].t1,
                    "Positive sewing portion collapsed in curve parameter precision",
                )?;
            }
            cells.push(Cell {
                domain: [bounds[0], bounds[1]],
                owners,
                portion: SewingPortion {
                    a: intervals[0],
                    b: intervals[1],
                },
            });
        }
        for side in 0..2 {
            let owners: BTreeSet<_> = cells.iter().map(|cell| cell.owners[side]).collect();
            ensure(
                owners.len() == self.paths[side].intervals.len(),
                "Positive sewing span collapsed in correspondence cell precision",
            )?;
        }
        Ok(cells)
    }

    fn geometry(&self, group: &SewingGroup, cells: &[Cell]) -> SewingGroupGeometry {
        SewingGroupGeometry {
            id: group.id,
            lengths: self.paths.each_ref().map(|path| path.total),
            portions: cells.iter().map(|cell| cell.portion).collect(),
        }
    }
}

impl Project {
    pub fn interval_length(&self, interval: CurveInterval) -> Result<f64> {
        Ok(IntervalMetric::new(self, interval, METRIC_BUDGET, &mut Work::new(&|| false))?.length)
    }

    pub fn interval_parameter_at_length(
        &self,
        interval: CurveInterval,
        length_m: f64,
    ) -> Result<f64> {
        let mut work = Work::new(&|| false);
        IntervalMetric::new(self, interval, METRIC_BUDGET, &mut work)?.parameter(
            length_m,
            METRIC_BUDGET,
            &mut work,
        )
    }

    pub fn sewing_group_geometry(&self, id: Id) -> Result<SewingGroupGeometry> {
        let group = self
            .sewing_groups
            .iter()
            .find(|group| group.id == id)
            .ok_or_else(|| Error("Unknown sewing group".into()))?;
        let mut work = Work::new(&|| false);
        let metric = GroupMetric::new(self, group, &mut work)?;
        let cells = metric.cells(group, &mut work)?;
        Ok(metric.geometry(group, &cells))
    }

    pub(crate) fn validate_sewing_metrics(&self, selected: Option<&BTreeSet<Id>>) -> Result<()> {
        let mut work = Work::new(&|| false);
        for group in &self.sewing_groups {
            if selected.is_none_or(|ids| ids.contains(&group.id)) {
                let metric = GroupMetric::new(self, group, &mut work)?;
                metric.cells(group, &mut work)?;
            }
        }
        Ok(())
    }

    pub(crate) fn append_sewing_groups(
        &self,
        mesh: &mut Mesh,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<()> {
        if self.sewing_groups.is_empty() {
            return Ok(());
        }
        let mut work = Work::new(cancelled);
        let mut boundary = BTreeMap::<Id, Vec<BoundarySample>>::new();
        for &sample in &mesh.boundary {
            work.step()?;
            boundary.entry(sample.curve).or_default().push(sample);
        }
        for samples in boundary.values_mut() {
            samples.sort_by(|a, b| a.t.total_cmp(&b.t));
        }
        for group in &self.sewing_groups {
            let metric = GroupMetric::new(self, group, &mut work)?;
            let cells = metric.cells(group, &mut work)?;
            ensure(
                mesh.sewing_groups
                    .iter()
                    .map(|g| g.portions.len())
                    .sum::<usize>()
                    + cells.len()
                    <= MAX_SEWING_PORTIONS,
                "Sewing portion limit exceeded",
            )?;
            for cell in &cells {
                let intervals = [cell.portion.a, cell.portion.b];
                let mut fractions = vec![cell.domain[0], cell.domain[1]];
                for (side, &interval) in intervals.iter().enumerate() {
                    let samples = boundary
                        .get(&interval.curve)
                        .ok_or_else(|| Error("Sewing group has no mesh boundary".into()))?;
                    let lo = interval.t0.min(interval.t1);
                    let hi = interval.t0.max(interval.t1);
                    let start = samples.partition_point(|sample| sample.t < lo);
                    for sample in samples[start..].iter().take_while(|sample| sample.t <= hi) {
                        work.step()?;
                        let fraction =
                            metric.paths[side].fraction(cell.owners[side], sample.t, &mut work)?;
                        let fraction = if side == 0 {
                            fraction
                        } else {
                            crate::geometry::correspondence(&group.correspondence, fraction, 1)
                        };
                        if (cell.domain[0]..=cell.domain[1]).contains(&fraction) {
                            fractions.push(fraction);
                        }
                    }
                }
                fractions.sort_by(f64::total_cmp);
                fractions.dedup();
                for fraction in fractions {
                    work.step()?;
                    ensure(
                        mesh.stitches.len() < MAX_SEWING_STITCHES,
                        "Sewing stitch limit exceeded",
                    )?;
                    let paired =
                        crate::geometry::correspondence(&group.correspondence, fraction, 0);
                    let a = metric.paths[0].parameter(
                        cell.owners[0],
                        fraction,
                        metric.precision,
                        &mut work,
                    )?;
                    let b = metric.paths[1].parameter(
                        cell.owners[1],
                        paired,
                        metric.precision,
                        &mut work,
                    )?;
                    mesh.stitches.push(Stitch {
                        seam: group.id,
                        a: crate::geometry::weighted_sample_at_t(
                            &boundary[&intervals[0].curve],
                            a,
                        )?,
                        b: crate::geometry::weighted_sample_at_t(
                            &boundary[&intervals[1].curve],
                            b,
                        )?,
                    });
                }
            }
            mesh.sewing_groups.push(metric.geometry(group, &cells));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_limits_cancel_or_reject_without_an_approximate_fallback() {
        assert!(
            Work::new(&|| true)
                .step()
                .unwrap_err()
                .0
                .contains("cancelled")
        );
        let mut exhausted = Work::new(&|| false);
        exhausted.used = MAX_METRIC_WORK;
        assert!(exhausted.step().unwrap_err().0.contains("work budget"));
        let p = [[0., 0.], [0.01, 1.], [0.7, 0.4], [1., 0.]];
        assert!(
            integrate(
                p,
                [0., 1.],
                1e-30,
                MAX_METRIC_DEPTH,
                &mut Work::new(&|| false),
                &mut Vec::new()
            )
            .unwrap_err()
            .0
            .contains("precision budget")
        );
    }

    #[test]
    fn new_groups_do_not_snap_nearby_parameters_by_a_legacy_t_tolerance() {
        let samples = [
            BoundarySample {
                piece: 1,
                curve: 7,
                t: 0.,
                vertex: 0,
            },
            BoundarySample {
                piece: 1,
                curve: 7,
                t: 1.,
                vertex: 1,
            },
        ];
        let weighted = crate::geometry::weighted_sample_at_t(&samples, 1e-11).unwrap();
        assert_eq!(weighted.vertices, [0, 1]);
        assert_eq!(weighted.weight, 1e-11);
    }
}
