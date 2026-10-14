"""Dimension-only hemisphere/box union volume; composite Simpson convergence."""
from math import pi, sqrt, asin, fsum, sin, cos

def volume(n):
    # Exact y antiderivative of sqrt(400-x*x-y*y), exploiting both symmetries.
    f = lambda x: 2 * (10*sqrt(300-x*x) + (400-x*x)*asin(10/sqrt(400-x*x)))
    h = 10/n
    integral = h/3 * fsum([f(0), f(10)] + [(4 if i % 2 else 2)*f(i*h) for i in range(1,n)])
    return 16000*pi/3 + 12000 - integral

def removed_volume(n):
    # The rounded vertical corner removes a square minus a quadrant above the sphere.
    # x=8+2*sin(t) removes the square-root endpoint in its circular lower y bound.
    def f(t):
        x, low = 8 + 2*sin(t), 8 + 2*cos(t)
        a = 400-x*x
        primitive = lambda y: (y*sqrt(a-y*y) + a*asin(y/sqrt(a)))/2
        return 2*cos(t) * (30*(10-low) - primitive(10) + primitive(low))
    h = pi/(2*n)
    return h/3 * fsum([f(0), f(pi/2)] + [(4 if i % 2 else 2)*f(i*h) for i in range(1,n)])

if __name__ == '__main__':
    for n in [64, 128, 256, 512, 1024, 2048]:
        print(n, 'input', format(volume(n), '.17g'), 'removed', format(removed_volume(n), '.17g'), 'fillet', format(volume(n)-removed_volume(n), '.17g'))
