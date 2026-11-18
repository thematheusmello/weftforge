"""Independent incremental-energy oracle and one declared local solver comparison."""
import json
from pathlib import Path

import numpy as np
from scipy.optimize import least_squares

OUT = Path(__file__).resolve().parent
uv = np.array([[0.0, 0.0], [0.01, 0.0], [0.003, 0.07]])
predicted = np.array([0.012, 0.002])
h = (1.0 / 60.0) / 8.0
area = np.linalg.det(np.column_stack((uv[1] - uv[0], uv[2] - uv[0]))) / 2.0
mass = 0.18 * area / 3.0
k = area * np.array([400.0, 400.0, 100.0])
angle = np.pi / 2.0
rotation = np.array([[np.cos(angle), -np.sin(angle)], [np.sin(angle), np.cos(angle)]])
dm_inverse = np.linalg.inv(np.column_stack((uv[1] - uv[0], uv[2] - uv[0])))


def constraints(q):
    # Direct physical deformation matrix, independent of the runtime's cached coefficients.
    ds = np.column_stack((q - uv[0], uv[2] - uv[0]))
    f = ds @ dm_inverse @ rotation
    return np.array([(f[:, 0] @ f[:, 0] - 1.0) / 2.0,
                     (f[:, 1] @ f[:, 1] - 1.0) / 2.0,
                     f[:, 0] @ f[:, 1]])


def energy_residual(q):
    return np.concatenate((np.sqrt(mass) / h * (q - predicted), np.sqrt(k) * constraints(q)))


def energy(q):
    r = energy_residual(q)
    return (r @ r) / 2.0


def jacobian(fun, q):
    return np.column_stack([np.imag(fun(q.astype(complex) + np.eye(2)[i] * 1e-30j)) / 1e-30 for i in range(2)])


def force(q):
    return jacobian(energy_residual, q).T @ energy_residual(q)


oracle = least_squares(energy_residual, predicted, jac='cs', xtol=1e-14, ftol=1e-14, gtol=1e-14, max_nfev=100)
reference = oracle.x
assert np.linalg.norm(force(reference)) < 1e-9
step = 1e-7
hessian = np.column_stack([(force(reference + np.eye(2)[i] * step) - force(reference - np.eye(2)[i] * step)) / (2.0 * step) for i in range(2)])
assert np.linalg.eigvalsh(hessian).min() > 0.0

position_tolerance = 1e-8
stationarity_tolerance = 1e-6
results = {}
for mode in ['scalar_xpbd', 'block_lambda_xpbd', 'primal_correction_gauss_newton']:
    q = predicted.copy()
    lam = np.zeros(3)
    correction = np.zeros(2)
    energy_path = []
    for iteration in range(32):
        if mode == 'scalar_xpbd':
            for kind in range(3):
                c = constraints(q)
                j = jacobian(constraints, q)[kind]
                alpha = 1.0 / k[kind] / (h * h)
                delta = (-c[kind] - alpha * lam[kind]) / ((j @ j) / mass + alpha)
                lam[kind] += delta
                q += j / mass * delta
        else:
            c = constraints(q)
            j = jacobian(constraints, q)
            alpha = 1.0 / k / (h * h)
            matrix = (j @ j.T) / mass + np.diag(alpha)
            if mode == 'block_lambda_xpbd':
                delta = np.linalg.solve(matrix, -c - alpha * lam)
                lam += delta
                q += j.T @ delta / mass
            else:
                anchor = q - correction
                multiplier = np.linalg.solve(matrix, j @ correction - c)
                direction = -correction + j.T @ multiplier / mass
                value = mass / (2.0 * h * h) * (correction @ correction) + (k * c * c).sum() / 2.0
                gradient = mass / (h * h) * correction + j.T @ (k * c)
                derivative = gradient @ direction
                fraction = 1.0
                for _ in range(16):
                    trial = q + fraction * direction
                    trial_correction = correction + fraction * direction
                    trial_c = constraints(trial)
                    trial_value = mass / (2.0 * h * h) * (trial_correction @ trial_correction) + (k * trial_c * trial_c).sum() / 2.0
                    if trial_value <= value + 1e-4 * fraction * derivative:
                        q = trial
                        correction = trial_correction
                        break
                    fraction *= 0.5
                assert np.linalg.norm(q - correction - anchor) < 1e-14
        energy_path.append(float(energy(q)))
    results[mode] = {
        'position_m': q.tolist(),
        'position_error_m': float(np.linalg.norm(q - reference)),
        'stationarity_N': force(q).tolist(),
        'stationarity_norm_N': float(np.linalg.norm(force(q))),
        'energy_J': float(energy(q)),
        'energy_path_J': energy_path,
        'passes_declared_position_and_force': bool(np.linalg.norm(q - reference) < position_tolerance and np.linalg.norm(force(q)) < stationarity_tolerance),
    }

report = {
    'scope': 'One declared analytic material patch, not a garment or reference-app comparison; prototype is not production red/green evidence.',
    'uv_m': uv.tolist(), 'pinned_vertices': [0, 2], 'free_vertex': 1,
    'predicted_free_position_m': predicted.tolist(), 'substep_h_s': h,
    'area_m2': float(area), 'free_mass_kg': float(mass), 'energy_coefficients_Nm': k.tolist(),
    'iterations': 32, 'position_tolerance_m': position_tolerance, 'stationarity_tolerance_N': stationarity_tolerance,
    'oracle': 'SciPy least_squares with complex-step Jacobian of explicit deformation-gradient energy residual; no runtime constraint helpers.',
    'reference_position_m': reference.tolist(), 'reference_stationarity_N': force(reference).tolist(),
    'reference_energy_J': float(energy(reference)), 'reference_hessian_eigenvalues_N_per_m': np.linalg.eigvalsh(hessian).tolist(),
    'results': results,
}
(OUT / 'REFERENCE.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report, indent=2))
