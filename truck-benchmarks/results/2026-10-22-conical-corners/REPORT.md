# Conical support corners with planar terminations

Three selected edges on an outward cone and two planes now join an exact rolling
sphere, toroidal cap rounds and fitted cone/radial-plane blends. Offset-cone
intersections locate the sphere. Hyperbolic rolling centers use a regular lateral
parameter at axial terminals. Fitted side contacts intersect remote caps before
shared trim edges are assembled. Existing validity and trim guards remain.

Both new regressions fail on parent 86864851f3cec4b934a59d62886d9c468e5ed56c with identical sources
and fixtures. Clean package builds avoid shared-worktree Cargo cache reuse.
The 48 procedural operations cover truncated 90/270-degree cones, apex sectors
and meridian terminations, three radii, two placements and both selection orders.
Exact analytic radii/center, fitted radius residual below 1e-5, nine tangent joins
with normal dot greater than 1-1e-7, closed material, STEP and input immutability
are checked. Eight oversized requests reject. The initial prototype fit failed
the tangency control; the new path uses tol/100, without weakening the assertion.

Material is independently integrated by projecting onto the inward cone
intersected with two selected offset half-spaces and integrating its radius-r
dilation inside the unselected boundaries. Active-constraint projection is
independent of the builder and its surface fitter. Domain-aware 128/256-node
section quadrature differs by at most 1.1077845e-08; the 270-degree case adds a
separate analytic half-cone/cap-round integral. Eight normalized native selections
and four original-scale production replays preserve all three selected edges.

All four native comparisons remain discrepancies. C2 and C8 native corner samples
lie respectively about 0.1655 and 4.8284 away from their exact radius-10 sphere;
corresponding Truck samples lie on the sphere to numerical precision. Evidence
is retained without changing comparator statuses or claiming all discrepancies
are explained. Independent normalized volumes are 0.5102133441641101 (C2),
1.5599191504183765 (C6), 0.06486303410642758 (C8), 0.06687424247638343 (D1).
Native-reader STEP tolerances remain below 0.001. Sampled checks establish neither
a global error bound nor full native parity.

Validation: 467 release shapeops tests (one existing ignore), 119 STEP tests,
74 replay tests, 13 capture tests, all-target Clippy and single-thread actions
smoke pass. Native reference 8, volume 27 and distance 24 checks pass. Existing
redundant-borrow and sphere-literal-precision warnings are unchanged. Geometry,
modeling and meshalgo evidence is reused only with identical source hashes; the
known pcurve_derivation precision failure remains explicit. Replay is built last.
No existing assertions, thresholds or replay fitting budgets were weakened.

Proof archive: 73 members; SHA256 4c27b36a3ddc43ceb90eeecdb6d714030c88ec4e7488d5c6e32063f0621dd360.
Implementation is committed before a fresh full 648-operation capture.
No consumer/pin changes, push, PR or performance claim.
