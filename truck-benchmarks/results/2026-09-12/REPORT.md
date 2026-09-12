# PiezaCad / Truck versus FreeCAD — 2026-09-12

Both repositories were optimized and measured. The suite now contains **71 Truck cases
across 25 action types**, **53 FreeCAD cases**, and **39 complete PiezaCad corpus rebuilds**.
Twenty action types have a FreeCAD counterpart; internal parsing/conversion stages remain
separate. The comparison table has 56 paired rows because the cylinder tolerance sweep
compares several Truck settings with one OCCT operation. API differences are marked.

## Results

All times below are medians, in milliseconds. The primary before/after comparison uses
one Rayon worker and the system allocator on the same Ryzen 9 7950X3D.

| PiezaCad full rebuild | Before | After | Improvement |
|---|---:|---:|---:|
| Fillet then chamfer | 2793.94 | 8.49 | 329× |
| Hole pattern and mirrored boss | 4937.74 | 70.71 | 70× |
| Revolve with groove | 2777.28 | 14.29 | 194× |
| Circular removal | 22.44 | 16.53 | 1.36× |

The new 100-hole pattern rebuilds in **322.64 ms with one worker**, or **280.20 ms with
four**, including sketches, names and display meshes. FreeCAD's matching PartDesign
MultiTransform recomputes in **216.19 ms**, or **280.59 ms including fresh meshes of the
pad, pocket and final pattern at 0.05 mm**. Treat the complete-workflow result as a tie,
not a 0.1% win. The kernels partition the cylinder walls differently (406 versus 106 faces).
The app's under-two-second 100-hole rebuild target is met.

| CPU operation | Truck | FreeCAD | FreeCAD / Truck |
|---|---:|---:|---:|
| Batch subtraction, 100 holes | 63.09 | 164.16 | 2.60× |
| Circular pocket boolean | 2.18 | 5.82 | 2.67× |
| Hidden lines, 30 holes | 5.78 | 6.41 | 1.11× |
| Sequential subtraction, 100 holes | 1158 | 656.78 | 0.57× |

Truck has lower medians in 53 of the 56 paired table rows, including rows with explicitly
marked differences in algorithms or API scope. This is **not a claim of universal CAD
parity or equivalent numerical contracts**. Extrude, revolve, sweep, transforms, planar
blends, shell, draft and batch booleans are strong results. FreeCAD still clearly wins the
100-hole sequential workload. The 30-hole sequential case and the 0.001 mm cylinder
intersection are close, slightly favoring FreeCAD in the primary run. The two final Truck
runs agree within 15% on every case; the substantial speedups repeat.

See [all operation and app tables](tables.md), [raw Truck timings](truck-final.txt),
[the repeat run](truck-final-repeat.txt), [FreeCAD timings](freecad-final.txt), and
[four-worker app timings](app-four-workers.jsonl). Values in the full tables are authoritative;
summary values are rounded. The original baselines are also included in this directory.

## What changed

- Truck's nearest search on revolved surfaces uses its meridian solver for similarity
  transforms, retaining the world-space grid for other transforms and range hints. Axial-line
  cylinders have an exact parameter inverse and reject off-surface points immediately.
- Hidden-line classification prepares triangle inverses once per view and rejects rays
  outside each triangle's projected bounds. The original intersection test, grazing rules,
  barycentric slack and nudge behavior remain. The 30-hole case fell from 11.61 to 5.78 ms.
- One-worker tessellation uses the existing sequential implementation, avoiding scheduling
  and parallel-map overhead. Serial and parallel meshes are checked for exact equality.
- PiezaCad calls the native kernel directly instead of rebuilding topology through its
  former evaluation wrappers. The old surface adapter remains only as a test oracle.
- Planar straight-edge blends use the kernel's existing multi-edge operation, including
  disjoint selections. Unsupported cases retain the general fallback. Face history and
  downstream names remain checked by the unchanged corpus expectations.
- Naming meshes shared topology together and reuses source surfaces across candidate tests.
- The app uses the sibling Truck checkout. Its launcher defaults to four Rayon workers;
  an existing `RAYON_NUM_THREADS` setting takes precedence. No dependency was added.

Production tolerances were not changed. The app batch/sequential equivalence test now uses
0.0001 mm tessellation instead of 0.001 mm: the coarser measurement obscured the existing
1e-9 relative volume criterion on 100 holes. The error bound, topology and naming assertions
are unchanged. This increases validation precision rather than relaxing the test.

## Validation and limits

**595 tests/doctests passed**: 342 across the affected kernel crates and 253 in cad-core.
All 39 corpus models retain their expected diagnostics, topology counts and volumes.
The new planar-blend regression [fails with the original adapter](planar-blend-parent-test.txt).
The whole app workspace compiles against the local kernel, and the relevant Clippy checks
pass. Existing shared-fixture cfg warnings and nom/quick-xml future-compatibility notices
remain. Commands and detailed outcomes are in [validation.txt](validation.txt).

The additional app 100-hole STEP export imports into FreeCAD as one valid solid, volume
2571.68144735 mm³ versus the analytic 2571.68146928 mm³. However, FreeCAD's stricter
`check(True)` reports `BOPAlgo_InvalidCurveOnSurface` on several edges/faces. That strict
export-validation issue remains unresolved and is **not counted as a clean export result**.
This does not affect the standard shared plate STEP fixtures' basic validity/volume checks.

The comparison measures warmed CPU work, not launch time, mouse-to-display latency, GPU
rendering, or all possible models. Fixtures are prepared and checked outside timed closures.
Output destruction is excluded where practical; intermediate work remains included. App
rebuild timing excludes document loading and checks results outside timing. Some corpus
cases intentionally exercise feature failures and must preserve those failures.

FreeCAD 1.1.3 / OCCT 7.8.1 runs through its installed Flatpak console, with startup excluded.
The primary Truck run uses one Rayon worker; FreeCAD retains its native internal threading.
The four-worker Truck results are supplemental and clearly labeled. No CPU affinity was set;
the desktop remained running, and engines/builds were serialized during measurement.

Tessellation uses the same absolute linear deflection; FreeCAD uses
`MeshPart.meshFromShape` with `Relative=False`, a loose π angular cap, and a geometry copy
without cached triangulation. Mesh algorithms and output structures differ.
[FreeCAD meshing API](https://github.com/FreeCAD/FreeCAD/blob/1.1.3/src/Mod/MeshPart/App/AppMeshPartPy.cpp).

Other qualifications are explicit in the tables: default lofts interpolate differently;
FreeCAD's NURBS conversion converts caps too; STEP import includes native B-rep healing in
FreeCAD but returns compressed topology in Truck; FreeCAD draft includes recompute; HLR
uses exact OCCT visibility versus Truck's sampled occlusion. Cylinder booleans compare
Truck chord tolerances with OCCT's own geometric tolerance policy. These rows are useful
performance observations, not proofs that the APIs do identical work.

## Reproduce

Follow [BENCHMARKS.md](../../../BENCHMARKS.md#comparing-with-freecad-and-the-app).
The scripts and fixtures are generated locally; FreeCAD must be installed separately.
`report_comparison.py` rejects missing results, duplicate FreeCAD rows and failed/incomplete
FreeCAD runs, and retains losses in the output table. [metadata.json](metadata.json) records
versions, revisions and settings. Baseline Truck is 2ed10470; baseline PiezaCad is 06f53ae
and pins b115af07. The app therefore also benefits from the already-present 2ed10470 face
division improvement when switching to the sibling kernel. The final source changes are
uncommitted; no commits, pushes or PRs were made.
