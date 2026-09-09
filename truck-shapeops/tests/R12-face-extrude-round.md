# R12: round face extension

The operands are the exact compressed modeling solids supplied by the consuming
app's R12 report, reproduced at `91e6a0bc5cfb82d51689578fd7b37baf7bc4eb56`.
Run the regression with:

```sh
cargo test -p truck-shapeops --test R12-face-extrude-round
```

The coincident-face crossing pass was treating adjacent segments of the same
shared circular boundary as crossings at their tessellation vertices. Solving
for an intersection of the identical exact curves then failed. Shared edges with
matching complete polylines now remain in the existing overlap-merging path.
Distinct boundaries still go through crossing detection.

The tests check the supplied tube, a disk extrusion, and a rectangular face with
two circular holes in both Z directions and operand orders. They verify closed
topology and meshes, signed volume, one connected solid, unchanged operands, and
exact input curves. Further tests subtract the extension, intersect it, rejoin
the pieces, and export/import STEP.

## Separate limitation found during validation

A horizontal slab cut through a cylindrical wall still reports
`InvalidOutputTopology` at `validate_output`. This also fails for a fresh 10 mm
tube built directly from the original cap, without the R12 union. It is outside
the shared-boundary correction. To reproduce using the helpers in the test:

```rust
let [base, _] = operands();
let cap = builder::translated(&top_face(&base), -5.0 * Vector3::unit_z());
let fresh = builder::tsweep(&cap, 10.0 * Vector3::unit_z());
let slab = common::modeling::cuboid(
    Point3::new(-6.0, -6.0, 7.0),
    Point3::new(6.0, 6.0, 11.0),
);
let result = truck_shapeops::try_subtract(&fresh, &slab, 0.01);
```
