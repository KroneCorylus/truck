# Exact spherical STEP parameter curves

The F1 hemisphere/box union was rejected after closed-face splitting created a
diagonal parameter curve on its sphere. The importer now maps STEP longitude
and latitude to the existing modeling sphere's colatitude and longitude exactly.
It retains the source placement and handles inverted support coordinates before
mapping. No new surface type or spatial curve approximation is introduced.
Finite/nonsingular transforms and the existing pcurve regularity checks apply.

The STEP sphere's generalized derivative adapter also now swaps derivative
orders and applies the latitude sign. Explicit first/second derivatives were
already correct; the generalized adapter disagreed with them.

## Regression proof

At parent `2a0c0968ee57b2233069296da2fe932545ee0ccb`, the spherical position/derivative conversion test
fails as unsupported and the mixed-derivative test fails at derivative (0,1).
Both final solid tests also fail on the parent production sources at import.
`parent-pcurves.txt` and `parent-solid.txt` retain those failures. The invalid
geometry test already passed by rejection and remains required after support
is added. No existing assertion was removed, relaxed or ignored.

Three importer tests cover derivative orders zero through four in each axis,
affine placements/reflections, support inversion, reversed traces, longitude
seams, both poles, diagonal/isoparametric traces, cutting and invalid geometry.
Two solid tests cover the unmodified native STEP input and its initial radius-2
fillet, independent material, radius and outward normals, original-to-final face
history, immutable input, closed meshes and STEP round trips. The fillet is also
checked after rotation/translation and at scale 0.1.

## Independent material and native checks

`independent-volume.py` derives the hemisphere/box overlap from the dimensions,
using the exact inner antiderivative and a converged outer Simpson integration.
It derives the removed vertical corner above the sphere separately. The fixture
README records the equations and unchanged input provenance.

- Input: 21467.43404299306.
- Filleted material: 21454.331120055493.
- Native integration of Truck's exported STEP: 21454.330821521886.
- Absolute error: 0.000298533607 (relative about 1.39e-8).
- Native STEP validity: true; one solid; maximum edge/vertex tolerance 3.251e-6.
- Native-to-Truck sampled distance: 1.185324e-6 (1722 samples).
- Truck-to-native sampled distance: 1.909951e-6 (1905 samples).

These fit the official replay tolerance 0.001. Distances are sampled, not a
certified global bound. The capture covers the initial fillet in draft/angle/F1,
not its later drafting operation. The existing fillet solver needed no changes.

## Validation

Complete stepio and shapeops release suites: **493 passed, 0 failed,
1 existing ignored**, across 96 test/doc targets.
All 8 replay tests, release all-target Clippy for both affected crates and
benchmarks, single-thread action benchmark smoke and the final replay build pass.
`metadata.json` records commands, exit codes and frozen source/executable hashes.
No dependencies, public signatures, diagnostic meanings or consumer files changed.
No performance claim is made. The prior geometry PCurve precision assertion is
outside this affected-crate run and remains open; geometry sources are unchanged.

Six native input/reference files are byte-identical to the preceding full capture.
The targeted archive is byte-verified: `d9869c30e7325833874f11d72b09e93816aa81c68e6ad8df0e7c5cc95ad4001a`.
A clean full capture follows the implementation/proof commit and is documented
in the audit checkpoint separately.
