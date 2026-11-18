"""Independent six-coordinate energy reference; never imports or executes production."""
import hashlib
import json
import math
from pathlib import Path
import sys

import mpmath as mp

ROOT = Path(__file__).absolute().parent
INPUT = ROOT / 'INPUT.json'
DATA = json.loads(INPUT.read_bytes())
FREE = DATA['free_local_nodes']
N = 6


def vector_dot(a, b):
    return mp.fsum(x * y for x, y in zip(a, b))


class Energy:
    def __init__(self):
        self.xyz = [[mp.mpf(x) for x in p] for p in DATA['predictor_positions_m']]
        self.uv = [[mp.mpf(x) for x in p] for p in DATA['uv_m']]
        self.y = mp.matrix([x for node in FREE for x in self.xyz[node]])
        self.h = mp.mpf(DATA['settings']['h_s'])
        self.mass = [mp.mpf(x) for x in DATA['free_masses_kg']]
        self.constraints = []
        for tri, material in zip(DATA['triangles'], DATA['triangle_materials']):
            a, b, c = [self.uv[v] for v in tri]
            du1, dv1 = b[0] - a[0], b[1] - a[1]
            du2, dv2 = c[0] - a[0], c[1] - a[1]
            det = du1 * dv2 - du2 * dv1
            u = [(dv1 - dv2) / det, dv2 / det, -dv1 / det]
            v = [(du2 - du1) / det, -du2 / det, du1 / det]
            s, co = mp.sin(mp.mpf(material['grain_angle'])), mp.cos(mp.mpf(material['grain_angle']))
            w, f = [co * p + s * q for p, q in zip(u, v)], [-s * p + co * q for p, q in zip(u, v)]
            constants, derivatives = [], []
            for coefficients in (w, f):
                constants.append([mp.fsum(coefficients[j] * self.xyz[node][axis]
                                           for j, node in enumerate(tri) if node not in FREE)
                                  for axis in range(3)])
                derivatives.append([[coefficients[tri.index(FREE[i // 3])]
                                     if axis == i % 3 and FREE[i // 3] in tri else mp.mpf(0)
                                     for axis in range(3)] for i in range(N)])
            wb, fb = derivatives
            hessians = [
                mp.matrix([[vector_dot(wb[i], wb[j]) for j in range(N)] for i in range(N)]),
                mp.matrix([[vector_dot(fb[i], fb[j]) for j in range(N)] for i in range(N)]),
                mp.matrix([[vector_dot(wb[i], fb[j]) + vector_dot(fb[i], wb[j])
                            for j in range(N)] for i in range(N)]),
            ]
            stiffness = [abs(det) / 2 * mp.mpf(material[k])
                         for k in ('warp_stiffness', 'weft_stiffness', 'shear_stiffness')]
            self.constraints.append((constants, derivatives, hessians, stiffness))

    def rows(self, q):
        result = []
        for constants, derivatives, hessians, stiffness in self.constraints:
            vectors = [[constants[a][d] + mp.fsum(derivatives[a][j][d] * q[j] for j in range(N))
                        for d in range(3)] for a in range(2)]
            w, f = vectors
            wb, fb = derivatives
            values = [(vector_dot(w, w) - 1) / 2, (vector_dot(f, f) - 1) / 2, vector_dot(w, f)]
            grads = [[vector_dot(w, wb[j]) for j in range(N)],
                     [vector_dot(f, fb[j]) for j in range(N)],
                     [vector_dot(wb[j], f) + vector_dot(w, fb[j]) for j in range(N)]]
            result.extend(zip(values, grads, hessians, stiffness))
        return result

    def evaluate(self, q):
        value = mp.mpf(0)
        gradient = mp.matrix(N, 1)
        hessian = mp.matrix(N, N)
        for i in range(N):
            scale = self.mass[i // 3] / self.h**2
            value += scale * (q[i] - self.y[i])**2 / 2
            gradient[i] = scale * (q[i] - self.y[i])
            hessian[i, i] = scale
        for c, g, hc, k in self.rows(q):
            value += k * c * c / 2
            for i in range(N):
                gradient[i] += k * c * g[i]
                for j in range(N):
                    hessian[i, j] += k * (g[i] * g[j] + c * hc[i, j])
        return value, gradient, hessian

    def direct_energy(self, q):
        """Separate direct F computation, without the polynomial derivative tables."""
        xyz = [list(p) for p in self.xyz]
        for i, node in enumerate(FREE):
            xyz[node] = list(q[3 * i:3 * i + 3])
        value = mp.fsum(self.mass[i // 3] * (q[i] - self.y[i])**2 / (2 * self.h**2) for i in range(N))
        for tri, material in zip(DATA['triangles'], DATA['triangle_materials']):
            a, b, c = [self.uv[v] for v in tri]
            du1, dv1 = b[0] - a[0], b[1] - a[1]
            du2, dv2 = c[0] - a[0], c[1] - a[1]
            det = du1 * dv2 - du2 * dv1
            e1 = [xyz[tri[1]][i] - xyz[tri[0]][i] for i in range(3)]
            e2 = [xyz[tri[2]][i] - xyz[tri[0]][i] for i in range(3)]
            u = [(e1[i] * dv2 - e2[i] * dv1) / det for i in range(3)]
            v = [(e2[i] * du1 - e1[i] * du2) / det for i in range(3)]
            s, co = mp.sin(mp.mpf(material['grain_angle'])), mp.cos(mp.mpf(material['grain_angle']))
            w = [co * a + s * b for a, b in zip(u, v)]
            f = [-s * a + co * b for a, b in zip(u, v)]
            cs = [(vector_dot(w, w) - 1) / 2, (vector_dot(f, f) - 1) / 2, vector_dot(w, f)]
            value += abs(det) / 4 * mp.fsum(mp.mpf(material[k]) * c**2
                                           for k, c in zip(('warp_stiffness', 'weft_stiffness', 'shear_stiffness'), cs))
        return value


def maximum(values):
    return max(abs(x) for x in values)


def row_norm(matrix):
    return max(mp.fsum(abs(matrix[i, j]) for j in range(N)) for i in range(N))


def describe(x):
    return mp.nstr(x, 64)


def run(dps):
    with mp.workdps(dps):
        model = Energy()
        q = mp.matrix(model.y)
        trace = []
        for iteration in range(80):
            energy, g, hessian = model.evaluate(q)
            residual = mp.sqrt(vector_dot(g, g))
            trace.append({'iteration': iteration, 'gradient_norm_N': describe(residual), 'energy_J': describe(energy)})
            if residual < mp.mpf('1e-55'):
                break
            eigenvalues = mp.eigsy(hessian, eigvals_only=True)
            shift = max(mp.mpf(0), 1 - eigenvalues[0])
            direction = mp.lu_solve(hessian + shift * mp.eye(N), -g)
            slope = vector_dot(g, direction)
            assert slope < 0
            alpha = mp.mpf(1)
            for _ in range(100):
                trial = q + alpha * direction
                trial_e, trial_g, _ = model.evaluate(trial)
                if trial_e <= energy + mp.mpf('0.0001') * alpha * slope or (
                    residual < mp.mpf('1e-20') and mp.sqrt(vector_dot(trial_g, trial_g)) < residual / 2
                ):
                    q = trial
                    break
                alpha /= 2
            else:
                raise ArithmeticError('Independent Newton line search failed; fixture remains fixed')
        else:
            raise ArithmeticError('Independent Newton reference did not converge; fixture remains fixed')
        energy, gradient, hessian = model.evaluate(q)
        eigenvalues = mp.eigsy(hessian, eigvals_only=True)
        assert maximum(gradient) < mp.mpf('1e-55') and eigenvalues[0] > 0
        # Quartic energy: a fourth-order difference differentiates its polynomial exactly.
        epsilon = mp.mpf('1e-8')
        selfchecks = []
        for name, point in [('predictor', model.y), ('reference', q)]:
            e, g, h = model.evaluate(point)
            energy_error = abs(e - model.direct_energy(point))
            gradient_error, hessian_error = mp.mpf(0), mp.mpf(0)
            for axis in range(N):
                basis = mp.matrix(N, 1)
                basis[axis] = epsilon
                em2, em1, ep1, ep2 = [model.direct_energy(point + a * basis) for a in [-2, -1, 1, 2]]
                gradient_error = max(gradient_error, abs((em2 - 8 * em1 + 8 * ep1 - ep2) / (12 * epsilon) - g[axis]))
                gradients = [model.evaluate(point + a * basis)[1] for a in [-2, -1, 1, 2]]
                for i in range(N):
                    fd = (gradients[0][i] - 8 * gradients[1][i] + 8 * gradients[2][i] - gradients[3][i]) / (12 * epsilon)
                    hessian_error = max(hessian_error, abs(fd - h[i, axis]))
            assert max(energy_error, gradient_error, hessian_error) < mp.mpf('1e-50')
            selfchecks.append({'state': name, 'direct_energy_error_J': describe(energy_error),
                               'fourth_order_gradient_error_N': describe(gradient_error),
                               'fourth_order_Hessian_error_N_per_m': describe(hessian_error)})
        # Conservative analytic Hessian variation over the declared 1e-8m infinity box.
        radius = mp.mpf('1e-8')
        variation = mp.matrix(N, N)
        for c, g, hc, k in model.rows(q):
            a = [radius * mp.fsum(abs(hc[i, j]) for j in range(N)) for i in range(N)]
            b = radius * mp.fsum(abs(x) for x in g) + radius**2 * mp.fsum(abs(x) for x in hc) / 2
            for i in range(N):
                for j in range(N):
                    variation[i, j] += k * (abs(g[i]) * a[j] + abs(g[j]) * a[i] + a[i] * a[j] + b * abs(hc[i, j]))
        delta = row_norm(variation)
        inverse = hessian**-1
        contraction = row_norm(inverse) * delta
        center_step = maximum(inverse * gradient)
        lower = eigenvalues[0] - delta
        assert lower > 0 and contraction < 1 and center_step + contraction * radius < radius
        q64 = [float(x) for x in q]
        _, rounded_g, _ = model.evaluate(mp.matrix([mp.mpf(x) for x in q64]))
        return {'precision_decimal_digits': dps,
                'reference_free_positions_m': [[describe(q[3*i+j]) for j in range(3)] for i in range(2)],
                'reference_free_positions_f64': [q64[:3], q64[3:]],
                'energy_J': describe(energy), 'predictor_energy_J': describe(model.direct_energy(model.y)),
                'gradient_norm_N': describe(mp.sqrt(vector_dot(gradient, gradient))),
                'f64_reference_gradient_norm_N': describe(mp.sqrt(vector_dot(rounded_g, rounded_g))),
                'Hessian_eigenvalues_N_per_m': [describe(x) for x in eigenvalues],
                'local_certificate': {'infinity_box_radius_m': describe(radius),
                                      'Hessian_variation_spectral_upper_bound_N_per_m': describe(delta),
                                      'Hessian_lower_bound_N_per_m': describe(lower),
                                      'fixed_Newton_map_contraction_upper_bound': describe(contraction),
                                      'center_Newton_step_infinity_norm_m': describe(center_step),
                                      'self_mapping_bound_m': describe(center_step + contraction * radius),
                                      'interpretation': 'Analytic quadratic-constraint bounds evaluated at high precision; unique stationary point/strict minimum in this local box. Not a formally directed-rounding interval implementation or global-minimum proof.'},
                'selfchecks': selfchecks, 'newton_trace': trace}


if __name__ == '__main__':
    a, b = run(80), run(100)
    with mp.workdps(110):
        cross = mp.sqrt(mp.fsum((mp.mpf(x) - mp.mpf(y))**2 for p, q in zip(a['reference_free_positions_m'], b['reference_free_positions_m']) for x, y in zip(p, q)))
        assert cross < mp.mpf('1e-50')
    result = {'status': 'INDEPENDENT_REFERENCE_BEFORE_PRODUCTION_RED',
              'input_sha256': hashlib.sha256(INPUT.read_bytes()).hexdigest(),
              'python': sys.version, 'mpmath_version': mp.__version__,
              'mpmath_init_sha256': hashlib.sha256(Path(mp.__file__).read_bytes()).hexdigest(),
              'reference80': a, 'reference100': b, 'precision_crosscheck_position_norm_m': describe(cross),
              'global_minimum_or_uniqueness_established': False,
              'scope': 'One fixed six-free-coordinate material-only component. Positive local Hessian/stationarity plus a conservative local-box contraction certificate; no global claim or dynamics/model execution.',
              'production_Cloth_initializations': 0, 'production_steps_tests_Cargo': 0}
    with (ROOT / 'REFERENCE.json').open('x') as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
        stream.write('\n')
    print(json.dumps(result, indent=2, allow_nan=False))
