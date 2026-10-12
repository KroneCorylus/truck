# Toroidal sector inputs

Captured unchanged from OCCT V7_9_3 `tests/blend/simple/E5` and `E9` before
filleting. The native commands create torus sectors with `ptorus` and scale
them by 10. E5 has major/minor radii 100/100 and meridian angle pi/4;
E9 has radii 200/50 and meridian angle 8pi/9. Both sweep pi/2 around Z.
The complete input provenance is retained in
`audits/fillet-chamfer/capture/2026-10-12-spherical-pole/artifacts.zip`.

Both STEP inputs store the whole toroidal support, including a complete
circular meridian. Converting that circle to one rational quadratic gave a
zero denominator at the midpoint, so the original boundary inclusion check
rejected these inputs. E5 additionally exposed undefined normal sampling at
the horn-torus midpoint during STEP orientation classification.

The solid regressions check the input geometry, closed topology and meshes,
independent volume and STEP round trips. E5 is tested at scale 0.01; E9 at
scales 0.01 and 1, with a rigid transform on the smaller case. A separate
surface regression checks the horn-torus orientation under scaling, rigid
motion and reversal. This is input/round-trip coverage, not proof that every
fillet requested on these inputs succeeds.

## Independent volume

The sectors close to the revolution axis with horizontal planar caps. At
meridian angle v, their outer radius is R+r cos(v) and height is r sin(v).
Integrating horizontal disk sectors gives

```
V = pi/4 integral(v=0..beta) (R+r cos(v))^2 r cos(v) dv
  = pi/4 [r R^2 sin(beta) + R r^2 (beta+sin(2beta)/2)
          + r^3 (sin(beta)-sin(beta)^3/3)].
```

For E9 the meridian passes its maximum height; the descending inner branch
subtracts the correct annular hole. The formula yields 2027710.030094768
for E5 and 1539924.109587615 for E9, independently of either kernel's
output. Native input volumes are 2027710.0300947938 and 1539924.1095876284.


## Periodic inputs

J3/J4/J5 are unchanged inputs from the native cases with those names,
archived in `capture/2026-10-12-toroidal-sectors/artifacts.zip` under the
audit directory. All have major/minor radii 200/50. J3 has a complete
meridian and a 270-degree major revolution. J4/J5 have complete major
revolutions and meridians of 90/150 degrees, respectively.

For arbitrary major revolution theta, replace pi/4 in the formula above
by theta/2. Independent volumes are 7402203.300817017 (J3),
9012385.795251075 (J4), and 6753740.138234442 (J5). J3 is checked at
scales 0.01 and 1, with rigid motion in the smaller case; J4/J5 at scale
0.01, with rigid motion for J5. All require consistent closed geometry,
meshes, independent material volume and STEP round trips.

The previous torus inverse projected exact axis points off their support
because normalizing and applying inverse cosine amplified rounding error.
After projection is repaired, preprocessing generates circular toroidal
pcurves that must become exact modeling conics. J3 additionally requires
a 270-degree split in its existing closed parameter chart: choosing the
shortest periodic arc instead cuts the complementary 90-degree region.
