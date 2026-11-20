# Projected garment faces

At any viewing angle, a solid garment face fills the triangle between its
three projected vertices. A face viewed edge-on may cover less than a pixel.
Both windings use the material color and the face's existing lighting.
The studio draws from back to front, drawing a selected face's wire edges
immediately after its fill so that nearer faces can cover them. Picking uses
the same projected triangles and depth order.

In the native 60-step assembly and 120-step drape of Harbor with a 50 mm
front hem extension, OBJ faces 192 and 1067 appeared as blue lines beyond the
cloth. The projected vertices of face 192 span 9.3 points vertically. The
previous filled-path tessellator expanded its acute corner across more than
12,000 points. The studio now submits the three projected vertices directly
as an indexed triangle. The regression test exercises both faces, both
windings, and pixel densities of one and two.
