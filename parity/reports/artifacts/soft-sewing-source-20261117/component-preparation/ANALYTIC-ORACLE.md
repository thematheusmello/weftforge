This is an independent specification, not a test/result or garment simulation.

Use two free Cartesian points, fresh predictors p0=(0,0,.015)m and
p1=(0,0,-.015)m, masses m0=4.5714285714285506e-05kg and m1=4.571428571428556e-05kg from rest vertices45/111,
h=1/480s, and two identical endpoint-distance springs in input order. Each has
compliance1e-8m/N (k=1e8N/m), target t=0.01333333333333439m. The target is a literal
prescribed scalar, not an archived dynamic state. Histories start Vec3 zero
once; perform exactly32 production sewing sweeps. No material, bending, contact,
gravity, warmstart or fullCloth stage is part of this component.

The independent complete energy is
 E = m0|q0-p0|²/(2h²)+m1|q1-p1|²/(2h²)+k(|q0-q1|-t)².
Both duplicate energy terms are retained: .5k+.5k=k. Removing a row would alter
physical energy and is not a permissible fixture/solver repair.

Let M=m0+m1, C=(m0p0+m1p1)/M, d=q0-q1, mu=m0m1/M and D=mu/h².
Eliminating COM gives .5D|d-d0|²+k(|d|-t)², d0=p0-p1=(0,0,.03).
For fixed radius r, its unique angular minimum aligns d with d0, since the
only angular term is -D dot(d,d0) and d0 is nonzero. The scalar energy
.5D(r-.03)²+k(r-t)² has curvature D+2k>0 and positive unique minimizer
 r*=(D*.03+2kt)/(D+2k)=0.013333333772191522m.
Thus the global Cartesian minimum is unique (not an arbitrary sphere):
 q0*=C+(m1/M)r*ez=(0,0,0.006666666886095756),
 q1*=C-(m0/M)r*ez=(0,0,-0.006666666886095767).

Independent force oracle, with u=(q0-q1)/|q0-q1|:
 R0=m0(q0-p0)/h²+2k(|q0-q1|-t)u,
 R1=m1(q1-p1)/h²-2k(|q0-q1|-t)u.
Declare before any candidate: max Cartesian position error<=1e-8m,
max(|R0|,|R1|)<=1e-6N, finite positions/histories and COM preservation.
The position assertion alone is insufficient for this stiff problem.

Source-level exact-real recurrence predicts a failure; no Python projection
emulation or Rust command has executed it. With c=D/(D+k), a local radial
prox sends e=r-t to c(e-p_j) and history p_j to -(1-c)(e-p_j).
Dykstra conservation gives e=e0+p0_history+p1_history. Eliminating the first
row gives the second-history recurrence b_n=(1-c)²b_(n-1)-(1-c)c e0.
Starting histories zero yields
 r_N-t=(r*-t)(1-rho^N), rho=(k/(D+k))²=0.999999894674294.
The physical force residual magnitude is D(.03-t)rho^N, hence at32
 0.08777113274509969N, while maximum point error is only
 2.1942782602963774e-10m.
This is the specific frozen expected RED criterion, not an actual product RED.
Ordinary radii stay positive, so zero-anchor/subnormal safeguards are not involved.

This component has a fixed director by symmetry. It can establish a general
finite-budget shared-row coupling defect, not the garment's frame28 rotation,
its contact reactions or the frame29 collar failure. Those require stage
predictors/history/contact evidence or a separately proved noncollinear coupled
case before any causal source repair. No prefix observer is started/proposed
for execution here; root owns later scope and all new compute GOs.
