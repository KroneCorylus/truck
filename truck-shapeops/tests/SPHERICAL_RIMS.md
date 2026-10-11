# Plane/sphere rim fillets

The input is the portion of a radius-5 sphere between `z=-4` and `z=h`, with
`h=-3` or `h=3`. Its top circular edge receives a radius-0.5 rolling fillet. The
lower cut avoids a pole while the two top heights exercise opposite meridian slopes.
The cavity version reverses this closed shell inside the box `[-8,8]³`.

The exterior matrix covers native Sphere, revolved circular generatrix and exact
tensor-product NURBS representations, before/after rigid motion, with either wire
orientation: 24 operations. Four cavity operations cover Sphere and NURBS supports.
The NURBS construction multiplies the meridian and azimuth homogeneous coordinates;
it represents the original sphere exactly, without surface fitting.

Independent checks include radius and normals on a 9-by-17 surface grid, sampled G1
normal agreement across every generated boundary, complete face history, native
geometry, topology, closed meshes, unchanged input and STEP round trips. Geometry
and mesh tolerance is 0.001; STEP fitting tolerance is 0.00005. No production code
needed to change for these cases. This is new verification of existing behavior.

## Independent dimensions and volume

For sphere radius `R`, top height `h` and fillet radius `r`, the rolling-circle center
in a meridian section is

```
z_c = h-r
ρ_c = sqrt((R-r)²-z_c²)
z_s = R*z_c/(R-r)
```

`z_s` is the sphere contact height. The fillet satisfies
`(ρ-ρ_c)²+(z-z_c)²=r²`, with radial normal proportional to
`((ρ-ρ_c)*x/ρ, (ρ-ρ_c)*y/ρ, z-z_c)`. The volume follows from horizontal disks:

```
S(z) = R²z-z³/3
A(u) = (ρ_c²+r²)u-u³/3 + ρ_c[u*sqrt(r²-u²)+r²*asin(u/r)]
V = π[S(z_s)-S(-4)+A(r)-A(z_s-z_c)]
```

The input volume is `π[S(h)-S(-4)]`; cavity volume is `4096-V`.

## Paired OCCT observations

`audits/fillet-chamfer/probes/spherical_rim_blends.cpp` performs the four native
analytic exterior/cavity tasks in OCCT 7.9.3. Every result is a valid single solid.
The fitted OCCT patches have maximum sampled radius errors of about `3.91e-6` and
`4.06e-6` at the two heights, below the Truck modeling and STEP fitting tolerances.
Volume deviations from the independent formula are about `1.27e-6` and `2.22e-6`.
The probe records these deviations rather than claiming exact surface identity.
It does not test OCCT's transformed, NURBS, wire-reversal, G1, mesh or STEP outcomes.

Arbitrary sphere intersections, toroidal/cylinder support pairs, pole contact,
apex collapse, general spline supports and chamfer-distance semantics remain open.
