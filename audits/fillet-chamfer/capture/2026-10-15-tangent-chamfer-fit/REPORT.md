# Full tangent chamfer fitting capture

Clean revision c4706b38a3118322407abae9bbc091ba7fe23955. All396 native entries pass, yielding648
operation captures. 222 constructions and 208 combined limited
comparisons at.001. 195 prior successful result/comparison records are
EXACTLY unchanged; 27 prior successful records have numeric changes.
No previous successful check regresses (see comparison-summary.json).

Statuses: {"input-import-failed": 96, "input-inconsistent": 9, "operation-failed": 195, "operation-succeeded": 222, "unhandled-intent": 126}.
No incomplete/panic/timeout unless explicitly listed above. This capture remains
seed-only for native tangent intent. The separate expanded A4 proof uses all3
native contour edges and produces valid single-solid STEP with sampled maximum
difference6.1772e-06. Do not merge that explicit proof into automatic corpus counts.
Sequences replay native inputs independently, not prior Truck outputs.

Changes refine spatial contact splines to agree geometrically with exact support
pcurves, and evaluate ruled-surface orientation in its normalized parameter chart.
Parent-failing regressions verify small-scale and shifted-parameter chamfers,
analytic material/bevels, rigid motion, reversal, closed meshes and STEP. A
face-level control retains behavior. Existing cone tests remain unchanged.

7717 byte-verified archive members; SHA256 d205f54b28111c7f33a0676201492b4d5f9b2bde0571df1f9d67e5a3debbd84d.
Raw target/occt-capture-tangent-chamfer-fit. Validation logs/metadata are in
truck-benchmarks/results/2026-10-15-tangent-chamfer-fit. Catalog checks and Python tests are in
catalog-checks.txt. No performance or full-parity claim.

Remaining: native tangent contour capture/first-assignment semantics; separate
simple_chamfer panic on a two-edge face; C1 input fidelity; curved reference
support; native inputs without solids; four missing fixture names; sampled
discrepancies and the prior PCurve precision assertion.
