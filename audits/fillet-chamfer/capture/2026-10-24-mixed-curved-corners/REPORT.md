# Mixed-corner full corpus capture

Complete serial capture of committed mixed-corner implementation 9e3ce08acbf85cbff64baae84d517731ac27b382: 396 native cases, 648 operations, 482 constructions and 438 limited comparison passes. 121 of 559 positive-solid operations remain unresolved.

All 480 prior successful Truck result/comparison records are exactly unchanged. Every raw input task, summarized task, native result, physical selection and operation key was reconciled. Native BREP bytes and native STEP bytes excluding only the generated FILE_NAME capture timestamp are unchanged. Source, replay, plugin, runtime library, harness and external-data hashes were verified. One worker and a 120-second timeout were retained.

F1 is a newly confirmed limited-comparison pass. H7 constructs but remains a comparison discrepancy against the unchanged 0.001 sampled-distance threshold. Construction and sampled comparison are separate outcomes; neither establishes universal parity or a global geometric-error bound.

Statuses: {"input-import-failed": 95, "input-inconsistent": 10, "operation-failed": 61, "operation-succeeded": 482}. There are 44 constructed comparison discrepancies. The 88 no-solid inputs already contain native shells and faces before Truck import; their shell boundary contracts remain unverified. One rejection matches native.

All 13 capture-harness tests pass. This capture reuses the exact source and replay from the mixed implementation validation: 471 shapeops tests (one existing ignore), 119 STEP tests, 76 replay tests, all-target Clippy, benchmark smoke and native checks. Those Rust suites were not rerun for this evidence-only checkpoint. The known pcurve_derivation precision failure and two existing test warnings remain explicit.

Independent review classified the 43 preceding constructed discrepancies conservatively: six have demonstrated native-reference contributors, three have unresolved physical-convention questions, and 34 remain unresolved. H7 adds a reference witness, not an accepted exception or a pass. The canonical failed comparisons remain intact.

The actual sibling consumer is PiezaCAD, with local ../truck dependencies. Source review identifies mixed-selection dispatch restrictions and missing general-curve sketch fillets/sketch chamfers. No application files or dependency declarations were changed; product behavior was not executed in this review.

F5 tessellation and clustered-knot defects have scratch investigations and candidate fixes, but this checkpoint contains no new kernel implementation. Their validation and integration are separate work. The stopped E4 stress experiment was not restarted.

Raw corpus archive SHA256: 5f5b0bf951da9e2e3809c52b17aa5b72e7b1c177f2ddc3086002a8828052c6be.
