# Ellipsoid sector input

`H2.step` is the unchanged OCCT V7_9_3 `tests/blend/simple/H2` input from
`audits/fillet-chamfer/capture/2026-10-12-toroidal-sectors/artifacts.zip`,
entry `blend/simple/H2/capture/001.input.step`.
SHA-256: `9479ad8f07aaeca6de19ab72454a035a6ffd8d1af61c708a5fb7bc969c61a3e7`.

The profile is an upper quarter ellipse of radial radius 150 and height 100,
joined to the cone `r = 150 - z` for `-100 <= z <= 0`, with a flat base.
It revolves through 270 degrees. STEP stores a complete ellipse as the
support meridian; its bounded linear pcurves identify the actual quarter.
Revolving the complete ellipse double covers the upper ellipsoid and permits
inverse searches to cross the pole onto the wrong sheet.

The independent input volume is the horizontal disk-sector integral:

```
V = (3 pi / 4) [ integral(-100..0) (150-z)^2 dz
                 + integral(0..100) 150^2 (1-z^2/100^2) dz ]
  = (3 pi / 4) [(250^3-150^3)/3 + 2*150^2*100/3]
  = 13155419.23690726.
```

The native adaptive volume is 13155419.203132059. Regression variants reverse
the ellipse's radial axes and map its pcurves by `t -> pi-t` to retain the
same physical shape on the opposite meridian sheet. Rotated boundary starts,
rigid transforms and two scales exercise import, closed topology/tessellation,
independent material volume and STEP round trips. Invalid and hole-only
pcurves must leave the support domain unchanged.

## H2 junction fillet

For the radius-10 round, the ellipse contact is
`P = (150 cos(phi), 100 sin(phi))`. Its outward unit normal is the normalized
vector `(100 cos(phi), 150 sin(phi))`. The rolling-circle center `C = P + 10 N`
satisfies `C.x + C.y = 150 + 10 sqrt(2)` at its simultaneous cone contact.
Bisection on `0 <= phi <= atan(100/150)` solves this independent scalar equation.
The contact heights are `z0 = C.y - 10/sqrt(2)` and `z1 = 100 sin(phi)`.

Across that interval, replace the original cone/ellipse squared radii with
`[C.x - sqrt(100-(z-C.y)^2)]^2`. Integrating the difference and multiplying
by `3 pi/4` gives rounded volume **13156727.672385866**. The native adaptive
volume is 13156727.639241457. The regression checks the generated toroidal
section against the independently solved center, radius, outward normals and
contact heights, as well as the shared solid/STEP checks and input immutability.
The small-scale cases use both meridian sheets, rigid motion and reversed wires.

H4 selects the central vertical axis of the same input shape. Its output still
has an open Truck mesh and fails native STEP validity; it is not covered by the
successful H2 fillet assertions.
