# Plane/cone rim blends

`conical_blends.rs` uses frusta of height 8 and bottom radius 5. The radius changes
with height at slope `s = -0.25` or `+0.25`, giving top radius 3 or 7. Each exterior
case uses both a revolved straight generatrix and an exact rational ruled surface,
original and rigidly transformed coordinates, and both wire orientations. Reversing
a chamfer wire also swaps the two distances to retain the same physical selection.
Separate cavity cases put each analytic frustum inside a box and reverse its shell.

The tests check input and output geometry, independent volume, closed topology and
meshes, sampled fillet radius and chamfer setbacks, outward normals, complete face
history, unchanged inputs and STEP round trips. Fillet radius is 0.5. Chamfer setbacks
are 0.5 on the top plane and 0.3 down the cone's straight meridian. Mesh tolerance is
0.001 and STEP fitting tolerance is 0.00005.

This matrix exposed three issues:

- Revolved-surface inclusion tested world-space curves against the untransformed
  generating geometry. Curves now move into the surface's coordinate system first;
  singular transforms reject inclusion. The modeling regression also covers B-spline,
  rational and circular generatrices, reversed orientation, scale and affine shear,
  including false-positive checks for curves in the wrong coordinate system.
- Independently solved contact parameters can jump at a periodic seam. Fillet fitting
  now interpolates a continuous sequence of equivalent periodic coordinates and seeds
  complete turns with quarter stations. Chamfer sampling retains the previous period
  branch as well. The geometry regression crosses a seam in both angular ranges and
  fits a full circle, comparing physical geometry with the exact contact sections.
- A procedural boundary fit could pass its adaptive interior sample and fail STEP's
  later validation. Export now retries with finer initial intervals and tighter point
  fitting error, retaining total dyadic subdivision depth 16 and the same final distance
  tolerance. A manufactured degree-six bulge independently tests this failure mode.

## Independent volume and dimensions

Let `R = 5 + 8s` be the top radius and `q = sqrt(1+s²)`. The frustum volume is
`V₀ = 8π(25 + 5R + R²)/3`.

For a fillet of radius `r`, the circular meridian has center
`(ρ_c,z_c) = (R-r(s+q), 8-r)` and radius `r`. Its swept surface therefore satisfies
`(ρ-ρ_c)²+(z-z_c)²=r²`, where `ρ=sqrt(x²+y²)`. Let `β=acos(-s/q)`.
The removed meridian area is the contact quadrilateral minus a circular sector:

```
A = r²(q+s-β/2)
M_quadrilateral = r³(q+s)[2(q+s)+1/q]/6
M_sector = r³(1+s/q)/3
```

The moments are radial first moments about the circle center. Pappus's theorem gives
removed volume `2π[ρ_c A + M_quadrilateral - M_sector]`.

For plane setback `a` and cone setback `b`, the removed triangle has area `ab/(2q)`
and radial centroid `R-(a+sb/q)/3`. Its volume is `2π` times their product. The bevel
meridian satisfies `ρ+(aq/b-s)(z-8)=R-a`; the tests sample this equation and its normal.

Cavity results have volume `6336 - V_rounded_void`, with two closed boundary shells
and reversed cavity normals. The formulas are independent of either kernel's output.

## OCCT comparison and remaining scope

`audits/fillet-chamfer/probes/conical_blends.cpp` performs the same native-coordinate
analytic tasks in OCCT 7.9.3, recording validity, solid count, volume and sampled
dimension equations. Its source and raw output accompany the audit. Truck's transformed,
reversed-selection, NURBS, mesh and STEP checks are additional independent validation;
the probe does not establish those OCCT outcomes.

Oblique plane/cone intersections, varied cone angles, apex/contact collapse, curved
terminations and arbitrary support pairs remain broader audit work.
