The sewing-only repair retains the complete original soft energy and nodal
anchor z=x-p. Singleton analytic prox, material and bending stay unchanged.

With scaled residual r, J=dr/dx, W=diag inverse masses and p_free zero at pins:
 a=p_free/sqrt(W), B=J sqrt(W), g=a+B^T r.
Primal GN is (I+B^TB)u=-a-B^Tr.
Woodbury gives (I+BB^T)eta=B a-r, u=-a+B^Teta.
Thus delta_free=-p_free+WJ^Teta, delta_pin0. Jp is never discarded.
At fraction alpha, p_new=(1-alpha)p+alpha WJ^Teta, before position rounding.
This is weighted row action at every finite dual iterate when old p is row action;
no COM recentering, duplicate deletion, coefficient normalization or history reset.
The actual true slope is g·u, not a dual quadratic surrogate. Existing full
sewing_energy controls the same1e-4/16 Armijo. PCG stays32/relative1e-12.
RHS0 skips CG, but still recovers -p and tests true descent/energy.

The matrix-free preconditioned operator uses row scales D=sqrt(diag(I+BB^T)):
 Hbar v=D^-2 v + (D^-1 B)(D^-1 B)^T v.
RHS=(Ba-r)/D, normalized by its max absolute entry. Recovery scatters the
normalized rows times the restored solution then subtracts old a.
Pins have zero a/B/recovered displacement and unchanged coordinates.

General proof: any k in ker(J_free) obeys k^T W^-1 (p_new-(1-alpha)p)=0;
for zero-target rows the coefficient-space kernel is fixed and allows direct
independent testing. It is a stronger check than only total momentum.

The new guard uses C0=.5A0+.5A1-.25B0-.75B1 and
C1=.5A1+.5A2-.25B1-.75B2. Both constant1 and
k=[0,1,2,-.25,.75,1.75] are exact null vectors. Node A0 can be pinned because k0=0.
Masses[1,4,.25,1,4,.25], h=.5,c=.25 and dyadic positions/impulses retain an
exact zero-RHS configuration. Manufacture q*, r*=Cq*, z=q*+W C^T(h²/c)r*;
choose nonzero p0=W C^T old_impulse and x=z+p0. Then the independently computed
free physical gradient atq* vanishes. Positive inertia makes the quadratic
strictly convex, so q* is the unique free-node minimum. One configuration has
r*=0 but p0!=0, catching a mistaken zero-RHS return. Bounds are1e-12m/1e-12N,
1e-12kgm kernel/total action and1e-14m actual nodal-history transport, with exact
pins/finite checks each of32 sweeps. All old tests/oracles/thresholds stay intact.

New dual work is O(N+R), at most32 operator products and17 full objective scans.
Checkpoint bound71N+166R+19S+48 <=71N+517S+48. Polling remains entry/every64th
checkpoint/final pre-copy or successful return, at most63 skipped units; copying
and allocation are uninterruptible. All numerical/cancellation failures precede
any x/p copy. Workspaces/history remain local to the whole-Cloth transaction.
No runtime has occurred. Root+independent static review precedes the one focused
unchanged failed core test, and only if it passes, the full82 sim-lib tests.
