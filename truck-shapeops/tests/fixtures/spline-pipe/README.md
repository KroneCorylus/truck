# Closed spline pipe input

Generated from OCCT V7_9_3 `tests/blend/simple/J9` by the fillet/chamfer capture
harness before its blend operation. The source sweeps a radius-50 disk along
a cubic Bezier with control points (0,0,0), (100,0,0), (100,100,0), (200,100,0).
OCCT exports a padded periodic rational spline representation for the circular
direction. Captured input provenance is retained in
`audits/fillet-chamfer/capture/2026-10-12-unclamped-domains/artifacts.zip`.

The independent ideal sweep volume is pi * 50² * the cubic's arc length:
1815077.75712917, using converged Simpson quadrature. The captured approximation
has native volume 1815077.51635755; the difference is below the regression's
physical tessellation allowance. The regression checks the imported solid
and STEP round trip at scales 0.01 and 0.025, including a rigid transform.
This fixture does not assert successful filleting of the pipe rim.
