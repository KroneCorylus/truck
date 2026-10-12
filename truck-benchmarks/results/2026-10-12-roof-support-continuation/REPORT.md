# Closed-wire continuation onto a tangent roof support

Captured blend/simple/Q4 operation 2 now constructs a valid solid when the
post-root fillet reaches the existing rounded roof. The original false-success
path was rejected at 67cf761f; that rejection remains the fallback for unsupported
continuations. The same original STEP fixture and physical selection are used.

The construction finds where the rolling contact leaves its planar support,
splits the blend there, and solves further sections on the adjacent tangent face.
It shares crossing vertices and reconnects surviving original boundary paths
through the new contacts. Periodic parameter-space regions check that contacts
stay inside their supports and outside unrelated holes. Planar output validation
now handles changing loop counts. Existing edge intersection solves share a
fallible implementation so failed continuation samples propagate without panic.
The normal path and public modeling APIs are unchanged.

The integration regression replaces the intermediate rejection expectation with
a stronger successful-solid contract. It fails on 67cf761f with OutsideNeighbour.
It checks the input, independent contact positions/radius/normals, analytic
surface-integral volume, topology, closed meshes, STEP, immutable inputs, reversed
selection, rigid transforms and scales 0.25, 1 and 10. The smaller radius-0.2
fillet remains tested. A focused unit checks a cylindrical trim and hole across
the periodic seam. No existing assertion was removed to accept a worse result.

The independent integral converges to about 149.083817359215; its derivation is
in truck-shapeops/tests/ROUNDED_ROOF_POST.md. The targeted native comparison
reports one valid solid from both kernels, native volume 149.08377356777387 and
Truck STEP volume 149.08379605764409. Maximum sampled distances are
0.00032652459800665539 (2680 native-to-Truck samples) and
0.0003299252872722417 (3137 Truck-to-native samples), below the 0.001 limited
comparison threshold. These are sampled checks, not a full parity verdict.

Full release shapeops: 329 passed, zero failed, one existing ignored, 62 targets.
All-target shapeops/benchmarks Clippy and action benchmark smoke pass. The broader
geometry/modeling/meshalgo/shapeops/STEP checks for the preceding contact-solver
change are recorded in ../2026-10-12-continued-contacts, including the unchanged
preexisting PCurve derivative failure.

## Timings

truck-benchmarks/examples/rounded_roof_post.rs imports and selects outside timing.
RAYON_NUM_THREADS=1, one warmup and five samples, release build; compare 67cf761f
with this implementation. No other build or test runs overlap the measurements.

| Radius | Before construction median | After construction median | Before STEP median | After STEP median |
| --- | ---: | ---: | ---: | ---: |
| 0.2 | 1.459403 ms | 1.282273 ms | 110.232732 ms | 96.268942 ms |
| 1.0 | rejected, 1.084152 ms | 3.815339 ms | unavailable | 189.600051 ms |

All five radius-0.2 runs construct and export on both versions. All five radius-1
runs reject on the parent and construct/export now. Failure and success timings
are different work; the radius-1 row is not an export or construction speedup.

The verified new scope is closed-wire propagation from a planar outer boundary
onto one tangent neighbor per original edge and side. Open-wire propagation,
multiple different neighbors within one original edge, propagation through holes,
global collisions and downstream edits of Q4 remain unverified or unsupported.
