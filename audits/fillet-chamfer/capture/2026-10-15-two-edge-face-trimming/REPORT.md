# Full shared-neighbor trimming capture

Clean implementation revision c26b6780c14e260bf95284a490a9ac0b0e37d280. All 396 native entries pass, yielding
648 operation captures. 222 constructions and 208 combined
limited comparisons at 0.001. 222 previous successful result/comparison
records are exactly unchanged; 0 have numerical changes. No previous
successful check regresses. Status changes: [].
Statuses: {"input-import-failed": 96, "input-inconsistent": 9, "operation-failed": 195, "operation-succeeded": 222, "unhandled-intent": 126}.

The targeted parent-failing tests exercise public simple chamfer and fillet face
patches bounded by an arc and a chord. They verify both neighbor cuts, retained
material boundary, analytic contacts/blend surfaces, orientation and immutable
input. They do not establish a full solid edit for that fixture.

The automatic capture remains seed-only for native tangent propagation. Sequence
steps replay independent native inputs. No complete-parity or performance claim.
7717 byte-verified archive members; SHA256 15c281696796711c09ed8eead3028809e7dada7baf1d2a424f4006cb2b94c5b5.
Raw capture: target/occt-capture-two-edge-face-trimming.
Implementation proof: truck-benchmarks/results/2026-10-15-two-edge-face-trimming.
Catalog verification and 31 audit tests: catalog-checks.txt.

Next: native tangent contour capture and native first-assignment semantics.
Other remaining gaps include curved reference faces, C1 input fidelity, native
shell inputs, four missing fixtures, sampled differences, unsupported operation
families and the previously recorded PCurve precision test.
