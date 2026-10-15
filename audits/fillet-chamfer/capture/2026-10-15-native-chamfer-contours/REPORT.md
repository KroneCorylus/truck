# Complete native chamfer contour capture

Clean revision 099ccb7ea7a6f07aaf8e352238c413197fb99edb, following contour implementation 2346b975 and its fillet
export-budget correction. All 396 native entries pass, yielding 648 captures:
487 complete native chamfer scope records and
161 seed-only fillet records.

58 requests have corrected propagated/ignored assignment scope;
590 retain the same intent. Native reference metadata
and raw seed selections are exactly unchanged. No previous successful check
regresses for unchanged intent. 2 formerly successful seed-only records
lose checks under corrected complete scope; those are recorded separately and
must not be presented as equivalent successful native operations.

Statuses: {"input-import-failed": 96, "input-inconsistent": 9, "operation-failed": 182, "operation-succeeded": 221, "selection-unmatched": 8, "unhandled-intent": 132}.
221 constructions pass immutable-input, consistent-geometry,
closed-mesh and native-valid single-solid STEP checks. 210 pass combined
limited comparisons at 0.001; 11 retain sampled differences.
165 prior successful result/comparison records are exactly unchanged after
excluding added scope/tolerance metadata; 55 have numerical/selection
changes. Read comparison-summary.json for every transition and measured value.

Automatic A4 now captures and applies its complete three-edge tangent contour.
Its native-valid STEP has sampled maximum boundary difference 2.229100205894138e-8.
Eight native controls verify all chamfer modes, native first-assignment and
parameter gates, including an ignored different seed in the same contour.

Replay fits chamfer STEP at 1e-7 and fillet STEP at the established 5e-5. An initial
full run with globally tightened fitting regressed 13 fillet exports. That run
is archived explicitly in truck-benchmarks/results/2026-10-15-fillet-replay-export;
a parent-failing D4 test and native proof restore its exact previous comparison.
The present clean run is the stable corrected capture. No assertion was loosened.
Final validation: 25 Rust replay tests; 22 audit and 10 capture tests; all-target
release Clippy and action smoke. Kernel sources retain their preceding 504-pass
validation (0 failed, 1 existing ignored), rather than claiming a repeated suite.

Archive: 7714 byte-verified members, SHA256 3dfeebfe1bd343a1ff1ad0af724e222b063619431ca6d1727705d02972c24fb1.
Raw target/occt-capture-native-chamfer-contours-stable. Sequences use independent
native inputs, not prior Truck outputs. No full-parity or performance claim.

Remaining: fillet tangent propagation and effective radius laws, curved reference
faces, complete-contour selection mismatches, unsupported operations, C1 input
fidelity, shell inputs without solids, four missing fixtures, sampled differences,
editing/continuity evidence and the prior PCurve precision test.
