# Separated planar corners and curved fillet rims

A selection containing a planar corner and a separate curved rim now routes each
connected component through its existing supported construction. Tangent components
use the wire API; corner components use the edge API. Components follow input-shell
order and must preserve subsequent selected edges and all earlier generated faces.
The existing degree-two and at-least-two-components guards remain. Recursion uses
strictly smaller connected selections. Failures leave the input unchanged.

Both new regressions fail on parent c649eaa905eac9ce64aec3335e0034724139c3e2 with identical test sources
and fixtures. The oversized-cut rejection passes on parent and candidate. Clean
package builds avoid shared-worktree Cargo cache reuse.
The 24 procedural operations cover two radii, two placements, both face orders and
three selection permutations. Independent quarter-cylinder overlap and toroidal
rim integrals verify volume. Exact rim contacts, consistent closed solids, STEP,
face history and input immutability are checked.

Native Q2 contains three selected edges: two meeting at a planar corner and one
separate curved rim. Two selection orders pass raw material and STEP tests. The
verified production replay retains all three edges. Native comparison is a valid
single solid; maximum sampled distance 5.9345765e-05; stored STEP tolerances below
0.001. These checks do not establish a global error bound or full parity.

Validation: 463 release shapeops tests (one existing ignore), 119 STEP tests,
72 replay tests, 13 capture tests, all-target Clippy and single-thread actions
smoke pass. Native reference 8, volume 27 and distance 24 checks pass. An existing
redundant-borrow warning in curved_terminal_junctions.rs is unchanged. Geometry,
modeling and meshalgo evidence is reused only with identical source hashes; the
existing pcurve_derivation precision failure remains explicit. Replay is built last.
No fitting threshold, comparison threshold or existing assertion was weakened.

Proof archive: 17 members; SHA256 6092ad14ebc168d6f90f97873d6d41d1c5654155b1a6019b2aa3dd0dc6c8d222.
Implementation is committed before a fresh full 648-operation capture.
No consumer/pin changes, push, PR or performance claim.
