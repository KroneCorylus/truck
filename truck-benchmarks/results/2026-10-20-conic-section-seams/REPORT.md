# Projected conic section seams

An exact conic generator can span a full period even when its face covers only a quarter cylinder. Independently finding endpoint parameters and cutting between their minimum and maximum can select the major arc across the parameter seam. The endpoints and support membership are correct, but the interior section runs around the far side of the cylinder. The old cut also mishandles reversed bounded conics.

Projected conics now recover intrinsic angles from the affine inverse. A full-period support uses the local connecting arc; a bounded source preserves its explicit interval, including arcs longer than half a turn. Endpoint and bounds checks remain, and non-conic trimming is unchanged. This is a local section contract, not a general solution to ambiguous surface intersection branches.

Parent 8941173b5533b4e36dcdd1c945b9b0a3ebf5bee5: two arc regressions fail and the solid regression produces an open tessellation. The existing section test and new bounded-long-arc guard pass. All four new tests now pass. Arc checks cover three scales, source reversal, both traversal directions, seam crossings, exact expected points and membership in both supports. Two solid variants place the generator seam inside the requested chamfer, checking independent material, genus0, closed meshes, shared STEP round trips, contact vertices, input preservation and original face correspondence. An initial sequential test draft failed at an unrelated top-chamfer precondition; the final parent test isolates the bottom-edge section directly. No existing assertion was changed.

Three fresh native controls chamfer the bottom radial edge of an R5,H10 quarter cylinder with planar setback a=.2 and axial setback b=.2,.1,.2/sqrt(3). Removed material is the integral of (b-b*y/a)*sqrt(R²-y²), from y=0 to a. All native results are valid single solids, preserve their inputs, and match analytic volume within3e-12. Before/after native artifacts are retained.

All three Truck controls preserve input and produce consistent, closed, valid single-solid STEP shapes. Exact contact vertices and sampled bevel planes match requested setbacks. Maximum material error 2.4442670110147446e-12; maximum native/Truck sampled distance 1.8152278718840466e-14; maximum plane residual 2.2204460492503131e-16. Finite samples do not establish a global error bound.

Validation:424 shapeops tests(one existing ignore),59 replay tests,13 capture tests,all-target release Clippy and single-thread actions smoke. Replay built last. Unchanged mesher102-test evidence is verified by source hashes. Implementation and proof precede clean full capture. No tolerance, consumer or dependency changes, push, PR, performance or full-parity claims.

Archive:69 byte-verified members;SHA256 be49e3fc0f3b54da046ae4e9c570ff670bc497bcde7be1c7bfc721c0f6dd6702.
