"""Whole-input oracle closure, including the hole and curved boundary pieces."""
import itertools
import unittest
import numpy as np
from a5_assembly_material import build, crossings, flux, incidence, surface_witnesses


class WholeAssembly(unittest.TestCase):
    def test_closed_material_and_complete_ray_partitions(self):
        for mode in ['equal_dist', 'dist_dist', 'dist_angle']:
            with self.subTest(mode=mode):
                model = build(mode)
                self.assertEqual(incidence(model)['bad_edges'], [])
                points = list(itertools.product(
                    np.linspace(-.399173, .419827, 19),
                    np.linspace(-.349173, .749827, 23),
                    np.linspace(-.549173, .549827, 23)))
                side_points, sides = surface_witnesses(model)
                points.extend(side_points)
                a, bad, invalid = crossings(model, points, [1., .371139, .197531])
                b, bad2, invalid2 = crossings(model, points, [1., -.271931, .431719])
                self.assertFalse(any(bad | bad2))
                self.assertFalse(any(invalid | invalid2))
                np.testing.assert_array_equal(a, b)
                self.assertTrue(all(np.isin(a, [0, 1])))
                np.testing.assert_array_equal(a[-len(sides):], sides)
                self.assertGreater(flux(model)[0], 0.)


if __name__ == '__main__':
    unittest.main()
