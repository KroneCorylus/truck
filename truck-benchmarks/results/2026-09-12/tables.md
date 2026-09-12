| Operation | Truck before ms | Truck after ms | FreeCAD ms | FC / Truck | Comparison notes |
|---|---:|---:|---:|---:|---|
| boolean/circle_remove | — | 2.179 | 5.815 | 2.67× |  |
| boolean/intersect_cylinders/0.001 | 2.26 | 1.901 | 1.866 | 0.98× | Truck chord tolerance; FreeCAD uses OCCT geometric tolerances |
| boolean/intersect_cylinders/0.01 | 0.6958 | 0.5615 | 1.866 | 3.32× | Truck chord tolerance; FreeCAD uses OCCT geometric tolerances |
| boolean/intersect_cylinders/0.1 | 0.3455 | 0.2612 | 1.866 | 7.14× | Truck chord tolerance; FreeCAD uses OCCT geometric tolerances |
| boolean/intersect_nurbs_cylinders/0.001 | 11.33 | 9.601 | 63.33 | 6.60× | Truck chord tolerance; FreeCAD uses OCCT geometric tolerances; NURBS representation differs (FreeCAD converts caps too) |
| boolean/intersect_nurbs_cylinders/0.01 | 4.125 | 3.686 | 63.33 | 17.18× | Truck chord tolerance; FreeCAD uses OCCT geometric tolerances; NURBS representation differs (FreeCAD converts caps too) |
| boolean/intersect_nurbs_cylinders/0.1 | 1.924 | 1.739 | 63.33 | 36.42× | Truck chord tolerance; FreeCAD uses OCCT geometric tolerances; NURBS representation differs (FreeCAD converts caps too) |
| boolean/subtract_batch/1 | 0.6331 | 0.4703 | 1.482 | 3.15× |  |
| boolean/subtract_batch/10 | 5.579 | 4.621 | 15.54 | 3.36× |  |
| boolean/subtract_batch/30 | 16.14 | 14.92 | 49.66 | 3.33× |  |
| boolean/subtract_batch/100 | 67.93 | 63.09 | 164.2 | 2.60× |  |
| boolean/subtract_sequential/1 | 0.479 | 0.4853 | 1.38 | 2.84× |  |
| boolean/subtract_sequential/10 | 14.75 | 14.09 | 19.61 | 1.39× |  |
| boolean/subtract_sequential/30 | 95.74 | 91.06 | 89.73 | 0.99× |  |
| boolean/subtract_sequential/100 | 1198 | 1158 | 656.8 | 0.57× |  |
| boolean/union_cylinders/0.001 | 1.92 | 1.843 | 1.996 | 1.08× | Truck chord tolerance; FreeCAD uses OCCT geometric tolerances |
| boolean/union_cylinders/0.01 | 0.5918 | 0.5499 | 1.996 | 3.63× | Truck chord tolerance; FreeCAD uses OCCT geometric tolerances |
| boolean/union_cylinders/0.1 | 0.3064 | 0.2608 | 1.996 | 7.65× | Truck chord tolerance; FreeCAD uses OCCT geometric tolerances |
| local/chamfer/1 | 0.02589 | 0.02548 | 0.5061 | 19.86× |  |
| local/chamfer/12 | 0.06491 | 0.07435 | 3.237 | 43.54× |  |
| local/draft_box/1 | 0.01697 | 0.01605 | 0.5728 | 35.69× | FreeCAD includes document recompute |
| local/draft_box/4 | 0.02165 | 0.02059 | 0.8942 | 43.43× | FreeCAD includes document recompute |
| local/fillet/1 | 0.02566 | 0.02504 | 0.5256 | 20.99× |  |
| local/fillet/12 | 0.05539 | 0.05259 | 3.508 | 66.70× |  |
| local/shell_box | 0.03611 | 0.03513 | 2.081 | 59.22× |  |
| modeling/extrude/4 | 0.001542 | 0.001407 | 0.02717 | 19.31× |  |
| modeling/extrude/32 | 0.01152 | 0.01096 | 0.1536 | 14.01× |  |
| modeling/extrude/128 | 0.04424 | 0.04187 | 0.5939 | 14.18× |  |
| modeling/loft/2 | 0.0279 | 0.02652 | 0.6988 | 26.35× | different default spline interpolation; equivalent sections, not identical surfaces |
| modeling/loft/8 | 0.08977 | 0.08816 | 2.915 | 33.07× | different default spline interpolation; equivalent sections, not identical surfaces |
| modeling/loft/32 | 0.4203 | 0.4287 | 4.923 | 11.48× | different default spline interpolation; equivalent sections, not identical surfaces |
| modeling/revolve/4 | 0.00306 | 0.002945 | 0.1035 | 35.16× |  |
| modeling/revolve/32 | 0.02651 | 0.02695 | 0.7329 | 27.20× |  |
| modeling/revolve/128 | 0.1003 | 0.1057 | 2.787 | 26.37× |  |
| modeling/sweep_path/1 | 0.002624 | 0.002584 | 0.2351 | 90.96× |  |
| modeling/sweep_path/8 | 0.01515 | 0.01574 | 0.4346 | 27.61× |  |
| modeling/sweep_path/32 | 0.05936 | 0.06091 | 1.11 | 18.23× |  |
| modeling/transform/1 | 0.00269 | 0.00277 | 0.08173 | 29.50× |  |
| modeling/transform/10 | 0.01019 | 0.01079 | 0.224 | 20.76× |  |
| modeling/transform/30 | 0.02698 | 0.0285 | 0.5331 | 18.71× |  |
| projection/hidden_lines/1 | 0.3109 | 0.2434 | 0.3885 | 1.60× | Truck sampled occlusion; FreeCAD exact HLR |
| projection/hidden_lines/10 | 2.708 | 1.809 | 1.923 | 1.06× | Truck sampled occlusion; FreeCAD exact HLR |
| projection/hidden_lines/30 | 11.61 | 5.782 | 6.406 | 1.11× | Truck sampled occlusion; FreeCAD exact HLR |
| step/convert/1 | 0.01918 | 0.01942 | — | — | internal stage; no equivalent FreeCAD public API |
| step/convert/10 | 0.0789 | 0.08105 | — | — | internal stage; no equivalent FreeCAD public API |
| step/convert/30 | 0.2105 | 0.2184 | — | — | internal stage; no equivalent FreeCAD public API |
| step/export/1 | 0.02117 | 0.02225 | — | — | internal stage; no equivalent FreeCAD public API |
| step/export/10 | 0.08607 | 0.08983 | — | — | internal stage; no equivalent FreeCAD public API |
| step/export/30 | 0.2349 | 0.2379 | — | — | internal stage; no equivalent FreeCAD public API |
| step/export_file/1 | — | 0.07655 | 0.8543 | 11.16× |  |
| step/export_file/10 | — | 0.2001 | 3.012 | 15.05× |  |
| step/export_file/30 | — | 0.4734 | 11.41 | 24.11× |  |
| step/import/1 | 0.7517 | 0.8081 | — | — | internal stage; no equivalent FreeCAD public API |
| step/import/10 | 2.877 | 3.046 | — | — | internal stage; no equivalent FreeCAD public API |
| step/import/30 | 7.837 | 7.885 | — | — | internal stage; no equivalent FreeCAD public API |
| step/import_file/1 | — | 0.7799 | 2.267 | 2.91× | same STEP bytes; Truck returns compressed topology, FreeCAD heals native B-rep |
| step/import_file/10 | — | 2.955 | 8.555 | 2.90× | same STEP bytes; Truck returns compressed topology, FreeCAD heals native B-rep |
| step/import_file/30 | — | 7.904 | 23.22 | 2.94× | same STEP bytes; Truck returns compressed topology, FreeCAD heals native B-rep |
| step/parse/1 | 0.7528 | 0.7566 | — | — | internal stage; no equivalent FreeCAD public API |
| step/parse/10 | 2.853 | 2.822 | — | — | internal stage; no equivalent FreeCAD public API |
| step/parse/30 | 7.302 | 7.352 | — | — | internal stage; no equivalent FreeCAD public API |
| tessellation/cylinder_tolerance/0.001 | 0.419 | 0.3999 | 4.918 | 12.30× | absolute deflection; independent mesh algorithms |
| tessellation/cylinder_tolerance/0.01 | 0.128 | 0.1018 | 1.64 | 16.11× | absolute deflection; independent mesh algorithms |
| tessellation/cylinder_tolerance/0.1 | 0.06064 | 0.0412 | 1.072 | 26.03× | absolute deflection; independent mesh algorithms |
| tessellation/mesh_to_polygon/1 | 0.00273 | 0.002985 | — | — | internal stage; no equivalent FreeCAD public API |
| tessellation/mesh_to_polygon/10 | 0.01758 | 0.01842 | — | — | internal stage; no equivalent FreeCAD public API |
| tessellation/mesh_to_polygon/30 | 0.05945 | 0.05673 | — | — | internal stage; no equivalent FreeCAD public API |
| tessellation/plate/1 | 0.192 | 0.1561 | 2.087 | 13.37× | absolute 0.01 mm deflection; independent mesh algorithms |
| tessellation/plate/10 | 1.091 | 1.032 | 8.283 | 8.03× | absolute 0.01 mm deflection; independent mesh algorithms |
| tessellation/plate/30 | 3.733 | 3.49 | 24.99 | 7.16× | absolute 0.01 mm deflection; independent mesh algorithms |
| tessellation/plate/100 | 17.13 | 16.62 | 116.6 | 7.02× | absolute 0.01 mm deflection; independent mesh algorithms |

A ratio above 1 favors Truck. Values below 1 are losses; small differences need repeated runs.

| PiezaCad complete rebuild | Before ms | After ms | Speedup |
|---|---:|---:|---:|
| boolean-boss | 1.434 | 0.7663 | 1.87× |
| boolean-cavity | 5.739 | 2.511 | 2.29× |
| boolean-consumed | 0.7966 | 0.292 | 2.73× |
| boolean-failure-atomic | 3.644 | 1.413 | 2.58× |
| boolean-intersect | 1.253 | 0.5554 | 2.26× |
| boolean-pocket | 2.964 | 1.31 | 2.26× |
| boolean-split-cut | 1.419 | 0.6996 | 2.03× |
| boolean-split-rejoin | 4.332 | 2.24 | 1.93× |
| boss-on-face | 0.409 | 0.1882 | 2.17× |
| circle-remove | 22.44 | 16.53 | 1.36× |
| circle-remove-compound | 154.6 | 116.3 | 1.33× |
| constrained-frame | 2.88 | 2.418 | 1.19× |
| corner-chamfer-2 | 0.5884 | 0.2596 | 2.27× |
| corner-chamfer-3 | 0.7946 | 0.3041 | 2.61× |
| corner-fillet-2 | 0.8898 | 0.5605 | 1.59× |
| counterbored-plate | 973.2 | 26.08 | 37.31× |
| cylinder | 3.72 | 3.377 | 1.10× |
| datums | 0.8713 | 0.3886 | 2.24× |
| dimensioned-plate | 2.907 | 2.46 | 1.18× |
| disjoint-regions | 0.409 | 0.1853 | 2.21× |
| extrude-second-direction | 0.2028 | 0.09642 | 2.10× |
| extrude-symmetric | 0.2025 | 0.09443 | 2.14× |
| extrude-thin-arc | 2.569 | 1.945 | 1.32× |
| extrude-through-all | 1.497 | 0.7997 | 1.87× |
| extrude-up-to-face | 3.034 | 1.322 | 2.29× |
| face-extrude | 3.463 | 1.61 | 2.15× |
| face-extrude-round | 16.97 | 14.37 | 1.18× |
| face-extrude-round-add-failure | 491.8 | 372 | 1.32× |
| fillet-chamfer | 2794 | 8.487 | 329.22× |
| import-bracket | 1.273 | 1.072 | 1.19× |
| naming-coincident-domains | 5.293 | 2.782 | 1.90× |
| pattern-mirror | 4938 | 70.71 | 69.83× |
| plate | 0.2108 | 0.09944 | 2.12× |
| plate-100-pattern | — | 322.6 | — |
| plate-with-hole | 2.941 | 2.463 | 1.19× |
| projected-edge | 8.513 | 7.818 | 1.09× |
| revolve-groove | 2777 | 14.29 | 194.33× |
| rounded-slot | 2.705 | 2.365 | 1.14× |
| shared-wall | 0.199 | 0.09582 | 2.08× |
