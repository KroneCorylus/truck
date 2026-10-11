# Revolution tessellation and smooth polar contacts

OCCT `blend/simple/D4` imports as a radius-150 spherical sector. Its original
Truck tessellation at tolerance 0.001 is closed, but loses 134.3034 volume units;
the independent curved-area error budget is about 61.21. The original revolution
sampler mistakenly takes the square root of an already-computed radial distance.

The sampler now uses the radial distance, includes the profile's sampling error
in that bound, and shares the error budget between profile and angular division.
Cancellation-resistant angular spacing also handles small relative tolerances.
Cylinder/spherical-meridian regressions check both cell diagonals at radii 0.01,
1, 150 and 900. The captured STEP input is retained byte-for-byte and checked with
the common solid harness against the independent spherical-sector volume
`pi/2 * (R^2 H - H^3/3)`, where `H = R sqrt(3)/2`.

The corrected grid exposed existing sensitivity to the tessellation seeds:

- Revolution nearest search rejected a point one ULP inside a fixed pole. Analytic
  sphere nearest search returned NaN azimuth at its poles. Both now return finite
  parameters, and the existing positive-seam convention is preserved.
- Surface Newton refinement tried to invert a singular Jacobian even when its
  initial point residual was already below 1e-12. Such a seed now returns directly.
- Tangent sphere/bore interference includes mesh seeds whose normal angle exceeds
  the old fixed cutoff. Refinement now uses a neighborhood scaled by local curvature
  and mesh tolerance, per-parameter damping, and a 1e-8 coincidence/normal residual.
  The neighborhood is a local refinement guard, not a global curvature certificate.
- Smooth poles have a singular parameter Jacobian but a well-defined curvature
  form. Meridians along a collapsed parameter line provide exact directional
  accelerations; their fitted quadratic form must agree across eight directions.
  Tests preserve tangent crossings, both operand orders and reversed orientations,
  and reject a cone apex. Unclassified singularities remain unsupported.
- Near-collinear parameter-space meridian facets can have a world-space normal
  perpendicular to the surface normals. Flipping them from the sign of rounding
  noise opens an otherwise closed mesh. Orientation now retains the UV winding
  when the normalized dot product is within the kernel's angular tolerance. Thin
  facets with a reliable opposite normal still flip. The rounded-box STEP and
  thin curved-loft regressions both remain unchanged.

Parent failures are recorded for the revolution chord, imported volume, revolution
pole, sphere pole, converged Newton seed, polar curvature, contact refinement and
ambiguous-facet regressions. Component replays restore the parent implementation
under the current private helper signature where needed; test assertions are
unchanged. The flat-contact regression additionally catches normal roundoff in a
zero-curvature refinement budget. No public API or diagnostic code changed.

## Cost and accuracy

`revolution_mesh` loads/preprocesses the fixture before timing, warms up once,
then measures five `triangulation(0.001).to_polygon()` calls with
`RAYON_NUM_THREADS=1`. Closure and volume are checked after timing. Before/after
binaries use identical geometry, fixture, tolerance and benchmark code.

The final paired rerun is in `mesh-before-final.json` and `mesh-final.json`:
351.747 ms versus 1668.584 ms median (4.74x), 35,774 versus 175,032 welded
vertices. The earlier sampler-only measurements, 336.176 versus 1684.456 ms,
remain in `mesh-before.json` and `mesh-after.json` for comparison.
Independent volume is 3,443,384.730538398; mesh volume improves from
3,443,250.427147223 to 3,443,356.700519925. The resulting 28.0300 error passes the
unchanged budget. This is an accuracy repair with about five times the meshing
cost on this large, tightly tessellated sphere, not a speed improvement.

## Validation and limits

Final release, all-target Clippy and action-smoke logs are retained alongside this
report. The full release command covers `truck-geotrait`, `truck-geometry`,
`truck-modeling`, `truck-meshalgo`, `truck-shapeops` and `truck-stepio`, without
skipping any target. It finishes with 865 passed, one failed and six existing
ignored tests across 130 target results. The sole failure is the pre-existing
PCurve property described below. All-target Clippy and the action smoke pass.

The existing `pcurve_derivation` property test also fails on the parent:
it compares two floating-point polynomial evaluation paths with absolute 1e-6
error at a derivative magnitude near 2.83e9. The recorded difference is a few ULP.
`pcurve-parent-existing-failure.txt` and `pcurve-replay-seed.txt` retain the failure
and replay seed. Its code/assertions were not modified, ignored or weakened.

D4's fillet STEP export remains a separate open issue: the intersection leader
crosses a quadratic fillet support's knot, causing a derivative discontinuity that
the current C1 cubic fitter does not split explicitly. The parked export probe's
original failure is included in `parent-regression.txt`; this batch only claims
its input tessellation repair. No full OCCT parity verdict follows from these
regressions. The broader captured outcomes are recorded separately in the audit.
