import copy
import unittest
from c2_extrusion_material import PARAMETERS, area, contacts, inside, roof, section, volume_bounds


class ClosedMaterial(unittest.TestCase):
    def test_whole_volume_bracket(self):
        lo, hi, rounding = volume_bounds(PARAMETERS)
        self.assertLess(lo, 138.66070708439852)
        self.assertGreater(hi, 138.66070708439852)
        self.assertLess(hi-lo, 0.0001)
        self.assertGreater(rounding, 0.)
        self.assertAlmostEqual(area(PARAMETERS, 0, 1.)*PARAMETERS['length'], 141.91202608870574, places=10)

    def test_material_sides_all_branches(self):
        p = PARAMETERS
        for cap in range(2):
            for t in [.001, .1, .25, .35, .45, .55, .65]:
                a, b, low, high, flat, ro, ri, q, s = section(p, cap, t)
                for left, right in [(a, q), (q, s), (s, p['center'][0]), (p['center'][0], b)]:
                    x = (left+right)/2
                    y = roof(p, x, cap, t)
                    z = p['length']-t if cap else t
                    self.assertTrue(inside(p, [x, y-.0001, z]))
                    self.assertFalse(inside(p, [x, y+.0001, z]))
        self.assertFalse(inside(p, [20, 1, -.001]))
        self.assertFalse(inside(p, [20, 1, p['length']+.001]))

    def test_conic_is_not_endpoint_chord(self):
        for cap, chain in [(0, .4), (1, .5)]:
            for lo, hi in [(0., chain), (chain, .6)]:
                p, q = [contacts(PARAMETERS, cap, t)[2] for t in [lo, hi]]
                middle = contacts(PARAMETERS, cap, (lo+hi)/2)[2]
                self.assertGreater(abs(middle[0]-(p[0]+q[0])/2), 1e-5)

    def test_independent_cap_setbacks(self):
        p = copy.deepcopy(PARAMETERS)
        p['setbacks'][0][4] += .01
        self.assertNotEqual(area(p, 0, .1), area(PARAMETERS, 0, .1))
        self.assertEqual(area(p, 1, .1), area(PARAMETERS, 1, .1))


if __name__ == '__main__':
    unittest.main()
