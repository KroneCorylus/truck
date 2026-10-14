# Spherical parameter curve fixture

`F1.step` is the unchanged `draft/angle/F1/capture/001.input.step` from
`audits/fillet-chamfer/capture/2026-10-14-unequal-radii/artifacts.zip`.
SHA-256: `cf4a66c00cf04e14785fa8e934eb945b0fdb8c79d51e5292599886a8c18f3c50`.

The native script unions the upper hemisphere of radius 20 with the box
`(-10,-10,10)` to `(10,10,30)`. Closed-face splitting introduces a diagonal
UV line on the spherical support, from `(pi,0)` to `(3*pi/4,pi/4)`.
The original STEP contains no skipped entities. Conversion must retain the
spatial curve exactly, including the spherical longitude/latitude chart.

The input volume follows from the dimensions alone. Subtract the overlap
`integral[-10,10]^2 (sqrt(400-x*x-y*y)-10) dx dy` from the hemisphere plus box:

```
V = 16000*pi/3 + 12000
    - 2*integral[0,10] (10*sqrt(300-x*x)
                       + (400-x*x)*asin(10/sqrt(400-x*x))) dx
  = 21467.43404299306
```

The one-dimensional integrand uses the exact antiderivative in y. Composite
Simpson convergence is recorded by the standalone dimension-only calculation
in `truck-benchmarks/results/2026-10-14-spherical-pcurves/independent-volume.py`.
No native result volume is used to define the expectation.

The selected edge is `(10,-10,sqrt(200))` to `(10,-10,30)`, radius 2.
Its fillet is the vertical cylinder of radius 2 centered on `(8,-8,z)`.
Writing `Y=-y`, the removed region has `8 <= x <= 10` and
`8+sqrt(4-(x-8)^2) <= Y <= 10`, with height `30-sqrt(400-x*x-Y*Y)`.
Integrate that height using the same exact Y antiderivative, then substitute
`x=8+2*sin(t)` to remove the circular bound's square-root endpoint.
The independently computed removed volume is `13.102922937567955`;
the filleted material is `21454.331120055493`. The standalone calculation
prints convergence for both input and fillet. This covers the captured initial
fillet, not the later `depouille` operation in the native draft script.
