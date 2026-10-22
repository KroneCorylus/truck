# Toroidal support corners with planar terminations

Three convex selected edges on an outward torus and two planes now join an exact
rolling sphere, toroidal cap rounds and fitted torus/radial-plane blends. Offset
torus intersections locate the sphere. Minor-angle rolling centers preserve the
selected meridian branch. Fitted contacts intersect remote caps before shared
trim edges are assembled. Existing validity and trim guards remain.

Both new regressions fail on parent 531505c8bc06df772204c9c7075d9cfa47e8b735 with identical sources
and fixtures. Clean package builds avoid shared-worktree Cargo cache reuse.
The 36 procedural operations cover 45/160-degree profiles, three radii, two
placements and both selection orders. Exact radii and center, fitted radius
residual below 1e-5, nine tangent joins with normal dot greater than 1-1e-7,
closed material, STEP and input immutability are checked. Six oversized requests
and one mixed convex/concave selection reject. F1 remains unsupported.

Material is independently integrated by projecting onto the inward outer torus
intersected with selected offset half-spaces and integrating its radius-r dilation.
The nonconvex F3 profile subtracts its inner cavity using the exact rolling-sphere
envelope on its rounded wall strip. This calculation is independent of the builder,
surface fitter and mesh. The 128/256-node controls differ by at most 6.1102407e-08;
a separate 512-node radius-0.1 run checks convergence. Six normalized native
selections and three original-scale production replays preserve the three edges.

All three native comparisons remain discrepancies. Their construction checks and
independent material controls pass, but those observations do not establish full
native parity or a global error bound. Native-reader STEP tolerances remain below
0.001. Cone arithmetic is preserved; isolated original-scale cone replays and their
native comparisons match exactly. A fresh full corpus capture remains required.

Validation: 469 release shapeops tests (one existing ignore), 119 STEP tests,
75 replay tests, 13 capture tests, all-target Clippy and single-thread actions
smoke pass. Native reference 8, volume 27 and distance 24 checks pass. Existing
test warnings remain recorded. Geometry, modeling and meshalgo evidence is reused
only with identical source hashes; the known pcurve_derivation precision failure
remains explicit. Replay is built last. No existing assertions, thresholds or
replay fitting budgets were weakened.

Proof archive: 56 members; SHA256 fb76c63395724ae041b9e9a403c36b1ee9e3699f3f6e78a72c84a89ecfe995c5.
Implementation is committed before a fresh full 648-operation capture.
No consumer/pin changes, push, PR or performance claim.
