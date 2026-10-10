# Full Truck action benchmarks

All 72 workloads completed on the parent and optimized kernels with one and four Rayon workers. Each configuration has two rounds of 20 samples, with engine order reversed in the second round. Every workload passed its fixture checks, and every measured row contains all 20 requested samples. This is 11,520 timed operations across eight suite runs.

Times are milliseconds. Medians below are the median of the two displayed Divan round medians. A speedup above 1 favors the optimized kernel. Small changes require repeat measurements; these tables retain every result.

The action suite uses its existing 0.01 modeling tolerance except for explicit tolerance sweeps. Its sequential-hole numbers therefore differ from the direct OCCT suite, which uses 0.001. Fixtures, output handling and correctness gates are unchanged. No kernel source was edited during this benchmark run.

## One Rayon worker

| Action | Parent ms | Optimized ms | Speedup | Parent round medians | Optimized round medians |
|---|---:|---:|---:|---|---|
| `boolean/circle_remove` | 2.313 | 2.36 | 0.980× | 2.352, 2.274 | 2.338, 2.382 |
| `boolean/intersect_cylinders/0.001` | 2.0925 | 2.0285 | 1.032× | 2.169, 2.016 | 2.021, 2.036 |
| `boolean/intersect_cylinders/0.01` | 0.57205 | 0.5636 | 1.015× | 0.5932, 0.5509 | 0.5735, 0.5537 |
| `boolean/intersect_cylinders/0.1` | 0.2649 | 0.2666 | 0.994× | 0.2685, 0.2613 | 0.268, 0.2652 |
| `boolean/intersect_nurbs_cylinders/0.001` | 11.28 | 11.05 | 1.021× | 11.23, 11.33 | 11.22, 10.88 |
| `boolean/intersect_nurbs_cylinders/0.01` | 4.73 | 4.6625 | 1.014× | 4.692, 4.768 | 4.742, 4.583 |
| `boolean/intersect_nurbs_cylinders/0.1` | 2.591 | 2.5705 | 1.008× | 2.555, 2.627 | 2.593, 2.548 |
| `boolean/subtract_batch/1` | 0.4592 | 0.01764 | 26.032× | 0.456, 0.4624 | 0.01744, 0.01784 |
| `boolean/subtract_batch/10` | 4.6705 | 4.6105 | 1.013× | 4.736, 4.605 | 4.63, 4.591 |
| `boolean/subtract_batch/100` | 65.885 | 65.635 | 1.004× | 66.31, 65.46 | 65.81, 65.46 |
| `boolean/subtract_batch/30` | 14.98 | 14.81 | 1.011× | 15.2, 14.76 | 14.85, 14.77 |
| `boolean/subtract_sequential/1` | 0.46705 | 0.01879 | 24.856× | 0.4767, 0.4574 | 0.01913, 0.01845 |
| `boolean/subtract_sequential/10` | 14.6 | 0.4834 | 30.203× | 14.69, 14.51 | 0.4819, 0.4849 |
| `boolean/subtract_sequential/100` | 1224 | 33.49 | 36.548× | 1234, 1214 | 33.62, 33.36 |
| `boolean/subtract_sequential/30` | 94.52 | 3.4185 | 27.650× | 95.16, 93.88 | 3.428, 3.409 |
| `boolean/union_cylinders/0.001` | 1.989 | 2.0315 | 0.979× | 1.994, 1.984 | 2.075, 1.988 |
| `boolean/union_cylinders/0.01` | 0.57215 | 0.5687 | 1.006× | 0.5657, 0.5786 | 0.5849, 0.5525 |
| `boolean/union_cylinders/0.1` | 0.27515 | 0.27545 | 0.999× | 0.27, 0.2803 | 0.2806, 0.2703 |
| `local/chamfer/1` | 0.03007 | 0.029885 | 1.006× | 0.0303, 0.02984 | 0.02988, 0.02989 |
| `local/chamfer/12` | 0.088955 | 0.08976 | 0.991× | 0.08888, 0.08903 | 0.09123, 0.08829 |
| `local/draft_box/1` | 0.046665 | 0.0476 | 0.980× | 0.04627, 0.04706 | 0.04813, 0.04707 |
| `local/draft_box/4` | 0.109 | 0.107 | 1.019× | 0.1104, 0.1076 | 0.1088, 0.1052 |
| `local/fillet/1` | 0.093225 | 0.08874 | 1.051× | 0.09774, 0.08871 | 0.08909, 0.08839 |
| `local/fillet/12` | 0.9244 | 0.86605 | 1.067× | 0.979, 0.8698 | 0.8765, 0.8556 |
| `local/rolling_fillet` | 16.47 | 16.42 | 1.003× | 16.53, 16.41 | 16.51, 16.33 |
| `local/shell_box` | 0.1199 | 0.12115 | 0.990× | 0.1229, 0.1169 | 0.1237, 0.1186 |
| `modeling/extrude/128` | 0.040515 | 0.041 | 0.988× | 0.03955, 0.04148 | 0.04196, 0.04004 |
| `modeling/extrude/32` | 0.01029 | 0.01051 | 0.979× | 0.01031, 0.01027 | 0.01058, 0.01044 |
| `modeling/extrude/4` | 0.001277 | 0.0013165 | 0.970× | 0.001257, 0.001297 | 0.001362, 0.001271 |
| `modeling/loft/2` | 0.031405 | 0.03092 | 1.016× | 0.03097, 0.03184 | 0.03099, 0.03085 |
| `modeling/loft/32` | 0.5045 | 0.51535 | 0.979× | 0.505, 0.504 | 0.5133, 0.5174 |
| `modeling/loft/8` | 0.10425 | 0.101 | 1.032× | 0.1068, 0.1017 | 0.101, 0.101 |
| `modeling/revolve/128` | 0.10245 | 0.099815 | 1.026× | 0.1028, 0.1021 | 0.1008, 0.09883 |
| `modeling/revolve/32` | 0.02533 | 0.02602 | 0.973× | 0.02529, 0.02537 | 0.02579, 0.02625 |
| `modeling/revolve/4` | 0.0029875 | 0.003075 | 0.972× | 0.00295, 0.003025 | 0.00313, 0.00302 |
| `modeling/sweep_path/1` | 0.002549 | 0.0026265 | 0.970× | 0.002609, 0.002489 | 0.002649, 0.002604 |
| `modeling/sweep_path/32` | 0.062385 | 0.062255 | 1.002× | 0.06297, 0.0618 | 0.06258, 0.06193 |
| `modeling/sweep_path/8` | 0.01568 | 0.01597 | 0.982× | 0.01556, 0.0158 | 0.01602, 0.01592 |
| `modeling/transform/1` | 0.002263 | 0.002219 | 1.020× | 0.002273, 0.002253 | 0.002194, 0.002244 |
| `modeling/transform/10` | 0.009337 | 0.009187 | 1.016× | 0.009362, 0.009312 | 0.009112, 0.009262 |
| `modeling/transform/30` | 0.02556 | 0.02503 | 1.021× | 0.02506, 0.02606 | 0.02498, 0.02508 |
| `projection/hidden_lines/1` | 0.22005 | 0.21985 | 1.001× | 0.2205, 0.2196 | 0.2182, 0.2215 |
| `projection/hidden_lines/10` | 1.708 | 1.737 | 0.983× | 1.722, 1.694 | 1.727, 1.747 |
| `projection/hidden_lines/30` | 5.5165 | 5.568 | 0.991× | 5.542, 5.491 | 5.523, 5.613 |
| `step/convert/1` | 0.020875 | 0.020375 | 1.025× | 0.02103, 0.02072 | 0.02064, 0.02011 |
| `step/convert/10` | 0.08648 | 0.08557 | 1.011× | 0.08765, 0.08531 | 0.0878, 0.08334 |
| `step/convert/30` | 0.23765 | 0.23545 | 1.009× | 0.2388, 0.2365 | 0.241, 0.2299 |
| `step/export/1` | 0.024615 | 0.023855 | 1.032× | 0.02373, 0.0255 | 0.02414, 0.02357 |
| `step/export/10` | 0.094885 | 0.09424 | 1.007× | 0.09401, 0.09576 | 0.09458, 0.0939 |
| `step/export/30` | 0.26255 | 0.2593 | 1.013× | 0.2602, 0.2649 | 0.2618, 0.2568 |
| `step/export_file/1` | 0.084405 | 0.09235 | 0.914× | 0.08699, 0.08182 | 0.09643, 0.08827 |
| `step/export_file/10` | 0.2158 | 0.21545 | 1.002× | 0.2062, 0.2254 | 0.2155, 0.2154 |
| `step/export_file/30` | 0.47345 | 0.47865 | 0.989× | 0.4815, 0.4654 | 0.4869, 0.4704 |
| `step/import/1` | 0.84185 | 0.8409 | 1.001× | 0.8441, 0.8396 | 0.8413, 0.8405 |
| `step/import/10` | 3.204 | 3.231 | 0.992× | 3.205, 3.203 | 3.26, 3.202 |
| `step/import/30` | 8.364 | 8.3615 | 1.000× | 8.352, 8.376 | 8.423, 8.3 |
| `step/import_file/1` | 0.85565 | 0.85025 | 1.006× | 0.8648, 0.8465 | 0.8603, 0.8402 |
| `step/import_file/10` | 3.2355 | 3.265 | 0.991× | 3.25, 3.221 | 3.304, 3.226 |
| `step/import_file/30` | 8.351 | 8.409 | 0.993× | 8.376, 8.326 | 8.454, 8.364 |
| `step/parse/1` | 0.8081 | 0.8167 | 0.989× | 0.8177, 0.7985 | 0.8231, 0.8103 |
| `step/parse/10` | 3.111 | 3.1045 | 1.002× | 3.131, 3.091 | 3.115, 3.094 |
| `step/parse/30` | 8.0395 | 8.0405 | 1.000× | 8.054, 8.025 | 8.073, 8.008 |
| `tessellation/cylinder_tolerance/0.001` | 0.46265 | 0.4588 | 1.008× | 0.4598, 0.4655 | 0.4592, 0.4584 |
| `tessellation/cylinder_tolerance/0.01` | 0.11685 | 0.116 | 1.007× | 0.1168, 0.1169 | 0.1164, 0.1156 |
| `tessellation/cylinder_tolerance/0.1` | 0.046355 | 0.046015 | 1.007× | 0.04649, 0.04622 | 0.04605, 0.04598 |
| `tessellation/mesh_to_polygon/1` | 0.0032955 | 0.00312 | 1.056× | 0.00334, 0.003251 | 0.003085, 0.003155 |
| `tessellation/mesh_to_polygon/10` | 0.021235 | 0.020375 | 1.042× | 0.0211, 0.02137 | 0.02026, 0.02049 |
| `tessellation/mesh_to_polygon/30` | 0.064065 | 0.06402 | 1.001× | 0.06444, 0.06369 | 0.06368, 0.06436 |
| `tessellation/plate/1` | 0.17 | 0.17315 | 0.982× | 0.169, 0.171 | 0.1735, 0.1728 |
| `tessellation/plate/10` | 1.155 | 1.177 | 0.981× | 1.155, 1.155 | 1.171, 1.183 |
| `tessellation/plate/100` | 17.81 | 17.84 | 0.998× | 17.83, 17.79 | 17.85, 17.83 |
| `tessellation/plate/30` | 3.846 | 3.8515 | 0.999× | 3.85, 3.842 | 3.851, 3.852 |

## Four Rayon workers

| Action | Parent ms | Optimized ms | Speedup | Parent round medians | Optimized round medians |
|---|---:|---:|---:|---|---|
| `boolean/circle_remove` | 1.675 | 1.6815 | 0.996× | 1.678, 1.672 | 1.691, 1.672 |
| `boolean/intersect_cylinders/0.001` | 1.3065 | 1.25 | 1.045× | 1.305, 1.308 | 1.243, 1.257 |
| `boolean/intersect_cylinders/0.01` | 0.49535 | 0.52635 | 0.941× | 0.4982, 0.4925 | 0.5318, 0.5209 |
| `boolean/intersect_cylinders/0.1` | 0.2936 | 0.2995 | 0.980× | 0.2829, 0.3043 | 0.2984, 0.3006 |
| `boolean/intersect_nurbs_cylinders/0.001` | 9.2085 | 9.055 | 1.017× | 9.042, 9.375 | 8.956, 9.154 |
| `boolean/intersect_nurbs_cylinders/0.01` | 4.2905 | 4.2765 | 1.003× | 4.207, 4.374 | 4.27, 4.283 |
| `boolean/intersect_nurbs_cylinders/0.1` | 2.5585 | 2.5755 | 0.993× | 2.528, 2.589 | 2.59, 2.561 |
| `boolean/subtract_batch/1` | 0.4535 | 0.017725 | 25.585× | 0.4574, 0.4496 | 0.01802, 0.01743 |
| `boolean/subtract_batch/10` | 4.4505 | 4.4635 | 0.997× | 4.422, 4.479 | 4.458, 4.469 |
| `boolean/subtract_batch/100` | 63.635 | 63.79 | 0.998× | 63.88, 63.39 | 63.72, 63.86 |
| `boolean/subtract_batch/30` | 14.11 | 14.255 | 0.990× | 14.19, 14.03 | 14.11, 14.4 |
| `boolean/subtract_sequential/1` | 0.4777 | 0.0188 | 25.410× | 0.4539, 0.5015 | 0.01935, 0.01825 |
| `boolean/subtract_sequential/10` | 10.365 | 0.4883 | 21.227× | 10.33, 10.4 | 0.4892, 0.4874 |
| `boolean/subtract_sequential/100` | 813.5 | 33.61 | 24.204× | 814.5, 812.5 | 33.62, 33.6 |
| `boolean/subtract_sequential/30` | 62.855 | 3.4195 | 18.381× | 62.92, 62.79 | 3.416, 3.423 |
| `boolean/union_cylinders/0.001` | 1.2655 | 1.257 | 1.007× | 1.245, 1.286 | 1.273, 1.241 |
| `boolean/union_cylinders/0.01` | 0.52755 | 0.5168 | 1.021× | 0.5221, 0.533 | 0.5135, 0.5201 |
| `boolean/union_cylinders/0.1` | 0.31075 | 0.2971 | 1.046× | 0.32, 0.3015 | 0.2966, 0.2976 |
| `local/chamfer/1` | 0.030315 | 0.03074 | 0.986× | 0.0303, 0.03033 | 0.03164, 0.02984 |
| `local/chamfer/12` | 0.090205 | 0.090605 | 0.996× | 0.09087, 0.08954 | 0.09218, 0.08903 |
| `local/draft_box/1` | 0.047435 | 0.047485 | 0.999× | 0.04779, 0.04708 | 0.04676, 0.04821 |
| `local/draft_box/4` | 0.10655 | 0.1172 | 0.909× | 0.1088, 0.1043 | 0.1272, 0.1072 |
| `local/fillet/1` | 0.098405 | 0.09008 | 1.092× | 0.08791, 0.1089 | 0.09147, 0.08869 |
| `local/fillet/12` | 0.8724 | 0.8698 | 1.003× | 0.8748, 0.87 | 0.866, 0.8736 |
| `local/rolling_fillet` | 16.29 | 16.31 | 0.999× | 16.22, 16.36 | 16.19, 16.43 |
| `local/shell_box` | 0.1256 | 0.12255 | 1.025× | 0.1242, 0.127 | 0.1209, 0.1242 |
| `modeling/extrude/128` | 0.04233 | 0.04232 | 1.000× | 0.04222, 0.04244 | 0.04181, 0.04283 |
| `modeling/extrude/32` | 0.010435 | 0.0109 | 0.957× | 0.01026, 0.01061 | 0.01106, 0.01074 |
| `modeling/extrude/4` | 0.0013015 | 0.0013345 | 0.975× | 0.001276, 0.001327 | 0.001327, 0.001342 |
| `modeling/loft/2` | 0.03168 | 0.031375 | 1.010× | 0.03158, 0.03178 | 0.03234, 0.03041 |
| `modeling/loft/32` | 0.5025 | 0.51985 | 0.967× | 0.5053, 0.4997 | 0.5188, 0.5209 |
| `modeling/loft/8` | 0.1043 | 0.1037 | 1.006× | 0.1063, 0.1023 | 0.1058, 0.1016 |
| `modeling/revolve/128` | 0.09998 | 0.09935 | 1.006× | 0.09966, 0.1003 | 0.0985, 0.1002 |
| `modeling/revolve/32` | 0.025215 | 0.026425 | 0.954× | 0.02557, 0.02486 | 0.02673, 0.02612 |
| `modeling/revolve/4` | 0.003744 | 0.003085 | 1.214× | 0.004493, 0.002995 | 0.003075, 0.003095 |
| `modeling/sweep_path/1` | 0.002732 | 0.002709 | 1.008× | 0.002745, 0.002719 | 0.002759, 0.002659 |
| `modeling/sweep_path/32` | 0.062865 | 0.06346 | 0.991× | 0.06284, 0.06289 | 0.06522, 0.0617 |
| `modeling/sweep_path/8` | 0.015955 | 0.01636 | 0.975× | 0.01611, 0.0158 | 0.0172, 0.01552 |
| `modeling/transform/1` | 0.0023565 | 0.0022285 | 1.057× | 0.002414, 0.002299 | 0.002254, 0.002203 |
| `modeling/transform/10` | 0.00957 | 0.0095675 | 1.000× | 0.009493, 0.009647 | 0.009608, 0.009527 |
| `modeling/transform/30` | 0.02629 | 0.025705 | 1.023× | 0.02629, 0.02629 | 0.02515, 0.02626 |
| `projection/hidden_lines/1` | 0.18045 | 0.1902 | 0.949× | 0.1926, 0.1683 | 0.1877, 0.1927 |
| `projection/hidden_lines/10` | 1.103 | 1.101 | 1.002× | 1.099, 1.107 | 1.088, 1.114 |
| `projection/hidden_lines/30` | 3.541 | 3.5865 | 0.987× | 3.539, 3.543 | 3.569, 3.604 |
| `step/convert/1` | 0.02073 | 0.020425 | 1.015× | 0.02092, 0.02054 | 0.0204, 0.02045 |
| `step/convert/10` | 0.0845 | 0.086 | 0.983× | 0.08396, 0.08504 | 0.08651, 0.08549 |
| `step/convert/30` | 0.23425 | 0.2403 | 0.975× | 0.2329, 0.2356 | 0.2367, 0.2439 |
| `step/export/1` | 0.02363 | 0.02367 | 0.998× | 0.02362, 0.02364 | 0.02379, 0.02355 |
| `step/export/10` | 0.09406 | 0.094 | 1.001× | 0.09352, 0.0946 | 0.0933, 0.0947 |
| `step/export/30` | 0.25695 | 0.2553 | 1.006× | 0.2575, 0.2564 | 0.257, 0.2536 |
| `step/export_file/1` | 0.09104 | 0.09193 | 0.990× | 0.0831, 0.09898 | 0.08756, 0.0963 |
| `step/export_file/10` | 0.19545 | 0.21715 | 0.900× | 0.1939, 0.197 | 0.2177, 0.2166 |
| `step/export_file/30` | 0.4436 | 0.4828 | 0.919× | 0.444, 0.4432 | 0.483, 0.4826 |
| `step/import/1` | 0.8474 | 0.8399 | 1.009× | 0.8378, 0.857 | 0.8435, 0.8363 |
| `step/import/10` | 3.2185 | 3.2365 | 0.994× | 3.231, 3.206 | 3.228, 3.245 |
| `step/import/30` | 8.3505 | 8.294 | 1.007× | 8.373, 8.328 | 8.282, 8.306 |
| `step/import_file/1` | 0.85175 | 0.846 | 1.007× | 0.8454, 0.8581 | 0.8494, 0.8426 |
| `step/import_file/10` | 3.24 | 3.228 | 1.004× | 3.229, 3.251 | 3.232, 3.224 |
| `step/import_file/30` | 8.375 | 8.307 | 1.008× | 8.396, 8.354 | 8.317, 8.297 |
| `step/parse/1` | 0.8128 | 0.8072 | 1.007× | 0.8142, 0.8114 | 0.8083, 0.8061 |
| `step/parse/10` | 3.1055 | 3.0975 | 1.003× | 3.098, 3.113 | 3.087, 3.108 |
| `step/parse/30` | 8.095 | 7.996 | 1.012× | 8.113, 8.077 | 7.988, 8.004 |
| `tessellation/cylinder_tolerance/0.001` | 0.225 | 0.2253 | 0.999× | 0.2197, 0.2303 | 0.2327, 0.2179 |
| `tessellation/cylinder_tolerance/0.01` | 0.0617 | 0.074855 | 0.824× | 0.05714, 0.06626 | 0.06134, 0.08837 |
| `tessellation/cylinder_tolerance/0.1` | 0.04401 | 0.048285 | 0.911× | 0.04794, 0.04008 | 0.0465, 0.05007 |
| `tessellation/mesh_to_polygon/1` | 0.003218 | 0.003035 | 1.060× | 0.003205, 0.003231 | 0.003015, 0.003055 |
| `tessellation/mesh_to_polygon/10` | 0.020925 | 0.01994 | 1.049× | 0.02061, 0.02124 | 0.01972, 0.02016 |
| `tessellation/mesh_to_polygon/30` | 0.06382 | 0.06301 | 1.013× | 0.06286, 0.06478 | 0.0619, 0.06412 |
| `tessellation/plate/1` | 0.13395 | 0.1313 | 1.020× | 0.133, 0.1349 | 0.1331, 0.1295 |
| `tessellation/plate/10` | 0.60325 | 0.6007 | 1.004× | 0.599, 0.6075 | 0.5937, 0.6077 |
| `tessellation/plate/100` | 8.3225 | 8.3095 | 1.002× | 8.301, 8.344 | 8.386, 8.233 |
| `tessellation/plate/30` | 1.7605 | 1.8005 | 0.978× | 1.764, 1.757 | 1.774, 1.827 |

## Follow-up measurements

Two four-worker rows exceeded 10% slowdown in this run. Neither slowdown reproduced with 500 samples across five follow-up rounds; the original values below remain unchanged. See the [full report](../REPORT.md#repeated-measurements-of-apparent-slowdowns) and [follow-up records](../rechecks/records.json).

## Raw runs

[Structured results](records.json), [summary](summary.json), [build and binary metadata](metadata.json), and [dependency lockfile](Cargo.lock).

- [r1-before-1t.txt](r1-before-1t.txt): 32.79 seconds, exit 0.
- [r1-after-1t.txt](r1-after-1t.txt): 5.27 seconds, exit 0.
- [r1-before-4t.txt](r1-before-4t.txt): 22.58 seconds, exit 0.
- [r1-after-4t.txt](r1-after-4t.txt): 4.70 seconds, exit 0.
- [r2-after-1t.txt](r2-after-1t.txt): 5.23 seconds, exit 0.
- [r2-before-1t.txt](r2-before-1t.txt): 32.28 seconds, exit 0.
- [r2-after-4t.txt](r2-after-4t.txt): 4.72 seconds, exit 0.
- [r2-before-4t.txt](r2-before-4t.txt): 22.58 seconds, exit 0.

Measurement command after release compilation:

```bash
RAYON_NUM_THREADS=1 taskset -c 0,1,2,3 /path/to/actions-binary \
  --bench --color never --sample-count 20 --sample-size 1 --max-time 60
```

Repeat with four Rayon workers and reverse parent/optimized order in the second round. `--bench` selects measurements when launching a Divan executable directly; `cargo bench` normally supplies it.
