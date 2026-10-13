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

## H4 central-axis fillet

H4 selects the central vertical axis of the same input shape. The radius-10
concave round fills the missing quadrant outside the circle centered at
`(10, -10)` in the XY plane. Write `y = -Y` for its nonnegative coordinate.
At each `(x, y)` the filled height is
`100 + 100 sqrt(1 - (x^2 + y^2)/150^2)`.

Parameterize the corner domain by `x = 10 (1-cos(t))`,
`0 <= y <= 10 (1-sin(t))`, `0 <= t <= pi/2`, with `dx = 10 sin(t) dt`.
For `A = 1-x^2/150^2`, the integral over height and y is

```
100 y + 50 [y sqrt(A-y^2/150^2) + 150 A asin(y/(150 sqrt(A)))].
```

Simpson integration with 1024 intervals gives added volume
**4291.225257665**, hence total **13159710.462164925**. The regression checks
this independent integral, cylinder radius/normals, closed mesh, STEP round
trip, the exported pole position, and unchanged input on both meridian sheets
with rigid motion, two scales and reversed selection.

The captured native reference's volume is **13151259.698596016**, and its
maximum edge/vertex tolerance is approximately **22.9741 model units**. Its
validity result is not an accurate material reference near the pole. Native
STEP validity and agreement with the independent integral are checked
separately from the sampled comparison against that reference.
