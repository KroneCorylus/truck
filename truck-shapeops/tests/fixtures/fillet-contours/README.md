# Final native fillet contour fixtures

OCCT 7.9.3 controls captured by the schema-3 plugin in a fresh DRAW process.
The box is 10 × 20 × 30. Its same vertical edge receives radius 2, then 3;
the independent expected volume is 6000 − (1 − π/4) × 3² × 30.

A4 uses the public OCCT geometry documented by the chamfer contour fixture.
Single radius .03 propagates over three tangent edges. Repeating the same seed
with .04 replaces the contour radius. Assigning .04 to a different seed creates
a nonconstant contour. Zero followed by .03 on the same seed is effective .03.
Tests preserve native JSON bytes; the final contour is queried after all Adds.
Native reference volumes .5634899567596924 and .5632421564730637 are comparison
oracles, not independently derived material formulas.

Source records: target/native-fillet-contour-controls/capture/001,002,003,005,011.
The implementation proof archive preserves these controls and a fresh final run.
A4 BREP loading normalizes some direction components at roundoff; compare the
first in-memory export with subsequent exports, not with the pre-read file.
Every input export and the final input export are byte-identical. Truck replay
also checks immutable input. Constructed solids use the shared solid/material
harness. Fitted fillet STEP follows the documented robust triangulation contract,
with unchanged 5e-5 export and .001 mesh tolerances.

SHA-256:

- `A4-replaced.json`: `66437ca60b46b0a4a6f5d9d689b5b79c0edcc4bac92b896cad813754ff3f5f46`
- `A4-single.json`: `8df1b09c5852cb36dc3f9a0ee3e3ce63fe5156a4dfd09079a2823a8c62d7028b`
- `A4-variable.json`: `787d620ff3d05e7619a1fdf49724dc65204a2d3b312de91ad218506c5667e540`
- `A4-zero-replaced.json`: `caa186344a7b357ab42e793e1b614b7def7254ce7a350752bb2353a1272c1ff5`
- `A4.step`: `1be2f36698c32d5dc1e73fd4be78d8013956058fa5493bc13d4cd08823ea8736`
- `box-replaced.json`: `c3204f0bba7a9e28220f40e52f862f0a4cfd268c7e5161414ed7ddcd1824c7a9`
- `box.step`: `e3fab4ecfbbc87dc8390377676027c0a950e1cf30095c94458625b5f4594253c`
