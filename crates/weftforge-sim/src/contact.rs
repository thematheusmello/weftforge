use crate::{
    CANCELLATION_CHECK_INTERVAL, Cloth, Collider, SimulationError, Vec3, capsule, check_cancel,
    math::*,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(crate) struct InitialRecoveryEligibility {
    triangle_capsules: BTreeSet<(usize, usize)>,
    cloth: BTreeSet<(usize, usize)>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RecoveryProvenance {
    // Kinematic displacement is attribution only; balanced contact never retracts it.
    triangle_capsules: BTreeMap<(usize, usize), [Vec3; 3]>,
    cloth: BTreeMap<(usize, usize), [Vec3; 4]>,
    pub projections: usize,
}

struct InitialRecoveryPhase {
    eligibility: InitialRecoveryEligibility,
    provenance: RecoveryProvenance,
}

#[derive(Default)]
pub(crate) struct ContactState {
    vertex_colliders: BTreeMap<(usize, usize), [Vec3; 1]>,
    triangle_capsules: BTreeMap<(usize, usize), Vec<CapsuleSupport>>,
    cloth: BTreeMap<(usize, usize), [Vec3; 4]>,
    recovery: Option<InitialRecoveryPhase>,
}

impl ContactState {
    pub(crate) fn with_initial_recovery(
        eligibility: &InitialRecoveryEligibility,
        provenance: RecoveryProvenance,
    ) -> Self {
        if eligibility.triangle_capsules.is_empty() && eligibility.cloth.is_empty() {
            return Self::default();
        }
        Self {
            recovery: Some(InitialRecoveryPhase {
                eligibility: eligibility.clone(),
                provenance,
            }),
            ..Self::default()
        }
    }

    pub(crate) fn into_recovery_provenance(self) -> Option<RecoveryProvenance> {
        self.recovery.map(|phase| phase.provenance)
    }
}

struct CapsuleSupport {
    // This material point keeps its own action when another closest point appears.
    bary: Vec3,
    action: [Vec3; 3],
}

fn contact_anchor<const N: usize>(
    vertices: [usize; N],
    x: &[Vec3],
    inverse_mass: &[f64],
    old: [Vec3; N],
) -> Result<[Vec3; N], SimulationError> {
    let unstable = || SimulationError::Unstable("unusable contact action or geometry".into());
    let mut anchor = [[0.; 3]; N];
    for j in 0..N {
        let v = vertices[j];
        let weight = inverse_mass[v];
        if !weight.is_finite()
            || weight < 0.
            || !finite(x[v])
            || !finite(old[j])
            || (weight == 0. && old[j] != [0.; 3])
        {
            return Err(unstable());
        }
        anchor[j] = if weight > 0. { sub(x[v], old[j]) } else { x[v] };
        if !finite(anchor[j]) {
            return Err(unstable());
        }
    }
    Ok(anchor)
}

struct Contact<const N: usize> {
    vertices: [usize; N],
    coefficients: [f64; N],
    normal: Vec3,
    // Signed gap at the queried stencil; nonnegative means allowed separation.
    gap: f64,
    friction: f64,
    minimum_denominator: f64,
}

impl<const N: usize> Contact<N> {
    fn recover(
        self,
        x: &mut [Vec3],
        previous: &[Vec3],
        inverse_mass: &[f64],
        provenance: &mut BTreeMap<(usize, usize), [Vec3; N]>,
        projections: &mut usize,
        key: (usize, usize),
    ) -> Result<usize, SimulationError> {
        let unstable = || SimulationError::Unstable("unusable initial contact recovery".into());
        if !self.gap.is_finite()
            || !finite(self.normal)
            || !self.friction.is_finite()
            || self.friction < 0.
            || !self.coefficients.into_iter().all(f64::is_finite)
        {
            return Err(unstable());
        }
        if self.gap >= 0. {
            return Ok(0);
        }
        let before = contact_anchor(self.vertices, x, inverse_mass, [[0.; 3]; N])?;
        let mut denominator = 0.;
        for (j, &v) in self.vertices.iter().enumerate() {
            if !finite(previous[v]) {
                return Err(unstable());
            }
            denominator += inverse_mass[v] * self.coefficients[j].powi(2);
        }
        if !denominator.is_finite() {
            return Err(unstable());
        }
        if denominator <= self.minimum_denominator {
            return Ok(0);
        }
        let multiplier = -self.gap / denominator;
        if !multiplier.is_finite() {
            return Err(unstable());
        }
        let mut trial = before;
        for (j, &v) in self.vertices.iter().enumerate() {
            if inverse_mass[v] > 0. {
                trial[j] = add(
                    trial[j],
                    mul(
                        self.normal,
                        inverse_mass[v] * self.coefficients[j] * multiplier,
                    ),
                );
            }
        }
        let mut relative = [0.; 3];
        for (j, &v) in self.vertices.iter().enumerate() {
            relative = add(
                relative,
                mul(sub(trial[j], previous[v]), self.coefficients[j]),
            );
        }
        let tangent = sub(relative, mul(self.normal, dot(relative, self.normal)));
        let distance = length(tangent);
        if !distance.is_finite() {
            return Err(unstable());
        }
        if distance > 1e-14 {
            let amount = (self.friction * multiplier).min(distance / denominator);
            let direction = mul(tangent, distance.recip());
            for (j, &v) in self.vertices.iter().enumerate() {
                if inverse_mass[v] > 0. {
                    trial[j] = sub(
                        trial[j],
                        mul(direction, inverse_mass[v] * self.coefficients[j] * amount),
                    );
                }
            }
        }
        let mut cumulative = provenance.get(&key).copied().unwrap_or([[0.; 3]; N]);
        for j in 0..N {
            cumulative[j] = add(cumulative[j], sub(trial[j], before[j]));
            if !finite(trial[j]) || !finite(cumulative[j]) {
                return Err(unstable());
            }
        }
        if trial != before {
            let count = projections.checked_add(1).ok_or_else(|| {
                SimulationError::Unstable("initial contact recovery counter overflow".into())
            })?;
            provenance.insert(key, cumulative);
            *projections = count;
        }
        for (v, value) in self.vertices.into_iter().zip(trial) {
            x[v] = value;
        }
        Ok(1)
    }

    fn project(
        self,
        x: &mut [Vec3],
        previous: &[Vec3],
        inverse_mass: &[f64],
        history: &mut BTreeMap<(usize, usize), [Vec3; N]>,
        key: (usize, usize),
    ) -> Result<usize, SimulationError> {
        let mut action = history.get(&key).copied().unwrap_or([[0.; 3]; N]);
        let active = self.replace(x, previous, inverse_mass, &mut action)?;
        if action == [[0.; 3]; N] {
            history.remove(&key);
        } else {
            history.insert(key, action);
        }
        Ok(active)
    }

    fn replace(
        self,
        x: &mut [Vec3],
        previous: &[Vec3],
        inverse_mass: &[f64],
        owned_action: &mut [Vec3; N],
    ) -> Result<usize, SimulationError> {
        let unstable = || SimulationError::Unstable("unusable contact action or geometry".into());
        let old = *owned_action;
        let anchor = contact_anchor(self.vertices, x, inverse_mass, old)?;
        let mut denominator = 0.;
        if !self.gap.is_finite()
            || !finite(self.normal)
            || !self.friction.is_finite()
            || self.friction < 0.
            || !self.coefficients.into_iter().all(f64::is_finite)
        {
            return Err(unstable());
        }
        for j in 0..N {
            let v = self.vertices[j];
            let weight = inverse_mass[v];
            if !finite(previous[v]) {
                return Err(unstable());
            }
            denominator += weight * self.coefficients[j].powi(2);
        }
        if !denominator.is_finite() {
            return Err(unstable());
        }
        let mut impulse = [0.; 3];
        let mut active = 0;
        if denominator > self.minimum_denominator {
            let normal_length = length(self.normal);
            if !normal_length.is_finite() || normal_length == 0. {
                return Err(unstable());
            }
            let normal = mul(self.normal, normal_length.recip());
            let multiplier = -self.gap / denominator;
            if !multiplier.is_finite() {
                return Err(unstable());
            }
            let multiplier = multiplier.max(0.);
            impulse = mul(normal, multiplier);
            let mut relative = [0.; 3];
            for (j, &v) in self.vertices.iter().enumerate() {
                relative = add(
                    relative,
                    mul(sub(anchor[j], previous[v]), self.coefficients[j]),
                );
            }
            let tangent = sub(relative, mul(normal, dot(relative, normal)));
            let distance = length(tangent);
            if !distance.is_finite() {
                return Err(unstable());
            }
            if distance > 1e-14 {
                let amount = (self.friction * multiplier).min(distance / denominator);
                impulse = sub(impulse, mul(tangent, amount / distance));
            }
            active = usize::from(multiplier > 0.);
        }
        // Replace the old support action; a changed normal or barycentric point cannot retain it.
        let mut trial = anchor;
        let mut action = [[0.; 3]; N];
        for j in 0..N {
            let weight = inverse_mass[self.vertices[j]];
            if weight > 0. {
                trial[j] = add(anchor[j], mul(impulse, weight * self.coefficients[j]));
                action[j] = sub(trial[j], anchor[j]);
            }
            if !finite(trial[j]) || !finite(action[j]) {
                return Err(unstable());
            }
        }
        for (v, value) in self.vertices.into_iter().zip(trial) {
            x[v] = value;
        }
        *owned_action = action;
        Ok(active)
    }
}

#[derive(Default)]
pub(crate) struct ContactErrors {
    pub maximum: f64,
    pub intersections: usize,
    pub uncertain_capsule_pairs: usize,
    pub recovery_eligibility: InitialRecoveryEligibility,
}

impl Cloth {
    pub(crate) fn project_colliders(
        &self,
        x: &mut [Vec3],
        previous: &[Vec3],
        history: &mut ContactState,
        cancelled: &impl Fn() -> bool,
    ) -> Result<usize, SimulationError> {
        let mut contacts = 0;
        for i in 0..x.len() {
            if i % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(cancelled)?;
            }
            if self.inverse_masses[i] == 0. {
                continue;
            }
            for (ci, &collider) in self.input.colliders.iter().enumerate() {
                if ci % CANCELLATION_CHECK_INTERVAL == 0 {
                    check_cancel(cancelled)?;
                }
                let key = (i, ci);
                let anchor = contact_anchor(
                    [i],
                    x,
                    &self.inverse_masses,
                    history
                        .vertex_colliders
                        .get(&key)
                        .copied()
                        .unwrap_or([[0.; 3]]),
                )?;
                if let Some((normal, gap)) = collider_feature(
                    collider,
                    anchor[0],
                    self.thickness[i],
                    history.vertex_colliders.contains_key(&key),
                ) {
                    contacts += Contact {
                        vertices: [i],
                        coefficients: [1.],
                        normal,
                        gap,
                        friction: self.friction[i],
                        minimum_denominator: 0.,
                    }
                    .project(
                        x,
                        previous,
                        &self.inverse_masses,
                        &mut history.vertex_colliders,
                        key,
                    )?;
                }
            }
        }
        for (ti, &tri) in self.input.triangles.iter().enumerate() {
            if ti % CANCELLATION_CHECK_INTERVAL == 0 {
                check_cancel(cancelled)?;
            }
            let thickness = tri
                .map(|i| self.thickness[i])
                .into_iter()
                .fold(0., f64::max);
            for (ci, &collider) in self.input.colliders.iter().enumerate() {
                if ci % CANCELLATION_CHECK_INTERVAL == 0 {
                    check_cancel(cancelled)?;
                }
                let Collider::Capsule { a, b, radius } = collider else {
                    continue;
                };
                let key = (ti, ci);
                if let Some(phase) = history.recovery.as_mut()
                    && phase.eligibility.triangle_capsules.contains(&key)
                {
                    let feature = capsule::closest(tri.map(|i| x[i]), a, b);
                    let depth = radius + thickness - feature.distance;
                    if depth <= 0. {
                        continue;
                    }
                    contacts += Contact {
                        vertices: tri,
                        coefficients: feature.bary,
                        normal: capsule::normal(
                            feature,
                            tri.map(|i| x[i]),
                            tri.map(|i| previous[i]),
                            a,
                            b,
                        ),
                        gap: -depth,
                        friction: (0..3)
                            .map(|j| self.friction[tri[j]] * feature.bary[j])
                            .sum(),
                        minimum_denominator: 0.,
                    }
                    .recover(
                        x,
                        previous,
                        &self.inverse_masses,
                        &mut phase.provenance.triangle_capsules,
                        &mut phase.provenance.projections,
                        key,
                    )?;
                    continue;
                }
                if let Some(supports) = history.triangle_capsules.get_mut(&key) {
                    for support in supports.iter_mut() {
                        check_cancel(cancelled)?;
                        contacts += self.project_capsule_support(
                            tri,
                            (a, b, radius),
                            thickness,
                            support,
                            x,
                            previous,
                        )?;
                    }
                    supports.retain(|support| support.action != [[0.; 3]; 3]);
                }
                let feature = capsule::closest(tri.map(|i| x[i]), a, b);
                let depth = radius + thickness - feature.distance;
                if !depth.is_finite()
                    || !feature.axis_fraction.is_finite()
                    || !feature.bary.into_iter().all(f64::is_finite)
                {
                    return Err(SimulationError::Unstable(
                        "unusable capsule support discovery".into(),
                    ));
                }
                if depth > 0. {
                    let supports = history.triangle_capsules.entry(key).or_default();
                    let mut existing = None;
                    for (index, support) in supports.iter().enumerate() {
                        check_cancel(cancelled)?;
                        if support.bary == feature.bary {
                            existing = Some(index);
                            break;
                        }
                    }
                    check_cancel(cancelled)?;
                    if let Some(index) = existing {
                        contacts += self.project_capsule_support(
                            tri,
                            (a, b, radius),
                            thickness,
                            &mut supports[index],
                            x,
                            previous,
                        )?;
                    } else {
                        // shortcut: one discovery per pass bounds storage; revisit if manifold residuals stall.
                        let mut support = CapsuleSupport {
                            bary: feature.bary,
                            action: [[0.; 3]; 3],
                        };
                        contacts += self.project_capsule_support(
                            tri,
                            (a, b, radius),
                            thickness,
                            &mut support,
                            x,
                            previous,
                        )?;
                        supports.push(support);
                    }
                    supports.retain(|support| support.action != [[0.; 3]; 3]);
                }
                if history
                    .triangle_capsules
                    .get(&key)
                    .is_some_and(Vec::is_empty)
                {
                    history.triangle_capsules.remove(&key);
                }
            }
        }
        Ok(contacts)
    }

    fn project_capsule_support(
        &self,
        vertices: [usize; 3],
        capsule: (Vec3, Vec3, f64),
        thickness: f64,
        support: &mut CapsuleSupport,
        x: &mut [Vec3],
        previous: &[Vec3],
    ) -> Result<usize, SimulationError> {
        let unstable = || SimulationError::Unstable("unusable fixed capsule support".into());
        if !support.bary.into_iter().all(f64::is_finite) {
            return Err(unstable());
        }
        let anchor = contact_anchor(vertices, x, &self.inverse_masses, support.action)?;
        let point = bary_point(&anchor, [0, 1, 2], support.bary);
        if !finite(point) {
            return Err(unstable());
        }
        let (a, b, radius) = capsule;
        let mut feature = capsule::closest([point; 3], a, b);
        if !feature.distance.is_finite() || !feature.axis_fraction.is_finite() || feature.uncertain
        {
            return Err(unstable());
        }
        feature.bary = support.bary;
        let normal = capsule::normal(feature, anchor, vertices.map(|i| previous[i]), a, b);
        Contact {
            vertices,
            coefficients: support.bary,
            normal,
            gap: -(radius + thickness - feature.distance),
            friction: (0..3)
                .map(|j| self.friction[vertices[j]] * support.bary[j])
                .sum(),
            minimum_denominator: 0.,
        }
        .replace(x, previous, &self.inverse_masses, &mut support.action)
    }

    pub(crate) fn project_cloth(
        &self,
        x: &mut [Vec3],
        previous: &[Vec3],
        history: &mut ContactState,
        cancelled: &impl Fn() -> bool,
    ) -> Result<usize, SimulationError> {
        let max_thickness = self.thickness.iter().copied().fold(0., f64::max);
        let bounds = triangle_bounds_cancellable(
            &self.input.triangles,
            x,
            previous,
            2. * max_thickness,
            cancelled,
        )?;
        let hierarchy = TriangleHierarchy::new(&bounds, cancelled)?;
        let mut candidates = Vec::new();
        let mut contacts = 0;
        for v in 0..x.len() {
            if v % 32 == 0 && cancelled() {
                return Err(SimulationError::Cancelled);
            }
            candidates.clear();
            hierarchy.visit_swept(0, x[v], previous[v], cancelled, &mut |i| {
                if overlaps(&bounds[i], x[v], previous[v]) {
                    candidates.push(i);
                }
                Ok(())
            })?;
            for (&(_, i), _) in history.cloth.range((v, 0)..=(v, usize::MAX)) {
                check_cancel(cancelled)?;
                candidates.push(i);
            }
            candidates.sort_unstable();
            candidates.dedup();
            let (mut next, mut candidate, mut linear) = (0, 0, false);
            while next < self.input.triangles.len() {
                let i = if linear {
                    next
                } else {
                    let Some(&i) = candidates.get(candidate) else {
                        break;
                    };
                    candidate += 1;
                    i
                };
                next = i + 1;
                let tri = self.input.triangles[i];
                let key = (v, i);
                let had_action = history.cloth.contains_key(&key);
                if tri.iter().any(|i| self.excluded[v].contains(i))
                    || (!had_action && !overlaps(&bounds[i], x[v], previous[v]))
                {
                    continue;
                }
                let gap = self.thickness[v]
                    + tri
                        .map(|i| self.thickness[i])
                        .into_iter()
                        .fold(0., f64::max);
                let vertices = [v, tri[0], tri[1], tri[2]];
                if let Some(phase) = history.recovery.as_mut()
                    && phase.eligibility.cloth.contains(&key)
                {
                    let Some((normal, bary, depth)) = cloth_contact(v, tri, x, previous, gap, true)
                    else {
                        continue;
                    };
                    let before = x[v];
                    contacts += Contact {
                        vertices,
                        coefficients: [1., -bary[0], -bary[1], -bary[2]],
                        normal,
                        gap: -depth,
                        friction: (self.friction[v]
                            + tri.map(|i| self.friction[i]).into_iter().sum::<f64>() / 3.)
                            * 0.5,
                        minimum_denominator: 1e-20,
                    }
                    .recover(
                        x,
                        previous,
                        &self.inverse_masses,
                        &mut phase.provenance.cloth,
                        &mut phase.provenance.projections,
                        key,
                    )?;
                    linear |= x[v] != before;
                    continue;
                }
                let anchor = contact_anchor(
                    vertices,
                    x,
                    &self.inverse_masses,
                    history.cloth.get(&key).copied().unwrap_or([[0.; 3]; 4]),
                )?;
                // shortcut: one feature can stall on a moving triangle; use a converged nonlinear block for stronger clearance guarantees.
                let Some((normal, bary, gap)) =
                    cloth_feature(anchor, vertices.map(|i| previous[i]), gap, true, had_action)
                else {
                    if had_action {
                        return Err(SimulationError::Unstable(
                            "active cloth contact lost its geometric feature".into(),
                        ));
                    }
                    continue;
                };
                let before = x[v];
                contacts += Contact {
                    vertices,
                    coefficients: [1., -bary[0], -bary[1], -bary[2]],
                    normal,
                    gap,
                    friction: (self.friction[v]
                        + tri.map(|i| self.friction[i]).into_iter().sum::<f64>() / 3.)
                        * 0.5,
                    minimum_denominator: 1e-20,
                }
                .project(
                    x,
                    previous,
                    &self.inverse_masses,
                    &mut history.cloth,
                    key,
                )?;
                // shortcut: moving queries finish the old scan; revisit if dense contact dominates.
                // Later bounds may become reachable, and projection order must remain unchanged.
                linear |= x[v] != before;
            }
        }
        Ok(contacts)
    }

    #[cfg(test)]
    pub(crate) fn contact_errors(&self, x: &[Vec3]) -> ContactErrors {
        self.analyze_contacts(x, &|| false, true, false)
            .expect("A non-cancelling callback cannot interrupt contact analysis")
    }

    #[cfg(test)]
    pub(crate) fn exhaustive_contact_errors(&self, x: &[Vec3]) -> ContactErrors {
        self.analyze_contacts(x, &|| false, false, false).unwrap()
    }

    pub(crate) fn contact_errors_cancellable(
        &self,
        x: &[Vec3],
        cancelled: &impl Fn() -> bool,
    ) -> Result<ContactErrors, SimulationError> {
        self.analyze_contacts(x, cancelled, true, false)
    }

    pub(crate) fn initial_contact_errors_cancellable(
        &self,
        x: &[Vec3],
        cancelled: &impl Fn() -> bool,
    ) -> Result<ContactErrors, SimulationError> {
        self.analyze_contacts(x, cancelled, true, true)
    }

    fn analyze_contacts(
        &self,
        x: &[Vec3],
        cancelled: &impl Fn() -> bool,
        indexed: bool,
        collect_recovery: bool,
    ) -> Result<ContactErrors, SimulationError> {
        let mut max_thickness = 0_f64;
        for &thickness in &self.thickness {
            check_cancel(cancelled)?;
            max_thickness = max_thickness.max(thickness);
        }
        let bounds = triangle_bounds_cancellable(
            &self.input.triangles,
            x,
            x,
            2. * max_thickness,
            cancelled,
        )?;
        let hierarchy = if indexed {
            Some(TriangleHierarchy::new(&bounds, cancelled)?)
        } else {
            None
        };
        let mut maximum = 0_f64;
        let mut count = 0;
        let mut recovery_eligibility = InitialRecoveryEligibility::default();
        for v in 0..x.len() {
            check_cancel(cancelled)?;
            for &collider in &self.input.colliders {
                check_cancel(cancelled)?;
                if let Some((_, depth)) = collider_contact(collider, x[v], self.thickness[v]) {
                    maximum = maximum.max(depth);
                    if depth > 1e-8 {
                        count += 1;
                    }
                }
            }
            let mut consider = |i: usize| {
                let tri = self.input.triangles[i];
                if tri.iter().any(|i| self.excluded[v].contains(i))
                    || !overlaps(&bounds[i], x[v], x[v])
                {
                    return Ok(());
                }
                check_cancel(cancelled)?;
                let gap = self.thickness[v]
                    + tri
                        .map(|i| self.thickness[i])
                        .into_iter()
                        .fold(0., f64::max);
                if let Some((_, _, depth)) = cloth_contact(v, tri, x, x, gap, false) {
                    maximum = maximum.max(depth);
                    if depth > 1e-8 {
                        count += 1;
                        if collect_recovery {
                            recovery_eligibility.cloth.insert((v, i));
                        }
                    }
                }
                Ok(())
            };
            if let Some(hierarchy) = &hierarchy {
                hierarchy.visit(0, x[v], cancelled, &mut consider)?;
            } else {
                for i in 0..self.input.triangles.len() {
                    check_cancel(cancelled)?;
                    consider(i)?;
                }
            }
        }
        let mut uncertain_capsule_pairs = 0;
        for (ti, &tri) in self.input.triangles.iter().enumerate() {
            check_cancel(cancelled)?;
            let thickness = tri
                .map(|i| self.thickness[i])
                .into_iter()
                .fold(0., f64::max);
            for (ci, &collider) in self.input.colliders.iter().enumerate() {
                check_cancel(cancelled)?;
                let Collider::Capsule { a, b, radius } = collider else {
                    continue;
                };
                let feature = capsule::closest_cancellable(tri.map(|i| x[i]), a, b, cancelled)?;
                uncertain_capsule_pairs += usize::from(feature.uncertain);
                let depth = radius + thickness - feature.distance;
                maximum = maximum.max(depth);
                if depth > 1e-8 {
                    count += 1;
                    if collect_recovery {
                        recovery_eligibility.triangle_capsules.insert((ti, ci));
                    }
                }
            }
        }
        Ok(ContactErrors {
            maximum,
            intersections: count,
            uncertain_capsule_pairs,
            recovery_eligibility,
        })
    }
}

type Bounds = [Vec3; 2];

struct TriangleHierarchy {
    indices: Vec<usize>,
    nodes: Vec<TriangleNode>,
}

struct TriangleNode {
    bounds: Bounds,
    range: std::ops::Range<usize>,
    children: Option<[usize; 2]>,
}

impl TriangleHierarchy {
    fn new(bounds: &[Bounds], cancelled: &impl Fn() -> bool) -> Result<Self, SimulationError> {
        let indices = (0..bounds.len())
            .map(|i| {
                check_cancel(cancelled)?;
                Ok(i)
            })
            .collect::<Result<_, SimulationError>>()?;
        let mut hierarchy = Self {
            indices,
            nodes: Vec::new(),
        };
        hierarchy.build(0..bounds.len(), bounds, cancelled)?;
        Ok(hierarchy)
    }

    fn build(
        &mut self,
        range: std::ops::Range<usize>,
        bounds: &[Bounds],
        cancelled: &impl Fn() -> bool,
    ) -> Result<usize, SimulationError> {
        check_cancel(cancelled)?;
        let mut combined = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
        for &triangle in &self.indices[range.clone()] {
            check_cancel(cancelled)?;
            for axis in 0..3 {
                combined[0][axis] = combined[0][axis].min(bounds[triangle][0][axis]);
                combined[1][axis] = combined[1][axis].max(bounds[triangle][1][axis]);
            }
        }
        let node = self.nodes.len();
        self.nodes.push(TriangleNode {
            bounds: combined,
            range: range.clone(),
            children: None,
        });
        if range.len() > 8 {
            let axis = (0..3)
                .max_by(|&a, &b| {
                    (combined[1][a] - combined[0][a]).total_cmp(&(combined[1][b] - combined[0][b]))
                })
                .unwrap();
            partition_triangles(&mut self.indices[range.clone()], bounds, axis, cancelled)?;
            let middle = range.start + range.len() / 2;
            let left = self.build(range.start..middle, bounds, cancelled)?;
            let right = self.build(middle..range.end, bounds, cancelled)?;
            self.nodes[node].children = Some([left, right]);
        }
        Ok(node)
    }

    fn visit(
        &self,
        node: usize,
        point: Vec3,
        cancelled: &impl Fn() -> bool,
        consider: &mut impl FnMut(usize) -> Result<(), SimulationError>,
    ) -> Result<(), SimulationError> {
        self.visit_swept(node, point, point, cancelled, consider)
    }

    fn visit_swept(
        &self,
        node: usize,
        point: Vec3,
        previous: Vec3,
        cancelled: &impl Fn() -> bool,
        consider: &mut impl FnMut(usize) -> Result<(), SimulationError>,
    ) -> Result<(), SimulationError> {
        check_cancel(cancelled)?;
        let node = &self.nodes[node];
        if overlaps(&node.bounds, point, previous) {
            if let Some(children) = node.children {
                for child in children {
                    self.visit_swept(child, point, previous, cancelled, consider)?;
                }
            } else {
                for &triangle in &self.indices[node.range.clone()] {
                    check_cancel(cancelled)?;
                    consider(triangle)?;
                }
            }
        }
        Ok(())
    }
}

fn partition_triangles(
    indices: &mut [usize],
    bounds: &[Bounds],
    axis: usize,
    cancelled: &impl Fn() -> bool,
) -> Result<(), SimulationError> {
    let middle = indices.len() / 2;
    let (mut start, mut end) = (0, indices.len());
    let mut work = 0;
    // shortcut: capped selection can yield coarse splits, upgrade if measured pruning regresses.
    while end - start > 1 && work < indices.len().saturating_mul(4) {
        check_cancel(cancelled)?;
        let pivot = bounds[indices[start + (end - start) / 2]][0][axis];
        let (mut lower, mut i, mut upper) = (start, start, end);
        while i < upper {
            check_cancel(cancelled)?;
            work += 1;
            match bounds[indices[i]][0][axis].total_cmp(&pivot) {
                std::cmp::Ordering::Less => {
                    indices.swap(i, lower);
                    lower += 1;
                    i += 1;
                }
                std::cmp::Ordering::Equal => i += 1,
                std::cmp::Ordering::Greater => {
                    upper -= 1;
                    indices.swap(i, upper);
                }
            }
        }
        if middle < lower {
            end = lower;
        } else if middle >= upper {
            start = upper;
        } else {
            break;
        }
    }
    Ok(())
}

fn triangle_bounds_cancellable(
    triangles: &[[usize; 3]],
    x: &[Vec3],
    previous: &[Vec3],
    gap: f64,
    cancelled: &impl Fn() -> bool,
) -> Result<Vec<Bounds>, SimulationError> {
    triangles
        .iter()
        .map(|tri| {
            check_cancel(cancelled)?;
            let mut bounds = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
            for &v in tri {
                for axis in 0..3 {
                    bounds[0][axis] = bounds[0][axis].min(x[v][axis].min(previous[v][axis]) - gap);
                    bounds[1][axis] = bounds[1][axis].max(x[v][axis].max(previous[v][axis]) + gap);
                }
            }
            Ok(bounds)
        })
        .collect()
}
fn overlaps(bounds: &Bounds, a: Vec3, b: Vec3) -> bool {
    (0..3).all(|axis| {
        a[axis].max(b[axis]) >= bounds[0][axis] && a[axis].min(b[axis]) <= bounds[1][axis]
    })
}

fn cloth_contact(
    v: usize,
    tri: [usize; 3],
    x: &[Vec3],
    previous: &[Vec3],
    gap: f64,
    swept: bool,
) -> Option<(Vec3, Vec3, f64)> {
    let vertices = [v, tri[0], tri[1], tri[2]];
    cloth_feature(
        vertices.map(|i| x[i]),
        vertices.map(|i| previous[i]),
        gap,
        swept,
        false,
    )
    .map(|(normal, bary, gap)| (normal, bary, -gap))
}

fn cloth_feature(
    x: [Vec3; 4],
    previous: [Vec3; 4],
    gap: f64,
    swept: bool,
    include_separated: bool,
) -> Option<(Vec3, Vec3, f64)> {
    let [vertex, a, b, c] = x;
    let (point, bary) = closest_triangle(vertex, a, b, c);
    let delta = sub(vertex, point);
    let distance = length(delta);
    let face_normal = unit(cross(sub(b, a), sub(c, a)));
    if length(face_normal) < 0.5 {
        return None;
    }
    let old_normal = unit(cross(
        sub(previous[2], previous[1]),
        sub(previous[3], previous[1]),
    ));
    let old_side = dot(
        sub(previous[0], bary_point(&previous, [1, 2, 3], bary)),
        old_normal,
    );
    let side = dot(delta, face_normal);
    // Linearized swept face crossing also catches particles whose endpoint has passed beyond thickness.
    // shortcut: this is not full continuous deforming-triangle or edge-edge collision, upgrade before thin/high-speed production scenes.
    let crossed = swept && old_side * side < -1e-18 && (distance - side.abs()).abs() < 1e-8;
    if crossed {
        let normal = mul(face_normal, if old_side >= 0. { 1. } else { -1. });
        return Some((normal, bary, -(gap - dot(delta, normal))));
    }
    if distance >= gap && !include_separated {
        return None;
    }
    let normal = if distance > 1e-12 {
        mul(delta, distance.recip())
    } else {
        mul(face_normal, if old_side < 0. { -1. } else { 1. })
    };
    Some((normal, bary, -(gap - distance)))
}

pub(crate) fn collider_contact(collider: Collider, p: Vec3, thickness: f64) -> Option<(Vec3, f64)> {
    collider_feature(collider, p, thickness, false).map(|(normal, gap)| (normal, -gap))
}

fn collider_feature(
    collider: Collider,
    p: Vec3,
    thickness: f64,
    include_separated: bool,
) -> Option<(Vec3, f64)> {
    match collider {
        Collider::Ground { height } => {
            let depth = height + thickness - p[1];
            (depth > 0. || include_separated).then_some(([0., 1., 0.], -depth))
        }
        Collider::Capsule { a, b, radius } => {
            let edge = sub(b, a);
            let edge_squared = dot(edge, edge);
            let t = if edge_squared > 1e-20 {
                (dot(sub(p, a), edge) / edge_squared).clamp(0., 1.)
            } else {
                0.
            };
            let delta = sub(p, add(a, mul(edge, t)));
            let distance = length(delta);
            let depth = radius + thickness - distance;
            if depth <= 0. && !include_separated {
                return None;
            }
            let normal = if distance > 1e-12 {
                mul(delta, distance.recip())
            } else {
                let reference = if edge[0].abs() < edge[1].abs() {
                    [1., 0., 0.]
                } else {
                    [0., 1., 0.]
                };
                let normal = unit(cross(edge, reference));
                if length(normal) > 0.5 {
                    normal
                } else {
                    [1., 0., 0.]
                }
            };
            Some((normal, -depth))
        }
        Collider::Torso { center, radii } => {
            let d = sub(p, center);
            let r = radii.map(|r| r + thickness);
            let q = (0..3).map(|i| (d[i] / r[i]).powi(2)).sum::<f64>().sqrt();
            if q >= 1. && !include_separated {
                return None;
            }
            // shortcut: radial inflated-ellipsoid contact approximates a constant offset, replace with exact signed distance for calibrated avatars.
            let target = if q > 1e-12 {
                mul(d, q.recip())
            } else {
                [r[0], 0., 0.]
            };
            let correction = sub(target, d);
            let depth = length(correction);
            let direction = if q >= 1. { d } else { correction };
            let norm = direction[0].hypot(direction[1]).hypot(direction[2]);
            let normal = if norm > 0. {
                direction.map(|component| component / norm)
            } else {
                [0.; 3]
            };
            Some((normal, if q >= 1. { depth } else { -depth }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClothInput, Material, Pin};
    use std::cell::Cell;

    fn planes_input(upper_y: f64) -> ClothInput {
        let positions = vec![
            [0.25, 0.004, 0.25],
            [0.25, 2., 0.25],
            [0.25, 0.004, 2.],
            [0., 0., 0.],
            [0., 0., 1.],
            [1., 0., 0.],
            [0., upper_y, 0.],
            [0., upper_y, 1.],
            [1., upper_y, 0.],
        ];
        ClothInput {
            pins: (1..positions.len())
                .map(|vertex| Pin {
                    vertex,
                    target: positions[vertex],
                })
                .collect(),
            positions: positions.clone(),
            triangles: vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]],
            material_uv: [[0., 0.], [1., 0.], [0., 1.]].repeat(3),
            triangle_materials: vec![
                Material {
                    friction: 0.,
                    collision_thickness: 0.003,
                    ..Default::default()
                };
                3
            ],
            stitches: vec![],
            colliders: vec![],
        }
    }

    #[test]
    fn initial_recovery_eligibility_is_pair_specific_and_frozen_before_motion() {
        let input = planes_input(1.);
        let positions = input.positions.clone();
        let cloth = Cloth::new(input).unwrap();
        assert_eq!(cloth.diagnostics().initial_intersections, 1);
        assert!(cloth.initial_recovery_eligibility.cloth.contains(&(0, 1)));
        assert!(!cloth.initial_recovery_eligibility.cloth.contains(&(0, 2)));
        let mut balanced = ContactState::default();
        let mut x = positions.clone();
        cloth
            .project_cloth(&mut x, &positions, &mut balanced, &|| false)
            .unwrap();
        assert!(balanced.cloth.contains_key(&(0, 1)));
        assert!(balanced.recovery.is_none());

        let mut phase = ContactState::with_initial_recovery(
            &cloth.initial_recovery_eligibility,
            RecoveryProvenance::default(),
        );
        let mut x = positions.clone();
        x[0][1] = 0.999;
        cloth
            .project_cloth(&mut x, &positions, &mut phase, &|| false)
            .unwrap();
        assert!(
            phase.cloth.contains_key(&(0, 2)),
            "new collision remains balanced"
        );
        assert_eq!(phase.recovery.as_ref().unwrap().provenance.projections, 0);
        assert!(phase.recovery.as_ref().unwrap().provenance.cloth.is_empty());

        let mut phase = ContactState::with_initial_recovery(
            &cloth.initial_recovery_eligibility,
            RecoveryProvenance::default(),
        );
        let mut x = positions.clone();
        cloth
            .project_cloth(&mut x, &positions, &mut phase, &|| false)
            .unwrap();
        let recovery = phase.recovery.as_ref().unwrap();
        assert_eq!(recovery.provenance.projections, 1);
        assert!(length(sub(recovery.provenance.cloth[&(0, 1)][0], [0., 0.002, 0.])) <= 1e-14);
        assert!(phase.cloth.is_empty());

        let mut tiny = planes_input(1.);
        tiny.positions[0][1] = 0.006 - 2_f64.powi(-30);
        let tiny = Cloth::new(tiny).unwrap();
        assert!(tiny.diagnostics().max_contact_error > 0.);
        assert_eq!(tiny.diagnostics().initial_intersections, 0);
        assert!(tiny.initial_recovery_eligibility.cloth.is_empty());

        let mut clear = planes_input(1.);
        clear.positions[0][1] = 1.1;
        let clear = Cloth::new(clear).unwrap();
        assert_eq!(clear.diagnostics().initial_intersections, 0);
        let mut phase = ContactState::with_initial_recovery(
            &clear.initial_recovery_eligibility,
            RecoveryProvenance::default(),
        );
        let mut x = clear.positions.clone();
        x[0][1] = 0.9;
        clear
            .project_cloth(&mut x, &clear.positions, &mut phase, &|| false)
            .unwrap();
        assert!(phase.recovery.is_none());
        assert!(phase.cloth.contains_key(&(0, 2)));
        assert!((x[0][1] - 1.006).abs() <= 1e-14);
        assert_eq!(clear.diagnostics().initial_contact_recovery_projections, 0);
    }

    #[test]
    fn initial_capsule_recovery_owns_only_frozen_pairs_and_keeps_displacement_provenance() {
        let positions = vec![[-1., 0., 0.125], [1., 0., 0.125], [0., 2., 0.125]];
        let input = ClothInput {
            positions: positions.clone(),
            material_uv: positions.iter().map(|p| [p[0], p[1]]).collect(),
            triangles: vec![[0, 1, 2]],
            triangle_materials: vec![Material {
                friction: 0.,
                collision_thickness: 0.0625,
                ..Default::default()
            }],
            colliders: [0., 1.]
                .map(|z| Collider::Capsule {
                    a: [0., -1., z],
                    b: [0., 1., z],
                    radius: 0.1875,
                })
                .to_vec(),
            ..Default::default()
        };
        let mut cloth = Cloth::new(input).unwrap();
        cloth.inverse_masses = vec![1., 3., 1.];
        assert_eq!(cloth.diagnostics().initial_intersections, 1);
        assert_eq!(
            cloth.initial_recovery_eligibility.triangle_capsules,
            BTreeSet::from([(0, 0)])
        );

        let mut phase = ContactState::with_initial_recovery(
            &cloth.initial_recovery_eligibility,
            RecoveryProvenance::default(),
        );
        let mut x: Vec<_> = positions.iter().map(|&p| add(p, [0., 0., 1.])).collect();
        cloth
            .project_colliders(&mut x, &positions, &mut phase, &|| false)
            .unwrap();
        assert!(phase.triangle_capsules.contains_key(&(0, 1)));
        assert_eq!(phase.recovery.as_ref().unwrap().provenance.projections, 0);

        let mut phase = ContactState::with_initial_recovery(
            &cloth.initial_recovery_eligibility,
            RecoveryProvenance::default(),
        );
        let mut x = positions.clone();
        cloth
            .project_colliders(&mut x, &positions, &mut phase, &|| false)
            .unwrap();
        assert!(phase.triangle_capsules.is_empty());
        let recovery = &phase.recovery.as_ref().unwrap().provenance;
        assert_eq!(recovery.projections, 1);
        for (j, expected_z) in [0.1875, 0.3125, 0.125].into_iter().enumerate() {
            assert_eq!(x[j][2], expected_z);
            assert_eq!(
                recovery.triangle_capsules[&(0, 0)][j],
                sub(x[j], positions[j])
            );
        }
        let retained = format!("{recovery:?}");
        let translated: Vec<_> = x.iter().map(|&p| add(p, [0., 0., 2.])).collect();
        x.clone_from(&translated);
        cloth
            .project_colliders(&mut x, &positions, &mut phase, &|| false)
            .unwrap();
        assert_eq!(x, translated);
        assert_eq!(
            format!("{:?}", phase.recovery.as_ref().unwrap().provenance),
            retained
        );
        assert!(phase.triangle_capsules.is_empty());
        assert!(phase.vertex_colliders.is_empty());
    }

    #[test]
    fn recovery_provenance_accumulates_actual_normal_and_friction_motion_atomically() {
        let initial = [[0.; 3], [0.; 3], [2., -0., 3.]];
        let previous = initial.map(|p| sub(p, [0.125, 0., 0.]));
        let weights = [1., 3., 0.];
        let contact = |normal, gap, friction| Contact {
            vertices: [0, 1, 2],
            coefficients: [0.5, 0.5, 0.],
            normal,
            gap,
            friction,
            minimum_denominator: 0.,
        };
        let mut x = initial;
        let mut provenance = BTreeMap::new();
        let mut projections = 0;
        contact([0., 0., 1.], -0.125, 0.5)
            .recover(
                &mut x,
                &previous,
                &weights,
                &mut provenance,
                &mut projections,
                (0, 0),
            )
            .unwrap();
        assert_eq!(projections, 1);
        assert_eq!(
            provenance[&(0, 0)],
            [[-0.03125, 0., 0.0625], [-0.09375, 0., 0.1875], [0.; 3]]
        );
        contact([0., 1., 0.], -0.25, 0.)
            .recover(
                &mut x,
                &previous,
                &weights,
                &mut provenance,
                &mut projections,
                (0, 0),
            )
            .unwrap();
        assert_eq!(projections, 2);
        for j in 0..3 {
            assert_eq!(provenance[&(0, 0)][j], sub(x[j], initial[j]));
        }
        assert_eq!(x[2].map(f64::to_bits), initial[2].map(f64::to_bits));
        let before = x;
        let retained = provenance.clone();
        contact([0., 1., 0.], 1., 0.)
            .recover(
                &mut x,
                &previous,
                &weights,
                &mut provenance,
                &mut projections,
                (0, 0),
            )
            .unwrap();
        assert_eq!(x, before);
        assert_eq!(provenance, retained);
        assert_eq!(projections, 2);

        let mut projections = usize::MAX;
        assert!(matches!(
            contact([0., 1., 0.], -0.25, 0.).recover(
                &mut x,
                &previous,
                &weights,
                &mut provenance,
                &mut projections,
                (0, 0)
            ),
            Err(SimulationError::Unstable(_))
        ));
        assert_eq!(x, before);
        assert_eq!(provenance, retained);
        assert_eq!(projections, usize::MAX);
        contact([0., 1., 0.], -f64::MIN_POSITIVE, 0.)
            .recover(
                &mut x,
                &previous,
                &weights,
                &mut provenance,
                &mut projections,
                (0, 0),
            )
            .unwrap();
        assert_eq!(x, before);
        assert_eq!(provenance, retained);
        assert_eq!(projections, usize::MAX);
        assert_eq!(
            contact([0., 1., 0.], -0.25, 0.)
                .recover(
                    &mut x,
                    &previous,
                    &[0.; 3],
                    &mut provenance,
                    &mut projections,
                    (0, 1)
                )
                .unwrap(),
            0
        );
        assert_eq!(x[2].map(f64::to_bits), initial[2].map(f64::to_bits));
        assert_eq!(provenance, retained);
        assert_eq!(projections, usize::MAX);
    }

    #[test]
    fn contact_action_replaces_inactive_and_rotating_normal_history() {
        for (current_gap, anchor_gap, old, expected) in [
            (-0.25, -0.25, [0.; 3], [0., 0.25, 0.]),
            (0.25, 0.25, [0.; 3], [0.; 3]),
            (0.75, 0.25, [0., 0.5, 0.], [0.; 3]),
            (0.25, -0.25, [0., 0.5, 0.], [0., 0.25, 0.]),
            (-0.125, -0.125, [0.25, 0., 0.], [0., 0.125, 0.]),
        ] {
            let mut x = [[0., current_gap, 0.]];
            let previous = x;
            let anchor = sub(x[0], old);
            let mut history = BTreeMap::from([((0, 0), [old])]);
            Contact {
                vertices: [0],
                coefficients: [1.],
                normal: [0., 1., 0.],
                gap: anchor_gap,
                friction: 0.,
                minimum_denominator: 0.,
            }
            .project(&mut x, &previous, &[2.], &mut history, (0, 0))
            .unwrap();
            assert!(length(sub(x[0], add(anchor, expected))) <= 1e-14);
            assert_eq!(
                history.get(&(0, 0)).copied().unwrap_or([[0.; 3]]),
                [expected]
            );
            assert_eq!(history.is_empty(), expected == [0.; 3]);
        }
        let mut x = [[1., 2., 3.]];
        let before = x;
        let mut history = BTreeMap::from([((0, 0), [[0.25, 0., 0.]])]);
        let old = history.clone();
        assert!(matches!(
            Contact {
                vertices: [0],
                coefficients: [1.],
                normal: [f64::NAN, 1., 0.],
                gap: -0.25,
                friction: 0.,
                minimum_denominator: 0.,
            }
            .project(&mut x, &before, &[2.], &mut history, (0, 0)),
            Err(SimulationError::Unstable(_))
        ));
        assert_eq!(x, before);
        assert_eq!(history, old);
    }

    #[test]
    fn contact_action_replaces_barycentric_support_with_nonzero_pinned_weight() {
        let mut x = [[0., 0.1, 0.], [0., 0.3, 0.], [-0., 0., -0.]];
        let previous = x;
        let pin_bits = x[2].map(f64::to_bits);
        let old = [[0., 0.1, 0.], [0., 0.3, 0.], [0.; 3]];
        let anchor = std::array::from_fn::<_, 3, _>(|j| sub(x[j], old[j]));
        let mut history = BTreeMap::from([((0, 0), old)]);
        Contact {
            vertices: [0, 1, 2],
            coefficients: [0.25, 0.5, 0.25],
            normal: [0., 1., 0.],
            gap: -0.075,
            friction: 0.,
            minimum_denominator: 0.,
        }
        .project(&mut x, &previous, &[1., 3., 0.], &mut history, (0, 0))
        .unwrap();
        // D=13/16 kg^-1 and lambda=6/65 kg m, including the immovable support.
        for (j, expected) in [3. / 130., 9. / 65., 0.].into_iter().enumerate() {
            assert!((history[&(0, 0)][j][1] - expected).abs() <= 1e-14);
            assert!((x[j][1] - anchor[j][1] - expected).abs() <= 1e-14);
        }
        assert_eq!(x[2].map(f64::to_bits), pin_bits);
        assert_eq!(
            Contact {
                vertices: [0, 1, 2],
                coefficients: [0., 0., 1.],
                normal: [0., 1., 0.],
                gap: -0.1,
                friction: 0.5,
                minimum_denominator: 0.,
            }
            .project(&mut x, &previous, &[1., 3., 0.], &mut history, (0, 0))
            .unwrap(),
            0
        );
        assert!(history.is_empty());
        assert_eq!([x[0], x[1]], [anchor[0], anchor[1]]);
        assert_eq!(x[2].map(f64::to_bits), pin_bits);
        let all_pin_bits = x.map(|point| point.map(f64::to_bits));
        Contact {
            vertices: [0, 1, 2],
            coefficients: [0.25, 0.5, 0.25],
            normal: [0., 1., 0.],
            gap: -0.1,
            friction: 0.5,
            minimum_denominator: 0.,
        }
        .project(&mut x, &previous, &[0.; 3], &mut history, (0, 0))
        .unwrap();
        assert_eq!(x.map(|point| point.map(f64::to_bits)), all_pin_bits);
        assert!(history.is_empty());
    }

    #[test]
    fn contact_friction_sticks_slides_and_releases_with_normal_action() {
        for (tangent, expected) in [(0.1, -0.1), (0.4, -0.125)] {
            let mut x = [[tangent, -0.25, 0.]];
            let previous = [[0., -0.25, 0.]];
            let mut history = BTreeMap::new();
            Contact {
                vertices: [0],
                coefficients: [1.],
                normal: [0., 1., 0.],
                gap: -0.25,
                friction: 0.5,
                minimum_denominator: 0.,
            }
            .project(&mut x, &previous, &[2.], &mut history, (0, 0))
            .unwrap();
            assert!(length(sub(history[&(0, 0)][0], [expected, 0.25, 0.])) <= 1e-14);
            assert!((x[0][0] - tangent - expected).abs() <= 1e-14);
        }
        let mut x = [[-0.1, 0.75, 0.]];
        let mut history = BTreeMap::from([((0, 0), [[-0.1, 0.5, 0.]])]);
        Contact {
            vertices: [0],
            coefficients: [1.],
            normal: [0., 1., 0.],
            gap: 0.25,
            friction: 0.5,
            minimum_denominator: 0.,
        }
        .project(&mut x, &[[0.; 3]], &[2.], &mut history, (0, 0))
        .unwrap();
        assert_eq!(x, [[0., 0.25, 0.]]);
        assert!(history.is_empty());
    }

    #[test]
    fn shallow_torso_contact_keeps_unit_normals_across_scales_and_at_boundary() {
        for radius in [2_f64.powi(-20), 1., 2_f64.powi(20)] {
            let thickness = radius / 64.;
            let collider = Collider::Torso {
                center: [0.; 3],
                radii: [radius - thickness; 3],
            };
            let inside = radius * (1. - 2_f64.powi(-48));
            let positions = vec![
                [inside, 0., 0.],
                [radius + 1., 0., 0.],
                [radius + 1., 1., 0.],
            ];
            let cloth = Cloth::new(ClothInput {
                positions: positions.clone(),
                material_uv: vec![[0., 0.], [1., 0.], [0., 1.]],
                triangles: vec![[0, 1, 2]],
                triangle_materials: vec![Material {
                    friction: 0.,
                    collision_thickness: thickness,
                    ..Default::default()
                }],
                colliders: vec![collider],
                ..Default::default()
            })
            .unwrap();
            let (normal, depth) = collider_contact(collider, positions[0], thickness).unwrap();
            assert_eq!(normal, [1., 0., 0.]);
            assert!(depth > 0.);
            let mut x = positions.clone();
            let mut history = ContactState::default();
            for _ in 0..2 {
                assert_eq!(
                    cloth
                        .project_colliders(&mut x, &positions, &mut history, &|| false)
                        .unwrap(),
                    1
                );
                assert_eq!(x[0], [radius, 0., 0.]);
                assert!(collider_contact(collider, x[0], thickness).is_none());
            }
        }
    }

    #[test]
    fn capsule_supports_resolve_distinct_material_points_and_release_after_translation() {
        let positions = vec![[-1., 0., 0.125], [1., 0., 0.125], [0., 2., 0.125]];
        let (a, b) = ([0., -1., 0.], [0., 1., 0.]);
        let mut cloth = Cloth::new(ClothInput {
            positions: positions.clone(),
            material_uv: positions.iter().map(|p| [p[0], p[1]]).collect(),
            triangles: vec![[0, 1, 2]],
            triangle_materials: vec![Material {
                friction: 0.,
                collision_thickness: 0.0625,
                ..Default::default()
            }],
            colliders: vec![Collider::Capsule {
                a,
                b,
                radius: 0.1875,
            }],
            ..Default::default()
        })
        .unwrap();
        cloth.inverse_masses = vec![1., 3., 1.];
        let mut x = positions.clone();
        let mut repeat = positions.clone();
        let mut history = ContactState::default();
        let mut repeat_history = ContactState::default();
        for visit in 0..30 {
            cloth
                .project_colliders(&mut x, &positions, &mut history, &|| false)
                .unwrap();
            cloth
                .project_colliders(&mut repeat, &positions, &mut repeat_history, &|| false)
                .unwrap();
            for (&point, &repeated) in x.iter().zip(&repeat) {
                assert_eq!(point.map(f64::to_bits), repeated.map(f64::to_bits));
            }
            assert!(x.iter().copied().all(finite));
            if visit == 0 {
                for (p, expected_z) in x.iter().zip([0.1875, 0.3125, 0.125]) {
                    assert!((p[2] - expected_z).abs() <= 1e-14);
                }
                assert_eq!(history.triangle_capsules[&(0, 0)].len(), 1);
                assert_eq!(history.triangle_capsules[&(0, 0)][0].bary, [0.5, 0.5, 0.]);
                let missed = bary_point(&x, [0, 1, 2], [0.25, 0.25, 0.5]);
                assert_eq!(missed, [0., 1., 0.1875]);
                assert_eq!(0.25 - missed[2], 0.0625);
            }
            if visit == 1 {
                assert!(history.triangle_capsules[&(0, 0)].len() > 1);
                assert_ne!(x[2], positions[2]);
            }
        }
        // Full-triangle clearance catches material points missed by the first edge projection.
        let full = capsule::closest([x[0], x[1], x[2]], a, b);
        assert!(full.distance >= 0.25 - 1e-6);
        assert!(cloth.contact_errors(&x).maximum <= 1e-6);
        let sample = bary_point(&x, [0, 1, 2], [0.25, 0.25, 0.5]);
        let axis_y = sample[1].clamp(-1., 1.);
        assert!(sample[0].hypot(sample[1] - axis_y).hypot(sample[2]) >= 0.25 - 1e-6);

        for point in &mut x {
            point[2] += 1.;
        }
        cloth
            .project_colliders(&mut x, &positions, &mut history, &|| false)
            .unwrap();
        assert!(history.triangle_capsules.is_empty());
        assert!(history.vertex_colliders.is_empty());
        for (actual, original) in x.iter().zip(&positions) {
            assert!(length(sub(*actual, add(*original, [0., 0., 1.]))) <= 1e-14);
        }
    }

    #[test]
    fn cloth_contact_queries_retracted_point_before_releasing_a_finite_face() {
        let mut input = planes_input(10.);
        for point in &mut input.positions {
            point[1] += 1.;
        }
        for material in &mut input.triangle_materials {
            material.collision_thickness = 0.01;
        }
        for pin in &mut input.pins {
            pin.target = input.positions[pin.vertex];
        }
        let positions = input.positions.clone();
        let cloth = Cloth::new(input).unwrap();
        let mut previous = positions.clone();
        previous[0] = [1., 1.1, 0.];
        let mut x = positions.clone();
        x[0] = [1.125, 0.75, -0.125];
        let old = [[0.375, 0.25, -0.25], [0.; 3], [0.; 3], [0.; 3]];
        let vertices = [0, 3, 4, 5];
        let (_, _, current_gap) = cloth_feature(
            vertices.map(|i| x[i]),
            vertices.map(|i| previous[i]),
            0.02,
            true,
            true,
        )
        .unwrap();
        assert!(current_gap > 0., "current point is outside the finite face");
        assert_eq!(sub(x[0], old[0]), [0.75, 0.5, 0.125]);
        let mut history = ContactState::default();
        history.cloth.insert((0, 1), old);
        assert_eq!(
            cloth
                .project_cloth(&mut x, &previous, &mut history, &|| false)
                .unwrap(),
            1
        );
        assert!(length(sub(x[0], [0.75, 1.02, 0.125])) <= 1e-14);
        assert!(length(sub(history.cloth[&(0, 1)][0], [0., 0.52, 0.])) <= 1e-14);
        for i in 1..x.len() {
            assert_eq!(x[i].map(f64::to_bits), positions[i].map(f64::to_bits));
        }
    }

    #[test]
    fn old_cloth_actions_release_outside_bounds_and_reject_lost_features() {
        let input = planes_input(0.0116);
        let positions = input.positions.clone();
        let cloth = Cloth::new(input).unwrap();
        let mut previous = positions.clone();
        previous[0] = [3., positions[0][1], 3.];
        let mut x = positions.clone();
        let mut history = ContactState::default();
        assert_eq!(
            cloth
                .project_cloth(&mut x, &previous, &mut history, &|| false)
                .unwrap(),
            2
        );
        assert_eq!(history.cloth.len(), 2);
        x[0] = [3., 0.5, 3.];
        assert_eq!(
            cloth
                .project_cloth(&mut x, &previous, &mut history, &|| false)
                .unwrap(),
            0
        );
        assert!((x[0][1] - 0.4984).abs() <= 1e-14);
        assert!(history.cloth.is_empty());
        assert_eq!(&x[1..], &positions[1..]);

        let mut x = positions;
        cloth
            .project_cloth(&mut x, &previous, &mut history, &|| false)
            .unwrap();
        x[3..6].copy_from_slice(&[[0., 0., 0.], [0.5, 0., 0.], [1., 0., 0.]]);
        let before = x.clone();
        let old = history.cloth.clone();
        assert!(matches!(
            cloth.project_cloth(&mut x, &previous, &mut history, &|| false),
            Err(SimulationError::Unstable(_))
        ));
        assert_eq!(x, before);
        assert_eq!(history.cloth, old);
    }

    #[test]
    fn moving_contact_query_reaches_a_later_plane() {
        let input = planes_input(0.0116);
        let positions = input.positions.clone();
        let cloth = Cloth::new(input).unwrap();
        let mut x = positions.clone();
        assert_eq!(
            cloth
                .project_cloth(&mut x, &positions, &mut ContactState::default(), &|| false)
                .unwrap(),
            2
        );
        assert!((x[0][1] - 0.0056).abs() < 1e-14);
        assert_eq!((x[0][0], x[0][2]), (0.25, 0.25));
        assert_eq!(&x[1..], &positions[1..]);
    }

    #[test]
    fn partitioned_queries_preserve_sequential_contact_order() {
        let mut input = planes_input(0.009);
        input.positions[6..].copy_from_slice(&[
            [-1., 0.009, -1.],
            [-1., 0.009, 2.],
            [2., 0.009, -1.],
        ]);
        for pin in &mut input.pins {
            pin.target = input.positions[pin.vertex];
        }
        for i in 0..16 {
            let offset = input.positions.len();
            let x = if i < 8 {
                -10. - 4. * i as f64
            } else {
                10. + 4. * i as f64
            };
            let points = [[x, 0., 4.], [x + 1., 0., 4.], [x, 0., 5.]];
            input.positions.extend(points);
            input.material_uv.extend([[0., 0.], [1., 0.], [0., 1.]]);
            input.triangles.push([offset, offset + 1, offset + 2]);
            input.triangle_materials.push(input.triangle_materials[0]);
            input
                .pins
                .extend(points.into_iter().enumerate().map(|(i, target)| Pin {
                    vertex: offset + i,
                    target,
                }));
        }
        let positions = input.positions.clone();
        let bounds =
            triangle_bounds_cancellable(&input.triangles, &positions, &positions, 0.006, &|| false)
                .unwrap();
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        let mut candidates = Vec::new();
        hierarchy
            .visit(0, positions[0], &|| false, &mut |i| {
                if overlaps(&bounds[i], positions[0], positions[0]) {
                    candidates.push(i);
                }
                Ok(())
            })
            .unwrap();
        assert!(candidates.iter().position(|i| *i == 2) < candidates.iter().position(|i| *i == 1));
        let cloth = Cloth::new(input).unwrap();
        let mut x = positions.clone();
        assert_eq!(
            cloth
                .project_cloth(&mut x, &positions, &mut ContactState::default(), &|| false)
                .unwrap(),
            2
        );
        assert!((x[0][1] - 0.003).abs() < 1e-14);
        assert_eq!((x[0][0], x[0][2]), (0.25, 0.25));
        assert_eq!(&x[1..], &positions[1..]);
    }

    #[test]
    fn swept_hierarchy_candidates_equal_exhaustive_inclusive_queries() {
        let mut bounds = separated_bounds();
        bounds.extend([[[0.; 3], [1.; 3]], [[-3.; 3], [-1.; 3]]]);
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        for (a, b) in [
            ([0.5; 3], [100.5, 0.5, 0.5]),
            ([-4.; 3], [2.; 3]),
            ([0.; 3], [0.; 3]),
            ([1.; 3], [1.; 3]),
            ([123.5, 0.5, 0.5], [12286., 1., 1.]),
        ] {
            let expected: Vec<_> = bounds
                .iter()
                .enumerate()
                .filter_map(|(i, bounds)| overlaps(bounds, a, b).then_some(i))
                .collect();
            let mut actual = Vec::new();
            hierarchy
                .visit_swept(0, a, b, &|| false, &mut |i| {
                    if overlaps(&bounds[i], a, b) {
                        actual.push(i);
                    }
                    Ok(())
                })
                .unwrap();
            actual.sort_unstable();
            assert_eq!(actual, expected);
        }
    }

    fn separated_bounds() -> Vec<Bounds> {
        (0..4096)
            .map(|i| [[i as f64 * 3., 0., 0.], [i as f64 * 3. + 1., 1., 1.]])
            .collect()
    }

    #[test]
    fn hierarchy_is_conservative_and_prunes_separated_bounds() {
        let mut bounds = separated_bounds();
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        let mut candidates = 0;
        hierarchy
            .visit(0, [0.5; 3], &|| false, &mut |_| {
                candidates += 1;
                Ok(())
            })
            .unwrap();
        assert!(
            candidates <= 8,
            "separated triangles should only visit one leaf"
        );
        bounds.extend([[[0.; 3], [1.; 3]]; 17]);
        bounds.push([[-f64::MAX; 3], [f64::MAX; 3]]);
        bounds.push([[f64::NEG_INFINITY; 3], [f64::INFINITY; 3]]);
        bounds.reverse();
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        for point in [
            [-0.; 3],
            [0.5; 3],
            [1.; 3],
            [123.5, 0.5, 0.5],
            [12286., 1., 1.],
            [f64::MAX; 3],
            [-f64::MAX; 3],
        ] {
            let expected: Vec<_> = bounds
                .iter()
                .enumerate()
                .filter_map(|(i, bounds)| overlaps(bounds, point, point).then_some(i))
                .collect();
            let mut actual = Vec::new();
            hierarchy
                .visit(0, point, &|| false, &mut |i| {
                    if overlaps(&bounds[i], point, point) {
                        actual.push(i);
                    }
                    Ok(())
                })
                .unwrap();
            actual.sort_unstable();
            assert_eq!(actual, expected);
        }
        let bounds: Vec<_> = (0..4096)
            .map(|i| {
                let radius = ((i * 4051) % 4096 + 1) as f64;
                [[-radius; 3], [radius; 3]]
            })
            .collect();
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        let mut candidates = Vec::new();
        hierarchy
            .visit(0, [0.; 3], &|| false, &mut |i| {
                candidates.push(i);
                Ok(())
            })
            .unwrap();
        candidates.sort_unstable();
        assert_eq!(candidates, (0..bounds.len()).collect::<Vec<_>>());
    }

    #[test]
    fn hierarchy_build_and_partition_cancel_during_work() {
        let bounds = separated_bounds();
        let polls = Cell::new(0);
        TriangleHierarchy::new(&bounds, &|| {
            polls.set(polls.get() + 1);
            false
        })
        .unwrap();
        for stop_at in [1, 2, polls.get() / 2, polls.get()] {
            let calls = Cell::new(0);
            let result = TriangleHierarchy::new(&bounds, &|| {
                calls.set(calls.get() + 1);
                assert!(calls.get() <= stop_at);
                calls.get() == stop_at
            });
            assert!(matches!(result, Err(SimulationError::Cancelled)));
            assert_eq!(calls.get(), stop_at);
        }
        let mut indices: Vec<_> = (0..bounds.len()).collect();
        let calls = Cell::new(0);
        assert_eq!(
            partition_triangles(&mut indices, &bounds, 0, &|| {
                calls.set(calls.get() + 1);
                assert!(calls.get() <= 2);
                calls.get() == 2
            }),
            Err(SimulationError::Cancelled)
        );
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn hierarchy_query_cancels_when_all_bounds_overlap() {
        let bounds = vec![[[0.; 3], [1.; 3]]; CANCELLATION_CHECK_INTERVAL + 1];
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        let polls = Cell::new(0);
        hierarchy
            .visit(
                0,
                [0.5; 3],
                &|| {
                    polls.set(polls.get() + 1);
                    false
                },
                &mut |_| Ok(()),
            )
            .unwrap();
        assert_eq!(polls.get(), hierarchy.nodes.len() + bounds.len());
        for stop_at in [2, CANCELLATION_CHECK_INTERVAL + 1, polls.get() / 2] {
            let calls = Cell::new(0);
            let mut candidates = 0;
            assert_eq!(
                hierarchy.visit(
                    0,
                    [0.5; 3],
                    &|| {
                        calls.set(calls.get() + 1);
                        assert!(calls.get() <= stop_at);
                        calls.get() == stop_at
                    },
                    &mut |_| {
                        candidates += 1;
                        Ok(())
                    },
                ),
                Err(SimulationError::Cancelled)
            );
            assert_eq!(calls.get(), stop_at);
            assert!(candidates < bounds.len());
        }
    }

    #[test]
    fn leaf_candidates_and_bounds_stop_before_each_primitive() {
        let bounds = vec![[[0.; 3], [1.; 3]]; 8];
        let hierarchy = TriangleHierarchy::new(&bounds, &|| false).unwrap();
        for stop_at in 2..=bounds.len() + 1 {
            let calls = Cell::new(0);
            let mut candidates = 0;
            let result = hierarchy.visit(
                0,
                [0.5; 3],
                &|| {
                    calls.set(calls.get() + 1);
                    assert!(calls.get() <= stop_at);
                    calls.get() == stop_at
                },
                &mut |_| {
                    candidates += 1;
                    Ok(())
                },
            );
            assert_eq!(result, Err(SimulationError::Cancelled));
            assert_eq!(calls.get(), stop_at);
            assert_eq!(candidates, stop_at - 2);
        }
        let mut triangles = vec![[0, 1, 2]; CANCELLATION_CHECK_INTERVAL + 2];
        *triangles.last_mut().unwrap() = [usize::MAX; 3];
        let x = [[0.; 3], [1., 0., 0.], [0., 1., 0.]];
        let calls = Cell::new(0);
        let result = triangle_bounds_cancellable(&triangles, &x, &x, 0.002, &|| {
            calls.set(calls.get() + 1);
            assert!(calls.get() <= triangles.len());
            calls.get() == triangles.len()
        });
        assert_eq!(result, Err(SimulationError::Cancelled));
        assert_eq!(calls.get(), triangles.len());
    }
}
