# FIT-002 pattern visibility

The Scene dock lets the user inspect individual garment pieces in both views.
**Hide selected** removes the selected pattern from the two canvases and clears
its selection. **Isolate selected** keeps that pattern selected and hides the
other current patterns. **Show all** restores the visible set. Scene keeps each
hidden pattern’s labeled row.

Hidden patterns are excluded from canvas picking, new sewing choices and
**Frame all**. Applied sewing guides that connect to a hidden pattern are
also hidden. Frame all fits visible outlines in the 2D canvas. The avatar
stays visible.

The visibility set belongs to the open desktop document session. Saving,
mesh export and simulation include every pattern piece. An active construction
gesture keeps its inputs and prevents Hide or Isolate. Hiding a selection with
an unresolved property edit requires the edit to be applied or canceled first.
New, successfully opened or recovered documents start with every pattern visible.
A failed load retains the existing view. Newly added patterns start visible.

For the bounded implementation check, open a two-pattern original document.
Inspect both canvases and the Scene selection before and after Hide, Isolate and
Show all. Check hidden picking and the frame extent. Compare the editable
document, simulation identity and exported geometry across the display changes.
Pins, cloth grabbing, layer controls, avatar visibility, per-view visibility
and reference behavior remain open.
