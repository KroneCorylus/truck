# Exact adaptive cubic endpoints and K4 STEP recovery

Cubic Hermite interpolation now writes its clamped endpoint knots directly
from the requested range. Reconstructing the endpoint as `start + width`
could overshoot by one floating-point step. An adaptive neighbor then began
before that endpoint, so the existing concatenation guard correctly rejected
the overlap. This interrupted K4 contact-curve export before surface fitting.
The fix keeps that guard, all fitting tolerances, and the subdivision budgets.

## Reduced regressions

The captured K4 failure reduces to a five-control-point quartic spline on
`[-0.022964907877347344, 0.5278476871165522]`. The parent returns no cubic fit;
the fixed implementation joins its adaptive pieces successfully. A separate
line regression exposes the endpoint overshoot directly:
`-0.005752014283787984` instead of requested `-0.005752014283787985`.

Tests require exact parameter bounds and endpoint controls, continuous cubic
join multiplicities, dense point/derivative accuracy across every output knot
span, reversed curves, rigid motion and scaling. The initial new test required
bitwise equality of evaluated endpoint positions. General B-spline basis
summation introduces ordinary roundoff, so the final regression instead checks
bitwise equality of the endpoint controls with the requested evaluated source
points, plus endpoint positions within 1e-12. Existing tests are unchanged.
The final new tests fail against the recorded parent production source.

## Native K4 result

The original captured K4 input now prepares STEP. Native comparison accepts
one valid solid. The construction and closed mesh are unchanged, including
mesh volume **214207.02817010845**. The native STEP volume estimate is
**214210.88280914773**, while the captured native reference reports
**214190.18597261963**. Maximum sampled distances are approximately **0.04617**
and **0.05618**. These discrepancies remain recorded; this is an export
recovery, not a sampled parity pass.

An independent diagnostic checks the two cylinder supports' offset equations.
For each sampled fillet point P and unit normal N, the radius-5 ball center
must be one of P +/- 5N. It must lie inside the outer cylinder at radius R-5
and outside the cut cylinder at radius R+5. K2/K3 use outer axis (50,50,z),
radius 50, and hole axis (50,y,70), radius 30. K4 uses outer radius 30 and a
radius-40 hole with origin (50,140,-30), axis (0,-sin(80 deg),cos(80 deg)).
The diagnostic reports the smaller residual over the two normal signs.

On a 63-by-63 interior grid per spline face, the K4 Truck STEP maximum
residual is **0.00037817 model units**, versus **0.07369132** for the captured
native fillet. For K2 the corresponding values are **0.00020379** and
**0.05543514**. These sampled point/normal constraints support the geometric
quality of the Truck output; they are not a certified global shape or volume
proof. Native edge/vertex tolerances alone do not bound fillet-interior error.
The original artifacts and comparison outcomes are retained unchanged.
K2 diagnostics use the unchanged STEP and native BREP bytes from the
`2026-10-13-ellipsoid-domains` capture; K4 uses the newly generated STEP.

## Validation

The five affected release suites (`truck-geometry`, `truck-modeling`,
`truck-meshalgo`, `truck-stepio`, `truck-shapeops`) completed with
**912 passed, 1 known unchanged failure, and 5 existing ignored tests**,
across 142 test/doc targets. The only failure is the documented PCurve
derivative precision test; `parent-precision-test.txt` reproduces the same input
and values on the parent. Existing assertions were retained; no test was newly ignored.
All-target release Clippy, the single-threaded action benchmark smoke and the
final replay build passed. Source/executable hashes and byte-verified replay
artifacts are recorded in `metadata.json` and `replays.zip`.
No performance claim is made. Full native capture follows the production
commit and is archived separately.

Rebuild and run the independent diagnostic from the repository root:

```sh
c++ -O2 -std=c++20 -I target/occt-sdk/usr/include/opencascade \
  truck-benchmarks/results/2026-10-13-cubic-endpoints/check-cylinder-ball.cpp \
  -L target/occt-sdk/usr/lib64 -lTKDESTEP -lTKXSBase -lTKTopAlgo -lTKBRep \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o target/check-cylinder-ball
LD_LIBRARY_PATH=target/occt-sdk/usr/lib64 target/check-cylinder-ball K4.truck.step K4
LD_LIBRARY_PATH=target/occt-sdk/usr/lib64 target/check-cylinder-ball K4.result.brep K4
```
