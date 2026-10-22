# Open chamfer contacts and cylindrical terminals

Open cylinder/plane rim contacts now continue beyond their original edge endpoints
to reach curved terminal boundaries. Internal and closed-wire seam stations stay
shared. Changed cylindrical faces whose displaced boundaries leave their finite
parameter domain reuse the existing analytic-domain extension helper. Already
consistent faces are returned unchanged. The analytic cylinders and all existing
validity guards remain intact.

Both new regressions fail on parent 7d271ce02cd44d3ac79f79da9dbb0ab522872f06 with identical test
sources and fixtures. Two oversized rejection controls pass on parent and candidate.
Clean package builds avoid shared-worktree Cargo cache reuse.
The 24 procedural lens-prism operations cover three setback pairs, both caps,
both wire directions and rigid/uniform-scale placements. Tests verify exact
support contacts, generated faces, unchanged input, consistent closed solids,
independent material and STEP. The exact intersection area of two disks is
integrated through the chamfer depth; 1024/4096 intervals agree within 1e-10.

Native C3 sequence capture 002 passes equal-distance, two-distance and reference-angle
modes. Six direct-kernel selections test both wire directions, material and STEP;
replay checks the captured physical selection and exported material. Targeted native
comparisons are valid single solids; maximum sampled distance 6.2206274e-05.
Native-reader STEP edge/vertex tolerances stay below 0.001. These sampled checks do
not establish a global error bound or full parity.

Validation: 461 release shapeops tests (one existing ignore), 119 STEP tests,
71 replay tests, 13 capture tests, all-target Clippy and single-thread actions
smoke pass. Native reference 8, volume 27 and distance 24 checks pass. An existing
redundant-borrow warning in curved_terminal_junctions.rs is unchanged. Geometry,
modeling and meshalgo evidence is reused only with identical source hashes; the
existing pcurve_derivation precision failure remains explicit. Replay is built last.
No fitting threshold, comparison threshold or existing assertion was weakened.

Proof archive: 39 members; SHA256 574baddb49fdfb04abe66262260f006901694c2fec120f828433b80fddf0c9d9.
Implementation is committed before a fresh full 648-operation capture.
No consumer/pin changes, push, PR or performance claim.
