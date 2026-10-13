# Conical pole normals and mesh refinement

At a revolution endpoint on the axis, the normal must use the meridian tangent
rotated to the requested revolution angle. The old endpoint branches used the
unrotated tangent, returning the azimuth-zero normal around an entire cone tip.
Both endpoint branches now use `uder(u, v)`. Regressions compare the tip normal
against the regular derivative limit at 33 angles, with either generator end at
the pole, reversed surface orientation and affine transforms.

Interior mesh refinement now uses the same shared boundary positions as the
final mesh. Re-evaluating those positions on the surface could produce distinct
floating-point copies of one pole. A triangle between those copies has an
unreliable plane, despite a large apparent point-to-plane error. Refinement also
excludes facet planes with coincident vertices at the existing kernel tolerance;
these facets collapse when the shared mesh vertices are welded. The geometric
error threshold and refinement budget are unchanged.

The mesh regression checks a 270-degree cone with both exact shared pole points
and a 1.8e-10 endpoint perturbation. It limits refinement to the sampled boundary
size, verifies boundary positions, and checks over 18,000 surface samples against
the actual mesh for each variant, including samples in UV triangles that collapse
spatially. It does not use a plane-distance estimate for collapsed triangles.

Both new regressions fail against the parent production sources. The normal
regression first fails at angle pi/16; the mesh regression produces 2231 vertices
for 473 angular divisions, exceeding the 946-vertex boundary-size bound. Parent
failures are retained in `parent-normals.txt` and `parent-refinement.txt`.
No existing test or assertion was removed, relaxed or newly ignored.

These changes prepare the conical STEP input repair. The uncommitted cone-domain
prototype is diagnostic only and is not included in the production replay counts.
No public API or dependency changed, and no performance claim is made.

## Validation

The five affected release suites ran **921 passing tests**, one known
unchanged PCurve precision failure and 5 existing ignored tests across
143 test/doc targets. The failure still compares -2829663063.761872
with -2829663063.7618704; its parent reproduction is retained in
`../2026-10-13-cubic-endpoints/parent-precision-test.txt`.

All-target release Clippy for the five crates and benchmarks, single-threaded
action benchmark smoke, and the final production replay build passed. Exact
commands, exit codes and source/executable hashes are in `metadata.json`. The
frozen source hashes were unchanged after validation.
