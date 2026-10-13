# Complete conical STEP support domains

The lower cone in native H6/H7/H9 retains a unit generator near radius 200,
while its boundary reaches the apex roughly 200 generator parameters away.
Ordinary points invert on the infinite cone, but the apex is not recognized as
a bounded generator endpoint, leaving the imported solid inconsistent.

The importer now projects complete boundary control hulls onto the cone's local
axis to bound its generator. It preserves the existing interval on the selected
nappe and adds tolerance padding, but stops at the analytical apex. A hull that
straddles the apex is rejected by this fitter rather than clipped. The fitted
pole is projected exactly onto the local axis. Transform, orientation and the
conical surface representation are preserved. The boundary-hull collector is
shared with the existing cylinder/extrusion fitter without changing its bounds.

STEP output now reconstructs a cone's slope as `dx/dz` and its reference-plane
radius as `x - slope*z`. The former unit-generator assumption would export a
changed cone after extending or shifting its generator.

## Regression proof

The importer regression covers both cone nappes, both slope signs, reversed
surface orientation and affine frames. A quadratic meridian trim has an interior
extent beyond its endpoints. All 16 variants check bounded inversion, complete
curve inclusion, positions, normals and a generator endpoint on the apex.
Another regression rejects a control hull spanning both nappes. The STEP export
regression checks 12 shifted/scaled generator and rigid-frame variants for
physical position and normal preservation.

Native input regressions verify complete boundary domains, cone normals,
consistency, topology, closed meshes, independent volume and STEP round trips
at small/original scales and under rigid motion. The recovered neck fillet
additionally checks unchanged input, reversed selection, generated-face count,
its rolling-circle radius and normals, both contact limits, volume and STEP.
That regression requests construction tolerance 1e-6 for its radius check;
the native replay retains its standard 0.001 tolerance.

Independent volumes come from the source dimensions, not native result geometry:
**13155419.236907259** for the input and **13160701.562723719** after rounding.
The fixture README derives the cone/cylinder volume and the added circular
corner's first moment. It also records the unchanged native STEP provenance.

The bounds, export and both native-solid regressions fail against the parent
production sources. Exact parent failures are retained here. Existing assertions
and tests are unchanged; none was removed, relaxed or newly ignored.

## Targeted native replay

H6 constructs with unchanged input, consistent geometry, a closed Truck mesh
and native-valid single-solid STEP. Its native reference volume is
13160701.562721223; the comparison harness reports Truck STEP volume
13160701.556814155. The two maximum sampled distances are
**0.0000191017411** and **0.0000191019487**, both below 0.001.

Independent native STEP integration gives 13160701.555398194, differing from
the analytical formula by **0.0073255245**. Its maximum edge/vertex tolerance
is **0.0000168756318**. Different native integration paths yield slightly
different last digits; both results are retained as separate evidence.

H7 and H9 also pass import consistency and reach the existing unsupported
three-edge nonplanar-corner path. Both failures preserve their inputs. These
are sampled and targeted checks; the full clean-commit capture follows the
production commit. No public API or dependency changed, and no performance
claim is made.

## Validation

The complete stepio and shapeops release suites passed: **466 passed,
0 failed and 1 existing ignored test**, across 88 test/doc targets.
All-target release Clippy for both crates and benchmarks, single-threaded
action benchmark smoke and the final replay build passed. Commands and frozen
source/executable hashes are in `metadata.json`. All **28** targeted replay
archive members were byte-verified, and all native input/result artifacts and
the fixture were additionally verified against the original committed archive.
The final replay executable matches the one used for the targeted results.
