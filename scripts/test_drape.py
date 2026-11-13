import unittest
from check_drape import strain
class IndependentStrainTests(unittest.TestCase):
    def test_rigid_rotation_and_known_stretch(self):
        uv=[[0,0],[1,0],[0,1]]
        self.assertAlmostEqual(strain([[0,0,0],[0,0,1],[0,1,0]],uv),0)
        self.assertAlmostEqual(strain([[0,0,0],[1.1,0,0],[0,1,0]],uv),0.1)
    def test_degenerate_uv_rejected(self):
        with self.assertRaises(AssertionError):strain([[0,0,0],[1,0,0],[0,1,0]],[[0,0],[1,0],[2,0]])
