# Stored native tolerances are separate evidence

Capture metadata now records maximum stored face, edge and vertex tolerances for
input/native result shapes. STEP comparison records them separately for native
BREP and imported Truck STEP. A shared native helper validates finite nonnegative
values and is included in source fingerprints. No geometry, validity flag,
comparison threshold or measured-distance algorithm changes.

A native regression fails on parent 24e5c23b13bc5d78adbc797bdfc3f7e2e983fbda: the fields are absent.
Its dimension-defined 10×20×30 box has deliberately assigned face=.05, edge=.1,
vertex=.2 tolerances. Capture reports each maximum exactly. Both shapes remain
valid single solids, volume6000, with zero sampled difference; imported STEP has
1e-7 stored tolerances. This proves a stored budget is not a measured error and
must not be converted into an automatic shape-failure verdict.

A byte-verified inventory of all647 available native result BREPs in the previous
full corpus finds28 with a stored maximum above the .001 comparison threshold.
The one rejected native operation has no result BREP and is excluded explicitly.
Every inventoried BREP is checked against its committed archive member and SHA.

H4's input maximum is about1e-6, but its native result reaches22.9741069520462.
Truck STEP maximum is1e-5. The native BRepCheck validity flag therefore cannot
establish accuracy at .001. The separate 270-degree cylinder control has normal
1e-7 budgets and the expected additive concave-round material. This diagnoses a
reference precision concern; it does not establish H4's exact error, declare its
whole geometry invalid, or upgrade Truck to parity. H4 remains a sampled mismatch.
The other discrepant reference above threshold is tolblend_simple/A1 at
.00235108915834048. G1's native budget is4.39707311635729e-5; its large sampled
difference remains a separate unresolved geometry investigation.

Fresh native G1,H4,tolblend_simple/A1 captures preserve all prior replay results,
native reference properties and complete comparisons EXACTLY after excluding
only the new tolerance fields. Their input tolerances are also captured.
13 capture+22 audit tests pass; C++ builds clean with Wall/Wextra/Wpedantic,
all-target release Clippy, action smoke and final replay build pass. Rust replay
and kernel source are unchanged from33 replay and504 affected-kernel passing
checks; those suites were not redundantly rerun or claimed as new results.

Proof archive:68 byte-verified members, SHA256 a71e2ae15d464bf5ac769984618f5d6243db5d0cc2eb520016a1276ebfe6dc0f.
The full647-reference inventory is separately stored with its baseline archive
hash. A fresh clean full capture follows the implementation commit. This evidence
adds interpretation context and leaves existing finite sampled verdicts intact.
No performance, global-distance or full-parity claim; no consumer/pin changes.
