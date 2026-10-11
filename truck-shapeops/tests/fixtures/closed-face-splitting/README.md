# Tangent stacked cylinders

`tangent-stacked-cylinders.step` is the captured input of OCCT V7_9_3
`tests/blend/simple/Z1`, before its radius-1 fillet. The lower cylinder has radius
10 and height 20. The upper cylinder has radius 5, height 20 and origin (5,0,20).
Their union has volume 2500*pi. The shared cap contains tangent outer/inner loops;
STEP vertices #22 and #113 have identical coordinates but distinct identifiers.

The fixture is identical to `blend/simple/Z1/capture/001.input.step` in
`audits/fillet-chamfer/capture/2026-10-11-transformed-division/artifacts.zip`.
Native task, output, physical selection, verdict and SHA-256 hashes are retained
with that run's `results.json`.
