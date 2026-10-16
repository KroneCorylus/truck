# Cylindrical reference faces in chamfer capture and replay

Face-dependent chamfers previously captured only reference planes. A cylindrical
reference was null, leaving an otherwise supported perpendicular plane/cylinder
rim unhandled. Schema 2 now optionally records reference_cylinder with origin,
axis and radius, retaining reference_plane:null. Replay identifies exactly one
adjacent seed face by radius and axis line, then uses the existing oriented-contour
side propagation. Axis sign and origin position along the axis are irrelevant.
Missing references and unsupported support pairs remain explicit failures.
No geometry solver, public API, fitting tolerance or selection scope was changed.

Three new replay regressions fail on parent 48cb55cddfcfc4f5aa6457a7c1948c543c531ab7 and pass unchanged
after the fix. They cover unequal distances and distance-angle requests, original
and transformed/scaled solids, reversed samples, shifted/reversed/scaled cylinder
axes, wrong identities, malformed descriptors and ambiguous plane/cylinder inputs.
Eight constructed variants check side assignment, immutable input, analytic
material, closed topology/mesh and robust STEP through the shared harnesses.
The fixture is byte-preserved from the earlier native shared-end proof; synthetic
reference and transform variants are explicitly identified in its README.
No existing assertion or test was changed, ignored or loosened.

Three fresh native OCCT 7.9.3 controls explicitly select the cylindrical face.
Two use unequal distances at scales 1 and 0.1; one uses a 60-degree distance-angle
request. Native validity, one contour, analytic material and original BREP bytes
are checked before requiring the new descriptor. That same native harness fails
against the old plugin because reference_cylinder is missing, then passes against
the new plugin. All other captured input fields and native result properties are
identical between the parent and fixed runs.

For radius R, height H, radial inset a and vertical setback b:

    remaining = pi R² H/2 - pi/2 * (R a b - a² b/3).

The cylinder reference assigns b first and a second. A distance b and angle alpha
on this perpendicular rim give a = b tan(alpha). These formulas use dimensions
and requested parameters, independently of either kernel's output.
All three Truck replays construct valid single-solid STEP results and preserve
input. Maximum sampled boundary difference: 6.9669054156017117e-09; maximum Truck STEP
analytic volume difference: 1.9356150460225763e-08. Sampling is not a global bound.

Validation: all 36 replay tests (33 existing plus 3 new), 13 capture tests,
all-target release Clippy, single-threaded action smoke and final replay build pass.
The geometry solver is unchanged; the previous full 391-pass shapeops run remains
recorded at the shared-chamfer-ends checkpoint. This group did not rerun that suite.
Source and executable hashes are recorded in metadata.json.
Archive: 114 byte-verified members, SHA256 94eff77cea2d2328eecaf885292e8f3555e9f3d94325452457e58bd16f82632f.
A clean full corpus capture follows this implementation/proof commit.
No full-parity, performance or existing-corpus recovery claim is made here.
No consumer/pin changes, push or PR.
