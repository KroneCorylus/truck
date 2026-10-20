# Planar boundary loop order

Planar blend validation previously assumed boundary zero was outer. Imported faces can place a hole first, so valid outer trims and expanding hole rims were rejected. Both boundary validation and additive patch containment now locate the outer loop by projected area. Trim validation preserves the ORIGINAL outer-loop index, so an enlarged or invalid hole cannot be reclassified as outer. Polygon orientation, intersection, hole containment and clearance checks remain; no tolerance changes.

Parent 39b40375f0516f9af4f0de589e5dd195d0f87c7d: three boundary/patch regressions fail while two new collision guards and three existing tests pass. Both solid-level regressions also fail. All seven new tests pass after the fix. Boundary/patch tests cover all six orders of an outer loop plus two holes under identity and transformed coordinates; guards reject crossing, touching, enclosing, nested and outside patches/holes. Sixteen solid variants cover outer and inner rim chamfers, four loop orders, scaling/rotation/translation, independent material, genus2, closed meshes, shared STEP round trips, input preservation and complete original face correspondence. No assertion was weakened.

Four fresh native controls use a10×10×5 prism with two2×2 through holes: initial volume460. A single rim chamfer removes length*planar*axial/2; length10 on the outer rim and2 on the inner rim. Top-reference setbacks are(.2,.2) and(.2,.1). All native controls are valid and match analytic volume within1.2e-13. Both inner-rim native operations mutate their input; before/after raw BREP and properties are retained. Outer native inputs remain unchanged.

All four Truck controls preserve input and produce consistent, closed, valid single-solid STEP shapes. Their exact contact vertices and sampled bevel planes match the requested setbacks. Maximum material error 1.7053025658242404e-13; maximum native/Truck sampled distance 4.5288390936029406e-15; maximum plane residual 2.2204460492503131e-15. Finite samples do not establish a global error bound.

Validation:420 shapeops tests(one existing ignore),59 replay tests,13 capture tests,all-target release Clippy and single-thread actions smoke. Replay built last. Unchanged mesher102-test evidence is verified by source hashes. Implementation and proof precede clean full capture. No consumer/dependency change,push,PR or performance/full-parity claim.

Archive:90 byte-verified members;SHA256 502a5d48e1d204fd9117494bf395c464c4847034175ab00715ccf9cf4661bc0e.
