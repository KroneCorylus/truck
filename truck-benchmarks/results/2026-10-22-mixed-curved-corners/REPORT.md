# Mixed curved corners with a shared planar wall

A trivalent selection with one concave rim and two convex edges sharing a planar
wall rounds the rim first, then the resulting three-edge tangent chain. If the
original open-chain trim fails, internal contacts continue across the common
support. Remote terminals and final geometry/trim guards still bound the result.
Final face correspondence is computed against the original solid.

The new procedural and native regressions fail on parent 9f97532ab746207f3d716696dc6b4640a026662e
with identical sources and fixtures. The previous F1 unsupported-case assertion
was intentionally rewritten before implementation to check four generated faces,
closed independent material, STEP and unchanged input. All six oversized torus
rejections remain unchanged, and four mixed oversized rejections are added.
The revised contract also fails on the parent. Clean package builds prevent
shared-worktree Cargo cache reuse.

Twelve procedural F1 operations and24 imported F1/H7 operations cover three
radii, two placements and both orders. Four fitted faces, constant radius residual
below1e-5, ten tangent joins with normal dot greater than1-1e-7, independent volume,
closed tessellation, STEP and input immutability are checked. H7 initially failed
the strict tangency check; tightening only the two composed fits to tol/100 fixes
it without weakening the assertion. Failed and passing diagnostic logs are retained. An isolated combined native
run reached all24 radius/G1 checks but hit its300-second runner limit during
final material/STEP work; that incomplete attempt is retained separately from
the required completed full replay suite.

Independent boundary-flux quadrature computes F1 torus and H7 cylinder/cap/wall
material. It uses no CAD builder, fitted surface, native library or mesh. Sources,
derivations and128/256-node convergence at all three radii are archived.

Two original-scale production replays construct valid closed STEP with unchanged
physical selections. F1 passes the limited native comparison. H7 retains its
.001525464/.001514833 discrepancy against the .001 threshold. At the two worst
sample witnesses, analytic radius residuals are .001525552 for the native point
and approximately7.72e-7 for the Truck point. The diagnostic comparison exactly
matches the canonical comparison; these witnesses do not establish a global bound.

Validation:471 release shapeops tests (one existing ignore),119 STEP tests,
76 replay tests,13 capture tests, all-target Clippy and single-thread actions
smoke pass. Native reference8,volume27,distance24 checks pass. Existing warnings
and the known geometry precision failure remain explicit; reused dependency
validation is allowed only with identical source hashes. Replay is built last.

Proof archive:76 members; SHA256 ffd52c1c367d115c1c55e8002f940e2d7926fcc2fd895addf29dc8c50ee93d8e.
Implementation is committed before a fresh full648-operation capture.
No canonical comparison thresholds, replay STEP budgets, consumer pins, push,
PR, parity guarantee or performance claim.
