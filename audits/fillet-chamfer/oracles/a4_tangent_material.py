"""Independent bounded cone/plane material and rounded-rectangle volume oracle.
Usage: python3 a4_tangent_material.py REPLAY_DIRECTORY CLASSIFIER OUTPUT_JSON
Classifier protocol: a6_preference_material.cpp, BREP/STEP path and 'batch'.
"""
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys

root, reader, output = map(Path, sys.argv[1:])
records = {}
for mode, b in [('equal_dist', .03), ('dist_dist', .04), ('dist_angle', .03*math.sqrt(3))]:
    a, radius = .03, .1
    origin = [-.243878903266, .604420820973, .5]
    n = [.004092135617692, .999991627177991, 0.]
    t = [-n[1], n[0], 0.]
    probes = []
    for fraction in [.25, .5, .75]:
        rho = radius-b+b*fraction
        for theta in [0., .2, .7, 1.2, math.pi/2]:
            for offset in [-.0001, .0001]:
                r = rho+offset
                point = [origin[i]+r*(math.cos(theta)*n[i]+math.sin(theta)*t[i])-(a*fraction if i == 2 else 0.) for i in range(3)]
                probes.append(dict(point=point, expected=0 if offset < 0 else 1, region='cone', fraction=fraction, theta=theta))
        # Straight material outside the circular sector at both tangent seams.
        for normal, tangent in [(n,t),(t,n)]:
            for offset in [-.0001, .0001]:
                point = [origin[i]+(rho+offset)*normal[i]-.01*tangent[i]-(a*fraction if i == 2 else 0.) for i in range(3)]
                probes.append(dict(point=point, expected=0 if offset < 0 else 1, region='straight', fraction=fraction))
    results = {}
    for suffix in ['result.brep','truck.step']:
        path = root/'chamfer'/f'{mode}_complex'/'A4'/'capture'/f'001.{suffix}'
        text = subprocess.check_output([str(reader),str(path),'batch'], input='\n'.join(' '.join(map(str,p['point'])) for p in probes)+'\n',text=True)
        states = [int(line) for line in text.splitlines() if line in ['0','1','2','3']]
        assert len(states) == len(probes)
        results[suffix] = dict(sha256=hashlib.sha256(path.read_bytes()).hexdigest(), states=states, mismatches=sum(x != p['expected'] for x,p in zip(states,probes)))
    area=1.1**2-(1-math.pi/4)*radius**2
    perimeter=4.2+math.pi*radius/2
    c=3+math.pi/4
    exact=area-a*(perimeter*b/2-c*b*b/3)
    quadrature=[]
    for count in [16,32,64,128,256]:
        value=(1-a)*area+a/count*sum(area-perimeter*(b*(1-(i+.5)/count))+c*(b*(1-(i+.5)/count))**2 for i in range(count))
        bound=a*c*b*b/(12*count*count)
        assert abs(value-exact) <= bound+2e-15
        quadrature.append(dict(count=count,value=value,bound=bound))
    records[mode]=dict(probes=probes,results=results,procedural_volume=exact,quadrature=quadrature)
output.write_text(json.dumps(records,indent=2))
assert all(r['mismatches']==0 for v in records.values() for r in v['results'].values())
print('126 independent material witnesses pass on both native BREP and Truck STEP.')
