"""Independent six-DOF physical-energy oracle; never imports or invokes Rust."""
import hashlib
import json
from pathlib import Path
import sys

import mpmath as mp

HERE = Path(__file__).resolve().parent
INPUT = json.loads((HERE / "INPUT.json").read_text())


def model(dps):
    mp.mp.dps = dps
    rest = [[mp.mpf(v) for v in row] for row in INPUT["rest_positions_m"]]
    uv = [[mp.mpf(v) for v in row] for row in INPUT["uv_m"]]
    y = mp.matrix([mp.mpf(v) for node in [1, 2] for v in INPUT["actual_f64_predictor_m"][node]])
    h = mp.mpf(INPUT["settings"]["dt"])
    theta = mp.mpf(INPUT["material"]["grain_angle"])
    si, co = mp.sin(theta), mp.cos(theta)
    areas = []
    masses = [mp.mpf(0)] * 4
    elements = []
    for triangle in INPUT["triangles"]:
        a, b, c = triangle
        dm = mp.matrix([[uv[b][0]-uv[a][0], uv[c][0]-uv[a][0]],
                        [uv[b][1]-uv[a][1], uv[c][1]-uv[a][1]]])
        area = abs(mp.det(dm)) / 2
        areas.append(area)
        for node in triangle:
            masses[node] += mp.mpf(INPUT["material"]["areal_density"]) * area / 3
        elements.append((triangle, dm**-1, area))

    def columns(q, element):
        triangle, inv, _ = element
        x = [mp.matrix(row) for row in rest]
        x[1], x[2] = mp.matrix(q[:3]), mp.matrix(q[3:])
        a, b, c = triangle
        ds = mp.matrix([[x[b][i]-x[a][i], x[c][i]-x[a][i]] for i in range(3)])
        f = ds * inv
        u, v = f[:, 0], f[:, 1]
        return co*u+si*v, -si*u+co*v

    constraints = []
    origin = mp.matrix([0]*6)
    for element in elements:
        w0, f0 = columns(origin, element)
        dw, df = mp.zeros(3, 6), mp.zeros(3, 6)
        for j in range(6):
            e = origin.copy(); e[j] = 1
            w, f = columns(e, element)
            dw[:, j], df[:, j] = w-w0, f-f0
        area = element[2]
        for a, b, c, k in [
            (dw.T*dw, dw.T*w0, (mp.fdot(w0,w0)-1)/2, 400*area),
            (df.T*df, df.T*f0, (mp.fdot(f0,f0)-1)/2, 400*area),
            (dw.T*df+df.T*dw, dw.T*f0+df.T*w0, mp.fdot(w0,f0), 100*area),
        ]:
            constraints.append((a, b, c, k))
    diagonal = mp.matrix([masses[node]/h**2 for node in [1,2] for _ in range(3)])

    def energy(q):
        value = mp.fdot(diagonal, mp.matrix([(q[i]-y[i])**2 for i in range(6)]))/2
        for element in elements:
            w, f = columns(q, element)
            cw, cf, cs = (mp.fdot(w,w)-1)/2, (mp.fdot(f,f)-1)/2, mp.fdot(w,f)
            value += element[2]*(400*cw*cw+400*cf*cf+100*cs*cs)/2
        return value

    def derivatives(q):
        g = mp.matrix([diagonal[i]*(q[i]-y[i]) for i in range(6)])
        hessian = mp.diag(diagonal)
        polynomial = mp.fdot(diagonal, mp.matrix([(q[i]-y[i])**2 for i in range(6)]))/2
        for a,b,c,k in constraints:
            value = mp.fdot(q, a*q)/2 + mp.fdot(b,q) + c
            j = a*q+b
            polynomial += k*value*value/2
            g += k*value*j
            hessian += k*(j*j.T+value*a)
        return g, hessian, polynomial

    q = mp.matrix([v for node in [1,2] for v in rest[node]])
    iterations = []
    for iteration in range(20):
        g, hh, polynomial = derivatives(q)
        iterations.append({"iteration": iteration, "gradient_norm_N": mp.nstr(mp.norm(g), 35)})
        assert abs(polynomial-energy(q)) < mp.mpf("1e-60")
        if mp.norm(g) < mp.mpf("1e-65"):
            break
        q -= mp.lu_solve(hh, g)
    else:
        raise ArithmeticError("independent Newton did not converge")
    g, hh, _ = derivatives(q)
    eigenvalues = list(mp.eigsy(hh, eigvals_only=True))
    assert min(eigenvalues) > 0

    # A direct closed-form reduction at exact quarter turn is independent of
    # the six-coordinate polynomial/Newton assembly above.
    length, halfedge = -rest[0][0], rest[2][1]
    delta = y[4]-halfedge
    mass_h2 = masses[1]/h**2
    kw, ks = 400*areas[0], 100*areas[0]
    ratio_predicted = y[4]/halfedge
    ratio = mp.findroot(lambda r: 2*mass_h2*halfedge**2*(r-ratio_predicted)+kw*r*(r*r-1), 1)
    ideal = mp.matrix([0,-halfedge*ratio,0,0,halfedge*ratio,0])

    # All minimizers have Phi <= Phi(rest), so inertia confines them to
    # ||q-y|| <= sqrt(2)*delta. In center/edge coordinates c,e this gives
    # ||c||<=delta and 1<=||e||<=1+2delta/b. The ideal warp term is convex
    # there; weft is a convex center quartic. Only (e.c/L)^2 needs a lower
    # Hessian bound. Account for the authored f64 grain's exact tiny rotation.
    shear_negative_bound = ks*(1+2*delta/halfedge)*delta/(halfedge*length**2)
    u_bound, v_bound = 1+delta/length, 1+2*delta/halfedge
    bnorm = u_bound+v_bound
    dnorm = mp.sqrt(2)/(2*length)+mp.sqrt(2)/(2*halfedge)
    j, hc = dnorm*bnorm, dnorm**2
    axis_theta_bound = 2*j*(2*j)+bnorm**2*hc+(bnorm**2+1)/2*(2*hc)
    shear_theta_bound = 2*(2*j)*(4*j)+2*bnorm**2*(2*hc)+bnorm**2*(4*hc)
    rotation_derivative_bound = 2*(2*kw*axis_theta_bound+ks*shear_theta_bound)
    rotation_perturbation_bound = rotation_derivative_bound*abs(theta-mp.pi/2)
    unique_hessian_lower_bound = mass_h2-shear_negative_bound-rotation_perturbation_bound
    assert unique_hessian_lower_bound > 6
    assert energy(q) <= energy(mp.matrix([v for node in [1,2] for v in rest[node]]))
    assert mp.norm(q-y) <= mp.sqrt(2)*delta

    # Check analytic derivatives against central differences of the separate
    # direct deformation-gradient energy at the three predeclared states.
    fd_errors = []
    epsilon = mp.mpf(INPUT["independent_force_FD_epsilon_m"])
    for label, point in [("rest", mp.matrix([v for node in [1,2] for v in rest[node]])), ("predictor", y), ("reference", q)]:
        analytic, _, _ = derivatives(point)
        errors = []
        for i in range(6):
            plus, minus = point.copy(), point.copy()
            plus[i] += epsilon; minus[i] -= epsilon
            errors.append(abs((energy(plus)-energy(minus))/(2*epsilon)-analytic[i]))
        fd_errors.append({"state": label, "maximum_component_error_N": mp.nstr(max(errors),35)})
        assert max(errors) < mp.mpf("1e-9")
    # Exact quartic central-difference truncation bound for |q_i|<=0.01m.
    fd_box_bound = []
    for i in range(6):
        bound = sum(k*abs(a[i,i])*(abs(b[i])+mp.mpf(".01")*sum(abs(a[i,j]) for j in range(6))) for a,b,_,k in constraints)*epsilon**2/2
        fd_box_bound.append(bound)
    assert max(fd_box_bound) < mp.mpf("1e-8")
    return {
        "precision_decimal_digits":dps,
        "areas_m2":[mp.nstr(a,70) for a in areas],
        "independent_masses_kg":[mp.nstr(m,70) for m in masses],
        "reference_free_positions_m":[[mp.nstr(q[3*i+j],75) for j in range(3)] for i in range(2)],
        "reference_free_positions_f64":[[float(q[3*i+j]) for j in range(3)] for i in range(2)],
        "gradient_norm_N":mp.nstr(mp.norm(g),35),
        "hessian_eigenvalues_N_per_m":[mp.nstr(v,35) for v in eigenvalues],
        "hessian_condition_number":mp.nstr(max(eigenvalues)/min(eigenvalues),35),
        "global_energy_sublevel_hessian_lower_bound_N_per_m":mp.nstr(unique_hessian_lower_bound,35),
        "shear_negative_curvature_bound_N_per_m":mp.nstr(shear_negative_bound,35),
        "grain_rotation_hessian_perturbation_bound_N_per_m":mp.nstr(rotation_perturbation_bound,35),
        "ideal_quarter_turn_cubic_crosscheck_position_error_m":mp.nstr(mp.norm(q-ideal),35),
        "reference_energy_J":mp.nstr(energy(q),35),
        "rest_trial_energy_J":mp.nstr(energy(mp.matrix([v for node in [1,2] for v in rest[node]])),35),
        "finite_difference_errors":fd_errors,
        "quartic_FD_component_bound_on_0_01m_box_N":mp.nstr(max(fd_box_bound),35),
        "newton_iterations":iterations,
    }


if __name__ == "__main__":
    assert sys.flags.optimize == 0
    result80, result100 = model(80), model(100)
    q80 = [mp.mpf(v) for row in result80["reference_free_positions_m"] for v in row]
    q100 = [mp.mpf(v) for row in result100["reference_free_positions_m"] for v in row]
    precision_error = mp.norm(mp.matrix(q80)-mp.matrix(q100))
    assert precision_error < mp.mpf("1e-60")
    result = {
        "status":"independent_reference_prepared_before_any_production_execution",
        "input_sha256":hashlib.sha256((HERE/"INPUT.json").read_bytes()).hexdigest(),
        "python":sys.version,
        "mpmath_version":mp.__version__,
        "mpmath_source_sha256":hashlib.sha256(Path(mp.__file__).read_bytes()).hexdigest(),
        "reference80":result80,
        "reference100":result100,
        "precision_crosscheck_position_norm_m":mp.nstr(precision_error,35),
        "unique_global_minimizer_argument":"Coercive positive inertia confines every global minimizer to the rest-energy ball. On that convex ball the ideal-quarter-turn Hessian has a positive lower bound after the only potentially negative shear cross-term; a conservative angular derivative bound subtracts the exact authored-f64 grain perturbation. Thus full physical energy is strictly convex on the containing ball. Dense Newton's stationary point is in it, proving unique global minimum and local stability.",
        "Rust_or_production_executed":False,
    }
    with (HERE/"REFERENCE.json").open("x") as f:
        json.dump(result,f,indent=2,allow_nan=False);f.write("\n")
    print(json.dumps(result,indent=2))
