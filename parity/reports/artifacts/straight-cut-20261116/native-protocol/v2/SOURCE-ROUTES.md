# Source-only route notes

Candidate `19b1e88245dc627cfec8164f4924b335fe44853d`.
Native `apps/weftforge/src/main.rs` SHA256
`2e8a3f7003ab5ee56c38042162bcdfc325d30428510b878b0d0bf66053e68343`.
This source read supplied labels/routes only, never expected geometry. Root still
verifies all 69 bound source inputs, integrated gates and actual native capability.
No snapshot of implementation geometry or test expected values was used.

- Creation inspector (2607–2611): `Pattern name`, `W / H m` first/second fields,
  `X / Y m` first/second fields, `Add rectangle`.
- View (2860): `Frame all`.
- Piece labels (2517): `<name> · #<id>`, hence `Original Cut R240 · #2` and
  `Cut piece #15 · #15`. Select the ID, not only its name.
- Tool rail/menu/search (2003, 2103, 2136): `Cut piece`; tool options precise
  `Cut piece…` (ellipsis U+2026). The precise action opens `Review cut piece`.
- Review (3233): `Start boundary`, `End boundary`; option label
  `Boundary 1 · curve #7`, `Boundary 3 · curve #9`; selected text `Curve #7`/`Curve #9`.
  Text inputs are `Start t`, `End t`. Buttons are `Apply cut` and `Cancel cut`.
  The review shows captured owner ID/revision, endpoint readouts, cut length,
  retained/new IDs and areas. These readouts supplement saved-state checks.
- Review error text comes from the raw input/preview; native caret retention
  requires the actual `1e-` → Left → refused Apply → insertion0 → `1e0-` witness.
  No numeric helper, alternate focus operation or later output can replace it.
- Two pointer boundary picks have captured-owner semantics and the second opens
  review. Background `Undo` is used only as a backdrop-cancel click during this
  review; root must observe that no actual Undo or allocator change happens.
- Inspector `Size m` (2573) is width then height. CUT-R2 edits the second current
  observed field with literal `0.40`, then one Tab; field name `HEIGHT` in the
  original plan is a semantic locator, not a promised AX label.
- File menu (1982, 1986): `Save`, `Save as…`. OS dialog Name/Save capabilities
  are runtime observations, not inferred from source.

All click coordinates/AX indices come from a fresh retained observation. There is
no drag, titlebar-origin correction, unplanned navigation, retry or fallback route.
