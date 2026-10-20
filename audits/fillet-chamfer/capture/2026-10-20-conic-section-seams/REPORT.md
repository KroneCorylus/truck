# Conic section seams: clean full capture

Runtime 2ea838f2b4f81549e028082275c31dc127e2865f. All 396 native entries pass, yielding 648 operations with unchanged requests, scopes and native result properties. All 438 prior constructions retain checks; 426 complete result/comparison records are exactly unchanged. Numerical changes: 12.

438 constructions (+0); 408 limited sampled passes (+0) at 0.001. 30 constructed discrepancies remain. Every construction preserves input, consistent geometry, closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.
Statuses: {"input-import-failed": 96, "operation-succeeded": 438, "operation-failed": 99, "unhandled-intent": 6, "input-inconsistent": 9}.
Status changes:
None.

Projected conic sections now connect periodic contacts locally across the parameter seam. Explicitly bounded long arcs retain their interval, including reversed sources. Endpoint and bounds checks remain; no tolerance is relaxed.

Two arc regressions and a solid regression fail on the parent; the existing section test and new long-arc guard pass. All four new tests now pass. Three scales and source reversal cover exact arc points and both supports. Two solid variants check independent material, genus 0, closed meshes, STEP, contacts, input preservation and face correspondence. Three fresh native controls preserve input and match independent material and contact geometry.

424 shapeops tests (one existing ignore), 59 replay tests, 13 capture tests, all-target release Clippy and single-thread actions smoke pass. Replay built last. Unchanged mesher 102-test evidence is verified by source hashes.

12 prior mesh-volume values change under the new parameterization; all prior STEP comparison records are exactly unchanged. The largest observed mesh-volume shift is 0.00010509008643566631. These are retained numerical changes, not a global geometry-equivalence claim.

Proof:truck-benchmarks/results/2026-10-20-conic-section-seams/REPORT.md.
Archive: 8365 byte-verified members;SHA256 53cac8b33c002eccee1118dc687ce441669ddf5764d7a76d656cc81798f69997.
Finite samples do not establish a global error bound or full parity. No consumer, dependency or tolerance change, push, PR or performance claim.
