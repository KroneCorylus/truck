# Captured replay after periodic torus import repair

The clean run at 2099d2d8 (kernel change 16958067) passes all 156 native
entries. Its 161 captures contain 80 constructed Truck results, 37
construction failures, 21 import failures, 11 inconsistent inputs, eight
unhandled intents, two timeouts, one panic and one incomplete capture.

J3, J4 and J5 advance from failed import to successful construction.
G9's second operation advances past torus projection and exposes a panic
in closed-face preprocessing: its proposed divisor reuses the endpoints
of an existing seam, creating a junction the boundary assembler cannot
handle. This run records that panic; a later fix does not change its result.
H2/H4 still exceed the 120-second replay limit. No previously constructed
operation regresses.

All 80 constructed results preserve input and geometric consistency, close
their Truck meshes, prepare STEP and produce native-valid single solids.
Seventy-eight meet the combined limited checks, including sampled boundary
distances no greater than 0.001 in either direction. G1 and tolerance-grid
A1 retain their preceding discrepancies.

| Case | Native to Truck maximum | Truck to native maximum | Truck STEP volume |
| --- | --- | --- | --- |
| J3 | 0.00004582396029 | 0.00004587250482 | 7395806.027226597 |
| J4 | 0.00001369635546 | 0.00001441799231 | 8964166.27798636 |
| J5 | 0.000006188411572 | 0.000007273034087 | 6757984.30110215 |

Input regressions retain independent material-volume checks, including
original-scale and transformed J3. Projection and pcurve tests isolate
the parent failures. Five-crate release validation records 893 passes,
one unchanged PCurve precision failure and five existing ignored tests.
All-target Clippy and action smoke pass. Import medians and their measured
validation overhead are retained in
`truck-benchmarks/results/2026-10-12-periodic-tori/REPORT.md`.

All 2,150 case artifact hashes were verified before archiving. The manifest
records the clean source, replay executable, native runtime and harness.
These sampled comparisons establish limited evidence for the three new
constructed results; full feature-parity verdicts remain unverified.
