from fractions import Fraction
from math import comb


def area(points):
    degree = len(points) - 1
    coefficients = [
        [sum(Fraction(p[axis]) * comb(degree, i) * comb(degree-i, k-i)
             * (-1)**(k-i) for i, p in enumerate(points) if i <= k)
         for k in range(degree+1)]
        for axis in [0, 1]
    ]
    x, y = coefficients
    integral = sum((x[i]*y[j] - y[i]*x[j]) * j / Fraction(i+j)
                   for i in range(degree+1) for j in range(1, degree+1))
    chord = points[-1][0]*points[0][1] - points[-1][1]*points[0][0]
    return (integral + chord) / 2


for name, points in [
    ('G3', [(0, 0), (200, 0), (200, 50), (250, 100), (100, 200)]),
    ('G7', [(0, 0), (20, 100), (40, 60), (100, 0)]),
]:
    signed_area = area(points)
    print(name, 'signed area', signed_area, 'volume', abs(signed_area)*100)
