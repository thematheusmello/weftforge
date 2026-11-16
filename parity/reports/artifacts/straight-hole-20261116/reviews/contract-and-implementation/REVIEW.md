# First straight-hole proposal — independent source review

**Result: one contract wording blocker; no other blocker found in the bounded
proposal.** Review is source-only and grants no implementation, native execution,
reference parity or acceptance authority. No implementation, source change,
Cargo/product/CLI/formal evaluator, GUI/CUA or launch was performed.

Reviewed input SHA256:

- CONTRACT.md: `88724542d5b3577bc03564223cd8efa9bd7f19d82c60addd417be8542fe393d8`
- ORACLE-SUMMARY.json: `6cce2ccc38d28e2bc4b724e32867a052461c63c9b711378d1a2b414a63a84ba2`
- SOURCE-HASHES.json: `6c3417a60769d0c790d312b3aa0c8779d970dfc1f715ede944daa3be33cf2884`

## Blocker: failed-save recovery promise contradicts retained behavior

CONTRACT.md:151–153 promises byte-exact destination **and recovery** after every
failed save while retaining interrupted-save behavior. Current
`crates/weftforge-core/src/persistence.rs:292–307` stages the new primary, then
atomically replaces recovery with the exact old primary at line 300, and only
afterward replaces the primary. A later failure or injected interruption can
therefore leave primary unchanged while recovery advances. This matches the
existing straight-cut policy; two-file atomicity is not available or needed.

Static failure recipe: primary B and valid older recovery A exist; save C; recovery
successfully becomes B; inject the existing pre-primary-replace interruption.
Primary remains B and recovery is now B. The proposal's unconditional recovery
claim would incorrectly fail this retained behavior. This recipe was not executed.

Minimum replacement wording:

> Failed validation or early refusal preserves the native primary, export
> destination and existing recovery bytes. A late native-save I/O failure leaves
> the primary byte-exact and may advance recovery to the exact previous valid
> primary. Unsupported DXF export refuses before staging and leaves destination
> and existing recovery unchanged. Retain the current staged fsync/replace,
> interrupted-save and same-path execute behavior; no atomic replacement of both
> primary and recovery is promised.

The DXF requirement at CONTRACT.md:188–193 remains valid and distinct: existing
`export_dxf` renders before `StagedFile::write` (`persistence.rs:380–384`), so a
shared unsupported-hole guard before staging preserves a sentinel destination.
No runtime or persistence redesign is requested.

## Scope checked without another blocker

- Stable nested Hole ID survives wrap JoinCurve retirement of its first curve.
  The stated last→first fixture and split/join allocation recipes agree with
  current selected-curve-survives semantics. Global ownership/count/high-water,
  loop-aware edits/removal, history and selection/rebuild reach are identified.
- Strict v5 with explicit v4 decoding preserves prior version-specific schemas;
  empty holes in migrations and all Piece constructors are included. Two shared
  variants correctly change current counts 30/29 to 32/31 with nine MCP tools.
- Authored straight-loop eligibility, winding, annotation exclusion and shared
  work limits are separate from meshing limits. Existing outer-only sewing guards
  remain narrow. earcutr 0.5.0 already accepts hole offsets; boundary association,
  collinear recovery, refinement and free-boundary bending require the declared
  annular checks, not another solver or triangulator.
- Independent annular topology/domain checks and actual Cloth mass sums are
  explicit. Stdlib rational arithmetic independently confirms all listed base,
  edited and resized area/mass fractions. The resource fixture needs at least
  4,524,541 faces under the .007m edge bound, correctly demanding bounded refusal.
- Compound SVG, explicit current SVG-import refusal, annular OBJ and unsupported
  pre-staging DXF are coherent. Numeric creation/edit, invalid/cancel shielding,
  restart, history, current-revision mesh and init/reset coverage are included;
  actual native feasibility remains unobserved until a separately frozen run.

All 38 declared source pins and both installed earcutr source pins matched in a
read-only hash check. The proposal's public-manual intent is consistent with the
[hole conversion](https://support.marvelousdesigner.com/hc/en-us/articles/47358440603161-Convert-to-Hole-Internal-Shape),
[internal polygon](https://support.marvelousdesigner.com/hc/en-us/articles/47358276422297-Internal-Polygon-Line)
and [Cut/Cut & Sew](https://support.marvelousdesigner.com/hc/en-us/articles/47358433146393-Cut-Cut-Sew)
manuals; none establishes implementation-specific lifecycle or parity.

Skipped implementation and all native/runtime checks. Risk remains the deliberate
v5 format transition and explicitly unsupported hole import/DXF paths.
