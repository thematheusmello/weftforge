# FILE-005 — Bounded DXF export-route inspection

Case `REF-FILE-005-DXF-005` inspects Marvelous Designer Personal
**2026.1.237 (r58920)**, executable `e7696388…c76e`, on the saved original
reversed partial-sewing fixture. Baseline revision 1 remains provisional and
unfrozen. The [structured record](../observations/REF-FILE-005-DXF-005.json)
pins the evidence and keeps the entry gap separate from workspace access.

The declared task was to inspect `File > Export > DXF` options and cancel,
without exporting or changing document/preferences. **No DXF entry appears in
the exposed current File > Export AX tree. No DXF dialog is opened.** This is a
finding about this route, build, fixture and observed state; it does not establish
product-wide format support, an edition/license explanation or alternate routes.

## Observed route and boundary

Opening File exposes these 16 Export child labels:

- OBJ; OBJ (Selected); FBX.
- glTF 2.0 (GLTF); glTF 2.0 (GLB); Alembic; USD; VRM.
- Animation; Maya Cache (MC); Maya Cache (MCX); Point Cache 2.
- MDD Cache (Standard); MDD Cache (Maya/3ds Max); OBJ Sequence.
- Bone Information (JSON).

Animation has an exposed disabled marker. Other entries have no disabled marker;
this is not an independent enabled-state or behavior qualification. The labels
match the existing catalog's Export children exactly. They include headings or
cache actions and are not a count of verified format implementations.

The live File record contains AX text only; no menu screenshot is returned.
Clicking the observed Export parent returns the standard workspace instead of
leaving a visual Export popup open. That actual after-state is retained. No format
item is invoked, no dialog is inspected, and no DXF cancellation behavior is
claimed. No speculative hover or alternate command is substituted.

DXF tabs, options, units, current/default values, ranges, errors and output
semantics all remain **unobserved**. The full WeftForge DXF target is unchanged;
this case supplies no comparative DXF options or output oracle. The frozen
949-record catalog remains byte-identical, with zero new IDs.

## Preservation and provenance

The protected original native fixture remains 100,554 bytes, SHA-256
`0afcfe4281526e2250bef6b6d945bdc7e48f388b8236c765623bafb17702ba8d`,
before and after. The selected original reversed pair and its property values
remain visible in Edit Sewing, with both 100.0 labels and uncrossed guides.
Simulation stays off; no menu, dialog or unfinished gesture remains at handoff.
The native container stays private and is not decoded.

| Stage | Retained evidence |
| --- | --- |
| Before menu inspection | `ev_22fdf5e2c6ccbd6a31f9a0f20ab38d3a26cb04001828f03da5fb8d57e7a86ac6` |
| Live File AX tree | `CUA-DXF005-LIVE-FILE-MENU` |
| Workspace after Export-parent click | `CUA-DXF005-AFTER-EXPORT-PARENT` |
| Final workspace | `ev_43eafac4eeb88048db8360bfdedb6bffb47aba7c8db83e611cae041ee0365069` |

The before/after REA observations each have 142 nodes, no truncation and no gaps.
The actual kernel executable path/hash is checked before and after. Raw
commercial/account UI, recent personal paths and native files remain private;
the public record contains reviewed labels, states, hashes and limits only.

No activation, native pointer fallback, preference, account, permission, TCC or
WeftForge action is performed. No factory-default, format-output, physics,
complete broad feature contract or verified parity claim follows from this case.
