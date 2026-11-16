//! Applied material offsets. The interval predicates certify continuous geometry, not samples.
use crate::{Curve, Error, Id, Mesh, Point, Project, Result, ensure};
use std::cell::Cell;
use std::collections::BTreeSet;
use weftforge_sim::Cloth;

const MAX_INPUT: usize = 4096;
const MAX_OUTPUT: usize = 65_536;
const MAX_WORK: usize = 16_777_216;
const MAX_DEPTH: u32 = 24;
type V = [f64; 2];
type IV = [I; 2];
type B = [IV; 4];

/// Each elementary operation encloses the exact real operation on its operands.
/// Overflow, underflow and cancellation widen the enclosure; uncertain predicates refuse.
#[derive(Clone, Copy, Debug)]
struct I {
    lo: f64,
    hi: f64,
}
impl I {
    fn entire() -> Self {
        Self {
            lo: f64::NEG_INFINITY,
            hi: f64::INFINITY,
        }
    }
    fn point(x: f64) -> Self {
        Self { lo: x, hi: x }
    }
    fn range(a: f64, b: f64) -> Self {
        Self {
            lo: a.min(b),
            hi: a.max(b),
        }
    }
    fn hull(self, b: Self) -> Self {
        Self {
            lo: self.lo.min(b.lo),
            hi: self.hi.max(b.hi),
        }
    }
    fn add(self, b: Self) -> Self {
        if (self.lo + b.lo).is_nan() || (self.hi + b.hi).is_nan() {
            return Self::entire();
        }
        Self {
            lo: (self.lo + b.lo).next_down(),
            hi: (self.hi + b.hi).next_up(),
        }
    }
    fn neg(self) -> Self {
        Self {
            lo: -self.hi,
            hi: -self.lo,
        }
    }
    fn sub(self, b: Self) -> Self {
        self.add(b.neg())
    }
    fn mul(self, b: Self) -> Self {
        let p = [
            self.lo * b.lo,
            self.lo * b.hi,
            self.hi * b.lo,
            self.hi * b.hi,
        ];
        if p.into_iter().any(f64::is_nan) {
            return Self::entire();
        }
        Self {
            lo: p.into_iter().fold(f64::INFINITY, f64::min).next_down(),
            hi: p.into_iter().fold(f64::NEG_INFINITY, f64::max).next_up(),
        }
    }
    fn scale(self, x: f64) -> Self {
        self.mul(Self::point(x))
    }
    fn div(self, b: Self) -> Self {
        if b.lo <= 0. && b.hi >= 0. {
            return Self {
                lo: f64::NEG_INFINITY,
                hi: f64::INFINITY,
            };
        }
        self.mul(Self {
            lo: (1. / b.hi).next_down(),
            hi: (1. / b.lo).next_up(),
        })
    }
    fn sq(self) -> Self {
        let lo = if self.lo <= 0. && self.hi >= 0. {
            0.
        } else {
            let x = self.lo.abs().min(self.hi.abs());
            (x * x).next_down().max(0.)
        };
        let x = self.lo.abs().max(self.hi.abs());
        Self {
            lo,
            hi: (x * x).next_up(),
        }
    }
    fn sqrt(self) -> Self {
        Self {
            lo: self.lo.max(0.).sqrt().next_down().max(0.),
            hi: self.hi.max(0.).sqrt().next_up(),
        }
    }
    fn mid(self) -> f64 {
        self.lo + (self.hi - self.lo) * 0.5
    }
    fn excludes_zero(self) -> bool {
        self.lo > 0. || self.hi < 0.
    }
}
fn iv(p: V) -> IV {
    p.map(I::point)
}
fn va(a: IV, b: IV) -> IV {
    std::array::from_fn(|i| a[i].add(b[i]))
}
fn vs(a: IV, b: IV) -> IV {
    std::array::from_fn(|i| a[i].sub(b[i]))
}
fn vm(a: IV, b: I) -> IV {
    a.map(|x| x.mul(b))
}
fn dot(a: IV, b: IV) -> I {
    a[0].mul(b[0]).add(a[1].mul(b[1]))
}
fn cross(a: IV, b: IV) -> I {
    a[0].mul(b[1]).sub(a[1].mul(b[0]))
}
fn norm(a: IV) -> I {
    a[0].sq().add(a[1].sq()).sqrt()
}
fn mid(a: IV) -> V {
    a.map(I::mid)
}
fn hull(a: IV, b: IV) -> IV {
    std::array::from_fn(|i| a[i].hull(b[i]))
}
fn lerp(a: IV, b: IV, t: I) -> IV {
    va(vm(a, I::point(1.).sub(t)), vm(b, t))
}
fn split(p: B, t: I) -> (B, B) {
    let a = lerp(p[0], p[1], t);
    let b = lerp(p[1], p[2], t);
    let c = lerp(p[2], p[3], t);
    let d = lerp(a, b, t);
    let e = lerp(b, c, t);
    let f = lerp(d, e, t);
    ([p[0], a, d, f], [f, e, c, p[3]])
}
fn restrict(p: B, a: f64, b: f64) -> B {
    if a == 0. && b == 1. {
        return p;
    }
    let left = if b == 1. { p } else { split(p, I::point(b)).0 };
    if a == 0. {
        left
    } else {
        split(left, I::point(a).div(I::point(b))).1
    }
}
fn eval(p: B, t: I) -> IV {
    split(p, t).0[3]
}
fn derivative(p: B) -> [IV; 3] {
    std::array::from_fn(|i| vm(vs(p[i + 1], p[i]), I::point(3.)))
}
fn accel(p: B) -> [IV; 2] {
    let v = derivative(p);
    std::array::from_fn(|i| vm(vs(v[i + 1], v[i]), I::point(2.)))
}
fn controls_hull<const N: usize>(p: [IV; N]) -> IV {
    p.into_iter().reduce(hull).unwrap()
}
fn velocity(p: B, t: I) -> IV {
    let v = derivative(p);
    lerp(lerp(v[0], v[1], t), lerp(v[1], v[2], t), t)
}
fn acceleration(p: B, t: I) -> IV {
    let a = accel(p);
    lerp(a[0], a[1], t)
}
fn split_quadratic(p: [IV; 3], t: I) -> ([IV; 3], [IV; 3]) {
    let a = lerp(p[0], p[1], t);
    let b = lerp(p[1], p[2], t);
    let c = lerp(a, b, t);
    ([p[0], a, c], [c, b, p[2]])
}
fn restrict_quadratic(p: [IV; 3], a: f64, b: f64) -> [IV; 3] {
    let left = if b == 1. {
        p
    } else {
        split_quadratic(p, I::point(b)).0
    };
    if a == 0. {
        left
    } else {
        split_quadratic(left, I::point(a).div(I::point(b))).1
    }
}

pub(crate) struct Work<'a> {
    units: Cell<usize>,
    cancelled: &'a dyn Fn() -> bool,
}
impl Work<'_> {
    pub(crate) fn step(&self) -> Result<()> {
        let units = self.units.get();
        if units.is_multiple_of(256) {
            self.check()?;
        }
        ensure(
            units < MAX_WORK,
            "OffsetBudgetExceeded: request geometry work limit reached",
        )?;
        self.units.set(units + 1);
        Ok(())
    }
    pub(crate) fn check(&self) -> Result<()> {
        ensure(
            !(self.cancelled)(),
            "OffsetCancelled: previous project retained",
        )
    }
}

#[derive(Clone)]
struct Source {
    id: Id,
    start: Id,
    p: B,
    line: bool,
}
impl Source {
    fn from_curve(project: &Project, curve: &Curve) -> Self {
        let a = project.point(curve.start).unwrap().position;
        let b = project.point(curve.end).unwrap().position;
        // Lines have exact endpoint arithmetic throughout; cubic controls below are unused for them.
        let [c, d] = curve.controls.unwrap_or([a, b]);
        Self {
            id: curve.id,
            start: curve.start,
            p: [a, c, d, b].map(iv),
            line: curve.controls.is_none(),
        }
    }
    fn v(&self, t: I) -> IV {
        if self.line {
            vs(self.p[3], self.p[0])
        } else {
            velocity(self.p, t)
        }
    }
    fn c(&self, t: I) -> IV {
        if self.line {
            lerp(self.p[0], self.p[3], t)
        } else {
            eval(self.p, t)
        }
    }
    fn q(&self, t: I, d: I) -> IV {
        let v = self.v(t);
        let s = norm(v);
        va(self.c(t), vm([v[1], v[0].neg()], d.div(s)))
    }
    fn qp(&self, t: I, d: I) -> IV {
        let v = self.v(t);
        if self.line {
            return v;
        }
        let s = norm(v);
        let k = cross(v, acceleration(self.p, t)).div(s.mul(s).mul(s));
        vm(v, I::point(1.).add(d.mul(k)))
    }
    fn bounds(&self, a: f64, b: f64, d: I) -> IV {
        let p = if self.line {
            [
                self.c(I::point(a)),
                self.c(I::point(a)),
                self.c(I::point(b)),
                self.c(I::point(b)),
            ]
        } else {
            restrict(self.p, a, b)
        };
        let v = self.velocity_range([a, b]);
        let s = norm(v);
        va(controls_hull(p), vm([v[1], v[0].neg()], d.div(s)))
    }
    fn velocity_range(&self, range: [f64; 2]) -> IV {
        if self.line {
            self.v(I::point(0.))
        } else {
            controls_hull(restrict_quadratic(derivative(self.p), range[0], range[1]))
        }
    }
    fn normal_range(&self, range: [f64; 2]) -> IV {
        let v = self.velocity_range(range);
        vm([v[1], v[0].neg()], I::point(1.).div(norm(v)))
    }
    fn prime_range(&self, range: [f64; 2], d: I) -> IV {
        if self.line {
            return self.v(I::point(0.));
        }
        let v = self.velocity_range(range);
        let acc = accel(self.p);
        let a = hull(
            lerp(acc[0], acc[1], I::point(range[0])),
            lerp(acc[0], acc[1], I::point(range[1])),
        );
        let speed = norm(v);
        vm(
            v,
            I::point(1.).add(d.mul(cross(v, a).div(speed.mul(speed).mul(speed)))),
        )
    }
}

/// Target-piece measurements from authored integrals, triangle UVs and a real solver Cloth.
#[derive(Debug, Clone, PartialEq)]
pub struct OutlineMeasurements {
    pub bounds: [[f64; 2]; 2],
    pub authored_area: f64,
    pub rest_area: f64,
    pub cloth_mass: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct OutlineOffsetSpan {
    pub source_curve: Id,
    pub source_interval: [f64; 2],
    pub output_curve: Id,
    pub error_bound: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct OutlineOffsetCertificate {
    pub tolerance: f64,
    pub maximum_error: f64,
    /// Charged Rust work; opaque earcut dependency predicates are not individually observable.
    pub geometry_work: usize,
    pub spans: Vec<OutlineOffsetSpan>,
    pub joins: Vec<OutlineOffsetJoin>,
    pub continuous_topology: bool,
    pub continuation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlineJoinKind {
    Smooth,
    Miter,
    Bevel,
    AdjacentTrim,
}
#[derive(Debug, Clone, PartialEq)]
pub struct OutlineOffsetJoin {
    pub source_vertex: Id,
    pub incoming_curve: Id,
    pub outgoing_curve: Id,
    pub kind: OutlineJoinKind,
    pub trim_parameters: Option<[[f64; 2]; 2]>,
}

/// Immutable, revision-bound result of complete authoring/rest/solver preparation.
#[derive(Debug)]
pub struct PreparedOutlineOffset {
    pub(crate) source: Project,
    pub(crate) candidate: Project,
    pub(crate) mesh: Mesh,
    pub(crate) cloth: Cloth,
    piece: Id,
    width: f64,
    outward: bool,
    before: OutlineMeasurements,
    after: OutlineMeasurements,
    certificate: OutlineOffsetCertificate,
}
impl PreparedOutlineOffset {
    pub fn source_revision(&self) -> u64 {
        self.source.revision
    }
    pub fn piece(&self) -> Id {
        self.piece
    }
    pub fn width(&self) -> f64 {
        self.width
    }
    pub fn outward(&self) -> bool {
        self.outward
    }
    pub fn project(&self) -> &Project {
        &self.candidate
    }
    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }
    pub fn before(&self) -> &OutlineMeasurements {
        &self.before
    }
    pub fn after(&self) -> &OutlineMeasurements {
        &self.after
    }
    pub fn certificate(&self) -> &OutlineOffsetCertificate {
        &self.certificate
    }
}

fn source_area(s: &Source) -> I {
    if s.line {
        return cross(s.p[0], s.p[3]).scale(0.5);
    }
    let v = derivative(s.p);
    let c3 = [1., 3., 3., 1.];
    let c2 = [1., 2., 1.];
    let c5 = [1., 5., 10., 10., 5., 1.];
    let mut a = I::point(0.);
    for (i, p) in s.p.iter().enumerate() {
        for (j, q) in v.iter().enumerate() {
            a = a.add(
                cross(*p, *q)
                    .scale(c3[i] * c2[j])
                    .div(I::point(12. * c5[i + j])),
            );
        }
    }
    a
}

fn regular(s: &Source, d: I, depth: u32, work: &mut Work<'_>) -> Result<()> {
    work.step()?;
    let v = if s.line {
        s.v(I::point(0.))
    } else {
        controls_hull(derivative(s.p))
    };
    let speed = norm(v);
    if speed.lo > 0. {
        let factor = if s.line {
            I::point(1.)
        } else {
            let a = controls_hull(accel(s.p));
            I::point(1.).add(d.mul(cross(v, a).div(speed.mul(speed).mul(speed))))
        };
        if factor.lo > 0. {
            return Ok(());
        }
        ensure(
            factor.hi > 0.,
            &format!("OffsetCuspOrReversal: curve {} loses regularity", s.id),
        )?;
    }
    ensure(
        depth < MAX_DEPTH,
        &format!(
            "OffsetSingularTangent: curve {} regularity could not be certified",
            s.id
        ),
    )?;
    ensure(
        !s.line,
        &format!(
            "OffsetSingularTangent: line {} has no certified tangent",
            s.id
        ),
    )?;
    let (a, b) = split(s.p, I::point(0.5));
    regular(&Source { p: a, ..s.clone() }, d, depth + 1, work)?;
    regular(&Source { p: b, ..s.clone() }, d, depth + 1, work)
}

// Error-free transforms determine exact collinearity of the authored binary64 tangents.
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let x = a + b;
    let v = x - a;
    (x, (a - (x - v)) + (b - v))
}
fn expansion_add(e: &mut Vec<f64>, x: f64) {
    let mut q = x;
    let mut n = Vec::with_capacity(e.len() + 1);
    for &v in e.iter() {
        let (s, r) = two_sum(q, v);
        if r != 0. {
            n.push(r);
        }
        q = s;
    }
    if q != 0. || n.is_empty() {
        n.push(q);
    }
    *e = n;
}
fn turn(a: &Source, b: &Source) -> Result<i32> {
    let enclosed = cross(a.v(I::point(1.)), b.v(I::point(0.)));
    if enclosed.excludes_zero() {
        return Ok(if enclosed.lo > 0. { 1 } else { -1 });
    }
    let [ae, ac] = if a.line {
        [a.p[3], a.p[0]]
    } else {
        [a.p[3], a.p[2]]
    };
    let [be, bc] = if b.line {
        [b.p[3], b.p[0]]
    } else {
        [b.p[1], b.p[0]]
    };
    let diff = |x: IV, y: IV| {
        std::array::from_fn::<_, 2, _>(|i| {
            let (s, e) = two_sum(x[i].lo, -y[i].lo);
            [e, s]
        })
    };
    let u = diff(ae, ac);
    let v = diff(be, bc);
    let mut expansion = Vec::new();
    for (axis, sign) in [(0, 1.), (1, -1.)] {
        for x in u[axis] {
            for y in v[1 - axis] {
                let p = x * y;
                ensure(
                    // Keep the full product tail representable for the error-free fused residual.
                    p.is_finite()
                        && (x == 0. || y == 0. || p.abs() >= f64::MIN_POSITIVE * 2.0_f64.powi(54)),
                    "OffsetNumericUncertainty: tangent determinant underflow",
                )?;
                expansion_add(&mut expansion, sign * x.mul_add(y, -p));
                expansion_add(&mut expansion, sign * p);
            }
        }
    }
    Ok(expansion.last().map_or(0, |x| {
        if *x > 0. {
            1
        } else if *x < 0. {
            -1
        } else {
            0
        }
    }))
}

#[derive(Clone)]
struct Span {
    a: f64,
    b: f64,
    start: V,
    end: V,
}
#[derive(Clone)]
struct Corner {
    vertices: Vec<V>,
    retained: Option<usize>,
    sweep: IV,
    kind: OutlineJoinKind,
    trim_parameters: Option<[[f64; 2]; 2]>,
}
fn line_intersection(p: IV, u: IV, q: IV, v: IV) -> Result<(IV, I, I)> {
    let det = cross(u, v);
    ensure(
        det.excludes_zero(),
        "OffsetUncertainJoin: tangent lines have no certified intersection",
    )?;
    let delta = vs(q, p);
    let a = cross(delta, v).div(det);
    let b = cross(delta, u).div(det);
    Ok((va(p, vm(u, a)), a, b))
}
fn bounded_point(p: IV, epsilon: f64) -> Result<V> {
    let m = mid(p);
    ensure(
        m.into_iter().all(f64::is_finite) && norm(vs(p, iv(m))).hi <= epsilon,
        "OffsetNumericUncertainty: arithmetic enclosure exhausts geometric tolerance",
    )?;
    Ok(m)
}

/// Uniform transverse derivative proves one root at every intermediate width: endpoint values
/// are affine in d. Implicit differentiation (1-cos(theta))/sin(theta) makes both trim parameters
/// monotone from their shared source vertex. Bisection retains the final root uncertainty.
fn trim_line_cubic(
    line: &Source,
    curve: &Source,
    d: f64,
    epsilon: f64,
    work: &mut Work<'_>,
) -> Result<(IV, I, I)> {
    let direction = line.v(I::point(0.));
    let origin = line.q(I::point(0.), I::point(d));
    let value = |t: f64| cross(vs(curve.q(I::point(t), I::point(d)), origin), direction);
    let derivative_box = |a: f64, b: f64, drange: I| {
        let local = Source {
            p: restrict(curve.p, a, b),
            ..curve.clone()
        };
        let v = controls_hull(derivative(local.p));
        let speed = norm(v);
        let factor = I::point(1.).add(
            drange.mul(cross(v, controls_hull(accel(local.p))).div(speed.mul(speed).mul(speed))),
        );
        cross(vm(v, factor), direction)
    };
    let mut tasks = vec![(0., 1., 0u32)];
    let mut sign = 0;
    while let Some((a, b, depth)) = tasks.pop() {
        work.step()?;
        let v = derivative_box(a, b, I::range(0., d));
        if v.excludes_zero() {
            let s = if v.lo > 0. { 1 } else { -1 };
            ensure(
                sign == 0 || sign == s,
                &format!(
                    "OffsetAmbiguousTrim: curve {} derivative changes sign",
                    curve.id
                ),
            )?;
            sign = s;
        } else {
            ensure(
                depth < MAX_DEPTH,
                &format!(
                    "OffsetUnprovedTrim: curve {} transversality unresolved",
                    curve.id
                ),
            )?;
            let m = (a + b) * 0.5;
            tasks.extend([(m, b, depth + 1), (a, m, depth + 1)]);
        }
    }
    let (mut a, mut b) = (0., 1.);
    let fa = value(a);
    let fb = value(b);
    ensure(
        (fa.lo > 0. && fb.hi < 0.) || (fa.hi < 0. && fb.lo > 0.),
        &format!(
            "OffsetLostSpan: curve {} has no interior adjacent trim root",
            curve.id
        ),
    )?;
    let positive = fa.lo > 0.;
    for _ in 0..MAX_DEPTH {
        work.step()?;
        let m = (a + b) * 0.5;
        let f = value(m);
        let box_q = curve.bounds(a, b, I::point(d));
        if norm(vs(box_q, iv(mid(box_q)))).hi <= epsilon / 64. {
            // Both coordinates enclose the exact intersection, not merely a sampled root.
            let root = curve.bounds(a, b, I::point(d));
            let tline = dot(vs(root, origin), direction).div(dot(direction, direction));
            return Ok((root, I::range(a, b), tline));
        }
        if f.lo > 0. {
            if positive {
                a = m;
            } else {
                b = m;
            }
        } else if f.hi < 0. {
            if positive {
                b = m;
            } else {
                a = m;
            }
        } else {
            // Preserve the proven bracket while narrowing around an arithmetic-uncertain midpoint.
            let l = (a + m) * 0.5;
            let r = (m + b) * 0.5;
            let fl = value(l);
            let fr = value(r);
            if (fl.lo > 0. && fr.hi < 0.) || (fl.hi < 0. && fr.lo > 0.) {
                a = l;
                b = r;
            } else {
                return Err(Error(format!(
                    "OffsetNumericUncertainty: curve {} trim root cannot be isolated",
                    curve.id
                )));
            }
        }
    }
    Err(Error(format!(
        "OffsetTrimBudgetExceeded: curve {} root depth limit",
        curve.id
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbounded_interval_arithmetic_never_narrows_nan_products() {
        for x in [
            I::entire(),
            I {
                lo: 0.,
                hi: f64::INFINITY,
            },
            I::point(f64::INFINITY),
        ] {
            let p = x.mul(I::point(0.));
            assert_eq!(p.lo, f64::NEG_INFINITY);
            assert_eq!(p.hi, f64::INFINITY);
        }
        let p = I::point(f64::INFINITY).add(I::point(f64::NEG_INFINITY));
        assert_eq!(p.lo, f64::NEG_INFINITY);
        assert_eq!(p.hi, f64::INFINITY);
        let p = I::point(1.).div(I::range(-1., 1.)).mul(I::point(0.));
        assert!(p.lo <= 0. && p.hi >= 0.);
    }

    #[test]
    fn endpoint_only_fit_is_rejected_by_the_whole_cell_bound() {
        let source = [[0.4, 0.], [0.5, 0.2], [0.5, 0.4], [0.4, 0.6]].map(iv);
        let s = Source {
            id: 1,
            start: 1,
            p: source,
            line: false,
        };
        let a = mid(s.q(I::point(0.), I::point(0.01)));
        let b = mid(s.q(I::point(1.), I::point(0.01)));
        let candidate = [a, [0.8, 0.2], [0.8, 0.4], b].map(iv);
        let mut work = Work {
            units: Cell::new(0),
            cancelled: &|| false,
        };
        assert!(
            certificate(source, candidate, 0.01, 1e-5, 0, &mut work)
                .unwrap()
                .is_none()
        );
        assert!(work.units.get() > 1);
    }

    #[test]
    fn validation_subdivision_uses_the_shared_request_poll_and_work_counter() {
        let mut editor = crate::Editor::default();
        editor
            .execute(crate::Command::AddEllipse {
                name: "Original validation cancellation".into(),
                origin: [0., 0.],
                width: 0.8,
                height: 0.6,
            })
            .unwrap();
        let polls = Cell::new(0);
        let cancelled = || {
            polls.set(polls.get() + 1);
            polls.get() == 2
        };
        let work = Work {
            units: Cell::new(0),
            cancelled: &cancelled,
        };
        let error = editor.project.validate_for_offset(&work).unwrap_err();
        assert!(error.to_string().contains("OffsetCancelled"));
        assert_eq!(polls.get(), 2);
        assert!(work.units.get() > 0 && work.units.get() <= 256);
        let complete = Work {
            units: Cell::new(0),
            cancelled: &|| false,
        };
        editor.project.validate_for_offset(&complete).unwrap();
        assert!(complete.units.get() > 256);
        let work = Work {
            units: Cell::new(MAX_WORK),
            cancelled: &|| false,
        };
        assert!(
            work.step()
                .unwrap_err()
                .to_string()
                .contains("OffsetBudgetExceeded")
        );
        assert_eq!(work.units.get(), MAX_WORK);
    }

    #[test]
    fn cusp_regularization_and_unrepresentable_arithmetic_refuse() {
        let source = Source {
            id: 77,
            start: 3,
            p: [[0.2, 0.], [0., 0.4], [0.2, 0.4], [0., 0.]].map(iv),
            line: false,
        };
        let mut work = Work {
            units: Cell::new(0),
            cancelled: &|| false,
        };
        let error = regular(&source, I::range(0., 0.01), 0, &mut work).unwrap_err();
        assert!(error.to_string().contains("77"));
        assert!(bounded_point([I::range(1000., 1000_f64.next_up()), I::point(0.)], 1e-20).is_err());
    }
}

type Matrix = [[I; 2]; 2];
fn jacobian(a: &Source, b: &Source, x: [I; 2], d: I) -> Matrix {
    let av = a.prime_range([x[0].lo, x[0].hi], d);
    let bv = b.prime_range([x[1].lo, x[1].hi], d);
    [[av[0], bv[0].neg()], [av[1], bv[1].neg()]]
}
fn inverse(j: Matrix) -> Option<Matrix> {
    let det = j[0][0].mul(j[1][1]).sub(j[0][1].mul(j[1][0]));
    det.excludes_zero().then(|| {
        [
            [j[1][1].div(det), j[0][1].neg().div(det)],
            [j[1][0].neg().div(det), j[0][0].div(det)],
        ]
    })
}
fn matrix_vector(m: Matrix, v: [I; 2]) -> [I; 2] {
    std::array::from_fn(|i| m[i][0].mul(v[0]).add(m[i][1].mul(v[1])))
}

/// A uniform contraction maps this parameter box into its interior for every d in the interval.
/// Thus a unique intersection exists for every intermediate distance, by the fixed-point theorem.
fn root_box(a: &Source, b: &Source, x: [I; 2], d: I, work: &mut Work<'_>) -> Result<bool> {
    work.step()?;
    let center = x.map(I::mid);
    let v = a.qp(I::point(center[0]), I::point(d.mid()));
    let w = b.qp(I::point(center[1]), I::point(d.mid()));
    let j0 = [[v[0], w[0].neg()], [v[1], w[1].neg()]];
    let Some(y) = inverse(j0).map(|m| m.map(|row| row.map(|x| I::point(x.mid())))) else {
        return Ok(false);
    };
    let j = jacobian(a, b, x, d);
    let residual = vs(a.q(I::point(center[0]), d), b.q(I::point(center[1]), d));
    let base = vs(center.map(I::point), matrix_vector(y, residual));
    let r: Matrix = std::array::from_fn(|i| {
        std::array::from_fn(|k| {
            I::point(if i == k { 1. } else { 0. })
                .sub(y[i][0].mul(j[0][k]).add(y[i][1].mul(j[1][k])))
        })
    });
    if r.iter().any(|row| {
        I::point(row[0].lo.abs().max(row[0].hi.abs()))
            .add(I::point(row[1].lo.abs().max(row[1].hi.abs())))
            .hi
            >= 1.
    }) {
        return Ok(false);
    }
    let radius = std::array::from_fn(|i| x[i].sub(I::point(center[i])));
    let k = va(base, matrix_vector(r, radius));
    Ok((0..2).all(|i| k[i].lo > x[i].lo && k[i].hi < x[i].hi))
}
fn exclude_other_roots(
    a: &Source,
    b: &Source,
    ranges: [[f64; 2]; 2],
    known: [I; 2],
    d: I,
    depth: u32,
    work: &mut Work<'_>,
) -> Result<()> {
    work.step()?;
    let [ar, br] = ranges;
    if ar[0] >= known[0].lo && ar[1] <= known[0].hi && br[0] >= known[1].lo && br[1] <= known[1].hi
    {
        return Ok(());
    }
    let ab = a.bounds(ar[0], ar[1], d);
    let bb = b.bounds(br[0], br[1], d);
    if disjoint(ab, bb) {
        return Ok(());
    }
    ensure(
        depth < MAX_DEPTH,
        &format!(
            "OffsetAmbiguousTrim: curves {} and {} have another or unresolved intersection",
            a.id, b.id
        ),
    )?;
    if box_size(ab) >= box_size(bb) {
        let m = (ar[0] + ar[1]) * 0.5;
        exclude_other_roots(a, b, [[ar[0], m], br], known, d, depth + 1, work)?;
        exclude_other_roots(a, b, [[m, ar[1]], br], known, d, depth + 1, work)
    } else {
        let m = (br[0] + br[1]) * 0.5;
        exclude_other_roots(a, b, [ar, [br[0], m]], known, d, depth + 1, work)?;
        exclude_other_roots(a, b, [ar, [m, br[1]]], known, d, depth + 1, work)
    }
}
fn trim_cubic_pair(
    a: &Source,
    b: &Source,
    d: f64,
    epsilon: f64,
    work: &mut Work<'_>,
) -> Result<(IV, I, I)> {
    let p = a.q(I::point(1.), I::point(d));
    let q = b.q(I::point(0.), I::point(d));
    let (_, x, y) = line_intersection(
        p,
        a.qp(I::point(1.), I::point(d)),
        q,
        b.qp(I::point(0.), I::point(d)),
    )?;
    let mut t = [1. + x.mid(), y.mid()];
    for _ in 0..32 {
        work.step()?;
        ensure(
            t.into_iter().all(|v| v > 0. && v < 1.),
            &format!(
                "OffsetLostSpan: curves {} and {} have no local ordered trim",
                a.id, b.id
            ),
        )?;
        let v = a.qp(I::point(t[0]), I::point(d));
        let w = b.qp(I::point(t[1]), I::point(d));
        let j = [[v[0], w[0].neg()], [v[1], w[1].neg()]];
        let inv = inverse(j)
            .ok_or_else(|| Error("OffsetUnprovedTrim: singular intersection Jacobian".into()))?;
        let step = matrix_vector(
            inv,
            vs(
                a.q(I::point(t[0]), I::point(d)),
                b.q(I::point(t[1]), I::point(d)),
            ),
        )
        .map(I::mid);
        let next = std::array::from_fn(|i| t[i] - step[i]);
        if t == next {
            break;
        }
        t = next;
    }
    let speed = norm(a.v(I::point(t[0])))
        .hi
        .max(norm(b.v(I::point(t[1]))).hi);
    let radius = epsilon / (4096. * speed);
    ensure(
        radius > 0. && radius.is_finite(),
        "OffsetNumericUncertainty: trim parameter enclosure is unrepresentable",
    )?;
    let known = t.map(|t| I::range((t - radius).next_down(), (t + radius).next_up()));
    ensure(
        root_box(a, b, known, I::point(d), work)?,
        &format!(
            "OffsetUnprovedTrim: curves {} and {} local root certificate failed",
            a.id, b.id
        ),
    )?;
    let da = 1. - known[0].lo;
    let db = known[1].hi;
    let drange = I::range(0., d);
    let mut certified = None;
    for padding in [0.25, 0.5, 1., 2.] {
        let branch = [
            I::range(
                (known[0].lo - da * padding).next_down(),
                (1. + da * padding).next_up(),
            ),
            I::range(
                (-db * padding).next_down(),
                (known[1].hi + db * padding).next_up(),
            ),
        ];
        if root_box(a, b, branch, drange, work)? {
            certified = Some(branch);
            break;
        }
    }
    let branch=certified.ok_or_else(||Error(format!("OffsetUnprovedContinuation: curves {} and {} trim branch could not be certified across widths",a.id,b.id)))?;
    let inv = inverse(jacobian(a, b, branch, drange)).ok_or_else(|| {
        Error("OffsetUnprovedContinuation: trim branch loses transversality".into())
    })?;
    let rhs = vm(
        vs(
            b.normal_range([branch[1].lo, branch[1].hi]),
            a.normal_range([branch[0].lo, branch[0].hi]),
        ),
        I::point(d.signum()),
    );
    let motion = matrix_vector(inv, rhs);
    ensure(
        motion[0].hi < 0. && motion[1].lo > 0.,
        "OffsetUnprovedContinuation: adjacent source parameters do not trim monotonically",
    )?;
    exclude_other_roots(a, b, [[0., 1.], [0., 1.]], branch, drange, 0, work)?;
    let point = hull(
        a.bounds(known[0].lo, known[0].hi, I::point(d)),
        b.bounds(known[1].lo, known[1].hi, I::point(d)),
    );
    Ok((point, known[0], known[1]))
}

fn joins(
    sources: &[Source],
    d: f64,
    epsilon: f64,
    work: &mut Work<'_>,
) -> Result<(Vec<Span>, Vec<Corner>)> {
    let n = sources.len();
    let mut spans = Vec::with_capacity(n);
    for s in sources {
        spans.push(Span {
            a: 0.,
            b: 1.,
            start: bounded_point(s.q(I::point(0.), I::point(d)), epsilon / 64.)?,
            end: bounded_point(s.q(I::point(1.), I::point(d)), epsilon / 64.)?,
        });
    }
    let mut corners = Vec::with_capacity(n);
    for j in 0..n {
        work.step()?;
        let i = (j + n - 1) % n;
        let a = &sources[i];
        let b = &sources[j];
        let old = b.p[0];
        let p = a.q(I::point(1.), I::point(d));
        let q = b.q(I::point(0.), I::point(d));
        let u = a.v(I::point(1.));
        let v = b.v(I::point(0.));
        let t = turn(a, b)?;
        if t == 0 {
            ensure(
                dot(u, v).lo > 0.,
                &format!("OffsetAntiparallelTangents: vertex {}", b.start),
            )?;
            let point = bounded_point(hull(p, q), epsilon / 64.)?;
            spans[i].end = point;
            spans[j].start = point;
            corners.push(Corner {
                vertices: vec![point],
                retained: Some(0),
                sweep: hull(old, iv(point)),
                kind: OutlineJoinKind::Smooth,
                trim_parameters: None,
            });
            continue;
        }
        if f64::from(t) * d > 0. {
            let (m, x, y) = line_intersection(p, u, q, v)?;
            ensure(
                x.lo > 0. && y.hi < 0.,
                &format!(
                    "OffsetUnprovedJoin: vertex {} tangent arms reverse",
                    b.start
                ),
            )?;
            let ratio = norm(vs(m, old)).div(I::point(d.abs()));
            if ratio.lo > 4. {
                let p = bounded_point(p, epsilon / 64.)?;
                let q = bounded_point(q, epsilon / 64.)?;
                corners.push(Corner {
                    vertices: vec![p, q],
                    retained: None,
                    sweep: hull(hull(old, iv(p)), iv(q)),
                    kind: OutlineJoinKind::Bevel,
                    trim_parameters: None,
                });
            } else {
                ensure(
                    ratio.hi <= 4.,
                    &format!("OffsetUncertainMiterLimit: vertex {}", b.start),
                )?;
                let point = bounded_point(m, epsilon / 64.)?;
                let mut vertices = Vec::new();
                if a.line {
                    spans[i].end = point;
                    spans[i].b = I::point(1.).add(x).mid();
                } else {
                    vertices.push(spans[i].end);
                }
                let retained = vertices.len();
                vertices.push(point);
                if b.line {
                    spans[j].start = point;
                    spans[j].a = y.mid();
                } else {
                    vertices.push(spans[j].start);
                }
                corners.push(Corner {
                    vertices,
                    retained: Some(retained),
                    sweep: hull(hull(hull(old, p), q), m),
                    kind: OutlineJoinKind::Miter,
                    trim_parameters: None,
                });
            }
        } else {
            let (point, ta, tb) = if a.line && b.line {
                let (m, x, y) = line_intersection(p, u, q, v)?;
                (m, I::point(1.).add(x), y)
            } else if a.line {
                let (m, tcurve, tline) = trim_line_cubic(a, b, d, epsilon, work)?;
                (m, tline, tcurve)
            } else if b.line {
                let (m, tcurve, tline) = trim_line_cubic(b, a, d, epsilon, work)?;
                (m, tcurve, tline)
            } else {
                trim_cubic_pair(a, b, d, epsilon, work)?
            };
            ensure(
                ta.lo > 0. && ta.hi < 1. && tb.lo > 0. && tb.hi < 1.,
                &format!(
                    "OffsetLostSpan: vertex {} trim is outside source spans",
                    b.start
                ),
            )?;
            let point = bounded_point(point, epsilon / 32.)?;
            spans[i].b = ta.mid();
            spans[i].end = point;
            spans[j].a = tb.mid();
            spans[j].start = point;
            corners.push(Corner {
                vertices: vec![point],
                retained: Some(0),
                // Both trim parameters move monotonically from the shared source vertex.
                // Enclose the complete continuous trim loci across every intermediate width.
                sweep: hull(
                    a.bounds(ta.lo, 1., I::range(0., d)),
                    b.bounds(0., tb.hi, I::range(0., d)),
                ),
                kind: OutlineJoinKind::AdjacentTrim,
                trim_parameters: Some([[ta.lo, ta.hi], [tb.lo, tb.hi]]),
            });
        }
    }
    for (s, span) in sources.iter().zip(&spans) {
        ensure(
            span.b > span.a,
            &format!("OffsetCollapse: curve {} source interval reversed", s.id),
        )?;
        if s.line {
            ensure(
                dot(vs(iv(span.end), iv(span.start)), s.v(I::point(0.))).lo > 0.,
                &format!("OffsetCollapse: line {} projection reversed", s.id),
            )?;
        }
    }
    Ok((spans, corners))
}

#[derive(Clone)]
struct Fit {
    p: [V; 4],
    line: bool,
    source: Option<(usize, [f64; 2], f64)>,
    start_id: Option<Id>,
}

fn certificate(
    source: B,
    candidate: B,
    d: f64,
    epsilon: f64,
    depth: u32,
    work: &mut Work<'_>,
) -> Result<Option<f64>> {
    work.step()?;
    let s = Source {
        id: 0,
        start: 0,
        p: source,
        line: false,
    };
    let r0 = vs(s.q(I::point(0.), I::point(d)), candidate[0]);
    let r1 = vs(s.q(I::point(1.), I::point(d)), candidate[3]);
    let e0 = norm(r0);
    let e1 = norm(r1);
    if e0.lo > epsilon || e1.lo > epsilon {
        return Ok(None);
    }
    let v0 = norm(controls_hull(derivative(source))).lo;
    if v0 > 0. {
        let a = accel(source)
            .map(|x| norm(x).hi)
            .into_iter()
            .fold(0., f64::max);
        let b = norm(vm(vs(accel(source)[1], accel(source)[0]), I::point(1.))).hi;
        let k = accel(candidate)
            .map(|x| norm(x).hi)
            .into_iter()
            .fold(0., f64::max);
        let v = I::point(v0);
        let a = I::point(a);
        let m = a
            .add(I::point(d.abs()).mul(I::point(b).div(v).add(a.mul(a).scale(6.).div(v.mul(v)))))
            .add(I::point(k));
        let bound = I::point(e0.hi.max(e1.hi)).add(m.scale(0.125)).hi;
        if bound <= epsilon {
            return Ok(Some(bound));
        }
    }
    if depth == MAX_DEPTH {
        return Ok(None);
    }
    let (sa, sb) = split(source, I::point(0.5));
    let (fa, fb) = split(candidate, I::point(0.5));
    let Some(a) = certificate(sa, fa, d, epsilon, depth + 1, work)? else {
        return Ok(None);
    };
    Ok(certificate(sb, fb, d, epsilon, depth + 1, work)?.map(|b| a.max(b)))
}

struct Fitter<'a, 'b> {
    d: f64,
    epsilon: f64,
    limit: usize,
    work: &'a mut Work<'b>,
    output: Vec<Fit>,
}
impl Fitter<'_, '_> {
    fn fit(&mut self, s: &Source, index: usize, span: &Span, depth: u32) -> Result<()> {
        self.work.step()?;
        ensure(
            self.output.len() < self.limit,
            "OffsetOutputBudgetExceeded: remaining document object budget exhausted",
        )?;
        if s.line {
            let a = s.q(I::point(span.a), I::point(self.d));
            let b = s.q(I::point(span.b), I::point(self.d));
            let error = norm(vs(a, iv(span.start)))
                .hi
                .max(norm(vs(b, iv(span.end))).hi);
            ensure(
                error <= self.epsilon,
                &format!(
                    "OffsetNumericUncertainty: line {} error enclosure exceeds tolerance",
                    s.id
                ),
            )?;
            self.output.push(Fit {
                p: [span.start, span.start, span.end, span.end],
                line: true,
                source: Some((index, [span.a, span.b], error)),
                start_id: None,
            });
            return Ok(());
        }
        let h = I::point(span.b).sub(I::point(span.a)).div(I::point(3.));
        let p = [
            span.start,
            mid(va(
                iv(span.start),
                vm(s.qp(I::point(span.a), I::point(self.d)), h),
            )),
            mid(vs(
                iv(span.end),
                vm(s.qp(I::point(span.b), I::point(self.d)), h),
            )),
            span.end,
        ];
        if let Some(error) = certificate(
            restrict(s.p, span.a, span.b),
            p.map(iv),
            self.d,
            self.epsilon,
            0,
            self.work,
        )? {
            self.output.push(Fit {
                p,
                line: false,
                source: Some((index, [span.a, span.b], error)),
                start_id: None,
            });
            return Ok(());
        }
        ensure(
            depth < MAX_DEPTH,
            &format!(
                "OffsetFitBudgetExceeded: curve {} cannot establish a whole-cell fit bound",
                s.id
            ),
        )?;
        let t = (span.a + span.b) * 0.5;
        let q = bounded_point(s.q(I::point(t), I::point(self.d)), self.epsilon / 64.)?;
        self.fit(
            s,
            index,
            &Span {
                b: t,
                end: q,
                ..span.clone()
            },
            depth + 1,
        )?;
        self.fit(
            s,
            index,
            &Span {
                a: t,
                start: q,
                ..span.clone()
            },
            depth + 1,
        )
    }
}

fn disjoint(a: IV, b: IV) -> bool {
    (0..2).any(|i| a[i].hi < b[i].lo || b[i].hi < a[i].lo)
}
fn box_size(a: IV) -> f64 {
    (a[0].hi - a[0].lo).max(a[1].hi - a[1].lo)
}

fn swept_pair(
    a: &Source,
    ar: [f64; 2],
    b: &Source,
    br: [f64; 2],
    d: I,
    depth: u32,
    work: &mut Work<'_>,
) -> Result<()> {
    work.step()?;
    let ab = a.bounds(ar[0], ar[1], d);
    let bb = b.bounds(br[0], br[1], d);
    if disjoint(ab, bb) {
        return Ok(());
    }
    ensure(
        depth < MAX_DEPTH,
        &format!(
            "OffsetNonAdjacentContact: swept curves {} and {} intersect or cannot be separated before the requested width",
            a.id, b.id
        ),
    )?;
    if box_size(ab) >= box_size(bb) {
        let m = (ar[0] + ar[1]) * 0.5;
        swept_pair(a, [ar[0], m], b, br, d, depth + 1, work)?;
        swept_pair(a, [m, ar[1]], b, br, d, depth + 1, work)
    } else {
        let m = (br[0] + br[1]) * 0.5;
        swept_pair(a, ar, b, [br[0], m], d, depth + 1, work)?;
        swept_pair(a, ar, b, [m, br[1]], d, depth + 1, work)
    }
}
fn swept_corner(
    s: &Source,
    range: [f64; 2],
    corner: IV,
    d: I,
    depth: u32,
    work: &mut Work<'_>,
) -> Result<()> {
    work.step()?;
    if disjoint(s.bounds(range[0], range[1], d), corner) {
        return Ok(());
    }
    ensure(
        depth < MAX_DEPTH,
        &format!(
            "OffsetNonAdjacentContact: curve {} meets another corner's swept enclosure",
            s.id
        ),
    )?;
    let m = (range[0] + range[1]) * 0.5;
    swept_corner(s, [range[0], m], corner, d, depth + 1, work)?;
    swept_corner(s, [m, range[1]], corner, d, depth + 1, work)
}
fn monotone_source(s: &Source, range: [f64; 2], direction: IV) -> bool {
    dot(s.velocity_range(range), direction).lo > 0.
}
fn monotone_chord(s: &Source, range: [f64; 2]) -> bool {
    let direction = iv(mid(vs(s.c(I::point(range[1])), s.c(I::point(range[0])))));
    monotone_source(s, range, direction)
}
fn same_source_cells(
    s: &Source,
    ar: [f64; 2],
    br: [f64; 2],
    d: I,
    depth: u32,
    work: &mut Work<'_>,
) -> Result<()> {
    work.step()?;
    // Regular offsets retain the source tangent, so this projection is ordered for every width.
    if monotone_chord(s, [ar[0], br[1]]) {
        return Ok(());
    }
    let a = s.bounds(ar[0], ar[1], d);
    let b = s.bounds(br[0], br[1], d);
    if disjoint(a, b) {
        return Ok(());
    }
    ensure(
        depth < MAX_DEPTH,
        &format!(
            "OffsetSelfContactUnproved: curve {} has unresolved nonlocal swept contact",
            s.id
        ),
    )?;
    if box_size(a) >= box_size(b) {
        let m = (ar[0] + ar[1]) * 0.5;
        same_source_cells(s, [ar[0], m], br, d, depth + 1, work)?;
        same_source_cells(s, [m, ar[1]], br, d, depth + 1, work)
    } else {
        let m = (br[0] + br[1]) * 0.5;
        same_source_cells(s, ar, [br[0], m], d, depth + 1, work)?;
        same_source_cells(s, ar, [m, br[1]], d, depth + 1, work)
    }
}
fn self_sweep(s: &Source, range: [f64; 2], d: I, depth: u32, work: &mut Work<'_>) -> Result<()> {
    work.step()?;
    if monotone_chord(s, range) {
        return Ok(());
    }
    ensure(
        depth < MAX_DEPTH,
        &format!(
            "OffsetSelfContactUnproved: curve {} injectivity across widths could not be certified",
            s.id
        ),
    )?;
    let m = (range[0] + range[1]) * 0.5;
    let a = [range[0], m];
    let b = [m, range[1]];
    self_sweep(s, a, d, depth + 1, work)?;
    self_sweep(s, b, d, depth + 1, work)?;
    same_source_cells(s, a, b, d, depth + 1, work)
}

fn bisector(a: &Source, b: &Source) -> IV {
    let u = a.v(I::point(1.));
    let v = b.v(I::point(0.));
    // A fixed represented direction suffices; all sign decisions remain enclosed.
    iv(mid(va(
        vm(u, I::point(1.).div(norm(u))),
        vm(v, I::point(1.).div(norm(v))),
    )))
}
fn gap_sources(
    a: &Source,
    b: &Source,
    ranges: [[f64; 2]; 2],
    direction: IV,
    d: I,
    depth: u32,
    work: &mut Work<'_>,
) -> Result<()> {
    work.step()?;
    let [ar, br] = ranges;
    if ar[1] == 1.
        && br[0] == 0.
        && monotone_source(a, ar, direction)
        && monotone_source(b, br, direction)
    {
        return Ok(());
    }
    let ab = a.bounds(ar[0], ar[1], d);
    let bb = b.bounds(br[0], br[1], d);
    if disjoint(ab, bb) {
        return Ok(());
    }
    ensure(
        depth < MAX_DEPTH,
        &format!(
            "OffsetAdjacentContactUnproved: curves {} and {} have unresolved nonlocal swept contact",
            a.id, b.id
        ),
    )?;
    if box_size(ab) >= box_size(bb) {
        let m = (ar[0] + ar[1]) * 0.5;
        gap_sources(a, b, [[ar[0], m], br], direction, d, depth + 1, work)?;
        gap_sources(a, b, [[m, ar[1]], br], direction, d, depth + 1, work)
    } else {
        let m = (br[0] + br[1]) * 0.5;
        gap_sources(a, b, [ar, [br[0], m]], direction, d, depth + 1, work)?;
        gap_sources(a, b, [ar, [m, br[1]]], direction, d, depth + 1, work)
    }
}
fn incident_join(
    s: &Source,
    range: [f64; 2],
    incoming: bool,
    join: (IV, IV),
    d: I,
    depth: u32,
    work: &mut Work<'_>,
) -> Result<()> {
    work.step()?;
    let (corner, direction) = join;
    let to_vertex = if incoming {
        [range[0], 1.]
    } else {
        [0., range[1]]
    };
    if monotone_source(s, to_vertex, direction) || disjoint(s.bounds(range[0], range[1], d), corner)
    {
        return Ok(());
    }
    ensure(
        depth < MAX_DEPTH,
        &format!(
            "OffsetJoinContactUnproved: curve {} meets its own corner sweep",
            s.id
        ),
    )?;
    let m = (range[0] + range[1]) * 0.5;
    incident_join(s, [range[0], m], incoming, join, d, depth + 1, work)?;
    incident_join(s, [m, range[1]], incoming, join, d, depth + 1, work)
}
fn continuation(sources: &[Source], corners: &[Corner], d: f64, work: &mut Work<'_>) -> Result<()> {
    let n = sources.len();
    let drange = I::range(0., d);
    for s in sources {
        self_sweep(s, [0., 1.], drange, 0, work)?;
    }
    for (j, corner) in corners.iter().enumerate() {
        if corner.kind == OutlineJoinKind::AdjacentTrim {
            continue;
        }
        let a = &sources[(j + n - 1) % n];
        let b = &sources[j];
        let direction = bisector(a, b);
        if corner.kind != OutlineJoinKind::Smooth {
            let coefficient = dot(
                vs(b.normal_range([0., 0.]), a.normal_range([1., 1.])),
                direction,
            )
            .scale(d.signum());
            ensure(
                coefficient.lo > 0.,
                &format!(
                    "OffsetUnprovedGap: vertex {} endpoint separation is uncertain",
                    b.start
                ),
            )?;
        }
        gap_sources(a, b, [[0., 1.], [0., 1.]], direction, drange, 0, work)?;
        incident_join(
            a,
            [0., 1.],
            true,
            (corner.sweep, direction),
            drange,
            0,
            work,
        )?;
        incident_join(
            b,
            [0., 1.],
            false,
            (corner.sweep, direction),
            drange,
            0,
            work,
        )?;
    }
    for (i, a) in sources.iter().enumerate() {
        for (j, b) in sources.iter().enumerate().skip(i + 1) {
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue;
            }
            swept_pair(a, [0., 1.], b, [0., 1.], drange, 0, work)?;
        }
        for (j, c) in corners.iter().enumerate() {
            if j == i || j == (i + 1) % n {
                continue;
            }
            swept_corner(a, [0., 1.], c.sweep, drange, 0, work)?;
        }
    }
    for i in 0..n {
        for j in i + 1..n {
            work.step()?;
            ensure(
                disjoint(corners[i].sweep, corners[j].sweep),
                &format!(
                    "OffsetCollapseOrCornerContact: vertices {} and {} have overlapping swept enclosures",
                    sources[i].start, sources[j].start
                ),
            )?;
        }
    }
    Ok(())
}

#[derive(Clone)]
struct ContourNode {
    p: B,
    line: bool,
}
impl ContourNode {
    fn bounds(&self) -> IV {
        controls_hull(self.p)
    }
    fn half(&self) -> (Self, Self) {
        let (a, b) = split(self.p, I::point(0.5));
        (
            Self {
                p: a,
                line: self.line,
            },
            Self {
                p: b,
                line: self.line,
            },
        )
    }
}
fn separated_at_vertex(a: &ContourNode, b: &ContourNode, common: IV) -> bool {
    let incoming = vs(a.p[3], if a.line { a.p[0] } else { a.p[2] });
    let outgoing = vs(if b.line { b.p[3] } else { b.p[1] }, b.p[0]);
    let direction = va(
        vm(incoming, I::point(1.).div(norm(incoming))),
        vm(outgoing, I::point(1.).div(norm(outgoing))),
    );
    a.p[..3]
        .iter()
        .enumerate()
        .all(|(i, p)| (a.line && i > 0) || dot(vs(*p, common), direction).hi < 0.)
        && b.p[1..]
            .iter()
            .enumerate()
            .all(|(i, p)| (b.line && i < 2) || dot(vs(*p, common), direction).lo > 0.)
}
fn contour_pair(
    a: &ContourNode,
    b: &ContourNode,
    common: Option<IV>,
    depth: u32,
    work: &mut Work<'_>,
) -> Result<()> {
    work.step()?;
    if disjoint(a.bounds(), b.bounds()) || common.is_some_and(|p| separated_at_vertex(a, b, p)) {
        return Ok(());
    }
    ensure(
        depth < MAX_DEPTH,
        "OffsetTopologyUnproved: actual authored curves intersect, touch or cannot be separated",
    )?;
    if box_size(a.bounds()) >= box_size(b.bounds()) {
        let (x, y) = a.half();
        contour_pair(&x, b, None, depth + 1, work)?;
        contour_pair(&y, b, common, depth + 1, work)
    } else {
        let (x, y) = b.half();
        contour_pair(a, &x, common, depth + 1, work)?;
        contour_pair(a, &y, None, depth + 1, work)
    }
}
fn contour_nodes(
    node: ContourNode,
    depth: u32,
    out: &mut Vec<ContourNode>,
    work: &mut Work<'_>,
) -> Result<()> {
    work.step()?;
    ensure(
        out.len() < MAX_OUTPUT,
        "OffsetTopologyBudgetExceeded: pending contour tasks limit",
    )?;
    let chord = vs(node.p[3], node.p[0]);
    if node.line
        || derivative(node.p)
            .into_iter()
            .all(|v| dot(v, chord).lo > 0.)
    {
        out.push(node);
        return Ok(());
    }
    ensure(
        depth < MAX_DEPTH,
        "OffsetTopologyUnproved: cubic injectivity could not be certified",
    )?;
    let (a, b) = node.half();
    contour_nodes(a, depth + 1, out, work)?;
    contour_nodes(b, depth + 1, out, work)
}
fn simple_contour(fits: &[Fit], work: &mut Work<'_>) -> Result<()> {
    let mut nodes = Vec::new();
    for f in fits {
        contour_nodes(
            ContourNode {
                p: f.p.map(iv),
                line: f.line,
            },
            0,
            &mut nodes,
            work,
        )?;
    }
    let n = nodes.len();
    for i in 0..n {
        for j in i + 1..n {
            if j == i + 1 {
                contour_pair(&nodes[i], &nodes[j], Some(nodes[i].p[3]), 0, work)?;
            } else if i == 0 && j == n - 1 {
                contour_pair(&nodes[j], &nodes[i], Some(nodes[j].p[3]), 0, work)?;
            } else {
                contour_pair(&nodes[i], &nodes[j], None, 0, work)?;
            }
        }
    }
    Ok(())
}

fn preflight(project: &Project, piece: Id, width: f64) -> Result<()> {
    ensure(
        width.is_finite() && width > 0.,
        "OffsetInvalidWidth: width must be finite positive SI metres",
    )?;
    let owner = project
        .piece(piece)
        .ok_or_else(|| Error(format!("OffsetUnknownPiece: {piece}")))?;
    ensure(
        owner.boundary.len() <= MAX_INPUT,
        &format!("OffsetInputBudgetExceeded: piece {piece} exceeds 4096 source segments"),
    )?;
    ensure(
        owner.holes.is_empty(),
        &format!(
            "OffsetUnsupportedHole: piece {piece} contains hole {}",
            owner.holes.first().map_or(0, |h| h.id)
        ),
    )?;
    ensure(
        owner.internal_curves.is_empty(),
        &format!(
            "OffsetUnsupportedInternalConstruction: piece {piece} contains curve {}",
            owner.internal_curves.first().copied().unwrap_or(0)
        ),
    )?;
    let owned: BTreeSet<_> = owner.boundary.iter().copied().collect();
    for seam in &project.seams {
        ensure(
            !owned.contains(&seam.a.curve) && !owned.contains(&seam.b.curve),
            &format!(
                "OffsetUnsupportedSeam: piece {piece} is attached to seam {}",
                seam.id
            ),
        )?;
    }
    for group in &project.sewing_groups {
        ensure(
            !group
                .a
                .iter()
                .chain(&group.b)
                .any(|i| owned.contains(&i.curve)),
            &format!(
                "OffsetUnsupportedSewingGroup: piece {piece} is attached to group {}",
                group.id
            ),
        )?;
    }
    Ok(())
}

fn measurements(
    project: &Project,
    piece: Id,
    mesh: &Mesh,
    cloth: &Cloth,
) -> Result<OutlineMeasurements> {
    let owner = project
        .piece(piece)
        .ok_or_else(|| Error("OffsetUnknownPiece".into()))?;
    let mut area = I::point(0.);
    for id in &owner.boundary {
        area = area.add(source_area(&Source::from_curve(
            project,
            project.curve(*id).unwrap(),
        )));
    }
    let rest_area = mesh
        .triangles
        .iter()
        .filter(|t| mesh.piece_ids[t[0]] == piece)
        .map(|t| {
            let [a, b, c] = t.map(|i| mesh.uv[i]);
            ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() * 0.5
        })
        .sum();
    let cloth_mass = cloth
        .masses()
        .iter()
        .zip(&mesh.piece_ids)
        .filter(|(_, id)| **id == piece)
        .map(|(m, _)| m)
        .sum();
    Ok(OutlineMeasurements {
        bounds: project.piece_bounds(piece)?,
        authored_area: area.mid().abs(),
        rest_area,
        cloth_mass,
    })
}

impl Project {
    /// No editor state, permanent IDs, file or running solver is changed by preparation.
    /// The candidate includes a new rest mesh and an actual initialized Cloth before it can commit.
    pub fn prepare_outline_offset(
        &self,
        piece: Id,
        width: f64,
        outward: bool,
        cancelled: impl Fn() -> bool,
    ) -> Result<PreparedOutlineOffset> {
        let mut work = Work {
            units: Cell::new(0),
            cancelled: &cancelled,
        };
        work.check()?;
        preflight(self, piece, width)?;
        self.validate_for_offset(&work)?;
        work.check()?;
        let owner = self.piece(piece).unwrap();
        let sources: Vec<_> = owner
            .boundary
            .iter()
            .map(|id| Source::from_curve(self, self.curve(*id).unwrap()))
            .collect();
        let mut area = I::point(0.);
        for s in &sources {
            work.step()?;
            area = area.add(source_area(s));
        }
        ensure(
            area.excludes_zero(),
            &format!("OffsetUncertainWinding: piece {piece} has no certified area sign"),
        )?;
        let d = if (area.lo > 0.) == outward {
            width
        } else {
            -width
        };
        let epsilon = 1e-5_f64.next_down().min((width / 64.).next_down());
        ensure(
            epsilon > 0.,
            "OffsetNumericUncertainty: width is not representable at the required tolerance",
        )?;
        for s in &sources {
            regular(s, I::range(0., d), 0, &mut work)?;
        }
        let originals: Vec<_> = sources
            .iter()
            .map(|s| Fit {
                p: s.p.map(mid),
                line: s.line,
                source: None,
                start_id: None,
            })
            .collect();
        simple_contour(&originals, &mut work)?;
        let (spans, corners) = joins(&sources, d, epsilon, &mut work)?;
        continuation(&sources, &corners, d, &mut work)?;
        let objects = self.points.len()
            + self.curves.len()
            + self.pieces.len()
            + self.materials.len()
            + self.seams.len()
            + self.sewing_groups.len()
            + self.pieces.iter().map(|p| p.holes.len()).sum::<usize>();
        let other = objects - 2 * sources.len();
        let limit = MAX_OUTPUT.min((crate::MAX_OBJECTS - other) / 2);
        let join_count = corners.iter().map(|c| c.vertices.len() - 1).sum::<usize>();
        ensure(
            join_count < limit,
            "OffsetOutputBudgetExceeded: no document object budget for offset spans",
        )?;
        let mut output = Vec::new();
        for (i, s) in sources.iter().enumerate() {
            let c = &corners[i];
            for k in 0..c.vertices.len() - 1 {
                output.push(Fit {
                    p: [
                        c.vertices[k],
                        c.vertices[k],
                        c.vertices[k + 1],
                        c.vertices[k + 1],
                    ],
                    line: true,
                    source: None,
                    start_id: if c.retained == Some(k) {
                        Some(s.start)
                    } else {
                        None
                    },
                });
            }
            let remaining_joins = join_count
                - corners[..=i]
                    .iter()
                    .map(|c| c.vertices.len() - 1)
                    .sum::<usize>();
            let remaining = limit
                .checked_sub(output.len() + remaining_joins)
                .filter(|remaining| *remaining > 0)
                .ok_or_else(|| {
                    Error(
                        "OffsetOutputBudgetExceeded: remaining document object budget exhausted"
                            .into(),
                    )
                })?;
            let mut fitter = Fitter {
                d,
                epsilon,
                limit: remaining,
                work: &mut work,
                output: Vec::new(),
            };
            fitter.fit(s, i, &spans[i], 0)?;
            fitter.output[0].start_id = if c.retained == Some(c.vertices.len() - 1) {
                Some(s.start)
            } else {
                None
            };
            output.extend(fitter.output);
        }
        ensure(
            output.len() <= limit,
            "OffsetOutputBudgetExceeded: global document object limit",
        )?;
        simple_contour(&output, &mut work)?;
        let mut candidate = self.clone();
        let old_curves: BTreeSet<_> = sources.iter().map(|s| s.id).collect();
        let old_points: BTreeSet<_> = sources.iter().map(|s| s.start).collect();
        candidate.curves.retain(|c| !old_curves.contains(&c.id));
        candidate.points.retain(|p| !old_points.contains(&p.id));
        let mut point_ids = Vec::with_capacity(output.len());
        for f in &output {
            work.step()?;
            let id = match f.start_id {
                Some(id) => id,
                None => candidate.allocate_id()?,
            };
            point_ids.push(id);
            candidate.points.push(Point {
                id,
                position: f.p[0],
            });
        }
        let mut retained = BTreeSet::new();
        let mut boundary = Vec::new();
        let mut proof = Vec::new();
        let mut maximum_error: f64 = 0.;
        for (i, f) in output.iter().enumerate() {
            work.step()?;
            let id = match f.source {
                Some((s, _, _)) if retained.insert(s) => sources[s].id,
                _ => candidate.allocate_id()?,
            };
            candidate.curves.push(Curve {
                id,
                start: point_ids[i],
                end: point_ids[(i + 1) % point_ids.len()],
                controls: if f.line { None } else { Some([f.p[1], f.p[2]]) },
            });
            boundary.push(id);
            if let Some((s, range, error)) = f.source {
                maximum_error = maximum_error.max(error);
                proof.push(OutlineOffsetSpan {
                    source_curve: sources[s].id,
                    source_interval: range,
                    output_curve: id,
                    error_bound: error,
                });
            }
        }
        candidate
            .pieces
            .iter_mut()
            .find(|p| p.id == piece)
            .unwrap()
            .boundary = boundary;
        candidate.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| Error("Document revision limit reached".into()))?;
        candidate.validate_for_offset(&work)?;
        check_serialized_size(&candidate)?;
        work.check()?;
        // Meshing and solver initialization are publication prerequisites, not a later best effort.
        let before_mesh = self.mesh_for_offset(&work)?;
        let before_cloth = crate::simulation::create_cloth_for_offset(self, &before_mesh, &work)?;
        let before = measurements(self, piece, &before_mesh, &before_cloth)?;
        work.check()?;
        let mesh = candidate.mesh_for_offset(&work)?;
        ensure(
            mesh.piece_ids.iter().filter(|id| **id == piece).count() <= MAX_OUTPUT,
            "OffsetDerivedVertexBudgetExceeded: target piece exceeds 65536 vertices",
        )?;
        let cloth = crate::simulation::create_cloth_for_offset(&candidate, &mesh, &work)?;
        let after = measurements(&candidate, piece, &mesh, &cloth)?;
        work.check()?;
        Ok(PreparedOutlineOffset {
            source: self.clone(),
            candidate,
            mesh,
            cloth,
            piece,
            width,
            outward,
            before,
            after,
            certificate: OutlineOffsetCertificate {
                tolerance: epsilon,
                maximum_error,
                geometry_work: work.units.get(),
                spans: proof,
                joins: corners
                    .iter()
                    .enumerate()
                    .map(|(i, c)| OutlineOffsetJoin {
                        source_vertex: sources[i].start,
                        incoming_curve: sources[(i + sources.len() - 1) % sources.len()].id,
                        outgoing_curve: sources[i].id,
                        kind: c.kind,
                        trim_parameters: c.trim_parameters,
                    })
                    .collect(),
                continuous_topology: true,
                continuation: true,
            },
        })
    }
}

pub(crate) fn recheck(prepared: &PreparedOutlineOffset, source: &Project) -> Result<()> {
    ensure(
        source == &prepared.source,
        "OffsetStalePreparation: source construction changed; previous project retained",
    )?;
    preflight(source, prepared.piece, prepared.width)
}

pub(crate) fn check_serialized_size(project: &Project) -> Result<()> {
    #[derive(serde::Serialize)]
    struct Native<'a> {
        format: &'static str,
        version: u32,
        project: &'a Project,
    }
    let bytes = serde_json::to_vec_pretty(&Native {
        format: "weftforge",
        version: crate::FORMAT_VERSION,
        project,
    })?;
    ensure(
        bytes.len() as u64 <= crate::persistence::MAX_PROJECT_BYTES,
        "Project exceeds the 16 MiB file limit",
    )
}

fn chord_error(p: B, a: V, b: V) -> f64 {
    let delta = vs(iv(b), iv(a));
    let length = dot(delta, delta);
    if length.lo <= 0. {
        return f64::INFINITY;
    }
    p.into_iter()
        .map(|point| {
            let t = dot(vs(point, iv(a)), delta).div(length);
            let t = I {
                lo: t.lo.clamp(0., 1.),
                hi: t.hi.clamp(0., 1.),
            };
            norm(vs(point, va(iv(a), vm(delta, t)))).hi
        })
        .fold(0., f64::max)
}

pub(crate) fn sample_cubic_certified(
    p: [V; 4],
    spacing: f64,
    cancelled: &dyn Fn() -> bool,
    shared: Option<&Work<'_>>,
) -> Result<Vec<(f64, V)>> {
    fn flatten(
        p: B,
        range: [f64; 2],
        spacing: f64,
        depth: u32,
        out: &mut Vec<(f64, V)>,
        work: &Work<'_>,
    ) -> Result<()> {
        work.step()?;
        ensure(
            out.len() < 20_000,
            "Cubic tessellation sample limit exceeded",
        )?;
        let a = mid(p[0]);
        let b = mid(p[3]);
        if norm(vs(iv(a), iv(b))).hi <= spacing
            && chord_error(p, a, b) <= 1e-5_f64.next_down().min(spacing * 0.01)
        {
            out.push((range[1], b));
            return Ok(());
        }
        ensure(
            depth < 20,
            "Cubic could not be tessellated within certified 1e-5 metre tolerance",
        )?;
        let (a, b) = split(p, I::point(0.5));
        let m = (range[0] + range[1]) * 0.5;
        flatten(a, [range[0], m], spacing, depth + 1, out, work)?;
        flatten(b, [m, range[1]], spacing, depth + 1, out, work)
    }
    let local = Work {
        units: Cell::new(0),
        cancelled,
    };
    let work = shared.unwrap_or(&local);
    work.check()?;
    let mut samples = vec![(0., p[0])];
    flatten(p.map(iv), [0., 1.], spacing, 0, &mut samples, work)?;
    work.check()?;
    Ok(samples)
}
