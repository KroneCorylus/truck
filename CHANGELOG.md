# Changelog

Every change to this fork, newest first. A line starting with `Breaking:` changes a public
API and says how to migrate. The fork starts from upstream `ricosjp/truck` at `efe39930`
(2026-08-31); earlier history is in git.

## Unreleased

- Make `Shell::connected_components` deterministic. Faces sharing an edge form a component, every face appears once, and faces and components keep the shell's order (a component comes where its first face does). It used to follow the hash order of face addresses, so boolean results and sweeps changed face and shell order from run to run, and sequential booleans drifted by an ulp. A face without edges is now its own component instead of being dropped, and a face listed twice is kept twice.

- Speed up boolean interference and classification without changing results. The mesh collision sweep skips all pair work for triangles outside the other mesh's bounding box and no longer copies its active list per triangle; each interference-polyline vertex runs its tangency tests from one nearest-parameter search per surface instead of up to three; inside/outside ray casting reuses the per-shell meshes of the nesting check; `KnotVec::floor` binary-searches. Medians of two alternating runs with one Rayon worker: cylinder union and intersection 5–13% faster, NURBS cylinder intersection 3–10%, sequential subtraction of 30 and 100 holes 3–5% (1.297 s to 1.254 s), batch subtraction within 3%. In the stage diagnostic at 100 holes cut one by one, interference drops from 179 to 125 ms and classification from 13.2 to 1.4 ms. An order-independent comparison of 18 boolean results with the parent shows identical geometry, up to the 1-ulp run-to-run variation the parent already has.

- Breaking: `KnotVec::try_from`, `KnotVec::from_single_multi` and `KnotVec` deserialization reject knot vectors containing NaN with `Error::NotSortedVector` (`TRUCK_GEOMETRY_NOT_SORTED_VECTOR` now also covers NaN). They used to be accepted and evaluated to meaningless basis values; sorted knot vectors without NaN are unaffected. The app builds no knot vectors directly and STEP numbers cannot express NaN, so no migration is needed. This lets `KnotVec::floor` use a binary search.

- Replace upstream's README, the generated crate READMEs and upstream's release notes with `AGENTS.md` and a README for this fork, and remove `readme-generator`.

- Divide affine B-spline and NURBS rulings only along their nonlinear boundary curves, and seed each intersection-curve sample from the previous one instead of a presearch grid (R7). This makes exact extruded-surface booleans interactive without changing geometry or tolerance: the reference NURBS cut drops from a 106.8 ms to a 29.7 ms median. A seeded projection is accepted only if it lands on the sample, stays inside both surfaces' ranges, and is named by its own parameters; otherwise the unseeded search still runs, so results at periodic seams and near-singular solves are unchanged.

- Support two-edge fillets at orthogonal convex corners with exact cylindrical miter joins, and add `chamfer_solid_edges` / `try_chamfer_solid_edges` for equal-distance planar corner chamfers (R11). Preserve unselected edges, return face history, and distinguish unsupported junctions from sizes that do not fit. Regressions cover selection order, rigid transforms, analytical volumes, booleans through the corner, and prepared STEP round trips.

- Make rolling-ball fillet approximation propagate failed adaptive contact solves and singular tangent frames without panicking (R10). Reject folded contact offsets and document the unsupported equal-radius sequential rim case; preserve smaller-radius sequential blends and face history.

- Fix fine revolution meshes (R9): measure interior refinement against triangle planes so flat polar caps stay compact and closed. Weld attribute components directly from spatial buckets without storing a quadratic graph of equal normals, preserving the strict tolerance and transitive connections.

- Fix coplanar split-cut booleans by classifying overlap after all face cuts. Process each solid's full oriented boundary set, preserving disconnected bodies, cavities and nested islands regardless of shell order. Add `solid_components` for exterior/cavity grouping before STEP export and `subtract` for empty-safe subtraction. Reject detected invalid nesting, invalid result topology, invalid tolerances and unrepresentable whole-space results with `None`.

- Point the resources submodule at its published commit and keep the STEP fixture provenance in truck-stepio, fixing fresh dependency fetches.

- Reuse intersection-curve parameter divisions across unchanged clones and boolean calls, reducing the 100-hole plate benchmark from 81.0 s to 11.3 s with identical geometry.

- Add `fillet_edges` for convex planar shells, joining equal-radius straight-edge fillets with exact spherical corners where three edges meet.

- Support sampled tangent-crossing boolean branches and boundary contacts: Steinmetz intersection and union, sphere-in-bore subtraction and rod-in-slot subtraction.

- Reject detected second-order tangent crossings before boolean curve lifting, so the tested Steinmetz operations return `None` instead of panicking; record the four-branch loops-store limitation.
- Record boolean failures at tangent crossings with analytic-volume tests for Steinmetz intersection and union, a sphere in an equal-radius bore and a rod in an equal-width slot.

- Add silhouettes to `truck-hlr`: the outlines of curved faces, traced on their tessellation and exact for cylinders, cones and spheres, drawn and classified like edges, with pieces on a grazed face decided by how it bends; the curve division of a `Processor` in `truck-geometry` accepts a singular transform, an ellipse seen edge-on.
- Add `truck-hlr`: hidden-line projection of a solid onto a plane, its edges as `truck-drafting` curves split at their crossings and classified visible or hidden.
- Add `local::thicken` to `truck-shapeops`: a planar shell made into a slab of a thickness.
- Add `local::shell` to `truck-shapeops`: a convex-edged solid hollowed inward to walls of a thickness, with chosen faces opened.
- Let `local::move_faces` in `truck-shapeops` re-intersect the moved surfaces when the faces cannot move rigidly, and add `local::offset_faces`.
- Add `Surface::offset` to `truck-modeling`: planes, extruded circles and revolved lines and circles offset within their own kind, so cylinders, cones, spheres and tori stay exact.
- Add `local::draft` to `truck-shapeops`: planar and cylindrical faces tilted about a neutral plane, re-intersected with their neighbours.
- Let `local::replace_surfaces` in `truck-shapeops` take cylinders and cones as neighbours, keeping the curves of edges between unreplaced faces; fix the exact plane-against-revolved-surface intersection, whose angle is the second parameter.
- Add `local::delete_face` to `truck-shapeops`: a fillet or chamfer face between planes removed and the sharp edge restored.
- Add `local::replace_surfaces` to `truck-shapeops`: faces given new surfaces and re-intersected with their neighbours, for planes.
- Add `local::intersect_surfaces` and `parameter_domain` to `truck-shapeops`: the curves where two surfaces meet, exact for planes against planes, cylinders and cones; `truck-shapeops` now depends on `truck-modeling`.
- Add `local::move_faces` to `truck-shapeops`, moving a group of faces such as a hole through a plate rigidly along its neighbours.
- Follow spline path edges in `builder::sweep_along_wire` by rotation-minimising frames and a loft through the profile copies; the sweep takes the tolerance of that division.
- Add `builder::sweep_along_wire` and `sweep_wire_along_wire` to `truck-modeling`: a profile carried along a tangent-continuous path of lines and circular arcs, exact.
- Add `builder::try_loft_shell`, `try_loft` and `align_sections` to `truck-modeling`: a NURBS skin through aligned sections whose iso-curves are the sections themselves.
- Let `fillet_along_wire` in `truck-shapeops` take a radius that varies along the chain, as any `ScalarFunctionD1` of the wire parameter.
- Report the faces, edges and vertices the STEP reader of `truck-stepio` leaves out, with the entity that caused it, instead of dropping them silently.
- Add `Table::try_from_step` and `Table::unsupported` to `truck-stepio`, reporting why a file is not STEP, which records failed and which entity types are not implemented.
- Add an ignored benchmark of set operations on a plate with holes to `truck-shapeops`, reporting wall time per stage.
- Document what the `tol` of `and` and `or` in `truck-shapeops` controls, and test that the result does not depend on it.
- Write revolved cylinders, cones, spheres, tori and flat ends of `truck-modeling` as the elementary STEP surfaces, with the right `same_sense`.
- Fix parameter searches on an inverted `Processor` surface reading the hint in the wrong parameter order.
- Add `Surface::elementary` to `truck-modeling`, recognising planes, cylinders, cones, spheres and tori with their parameters.
- Add `try_mapped` to `CompressedShell` and `CompressedSolid`.
- Convert curves and surfaces read from STEP into the `truck-modeling` enums, keeping conics and elementary surfaces exact.
- Read the 3D curve of a STEP `SURFACE_CURVE` even when a pcurve is its master representation.
- Write swept circles of `truck-modeling` as `CYLINDRICAL_SURFACE` in STEP output.
- Add `Surface::Extruded` to `truck-modeling`, so `tsweep` keeps swept circles and arcs exact.
- Add `Curve::Conic` to `truck-modeling`, so `builder::circle_arc` keeps circles and arcs exact.
- Fix parameter searches on `TrimmedCurve` and `Processor` answering outside the trimmed range of a periodic curve.
- Support set operations between solids with coincident faces, such as stacked boxes and flush pockets.
- Support set operations between solids whose faces only touch along a curve or an edge.
- Fix set operations dropping whole faces of an inverted operand.
- Fix intersection-curve edges of set operations wiggling and failing to project near cylinder seams.
- Classify leftover face pieces of set operations from an interior point, and skip face pairs with disjoint bounding boxes.
- Allow `fillet_along_wire` to end at vertices with more than three faces.
- Add `fillet_along_wire` to `truck-shapeops`.
- Remove the unfinished multi-edge fillet prototype from `truck-shapeops`.
- Add `simple_chamfer` and `chamfer_with_side` to `truck-shapeops`.
- Fix `fillet_with_side` for side faces with inverted orientation.
- Add a geometric test harness to `truck-shapeops` and boolean tests for tangent cases.
