This is an unexecuted source candidate for the demonstrated shared-row component.
No garment cause, accepted garment grade, reference parity or speed claim follows.

`Cloth::step` resets one nodal soft-sewing displacement p_s once per substep,
then calls `solve_sewing` in the existing sewing slot. The source passes the
same input rows and unchanged temporary target array. Material, bending, contact,
pin, target-update, velocity and final whole-call commit order stay unchanged.
Inside sewing, all soft rows form one block, followed by the unchanged hard
projections in input order. This explicit mixed hard/soft ordering changes their
finite-iteration interaction; all-hard order/arithmetic remains identical.

Energy and units

For inverse mass w_i and anchor z_i=x_i-p_s_i, minimize
 h² Phi(q)=sum_free |q_i-z_i|²/(2w_i)
           +sum_soft h² (|a_s(q)-b_s(q)|-target_s)²/(2c_s).
Every supplied soft spring energy is included once, even exact duplicates.
Repeated endpoint indices merge signed coefficients beta only for scatter;
accepted weights and endpoint gathers are neither snapped nor normalized.
The zero-target vector energy uses three Cartesian residuals; positive targets
use their radial residual. Physical c, masses, h, targets and outer32 are unchanged.

p_s is metres, not force. p_s/sqrt(w_i) and each h/sqrt(c_s) spring residual have
units metres*sqrt(kg), giving h² times physical energy. The mass-normalized GN
system is (I+J^T J)u=-p_s/sqrt(W)-J^T r, with delta_x=sqrt(W)u and zero pin entries.
It shares one inertia term per free node across all incident springs. Sparse
weighted gradients have equal/opposite physical internal reactions for exact
convex point weights; the fixed finite-budget linear solve is not an arbitrary
mesh equilibrium certificate. The frozen two-free-node COM check remains1e-14m.

Exactly one compliant input spring takes the general analytic proximal path,
not an ID or duplicate-row special case. Eliminating nodal inertia for W=sum(w beta²)>0
gives |D-d_z|²/(2W)+(|D|-target)²/(2alpha), alpha=c/h². Its minimizer has
 H=unit(d_z)*(target-|d_z|)/(alpha+W), q_i=z_i+w_i beta_i H;
for target0, H=-d_z/(alpha+W) preserves the original component division and
weighted-endpoint arithmetic. Free nodes outside the spring minimize at z;
pins stay unchanged. At zero anchor any direction minimizes the radial term;
use the current direction when nonzero, otherwise+X. Both unit paths use maxabs
scaling. The original finite/denominator guard is retained. A private finite
trial must not increase the same full inertial+spring objective. No per-row soft
history survives: p_s accumulates actual rounded position displacement.

For two or more compliant inputs, including duplicates, the existing bending
fixed-four-node PCG/Armijo tail is mechanically extracted as CoupledBlock::update.
Its numerical expressions, operation/visit order, diagonal and RHS scaling,
32 linear cap, relative1e-12 residual, Armijo1e-4/16 trials and final copy guards
are unchanged. Bending assembly/energy only rename the private row/index fields;
material and Bend constructor/dihedral/project arithmetic is byte-exact.
The actual radial/vector objective governs trials, including zero-gap cases.
At a positive-target zero current gap, the block uses a nonzero weighted anchor
direction or+X. This deterministic cusp choice is no global convergence proof.

Cost and cancellation

For N vertices, S input stitches and R=3*S_zero+S_positive<=3S soft rows,
shared workspace is O(N+R). Each coupled sweep has at most32 matrix products and
17 objective scans; checkpoint bound is168N+34R+19S+48<=168N+121S+48.
The singleton has O(N) workspace, at most two objective scans, and at most
S+4N+2 checkpoints, below the shared bound. Scatter/merge use bounded four-node
loops. Original cancellation polls occur at entry, each64th checkpoint and
immediately before every successful return/copy: no more than63 skipped sites;
callback bound2+floor(bound/64). Allocations and final copies are uninterruptible,
so no wall-time response is promised. No Cancelled result occurs after copying.
All candidates are private until finite/descent/cancellation acceptance. New
histories remain substep locals within the existing whole-call transaction;
reset/clone introduce no persistent fields or serialization changes.

Tests and scope

CRITERION-PROOF.json binds the protected duplicate oracle/32/position1e-8m/
force1e-6N/COM-finite checks outside the approved adapter/history-unit changes.
The existing mixed positive/zero independent oracle keeps its position/force
bounds. Both min-subnormal45-degree cases keep original expected coordinates,
physical radius/force1e-14 bounds, pins and finiteness. All physical bit guards
are retained. Retired private row-history/operator-ledger checks become current
nodal history, independent constitutive impulses and total-action balance;
they do not pretend the group has unique per-row applied displacements.

Three new guards cover an independent weighted/repeated-index/noncollinear
zero-target2x2 SPD energy, every actual callback of a moving direct block plus
finite/no-op atomicity, and every whole-Cloth callback plus clone/reset. For the
SPD case each Cartesian Hessian is[[12.125,-1.625],[-1.625,6.125]]; positive leading
minor and determinant71.625 prove the unique minimum. RHS and full force gradients
are literal independent formulas in the test; no numerical-oracle framework.
All old material/bending and existing public rollback/reset tests are byte-exact.
No public API/schema/dependency change, tuning, row deletion or garment run exists.

Pre-edit source91 is archived at ../red-source-archive-001 with Git d8bcb3b7.
Original proposal/RED seals and raw controller misclassification qualification
remain untouched. Existing own target is from that one RED; no cache/binary copy.
No formatter, Cargo, test, constructor, model, analyzer or GUI has run on candidate.
Root plus independent static review and a distinct first-GREEN release are next.
