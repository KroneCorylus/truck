# Spherical support corners with planar terminations

Three selected edges meeting on an outward sphere and two planes now join an exact
rolling sphere, two toroidal blends and a cylindrical round. Inward-offset support
intersections locate the corner; exact contact circles and surface intersections
trim the remote ends. Existing geometry, closure and trim guards remain. Consumed
terminal faces and collapsed inward offsets are rejected without changing input.

Both new regressions fail on parent eaf8b1570a27de64ac581d73e2794d553d9e30e3 with identical test sources
and fixtures. Clean package builds avoid shared-worktree Cargo cache reuse.
The 36 procedural operations cover 90/270-degree truncated sectors, a polar sector,
three radii, two placements and both selection orders. Exact radii and center,
nine tangent boundaries, closed material, STEP and input immutability are checked.
Nine oversized requests reject. Material is independently integrated by projecting
onto an inward ball intersected with the offset planes and integrating its radius-r
dilation within the unselected terminal planes. 512/1024-node quadrature differences
stay below 1.3e-7 at unit scale; an extra half-sector uses a separate section integral.

Six normalized imported selections preserve native D5/D9/E4 intent and verify
material and STEP. Three original-scale production replays preserve all three
edges and pass their existing output checks. D5 passes the unchanged 0.001 sampled
distance threshold. D9 and E4 remain discrepancies; do not count them as resolved.
Native-reader STEP tolerances remain below 0.001. These sampled checks establish
neither a global error bound nor full parity.

E4 diagnostics remain explicit. A native corner point is 0.0887015 away from the
exact radius-10 sphere. Independent material is about 3464310.44; native tessellation
of Truck STEP agrees with Truck raw mesh within the chord budget. OCCT surface-volume
integrators disagree with those meshes, including each other before/after boundary
splitting. The canonical comparator and its statuses are unchanged. A repeated
original-scale STEP stress experiment completed D5/D9 but was stopped during E4
mesh construction after more than 20 minutes and 8 GB; it is incomplete, not passing.
The permanent imported regression uses scaled geometry/radii, alongside full-scale
corpus replay and comparison. Profiling and all numerical evidence are archived.

Validation: 465 release shapeops tests (one existing ignore), 119 STEP tests,
73 replay tests, 13 capture tests, all-target Clippy and single-thread actions
smoke pass. Native reference 8, volume 27 and distance 24 checks pass. An existing
redundant-borrow warning in curved_terminal_junctions.rs is unchanged. Geometry,
modeling and meshalgo evidence is reused only with identical source hashes; the
existing pcurve_derivation precision failure remains explicit. Replay is built last.
No fitting threshold, comparison threshold or existing assertion was weakened.

Proof archive: 71 members; SHA256 9ddbc8aa55ae23f8c3a06612c6d5953b3456f376c9b1f13ff0e024ab2f3492c4.
Implementation is committed before a fresh full 648-operation capture.
No consumer/pin changes, push, PR or performance claim.
