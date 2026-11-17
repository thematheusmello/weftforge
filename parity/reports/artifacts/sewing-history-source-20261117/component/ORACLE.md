# Independent positive-target sewing oracle

This derivation uses only the declared physical energy. It does not use private
solver multipliers, the constraint projection formula, or a simulation result.
All numerical assertions are prospective; this source-only packet has not run.

Let the free point be q, predictor p=(0,1,0), pinned origin o=(0,0,0), and pinned
anchor a=(1,0,0). The free mass is m=.01 kg, h=.1 s, radial target t=.5 m,
and both stiffnesses are k=.5 N/m (compliance 2 m/N). Thus m/h^2=1 kg/s^2.
The backward-Euler incremental objective in joules is

    Phi(q) = .5 |q-p|^2 + .25 (|q|-.5)^2 + .25 |q-a|^2.

The inertia plus closed-spring coefficient is A=1+.5=1.5; after adding the
radial spring, the quadratic coefficient is A+k=2. Define

    B = p + .5 a = (.5,1,0),  |B| = sqrt(5)/2.

For radius r and unit direction n, direct expansion gives

    Phi(r n) = r^2 - r B.n - .25 r + 13/16.

For each r>0, Cauchy-Schwarz gives the unique minimizing direction n=B/|B|.
Minimizing the resulting scalar quadratic gives the unique global minimum

    r* = (|B|+.25)/2 = sqrt(5)/4 + 1/8,
    q* = (.25 + 1/(8 sqrt(5)), .5 + 1/(4 sqrt(5)), 0),
    Phi(q*) = 31/64 - sqrt(5)/16.

The origin cannot minimize Phi: its one-sided derivative along B is
-(|B|+.25)<0. There is no antipodal stationary point with positive radius,
because it would require .25-2r=|B|, whereas |B|>.25. No global convexity claim
is needed for this argument. The unique minimum has r*>.5 m; it is far from
the radial singularity. The fixture starts at radius 1 m and guards every
stage against approaching within 1e-6 m of the origin. h, compliance, and
inverse masses are ordinary finite values; the positive denominators are
approximately 300 kg^-1, without overflow or guard-limit inputs.

Independent elastic forces on the free node, in newtons, are

    f_radial(q) = -.5 (|q|-.5) q/|q|,
    f_closed(q) = -.5 (q-a).

The independent stationarity residual is

    R(q) = (m/h^2)(q-p) - f_radial(q) - f_closed(q)
         = 2q - B - .25 q/|q|.

The test evaluates the force equations directly, independently of production
math helpers and multiplier conventions. It verifies R(q*) within 1e-14 N,
then freezes 1e-8 m position and 1e-6 N residual bounds after exactly 32 sweeps.
Binary f64 representations of .01 and .1 introduce only roundoff relative to
the exact SI coefficient 1; the test uses m/h^2 for the measured residual.

The test also independently accumulates each stage's actual free-node position
change, J_i=(m/h^2) sum(delta_q_i), in force-equivalent newtons. Their sum
telescopes to (m/h^2)(q-p). At a physically stationary split solution, each
J_i must match that spring's independently differentiated force; those two
action-balance bounds are also fixed at 1e-6 N. It reports J_i, the current
multiplier-derived force, elastic force, final-stage displacement, position,
energy, and initial/final radial directions before the acceptance assertions.

The bounded hypothesis is that summing scalar changes against changing radial
directions need not equal the final scalar multiplier against the final
direction. The closed spring rotates the direction: p is not parallel to B.
An isolated radial pair would keep a constant direction and cannot test this.
Reported private history is explanatory evidence only, never the oracle.

A future failure alone is not automatically a history diagnosis: the retained
final-stage updates and force/action split must distinguish unresolved finite
iteration work from persistent inconsistent history. This component cannot
establish any garment failure's cause or a garment/physics/parity pass.
