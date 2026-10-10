# Truck versus direct OpenCascade

OCCT 7.9.3; Truck `3729a083932f5d73a29a4e84cc0b03866869aa27` plus `source.patch` and `sources.sha256`. 15 samples × 2 rounds; CPUs [0, 1, 2, 3].

Native warmed operation latency in milliseconds. Setup, process startup, final-result destruction and QA are outside timers. Preflight and every timed process must pass native geometry checks, closed mesh and analytic volume within 0.2%. A failed preflight is retained and that engine/case is skipped in subsequent rounds. Ratios require both engines to pass and agree on volume. Ratios above 1 favor Truck; none is an overall speed claim.

Truck operation/mesh tolerance: 0.001 mm; QA deflection: 0.001 mm, tightened for narrow intersections/walls, with a shared 1e-6 mm floor. OCCT uses native geometric tolerances, additional fuzzy tolerance 0 mm, and absolute mesh deflection matching Truck. These settings are different contracts.

| Case | Workers | Truck median ms | OCCT median ms | OCCT / Truck | QA |
|---|---:|---:|---:|---:|---|
| holes_batch/1 | 1 | 0.02073 | 1.538 | 74.19× | pass |
| holes_batch/1 | 4 | 0.01969 | 1.38 | 70.08× | pass |
| holes_batch/10 | 1 | 14.55 | 9.33 | 0.64× | pass |
| holes_batch/10 | 4 | 11.92 | 6.824 | 0.57× | pass |
| holes_batch/30 | 1 | 44.76 | 26.57 | 0.59× | pass |
| holes_batch/30 | 4 | 37.32 | 18.97 | 0.51× | pass |
| holes_batch/100 | 1 | 186.2 | 89.77 | 0.48× | pass |
| holes_batch/100 | 4 | 162.2 | 61.91 | 0.38× | pass |
| holes_sequential/1 | 1 | 0.02052 | 1.526 | 74.34× | pass |
| holes_sequential/1 | 4 | 0.02004 | 1.361 | 67.92× | pass |
| holes_sequential/10 | 1 | 0.4829 | 20.57 | 42.59× | pass |
| holes_sequential/10 | 4 | 0.4859 | 18.3 | 37.67× | pass |
| holes_sequential/30 | 1 | 3.355 | 99.77 | 29.73× | pass |
| holes_sequential/30 | 4 | 3.402 | 85.65 | 25.18× | pass |
| holes_sequential/100 | 1 | 32.86 | 791.1 | 24.08× | pass |
| holes_sequential/100 | 4 | 32.85 | 651.3 | 19.83× | pass |
| knurl/16 | 1 | 9.512 | 29.79 | 3.13× | pass |
| knurl/16 | 4 | 8.066 | 19.37 | 2.40× | pass |
| knurl/64 | 1 | 34.96 | 133.6 | 3.82× | pass |
| knurl/64 | 4 | 34.38 | 88.39 | 2.57× | pass |
| knurl/128 | 1 | 86.59 | 307.3 | 3.55× | pass |
| knurl/128 | 4 | 88.49 | 211.4 | 2.39× | pass |
| thread/1 | 1 | 116 | 760.7 | 6.56× | pass |
| thread/1 | 4 | 92.52 | 387.2 | 4.19× | pass |
| thread/4 | 1 | 321.4 | 1547 | 4.81× | pass |
| thread/4 | 4 | 276.8 | 862.6 | 3.12× | pass |
| near_tangent/0.01 | 1 | 2.174 | 3.589 | 1.65× | pass |
| near_tangent/0.01 | 4 | 1.623 | 3.124 | 1.92× | pass |
| near_tangent/0.001 | 1 | 2.265 | 4.301 | 1.90× | pass |
| near_tangent/0.001 | 4 | 1.708 | 3.494 | 2.05× | pass |
| thin_wall/0.1 | 1 | 0.01669 | 1.491 | 89.35× | pass |
| thin_wall/0.1 | 4 | 0.01633 | 1.338 | 81.94× | pass |
| thin_wall/0.01 | 1 | 0.01744 | 1.533 | 87.91× | pass |
| thin_wall/0.01 | 4 | 0.01564 | 1.392 | 88.98× | pass |
| fillet_bore/0.5 | 1 | 10.4 | 0.3096 | 0.03× | pass |
| fillet_bore/0.5 | 4 | 10.47 | 0.3179 | 0.03× | pass |
| mesh_plate/1 | 1 | 0.5908 | 3.105 | 5.26× | pass |
| mesh_plate/1 | 4 | 0.3562 | 1.29 | 3.62× | pass |
| mesh_plate/100 | 1 | 61.58 | 593.8 | 9.64× | pass |
| mesh_plate/100 | 4 | 28.96 | 280.7 | 9.69× | pass |

## Variation and memory

P95 is the empirical nearest-rank value across raw samples, not a confidence interval. Peak RSS is the process high-water mark read **before postflight QA**, including fixture setup, warmup, libraries, thread pools and retained output. It is not isolated allocation cost. Thread setup includes STEP import/checking on OCCT.

| Case | Workers | Engine | P95 ms | Max ms | Peak RSS MiB | Round medians ms |
|---|---:|---|---:|---:|---:|---|
| holes_batch/1 | 1 | truck | 0.02533 | 0.02831 | 5.05 | 0.0218, 0.01965 |
| holes_batch/1 | 1 | occt | 1.62 | 1.638 | 30.02 | 1.495, 1.58 |
| holes_batch/1 | 4 | truck | 0.02667 | 0.06005 | 5.05 | 0.01992, 0.01946 |
| holes_batch/1 | 4 | occt | 1.492 | 1.522 | 30.17 | 1.379, 1.38 |
| holes_batch/10 | 1 | truck | 14.99 | 15 | 8.74 | 14.38, 14.73 |
| holes_batch/10 | 1 | occt | 9.706 | 9.863 | 31.08 | 9.286, 9.373 |
| holes_batch/10 | 4 | truck | 12.49 | 12.68 | 9.36 | 12.02, 11.83 |
| holes_batch/10 | 4 | occt | 7.047 | 7.157 | 31.40 | 6.749, 6.898 |
| holes_batch/30 | 1 | truck | 45.59 | 45.59 | 13.98 | 44.38, 45.14 |
| holes_batch/30 | 1 | occt | 26.88 | 26.89 | 33.53 | 26.57, 26.57 |
| holes_batch/30 | 4 | truck | 38.1 | 38.49 | 14.86 | 37.34, 37.31 |
| holes_batch/30 | 4 | occt | 19.31 | 19.33 | 33.59 | 18.9, 19.03 |
| holes_batch/100 | 1 | truck | 189.6 | 189.7 | 32.47 | 184.1, 188.3 |
| holes_batch/100 | 1 | occt | 90.83 | 98.79 | 41.36 | 89.01, 90.54 |
| holes_batch/100 | 4 | truck | 165.3 | 165.4 | 33.75 | 161.6, 162.8 |
| holes_batch/100 | 4 | occt | 62.65 | 62.84 | 41.40 | 61.92, 61.9 |
| holes_sequential/1 | 1 | truck | 0.05029 | 0.07622 | 4.99 | 0.02001, 0.02104 |
| holes_sequential/1 | 1 | occt | 1.62 | 1.639 | 30.00 | 1.532, 1.52 |
| holes_sequential/1 | 4 | truck | 0.02427 | 0.02436 | 4.84 | 0.01953, 0.02056 |
| holes_sequential/1 | 4 | occt | 1.411 | 1.468 | 30.20 | 1.348, 1.374 |
| holes_sequential/10 | 1 | truck | 0.5301 | 0.5407 | 5.27 | 0.4762, 0.4897 |
| holes_sequential/10 | 1 | occt | 20.77 | 20.97 | 30.10 | 20.63, 20.5 |
| holes_sequential/10 | 4 | truck | 0.5825 | 0.6069 | 5.43 | 0.4865, 0.4854 |
| holes_sequential/10 | 4 | occt | 18.74 | 18.82 | 30.62 | 18.4, 18.2 |
| holes_sequential/30 | 1 | truck | 3.502 | 3.583 | 5.80 | 3.344, 3.367 |
| holes_sequential/30 | 1 | occt | 100.6 | 101 | 30.86 | 100.2, 99.38 |
| holes_sequential/30 | 4 | truck | 3.702 | 3.718 | 5.82 | 3.433, 3.372 |
| holes_sequential/30 | 4 | occt | 86.31 | 86.85 | 31.45 | 85.98, 85.33 |
| holes_sequential/100 | 1 | truck | 33.31 | 33.37 | 6.53 | 32.8, 32.92 |
| holes_sequential/100 | 1 | occt | 795.1 | 795.4 | 32.33 | 792.8, 789.4 |
| holes_sequential/100 | 4 | truck | 33.82 | 34.6 | 6.60 | 32.79, 32.91 |
| holes_sequential/100 | 4 | occt | 661.6 | 667.5 | 33.60 | 650.3, 652.3 |
| knurl/16 | 1 | truck | 9.748 | 9.937 | 7.82 | 9.487, 9.538 |
| knurl/16 | 1 | occt | 30.11 | 30.18 | 33.58 | 29.85, 29.74 |
| knurl/16 | 4 | truck | 8.305 | 8.707 | 8.61 | 8.011, 8.12 |
| knurl/16 | 4 | occt | 19.64 | 19.65 | 34.24 | 19.22, 19.52 |
| knurl/64 | 1 | truck | 35.56 | 35.63 | 12.01 | 34.59, 35.34 |
| knurl/64 | 1 | occt | 135.5 | 135.5 | 43.33 | 132.5, 134.7 |
| knurl/64 | 4 | truck | 34.91 | 35.21 | 12.80 | 34.18, 34.58 |
| knurl/64 | 4 | occt | 89.35 | 89.62 | 43.31 | 87.98, 88.8 |
| knurl/128 | 1 | truck | 88.15 | 88.55 | 20.11 | 86.13, 87.06 |
| knurl/128 | 1 | occt | 312.7 | 316.2 | 56.24 | 306.2, 308.5 |
| knurl/128 | 4 | truck | 90.41 | 90.97 | 20.77 | 88.03, 88.94 |
| knurl/128 | 4 | occt | 213.3 | 214.3 | 57.46 | 210.7, 212.1 |
| thread/1 | 1 | truck | 116.7 | 117 | 15.54 | 116.1, 116 |
| thread/1 | 1 | occt | 770.1 | 773.3 | 112.38 | 756.6, 764.8 |
| thread/1 | 4 | truck | 96.02 | 96.12 | 19.00 | 91.77, 93.26 |
| thread/1 | 4 | occt | 389 | 389.1 | 134.79 | 387.4, 387 |
| thread/4 | 1 | truck | 325.1 | 325.7 | 26.35 | 321.8, 321 |
| thread/4 | 1 | occt | 1562 | 1567 | 187.45 | 1549, 1545 |
| thread/4 | 4 | truck | 282.1 | 282.6 | 31.52 | 276.2, 277.5 |
| thread/4 | 4 | occt | 874.8 | 882.6 | 242.21 | 862.8, 862.5 |
| near_tangent/0.01 | 1 | truck | 2.358 | 2.449 | 6.53 | 2.187, 2.161 |
| near_tangent/0.01 | 1 | occt | 3.793 | 3.867 | 30.36 | 3.555, 3.623 |
| near_tangent/0.01 | 4 | truck | 1.954 | 2.181 | 7.01 | 1.584, 1.662 |
| near_tangent/0.01 | 4 | occt | 3.199 | 3.255 | 30.68 | 3.134, 3.114 |
| near_tangent/0.001 | 1 | truck | 2.401 | 2.416 | 6.53 | 2.272, 2.259 |
| near_tangent/0.001 | 1 | occt | 4.5 | 4.581 | 30.32 | 4.257, 4.345 |
| near_tangent/0.001 | 4 | truck | 1.971 | 1.986 | 6.82 | 1.687, 1.729 |
| near_tangent/0.001 | 4 | occt | 3.667 | 3.755 | 30.82 | 3.441, 3.546 |
| thin_wall/0.1 | 1 | truck | 0.02251 | 0.02734 | 5.08 | 0.01564, 0.01774 |
| thin_wall/0.1 | 1 | occt | 1.648 | 1.664 | 30.12 | 1.455, 1.527 |
| thin_wall/0.1 | 4 | truck | 0.02272 | 0.0252 | 5.05 | 0.01541, 0.01724 |
| thin_wall/0.1 | 4 | occt | 1.474 | 1.57 | 30.44 | 1.331, 1.344 |
| thin_wall/0.01 | 1 | truck | 0.02171 | 0.02193 | 5.08 | 0.01753, 0.01735 |
| thin_wall/0.01 | 1 | occt | 1.721 | 1.757 | 30.03 | 1.437, 1.629 |
| thin_wall/0.01 | 4 | truck | 0.01959 | 0.02127 | 5.09 | 0.01573, 0.01556 |
| thin_wall/0.01 | 4 | occt | 1.587 | 1.588 | 30.36 | 1.397, 1.387 |
| fillet_bore/0.5 | 1 | truck | 10.76 | 10.8 | 5.80 | 10.45, 10.36 |
| fillet_bore/0.5 | 1 | occt | 0.3676 | 0.3716 | 32.50 | 0.3069, 0.3122 |
| fillet_bore/0.5 | 4 | truck | 10.86 | 10.9 | 5.88 | 10.32, 10.62 |
| fillet_bore/0.5 | 4 | occt | 0.3804 | 0.3876 | 32.62 | 0.3258, 0.3101 |
| mesh_plate/1 | 1 | truck | 0.8188 | 0.8809 | 5.74 | 0.5681, 0.6134 |
| mesh_plate/1 | 1 | occt | 3.234 | 3.302 | 28.51 | 3.135, 3.076 |
| mesh_plate/1 | 4 | truck | 0.4883 | 0.4915 | 5.97 | 0.3689, 0.3434 |
| mesh_plate/1 | 4 | occt | 1.449 | 1.52 | 29.60 | 1.257, 1.322 |
| mesh_plate/100 | 1 | truck | 64.34 | 72.56 | 32.52 | 61.29, 61.87 |
| mesh_plate/100 | 1 | occt | 618.3 | 632.1 | 135.33 | 588.6, 599 |
| mesh_plate/100 | 4 | truck | 32.44 | 34.49 | 47.56 | 28.69, 29.22 |
| mesh_plate/100 | 4 | occt | 295.8 | 297.7 | 343.73 | 279.7, 281.6 |

## Failures

All recorded checks passed.

## Scope

- Knurl cases are straight axial triangular grooves, not diamond/helical knurling. Threads are external 60° grooves, 1.25 mm pitch, radius 4 mm, depth 0.4 mm; native Truck cutters are exported to STEP for OCCT outside timing. OCCT's standard STEP transfer may heal geometry; operands are not asserted to have identical topology.
- Hole and meshing fixtures reuse the existing benchmark dimensions. Mesh results include fresh triangulation and triangle extraction; OCCT also copies topology without cached triangulation in the timed operation.
- One worker disables OCCT Boolean/mesh parallelism. Multiple workers use OCCT's own thread pool with the stated limit; Truck uses Rayon. Both share the same CPU affinity. Fillet internal parallelism is API-dependent.
- Mesh volume/closure and native checks are sanity gates, not proof of identical surfaces or universal modeling correctness. Truck's mesh checks and OCCT's native analyzer have different implementations. Topology counts may differ.
- All engines and builds run sequentially. Even-numbered rounds reverse execution order. The active desktop, power policy, installed OCCT build and compiler differences remain sources of variation. No UI latency is measured.

See `metadata.json`, `records.jsonl`, `summary.json`, `sources.sha256`, `source.patch` and raw logs. Methodology and reproduction: [BENCHMARKS.md](../../../../BENCHMARKS.md#direct-opencascade-comparison).
