"""Procedural tests: analytic roots, singular and boundary rejection, knot insertion."""
import unittest
from c1_contact_isolation import isolate, spans, value


class ContactIsolation(unittest.TestCase):
    def test_two_simple_contacts(self):
        # S=(u,v,0), G=0, G'=(1,0,0): u=0, v=+-1.
        net=[[[u,v,0] for v in (-2,3)] for u in (-2,3)]
        r=isolate(net,(-2,3,-2,3),(0,0,0),(1,0,0),1)
        self.assertTrue(r['complete_at_station'])
        self.assertEqual(len(r['roots']),2)
        for root in r['roots']:
            a,b,c,d=root['box']
            self.assertLess(a,0);self.assertGreater(b,0)
            self.assertTrue(c < 1 < d or c < -1 < d)
            self.assertLess(root['contraction'],1)

    def test_no_contact_in_original_chart(self):
        net=[[[u,v,0] for v in (0,2)] for u in (1,3)]
        r=isolate(net,(1,3,0,2),(0,0,0),(1,0,0),1)
        self.assertTrue(r['complete_at_station'])
        self.assertEqual(r['roots'],[])

    def test_boundary_root_is_unresolved(self):
        net=[[[u,v,0] for v in (0,2)] for u in (0,2)]
        r=isolate(net,(0,2,0,2),(0,0,0),(1,0,0),1,max_depth=12)
        self.assertFalse(r['complete_at_station'])
        self.assertEqual(r['roots'],[])

    def test_singular_contact_is_unresolved(self):
        net=[[[u,v,0] for v in (-1,1)] for u in (-1,1)]
        r=isolate(net,(-1,1,-1,1),(0,0,0),(1,0,0),0,max_depth=12)
        self.assertFalse(r['complete_at_station'])
        self.assertEqual(r['roots'],[])

    def test_budget_exhaustion_is_unresolved(self):
        net=[[[u,v,0] for v in (-2,3)] for u in (-2,3)]
        r=isolate(net,(-2,3,-2,3),(0,0,0),(1,0,0),1,max_boxes=1)
        self.assertFalse(r['complete_at_station'])
        self.assertEqual(r['roots'],[])

    def test_nonfinite_input_is_unresolved(self):
        net=[[[float('nan'),0,0],[0,1,0]],[[1,0,0],[1,1,0]]]
        r=isolate(net,(0,1,0,1),(0,0,0),(1,0,0),1)
        self.assertFalse(r['complete_at_station'])

    def test_knot_insertion_preserves_quadratic(self):
        # Quadratic B-spline with simple knot .5, checked by explicit basis values.
        s=spans([0,0,0,.5,1,1,1],[[0],[1],[3],[4]],2)
        self.assertEqual(len(s),2)
        self.assertEqual(value(s[0][2],1),[2])
        self.assertEqual(value(s[1][2],0),[2])
        self.assertEqual(value(s[0][2],.5),[1])
        self.assertEqual(value(s[1][2],.5),[3])


if __name__=='__main__': unittest.main()
