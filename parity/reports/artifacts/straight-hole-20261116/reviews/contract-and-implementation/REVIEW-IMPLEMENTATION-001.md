What this change does: Adds one stable, editable straight Hole per piece through
the shared Rust command engine and strict native v5 format. Meshing uses the
existing earcut hole offsets; SVG/OBJ preserve the opening and unsupported DXF
export refuses before writing.

**No concrete Must fix or Should fix finding in the reviewed core + IO scope.**
Review compared the working tree `.local/straight-holes-20261116` with base
`982ad0340d3c848bb8a959dca7a934886f1d7ae9` and the approved contract SHA256
`dc6b66c6d278ec25165979c2a2c729b4a871f8798baa1f7fe53a5d7299dd8797`.
Assumed use is one local author within the existing document and mesh caps.

Reviewed the diff, connected callers and meaningful tests:

- `document.rs:170–202,439–645,893–971,1035–1128,1274–1413,1586–1631,2036–2042`:
  stable nested Hole IDs; global ownership/object/high-water checks; first-vertex
  winding normalization; preflight before allocation; loop-aware split, wrap join,
  resize, transform and removal. The selected join curve survives while only the
  next curve/common point retire. Snapshot history retains allocator high-water.
  Command work and final validation share one containment budget; empty/nested/
  excessive typed transactions refuse without committing the candidate.
- `geometry.rs:210–289,650–854,1148–1237`: raw straight domain/winding/clearance and
  annotation exclusion stay independent of mesh resolution. Flattened outer/hole
  offsets retain each curve's endpoint associations. Collinear repair splits all
  faces sharing the missing contour edge and verifies authored neighbor edges.
  Existing refinement/resource caps remain. Read `holes.rs:133–330`: triangle
  clipping and overlap checks, full associated-edge equality, one connected
  component/two closed boundaries/Euler zero, actual Cloth mass and reset, at all
  three declared resolutions. Resource refusal preserves the editable document.
- Sewing remains restricted to exterior curves in creation, update, load, length
  sewing and group metrics. The new `seam_lengths` guard does not narrow generic
  interval measurement, which still measures hole/annotation curves. Unrelated
  sewing survives hole edits/split/join/removal; no solver change was introduced.
- Strict `ProjectV4/PieceV4` dispatch preserves old schemas, IDs and data while
  adding empty holes; v5 requires Hole/holes fields and rejects malformed ownership,
  duplicate/unknown fields and wrong winding. Save still stages recovery before
  primary replacement, matching the approved late-failure wording. SVG uses one
  compound evenodd path, current import refuses it explicitly, and DXF checks holes
  before staging. IO tests retain primary/recovery/sentinel bytes.
- CLI/MCP use the shared variants, advertise the closed 31-alternative command
  schema and retain nine tools. Tests exercise original-byte duplicate/refusal
  handling, required fields, IDs, coordinate/count bounds, same-path rollback,
  migrations, export descriptions and the current writer-version assertion.

**Verdict: Ship the reviewed core/IO scope after the required local gates.** This
is a static code-review result only; it is not integrated feature acceptance.

Current SHA256 snapshot (paths relative to the hole worktree):

```text
4ce5ea5d3919e4fa180d723558ad30975ce2158f68772bc3c151e86857bb05f0  crates/weftforge-core/src/document.rs
d4ba435fe2fbcd907c97a01b16be2b8bcfa934a1e3cbb4882fc75ab43f380105  crates/weftforge-core/src/geometry.rs
42c3d163bf347af1dcf3bb007ac15c7ba3e4836c829b18904592cb96b6cd9812  crates/weftforge-core/src/persistence.rs
d45cdaaf7f22359cdc22d32de64edc05a5f14a1b8c227899f95f6b48fd7683f7  crates/weftforge-core/src/persistence/svg.rs
7049615180a793ad1557a2bbd3a27797b4b06aac060a2cb0620bd40436e64e0b  apps/weftforge-cli/src/main.rs
34af8f549de7eac7f025089fddb8c1d798fbd1e65b85d5fce74de473ba72c7cb  apps/weftforge-cli/src/mcp.rs
eaf21904ea94858632e6d11caddda63c0bf220845788633481c3bd114a355594  crates/weftforge-core/tests/holes.rs
dfe604ad08e660e9feecb88d99153eee78bb1cadcbb8307350ad9983956025a2  crates/weftforge-core/tests/hole_io.rs
1f4b8aec8bfa9622c7c69b2b7ef2a9ec2c745340c5d56c5edab87d40d7f66418  crates/weftforge-core/tests/migration.rs
c5b673f0a69a43086c3a6d5b7aa58e4b68bcf061637f57c4731a83af4879f018  crates/weftforge-core/tests/cut_piece.rs
a3e9ad0393b0039b7257193b5bcd0f54e7aafe5ab5566ae8459c1392a0ce6f41  crates/weftforge-core/tests/sewing_groups.rs
c7e0656dcb756db3ced588e52c08c969a997b99661f85bbb8d5079a27421ad77  apps/weftforge-cli/tests/holes.rs
23f3d0d1ee01d86dfb426e2a89b514a399ba53b51159dedbf932b18ce93753f4  apps/weftforge-cli/tests/mcp_stdio.rs
b68442ce95e3207bcfb34745e2c653eddaea1ce65fa8feda56321bbcc9d415ed  crates/weftforge-core/src/sewing.rs
73ee0a5d1c6d5b58c70a9ccac74898e12e4ffb51bfe2e3f2002b339912715f29  crates/weftforge-core/src/simulation.rs
5bfc996e6bf1067ac261358fdd043815adf304a5d8d512a0e6429c913e90fa76  crates/weftforge-sim/src/lib.rs
```

Not checked: Cargo/tests/evaluators were not run; active native main.rs integration,
GUI behavior and reference runtime were excluded. No native/reference acceptance
or parity claim follows from this review; later edits require fresh review/checks.
