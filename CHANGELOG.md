# Changelog

Every change to this fork, newest first. A line starting with `Breaking:` changes a public
API and says how to migrate. The fork starts from upstream `ricosjp/truck` at `efe39930`
(2026-08-31); earlier history is in git.

## Unreleased

- Add `truck_shapeops::thread::thread_groove` for single-start internal/external,
  right/left-handed 60-degree helical B-rep cutters with validated pitch, depth,
  turn count, and sampled helix accuracy. Exact rational radial sections preserve
  cylindrical contacts; cutters participate in ordinary booleans and STEP export.
- Tessellation anchors sampled edge endpoints to their shared topology vertices,
  closing numerical seams at fitted intersections in both ordinary and compressed solids.
- Preserve exact rational plane sections and constant-parameter sections of ruled
  NURBS during boolean intersection conversion. This avoids oscillating fitted
  boundaries on helical flanks and allows closed fine meshes and STEP round trips.


- Accept near-limit all-edge fillets and miter chamfers on boxes without discarding their tiny remaining faces. Stop surface inversion on an exact physical residual, distinguish half-space roundoff from modeling tolerance, retain small tessellation triangles, and budget sphere chord error across both parameters. STEP export retains exact circular arcs and aligns spherical octant parameters to their boundaries. Regressions cover analytic volume, closed meshes, rigid transforms, reversed selections, unchanged input, and strict FreeCAD STEP validation through the consumer's test script.

- Support straight plane/cylinder lips and connected channel rims in both fillet and chamfer, including unequal chamfer distances. Preserve analytic supports, traverse adjacent end-face patches, and join blend surfaces with exact conic projections and cylindrical miters; tangent contacts and selection ordering remain stable under rigid transforms and scale changes. Regressions check independent section/volume expectations, closed meshes, STEP round trips, and unchanged input on rejected sizes.

- Validate exact cylindrical extrusion boundaries through analytic cylinder parameters while retaining knot-span sampling, physical-space tolerance and finite-domain checks. This avoids false geometry failures on small or transformed conic strips and rejects curves whose initial endpoint lies outside the extrusion. Clamp analytic angular endpoint roundoff only when the evaluated point remains within geometric tolerance.

- Keep Boolean intersection samples along one face-boundary edge inside a single edge, while retaining boundary crossings, contacts and topological endpoints. Triangle intersections now recognize seam endpoints within geometric tolerance before rejecting same-side segments, so adjoining patches report the complete seam despite roundoff. Circular channel cuts no longer turn mesh samples into extra rim edges and vertices; regressions cover face order, exact volume, closed meshes and the application's later fillet references.

- Correct odd derivatives of reversed `Processor` curves. Fillets on Boolean-created hole rims now retain consistent exact surfaces and outward normals. Regressions pair Boolean and extruded holes, through and blind cuts, reversed wires, analytic volumes, closed meshes, STEP, upstream edits and saved documents.
- Support oblique planar fillet ends and convex/concave two-edge fillet transitions; construct exact elliptical trims and toroidal patches while preserving the remaining sharp edge. Mixed two-edge chamfers with different end contacts now use a shared triangular transition, including unequal distances and transitions at both ends.
- Keep tessellation grid points out of narrow curved-boundary regions where a UV chord and its spatial chord disagree about sidedness. Boundary sampling and surface normals remain unchanged; regressions require closed, consistently oriented meshes for the affected fillets.
- Support planar concave shelling, including inward/outward and open/closed L- and U-shaped sections. Validate offset loops before constructing openings so collapsed walls still fail without changing the input.
- Recognize exact rational quadratic cylindrical strips and offset them without fitting approximate geometry. Preserve angular and height domains when rebuilding cylindrical boundaries, and intersect tangent plane/cylinder/cone neighborhoods analytically. Rounded box and L-section shelling and drafting now compose after vertical-edge fillets.
- Breaking: `try_draft` now propagates to connected tangent planar/cylindrical walls parallel to the pull direction. Such walls previously remained fixed, causing valid rounded-wall drafts to fail. Non-tangent joins stop propagation; callers retain the same selected-face API and no signature migration is required.
- Correct cone angular domains and trimming of reversed surface-intersection curves in annular drafts. Assign Boolean hole loops to their innermost containing exterior; annular cuts now preserve central shaft material with both extruded and revolved tools.
- Export affine-transformed circles as exact ellipses using principal axes, including nonorthogonal transforms used by planar extrusion limits. Tests cover circle, ellipse and holed-profile STEP round trips.
- Count each closed-wire vertex once when aligning loft sections. A repeated closure vertex biased the correspondence cost and could reverse a translated small circular section; shifted, reversed and rotated sections now retain their intended winding.
- Refine shared shell edge sampling when coarse spatial triangles reverse their parameter-space winding. At most seven tolerance halvings preserve shared boundaries across serial, parallel and compressed tessellation paths. Narrow curved loft walls now remain closed at ordinary display tolerances; meshes with consistent winding retain their first-pass tessellation.
- Preserve UV/XYZ correspondence when a boundary crosses a surface pole, and discard singular projection hints before searching the next point. Revolved sphere strips now tessellate consistently without jumping to a reflected meridian; tangent sphere-in-bore Booleans remain valid under adaptive tessellation.
- Breaking: blend APIs now return `BlendConstructionFailed` at `validate_geometry` if generated or modified boundaries do not lie on their surfaces, instead of accepting inconsistent geometry. Newly supported planar concave shell offsets can return `OutsideNeighbour` at `validate_offset_boundaries` when walls collapse, where such input previously returned `ConcaveEdge`. Callers should display the supplied diagnostic; no signature changes are required.

- Share straight-edge construction between fillet and chamfer for planar support faces with curved terminations. Intersect exact blend and end surfaces, trim or extend adjacent straight rulings, and extend cylindrical parameter domains without changing their geometry. Regressions cover blind circular channels, unequal distances, rigid transforms, extruded/revolved cylinders, immutable inputs, analytic section-integral volumes, closed tessellation and STEP round trips.
- Breaking: `try_chamfer_solid_edge` and `try_fillet_solid_edges` can return `OutsideNeighbour` for excessive sizes at newly supported curved terminations, instead of `BlendConstructionFailed` or an unsupported-geometry diagnostic. Callers should display the supplied diagnostic; no signature changes are required.

- Extend `try_fillet_solid_edges` / `fillet_solid_edges` to local orthogonal planar neighborhoods on nonconvex bodies, preserving unrelated curved faces, exact boundary curves and holes. Join two convex rounds and one concave round with an exact toroidal patch tangent to all four neighboring faces. Support isolated edges, same-sense two-edge miters, spherical corners and both ends of a selected edge. Share boundary validation with chamfer; keep the existing generic convex construction. Tests cover the captured app model, analytic volumes, tangency, closed tessellation, STEP round trips, selection order, transforms, cavities, collisions and immutable inputs.
- Breaking: oversized selections in the newly supported nonconvex fillet neighborhoods return `OutsideNeighbour` from `try_fillet_solid_edges`, where the previous implementation rejected the body with `UnsupportedGeometry` or `NonPlanarFace`. Callers should display the supplied diagnostic; the app already does so.

- Mixed three-edge chamfer corners with one concave edge and matching setbacks on the common face now include a planar transition above the inside bevel, instead of a V-shaped triple-plane junction. The construction uses local support planes, handles different face depths and both ends of an edge, and preserves selection-order invariance. Unequal common-face setbacks retain the existing three-plane miter. Regressions check transition vertices, analytical volume, closed tessellation, rigid transforms and STEP round trips; the application corpus retains the reported 3 mm model with its circular cut.

- Breaking: planar coplanar seams now return `BlendConstructionFailed` from `try_chamfer_solid_edges` and `try_chamfer_solid_edge` instead of the former `ConcaveEdge` rejection. Sharp concave selections are supported; the application continues to display the returned diagnostic.

- Enable concave straight-edge chamfers in the shared planar construction, including separate distances, oblique support faces and cavity corners. Reject coplanar seams before the generic single-edge fallback can construct a zero-angle strip. Replace the concave-edge rejection regression with exact-volume, topology and STEP checks; retain size/collision guards and fillet restrictions. A downstream Boolean failure on the unchanged rotated bracket is recorded by a paired control instead of attributed to chamfer construction.

- Construct chamfer junctions from local supporting planes on nonconvex and mixed-surface bodies, preserving unrelated curves and holes. Support isolated inset edges, two-/three-edge corners and separate face distances through `try_chamfer_solid_edges_with_distances`. The single-edge API uses the same planar construction. Reject collapsed boundaries and contacts crossing or touching holes. Regressions cover distances, order, transforms, exact volumes, face history, downstream booleans and STEP round trips.
- Breaking: `try_chamfer_solid_edge` now preserves `OutsideNeighbour` from planar construction instead of falling through to a generic `BlendConstructionFailed`. Callers matching failed planar chamfers should handle `OutsideNeighbour`; the app displays the supplied diagnostic. The sequential-corner regression retains failure and immutability checks with the more specific code.

- STEP preparation retains exact circular intersections of perpendicular planes and cylinders instead of fitting splines. This removes OpenCASCADE curve-on-surface errors on the application's counterbored part while preserving its vertices, topology and analytic surfaces.

- Preserve smooth seam intersections in Boolean cuts, reusing their exact boundary edges. Corner-centered holes now divide the adjoining box walls. Project exact curve points onto coincident surfaces before Newton refinement, allowing a revolved sphere to cut an equal-radius blind bore. Both application-derived regressions pass topology, closed-mesh and analytic-volume checks.

- Add `sew_shell` for tolerance-bounded vertex and opposite boundary-edge welding. It preserves surface geometry and open boundaries; closed analytic shells can use `local::replace_surfaces` to reconstruct consistent intersections after sewing. Tests cover a near-coincident seam, immutable inputs, tolerance limits, and an unrepaired missing face.
- Escape apostrophes and reverse solidi in STEP assembly product and occurrence labels. Protect escaped string characters during ruststep 0.4 parsing and restore their original labels after parsing. Quoted labels, backslashes, comments and shared assembly geometry round-trip.


- `builder::sweep_along_wire_fixed` translates planar profiles along exact NURBS paths without rotating their normals, with closed-solid and section/volume regressions. Paths must advance along the profile normal; sampled folds are rejected.
- Breaking: `truck_modeling::errors::Error` adds `InvalidSweepTolerance` and `FixedSweepFold`. Exhaustive Rust matches must handle these variants; the CAD adapter uses the error display and requires no migration.

- Circle subdivision accepts positive local chord tolerances below 1e-6, fixing STEP import of scaled analytic circles; stable half-angle evaluation preserves the requested chord error.

- Breaking: `local::parameter_domain` now derives plane domains from trimmed face loops instead of the plane basis unit square. Its signature is unchanged; callers depending on the old unit-square values should supply explicit domains. This fixes drafting larger parts whose new unit-length plane bases placed intersections outside the artificial search range. The app has no direct calls; its Draft adapter uses the corrected reconstruction.

- Add `local::try_shell_outward` for convex plane/cylinder/cone solids. The original body becomes the cavity, kept surfaces grow outward, and opening planes stay fixed. Positive thickness and the existing inward API keep their contracts. Box and cylinder regressions check closed topology, tessellation, analytic volume and unchanged input.

- Accelerate revolution searches using the meridian solver for similarity transforms and an exact inverse for axial-line cylinders. Keep the grid fallback for other transforms and range hints. Reuse triangle inverses and projected bounds during hidden-line classification, and avoid Rayon scheduling when tessellating with one worker. Add a reproducible FreeCAD feature comparison and STEP file benchmarks. See `truck-benchmarks/results/2026-09-12/REPORT.md` for measured medians, correctness checks and limitations. No public API or production tolerance changes.

- Speed up boolean face division without changing results. Division divided every new cut edge's exact intersection curve again, although those samples only decide which loops bound which face; it now follows the intersection points the edge's leading polyline already holds, between the edge's vertices. `Shell::extract_boundaries` no longer rescans the shell's vertices for every boundary wire, and grouping undecided faces looks edges up in sets. Medians of two alternating runs with one Rayon worker: batch subtraction 41–43% faster (100 holes: 111.6 to 65.4 ms), sequential subtraction of 10, 30 and 100 holes 22%, 11% and 5% (1.252 to 1.188 s), NURBS cylinder intersection 20–55%, cylinder union and intersection 8–40%; the R7 NURBS cut drops from 29.2 to 18.4 ms. In the stage diagnostic at 100 holes, division drops from 49.3 to 4.4 ms in one boolean and from 107.7 to 38.4 ms cut one by one. The 18-case boolean dump is bytewise identical to the parent.

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
