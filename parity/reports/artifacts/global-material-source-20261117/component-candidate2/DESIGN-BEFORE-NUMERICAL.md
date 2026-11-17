Root-approved global design, before first numerical candidate execution

Replace all local pair/singleton material projections with one production material
sweep over every triangle and all free nodal inertia, using one actual rounded
material displacement p per vertex. Reset p per substep; retain p during outer
sweeps so nonmaterial stage displacements move the next anchor x-p.

Mass-weighted matrix-free Gauss–Newton, symmetric diagonal preconditioning,
bounded conjugate gradients, and the same complete h²Phi Armijo objective govern
every stage call, in production and in the independent network adapter.

The initial pre-execution proposal used max32 linear iterations and relative1e-8.
Existing physical analytic single-sweep tests demand1e-14m agreement with exact
small linear solves; relative1e-8 is insufficient to promise that assertion.
Before any candidate numerical execution, root explicitly approved the stronger
relative1e-12 stopping rule (squared1e-24), retaining max32PCG, original analytic
assertions, all network physical1e-8m/1e-6N criteria, and16 Armijo trials. No
material, input, geometry, damping, contact, seam, public setting or oracle threshold
was retuned.

There is no explicit N/T cap in weftforge-sim validation. Root accepted an O(N,T)
bound for finite allocated vectors, max32 matrix products and16 line-search
trials, with no new option or smaller input cap. Existing1..=1000 substeps/outer
iterations remain unchanged. Source documentation records actual extra linear
work and cancellation scan counts.

Old RED test and patch are archived verbatim separately. The stage adapter removes
obsolete MaterialGroup pair/order assertions, asserting exact retained membrane
triangle order and the original free/pinned partition instead. Original literal
physical data, mass-bit checks, independent direct-F objective, four-point FD
oracle, reference and tolerances remain unchanged.
