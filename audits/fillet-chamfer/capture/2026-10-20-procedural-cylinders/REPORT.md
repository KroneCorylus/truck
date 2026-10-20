# Procedural cylinder recognition: clean full capture

Runtime 51e609ebdb43de00081d04b93c984dc16af6d70f. All 396 native entries pass; 648 operations retain requests, scopes and native properties. All 443 previous constructions retain checks; 441 complete result/comparison records are exactly unchanged. Numerical changes: 2. Unchanged-status failure diagnostic changes: 0.

443 constructions (+0); 413 limited sampled passes (+0) at 0.001. 30 constructed discrepancies remain.
Statuses: {"input-import-failed": 96, "operation-succeeded": 443, "operation-failed": 94, "unhandled-intent": 6, "input-inconsistent": 9}.
- No status changes.

Procedural fillets supported by two planes can now be recognized as cylinders when their stored contact polygons and section coefficients prove a translated circular section. The original R10 fixture and six newly generated continuations now use the existing collapsed-contact construction. Original support data is preserved; no fitted-sample classifier or new corner solver is introduced.

Three success contracts fail on the parent; seven parent controls pass. Twelve recognition variants cover scale, orientation and shifted knots, with explicit malformed/noncircular/anisotropic rejection controls. The original R10 rejection was deliberately rewritten first with stronger history, material, STEP, sphere and input-preservation checks. Four fresh native controls verify independent material, 972 new surface samples and 324 retained-support samples. Maximum final material error is 1.779e-7 and maximum sampled distance is 2.936e-14.

C7 and H4 retain byte-identical Truck result records and mesh volumes. Existing STEP circular_intersection recognition replaces one fitted B-spline boundary with an exact circle in each case. C7 STEP volume changes by +0.00006263046 and native-to-Truck sampled distance changes from 1.70814e-6 to 5.44646e-7; reverse distance remains 2.75159e-6. H4 STEP volume changes by -0.000173859; its sampled distances remain 0.00218067 and 4.14211, so its known discrepancy remains. No pass is gained or lost.

Release tests: 289 geometry, 91 modeling, 102 meshalgo, 115 stepio and 435 shapeops pass; five existing ignores remain. Geometry also retains the independently reproduced parent pcurve_derivation failure: -2829663063.761872 versus -2829663063.7618704 under its unchanged absolute 1e-6 assertion. No test was skipped or weakened. All 59 replay and 13 capture tests, all-target release Clippy and single-thread actions smoke pass. Replay built last.

Proof: truck-benchmarks/results/2026-10-20-procedural-cylinders/REPORT.md.
Archive: 8380 byte-verified members; SHA256 b1170718bdf6b1811b9bf2470b043c712304952357f98c26eac34c9bf34d01d2.
Finite samples do not establish a global bound or full parity. No consumer/dependency/tolerance change, push, PR or performance claim.
