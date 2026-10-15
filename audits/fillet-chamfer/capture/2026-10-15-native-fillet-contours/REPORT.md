# Final native fillet contours: clean full capture

Clean runtime e3fab5a68a5d3d3aafdd365336f95af0e6270b94. {'passed': 396} native entries; 648 captures.
Scopes: {"fillet-native-contours": 161, "chamfer-native-contours": 487}. Raw seeds and native reference properties are
exactly unchanged. 4 requests now express corrected effective contour
scope; 644 retain the same intent.

Statuses: {"input-import-failed": 96, "operation-succeeded": 231, "operation-failed": 202, "unhandled-intent": 110, "input-inconsistent": 9}.
Checks among 231 constructions: {"constructed": 231, "input_unchanged": 231, "geometry_consistent": 231, "mesh_closed": 231, "step_prepared": 231, "native_valid": 231, "truck_step_valid": 231, "single_solid": 231, "sampled_distance": 220}.
220 pass the combined limited sampled comparison at 0.001.
No prior successful check regresses for unchanged intent. 0 previously
successful seed-only records lose checks under corrected full scope; these are
explicitly listed in comparison-summary.json and cannot be counted as equivalent
successful native operations. 231 prior successful results/comparisons are
exactly unchanged after excluding scope metadata; 0 have numerical changes.

Schema3 fillets capture the final native contour state after all Add calls.
Repeated seeds can replace radii; nonconstant laws are retained and explicitly
unhandled. Exact observed native settings drive contour discovery. Replay supports
rational C1 constant positive contours through existing solid/wire APIs and
rejects incomplete or contradictory scope. Legacy schema1 remains seed-only;
new native chamfer captures retain schema2 and their existing first-assignment
semantics. Native and Truck approximation tolerances are recorded independently;
no solver-setting equivalence is claimed.

33 Rust replay tests,13 capture tests,Clippy/action smoke/plugin and final replay
build pass. Five Rust and three Python regressions fail before their respective
implementation changes. Native controls:11valid outputs; replay7constructs and
4explicit unsupported intents. All7exportvalidsingleSTEP and meet .001 sampled
comparisons. A4maximum6.671012283394402e-6; box1.0223701381680905e-14 with analytic
material. Constructed solids and robust STEP are checked by shared harnesses.
No kernel source changed; prior504kernelpasses were not rerun or claimed anew.

Archive:7744 byte-verified members, SHA256 1921b48338d1380ebede0a022c0c2646443bf0d99a5205059fc775df35422e7f.
Finite sampled comparisons do not certify global distance, full parity,
continuity/history or latency. Native sequence inputs remain independent.
No push,PR,consumerchange or kernelpinchange.
