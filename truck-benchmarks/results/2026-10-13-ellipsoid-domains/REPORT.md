# Trimmed elliptical revolutions and H2 junction fillet

STEP revolution imports now use complete bounded linear pcurves to retain the
elliptical meridian sheet and range of a face. Endpoint agreement, oriented
outer-loop area and pole continuity are checked before narrowing the support.
Hole-only, non-finite and mismatched pcurves leave the original domain intact.
Non-elementary revolved conics export their bounded meridians as exact rational
arcs, since an untrimmed STEP ellipse would restore the unwanted full support.

The mesher closes a single collapsed polar bridge using the boundary's winding.
Choosing the shortest angular jump instead changes a 270-degree sector to its
90-degree complement. Revolution inversion also avoids rounded polar seeds:
finite radial derivatives at a nearly axial point can point onto the opposite
meridian sheet. Interior regular seeds preserve the intended sheet.

## Regressions

The unchanged native H2 input is an ellipsoid/cone sector. Tests cover both
meridian sheets, all three boundary starting edges, rigid motion, original
scale and a smaller scale. Shared solid and STEP checks require consistent
geometry, closed topology/meshes and independent material volume.

The H2 ellipse/cone junction fillet additionally checks radius, outward normals,
contact heights, independently integrated volume, unchanged input and STEP
round trips on both meridian sheets with transformed/reversed selections.
The fixture README derives the input and rounded volumes; neither comes from
the native output. A mesher unit regression checks the 270-degree polar chart
under cyclic boundary starts and reversed surface/wire orientations. Inverse
search tests cover exact and nearest searches at three scales and multiple
near-pole parameters/seeds.

`parent-final-regressions.txt` runs the final solid regressions against parent
48539317 production sources: all three capability tests fail at the original
full meridian domain; invalid pcurve fallback passes. `parent-polar-boundary.txt`
and `parent-pole-inverse.txt` isolate the independent winding and inverse-seed
failures. Other failing logs document intermediate implementations; they are
retained as investigation evidence, not final validation.

## Targeted native comparison

At the unchanged native scale, H2 now constructs a consistent solid with a
closed mesh, preserves its input and prepares STEP. Native comparison accepts
the STEP as a single valid solid, with sampled distances below 0.000015 in both
directions. Its native volume is 13156727.639241457, Truck STEP volume is
13156726.838964274, and independent ideal volume is 13156727.672385866.
The native Gauss–Kronrod volume diagnostic is 13156727.672235012, much closer
to the independent integral; its recorded edge/vertex tolerances are below
0.000025. These are limited checks, not a full parity proof.

H4 now reaches construction and STEP export, but its Truck mesh is open and
its STEP fails native validity. Its sampled geometry and volume also disagree.
H4 remains unsupported. The original-scale input coverage does not establish
correctness of that central-axis fillet. Its native result also carries edge
and vertex tolerances of approximately **22.9741 model units**, despite passing
the native validity analyzer. `inspect-native.cpp` and `H4-native-tolerances.txt`
record that independent diagnostic. That reference is unsuitable as evidence
of accurate material near the pole without additional analysis; it does not
excuse the invalid Truck output.

No performance claim is made. The full clean capture has not yet been rerun
at this checkpoint; the previous captured run remains unchanged.

## Validation

- Five affected release suites (`truck-geometry`, `truck-modeling`,
  `truck-meshalgo`, `truck-stepio`, `truck-shapeops`): **905 passed, one known
  unchanged failure, five existing ignored tests**, across 141 test/doc targets.
- The sole failure is `truck-geometry/tests/pcurve.rs::pcurve_derivation`, with
  the exact same values `-2829663063.761872` versus `-2829663063.7618704` recorded
  in `../2026-10-11-spherical-meridian-step/pcurve-parent-existing-failure.txt`.
  Its assertion is unchanged; no new test is ignored.
- All-target release Clippy for those five crates and benchmarks: passed.
- Action benchmark smoke, with `RAYON_NUM_THREADS=1`: passed.
- Final replay example build: passed.
- Clippy initially rejected the deliberately rounded decimal pole literal in
  the new inverse test. The same value is now calculated by rounding pi/2 to
  12 decimal places; the focused release regression and final Clippy both pass.
  No geometric assertion changed.

Commands and exit statuses are in the final validation logs. Touched Rust
files were formatted individually with nightly rustfmt and `skip_children`.
`metadata.json` records source/executable hashes and verifies every targeted
native input/reference against the previous committed capture archive.

The standalone native diagnostic can be rebuilt from the repository root:

```sh
c++ -O2 -std=c++17 -I target/occt-sdk/usr/include/opencascade \
  truck-benchmarks/results/2026-10-13-ellipsoid-domains/inspect-native.cpp \
  -L target/occt-sdk/usr/lib64 -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d \
  -lTKMath -lTKernel -o target/inspect-ellipsoid-native
LD_LIBRARY_PATH=target/occt-sdk/usr/lib64 target/inspect-ellipsoid-native CASE.result.brep
```
