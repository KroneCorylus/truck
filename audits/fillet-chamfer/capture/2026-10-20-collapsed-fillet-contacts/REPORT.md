# Collapsed analytic fillet contacts: clean full capture

Runtime 0258681b94d8a6a7fb5c14ad44c0ec878c173339. All 396 native entries pass, yielding 648 operations with unchanged requests, scopes and native result properties. All 442 prior constructions retain checks; 442 complete result/comparison records are exactly unchanged. Numerical changes: 0. Unchanged-status failure diagnostic changes: 0.

443 constructions (+1); 413 limited sampled passes (+1) at 0.001. 30 constructed discrepancies remain. Every construction preserves input, consistent geometry, closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.
Statuses: {"input-import-failed": 96, "operation-succeeded": 443, "operation-failed": 94, "unhandled-intent": 6, "input-inconsistent": 9}.
Status changes:
- blend/simple/Q7/002: operation-failed → operation-succeeded

Equal-radius contacts on analytic cylindrical corners can now collapse to one shared cap vertex. Straight rim sections become cylindrical strips, and collapsed corners become spherical patches. The fallback preserves original support surfaces and uses the existing final geometry, planar-trim, closed-topology and face-history validation.

Three positive contracts fail on the parent; four controls pass. Seven focused tests cover 16 equal-radius primitive variants, four transformed variants, 16 closed-rim start/direction variants and four full-cap/short-wall rejection variants. The generated equal-radius matrix was deliberately rewritten first for stronger success, material, STEP, radius, history and input-preservation checks. The original procedural-fixture rejection and larger-radius fold rejection remain. Four fresh native controls verify independent material, exact contact vertices and 1,377 surface samples.

434 shapeops tests (one existing ignore), 59 replay tests, 13 capture tests, all-target release Clippy and single-thread actions smoke pass. Replay built last. Unchanged mesher 102-test evidence is verified by source hashes.

Q7 operation2 applies a second radius-2 round to a 5×5×5 cube. Independent material is 113.46312408709453; native and Truck STEP volumes agree within1e-7, with maximum sampled distance 7.5495165674510645e-15.

Proof: truck-benchmarks/results/2026-10-20-collapsed-fillet-contacts/REPORT.md.
Archive: 8380 byte-verified members; SHA256 7550736400fdd8bddf873b62c01de62e93dc970c338e9d85ae8668986ff197ee.
Finite samples do not establish a global error bound or full parity. No consumer, dependency or tolerance change, push, PR or performance claim.
