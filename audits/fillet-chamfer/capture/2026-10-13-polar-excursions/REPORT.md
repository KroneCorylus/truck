# Full capture after isolated polar seam recovery

Kernel: `5f5d5c255a3d69985fd1c9abc5b7398409e7a8fb`, from a clean tracked
checkout. Source and executable hashes were verified before and after the run
and again while packaging. All 156 native entries passed, producing 161
operation captures.

| Check | Previous capture | This capture |
| --- | ---: | ---: |
| Constructed with unchanged input and consistent geometry | 95 | 96 |
| Closed Truck mesh | 95 | 96 |
| Native-valid single-solid STEP export | 95 | 96 |
| Combined limited comparison pass at 0.001 | 86 | 87 |
| Operation failed | 48 | 48 |
| Input import failed | 9 | 8 |
| Input inconsistent | 0 | 0 |
| Unhandled intent | 8 | 8 |
| Incomplete capture | 1 | 1 |
| Timeout / panic | 0 / 0 | 0 / 0 |

J2's capped sphere moves from `NotSimpleWire` during import to successful
construction, closed mesh, valid STEP and a limited sampled comparison pass.
The targeted proof records independent sphere/fillet volume, radius, contact
normal, transformed/reversed selection and STEP round-trip checks:
`truck-benchmarks/results/2026-10-13-polar-excursions/REPORT.md`.

**All 95 previous successful Truck results and sampled comparison records
are exactly unchanged.** No check regressed and no previous numeric field
changed. The manifest and field-level comparison are adjacent.

All **2,201** archive members were byte-verified against recorded hashes.
Archive SHA-256:
`920a473a26e36e8d4b9d7ef4d0adf99c5a590487866786ed8885d378623c9cf8`.

These checks cover this captured subset. They do not establish full parity,
resolve the remaining nine sampled discrepancies, or fill the missing
dedicated chamfer fixture corpus. No performance claim is made.
