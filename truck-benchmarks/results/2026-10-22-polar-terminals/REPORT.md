# Sphere-pole terminal contacts

Line/sphere contacts now use an analytic intersection when the existing numerical
solver fails. At a sphere parameter pole the parameter Jacobian is singular,
even when the spatial line/sphere intersection is well-defined. The analytic
fallback projects the sphere center onto the line and selects the chord endpoint
nearest the requested contact. Successful numerical intersections and other
surface types keep their existing behavior.

Two regressions fail on parent 3b401517f65ab394c58a436c59cdcdae5c4f6f04 with identical test sources
and fixtures: procedural polar terminal construction and native E2 replay. Two
oversized controls reject without mutation on both parent and candidate. Clean
package builds prevent shared-worktree Cargo cache reuse.

Eight procedural operations cover two radii, two rigid/uniform-scale placements
and reversed selections. They check immutable inputs, exact blend radii, 264
terminal curve samples on both supports, independent material, closed meshes and
STEP. Two native selections check both orders, the ordinary robust STEP contract
and strict closed tessellation at the additional tighter fitting budget.

The independent section integral converges at 1024/4096/16384 intervals and
agrees with native volume within 1.76e-6 absolute volume. The targeted E2
comparison is valid with one solid on each side and sampled distances below
8.72e-6. These are sampled checks, not a global error or parity proof.

Validation: 456 release shapeops tests (one existing ignore), 119 stepio,
68 replay, 13 capture, all-target Clippy and single-thread actions smoke pass.
Native reference 8, volume 27 and distance 24 checks pass. Geometry, modeling
and meshalgo evidence is reused only with identical source hashes. The existing
pcurve_derivation precision failure remains explicit. Replay is built last.

Proof archive: 20 members; SHA256 ad112dbfda4f3c462d32b5e7dcd3a868d1582b1619fb66d0f0887bc5ae64d33b.
Implementation is committed before a fresh full 648-operation capture.
No consumer/pin changes, push, PR or performance claim.
