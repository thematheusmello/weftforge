import pathlib
import tempfile
import unittest

import original_obj_boundary
import original_obj_bounds


class BoundaryMeasurementTests(unittest.TestCase):
    def measure(self, vertices, faces):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "original-test-rectangle.obj"
            path.write_text("\n".join(
                ["v " + " ".join(map(str, vertex)) for vertex in vertices]
                + ["f " + " ".join(str(index + 1) for index in face) for face in faces]
            ) + "\n")
            bounds = original_obj_bounds.measure(path, "mm", [200, 300])
            boundary = original_obj_boundary.measure_boundary(path, "mm", [200, 300])
            return bounds, boundary

    def test_original_rectangle_at_fixed_physical_scale(self):
        _, result = self.measure(
            [(0, 0, 0), (200, 0, 0), (200, 300, 0), (0, 300, 0)],
            [(0, 1, 2), (0, 2, 3)],
        )
        self.assertEqual(result["status"], "passed")
        self.assertEqual(result["boundary_edges"], 4)
        self.assertEqual(result["projected_triangle_area_mm2"], 60000)

    def test_interior_hole_keeps_bounds_but_fails_boundary(self):
        vertices = [(x * 200 / 3, y * 100, 0) for y in range(4) for x in range(4)]
        faces = []
        for y in range(3):
            for x in range(3):
                if (x, y) == (1, 1):
                    continue
                a = 4 * y + x
                faces.extend([(a, a + 1, a + 5), (a, a + 5, a + 4)])
        bounds, result = self.measure(vertices, faces)
        self.assertEqual(bounds["status"], "passed")
        self.assertEqual(result["status"], "failed")
        self.assertEqual(result["boundary_components"], 2)
        self.assertFalse(result["checks"]["boundary_on_declared_rectangle"])

    def test_reversed_triangle_keeps_bounds_but_fails_winding(self):
        bounds, result = self.measure(
            [(0, 0, 0), (200, 0, 0), (200, 300, 0), (0, 300, 0)],
            [(0, 1, 2), (0, 3, 2)],
        )
        self.assertEqual(bounds["status"], "passed")
        self.assertEqual(result["status"], "failed")
        self.assertFalse(result["checks"]["consistent_nondegenerate_projected_winding"])


if __name__ == "__main__":
    unittest.main()
