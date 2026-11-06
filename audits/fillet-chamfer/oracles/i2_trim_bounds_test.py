"""Rejection and algebra controls for the I2 mathematical probe."""
import math
import unittest
from i2_trim_bounds import I, const, implicit, inv, mul, root_box, run, sqrt, trig


class TrimBounds(unittest.TestCase):
    def setUp(self):
        self.coefficients = tuple(I(x) for x in [20,30,1,1])

    def test_unknown_inverse_and_nonfinite_are_rejected(self):
        for box in [I(-1,1), I(0)]:
            with self.assertRaises(ValueError):
                I(1)/box
        with self.assertRaises(ValueError):
            I(math.inf)
        with self.assertRaises(ValueError):
            sqrt(const(-1))

    def test_trig_endpoint_extrema_are_enclosed(self):
        for u in [0, math.pi/2, math.pi]:
            for cosine in [False,True]:
                box = trig(I(u),cosine)
                reference = math.cos(u) if cosine else math.sin(u)
                self.assertLessEqual(box.a,reference)
                self.assertGreaterEqual(box.b,reference)
        with self.assertRaises(ValueError):
            trig(I(-1,1))

    def test_jet_inverse_preserves_identity(self):
        a = [I(2),I(1),I(0),I(0),I(0)]
        product = mul(a,inv(a))
        for j,v in enumerate(product):
            expected = 1 if j == 0 else 0
            self.assertLessEqual(v.a,expected)
            self.assertGreaterEqual(v.b,expected)

    def test_endpoint_equality_and_unsquared_bracket(self):
        for kind in ['BC','DC']:
            _,_,denom = implicit(I(0),1,self.coefficients,kind)
            self.assertLessEqual(denom.a,1)
            self.assertGreaterEqual(denom.b,1)
        box = root_box(I(0),1,self.coefficients,'BC')
        self.assertLessEqual(box.a,1)
        self.assertGreaterEqual(box.b,1)
        with self.assertRaises(ValueError):
            root_box(I(.3),1,tuple(I(x) for x in [200,30,1,1]),'CI')

    def test_unknown_stationary_geometry_is_not_admitted(self):
        with self.assertRaises(ValueError):
            run(tuple(I(x) for x in [20,30,.1,1]),1)


if __name__ == '__main__':
    unittest.main()
