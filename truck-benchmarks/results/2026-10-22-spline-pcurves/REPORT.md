# Exact spline parameter curves on extruded supports

STEP spline parameter curves on extruded lines, ellipses and polynomial or
rational profiles now map directly into the existing modeling PCurve type.
Points, first and second derivatives and parameter ranges are preserved.
Profiles whose conversion changes their parameterization remain unsupported.
Non-finite and collapsed traces are rejected, including exactly constant traces
whose numerically evaluated derivatives can contain roundoff.

Finite extrusion bounds now include the complete sweep-coordinate hull of
linear and spline parameter curves. Positive profile control hulls are translated
across that interval. STEP round trips can move endpoint parameters a few
floating-point steps beyond a spline profile's knot range. Bezier blossom
controls bound that polynomial continuation; positive homogeneous weights are
required. No curve, vertex, consistency check or tolerance is changed.

Controls cover64 exact traces and8256 point/tangent/second-derivative samples,
including reversed traces, reversed profiles, affine transforms and nondefault
parameter ranges. Sixteen imported face controls check8256 boundary samples,
finite support coordinates and normals. The unchanged native C1 equal-distance
capture006 verifies exact trace conversion and its actual extruded face bounds.
The invalid-geometry controls continue to reject malformed traces.

Three regressions fail on parent e3568eb7917d87198a5f73c01754bc505fd8fca0: exact spline conversion,
complete parameter-curve bounds and the actual C1 fixture. Parent and candidate
use byte-identical tests and fixtures, without an adapter. Both sides clean
stepio artifacts before building; final candidate validation also cleans
shapeops to avoid stale shared-worktree caches.

Validation:452 release shapeops tests (one existing ignore),119 stepio,66 replay,
13 capture, all-target Clippy and single-thread actions smoke pass. Native
reference8, volume27 and distance24 checks pass. Unchanged geometry, modeling
and meshalgo evidence is reused by exact source hash; the known geometry
pcurve_derivation precision failure remains explicit. Replay is built last.

Fresh replay of all24 C1 sequence operations changes exactly one status:
equal_dist_sequence/C1/006 moves from input-import-failed to input-inconsistent.
Its missing representation and face-domain issue are fixed. Other supplied
edges still depart from their support by about2.012e-6 at knot-span samples,
above the strict kernel check. Seven angle-sequence inputs also still fail
bounded endpoint checks. The three previously constructed C1 result/comparison
records and all native inputs/results remain unchanged. This is an import
capability improvement, not a newly successful chamfer or comparison pass.
The authoritative full capture follows the implementation commit.

Controls archive:264 members, SHA256 aa7fbe356e9af55391187fa07ac832b0d266ab41de9e010050aa97f8122cb740.
See validation.json, parent.json, parent-trace-tests.txt, parent-bounds-tests.txt
and controls-summary.json. No consumer pin, push, PR, parity or speed claim.
