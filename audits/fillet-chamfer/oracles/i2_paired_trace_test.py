"""Mechanism controls; optional --evidence validates serialized paired traces.

Point checks corroborate chart/order and endpoint construction only. The ledger,
not these samples, supplies the continuous-domain truncation bound.
"""
import argparse
import json
import math
from pathlib import Path
import sys
import unittest
from i2_trim_bounds import I
from i2_endpoint_trace import endpoint, midpoint
from i2_paired_trace import BASE, charts, gate, lipschitz, similarity_charge


def support(s,uv,r,coeff):
    u,v=uv
    a,b,n,m=[midpoint(x) for x in coeff]
    if s=='B': return [v,r*(1-math.cos(u)),r*(1-math.sin(u))]
    if s=='A':
        R=a+n*r+r*math.sqrt(1+n*n)
        return [(R+r*math.cos(u))*math.cos(v),(R+r*math.cos(u))*math.sin(v),r+r*math.sin(u)]
    if s=='C':
        q=b-m*u-r*math.sqrt(1+m*m); h=math.sqrt(q*q-r*r)
        d=math.sqrt((1+m*m)*q*q-r*r)
        return [h+r*h/d*2*v/(1+v*v),r*2*v*v/(1+v*v),u+r*m*q/d*2*v/(1+v*v)]
    if s=='S': return [(250+v)/10,0.,(14.644660940673-u)/10]
    if s=='D':
        o,a,b=BASE
        p=[o[j]+u*(a[j]-o[j])+v*(b[j]-o[j]) for j in range(3)]
        return [(p[0]+200)/10,p[1]/10,p[2]/10]
    if s=='I':
        x=(199.99999900004548+u*(251.0000009999991-199.99999900004548))/10
        z=(-50.000001000000005+u*(1.000001+50.000001000000005)+50)/10
        return [x*math.cos(v),x*math.sin(v),z]
    raise ValueError(s)


def bezier(points,t):
    p=[list(row) for row in points]
    while len(p)>1:
        p=[[(1-t)*a+t*b for a,b in zip(x,y)] for x,y in zip(p,p[1:])]
    return p[0]


class Controls(unittest.TestCase):
    def test_unknown_nonfinite_and_overbudget_reject(self):
        self.assertFalse(gate(8e-11,60))
        for e in [math.nan,math.inf,-1,1e-9]: self.assertFalse(gate(8e-11,60,e))
        self.assertFalse(gate(1.1e-9,0))
        self.assertTrue(gate(8e-11,60,1e-15))

    def test_similarity_terms_scale_and_reject_translation(self):
        for scale in [.1,1,10,100]:
            for translation in [0,1000]:
                self.assertTrue(gate(9e-11,70,1e-15,scale,translation))
                self.assertLess(similarity_charge(scale,translation),1e-9*scale)
            self.assertFalse(gate(9e-11,70,1e-15,scale,1e12))

    def test_l1_chart_constants_cover_actual_derivatives(self):
        # Old CI constants 7.213/30 and canal 1.6/2r fail these L1 controls.
        self.assertGreater(lipschitz('I',1)[0],5.1000002*(1+math.sqrt(2)))
        self.assertGreater(lipschitz('I',1)[1],30*math.sqrt(2))
        self.assertGreater(lipschitz('C',1)[0],2.)
        self.assertGreater(lipschitz('C',1)[1],2*(1+math.sqrt(2)))

    def test_charts_reconstruct_at_same_parameter_and_slivers(self):
        from i2_trim_bounds import coefficients
        coeff=coefficients(Path('target/i2-representation-budget-20261104/results.json'))
        from i2_endpoint_trace import domains
        for r in [.5,1.,1.5]:
            for kind,(lo,hi) in domains(coeff,r).items():
                for f in [0.,2**-40,.3,1-2**-40,1.]:
                    phi=midpoint(lo)+(midpoint(hi)-midpoint(lo))*f
                    xyz=[midpoint(v) for v in endpoint(kind,I(phi),r,coeff)]
                    for s,uv in charts(kind,I(phi),r,coeff).items():
                        actual=support(s,[midpoint(v[0]) for v in uv],r,coeff)
                        self.assertLess(sum(abs(a-b) for a,b in zip(actual,xyz)),1e-11,(kind,s,phi))


def evidence(path):
    from i2_trim_bounds import coefficients
    coeff=coefficients(Path('target/i2-representation-budget-20261104/results.json'))
    summary=[]
    for label in ['r05','r1-final','r15']:
        data=json.loads((path/(label+'.json')).read_text()); r=data['radius']
        source=Path('target/i2-endpoint-trace-20261105')/({'r1-final':'r1'}.get(label,label)+'.json')
        physical=json.loads(source.read_text())
        rows=data['paired_traces']
        assert rows['BC']['C']['controls'][-1][-1]==rows['DC']['C']['controls'][0][0]
        assert rows['AB']['A']['controls'][0][0]==rows['AS']['A']['controls'][-1][-1]
        assert not data['full_admission']
        for kind,paired in rows.items():
            assert len(paired)==2
            for s,row in paired.items():
                assert row['known_sum']<1e-9
                assert row['ledger']['uv_endpoint']<1e-10
                assert row['ledger']['uv_controls']<1e-10
                assert row['ledger']['uv_evaluation']+row['ledger']['support_evaluation_basic']<1e-10
                for i in [0,len(row['controls'])//2,len(row['controls'])-1]:
                    for f in [0.,2**-40,.5,1-2**-40,1.]:
                        uv=bezier(row['controls'][i],f)
                        xyz=bezier(physical['curves'][kind]['controls'][i],f)
                        actual=support(s,uv,r,coeff)
                        assert sum(abs(a-b) for a,b in zip(actual,xyz))<row['known_sum'],(r,kind,s,i,f)
                summary.append({'radius':r,'curve':kind,'support':s,'known_sum':row['known_sum'],
                    'trig_factor':row['runtime_trig_factor'],'required_trig_error':row['required_runtime_trig_abs_error_max']})
    with (path/'ledger.json').open('x') as f: json.dump(summary,f,indent=2)
    print('30 paired ledger rows; same-parameter corroboration and disjoint allocations pass')


if __name__=='__main__':
    parser=argparse.ArgumentParser(); parser.add_argument('--evidence',type=Path)
    args,remaining=parser.parse_known_args()
    if args.evidence: evidence(args.evidence)
    unittest.main(argv=[sys.argv[0]]+remaining)
