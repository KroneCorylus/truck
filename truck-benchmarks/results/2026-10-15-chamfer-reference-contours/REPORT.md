# Propagate chamfer reference sides along complete contours

The seed reference plane was previously matched independently at every contour
edge. A turning line/arc/line contour therefore failed when its side support
changed to a cylinder and another vertical plane. Replay now matches the seed
reference and propagates its side using oriented face traversal along the wire.
It preserves per-face setback order and permits the existing unequal wire API
when all ordered pairs agree within floating-point roundoff (1e-12 relative).

Two planar supports retain the existing oriented-angle conversion. Perpendicular
plane/cylinder rims use their exact right-angle section. Other curved support
pairs and curved seed reference faces remain explicitly unsupported. Physical
matching tolerances, kernel APIs, and STEP fitting budgets are unchanged.

Two native regressions fail on parent 2191a34387fe11b2a1385d4516bb55592cd85ecf. Both pass in all
four combinations of reversed contour/sample order and reference normal sign.
Assertions check exact intended top/side setbacks, all three selected edges,
immutable input, consistent geometry, closed mesh and the shared solid/material
and STEP harness. Native volumes are reference checks, not analytic oracles.
All 27 replay tests, all-target release Clippy, action smoke and final replay
build pass. No kernel source changed, so the preceding 504-test kernel result
is not represented as a repeated validation. No existing assertion was loosened.

Fresh OCCT 7.9.3 proof captures both A4 sequences (16 records, retained in full).
Their first complete three-edge operations now construct, export valid single
solids, agree with native volume within 5e-12, and have maximum sampled boundary
difference 2.229100205894138e-8. Later sequence records remain separate outcomes;
inputs are independently supplied by native OCCT, not chained Truck outputs.
No global-distance, full-sequence, full-parity or performance claim is made.

Proof archive: 212 byte-verified members, SHA256 f755d5266931d4947d59ea799311b8198b968b20212f75f43de0731060d0e0ee.
Metadata records source and executable hashes. A clean full-corpus comparison
follows the implementation commit.
