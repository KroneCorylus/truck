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

## Rounded offset cylinders

`rounded-offset-cylinders.step` is the unchanged input to the second operation
of OCCT V7_9_3 `tests/blend/complex/G9`, retained in
`audits/fillet-chamfer/capture/2026-10-12-periodic-tori/artifacts.zip`.
A radius-10, height-20 cylinder has its top rim rounded with radius 3, then
is united with a radius-5, height-20 cylinder based at (4.9,0,10).

The original preprocessor chose the endpoints of its existing toroidal seam
as a new divisor, then panicked on the resulting three-way junction. The
regression checks the raw imported topology, closed tessellation and independent
volume for ordinary/robust preprocessing with and without rigid motion. It also
checks exact modeling conversion of the generated diagonal surface curves,
geometric consistency, closed tessellation and volume, and STEP round trips.
The requested radius-4.9 fillet remains unsupported: the regression requires a
diagnostic without a panic or input mutation, including reversed selection.
This regression does not assert fillet parity.

The large cylinder's horizontal radius is 10 below z=17 and
`7+sqrt(9-(z-17)^2)` above it. Its exact rounded volume is
`1865*pi+31.5*pi^2`. The upper cylinder adds `250*pi` above z=20.
Between z=17+sqrt(9-2.9^2) and 20, integrate the radius-5 disk area
outside the larger disk using the standard two-circle intersection formula.
The test evaluates that independent one-dimensional integral with Simpson's
rule in the angle parameter z=17+3*sin(t), using 4096 intervals.
Total volume is approximately 6965.56812665.
