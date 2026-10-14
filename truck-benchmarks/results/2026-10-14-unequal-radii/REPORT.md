# Per-edge radii for separate straight fillets

Parent: `d3bc14fe9b225b37076bf86517c09145a4cd2ba6`. The lower-level straight-edge constructor already accepts a radius per edge, but the public solid API and native replay previously exposed equal radii only. `try_fillet_solid_edges_with_radii` now exposes separate straight edges atomically, including plane/cylinder lips. It validates every radius, identifies invalid assignments by selection index, rejects shared selected vertices for unequal radii, and preserves original-to-final face history. Equal and empty assignments delegate to the existing solid API, retaining its behavior.

The replay matches each physical selection with its captured radius. Same-radius aliases are deduplicated; conflicting assignments and invalid later radii are rejected. Unequal-radius requests are never silently replaced with one common radius. No lower-level geometric construction algorithm, dependency or existing public signature changed. PiezaCAD's current callers in `cad-core/src/kernel/blend.rs` need no migration; the app and its pin were not edited.

## Parent regressions and geometry

`parent-replay.txt` records all three new success regressions failing on the parent as `unhandled-intent`. Existing assertions were retained. The final replay tests additionally check same-radius aliases, conflicting radii and invalid second assignments.

Four public API regressions cover three native input shapes, three placements/scales and both selection orders. Checks include independently derived material, rolling-cylinder radius and oriented normals, exact compressed-output equality across selection order, original-to-final history, all generated faces, input preservation, closed meshes and STEP round trips. Negative cases cover invalid/nonfinite radii, duplicate/unknown edges, oversized rounds, unequal-radius junctions and overlapping parallel rounds. Nonoverlapping unequal rounds sharing a planar support also pass independent material and STEP checks. Equal-radius all-edge cube fillets and empty selection retain the existing results.

Native fixtures are unmodified `draft/angle/A2`, `B5`, `C4` inputs from the previous full capture. The captured blend precedes the native test's later draft operation; those later draft results are not covered here. `tests/fixtures/unequal-radii/model.rs` and `independent-volumes.py` derive expected material from box dimensions, polygon-corner areas and Green's theorem for line/circle corners, without native output as an oracle.

## Validation

- **380 shape-operations tests passed** across 76 targets, zero failed, one existing ignore.
- All **8 replay tests** passed, including the three parent-failing cases.
- Release all-target Clippy for shapeops and benchmarks, single-thread action benchmark smoke, and official release replay build passed.
- Geometry source is unchanged. The earlier PCurve derivative precision failure remains an open baseline issue outside this affected-crate run; its parent proof is in `../2026-10-13-cubic-endpoints/parent-precision-test.txt`.
- All three targeted native exports are valid single solids, with closed Truck meshes, consistent geometry and unchanged inputs. Maximum native edge/vertex tolerance is 1e-7.

| Capture | Dimension-derived volume | Independent native integration of Truck STEP | Absolute error | Sampled native→Truck / Truck→native distance |
| --- | ---: | ---: | ---: | ---: |
| draft/angle/A2 | 5972.1017612416681 | 5972.1017612416645 | 3.637978807091713e-12 | 1.11022e-14 / 3.87784e-15 |
| draft/angle/B5 | 13.39678797816957 | 13.396787978164388 | 5.1816329005305306e-12 | 1.67959e-12 / 1.74405e-12 |
| draft/angle/C4 | 2988.725888734009 | 2988.7258887340031 | 5.9117155615240335e-12 | 3.10862e-14 / 5.32907e-15 |

The independent volume acceptance is one ppm; observed errors are below 6e-12. Both sampled distance directions are below 0.001 (at most 1.745e-12 in these targets). Samples are not certified global bounds. The remaining five unequal-radius captures (`blend/simple/A4`, `P9`, `blend/tolblend_simple/A2`, `feat/featlf/E2`, `E3`) now return `UnsupportedTopology` with unchanged input: their selected edges meet at unsupported unequal-radius junctions. They are attempted failures, not recovered constructions.

`metadata.json` records commands, exit codes, source/executable hashes, independent formulas, native checks and all eight target results. All 48 native input/reference files were byte-checked against the previous archive. The target archive has **77** verified members; SHA-256 `f1b4cda3667f134d1f75be8ff72546b1e51976101adf633e7b7a33678e3292a8`.

The added scope is separate straight edges with radii constant along each edge. Unequal curved contours, general unequal-radius corner filling and distant-face collision detection remain outside this group. No performance claim is made. A clean full capture follows the implementation commit and is recorded separately.
