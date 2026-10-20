# Chamfer terminal growth at concave radial steps

Open tangent-wire chamfers can now grow straight terminal edges toward their contact curves. A private trim callback reuses the existing exact-line extension mechanism at open wire endpoints. Closed and interior edges retain bounded trimming, and curved terminal edges cannot enter the line-extension branch. The generic public wire API retains its existing bounds and behavior; modeling solid chamfers opt into guarded extension and retain final topology, geometry and boundary validation. No tolerance is relaxed.

Three regressions fail on parent d515003dd7735ad94aa94e9753f7fc0db0618232; convex and oversized-cut guards pass there. All five pass after the fix. Twelve positive variants cover stepped and convex circular sectors, equal/unequal/angle-derived setbacks, transforms, reversed faces and reversed wires. Eight oversized variants reject cuts without mutation. Shared solid/STEP checks verify independent material, exact contact vertices, consistent geometry and complete original face correspondence. Assertions are unchanged from the parent proof.

Fresh native controls use R3, outer radius4.5, H10 and sector angle105degrees. The convex control has initial volume theta/2*R²*H. The stepped control removes the annular sector from a full outer cylinder, giving pi*outer²*H-theta/2*(outer²-R²)*H. A chamfer removes theta/2*axial*(R*radial-radial²/3). Setbacks(radial,axial) are(1,1),(1,.5),(1,1/sqrt3).

All six native controls are valid and match independent material within7.4e-11. All three stepped native controls mutate input vertex tolerances from approximately1.5e-7 to1e-4; before/after BREP and properties remain archived. Convex native inputs are unchanged. All six Truck controls preserve input and produce closed, valid single-solid STEP results. Maximum STEP material error 1.2005216376564931e-08; maximum sampled comparison distance 4.8402990585599727e-09. Both contact circles and sampled conical geometry are independently checked; maximum cone residual 1.7763568394002505e-15. Finite samples are not a global error bound.

Validation:413 shapeops tests(one existing ignore),59 replay tests,13 capture tests,all-target release Clippy and single-thread actions smoke. Replay built last. Unchanged mesher102-test evidence is verified by source hashes. Implementation and proof precede clean full capture. No consumer/dependency change,push,PR or performance/full-parity claim.

Archive:151 byte-verified members;SHA256 9eee613e1a6c2aeae954270c8fff66736d7c69a682f20ea46c892d681f1dc0de.
