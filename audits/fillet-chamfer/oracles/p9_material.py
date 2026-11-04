"""Independent rectangular-box, unequal-cylinder crease material oracle.

Local orthonormal axes point inward from the selected corner. The large
radius a runs along v, the smaller b along u, and d is inward top depth.
Full retained sections are [a-fa(d),U] x [b-fb(d),V], with each lower
bound zero once depth exceeds that radius. No native mass is an input.
"""
import math


def inset(radius, depth):
    return 0.0 if depth >= radius else radius - math.sqrt(depth * (2 * radius - depth))


def inside(point, lengths=(5., 5., 5.), radii=(1., .5)):
    u, v, d = point
    U, V, W = lengths
    a, b = radii
    return (0 <= d <= W and inset(a, d) <= u <= U
            and inset(b, d) <= v <= V)


def volume(lengths=(5., 5., 5.), radii=(1., .5), n=16384):
    U, V, W = lengths
    a, b = sorted(radii, reverse=True)
    def f(t):
        d = b*t*t
        return inset(a, d)*inset(b, d)*2*b*t
    overlap = (f(0)+f(1)+sum((4 if i % 2 else 2)*f(i/n)
                            for i in range(1, n))) / (3*n)
    return U*V*W - (V*a*a+U*b*b)*(1-math.pi/4)+overlap


def witnesses():
    import random
    rng = random.Random(9)
    rows = []
    def add(label, p):
        rows.append(dict(label=label, point=p,
                         expected=0 if inside((5-p[0], p[1], 5-p[2])) else 1))
    for i in range(2000):
        p = [rng.uniform(-.1, 5.1) for _ in range(3)] if i < 1000 else [
            rng.uniform(3.9, 5.1), rng.uniform(-.1, .6), rng.uniform(3.9, 5.1)]
        add('random', p)
    for depth in [.0001, .01, .1, .25, .49, .5, .51, .7, .99]:
        x, y = 5-inset(1., depth), inset(.5, depth)
        for v in [y+.002, (y+5)/2, 4.99]:
            for e in [-1e-4, 1e-4]:
                add('large cylinder', [x+e*(x-4), v, 5-depth+e*(1-depth)])
        if depth < .5:
            for u in [.01, x/2, x-.002]:
                for e in [-1e-4, 1e-4]:
                    add('small cylinder', [u, y+e*2*(y-.5), 5-depth+e*(1-2*depth)])
            for sx in [-1, 1]:
                for sy in [-1, 1]:
                    add('crease sector', [x+sx*1e-5, y+sy*1e-5, 5-depth])
    for p, n in [([0,2,2],[-1,0,0]),([2,0,2],[0,-1,0]),
                 ([2,5,2],[0,1,0]),([5,2,2],[1,0,0]),
                 ([2,2,0],[0,0,-1]),([2,2,5],[0,0,1])]:
        for e in [-1e-4, 1e-4]:
            add('original plane', [p[i]+e*n[i] for i in range(3)])
    return rows


if __name__ == '__main__':
    assert abs(volume()-123.69042579279512) < 1e-11
    for scale in [.1, 1., 3.]:
        assert abs(volume(tuple(5*scale for _ in range(3)),
                          (scale, .5*scale))-volume()*scale**3) < 1e-9
    print(volume())

    import argparse, json, subprocess
    parser = argparse.ArgumentParser()
    parser.add_argument('--probe')
    parser.add_argument('--step')
    parser.add_argument('--output')
    args = parser.parse_args()
    if args.probe:
        rows = witnesses()
        result = subprocess.run([args.probe, args.step], check=True, text=True,
            input=''.join(' '.join(map(str, row['point']))+'\n' for row in rows),
            capture_output=True)
        states = [int(line.split()[1]) for line in result.stdout.splitlines()
                  if line.startswith('STATE ')]
        assert len(states) == len(rows)
        for row, state in zip(rows, states): row['actual'] = state
        report = dict(volume=volume(), checks=len(rows),
                      failures=[row for row in rows if row['actual'] != row['expected']],
                      witnesses=rows)
        if args.output:
            from pathlib import Path
            Path(args.output).write_text(json.dumps(report, indent=2)+'\n')
        print('material witnesses', len(rows), 'failures', len(report['failures']))
        assert not report['failures']
