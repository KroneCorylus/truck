import json
import math
from pathlib import Path
import unittest
from i2_trim_bounds import I, coefficients
from i2_endpoint_trace import midpoint
from i2_contact_trace import adapt, jets
from i2_paired_trace_test import support, bezier

EVIDENCE=Path('target/i2-paired-trace-20261105-a')


def outer(uv):
    theta,eta=uv
    rho=(249.99999900004548+eta*(301.0000009999991-249.99999900004548))/10
    z=-(-50.000001+eta*(1.000001+50.000001))/10
    return [rho*math.cos(theta),-rho*math.sin(theta),z]


class ContactControls(unittest.TestCase):
    def test_co_original_chart_order_orientation_and_cs(self):
        c=coefficients(Path('target/i2-representation-budget-20261104/results.json'))
        for r in [.5,1.,1.5]:
            for kind in ['CS','CO']:
                for t in [0.,.5,2.,3.]:
                    p,uv=jets(kind,I(t),r,c)
                    xyz=[midpoint(x[0]) for x in p]
                    for s,values in uv.items():
                        uvpoint=[midpoint(v[0]) for v in values]
                        actual=outer(uvpoint) if s=='O' else support(s,uvpoint,r,c)
                        self.assertLess(sum(abs(x-y) for x,y in zip(actual,xyz)),1e-11)

    def test_adapter_phase_swap_and_axial_scale(self):
        phi,theta=2.8,.04
        contact=3*math.pi/4
        meridian=phi-2*math.pi
        azimuth,section=adapt('A',[meridian,theta],30.,contact)
        self.assertAlmostEqual(azimuth+math.pi/2,theta)
        self.assertAlmostEqual(section+contact,phi)
        self.assertEqual(adapt('C',[3.,.5],30.,contact),[.5,3.])
        self.assertEqual(adapt('B',[.4,24.],30.,contact),[.4,.8])
        # Applying only the swap to A is a concrete wrong-phase control.
        self.assertGreater(abs(theta-(azimuth)),1.)

    def test_contact_evidence_endpoints_and_same_parameter(self):
        c=coefficients(Path('target/i2-representation-budget-20261104/results.json'))
        data=json.loads((EVIDENCE/'candidate-adapters-contacts-shared.json').read_text())
        for radius,case in data['radii'].items():
            r=float(radius)
            label={.5:'r05',1.:'r1',1.5:'r15'}[r]
            physical=json.loads((Path('target/i2-endpoint-trace-20261105')/(label+'.json')).read_text())
            for kind,row in case['contacts'].items():
                traces=row['traces']
                for i,(k,e) in enumerate({'CS':[('BC',0),('CI',0)],'CO':[('DC',1),('CI',1)]}[kind]):
                    self.assertEqual(traces['spatial']['controls'][0 if i==0 else -1][0 if i==0 else -1],physical['curves'][k]['controls'][0 if e==0 else -1][0 if e==0 else -1])
                for s,trace in traces.items():
                    self.assertLess(trace['derivative_remainder'],1e-8)
                    self.assertLess(trace['control_error'],1e-10)
                    self.assertLess(trace['endpoint_charge'],1e-10)
                    if s=='spatial': continue
                    self.assertLess(trace['known_sum'],1e-9)
                    for i in [0,len(trace['controls'])//2,len(trace['controls'])-1]:
                        for f in [0.,2**-40,.5,1-2**-40,1.]:
                            uv=bezier(trace['controls'][i],f)
                            xyz=bezier(traces['spatial']['controls'][i],f)
                            actual=outer(uv) if s=='O' else support(s,uv,r,c)
                            self.assertLess(sum(abs(a-b) for a,b in zip(actual,xyz)),trace['known_sum'])


if __name__=='__main__': unittest.main()
