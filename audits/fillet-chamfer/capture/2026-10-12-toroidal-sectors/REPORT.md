# Captured replay after circular-meridian and torus-orientation repair

The clean run at 0d3a0259 (kernel change 88f64f08) passes all 156 native
entries. Its 161 captures contain 77 constructed Truck results, 37
construction failures, 25 import failures, 11 inconsistent inputs, eight
unhandled intents, two timeouts and one incomplete capture. No panics occur.

Twelve inputs previously rejected as inconsistent now pass validation.
E5, E7 and F2 now construct; E6/E8/F1/F3/H3/H5 reach the existing nonplanar
multi-edge limitation, and E9 reaches but fails face trimming. H2 and H4
reach further execution but exceed the 120-second limit; the interrupted
processes do not establish their final geometry or the failing substage.
No previously constructed operation regresses.

All 77 constructed results preserve input and geometric consistency, close
their Truck meshes, prepare STEP and produce native-valid single solids.
Seventy-five meet the combined limited checks, including sampled boundary
distances no greater than 0.001 in either direction. G1 and tolerance-grid
A1 retain their preceding discrepancies.

| Case | Native to Truck maximum | Truck to native maximum | Truck STEP volume |
| --- | --- | --- | --- |
| E5 | 0.00001751353523 | 0.00002376984611 | 2025879.4706969126 |
| E7 | 0.000008712630293 | 0.000009136871315 | 2027063.093501775 |
| F2 | 0.000002562586783 | 0.000002101527631 | 1534561.2578725992 |

The input/round-trip regressions and independent material-volume formulas
are retained in `truck-shapeops/tests/imported_toroidal_sectors.rs` and its
fixture README. Circle and orientation regressions isolate the parent
failures. Five-crate release validation records 885 passes, one unchanged
PCurve precision failure and five existing ignored tests. Clippy and action
smoke pass; construction controls retain identical volumes and similar
medians. See `truck-benchmarks/results/2026-10-12-toroidal-sectors/REPORT.md`.

All case artifact hashes were verified before archiving. The manifest
records the clean source, replay executable, native runtime and harness.
These sampled comparisons establish limited evidence for the three new
constructed results; full feature-parity verdicts remain unverified.
