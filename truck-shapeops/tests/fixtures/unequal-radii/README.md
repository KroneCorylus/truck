# Unequal-radius straight-edge fixtures

Unmodified native inputs from `audits/fillet-chamfer/capture/2026-10-14-separate-contours/artifacts.zip`:

- `A2.step`: `draft/angle/A2/capture/001.input.step`, SHA-256 `3167c4c3e1c86fc9df9f8532ab3538ea79b542be7361e9f97d0aed28604de2d0`.
- `B5.step`: `draft/angle/B5/capture/001.input.step`, SHA-256 `d2a7c712edafb7e66970f0e3933afcfe5ce37d13dade0f21fcf5d19229b73fae`.
- `C4.step`: `draft/angle/C4/capture/001.input.step`, SHA-256 `46957414429098d57904d03e9b78f360309a62cbe3c71bda76104cf586174dd5`.

`model.rs` derives material and rolling-cylinder centers from dimensions, independently of native results.

- A2: box 10 × 20 × 30, radii 2 and 3 on opposite length-10 edges. Remove `10 * (2² + 3²) * (1 − π/4)` from 6000.
- B5: height-5 extrusion of (0,0), (1,0), (5,1), (0,1). Radii 0.2 at (5,1) and 0.5 at (0,0). A convex polygon corner of interior angle θ loses area `r² * (cot(θ/2) − (π−θ)/2)`; the acute angle is atan(1/4).
- C4: height-20 semiannulus with radii 2 and 10. Fillet radii 2 and 1 lie at the outer positive-x and inner negative-x rulings. Their circle centers are (√60,2) and (−√8,1). Green’s theorem replaces the original circular boundary segments by their tangent circular fillet arcs.

These captures precede subsequent draft operations in the native tests. The group validates the captured fillets, not the later drafting results.
