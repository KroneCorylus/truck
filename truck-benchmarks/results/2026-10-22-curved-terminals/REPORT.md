# Planar fillet corners with curved terminal faces

Multi-edge planar fillet junctions now accept curved terminal faces where only
one selected edge meets the end. Existing exact cylindrical blends and sphere
or torus corners are retained. Terminal boundaries are intersections of each
blend cylinder and its neighboring curved support; existing conic edges are
trimmed in their own geometry and shared vertices preserve topology.

A conical wall may place a valid contact beyond the old selected edge endpoint.
The old endpoint is not a geometric clipping plane. The neighboring planar
face boundary checks, noncollapsed line checks and final solid consistency
remain in force. Generated cylinder domains include terminal and original
endpoints to contain nonplanar intersection arcs. Single-edge dispatch and
chamfer neighborhood restrictions remain unchanged.

Two regressions fail on parent 3d8fc852acc8640c7bc63d9bd1d21f9db72bc28c with byte-identical
sources and fixtures: procedural cylinder/cone corner construction and native
contour replay. Eight oversized controls reject without mutation on both parent
and candidate. Parent and root clean shapeops before their builds to avoid
shared-worktree cache reuse.

Controls cover32 successful operations: convex/concave cylinder/cone sectors,
two radii, affine placement with uniform scale and reversed selections. They
check exact cylinder/sphere/torus radii,5168 tangent-normal pairs, closed solids,
independent material integrals and STEP. Eight captured selections (four cases,
two orientations) preserve full native contour scope and unchanged inputs.
They check exact solids, ordinary-budget robust STEP round trips and additional
strict closed-mesh checks at tighter STEP fitting tolerance. The new test was
specified this way because normal-budget STEP fitting permits an error above
the strict mesh path's point tolerance; no production fitting budget changed.

Independent section integration agrees at1024/4096/16384 Simpson intervals.
Expected material agrees with native B1/B5/B9/C4 within3.0e-7 absolute volume.
All four targeted comparisons pass existing checks; maximum sampled distance
is below2.8e-6. These are sampled checks, not a global error or parity proof.

Validation:454 release shapeops tests (one existing ignore),119 stepio,
67 replay,13 capture, all-target Clippy and single-thread actions smoke pass.
Native reference8, volume27 and distance24 checks pass. Geometry, modeling and
meshalgo evidence is reused only with identical source hashes. The pre-existing
pcurve_derivation precision failure remains explicit. Replay is built last.

Proof archive: 63 members; SHA256 8a64750c549c5009b79bd8181d54eaaa8ebbce4110f00fe2970326cfd98d4fc9.
Implementation is committed before a fresh full648-operation capture. No
consumer/pin changes, push, PR or performance claim.
