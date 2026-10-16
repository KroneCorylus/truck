# Native chord distances on longitudinal cylinder edges

Native equal and unequal chamfers measure the cylinder setback as a chord.
Truck's documented kernel API instead projects a tangent offset onto the cylinder.
Replay previously passed equal distances through unchanged and rejected unequal
requests on these supports. It now translates native intent on an exact straight
edge parallel to the cylinder axis, with the plane normal perpendicular to that
axis. Adjacent face order and the captured reference side remain authoritative.
Distance-angle requests remain unsupported for this configuration.

For cylinder radius R and native chord D, the circular angle is
psi=2*asin(D/(2*R)). A tangent offset d projects to that contact when
atan(d/R)=psi, hence d=R*tan(psi). Only 0<psi<pi/2 is supported by this local
projection branch; nonlocal and nonfinite conversions are rejected explicitly.
No kernel distance contract, geometry solver, tolerance or selection scope changes.

Three tests fail on parent 48e89fdb479e9312710e53ca64d80243785ae8ce and pass unchanged after the fix.
Six constructed variants cover equal distance, unequal distances with both
reference surfaces, scaling, rotation/translation, reversed face order and
reversed captured edge points. They check converted assignments, actual exported
contact vertices, independent material, unchanged input and shared solid/STEP
checks. A guard test rejects out-of-branch contacts without mutation. All previous
39 replay tests remain unchanged; the full replay suite now passes 42 tests.

The parent native replay constructs the equal-distance control with a sampled
boundary error of 0.0012747608554942742, exceeding the unchanged 0.001 comparison
threshold. Both unequal reference-side controls were unhandled. These original
results are archived beside the final replay evidence.

Twenty-one native controls now pass: three center-plane controls plus eighteen
controls spanning convex/concave edges, plane offsets -2,0,2 at radius5, equal
setbacks0.5 and unequal plane0.7/cylinder1.5, and both reference surfaces.
All preserve input, construct closed consistent geometry and export native-valid
single-solid STEP. Maximum sampled boundary difference is 2.007283228522283e-13; maximum
analytic STEP volume error is 6.8666849983856082e-11. Finite sampling is not a global bound.
The extra native angle captures retained with their original fixture run are not
included in these 21 replayed controls or claimed as supported angle cases.

The material oracle uses the section's exact line and circle integrals. For
S=(sqrt(R²-h²),h), plane contact P and cylinder contact Q, the removed area is
abs(cross(S,P)+cross(P,Q)-R²*psi)/2. The original convex section has area
R²*acos(h/R)-h*sqrt(R²-h²); the concave section is its complement in the rectangle.
Multiplying the remaining area by the extrusion height gives the exact volume.
For the central convex fixture this simplifies to
pi*R²*H/2 - H*(R²*psi-R*(R-a)*sin(psi))/2.
These geometric oracles are independent of both kernels' output volumes.

Validation: 42 replay tests, 13 capture tests, all-target release Clippy,
single-threaded actions smoke and final replay build pass. Production kernel
sources and core tests are byte-identical to the 4211cfe0 validation with
393 shapeops passes, zero failures and one existing ignored test; that unchanged
suite was not redundantly rerun. Source, fixture and executable hashes are recorded.
Archive: 459 byte-verified members, SHA256 8d61e722d4775157b4d644f3fe11bf8134ed4a37d98dbece233cd940f0707ff1.
A clean full corpus capture follows this implementation/proof commit.

Work left: distance-angle conventions and offset contact behavior need separate
investigation; nonlongitudinal curves, nonlocal contacts and unsupported surface
pairs remain outside this change. No consumer/pin changes, push, PR or speed claim.
