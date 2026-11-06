"""Independent regression controls for asymmetric cap layers and oracle scaling."""
import copy
import json
import math
from pathlib import Path
import unittest
from c2_unequal_material import inside, section, volume_bounds

FIXTURES = json.loads(Path(__file__).with_name('c2_unequal_fixture.json').read_text())

class MaterialTests(unittest.TestCase):
    def test_halfway_crossing_and_reflection(self):
        p = copy.deepcopy(FIXTURES['dist_dist']['parameters'])
        p['length'] = 1.5
        for reflect in (False, True):
            if reflect:
                p['setbacks'].reverse()
            z = 1.5-.72 if reflect else .72
            self.assertFalse(inside(p, (17.5, 4.045, z)))
            self.assertTrue(inside(p, (17.5, 4.03, z)))
            self.assertAlmostEqual(section(p, 0, z)[3], 4.03806587080493)

    def test_volume_reflection(self):
        for fixture in FIXTURES.values():
            p = copy.deepcopy(fixture['parameters'])
            p['setbacks'].reverse()
            lo, hi, _ = volume_bounds(p, 4096)
            self.assertLess(lo, fixture['volume_midpoint'])
            self.assertGreater(hi, fixture['volume_midpoint'])

    def test_scale_allowance(self):
        p = copy.deepcopy(FIXTURES['dist_dist']['parameters'])
        lo, hi, allowance = volume_bounds(p, 1024)
        for factor in (.1, 10.):
            q = {k: [v[0]*factor, v[1]*factor] if k == 'center' else
                 [[[u*factor,w*factor] for u,w in cap] for cap in v] if k == 'setbacks' else v*factor
                 for k,v in p.items()}
            a, b, scaled = volume_bounds(q, 1024)
            self.assertLess(a, hi*factor**3)
            self.assertGreater(b, lo*factor**3)
            self.assertTrue(.5*factor**3 < scaled/allowance < 2*factor**3)

if __name__ == '__main__':
    unittest.main()
