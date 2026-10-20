# Mixed chamfer components: clean full capture

Runtime eea16d3689f365fc58128f269f1883fb54b4e368. All 396 native entries pass, yielding 648 operations with unchanged requests, scopes and native result properties. All 438 prior constructions retain checks; 438 complete result/comparison records are exactly unchanged. Numerical changes: 0.

441 constructions (+3); 411 limited sampled passes (+3) at 0.001. 30 constructed discrepancies remain. Every construction preserves input, consistent geometry, closed mesh, STEP preparation and valid single-solid native/Truck comparison shapes.
Statuses: {"input-import-failed": 96, "operation-succeeded": 441, "operation-failed": 96, "unhandled-intent": 6, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_angle/A3/001: operation-failed → operation-succeeded
- chamfer/dist_dist/A3/001: operation-failed → operation-succeeded
- chamfer/equal_dist/A3/001: operation-failed → operation-succeeded

Disjoint chamfer dispatch can now combine a tangent curved rim with an isolated straight edge. One-edge components use the existing single-edge constructor when the wire constructor cannot handle their geometry. Distance-side orientation, selected-edge survival and guards against changing an earlier bevel remain enforced. OutsideNeighbour is not retried.

Three positive tests fail on the parent; the overlap guard passes. All four tests now pass, covering 18 positive and six negative variants across rim subdivisions, setback pairs, transforms and reversed face/selection order. Shared solid and STEP checks verify independent material, genus 0, contact vertices, input preservation and face correspondence. Three native controls preserve input and match analytic material and both bevel surfaces. The conic seam bug exposed during this investigation was fixed separately before this dispatcher change.

428 shapeops tests (one existing ignore), 59 replay tests, 13 capture tests, all-target release Clippy and single-thread actions smoke pass. Replay built last. Unchanged mesher 102-test evidence is verified by source hashes.

Proof:truck-benchmarks/results/2026-10-20-mixed-chamfer-components/REPORT.md.
Archive: 8374 byte-verified members;SHA256 0e74338e75d49b703c1d3018b38ee5d27630817434c013468713cc3257509222.
Finite samples do not establish a global error bound or full parity. No consumer, dependency or tolerance change, push, PR or performance claim.
