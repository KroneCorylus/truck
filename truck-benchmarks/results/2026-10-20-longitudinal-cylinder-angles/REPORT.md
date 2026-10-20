# Longitudinal cylinder-reference contact angles

Replay now handles distance-angle requests referenced to the cylindrical face of
a longitudinal plane/cylinder edge. The explicit geometric contract is the acute
angle between the bevel and the cylinder tangent at their new contact, together
with the captured chord distance from the old edge. This definition is tested
for both convex material and cylindrical cavities. It is not an assertion that
all native analytic branches implement that definition consistently.

Construction: convert chord D to sweep psi = 2 asin(D/(2R)), on the existing
local branch 0 < psi < pi/2. Rotate the seed radius into the face interior to get
the new contact Q. With outward cylinder normal N and inward tangent T at Q,
the ray toward the plane is -T cos(alpha) - N sin(alpha). Intersect that ray with
the planar support and require a finite positive ray parameter and planar inset.
The existing chord-to-tangent converter and core two-distance solver then apply.
Core APIs, diagnostics, tolerances and dependencies are unchanged.

Tests were written on parent fd791dea687a56947e8343e8e489de8478fd7e82 before implementation:
three tests fail as unsupported, while all four existing tests in the group pass.
The previous rejection of a valid cylinder request is intentionally replaced by
positive coverage and an oversized-chord guard, on the parent first. No assertion
was weakened. The test module hash is retained in parent.json.
Twenty positive variants cover offsets -2/0/+2 at 30 degrees, central 15/45-degree
requests, convex/concave material, scale/rigid transform, reversed face order and
reversed captured edge samples. Checks include actual contact vertices, outward
bevel plane, independent material, shared solid/STEP and input immutability.

Independent section oracle: R = 5, height H = 10, plane y = h,
seed S = (sqrt(R²-h²), h), and Q at angle asin(h/R) + psi.
Construct P on y = h using the specified angle at Q. The removed section area is
abs(Sx*h - h*Px + Px*Qy - h*Qx - R²*psi)/2.
Initial convex area is R² acos(h/R) - h*Sx. The cavity fixture has area
16*(8-h) minus that circular segment. Volumes scale by H and cube under scale.
This calculation is independent of either kernel's output-volume measurement.

Ten fresh native controls preserve their original BREP bytes and produce valid
single solids. All ten Truck outputs match requested chord, contact angle,
contacts and independent material, and export valid single-solid STEP.
Maximum Truck angle error: 5.872635711057228e-12 degrees.
Maximum contact-position error: 1.6608936448392342e-13.
Maximum STEP material error: 7.0031092036515474e-11.
Four native controls agree geometrically. Six retain raw discrepancies:
convex 30-degree requests measure 47.2538531 degrees at offsets -2/0 and
72.3288912 degrees at +2; convex 15/45 measure 32.2538531/62.2538531 degrees.
The concave +2 control measures 14.3044334 degrees for a 30-degree request.
These are measured from the actual native bevel plane and cylinder contact.

A separate source model reproduces the pinned native planar setback for all ten
controls (maximum contact error 7.9936057773011271e-15). Its unsigned sinA1 and
half-chord tangent addition explain the branch differences; source path/hash are
recorded in metadata.json. This model is only diagnostic: the Truck construction
uses the stated contact-angle geometry, not the native branch formula.
No raw comparison verdict is overridden to call these cases matching.

All 49 release replay tests, 13 capture tests, all-target release Clippy and
single-threaded benchmark smoke pass. Replay was built last. Core sources and
integration tests are byte-identical to the conic checkpoint, whose full release
validation passed 400 shapeops tests and 98 meshalgo tests with one existing ignore
each. That core validation is reused with file hashes, not claimed rerun.
Archive: 199 byte-verified members; SHA256 a5532e851f793389098a6c76cb5bfbf83d79a926b841e83ebfb92b54182744c1.
Finite samples do not prove a global boundary-error bound or full parity.
A clean full capture follows this implementation/evidence commit.
No consumer changes, push, PR or performance claim.
