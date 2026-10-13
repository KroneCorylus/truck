# Full capture after extrusion pcurve and STEP orientation recovery

Kernel: `7733ae63c1b9fd3fc30dae669ddc40920df2597a`, from a clean tracked
checkout. The recorded source and executable hashes were verified before and
after execution and again while packaging. All 156 native entries passed,
producing 161 operation captures.

| Check | Previous capture | This capture |
| --- | ---: | ---: |
| Constructed with unchanged input and consistent geometry | 93 | 95 |
| Closed Truck mesh | 93 | 95 |
| Native-valid single-solid STEP export | 93 | 95 |
| Combined limited comparison pass at 0.001 | 86 | 86 |
| Operation failed | 48 | 48 |
| Input import failed | 11 | 9 |
| Input inconsistent | 0 | 0 |
| Unhandled intent | 8 | 8 |
| Incomplete capture | 1 | 1 |
| Timeout / panic | 0 / 0 | 0 / 0 |

J6 and J8 move from unsupported imported geometry to successful construction,
closed mesh and valid STEP. Their maximum sampled native/Truck distances
remain above 0.001, so neither is counted as a limited comparison pass.
The targeted proof contains independent dimension-based volume, rolling-radius
and normal checks, including measurements of the native reference deviations:
`truck-benchmarks/results/2026-10-13-extruded-pcurves/REPORT.md`.

No previously passing check regressed. Of the 93 earlier successful captures,
**92 retain identical complete Truck results and sampled comparisons**. F4's
reported Truck STEP volume changes by one floating-point step, from
3105005.0821237112 to 3105005.082123711; all its other recorded fields are
unchanged. This is the only numerical change among earlier successes.

The full manifest and field-level comparison are adjacent. All **2,198**
archive members were byte-verified against their recorded hashes.
Archive SHA-256:
`ef7ef9dc061303f6fe98f2fe4ec9f14982f0e922753d9c0e8ce5f6e215726a23`.

These results establish regression checks over this captured subset. They do
not establish full fillet/chamfer parity, resolve the remaining discrepancies,
or fill the missing dedicated chamfer fixture corpus. No performance claim
is made.
