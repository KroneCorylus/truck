# Captured upstream blend baseline

Kernel baseline: `d1b2bade` (the short-spine fix). The replay executable and its
source, native plugin/comparator, libraries, source fixtures and upstream harness
are hashed in `results.json`. These observations precede the revolution-angle fix.
The source catalog snapshot was generated at `d1b2bade`; intervening `4ee3385b`
is audit-only. No consumer app code was changed or exercised by this replay.

All **156 instrumented native entries remained OK**, matching their unchanged
native baseline classifications. Of these, 148 actually invoked `blend`; eight
were lexical hits with no intercepted operation. There were 161 capture attempts.

| Replay observation | Operations |
| --- | ---: |
| Constructed output | 45 |
| Inconsistent imported input | 58 |
| Input import rejected | 31 |
| Attempted blend routes rejected | 12 |
| Unequal-radius intent not handled by replay | 7 |
| Panic caught by replay | 5 |
| 120-second replay timeout | 2 |
| Incomplete native capture | 1 |

All 45 constructed outputs preserved their inputs, passed the geometric-consistency
check and prepared STEP. Native OCCT accepted all 45 STEP files as valid single
solids. Forty-four had closed Truck meshes. Forty-three had bidirectional sampled
boundary distance no larger than the replay modeling tolerance of 0.001. Thus
42 satisfied all these limited checks; **none is declared complete feature parity**.
Independent dimensions/G1, editing, global intersections and latency remain unverified.

Concrete findings for follow-up:

- `blend/simple/C7`: quarter cone, base radius 50, height 120, radius-10 fillet on
  its axial edge. Truck's closed mesh volume is 76156.44, near native 76158.86 at
  this tessellation tolerance, but its STEP volume is 75618.39 and the maximum
  native-to-STEP sampled distance is 4.2283. This is a suspected export/trim gap,
  not yet an established wrong in-memory fillet.
- `blend/simple/I1`: constructed output has an open Truck mesh and mesh volume
  712866.01, while its valid STEP volume is 940831.30 versus native 940831.16.
  STEP boundary samples agree within 7.35e-5. Investigate meshing separately.
- `blend/tolblend_simple/A1`: valid closed output and STEP, but a 0.02238 sampled
  geometric difference exceeds the replay tolerance. This upstream grid uses
  different native tolerance settings; establish intended dimensions/geometry
  before classifying it as a kernel feature gap.
- Spherical-sector inputs such as `blend/simple/D2` exposed loss of angular
  precision in revolved-surface curve inclusion. A separate parent-failing kernel
  regression and fix are being validated; this baseline remains unchanged.
- Cylindrical imports use a unit-length extrusion support while face boundaries
  can span larger axial ranges. Bounded-support inclusion rejects those inputs.
  Imported support normalization needs investigation without weakening existing
  bounded-extrusion contracts.
- Five caught panics originate in closed-edge/face preprocessing, before the blend
  call. The log identifies a parameter-division tolerance below the algorithm's
  minimum. Two replay timeouts are `blend/simple/I3` and `blend/simple/J9`.
- Twelve failed blend-route observations often involve multiple selected curved
  supports. Missing contour expansion/sequencing or alternative constructions must
  be considered before assigning an unsupported feature verdict.

The archive retains every manifest-listed artifact: BREP/STEP inputs/results,
physical selections, native capture/console logs, invocation Tcl, replay reports,
Truck STEP files and sampled-comparison reports. Native screenshots are omitted.
The SHA-256 of every archived artifact and the archive itself is recorded. A local
preliminary run without closed-edge preprocessing is retained under
`target/occt-capture-2026-10-11`; it is not this baseline.

Validation of the tooling: three Rust selection/ordering tests and two Tcl wrapper
scope/error tests pass. Both C++ tools compile with `-Wall -Wextra -Wpedantic`.
The A1 box fillet comparison agrees to about 3e-14 with equal integrated volumes.
A negative control comparing that fillet with its unfilleted box produces boundary
residuals 2.92893 and 4.14214, confirming that interior points are not mistaken for
coincident boundaries. The comparator is sampled, not a certified geometric bound.
