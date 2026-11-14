"""Physical-scale counterexamples for the independent observation evaluator."""
import pathlib
import tempfile
import unittest

from original_obj_bounds import measure


class OriginalOBJBoundsTest(unittest.TestCase):
    def evaluate(self, width, height):
        with tempfile.TemporaryDirectory() as folder:
            path = pathlib.Path(folder) / "original-rectangle.obj"
            path.write_text(
                f"v 0 0 0\nv {width} 0 0\nv {width} {height} 0\nv 0 {height} 0\n"
                "f 1 2 3\nf 1 3 4\n"
            )
            return measure(path, "mm", [200.0, 300.0])

    def test_known_original_dimensions(self):
        result = self.evaluate(200, 300)
        self.assertEqual(result["status"], "passed")
        self.assertEqual(result["convex_hull_perimeter_mm"], 1000)

    def test_one_mm_physical_width_is_rejected(self):
        result = self.evaluate(1, 300)
        self.assertEqual(result["status"], "failed")
        self.assertGreater(max(result["extent_errors_mm"]), 100)

    def test_nonfinite_coordinates_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "nonfinite"):
            self.evaluate("nan", 300)


if __name__ == "__main__":
    unittest.main()
