# Exact extrusion pcurves and STEP profile orientation

Native J6/J8 inputs acquire linear UV seam curves when closed edges and faces
are split. The U endpoints differ by one floating-point unit, so they cannot
be replaced by constant-U lines without changing their geometry. Modeling
conversion now retains the exact UV trace on extruded lines, ellipses,
polynomial splines and rational splines. The existing finite/regular pcurve
validation is shared with elementary supports. Profiles whose conversion
changes their parameterization remain unsupported.

The solid regressions also exposed an existing STEP export defect: a reversed
elliptical profile writes the same ellipse entity, but the extrusion face sense
did not account for its reversed traversal. Exported extrusion senses now
compose the curve sense with the surface processor's orientation. Partial
non-elementary elliptical profiles use exact rational arcs to preserve their
bounds; full ellipses keep their periodic chart.

## Regression proof

Curve tests cover four profile types, shifted spline knot ranges, rational
weights, reversed profiles and traces, affine frames, nearly constant-U seams,
genuinely diagonal traces and constant-V traces. They compare positions and
both derivatives at 129 stations per variant. Nonfinite and physically
collapsed geometry is rejected, as are profiles requiring reparameterization.
The export regression checks partial/full ellipses, reversal and rigid motion
for endpoint, position and oriented-normal preservation.

Native input and rim-fillet regressions use unchanged STEP bytes. Both cases
are checked at original and small scales, with rigid motion and reversed wire
selection. They verify input immutability, geometric consistency, topology,
closed meshes, independent volume and STEP round trips. Generated blends also
check analytical rolling-circle radius and normal direction, plus tangency to
both the cap and lateral support. Construction tolerance is 1e-7 for these
strict geometric checks; official replay tolerance remains 0.001.

Independent volumes derive from the original ellipse and extrusion dimensions,
not the native results. A divergence-theorem integration of the analytical
rolling-circle surface gives **1560464.3531795258** for J6 and
**772406.7218958114** for J8. The fixture README derives the formula; its
standalone calculation converges to within 1e-9 across three quadrature orders
and matches a separate exact formula for the perpendicular circular case.

The final new curve, export and native-solid regressions all fail on the parent
production sources. The earlier export-domain failure is also retained.
Existing tests and assertions are unchanged; none was removed, relaxed or
newly ignored.

The generic STEP surface implementations now require `C: StepCurve` to read
the profile's export sense. The changelog gives the migration. A consumer
search found only concrete PiezaCAD exports, requiring no call-site changes.
No consumer files or dependency pin were changed. No dependency was added,
and no performance claim is made.

## Validation and native replay

The complete stepio and shapeops release suites passed: **472 passed, 0 failed
and 1 existing ignored test**, across 90 test/doc targets. All-target release
Clippy for both crates and benchmarks, single-threaded action benchmark smoke
and the final replay build passed. Commands and frozen hashes are recorded
in `metadata.json`. All 28 targeted archive files were byte-verified; native
input/result bytes and fixture bytes additionally match the committed capture.

Both targeted fillets preserve their input, retain consistent geometry, close
the Truck mesh and export native-valid single-solid STEP. The two sampled
maximum distances are **0.000911689211 / 0.00181454062** for J6 and
**0.0917580035 / 0.0890851236** for J8. These exceed the combined 0.001
comparison threshold. No full parity claim is made.

Independent native STEP volume integration differs from the dimension-based
formulas by **0.000907006** for J6 and **0.00198204** for J8. Maximum native
edge/vertex tolerances are 0.0000117136 and 0.00000483693. These checks passed
the existing independent checker's volume-error limit of 0.01 and geometry
tolerance limit of 0.001.

The independent C++ checker samples each trimmed B-spline blend at interior
stations and recovers the analytical rolling center from the oriented normal.
Across 3,969 samples on each native reference, the maximum radius residual is
**0.00179636117** for J6 and **0.0920950559** for J8. The maximum normal-vector
residuals are 0.00248208501 and 0.0681262877. These are sampled measurements,
not certified global bounds. They give independent evidence that the native
fitted surfaces depart from the ideal rolling-circle geometry; comparison
thresholds and regression expectations are unchanged.

The same checker samples 7,938 stations on each Truck STEP export. Maximum
radius residuals are **0.00000694862 / 0.0000395908** and normal-vector
residuals are **0.0000590274 / 0.000160148** for J6/J8 respectively. These
checks use the ordinary replay construction tolerance, not the tighter test
tolerance. The final replay executable matches the one used for these results.
