# Clipped chamfer contacts: clean capture

Implementation bb56dfdd30861c7d6f6a65397e89c9e03e322075.

Chamfers now retain clipped contact curves and recover their endpoint parameters on the original ruled surface. End boundaries follow the intersection with the terminal face when a ruling does not lie on that face. Parabolic-cap controls cover unequal setbacks, reverse selections and rigid transforms, with independently derived material and contact distances.

446 release shapeops checks, 115 stepio checks, 62 replay tests, 13 capture tests, all-target Clippy and single-thread actions smoke pass. One existing shapeops ignore and the previously reproduced geometry precision failure remain explicit. Two new tests fail on the parent and pass twelve operations with 3,096 contact samples, topology, geometry, closed meshes, immutable inputs and STEP. Three fresh native controls pass validity and bidirectional comparisons: maximum sampled difference 0.00000532213 at the unchanged 0.001 threshold. Exported volumes agree with the independent calculation within 1.5e-10.

Clean capture: all 396 native cases pass across 648 operations. 449 constructions (+4) and 418 limited comparison passes (+3). All prior successful checks survive; 442 complete prior records are exactly unchanged. 3 prior numerical changes and 0 unchanged-status diagnostic changes. Native references and selection scopes are unchanged. The three A3 records differ only in mesh and exported-solid volume values, by at most 1.53e-14; sampled distances and all checks are unchanged. The three newly constructed C3 cases pass comparisons (maximum distance 2.101e-8). The newly constructed C1 case passes structural checks but still differs from the native shape by up to 0.008422, above the unchanged 0.001 threshold.

Statuses: {"input-import-failed": 96, "operation-succeeded": 449, "operation-failed": 89, "unhandled-intent": 5, "input-inconsistent": 9}.

Status changes: [{"case": "chamfer/dist_angle_sequence/C3", "capture": "001", "changed_scope": false, "before": "operation-failed", "after": "operation-succeeded"}, {"case": "chamfer/dist_dist_sequence/C3", "capture": "001", "changed_scope": false, "before": "operation-failed", "after": "operation-succeeded"}, {"case": "chamfer/equal_dist_sequence/C1", "capture": "001", "changed_scope": false, "before": "operation-failed", "after": "operation-succeeded"}, {"case": "chamfer/equal_dist_sequence/C3", "capture": "001", "changed_scope": false, "before": "operation-failed", "after": "operation-succeeded"}].

Archive SHA256: de4e5ebd56d9ff9c67455d9ce5762a480112f192b573d28e1d09745e588c471b.

Generic chamfer_along_wire/try_chamfer_along_wire require IntersectionCurve<C,S,S>:ToSameGeometry<C>;modeling wrapper signatures and the sole consumer call are unchanged. No consumer changes, push, PR, parity or speed claim.
