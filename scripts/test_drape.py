import math
import unittest
from check_drape import strain
class IndependentStrainTests(unittest.TestCase):
    def test_rigid_rotation_and_known_stretch(self):
        uv=[[0,0],[1,0],[0,1]]
        self.assertAlmostEqual(strain([[0,0,0],[0,0,1],[0,1,0]],uv),0)
        self.assertAlmostEqual(strain([[0,0,0],[1.1,0,0],[0,1,0]],uv),0.1)
    def test_degenerate_uv_rejected(self):
        with self.assertRaises(AssertionError):strain([[0,0,0],[1,0,0],[0,1,0]],[[0,0],[1,0],[2,0]])

    def test_known_deformation_in_45_degree_material_axes(self):
        uv=[[0,0],[1,0],[0,1]]
        world=[[0,0,0],[1.2,0,0],[0,1,0]]
        # A unit diagonal fiber becomes (1.2/sqrt(2),1/sqrt(2)); both rotated axes have this length.
        expected=math.sqrt((1.2**2+1)/2)-1
        self.assertAlmostEqual(strain(world,uv,math.pi/4),expected)
        self.assertAlmostEqual(strain(world,uv),0.2)
        self.assertAlmostEqual(strain(world,uv,0.),strain(world,uv))

    def test_non_finite_grain_rejected(self):
        with self.assertRaises(AssertionError):
            strain([[0,0,0],[1,0,0],[0,1,0]],[[0,0],[1,0],[0,1]],math.nan)
