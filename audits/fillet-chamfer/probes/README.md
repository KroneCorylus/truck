# Independent OCCT outcome probes

These probes express the same modeling tasks as Truck regressions. They are separate
from the 1,026 inventoried OCCT test entries; those entries remain unexecuted.

## Pyramid chamfers

`pyramid_chamfers.cpp` constructs and validates a square pyramid with base 10 × 10 and
height 10, then requests equal-distance chamfers of 0.2 and 1.2 on five selection classes:
one ridge, two adjacent ridges, two opposite ridges, three ridges and all four ridges.
The matching Truck checks and analytic section integrals are in
`truck-shapeops/tests/r11_corners.rs` and `truck-shapeops/tests/R11.md`.

From the Truck repository root, with the local matching SDK:

```sh
g++ -std=c++17 -O2 -Wall -Wextra -Wpedantic \
  -isystem target/occt-sdk/usr/include/opencascade \
  audits/fillet-chamfer/probes/pyramid_chamfers.cpp \
  -Ltarget/occt-sdk/usr/lib64 \
  -lTKFillet -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKBRep \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o target/pyramid_chamfers_occt
target/pyramid_chamfers_occt
```

Recorded output uses installed OCCT 7.9.3 libraries, not the modified OCCT source
checkout inventoried by the catalog. All ten cases finish with a valid single solid
according to `BRepCheck_Analyzer`. The requested bevel planes match the independent
setback equations at sampled parameters, including when OCCT represents a planar bevel
as a spline. Volume integration uses relative accuracy `1e-9`.

The two kernels support every selection in this fixture, but use different corner
constructions for adjacent selections:

| Selected ridges | Truck faces | OCCT faces | Truck volume, d=1.2 | OCCT volume, d=1.2 |
| --- | ---: | ---: | ---: | ---: |
| One | 6 | 6 | 325.261212081 | 325.261212081 |
| Two adjacent | 7 | 8 | 317.820067215 | 317.024583711 |
| Two opposite | 7 | 7 | 317.567676660 | 317.567676660 |
| Three | 8 | 9 | 310.442019987 | 310.289324248 |
| Four | 9 | 10 | 303.316363314 | 302.778699217 |

Truck intersects the requested bevel planes. OCCT adds a face for the adjacent,
three- and four-ridge configurations, and removes more material around the junction.
The `matches_miter_volume` field compares OCCT with Truck's independently derived
miter construction, **not** with a required OCCT result. A false value records a corner
geometry difference, not an invalid solid or unsupported chamfer. The difference also
occurs at setback 0.2; raw values are in `pyramid_chamfers_occt.txt`.

Truck additionally checks exact-volume tolerance, closed tessellation, convexity,
face history, reversed selections, rigid transforms, immutable input and STEP round trips.
The OCCT probe does not establish its mesh, STEP, transformed-input or downstream-edit
behavior, nor does it establish parity of corner continuity. General high-valence fillets
and alternative chamfer corner transitions remain open audit work.

## High-valence fillets

`high_valence_fillets.cpp` constructs regular pyramids with 4, 5, 6 and 8 sides,
apothem 5 and height 10, and selects all apex ridges at radius 0.2 or 1.2. Build it
with the command above, substituting `high_valence_fillets` for `pyramid_chamfers`
in the source and output names. Recorded output is `high_valence_fillets_occt.txt`.
Each input is checked for native validity and independent volume, and copied for
each operation. Each successful result records native validity, solid/face counts,
integrated volume and whether it contains the expected analytic sphere.

Truck now supports an exact spherical corner on all eight tasks. Its tests also
repeat them after a rigid transform, measure G1 contact and validate topology,
mesh closure, STEP, order independence and input preservation. Two cases cut through
the actual spherical corner and round-trip the cut through STEP. The independent
volume derivation is in `truck-shapeops/tests/HIGH_VALENCE.md`.

OCCT 7.9.3 also returns valid single solids on all eight tasks, with matching face
counts, but its corner patches are not analytic spheres. Their integrated volumes
differ from the spherical construction:

| Ridges | Truck volume, r=1.2 | OCCT volume, r=1.2 |
| --- | ---: | ---: |
| 4 | 324.091282504 | 324.133589302 |
| 5 | 296.800583654 | 296.953082926 |
| 6 | 284.324800437 | 284.555630102 |
| 8 | 273.227060569 | 273.557929759 |

`matches_spherical_volume=false` records that geometric difference; it does not
mean either result is invalid. The paired probe establishes successful native
operations for these dimensions, not identical corner geometry or full continuity,
mesh, STEP or downstream-edit equivalence. Partially selected corners, inconsistent
offset planes, unequal radii and general curved corner filling remain open.

## Conical rims

`conical_blends.cpp` constructs frusta of height 8 and bottom radius 5, with top
radius 3 or 7. Each exterior solid and corresponding reversed cavity shell receives
a radius-0.5 fillet or chamfer with setbacks 0.5 on the top plane and 0.3 down the
cone meridian. Build with the pyramid command, substituting `conical_blends` and
adding `-lTKPrim` before `-lTKFillet`. Raw output is `conical_blends_occt.txt`.

All eight OCCT 7.9.3 tasks return valid single solids with the independently derived
volumes and sampled dimension equations used by Truck. The formulas and physical
selection are documented in `truck-shapeops/tests/CONICAL_BLENDS.md`.

| Top radius | Exterior fillet volume | Exterior chamfer volume |
| --- | ---: | ---: |
| 3 | 409.957836865 | 409.195040261 |
| 7 | 909.164428026 | 910.043365252 |

Cavity volumes equal 6336 minus the corresponding exterior value. Truck additionally
checks exact NURBS supports, rigid transforms, reversed selections, closed meshes,
history, unchanged inputs and STEP. Those additional OCCT outcomes were not probed.
Oblique intersections, varied cone angles and apex collapse remain open.
