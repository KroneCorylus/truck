# Polar cap seams and degenerate STEP boundaries

H4's central-axis fillet now produces a closed Truck mesh and a native-valid
single-solid STEP output. Its independent radius, normal and material checks
pass on both elliptical meridian sheets, including rigid motion, reversed
selection and two scales. The unchanged native H2 fixture also describes H4's
input; the fixture README derives the rounded volume independently.

## Mesh repair

An artificial polar seam starting on an existing meridian can overlap that
physical boundary and replace its shared samples. The cap now rotates its
parameter chart to cross a transverse boundary segment, retaining every
physical boundary sample. Both copies of the artificial periodic seam use the
same spatial samples; independently subdividing them can differ by one sample
due to roundoff and leave a triangular crack.

Unit regressions cover north/south caps, both surface orientations, boundary
starts, transformed ellipsoids, and a stepped meridian boundary. They require
one remaining physical boundary with exactly the original samples and correct
triangle winding. The H4 solid regression additionally requires consistent
geometry, unchanged input, closed mesh, independent volume and STEP round trips.

## STEP repair

A single winding face boundary can enclose a surface pole without passing
through it. The formatter now emits that collapsed boundary as a FACE_BOUND /
VERTEX_LOOP / VERTEX_POINT / CARTESIAN_POINT sequence. It checks periodicity,
collapsed support endpoints and the sampled boundary winding; an annular face,
a non-winding boundary or a pole already on the boundary gets no extra loop.
The compressed modeling topology is unchanged. Native STEP readers can then
reconstruct the full polar face, including its parameter seam.

Export regressions verify the pole's physical position across both poles,
parameter-axis reversals, face orientation, rigid motion and boundary starts.
Annular faces retain their two physical boundaries. Reimport preserves physical
vertex/edge/face counts and all oriented edge loci and tangent directions. The
initial new test compared internal edge indices directly; import reindexes and
reorients edges, so the final test compares the complete physical boundary and
33 geometric/tangent samples per edge instead. No pre-existing assertion changed.
H2 and the unrounded sector also assert that no extra pole is exported.

StepModel construction now needs the geometry traits used by the pole search.
The Breaking changelog entry lists the new generic bounds and every PiezaCAD
call site. The app already uses concrete modeling geometry at those sites and
needs no call-site changes.

## Native evidence

At original scale, the output passes the native STEP validity analyzer as one
solid. Its native Gauss–Kronrod volume is **13159710.46449488**, compared with the
independent integral **13159710.462164925** (absolute difference **0.002329955**).
Its maximum native edge/vertex tolerance is **0.00001 model units**. This check
uses the standard STEP reader without an additional ShapeFix pass.

The captured reference remains unsuitable for accurate material comparison
near the pole: its volume is **13151259.698596016** and edge/vertex tolerances
reach **22.9741**. The sampled comparison still disagrees (approximately
0.00218 and 4.14211 maximum distance in the two directions). That discrepancy
is retained and is not counted as a parity pass. The native reader's ordinary
volume estimate is **13159710.046042992**; the independent diagnostic uses the
more accurate Gauss–Kronrod integration.

The parent-failure log runs the final new tests against parent production
sources: missing STEP pole, nine cap boundaries instead of one, and an open
H4 mesh. Earlier logs preserve intermediate investigations and do not replace
the final validation results. The diagnostic copy with a manually inserted
vertex loop established the export cause; final evidence uses generated STEP.

## Validation

The three affected release suites (`truck-meshalgo`, `truck-stepio`,
`truck-shapeops`) passed: **549 passed, zero failures,
2 existing ignored tests**, across 97 test/doc targets.
All-target release Clippy for those crates and benchmarks, the single-threaded
action benchmark smoke, and the final replay example build passed.
`metadata.json` records source/executable hashes and verifies every targeted
native input/reference against the unchanged capture archive; all archived
replay files were verified byte for byte.
No performance claim is made. A full clean native capture follows the production
commit and is archived separately; targeted evidence alone is not a parity proof.

Rebuild the independent native diagnostic from the repository root:

```sh
c++ -O2 -std=c++17 -I target/occt-sdk/usr/include/opencascade \
  truck-benchmarks/results/2026-10-13-polar-cap-seams/inspect-polar-step.cpp \
  -L target/occt-sdk/usr/lib64 -lTKDESTEP -lTKXSBase -lTKTopAlgo -lTKBRep \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o target/inspect-polar-step
LD_LIBRARY_PATH=target/occt-sdk/usr/lib64 target/inspect-polar-step \
  H4.truck.step 13159710.462164925
```
