# Longitudinal cylinder chord distances: clean full capture

Runtime b5c98d1addc63b76c4c13fa34c09d90fbfad1e80: all 396 native entries pass, 648 operation records.
All requested parameters, complete selection scopes and native result properties
are unchanged. All 312 previous constructions retain their successful checks;
311 complete prior result/comparison records are exactly unchanged and
1 have numerical changes recorded in comparison-summary.json.
No prior successful check regressed.
The existing equal_dist/A2 case now passes the unchanged 0.001 sampled threshold:
maximum distance falls from 0.0013271799562406305 to 0.0006210601672568816.
Twelve unequal requests move from unhandled intent to construction failure.
They now reach the solver with the proper parameters; further kernel work is
required. This checkpoint adds no corpus constructions.

312 constructions (+0); 293 combined limited sampled
passes at 0.001 (+1). All constructed outputs preserve input and pass
consistent geometry, closed mesh, STEP preparation and native-valid single-solid
checks. 19 constructed discrepancies remain under these checks.
Statuses: {"input-import-failed": 96, "operation-succeeded": 312, "operation-failed": 211, "unhandled-intent": 20, "input-inconsistent": 9}.
Status changes:
- chamfer/dist_dist_complex/A6/001: unhandled-intent → operation-failed
- chamfer/dist_dist_complex/A7/001: unhandled-intent → operation-failed
- chamfer/dist_dist_complex/A8/001: unhandled-intent → operation-failed
- chamfer/dist_dist_complex/A9/001: unhandled-intent → operation-failed
- chamfer/dist_dist_sequence/A6/005: unhandled-intent → operation-failed
- chamfer/dist_dist_sequence/A6/006: unhandled-intent → operation-failed
- chamfer/dist_dist_sequence/A7/005: unhandled-intent → operation-failed
- chamfer/dist_dist_sequence/A7/006: unhandled-intent → operation-failed
- chamfer/dist_dist_sequence/A8/005: unhandled-intent → operation-failed
- chamfer/dist_dist_sequence/A8/006: unhandled-intent → operation-failed
- chamfer/dist_dist_sequence/A9/005: unhandled-intent → operation-failed
- chamfer/dist_dist_sequence/A9/006: unhandled-intent → operation-failed

Replay now converts equal and unequal native cylinder chord distances to Truck's
documented tangent-offset parameters on longitudinal plane/cylinder edges.
Three parent-failing tests cover six constructed variants plus nonlocal-contact
guards. Twenty-one native controls cover convex/concave supports, three plane
offsets and both reference sides; all match independent material and valid STEP.
Maximum sampled control boundary difference: 2.007283228522283e-13.
42 replay tests, 13 capture tests, Clippy, actions smoke and final build pass.
Kernel sources/core tests remain identical to the 393-pass checkpoint at 4211cfe0.
Proof: truck-benchmarks/results/2026-10-16-longitudinal-chamfer-distances/REPORT.md.
Archive: 7987 byte-verified members, SHA256 914529d3bc953228065b417b936513493b0c00e09544d55f203720f98ee98cef.
Finite sampling does not establish full parity or a global boundary-error bound.
No performance claim, consumer/pin changes, push or PR.
