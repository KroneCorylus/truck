# Normal-section distance-angle chamfers

The previous replay rejected curved and tilted distance-angle sections. The new
`try_chamfer_solid_along_wire_angle` takes a reference face, chord distance and
angle. It solves the reference contact in the guide's normal plane, then
intersects the other support with a ray at the requested angle to the reference
tangent at its new contact. Shared fitting, trimming, history and solid checks
retain their constraints. The existing projection and two-chord APIs retain
their behavior. General continuation and corner collapse are not enabled for
the angle solver; an attempted planar-cap continuation did not resolve E5 and
is not part of the change.

Replay carries typed angle constraints across the full tangent contour when
any member needs a variable section. Existing constant-section conversions
retain their numerical assertions. The former unsupported-angle expectations
are rewritten to verify reference-side assignments and unchanged inputs; the
new replay regression additionally checks 48 successful constructions against
independent contacts and material.

Independent sections z=1+k*x^2 use k=0.8 or -0.4, distance0.1, angles30/45/60,
both reference faces, reversed wires and rigid transforms. A curved reference
fixes x from x^2+k^2*x^4=d^2, then w=x*(2*k*x+tan(a))/(1-2*k*x*tan(a))-k*x^2.
A wall reference fixes w=d and x=2*d/(cot(a)+sqrt(cot(a)^2-4*k*d)). The retained
material is 1+k/3-w*x/2+k*x^3/6. No-intersection, invalid-angle, empty-selection
and unrelated-reference inputs remain explicit failures with preserved inputs.

The geometry tests cover48 operations and12384 contact samples; replay covers
48 operations and3168 contact samples. Shared topology, closed tessellation,
independent signed volume and STEP checks pass. Twelve native controls agree
with analytic material within 3.3306690738754696e-16.
Their exported Truck boundaries agree within 6.7653169299045945e-12
in sampled comparisons; exported material differs by at most
3.5005331966431186e-13. All native inputs are byte-preserved.

On committed parent 594db26f5ca043e7e6cbb28f5ed051519841ae77, two geometry tests and one replay test
fail. The new geometry API did not exist there: an explicitly archived adapter
binds the same geometry assertions to the old chord API with d and d*tan(a).
The replay regression uses unchanged assertions and fails as unsupported.
Both parent and candidate shapeops artifacts are cleaned before building to
avoid sharing stale worktree artifacts. No assertion or tolerance is weakened.

Final validation:451 release shapeops tests (one existing ignore),115 stepio,
65 replay,13 capture, all-target Clippy and single-thread actions smoke pass.
Native reference8, volume27 and distance24 checks pass. Unchanged geometry,
modeling and meshalgo evidence is reused by exact source hash; the known parent
pcurve_derivation precision failure remains explicit. Replay is built last.

The targeted C1 angle operation constructs but still fails native sampled
distances; E5 reaches trim_faces and fails. The authoritative full capture must
follow the implementation commit. No parity, global-error or performance claim.
The sole consumer call remains try_chamfer_solid_along_wire at
PiezaCAD/cad-core/src/kernel/blend.rs:229; no consumer/pin changes, push or PR.

Controls archive:138 members, SHA256 2abe43614b64e6affcf8c9a71b625ac1d183ae1eb3034adfcce6d272920aa88d.
See validation.json, parent.json, parent-tests.txt, parent-replay-tests.txt and
controls-summary.json for hashes and exact results.
