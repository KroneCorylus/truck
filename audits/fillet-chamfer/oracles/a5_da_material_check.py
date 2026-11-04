"""Check exported DA material against the complete input-derived A5 boundary."""
import argparse
import itertools
import json
import subprocess
from pathlib import Path
import numpy as np
from a5_assembly_material import build, crossings, flux, incidence, surface_witnesses


def check(shape, classifier):
    model = build('dist_angle')
    assert incidence(model)['bad_edges'] == []
    points = list(itertools.product(
        np.linspace(-.399173, .419827, 19),
        np.linspace(-.349173, .749827, 23),
        np.linspace(-.549173, .549827, 23)))
    sides, expected_sides = surface_witnesses(model)
    points.extend(sides)
    expected, bad, invalid = crossings(model, points, [1., .371139, .197531])
    other, bad2, invalid2 = crossings(model, points, [1., -.271931, .431719])
    assert not any(bad | bad2 | invalid | invalid2)
    np.testing.assert_array_equal(expected, other)
    np.testing.assert_array_equal(expected[-len(expected_sides):], expected_sides)
    assert all(np.isin(expected, [0, 1]))
    result = subprocess.run(
        [str(classifier.resolve()), str(shape.resolve()), 'classify'],
        input='\n'.join(' '.join(map(str, p)) for p in points),
        text=True, capture_output=True, check=True)
    states = [int(x) for x in result.stdout.splitlines() if x in ('0', '1', '2', '3')]
    assert len(states) == len(points)
    failures = [dict(point=list(points[i]), expected=int(1-expected[i]), actual=s)
                for i, s in enumerate(states) if s != 1-expected[i]]
    return dict(shape=str(shape), points=len(points), whole_volume=flux(model)[0],
                failures=failures)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('shape', type=Path)
    parser.add_argument('classifier', type=Path,
                        help='a6 material-reader supporting STEP/BREP and classify mode')
    args = parser.parse_args()
    report = check(args.shape, args.classifier)
    print(json.dumps(report, indent=2))
    raise SystemExit(bool(report['failures']))
