# Cylinder rim fillet corners

Three selected edges meeting between an outer cylinder, a perpendicular cap and
a radial plane now form an exact rolling-sphere corner. Offset cylinder/plane
intersections determine shared contacts. Two cylindrical rounds and a toroidal
rim share circular edges with the spherical corner. The existing neighborhood,
face assembly and validity guards are reused; this is a constant-radius local
construction, not arbitrary curved-support corner filling.

Circular contact arcs use the sweep between their trimmed endpoints. Reusing
the original rim midpoint can select the major arc after a large trim crosses
that midpoint. A radius-0.45 control on a radius-1 sector checks this case.
The radius must remain below half the outer cylinder radius and fit remote
terminations; oversized controls retain their failure and input immutability.

Two regressions fail on parent 3627c53f8ff108748f1b10354ad2f586ec2390db with identical test sources
and fixtures: procedural cylinder corners and native B3/B7 replay. Four oversized
controls pass on parent and candidate. Clean package builds avoid shared-worktree
Cargo cache reuse.

The 24 procedural operations cover 90- and 270-degree sectors, three radii,
rigid/uniform-scale placements and reversed selection order. They verify exact
surface radii, 3672 tangent-normal pairs, unchanged input, consistent closed
solids, independent material and STEP. Four native selections check both orders,
ordinary-budget robust STEP and additional strict closed tessellation at the
tighter export budget. No production fitting threshold changed.

Independent horizontal material integrals converge at 1024/4096/16384 intervals
and agree with native volumes within 1.2e-9 absolute volume. Targeted B3/B7
comparisons are valid single solids with maximum sampled distance 1.8012518e-13.
Sampled checks are not a global-error or parity proof.

Validation: 458 release shapeops tests (one existing ignore), 119 STEP tests,
69 replay tests, 13 capture tests, all-target Clippy and single-thread actions
smoke pass. Native reference 8, volume 27 and distance 24 checks pass. An existing
redundant-borrow warning in curved_terminal_junctions.rs is unchanged. Geometry,
modeling and meshalgo evidence is reused only with identical source hashes; the
existing pcurve_derivation precision failure remains explicit. Replay is built last.

Proof archive: 33 members; SHA256 99e21f490c70dd9e94ee5d11a795c37c0046a7f322e91afff7b4c7cb6f80a271.
Implementation is committed before a fresh full 648-operation capture.
No consumer/pin changes, push, PR or performance claim.
