# Truck versus direct OpenCascade

OCCT 7.9.3; Truck `f98a764d9e2c77fef50ff283f520e40b143cdf53` plus `source.patch` and `sources.sha256`. 15 samples × 2 rounds; CPUs [0, 1, 2, 3].

Native warmed operation latency in milliseconds. Setup, process startup, final-result destruction and QA are outside timers. Preflight and every timed process must pass native geometry checks, closed mesh and analytic volume within 0.2%. A failed preflight is retained and that engine/case is skipped in subsequent rounds. Ratios require both engines to pass and agree on volume. Ratios above 1 favor Truck; none is an overall speed claim.

Truck operation/mesh tolerance: 0.001 mm; QA deflection: 0.001 mm, tightened for narrow intersections/walls, with a shared 1e-6 mm floor. OCCT uses native geometric tolerances, additional fuzzy tolerance 0 mm, and absolute mesh deflection matching Truck. These settings are different contracts.

| Case | Workers | Truck median ms | OCCT median ms | OCCT / Truck | QA |
|---|---:|---:|---:|---:|---|
| holes_batch/1 | 1 | 1.538 | 1.566 | 1.02× | pass |
| holes_batch/1 | 4 | 1.229 | 1.384 | 1.13× | pass |
| holes_batch/10 | 1 | 14.61 | 9.416 | 0.64× | pass |
| holes_batch/10 | 4 | 12.03 | 6.871 | 0.57× | pass |
| holes_batch/30 | 1 | 44.95 | 27.08 | 0.60× | pass |
| holes_batch/30 | 4 | 38.26 | 18.93 | 0.49× | pass |
| holes_batch/100 | 1 | 188.6 | 89.42 | 0.47× | pass |
| holes_batch/100 | 4 | 164.6 | 62.1 | 0.38× | pass |
| holes_sequential/1 | 1 | 1.499 | 1.539 | 1.03× | pass |
| holes_sequential/1 | 4 | 1.244 | 1.395 | 1.12× | pass |
| holes_sequential/10 | 1 | 63.04 | 20.94 | 0.33× | pass |
| holes_sequential/10 | 4 | 37.32 | 18.58 | 0.50× | pass |
| holes_sequential/30 | 1 | 473 | 102.1 | 0.22× | pass |
| holes_sequential/30 | 4 | 255.2 | 86.76 | 0.34× | pass |
| holes_sequential/100 | 1 | 5586 | 796 | 0.14× | pass |
| holes_sequential/100 | 4 | 2975 | 649.5 | 0.22× | pass |
| knurl/16 | 1 | 9.534 | 30.12 | 3.16× | pass |
| knurl/16 | 4 | 8.001 | 19.17 | 2.40× | pass |
| knurl/64 | 1 | 34.85 | 133.4 | 3.83× | pass |
| knurl/64 | 4 | 34.29 | 88.28 | 2.57× | pass |
| knurl/128 | 1 | 86.41 | 310 | 3.59× | pass |
| knurl/128 | 4 | 89.07 | 212.8 | 2.39× | pass |
| thread/1 | 1 | 117.2 | 761.7 | 6.50× | pass |
| thread/1 | 4 | 91.91 | 389.4 | 4.24× | pass |
| thread/4 | 1 | 327.8 | 1549 | 4.72× | pass |
| thread/4 | 4 | 274 | 869.9 | 3.17× | pass |
| near_tangent/0.01 | 1 | 2.203 | 3.628 | 1.65× | pass |
| near_tangent/0.01 | 4 | 1.621 | 3.29 | 2.03× | pass |
| near_tangent/0.001 | 1 | 2.27 | 4.303 | 1.90× | pass |
| near_tangent/0.001 | 4 | 1.73 | 3.548 | 2.05× | pass |
| thin_wall/0.1 | 1 | 6.607 | 1.445 | 0.22× | pass |
| thin_wall/0.1 | 4 | 4.901 | 1.392 | 0.28× | pass |
| thin_wall/0.01 | 1 | 8.876 | 1.464 | 0.16× | pass |
| thin_wall/0.01 | 4 | 6.961 | 1.385 | 0.20× | pass |
| fillet_bore/0.5 | 1 | 10.63 | 0.3192 | 0.03× | pass |
| fillet_bore/0.5 | 4 | 10.59 | 0.3297 | 0.03× | pass |
| mesh_plate/1 | 1 | 0.6283 | 3.132 | 4.99× | pass |
| mesh_plate/1 | 4 | 0.3547 | 1.359 | 3.83× | pass |
| mesh_plate/100 | 1 | 62.23 | 594.7 | 9.56× | pass |
| mesh_plate/100 | 4 | 29.58 | 292.8 | 9.90× | pass |

## Variation and memory

P95 is the empirical nearest-rank value across raw samples, not a confidence interval. Peak RSS is the process high-water mark read **before postflight QA**, including fixture setup, warmup, libraries, thread pools and retained output. It is not isolated allocation cost. Thread setup includes STEP import/checking on OCCT.

| Case | Workers | Engine | P95 ms | Max ms | Peak RSS MiB | Round medians ms |
|---|---:|---|---:|---:|---:|---|
| holes_batch/1 | 1 | truck | 1.684 | 1.849 | 6.09 | 1.523, 1.552 |
| holes_batch/1 | 1 | occt | 1.679 | 1.684 | 30.05 | 1.591, 1.541 |
| holes_batch/1 | 4 | truck | 1.384 | 1.424 | 6.77 | 1.215, 1.243 |
| holes_batch/1 | 4 | occt | 1.473 | 1.49 | 30.14 | 1.367, 1.401 |
| holes_batch/10 | 1 | truck | 14.87 | 14.98 | 8.48 | 14.53, 14.69 |
| holes_batch/10 | 1 | occt | 9.625 | 9.74 | 31.07 | 9.376, 9.456 |
| holes_batch/10 | 4 | truck | 12.67 | 12.7 | 9.30 | 12.11, 11.94 |
| holes_batch/10 | 4 | occt | 7.002 | 7.048 | 31.48 | 6.862, 6.879 |
| holes_batch/30 | 1 | truck | 45.99 | 46.39 | 13.81 | 45.1, 44.81 |
| holes_batch/30 | 1 | occt | 27.7 | 27.72 | 33.46 | 26.74, 27.42 |
| holes_batch/30 | 4 | truck | 40.02 | 42.28 | 14.66 | 38.06, 38.46 |
| holes_batch/30 | 4 | occt | 19.28 | 19.35 | 33.93 | 18.89, 18.96 |
| holes_batch/100 | 1 | truck | 193.1 | 199.4 | 32.57 | 187.2, 190.1 |
| holes_batch/100 | 1 | occt | 92.25 | 95.3 | 41.37 | 89.11, 89.73 |
| holes_batch/100 | 4 | truck | 167.9 | 168.1 | 33.60 | 164.7, 164.5 |
| holes_batch/100 | 4 | occt | 63.24 | 63.44 | 42.22 | 61.9, 62.3 |
| holes_sequential/1 | 1 | truck | 1.605 | 1.622 | 6.18 | 1.488, 1.51 |
| holes_sequential/1 | 1 | occt | 1.764 | 1.78 | 30.00 | 1.536, 1.541 |
| holes_sequential/1 | 4 | truck | 1.474 | 1.579 | 6.71 | 1.19, 1.298 |
| holes_sequential/1 | 4 | occt | 1.498 | 1.51 | 30.23 | 1.378, 1.411 |
| holes_sequential/10 | 1 | truck | 68.53 | 69.26 | 11.08 | 63.05, 63.02 |
| holes_sequential/10 | 1 | occt | 22.6 | 24.08 | 30.46 | 20.65, 21.23 |
| holes_sequential/10 | 4 | truck | 38.72 | 39.02 | 12.62 | 37.16, 37.48 |
| holes_sequential/10 | 4 | occt | 19.85 | 20.23 | 30.75 | 18.5, 18.66 |
| holes_sequential/30 | 1 | truck | 489.5 | 495.9 | 20.41 | 465, 481 |
| holes_sequential/30 | 1 | occt | 106.5 | 107.2 | 30.82 | 102.1, 102.1 |
| holes_sequential/30 | 4 | truck | 261.1 | 262.4 | 29.14 | 257.7, 252.8 |
| holes_sequential/30 | 4 | occt | 94.1 | 103 | 31.43 | 85.5, 88.02 |
| holes_sequential/100 | 1 | truck | 5779 | 5832 | 56.33 | 5555, 5617 |
| holes_sequential/100 | 1 | occt | 805.9 | 826.2 | 32.79 | 793.8, 798.2 |
| holes_sequential/100 | 4 | truck | 3188 | 3194 | 91.31 | 2967, 2984 |
| holes_sequential/100 | 4 | occt | 666.2 | 669.5 | 33.82 | 651.6, 647.3 |
| knurl/16 | 1 | truck | 9.802 | 9.917 | 7.66 | 9.561, 9.506 |
| knurl/16 | 1 | occt | 30.41 | 30.48 | 33.57 | 30.18, 30.06 |
| knurl/16 | 4 | truck | 8.629 | 8.799 | 8.32 | 7.992, 8.01 |
| knurl/16 | 4 | occt | 19.67 | 19.69 | 34.15 | 19.2, 19.14 |
| knurl/64 | 1 | truck | 36.03 | 36.23 | 11.76 | 35.09, 34.61 |
| knurl/64 | 1 | occt | 136 | 136.3 | 43.41 | 134.4, 132.4 |
| knurl/64 | 4 | truck | 34.87 | 36.13 | 12.64 | 34.47, 34.11 |
| knurl/64 | 4 | occt | 89.17 | 89.4 | 44.33 | 88.53, 88.03 |
| knurl/128 | 1 | truck | 87.42 | 87.96 | 20.05 | 86.3, 86.52 |
| knurl/128 | 1 | occt | 318.5 | 328.7 | 56.38 | 313.3, 306.7 |
| knurl/128 | 4 | truck | 90.76 | 90.77 | 20.74 | 89.44, 88.7 |
| knurl/128 | 4 | occt | 220.4 | 226.5 | 57.64 | 215.7, 209.9 |
| thread/1 | 1 | truck | 118.6 | 147.1 | 15.29 | 116.7, 117.8 |
| thread/1 | 1 | occt | 780.7 | 793.7 | 112.48 | 768.2, 755.1 |
| thread/1 | 4 | truck | 94.91 | 98.94 | 19.12 | 93.56, 90.26 |
| thread/1 | 4 | occt | 393.2 | 394.7 | 131.07 | 389, 389.9 |
| thread/4 | 1 | truck | 332.8 | 333.2 | 24.57 | 324.4, 331.1 |
| thread/4 | 1 | occt | 1676 | 1722 | 187.46 | 1540, 1557 |
| thread/4 | 4 | truck | 277.8 | 283 | 31.14 | 271.3, 276.6 |
| thread/4 | 4 | occt | 884.7 | 886.4 | 239.78 | 875.9, 863.8 |
| near_tangent/0.01 | 1 | truck | 2.505 | 2.53 | 6.40 | 2.166, 2.241 |
| near_tangent/0.01 | 1 | occt | 3.723 | 3.777 | 30.31 | 3.588, 3.669 |
| near_tangent/0.01 | 4 | truck | 2.026 | 2.087 | 6.95 | 1.638, 1.604 |
| near_tangent/0.01 | 4 | occt | 4.456 | 4.867 | 30.82 | 3.496, 3.083 |
| near_tangent/0.001 | 1 | truck | 2.506 | 2.535 | 6.31 | 2.26, 2.28 |
| near_tangent/0.001 | 1 | occt | 4.401 | 4.595 | 30.18 | 4.321, 4.285 |
| near_tangent/0.001 | 4 | truck | 1.963 | 2.102 | 6.82 | 1.736, 1.723 |
| near_tangent/0.001 | 4 | occt | 3.714 | 3.853 | 30.68 | 3.574, 3.523 |
| thin_wall/0.1 | 1 | truck | 6.967 | 7.111 | 6.70 | 6.679, 6.535 |
| thin_wall/0.1 | 1 | occt | 1.567 | 1.819 | 30.09 | 1.45, 1.44 |
| thin_wall/0.1 | 4 | truck | 5.112 | 5.227 | 7.23 | 4.985, 4.818 |
| thin_wall/0.1 | 4 | occt | 1.525 | 1.543 | 30.38 | 1.403, 1.382 |
| thin_wall/0.01 | 1 | truck | 9.31 | 9.472 | 7.12 | 8.812, 8.939 |
| thin_wall/0.01 | 1 | occt | 1.533 | 1.557 | 30.02 | 1.473, 1.455 |
| thin_wall/0.01 | 4 | truck | 7.391 | 7.51 | 7.67 | 6.967, 6.955 |
| thin_wall/0.01 | 4 | occt | 1.518 | 1.529 | 30.23 | 1.393, 1.378 |
| fillet_bore/0.5 | 1 | truck | 11.24 | 11.58 | 6.45 | 10.64, 10.61 |
| fillet_bore/0.5 | 1 | occt | 0.3552 | 0.3612 | 32.57 | 0.3148, 0.3235 |
| fillet_bore/0.5 | 4 | truck | 11.17 | 11.23 | 6.88 | 10.69, 10.48 |
| fillet_bore/0.5 | 4 | occt | 0.4992 | 0.5012 | 32.73 | 0.3238, 0.3355 |
| mesh_plate/1 | 1 | truck | 0.8562 | 0.8874 | 5.62 | 0.6384, 0.6181 |
| mesh_plate/1 | 1 | occt | 3.352 | 3.398 | 28.82 | 3.204, 3.06 |
| mesh_plate/1 | 4 | truck | 0.5413 | 0.5907 | 6.20 | 0.381, 0.3284 |
| mesh_plate/1 | 4 | occt | 1.455 | 1.571 | 29.55 | 1.309, 1.409 |
| mesh_plate/100 | 1 | truck | 66.14 | 66.39 | 33.94 | 61.67, 62.79 |
| mesh_plate/100 | 1 | occt | 760.9 | 834.4 | 134.94 | 592.6, 596.8 |
| mesh_plate/100 | 4 | truck | 32.05 | 33.99 | 48.60 | 30.18, 28.98 |
| mesh_plate/100 | 4 | occt | 348.8 | 392.4 | 340.86 | 295.8, 289.9 |

## Failures

All recorded checks passed.

## Scope

- Knurl cases are straight axial triangular grooves, not diamond/helical knurling. Threads are external 60° grooves, 1.25 mm pitch, radius 4 mm, depth 0.4 mm; native Truck cutters are exported to STEP for OCCT outside timing. OCCT's standard STEP transfer may heal geometry; operands are not asserted to have identical topology.
- Hole and meshing fixtures reuse the existing benchmark dimensions. Mesh results include fresh triangulation and triangle extraction; OCCT also copies topology without cached triangulation in the timed operation.
- One worker disables OCCT Boolean/mesh parallelism. Multiple workers use OCCT's own thread pool with the stated limit; Truck uses Rayon. Both share the same CPU affinity. Fillet internal parallelism is API-dependent.
- Mesh volume/closure and native checks are sanity gates, not proof of identical surfaces or universal modeling correctness. Truck's mesh checks and OCCT's native analyzer have different implementations. Topology counts may differ.
- All engines and builds run sequentially. Even-numbered rounds reverse execution order. The active desktop, power policy, installed OCCT build and compiler differences remain sources of variation. No UI latency is measured.

See `metadata.json`, `records.jsonl`, `summary.json`, `sources.sha256`, `source.patch` and raw logs. Methodology and reproduction: [BENCHMARKS.md](../../../BENCHMARKS.md#direct-opencascade-comparison).
