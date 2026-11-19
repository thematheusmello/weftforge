# UI-001 frame the 3D garment

**Frame garment** in View and Command search centers and fits the visible cloth
in the 3D canvas at the current viewing angle. It uses the latest accepted
simulation positions when present and the rest mesh otherwise. Hidden pieces
stay hidden. The camera target becomes the center of the framed view, so mouse
orbit continues around the garment.

The action uses the current viewport size and reserves space around the cloth.
It is unavailable when there is no current visible surface or canvas to frame.
**Frame all** continues to fit the visible 2D patterns. Garment framing preserves
the 2D camera, pending inspector edits, selection, and the editable and simulated
states. A new, opened, or recovered project starts with the original orbit target.

Check the fitted local bounds against the live viewport after a resize. Repeat
with a hidden piece and an accepted simulation state. Capture the native 3D view
before and after framing. The 2D pixels, editable bytes, and complete export
must match the equivalent scene before framing. Reference focus, camera pan,
and custom views remain open.
