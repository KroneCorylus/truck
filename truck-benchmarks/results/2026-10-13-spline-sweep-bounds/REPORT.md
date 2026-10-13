# Complete spline extrusion support domains

The native spline-prism STEP inputs encode a unit extrusion direction. Their
boundaries require sweep parameters down to approximately -102, while the
imported support retained only `[0,1]`. Unbounded surface inversion found the
right points, but bounded geometric inclusion rejected them. The existing
support-domain fitter covered cylinders and conic profiles only.

The fitter now handles polynomial and positive-weight rational spline
profiles. For a point P on a sweep, `P = Q + v V`, where Q lies on the profile
and V is the extrusion vector. Projection along V gives
`v = (P-Q) dot V / (V dot V)`. Complete positive-weight control hulls bound
both P and Q, so subtracting their projection intervals bounds v even for
nonplanar profiles. The original interval and existing tolerance padding are
retained. The curve and surface geometry remain exact; only the support's
parameter interval changes. Conic handling is unchanged.

## Regression proof

The new importer regression round-trips finite polynomial and rational
extrusion supports with nonplanar profiles, negative trims and interior sweep
extrema beyond the boundary vertices. It checks every boundary's inclusion,
parameter bounds, physical positions and oriented normals, including reversed
profiles and affine frames. The parent fails the first boundary inclusion
check; the fixed version passes all eight variants.

Native G3/G7 input regressions check geometric consistency, complete boundary
domains, topology, closed meshes, independent volume and STEP round trips at
small/original scales and under rigid motion. Their independent volumes are
**2300000** and **372000**, from exact rational integration of the polynomial
profile area times the extrusion's normal height. `independent-volume.py`
uses no imported or native result geometry. Fixture provenance and checksums
are recorded under `truck-shapeops/tests/fixtures/spline-prisms/README.md`.

Both solid regressions fail against the parent at input consistency.
`parent-solids.txt` and `parent-bounds.txt` retain those failures. Existing
tests and assertions are unchanged; no test was newly ignored.

## Targeted native replays

All eight previously inconsistent spline-prism captures now import. G3, G5,
G7 and G9 construct geometrically consistent solids with unchanged inputs,
closed Truck meshes and native-valid single-solid STEP exports.

| Case | Native reference volume | Truck STEP volume | Native → Truck max distance | Truck → native max distance |
| --- | ---: | ---: | ---: | ---: |
| G3 | 2294150.6118989084 | 2294150.607043287 | 0.0000114095 | 0.0000094118 |
| G5 | 2292148.707844867 | 2292147.8343762597 | 0.0017069353 | 0.0017029063 |
| G7 | 371121.31979745557 | 371121.321158088 | 0.0000063918 | 0.0000008153 |
| G9 | 370856.8495906483 | 370856.7803314651 | 0.0008899970 | 0.0009068426 |

G3, G7 and G9 meet the 0.001 limited sampled comparison threshold. G5 retains
its discrepancy and is not counted as a comparison pass. These are sampled
checks, not certified global shape bounds. G4 and G8 still reject nontangent
multi-edge wires; G6 and H1 still require unsupported nonplanar corner
construction. Their failed operations preserve their inputs.

Raw native input and result bytes come from the committed
`2026-10-13-cubic-endpoints` capture. The full capture follows the production
commit and is archived separately. No public API or dependency changed.

## Validation

The complete stepio and shapeops release suites passed: **461 passed, 0 failed
and 1 existing ignored test**, across 87 test/doc targets. All-target release
Clippy for both crates and benchmarks, single-threaded action benchmark smoke
and the final replay build passed. Commands and source/executable hashes are
recorded in `metadata.json`. The frozen source and replay hashes were unchanged
after validation. All **64** replay archive members were byte-verified against
the local files; all native artifacts and both fixtures were additionally
checked against the original committed archive. No performance claim is made.
