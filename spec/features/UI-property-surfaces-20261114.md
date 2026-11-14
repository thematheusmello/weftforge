# Observed property and dialog surfaces

This review adds 105 component observations from existing private captures of
Marvelous Designer Personal **2026.1.237 (r58920)**. It performs no new UI action.
The executable remains the recorded `e7696388…c76e` reference and baseline revision
1 remains provisional. These observations establish control presence and values
in the stated contexts. They do not identify the solver implementation or verify
WeftForge parity.

The [component catalog](../inventory/md-2026.1.237-components.json) contains the
individual stable IDs, label paths, values, enabled-state evidence and unknown
ranges/error behavior. Its source table pins the complete private records by
SHA-256. Raw commercial UI, account text, paths and material resources remain
private.

| Surface | Added records | Observed context |
| --- | ---: | --- |
| Simulation Properties | 24 | Empty project, simulation off, fields read without changing settings |
| Selected pattern | 31 | One original 200 by 300 mm rectangle, unsimulated, full boundary selected |
| Create Rectangle | 8 | Immediately before accepting the original 200 by 300 mm constructor |
| Export OBJ | 29 | Reopened original rectangle, options unchanged, one actual export |
| Fit Properties | 13 | Stress page only, no simulated cloth or computed stress |

Simulation Properties displays Normal (Default), Time Step `0.033333`, one
simulation per frame, CG finish condition Iteration, 50 CG iterations, residual
`1e-04`, one self-collision iteration, Air Damping `1`, and Gravity `-9800`.
These controls do not print their units in the retained observation; no SI
conversion or solver equation is inferred. Automatic Maximum CPU is on and the
CPU count is disabled with value 12 on this host. Twelve must not become a
fixed application default. Ground is collapsed, so its child controls remain
unobserved.

The selected pattern exposes grain direction, particle distance, layer,
directional shrinkage, collision and rendering thickness, pressure, mesh type,
side geometry and UV-expansion controls. Particle Distance is explicitly **mm**;
the separately labeled Side Geometry Resolution must not be equated with it.
The grain-direction and pressure fields have no printed unit in this capture.
The catalog does not infer a wrap range or pressure scale. Name and mesh counts
are values of this original fixture, not constants for every new pattern.
Native counts 290 vertices and 525 faces agree with the independently parsed
original OBJ in the [rectangle run](PAT-003-runtime-20261114.md).

The rectangle catalog records **200 × 300 mm**, interval 300 mm, angle 90 degrees,
one shape, and the dimension link off immediately before OK. Those values must
not be confused with the earlier constructor defaults recorded in the rectangle
run. Only the width/height keyboard entries and one OK creation were executed.
Linked resizing, repetition, constructor Cancel and invalid numeric input remain
untested.

The export image visibly says **Export OBJ**, although its AX dialog title says
**Import OBJ**. The actual menu route was `File > Export > OBJ`. Selected unit
**mm (Default)** and 100% scale are independent input to the physical-size
evaluator; they are not inferred by fitting output bounds. Single/Multiple
Objects, Unweld/Weld/workspace-selected welding and Thin/Thick choices are
visible, but their alternate effects remain untested. Texture destination and
preset controls are cataloged without disclosing paths or vendor preset data.

Fit Properties visibly exposes **Stress** and **Pressure** tabs. Stress is the
displayed page, with eight colors, kPa, displayed range 0–100, Gradation, Mesh,
and Weft/Warp/Shear checked. An attempted Strain selection left the Stress page
unchanged. A Strain tab or its defaults are therefore unobserved in this state.
The separately observed `Display > Garment Fit Maps > Strain Map` menu command
remains in the menu catalog; this modal observation does not settle that map's
behavior.
Table cells being marked settable by AX does not prove that editing them works.
No stress values, material calibration, color palette or fit threshold are
derived from this empty-scene display.

Before implementing or qualifying these behaviors, test the relevant edit,
commit/Cancel, Undo/Redo, invalid-input and persistence cases. Preserve physical
units instead of adapting expected geometry to exported bounds. Keep controls
with unknown units or collapsed children explicitly unknown until a suitable
observation resolves them. There are zero complete broad feature contracts and
zero verified WeftForge comparisons in this surface inventory.
