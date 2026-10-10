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
