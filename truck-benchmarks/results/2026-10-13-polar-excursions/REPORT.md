# Isolated polar seams and fitted trim endpoints

Native J2's spherical cap imports with a meridian edge traversed twice in
opposite directions. It ends at an otherwise isolated south pole. The existing
general seam splitter cannot find two loop-interior vertices for this boundary,
leaving a nonsimple wire that the modeling topology correctly rejects.

Preprocessing now removes this isolated retraced excursion while retaining the
cap ring. It requires opposite adjacent uses of the same edge, exactly two
uses in the shell, an isolated terminal vertex, a nonempty remaining loop,
and agreement with a collapsed periodic surface direction. Quarter-period
positions must remain at the pole, preventing a stationary tangent on a
noncollapsed periodic curve from being mistaken for a pole. General seam
divisor selection and its existing safeguards are unchanged.

The new fillet regression also found a STEP round-trip meshing gap. Artificial
polar seams regenerated their endpoints from surface parameters, even when a
shared fitted boundary point lay slightly off that surface. In the small J2
case, that produced two trim points about 1.16e-6 apart and four open mesh
edges. The seam polyline now retains its supplied endpoint positions. Interior
sampling, welding tolerance and mesh assertions are unchanged.

## Regression proof

Guard tests cover both poles, boundary rotation and reversal, nonpolar and
off-surface endpoints, shared vertices, edges reused by another face, empty
remaining boundaries and a stationary but noncollapsed periodic direction.
Existing seam-endpoint, incomplete-graph and failed-conversion tests still pass.

The mesher regression uses a fitted trim displaced 6e-6 to 10e-6 from an
ellipsoid, under rigid motion, at both poles, in both orientations and from
three boundary starts. It checks consistent winding, exactly one boundary,
all 64 original boundary stations and their positions to 1e-12. The parent
has 66 boundary stations, exposing the artificial seam gap.

Native input and fillet tests check geometric consistency, topology, closed
mesh, independent volume and STEP at small/original sizes, including rigid
motion and reversed selections. The fillet also checks unchanged input,
generated-face count, analytical rolling radius and normal direction, and
both cap and sphere contact limits. The input reduces to two vertices, two
edges and two faces without changing its material.

The fixture README derives input volume **493199.54082823097** and fillet
volume **492592.0503127781** directly from the sphere/cap dimensions and a
circle integral. Regression construction tolerance is 1e-7; official native
replay tolerance remains 0.001. Both solid regressions and the mesh regression
fail on the parent production code. No existing test or assertion was removed,
relaxed or newly ignored.

No public API or dependency changed. No performance claim is made. Final
validation commands, results, source/executable hashes and targeted native
measurements are recorded in `metadata.json` and the adjacent logs.

## Validation and targeted replay

The complete meshalgo, stepio and shapeops release suites passed: **574 passed,
0 failed and 2 existing ignored tests**, across 102 test/doc targets. All-target
release Clippy for those crates and benchmarks, single-threaded action benchmark
smoke and the final replay build also passed. The final executable and source
hashes match the validated files.

The official J2 replay constructs with unchanged input, consistent geometry,
a closed mesh and native-valid single-solid STEP. The maximum sampled distances
are **0.00000743003 / 0.00000753546**, both below 0.001. Independent STEP
volume integration differs from the formula by **0.000980468**, and maximum
edge/vertex tolerance is **0.00001**; both pass the existing independent
checker's limits. The comparison harness's volume uses a different integration
path; both measurements are retained separately.

All **12** targeted archive members were byte-verified, and all native
input/result and fixture bytes were checked against the original committed
capture. A full clean-commit capture follows this implementation commit.
Successful sampled comparison remains limited evidence, not full parity.
