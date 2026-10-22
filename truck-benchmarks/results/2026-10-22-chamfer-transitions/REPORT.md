# Chamfer transition boundary orientation

Two unequal convex bevels on a nonconvex solid can leave a transition triangle
whose normal-sum orientation agrees with its adjacent face boundary. Orient the
triangle opposite the longer surviving original boundary on the unselected edge.
The construction, setbacks and existing validity guards are otherwise unchanged.

Both new regressions fail on parent 14ef4d5758173e0f4463a03e99f276ee06eca9c7 with identical test
sources and fixtures. Clean package builds avoid shared-worktree Cargo cache reuse.
The procedural bracket exercises 64 operations: two caps, two sides, four setback
pairs, both selection orders and rigid/uniform-scale placements. It checks exact
contacts, three generated faces, unchanged input, consistent closed solids,
independent material and STEP. Existing mixed chamfer controls pass unchanged.
For perpendicular setbacks a,b, split cut prisms by z=b*x/a. Removed material is
H*a*a/2 + L*b*b/2 - a*b*(a+b)/6. This independently fixes the orientation contract.

Native equal_dist_complex/A1 now constructs all 17 selected edges, in both orders,
with closed consistent material and STEP. It remains a geometry discrepancy:
Truck STEP volume 1.0931899122619453, native 1.0929679122618834; sampled distances
0.021213203435596465 and 0.028867513459481395.
The native replay material check uses the existing mesh budget; it does not prove
identical geometry. The procedural oracle supplies independent material proof.
No fitting threshold, comparison threshold or existing assertion was weakened.

Validation: 459 release shapeops tests (one existing ignore), 119 STEP tests,
70 replay tests, 13 capture tests, all-target Clippy and single-thread actions
smoke pass. Native reference 8, volume 27 and distance 24 checks pass. An existing
redundant-borrow warning in curved_terminal_junctions.rs is unchanged. Geometry,
modeling and meshalgo evidence is reused only with identical source hashes; the
existing pcurve_derivation precision failure remains explicit. Replay is built last.

Proof archive: 16 members; SHA256 c06eb77939f0b275434390ab9f4928012f838dafc7e6511844a34c7d3460b5b0.
Implementation is committed before a fresh full 648-operation capture.
No consumer/pin changes, push, PR or performance claim.
