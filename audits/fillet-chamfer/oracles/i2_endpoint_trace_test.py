"""Focused controls for physical endpoint and restricted evaluator construction."""
import math
import unittest
from i2_trim_bounds import I, trig
from i2_ci_uv import charts
from i2_canal_controls import construct as canal_controls
from i2_endpoint_trace import basis_error, domains, endpoint, hull, inverse_cos, midpoint, error


class PhysicalEndpoints(unittest.TestCase):
    def setUp(self):
        self.coeff = tuple(I(v) for v in [20,30,1,1])

    def test_inverse_cos_contains_reference(self):
        for x in [-.05,0,.7]:
            box = inverse_cos(I(x))
            self.assertLessEqual(box.a,math.acos(x))
            self.assertGreaterEqual(box.b,math.acos(x))
        with self.assertRaises(ValueError):
            inverse_cos(I(-2))

    def test_actual_residual_domains_not_extended_substitutes(self):
        for r in [.5,1.,1.5]:
            d = domains(self.coeff,r)
            self.assertLess(d['DC'][0].b,d['DC'][1].a)
            self.assertLess(d['DC'][1].b,d['CI'][1].a)
            self.assertLess(d['CI'][1].b,1.63)
            self.assertLessEqual(d['AS'][0].a,3*math.pi/4)
            self.assertGreaterEqual(d['AS'][0].b,3*math.pi/4)
            for group in [[('BC',1),('DC',0)],[('AB',0),('AS',1)]]:
                box = hull([endpoint(k,d[k][i],r,self.coeff) for k,i in group])
                self.assertLess(error([midpoint(v) for v in box],box),1e-10)

    def test_physical_remote_endpoint_on_ridge(self):
        for r in [.5,1.,1.5]:
            p = endpoint('CI',domains(self.coeff,r)['CI'][1],r,self.coeff)
            self.assertLessEqual(p[2].a,5)
            self.assertGreaterEqual(p[2].b,5)
            radial = (p[0]*p[0]+p[1]*p[1]).sqrt()
            self.assertLessEqual(radial.a,25)
            self.assertGreaterEqual(radial.b,25)

    def test_ci_chart_order_and_rational_angle(self):
        for phi in [0.,.4,1.59]:
            uv = charts(I(phi),1.,self.coeff)
            p = endpoint('CI',I(phi),1.,self.coeff)
            eta,theta = uv['inner_cone_eta_theta']
            expected_eta = (midpoint(p[2])+1.000000005e-7)/5.1000002
            expected_theta = math.atan2(midpoint(p[1]),midpoint(p[0]))
            for box,expected in [(eta[0],expected_eta),(theta[0],expected_theta),
                                 (uv['canal_T_tan_half_phi'][1][0],math.tan(phi/2))]:
                self.assertLessEqual(box.a,expected)
                self.assertGreaterEqual(box.b,expected)
            if phi > 0:
                self.assertNotAlmostEqual(midpoint(uv['canal_T_tan_half_phi'][1][0]),phi)

    def test_tensor_canal_uses_rational_half_angle(self):
        c = canal_controls(self.coeff,{'radius':1.})
        self.assertTrue(c['evaluation_budget_pass'])
        self.assertEqual(c['weight_lower'],1.)
        self.assertLess(c['projected_control_enclosure'],1e-10)
        self.assertLess(c['fit_truncation'],2e-9)
        from i2_trim_bounds import canal, const
        for local,s in [(.2,.3),(.7,1.05)]:
            b = [(1-local)**3,3*local*(1-local)**2,3*local*local*(1-local),local**3]
            v = s/1.1
            q = [(1-v)**2,2*v*(1-v),v*v]
            H = [sum(c['control_net'][i][j][k]*b[i]*q[j] for i in range(4) for j in range(3)) for k in range(4)]
            point = [x/H[3] for x in H[:3]]
            expected = canal(const(-1.5+.05*local),I(2*math.atan(s)),1.,self.coeff)
            self.assertLess(sum(abs(x-midpoint(y[0])) for x,y in zip(point,expected)),1e-11)

    def test_basis_bound_is_restricted_and_budgeted(self):
        self.assertLess(basis_error(40),1e-10)
        self.assertGreater(basis_error(1e6),1e-10)
        for v in [-1,math.inf,math.nan]:
            with self.assertRaises(ValueError):
                basis_error(v)
        with self.assertRaises(ValueError):
            basis_error(30,4)


if __name__ == '__main__':
    unittest.main()
